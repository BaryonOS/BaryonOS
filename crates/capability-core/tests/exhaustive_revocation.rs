//! Exhaust all 7^5 operation words (19,607 nonempty prefixes) over three processes.
//! This bound is not an unbounded proof. Mutation control: replace Engine::revoke's
//! marking loop with a no-op; the independent revoked-handle assertion must fail.
use capability_core::{
    Cap, Engine, Error, Message, ALL_RIGHTS, DERIVE, RECV, REVOKE, SEND, TRANSFER,
};
use std::collections::VecDeque;

const DEPTH: usize = 5;
const ACTIONS: usize = 7;
const LIMITED: u64 = SEND | DERIVE | TRANSFER;

#[derive(Clone, Copy)]
struct Handle {
    pid: u64,
    raw: u64,
    rights: u64,
    object: usize,
    closed: bool,
    revoked: bool,
}
#[derive(Clone, Default)]
struct Object {
    closed: bool,
}
#[derive(Clone)]
struct State {
    engine: Box<Engine>,
    handles: Vec<Handle>,
    objects: Vec<Object>,
    current: usize,
    queued: VecDeque<usize>,
    generations: [[u64; 256]; 3],
    ingress: u64,
    sender: u64,
    trace: Vec<usize>,
}
impl State {
    fn new() -> Self {
        let mut engine = Box::new(Engine::new());
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let ingress = engine.create_endpoint(3).unwrap();
        let sender = engine.bootstrap_delegate(3, ingress, 1, SEND).unwrap();
        let mut state = Self {
            engine,
            handles: Vec::new(),
            objects: Vec::new(),
            current: 0,
            queued: VecDeque::new(),
            generations: [[0; 256]; 3],
            ingress,
            sender,
            trace: Vec::new(),
        };
        // Seed the independent channel slots before checking subsequent reuse.
        state.observe_generation(3, ingress);
        state.observe_generation(1, sender);
        state.create();
        state
    }
    fn observe_generation(&mut self, pid: u64, raw: u64) {
        let slot = (raw & 0xffff_ffff) as usize;
        let generation = raw >> 32;
        let previous = &mut self.generations[pid as usize - 1][slot];
        assert!(generation > *previous, "nonmonotonic slot generation");
        *previous = generation;
    }
    fn add(&mut self, pid: u64, raw: u64, rights: u64, object: usize) {
        self.observe_generation(pid, raw);
        self.handles.push(Handle {
            pid,
            raw,
            rights,
            object,
            closed: false,
            revoked: false,
        });
    }
    fn create(&mut self) {
        match self.engine.create_endpoint(1) {
            Ok(raw) => {
                self.current = self.objects.len();
                self.objects.push(Object::default());
                self.add(1, raw, ALL_RIGHTS, self.current);
            }
            Err(Error::Quota) => {} // Four owned endpoints is the actual engine limit.
            other => panic!("unexpected create: {other:?}"),
        }
    }
    fn available(&self, handle: Handle) -> bool {
        !handle.closed && !self.objects[handle.object].closed && !handle.revoked
    }
    fn root(&self) -> Handle {
        *self
            .handles
            .iter()
            .find(|h| h.object == self.current)
            .unwrap()
    }
    fn step(&mut self, action: usize) {
        self.trace.push(action);
        let root = self.root();
        match action {
            0 => self.create(),
            1 => {
                // Process 2 receives a restricted branch of the current root.
                let result = self.engine.bootstrap_delegate(1, root.raw, 2, LIMITED);
                if self.available(root) {
                    self.add(2, result.unwrap(), LIMITED, root.object);
                } else {
                    assert!(result.is_err());
                }
            }
            2 => {
                // A genuine queued transfer, not a shortcut that grants directly.
                let source = self
                    .handles
                    .iter()
                    .rev()
                    .find(|h| h.pid == 1)
                    .copied()
                    .unwrap();
                let mut message = Message::new();
                message.tag = source.object as u64;
                message.cap_count = 1;
                message.caps[0] = Cap {
                    handle: source.raw,
                    rights: SEND,
                };
                let result = self.engine.send(1, self.sender, &message);
                if self.available(source) {
                    result.unwrap();
                    self.queued.push_back(source.object);
                } else {
                    assert!(result.is_err());
                }
            }
            3 => {
                // A closed object may still arrive, but its received handle must be unusable.
                let result = self.engine.receive(3, self.ingress);
                if let Some(object) = self.queued.pop_front() {
                    let message = result.unwrap();
                    assert_eq!(message.tag, object as u64);
                    assert_eq!(message.cap_count, 1);
                    assert_eq!(message.caps[0].rights, SEND);
                    self.add(3, message.caps[0].handle, SEND, object);
                } else {
                    assert_eq!(result, Err(Error::Again), "trace={:?}", self.trace);
                }
            }
            4 => {
                // Closing the current root removes its only RECV authority.
                let result = self.engine.close(1, root.raw);
                if root.closed {
                    assert!(result.is_err());
                } else {
                    result.unwrap();
                    self.objects[root.object].closed = true;
                    self.handles
                        .iter_mut()
                        .find(|h| h.raw == root.raw && h.pid == 1)
                        .unwrap()
                        .closed = true;
                }
            }
            5 => {
                let result = self.engine.revoke(1, root.raw);
                if self.available(root) {
                    result.unwrap();
                    // Revocation cuts existing descendants; the issuer root survives and
                    // may explicitly delegate a fresh branch afterwards.
                    for h in &mut self.handles {
                        if h.object == root.object && h.pid != 1 {
                            h.revoked = true;
                        }
                    }
                    self.queued.retain(|object| *object != root.object);
                } else {
                    assert!(result.is_err());
                }
            }
            6 => {
                // A restricted recipient must not acquire RECV/REVOKE by deriving.
                for h in self.handles.iter().filter(|h| h.pid != 1) {
                    assert!(self.engine.derive(h.pid, h.raw, ALL_RIGHTS).is_err());
                }
            }
            _ => unreachable!(),
        }
        self.check();
    }
    fn check(&self) {
        assert!(self.engine.check_invariants());
        for &h in &self.handles {
            let available = self.available(h);
            assert_eq!(
                self.engine.check_endpoint(h.pid, h.raw, 0).is_ok(),
                available,
                "stale/revoked handle regained authority: pid={} handle={} trace={:?}",
                h.pid,
                h.raw,
                self.trace
            );
            for right in [SEND, RECV, DERIVE, TRANSFER, REVOKE] {
                assert_eq!(
                    self.engine.check_endpoint(h.pid, h.raw, right).is_ok(),
                    available && h.rights & right != 0,
                    "rights amplification/loss"
                );
            }
        }
    }
}
fn explore(state: &State, depth: usize, counts: &mut [usize; DEPTH + 1]) {
    counts[depth] += 1;
    if depth == DEPTH {
        return;
    }
    for action in 0..ACTIONS {
        let mut next = state.clone();
        next.step(action);
        explore(&next, depth + 1, counts);
    }
}
#[test]
fn exhaustive_three_process_revocation() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let mut counts = [0; DEPTH + 1];
            explore(&State::new(), 0, &mut counts);
            assert_eq!(counts, [1, 7, 49, 343, 2401, 16807]);
            assert_eq!(counts[1..].iter().sum::<usize>(), 19607);
        })
        .unwrap()
        .join()
        .unwrap();
}
