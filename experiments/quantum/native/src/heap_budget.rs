//! Pure checked bump-allocation arithmetic shared with the native allocator.

/// Return the aligned start and exclusive end within a fixed arena.
/// The arena base must itself be aligned to `capacity` (64 KiB in the probe).
pub fn allocation_range(
    used: usize,
    size: usize,
    alignment: usize,
    capacity: usize,
) -> Option<(usize, usize)> {
    if !alignment.is_power_of_two()
        || !capacity.is_power_of_two()
        || alignment > capacity
        || used > capacity
    {
        return None;
    }
    let start = used.checked_add(alignment - 1)? & !(alignment - 1);
    let end = start.checked_add(size)?;
    if end > capacity {
        return None;
    }
    Some((start, end))
}
