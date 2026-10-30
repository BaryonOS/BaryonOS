//! Bounded user-process allocation over one exclusively owned arena.
//!
//! The first-fit/coalescing algorithm is `linked_list_allocator` 0.10.6. This
//! crate adds ownership checks, a short lock, fallible operations and local
//! accounting. It never obtains memory from an OS or another allocator.
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::{align_of, size_of, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use linked_list_allocator::Heap;

const WORD_ALIGN: usize = align_of::<usize>();
const MIN_ARENA: usize = 2 * size_of::<usize>();

// The upstream engine uses wrapping pointer arithmetic while trying a hole.
// Bound every possible intermediate of that search before invoking it. This
// conservatively rejects extreme high-address/alignment combinations, even if
// a particular first hole could fit, rather than allowing wrapped candidates.
fn allocation_arithmetic_fits(top: usize, layout: Layout) -> bool {
    layout
        .size()
        .max(MIN_ARENA)
        .checked_add(WORD_ALIGN - 1)
        .map(|bytes| bytes & !(WORD_ALIGN - 1))
        .and_then(|bytes| top.checked_add(layout.align() - 1)?.checked_add(bytes))
        .is_some()
}

/// Shape errors are detected without reading or writing the proposed arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitError {
    Null,
    ExtentOverflow,
    TooSmall,
    AlreadyInitialized,
}

/// A checked extent, **not** evidence that memory is mapped or owned.
///
/// Construction does not dereference the pointer. `initialize_raw` is unsafe
/// because pointer shape cannot establish lifetime, mapping or ownership.
#[derive(Debug)]
pub struct Arena {
    start: NonNull<u8>,
    usable_bytes: usize,
    provided_bytes: usize,
}

impl Arena {
    pub fn checked(start: *mut u8, bytes: usize) -> Result<Self, InitError> {
        let start = NonNull::new(start).ok_or(InitError::Null)?;
        let address = start.as_ptr() as usize;
        if bytes > isize::MAX as usize {
            return Err(InitError::ExtentOverflow);
        }
        let end = address
            .checked_add(bytes)
            .ok_or(InitError::ExtentOverflow)?;
        let aligned = address
            .checked_add(WORD_ALIGN - 1)
            .ok_or(InitError::ExtentOverflow)?
            & !(WORD_ALIGN - 1);
        let end = end & !(WORD_ALIGN - 1);
        let usable_bytes = end
            .checked_sub(aligned)
            .filter(|bytes| *bytes >= MIN_ARENA)
            .ok_or(InitError::TooSmall)?;
        // Preserve the caller pointer's provenance; no dereference occurs here.
        let aligned_pointer = start.as_ptr().wrapping_add(aligned - address);
        Ok(Self {
            start: NonNull::new(aligned_pointer).ok_or(InitError::ExtentOverflow)?,
            usable_bytes,
            provided_bytes: bytes,
        })
    }

    pub fn start(&self) -> *mut u8 {
        self.start.as_ptr()
    }

    pub fn usable_bytes(&self) -> usize {
        self.usable_bytes
    }

    pub fn provided_bytes(&self) -> usize {
        self.provided_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocError {
    NotInitialized,
    ZeroSize,
    InvalidLayout,
    OutOfMemory,
}

/// All values describe only this allocator's arena. Engine usage includes
/// allocation padding/minimum blocks; free bytes can be fragmented. Historical
/// totals saturate and set `counters_saturated`, rather than wrapping.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    pub initialized: bool,
    pub provided_bytes: usize,
    pub capacity: usize,
    pub engine_used: usize,
    pub engine_free: usize,
    pub payload_used: usize,
    pub live_allocations: usize,
    pub high_water_engine: usize,
    pub high_water_payload: usize,
    pub allocations: u64,
    pub deallocations: u64,
    pub failed_allocations: u64,
    pub counters_saturated: bool,
}

struct State {
    heap: Heap,
    usage: Usage,
}

impl State {
    const fn empty() -> Self {
        Self {
            heap: Heap::empty(),
            usage: Usage {
                initialized: false,
                provided_bytes: 0,
                capacity: 0,
                engine_used: 0,
                engine_free: 0,
                payload_used: 0,
                live_allocations: 0,
                high_water_engine: 0,
                high_water_payload: 0,
                allocations: 0,
                deallocations: 0,
                failed_allocations: 0,
                counters_saturated: false,
            },
        }
    }

