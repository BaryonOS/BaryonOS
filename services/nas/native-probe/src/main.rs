#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! One acceptance ELF, three isolated manifest roles. Code possession is not
//! authority: only the role-specific kernel-granted endpoints authorize use.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("native NAS probe requires Nieuw OS, never host int80");

#[path = "../../../../sdk/examples/system/src/support.rs"]
mod support;

use core::sync::atomic::{AtomicUsize, Ordering};
use nieuw_os_nas_client::{Client as SyncClient, State};
use nieuw_os_nas_core::{
    Entropy, Error as NasError, NamespaceStatus, Service, Session, Upload, Workspace, TOKEN_BYTES,
};
use nieuw_os_nas_probe::{
    dispatch, volume_transport, Delivery, Frame, Ingress, LocalBinding, NaspRemote, Op,
    RequestContext, SIZE, TAG,
};
use nieuw_os_nas_volume::VolumeStorage;
use nieuw_os_sdk_runtime::{
    entropy::EntropySource, memory, native, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};
use volume_core::{client::Client, client::Transport, wire, Status};

const COORD: u64 = TAG + 1;
// Written only by the fixture kernel after observing the normal guest exit.
const GUEST_EXITED: u64 = 200;
const LIMIT: usize = 500_000;
const STACK_BASE: u64 = 0x7ffef000;
const STACK_END: u64 = 0x7ffff000;
const CANARY: u64 = 0xa55ac33ca55ac33c;
// Startup word 112: the fixture stage chosen by the kernel manifest.
const COLD: u64 = 1;
const INTERRUPT: u64 = 2;
const REPAIR: u64 = 3;
/// P02-T3: boot 1 commits, then edits offline after the NAS exits; boot 2
/// recovers the NAS on the same disk and reconciles through Head.
const OFFLINE: u64 = 4;
const RESUME: u64 = 5;
// Guest word 216 = 1 only after the fixture kernel observed the NAS exit.
const NAS_EXITED: u64 = 216;
// NAS word 208 = 1 only after the repair boot saw `ReopenRequired`, purged the
// retained history and recovered. Word 40 = 2 marks the fixture interruption.
const REPAIRED: u64 = 208;
const INTERRUPTED: u64 = 2;
/// P04-T1 interrupt stage only: PURGE requests left before the NAS process
/// dies. Armed to 2 by an Evict on an Active namespace (revoke's PURGE, then
/// evict's), so the process exits after evict's WRITE ACK and before its PURGE.
static PURGES_BEFORE_EXIT: AtomicUsize = AtomicUsize::new(0);

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
    stack_mark();
    support::put(40, 1);
    native::exit(0)
}

/// Fixture crash point: the NAS process exits with a distinct code and word
/// 40 = 2, never 1, so the kernel cannot mistake it for success or failure.
fn interrupted() -> ! {
    stack_mark();
    support::put(40, INTERRUPTED);
    native::exit(3)
}

fn stack_mark() {
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
/// Startup bytes 256..296 and 296..336: the exact boot-1 session and upload
/// handles (see CONTRACT.md). The normal guest publishes them here for the
/// kernel's private test handoff; the cold guest receives them here from it.
const REPLAY_SESSION: u64 = 256;
const REPLAY_UPLOAD: u64 = 296;
fn cold_replay_token(offset: u64) -> [u8; TOKEN_BYTES] {
    check(
        offset.is_multiple_of(8) && offset + TOKEN_BYTES as u64 <= 1024,
        102,
    );
    let mut token = [0; TOKEN_BYTES];
    for word in 0..TOKEN_BYTES / 8 {
        let start = word * 8;
        token[start..start + 8]
            .copy_from_slice(&support::word(offset + start as u64).to_le_bytes());
    }
    token
}
fn publish_replay_token(offset: u64, token: &[u8; TOKEN_BYTES]) {
    check(
        offset.is_multiple_of(8) && offset + TOKEN_BYTES as u64 <= 1024,
        105,
    );
    for word in 0..TOKEN_BYTES / 8 {
        let start = word * 8;
        let mut bytes = [0; 8];
        bytes.copy_from_slice(&token[start..start + 8]);
        support::put(offset + start as u64, u64::from_le_bytes(bytes));
    }
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
        if wire::header(bytes).operation == wire::PURGE {
            match PURGES_BEFORE_EXIT.load(Ordering::Relaxed) {
                0 => {}
                1 => interrupted(),
                left => PURGES_BEFORE_EXIT.store(left - 1, Ordering::Relaxed),
            }
        }
        volume_transport::exchange(
            || send(self.send, wire::TAG, bytes),
            || receive(self.receive, self.peer, wire::TAG),
        )
    }
}

