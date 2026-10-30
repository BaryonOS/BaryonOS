#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! One acceptance ELF, three isolated manifest roles. Code possession is not
//! authority: only the role-specific kernel-granted endpoints authorize use.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("native NAS probe requires Nieuw OS, never host int80");

#[path = "../../../../sdk/examples/system/src/support.rs"]
mod support;

use nieuw_os_nas_core::{Entropy, Error as NasError, NamespaceStatus, Service, Workspace};
use nieuw_os_nas_probe::{dispatch, Frame, Ingress, LocalBinding, Op, RequestContext, SIZE, TAG};
use nieuw_os_nas_volume::VolumeStorage;
use nieuw_os_sdk_runtime::{
    entropy::EntropySource, memory, native, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};
use volume_core::{client::Client, client::Transport, wire, Status};

const COORD: u64 = TAG + 1;
const LIMIT: usize = 500_000;
const STACK_BASE: u64 = 0x7ffef000;
const STACK_END: u64 = 0x7ffff000;
const CANARY: u64 = 0xa55ac33ca55ac33c;

// SAFETY: This acceptance manifest maps exactly this private 64-KiB RW/NX
// stack before entry, with no arguments on it. No Rust frame exists yet.
// The probe never returns through the initial unused return word.
core::arch::global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
.type _start,@function
_start:
    cld
    mov rdi, 0x7ffef000
    mov rcx, 8192
    mov rax, 0xa55ac33ca55ac33c
    rep stosq
    jmp nas_main
.size _start, .-_start
"#
);

fn finish() -> ! {
    let stack: u64;
    // SAFETY: Read the current stack register only; no memory or flags changed.
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) stack, options(nomem, nostack, preserves_flags));
    }
    let limit = stack & !7;
    check((STACK_BASE..STACK_END).contains(&limit), 101);
    let mut at = STACK_BASE;
    while at < limit {
        // SAFETY: The entry initialized every word. Only inactive stack below
        // current RSP is inspected, never a live Rust frame/borrow. All crates
        // are built with no-redzone; no allocator uses this range.
        if unsafe { (at as *const u64).read_volatile() } != CANARY {
            break;
        }
        at += 8;
    }
    support::put(448, STACK_END - at);
    support::put(40, 1);
    native::exit(0)
}

fn fail(step: u64) -> ! {
    support::put(240, step);
    native::exit(1)
}
fn check(ok: bool, step: u64) {
    if !ok {
        fail(step);
    }
}
fn bump(at: u64) {
    support::put(
        at,
        support::word(at).checked_add(1).unwrap_or_else(|| fail(90)),
    );
}
fn handle(at: u64) -> Handle {
    Handle::from_raw(support::word(at)).unwrap_or_else(|_| fail(91))
}
fn send(to: SendEndpoint, tag: u64, bytes: &[u8]) -> Result<(), Error> {
    let message = Outgoing::new(tag, bytes)?;
    for _ in 0..LIMIT {
        bump(8);
        match native::send(to, &message) {
            Ok(()) => return Ok(()),
            Err(error) if error.is_again() => native::yield_now()?,
            Err(error) => return Err(error),
        }
    }
    Err(Error::Kernel(-13))
}
fn receive<const N: usize>(from: ReceiveEndpoint, peer: u64, tag: u64) -> Result<[u8; N], Error> {
    for _ in 0..LIMIT {
        bump(8);
        match native::receive(from) {
            Ok(message) => {
                if message.cap_count() != 0 {
                    native::close_received_caps(&message)?;
                    return Err(Error::InvalidReturn);
                }
                if message.sender() != peer || message.tag() != tag || message.payload().len() != N
                {
                    return Err(Error::InvalidReturn);
                }
                let mut bytes = [0; N];
                bytes.copy_from_slice(message.payload());
                return Ok(bytes);
            }
            Err(error) if error.is_again() => native::yield_now()?,
            Err(error) => return Err(error),
        }
    }
    Err(Error::Kernel(-13))
}

struct Rpc {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
    next: u64,
}
impl Rpc {
    fn new(send_at: u64, receive_at: u64, peer_at: u64) -> Self {
        Self {
            send: SendEndpoint::from_handle(handle(send_at)),
            receive: ReceiveEndpoint::from_handle(handle(receive_at)),
            peer: support::word(peer_at),
            next: 1,
        }
    }
    fn exchange<const N: usize>(&mut self, tag: u64, bytes: &[u8; N]) -> Result<[u8; N], Error> {
        send(self.send, tag, bytes)?;
        receive(self.receive, self.peer, tag)
    }
    fn call(&mut self, mut request: Frame, status: Option<NasError>) -> Frame {
        request.request = self.next;
        self.next = self.next.checked_add(1).unwrap_or_else(|| fail(92));
        let bytes = request.encode(false).unwrap_or_else(|_| fail(93));
        let bytes = self.exchange(TAG, &bytes).unwrap_or_else(|_| fail(94));
        let reply = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(95));
        check(
            reply.op == request.op && reply.request == request.request && reply.status == status,
            96,
        );
        bump(if status.is_none() { 400 } else { 408 });
        reply
    }
}
impl Transport for Rpc {
    fn call(&mut self, bytes: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        self.exchange(wire::TAG, bytes)
            .map_err(|error| match error {
                Error::Kernel(code) => Status::from_code(code),
                _ => Status::Protocol,
            })
    }
}

