//! K06 run queue and thread state word (spec §6; step 5d-1, audit F16/F17, ADR-078).
//!
//! The kernel keeps one [`RunQueue`] per CPU behind a RUNQ-level lock, one
//! state word and one `queued` bit per thread. INV-3 ("one CPU per thread")
//! rests on [`dispatch`]: only a `compare_exchange` READY -> RUNNING(cpu) may
//! put a thread on a CPU. Every transition is a single atomic read-modify-write
//! (a CAS, or `fetch_update` with a pure step function, which is one
//! linearisable step), so the models in the tests use one step per call.
//!
//! Life cycle (BW-1, BW-3):
//! ```text
//! READY --dispatch--> RUNNING(c) --yield_cpu--> READY
//! RUNNING(c) --block_prepare--> BLOCKING --block_commit--> BLOCKED --wake--> READY
//!                               BLOCKING --wake--> READY_PENDING --block_commit--> READY
//! any live state --kill_request--> DEAD_PENDING --kill_commit (stack left)--> DEAD
//! ```

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const READY: u32 = 0;
/// Off every CPU and in no queue; only [`wake`] or [`kill_request`] moves it.
pub const BLOCKED: u32 = 1;
/// Still running on its CPU, but decided to wait (BW-1 phase 1).
pub const BLOCKING: u32 = 2;
/// A [`wake`] arrived while BLOCKING: the switch requeues instead of blocking.
pub const READY_PENDING: u32 = 3;
/// Kill requested; the thread may still be on a CPU or in a queue (BW-3).
pub const DEAD_PENDING: u32 = 4;
/// Off every CPU, off its stack and in no queue: cleanup may run.
pub const DEAD: u32 = 5;
const RUNNING_BASE: u32 = 8;

/// The state value of a thread running on CPU index `cpu`.
pub const fn running(cpu: usize) -> u32 {
    RUNNING_BASE + cpu as u32
}

/// READY -> RUNNING(cpu). `Err(observed)` means another CPU runs the thread
/// (or it is blocked or dying): the caller must not run it. Acquire pairs with
/// the Release of the transition that made it READY.
pub fn dispatch(state: &AtomicU32, cpu: usize) -> Result<(), u32> {
    state
        .compare_exchange(READY, running(cpu), Ordering::Acquire, Ordering::Acquire)
        .map(drop)
}

/// RUNNING(cpu) -> READY after the caller stored the thread's context.
/// `Err(BLOCKING | READY_PENDING)` -> [`block_commit`]; `Err(DEAD_PENDING)` ->
/// leave the CPU, requeue, and reap at a later switch with [`kill_commit`].
pub fn yield_cpu(state: &AtomicU32, cpu: usize) -> Result<(), u32> {
    state
        .compare_exchange(running(cpu), READY, Ordering::Release, Ordering::Acquire)
        .map(drop)
}

/// BW-1 phase 1, by the thread itself on CPU `cpu`: RUNNING(cpu) -> BLOCKING.
/// The caller checks its wait condition only AFTER this succeeds (or under the
/// lock its waker holds); a condition found true is settled by a [`wake`] of
/// itself. SeqCst: a lock-free waiter and [`wake`] form a store-buffering pair.
pub fn block_prepare(state: &AtomicU32, cpu: usize) -> Result<(), u32> {
    state
        .compare_exchange(running(cpu), BLOCKING, Ordering::SeqCst, Ordering::SeqCst)
        .map(drop)
}

/// BW-1 phase 2, in the context switch after the context is saved:
/// BLOCKING -> BLOCKED (`Ok(false)`: stays out of every queue) or
/// READY_PENDING -> READY (`Ok(true)`: the caller requeues). `Err(DEAD_PENDING)`
/// if a kill won; any other `Err` is a scheduler bug.
pub fn block_commit(state: &AtomicU32) -> Result<bool, u32> {
    state
        .fetch_update(Ordering::Release, Ordering::Acquire, |s| match s {
            BLOCKING => Some(BLOCKED),
            READY_PENDING => Some(READY),
            _ => None,
        })
        .map(|old| old == READY_PENDING)
}

