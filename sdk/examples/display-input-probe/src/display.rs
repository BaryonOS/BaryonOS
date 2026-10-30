#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod common;
use common::*;
use nieuw_os_display_client::{
    wire::{Blit, Rectangle},
    DisplayClient, DisplaySource, NativeTransport,
};
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
        output_denied(0x80, word(OLD_SOURCE), -4, OLD_DENIED, 10);
    }
    output_denied(0x80, word(WRONG_TYPE), -6, CAP_DENIED, 12);
    // No INPUT grant exists in this PID, including the stolen peer integer.
    denied(0x82, [word(PEER_INPUT), 0, 0, 0, 0, 0], -4, CAP_DENIED, 14);
    cap_restrictions(source, 4096);
    let mut client = DisplayClient::new(NativeTransport, DisplaySource::from_handle(source));
    let geometry = client.geometry().unwrap_or_else(|_| fail(40));
    put(WIDTH, u64::from(geometry.width));
    put(HEIGHT, u64::from(geometry.height));
    put(EPOCH, geometry.epoch);
    let (x, y) = tile_position(word(GENERATION));
    let rect = Rectangle {
        x,
        y,
        width: TILE_WIDTH as u16,
        height: TILE_HEIGHT as u16,
    };
    let mut pixels = [0u8; TILE_BYTES];
    for row in 0..TILE_HEIGHT {
        for col in 0..TILE_WIDTH {
            let at = (row * TILE_WIDTH + col) * 4;
            pixels[at..at + 4].copy_from_slice(&pixel(word(GENERATION), col, row).to_le_bytes());
        }
    }
    client.blit(rect, &pixels).unwrap_or_else(|_| fail(41));
    bump(BLITS);
    let good = Blit {
        epoch: geometry.epoch,
        rectangle: rect,
    }
    .encode()
    .unwrap_or_else(|_| fail(42));
    let mut malformed = good;
    malformed[..8].copy_from_slice(&(geometry.epoch + 1).to_le_bytes());
    denied(
        0x81,
        [
            source.raw(),
            malformed.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        43,
    );
    malformed = good;
    malformed[8..12].copy_from_slice(&(geometry.width - 1).to_le_bytes());
    denied(
        0x81,
        [
            source.raw(),
            malformed.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        44,
    );
    malformed = good;
    malformed[20] = 1;
    denied(
        0x81,
        [
            source.raw(),
            malformed.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        45,
    );
    malformed = good;
    malformed[16..18].copy_from_slice(&33u16.to_le_bytes());
    denied(
        0x81,
        [
            source.raw(),
            malformed.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        46,
    );
    // Last alpha byte invalid: no row may be stored before full validation.
    let rejected = [0x00ee_eeeeu32.to_le_bytes(); TILE_WIDTH * TILE_HEIGHT];
    for (destination, value) in pixels.chunks_exact_mut(4).zip(rejected) {
        destination.copy_from_slice(&value);
    }
    pixels[TILE_BYTES - 1] = 1;
    denied(
        0x81,
        [
            source.raw(),
            good.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        47,
    );
    denied(
        0x81,
        [
            source.raw(),
            good.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64 - 4,
            0,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        48,
    );
    denied(
        0x81,
        [
            source.raw(),
            BAD_POINTER,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -3,
        DISPLAY_DENIED,
        49,
    );
    denied(
        0x81,
        [
            source.raw(),
            good.as_ptr() as u64,
            PARTIAL_PIXELS,
            TILE_BYTES as u64,
            0,
            0,
        ],
        -3,
        DISPLAY_DENIED,
        50,
    );
    denied(
        0x81,
        [
            source.raw(),
            good.as_ptr() as u64,
            pixels.as_ptr() as u64,
            TILE_BYTES as u64,
            1,
            0,
        ],
        -1,
        DISPLAY_DENIED,
        51,
    );
    denied(
        0x80,
        [source.raw(), BAD_POINTER, 32, 0, 0, 0],
        -3,
        DISPLAY_DENIED,
        52,
    );
    prefix_unchanged();
    let mut out = [SENTINEL; 32];
    denied(
        0x80,
        [source.raw(), out.as_mut_ptr() as u64, 31, 0, 0, 0],
        -1,
        DISPLAY_DENIED,
        53,
    );
    check(out == [SENTINEL; 32], 54);
    bump(UNCHANGED);
    put(STAGE, 1);
    wait();
    check(native::close(source).is_ok(), 55);
    output_denied(0x80, source.raw(), -4, CLOSED, 56);
    finish()
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}