struct NativeEntropy(EntropySource);
impl Entropy for NativeEntropy {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), NasError> {
        let ticket = native::entropy_request(self.0, out.len()).map_err(|_| NasError::Entropy)?;
        native::entropy_wait(self.0, ticket, out, LIMIT).map_err(|_| NasError::Entropy)
    }
}

fn volume_storage(first_request: u64) -> VolumeStorage<Rpc> {
    VolumeStorage::new(Client::new(Rpc::new(88, 96, 184), 1, first_request), 0)
        .unwrap_or_else(|_| fail(3))
}

fn server(mode: u64) -> ! {
    let recover = matches!(mode, COLD | REPAIR | RESUME);
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
    let storage = volume_storage(1);
    let mut entropy = NativeEntropy(EntropySource::from_handle(handle(48)));
    let mut service = if mode == REPAIR {
        // Explicit startup repair, never an implicit retry: plain recovery
        // must refuse the retained history first. Cleanup validates the
        // current snapshot before purging, so it cannot pick older authority.
        if !matches!(
            Service::recover(storage, &mut entropy, &mut *work),
            Err(NasError::ReopenRequired)
        ) {
            fail(11);
        }
        volume_storage(1 << 20)
            .finish_cleanup()
            .unwrap_or_else(|_| fail(12));
        support::put(REPAIRED, 1);
        Service::recover(volume_storage(2 << 20), &mut entropy, work)
    } else if recover {
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
            if mode == INTERRUPT && bytes[6..8] == (Op::Evict as u16).to_le_bytes() {
                PURGES_BEFORE_EXIT.store(2, Ordering::Relaxed);
            }
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
        notify(4);
        wait(5);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(usage.data[4..12] == [96, 0, 0, 0, 0, 0, 0, 0], 35);
        notify(6);
        wait(7);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(usage.data[4..12] == [96, 0, 0, 0, 32, 0, 0, 0], 36);
        let mut quota = named(Op::SetQuota, namespace);
        quota.value = 127;
        rpc.call(quota, Some(NasError::Quota));
        let mut quota = named(Op::SetQuota, namespace);
        quota.value = 256;
        rpc.call(quota, None);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(
            usage.value == 256 && usage.data[4..12] == [96, 0, 0, 0, 32, 0, 0, 0],
            37,
        );
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
    if !recover {
        let mut create = Frame::new(Op::Invite, 1);
        create.value = 128;
        let invitation = rpc.call(create, None);
        check(invitation.namespace != namespace, 38);
        send(
            SendEndpoint::from_handle(handle(72)),
            COORD,
            &invitation.encode(true).unwrap_or_else(|_| fail(31)),
        )
        .unwrap_or_else(|_| fail(32));
        // Readiness is only coordination. Only the kernel exit observation
        // permits the offline revoke/evict below; no yield count proves death.
        wait(8);
        let mut exited = false;
        for _ in 0..LIMIT {
            if support::word(GUEST_EXITED) == 1 {
                exited = true;
                break;
            }
            native::yield_now().unwrap_or_else(|_| fail(39));
        }
        check(exited, 39);
        let namespace = invitation.namespace;
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(
            usage.data[0] == NamespaceStatus::Active as u8
                && usage.data[4..12] == [0, 0, 0, 0, 32, 0, 0, 0],
            50,
        );
        rpc.call(named(Op::Revoke, namespace), None);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(
            usage.data[0] == NamespaceStatus::Revoked as u8 && usage.data[4..12] == [0; 8],
            51,
        );
        rpc.call(named(Op::Evict, namespace), None);
        let usage = rpc.call(named(Op::Usage, namespace), None);
        check(
            usage.data[0] == NamespaceStatus::Evicted as u8 && usage.data[4..12] == [0; 8],
            52,
        );
    }
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
    }
    if recover && support::word(112) == COLD {
        // Exact boot-1 handles from the kernel's private handoff; absent, zero
        // or non-canonical input fails here before any replay request.
        let session = cold_replay_token(REPLAY_SESSION);
        let upload = cold_replay_token(REPLAY_UPLOAD);
        check(Session::from_bytes(&session).is_ok(), 47);
        check(Upload::from_bytes(&upload).is_ok(), 48);
        let mut read = Frame::new(Op::Read, 1);
        read.session = session;
        read.revision = 1;
        rpc.call(read, Some(NasError::Stale));
        let mut replay = Frame::new(Op::Commit, 1);
        replay.session = session;
        replay.upload = upload;
        rpc.call(replay, Some(NasError::Stale));
    } else if recover {
        // Repair boot: no token handoff exists for this image.
        check(cold_replay_token(REPLAY_SESSION) == [0; TOKEN_BYTES], 106);
        check(cold_replay_token(REPLAY_UPLOAD) == [0; TOKEN_BYTES], 107);
    } else {
        check(cold_replay_token(REPLAY_SESSION) == [0; TOKEN_BYTES], 103);
        check(cold_replay_token(REPLAY_UPLOAD) == [0; TOKEN_BYTES], 104);
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
        let mut extra = Frame::new(Op::Stage, 1);
        extra.session = session;
        extra.revision = 1;
        extra.length = 1;
        extra.data[0] = 0x55;
        rpc.call(extra, Some(NasError::Quota));
        notify(1);
        wait(4);
        let mut cancel = Frame::new(Op::Cancel, 1);
        cancel.session = session;
        cancel.upload = pending;
        rpc.call(cancel, None);
        notify(5);
        wait(6);
        let mut retry = Frame::new(Op::Stage, 1);
        retry.session = session;
        retry.revision = 1;
        retry.length = 32;
        retry.data[..32].fill(0x55);
        let pending = rpc.call(retry, None).upload;
        // The exact Stage-reply pair of this boot, published for the private
        // handoff only after both handles were issued by the real NAS core.
        publish_replay_token(REPLAY_SESSION, &session);
        publish_replay_token(REPLAY_UPLOAD, &pending);
        notify(7);
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
    if !recover {
        let bytes = receive(
            ReceiveEndpoint::from_handle(handle(80)),
            support::word(168),
            COORD,
        )
        .unwrap_or_else(|_| fail(53));
        let invitation = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(54));
        check(
            invitation.op == Op::Invite && invitation.namespace != namespace,
            55,
        );
        let mut accept = named(Op::Accept, invitation.namespace);
        accept.value = invitation.value;
        accept.length = 32;
        accept.data[..32].copy_from_slice(&invitation.data[..32]);
        rpc.call(accept, None);
        let session = rpc
            .call(named(Op::Open, invitation.namespace), None)
            .session;
        let mut held = Frame::new(Op::Stage, 1);
        held.session = session;
        held.length = 32;
        held.data[..32].fill(0x66);
        rpc.call(held, None);
        notify(8);
    }
    finish()
}

