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
//! Evidence (written before each reply): 40 status (1 done, 0x1xx error),
//! 144 fixture applied, 192 read ok, 200 denied, 208 close ok, 216 claimed
//! actor differed from sender, 224 bad requests, 256.. decision log.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("task-grant-broker is only for the Baryon OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{native, Handle, ReceiveEndpoint, SendEndpoint};
use nieuw_os_task_grant_broker_example::{
    decode_request, encode_reply, Broker, OP_CLOSE, OP_READ, STATUS_BAD_REQUEST, STATUS_DENIED,
    STATUS_GO, STATUS_OK, STATUS_RESUME, TAG,
};

const REQUESTS: u64 = 8;
const OWNER_FIRST_PHASE: u64 = 3;
const HELPER_PHASE: u64 = 2;
const LOG: u64 = 256;

fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
}

fn endpoint(offset: u64) -> Result<Handle, ()> {
    Handle::from_raw(support::word(offset)).map_err(|_| ())
}

fn run() -> Result<(), ()> {
    native::check_abi().map_err(|_| ())?;
    let owner = support::word(176);
    let helper = support::word(184);
    if support::word(32) == 0 || owner == 0 || helper == 0 || owner == helper {
        return Err(());
    }
    let requests = ReceiveEndpoint::from_handle(endpoint(56)?);
    let owner_reply = SendEndpoint::from_handle(endpoint(64)?);
    let helper_reply = SendEndpoint::from_handle(endpoint(72)?);

    let mut broker = Broker::new();
    if support::word(96) == 1 {
        broker
            .trusted_grant(
                support::word(104),
                support::word(112),
                support::word(120),
                support::word(128),
                support::word(136),
            )
            .map_err(|_| ())?;
        support::put(144, 1);
    }

    let (mut served, mut from_owner, mut from_helper) = (0u64, 0u64, 0u64);
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
        let (status, data, op, object) = match request {
            Some(request) => {
                if request.claimed_actor != sender {
                    bump(216);
                }
                let (status, data) = broker.handle(sender, request);
                (status, data, request.op, request.object & 0xff)
            }
            None => (STATUS_BAD_REQUEST, &[][..], 0, 0),
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
        support::send(reply, TAG, &encode_reply(status, data))?;
        if sender == owner {
            from_owner += 1;
            if from_owner == OWNER_FIRST_PHASE {
                support::send(helper_reply, TAG, &encode_reply(STATUS_GO, &[]))?;
            }
        } else {
            from_helper += 1;
            if from_helper == HELPER_PHASE {
                support::send(owner_reply, TAG, &encode_reply(STATUS_RESUME, &[]))?;
            }
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
