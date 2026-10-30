#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
extern crate alloc;

#[path = "../../system/src/support.rs"]
mod support;

use core::{arch::global_asm, panic::PanicInfo};
use nieuw_os_heap_allocator::{Arena, UserAllocator};
use nieuw_os_image_codec::{
    decode, pixel_hash, Cancellation, Error, Format, Image, Limits, Pixels,
};
use nieuw_os_sdk_runtime::{memory, native};

#[global_allocator]
static ALLOCATOR: UserAllocator = UserAllocator::new();

// The existing loader supplies SysV entry RSP%16=8. JMP preserves it for the
// first extern-C Rust function; an extra CALL here would misalign that frame.
global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    jmp image_main
"#
);

const DONE: u64 = 40;
const FAILURE: u64 = 240;
const PNG: u64 = 400;
const JPEG: u64 = 408;
const PIXELS: u64 = 416;
const NEGATIVES: u64 = 424;
const HASH: u64 = 432;
const BYTES: u64 = 440;
const GROWTH: u64 = 448;
const HIGH_WATER: u64 = 456;
const LIVE: u64 = 464;
const CANCELS: u64 = 472;
const START: u64 = 480;
const END: u64 = 488;
const QUALITY_OPEN: u64 = 496;
const PAGES: u64 = 1024;

fn fail(step: u64) -> ! {
    support::put(FAILURE, step);
    native::exit(1)
}
fn check(value: bool, step: u64) {
    if !value {
        fail(step)
    }
}
fn bump(field: u64, value: u64) {
    support::put(
        field,
        support::word(field)
            .checked_add(value)
            .unwrap_or_else(|| fail(90)),
    );
}
fn limits() -> Limits {
    Limits {
        input_bytes: 1024 * 1024,
        width: 4096,
        height: 4096,
        pixels: 1_000_000,
        work_bytes: 4 * 1024 * 1024,
        metadata_bytes: 65536,
        png_chunks: 1024,
        jpeg_scans: 100,
    }
}

struct Fixture {
    bytes: &'static [u8],
    width: usize,
    height: usize,
    hash: u64,
}
macro_rules! fixture {
    ($name:literal, $w:literal, $h:literal, $hash:literal) => {
        Fixture {
            bytes: include_bytes!(concat!("../../../../services/image-codec/fixtures/", $name)),
            width: $w,
            height: $h,
            hash: $hash,
        }
    };
}
const FIXTURES: [Fixture; 10] = [
    fixture!("rgb.png", 231, 149, 0x9e835dc6cac66b1d),
    fixture!("alpha.png", 128, 96, 0x62a32f96525fd225),
    fixture!("palette4.png", 73, 47, 0x6c049d0ec83edb44),
    fixture!("mono1.png", 65, 33, 0x37c419a317c8ebe0),
    fixture!("gray-alpha.png", 97, 65, 0x86d63f4b7cfd3255),
    fixture!("adam7.png", 129, 77, 0x62078a4803164208),
    fixture!("rgba16.png", 97, 65, 0x5902cbac329821a4),
    fixture!("baseline.jpg", 257, 129, 0xc6b9c5a1c94d9d26),
    fixture!("gray.jpg", 113, 67, 0x7a4e3b2178deb211),
    fixture!("progressive.jpg", 193, 113, 0x9759403be96e9522),
];

fn record(image: &Image<'_>, expected: &Fixture, step: u64) {
    check(
        (image.width, image.height) == (expected.width, expected.height),
        step,
    );
    let hash = pixel_hash(image);
    check(hash == expected.hash, step + 1);
    check(
        image.source.as_ptr() == expected.bytes.as_ptr() && image.source == expected.bytes,
        step + 2,
    );
    bump(
        if image.format == Format::Png {
            PNG
        } else {
            JPEG
        },
        1,
    );
    bump(PIXELS, (image.width * image.height) as u64);
    let bytes = match &image.pixels {
        Pixels::Rgba8(x) => x.len(),
        Pixels::Rgba16(x) => x.len() * 2,
    };
    bump(BYTES, bytes as u64);
    support::put(
        HASH,
        (support::word(HASH) ^ hash)
            .rotate_left(7)
            .wrapping_mul(0x9e3779b97f4a7c15),
    );
}

fn negative(input: &[u8], bounded: Limits, cancel: &Cancellation, expected: Error, step: u64) {
    match decode(input, bounded, cancel) {
        Err(error) if error == expected => bump(NEGATIVES, 1),
        _ => fail(step),
    }
}

