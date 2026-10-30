//! Fixed QEMU CRB bootstrap experiment, not a product TPM grant or key policy.
use super::{print_number, serial, Frames, Paging, State, FRAME_COUNT};
use core::{
    arch::asm,
    ptr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
use nieuw_os_tpm_crb::{
    codec::{self, NvIndex, Password, Request, Response},
    transport::{self, Effect, Failure, Fault, Io, IoError, Poll, Register, Ticket, Transport},
};

const WINDOW: u64 = 0x3f180000;
const LIMIT: u64 = 50;
const NV_INDEX: u32 = 0x0180_4e56;
const NV_AUTH: &[u8] = b"PUBLIC-TPM-NV-FIXTURE-ONLY";
const NV_PROVISION: bool = cfg!(feature = "test-tpm-nv-provision");
const NV_READ: bool = cfg!(feature = "test-tpm-nv-read");
const NV_AUTH_TEST: bool = cfg!(feature = "test-tpm-nv-auth");
const NV_MISSING: bool = cfg!(feature = "test-tpm-nv-missing");
const NV_ENABLED: bool = NV_PROVISION || NV_READ || NV_AUTH_TEST || NV_MISSING;
const _: () = assert!(
    (NV_PROVISION as u8 + NV_READ as u8 + NV_AUTH_TEST as u8 + NV_MISSING as u8) <= 1
        && !(NV_ENABLED && cfg!(feature = "test-tpm-crb-absent"))
);
static CLAIMED: AtomicBool = AtomicBool::new(false);
static QUARANTINED: AtomicBool = AtomicBool::new(false);
static WRITES: AtomicU64 = AtomicU64::new(0);
static mut RUNTIME: Option<Runtime> = None;

struct NativeIo;

fn barrier() {
    // SAFETY: x86-64/QEMU supports MFENCE. No stack/register operands; the
    // implicit compiler memory clobber and CPU fence order volatile UC accesses.
    unsafe { asm!("mfence", options(nostack, preserves_flags)) }
}

impl NativeIo {
    fn buffer(offset: usize, len: usize) -> Result<u64, IoError> {
        if QUARANTINED.load(Ordering::Acquire)
            || len > 128
            || offset
                .checked_add(len)
                .is_none_or(|end| end > transport::BUFFER_SIZE)
        {
            return Err(IoError);
        }
        Ok(WINDOW + transport::BUFFER_OFFSET as u64 + offset as u64)
    }
}

impl Io for NativeIo {
    fn read32(&mut self, register: Register) -> Result<u32, IoError> {
        if QUARANTINED.load(Ordering::Acquire) {
            return Err(IoError);
        }
        barrier();
        // SAFETY: prepare exclusively pins the exact supervisor RW/NX/UC page
        // for the VM lifetime. Typed offsets are aligned 32-bit registers in
        // that page. No references or user/device-supplied pointers are formed.
        let value = unsafe { ((WINDOW + register as u64) as *const u32).read_volatile() };
        barrier();
        Ok(u32::from_le(value))
    }

    fn write32(&mut self, register: Register, value: u32) -> Result<(), IoError> {
        if QUARANTINED.load(Ordering::Acquire)
            || !matches!(
                register,
                Register::LocalityControl
                    | Register::ControlRequest
                    | Register::Cancel
                    | Register::Start
            )
        {
            return Err(IoError);
        }
        barrier();
        // SAFETY: Same permanent, exclusive aligned mapping as read32. Only
        // four writable control registers can pass the whitelist above.
        unsafe { ((WINDOW + register as u64) as *mut u32).write_volatile(value.to_le()) };
        barrier();
        WRITES.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn read_buffer(&mut self, offset: usize, out: &mut [u8]) -> Result<(), IoError> {
        let address = Self::buffer(offset, out.len())?;
        barrier();
        for (i, byte) in out.iter_mut().enumerate() {
            // SAFETY: Full relative range checked before any access; each byte
            // stays in the permanently mapped hardware buffer. out is owned
            // kernel RAM, not an alias or user buffer, and no borrow is retained.
            *byte = unsafe { ((address + i as u64) as *const u8).read_volatile() };
        }
        barrier();
        Ok(())
    }

    fn write_buffer(&mut self, offset: usize, bytes: &[u8]) -> Result<(), IoError> {
        let address = Self::buffer(offset, bytes.len())?;
        barrier();
        for (i, byte) in bytes.iter().enumerate() {
            // SAFETY: Same complete bounded MMIO range as read_buffer; no
            // references to device memory. Ordered exact copy, no DMA/RAM alias.
            unsafe { ((address + i as u64) as *mut u8).write_volatile(*byte) };
        }
        barrier();
        WRITES.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    fn quarantine(&mut self) {
        QUARANTINED.store(true, Ordering::Release);
    }
}

#[derive(Clone, Copy)]
enum Step {
    Fresh,
    Acquire(Ticket),
    Execute(Ticket),
    Nv(Ticket, NvStep),
    Release(Ticket),
    Complete,
    Absent,
}

#[derive(Clone, Copy)]
enum NvStep {
    Define,
    Public,
    Increment,
    FirstRead,
    Advance,
    FinalRead,
    WrongAuth,
    Missing,
}

struct Runtime {
    transport: Transport<NativeIo>,
    step: Step,
    counter: Option<u64>,
}

fn nv_request(step: NvStep) -> Request<'static> {
    let index = NvIndex::new(NV_INDEX).unwrap_or_else(|_| fail(b"NV_INDEX"));
    let auth = Password::new(NV_AUTH).unwrap_or_else(|_| fail(b"NV_AUTH"));
    match step {
        NvStep::Define => Request::NvDefineCounter {
            index,
            owner_auth: Password::new(b"").unwrap_or_else(|_| fail(b"NV_OWNER_AUTH")),
            index_auth: auth,
        },
        NvStep::Public => Request::NvReadPublic { index },
        NvStep::Increment | NvStep::Advance => Request::NvIncrement { index, auth },
        NvStep::FirstRead | NvStep::FinalRead => Request::NvReadCounter { index, auth },
        NvStep::WrongAuth => Request::NvReadCounter {
            index,
            auth: Password::new(b"WRONG-PUBLIC-FIXTURE").unwrap_or_else(|_| fail(b"NV_BAD_AUTH")),
        },
        NvStep::Missing => Request::NvReadPublic {
            index: NvIndex::new(NV_INDEX + 1).unwrap_or_else(|_| fail(b"NV_MISSING_INDEX")),
        },
    }
}

fn nv_start(r: &mut Runtime, step: NvStep, now: u64, deadline: u64) -> Step {
    let mut bytes = [0; codec::MAX_COMMAND];
    let command =
        codec::encode(nv_request(step), &mut bytes).unwrap_or_else(|_| fail(b"NV_ENCODE"));
    let ticket = r
        .transport
        .execute(&command, now, deadline, 10)
        .unwrap_or_else(|error| failure(error));
    Step::Nv(ticket, step)
}

fn nv_poll(
    r: &mut Runtime,
    ticket: Ticket,
    step: NvStep,
    now: u64,
    deadline: u64,
) -> Result<Step, Failure> {
    let response = match r.transport.poll(ticket, now) {
        Ok(Poll::Pending) => return Ok(Step::Nv(ticket, step)),
        Ok(Poll::Complete(response)) => response,
        Err(error) if error.effect == Effect::Completed && !r.transport.is_quarantined() => {
            let expected = match step {
                NvStep::WrongAuth => matches!(error.fault, Fault::Tpm(0x98e | 0x9a2)),
                NvStep::Missing => error.fault == Fault::Tpm(0x18b),
                _ => false,
            };
            if !expected {
                return Err(error);
            }
            if let Fault::Tpm(code) = error.fault {
                print_number(b"TPMNV:EXPECTED_RC:", u64::from(code));
            }
            return Ok(nv_start(r, NvStep::FinalRead, now, deadline));
        }
        Err(error) => return Err(error),
    };
    let next = match (step, response) {
        (NvStep::Define, Response::Empty) => NvStep::Public,
        (NvStep::Public, Response::NvPublic { index, .. }) if index.raw() == NV_INDEX => {
            serial::write(b"TPMNV:PUBLIC:PASS\n");
            if NV_PROVISION {
                NvStep::Increment
            } else {
                NvStep::FirstRead
            }
        }
        (NvStep::Increment, Response::Empty) => NvStep::FirstRead,
        (NvStep::Advance, Response::Empty) => NvStep::FinalRead,
        (NvStep::FirstRead, Response::Counter(value)) => {
            r.counter = Some(value);
            print_number(b"TPMNV:BEFORE:", value);
            if NV_PROVISION {
                NvStep::Advance
            } else if NV_AUTH_TEST {
                NvStep::WrongAuth
            } else if NV_MISSING {
                NvStep::Missing
            } else {
                return Ok(Step::Release(r.transport.release(now, deadline)?));
            }
        }
        (NvStep::FinalRead, Response::Counter(value)) => {
            let before = r.counter.unwrap_or_else(|| fail(b"NV_NO_BASELINE"));
            let expected = if NV_PROVISION {
                before
                    .checked_add(1)
                    .unwrap_or_else(|| fail(b"NV_OVERFLOW"))
            } else {
                before
            };
            if value != expected {
                fail(b"NV_COUNTER_CHANGED");
            }
            print_number(b"TPMNV:AFTER:", value);
            return Ok(Step::Release(r.transport.release(now, deadline)?));
        }
        _ => fail(b"NV_UNEXPECTED_RESPONSE"),
    };
    Ok(nv_start(r, next, now, deadline))
}

fn runtime() -> &'static mut Runtime {
    // SAFETY: Initialized once by prepare; one CPU, IF=0 during each bounded
    // selection hook. Borrow ends before scheduling/idle; ISR never uses this.
    unsafe {
        (&mut *ptr::addr_of_mut!(RUNTIME))
            .as_mut()
            .expect("TPM prepared")
    }
}

fn fail(reason: &[u8]) -> ! {
    serial::write(b"TPMCRB:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}

fn failure(error: Failure) -> ! {
    print_number(b"TPMCRB:EFFECT:", error.effect as u64);
    let reason: &[u8] = match error.fault {
        Fault::Input => b"INPUT",
        Fault::State => b"STATE",
        Fault::Ticket => b"TICKET",
        Fault::Exhausted => b"EXHAUSTED",
        Fault::Clock => b"CLOCK",
        Fault::Unsupported => b"UNSUPPORTED",
        Fault::Locality => b"LOCALITY",
        Fault::Device => b"DEVICE",
        Fault::Busy => b"BUSY",
        Fault::Buffer => b"BUFFER",
        Fault::Io => b"IO",
        Fault::Timeout => b"TIMEOUT",
        Fault::Cancelled => b"CANCELLED",
        Fault::CancelTimeout => b"CANCEL_TIMEOUT",
        Fault::Protocol => b"PROTOCOL",
        Fault::Quarantined => b"QUARANTINED",
        Fault::Tpm(code) => {
            print_number(b"TPMCRB:TPM_RC:", u64::from(code));
            b"TPM_RESPONSE"
        }
    };
    fail(reason)
}

pub fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    if CLAIMED.swap(true, Ordering::AcqRel) {
        fail(b"ALREADY_CLAIMED");
    }
    kernel.map_tpm_crb(frames).expect("fixed TPM UC mapping");
    assert_eq!(
        kernel.leaf(WINDOW) & (0x000f_ffff_ffff_f000 | (1 << 63) | 31),
        transport::BASE | (1 << 63) | 27
    );
    assert_eq!(
        kernel.permissions(WINDOW),
        Some((transport::BASE, true, false, false))
    );
    let mut io = NativeIo;
    let mut bytes = [0u8; 129];
    if io.write_buffer(usize::MAX, &[1]) != Err(IoError)
        || io.write_buffer(transport::BUFFER_SIZE, &[1]) != Err(IoError)
        || io.write_buffer(0, &bytes) != Err(IoError)
        || io.read_buffer(usize::MAX, &mut bytes[..1]) != Err(IoError)
        || io.read_buffer(transport::BUFFER_SIZE, &mut bytes[..1]) != Err(IoError)
        || io.read_buffer(0, &mut bytes) != Err(IoError)
        || io.write32(Register::InterfaceId, 1) != Err(IoError)
        || WRITES.load(Ordering::Relaxed) != 0
    {
        fail(b"ADAPTER_BOUNDS");
    }
    // SAFETY: Exclusive bootstrap initializes once, after pinning the mapping;
    // the non-Copy adapter moves into its sole transport. No detach/regrant API.
    unsafe {
        ptr::addr_of_mut!(RUNTIME).write(Some(Runtime {
            transport: Transport::new(io),
            step: Step::Fresh,
            counter: None,
        }));
    }
    serial::write(b"TPMCRB:BOUNDS:PASS\n");
}

pub fn manifest(state: &State) {
    for process in &state.processes[..state.amount] {
        if process.paging.as_ref().unwrap().permissions(WINDOW)
            != Some((transport::BASE, true, false, false))
        {
            fail(b"USER_MAPPING");
        }
    }
    serial::write(b"TPMCRB:MAPPING:PASS\n");
}

pub fn poll(now: u64) {
    let r = runtime();
    let deadline = now
        .checked_add(LIMIT)
        .unwrap_or_else(|| fail(b"CLOCK_OVERFLOW"));
    let outcome = match r.step {
        Step::Fresh => match r.transport.acquire(now, deadline) {
            Ok(ticket) => Ok(Step::Acquire(ticket)),
            Err(error)
                if cfg!(feature = "test-tpm-crb-absent")
                    && error.fault == Fault::Unsupported
                    && error.effect == Effect::NotStarted
                    && r.transport.is_quarantined()
                    && WRITES.load(Ordering::Relaxed) == 0 =>
            {
                Ok(Step::Absent)
            }
            Err(error) => Err(error),
        },
        Step::Acquire(ticket) => r.transport.poll_ready(ticket, now).map(|ready| {
            if ready {
                let mut bytes = [0u8; codec::MAX_COMMAND];
                let command = codec::encode(
                    Request::Properties {
                        first: 0x100,
                        count: 1,
                    },
                    &mut bytes,
                )
                .unwrap_or_else(|_| fail(b"ENCODE"));
                Step::Execute(
                    r.transport
                        .execute(&command, now, deadline, 10)
                        .unwrap_or_else(|error| failure(error)),
                )
            } else {
                Step::Acquire(ticket)
            }
        }),
        Step::Execute(ticket) => match r.transport.poll(ticket, now) {
            Ok(poll) => Ok(match poll {
                Poll::Pending => Step::Execute(ticket),
                Poll::Complete(Response::Properties {
                    count: 1, values, ..
                }) if values[0].property == 0x100 && values[0].value == 0x322e3000 => {
                    print_number(b"TPMCRB:FAMILY:", u64::from(values[0].value));
                    if NV_ENABLED {
                        nv_start(
                            r,
                            if NV_PROVISION {
                                NvStep::Define
                            } else {
                                NvStep::Public
                            },
                            now,
                            deadline,
                        )
                    } else {
                        Step::Release(
                            r.transport
                                .release(now, deadline)
                                .unwrap_or_else(|error| failure(error)),
                        )
                    }
                }
                Poll::Complete(_) => fail(b"FAMILY_RESPONSE"),
            }),
            Err(error) => Err(error),
        },
        Step::Nv(ticket, step) => nv_poll(r, ticket, step, now, deadline),
        Step::Release(ticket) => r.transport.poll_released(ticket, now).map(|released| {
            if released {
                let _io = r.transport.detach().unwrap_or_else(|error| failure(error));
                Step::Complete
            } else {
                Step::Release(ticket)
            }
        }),
        Step::Complete | Step::Absent => return,
    };
    r.step = outcome.unwrap_or_else(|error| failure(error));
}

pub fn finish() {
    print_number(b"TPMCRB:WRITES:", WRITES.load(Ordering::Relaxed));
    print_number(b"TPMCRB:BOUNDS_REJECTED:", 7);
    print_number(
        b"TPMCRB:QUARANTINED:",
        u64::from(QUARANTINED.load(Ordering::Acquire)),
    );
    match runtime().step {
        Step::Complete
            if !cfg!(feature = "test-tpm-crb-absent")
                && !QUARANTINED.load(Ordering::Acquire)
                && WRITES.load(Ordering::Relaxed) > 0 =>
        {
            serial::write(b"TPMCRB:RELEASED:PASS\nTPMCRB:PASS\n");
            if NV_ENABLED {
                let marker: &[u8] = if NV_PROVISION {
                    b"provision"
                } else if NV_READ {
                    b"read"
                } else if NV_AUTH_TEST {
                    b"auth"
                } else {
                    b"missing"
                };
                serial::write(b"TPMNV:PASS:");
                serial::write(marker);
                serial::write(b"\n");
            }
        }
        Step::Absent if cfg!(feature = "test-tpm-crb-absent") => {
            serial::write(b"TPMCRB:ABSENT:PASS\n");
        }
        _ => fail(b"INCOMPLETE_OR_WRONG_CASE"),
    }
}
