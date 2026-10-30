//! Host integration: real capability-engine IPC and the existing Key Service
//! core/record dispatcher. Deterministic entropy is a test fixture, not native
//! entropy or proof that the client already runs inside the OS.
use capability_core::{Cap, Engine, Message, RECV, SEND};
use keyring_core::{wire, EntropySource, Service, Status as KeyStatus, MAX_PLAINTEXT, MAX_SEALED};
use nieuw_os_keyring_client::{
    Cancelled, Delivery, Error, KeyIndex, Operation, Outcome, Poll, RecordContext, RecordKind,
    Reply, Session, Transport,
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};

struct Entropy(u8);
impl EntropySource for Entropy {
    fn read(&mut self, out: &mut [u8]) -> Result<(), KeyStatus> {
        for b in out {
            *b = self.0;
            self.0 = self.0.wrapping_add(1);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    NoEnqueue,
    DropReply,
    WrongSender,
    WrongTag,
    Caps,
    Patch(usize, u8),
    ShortGenerate,
    ErrorBody,
    ZeroStatusKeys,
    DecreasingEntropy,
}

struct Host {
    engine: Engine,
    service: Service,
    entropy: Entropy,
    domain: usize,
    ingress_root: u64,
    ingress: u64,
    send: u64,
    receive: u64,
    reply: u64,
    fault: Fault,
    closed: Vec<u64>,
    sent: usize,
    dispatched: usize,
}
impl Host {
    fn new(domain: usize) -> Rc<RefCell<Self>> {
        let mut engine = Engine::new();
        engine.register(1).unwrap();
        engine.register(2).unwrap();
        let ingress_root = engine.create_endpoint(2).unwrap();
        let ingress = engine.derive(2, ingress_root, RECV).unwrap();
        let send = engine.bootstrap_delegate(2, ingress_root, 1, SEND).unwrap();
        let receive = engine.create_endpoint(1).unwrap();
        let reply = engine.bootstrap_delegate(1, receive, 2, SEND).unwrap();
        let mut service = Service::new();
        let mut entropy = Entropy(0);
        service.start(&mut entropy).unwrap();
        Rc::new(RefCell::new(Self {
            engine,
            service,
            entropy,
            domain,
            ingress_root,
            ingress,
            send,
            receive,
            reply,
            fault: Fault::None,
            closed: Vec::new(),
            sent: 0,
            dispatched: 0,
        }))
    }
    fn pump(&mut self) {
        let message = match self.engine.receive(2, self.ingress) {
            Ok(message) => message,
            Err(capability_core::Error::Again) => return,
            Err(e) => panic!("receive {e:?}"),
        };
        assert_eq!(
            (
                message.sender_pid,
                message.tag,
                message.len,
                message.cap_count
            ),
            (1, wire::TAG, wire::SIZE as u32, 0)
        );
        let payload = &message.data[..wire::SIZE];
        let h = wire::Header::decode(payload).unwrap();
        assert_ne!(h.request, 0);
        assert_eq!(h.status, 0);
        let body = &payload[wire::HEADER..wire::HEADER + usize::from(h.length)];
        self.dispatched += 1;
        // The ingress, never a request body field, supplies the true domain.
        let domain = self.domain;
        let mut response = match h.op {
            wire::STATUS => wire::reply(
                &h,
                KeyStatus::Ok,
                &wire::status_body(
                    self.service.state() as u8,
                    self.service.reason() as u8,
                    [self.service.keys_created(0), self.service.keys_created(1)],
                    self.service.counters.entropy_bytes,
                ),
            ),
            wire::GENERATE => match self.service.generate(domain, &mut self.entropy) {
                Ok(index) => wire::reply(&h, KeyStatus::Ok, &[index]),
                Err(s) => wire::reply(&h, s, &[]),
            },
            wire::REVOKE => match self.service.revoke(domain, h.index) {
                Ok(()) => wire::reply(&h, KeyStatus::Ok, &[]),
                Err(s) => wire::reply(&h, s, &[]),
            },
            wire::SEAL => {
                let mut out = [0; MAX_SEALED];
                match self
                    .service
                    .seal(domain, h.index, body, &mut self.entropy, &mut out)
                {
                    Ok(n) => wire::reply(&h, KeyStatus::Ok, &out[..n]),
                    Err(s) => wire::reply(&h, s, &[]),
                }
            }
            wire::UNSEAL => {
                let mut out = [0; MAX_PLAINTEXT];
                match self
                    .service
                    .unseal(domain, h.index, body, &mut self.entropy, &mut out)
                {
                    Ok(n) => wire::reply(&h, KeyStatus::Ok, &out[..n]),
                    Err(s) => wire::reply(&h, s, &[]),
                }
            }
            wire::SEAL_RECORD | wire::UNSEAL_RECORD => {
                wire::dispatch_record(&mut self.service, &mut self.entropy, domain, &h, body)
            }
            _ => panic!("ordinary client exposed forbidden opcode {}", h.op),
        };
        match self.fault {
            Fault::DropReply => return,
            Fault::Patch(at, xor) => response[at] ^= xor,
            Fault::ShortGenerate => {
                response[2..4].fill(0);
                response[wire::HEADER..].fill(0);
            }
            Fault::ErrorBody => {
                response[12..16].copy_from_slice(&KeyStatus::Auth.code().to_le_bytes());
                response[2..4].copy_from_slice(&1u16.to_le_bytes());
                response[wire::HEADER] = 0x42;
            }
            Fault::ZeroStatusKeys => response[wire::HEADER + 2..wire::HEADER + 6].fill(0),
            Fault::DecreasingEntropy => response[wire::HEADER + 6..wire::HEADER + 14]
                .copy_from_slice(&1024u64.to_le_bytes()),
            _ => {}
        }
        let mut reply = Message::new();
        reply.tag = if matches!(self.fault, Fault::WrongTag) {
            wire::TAG + 1
        } else {
            wire::TAG
        };
        reply.len = wire::SIZE as u32;
        reply.data.copy_from_slice(&response);
        if matches!(self.fault, Fault::Caps) {
            let cap = self.engine.create_endpoint(2).unwrap();
            reply.cap_count = 1;
            reply.caps[0] = Cap {
                handle: cap,
                rights: SEND,
            };
        }
        // Cancellation closes the receiver; the real service operation can
        // still have completed before the failed reply publication.
        let _ = self.engine.send(2, self.reply, &reply);
    }
}

struct Port(Rc<RefCell<Host>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut host = self.0.borrow_mut();
        host.sent += 1;
        if matches!(host.fault, Fault::NoEnqueue) {
            return Err(RuntimeError::Kernel(-8));
        }
        host.engine
            .send(
                1,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut host = self.0.borrow_mut();
        host.pump();
        let message = host
            .engine
            .receive(1, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let mut bytes = message.encode();
        if matches!(host.fault, Fault::WrongSender) {
            bytes[..8].copy_from_slice(&3u64.to_le_bytes());
        }
        Received::from_bytes(bytes, message.len as u64)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut host = self.0.borrow_mut();
        host.closed.push(handle.raw());
        host.engine
            .close(1, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
}
fn connect(host: &Rc<RefCell<Host>>, expected_domain: usize) -> Session<Port> {
    let h = host.borrow();
    Session::new(
        Port(Rc::clone(host)),
        2,
        expected_domain,
        SendEndpoint::from_handle(Handle::from_raw(h.send).unwrap()),
        ReceiveEndpoint::from_handle(Handle::from_raw(h.receive).unwrap()),
        3,
    )
    .unwrap()
}
fn generated(session: &mut Session<Port>) -> KeyIndex {
    match session.exchange(Operation::Generate).unwrap() {
        Reply::Generated(key) => key,
        _ => panic!("wrong typed reply"),
    }
}
fn context(kind: RecordKind, length: usize) -> RecordContext {
    RecordContext {
        volume: [0x42; 16],
        domain: 0,
        object: 5,
        revision: 7,
        chunk: 3,
        length: length as u16,
        kind,
        schema: 1,
    }
}
fn run(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn record_names_and_content_are_not_interchangeable_and_no_key_leaves_service() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let key = generated(&mut client);
        let plain = b"secret record";
        let sealed = match client
            .exchange(Operation::SealRecord {
                key,
                context: context(RecordKind::Name, plain.len()),
                plaintext: plain,
            })
            .unwrap()
        {
            Reply::Sealed(s) => s,
            _ => panic!(),
        };
        let opened = match client
            .exchange(Operation::UnsealRecord {
                key,
                context: context(RecordKind::Name, plain.len()),
                sealed: sealed.as_bytes(),
            })
            .unwrap()
        {
            Reply::Plaintext(p) => p,
            _ => panic!(),
        };
        assert_eq!(opened.as_bytes(), plain);
        assert!(!format!("{opened:?}").contains("secret record"));
        assert_eq!(
            client
                .exchange(Operation::UnsealRecord {
                    key,
                    context: context(RecordKind::Content, plain.len()),
                    sealed: sealed.as_bytes()
                })
                .unwrap_err(),
            Error::Service(KeyStatus::Auth)
        );
        let original = context(RecordKind::Name, plain.len());
        let mut changed_volume = original;
        changed_volume.volume[0] ^= 1;
        for changed in [
            changed_volume,
            RecordContext {
                object: original.object + 1,
                ..original
            },
            RecordContext {
                revision: original.revision + 1,
                ..original
            },
            RecordContext {
                chunk: original.chunk + 1,
                ..original
            },
            RecordContext {
                schema: original.schema + 1,
                ..original
            },
        ] {
            assert_eq!(
                client
                    .exchange(Operation::UnsealRecord {
                        key,
                        context: changed,
                        sealed: sealed.as_bytes()
                    })
                    .unwrap_err(),
                Error::Service(KeyStatus::Auth)
            );
        }
        let mut damaged = sealed.as_bytes().to_vec();
        damaged[24] ^= 1;
        assert_eq!(
            client
                .exchange(Operation::UnsealRecord {
                    key,
                    context: original,
                    sealed: &damaged
                })
                .unwrap_err(),
            Error::Service(KeyStatus::Auth)
        );
        client.exchange(Operation::Revoke(key)).unwrap();
        assert_eq!(
            client
                .exchange(Operation::UnsealRecord {
                    key,
                    context: context(RecordKind::Name, plain.len()),
                    sealed: sealed.as_bytes()
                })
                .unwrap_err(),
            Error::Service(KeyStatus::Revoked)
        );
        assert!(!client.is_closed());
        assert_eq!(host.borrow().service.counters.auth_failures, 7);
    });
}

#[test]
fn every_record_bound_and_legacy_empty_content_round_trip() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let key = generated(&mut client);
        for length in [0, 1, 128] {
            let plaintext = vec![0x61; length];
            let sealed = match client
                .exchange(Operation::SealRecord {
                    key,
                    context: context(RecordKind::Content, length),
                    plaintext: &plaintext,
                })
                .unwrap()
            {
                Reply::Sealed(s) => s,
                _ => panic!(),
            };
            assert_eq!(sealed.as_bytes().len(), length + 40);
            let reply = client
                .exchange(Operation::UnsealRecord {
                    key,
                    context: context(RecordKind::Content, length),
                    sealed: sealed.as_bytes(),
                })
                .unwrap();
            match reply {
                Reply::Plaintext(p) => assert_eq!(p.as_bytes(), plaintext),
                _ => panic!(),
            }
        }
        let sealed = match client
            .exchange(Operation::Seal {
                key,
                plaintext: &[],
            })
            .unwrap()
        {
            Reply::Sealed(s) => s,
            _ => panic!(),
        };
        match client
            .exchange(Operation::Unseal {
                key,
                sealed: sealed.as_bytes(),
            })
            .unwrap()
        {
            Reply::Plaintext(p) => assert!(p.as_bytes().is_empty()),
            _ => panic!(),
        }
    });
}

