//! Bounded IPC queue waits. Retain scalar copies, never user pointers, CopyPlans,
//! object indices, node references, messages or reservations across scheduling.
use capability_core::{Engine, Error, RECV, SEND};

pub(super) const MAX_CONDITIONS: usize = 16;
pub(super) const HORIZON_TICKS: u64 = 1000;
const CONDITION_BYTES: usize = 16;
const WIRE_BYTES: usize = MAX_CONDITIONS * CONDITION_BYTES;

fn validate_arguments(args: [u64; 6], now: u64) -> Result<usize, Error> {
    if args[3..].iter().any(|arg| *arg != 0)
        || args[1] > MAX_CONDITIONS as u64
        || args[1] == 0 && args[0] != 0
        || args[2].saturating_sub(now) > HORIZON_TICKS
    {
        return Err(Error::Invalid);
    }
    Ok(args[1] as usize)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Condition {
    handle: u64,
    interest: u64,
}
const EMPTY: Condition = Condition {
    handle: 0,
    interest: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WaitSet {
    pid: u64,
    registered_at: u64,
    deadline: u64,
    count: usize,
    conditions: [Condition; MAX_CONDITIONS],
}
impl WaitSet {
    fn decode(
        pid: u64,
        now: u64,
        deadline: u64,
        count: usize,
        bytes: &[u8; WIRE_BYTES],
    ) -> Result<Self, Error> {
        if pid == 0 || count > MAX_CONDITIONS || deadline.saturating_sub(now) > HORIZON_TICKS {
            return Err(Error::Invalid);
        }
        let mut result = Self {
            pid,
            registered_at: now,
            deadline,
            count,
            conditions: [EMPTY; MAX_CONDITIONS],
        };
        for (entry, wire) in result.conditions[..count]
            .iter_mut()
            .zip(bytes.chunks_exact(CONDITION_BYTES))
        {
            let interest = u64::from_le_bytes(wire[8..16].try_into().unwrap());
            if interest != SEND && interest != RECV {
                return Err(Error::Invalid);
            }
            *entry = Condition {
                handle: u64::from_le_bytes(wire[..8].try_into().unwrap()),
                interest,
            };
        }
        Ok(result)
    }

    // None means still blocked; Some(0) is expiration; other masks are advisory
    // queue readiness. Validate ALL conditions before returning any ready bit.
    fn evaluate(&self, engine: &Engine, pid: u64, now: u64) -> Result<Option<u64>, Error> {
        if pid != self.pid {
            return Err(Error::BadHandle);
        }
        let mut ready = 0u64;
        for (index, condition) in self.conditions[..self.count].iter().enumerate() {
            if engine.endpoint_queue_ready(pid, condition.handle, condition.interest)? {
                ready |= 1 << index;
            }
        }
        if now < self.registered_at {
            return Err(Error::Invalid);
        }
        if ready != 0 {
            Ok(Some(ready))
        } else if now >= self.deadline {
            Ok(Some(0))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Metrics {
    pub calls: u64,
    pub blocked: u64,
    pub immediate_ready: u64,
    pub immediate_timeout: u64,
    pub wake_ready: u64,
    pub wake_timeout: u64,
    pub wake_error: u64,
    pub idle_irqs: u64,
}
impl Metrics {
    pub const EMPTY: Self = Self {
        calls: 0,
        blocked: 0,
        immediate_ready: 0,
        immediate_timeout: 0,
        wake_ready: 0,
        wake_timeout: 0,
        wake_error: 0,
        idle_irqs: 0,
    };
}

// Only the pure decoder/evaluator above is included in the host test harness.
// The native adapter is compiled against the real m3 State and copy boundary.
#[cfg(not(test))]
pub(super) fn syscall(state: &mut super::State, slot: usize, args: [u64; 6]) -> (i64, u64) {
    let before = super::m4::observe_authority();
    state.wait_metrics.calls = state.wait_metrics.calls.saturating_add(1);
    let result = register(state, slot, args);
    let unchanged = before.matches_current();
    // All results, including a successful/pending wait, preserve authority.
    // Erase and release before the scheduler can park or resume this process.
    drop(before);
    assert!(
        unchanged,
        "IPC wait registration changed authority/queue state"
    );
    match result {
        Ok(Some(mask)) => {
            if mask == 0 {
                state.wait_metrics.immediate_timeout =
                    state.wait_metrics.immediate_timeout.saturating_add(1);
            } else {
                state.wait_metrics.immediate_ready =
                    state.wait_metrics.immediate_ready.saturating_add(1);
            }
            (0, mask)
        }
        Ok(None) => {
            state.wait_metrics.blocked = state.wait_metrics.blocked.saturating_add(1);
            // This result is not returned to userspace until refresh supplies
            // the actual wake result and makes the process Ready again.
            (0, 0)
        }
        Err(error) => (error.code(), 0),
    }
}

#[cfg(not(test))]
fn register(state: &mut super::State, slot: usize, args: [u64; 6]) -> Result<Option<u64>, Error> {
    assert!(state.processes[slot].wait.is_none());
    let count = validate_arguments(args, state.ticks)?;
    let mut wire = [0; WIRE_BYTES];
    if count != 0 {
        let length = count * CONDITION_BYTES;
        let plan = super::m4::plan(state, slot, args[0], length as u64, false)?;
        let snapshot = super::m4::read_plan(&plan);
        wire[..length].copy_from_slice(&snapshot[..length]);
        // The CopyPlan and pointer have no role after this scope. The retained
        // WaitSet below contains only copied numeric values.
    }
    let pid = state.processes[slot].pid;
    let waiting = WaitSet::decode(pid, state.ticks, args[2], count, &wire)?;
    let result = waiting.evaluate(super::m4::engine(), pid, state.ticks)?;
    if result.is_none() {
        state.processes[slot].wait = Some(waiting);
    }
    Ok(result)
}

#[cfg(not(test))]
pub(super) fn refresh(state: &mut super::State) {
    for slot in 0..state.amount {
        if state.processes[slot].status != super::Status::Waiting {
            continue;
        }
        let waiting = state.processes[slot]
            .wait
            .expect("Waiting owns scalar conditions");
        let pid = state.processes[slot].pid;
        let result = waiting.evaluate(super::m4::engine(), pid, state.ticks);
        let (primary, secondary) = match result {
            Ok(None) => continue,
            Ok(Some(mask)) => {
                if mask == 0 {
                    state.wait_metrics.wake_timeout =
                        state.wait_metrics.wake_timeout.saturating_add(1);
                } else {
                    state.wait_metrics.wake_ready = state.wait_metrics.wake_ready.saturating_add(1);
                }
                (0, mask)
            }
            Err(error) => {
                state.wait_metrics.wake_error = state.wait_metrics.wake_error.saturating_add(1);
                (error.code(), 0)
            }
        };
        let process = &mut state.processes[slot];
        process.wait = None;
        process.context[14] = primary as u64;
        process.context[12] = secondary;
        process.status = super::Status::Ready;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use capability_core::{Message, QUEUE_CAPACITY};
    fn run(f: impl FnOnce(&mut Engine) + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut engine = Engine::new();
                for pid in 1..=3 {
                    engine.register(pid).unwrap();
                }
                f(&mut engine);
                assert!(engine.check_invariants());
            })
            .unwrap()
            .join()
            .unwrap();
    }
    fn wire(entries: &[(u64, u64)]) -> [u8; WIRE_BYTES] {
        let mut bytes = [0; WIRE_BYTES];
        for ((handle, interest), entry) in entries.iter().zip(bytes.chunks_exact_mut(16)) {
            entry[..8].copy_from_slice(&handle.to_le_bytes());
            entry[8..].copy_from_slice(&interest.to_le_bytes());
        }
        bytes
    }
    #[test]
    fn scalar_arguments_and_every_interest_are_checked_before_registration() {
        assert_eq!(validate_arguments([0, 0, 10, 0, 0, 0], 10), Ok(0));
        assert_eq!(
            validate_arguments([1, 0, 10, 0, 0, 0], 10),
            Err(Error::Invalid)
        );
        assert_eq!(
            validate_arguments([0, 17, 10, 0, 0, 0], 10),
            Err(Error::Invalid)
        );
        assert_eq!(
            validate_arguments([0, u64::MAX, 10, 0, 0, 0], 10),
            Err(Error::Invalid)
        );
        for reserved in 3..6 {
            let mut args = [0, 0, 10, 0, 0, 0];
            args[reserved] = 1;
            assert_eq!(validate_arguments(args, 10), Err(Error::Invalid));
        }
        assert_eq!(validate_arguments([1, 16, 1010, 0, 0, 0], 10), Ok(16));
        assert_eq!(
            validate_arguments([1, 16, 1011, 0, 0, 0], 10),
            Err(Error::Invalid)
        );
        assert_eq!(validate_arguments([0, 0, 0, 0, 0, 0], 10), Ok(0));
        for index in 0..16 {
            for invalid in [0, SEND | RECV, u64::MAX] {
                let mut entries = [(1, SEND); 16];
                entries[index].1 = invalid;
                assert_eq!(
                    WaitSet::decode(1, 10, 11, 16, &wire(&entries)),
                    Err(Error::Invalid)
                );
            }
        }
        assert_eq!(
            WaitSet::decode(0, 10, 11, 0, &[0; 256]),
            Err(Error::Invalid)
        );
        assert_eq!(
            WaitSet::decode(1, 10, 11, 17, &[0; 256]),
            Err(Error::Invalid)
        );
    }
    #[test]
    fn absolute_expiration_horizon_and_clock_rollback_are_not_renewed() {
        run(|engine| {
            let sleep = WaitSet::decode(1, 10, 11, 0, &[0; 256]).unwrap();
            assert_eq!(sleep.evaluate(engine, 1, 10), Ok(None));
            assert_eq!(sleep.evaluate(engine, 1, 11), Ok(Some(0)));
            assert_eq!(sleep.evaluate(engine, 1, 100), Ok(Some(0)));
            assert_eq!(sleep.evaluate(engine, 1, 9), Err(Error::Invalid));
            assert_eq!(
                WaitSet::decode(1, 10, 0, 0, &[0; 256])
                    .unwrap()
                    .evaluate(engine, 1, 10),
                Ok(Some(0))
            );
            assert!(WaitSet::decode(1, u64::MAX - 1, u64::MAX, 0, &[0; 256]).is_ok());
            let root = engine.create_endpoint(1).unwrap();
            let ready = WaitSet::decode(1, 10, 10, 1, &wire(&[(root, SEND)])).unwrap();
            assert_eq!(ready.evaluate(engine, 1, 10), Ok(Some(1)));
        });
    }
    #[test]
    fn all_caps_are_validated_even_after_an_earlier_ready_condition_or_expiry() {
        run(|engine| {
            let root = engine.create_endpoint(1).unwrap();
            let bad = WaitSet::decode(1, 10, 10, 2, &wire(&[(root, SEND), (0, RECV)])).unwrap();
            let before = engine.clone();
            assert_eq!(bad.evaluate(engine, 1, 10), Err(Error::BadHandle));
            assert_eq!(*engine, before);
            let all = WaitSet::decode(1, 10, 11, 16, &wire(&[(root, SEND); 16])).unwrap();
            assert_eq!(all.evaluate(engine, 1, 10), Ok(Some(0xffff)));
        });
    }
    #[test]
    fn copied_snapshot_pid_and_handle_generation_cannot_rebind() {
        run(|engine| {
            let root = engine.create_endpoint(1).unwrap();
            let mut input = wire(&[(root, RECV)]);
            let waiting = WaitSet::decode(1, 10, 20, 1, &input).unwrap();
            input[..16].fill(0xff);
            assert_eq!(waiting.conditions[0].handle, root);
            assert_eq!(waiting.evaluate(engine, 2, 11), Err(Error::BadHandle));
            engine.close(1, root).unwrap();
            let replacement = engine.create_endpoint(1).unwrap();
            assert_eq!(root as u32, replacement as u32);
            assert_ne!(root, replacement);
            assert_eq!(waiting.evaluate(engine, 1, 11), Err(Error::BadHandle));
        });
    }
    #[test]
    fn real_queue_changes_between_poll_and_registration_are_observed() {
        run(|engine| {
            let root = engine.create_endpoint(1).unwrap();
            let sender = engine.bootstrap_delegate(1, root, 2, SEND).unwrap();
            let waiting = WaitSet::decode(1, 10, 20, 1, &wire(&[(root, RECV)])).unwrap();
            assert_eq!(waiting.evaluate(engine, 1, 10), Ok(None));
            engine.send(2, sender, &Message::new()).unwrap();
            let before = engine.clone();
            assert_eq!(waiting.evaluate(engine, 1, 10), Ok(Some(1)));
            assert_eq!(*engine, before);
            assert_eq!(engine.receive(1, root).unwrap().sender_pid, 2);
        });
    }
    #[test]
    fn full_sender_is_woken_by_revocation_without_queue_progress() {
        run(|engine| {
            let root = engine.create_endpoint(1).unwrap();
            let sender = engine.bootstrap_delegate(1, root, 2, SEND).unwrap();
            for _ in 0..QUEUE_CAPACITY {
                engine.send(2, sender, &Message::new()).unwrap();
            }
            let waiting = WaitSet::decode(2, 10, 20, 1, &wire(&[(sender, SEND)])).unwrap();
            assert_eq!(waiting.evaluate(engine, 2, 10), Ok(None));
            engine.revoke(1, root).unwrap();
            assert_eq!(engine.stats().queued_messages, QUEUE_CAPACITY);
            assert_eq!(waiting.evaluate(engine, 2, 11), Err(Error::Revoked));
        });
    }
    #[test]
    fn multiple_senders_observe_space_without_a_reservation() {
        run(|engine| {
            let root = engine.create_endpoint(1).unwrap();
            let a = engine.bootstrap_delegate(1, root, 2, SEND).unwrap();
            let b = engine.bootstrap_delegate(1, root, 3, SEND).unwrap();
            for _ in 0..QUEUE_CAPACITY {
                engine.send(2, a, &Message::new()).unwrap();
            }
            let a = WaitSet::decode(2, 10, 20, 1, &wire(&[(a, SEND)])).unwrap();
            let b = WaitSet::decode(3, 10, 20, 1, &wire(&[(b, SEND)])).unwrap();
            engine.receive(1, root).unwrap();
            assert_eq!(a.evaluate(engine, 2, 11), Ok(Some(1)));
            assert_eq!(b.evaluate(engine, 3, 11), Ok(Some(1)));
            engine
                .send(2, a.conditions[0].handle, &Message::new())
                .unwrap();
            assert_eq!(b.evaluate(engine, 3, 11), Ok(None));
        });
    }
}