/// Cross-CPU wake (spec §6.4). `Ok(true)`: BLOCKED -> READY, the caller must
/// enqueue on the thread's CPU and send 0xe8. `Ok(false)`: BLOCKING ->
/// READY_PENDING, the thread's own switch requeues it. `Err(observed)`: not
/// waiting (running, ready, already pending, or dying); nothing to do, since a
/// running target checks its condition after [`block_prepare`].
pub fn wake(state: &AtomicU32) -> Result<bool, u32> {
    state
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |s| match s {
            BLOCKED => Some(READY),
            BLOCKING => Some(READY_PENDING),
            _ => None,
        })
        .map(|old| old == BLOCKED)
}

/// BW-3: any live state -> DEAD_PENDING, `Ok(previous)`. Previous BLOCKED: the
/// thread is in no queue, so the caller enqueues it on its CPU, whose next
/// switch reaps it. READY: it is queued already. Running, BLOCKING or
/// READY_PENDING: its CPU requeues it at the next switch. Then the caller sends
/// 0xe8 and waits (bounded) for DEAD. `Err` if it is already dying or dead.
pub fn kill_request(state: &AtomicU32) -> Result<u32, u32> {
    state.fetch_update(Ordering::AcqRel, Ordering::Acquire, |s| match s {
        DEAD_PENDING | DEAD => None,
        _ => Some(DEAD_PENDING),
    })
}

/// BW-3, by the CPU that popped a DEAD_PENDING thread from its queue:
/// DEAD_PENDING -> DEAD, only once no CPU is on the thread's stack (`on_cpu`
/// cleared by the switch stub, audit F1). `on_cpu` cannot be set again: that
/// needs a successful [`dispatch`]. `Err(DEAD_PENDING)`: still on a stack,
/// requeue and retry at a later switch. Release: whoever sees DEAD (Acquire)
/// may free the stack and run the K05 cleanup.
pub fn kill_commit(state: &AtomicU32, on_cpu: &AtomicBool) -> Result<(), u32> {
    if on_cpu.load(Ordering::Acquire) {
        return Err(DEAD_PENDING);
    }
    state
        .compare_exchange(DEAD_PENDING, DEAD, Ordering::Release, Ordering::Acquire)
        .map(drop)
}

/// Audit F1: a CPU publishes READY ([`yield_cpu`]) while it still runs on the
/// thread's stack, and leaves that stack only in the interrupt stub. The
/// thread's `on_cpu` flag is cleared (Release) by that stub after `mov rsp`;
/// a dispatcher must claim it (Acquire) before resuming the thread. Never
/// spin on it inside the switch (two CPUs could wait on each other): on
/// `false`, give the thread back with [`yield_cpu`] and requeue it.
pub fn claim_stack(on_cpu: &AtomicBool) -> bool {
    on_cpu
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}