#[test]
fn local_context_and_length_errors_publish_nothing() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let key = KeyIndex::new(0).unwrap();
        let mut wrong = context(RecordKind::Content, 1);
        wrong.domain = 1;
        assert_eq!(
            client.begin(Operation::SealRecord {
                key,
                context: wrong,
                plaintext: b"a"
            }),
            Err(Error::WrongDomain)
        );
        wrong.domain = 0;
        wrong.schema = 0;
        assert_eq!(
            client.begin(Operation::SealRecord {
                key,
                context: wrong,
                plaintext: b"a"
            }),
            Err(Error::InvalidInput)
        );
        assert_eq!(
            client.begin(Operation::Seal {
                key,
                plaintext: &[0; 129]
            }),
            Err(Error::InvalidInput)
        );
        assert_eq!(
            client.begin(Operation::Unseal {
                key,
                sealed: &[0; 39]
            }),
            Err(Error::InvalidInput)
        );
        assert_eq!(
            client.begin(Operation::SealRecord {
                key,
                context: context(RecordKind::Name, 2),
                plaintext: b"a"
            }),
            Err(Error::InvalidInput)
        );
        assert_eq!(host.borrow().sent, 0);
        assert!(!client.is_busy());
        assert_eq!(KeyIndex::new(16), Err(Error::InvalidKey));
    });
}

