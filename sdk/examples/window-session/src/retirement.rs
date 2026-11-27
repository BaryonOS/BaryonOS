//! Bounded observational leaf-retirement publication; no recovery authority.

/// Startup-page byte offset in the compositor's private page.
pub const MASK_OFFSET: u64 = 536;

/// Latch one of the two leaf bits. Invalid indices or reserved bits are rejected.
pub const fn record(mask: u64, leaf: usize) -> Option<u64> {
    if mask & !3 != 0 || leaf >= 2 {
        return None;
    }
    Some(mask | (1u64 << leaf))
}
