use core::alloc::Layout;
use core::mem::MaybeUninit;
use nieuw_os_heap_allocator::{AllocError, UserAllocator};

#[test]
fn failed_allocation_allows_one_owned_cleanup_and_one_retry() {
    let allocator = UserAllocator::new();
    let backing = Box::leak(vec![MaybeUninit::uninit(); 16 * 1024].into_boxed_slice());
    allocator.initialize(backing).unwrap();
    let retained_layout = Layout::from_size_align(64, 8).unwrap();
    let retained = allocator.try_allocate(retained_layout).unwrap();
    unsafe { retained.as_ptr().write_bytes(0x71, retained_layout.size()) };
    let held_layout = Layout::from_size_align(2 * 1024, 8).unwrap();
    let held = allocator.try_allocate(held_layout).unwrap();
    let remaining = allocator.usage().engine_free;
    let request = Layout::from_size_align(remaining + 1024, 8).unwrap();
    let mut opportunities = 0;

    let recovered = allocator
        .try_allocate_with_reclaim_once(request, || {
            opportunities += 1;
            // SAFETY: `held` is a live block from this allocator with this
            // exact layout; the one-shot callback transfers it back exactly once.
            unsafe { allocator.deallocate(held, held_layout) };
        })
        .unwrap();

    assert_eq!(opportunities, 1);
    assert_eq!(allocator.usage().failed_allocations, 1);
    assert!(
        unsafe { core::slice::from_raw_parts(retained.as_ptr(), retained_layout.size()) }
            .iter()
            .all(|byte| *byte == 0x71)
    );
    unsafe {
        allocator.deallocate(recovered, request);
        allocator.deallocate(retained, retained_layout);
    }
    assert_eq!(allocator.usage().live_allocations, 0);
    assert_eq!(allocator.usage().engine_free, allocator.usage().capacity);
}

#[test]
fn cleanup_opportunity_is_skipped_when_initial_allocation_succeeds() {
    let allocator = UserAllocator::new();
    let backing = Box::leak(vec![MaybeUninit::uninit(); 4096].into_boxed_slice());
    allocator.initialize(backing).unwrap();
    let layout = Layout::from_size_align(64, 8).unwrap();
    let allocation = allocator
        .try_allocate_with_reclaim_once(layout, || panic!("no OOM; no cleanup"))
        .unwrap();
    assert_eq!(allocator.usage().failed_allocations, 0);
    unsafe { allocator.deallocate(allocation, layout) };
}

#[test]
fn cleanup_is_bounded_even_when_it_does_not_make_the_request_fit() {
    let allocator = UserAllocator::new();
    let backing = Box::leak(vec![MaybeUninit::uninit(); 4096].into_boxed_slice());
    allocator.initialize(backing).unwrap();
    let layout = Layout::from_size_align(8 * 1024, 8).unwrap();
    let mut opportunities = 0;
    assert_eq!(
        allocator.try_allocate_with_reclaim_once(layout, || opportunities += 1),
        Err(AllocError::OutOfMemory)
    );
    assert_eq!(opportunities, 1);
    assert_eq!(allocator.usage().failed_allocations, 2);
}