#[test]
fn actual_ingress_remains_authoritative_despite_client_expected_domain() {
    run(|| {
        let host = Host::new(1);
        {
            let mut h = host.borrow_mut();
            let Host {
                service, entropy, ..
            } = &mut *h;
            service.generate(0, entropy).unwrap();
        }
        let mut client = connect(&host, 0);
        let key = generated(&mut client);
        let ctx = context(RecordKind::Name, 4);
        let sealed = match client
            .exchange(Operation::SealRecord {
                key,
                context: ctx,
                plaintext: b"name",
            })
            .unwrap()
        {
            Reply::Sealed(s) => s,
            _ => panic!(),
        };
        let mut h = host.borrow_mut();
        let Host {
            service, entropy, ..
        } = &mut *h;
        let mut out = [0x55; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(0, key.raw(), ctx, sealed.as_bytes(), entropy, &mut out),
            Err(KeyStatus::Auth)
        );
        assert_eq!(out, [0x55; MAX_PLAINTEXT]);
        let ctx = RecordContext { domain: 1, ..ctx };
        assert_eq!(
            service.unseal_record(1, key.raw(), ctx, sealed.as_bytes(), entropy, &mut out),
            Ok(4)
        );
    });
}

#[test]
fn malformed_reply_identity_padding_status_and_lengths_never_ack() {
    run(|| {
        for fault in [
            Fault::WrongSender,
            Fault::WrongTag,
            Fault::Caps,
            Fault::Patch(0, 1),
            Fault::Patch(1, 1),
            Fault::Patch(4, 1),
            Fault::Patch(12, 127),
            Fault::Patch(255, 1),
            Fault::Patch(3, 1),
            Fault::ShortGenerate,
            Fault::ErrorBody,
        ] {
            let host = Host::new(0);
            let mut client = connect(&host, 0);
            host.borrow_mut().fault = fault;
            assert!(matches!(
                client.exchange(Operation::Generate),
                Err(Error::Protocol {
                    outcome: Outcome::MayHaveCompleted,
                    ..
                })
            ));
            assert!(client.is_closed());
            assert!(client.is_uncertain());
            assert_eq!(
                host.borrow().closed.len(),
                if matches!(fault, Fault::Caps) { 3 } else { 2 }
            );
            let sent = host.borrow().sent;
            assert_eq!(client.begin(Operation::Generate), Err(Error::Closed));
            assert_eq!(host.borrow().sent, sent);
        }
    });
}

