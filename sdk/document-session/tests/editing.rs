//! Host model using real capability IPC, G03/B01, journal records and recovery.
//! WRITE/FLUSH below are ordered in-memory completions, not native durability.
use capability_core::{Engine, Message, SEND};
use catalog_core::{
    journal::{self, Recovery, State, SECTOR, SLOTS},
    wire,
};
use nieuw_os_catalog_client::{Delivery, Error as TransportError, Session, Transport};
use nieuw_os_document_session::{DocumentSession, EditMode, Error, SaveOutcome, Status};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Write,
    Flush,
    Commit,
    Reply,
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Fault {
    #[default]
    None,
    DropReply,
    Unknown,
    NoEnqueue,
    Denied,
}

struct Service {
    engine: Engine,
    endpoint: u64,
    reply_handles: Vec<(u64, u64)>,
    epoch: u64,
    state: State,
    records: [[u8; SECTOR]; SLOTS],
    staged: Option<(usize, [u8; SECTOR])>,
    events: Vec<Event>,
    requests: usize,
    puts: usize,
    attempts: usize,
    expected_revisions: Vec<u64>,
    closed: usize,
    fault: Fault,
}

impl Service {
    fn recovered(records: [[u8; SECTOR]; SLOTS], epoch: u64) -> Self {
        let mut recovery = Recovery::new();
        for (index, record) in records.iter().enumerate() {
            recovery.push(index, record).unwrap();
        }
        let state = recovery.finish().unwrap();
        let mut engine = Engine::new();
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let endpoint = engine.create_endpoint(2).unwrap();
        Self {
            engine,
            endpoint,
            reply_handles: Vec::new(),
            epoch,
            state,
            records,
            staged: None,
            events: Vec::new(),
            requests: 0,
            puts: 0,
            attempts: 0,
            expected_revisions: Vec::new(),
            closed: 0,
            fault: Fault::None,
        }
    }

    fn apply(
        &mut self,
        request: wire::Header,
        name: &[u8],
        value: &[u8],
    ) -> Result<(), journal::Error> {
        let prepared = self.state.prepare(
            request.object,
            request.revision,
            request.request_id,
            request.operation,
            name,
            value,
        )?;
        let index = self.state.sequence() as usize;
        // Model the existing adapter contract: append WRITE, successful FLUSH,
        // then State::commit and reply. Cold recovery reads only record bytes.
        self.staged = Some((index, *prepared.record()));
        self.events.push(Event::Write);
        let (index, bytes) = self.staged.take().unwrap();
        self.records[index] = bytes;
        self.events.push(Event::Flush);
        self.state.commit(prepared)?;
        self.events.push(Event::Commit);
        Ok(())
    }

    fn seed(&mut self, value: &[u8]) {
        let revision = self.state.get(0).unwrap().revision;
        let request = wire::request(
            self.state.sequence() + 1,
            self.epoch,
            0,
            revision,
            wire::PUT,
            Some(b"Document"),
            Some(value),
        );
        self.apply(wire::header(&request), b"Document", value)
            .unwrap();
    }