fn negatives() {
    let cancel = Cancellation::default();
    let alpha = FIXTURES[1].bytes;
    negative(b"not an image", limits(), &cancel, Error::UnknownFormat, 50);
    negative(
        &alpha[..alpha.len() / 2],
        limits(),
        &cancel,
        Error::InvalidData,
        51,
    );
    negative(
        alpha,
        Limits {
            input_bytes: 8,
            ..limits()
        },
        &cancel,
        Error::InputLimit,
        52,
    );
    negative(
        alpha,
        Limits {
            work_bytes: 1000,
            ..limits()
        },
        &cancel,
        Error::WorkLimit,
        53,
    );
    let mut corrupt = alpha.to_vec();
    *corrupt.last_mut().unwrap_or_else(|| fail(54)) ^= 1;
    negative(&corrupt, limits(), &cancel, Error::InvalidData, 54);
    corrupt[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    negative(&corrupt, limits(), &cancel, Error::Dimensions, 55);
    negative(
        include_bytes!("../../../../services/image-codec/fixtures/icc-bomb.png"),
        limits(),
        &cancel,
        Error::MetadataLimit,
        56,
    );
    negative(
        include_bytes!("../../../../services/image-codec/fixtures/cmyk.jpg"),
        limits(),
        &cancel,
        Error::UnsupportedColor,
        57,
    );
    let mut apng = alpha.to_vec();
    let offset = apng
        .windows(4)
        .position(|x| x == b"IDAT")
        .unwrap_or_else(|| fail(58));
    apng[offset..offset + 4].copy_from_slice(b"fdAT");
    negative(&apng, limits(), &cancel, Error::UnsupportedAnimation, 58);
    cancel.cancel();
    negative(alpha, limits(), &cancel, Error::Cancelled, 59);
    bump(CANCELS, 1);
}

#[no_mangle]
extern "C" fn image_main() -> ! {
    // Startup word 0 is the boot mode. Query the running kernel's ABI through
    // the SDK instead of interpreting that unrelated startup value as a version.
    check(native::check_abi().is_ok(), 1);
    support::put(START, native::monotonic_ticks().unwrap_or_else(|_| fail(2)));
    let pages = native::heap_grow(PAGES).unwrap_or_else(|_| fail(3));
    check(
        pages.base_address() == memory::HEAP_BASE + memory::INITIAL_HEAP_BYTES
            && pages.page_count() == PAGES,
        4,
    );
    let arena =
        Arena::checked(pages.as_ptr(), pages.len_bytes() as usize).unwrap_or_else(|_| fail(5));
    // SAFETY: SDK returns a fresh, private RW/NX extent owned by this process.
    // No references to its contents exist; ownership transfers until process exit.
    unsafe {
        ALLOCATOR.initialize_raw(arena).unwrap_or_else(|_| fail(6));
    }
    support::put(GROWTH, pages.page_count());
    for fixture in &FIXTURES {
        {
            let cancel = Cancellation::default();
            let image = decode(fixture.bytes, limits(), &cancel).unwrap_or_else(|_| fail(10));
            record(&image, fixture, 11);
        }
        check(ALLOCATOR.usage().live_allocations == 0, 14);
        native::yield_now().unwrap_or_else(|_| fail(15));
    }
    negatives();
    check(ALLOCATOR.usage().live_allocations == 0, 60);
    {
        // A real successful decode after malformed/limited/cancelled requests.
        let image = decode(FIXTURES[1].bytes, limits(), &Cancellation::default())
            .unwrap_or_else(|_| fail(61));
        record(&image, &FIXTURES[1], 62);
    }
    let usage = ALLOCATOR.usage();
    check(
        usage.live_allocations == 0 && usage.failed_allocations == 0,
        65,
    );
    support::put(HIGH_WATER, usage.high_water_engine as u64);
    support::put(LIVE, usage.live_allocations as u64);
    // The specific progressive fixture now meets its unchanged max3 oracle
    // after the isolated h2v2 rounding correction. General image acceptance,
    // metadata transforms and untested variants remain separate open gates.
    support::put(QUALITY_OPEN, 0);
    support::put(END, native::monotonic_ticks().unwrap_or_else(|_| fail(66)));
    support::put(DONE, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    fail(99)
}