#[test]
fn lost_generate_reply_does_not_repeat_allocation() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        host.borrow_mut().fault = Fault::DropReply;
        assert!(matches!(
            client.exchange(Operation::Generate),
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                outcome: Outcome::MayHaveCompleted,
                ..
            })
        ));
        assert_eq!(host.borrow().service.keys_created(0), 1);
        assert_eq!(host.borrow().dispatched, 1);
        assert!(client.is_closed());
        assert!(client.is_uncertain());
    });
}

#[test]
fn lost_revoke_reply_keeps_the_real_revocation_and_never_reports_ack() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let key = generated(&mut client);
        host.borrow_mut().fault = Fault::DropReply;
        assert!(matches!(
            client.exchange(Operation::Revoke(key)),
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                outcome: Outcome::MayHaveCompleted,
                ..
            })
        ));
        assert!(client.is_uncertain());
        assert!(client.is_closed());
        let mut h = host.borrow_mut();
        assert_eq!(h.service.counters.revoked, 1);
        assert_eq!(h.service.revoke(0, key.raw()), Err(KeyStatus::Revoked));
        assert_eq!(h.sent, 2);
    });
}

#[test]
fn canonical_bad_handle_refusal_is_not_plaintext_or_a_terminal_wire_error() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        assert_eq!(
            client
                .exchange(Operation::Seal {
                    key: KeyIndex::new(15).unwrap(),
                    plaintext: b"private"
                })
                .unwrap_err(),
            Error::Service(KeyStatus::BadHandle)
        );
        assert!(!client.is_closed());
        assert!(!client.is_uncertain());
        assert_eq!(generated(&mut client).raw(), 0);
    });
}

