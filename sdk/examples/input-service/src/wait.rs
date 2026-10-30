//! Read-only scheduling plans; never consumes an IPC message or changes a token.
use nieuw_os_sdk_runtime::{ReceiveEndpoint, SendEndpoint};

pub const MAX_CONDITIONS: usize = 16;
pub const HORIZON: u64 = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interest {
    Send(SendEndpoint),
    Receive(ReceiveEndpoint),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Legacy,
    Wait,
}
impl Mode {
    /// Only the two explicitly implemented ABI versions are accepted.
    pub const fn from_abi(info: u64) -> Option<Self> {
        match info {
            0x1_0000_0008 => Some(Self::Legacy),
            0x1_0000_000a => Some(Self::Wait),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Clock,
    Capacity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    interests: [Option<Interest>; MAX_CONDITIONS],
    len: usize,
    deadline: u64,
}
impl Plan {
    pub fn until(deadline: u64) -> Result<Self, Error> {
        if deadline == 0 {
            return Err(Error::Clock);
        }
        Ok(Self {
            interests: [None; MAX_CONDITIONS],
            len: 0,
            deadline,
        })
    }
    pub fn idle(now: u64) -> Result<Self, Error> {
        Self::until(now.checked_add(HORIZON).ok_or(Error::Clock)?)
    }
    pub const fn deadline(&self) -> u64 {
        self.deadline
    }
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        self.interests[..self.len].iter().map(|i| i.unwrap())
    }
    pub fn add(&mut self, interest: Interest) -> Result<(), Error> {
        if self.interests().any(|i| i == interest) {
            return Ok(());
        }
        if self.len == MAX_CONDITIONS {
            return Err(Error::Capacity);
        }
        self.interests[self.len] = Some(interest);
        self.len += 1;
        Ok(())
    }
    pub fn sooner(&mut self, deadline: u64) -> Result<(), Error> {
        if deadline == 0 {
            return Err(Error::Clock);
        }
        self.deadline = self.deadline.min(deadline);
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Fixed 16-condition value (272 bytes); these no-allocator daemons deliberately
// keep it on their existing 64 KiB stack instead of adding a heap/indirection.
#[allow(clippy::large_enum_variant)]
pub enum Action {
    Run,
    Park(Plan),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParkError {
    Clock(nieuw_os_sdk_runtime::Error),
    Yield(nieuw_os_sdk_runtime::Error),
    Wait(nieuw_os_sdk_runtime::Error),
    Mask,
}

/// Preserve the existing `now - started > limit` admission boundary exactly.
pub fn expires_after(started: u64, limit: u64) -> Result<u64, Error> {
    started
        .checked_add(limit)
        .and_then(|n| n.checked_add(1))
        .ok_or(Error::Clock)
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn park(plan: &Plan) -> Result<(), ParkError> {
    use nieuw_os_sdk_runtime::{native, WaitCondition};
    let now = native::monotonic_ticks().map_err(ParkError::Clock)?;
    if now >= plan.deadline {
        return Ok(());
    }
    let convert = |interest| match interest {
        Interest::Send(endpoint) => WaitCondition::send(endpoint),
        Interest::Receive(endpoint) => WaitCondition::receive(endpoint),
    };
    let mask = if let Some(first) = plan.interests().next() {
        let mut conditions = [convert(first); MAX_CONDITIONS];
        for (i, interest) in plan.interests().enumerate() {
            conditions[i] = convert(interest);
        }
        native::ipc_wait(&conditions[..plan.len], plan.deadline).map_err(ParkError::Wait)?
    } else {
        native::ipc_wait(&[], plan.deadline).map_err(ParkError::Wait)?
    };
    let allowed = ((1u32 << plan.len) - 1) as u16;
    if mask & !allowed != 0 {
        return Err(ParkError::Mask);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::{
        policy::{Client, Policy, Replies},
        source::{Configuration as SourceConfiguration, Progress, RawInput, Source},
        Ipc,
    };
    use capability_core::{Engine, Message, RECV, SEND};
    use nieuw_os_display_client::{
        wire::{Reset, Sample},
        Error as RawError, ResetProgress,
    };
    use nieuw_os_input_core::{
        wire::{self, Operation, Request},
        Actor, ClientBinding, Configuration,
    };
    use nieuw_os_sdk_runtime::{Error as IpcError, Handle, Outgoing, Received};
    use std::{boxed::Box, cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
    #[test]
    fn exact_abi_modes_no_future_or_old_version_guessing() {
        assert_eq!(Mode::from_abi(0x1_0000_0008), Some(Mode::Legacy));
        assert_eq!(Mode::from_abi(0x1_0000_000a), Some(Mode::Wait));
        for v in [
            0,
            0x1_0000_0007,
            0x1_0000_0009,
            0x1_0000_000b,
            0x2_0000_000a,
            u64::MAX,
        ] {
            assert_eq!(Mode::from_abi(v), None);
        }
    }
    #[test]
    fn bounded_plan_deduplicates_exact_conditions_and_keeps_absolute_deadline() {
        let mut p = Plan::idle(20).unwrap();
        for i in 1..=16 {
            p.add(Interest::Send(SendEndpoint::from_handle(
                Handle::from_raw(0x1_0000_0000 | i).unwrap(),
            )))
            .unwrap();
        }
        let old = p;
        let first = p.interests().next().unwrap();
        p.add(first).unwrap();
        assert_eq!(p, old);
        assert_eq!(
            p.add(Interest::Receive(ReceiveEndpoint::from_handle(
                Handle::from_raw(0x1_0000_0001).unwrap()
            ))),
            Err(Error::Capacity)
        );
        assert_eq!(p, old);
        p.sooner(80).unwrap();
        p.sooner(90).unwrap();
        assert_eq!(p.deadline(), 80);
        assert_eq!(expires_after(10, 200), Ok(211));
        assert_eq!(expires_after(u64::MAX - 200, 200), Err(Error::Clock));
        assert_eq!(Plan::idle(u64::MAX), Err(Error::Clock));
    }
    const POLICY: u64 = 12;
    fn recv(h: u64) -> ReceiveEndpoint {
        ReceiveEndpoint::from_handle(Handle::from_raw(h).unwrap())
    }
    fn send(h: u64) -> SendEndpoint {
        SendEndpoint::from_handle(Handle::from_raw(h).unwrap())
    }
    struct Port {
        pid: u64,
        engine: Rc<RefCell<Box<Engine>>>,
        sends: usize,
    }
    impl Ipc for Port {
        fn send(&mut self, e: SendEndpoint, m: &Outgoing) -> Result<(), IpcError> {
            self.sends += 1;
            self.engine
                .borrow_mut()
                .send(
                    self.pid,
                    e.handle().raw(),
                    &Message::decode(m.as_bytes()).unwrap(),
                )
                .map_err(|e| IpcError::Kernel(e.code()))
        }
        fn receive(&mut self, e: ReceiveEndpoint) -> Result<Received, IpcError> {
            let m = self
                .engine
                .borrow_mut()
                .receive(self.pid, e.handle().raw())
                .map_err(|e| IpcError::Kernel(e.code()))?;
            Received::from_bytes(m.encode(), u64::from(m.len))
        }
        fn close(&mut self, h: Handle) -> Result<(), IpcError> {
            self.engine
                .borrow_mut()
                .close(self.pid, h.raw())
                .map_err(|e| IpcError::Kernel(e.code()))
        }
    }
    struct Model {
        engine: Rc<RefCell<Box<Engine>>>,
        policy: Policy,
        io: Port,
        peers: Vec<Port>,
        ingress: [u64; 4],
        requests: Vec<u64>,
        responses: Vec<u64>,
        replies: Vec<u64>,
    }
    impl Model {
        fn new(clients: usize) -> Self {
            let mut e = Box::new(Engine::new());
            for pid in 1..=(3 + clients) as u64 {
                e.register(pid).unwrap();
            }
            e.register(POLICY).unwrap();
            let ingress = core::array::from_fn(|_| e.create_endpoint(POLICY).unwrap());
            let requests = (0..3 + clients)
                .map(|i| {
                    e.bootstrap_delegate(POLICY, ingress[i.min(3)], i as u64 + 1, SEND)
                        .unwrap()
                })
                .collect::<Vec<_>>();
            let responses = (1..=(3 + clients) as u64)
                .map(|pid| e.create_endpoint(pid).unwrap())
                .collect::<Vec<_>>();
            let replies = responses
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    e.bootstrap_delegate(i as u64 + 1, *h, POLICY, SEND)
                        .unwrap()
                })
                .collect::<Vec<_>>();
            let config = Configuration {
                epoch: 70,
                source: Actor {
                    endpoint: recv(ingress[0]),
                    pid: 1,
                },
                compositor: Actor {
                    endpoint: recv(ingress[1]),
                    pid: 2,
                },
                supervisor: Actor {
                    endpoint: recv(ingress[2]),
                    pid: 3,
                },
                applications: recv(ingress[3]),
                width: 800,
                height: 600,
            };
            let table = (0..clients)
                .map(|i| Client {
                    binding: ClientBinding {
                        id: 101 + i as u64,
                        pid: 4 + i as u64,
                    },
                    reply: send(replies[3 + i]),
                })
                .collect::<Vec<_>>();
            let policy = Policy::new(
                config,
                Replies {
                    source: send(replies[0]),
                    compositor: send(replies[1]),
                    supervisor: send(replies[2]),
                },
                &table,
            )
            .unwrap();
            let engine = Rc::new(RefCell::new(e));
            let peers = (1..=(3 + clients) as u64)
                .map(|pid| Port {
                    pid,
                    engine: Rc::clone(&engine),
                    sends: 0,
                })
                .collect();
            Self {
                engine: Rc::clone(&engine),
                policy,
                io: Port {
                    pid: POLICY,
                    engine,
                    sends: 0,
                },
                peers,
                ingress,
                requests,
                responses,
                replies,
            }
        }
        fn step(&mut self) {
            self.policy.step(&mut self.io).unwrap();
        }
        fn request(&mut self, peer: usize, op: Operation, id: u64, arg0: u64) {
            let m = Outgoing::new(
                wire::TAG,
                &Request {
                    operation: op,
                    id,
                    epoch: 70,
                    arg0,
                    arg1: 0,
                }
                .encode(),
            )
            .unwrap();
            self.peers[peer]
                .send(send(self.requests[peer]), &m)
                .unwrap();
        }
        fn fill(&mut self, peer: usize) {
            for _ in 0..16 {
                self.io
                    .send(send(self.replies[peer]), &Outgoing::new(0, &[]).unwrap())
                    .unwrap();
            }
        }
        fn plan(&self, now: u64) -> Plan {
            match self.policy.wait_plan(now).unwrap() {
                Action::Park(p) => p,
                _ => panic!("unexpected local work"),
            }
        }
        fn readiness(&self, plan: Plan) -> Result<u16, IpcError> {
            let e = self.engine.borrow();
            let mut mask = 0;
            for (i, c) in plan.interests().enumerate() {
                let (h, r) = match c {
                    Interest::Send(e) => (e.handle(), SEND),
                    Interest::Receive(e) => (e.handle(), RECV),
                };
                if e.endpoint_queue_ready(POLICY, h.raw(), r)
                    .map_err(|e| IpcError::Kernel(e.code()))?
                {
                    mask |= 1 << i;
                }
            }
            Ok(mask)
        }
    }
    #[derive(Default)]
    struct Raw {
        epoch: u64,
        pending: bool,
        samples: VecDeque<Option<Sample>>,
        reads: usize,
    }
    impl RawInput for Raw {
        fn begin_reset(&mut self) -> Result<u64, RawError> {
            self.epoch += 1;
            Ok(self.epoch)
        }
        fn poll_reset(&mut self) -> Result<ResetProgress, RawError> {
            if self.pending {
                Ok(ResetProgress::Pending)
            } else {
                Ok(ResetProgress::Ready(Reset {
                    ticket: self.epoch,
                    epoch: self.epoch,
                    discarded: 0,
                    operations: 30,
                }))
            }
        }
        fn read(&mut self) -> Result<Option<Sample>, RawError> {
            self.reads += 1;
            Ok(self.samples.pop_front().unwrap_or(None))
        }
    }
    fn source(m: &Model) -> Source {
        Source::new(SourceConfiguration {
            epoch: 70,
            policy_pid: POLICY,
            request: send(m.requests[0]),
            response: recv(m.responses[0]),
        })
        .unwrap()
    }
    fn parked(s: &Source, now: u64) -> Plan {
        match s.wait_plan(now).unwrap() {
            Action::Park(p) => p,
            _ => panic!("not blocked"),
        }
    }

    #[test]
    fn source_only_waits_after_observed_block_and_raw_idle_keeps_original_tick() {
        let mut m = Model::new(1);
        let mut s = source(&m);
        let mut raw = Raw::default();
        assert_eq!(s.wait_plan(0).unwrap(), Action::Run);
        s.step(0, &mut raw, &mut m.peers[0]).unwrap();
        raw.pending = true;
        s.step(1, &mut raw, &mut m.peers[0]).unwrap();
        assert_eq!(s.wait_plan(1).unwrap(), Action::Run);
        raw.pending = false;
        s.step(2, &mut raw, &mut m.peers[0]).unwrap();
        assert_eq!(s.wait_plan(2).unwrap(), Action::Run);
        s.step(3, &mut raw, &mut m.peers[0]).unwrap();
        let p = parked(&s, 3);
        assert_eq!(p.deadline(), 203);
        assert_eq!(
            p.interests().collect::<Vec<_>>(),
            [Interest::Receive(recv(m.responses[0]))]
        );
        let counts = s.counts();
        assert_eq!(parked(&s, 150), p);
        assert_eq!(s.counts(), counts);
        m.step();
        assert_eq!(
            s.step(4, &mut raw, &mut m.peers[0]),
            Ok(Progress::Ready { raw_epoch: 1 })
        );
        assert_eq!(s.wait_plan(4).unwrap(), Action::Run);
        assert_eq!(s.step(5, &mut raw, &mut m.peers[0]), Ok(Progress::Idle));
        let idle = parked(&s, 5);
        assert!(idle.is_empty());
        assert_eq!(idle.deadline(), 6);
        assert_eq!(s.wait_plan(6).unwrap(), Action::Run);
        assert_eq!(s.wait_plan(4), Err(crate::source::Fatal::Clock));
        raw.samples.push_back(Some(Sample {
            epoch: 1,
            sequence: 1,
            status: 1,
            byte: 0x1e,
        }));
        s.step(6, &mut raw, &mut m.peers[0]).unwrap();
        assert_eq!(s.wait_plan(6).unwrap(), Action::Run);
    }
    #[test]
    fn source_full_request_queue_waits_send_then_receive_without_replay_or_new_deadline() {
        let mut m = Model::new(1);
        let mut s = source(&m);
        let mut raw = Raw::default();
        for _ in 0..16 {
            m.peers[0]
                .send(send(m.requests[0]), &Outgoing::new(0, &[]).unwrap())
                .unwrap();
        }
        for tick in 0..=2 {
            s.step(tick, &mut raw, &mut m.peers[0]).unwrap();
        }
        let p = parked(&s, 2);
        assert_eq!(
            p.interests().collect::<Vec<_>>(),
            [Interest::Send(send(m.requests[0]))]
        );
        assert_eq!(p.deadline(), 202);
        assert_eq!(parked(&s, 180), p);
        assert_eq!(raw.reads, 0);
        for _ in 0..16 {
            m.engine.borrow_mut().receive(POLICY, m.ingress[0]).unwrap();
        }
        s.step(3, &mut raw, &mut m.peers[0]).unwrap();
        let sends = m.peers[0].sends;
        assert_eq!(
            parked(&s, 3).interests().collect::<Vec<_>>(),
            [Interest::Receive(recv(m.responses[0]))]
        );
        s.step(201, &mut raw, &mut m.peers[0]).unwrap();
        assert_eq!(m.peers[0].sends, sends);
        assert_eq!(s.wait_plan(202).unwrap(), Action::Run);
        assert_eq!(
            s.step(202, &mut raw, &mut m.peers[0]),
            Err(crate::source::Fatal::Deadline)
        );
    }
    #[test]
    fn idle_policy_plan_is_read_only_and_new_queued_work_is_immediately_visible() {
        let mut m = Model::new(2);
        assert_eq!(m.policy.wait_plan(0).unwrap(), Action::Run);
        m.step();
        let before = m.policy.counts();
        let p = m.plan(10);
        assert_eq!(p.len(), 4);
        assert!(p.interests().all(|c| matches!(c, Interest::Receive(_))));
        assert_eq!(m.readiness(p), Ok(0));
        assert_eq!(m.policy.counts(), before);
        m.request(3, Operation::Peek, 1, 0);
        assert_ne!(m.readiness(p).unwrap(), 0);
        let queued = m.engine.borrow().stats();
        let _ = m.plan(20);
        assert_eq!(m.engine.borrow().stats(), queued);
        m.step();
        assert_eq!(m.plan(21).len(), 4);
        assert_eq!(m.policy.counts().handled, before.handled + 1);
    }
    #[test]
    fn only_retained_full_reply_send_is_waited_and_ready_hint_does_not_consume() {
        let mut m = Model::new(2);
        m.step();
        m.fill(3);
        m.request(3, Operation::Peek, 1, 0);
        m.step();
        let p = m.plan(1);
        let interests = p.interests().collect::<Vec<_>>();
        assert_eq!(p.len(), 5);
        assert!(interests.contains(&Interest::Send(send(m.replies[3]))));
        assert!(!interests.contains(&Interest::Send(send(m.replies[4]))));
        assert_eq!(m.readiness(p), Ok(0));
        m.peers[3].receive(recv(m.responses[3])).unwrap();
        assert_ne!(m.readiness(p).unwrap(), 0);
        let counts = m.policy.counts();
        assert_eq!(m.policy.counts(), counts);
        m.step();
        assert_eq!(m.plan(2).len(), 4);
        assert_eq!(m.policy.counts().handled, 1);
    }
    #[test]
    fn blocked_protected_ingress_is_excluded_then_restored_without_lost_work() {
        let mut m = Model::new(1);
        m.step();
        m.fill(0);
        m.request(0, Operation::Sync, 1, 1);
        m.step();
        m.request(0, Operation::Sync, 2, 1);
        let p = m.plan(1);
        assert!(!p
            .interests()
            .any(|i| i == Interest::Receive(recv(m.ingress[0]))));
        assert!(p
            .interests()
            .any(|i| i == Interest::Send(send(m.replies[0]))));
        assert_eq!(m.readiness(p), Ok(0));
        m.peers[0].receive(recv(m.responses[0])).unwrap();
        m.step();
        let next = m.plan(2);
        assert!(next
            .interests()
            .any(|i| i == Interest::Receive(recv(m.ingress[0]))));
        assert_ne!(m.readiness(next).unwrap(), 0);
        m.step();
        assert_eq!(m.policy.counts().handled, 2);
    }
    #[test]
    fn all_eleven_replies_fit_and_closed_condition_is_not_a_positive_mask() {
        let mut m = Model::new(8);
        m.step();
        for peer in 0..11 {
            m.fill(peer);
        }
        m.request(0, Operation::Sync, 1, 1);
        m.request(1, Operation::Focus, 1, 101);
        m.request(2, Operation::SourceDead, 1, 0);
        for peer in 3..11 {
            m.request(peer, Operation::Peek, 1, 0);
        }
        for _ in 0..4 {
            m.step();
        }
        let p = m.plan(10);
        assert_eq!(
            p.interests()
                .filter(|i| matches!(i, Interest::Send(_)))
                .count(),
            11
        );
        assert_eq!(p.len(), 12);
        assert_eq!(m.readiness(p), Ok(0));
        m.engine.borrow_mut().close(1, m.responses[0]).unwrap();
        assert_eq!(m.readiness(p), Err(IpcError::Kernel(-9)));
        m.policy.shutdown();
        assert_eq!(m.policy.wait_plan(11), Err(crate::policy::Fatal::Halted));
    }
    #[test]
    fn lifetime_wait_error_revalidates_and_retires_only_the_actual_bad_app_lane() {
        for fault in [-9, -10, -4] {
            let mut m = Model::new(2);
            m.step();
            m.fill(3);
            m.request(3, Operation::Peek, 1, 0);
            m.step();
            let plan = m.plan(1);
            m.request(4, Operation::Peek, 1, 0);
            match fault {
                -9 => m.engine.borrow_mut().close(4, m.responses[3]).unwrap(),
                -10 => m.engine.borrow_mut().revoke(4, m.responses[3]).unwrap(),
                -4 => m.engine.borrow_mut().close(POLICY, m.replies[3]).unwrap(),
                _ => unreachable!(),
            }
            assert_eq!(m.readiness(plan), Err(IpcError::Kernel(fault)));
            m.policy
                .wait_failed(&mut m.io, IpcError::Kernel(fault))
                .unwrap();
            assert_eq!(m.policy.counts().retired, 1);
            assert_eq!(m.policy.counts().handled, 2);
            assert!(!m
                .plan(2)
                .interests()
                .any(|i| i == Interest::Send(send(m.replies[3]))));
            let other = m.peers[4].receive(recv(m.responses[4])).unwrap();
            assert_eq!(other.sender(), POLICY);
            assert_eq!(other.payload()[24..26], [1, 0]); // Canonical Empty PEEK.
            assert_eq!(
                u64::from_le_bytes(other.payload()[32..40].try_into().unwrap()),
                102
            );
        }
    }
    #[test]
    fn critical_unexplained_or_non_lifetime_wait_fault_never_becomes_poll_success() {
        let mut m = Model::new(1);
        m.step();
        let before = m.policy.counts();
        assert_eq!(
            m.policy.wait_failed(&mut m.io, IpcError::Kernel(-6)),
            Err(crate::policy::Fatal::Wait(IpcError::Kernel(-6)))
        );
        assert_eq!(m.policy.counts(), before);
        assert_eq!(
            m.policy.wait_failed(&mut m.io, IpcError::Kernel(-9)),
            Err(crate::policy::Fatal::Wait(IpcError::Kernel(-9)))
        );
        assert_eq!(m.policy.counts().retired, 0);
        m.engine.borrow_mut().close(POLICY, m.ingress[0]).unwrap();
        assert_eq!(
            m.policy.wait_failed(&mut m.io, IpcError::Kernel(-4)),
            Err(crate::policy::Fatal::Receive(IpcError::Kernel(-4)))
        );
    }
}