/// P04-T1 interrupt stage: one guest commits synthetic opaque bytes and
/// exits; the administrator evicts the Active namespace, whose reply never
/// comes because the NAS dies before evict's history PURGE.
fn admin_interrupt() -> ! {
    let mut rpc = Rpc::new(56, 64, 160);
    let mut create = Frame::new(Op::Invite, 1);
    create.value = 128;
    let invitation = rpc.call(create, None);
    let namespace = invitation.namespace;
    send(
        SendEndpoint::from_handle(handle(72)),
        COORD,
        &invitation.encode(true).unwrap_or_else(|_| fail(31)),
    )
    .unwrap_or_else(|_| fail(32));
    wait(1);
    let usage = rpc.call(named(Op::Usage, namespace), None);
    check(
        usage.data[0] == NamespaceStatus::Active as u8
            && usage.data[4..12] == [96, 0, 0, 0, 0, 0, 0, 0],
        60,
    );
    let mut evict = named(Op::Evict, namespace);
    evict.request = rpc.next;
    let bytes = evict.encode(false).unwrap_or_else(|_| fail(61));
    // Publish the waiting marker before Evict: the NAS may exit, and the
    // kernel check word 40, before this process runs again after the send.
    stack_mark();
    support::put(40, INTERRUPTED);
    send(rpc.send, TAG, &bytes).unwrap_or_else(|_| fail(62));
    // No reply is expected. The kernel closes the case on the NAS exit and
    // reaps this process; it never reaches the bounded failure below.
    for _ in 0..LIMIT {
        native::yield_now().unwrap_or_else(|_| fail(63));
    }
    fail(64)
}