struct NativeEntropy(EntropySource);
impl Entropy for NativeEntropy {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), NasError> {
        let ticket = native::entropy_request(self.0, out.len()).map_err(|_| NasError::Entropy)?;
        native::entropy_wait(self.0, ticket, out, LIMIT).map_err(|_| NasError::Entropy)
    }
}

fn server(recover: bool) -> ! {
    let binding =
        LocalBinding::new(support::word(160), support::word(168)).unwrap_or_else(|_| fail(1));
    check(
        core::mem::size_of::<Workspace>() <= memory::INITIAL_HEAP_BYTES as usize
            && memory::HEAP_BASE.is_multiple_of(core::mem::align_of::<Workspace>() as u64),
        2,
    );
    // SAFETY: This manifest maps the initial 64-KiB private RW/NX heap for
    // this single-thread process. No allocator or other heap user exists in
    // this ELF. Called once; the exclusive reference lives until process exit.
    // `write` constructs a valid Workspace, not a zero-bit-layout assumption.
    let work = unsafe {
        let pointer = memory::HEAP_BASE as *mut Workspace;
        pointer.write(Workspace::new());
        &mut *pointer
    };
    let storage =
        VolumeStorage::new(Client::new(Rpc::new(88, 96, 184), 1, 1), 0).unwrap_or_else(|_| fail(3));
    let mut entropy = NativeEntropy(EntropySource::from_handle(handle(48)));
    let mut service = if recover {
        Service::recover(storage, &mut entropy, work)
    } else {
        Service::initialize(
            storage,
            LocalBinding::principal(binding.admin_pid),
            &mut entropy,
            work,
        )
    }
    .unwrap_or_else(|_| fail(4));
    support::put(192, 1);
    loop {
        for (ingress, root, reply) in [(Ingress::Administrator, 56, 72), (Ingress::Guest, 64, 80)] {
            bump(8);
            let message = match native::receive(ReceiveEndpoint::from_handle(handle(root))) {
                Ok(message) => message,
                Err(error) if error.is_again() => continue,
                Err(_) => fail(5),
            };
            if message.cap_count() != 0 {
                native::close_received_caps(&message).unwrap_or_else(|_| fail(6));
                bump(424);
                continue;
            }
            if message.tag() != TAG || message.payload().len() != SIZE {
                bump(424);
                continue;
            }
            let mut bytes = [0; SIZE];
            bytes.copy_from_slice(message.payload());
            let now = native::monotonic_ticks().unwrap_or_else(|_| fail(7));
            let context = RequestContext {
                binding: &binding,
                ingress,
                sender: message.sender(),
                now,
                recover,
            };
            match dispatch(&mut service, &context, &bytes, &mut entropy) {
                Ok((bytes, shutdown)) => {
                    let frame = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(8));
                    bump(if frame.status.is_none() { 400 } else { 408 });
                    send(SendEndpoint::from_handle(handle(reply)), TAG, &bytes)
                        .unwrap_or_else(|_| fail(9));
                    if shutdown {
                        finish();
                    }
                }
                Err(NasError::Denied) => bump(416),
                Err(_) => bump(424),
            }
        }
        native::yield_now().unwrap_or_else(|_| fail(10));
    }
}

