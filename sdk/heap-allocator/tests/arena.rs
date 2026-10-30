use core::alloc::{GlobalAlloc, Layout};
use core::mem::MaybeUninit;
use nieuw_os_heap_allocator::{AllocError, Arena, InitError, UserAllocator};

struct Fixture {
    allocator: UserAllocator,
    backing: *mut u8,
    backing_bytes: usize,
    usable_start: usize,
    usable_end: usize,
}

impl Fixture {
    fn new(bytes: usize) -> Self {
        // The engine requires 'static storage. Leak only this test's arena;
        // host allocation is a fixture supplier, never an allocator fallback.
        let backing = Box::leak(vec![MaybeUninit::new(0xa5u8); bytes + 128].into_boxed_slice());
        let backing_bytes = backing.len();
        let backing = backing.as_mut_ptr().cast::<u8>();
        let start = unsafe { backing.add(33) };
        let arena = Arena::checked(start, bytes).unwrap();
        let usable_start = arena.start() as usize;
        let usable_end = usable_start + arena.usable_bytes();
        let allocator = UserAllocator::new();
        // SAFETY: this test transfers an exclusive leaked subextent; no other
        // accesses to the arena happen except through live returned blocks.
        unsafe { allocator.initialize_raw(arena).unwrap() };
        Self {
            allocator,
            backing,
            backing_bytes,
            usable_start,
            usable_end,
        }
    }

    fn check_guards(&self) {
        // SAFETY: read only disjoint bytes outside the normalized engine extent,
        // including alignment bytes that the engine must leave untouched.
        unsafe {
            let prefix = core::slice::from_raw_parts(
                self.backing,
                self.usable_start - self.backing as usize,
            );
            let suffix = core::slice::from_raw_parts(
                self.backing.add(self.usable_end - self.backing as usize),
                self.backing as usize + self.backing_bytes - self.usable_end,
            );
            assert!(prefix.iter().chain(suffix).all(|byte| *byte == 0xa5));
        }
    }

    fn check_block(&self, pointer: *mut u8, layout: Layout) {
        let start = pointer as usize;
        assert_eq!(start % layout.align(), 0);
        assert!(start >= self.usable_start);
        assert!(start.checked_add(layout.size()).unwrap() <= self.usable_end);
    }
}

#[test]
fn invalid_extent_shapes_do_not_dereference_or_initialize() {
    assert_eq!(
        Arena::checked(core::ptr::null_mut(), 4096).unwrap_err(),
        InitError::Null
    );
    assert_eq!(
        Arena::checked(core::ptr::dangling_mut::<u8>(), usize::MAX).unwrap_err(),
        InitError::ExtentOverflow
    );
    assert_eq!(
        Arena::checked((usize::MAX - 15) as *mut u8, 32).unwrap_err(),
        InitError::ExtentOverflow
    );
    assert_eq!(
        Arena::checked(core::ptr::dangling_mut::<u8>(), 15).unwrap_err(),
        InitError::TooSmall
    );
    let allocator = UserAllocator::new();
    let memory = Box::leak(vec![MaybeUninit::new(0x5a); 7].into_boxed_slice());
    let pointer = memory.as_ptr();
    assert_eq!(allocator.initialize(memory), Err(InitError::TooSmall));
    // SAFETY: initialization failed before taking engine ownership or writing;
    // this test kept a raw observation pointer to the initialized fixture bytes.
    assert!(
        unsafe { core::slice::from_raw_parts(pointer.cast::<u8>(), 7) }
            .iter()
            .all(|byte| *byte == 0x5a)
    );
    assert!(!allocator.usage().initialized);
}

#[test]
fn missing_init_and_malformed_requests_are_visible() {
    let allocator = UserAllocator::new();
    let layout = Layout::new::<u64>();
    assert_eq!(
        allocator.try_allocate(layout),
        Err(AllocError::NotInitialized)
    );
    // SAFETY: nonzero valid GlobalAlloc Layout, which must return null here.
    assert!(unsafe { allocator.alloc(layout) }.is_null());
    assert_eq!(
        allocator.try_allocate_bytes(1, 0),
        Err(AllocError::InvalidLayout)
    );
    assert_eq!(
        allocator.try_allocate_bytes(1, 3),
        Err(AllocError::InvalidLayout)
    );
    assert_eq!(
        allocator.try_allocate_bytes(usize::MAX, 8),
        Err(AllocError::InvalidLayout)
    );
    assert_eq!(allocator.usage().failed_allocations, 5);
}

