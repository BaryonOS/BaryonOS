//! Host transport checks through the actual authority and catalog cores.
//! In-memory State::commit below is not a disk, WRITE or FLUSH proof.
use capability_core::{Engine, Message, SEND, TRANSFER};
use catalog_core::{
    client::{Error as ClientError, Failure, ProtocolError},
    journal, wire,
};
use nieuw_os_catalog_client::{
    Delivery, Error, Operation, Reply, Session, Status, Transport, MAX_POLL_BUDGET,
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};

#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    DropReply,
    Unknown,
    WrongSender,
    WrongTag,
    WrongRequest,
    Caps,
    SendReturn,
    Receive,
    Yield,
}

struct Model {
    engine: Engine,
    state: journal::State,
    service_root: u64,
    reply_send: u64,
    intruder_send: u64,
    transferable: u64,
    endpoint_send: SendEndpoint,
    endpoint_receive: ReceiveEndpoint,
    fault: Fault,
    attempts: usize,
    publications: usize,
    requests: usize,
    receives: usize,
    yields: usize,
    closed: Vec<u64>,
    release_queue_on_yield: bool,
    close_error: Option<u64>,
    fail_cap_close: bool,
}

impl Model {
    fn new() -> Self {
        let mut engine = Engine::new();
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let service_root = engine.create_endpoint(2).unwrap();
        let client_root = engine.create_endpoint(1).unwrap();
        let endpoint_send = engine.bootstrap_delegate(2, service_root, 1, SEND).unwrap();
        let reply_send = engine.bootstrap_delegate(1, client_root, 2, SEND).unwrap();
        let intruder_send = engine.bootstrap_delegate(1, client_root, 3, SEND).unwrap();
        let extra = engine.create_endpoint(2).unwrap();
        let transferable = engine.derive(2, extra, SEND | TRANSFER).unwrap();
        Self {
            engine,
            state: journal::State::new(),
            service_root,
            reply_send,
            intruder_send,
            transferable,
            endpoint_send: SendEndpoint::from_handle(Handle::from_raw(endpoint_send).unwrap()),
            endpoint_receive: ReceiveEndpoint::from_handle(Handle::from_raw(client_root).unwrap()),
            fault: Fault::None,
            attempts: 0,
            publications: 0,
            requests: 0,
            receives: 0,
            yields: 0,
            closed: Vec::new(),
            release_queue_on_yield: false,
            close_error: None,
            fail_cap_close: false,
        }
    }

    fn fill_queue(&mut self) {
        for _ in 0..16 {
            self.engine
                .send(1, self.endpoint_send.handle().raw(), &Message::new())
                .unwrap();
        }
    }