fn notify(op: u8) {
    send(SendEndpoint::from_handle(handle(72)), COORD, &[op]).unwrap_or_else(|_| fail(20));
}
fn wait(op: u8) {
    let bytes: [u8; 1] = receive(
        ReceiveEndpoint::from_handle(handle(80)),
        support::word(168),
        COORD,
    )
    .unwrap_or_else(|_| fail(21));
    check(bytes == [op], 22);
}
fn named(op: Op, namespace: [u8; 32]) -> Frame {
    let mut frame = Frame::new(op, 1);
    frame.namespace = namespace;
    frame
}
fn admin(recover: bool) -> ! {
    let mut rpc = Rpc::new(56, 64, 160);
    let invitation = if recover {
        let usage = rpc.call(Frame::new(Op::UsageAt, 1), None);
        check(
            usage.data[0] == NamespaceStatus::Evicted as u8
                && usage.data[4..12] == [0; 8]
                && usage.revision == 2,
            30,
        );
        usage
    } else {
        let mut create = Frame::new(Op::Invite, 1);
        create.value = 128;
        rpc.call(create, None)
    };
    let namespace = invitation.namespace;
    send(
        SendEndpoint::from_handle(handle(72)),
        COORD,
        &invitation.encode(true).unwrap_or_else(|_| fail(31)),
    )
    .unwrap_or_else(|_| fail(32));
    if !recover {
        wait(1);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(
            usage.data[0] == NamespaceStatus::Active as u8
                && u32::from_le_bytes(usage.data[4..8].try_into().unwrap()) == 96
                && u32::from_le_bytes(usage.data[8..12].try_into().unwrap()) == 32,
            33,
        );
        let mut quota = named(Op::SetQuota, namespace);
        quota.value = 127;
        rpc.call(quota, Some(NasError::Quota));
        rpc.call(named(Op::Revoke, namespace), None);
        notify(2);
    }
    wait(3);
    rpc.call(named(Op::Evict, namespace), None);
    let usage = rpc.call(named(Op::Usage, namespace), None);
    check(
        usage.data[0] == NamespaceStatus::Evicted as u8 && usage.data[4..12] == [0; 8],
        34,
    );
    rpc.call(Frame::new(Op::Shutdown, 1), None);
    finish()
}

fn guest(recover: bool) -> ! {
    let mut rpc = Rpc::new(56, 64, 160);
    let bytes = receive(
        ReceiveEndpoint::from_handle(handle(80)),
        support::word(168),
        COORD,
    )
    .unwrap_or_else(|_| fail(40));
    let invitation = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(41));
    check(
        invitation.op == if recover { Op::UsageAt } else { Op::Invite },
        42,
    );
    let namespace = invitation.namespace;
    rpc.call(Frame::new(Op::UsageAt, 1), Some(NasError::Denied));
    rpc.call(named(Op::Revoke, namespace), Some(NasError::Denied));
    check(
        matches!(
            native::receive(ReceiveEndpoint::from_handle(handle(56))),
            Err(Error::Kernel(-5))
        ),
        44,
    );
    bump(416);
    if recover {
        rpc.call(named(Op::Open, namespace), Some(NasError::Denied));
        let mut read = Frame::new(Op::Read, 1);
        read.session = [42; 40];
        read.revision = 1;
        rpc.call(read, Some(NasError::Stale));
    } else {
        let mut accept = named(Op::Accept, namespace);
        accept.value = invitation.value;
        accept.length = 32;
        accept.data[..32].copy_from_slice(&invitation.data[..32]);
        rpc.call(accept, None);
        let session = rpc.call(named(Op::Open, namespace), None).session;
        let mut stage = Frame::new(Op::Stage, 1);
        stage.session = session;
        stage.length = 96;
        for (i, byte) in stage.data[..96].iter_mut().enumerate() {
            *byte = (i * 17 + 9) as u8; // Synthetic opaque bytes, not E2EE proof.
        }
        let upload = rpc.call(stage, None).upload;
        let mut commit = Frame::new(Op::Commit, 1);
        commit.session = session;
        commit.upload = upload;
        check(rpc.call(commit, None).revision == 1, 45);
        let mut read = Frame::new(Op::Read, 1);
        read.session = session;
        read.revision = 1;
        let reply = rpc.call(read, None);
        check(
            reply.length == 96
                && reply.data[..96]
                    .iter()
                    .enumerate()
                    .all(|(i, b)| *b == (i * 17 + 9) as u8),
            46,
        );
        let mut pending = Frame::new(Op::Stage, 1);
        pending.session = session;
        pending.revision = 1;
        pending.length = 32;
        pending.data[..32].fill(0x55);
        let pending = rpc.call(pending, None).upload;
        notify(1);
        wait(2);
        let mut old = Frame::new(Op::Commit, 1);
        old.session = session;
        old.upload = pending;
        rpc.call(old, Some(NasError::Stale));
        let mut read = Frame::new(Op::Read, 1);
        read.session = session;
        read.revision = 1;
        rpc.call(read, Some(NasError::Stale));
        rpc.call(named(Op::Open, namespace), Some(NasError::Denied));
    }
    notify(3);
    finish()
}

#[no_mangle]
pub extern "C" fn nas_main() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(97));
    let recover = match support::word(112) {
        0 => false,
        1 => true,
        _ => fail(98),
    };
    match support::word(120) {
        0 => admin(recover),
        1 => server(recover),
        3 => guest(recover),
        _ => fail(99),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    fail(100)
}