fn guest_interrupt() -> ! {
    let mut rpc = Rpc::new(56, 64, 160);
    let bytes = receive(
        ReceiveEndpoint::from_handle(handle(80)),
        support::word(168),
        COORD,
    )
    .unwrap_or_else(|_| fail(40));
    let invitation = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(41));
    check(invitation.op == Op::Invite, 42);
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
    check(cold_replay_token(REPLAY_SESSION) == [0; TOKEN_BYTES], 103);
    check(cold_replay_token(REPLAY_UPLOAD) == [0; TOKEN_BYTES], 104);
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
    notify(1);
    finish()
}

/// Synthetic opaque bytes, not E2EE proof: revision 1 (boot 1), the edit
/// queued while the NAS is down, and a stale edit from an older base.
fn synthetic<const N: usize>(mul: usize, add: usize) -> [u8; N] {
    core::array::from_fn(|i| (i * mul + add) as u8)
}
const FIRST: (usize, usize) = (17, 9);
const EDIT: (usize, usize) = (29, 3);

/// The guest's `native-client` over NASP. A send that queued nothing is
/// `NotSent`; once queued, a missing or bad reply is `Sent` (`Unknown`).
fn client_remote(
    rpc: &Rpc,
) -> NaspRemote<impl FnMut(&[u8; SIZE]) -> Result<[u8; SIZE], Delivery> + '_> {
    NaspRemote::new(LocalBinding::principal(support::word(176)), move |bytes| {
        send(rpc.send, TAG, bytes).map_err(|_| Delivery::NotSent(NasError::Io))?;
        let reply: [u8; SIZE] = receive(rpc.receive, rpc.peer, TAG).map_err(|_| Delivery::Sent)?;
        bump(if reply[16..20] == [0; 4] { 400 } else { 408 });
        Ok(reply)
    })
}

/// P02-T3 administrator. Boot 1 creates the namespace and stops the NAS
/// after the guest's first commit; boot 2 finds the namespace through the
/// recovered core and checks the guest's reconciled commit.
fn admin_resume(resume: bool) -> ! {
    let mut rpc = Rpc::new(56, 64, 160);
    let announce = if resume {
        let usage = rpc.call(Frame::new(Op::UsageAt, 1), None);
        check(
            usage.data[0] == NamespaceStatus::Active as u8
                && usage.data[4..12] == [96, 0, 0, 0, 0, 0, 0, 0]
                && usage.revision == 1,
            70,
        );
        usage
    } else {
        let mut create = Frame::new(Op::Invite, 1);
        create.value = 128;
        rpc.call(create, None)
    };
    let namespace = announce.namespace;
    send(
        SendEndpoint::from_handle(handle(72)),
        COORD,
        &announce.encode(true).unwrap_or_else(|_| fail(71)),
    )
    .unwrap_or_else(|_| fail(72));
    wait(1);
    let usage = rpc.call(named(Op::Usage, namespace), None);
    let used: u8 = if resume { 32 } else { 96 };
    check(
        usage.data[0] == NamespaceStatus::Active as u8
            && usage.data[4..12] == [used, 0, 0, 0, 0, 0, 0, 0],
        73,
    );
    rpc.call(Frame::new(Op::Shutdown, 1), None);
    finish()
}