/// FIFO ring of thread IDs; no allocator. BW-2: `queued[tid]`, shared by all
/// queues, is set by [`push`](Self::push) and cleared by [`pop`](Self::pop), so a
/// thread is in at most one queue and a capacity >= the thread count suffices.
#[derive(Clone)]
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

    /// BW-2: `queued[tid]` 0 -> 1, then append. `Err(tid)` when the thread is
    /// already in some queue (`SMP:SCHED:DOUBLE_ENQUEUE`, fatal) or, which the
    /// bit rules out for `queued.len() <= N`, when the ring is full.
    pub fn push(&mut self, tid: u32, queued: &[AtomicBool]) -> Result<(), u32> {
        if self.len == N
            || queued[tid as usize]
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
        {
            return Err(tid);
        }
        self.items[(self.head + self.len) % N] = tid;
        self.len += 1;
        Ok(())
    }

    /// Take the oldest thread and clear its `queued` bit (1 -> 0).
    pub fn pop(&mut self, queued: &[AtomicBool]) -> Option<u32> {
        if self.len == 0 {
            return None;
        }
        let tid = self.items[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        let was = queued[tid as usize].swap(false, Ordering::Release);
        debug_assert!(was, "queued thread without its bit");
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

    const SC: Ordering = Ordering::SeqCst;

    fn bits<const N: usize>() -> [AtomicBool; N] {
        [const { AtomicBool::new(false) }; N]
    }

    #[test]
    fn queue_is_fifo_bounded_and_wraps() {
        let queued = bits::<4>();
        let mut queue = RunQueue::<3>::new();
        assert_eq!(queue.pop(&queued), None);
        for _ in 0..4 {
            for tid in 0..3 {
                queue.push(tid, &queued).unwrap();
            }
            assert_eq!(queue.push(3, &queued), Err(3), "full");
            assert!(!queued[3].load(SC), "a refused push leaves the bit clear");
            for tid in 0..3 {
                assert_eq!(queue.pop(&queued), Some(tid));
            }
            assert!(queue.is_empty());
            assert!(queued.iter().all(|bit| !bit.load(SC)));
        }
    }

    #[test]
    fn second_push_is_refused_in_any_queue() {
        // BW-2 / F17: the double enqueue is caught at the push, not by timing.
        let queued = bits::<2>();
        let (mut a, mut b) = (RunQueue::<2>::new(), RunQueue::<2>::new());
        a.push(0, &queued).unwrap();
        assert_eq!(a.push(0, &queued), Err(0));
        assert_eq!(b.push(0, &queued), Err(0));
        assert_eq!(a.len() + b.len(), 1);
        assert_eq!(a.pop(&queued), Some(0));
        b.push(0, &queued).unwrap();
    }

    #[test]
    fn racing_pushes_of_one_thread_have_exactly_one_winner() {
        for _ in 0..200 {
            let queued = Arc::new(bits::<1>());
            let winners: usize = (0..4)
                .map(|_| {
                    let queued = Arc::clone(&queued);
                    thread::spawn(move || RunQueue::<1>::new().push(0, &queued[..]).is_ok())
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| usize::from(handle.join().unwrap()))
                .sum();
            assert_eq!(winners, 1);
        }
    }

    #[test]
    fn state_transitions_reject_double_run_and_foreign_yield() {
        let state = AtomicU32::new(BLOCKED);
        assert_eq!(dispatch(&state, 0), Err(BLOCKED));
        assert_eq!(wake(&state), Ok(true));
        assert_eq!(wake(&state), Err(READY));
        dispatch(&state, 1).unwrap();
        assert_eq!(dispatch(&state, 2), Err(running(1)));
        assert_eq!(yield_cpu(&state, 2), Err(running(1)));
        assert_eq!(block_prepare(&state, 2), Err(running(1)));
        yield_cpu(&state, 1).unwrap();
        dispatch(&state, 2).unwrap();
    }

    #[test]
    fn block_wake_and_kill_follow_the_life_cycle() {
        let state = AtomicU32::new(running(1));
        block_prepare(&state, 1).unwrap();
        assert_eq!(yield_cpu(&state, 1), Err(BLOCKING));
        assert_eq!(block_commit(&state), Ok(false));
        assert_eq!(state.load(SC), BLOCKED);
        assert_eq!(block_commit(&state), Err(BLOCKED));
        // Woken between the two phases: the switch requeues it.
        state.store(running(1), SC);
        block_prepare(&state, 1).unwrap();
        assert_eq!(wake(&state), Ok(false));
        assert_eq!(wake(&state), Err(READY_PENDING));
        assert_eq!(block_commit(&state), Ok(true));
        assert_eq!(state.load(SC), READY);
        // Kill from every live state; never twice, never back to life.
        let on_cpu = AtomicBool::new(false);
        for live in [READY, BLOCKED, BLOCKING, READY_PENDING, running(3)] {
            state.store(live, SC);
            assert_eq!(kill_request(&state), Ok(live));
            assert_eq!(kill_request(&state), Err(DEAD_PENDING));
            assert_eq!(wake(&state), Err(DEAD_PENDING));
            assert_eq!(dispatch(&state, 0), Err(DEAD_PENDING));
            assert_eq!(block_commit(&state), Err(DEAD_PENDING));
            on_cpu.store(true, SC);
            assert_eq!(
                kill_commit(&state, &on_cpu),
                Err(DEAD_PENDING),
                "stack in use"
            );
            on_cpu.store(false, SC);
            kill_commit(&state, &on_cpu).unwrap();
            assert_eq!(kill_request(&state), Err(DEAD));
            assert_eq!(kill_commit(&state, &on_cpu), Err(DEAD));
        }
    }

    #[test]
    fn stack_is_never_shared_after_early_ready() {
        // F1 model: CPU A publishes READY, keeps using the stack, then leaves
        // it (`on_cpu` cleared, as the stub does). CPU B dispatches the thread
        // as soon as it is READY and may resume it only after claiming it.
        for _ in 0..200 {
            let state = Arc::new(AtomicU32::new(READY));
            let on_cpu = Arc::new(AtomicBool::new(false));
            let a_on_stack = Arc::new(AtomicBool::new(false));
            dispatch(&state, 0).unwrap();
            assert!(claim_stack(&on_cpu));
            a_on_stack.store(true, Ordering::Relaxed);
            let b = {
                let (state, on_cpu, a_on_stack) = (
                    Arc::clone(&state),
                    Arc::clone(&on_cpu),
                    Arc::clone(&a_on_stack),
                );
                thread::spawn(move || loop {
                    if dispatch(&state, 1).is_err() {
                        continue;
                    }
                    if claim_stack(&on_cpu) {
                        assert!(!a_on_stack.load(Ordering::Relaxed), "stack shared");
                        return;
                    }
                    yield_cpu(&state, 1).unwrap();
                })
            };
            yield_cpu(&state, 0).unwrap();
            for _ in 0..1000 {
                std::hint::spin_loop();
            }
            a_on_stack.store(false, Ordering::Relaxed);
            on_cpu.store(false, Ordering::Release);
            b.join().unwrap();
        }
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

    // Exhaustive interleaving models (BW-1, BW-3). Each actor step is ONE
    // atomic operation of the real functions above; `explore` runs every
    // order of those steps and checks the end state. This covers sequentially
    // consistent executions only, not weaker orderings (loom would; ADR-078
    // records why loom is not a dependency). Each model has a mutant that the
    // same check must refute, so a vacuous model cannot pass.

    trait Model: Clone {
        /// Actors that can take a step now.
        fn enabled(&self) -> Vec<usize>;
        fn step(&mut self, actor: usize);
        /// `Err` names a violated invariant at the end of an execution.
        fn check(&self) -> Result<(), &'static str>;
    }

    /// (executions explored, first violation found).
    fn explore<M: Model>(model: &M) -> (usize, Option<&'static str>) {
        let actors = model.enabled();
        if actors.is_empty() {
            return (1, model.check().err());
        }
        actors.into_iter().fold((0, None), |(runs, bad), actor| {
            let mut next = model.clone();
            next.step(actor);
            let (more, found) = explore(&next);
            (runs + more, bad.or(found))
        })
    }

    fn copy(word: &AtomicU32) -> AtomicU32 {
        AtomicU32::new(word.load(SC))
    }

    fn copy_bit(bit: &AtomicBool) -> AtomicBool {
        AtomicBool::new(bit.load(SC))
    }

    /// BW-1: blocker `i` runs on CPU `i` and waits for `cond[i]`; one waker on
    /// a third CPU sets both conditions and wakes both threads. `naive` is the
    /// 5d-1 protocol (check, then BLOCKED at the switch; wake only BLOCKED).
    struct BlockWake {
        naive: bool,
        state: [AtomicU32; 2],
        cond: [AtomicBool; 2],
        queued: [AtomicBool; 2],
        queue: [RunQueue<2>; 2],
        pc: [u8; 3],
    }

    impl Clone for BlockWake {
        fn clone(&self) -> Self {
            Self {
                naive: self.naive,
                state: self.state.each_ref().map(copy),
                cond: self.cond.each_ref().map(copy_bit),
                queued: self.queued.each_ref().map(copy_bit),
                queue: self.queue.clone(),
                pc: self.pc,
            }
        }
    }

    const DONE: u8 = 9;

    impl BlockWake {
        fn new(naive: bool) -> Self {
            Self {
                naive,
                state: [AtomicU32::new(running(0)), AtomicU32::new(running(1))],
                cond: bits(),
                queued: bits(),
                queue: [RunQueue::new(), RunQueue::new()],
                pc: [0; 3],
            }
        }

        fn enqueue(&mut self, tid: usize) {
            self.queue[tid].push(tid as u32, &self.queued).unwrap();
        }

        fn blocker(&mut self, tid: usize) {
            let state = &self.state[tid];
            self.pc[tid] = match (self.naive, self.pc[tid]) {
                (false, 0) => {
                    block_prepare(state, tid).unwrap();
                    1
                }
                (false, 1) => {
                    if self.cond[tid].load(SC) {
                        // The waker may have been first (READY_PENDING already).
                        assert!(matches!(wake(state), Ok(false) | Err(READY_PENDING)));
                    }
                    2
                }
                (false, _) => {
                    // The switch: yield refuses a waiting thread, commit instead.
                    assert!(yield_cpu(state, tid).is_err());
                    if block_commit(state).unwrap() {
                        self.enqueue(tid);
                    }
                    DONE
                }
                (true, 0) if self.cond[tid].load(SC) => DONE,
                (true, 0) => 1,
                (true, _) => {
                    let _ = state.compare_exchange(running(tid), BLOCKED, SC, SC);
                    DONE
                }
            };
        }

        fn waker(&mut self) {
            let tid = usize::from(self.pc[2] / 2);
            if self.pc[2].is_multiple_of(2) {
                self.cond[tid].store(true, SC);
            } else {
                let woken = if self.naive {
                    self.state[tid]
                        .compare_exchange(BLOCKED, READY, SC, SC)
                        .is_ok()
                } else {
                    wake(&self.state[tid]) == Ok(true)
                };
                if woken {
                    self.enqueue(tid);
                }
            }
            self.pc[2] += 1;
        }
    }

    impl Model for BlockWake {
        fn enabled(&self) -> Vec<usize> {
            (0..3)
                .filter(|&actor| {
                    if actor == 2 {
                        self.pc[2] < 4
                    } else {
                        self.pc[actor] != DONE
                    }
                })
                .collect()
        }

        fn step(&mut self, actor: usize) {
            if actor == 2 {
                self.waker();
            } else {
                self.blocker(actor);
            }
        }

        fn check(&self) -> Result<(), &'static str> {
            for tid in 0..2 {
                let state = self.state[tid].load(SC);
                if state == BLOCKED {
                    return Err("lost wakeup: woken thread ended BLOCKED");
                }
                if (state == READY) != self.queued[tid].load(SC) {
                    return Err("READY thread not queued exactly once");
                }
            }
            Ok(())
        }
    }

    #[test]
    fn model_block_wake_never_loses_a_wakeup() {
        let (runs, bad) = explore(&BlockWake::new(false));
        assert_eq!(bad, None);
        assert!(runs > 1000, "explored {runs} executions");
    }

    #[test]
    fn model_finds_the_lost_wakeup_of_the_single_phase_protocol() {
        // Teeth: the 5d-1 protocol (F16) must be refuted by the same model.
        let (_, bad) = explore(&BlockWake::new(true));
        assert_eq!(bad, Some("lost wakeup: woken thread ended BLOCKED"));
    }

    /// BW-3: thread 0 runs on CPU 1 (`d`); the killer requests the kill at any
    /// point and starts "cleanup" once it sees DEAD. `d` performs `TICKS`
    /// switches as `sched_switch` does: leave current, pop, dispatch or reap,
    /// claim, stub clears `on_cpu`. `eager` is the mutant that publishes DEAD at
    /// the switch itself, while still on the stack.
    struct Kill {
        eager: bool,
        state: AtomicU32,
        on_cpu: AtomicBool,
        queued: [AtomicBool; 1],
        queue: RunQueue<1>,
        current: bool,
        next: bool,
        d_pc: u8,
        tick: u8,
        killed_at: Option<u8>,
        cleanup: bool,
        violation: Option<&'static str>,
    }

    impl Clone for Kill {
        fn clone(&self) -> Self {
            Self {
                state: copy(&self.state),
                on_cpu: copy_bit(&self.on_cpu),
                queued: self.queued.each_ref().map(copy_bit),
                queue: self.queue.clone(),
                ..*self
            }
        }
    }

    const TICKS: u8 = 4;

    impl Kill {
        fn new(eager: bool) -> Self {
            Self {
                eager,
                state: AtomicU32::new(running(1)),
                on_cpu: AtomicBool::new(true),
                queued: bits(),
                queue: RunQueue::new(),
                current: true,
                next: false,
                d_pc: 0,
                tick: 0,
                killed_at: None,
                cleanup: false,
                violation: None,
            }
        }

        fn push(&mut self) {
            if self.queue.push(0, &self.queued).is_err() {
                self.violation = Some("double enqueue");
            }
        }

        /// Off the CPU and back into the queue (a dying thread too, for reaping).
        fn leave(&mut self) {
            match yield_cpu(&self.state, 1) {
                Ok(()) | Err(DEAD_PENDING) => self.push(),
                Err(_) => self.violation = Some("double run"),
            }
        }

        fn cpu_d(&mut self) {
            self.d_pc = match self.d_pc {
                // §6.3 step 4: save and leave the current thread.
                0 => {
                    if self.current {
                        if self.eager && self.state.load(SC) == DEAD_PENDING {
                            self.state.store(DEAD, SC);
                        } else {
                            self.leave();
                        }
                    }
                    1
                }
                // Pop and dispatch, or find it dying.
                1 => {
                    self.next = false;
                    if self.queue.pop(&self.queued).is_none() {
                        4
                    } else {
                        match dispatch(&self.state, 1) {
                            Ok(()) => {
                                self.next = true;
                                3
                            }
                            Err(DEAD_PENDING) => 2,
                            Err(_) => {
                                self.violation = Some("double run");
                                4
                            }
                        }
                    }
                }
                // Reap only once the stack is left; else requeue for later.
                2 => {
                    if kill_commit(&self.state, &self.on_cpu).is_err() {
                        self.push();
                    }
                    4
                }
                // F1: claim the stack unless it is this CPU's current thread.
                3 => {
                    if !self.current && !claim_stack(&self.on_cpu) {
                        self.leave();
                        self.next = false;
                    }
                    4
                }
                // The stub: off the old stack, then clear its `on_cpu`.
                _ => {
                    if self.current && !self.next {
                        self.on_cpu.store(false, Ordering::Release);
                    }
                    self.current = self.next;
                    self.tick += 1;
                    0
                }
            };
        }

        fn killer(&mut self) {
            if self.killed_at.is_none() {
                match kill_request(&self.state) {
                    Ok(BLOCKED) => self.push(),
                    Ok(_) => {}
                    Err(_) => self.violation = Some("killed twice"),
                }
                self.killed_at = Some(self.tick);
                return;
            }
            // Enabled only once DEAD is visible (Acquire): the cleanup starts.
            if self.on_cpu.load(Ordering::Acquire) || self.current {
                self.violation = Some("cleanup while a CPU is on the stack");
            } else if self.queued[0].load(SC) {
                self.violation = Some("DEAD thread still queued");
            }
            self.cleanup = true;
        }
    }

    impl Model for Kill {
        fn enabled(&self) -> Vec<usize> {
            if self.violation.is_some() {
                return Vec::new();
            }
            let dead = self.state.load(Ordering::Acquire) == DEAD;
            let mut actors = Vec::new();
            if self.tick < TICKS {
                actors.push(0);
            }
            if !self.cleanup && (self.killed_at.is_none() || dead) {
                actors.push(1);
            }
            actors
        }

        fn step(&mut self, actor: usize) {
            if actor == 0 {
                self.cpu_d();
            } else {
                self.killer();
            }
        }

        fn check(&self) -> Result<(), &'static str> {
            if let Some(violation) = self.violation {
                return Err(violation);
            }
            // Bounded liveness: two full switches after the request reap it.
            match self.killed_at {
                Some(at) if at + 2 < TICKS && !self.cleanup => Err("no DEAD after two switches"),
                _ => Ok(()),
            }
        }
    }

    #[test]
    fn model_kill_reaps_only_after_the_stack_is_left() {
        let (runs, bad) = explore(&Kill::new(false));
        assert_eq!(bad, None);
        assert!(runs > 20, "explored {runs} executions");
    }

    #[test]
    fn model_finds_dead_published_on_the_stack() {
        // Teeth: DEAD at the switch itself lets the cleanup race the stub.
        let (_, bad) = explore(&Kill::new(true));
        assert_eq!(bad, Some("cleanup while a CPU is on the stack"));
    }
}