    fn pump(&mut self) {
        let request = match self.engine.receive(2, self.endpoint) {
            Ok(request) => request,
            Err(capability_core::Error::Again) => return,
            Err(error) => panic!("unexpected receive {error:?}"),
        };
        self.requests += 1;
        let bytes: [u8; wire::SIZE] = request.data[..wire::SIZE].try_into().unwrap();
        let header = wire::validate_request(&bytes, self.epoch, 0).unwrap();
        let reply = match header.operation {
            wire::LIST => {
                if self.fault == Fault::Denied {
                    wire::catalog_reply(header, self.epoch, Status::Access, None)
                } else {
                    wire::catalog_reply(
                        header,
                        self.epoch,
                        Status::Ok,
                        Some(&self.state.catalog(0).unwrap()),
                    )
                }
            }
            wire::GET => {
                let object = self.state.get(header.object).unwrap();
                wire::reply(
                    header,
                    self.epoch,
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
            wire::PUT => {
                self.puts += 1;
                self.expected_revisions.push(header.revision);
                if self.fault == Fault::Denied {
                    wire::reply(
                        header,
                        self.epoch,
                        header.revision,
                        Status::Access,
                        None,
                        None,
                    )
                } else if self.fault == Fault::Unknown {
                    wire::reply(header, self.epoch, 0, Status::Unknown, None, None)
                } else {
                    let result = self.apply(
                        header,
                        &bytes[192..192 + header.name_length as usize],
                        &bytes[64..64 + header.length as usize],
                    );
                    let status = match result {
                        Ok(()) => Status::Ok,
                        Err(journal::Error::Stale) => Status::Stale,
                        Err(journal::Error::Full) => Status::Full,
                        Err(error) => panic!("unexpected journal rejection {error:?}"),
                    };
                    wire::reply(
                        header,
                        self.epoch,
                        self.state.get(header.object).unwrap().revision,
                        status,
                        None,
                        None,
                    )
                }
            }
            _ => panic!("document session sent unexpected operation"),
        };
        if self.fault == Fault::DropReply && header.operation == wire::PUT {
            return;
        }
        let mut message = Message::new();
        message.tag = wire::TAG;
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&reply);
        let endpoint = self
            .reply_handles
            .iter()
            .find(|(pid, _)| *pid == request.sender_pid)
            .unwrap()
            .1;
        self.engine.send(2, endpoint, &message).unwrap();
        self.events.push(Event::Reply);
    }
}

struct Port {
    service: Rc<RefCell<Service>>,
    pid: u64,
}
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut service = self.service.borrow_mut();
        service.attempts += 1;
        if service.fault == Fault::NoEnqueue {
            return Err(RuntimeError::Kernel(capability_core::Error::Again.code()));
        }
        service
            .engine
            .send(
                self.pid,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|error| RuntimeError::Kernel(error.code()))
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut service = self.service.borrow_mut();
        service.pump();
        let message = service
            .engine
            .receive(self.pid, endpoint.handle().raw())
            .map_err(|error| RuntimeError::Kernel(error.code()))?;
        Received::from_bytes(message.encode(), u64::from(message.len))
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut service = self.service.borrow_mut();
        service.closed += 1;
        service
            .engine
            .close(self.pid, handle.raw())
            .map_err(|error| RuntimeError::Kernel(error.code()))
    }
}

