#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! A03 broker client. One ELF, two processes: role 1 is the owner app, role 2
//! is a hostile helper that claims the owner's identity in every request,
//! including a forged RETIRE and a read after the owner has exited.
//!
//! Startup words (kernel bootstrap): 0 role, 56 reply receive handle, 64
//! request send handle, 176 broker PID, 184 owner PID (the claimed actor).
//! Local diagnostics: 192.. one reply status per step, 320/328 the bytes of
//! the last successful read. The kernel checks them against the broker log.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("task-grant-client is only for the Baryon OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{native, Handle, ReceiveEndpoint, SendEndpoint};
use nieuw_os_task_grant_broker_example::{
    decode_reply, encode_request, Request, DATA_MAX, OBJECT_ONE, OBJECT_TWO, OP_CLOSE, OP_READ,
    OP_RETIRE, OP_WRITE, STATUS_GO, STATUS_OK, STATUS_RESUME, TAG,
};

const OWNER: u64 = 1;
const HELPER: u64 = 2;

struct Client {
    requests: SendEndpoint,
    replies: ReceiveEndpoint,
    broker: u64,
    claimed_actor: u64,
    step: u64,
}

// The reply copy below splits the data into two 8-byte words.
const _: () = assert!(DATA_MAX == 16);

impl Client {
    /// Accepts only a well-formed reply whose kernel-supplied sender is the broker.
    fn reply(&mut self) -> Result<u64, ()> {
        let message = support::receive(self.replies)?;
        if message.cap_count() != 0 || message.sender() != self.broker || message.tag() != TAG {
            let _ = native::close_received_caps(&message);
            return Err(());
        }
        let (status, data, length) = decode_reply(message.payload()).ok_or(())?;
        if status == STATUS_OK && length != 0 {
            let mut words = [0u8; DATA_MAX];
            words[..length].copy_from_slice(&data[..length]);
            let mut first = [0; 8];
            let mut second = [0; 8];
            first.copy_from_slice(&words[..8]);
            second.copy_from_slice(&words[8..]);
            support::put(320, u64::from_le_bytes(first));
            support::put(328, u64::from_le_bytes(second));
        }
        support::put(192 + self.step * 8, status);
        self.step += 1;
        Ok(status)
    }

    fn call(&mut self, op: u64, object: u64) -> Result<u64, ()> {
        let request = Request {
            op,
            object,
            claimed_actor: self.claimed_actor,
        };
        support::send(self.requests, TAG, &encode_request(request))?;
        self.reply()
    }

    fn wait(&mut self, signal: u64) -> Result<(), ()> {
        if self.reply()? == signal {
            Ok(())
        } else {
            Err(())
        }
    }
}

fn run() -> Result<(), ()> {
    native::check_abi().map_err(|_| ())?;
    let role = support::word(0);
    let mut client = Client {
        requests: SendEndpoint::from_handle(Handle::from_raw(support::word(64)).map_err(|_| ())?),
        replies: ReceiveEndpoint::from_handle(Handle::from_raw(support::word(56)).map_err(|_| ())?),
        broker: support::word(176),
        claimed_actor: support::word(184),
        step: 0,
    };
    if client.broker == 0 || client.claimed_actor == 0 {
        return Err(());
    }
    // Outcomes are recorded, not judged here: the kernel compares them with
    // the broker's own log for the selected profile.
    match role {
        OWNER => {
            client.call(OP_READ, OBJECT_ONE)?;
            client.call(OP_WRITE, OBJECT_ONE)?;
            client.call(OP_READ, OBJECT_TWO)?;
            client.wait(STATUS_RESUME)?;
            client.call(OP_READ, OBJECT_ONE)?;
            client.call(OP_CLOSE, OBJECT_ONE)?;
            client.call(OP_READ, OBJECT_ONE)?;
            // Second fixture grant, then a read after the helper's forged
            // RETIRE; exiting afterwards triggers the kernel exit notice.
            client.call(OP_READ, OBJECT_ONE)?;
            client.wait(STATUS_RESUME)?;
            client.call(OP_READ, OBJECT_ONE)?;
        }
        HELPER => {
            client.wait(STATUS_GO)?;
            client.call(OP_READ, OBJECT_ONE)?;
            client.call(OP_CLOSE, OBJECT_ONE)?;
            client.wait(STATUS_GO)?;
            client.call(OP_RETIRE, OBJECT_ONE)?;
            // Released only after the broker retired the exited owner.
            client.wait(STATUS_GO)?;
            client.call(OP_READ, OBJECT_ONE)?;
        }
        _ => return Err(()),
    }
    Ok(())
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    match run() {
        Ok(()) => native::exit(0),
        Err(_) => native::exit(1),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    native::exit(2)
}
