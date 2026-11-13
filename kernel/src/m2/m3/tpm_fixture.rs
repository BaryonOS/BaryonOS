//! Fixed QEMU CRB bootstrap experiment, not a product TPM grant or key policy.
use super::{print_number, serial, Frames, Paging, State, FRAME_COUNT};
use core::{
    arch::asm,
    ptr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
use nieuw_os_tpm_crb::{
    codec::{self, Request, Response},
    transport::{self, Effect, Failure, Fault, Io, IoError, Poll, Register, Ticket, Transport},
};

const WINDOW: u64 = 0x3f180000;
const LIMIT: u64 = 50;
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
    Release(Ticket),
    Complete,
    Absent,
}

struct Runtime {
    transport: Transport<NativeIo>,
    step: Step,
}

fn runtime() -> &'static mut Runtime {
    // SAFETY: Initialized once by prepare; KERNEL held, IF=0 during each bounded
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
        Step::Execute(ticket) => r.transport.poll(ticket, now).map(|poll| match poll {
            Poll::Pending => Step::Execute(ticket),
            Poll::Complete(Response::Properties {
                count: 1, values, ..
            }) if values[0].property == 0x100 && values[0].value == 0x322e3000 => {
                print_number(b"TPMCRB:FAMILY:", u64::from(values[0].value));
                Step::Release(
                    r.transport
                        .release(now, deadline)
                        .unwrap_or_else(|error| failure(error)),
                )
            }
            Poll::Complete(_) => fail(b"FAMILY_RESPONSE"),
        }),
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
        }
        Step::Absent if cfg!(feature = "test-tpm-crb-absent") => {
            serial::write(b"TPMCRB:ABSENT:PASS\n");
        }
        _ => fail(b"INCOMPLETE_OR_WRONG_CASE"),
    }
}
