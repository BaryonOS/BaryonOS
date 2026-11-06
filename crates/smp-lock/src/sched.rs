//! K06 step 5d-1: fixed-capacity run queue and the thread state word (spec §6).
//!
//! The kernel keeps one [`RunQueue`] per CPU behind a RUNQ-level lock and one
//! state word per thread. INV-3 ("one CPU per thread") rests on [`dispatch`]:
//! only a `compare_exchange` READY -> RUNNING(cpu) may put a thread on a CPU,
//! so a thread that is (wrongly) queued twice can never run twice at once.

use core::sync::atomic::{AtomicU32, Ordering};

pub const READY: u32 = 0;
pub const BLOCKED: u32 = 1;

/// The state value of a thread running on CPU index `cpu`.
pub const fn running(cpu: usize) -> u32 {
    2 + cpu as u32
}

/// READY -> RUNNING(cpu). `Err(observed)` means another CPU runs the thread
/// (or it is blocked): the caller must stop (`SMP:SCHED:DOUBLE_RUN`). Acquire
/// pairs with the Release in [`yield_cpu`], so the saved context is visible.
pub fn dispatch(state: &AtomicU32, cpu: usize) -> Result<(), u32> {
    state
        .compare_exchange(READY, running(cpu), Ordering::Acquire, Ordering::Relaxed)
        .map(drop)
}

/// RUNNING(cpu) -> READY after the caller stored the thread's context.
pub fn yield_cpu(state: &AtomicU32, cpu: usize) -> Result<(), u32> {
    state
        .compare_exchange(running(cpu), READY, Ordering::Release, Ordering::Relaxed)
        .map(drop)
}

/// BLOCKED -> READY (cross-CPU wake, spec §6.4); the caller then enqueues.
pub fn wake(state: &AtomicU32) -> Result<(), u32> {
    state
        .compare_exchange(BLOCKED, READY, Ordering::Release, Ordering::Relaxed)
        .map(drop)
}

/// FIFO ring of thread IDs; no allocator. Capacity equals the thread count,
/// so a correct scheduler (each thread in at most one queue) never overflows.
pub struct RunQueue<const N: usize> {
    items: [u32; N],
    head: usize,
    len: usize,
}

impl<const N: usize> RunQueue<N> {
    pub const fn new() -> Self {
        Self {
            items: [0; N],
            head: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// `Err(tid)` when full: a thread is queued twice, which callers treat as fatal.
    pub fn push(&mut self, tid: u32) -> Result<(), u32> {
        if self.len == N {
            return Err(tid);
        }
        self.items[(self.head + self.len) % N] = tid;
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<u32> {
        if self.len == 0 {
            return None;
        }
        let tid = self.items[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(tid)
    }
}

impl<const N: usize> Default for RunQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn queue_is_fifo_bounded_and_wraps() {
        let mut queue = RunQueue::<3>::new();
        assert_eq!(queue.pop(), None);
        for round in 0..4 {
            for tid in 0..3 {
                queue.push(round * 10 + tid).unwrap();
            }
            assert_eq!(queue.push(99), Err(99));
            for tid in 0..3 {
                assert_eq!(queue.pop(), Some(round * 10 + tid));
            }
            assert!(queue.is_empty());
        }
    }

    #[test]
    fn state_transitions_reject_double_run_and_foreign_yield() {
        let state = AtomicU32::new(BLOCKED);
        assert_eq!(dispatch(&state, 0), Err(BLOCKED));
        wake(&state).unwrap();
        assert_eq!(wake(&state), Err(READY));
        dispatch(&state, 1).unwrap();
        assert_eq!(dispatch(&state, 2), Err(running(1)));
        assert_eq!(yield_cpu(&state, 2), Err(running(1)));
        yield_cpu(&state, 1).unwrap();
        dispatch(&state, 2).unwrap();
    }

    #[test]
    fn concurrent_dispatch_has_exactly_one_winner() {
        for _ in 0..200 {
            let state = Arc::new(AtomicU32::new(READY));
            let winners: usize = (0..4)
                .map(|cpu| {
                    let state = Arc::clone(&state);
                    thread::spawn(move || dispatch(&state, cpu).is_ok())
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| usize::from(handle.join().unwrap()))
                .sum();
            assert_eq!(winners, 1);
        }
    }
}