    fn reply(&mut self, request: Message) -> Message {
        assert_eq!(request.sender_pid, 1);
        let payload: [u8; wire::SIZE] = request.data[..wire::SIZE].try_into().unwrap();
        let header = wire::validate_request(&payload, 11, 0).unwrap();
        let reply = match header.operation {
            wire::LIST => wire::catalog_reply(
                header,
                11,
                Status::Ok,
                Some(&self.state.catalog(0).unwrap()),
            ),
            wire::GET => {
                let object = self.state.get(header.object).unwrap();
                wire::reply(
                    header,
                    11,
                    object.revision,
                    if object.present {
                        Status::Ok
                    } else {
                        Status::NotFound
                    },
                    Some(&object.name[..object.name_length]),
                    Some(&object.value[..object.length]),
                )
            }
            wire::PUT | wire::RENAME | wire::DELETE => {
                match self.state.prepare(
                    header.object,
                    header.revision,
                    header.request_id,
                    header.operation,
                    &payload[192..192 + header.name_length as usize],
                    &payload[64..64 + header.length as usize],
                ) {
                    Ok(prepared) => {
                        self.state.commit(prepared).unwrap();
                        wire::reply(header, 11, header.revision + 1, Status::Ok, None, None)
                    }
                    Err(error) => {
                        let status = match error {
                            journal::Error::Invalid => Status::Invalid,
                            journal::Error::Range => Status::Range,
                            journal::Error::Stale => Status::Stale,
                            journal::Error::Full => Status::Full,
                            journal::Error::Corrupt => Status::Corrupt,
                            journal::Error::NotFound => Status::NotFound,
                        };
                        wire::reply(
                            header,
                            11,
                            self.state.get(header.object).unwrap().revision,
                            status,
                            None,
                            None,
                        )
                    }
                }
            }
            _ => unreachable!(),
        };
        let reply = if matches!(self.fault, Fault::Unknown) {
            wire::reply(header, 11, 0, Status::Unknown, None, None)
        } else {
            reply
        };
        let mut message = Message::new();
        message.tag = wire::TAG;
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&reply);
        if matches!(self.fault, Fault::WrongTag) {
            message.tag ^= 1;
        }
        if matches!(self.fault, Fault::WrongRequest) {
            message.data[8] ^= 0x40;
        }
        if matches!(self.fault, Fault::Caps) {
            message.cap_count = 2;
            for cap in &mut message.caps {
                *cap = capability_core::Cap {
                    handle: self.transferable,
                    rights: SEND,
                };
            }
        }
        message
    }

    fn pump(&mut self) {
        // A bounded service step, including possible queued empty fillers used
        // solely to exercise the real 16-message endpoint backpressure limit.
        for _ in 0..16 {
            match self.engine.receive(2, self.service_root) {
                Ok(message) if message.tag == 0 => continue,
                Ok(message) => {
                    self.requests += 1;
                    let reply = self.reply(message);
                    if !matches!(self.fault, Fault::DropReply) {
                        let (sender, endpoint) = if matches!(self.fault, Fault::WrongSender) {
                            (3, self.intruder_send)
                        } else {
                            (2, self.reply_send)
                        };
                        self.engine.send(sender, endpoint, &reply).unwrap();
                    }
                }
                Err(capability_core::Error::Again) => break,
                Err(error) => panic!("unexpected service engine error {error:?}"),
            }
        }
    }

    fn session(self, budget: usize) -> Session<Self> {
        let (send, receive) = (self.endpoint_send, self.endpoint_receive);
        Session::new(self, 11, 2, 0, send, receive, budget).unwrap()
    }
}

impl Transport for Model {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        self.attempts += 1;
        self.engine
            .send(
                1,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        self.publications += 1;
        if matches!(self.fault, Fault::SendReturn) {
            return Err(RuntimeError::InvalidReturn);
        }
        Ok(())
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        self.receives += 1;
        self.pump();
        if matches!(self.fault, Fault::Receive) {
            return Err(RuntimeError::Kernel(-9));
        }
        let response = self
            .engine
            .receive(1, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let received = Received::from_bytes(response.encode(), u64::from(response.len))?;
        if self.fail_cap_close {
            self.close_error = received.cap(0).map(|cap| cap.handle.raw());
        }
        Ok(received)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        self.yields += 1;
        if matches!(self.fault, Fault::Yield) {
            return Err(RuntimeError::Kernel(-1));
        }
        if self.release_queue_on_yield {
            let _ = self.engine.receive(2, self.service_root);
            self.release_queue_on_yield = false;
        }
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        self.closed.push(handle.raw());
        if self.close_error == Some(handle.raw()) {
            return Err(RuntimeError::Kernel(-7));
        }
        self.engine
            .close(1, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
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

fn put(value: &[u8], revision: u64) -> Operation<'_> {
    Operation::Put {
        object: 0,
        expected_revision: revision,
        name: b"Notities",
        value,
    }
}

#[test]
fn actual_core_roundtrip_lists_creates_reads_cas_renames_and_deletes() {
    run(|| {
        let mut session = Model::new().session(4);
        assert!(matches!(session.exchange(Operation::List), Ok(Reply::Catalog(c)) if c.count == 0));
        assert_eq!(
            session.exchange(Operation::Get { object: 0 }),
            Ok(Reply::NotFound {
                object: 0,
                revision: 0
            })
        );
        assert_eq!(
            session.exchange(put(b"eerste tekst", 0)),
            Ok(Reply::Mutation {
                object: 0,
                revision: 1
            })
        );
        let Ok(Reply::Document(document)) = session.exchange(Operation::Get { object: 0 }) else {
            panic!("document");
        };
        assert_eq!(document.name(), b"Notities");
        assert_eq!(document.value(), b"eerste tekst");
        assert_eq!(
            session.exchange(put(b"lost edit", 0)),
            Ok(Reply::ServiceError(Status::Stale))
        );
        assert!(!session.is_closed());
        assert_eq!(
            session.exchange(Operation::Rename {
                object: 0,
                expected_revision: 1,
                name: b"Werk"
            }),
            Ok(Reply::Mutation {
                object: 0,
                revision: 2
            })
        );
        assert!(
            matches!(session.exchange(Operation::List), Ok(Reply::Catalog(c)) if c.count == 1 && c.generation == 2)
        );
        assert_eq!(
            session.exchange(Operation::Delete {
                object: 0,
                expected_revision: 2
            }),
            Ok(Reply::Mutation {
                object: 0,
                revision: 3
            })
        );
        assert_eq!(
            session.exchange(Operation::Get { object: 0 }),
            Ok(Reply::NotFound {
                object: 0,
                revision: 3
            })
        );
        assert_eq!(session.transport().state.sequence(), 3);
        session.close().unwrap();
        assert!(session.is_closed());
        assert_eq!(session.transport().closed.len(), 2);
    });
}

#[test]
fn real_full_queue_can_retry_only_before_publication() {
    run(|| {
        let mut model = Model::new();
        model.fill_queue();
        model.release_queue_on_yield = true;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(Operation::List),
            Ok(Reply::Catalog(_))
        ));
        assert_eq!(session.transport().attempts, 2);
        assert_eq!(session.transport().publications, 1);
        assert_eq!(session.transport().requests, 1);
    });
}

#[test]
fn full_queue_budget_exhaustion_never_enqueues_or_replays() {
    run(|| {
        let mut model = Model::new();
        model.fill_queue();
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(put(b"niet verzonden", 0)),
            Err(Error::Interrupted {
                delivery: Delivery::NotEnqueued,
                ..
            })
        ));
        assert_eq!(session.transport().attempts, 3);
        assert_eq!(session.transport().publications, 0);
        assert_eq!(session.transport().state.sequence(), 0);
        assert!(session.is_closed());
        assert_eq!(
            session.exchange(put(b"niet verzonden", 0)),
            Err(Error::Client(ClientError::Closed))
        );
        assert_eq!(session.transport().attempts, 3);
    });
}

