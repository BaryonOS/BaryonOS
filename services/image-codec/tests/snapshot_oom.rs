//! Host-only allocation fault injection through the actual scalar JPEG engine.
//! The allocator uses System for storage; only selected snapshot reservations
//! fail. It never substitutes a decoder, pixel result, or native allocator.
#![deny(unsafe_op_in_unsafe_fn)]
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_jpeg::{errors::DecodeErrors, JpegDecoder};

#[derive(Clone, Copy)]
struct Control {
    trace: bool,
    sizes: [usize; 256],
    calls: usize,
    watch_size: usize,
    matched: usize,
    fail_match: usize,
    pointers: [usize; 128],
    live: usize,
}
impl Control {
    const EMPTY: Self = Self {
        trace: false,
        sizes: [0; 256],
        calls: 0,
        watch_size: 0,
        matched: 0,
        fail_match: 0,
        pointers: [0; 128],
        live: 0,
    };
}
thread_local! { static CONTROL: Cell<Control> = const { Cell::new(Control::EMPTY) }; }
struct FaultAllocator;
#[global_allocator]
static ALLOCATOR: FaultAllocator = FaultAllocator;
fn requested(size: usize) -> bool {
    CONTROL
        .try_with(|control| {
            let mut c = control.get();
            if c.trace {
                if c.calls < c.sizes.len() {
                    c.sizes[c.calls] = size;
                }
                c.calls += 1;
            }
            let mut fail = false;
            if c.watch_size != 0 && size == c.watch_size {
                c.matched += 1;
                fail = c.fail_match != 0 && c.matched == c.fail_match;
            }
            control.set(c);
            fail
        })
        .unwrap_or(false)
}
fn allocated(pointer: *mut u8, size: usize) {
    if pointer.is_null() {
        return;
    }
    let _ = CONTROL.try_with(|control| {
        let mut c = control.get();
        if c.watch_size != 0 && size == c.watch_size {
            let slot = c
                .pointers
                .iter()
                .position(|x| *x == 0)
                .expect("bounded test pointer log");
            c.pointers[slot] = pointer as usize;
            c.live += 1;
        }
        control.set(c);
    });
}
fn freed(pointer: *mut u8) {
    let _ = CONTROL.try_with(|control| {
        let mut c = control.get();
        if let Some(slot) = c
            .pointers
            .iter()
            .position(|x| *x != 0 && *x == pointer as usize)
        {
            c.pointers[slot] = 0;
            c.live -= 1;
        }
        control.set(c);
    });
}
// SAFETY: All successful operations delegate the original layout/pointer to
// System. A selected allocation returns null, as GlobalAlloc permits. Bookkeeping
// is bounded thread-local plain data and cannot allocate or share a test gate.
unsafe impl GlobalAlloc for FaultAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if requested(layout.size()) {
            return std::ptr::null_mut();
        }
        // SAFETY: GlobalAlloc caller supplies a valid layout.
        let result = unsafe { System.alloc(layout) };
        allocated(result, layout.size());
        result
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if requested(layout.size()) {
            return std::ptr::null_mut();
        }
        // SAFETY: GlobalAlloc caller supplies a valid layout.
        let result = unsafe { System.alloc_zeroed(layout) };
        allocated(result, layout.size());
        result
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        freed(pointer);
        // SAFETY: System produced this allocation with this layout.
        unsafe {
            System.dealloc(pointer, layout);
        }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if requested(size) {
            return std::ptr::null_mut();
        }
        // SAFETY: System produced pointer/layout; caller supplies valid new size.
        let result = unsafe { System.realloc(pointer, layout, size) };
        if !result.is_null() {
            freed(pointer);
            allocated(result, size);
        }
        result
    }
}
fn state() -> Control {
    CONTROL.with(Cell::get)
}
fn reset(watch_size: usize, fail_match: usize, trace: bool) {
    assert_eq!(state().live, 0);
    CONTROL.with(|x| {
        x.set(Control {
            watch_size,
            fail_match,
            trace,
            ..Control::EMPTY
        })
    });
}
fn disarm() {
    CONTROL.with(|x| {
        let mut c = x.get();
        c.fail_match = 0;
        c.trace = false;
        x.set(c);
    });
}
fn decoder(input: &[u8]) -> JpegDecoder<ZCursor<&[u8]>> {
    let mut d = JpegDecoder::new_with_options(
        ZCursor::new(input),
        DecoderOptions::new_safe().jpeg_set_out_colorspace(ColorSpace::RGBA),
    );
    d.set_incremental_mode(true);
    d
}
const BASELINE: &[u8] = include_bytes!("../fixtures/baseline.jpg");
const PROGRESSIVE: &[u8] = include_bytes!("../fixtures/progressive.jpg");
const SCANS: &[u8] =
    include_bytes!("../fixtures/scan-rollback/baseline-three-sos-table-changes.jpg");

