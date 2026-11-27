#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! A03 ring-3 grant broker. Every request is authorized by the kernel-supplied
//! sender PID; the claimed actor field and object numbers never authenticate.
//!
//! Startup words (kernel bootstrap): 32 epoch, 56 request receive handle,
//! 64/72 reply send handles for the owner/helper, 176/184 owner/helper PID.
//! FIXTURE standing in for the trusted permission dialog: 96 present flag,
//! 104 PID, 112 object, 120 revision, 128 rights bits, 136 duration code.
//! FIXTURE: a second grant decision of the same form at 360..400, applied
//! after the owner's close.
//! A03-T2c-3 exit detection (owner option A, reply-endpoint closure): each
//! reply endpoint is owned by its client, so the kernel's capability lifecycle
//! revokes it no later than the client's exit. The broker probes the reply
//! handles with a non-consuming `ipc_wait` SEND condition and retires every
//! grant of a client whose handle reports Revoked or Closed (`client_gone`).
//! Detection is polled; no kernel-written notice and no new ABI.
//! Evidence (written before each reply): 40 status (1 done, 0x1xx error),
//! 144 fixtures applied, 152 client exits detected, 160 owner grants after the
//! forged retire, 192 read ok, 200 denied, 208 close ok, 216 claimed actor
//! differed from sender, 224 bad requests, 232/240/248 owner grants before
//! retirement, grants retired, owner grants after, 256.. decision log,
//! 440 retirements caused by a failed reply send, 448 kernel code that
//! revealed the exit (two's complement), 456 PID retired on that code, 464
//! probe results that showed a live client.
//! A03-T2c-2 volume mode (word 80 non-zero): 80 volume ingress send handle,
//! 88 volume reply receive handle, 408 volume service PID; the broker is the
//! sole domain-0 client. Evidence: 48 set after the startup STATs, then the
//! broker waits for the kernel to set 416; 424 volume calls during startup,
//! 432 volume calls in total, 520.. volume calls caused by each served request.
//! Both modes take the boot epoch from word 32 (the volume service reads the same word).

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("task-grant-broker is only for the Baryon OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{
    native, Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, SendEndpoint, WaitCondition,
};
use nieuw_os_task_grant_broker_example::{
    client_gone, decode_request, encode_reply, Broker, FixtureVolume, DATA_MAX, OP_CLOSE, OP_READ,
    STATUS_BAD_REQUEST, STATUS_DENIED, STATUS_GO, STATUS_OK, STATUS_RESUME, TAG,
};
use volume_core::{client::Transport, wire, Status};

const REQUESTS: u64 = 12;
const LOG: u64 = 256;
const SEND_RETIRED: u64 = 440;
const EXIT_CODE: u64 = 448;
const EXIT_PID: u64 = 456;
const LIVE_PROBES: u64 = 464;
const VOLUME_CALLS: u64 = 432;
const VOLUME_LOG: u64 = 520;
const LIMIT: usize = 400_000;

/// A03-T2c-2: volume domain-0 transport over kernel IPC (settings-volume
/// pattern). Replies count only from the volume service PID; nothing is resent.
struct NativeVolume {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    service: u64,
}

impl Transport for NativeVolume {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        bump(VOLUME_CALLS);
        let message = Outgoing::new(wire::TAG, request).map_err(|_| Status::Protocol)?;
        let mut sent = false;
        for _ in 0..LIMIT {
            support::tick();
            match native::send(self.send, &message) {
                Ok(()) => {
                    sent = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Protocol),
            }
        }
        if !sent {
            return Err(Status::Timeout);
        }
        for _ in 0..LIMIT {
            support::tick();
            match native::receive(self.receive) {
                Ok(reply) => {
                    let caps = reply.cap_count() != 0;
                    if caps {
                        let _ = native::close_received_caps(&reply);
                    }
                    if caps
                        || reply.sender() != self.service
                        || reply.tag() != wire::TAG
                        || reply.payload().len() != wire::SIZE
                    {
                        return Err(Status::Unknown);
                    }
                    let mut body = [0; wire::SIZE];
                    body.copy_from_slice(reply.payload());
                    return Ok(body);
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Unknown),
            }
        }
        Err(Status::Unknown)
    }
}

fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
}

fn endpoint(offset: u64) -> Result<Handle, ()> {
    Handle::from_raw(support::word(offset)).map_err(|_| ())
}

/// FIXTURE standing in for the trusted dialog: a kernel-written decision of
/// (present, PID, object, revision, rights, duration) words at `base`.
fn apply_fixture<T: Transport>(broker: &mut Broker<T>, base: u64) -> Result<(), ()> {
    if support::word(base) != 1 {
        return Ok(());
    }
    broker
        .trusted_grant(
            support::word(base + 8),
            support::word(base + 16),
            support::word(base + 24),
            support::word(base + 32),
            support::word(base + 40),
        )
        .map_err(|_| ())?;
    bump(144);
    Ok(())
}

/// Kernel error code of a failed runtime call, if it is one.
fn code(error: RuntimeError) -> Option<i64> {
    match error {
        RuntimeError::Kernel(code) => Some(code),
        _ => None,
    }
}

/// Probe the clients' reply endpoints until one reports its authority gone.
/// A zero deadline makes `ipc_wait` an immediate, non-consuming readiness
/// query: a live client (ready or full) sees no message, and nothing is
/// reserved. Returns the departed client's PID and the code that showed it.
fn wait_client_gone(clients: [(u64, SendEndpoint); 2]) -> Result<(u64, i64), ()> {
    for _ in 0..100_000 {
        support::tick();
        for (pid, reply) in clients {
            match native::ipc_wait(&[WaitCondition::send(reply)], 0) {
                // Ready or full (mask 0): the client is alive.
                Ok(_) => bump(LIVE_PROBES),
                Err(error) => {
                    match code(error).and_then(|c| client_gone(c).map(|gone| (c, gone))) {
                        Some((c, true)) => return Ok((pid, c)),
                        Some((_, false)) => {}
                        None => return Err(()),
                    }
                }
            }
        }
        // Sleep one PIT tick between rounds instead of spinning (a SEND
        // condition on a live client is always ready, so it cannot block).
        let now = native::monotonic_ticks().map_err(|_| ())?;
        native::ipc_wait(&[], now + 1).map_err(|_| ())?;
    }
    Err(())
}

/// Send to a client; a reply endpoint that reports the client gone retires
/// that client's grants (fail closed) instead of aborting the broker.
fn deliver<T: Transport>(
    broker: &mut Broker<T>,
    pid: u64,
    reply: SendEndpoint,
    payload: &[u8],
) -> Result<(), ()> {
    let message = Outgoing::new(TAG, payload).map_err(|_| ())?;
    for _ in 0..100_000 {
        support::tick();
        match native::send(reply, &message) {
            Ok(()) => return Ok(()),
            Err(error) if error.is_again() => native::yield_now().map_err(|_| ())?,
            Err(error) => {
                return match code(error).and_then(client_gone) {
                    Some(true) => {
                        broker.retire_process(pid).map_err(|_| ())?;
                        bump(SEND_RETIRED);
                        Ok(())
                    }
                    _ => Err(()),
                }
            }
        }
    }
    Err(())
}

fn run() -> Result<(), ()> {
    native::check_abi().map_err(|_| ())?;
    let owner = support::word(176);
    let helper = support::word(184);
    if support::word(32) == 0 || owner == 0 || helper == 0 || owner == helper {
        return Err(());
    }
    // Both modes use the kernel-supplied boot epoch, never the host-test constant.
    let epoch = support::word(32);
    if support::word(80) == 0 {
        // FIXTURE volume for `native-task-grant`; T2c-2 uses the real domain 0.
        return serve(Broker::new(FixtureVolume { epoch }, epoch), owner, helper);
    }
    let service = support::word(408);
    if service == 0 {
        return Err(());
    }
    let volume = NativeVolume {
        send: SendEndpoint::from_handle(endpoint(80)?),
        receive: ReceiveEndpoint::from_handle(endpoint(88)?),
        service,
    };
    let broker = Broker::new(volume, epoch);
    support::put(424, support::word(VOLUME_CALLS));
    support::put(48, 1);
    // The kernel snapshots the volume's own counters before any request.
    for _ in 0..100_000 {
        if support::word(416) == 1 {
            return serve(broker, owner, helper);
        }
        support::tick();
        native::yield_now().map_err(|_| ())?;
    }
    Err(())
}

