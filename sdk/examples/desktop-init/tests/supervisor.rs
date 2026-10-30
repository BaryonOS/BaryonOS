//! Host protocol/failure models over actual capability-engine IPC envelopes.
//! Peer messages are protocol test data, never proof of real focus/display/I/O.
use capability_core::{Engine, Message, SEND};
use nieuw_os_native_desktop_init::{
    boot::{self, Bootstrap},
    controller,
    supervisor::{Fatal, Phase, Supervisor, BOOT_TICKS},
    wire::{self, StopRequest, StopStatus},
};
use nieuw_os_native_input_service::Ipc;
use nieuw_os_sdk_runtime::{
    Capability, Error, Handle, Outgoing, ReceiveEndpoint, Received, Rights, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
fn h(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(h(raw))
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(h(raw))
}
const SURFACE: u64 = 0x5355520000000001;
const POLICY: u64 = 0x494e500000000001;
struct Port {
    pid: u64,
    engine: Rc<RefCell<Box<Engine>>>,
    sends: usize,
    reads: usize,
    closed: Vec<u64>,
}
impl Ipc for Port {
    fn send(&mut self, e: SendEndpoint, m: &Outgoing) -> Result<(), Error> {
        self.sends += 1;
        self.engine
            .borrow_mut()
            .send(
                self.pid,
                e.handle().raw(),
                &Message::decode(m.as_bytes()).unwrap(),
            )
            .map_err(|e| Error::Kernel(e.code()))
    }
    fn receive(&mut self, e: ReceiveEndpoint) -> Result<Received, Error> {
        self.reads += 1;
        let m = self
            .engine
            .borrow_mut()
            .receive(self.pid, e.handle().raw())
            .map_err(|e| Error::Kernel(e.code()))?;
        Received::from_bytes(m.encode(), u64::from(m.len))
    }
    fn close(&mut self, h: Handle) -> Result<(), Error> {
        self.closed.push(h.raw());
        self.engine
            .borrow_mut()
            .close(self.pid, h.raw())
            .map_err(|e| Error::Kernel(e.code()))
    }
}
struct Model {
    engine: Rc<RefCell<Box<Engine>>>,
    ports: Vec<Port>,
    config: Bootstrap,
    registry_send: [u64; 6],
    app_recv: u64,
    comp_recv: u64,
    controller_send: u64,
    controller_recv: u64,
}
impl Model {
    fn new(optional: bool) -> Self {
        let mut e = Box::new(Engine::new());
        for p in [1, 4, 5, 6, 7, 8, 9] {
            e.register(p).unwrap();
        }
        let registry = e.create_endpoint(1).unwrap();
        let registry_send = core::array::from_fn(|i| {
            e.bootstrap_delegate(1, registry, [4, 5, 6, 7, 8, 9][i], SEND)
                .unwrap()
        });
        let app_recv = e.create_endpoint(4).unwrap();
        let app_control = e.bootstrap_delegate(4, app_recv, 1, SEND).unwrap();
        let comp_recv = e.create_endpoint(5).unwrap();
        let comp_control = e.bootstrap_delegate(5, comp_recv, 1, SEND).unwrap();
        let init_control = e.create_endpoint(1).unwrap();
        let controller_send = e.bootstrap_delegate(1, init_control, 8, SEND).unwrap();
        let controller_recv = e.create_endpoint(8).unwrap();
        let controller_reply = e.bootstrap_delegate(8, controller_recv, 1, SEND).unwrap();
        let mut words = [0; 128];
        for (o, v) in [
            (32, boot::MAGIC),
            (40, 1),
            (48, registry),
            (56, app_control),
            (64, comp_control),
            (80, 4),
            (88, 5),
            (96, 6),
            (104, 7),
            (112, SURFACE),
            (120, POLICY),
            (128, 1),
            (136, 101),
        ] {
            words[o / 8] = v;
        }
        if optional {
            words[72 / 8] = init_control;
            words[144 / 8] = controller_reply;
            words[152 / 8] = 8;
        }
        let config = Bootstrap::decode(&words).unwrap();
        let engine = Rc::new(RefCell::new(e));
        let ports = [1, 4, 5, 6, 7, 8, 9]
            .into_iter()
            .map(|pid| Port {
                pid,
                engine: engine.clone(),
                sends: 0,
                reads: 0,
                closed: vec![],
            })
            .collect();
        Self {
            engine,
            ports,
            config,
            registry_send,
            app_recv,
            comp_recv,
            controller_send,
            controller_recv,
        }
    }
    fn supervisor(&self) -> Supervisor {
        Supervisor::new(self.config, 0).unwrap()
    }
    fn role(&mut self, index: usize, role: u16, epoch: u64, raw: u64, generation: u64) {
        let mut b = [0; 32];
        b[..4].copy_from_slice(b"IPD1");
        b[4..6].copy_from_slice(&1u16.to_le_bytes());
        b[6..8].copy_from_slice(&role.to_le_bytes());
        for (o, v) in [(8, epoch), (16, generation), (24, raw)] {
            b[o..o + 8].copy_from_slice(&v.to_le_bytes());
        }
        self.ports[index + 1]
            .send(
                send(self.registry_send[index]),
                &Outgoing::new(wire::IPD, &b).unwrap(),
            )
            .unwrap();
    }
    fn app(&mut self, operation: u8) {
        self.ports[1]
            .send(send(self.registry_send[0]), &wire::app(operation).unwrap())
            .unwrap();
    }
    fn ready(&mut self) {
        self.role(2, 1, POLICY, 1, 1);
        self.role(3, 2, POLICY, 0, 1);
        self.role(1, 3, SURFACE, 1, 1);
        self.app(wire::READY);
    }
    fn ack(&mut self, retire: bool) {
        let m = self.ports[2].receive(recv(self.comp_recv)).unwrap();
        assert_eq!(m.sender(), 1);
        let mut b = m.payload().to_vec();
        assert_eq!(u16::from_le_bytes([b[6], b[7]]), if retire { 3 } else { 1 });
        b[6..8].copy_from_slice(&(0x8000 | if retire { 3u16 } else { 1u16 }).to_le_bytes());
        b[40..48].copy_from_slice(&(if retire { 2u64 } else { 1u64 }).to_le_bytes());
        self.ports[2]
            .send(
                send(self.registry_send[1]),
                &Outgoing::new(wire::CMP, &b).unwrap(),
            )
            .unwrap();
    }
    fn running(&mut self, s: &mut Supervisor) {
        self.ready();
        assert_eq!(s.step(0, &mut self.ports[0]), Ok(Phase::Focus));
        assert!(self.ports[1]
            .receive(recv(self.app_recv))
            .unwrap_err()
            .is_again());
        self.ack(false);
        assert_eq!(s.step(1, &mut self.ports[0]), Ok(Phase::Running));
        assert_eq!(
            self.ports[1]
                .receive(recv(self.app_recv))
                .unwrap()
                .payload(),
            [wire::GO]
        );
    }
    fn stop(&mut self, s: &mut Supervisor, id: u64, tick: u64) {
        let request = StopRequest {
            id,
            epoch: SURFACE,
            generation: 1,
        };
        self.ports[5]
            .send(
                send(self.controller_send),
                &Outgoing::new(wire::CONTROL, &request.encode()).unwrap(),
            )
            .unwrap();
        assert_eq!(s.step(tick, &mut self.ports[0]), Ok(Phase::Stop));
        assert_eq!(
            self.ports[1]
                .receive(recv(self.app_recv))
                .unwrap()
                .payload(),
            [wire::STOP]
        );
    }
    fn controller(&self) -> controller::Controller {
        controller::Controller::new(
            controller::Configuration {
                generation: 1,
                send: send(self.controller_send),
                receive: recv(self.controller_recv),
                init_pid: 1,
                surface_epoch: SURFACE,
            },
            0,
        )
        .unwrap()
    }
}
fn missing_role_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.role(2, 1, POLICY, 1, 1);
    m.role(3, 2, POLICY, 0, 1);
    m.app(wire::READY);
    assert_eq!(s.step(0, &mut m.ports[0]), Ok(Phase::Boot));
    assert_eq!(s.counts().go_sent, 0);
    assert_eq!(s.step(BOOT_TICKS, &mut m.ports[0]), Err(Fatal::Deadline));
    assert!(m.ports[2]
        .receive(recv(m.comp_recv))
        .unwrap_err()
        .is_again());
}
fn bad_role_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.role(1, 1, POLICY, 1, 1);
    m.role(2, 1, SURFACE, 1, 1);
    m.role(3, 2, POLICY, 0, 2);
    m.role(2, 1, POLICY, 0, 1);
    s.step(0, &mut m.ports[0]).unwrap();
    assert_eq!(
        s.counts().source_ready + s.counts().policy_ready + s.counts().compositor_ready,
        0
    );
    assert_eq!(s.counts().rejected, 4);
    assert_eq!(s.counts().go_sent, 0);
}
fn no_replay_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.ready();
    s.step(0, &mut m.ports[0]).unwrap();
    let sent = m.ports[0].sends;
    for t in 1..100 {
        s.step(t, &mut m.ports[0]).unwrap();
    }
    assert_eq!(m.ports[0].sends, sent);
    assert_eq!(s.counts().focus_ack, 0);
    assert_eq!(s.counts().go_sent, 0);
    assert_eq!(s.step(500, &mut m.ports[0]), Err(Fatal::Deadline));
}
fn wrong_ack_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.ready();
    s.step(0, &mut m.ports[0]).unwrap();
    let focus = m.ports[2].receive(recv(m.comp_recv)).unwrap();
    let mut b = focus.payload().to_vec();
    b[6..8].copy_from_slice(&0x8001u16.to_le_bytes());
    b[40..48].copy_from_slice(&1u64.to_le_bytes());
    m.ports[5]
        .send(
            send(m.registry_send[4]),
            &Outgoing::new(wire::CMP, &b).unwrap(),
        )
        .unwrap();
    s.step(1, &mut m.ports[0]).unwrap();
    assert_eq!(s.phase(), Phase::Focus);
    assert_eq!(s.counts().go_sent, 0);
    b[8..16].copy_from_slice(&2u64.to_le_bytes());
    m.ports[2]
        .send(
            send(m.registry_send[1]),
            &Outgoing::new(wire::CMP, &b).unwrap(),
        )
        .unwrap();
    assert_eq!(s.step(2, &mut m.ports[0]), Err(Fatal::Protocol));
    assert_eq!(s.counts().go_sent, 0);
}
fn go_backpressure_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.ready();
    s.step(0, &mut m.ports[0]).unwrap();
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.ports[0].send(m.config.app_control, &filler).unwrap();
    }
    m.ack(false);
    assert_eq!(s.step(1, &mut m.ports[0]), Ok(Phase::Go));
    assert_eq!(s.counts().focus_ack, 1);
    assert_eq!(s.counts().go_sent, 0);
    for _ in 0..16 {
        m.ports[1].receive(recv(m.app_recv)).unwrap();
    }
    assert_eq!(s.step(2, &mut m.ports[0]), Ok(Phase::Running));
    assert_eq!(s.counts().go_sent, 1);
    assert_eq!(
        m.ports[1].receive(recv(m.app_recv)).unwrap().payload(),
        [wire::GO]
    );
    s.step(3, &mut m.ports[0]).unwrap();
    assert!(m.ports[1].receive(recv(m.app_recv)).unwrap_err().is_again());
}
fn refused_impl() {
    let mut m = Model::new(true);
    let mut s = m.supervisor();
    m.running(&mut s);
    m.stop(&mut s, 1, 2);
    m.app(wire::STOP_REFUSED);
    assert_eq!(s.step(3, &mut m.ports[0]), Ok(Phase::Running));
    assert_eq!(s.counts().refused, 1);
    assert_eq!(s.counts().app_done, 0);
    assert_eq!(s.counts().retire_ack, 0);
    let response = m.ports[5].receive(recv(m.controller_recv)).unwrap();
    assert_eq!(response.sender(), 1);
    assert_eq!(
        u64::from_le_bytes(response.payload()[32..40].try_into().unwrap()),
        1
    );
    assert!(m.ports[2]
        .receive(recv(m.comp_recv))
        .unwrap_err()
        .is_again());
}
fn stop_chain_impl() {
    let mut m = Model::new(true);
    let mut s = m.supervisor();
    let mut c = m.controller();
    m.running(&mut s);
    assert_eq!(
        c.step(2, 0, &mut m.ports[5]),
        Ok(controller::Phase::Barrier)
    );
    assert_eq!(c.counts(), (0, 0));
    assert_eq!(c.step(3, 1, &mut m.ports[5]), Ok(controller::Phase::Wait));
    assert_eq!(s.step(3, &mut m.ports[0]), Ok(Phase::Stop));
    assert_eq!(
        m.ports[1].receive(recv(m.app_recv)).unwrap().payload(),
        [wire::STOP]
    );
    m.app(wire::DONE);
    assert_eq!(s.step(4, &mut m.ports[0]), Ok(Phase::Retire));
    assert_eq!(s.counts().app_done, 1);
    assert_eq!(c.step(4, 1, &mut m.ports[5]), Ok(controller::Phase::Wait));
    m.ack(true);
    assert_eq!(s.step(5, &mut m.ports[0]), Ok(Phase::Stopped));
    assert_eq!(c.step(5, 1, &mut m.ports[5]), Ok(controller::Phase::Done));
    assert_eq!(s.counts().retire_ack, 1);
    assert_eq!(c.counts(), (1, 1));
}
fn unsolicited_done_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    m.running(&mut s);
    m.app(wire::DONE);
    m.app(wire::STOP_REFUSED);
    s.step(2, &mut m.ports[0]).unwrap();
    assert_eq!(s.phase(), Phase::Running);
    assert_eq!(s.counts().app_done, 0);
    assert_eq!(s.counts().refused, 0);
    assert_eq!(s.counts().stop_sent, 0);
    assert_eq!(s.counts().retire_ack, 0);
}
fn caps_impl() {
    let mut m = Model::new(false);
    let mut s = m.supervisor();
    let cap = m.engine.borrow_mut().create_endpoint(6).unwrap();
    let mut b = [0; 32];
    b[..4].copy_from_slice(b"IPD1");
    b[4..6].copy_from_slice(&1u16.to_le_bytes());
    b[6..8].copy_from_slice(&1u16.to_le_bytes());
    for (o, v) in [(8, POLICY), (16, 1), (24, 1)] {
        b[o..o + 8].copy_from_slice(&v.to_le_bytes());
    }
    let mut message = Outgoing::new(wire::IPD, &b).unwrap();
    message
        .push_cap(Capability {
            handle: h(cap),
            rights: Rights::SEND,
        })
        .unwrap();
    m.ports[3].send(send(m.registry_send[2]), &message).unwrap();
    s.step(0, &mut m.ports[0]).unwrap();
    assert_eq!(s.counts().source_ready, 0);
    assert_eq!(s.counts().closed_caps, 1);
    assert_eq!(m.ports[0].closed.len(), 1);
    assert_eq!(
        m.engine.borrow_mut().close(1, m.ports[0].closed[0]),
        Err(capability_core::Error::BadHandle)
    );
}
fn controller_barrier_impl() {
    let mut m = Model::new(true);
    let mut c = m.controller();
    for t in 0..100 {
        assert_eq!(
            c.step(t, 0, &mut m.ports[5]),
            Ok(controller::Phase::Barrier)
        );
    }
    assert_eq!(m.ports[5].sends, 0);
    assert_eq!(
        c.step(100, 2, &mut m.ports[5]),
        Err(controller::Fatal::Barrier)
    );
    assert_eq!(c.counts(), (0, 0));
    let mut c = m.controller();
    assert_eq!(
        c.step(controller::BARRIER_TICKS, 0, &mut m.ports[5]),
        Err(controller::Fatal::Deadline)
    );
}
fn controller_wrong_reply_impl() {
    let mut m = Model::new(true);
    let mut c = m.controller();
    c.step(0, 1, &mut m.ports[5]).unwrap();
    let request = StopRequest {
        id: 1,
        epoch: SURFACE,
        generation: 1,
    };
    let grant = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(8, m.controller_recv, 9, SEND)
        .unwrap();
    m.ports[6]
        .send(send(grant), &request.reply(StopStatus::Stopped).unwrap())
        .unwrap();
    assert_eq!(
        c.step(1, 1, &mut m.ports[5]),
        Err(controller::Fatal::Protocol)
    );
    assert_eq!(c.counts(), (1, 0));
}
fn controller_no_replay_impl() {
    let mut m = Model::new(true);
    let mut c = m.controller();
    c.step(0, 1, &mut m.ports[5]).unwrap();
    for t in 1..100 {
        c.step(t, 1, &mut m.ports[5]).unwrap();
    }
    assert_eq!(m.ports[5].sends, 1);
    assert_eq!(c.counts(), (1, 0));
    let response = StopRequest {
        id: 1,
        epoch: SURFACE,
        generation: 1,
    }
    .reply(StopStatus::Refused)
    .unwrap();
    m.ports[0]
        .send(m.config.controller.unwrap().reply, &response)
        .unwrap();
    assert_eq!(
        c.step(100, 1, &mut m.ports[5]),
        Err(controller::Fatal::Refused)
    );
    assert_eq!(c.counts(), (1, 0));
}
macro_rules! scenario {
    ($name:ident,$body:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn($body)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(startup_ready_cannot_replace_missing_role, missing_role_impl);
scenario!(
    role_sender_epoch_generation_and_raw_epoch_must_match,
    bad_role_impl
);
scenario!(
    published_focus_is_not_replayed_or_assumed_acknowledged,
    no_replay_impl
);
scenario!(
    wrong_kernel_sender_and_wrong_request_cannot_enable_go,
    wrong_ack_impl
);
scenario!(
    go_backpressure_preserves_exact_message_and_counts_only_queued,
    go_backpressure_impl
);
scenario!(
    stop_refusal_keeps_running_and_never_done_or_retire,
    refused_impl
);
scenario!(
    clean_stop_requires_app_done_then_compositor_retire_ack,
    stop_chain_impl
);
scenario!(
    unsolicited_app_done_is_not_shutdown_permission,
    unsolicited_done_impl
);
scenario!(unexpected_caps_are_closed_once_before_readiness, caps_impl);
scenario!(
    controller_barrier_is_not_ui_evidence_and_is_bounded,
    controller_barrier_impl
);
scenario!(
    controller_requires_actual_init_sender,
    controller_wrong_reply_impl
);
scenario!(
    controller_does_not_replay_published_stop_or_accept_refusal_as_done,
    controller_no_replay_impl
);
fn stopped_reply_backpressure_impl() {
    let mut m = Model::new(true);
    let mut s = m.supervisor();
    m.running(&mut s);
    m.stop(&mut s, 1, 2);
    m.app(wire::DONE);
    s.step(3, &mut m.ports[0]).unwrap();
    let filler = Outgoing::new(7, &[]).unwrap();
    for _ in 0..16 {
        m.ports[0]
            .send(m.config.controller.unwrap().reply, &filler)
            .unwrap();
    }
    m.ack(true);
    assert_eq!(s.step(4, &mut m.ports[0]), Ok(Phase::Reply));
    assert_eq!(s.counts().retire_ack, 1);
    assert_eq!(s.counts().controller_replies, 0);
    let sent = m.ports[0].sends;
    s.step(5, &mut m.ports[0]).unwrap();
    assert_eq!(m.ports[0].sends, sent + 1);
    assert!(m.ports[2]
        .receive(recv(m.comp_recv))
        .unwrap_err()
        .is_again());
    for _ in 0..16 {
        m.ports[5].receive(recv(m.controller_recv)).unwrap();
    }
    assert_eq!(s.step(6, &mut m.ports[0]), Ok(Phase::Stopped));
    let reply = m.ports[5].receive(recv(m.controller_recv)).unwrap();
    assert_eq!(
        reply.payload(),
        &StopRequest {
            id: 1,
            epoch: SURFACE,
            generation: 1
        }
        .reply(StopStatus::Stopped)
        .unwrap()
        .as_bytes()[64..112]
    );
    assert_eq!(s.counts().controller_replies, 1);
}
fn failed_retire_impl() {
    let mut m = Model::new(true);
    let mut s = m.supervisor();
    m.running(&mut s);
    m.stop(&mut s, 1, 2);
    m.app(wire::DONE);
    s.step(3, &mut m.ports[0]).unwrap();
    assert_eq!(s.step(503, &mut m.ports[0]), Ok(Phase::Reply));
    assert_eq!(s.counts().retire_ack, 0);
    assert_eq!(s.fatal(), Some(Fatal::Deadline));
    assert_eq!(s.step(504, &mut m.ports[0]), Ok(Phase::Failed));
    let reply = m.ports[5].receive(recv(m.controller_recv)).unwrap();
    assert_eq!(
        u64::from_le_bytes(reply.payload()[32..40].try_into().unwrap()),
        2
    );
    assert_ne!(s.phase(), Phase::Stopped);
}
#[test]
fn optional_controller_bootstrap_is_all_or_none_and_fixed_peer() {
    let mut w = [0; 128];
    for (o, v) in [
        (32, boot::MAGIC),
        (40, 1),
        (48, 0x100000001),
        (56, 0x100000002),
        (64, 0x100000003),
        (80, 4),
        (88, 5),
        (96, 6),
        (104, 7),
        (112, SURFACE),
        (120, POLICY),
        (128, 1),
        (136, 101),
    ] {
        w[o / 8] = v;
    }
    assert!(Bootstrap::decode(&w).unwrap().controller.is_none());
    w[72 / 8] = 0x100000004;
    assert!(Bootstrap::decode(&w).is_err());
    w[144 / 8] = 0x100000005;
    assert!(Bootstrap::decode(&w).is_err());
    w[152 / 8] = 8;
    assert!(Bootstrap::decode(&w).is_ok());
    w[152 / 8] = 4;
    assert!(Bootstrap::decode(&w).is_err());
    w[152 / 8] = 8;
    w[160 / 8] = 1;
    assert!(Bootstrap::decode(&w).is_err());
}
scenario!(
    stopped_controller_reply_backpressure_never_retires_twice,
    stopped_reply_backpressure_impl
);
scenario!(
    missing_retire_ack_reports_failed_without_stopped,
    failed_retire_impl
);
