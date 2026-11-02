#![cfg(feature = "client")]
//! Real capability-engine envelopes and SUR1 receiver; the blit callback is a
//! host recorder/fault, never native display evidence.
use capability_core::{Cap, Engine, Message, SEND};
use nieuw_os_catalog_client::{Delivery, Transport};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use nieuw_os_surface_protocol::{
    client::{Error as ClientError, Session},
    *,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    Drop,
    WrongPeer,
    WrongTag,
    Caps,
    WrongRequest,
    WrongPixels,
    Blit,
    AgainOnce,
    AgainAlways,
    UnknownSend,
    WrongEpoch,
    WrongFrame,
    WrongTile,
    WrongOperation,
    GridAsAck,
    OtherService,
    ServiceGrid,
    DropGrid,
}
struct Host {
    engine: Engine,
    receiver: Receiver,
    send: u64,
    receive: u64,
    ingress: u64,
    reply: u64,
    wrong_reply: u64,
    fault: Fault,
    now: u64,
    published: Vec<Operation>,
    payloads: Vec<Vec<u8>>,
    closed: Vec<u64>,
    blits: usize,
    bytes: Vec<u8>,
}
impl Host {
    fn new() -> Rc<RefCell<Self>> {
        let mut engine = Engine::new();
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let ingress_root = engine.create_endpoint(2).unwrap();
        let ingress = engine
            .derive(2, ingress_root, capability_core::RECV)
            .unwrap();
        let send = engine.bootstrap_delegate(2, ingress_root, 1, SEND).unwrap();
        let receive = engine.create_endpoint(1).unwrap();
        let reply = engine.bootstrap_delegate(1, receive, 2, SEND).unwrap();
        let wrong_reply = engine.bootstrap_delegate(1, receive, 3, SEND).unwrap();
        let geometry = Geometry {
            width: 800,
            height: 600,
            assigned: Rectangle {
                x: 8,
                y: 24,
                width: 784,
                height: 568,
            },
            epoch: 11,
        };
        Rc::new(RefCell::new(Self {
            engine,
            receiver: Receiver::new(1, geometry, 100).unwrap(),
            send,
            receive,
            ingress,
            reply,
            wrong_reply,
            fault: Fault::None,
            now: 0,
            published: Vec::new(),
            payloads: Vec::new(),
            closed: Vec::new(),
            blits: 0,
            bytes: Vec::new(),
        }))
    }
    fn pump(&mut self) {
        while let Ok(message) = self.engine.receive(2, self.ingress) {
            assert_eq!(message.tag, TAG);
            assert_eq!(message.cap_count, 0);
            self.now += 1;
            let payload = &message.data[..message.len as usize];
            let reply = match self.receiver.accept(message.sender_pid, self.now, payload) {
                Ok(Action::Staged) => None,
                Ok(Action::Info(reply)) => Some(reply),
                Ok(Action::Commit(token)) => {
                    let fail = matches!(self.fault, Fault::Blit);
                    let blits = &mut self.blits;
                    let bytes = &mut self.bytes;
                    let reply = self
                        .receiver
                        .commit_with(token, self.now, |rectangle, pixels| {
                            *blits += 1;
                            assert!(rectangle.x >= 8 && rectangle.y >= 24);
                            *bytes = pixels.to_vec();
                            if fail {
                                Err(())
                            } else {
                                Ok(())
                            }
                        })
                        .unwrap();
                    if matches!(self.fault, Fault::Drop) {
                        None
                    } else {
                        Some(reply)
                    }
                }
                Err(error) => self.receiver.reject(message.sender_pid, payload, error),
            };
            if let Some(reply) = reply {
                if matches!(self.fault, Fault::DropGrid) && reply.status() == Err(Error::Grid) {
                    continue;
                }
                let mut response = Message::new();
                response.tag = if matches!(self.fault, Fault::WrongTag) {
                    TAG + 1
                } else {
                    TAG
                };
                response.len = reply.bytes().len() as u32;
                response.data[..reply.bytes().len()].copy_from_slice(reply.bytes());
                if matches!(self.fault, Fault::WrongRequest) {
                    response.data[8] ^= 1;
                }
                if matches!(self.fault, Fault::WrongPixels) {
                    response.data[44] ^= 1;
                }
                for (fault, at) in [
                    (matches!(self.fault, Fault::WrongEpoch), 16),
                    (matches!(self.fault, Fault::WrongFrame), 24),
                    (matches!(self.fault, Fault::WrongTile), 32),
                ] {
                    if fault {
                        response.data[at] ^= 1;
                    }
                }
                if matches!(self.fault, Fault::WrongOperation) {
                    response.data[6..8]
                        .copy_from_slice(&(Operation::Begin as u16 | REPLY).to_le_bytes());
                }
                if matches!(self.fault, Fault::GridAsAck) {
                    response.data[40..44].fill(0);
                }
                if matches!(self.fault, Fault::OtherService | Fault::ServiceGrid) {
                    let status = if matches!(self.fault, Fault::OtherService) {
                        Error::Busy
                    } else {
                        Error::Grid
                    };
                    response.data[40..44].copy_from_slice(&(status as u32).to_le_bytes());
                    response.data[44..48].fill(0);
                }
                if matches!(self.fault, Fault::Caps) {
                    let handle = self.engine.create_endpoint(2).unwrap();
                    response.caps[0] = Cap {
                        handle,
                        rights: SEND,
                    };
                    response.cap_count = 1;
                }
                let (sender, send) = if matches!(self.fault, Fault::WrongPeer) {
                    (3, self.wrong_reply)
                } else {
                    (2, self.reply)
                };
                self.engine.send(sender, send, &response).unwrap();
            }
        }
    }
}
struct Port(Rc<RefCell<Host>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        if matches!(h.fault, Fault::AgainAlways) {
            return Err(RuntimeError::Kernel(-8));
        }
        if matches!(h.fault, Fault::AgainOnce) {
            h.fault = Fault::None;
            return Err(RuntimeError::Kernel(-8));
        }
        let decoded = Message::decode(message.as_bytes()).unwrap();
        let operation = Header::decode(&decoded.data[..decoded.len as usize])
            .unwrap()
            .operation;
        h.engine
            .send(1, endpoint.handle().raw(), &decoded)
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        h.published.push(operation);
        h.payloads
            .push(decoded.data[..decoded.len as usize].to_vec());
        if matches!(h.fault, Fault::UnknownSend) {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.pump();
        let message = h
            .engine
            .receive(1, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        Received::from_bytes(message.encode(), message.len as u64)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.closed.push(handle.raw());
        h.engine
            .close(1, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
}
fn session(host: &Rc<RefCell<Host>>) -> Session<Port> {
    let h = host.borrow();
    Session::new(
        Port(Rc::clone(host)),
        2,
        11,
        SendEndpoint::from_handle(Handle::from_raw(h.send).unwrap()),
        ReceiveEndpoint::from_handle(Handle::from_raw(h.receive).unwrap()),
        4,
    )
    .unwrap()
}

fn roundtrip_impl() {
    let host = Host::new();
    let mut client = session(&host);
    let g = client.geometry().unwrap();
    host.borrow_mut().fault = Fault::AgainOnce;
    let pixels = [12, 34, 56, 0].repeat(256);
    client.blit(5, 0, g.rectangle(0).unwrap(), &pixels).unwrap();
    let h = host.borrow();
    assert_eq!(h.blits, 1);
    assert_eq!(h.bytes, pixels);
    assert_eq!(
        h.published
            .iter()
            .filter(|op| **op == Operation::Begin)
            .count(),
        1
    );
    assert_eq!(
        h.published
            .iter()
            .filter(|op| **op == Operation::Chunk)
            .count(),
        5
    );
    assert_eq!(
        h.published
            .iter()
            .filter(|op| **op == Operation::Commit)
            .count(),
        1
    );
    drop(h);
    client.close().unwrap();
    client.close().unwrap();
    assert_eq!(host.borrow().closed.len(), 2);
}
fn wrong_binding_impl() {
    for fault in [
        Fault::WrongPeer,
        Fault::WrongTag,
        Fault::Caps,
        Fault::WrongRequest,
        Fault::WrongPixels,
    ] {
        let host = Host::new();
        let mut client = session(&host);
        let g = client.geometry().unwrap();
        host.borrow_mut().fault = fault;
        assert!(matches!(
            client.blit(1, 0, g.rectangle(0).unwrap(), &[0; 1024]),
            Err(ClientError::Protocol { .. })
        ));
        assert!(client.is_closed());
        assert_eq!(
            host.borrow().closed.len(),
            if matches!(fault, Fault::Caps) { 3 } else { 2 }
        );
        assert!(host.borrow().engine.check_invariants());
    }
}
fn lost_reply_impl() {
    let host = Host::new();
    let mut client = session(&host);
    let g = client.geometry().unwrap();
    host.borrow_mut().fault = Fault::Drop;
    assert!(matches!(
        client.blit(1, 0, g.rectangle(0).unwrap(), &[0; 1024]),
        Err(ClientError::Interrupted {
            delivery: Delivery::Enqueued,
            ..
        })
    ));
    assert_eq!(host.borrow().blits, 1);
    let published = host.borrow().published.len();
    assert_eq!(
        client.blit(1, 0, g.rectangle(0).unwrap(), &[0; 1024]),
        Err(ClientError::Closed)
    );
    assert_eq!(host.borrow().published.len(), published);
}
fn failures_impl() {
    for fault in [Fault::AgainAlways, Fault::UnknownSend, Fault::Blit] {
        let host = Host::new();
        let mut client = session(&host);
        let g = client.geometry().unwrap();
        host.borrow_mut().fault = fault;
        let error = client
            .blit(1, 0, g.rectangle(0).unwrap(), &[0; 1024])
            .unwrap_err();
        match fault {
            Fault::AgainAlways => assert!(matches!(
                error,
                ClientError::Interrupted {
                    delivery: Delivery::NotEnqueued,
                    ..
                }
            )),
            Fault::UnknownSend => assert!(matches!(
                error,
                ClientError::Interrupted {
                    delivery: Delivery::Unknown,
                    ..
                }
            )),
            Fault::Blit => assert!(matches!(
                error,
                ClientError::Service {
                    error: Error::Blit,
                    ..
                }
            )),
            _ => unreachable!(),
        }
        assert!(client.is_closed());
        assert_eq!(host.borrow().closed.len(), 2);
    }
}
fn preflight_impl() {
    let host = Host::new();
    let mut client = session(&host);
    assert_eq!(
        client.blit(
            1,
            0,
            Rectangle {
                x: 0,
                y: 0,
                width: 32,
                height: 8
            },
            &[0; 1024]
        ),
        Err(ClientError::NotReady)
    );
    let g = client.geometry().unwrap();
    let before = host.borrow().published.len();
    assert_eq!(
        client.blit(1, 1, g.rectangle(1).unwrap(), &[0; 1024]),
        Err(ClientError::InvalidTile)
    );
    let mut pixels = [0; 1024];
    pixels[3] = 1;
    assert_eq!(
        client.blit(1, 0, g.rectangle(0).unwrap(), &pixels),
        Err(ClientError::InvalidTile)
    );
    assert_eq!(host.borrow().published.len(), before);
    assert!(!client.is_closed());
}
fn isolation_probe_impl() {
    let host = Host::new();
    let mut client = session(&host);
    assert_eq!(client.verify_isolation_probe(), Err(ClientError::NotReady));
    assert!(host.borrow().published.is_empty());
    let geometry = client.geometry().unwrap();
    host.borrow_mut().fault = Fault::AgainOnce;
    client.verify_isolation_probe().unwrap();
    assert!(!client.is_closed());
    {
        let h = host.borrow();
        assert!(h.closed.is_empty());
        assert_eq!(h.blits, 0);
        assert_eq!(h.payloads.len(), 2);
        let begin = &h.payloads[1];
        let header = Header::decode(begin).unwrap();
        let tile = geometry.rectangle(0).unwrap();
        assert_eq!(header.operation, Operation::Begin);
        assert_eq!((header.request, header.frame, header.tile), (2, 1, 0));
        assert_eq!(header.arg0, u32::from(geometry.assigned.width));
        assert_eq!(header.arg1, tile.y);
        assert_eq!(begin.len(), 56);
        assert_eq!(
            u16::from_le_bytes(begin[48..50].try_into().unwrap()),
            tile.width
        );
        assert_eq!(
            u16::from_le_bytes(begin[50..52].try_into().unwrap()),
            tile.height
        );
        assert_eq!(
            u32::from_le_bytes(begin[52..56].try_into().unwrap()) as usize,
            tile.bytes()
        );
        assert_eq!(h.receiver.completed_frame(), None);
    }
    client
        .blit(1, 0, geometry.rectangle(0).unwrap(), &[0; 1024])
        .unwrap();
    assert_eq!(host.borrow().blits, 1);
    assert_eq!(
        Header::decode(&host.borrow().payloads[2]).unwrap().request,
        3
    );
    let before = host.borrow().published.len();
    assert_eq!(
        client.verify_isolation_probe(),
        Err(ClientError::InvalidTile)
    );
    assert_eq!(host.borrow().published.len(), before);
}
fn isolation_binding_impl() {
    for fault in [
        Fault::WrongPeer,
        Fault::WrongTag,
        Fault::WrongRequest,
        Fault::WrongEpoch,
        Fault::WrongFrame,
        Fault::WrongTile,
        Fault::WrongOperation,
        Fault::WrongPixels,
        Fault::Caps,
        Fault::GridAsAck,
    ] {
        let host = Host::new();
        let mut client = session(&host);
        client.geometry().unwrap();
        host.borrow_mut().fault = fault;
        assert!(matches!(
            client.verify_isolation_probe(),
            Err(ClientError::Protocol { .. })
        ));
        assert!(client.is_closed());
        assert_eq!(host.borrow().blits, 0);
        assert_eq!(
            host.borrow().closed.len(),
            if matches!(fault, Fault::Caps) { 3 } else { 2 }
        );
        assert!(host.borrow().engine.check_invariants());
    }
}
fn isolation_failures_impl() {
    let host = Host::new();
    let mut client = session(&host);
    client.geometry().unwrap();
    host.borrow_mut().fault = Fault::OtherService;
    assert!(matches!(
        client.verify_isolation_probe(),
        Err(ClientError::Service {
            error: Error::Busy,
            ..
        })
    ));
    assert!(client.is_closed());
    let host = Host::new();
    let mut client = session(&host);
    client.geometry().unwrap();
    host.borrow_mut().fault = Fault::DropGrid;
    assert!(matches!(
        client.verify_isolation_probe(),
        Err(ClientError::Interrupted {
            delivery: Delivery::Enqueued,
            ..
        })
    ));
    assert!(client.is_closed());
    assert_eq!(host.borrow().published, [Operation::Info, Operation::Begin]);

    // Grid is tolerated only by the explicit diagnostic. Ordinary blit keeps
    // its existing fail-closed behavior even for that same service status.
    let host = Host::new();
    let mut client = session(&host);
    let geometry = client.geometry().unwrap();
    host.borrow_mut().fault = Fault::ServiceGrid;
    assert!(matches!(
        client.blit(1, 0, geometry.rectangle(0).unwrap(), &[0; 1024]),
        Err(ClientError::Service {
            error: Error::Grid,
            ..
        })
    ));
    assert!(client.is_closed());
    assert_eq!(host.borrow().closed.len(), 2);
}
macro_rules! scenario {
    ($name:ident,$body:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(8 * 1024 * 1024)
                .spawn($body)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(
    actual_capability_envelopes_roundtrip_without_republishing_after_eagain,
    roundtrip_impl
);
scenario!(
    wrong_peer_tag_caps_request_and_pixel_count_fail_closed,
    wrong_binding_impl
);
scenario!(
    lost_reply_is_not_ack_and_never_replays_a_published_tile,
    lost_reply_impl
);
scenario!(
    bounded_unsent_unknown_send_and_backend_failure_are_distinct,
    failures_impl
);
scenario!(
    invalid_grid_and_noncanonical_pixels_fail_before_any_send,
    preflight_impl
);
scenario!(
    real_grid_probe_consumes_only_request_and_next_real_blit_works,
    isolation_probe_impl
);
scenario!(
    probe_requires_exact_grid_peer_tag_correlation_and_no_caps,
    isolation_binding_impl
);
scenario!(
    only_explicit_probe_tolerates_grid_and_lost_reply_is_not_success,
    isolation_failures_impl
);