fn guest_resume(resume: bool) -> ! {
    let rpc = Rpc::new(56, 64, 160);
    let bytes = receive(
        ReceiveEndpoint::from_handle(handle(80)),
        support::word(168),
        COORD,
    )
    .unwrap_or_else(|_| fail(40));
    let announce = Frame::decode(&bytes, true).unwrap_or_else(|_| fail(41));
    check(
        announce.op == if resume { Op::UsageAt } else { Op::Invite },
        42,
    );
    let namespace = announce.namespace;
    let mut rights = Rpc::new(56, 64, 160);
    rights.call(Frame::new(Op::UsageAt, 1), Some(NasError::Denied));
    rights.call(named(Op::Revoke, namespace), Some(NasError::Denied));
    check(
        matches!(
            native::receive(ReceiveEndpoint::from_handle(handle(56))),
            Err(Error::Kernel(-5))
        ),
        44,
    );
    bump(416);
    let mut client = SyncClient::new();
    let mut remote = client_remote(&rpc);
    // 32 bytes: the core's copy-on-write reservation counts the old 96-byte
    // revision too, and 96 + 32 is exactly the 128-byte quota.
    let edit: [u8; 32] = synthetic(EDIT.0, EDIT.1);
    if resume {
        // An edit from an older base meets revision 1: a conflict, no write.
        let stale = client
            .enqueue(namespace, 0, 0, &[0x77; 40])
            .unwrap_or_else(|_| fail(80));
        check(
            client.reconcile_latest_via(stale, &mut remote) == Ok(State::Conflict),
            81,
        );
        client.take(stale).unwrap_or_else(|_| fail(82));
        // The edit queued in boot 1, re-created from the probe constants:
        // this probe has no client persistence (an OWNER question).
        let slot = client
            .enqueue(namespace, 0, 1, &edit)
            .unwrap_or_else(|_| fail(83));
        check(
            client.reconcile_latest_via(slot, &mut remote) == Ok(State::Ready),
            84,
        );
        client
            .prepare_via(slot, &mut remote)
            .unwrap_or_else(|_| fail(85));
        check(client.commit_via(slot, &mut remote) == Ok(2), 86);
        notify(1);
        finish()
    }
    let mut accept = named(Op::Accept, namespace);
    accept.value = announce.value;
    accept.length = 32;
    accept.data[..32].copy_from_slice(&announce.data[..32]);
    rights.call(accept, None);
    let first: [u8; 96] = synthetic(FIRST.0, FIRST.1);
    let slot = client
        .enqueue(namespace, 0, 0, &first)
        .unwrap_or_else(|_| fail(87));
    client
        .prepare_via(slot, &mut remote)
        .unwrap_or_else(|_| fail(88));
    check(client.commit_via(slot, &mut remote) == Ok(1), 89);
    client.take(slot).unwrap_or_else(|_| fail(110));
    notify(1);
    // Only the kernel's exit observation proves the NAS is down.
    let mut exited = false;
    for _ in 0..LIMIT {
        if support::word(NAS_EXITED) == 1 {
            exited = true;
            break;
        }
        native::yield_now().unwrap_or_else(|_| fail(111));
    }
    check(exited, 111);
    let slot = client
        .enqueue(namespace, 0, 1, &edit)
        .unwrap_or_else(|_| fail(112));
    check(
        client
            .prepare_via(slot, &mut remote)
            .map_err(|f| f.operation)
            == Err(NasError::Io),
        113,
    );
    check(
        client.edit(slot).map(|e| e.state()) == Ok(State::NeedsReconcile),
        114,
    );
    finish()
}

#[no_mangle]
pub extern "C" fn nas_main() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(97));
    let mode = support::word(112);
    if mode > RESUME {
        fail(98);
    }
    let recover = matches!(mode, COLD | REPAIR);
    match (support::word(120), mode) {
        (0, INTERRUPT) => admin_interrupt(),
        (3, INTERRUPT) => guest_interrupt(),
        (0, OFFLINE | RESUME) => admin_resume(mode == RESUME),
        (3, OFFLINE | RESUME) => guest_resume(mode == RESUME),
        (0, _) => admin(recover),
        (1, _) => server(mode),
        (3, _) => guest(recover),
        _ => fail(99),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    fail(100)
}
