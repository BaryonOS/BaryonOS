//! This complete host executable (including std startup) has one bounded BSS
//! allocator and no System/malloc fallback. Lazy bootstrap belongs only to this
//! test; production initialization remains explicit.
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};
use nieuw_os_heap_allocator::{Arena, UserAllocator};

const ARENA_BYTES: usize = 8 * 1024 * 1024;

#[repr(align(4096))]
struct Storage(UnsafeCell<[MaybeUninit<u8>; ARENA_BYTES]>);
// SAFETY: only the single successful bootstrap transfers this memory to HEAP.
unsafe impl Sync for Storage {}
static MEMORY: Storage = Storage(UnsafeCell::new([MaybeUninit::uninit(); ARENA_BYTES]));

struct Bootstrap {
    phase: AtomicU8,
    heap: UserAllocator,
}

impl Bootstrap {
    fn ready(&self) -> bool {
        match self
            .phase
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                let arena = Arena::checked(MEMORY.0.get().cast::<u8>(), ARENA_BYTES);
                let initialized = arena.is_ok_and(|arena| {
                    // SAFETY: unique initializer, exclusive static BSS storage
                    // disjoint from allocator state; no other access exists.
                    unsafe { self.heap.initialize_raw(arena).is_ok() }
                });
                self.phase
                    .store(if initialized { 2 } else { 3 }, Ordering::Release);
                initialized
            }
            Err(_) => loop {
                match self.phase.load(Ordering::Acquire) {
                    1 => core::hint::spin_loop(),
                    value => break value == 2,
                }
            },
        }
    }
}

// SAFETY: forwards exactly to the bounded owned allocator once initialized;
// failure returns null, never invoking the host system allocator.
unsafe impl GlobalAlloc for Bootstrap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.ready() {
            unsafe { self.heap.alloc(layout) }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: a live pointer from alloc implies successful initialization.
        unsafe { self.heap.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static HEAP: Bootstrap = Bootstrap {
    phase: AtomicU8::new(0),
    heap: UserAllocator::new(),
};

fn inside_arena(pointer: *const u8, bytes: usize) {
    let start = MEMORY.0.get() as usize;
    let pointer = pointer as usize;
    assert!(pointer >= start);
    assert!(pointer.checked_add(bytes).unwrap() <= start + ARENA_BYTES);
}

#[repr(align(4096))]
struct Aligned([u8; 3072]);

fn main() {
    let mut numbers = Vec::new();
    numbers.try_reserve_exact(20_000).unwrap();
    numbers.extend((0u32..20_000).map(|value| value * 3));
    inside_arena(
        numbers.as_ptr().cast(),
        numbers.capacity() * size_of::<u32>(),
    );
    assert_eq!(numbers[19_999], 59_997);

    let mut text = String::new();
    text.try_reserve(1024).unwrap();
    for _ in 0..30 {
        text.push_str("Materiaal bewaart mijn werk. ");
    }
    inside_arena(text.as_ptr(), text.capacity());
    assert!(text.ends_with("mijn werk. "));

    let boxed = Box::new(Aligned([0x47; 3072]));
    let reused_address = (&*boxed as *const Aligned) as usize;
    inside_arena((&*boxed as *const Aligned).cast(), size_of::<Aligned>());
    assert_eq!(reused_address % 4096, 0);
    assert!(boxed.0.iter().all(|value| *value == 0x47));
    drop(boxed);
    let reused = Box::new(Aligned([0x28; 3072]));
    assert_eq!((&*reused as *const Aligned) as usize, reused_address);
    assert!(reused.0.iter().all(|value| *value == 0x28));

    let mut large = Vec::<u8>::new();
    large.try_reserve_exact(1024 * 1024).unwrap();
    large.resize(1024 * 1024, 0x63);
    inside_arena(large.as_ptr(), large.capacity());
    assert!(large.iter().all(|value| *value == 0x63));

    let before = HEAP.heap.usage();
    let original_capacity = numbers.capacity();
    let original_text_capacity = text.capacity();
    assert!(numbers.try_reserve_exact(ARENA_BYTES * 2).is_err());
    assert!(text.try_reserve_exact(ARENA_BYTES * 2).is_err());
    assert_eq!(numbers.len(), 20_000);
    assert_eq!(numbers[19_999], 59_997);
    assert_eq!(numbers.capacity(), original_capacity);
    assert_eq!(text.capacity(), original_text_capacity);
    assert!(text.ends_with("mijn werk. "));
    let after = HEAP.heap.usage();
    assert_eq!(after.payload_used, before.payload_used);
    assert_eq!(after.live_allocations, before.live_allocations);
    assert_eq!(after.failed_allocations, before.failed_allocations + 2);

    // alloc handles zero-sized types itself without violating GlobalAlloc's
    // nonzero-size contract.
    let zero_sized = vec![(); 100_000];
    let boxed_zero_sized = Box::new(());
    assert_eq!(zero_sized.len(), 100_000);
    assert_eq!(HEAP.heap.usage().allocations, after.allocations);

    drop((large, numbers, text, reused, zero_sized, boxed_zero_sized));
    let end = HEAP.heap.usage();
    assert!(end.high_water_payload >= 1024 * 1024);
    assert!(end.engine_used < before.engine_used);
    assert_eq!(end.capacity, ARENA_BYTES);
    println!(
        "bounded real Vec/Box/String: passed; capacity={} highwater={} failed={}",
        end.capacity, end.high_water_engine, end.failed_allocations
    );
}