#[test]
fn lost_reply_after_state_commit_closes_without_replaying_mutation() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::DropReply;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(put(b"mogelijk bewaard", 0)),
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                source: ClientError::Interrupted {
                    mutation_uncertain: true,
                    ..
                },
                ..
            })
        ));
        assert_eq!(session.transport().publications, 1);
        assert_eq!(session.transport().requests, 1);
        assert_eq!(session.transport().state.sequence(), 1);
        assert_eq!(session.transport().receives, 3);
        assert_eq!(session.transport().closed.len(), 2);
        assert_eq!(
            session.exchange(put(b"mogelijk bewaard", 0)),
            Err(Error::Client(ClientError::Closed))
        );
        assert_eq!(session.transport().publications, 1);
    });
}

#[test]
fn malformed_send_return_preserves_unknown_delivery_and_does_not_resend() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::SendReturn;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(put(b"onzeker", 0)),
            Err(Error::Interrupted {
                delivery: Delivery::Unknown,
                transport: Some(RuntimeError::InvalidReturn),
                ..
            })
        ));
        assert_eq!(session.transport().publications, 1);
        assert_eq!(session.transport().attempts, 1);
        assert!(session.is_closed());
    });
}

#[test]
fn service_unknown_is_never_a_mutation_ack_and_closes_endpoints() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::Unknown;
        let mut session = model.session(3);
        assert_eq!(
            session.exchange(put(b"onzeker", 0)),
            Ok(Reply::ServiceError(Status::Unknown))
        );
        assert!(session.is_closed());
        assert_eq!(session.transport().closed.len(), 2);
        assert_eq!(
            session.exchange(Operation::List),
            Err(Error::Client(ClientError::Closed))
        );
    });
}

#[test]
fn unexpected_capabilities_reach_b01_unchanged_then_all_are_closed() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::Caps;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(Operation::List),
            Err(Error::Interrupted {
                source: ClientError::Interrupted {
                    reason: Failure::Protocol(ProtocolError::Capabilities),
                    mutation_uncertain: false
                },
                cleanup: None,
                ..
            })
        ));
        assert_eq!(session.transport().closed.len(), 4);
        for handle in &session.transport().closed {
            assert_eq!(
                session.transport().engine.check_endpoint(1, *handle, SEND),
                Err(capability_core::Error::BadHandle)
            );
        }
        assert!(session.is_closed());
    });
}