fn connect(service: &Rc<RefCell<Service>>, pid: u64, mode: EditMode) -> DocumentSession<Port> {
    let mut state = service.borrow_mut();
    let receive = state.engine.create_endpoint(pid).unwrap();
    let endpoint = state.endpoint;
    let send = state
        .engine
        .bootstrap_delegate(2, endpoint, pid, SEND)
        .unwrap();
    let reply = state
        .engine
        .bootstrap_delegate(pid, receive, 2, SEND)
        .unwrap();
    state.reply_handles.push((pid, reply));
    let epoch = state.epoch;
    drop(state);
    let transport = Port {
        service: Rc::clone(service),
        pid,
    };
    DocumentSession::new(
        Session::new(
            transport,
            epoch,
            2,
            0,
            SendEndpoint::from_handle(Handle::from_raw(send).unwrap()),
            ReceiveEndpoint::from_handle(Handle::from_raw(receive).unwrap()),
            3,
        )
        .unwrap(),
        mode,
    )
}
fn setup(value: &[u8]) -> (Rc<RefCell<Service>>, DocumentSession<Port>) {
    let mut state = Service::recovered([[0; SECTOR]; SLOTS], 11);
    state.seed(value);
    let service = Rc::new(RefCell::new(state));
    let mut editor = connect(&service, 1, EditMode::ReadWrite);
    editor.open(0).unwrap();
    (service, editor)
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
fn load_edit_save_reopen_uses_exact_revision_and_flush_before_ack() {
    run(|| {
        let (service, mut editor) = setup(b"hello world");
        editor.replace_range(6..11, b"OS").unwrap();
        assert!(editor.is_dirty());
        assert_eq!(editor.draft().unwrap().loaded_revision(), 1);
        service.borrow_mut().events.clear();
        assert_eq!(editor.save(), Ok(SaveOutcome::Saved { revision: 2 }));
        assert_eq!(
            service.borrow().events,
            [Event::Write, Event::Flush, Event::Commit, Event::Reply]
        );
        assert_eq!(service.borrow().expected_revisions, [1]);
        assert!(!editor.is_dirty());
        editor.reopen().unwrap();
        let draft = editor.draft().unwrap();
        assert_eq!(
            (
                draft.object(),
                draft.loaded_revision(),
                draft.name(),
                draft.contents()
            ),
            (0, 2, b"Document".as_slice(), b"hello OS".as_slice())
        );
    });
}

#[test]
fn two_writers_conflict_without_rebase_or_draft_loss() {
    run(|| {
        let (service, mut first) = setup(b"original");
        let mut second = connect(&service, 3, EditMode::ReadWrite);
        second.open(0).unwrap();
        first.replace_contents(b"first change").unwrap();
        second.replace_contents(b"second draft").unwrap();
        first.save().unwrap();
        let draft = *second.draft().unwrap();
        assert_eq!(second.save(), Err(Error::Conflict));
        assert_eq!(second.draft(), Some(&draft));
        assert!(second.is_dirty() && second.is_conflicted());
        assert_eq!(service.borrow().expected_revisions, [1, 1]);
        let requests = service.borrow().requests;
        assert_eq!(second.save(), Err(Error::Conflict));
        assert_eq!(second.reopen(), Err(Error::UnsavedChanges));
        second.discard_changes().unwrap();
        assert_eq!(service.borrow().requests, requests);
        assert_eq!(second.draft().unwrap().loaded_revision(), 1);
        second.reopen().unwrap();
        assert!(!second.is_conflicted());
        assert_eq!(second.draft().unwrap().contents(), b"first change");
        second.replace_contents(draft.contents()).unwrap(); // explicit caller resolution
        assert_eq!(second.save(), Ok(SaveOutcome::Saved { revision: 3 }));
        assert_eq!(service.borrow().expected_revisions, [1, 1, 2]);
    });
}

#[test]
fn lost_reply_after_commit_retains_draft_without_replay_and_recovers_from_records() {
    run(|| {
        let (service, mut editor) = setup(b"before");
        editor
            .replace_contents(b"committed but unacknowledged")
            .unwrap();
        let draft = *editor.draft().unwrap();
        service.borrow_mut().fault = Fault::DropReply;
        assert!(matches!(
            editor.save(),
            Err(Error::Transport(TransportError::Interrupted {
                delivery: Delivery::Enqueued,
                ..
            }))
        ));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_uncertain() && editor.is_closed());
        let requests = service.borrow().requests;
        let attempts = service.borrow().attempts;
        assert_eq!(editor.save(), Err(Error::Uncertain));
        assert_eq!(
            (service.borrow().requests, service.borrow().attempts),
            (requests, attempts)
        );
        assert_eq!(service.borrow().puts, 1);
        assert_eq!(service.borrow().closed, 2);
        let records = service.borrow().records;
        let recovered = Rc::new(RefCell::new(Service::recovered(records, 12)));
        assert_eq!(recovered.borrow().state, service.borrow().state);
        let mut reopened = connect(&recovered, 1, EditMode::ReadWrite);
        reopened.open(0).unwrap();
        assert_eq!(reopened.draft().unwrap().loaded_revision(), 2);
        assert_eq!(reopened.draft().unwrap().contents(), draft.contents());
        assert_eq!(editor.draft().unwrap().loaded_revision(), 1);
        // A damaged persisted record is rejected by actual Recovery, not replaced
        // by the former in-memory State in this cold-service model.
        let mut bad = records;
        bad[1][64] ^= 1;
        let mut recovery = Recovery::new();
        recovery.push(0, &bad[0]).unwrap();
        assert_eq!(recovery.push(1, &bad[1]), Err(journal::Error::Corrupt));
    });
}

#[test]
fn service_unknown_remains_uncertain_even_after_explicit_local_discard() {
    run(|| {
        let (service, mut editor) = setup(b"original");
        editor.replace_contents(b"draft").unwrap();
        let draft = *editor.draft().unwrap();
        service.borrow_mut().fault = Fault::Unknown;
        assert_eq!(editor.save(), Err(Error::Service(Status::Unknown)));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_uncertain() && editor.is_closed());
        let requests = service.borrow().requests;
        editor.discard_changes().unwrap();
        assert!(!editor.is_dirty());
        assert_eq!(editor.reopen(), Err(Error::Uncertain));
        assert_eq!(editor.save(), Err(Error::Uncertain));
        assert_eq!(service.borrow().requests, requests);
    });
}

#[test]
fn preenqueue_timeout_preserves_draft_and_does_not_claim_a_commit() {
    run(|| {
        let (service, mut editor) = setup(b"old");
        editor.replace_contents(b"new").unwrap();
        let draft = *editor.draft().unwrap();
        service.borrow_mut().fault = Fault::NoEnqueue;
        assert!(matches!(
            editor.save(),
            Err(Error::Transport(TransportError::Interrupted {
                delivery: Delivery::NotEnqueued,
                ..
            }))
        ));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_closed());
        assert!(!editor.is_uncertain());
        assert_eq!(service.borrow().puts, 0);
        let attempts = service.borrow().attempts;
        assert_eq!(editor.save(), Err(Error::Closed));
        assert_eq!(service.borrow().attempts, attempts);
    });
}