#[test]
fn preenqueue_timeout_and_cancellation_do_not_report_completion() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        client.begin(Operation::Generate).unwrap();
        assert_eq!(client.begin(Operation::Status), Err(Error::Busy));
        assert_eq!(
            client.cancel(),
            Ok(Cancelled {
                delivery: Delivery::NotEnqueued,
                outcome: Outcome::NotPerformed,
                cleanup: None
            })
        );
        assert_eq!(host.borrow().sent, 0);
        assert!(!client.is_closed());
        host.borrow_mut().fault = Fault::NoEnqueue;
        assert!(matches!(
            client.exchange(Operation::Generate),
            Err(Error::Interrupted {
                delivery: Delivery::NotEnqueued,
                outcome: Outcome::NotPerformed,
                ..
            })
        ));
        assert_eq!(host.borrow().sent, 3);
        assert_eq!(host.borrow().dispatched, 0);
        assert!(!client.is_uncertain());
    });
}

#[test]
fn cancelling_published_request_is_not_a_wire_abort() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        client.begin(Operation::Generate).unwrap();
        assert!(matches!(
            client.poll(),
            Ok(Poll::Pending {
                delivery: Delivery::Enqueued
            })
        ));
        assert_eq!(
            client.cancel(),
            Ok(Cancelled {
                delivery: Delivery::Enqueued,
                outcome: Outcome::MayHaveCompleted,
                cleanup: None
            })
        );
        assert!(client.is_closed());
        assert!(client.is_uncertain());
        host.borrow_mut().pump();
        assert_eq!(host.borrow().service.keys_created(0), 1);
        assert_eq!(host.borrow().sent, 1);
        assert_eq!(host.borrow().closed.len(), 2);
    });
}

#[test]
fn kernel_revocation_is_not_overridden_by_client_configuration() {
    run(|| {
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let root = host.borrow().ingress_root;
        host.borrow_mut().engine.revoke(2, root).unwrap();
        assert!(matches!(
            client.exchange(Operation::Generate),
            Err(Error::Interrupted {
                delivery: Delivery::NotEnqueued,
                outcome: Outcome::NotPerformed,
                ..
            })
        ));
        assert_eq!(host.borrow().service.keys_created(0), 0);
        assert_eq!(host.borrow().dispatched, 0);
    });
}

#[test]
fn status_validates_state_reason_counter_bounds_and_monotonicity() {
    run(|| {
        for fault in [
            Fault::Patch(wire::HEADER, 9),
            Fault::Patch(wire::HEADER + 1, 9),
            Fault::Patch(wire::HEADER + 2, 32),
        ] {
            let host = Host::new(0);
            let mut client = connect(&host, 0);
            host.borrow_mut().fault = fault;
            assert!(matches!(
                client.exchange(Operation::Status),
                Err(Error::Protocol { .. })
            ));
        }
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        let _ = generated(&mut client);
        host.borrow_mut().fault = Fault::ZeroStatusKeys;
        assert!(matches!(
            client.exchange(Operation::Status),
            Err(Error::Protocol { .. })
        ));
        let host = Host::new(0);
        let mut client = connect(&host, 0);
        host.borrow_mut().service.counters.entropy_bytes = 2048;
        assert!(matches!(
            client.exchange(Operation::Status),
            Ok(Reply::Status(_))
        ));
        host.borrow_mut().fault = Fault::DecreasingEntropy;
        assert!(matches!(
            client.exchange(Operation::Status),
            Err(Error::Protocol { .. })
        ));
    });
}