fn profile(input: &[u8]) -> (usize, Vec<u8>) {
    reset(0, 0, true);
    let mut d = decoder(input);
    d.decode_headers().unwrap();
    disarm();
    let seen = state();
    assert!(seen.calls >= 3 && seen.calls < seen.sizes.len());
    // Actual successful headers end with the two equal owned table buffers and
    // their small scan-state owner. Observe, rather than recreate, table layout.
    let slot = seen.sizes[seen.calls - 3];
    assert_eq!(slot, seen.sizes[seen.calls - 2]);
    assert!(slot > 4096 && seen.sizes[seen.calls - 1] < slot);
    let mut output = vec![0; d.output_buffer_size().unwrap()];
    d.decode_into(&mut output).unwrap();
    drop(d);
    reset(0, 0, false);
    (slot, output)
}

#[test]
fn each_header_reservation_fails_without_publishing_headers_and_can_retry() {
    for input in [BASELINE, PROGRESSIVE, SCANS] {
        let (size, expected) = profile(input);
        for fail in 1..=2 {
            let mut d = decoder(input);
            reset(size, fail, false);
            let result = d.decode_headers();
            disarm();
            assert!(matches!(result, Err(DecodeErrors::AllocationFailed)));
            assert_eq!(state().matched, fail);
            assert_eq!(state().live, 0);
            assert!(d.info().is_none());
            assert!(d.output_buffer_size().is_none());
            d.decode_headers().unwrap();
            let mut output = vec![0; d.output_buffer_size().unwrap()];
            d.decode_into(&mut output).unwrap();
            assert_eq!(output, expected);
            drop(d);
            assert_eq!(state().live, 0);
            reset(0, 0, false);
        }
    }
}

#[test]
fn each_replay_or_checkpoint_reservation_fails_then_same_decoder_replays_exactly() {
    for input in [BASELINE, PROGRESSIVE, SCANS] {
        let (size, expected) = profile(input);
        let mut d = decoder(input);
        d.decode_headers().unwrap();
        let mut output = vec![0; d.output_buffer_size().unwrap()];
        reset(size, 0, false);
        d.decode_into(&mut output).unwrap();
        let count = state().matched;
        drop(d);
        assert_eq!(state().live, 0);
        reset(0, 0, false);
        assert!((2..128).contains(&count));
        if input == PROGRESSIVE || input == SCANS {
            assert!(count > 2);
        }
        println!(
            "HOST_SNAPSHOT_RESERVATIONS bytes={size} progressive={} failpoints={count}",
            input == PROGRESSIVE
        );
        for fail in 1..=count {
            let mut d = decoder(input);
            d.decode_headers().unwrap();
            let mut output = vec![0; d.output_buffer_size().unwrap()];
            reset(size, fail, false);
            let result = d.decode_into(&mut output);
            disarm();
            assert!(
                matches!(result, Err(DecodeErrors::AllocationFailed)),
                "failure {fail}/{count}: {result:?}"
            );
            assert_eq!(state().matched, fail);
            d.decode_into(&mut output).unwrap();
            assert_eq!(output, expected, "retry {fail}/{count}");
            drop(d);
            assert_eq!(state().live, 0);
            reset(0, 0, false);
        }
    }
}

