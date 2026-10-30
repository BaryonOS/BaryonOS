#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod common;
use common::*;
use nieuw_os_display_client::{InputClient, InputSource, NativeTransport, ResetProgress};
use nieuw_os_sdk_runtime::native;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}
#[inline(never)]
fn main() -> ! {
    boot();
    let source = handle(SOURCE);
    if word(GENERATION) == 2 {
        denied(0x82, [word(OLD_SOURCE), 0, 0, 0, 0, 0], -4, OLD_DENIED, 10);
    }
    denied(0x82, [word(WRONG_TYPE), 0, 0, 0, 0, 0], -6, CAP_DENIED, 12);
    output_denied(0x80, word(PEER_DISPLAY), -6, CAP_DENIED, 14);
    cap_restrictions(source, 8192);
    // A newly granted input source cannot inherit the old owner's Ready.
    output_denied(0x84, source.raw(), -5, CAP_DENIED, 16);
    let mut client = InputClient::new(NativeTransport, InputSource::from_handle(source));
    let ticket = client.begin_reset().unwrap_or_else(|_| fail(40));
    bump(RESETS);
    put(TICKET, ticket);
    check(!client.is_ready(), 41);
    denied(0x82, [source.raw(), 0, 0, 0, 0, 0], -8, CAP_DENIED, 42);
    let mut out = [SENTINEL; 32];
    denied(
        0x83,
        [source.raw(), ticket + 1, out.as_mut_ptr() as u64, 32, 0, 0],
        -1,
        CAP_DENIED,
        43,
    );
    check(out == [SENTINEL; 32], 44);
    bump(UNCHANGED);
    denied(
        0x83,
        [source.raw(), ticket, BAD_POINTER, 32, 0, 0],
        -3,
        POINTER_DENIED,
        45,
    );
    prefix_unchanged();
    let mut attempts = 0u64;
    let report = loop {
        attempts += 1;
        check(attempts <= 8192, 46);
        match client.poll_reset().unwrap_or_else(|_| fail(47)) {
            ResetProgress::Ready(report) => break report,
            ResetProgress::Pending => {
                bump(PENDING);
                check(!client.is_ready(), 48);
                check(native::yield_now().is_ok(), 49);
            }
        }
    };
    check(client.is_ready() && report.ticket == ticket, 50);
    put(EPOCH, report.epoch);
    put(RESET_OPERATIONS, u64::from(report.operations));
    put(RESET_DISCARDED, u64::from(report.discarded));
    bump(READY);
    // Empty Ready queue: kernel EAGAIN preserves all output bytes.
    output_denied(0x84, source.raw(), -8, CAP_DENIED, 51);
    put(STAGE, 1);
    for (index, expected) in INPUT_BYTES.into_iter().enumerate() {
        let mut attempts = 0u64;
        let sample = loop {
            attempts += 1;
            check(attempts <= 2_000_000, 53);
            // This happens before EVERY valid read, including a queued byte.
            // If any failed pointer consumes FIFO/advances seq, comparison fails.
            denied(
                0x84,
                [source.raw(), BAD_POINTER, 32, 0, 0, 0],
                -3,
                POINTER_DENIED,
                54,
            );
            prefix_unchanged();
            match client.read().unwrap_or_else(|_| fail(55)) {
                Some(sample) => break sample,
                None => {
                    check(native::yield_now().is_ok(), 56);
                }
            }
        };
        check(
            sample.epoch == report.epoch && sample.sequence == index as u64 + 1,
            57,
        );
        check(
            sample.byte == expected && sample.is_mouse() == (index >= 2) && !sample.needs_reset(),
            58 + index as u64,
        );
        put(
            LOG + index as u64 * 8,
            (u64::from(sample.status) << 8) | u64::from(sample.byte),
        );
        put(LOG_SEQUENCE + index as u64 * 8, sample.sequence);
        bump(SAMPLES);
    }
    put(STAGE, 2);
    wait();
    check(native::close(source).is_ok(), 70);
    output_denied(0x84, source.raw(), -4, CLOSED, 71);
    finish()
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}
