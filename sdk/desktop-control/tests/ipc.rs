//! Actual Engine envelopes; peer payloads are host protocol fixtures, not I/O.
use capability_core::{Engine, Message, RECV, SEND};
use nieuw_os_desktop_control::{app, switch, Error, RequestIds, SIZE};
use nieuw_os_sdk_runtime::{Capability, Handle, Outgoing, Received, Rights};

struct Port {
    sender: u64,
    recipient: u64,
    send: u64,
    receive: u64,
}
struct Bus {
    engine: Box<Engine>,
}
impl Bus {
    fn new() -> Self {
        let mut engine = Box::new(Engine::new());
        for pid in [1, 4, 5, 8, 9] {
            engine.register(pid).unwrap();
        }
        Self { engine }
    }
    fn port(&mut self, sender: u64, recipient: u64) -> Port {
        let root = self.engine.create_endpoint(recipient).unwrap();
        let receive = self.engine.derive(recipient, root, RECV).unwrap();
        let send = self
            .engine
            .bootstrap_delegate(recipient, root, sender, SEND)
            .unwrap();
        self.engine.close(recipient, root).unwrap();
        Port {
            sender,
            recipient,
            send,
            receive,
        }
    }
    fn deliver(&mut self, p: &Port, message: Outgoing) -> Received {
        self.engine
            .send(
                p.sender,
                p.send,
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .unwrap();
        let m = self.engine.receive(p.recipient, p.receive).unwrap();
        assert_eq!(m.sender_pid, p.sender);
        Received::from_bytes(m.encode(), u64::from(m.len)).unwrap()
    }
    fn attach(&mut self, sender: u64, mut message: Outgoing) -> Outgoing {
        let root = self.engine.create_endpoint(sender).unwrap();
        message
            .push_cap(Capability {
                handle: Handle::from_raw(root).unwrap(),
                rights: Rights::SEND,
            })
            .unwrap();
        message
    }
    fn close_caps(&mut self, recipient: u64, message: &Received) {
        for i in 0..message.cap_count() {
            self.engine
                .close(recipient, message.cap(i).unwrap().handle.raw())
                .unwrap();
        }
    }
}
fn binding() -> app::Binding {
    app::Binding::new(10, 20, 30).unwrap()
}
fn preset() -> switch::Preset {
    switch::Preset::new(
        switch::Identity::new(4, binding()).unwrap(),
        switch::Identity::new(8, app::Binding::new(10, 21, 31).unwrap()).unwrap(),
    )
    .unwrap()
}
fn app_request(op: app::Operation, id: u64, activation: u64) -> app::Request {
    app::Request::new(op, id, binding(), activation).unwrap()
}
fn payload(message: Outgoing) -> [u8; SIZE] {
    message.as_bytes()[64..128].try_into().unwrap()
}
fn put(bytes: &mut [u8], at: usize, n: u64) {
    bytes[at..at + 8].copy_from_slice(&n.to_le_bytes());
}
fn app_message(bytes: &[u8]) -> Outgoing {
    Outgoing::new(app::TAG, bytes).unwrap()
}
fn switch_message(bytes: &[u8]) -> Outgoing {
    Outgoing::new(switch::TAG, bytes).unwrap()
}

#[test]
fn canonical_app_layout_and_real_pause_resume_roundtrip() {
    let mut bus = Bus::new();
    let tx = bus.port(1, 4);
    let rx = bus.port(4, 1);
    let mut server = app::Ingress::new(1, binding()).unwrap();
    let mut ids = RequestIds::new();
    for operation in [app::Operation::Pause, app::Operation::Resume] {
        let req = app_request(operation, ids.issue().unwrap(), 1);
        let bytes = payload(req.outgoing());
        assert_eq!(&bytes[..4], b"APC2");
        assert_eq!(&bytes[48..], &[0; 16]);
        let accepted = server.accept(&bus.deliver(&tx, req.outgoing())).unwrap();
        assert_eq!(accepted, req);
        let reply = match operation {
            app::Operation::Pause => accepted.paused().unwrap(),
            app::Operation::Resume => accepted.presented(7).unwrap(),
        };
        let mut pending = req.expect(4, 6).unwrap();
        let received = bus.deliver(&rx, reply);
        assert_eq!(
            pending.receive(&received),
            Ok(match operation {
                app::Operation::Pause => app::Completion::Paused,
                app::Operation::Resume => app::Completion::Presented { frame: 7 },
            })
        );
        assert_eq!(pending.receive(&received), Err(Error::Closed));
    }
}
#[test]
fn payload_cannot_choose_sender_tag_length_or_capability_authority() {
    let mut bus = Bus::new();
    let init = bus.port(1, 4);
    let attacker = bus.port(9, 4);
    let req = app_request(app::Operation::Pause, 1, 1);
    let mut server = app::Ingress::new(1, binding()).unwrap();
    assert_eq!(
        server.accept(&bus.deliver(&attacker, req.outgoing())),
        Err(Error::Sender)
    );
    let b = payload(req.outgoing());
    assert_eq!(
        server.accept(&bus.deliver(&init, Outgoing::new(0x53595354454d0001, &b).unwrap())),
        Err(Error::Tag)
    );
    for length in [0, 1, 63, 65, 256] {
        let mut bytes = [0u8; 256];
        bytes[..64].copy_from_slice(&b);
        assert_eq!(
            server.accept(&bus.deliver(&init, app_message(&bytes[..length]))),
            Err(Error::Length)
        );
    }
    let with_caps = bus.attach(1, req.outgoing());
    let received = bus.deliver(&init, with_caps);
    assert_eq!(server.accept(&received), Err(Error::Capabilities));
    bus.close_caps(4, &received);
    assert_eq!(server.accept(&bus.deliver(&init, req.outgoing())), Ok(req));
}
#[test]
fn app_request_unknown_header_ops_bits_and_reserved_bytes_never_advance_ingress() {
    let mut bus = Bus::new();
    let p = bus.port(1, 4);
    let req = app_request(app::Operation::Pause, 1, 1);
    for at in [0, 1, 4, 5, 6, 7, 48, 49, 55, 56, 63] {
        let mut server = app::Ingress::new(1, binding()).unwrap();
        let mut b = payload(req.outgoing());
        b[at] ^= 0x40;
        assert!(
            server.accept(&bus.deliver(&p, app_message(&b))).is_err(),
            "byte{at}"
        );
        assert_eq!(server.accept(&bus.deliver(&p, req.outgoing())), Ok(req));
    }
}
#[test]
fn binding_and_zero_or_exhausted_activation_are_rejected_before_replay_state() {
    let mut bus = Bus::new();
    let p = bus.port(1, 4);
    let req = app_request(app::Operation::Pause, 1, 1);
    for (at, n) in [
        (16, 11),
        (24, 21),
        (40, 31),
        (32, 0),
        (32, u64::MAX),
        (8, 0),
    ] {
        let mut server = app::Ingress::new(1, binding()).unwrap();
        let mut b = payload(req.outgoing());
        put(&mut b, at, n);
        assert!(server.accept(&bus.deliver(&p, app_message(&b))).is_err());
        assert_eq!(server.accept(&bus.deliver(&p, req.outgoing())), Ok(req));
    }
}
#[test]
fn activation_monotonicity_allows_only_pause_then_resume_in_same_switch() {
    let mut bus = Bus::new();
    let p = bus.port(1, 4);
    let mut server = app::Ingress::new(1, binding()).unwrap();
    for (operation, id, activation, ok) in [
        (app::Operation::Pause, 1, 3, true),
        (app::Operation::Pause, 2, 3, false),
        (app::Operation::Resume, 2, 3, true),
        (app::Operation::Resume, 3, 3, false),
        (app::Operation::Pause, 3, 2, false),
        (app::Operation::Pause, 3, 4, true),
    ] {
        let req = app_request(operation, id, activation);
        assert_eq!(server.accept(&bus.deliver(&p, req.outgoing())).is_ok(), ok);
    }
}
#[test]
fn authenticated_counter_exhaustion_isolated_from_other_ingress() {
    let mut bus = Bus::new();
    let p = bus.port(1, 4);
    let attacker = bus.port(9, 4);
    let req = app_request(app::Operation::Pause, 1, 1);
    let mut b = payload(req.outgoing());
    put(&mut b, 8, u64::MAX);
    put(&mut b, 32, 2);
    let mut server = app::Ingress::new(1, binding()).unwrap();
    assert_eq!(
        server.accept(&bus.deliver(&attacker, app_message(&b))),
        Err(Error::Sender)
    );
    assert_eq!(server.accept(&bus.deliver(&p, req.outgoing())), Ok(req));
    assert_eq!(
        server.accept(&bus.deliver(&p, app_message(&b))),
        Err(Error::Exhausted)
    );
    assert_eq!(
        server.accept(&bus.deliver(&p, app_request(app::Operation::Resume, 2, 1).outgoing())),
        Err(Error::Exhausted)
    );
    let mut other = app::Ingress::new(1, binding()).unwrap();
    assert_eq!(other.accept(&bus.deliver(&p, req.outgoing())), Ok(req));
}
#[test]
fn app_completion_is_exact_one_shot_and_never_accepts_stale_full_frame() {
    let mut bus = Bus::new();
    let p = bus.port(4, 1);
    let req = app_request(app::Operation::Resume, 2, 9);
    let good = payload(req.presented(11).unwrap());
    for at in [0, 4, 6, 7, 8, 16, 24, 32, 40, 56, 63] {
        let mut b = good;
        b[at] ^= 0x40;
        let mut pending = req.expect(4, 10).unwrap();
        assert!(pending.receive(&bus.deliver(&p, app_message(&b))).is_err());
        assert_eq!(
            pending.receive(&bus.deliver(&p, app_message(&good))),
            Err(Error::Closed)
        );
    }
    for frame in [0, 1, 10, u64::MAX] {
        let mut b = good;
        put(&mut b, 48, frame);
        assert!(req
            .expect(4, 10)
            .unwrap()
            .receive(&bus.deliver(&p, app_message(&b)))
            .is_err());
    }
    assert_eq!(req.paused().err(), Some(Error::Operation));
    let pause = app_request(app::Operation::Pause, 1, 9);
    assert_eq!(pause.presented(11).err(), Some(Error::Operation));
    let mut bad = payload(pause.paused().unwrap());
    put(&mut bad, 48, 1);
    assert_eq!(
        pause
            .expect(4, 0)
            .unwrap()
            .receive(&bus.deliver(&p, app_message(&bad))),
        Err(Error::Reserved)
    );
}
#[test]
fn abandoned_or_wrong_peer_reply_cannot_revive_a_pending_request() {
    let mut bus = Bus::new();
    let good = bus.port(4, 1);
    let bad = bus.port(8, 1);
    let req = app_request(app::Operation::Pause, 1, 1);
    let mut pending = req.expect(4, 0).unwrap();
    pending.abandon();
    assert_eq!(
        pending.receive(&bus.deliver(&good, req.paused().unwrap())),
        Err(Error::Closed)
    );
    let mut pending = req.expect(4, 0).unwrap();
    assert_eq!(
        pending.receive(&bus.deliver(&bad, req.paused().unwrap())),
        Err(Error::Sender)
    );
    assert_eq!(
        pending.receive(&bus.deliver(&good, req.paused().unwrap())),
        Err(Error::Closed)
    );
    let with_caps = bus.attach(4, req.paused().unwrap());
    let received = bus.deliver(&good, with_caps);
    assert_eq!(
        req.expect(4, 0).unwrap().receive(&received),
        Err(Error::Capabilities)
    );
    bus.close_caps(1, &received);
}
#[test]
fn independent_apps_cannot_reuse_each_others_context_or_reply_identity() {
    let mut bus = Bus::new();
    let calc = bus.port(8, 1);
    let tx = bus.port(1, 8);
    let req = app_request(app::Operation::Resume, 1, 1);
    let cb = preset().identity(switch::App::Calculator).binding();
    let mut calculator = app::Ingress::new(1, cb).unwrap();
    assert_eq!(
        calculator.accept(&bus.deliver(&tx, req.outgoing())),
        Err(Error::Binding)
    );
    assert_eq!(
        req.expect(4, 0)
            .unwrap()
            .receive(&bus.deliver(&calc, req.presented(1).unwrap())),
        Err(Error::Sender)
    );
}
#[test]
fn preset_requires_distinct_pid_surface_and_same_boot() {
    let m = preset().identity(switch::App::Materiaal);
    let c = preset().identity(switch::App::Calculator);
    assert_eq!(switch::Preset::new(m, m), Err(Error::Binding));
    assert_eq!(
        switch::Preset::new(m, switch::Identity::new(8, binding()).unwrap()),
        Err(Error::Binding)
    );
    assert_eq!(
        switch::Preset::new(
            m,
            switch::Identity::new(8, app::Binding::new(11, 21, 31).unwrap()).unwrap()
        ),
        Err(Error::Binding)
    );
    assert_eq!(
        switch::Ingress::new(4, preset()).err(),
        Some(Error::Binding)
    );
    assert_eq!(c.pid(), 8);
    for n in [0, u64::MAX] {
        assert!(app::Binding::new(n, 20, 30).is_err());
        assert!(app::Request::new(app::Operation::Pause, n, binding(), 1).is_err());
        assert!(switch::Identity::new(n, binding()).is_err());
    }
}
#[test]
fn switch_targets_are_only_boot_preset_apps_with_actual_sender_authentication() {
    let mut bus = Bus::new();
    let comp = bus.port(5, 1);
    let attacker = bus.port(9, 1);
    let mut ingress = switch::Ingress::new(5, preset()).unwrap();
    for (id, app) in [(1, switch::App::Materiaal), (2, switch::App::Calculator)] {
        let req = switch::Request::new(id, app, preset()).unwrap();
        assert_eq!(
            ingress.accept(&bus.deliver(&attacker, req.outgoing())),
            Err(Error::Sender)
        );
        let accepted = ingress.accept(&bus.deliver(&comp, req.outgoing())).unwrap();
        assert_eq!(accepted, req);
        assert_eq!(accepted.identity(), preset().identity(app));
        assert_eq!(
            ingress.accept(&bus.deliver(&comp, req.outgoing())),
            Err(Error::Replay)
        );
    }
}
#[test]
fn switch_payload_pid_handle_result_or_unknown_op_never_becomes_permission() {
    let mut bus = Bus::new();
    let p = bus.port(5, 1);
    let req = switch::Request::new(1, switch::App::Calculator, preset()).unwrap();
    let good = payload(req.outgoing());
    for (at, n) in [
        (24, 0),
        (24, 3),
        (24, 8),
        (24, 0x100000001),
        (16, 11),
        (32, 1),
        (40, 31),
        (48, 9),
        (56, 1),
    ] {
        let mut b = good;
        put(&mut b, at, n);
        let mut server = switch::Ingress::new(5, preset()).unwrap();
        assert!(server.accept(&bus.deliver(&p, switch_message(&b))).is_err());
        assert_eq!(server.accept(&bus.deliver(&p, req.outgoing())), Ok(req));
    }
    for at in [0, 4, 6, 7] {
        let mut b = good;
        b[at] ^= 0x40;
        assert!(switch::Ingress::new(5, preset())
            .unwrap()
            .accept(&bus.deliver(&p, switch_message(&b)))
            .is_err());
    }
}
#[test]
fn switch_success_requires_bound_surface_and_new_activation_and_target_frame() {
    let mut bus = Bus::new();
    let p = bus.port(1, 5);
    let req = switch::Request::new(7, switch::App::Calculator, preset()).unwrap();
    let good = payload(req.completed(4, 9).unwrap());
    assert_eq!(
        req.expect(1, 3, 8)
            .unwrap()
            .receive(&bus.deliver(&p, switch_message(&good))),
        Ok(switch::Outcome::Completed {
            activation: 4,
            surface: 31,
            frame: 9
        })
    );
    for (at, n) in [
        (8, 6),
        (16, 11),
        (24, 1),
        (32, 0),
        (32, 3),
        (32, u64::MAX),
        (40, 30),
        (48, 0),
        (48, 8),
        (48, u64::MAX),
        (56, 9),
    ] {
        let mut b = good;
        put(&mut b, at, n);
        let mut pending = req.expect(1, 3, 8).unwrap();
        assert!(pending
            .receive(&bus.deliver(&p, switch_message(&b)))
            .is_err());
        assert_eq!(
            pending.receive(&bus.deliver(&p, switch_message(&good))),
            Err(Error::Closed)
        );
    }
}
#[test]
fn rejected_switch_has_no_positive_metadata_and_unknown_status_is_not_ack() {
    let mut bus = Bus::new();
    let p = bus.port(1, 5);
    let req = switch::Request::new(1, switch::App::Materiaal, preset()).unwrap();
    for failure in [
        switch::Failure::Busy,
        switch::Failure::Denied,
        switch::Failure::Failed,
    ] {
        let good = payload(req.rejected(failure));
        assert_eq!(
            req.expect(1, 0, 0)
                .unwrap()
                .receive(&bus.deliver(&p, switch_message(&good))),
            Ok(switch::Outcome::Rejected(failure))
        );
        for at in [32, 40, 48] {
            let mut b = good;
            put(&mut b, at, 1);
            assert_eq!(
                req.expect(1, 0, 0)
                    .unwrap()
                    .receive(&bus.deliver(&p, switch_message(&b))),
                Err(Error::Reserved)
            );
        }
    }
}
#[test]
fn switch_envelope_caps_and_lost_reply_fail_closed_without_replay() {
    let mut bus = Bus::new();
    let p = bus.port(1, 5);
    let attacker = bus.port(9, 5);
    let request_port = bus.port(5, 1);
    let req = switch::Request::new(1, switch::App::Materiaal, preset()).unwrap();
    assert_eq!(
        req.expect(1, 0, 0)
            .unwrap()
            .receive(&bus.deliver(&attacker, req.completed(1, 1).unwrap())),
        Err(Error::Sender)
    );
    let mut pending = req.expect(1, 0, 0).unwrap();
    pending.abandon();
    assert_eq!(
        pending.receive(&bus.deliver(&p, req.completed(1, 1).unwrap())),
        Err(Error::Closed)
    );
    let with_caps = bus.attach(5, req.outgoing());
    let received = bus.deliver(&request_port, with_caps);
    assert_eq!(
        switch::Ingress::new(5, preset()).unwrap().accept(&received),
        Err(Error::Capabilities)
    );
    bus.close_caps(1, &received);
    let b = payload(req.outgoing());
    for len in [0, 63, 65] {
        let mut bytes = [0u8; 65];
        bytes[..64].copy_from_slice(&b);
        assert_eq!(
            switch::Ingress::new(5, preset())
                .unwrap()
                .accept(&bus.deliver(&request_port, switch_message(&bytes[..len]))),
            Err(Error::Length)
        );
    }
}