#[test]
fn explicit_init_aligns_extent_and_reinitialization_preserves_live_data() {
    let fixture = Fixture::new(4099);
    let layout = Layout::from_size_align(300, 64).unwrap();
    let pointer = fixture.allocator.try_allocate(layout).unwrap();
    unsafe { pointer.as_ptr().write_bytes(0x73, layout.size()) };
    let second = Box::leak(vec![MaybeUninit::new(0x6b); 256].into_boxed_slice());
    let second_pointer = second.as_mut_ptr().cast::<u8>();
    assert_eq!(
        fixture.allocator.initialize(second),
        Err(InitError::AlreadyInitialized)
    );
    assert!(unsafe { core::slice::from_raw_parts(second_pointer, 256) }
        .iter()
        .all(|byte| *byte == 0x6b));
    assert!(
        unsafe { core::slice::from_raw_parts(pointer.as_ptr(), layout.size()) }
            .iter()
            .all(|byte| *byte == 0x73)
    );
    let usage = fixture.allocator.usage();
    assert_eq!(usage.provided_bytes, 4099);
    assert_eq!(usage.capacity, fixture.usable_end - fixture.usable_start);
    assert!(usage.capacity <= usage.provided_bytes);
    unsafe { fixture.allocator.deallocate(pointer, layout) };
    fixture.check_guards();
}

#[test]
fn minimum_arena_and_zero_size_have_defined_behavior() {
    #[repr(align(16))]
    struct Minimum([MaybeUninit<u8>; 2 * core::mem::size_of::<usize>()]);
    let memory = Box::leak(Box::new(Minimum(
        [MaybeUninit::uninit(); 2 * core::mem::size_of::<usize>()],
    )));
    let allocator = UserAllocator::new();
    allocator.initialize(&mut memory.0).unwrap();
    assert_eq!(
        allocator.try_allocate_bytes(0, 8),
        Err(AllocError::ZeroSize)
    );
    let layout = Layout::new::<u8>();
    let pointer = allocator.try_allocate(layout).unwrap();
    assert_eq!(
        allocator.usage().engine_used,
        2 * core::mem::size_of::<usize>()
    );
    assert_eq!(allocator.try_allocate(layout), Err(AllocError::OutOfMemory));
    unsafe { allocator.deallocate(pointer, layout) };
    assert_eq!(allocator.usage().engine_free, allocator.usage().capacity);
}

#[test]
fn multiple_alignments_and_extents_stay_disjoint() {
    let fixture = Fixture::new(128 * 1024);
    let mut live = Vec::new();
    for alignment in [1, 2, 4, 8, 16, 64, 256, 4096] {
        let layout = Layout::from_size_align(513, alignment).unwrap();
        let pointer = fixture.allocator.try_allocate(layout).unwrap();
        fixture.check_block(pointer.as_ptr(), layout);
        let range = pointer.as_ptr() as usize..pointer.as_ptr() as usize + layout.size();
        for (other, other_layout) in &live {
            let other: &core::ptr::NonNull<u8> = other;
            let other_layout: &Layout = other_layout;
            let start = other.as_ptr() as usize;
            assert!(range.end <= start || range.start >= start + other_layout.size());
        }
        unsafe { pointer.as_ptr().write_bytes(alignment as u8, layout.size()) };
        live.push((pointer, layout));
    }
    for (pointer, layout) in live.into_iter().rev() {
        assert!(
            unsafe { core::slice::from_raw_parts(pointer.as_ptr(), layout.size()) }
                .iter()
                .all(|byte| *byte == layout.align() as u8)
        );
        unsafe { fixture.allocator.deallocate(pointer, layout) };
    }
    assert_eq!(
        fixture.allocator.usage().engine_free,
        fixture.allocator.usage().capacity
    );
    fixture.check_guards();
}

#[test]
fn full_arena_failure_preserves_confirmed_allocations() {
    let fixture = Fixture::new(4096);
    let layout = Layout::from_size_align(512, 8).unwrap();
    let mut live = Vec::new();
    while let Ok(pointer) = fixture.allocator.try_allocate(layout) {
        unsafe { pointer.as_ptr().write_bytes(0x91, layout.size()) };
        live.push(pointer);
    }
    assert!(!live.is_empty());
    let before = fixture.allocator.usage();
    assert_eq!(
        fixture.allocator.try_allocate_bytes(8192, 8),
        Err(AllocError::OutOfMemory)
    );
    let after = fixture.allocator.usage();
    assert_eq!(before.payload_used, after.payload_used);
    assert_eq!(before.engine_used, after.engine_used);
    assert_eq!(before.live_allocations, after.live_allocations);
    for pointer in live {
        assert!(
            unsafe { core::slice::from_raw_parts(pointer.as_ptr(), layout.size()) }
                .iter()
                .all(|byte| *byte == 0x91)
        );
        unsafe { fixture.allocator.deallocate(pointer, layout) };
    }
    assert_eq!(fixture.allocator.usage().live_allocations, 0);
    fixture.check_guards();
}