    fn increment(counter: &mut u64, saturated: &mut bool) {
        match counter.checked_add(1) {
            Some(value) => *counter = value,
            None => *saturated = true,
        }
    }

    fn failed(&mut self, error: AllocError) -> AllocError {
        Self::increment(
            &mut self.usage.failed_allocations,
            &mut self.usage.counters_saturated,
        );
        error
    }

    fn refresh(&mut self) {
        self.usage.engine_used = self.heap.used();
        self.usage.engine_free = self.heap.free();
        self.usage.high_water_engine = self.usage.high_water_engine.max(self.heap.used());
        self.usage.high_water_payload = self.usage.high_water_payload.max(self.usage.payload_used);
    }
}

/// One allocator per process over a fixed arena. Initialization is explicit;
/// neither OOM nor a missing initializer can invoke a host/global fallback.
///
/// Intended native model: one user thread per process, preemption permitted.
/// A short atomic lock also serializes host threads for tests. Never reenter
/// this allocator from an interrupt/signal while the same thread holds its
/// lock. No syscall, callback, allocation, logging or yield occurs under it.
pub struct UserAllocator {
    locked: AtomicBool,
    state: UnsafeCell<State>,
}

// SAFETY: Heap is Send upstream. Its sole state and arena metadata can only
// be accessed through the acquire/release lock. Initialization transfers one
// exclusive 'static arena; no reference to State escapes the private guard.
unsafe impl Sync for UserAllocator {}

impl Default for UserAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl UserAllocator {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            state: UnsafeCell::new(State::empty()),
        }
    }

    fn lock(&self) -> Guard<'_> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Guard { allocator: self }
    }

    /// Takes exclusive ownership of a static byte buffer. Native callers must
    /// additionally ensure the kernel has mapped their arena RW+NX.
    pub fn initialize(&self, memory: &'static mut [MaybeUninit<u8>]) -> Result<(), InitError> {
        let arena = Arena::checked(memory.as_mut_ptr().cast(), memory.len())?;
        // SAFETY: the exclusive 'static reference proves accessible, uniquely
        // borrowed storage. No reference is returned or retained by the caller.
        unsafe { self.initialize_raw(arena) }
    }

    /// Initializes once, without obtaining or extending memory.
    ///
    /// # Safety
    /// The entire checked arena must be valid, contiguous, exclusively owned
    /// writable memory for the remainder of the program. It must not overlap
    /// this allocator, its state, another allocator or any live data/reference.
    /// The caller must transfer all access to this allocator and its returned
    /// allocations. Native mappings must belong to this process and be RW+NX;
    /// constructing `Arena` does not prove any of those properties.
    pub unsafe fn initialize_raw(&self, arena: Arena) -> Result<(), InitError> {
        let mut state = self.lock();
        if state.usage.initialized {
            return Err(InitError::AlreadyInitialized);
        }
        // SAFETY: caller supplied the ownership/lifetime above; Arena already
        // normalized alignment/size to Heap's minimum of two machine words.
        unsafe { state.heap.init(arena.start.as_ptr(), arena.usable_bytes) };
        state.usage.initialized = true;
        state.usage.provided_bytes = arena.provided_bytes;
        state.usage.capacity = arena.usable_bytes;
        state.refresh();
        Ok(())
    }

    /// Allocates uninitialized bytes. Zero size is an explicit error; callers
    /// such as Vec handle their own zero-sized sentinel without asking us.
    pub fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let mut state = self.lock();
        if !state.usage.initialized {
            return Err(state.failed(AllocError::NotInitialized));
        }
        if layout.size() == 0 {
            return Err(state.failed(AllocError::ZeroSize));
        }
        // Bounds also prevent enormous valid Layouts reaching engine arithmetic.
        if layout.size() > state.heap.free()
            || !allocation_arithmetic_fits(state.heap.top() as usize, layout)
        {
            return Err(state.failed(AllocError::OutOfMemory));
        }
        let pointer = state
            .heap
            .allocate_first_fit(layout)
            .map_err(|()| state.failed(AllocError::OutOfMemory))?;
        // For valid ownership, payload/live counts are bounded by arena size:
        // every engine block is at least one byte and >= its payload size.
        state.usage.payload_used += layout.size();
        state.usage.live_allocations += 1;
        let usage = &mut state.usage;
        State::increment(&mut usage.allocations, &mut usage.counters_saturated);
        state.refresh();
        Ok(pointer)
    }

    /// Checks malformed size/alignment and Layout overflow before allocation.
    pub fn try_allocate_bytes(
        &self,
        size: usize,
        alignment: usize,
    ) -> Result<NonNull<u8>, AllocError> {
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| self.lock().failed(AllocError::InvalidLayout))?;
        self.try_allocate(layout)
    }

    /// # Safety
    /// `pointer` must denote a currently live allocation returned by this exact
    /// allocator, with the identical `layout`. Deallocate it exactly once, with
    /// no accesses afterwards. Invalid pointers/layouts or double free are UB;
    /// this interface does not pretend to validate that unsafe contract.
    pub unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        let mut state = self.lock();
        // SAFETY: the caller promises the upstream ownership/Layout contract.
        unsafe { state.heap.deallocate(pointer, layout) };
        state.usage.payload_used -= layout.size();
        state.usage.live_allocations -= 1;
        let usage = &mut state.usage;
        State::increment(&mut usage.deallocations, &mut usage.counters_saturated);
        state.refresh();
    }

    pub fn usage(&self) -> Usage {
        self.lock().usage
    }
}

