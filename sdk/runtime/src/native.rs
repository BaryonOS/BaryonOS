//! Native-only syscall boundary. This module is unavailable on the host, so a
//! host test cannot accidentally issue Linux int80 syscalls with the OS ABI.
use core::arch::asm;
use zeroize::Zeroizing;

use crate::{
    block::{self, BlockDevice, READ_MAX, SECTOR_SIZE},
    decode_return,
    entropy::{EntropySource, ENTROPY_MAX},
    network::{NetworkSource, NET_FRAME_MAX},
    process::{spawn_cap_words, ChildProcess, LaunchImage, ProcessStatus},
    validate_abi, zero_return, Capability, Error, Handle, Outgoing, ReceiveEndpoint, Received,
    Rights, SendEndpoint, MESSAGE_SIZE,
};

fn call(number: u64, args: [u64; 6]) -> Result<u64, Error> {
    let mut primary = number;
    let mut secondary = args[2];
    // SAFETY: Nieuw OS ABI 1.0–1.10 preserves every GPR except RAX/RDX; int80 enters
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

/// ABI 1.6: monotone PIT interrupt count since this boot, nominally 100 Hz.
/// Available only with the explicit clock feature; older profiles return ENOSYS.
/// This is not a wall clock and cannot validate TLS certificate dates.
pub fn monotonic_ticks() -> Result<u64, Error> {
    call(3, [0; 6])
}

/// ABI 1.9 opt-in profile: return a current UTC Unix timestamp read from CMOS.
/// The result is explicitly unauthenticated and must not validate TLS
/// certificates, authorize expiry, or replace a trusted time provider.
pub fn wall_clock_utc() -> Result<crate::clock::WallClockReading, Error> {
    let raw = call(0x73, [0; 6])?;
    Ok(crate::clock::WallClockReading::unauthenticated_rtc(
        raw as i64,
    ))
}

/// ABI1.7: atomically grow this process's private, zeroed RW/NX heap suffix.
/// Physical exhaustion returns Kernel(memory::OUT_OF_MEMORY); quota/metadata
/// exhaustion returns Kernel(-7). Normal errors allocate nothing. InvalidReturn
/// signals a kernel ABI violation: do not automatically retry an uncertain call.
/// Older profiles return ENOSYS. No shrink, delegation or implicit allocator.
pub fn heap_grow(pages: u64) -> Result<crate::memory::OwnedHeapPages, Error> {
    crate::memory::decode_growth(call(4, [pages, 0, 0, 0, 0, 0])?, pages)
}

pub fn yield_now() -> Result<(), Error> {
    zero_return(call(1, [0; 6]))
}

/// ABI 1.10: wait for authorized IPC queue conditions or an absolute PIT tick.
/// The kernel copies at most sixteen scalar conditions before scheduling; no
/// pointer is dereferenced on wake. Ready bits reserve/consume nothing. Zero
/// means the deadline elapsed without a ready condition. Empty conditions are
/// a timed sleep, not a raw device/IRQ wait. The future horizon is at most
/// IPC_WAIT_HORIZON_TICKS; callers retain their own protocol deadlines.
pub fn ipc_wait(conditions: &[crate::WaitCondition], deadline: u64) -> Result<u16, Error> {
    let bytes = crate::encode_wait(conditions)?;
    let pointer = if conditions.is_empty() {
        0
    } else {
        bytes.as_ptr() as u64
    };
    let mask = call(0x22, [pointer, conditions.len() as u64, deadline, 0, 0, 0])?;
    crate::decode_wait_mask(mask, conditions.len())
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
    let mut bytes = Zeroizing::new([0; MESSAGE_SIZE]);
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
    match Received::from_bytes_mut(&mut bytes, length) {
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

/// Submit a READ of `length` (1..=128) bytes at `offset` within the scoped
/// `sector`. Returns the broker ticket; data arrives through `block_wait`.
pub fn block_read(
    device: BlockDevice,
    sector: u64,
    offset: u64,
    length: u64,
) -> Result<u64, Error> {
    call(0x40, [device.handle().raw(), sector, offset, length, 0, 0])
}

/// Submit one sector WRITE. The kernel copies all 512 bytes from this live
/// buffer before returning; completion does not imply durability (FLUSH).
pub fn block_write(
    device: BlockDevice,
    sector: u64,
    data: &[u8; SECTOR_SIZE],
) -> Result<u64, Error> {
    call(
        0x42,
        [
            device.handle().raw(),
            sector,
            data.as_ptr() as u64,
            SECTOR_SIZE as u64,
            0,
            0,
        ],
    )
}

pub fn block_flush(device: BlockDevice) -> Result<u64, Error> {
    call(0x43, [device.handle().raw(), 0, 0, 0, 0, 0])
}

/// Poll one ticket until completion, yielding while the broker reports
/// EAGAIN, for at most `limit` polls. Copies READ data into `out`. A kernel
/// error is returned unchanged; the caller decides whether it is uncertain.
pub fn block_wait(
    device: BlockDevice,
    ticket: u64,
    out: &mut [u8],
    limit: usize,
) -> Result<(), Error> {
    if out.len() > READ_MAX {
        return Err(Error::MessageSize);
    }
    for _ in 0..limit {
        let mut bytes = [0; MESSAGE_SIZE];
        match call(
            0x41,
            [
                device.handle().raw(),
                ticket,
                bytes.as_mut_ptr() as u64,
                MESSAGE_SIZE as u64,
                0,
                0,
            ],
        ) {
            Ok(returned) => {
                let data = block::completion(&bytes, returned, ticket, out.len())?;
                out.copy_from_slice(data);
                return Ok(());
            }
            Err(error) if error.is_again() => yield_now()?,
            Err(error) => return Err(error),
        }
    }
    Err(Error::Kernel(-13))
}

/// K05 (ABI 1.3): create an endpoint this process owns. The returned root
/// handle carries all endpoint rights; the kernel enforces the quota of four
/// owned endpoints per process and 64 in total.
pub fn endpoint_create() -> Result<Handle, Error> {
    Handle::from_raw(call(0x30, [0; 6])?)
}

/// K05 (ABI 1.3): start the image named by `image`, passing `arg` as startup
/// word 0 and at most two reduced capabilities (never RECV). Success returns
/// the only handle to the child's `Process` object.
pub fn spawn(image: LaunchImage, arg: u64, caps: &[Capability]) -> Result<ChildProcess, Error> {
    if caps.len() > 2 {
        return Err(Error::MessageSize);
    }
    let (h0, r0) = spawn_cap_words(caps.first().copied())?;
    let (h1, r1) = spawn_cap_words(caps.get(1).copied())?;
    Handle::from_raw(call(0x50, [image.handle().raw(), arg, h0, r0, h1, r1])?)
        .map(ChildProcess::from_handle)
}

/// K05 (ABI 1.3): end a running child. The kernel reclaims its memory,
/// closes its endpoints and revokes what it delegated before returning.
pub fn kill(child: ChildProcess) -> Result<(), Error> {
    zero_return(call(0x51, [child.handle().raw(), 0, 0, 0, 0, 0]))
}

/// K05 (ABI 1.3): read the lifecycle state of a child.
pub fn status(child: ChildProcess) -> Result<ProcessStatus, Error> {
    ProcessStatus::decode(call(0x52, [child.handle().raw(), 0, 0, 0, 0, 0])?)
}

/// B01 (ABI 1.4): ask the kernel entropy broker for `length` (1..=256) raw
/// device bytes. Returns the broker ticket; the bytes arrive through
/// `entropy_wait`. The kernel serializes requests: a second outstanding
/// request fails with EAGAIN.
pub fn entropy_request(source: EntropySource, length: usize) -> Result<u64, Error> {
    if length == 0 || length > ENTROPY_MAX {
        return Err(Error::MessageSize);
    }
    call(0x60, [source.handle().raw(), length as u64, 0, 0, 0, 0])
}

/// B01: poll one entropy ticket until the device answered, yielding on
/// EAGAIN, for at most `limit` polls. `out.len()` must equal the requested
/// length; the kernel writes exactly that many bytes or returns an error and
/// leaves `out` untouched. A short or empty device answer is an error, never
/// padded.
pub fn entropy_wait(
    source: EntropySource,
    ticket: u64,
    out: &mut [u8],
    limit: usize,
) -> Result<(), Error> {
    if out.is_empty() || out.len() > ENTROPY_MAX {
        return Err(Error::MessageSize);
    }
    for _ in 0..limit {
        match call(
            0x61,
            [
                source.handle().raw(),
                ticket,
                out.as_mut_ptr() as u64,
                out.len() as u64,
                0,
                0,
            ],
        ) {
            Ok(written) if written == out.len() as u64 => return Ok(()),
            Ok(_) => return Err(Error::InvalidReturn),
            Err(error) if error.is_again() => yield_now()?,
            Err(error) => return Err(error),
        }
    }
    Err(Error::Kernel(-13))
}

/// Publish one raw Ethernet frame. Success is a ticket, not device completion
/// or peer delivery. The timeout is 1..=500 monotone 100 Hz ticks; never replay
/// a failed/uncertain transmission automatically.
pub fn net_send(source: NetworkSource, frame: &[u8], timeout_ticks: u64) -> Result<u64, Error> {
    if !(14..=NET_FRAME_MAX).contains(&frame.len()) {
        return Err(Error::MessageSize);
    }
    if !(1..=500).contains(&timeout_ticks) {
        return Err(Error::Kernel(-1));
    }
    let ticket = call(
        0x70,
        [
            source.handle().raw(),
            frame.as_ptr() as u64,
            frame.len() as u64,
            timeout_ticks,
            0,
            0,
        ],
    )?;
    if ticket == 0 {
        Err(Error::InvalidReturn)
    } else {
        Ok(ticket)
    }
}

/// Poll a previously published TX ticket. EAGAIN leaves it pending; completion
/// confirms only device processing, never acknowledgement by a remote peer.
pub fn net_poll_tx(source: NetworkSource, ticket: u64) -> Result<usize, Error> {
    let count = call(0x71, [source.handle().raw(), ticket, 0, 0, 0, 0])?;
    if !(14..=NET_FRAME_MAX as u64).contains(&count) {
        Err(Error::InvalidReturn)
    } else {
        Ok(count as usize)
    }
}

/// Receive at most one complete raw frame. EAGAIN means no frame is queued.
/// The kernel validates the complete writable output before consumption or
/// copying; errors leave it untouched. Caller supplies its own bounded wait.
pub fn net_receive(source: NetworkSource, out: &mut [u8]) -> Result<usize, Error> {
    if !(14..=NET_FRAME_MAX).contains(&out.len()) {
        return Err(Error::MessageSize);
    }
    let count = call(
        0x72,
        [
            source.handle().raw(),
            out.as_mut_ptr() as u64,
            out.len() as u64,
            0,
            0,
            0,
        ],
    )?;
    if count < 14 || count > out.len() as u64 {
        Err(Error::InvalidReturn)
    } else {
        Ok(count as usize)
    }
}

/// ABI1.8: public geometry only; a Handle value never grants display authority.
pub fn display_info(source: Handle, out: &mut [u8; 32]) -> Result<u64, Error> {
    let count = call(0x80, [source.raw(), out.as_mut_ptr() as u64, 32, 0, 0, 0])?;
    if count == 32 {
        Ok(count)
    } else {
        Err(Error::InvalidReturn)
    }
}

/// Synchronous bounded tile transfer, without a vsync/presentation guarantee.
pub fn display_blit(source: Handle, descriptor: &[u8; 32], pixels: &[u8]) -> Result<u64, Error> {
    if pixels.is_empty() || pixels.len() > 1024 || !pixels.len().is_multiple_of(4) {
        return Err(Error::MessageSize);
    }
    let count = call(
        0x81,
        [
            source.raw(),
            descriptor.as_ptr() as u64,
            pixels.as_ptr() as u64,
            pixels.len() as u64,
            0,
            0,
        ],
    )?;
    if count == pixels.len() as u64 / 4 {
        Ok(count)
    } else {
        Err(Error::InvalidReturn)
    }
}

/// Start the fixed polled PS/2 initialization. A ticket is not Ready.
pub fn input_reset_begin(source: Handle) -> Result<u64, Error> {
    let ticket = call(0x82, [source.raw(), 0, 0, 0, 0, 0])?;
    if ticket != 0 {
        Ok(ticket)
    } else {
        Err(Error::InvalidReturn)
    }
}

/// One bounded reset step. EAGAIN leaves the output unchanged and is not Ready.
pub fn input_reset_poll(source: Handle, ticket: u64, out: &mut [u8; 32]) -> Result<u64, Error> {
    let count = call(
        0x83,
        [source.raw(), ticket, out.as_mut_ptr() as u64, 32, 0, 0],
    )?;
    if count == 32 {
        Ok(count)
    } else {
        Err(Error::InvalidReturn)
    }
}

/// One raw status/data sample. Only the isolated source service owns this cap.
pub fn input_read(source: Handle, out: &mut [u8; 32]) -> Result<u64, Error> {
    let count = call(0x84, [source.raw(), out.as_mut_ptr() as u64, 32, 0, 0, 0])?;
    if count == 32 {
        Ok(count)
    } else {
        Err(Error::InvalidReturn)
    }
}