#[test]
fn fragmentation_is_visible_and_neighbor_freeing_coalesces() {
    let fixture = Fixture::new(8192);
    let layout = Layout::from_size_align(512, 8).unwrap();
    let mut slots = [None; 16];
    let mut count = 0;
    for slot in &mut slots {
        match fixture.allocator.try_allocate(layout) {
            Ok(pointer) => {
                *slot = Some(pointer);
                count += 1;
            }
            Err(AllocError::OutOfMemory) => break,
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(count >= 12);
    for slot in slots.iter_mut().take(count).step_by(2) {
        unsafe { fixture.allocator.deallocate(slot.take().unwrap(), layout) };
    }
    let large = Layout::from_size_align(1024, 8).unwrap();
    assert!(fixture.allocator.usage().engine_free >= large.size());
    assert_eq!(
        fixture.allocator.try_allocate(large),
        Err(AllocError::OutOfMemory)
    );
    // Free the block between the first and third freed holes.
    unsafe {
        fixture
            .allocator
            .deallocate(slots[1].take().unwrap(), layout)
    };
    let joined = fixture.allocator.try_allocate(large).unwrap();
    fixture.check_block(joined.as_ptr(), large);
    unsafe { fixture.allocator.deallocate(joined, large) };
    for pointer in slots.into_iter().flatten() {
        unsafe { fixture.allocator.deallocate(pointer, layout) };
    }
    let all = Layout::from_size_align(fixture.allocator.usage().capacity, 8).unwrap();
    let pointer = fixture.allocator.try_allocate(all).unwrap();
    unsafe { fixture.allocator.deallocate(pointer, all) };
    fixture.check_guards();
}

#[test]
fn large_allocation_and_reuse_report_padding_and_high_water() {
    let fixture = Fixture::new(2 * 1024 * 1024);
    let tiny = Layout::new::<u8>();
    let one = fixture.allocator.try_allocate(tiny).unwrap();
    let usage = fixture.allocator.usage();
    assert_eq!(usage.payload_used, 1);
    assert!(usage.engine_used > usage.payload_used);
    unsafe { fixture.allocator.deallocate(one, tiny) };
    let layout = Layout::from_size_align(1024 * 1024, 4096).unwrap();
    let large = fixture.allocator.try_allocate(layout).unwrap();
    fixture.check_block(large.as_ptr(), layout);
    unsafe { large.as_ptr().write_bytes(0x42, layout.size()) };
    let peak = fixture.allocator.usage();
    assert!(peak.high_water_engine >= layout.size());
    assert_eq!(peak.high_water_payload, layout.size());
    unsafe { fixture.allocator.deallocate(large, layout) };
    let again = fixture.allocator.try_allocate(layout).unwrap();
    assert_eq!(again, large);
    unsafe { fixture.allocator.deallocate(again, layout) };
    let end = fixture.allocator.usage();
    assert_eq!(end.engine_used, 0);
    assert_eq!(end.payload_used, 0);
    assert_eq!(end.allocations, 3);
    assert_eq!(end.deallocations, 3);
    assert_eq!(end.high_water_engine, peak.high_water_engine);
    fixture.check_guards();
}

#[test]
fn huge_valid_layout_fails_without_panic_or_partial_mutation() {
    let fixture = Fixture::new(4096);
    let huge_size = Layout::from_size_align(isize::MAX as usize - 7, 8).unwrap();
    let huge_alignment = Layout::from_size_align(1, 1usize << (usize::BITS - 2)).unwrap();
    let before = fixture.allocator.usage();
    for layout in [huge_size, huge_alignment] {
        assert_eq!(
            fixture.allocator.try_allocate(layout),
            Err(AllocError::OutOfMemory)
        );
    }
    let after = fixture.allocator.usage();
    assert_eq!(after.engine_free, before.engine_free);
    assert_eq!(after.live_allocations, 0);
    assert_eq!(after.failed_allocations, 2);
    fixture.check_guards();
}

#[test]
fn default_global_zeroed_and_realloc_preserve_contract_on_failure() {
    let fixture = Fixture::new(8192);
    let layout = Layout::from_size_align(300, 32).unwrap();
    // SAFETY: nonzero valid layouts, pointers kept live until the matching call.
    unsafe {
        let pointer = fixture.allocator.alloc_zeroed(layout);
        assert!(!pointer.is_null());
        assert!(core::slice::from_raw_parts(pointer, layout.size())
            .iter()
            .all(|byte| *byte == 0));
        pointer.write_bytes(0x67, layout.size());
        assert!(fixture.allocator.realloc(pointer, layout, 16384).is_null());
        assert!(core::slice::from_raw_parts(pointer, layout.size())
            .iter()
            .all(|byte| *byte == 0x67));
        let grown = fixture.allocator.realloc(pointer, layout, 1024);
        assert!(!grown.is_null());
        assert!(core::slice::from_raw_parts(grown, layout.size())
            .iter()
            .all(|byte| *byte == 0x67));
        let grown_layout = Layout::from_size_align(1024, 32).unwrap();
        let shrunk = fixture.allocator.realloc(grown, grown_layout, 100);
        assert!(!shrunk.is_null());
        assert!(core::slice::from_raw_parts(shrunk, 100)
            .iter()
            .all(|byte| *byte == 0x67));
        fixture
            .allocator
            .dealloc(shrunk, Layout::from_size_align(100, 32).unwrap());
    }
    assert_eq!(fixture.allocator.usage().live_allocations, 0);
    fixture.check_guards();
}

#[test]
fn own_arena_metrics_do_not_include_other_allocator_activity() {
    let first = Fixture::new(4096);
    let second = Fixture::new(4096);
    let snapshot = second.allocator.usage();
    let layout = Layout::from_size_align(500, 8).unwrap();
    let pointer = first.allocator.try_allocate(layout).unwrap();
    assert_eq!(second.allocator.usage(), snapshot);
    unsafe { first.allocator.deallocate(pointer, layout) };
    assert_eq!(second.allocator.usage(), snapshot);
}

#[test]
fn lock_serializes_real_allocations_and_stats_across_host_threads() {
    let fixture = Fixture::new(128 * 1024);
    std::thread::scope(|scope| {
        for value in 1u8..=4 {
            let allocator = &fixture.allocator;
            scope.spawn(move || {
                for _ in 0..200 {
                    let layout = Layout::from_size_align(203, 64).unwrap();
                    let pointer = allocator.try_allocate(layout).unwrap();
                    unsafe {
                        pointer.as_ptr().write_bytes(value, layout.size());
                        std::thread::yield_now();
                        assert!(core::slice::from_raw_parts(pointer.as_ptr(), layout.size())
                            .iter()
                            .all(|byte| *byte == value));
                        allocator.deallocate(pointer, layout);
                    }
                }
            });
        }
    });
    let usage = fixture.allocator.usage();
    assert_eq!(usage.allocations, 800);
    assert_eq!(usage.deallocations, 800);
    assert_eq!(usage.live_allocations, 0);
    assert_eq!(usage.engine_free, usage.capacity);
    fixture.check_guards();
}

#[test]
fn deterministic_mixed_allocations_preserve_live_bytes_extents_and_accounting() {
    // Adapted from the independent reviewer's counterexample search. This is
    // deterministic test data, not entropy or a cryptographic random source.
    let fixture = Fixture::new(256 * 1024);
    let mut seed = 0x9384_ef28_a134_ac42u64;
    let mut live = Vec::<(core::ptr::NonNull<u8>, Layout, u8)>::new();
    for step in 0..40_000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        if !live.is_empty() && seed & 3 == 0 {
            let index = seed as usize % live.len();
            let (pointer, layout, value) = live.swap_remove(index);
            assert!(
                unsafe { core::slice::from_raw_parts(pointer.as_ptr(), layout.size()) }
                    .iter()
                    .all(|byte| *byte == value)
            );
            unsafe { fixture.allocator.deallocate(pointer, layout) };
        } else {
            let layout =
                Layout::from_size_align(seed as usize % 2048 + 1, 1usize << ((seed >> 20) % 13))
                    .unwrap();
            match fixture.allocator.try_allocate(layout) {
                Ok(pointer) => {
                    fixture.check_block(pointer.as_ptr(), layout);
                    let start = pointer.as_ptr() as usize;
                    for (other, other_layout, _) in &live {
                        let other = other.as_ptr() as usize;
                        assert!(
                            start + layout.size() <= other || other + other_layout.size() <= start
                        );
                    }
                    let value = (seed >> 40) as u8;
                    unsafe { pointer.as_ptr().write_bytes(value, layout.size()) };
                    live.push((pointer, layout, value));
                }
                Err(AllocError::OutOfMemory) => {}
                other => panic!("unexpected allocation result: {other:?}"),
            }
        }
        if step % 1000 == 0 {
            fixture.check_guards();
            let usage = fixture.allocator.usage();
            assert_eq!(usage.live_allocations, live.len());
            assert_eq!(
                usage.payload_used,
                live.iter().map(|(_, layout, _)| layout.size()).sum()
            );
        }
    }
    for (pointer, layout, value) in live {
        assert!(
            unsafe { core::slice::from_raw_parts(pointer.as_ptr(), layout.size()) }
                .iter()
                .all(|byte| *byte == value)
        );
        unsafe { fixture.allocator.deallocate(pointer, layout) };
    }
    let usage = fixture.allocator.usage();
    assert_eq!(usage.engine_free, usage.capacity);
    assert_eq!(usage.payload_used, 0);
    assert_eq!(usage.live_allocations, 0);
    fixture.check_guards();
}