#[test]
fn wrong_kernel_peer_tag_and_request_id_close_the_session() {
    run(|| {
        for (fault, expected) in [
            (Fault::WrongSender, ProtocolError::Sender),
            (Fault::WrongTag, ProtocolError::Tag),
            (Fault::WrongRequest, ProtocolError::Reply),
        ] {
            let mut model = Model::new();
            model.fault = fault;
            let mut session = model.session(3);
            let error = session.exchange(Operation::List).unwrap_err();
            assert!(
                matches!(error, Error::Interrupted { source: ClientError::Interrupted { reason: Failure::Protocol(found), .. }, .. } if found == expected)
            );
            assert!(session.is_closed());
            assert_eq!(session.transport().closed.len(), 2);
        }
    });
}

#[test]
fn transport_and_yield_errors_preserve_delivery_fact_and_close() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::Receive;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(Operation::List),
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                transport: Some(RuntimeError::Kernel(-9)),
                ..
            })
        ));
        assert_eq!(session.transport().closed.len(), 2);
        let mut model = Model::new();
        model.fault = Fault::Yield;
        model.fill_queue();
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(Operation::List),
            Err(Error::Interrupted {
                delivery: Delivery::NotEnqueued,
                transport: Some(RuntimeError::Kernel(-1)),
                ..
            })
        ));
        assert_eq!(session.transport().closed.len(), 2);
    });
}

#[test]
fn cleanup_failure_is_reported_and_other_endpoint_is_still_closed() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::Unknown;
        model.close_error = Some(model.endpoint_send.handle().raw());
        let mut session = model.session(3);
        assert_eq!(
            session.exchange(put(b"onzeker", 0)),
            Err(Error::Cleanup {
                service_status: Some(Status::Unknown),
                error: RuntimeError::Kernel(-7)
            })
        );
        assert_eq!(session.transport().closed.len(), 2);
        assert!(session.is_closed());
    });
}

#[test]
fn failed_unexpected_cap_close_does_not_skip_other_caps_or_endpoints() {
    run(|| {
        let mut model = Model::new();
        model.fault = Fault::Caps;
        model.fail_cap_close = true;
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(Operation::List),
            Err(Error::Interrupted {
                source: ClientError::Interrupted {
                    reason: Failure::Protocol(ProtocolError::Capabilities),
                    ..
                },
                cleanup: Some(RuntimeError::Kernel(-7)),
                ..
            })
        ));
        assert_eq!(session.transport().closed.len(), 4);
        for handle in &session.transport().closed[1..] {
            assert_eq!(
                session.transport().engine.check_endpoint(1, *handle, SEND),
                Err(capability_core::Error::BadHandle)
            );
        }
        assert!(session.is_closed());
    });
}

#[test]
fn revoked_endpoint_is_a_real_kernel_error_before_enqueue() {
    run(|| {
        let mut model = Model::new();
        model.engine.revoke(2, model.service_root).unwrap();
        let mut session = model.session(3);
        assert!(matches!(
            session.exchange(put(b"blocked", 0)),
            Err(Error::Interrupted {
                delivery: Delivery::NotEnqueued,
                transport: Some(RuntimeError::Kernel(-10)),
                ..
            })
        ));
        assert_eq!(session.transport().publications, 0);
        assert_eq!(session.transport().closed.len(), 2);
        session.close().unwrap();
        assert_eq!(session.transport().closed.len(), 2);
    });
}

#[test]
fn invalid_operation_is_local_and_leaves_session_usable() {
    run(|| {
        let mut session = Model::new().session(3);
        assert_eq!(
            session.exchange(Operation::Get { object: 2 }),
            Err(Error::Client(ClientError::InvalidRequest(Status::Access)))
        );
        assert_eq!(session.transport().attempts, 0);
        assert!(!session.is_closed());
        assert!(matches!(
            session.exchange(Operation::List),
            Ok(Reply::Catalog(_))
        ));
    });
}

#[test]
fn invalid_poll_budgets_and_endpoint_alias_are_rejected() {
    run(|| {
        for budget in [0, MAX_POLL_BUDGET + 1, usize::MAX] {
            let model = Model::new();
            let (send, receive) = (model.endpoint_send, model.endpoint_receive);
            assert!(matches!(
                Session::new(model, 11, 2, 0, send, receive, budget),
                Err(Error::InvalidBudget)
            ));
        }
        let model = Model::new();
        let send = model.endpoint_send;
        assert!(matches!(
            Session::new(
                model,
                11,
                2,
                0,
                send,
                ReceiveEndpoint::from_handle(send.handle()),
                1
            ),
            Err(Error::DuplicateEndpoints)
        ));
    });
}