#[test]
fn public_wrapper_reports_memory_error_and_next_decode_is_usable() {
    use nieuw_os_image_codec::{decode, Cancellation, Error, Limits};
    let limits = Limits {
        input_bytes: 1024 * 1024,
        width: 4096,
        height: 4096,
        pixels: 1_000_000,
        work_bytes: 4 * 1024 * 1024,
        metadata_bytes: 65536,
        png_chunks: 1024,
        jpeg_scans: 100,
    };
    for input in [BASELINE, PROGRESSIVE, SCANS] {
        let (size, _) = profile(input);
        // Header capture is reservations1/2; the initial decode replay plan is
        // reservations3/4, so both public error-mapping branches are exercised.
        for fail in 1..=4 {
            let cancel = Cancellation::default();
            reset(size, fail, false);
            let result = decode(input, limits, &cancel);
            disarm();
            assert!(matches!(result, Err(Error::OutOfMemory)));
            assert_eq!(state().matched, fail);
            assert_eq!(state().live, 0);
            let image = decode(input, limits, &cancel).unwrap();
            drop(image);
            assert_eq!(state().live, 0);
            reset(0, 0, false);
        }
    }
}

#[test]
fn inter_scan_table_changes_suspend_at_eof_then_retry_without_poisoning() {
    use std::rc::Rc;
    struct Growing {
        bytes: &'static [u8],
        available: Rc<Cell<usize>>,
    }
    impl AsRef<[u8]> for Growing {
        fn as_ref(&self) -> &[u8] {
            &self.bytes[..self.available.get()]
        }
    }
    let (_, expected) = profile(SCANS);
    // These prefixes stop after some/all DQT, DHT, DRI, COM and APP changes,
    // before the next SOS. The bytes are independently accepted by libjpeg.
    for cut in [211, 233, 239, 270, 279, 295] {
        let available = Rc::new(Cell::new(cut));
        let input = Growing {
            bytes: SCANS,
            available: available.clone(),
        };
        let mut d = JpegDecoder::new_with_options(
            ZCursor::new(input),
            DecoderOptions::new_safe().jpeg_set_out_colorspace(ColorSpace::RGBA),
        );
        d.set_incremental_mode(true);
        d.decode_headers().unwrap();
        let before = d.input_colorspace();
        let mut output = vec![0; d.output_buffer_size().unwrap()];
        let error = d.decode_into(&mut output).unwrap_err();
        assert!(error.is_recoverable_eof(), "cut {cut}: {error:?}");
        assert_eq!(d.input_colorspace(), before);
        available.set(SCANS.len());
        d.decode_into(&mut output).unwrap();
        assert_eq!(output, expected, "cut {cut}");
    }
}

#[test]
fn inter_scan_cancellation_retries_every_observed_poll_with_identical_pixels() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let (_, expected) = profile(SCANS);
    let polls = Arc::new(AtomicUsize::new(0));
    let observe = polls.clone();
    let mut d = decoder(SCANS);
    d.decode_headers().unwrap();
    d.set_cancel(move || {
        observe.fetch_add(1, Ordering::Relaxed);
        false
    });
    d.set_cancel_interval(1);
    let mut output = vec![0; d.output_buffer_size().unwrap()];
    d.decode_into(&mut output).unwrap();
    let count = polls.load(Ordering::Relaxed);
    assert!((2..256).contains(&count));
    println!("HOST_INTERSCAN_CANCEL_POLLS={count}");
    for at in 0..count {
        let polls = Arc::new(AtomicUsize::new(0));
        let observe = polls.clone();
        let mut d = decoder(SCANS);
        d.decode_headers().unwrap();
        d.set_cancel_interval(1);
        d.set_cancel(move || observe.fetch_add(1, Ordering::Relaxed) == at);
        let mut output = vec![0; d.output_buffer_size().unwrap()];
        assert!(
            matches!(d.decode_into(&mut output), Err(DecodeErrors::Cancelled)),
            "poll {at}"
        );
        d.set_cancel(|| false);
        d.decode_into(&mut output).unwrap();
        assert_eq!(output, expected, "poll {at}");
    }
}