#[test]
fn readonly_is_local_policy_and_server_access_denial_preserves_writable_draft() {
    run(|| {
        let (service, mut writer) = setup(b"original");
        let mut reader = connect(&service, 3, EditMode::ReadOnly);
        reader.open(0).unwrap();
        let requests = service.borrow().requests;
        assert_eq!(reader.replace_contents(b"changed"), Err(Error::ReadOnly));
        assert_eq!(reader.replace_range(0..1, b"x"), Err(Error::ReadOnly));
        assert_eq!(reader.save(), Err(Error::ReadOnly));
        assert_eq!(service.borrow().requests, requests);
        writer
            .replace_contents(b"not authorized by a Rust bool")
            .unwrap();
        let draft = *writer.draft().unwrap();
        service.borrow_mut().fault = Fault::Denied;
        assert_eq!(writer.save(), Err(Error::Service(Status::Access)));
        assert_eq!(writer.draft(), Some(&draft));
        assert!(writer.is_dirty());
        assert_eq!(service.borrow().requests, requests + 1);
        assert_eq!(service.borrow().state.sequence(), 1);
    });
}

#[test]
fn bounds_fail_before_mutation_or_transport() {
    run(|| {
        let (service, mut editor) = setup(b"bounded");
        let draft = *editor.draft().unwrap();
        let requests = service.borrow().requests;
        assert_eq!(
            editor.replace_contents(&[0; 129]),
            Err(Error::ContentTooLarge)
        );
        for range in [
            core::ops::Range { start: 3, end: 2 },
            0..8,
            usize::MAX..usize::MAX,
        ] {
            assert_eq!(editor.replace_range(range, b"x"), Err(Error::InvalidRange));
        }
        assert_eq!(
            editor.replace_range(0..0, &[0; 128]),
            Err(Error::ContentTooLarge)
        );
        assert_eq!(editor.draft(), Some(&draft));
        assert_eq!(service.borrow().requests, requests);
    });
}

#[test]
fn revoked_real_endpoint_cannot_be_restored_by_edit_mode() {
    run(|| {
        let (service, mut editor) = setup(b"loaded");
        editor.replace_contents(b"keep after revocation").unwrap();
        let draft = *editor.draft().unwrap();
        {
            let mut state = service.borrow_mut();
            let endpoint = state.endpoint;
            state.engine.revoke(2, endpoint).unwrap();
        }
        assert!(matches!(
            editor.save(),
            Err(Error::Transport(TransportError::Interrupted {
                delivery: Delivery::NotEnqueued,
                transport: Some(RuntimeError::Kernel(-10)),
                ..
            }))
        ));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_closed());
        assert!(!editor.is_uncertain());
        assert_eq!(service.borrow().puts, 0);
    });
}

#[test]
fn empty_maximum_and_non_utf8_contents_roundtrip() {
    run(|| {
        let (service, mut editor) = setup(b"");
        let bytes = core::array::from_fn::<_, 128, _>(|index| index as u8 ^ 0x80);
        editor.replace_range(0..0, &bytes).unwrap();
        assert_eq!(editor.save(), Ok(SaveOutcome::Saved { revision: 2 }));
        editor.reopen().unwrap();
        assert_eq!(editor.draft().unwrap().contents(), bytes);
        editor.replace_range(0..128, b"").unwrap();
        assert_eq!(editor.save(), Ok(SaveOutcome::Saved { revision: 3 }));
        let recovered = Rc::new(RefCell::new(Service::recovered(
            service.borrow().records,
            12,
        )));
        let mut reopened = connect(&recovered, 1, EditMode::ReadWrite);
        reopened.open(0).unwrap();
        assert_eq!(reopened.draft().unwrap().contents(), b"");
        assert_eq!(reopened.draft().unwrap().name(), b"Document");
    });
}

#[test]
fn open_missing_or_foreign_document_never_silently_discards_current_document() {
    run(|| {
        let (service, mut editor) = setup(b"loaded");
        editor.replace_contents(b"unsaved").unwrap();
        let requests = service.borrow().requests;
        assert_eq!(editor.open(1), Err(Error::UnsavedChanges));
        assert_eq!(service.borrow().requests, requests);
        editor.discard_changes().unwrap();
        let draft = *editor.draft().unwrap();
        assert_eq!(
            editor.open(1),
            Err(Error::NotFound {
                object: 1,
                revision: 0
            })
        );
        assert_eq!(editor.draft(), Some(&draft));
        let requests = service.borrow().requests;
        assert!(matches!(editor.open(2), Err(Error::Transport(_))));
        assert_eq!(editor.draft(), Some(&draft));
        assert_eq!(service.borrow().requests, requests);
    });
}

