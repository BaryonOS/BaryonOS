//! Native-only syscall boundary. This module is unavailable on the host, so a
//! host test cannot accidentally issue Linux int80 syscalls with the OS ABI.
use core::arch::asm;

use crate::{
    decode_return, validate_abi, zero_return, Error, Handle, Outgoing, ReceiveEndpoint, Received,
    Rights, SendEndpoint, MESSAGE_SIZE,
};

fn call(number: u64, args: [u64; 6]) -> Result<u64, Error> {
    let mut primary = number;
    let mut secondary = args[2];
    // SAFETY: Nieuw OS ABI 1.0–1.2 preserves every GPR except RAX/RDX; int80 enters
    // a checked kernel gate. No Rust references cross this boundary. Callers
    // below keep local message buffers live for the complete synchronous copy.
    // No nomem/readonly flag: IPC can read or write the supplied memory.
    unsafe {
        asm!("int 0x80", inout("rax") primary,
             in("rdi") args[0], in("rsi") args[1], inout("rdx") secondary,
             in("r10") args[3], in("r8") args[4], in("r9") args[5],
             options(nostack));
    }
    decode_return(primary as i64, secondary)
}

pub fn check_abi() -> Result<(), Error> {
    validate_abi(abi_info()?)
}

pub fn abi_info() -> Result<u64, Error> {
    call(0, [0; 6])
}

pub fn yield_now() -> Result<(), Error> {
    zero_return(call(1, [0; 6]))
}

pub fn exit(code: u64) -> ! {
    let _ = call(2, [code, 0, 0, 0, 0, 0]);
    // SAFETY: Exit may not return. An unexpected return terminates this user
    // process through the kernel's existing #UD boundary; never continue an app.
    unsafe {
        asm!("ud2", options(noreturn));
    }
}

pub fn close(handle: Handle) -> Result<(), Error> {
    zero_return(call(0x10, [handle.raw(), 0, 0, 0, 0, 0]))
}

pub fn derive(handle: Handle, reduced: Rights) -> Result<Handle, Error> {
    Handle::from_raw(call(0x11, [handle.raw(), reduced.bits(), 0, 0, 0, 0])?)
}

pub fn revoke(handle: Handle) -> Result<(), Error> {
    zero_return(call(0x12, [handle.raw(), 0, 0, 0, 0, 0]))
}

/// One nonblocking send. Success means queued, not acknowledged or durable.
/// No retry or inference about the peer's later behavior occurs here.
pub fn send(endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
    zero_return(call(
        0x20,
        [
            endpoint.handle().raw(),
            message.as_bytes().as_ptr() as u64,
            MESSAGE_SIZE as u64,
            0,
            0,
            0,
        ],
    ))
}

/// One nonblocking receive into a private stack buffer. The kernel validates
/// every output page and copies all bytes before returning to this process.
pub fn receive(endpoint: ReceiveEndpoint) -> Result<Received, Error> {
    let mut bytes = [0; MESSAGE_SIZE];
    let length = call(
        0x21,
        [
            endpoint.handle().raw(),
            bytes.as_mut_ptr() as u64,
            MESSAGE_SIZE as u64,
            0,
            0,
            0,
        ],
    )?;
    match Received::from_bytes(bytes, length) {
        Ok(message) => Ok(message),
        Err(error) => {
            // A successful kernel receive may have installed up to two caps.
            // Do not leak them if a future ABI mismatch rejects the envelope.
            // Read only the two bounded descriptor positions from kernel output.
            let count = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
            for i in 0..core::cmp::min(count as usize, 2) {
                if let Ok(handle) = Handle::from_raw(crate::message::get64(&bytes, 32 + i * 16)) {
                    let _ = close(handle);
                }
            }
            Err(error)
        }
    }
}

/// Explicitly discard delegated capabilities after protocol validation. The
/// original envelope is preserved for that validation and evidence. Attempts
/// every close, returning the first error. Call once per received message.
pub fn close_received_caps(message: &Received) -> Result<(), Error> {
    let mut first_error = None;
    for i in 0..message.cap_count() {
        if let Some(cap) = message.cap(i) {
            if let Err(error) = close(cap.handle) {
                first_error.get_or_insert(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}