fn serve<T: Transport>(mut broker: Broker<T>, owner: u64, helper: u64) -> Result<(), ()> {
    let requests = ReceiveEndpoint::from_handle(endpoint(56)?);
    let owner_reply = SendEndpoint::from_handle(endpoint(64)?);
    let helper_reply = SendEndpoint::from_handle(endpoint(72)?);
    apply_fixture(&mut broker, 96)?;

    let mut served = 0u64;
    while served < REQUESTS {
        let message = support::receive(requests)?;
        // Reply routing uses the kernel-supplied sender only.
        let sender = message.sender();
        let reply = if sender == owner {
            owner_reply
        } else if sender == helper {
            helper_reply
        } else {
            let _ = native::close_received_caps(&message);
            bump(224);
            continue;
        };
        let request = if message.cap_count() == 0 && message.tag() == TAG {
            decode_request(message.payload())
        } else {
            let _ = native::close_received_caps(&message);
            None
        };
        let (status, data, length, op, object) = match request {
            Some(request) => {
                if request.claimed_actor != sender {
                    bump(216);
                }
                let calls = support::word(VOLUME_CALLS);
                let (status, data, length) = broker.handle(sender, request);
                support::put(VOLUME_LOG + served * 8, support::word(VOLUME_CALLS) - calls);
                (status, data, length, request.op, request.object & 0xff)
            }
            None => (STATUS_BAD_REQUEST, [0; DATA_MAX], 0, 0, 0),
        };
        match (status, op) {
            (STATUS_OK, OP_READ) => bump(192),
            (STATUS_OK, OP_CLOSE) => bump(208),
            (STATUS_DENIED, _) => bump(200),
            (STATUS_BAD_REQUEST, _) => bump(224),
            _ => {}
        }
        support::put(
            LOG + served * 8,
            (sender << 32) | (op << 24) | (object << 16) | status,
        );
        served += 1;
        deliver(
            &mut broker,
            sender,
            reply,
            &encode_reply(status, &data[..length]),
        )?;
        // Fixed script; the kernel checks the resulting log order exactly.
        match served {
            3 | 9 => deliver(
                &mut broker,
                helper,
                helper_reply,
                &encode_reply(STATUS_GO, &[]),
            )?,
            5 => deliver(
                &mut broker,
                owner,
                owner_reply,
                &encode_reply(STATUS_RESUME, &[]),
            )?,
            8 => apply_fixture(&mut broker, 360)?,
            10 => {
                // After the helper's forged RETIRE: the owner's grant must survive.
                support::put(160, broker.live_grants(owner));
                deliver(
                    &mut broker,
                    owner,
                    owner_reply,
                    &encode_reply(STATUS_RESUME, &[]),
                )?;
            }
            11 => {
                // Both clients are probed, the still-live helper first, so the
                // live branch runs natively; the script expects the owner to end.
                let (exited, code) =
                    wait_client_gone([(helper, helper_reply), (owner, owner_reply)])?;
                if exited != owner {
                    return Err(());
                }
                support::put(EXIT_CODE, code as u64);
                support::put(EXIT_PID, exited);
                support::put(232, broker.live_grants(exited));
                support::put(240, broker.retire_process(exited).map_err(|_| ())?);
                support::put(248, broker.live_grants(exited));
                support::put(152, 1);
                deliver(
                    &mut broker,
                    helper,
                    helper_reply,
                    &encode_reply(STATUS_GO, &[]),
                )?;
            }
            _ => {}
        }
    }
    support::put(40, 1);
    Ok(())
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if run().is_err() {
        support::put(40, 0x101);
    }
    loop {
        support::tick();
        let _ = native::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    loop {
        let _ = native::yield_now();
    }
}