#[test]
fn journal_full_preserves_edit_and_loaded_revision_without_retry() {
    run(|| {
        let mut state = Service::recovered([[0; SECTOR]; SLOTS], 11);
        for _ in 0..SLOTS {
            state.seed(b"full");
        }
        let service = Rc::new(RefCell::new(state));
        let mut editor = connect(&service, 1, EditMode::ReadWrite);
        editor.open(0).unwrap();
        editor.replace_contents(b"retained draft").unwrap();
        let draft = *editor.draft().unwrap();
        assert_eq!(editor.save(), Err(Error::Service(Status::Full)));
        assert_eq!(editor.draft(), Some(&draft));
        assert_eq!(draft.loaded_revision(), 32);
        assert!(editor.is_dirty());
        assert_eq!(service.borrow().puts, 1);
    });
}

#[test]
fn reverted_edits_and_explicit_discard_are_local_and_clean_save_is_noop() {
    run(|| {
        let (service, mut editor) = setup(b"same");
        let requests = service.borrow().requests;
        editor.replace_contents(b"different").unwrap();
        editor.replace_contents(b"same").unwrap();
        assert!(!editor.is_dirty());
        assert_eq!(editor.save(), Ok(SaveOutcome::Unchanged { revision: 1 }));
        editor.replace_contents(b"other").unwrap();
        editor.discard_changes().unwrap();
        assert!(!editor.is_dirty());
        assert_eq!(service.borrow().requests, requests);
        editor.close().unwrap();
        assert_eq!(editor.draft().unwrap().contents(), b"same");
        assert_eq!(editor.save(), Err(Error::Closed));
        editor.close().unwrap();
        assert_eq!(service.borrow().closed, 2);
    });
}

#[test]
fn list_preserves_dirty_conflicted_draft_and_server_error() {
    run(|| {
        let (service, mut editor) = setup(b"original");
        let mut writer = connect(&service, 3, EditMode::ReadWrite);
        writer.open(0).unwrap();
        editor.replace_contents(b"retained local draft").unwrap();
        let draft = *editor.draft().unwrap();
        let catalog = editor.list().unwrap();
        assert_eq!((catalog.count, catalog.entries[0].revision), (1, 1));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && !editor.is_conflicted() && !editor.is_uncertain());
        writer.replace_contents(b"concurrent change").unwrap();
        writer.save().unwrap();
        assert_eq!(editor.save(), Err(Error::Conflict));
        let catalog = editor.list().unwrap();
        assert_eq!((catalog.count, catalog.entries[0].revision), (1, 2));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_conflicted() && !editor.is_uncertain());
        service.borrow_mut().fault = Fault::Denied;
        assert!(matches!(editor.list(), Err(Error::Service(Status::Access))));
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_conflicted() && !editor.is_uncertain());
        assert!(!editor.is_closed());
        assert_eq!(service.borrow().state.sequence(), 2);
    });
}

#[test]
fn list_after_uncertain_or_closed_never_sends_and_retains_draft() {
    run(|| {
        let (service, mut editor) = setup(b"original");
        editor.replace_contents(b"uncertain draft").unwrap();
        let draft = *editor.draft().unwrap();
        service.borrow_mut().fault = Fault::Unknown;
        assert_eq!(editor.save(), Err(Error::Service(Status::Unknown)));
        let attempts = service.borrow().attempts;
        assert!(matches!(editor.list(), Err(Error::Uncertain)));
        assert_eq!(service.borrow().attempts, attempts);
        assert_eq!(editor.draft(), Some(&draft));
        assert!(editor.is_dirty() && editor.is_uncertain());
        assert_eq!(service.borrow().closed, 2);

        let (other, mut closed) = setup(b"closed draft");
        let draft = *closed.draft().unwrap();
        closed.close().unwrap();
        let attempts = other.borrow().attempts;
        assert!(matches!(closed.list(), Err(Error::Closed)));
        assert_eq!(other.borrow().attempts, attempts);
        assert_eq!(closed.draft(), Some(&draft));
        assert_eq!(other.borrow().closed, 2);
    });
}