struct Guard<'a> {
    allocator: &'a UserAllocator,
}

impl Deref for Guard<'_> {
    type Target = State;
    fn deref(&self) -> &State {
        // SAFETY: the guard owns the only acquire lock; reference stays within
        // the private guard lifetime and is never exposed by the public API.
        unsafe { &*self.allocator.state.get() }
    }
}

impl DerefMut for Guard<'_> {
    fn deref_mut(&mut self) -> &mut State {
        // SAFETY: same unique lock, plus an exclusive borrow of this guard.
        unsafe { &mut *self.allocator.state.get() }
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.allocator.locked.store(false, Ordering::Release);
    }
}

// SAFETY: successful allocation comes only from the owned arena with Layout's
// alignment; engine preserves live disjoint blocks. Failure returns null, with
// no fallback. Deallocation obeys caller's GlobalAlloc contract. The default
// alloc_zeroed/realloc implement payload initialization/copy and preserve the
// original allocation on OOM. No wrapper path unwinds for valid inputs.
unsafe impl GlobalAlloc for UserAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.try_allocate(layout)
            .map_or(core::ptr::null_mut(), NonNull::as_ptr)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: GlobalAlloc caller guarantees a nonnull live allocation and
        // the identical Layout, owned by this allocator.
        unsafe { self.deallocate(NonNull::new_unchecked(pointer), layout) };
    }
}

#[cfg(test)]
mod tests {
    use super::{allocation_arithmetic_fits, State};
    use core::alloc::Layout;

    #[test]
    fn extreme_search_arithmetic_is_rejected_before_engine_access() {
        let ordinary = Layout::from_size_align(32, 64).unwrap();
        assert!(allocation_arithmetic_fits(0x6001_0000, ordinary));
        assert!(!allocation_arithmetic_fits(usize::MAX - 31, ordinary));
        let enormous_alignment = Layout::from_size_align(1, 1usize << (usize::BITS - 2)).unwrap();
        assert!(!allocation_arithmetic_fits(
            usize::MAX / 4 * 3,
            enormous_alignment
        ));
        // Test only integer boundaries; never invent a mapped high pointer.
        assert!(allocation_arithmetic_fits(4096, Layout::new::<u8>()));
    }

    #[test]
    fn historical_counters_saturate_without_wrapping() {
        let mut count = u64::MAX - 1;
        let mut saturated = false;
        State::increment(&mut count, &mut saturated);
        assert_eq!(count, u64::MAX);
        assert!(!saturated);
        State::increment(&mut count, &mut saturated);
        assert_eq!(count, u64::MAX);
        assert!(saturated);
    }
}
