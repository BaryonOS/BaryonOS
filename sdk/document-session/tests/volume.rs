//! Host integration of the real capability engine and volume Service. The
//! in-memory disk below models ordered I/O; these are not native disk proofs.
use capability_core::{Cap, Engine, Message, SEND};
use nieuw_os_catalog_client::{Delivery, Transport};
use nieuw_os_document_session::{
    volume::{Buffers, DocumentSession, Error, ListEntry, SaveProgress, VolumeSession},
    EditMode, SaveOutcome,
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
use volume_core::{
    client::{Client, Transport as CoreTransport},
    service::Service,
    table::{self, Table},
    upload, wire, Disk, Status, PER_DOMAIN, SECTOR, VOLUME_SECTORS,
};

const EPOCH: u64 = 17;

struct MemoryDisk {
    sectors: Vec<[u8; SECTOR]>,
    writes: usize,
    fail: bool,
    fail_write_at: Option<usize>,
}
impl MemoryDisk {
    fn new() -> Self {
        let mut sectors = vec![[0; SECTOR]; VOLUME_SECTORS as usize];
        for index in 0..volume_core::TABLE_SECTORS {
            sectors[index as usize] = Table::EMPTY.encode_sector(index);
        }
        Self {
            sectors,
            writes: 0,
            fail: false,
            fail_write_at: None,
        }
    }
}
impl Disk for MemoryDisk {
    fn read(&mut self, sector: u64, out: &mut [u8; SECTOR]) -> Result<(), Status> {
        *out = self.sectors[sector as usize];
        Ok(())
    }
    fn write(&mut self, sector: u64, data: &[u8; SECTOR]) -> Result<(), Status> {
        if self.fail || self.fail_write_at == Some(self.writes) {
            return Err(Status::Io);
        }
        self.writes += 1;
        self.sectors[sector as usize] = *data;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        if self.fail {
            Err(Status::Io)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    DropCommit,
    WrongPeer,
    WrongTag,
    CapReply,
    BadReply,
    ShortRead,
}

struct Host {
    engine: Engine,
    service: Service,
    disk: MemoryDisk,
    request_root: u64,
    request_receive: u64,
    reply_send: u64,
    app_send: u64,
    app_receive: u64,
    fault: Fault,
    requests: Vec<wire::Header>,
    closed: Vec<u64>,
}

impl Host {
    fn new() -> Rc<RefCell<Self>> {
        let mut engine = Engine::new();
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let request_root = engine.create_endpoint(2).unwrap();
        let request_receive = engine
            .derive(2, request_root, capability_core::RECV)
            .unwrap();
        let service_to_app = engine.bootstrap_delegate(2, request_root, 1, SEND).unwrap();
        let app_receive = engine.create_endpoint(1).unwrap();
        let reply_send = engine.bootstrap_delegate(1, app_receive, 2, SEND).unwrap();
        Rc::new(RefCell::new(Self {
            engine,
            service: Service::new(EPOCH, Table::EMPTY, false),
            disk: MemoryDisk::new(),
            request_root,
            request_receive,
            reply_send,
            app_send: service_to_app,
            app_receive,
            fault: Fault::None,
            requests: Vec::new(),
            closed: Vec::new(),
        }))
    }

    fn pump(&mut self) {
        let request = match self.engine.receive(2, self.request_receive) {
            Ok(message) => message,
            Err(capability_core::Error::Again) => return,
            Err(e) => panic!("receive: {e:?}"),
        };
        let body: [u8; wire::SIZE] = request.data[..wire::SIZE].try_into().unwrap();
        let header = wire::header(&body);
        self.requests.push(header);
        let mut response = self
            .service
            .handle_for(0, request.sender_pid, &body, &mut self.disk);
        if matches!(self.fault, Fault::DropCommit) && header.operation == wire::COMMIT {
            return;
        }
        if matches!(self.fault, Fault::BadReply) {
            response[8] ^= 1;
        }
        if matches!(self.fault, Fault::ShortRead)
            && header.operation == wire::READ
            && header.aux != 0
        {
            response = upload::wrap(wire::error_reply(&header, EPOCH, Status::Access, 0), 0);
        }
        let mut message = Message::new();
        message.tag = if matches!(self.fault, Fault::WrongTag) {
            upload::TAG + 1
        } else {
            upload::TAG
        };
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&response);
        if matches!(self.fault, Fault::CapReply) {
            let extra = self.engine.create_endpoint(2).unwrap();
            message.caps[0] = Cap {
                handle: extra,
                rights: SEND,
            };
            message.cap_count = 1;
        }
        self.engine.send(2, self.reply_send, &message).unwrap();
    }

    fn seed(&mut self, object: u64, content: &[u8]) {
        struct Direct<'a>(&'a mut Service, &'a mut MemoryDisk);
        impl CoreTransport for Direct<'_> {
            fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
                Ok(self.0.handle(0, request, self.1))
            }
        }
        let mut client = Client::new(Direct(&mut self.service, &mut self.disk), EPOCH, 100_000);
        let revision = client.stat(object).map_or(0, |s| s.revision);
        client
            .write(
                object,
                revision,
                b"Document",
                content.len(),
                |offset, out| {
                    out.copy_from_slice(&content[offset..offset + out.len()]);
                },
            )
            .unwrap();
    }
}

struct Port(Rc<RefCell<Host>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        self.0
            .borrow_mut()
            .engine
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
        if matches!(host.fault, Fault::WrongPeer) {
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

fn editor<'a>(
    host: &Rc<RefCell<Host>>,
    mode: EditMode,
    buffers: Buffers<'a>,
) -> DocumentSession<'a, Port> {
    let h = host.borrow();
    let session = VolumeSession::new(
        Port(Rc::clone(host)),
        EPOCH,
        2,
        0,
        SendEndpoint::from_handle(Handle::from_raw(h.app_send).unwrap()),
        ReceiveEndpoint::from_handle(Handle::from_raw(h.app_receive).unwrap()),
        4,
    )
    .unwrap();
    DocumentSession::new(session, mode, buffers).unwrap()
}

fn large_stream_roundtrip_and_actual_volume_recovery_impl() {
    let host = Host::new();
    let mut draft = vec![0; volume_core::MAX_LENGTH];
    let mut original = draft.clone();
    let mut staging = draft.clone();
    let mut editor = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut draft,
            original: &mut original,
            staging: &mut staging,
        },
    );
    editor.create(0, b"Large").unwrap();
    let content: Vec<_> = (0..volume_core::MAX_LENGTH)
        .map(|i| (i * 13) as u8)
        .collect();
    editor.replace_contents(&content).unwrap();
    assert_eq!(editor.save(), Ok(SaveOutcome::Saved { revision: 1 }));
    assert!(!editor.is_dirty());
    editor.open(0).unwrap();
    assert_eq!(editor.draft().unwrap().contents, content);
    assert_eq!(editor.metadata().unwrap().length, content.len());
    let mut list = [ListEntry::EMPTY; PER_DOMAIN];
    assert_eq!(editor.list(&mut list), Ok(1));
    assert_eq!(list[0].name_length, 5);
    let mut host = host.borrow_mut();
    let mut recovered = Table::EMPTY;
    table::recover(&mut host.disk, &mut recovered).unwrap();
    assert_eq!(&recovered, host.service.table());
    assert!(
        host.requests
            .iter()
            .filter(|h| h.operation == wire::APPEND)
            .count()
            > 400
    );
}

fn cancellation_retains_draft_and_never_publishes_commit_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Before");
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    e.replace_contents(&[4; 700]).unwrap();
    e.begin_save().unwrap();
    assert_eq!(
        e.save_next(),
        Ok(SaveProgress::Writing {
            completed: 160,
            total: 700
        })
    );
    assert_eq!(e.replace_contents(b"mutate"), Err(Error::Busy));
    e.cancel_save().unwrap();
    assert!(e.is_dirty());
    assert!(!e.is_saving());
    assert_eq!(host.borrow().service.table().entries[0].revision, 1);
    assert!(!host
        .borrow()
        .requests
        .iter()
        .any(|h| h.operation == wire::COMMIT));
    assert_eq!(e.save(), Ok(SaveOutcome::Saved { revision: 2 }));
}

fn stale_draft_has_no_automatic_retry_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"old");
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    e.replace_contents(b"mine").unwrap();
    host.borrow_mut().seed(0, b"theirs");
    assert_eq!(e.save(), Err(Error::Conflict));
    let requests = host.borrow().requests.len();
    assert_eq!(e.save(), Err(Error::Conflict));
    assert_eq!(host.borrow().requests.len(), requests);
    assert_eq!(e.draft().unwrap().contents, b"mine");
    assert_eq!(e.draft().unwrap().loaded_revision, 1);
    e.discard_changes().unwrap();
    e.open(0).unwrap();
    assert_eq!(e.draft().unwrap().contents, b"theirs");
}

fn lost_commit_reply_is_uncertain_and_recovery_sees_actual_saved_bytes_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"old");
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    e.replace_contents(b"saved once").unwrap();
    host.borrow_mut().fault = Fault::DropCommit;
    assert!(matches!(
        e.save(),
        Err(Error::Interrupted {
            delivery: Delivery::Enqueued,
            ..
        })
    ));
    assert!(e.is_uncertain());
    assert!(e.is_closed());
    assert!(e.is_dirty());
    let requests = host.borrow().requests.len();
    assert_eq!(e.save(), Err(Error::Uncertain));
    assert_eq!(host.borrow().requests.len(), requests);
    assert_eq!(e.draft().unwrap().contents, b"saved once");
    let mut h = host.borrow_mut();
    let mut table = Table::EMPTY;
    table::recover(&mut h.disk, &mut table).unwrap();
    assert_eq!(table.entries[0].revision, 2);
    assert_eq!(table.entries[0].current.length, 10);
    assert_eq!(
        h.requests
            .iter()
            .filter(|r| r.operation == wire::COMMIT)
            .count(),
        1
    );
    assert_eq!(h.closed.len(), 2);
}

fn kernel_revocation_denies_next_save_and_preserves_draft_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"old");
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    e.replace_contents(b"keep").unwrap();
    let root = host.borrow().request_root;
    host.borrow_mut().engine.revoke(2, root).unwrap();
    assert!(matches!(
        e.save(),
        Err(Error::Interrupted {
            delivery: Delivery::NotEnqueued,
            ..
        })
    ));
    assert!(!e.is_uncertain());
    assert!(e.is_closed());
    assert_eq!(e.draft().unwrap().contents, b"keep");
    assert_eq!(host.borrow().service.table().entries[0].revision, 1);
}

fn malformed_envelopes_close_both_endpoints_and_keep_prior_document_impl() {
    for fault in [
        Fault::WrongPeer,
        Fault::WrongTag,
        Fault::BadReply,
        Fault::CapReply,
    ] {
        let host = Host::new();
        host.borrow_mut().seed(0, b"prior");
        let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
        let mut e = editor(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        e.open(0).unwrap();
        host.borrow_mut().fault = fault;
        assert!(matches!(e.open(0), Err(Error::Protocol { .. })));
        assert_eq!(e.draft().unwrap().contents, b"prior");
        assert!(e.is_closed());
        assert_eq!(
            host.borrow().closed.len(),
            if matches!(fault, Fault::CapReply) {
                3
            } else {
                2
            }
        );
    }
}

fn partial_read_failure_preserves_previous_complete_document_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"prior");
    host.borrow_mut().seed(1, &[8; 600]);
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    host.borrow_mut().fault = Fault::ShortRead;
    assert_eq!(e.open(1), Err(Error::Service(Status::Access)));
    assert_eq!(e.draft().unwrap().object, 0);
    assert_eq!(e.draft().unwrap().contents, b"prior");
}

fn bounds_and_readonly_checks_do_not_send_or_change_draft_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"prior");
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    let requests = host.borrow().requests.len();
    assert_eq!(e.replace_contents(&[0; 513]), Err(Error::ContentTooLarge));
    assert_eq!(e.replace_range(8..9, b"x"), Err(Error::InvalidRange));
    assert_eq!(e.rename_draft(b"a/b"), Err(Error::InvalidRequest));
    assert_eq!(e.save(), Ok(SaveOutcome::Unchanged { revision: 1 }));
    assert_eq!(host.borrow().requests.len(), requests);
    assert_eq!(e.draft().unwrap().contents, b"prior");
    drop(e);
    let mut e = editor(
        &host,
        EditMode::ReadOnly,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    let requests = host.borrow().requests.len();
    assert_eq!(e.replace_contents(b"x"), Err(Error::ReadOnly));
    assert_eq!(e.save(), Err(Error::ReadOnly));
    assert_eq!(host.borrow().requests.len(), requests);
}

fn certain_data_io_failure_keeps_old_revision_and_local_edits_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"prior");
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
    let mut e = editor(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    e.open(0).unwrap();
    e.replace_contents(&[9; 700]).unwrap();
    host.borrow_mut().disk.fail = true;
    assert_eq!(e.save(), Err(Error::Service(Status::Io)));
    assert!(e.is_dirty());
    assert!(!e.is_uncertain());
    assert_eq!(host.borrow().service.table().entries[0].revision, 1);
    host.borrow_mut().disk.fail = false;
    assert_eq!(e.save(), Ok(SaveOutcome::Saved { revision: 2 }));
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
fn large_stream_roundtrip_and_actual_volume_recovery() {
    run(large_stream_roundtrip_and_actual_volume_recovery_impl);
}

#[test]
fn cancellation_retains_draft_and_never_publishes_commit() {
    run(cancellation_retains_draft_and_never_publishes_commit_impl);
}

#[test]
fn stale_draft_has_no_automatic_retry() {
    run(stale_draft_has_no_automatic_retry_impl);
}

#[test]
fn lost_commit_reply_is_uncertain_and_recovery_sees_actual_saved_bytes() {
    run(lost_commit_reply_is_uncertain_and_recovery_sees_actual_saved_bytes_impl);
}

#[test]
fn kernel_revocation_denies_next_save_and_preserves_draft() {
    run(kernel_revocation_denies_next_save_and_preserves_draft_impl);
}

#[test]
fn malformed_envelopes_close_both_endpoints_and_keep_prior_document() {
    run(malformed_envelopes_close_both_endpoints_and_keep_prior_document_impl);
}

#[test]
fn partial_read_failure_preserves_previous_complete_document() {
    run(partial_read_failure_preserves_previous_complete_document_impl);
}

#[test]
fn bounds_and_readonly_checks_do_not_send_or_change_draft() {
    run(bounds_and_readonly_checks_do_not_send_or_change_draft_impl);
}

#[test]
fn certain_data_io_failure_keeps_old_revision_and_local_edits() {
    run(certain_data_io_failure_keeps_old_revision_and_local_edits_impl);
}

#[test]
fn table_io_unknown_is_sticky_and_does_not_retry_after_discard() {
    run(|| {
        let host = Host::new();
        host.borrow_mut().seed(0, b"prior");
        let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
        let mut e = editor(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        e.open(0).unwrap();
        e.replace_contents(b"keep after error").unwrap();
        let writes = host.borrow().disk.writes;
        // Existing Service first writes the partial content sector, then the
        // table. Fail its first table write to exercise its real Unknown path.
        host.borrow_mut().disk.fail_write_at = Some(writes + 1);
        assert_eq!(e.save(), Err(Error::Service(Status::Unknown)));
        assert!(e.is_uncertain());
        assert!(e.is_closed());
        assert_eq!(e.draft().unwrap().contents, b"keep after error");
        e.discard_changes().unwrap();
        let requests = host.borrow().requests.len();
        assert_eq!(e.save(), Err(Error::Uncertain));
        assert_eq!(e.open(0), Err(Error::Uncertain));
        assert_eq!(e.metadata(), Err(Error::Uncertain));
        assert_eq!(host.borrow().requests.len(), requests);
        assert_eq!(host.borrow().service.table().entries[0].revision, 1);
    });
}

#[test]
fn malformed_commit_ack_never_marks_the_draft_saved() {
    run(|| {
        let host = Host::new();
        host.borrow_mut().seed(0, b"prior");
        let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
        let mut e = editor(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        e.open(0).unwrap();
        e.replace_contents(b"actual new").unwrap();
        e.begin_save().unwrap();
        assert_eq!(
            e.save_next(),
            Ok(SaveProgress::Writing {
                completed: 10,
                total: 10
            })
        );
        host.borrow_mut().fault = Fault::BadReply;
        assert!(matches!(e.save_next(), Err(Error::Protocol { .. })));
        assert!(e.is_uncertain());
        assert!(e.is_dirty());
        assert_eq!(e.draft().unwrap().loaded_revision, 1);
        assert_eq!(host.borrow().service.table().entries[0].revision, 2);
    });
}

#[test]
fn empty_document_rename_editing_and_foreign_selector() {
    run(|| {
        let host = Host::new();
        let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
        let mut e = editor(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        e.create(0, b"Empty").unwrap();
        assert!(e.is_dirty());
        assert_eq!(e.save(), Ok(SaveOutcome::Saved { revision: 1 }));
        assert!(!host
            .borrow()
            .requests
            .iter()
            .any(|h| h.operation == wire::APPEND));
        e.open(0).unwrap();
        assert!(e.draft().unwrap().contents.is_empty());
        e.rename_draft(b"Renamed").unwrap();
        e.replace_range(0..0, b"abc").unwrap();
        e.replace_range(1..2, b"XYZ").unwrap();
        assert_eq!(e.draft().unwrap().contents, b"aXYZc");
        e.discard_changes().unwrap();
        assert_eq!(e.draft().unwrap().name, b"Empty");
        e.rename_draft(b"Renamed").unwrap();
        assert_eq!(e.save(), Ok(SaveOutcome::Saved { revision: 2 }));
        e.open(0).unwrap();
        assert_eq!(e.draft().unwrap().name, b"Renamed");
        let requests = host.borrow().requests.len();
        assert_eq!(
            e.open(PER_DOMAIN as u64),
            Err(Error::Service(Status::Access))
        );
        assert_eq!(host.borrow().requests.len(), requests);
        assert_eq!(e.draft().unwrap().object, 0);
        e.replace_contents(b"retained").unwrap();
        e.begin_save().unwrap();
        e.close().unwrap();
        e.replace_contents(b"editable offline").unwrap();
        assert_eq!(e.draft().unwrap().contents, b"editable offline");
    });
}

struct IndependentPort {
    host: Rc<RefCell<Host>>,
    request_receive: u64,
    reply_send: u64,
}
impl Transport for IndependentPort {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        self.host
            .borrow_mut()
            .engine
            .send(
                1,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut host = self.host.borrow_mut();
        let request = host
            .engine
            .receive(2, self.request_receive)
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let body: [u8; wire::SIZE] = request.data[..wire::SIZE].try_into().unwrap();
        host.requests.push(wire::header(&body));
        let h = &mut *host;
        let response = h
            .service
            .handle_for(0, request.sender_pid, &body, &mut h.disk);
        let mut message = Message::new();
        message.tag = upload::TAG;
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&response);
        host.engine.send(2, self.reply_send, &message).unwrap();
        let message = host
            .engine
            .receive(1, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        Received::from_bytes(message.encode(), message.len as u64)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        self.host
            .borrow_mut()
            .engine
            .close(1, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
}
fn independent_editor<'a>(
    host: &Rc<RefCell<Host>>,
    buffers: Buffers<'a>,
) -> DocumentSession<'a, IndependentPort> {
    let mut h = host.borrow_mut();
    let root = h.engine.create_endpoint(2).unwrap();
    let request_receive = h.engine.derive(2, root, capability_core::RECV).unwrap();
    let app_send = h.engine.bootstrap_delegate(2, root, 1, SEND).unwrap();
    let app_receive = h.engine.create_endpoint(1).unwrap();
    let reply_send = h
        .engine
        .bootstrap_delegate(1, app_receive, 2, SEND)
        .unwrap();
    let session = VolumeSession::new(
        IndependentPort {
            host: Rc::clone(host),
            request_receive,
            reply_send,
        },
        EPOCH,
        2,
        0,
        SendEndpoint::from_handle(Handle::from_raw(app_send).unwrap()),
        ReceiveEndpoint::from_handle(Handle::from_raw(app_receive).unwrap()),
        4,
    )
    .unwrap();
    DocumentSession::new(session, EditMode::ReadWrite, buffers).unwrap()
}

#[test]
fn independent_sessions_cannot_mix_upload_name_and_contents() {
    run(|| {
        let host = Host::new();
        host.borrow_mut().seed(0, b"old");
        let (mut ad, mut ao, mut ast) = ([0; 512], [0; 512], [0; 512]);
        let (mut bd, mut bo, mut bst) = ([0; 512], [0; 512], [0; 512]);
        let mut a = independent_editor(
            &host,
            Buffers {
                draft: &mut ad,
                original: &mut ao,
                staging: &mut ast,
            },
        );
        let mut b = independent_editor(
            &host,
            Buffers {
                draft: &mut bd,
                original: &mut bo,
                staging: &mut bst,
            },
        );
        a.open(0).unwrap();
        b.open(0).unwrap();
        a.rename_draft(b"NameA").unwrap();
        b.rename_draft(b"NameB").unwrap();
        a.replace_contents(b"AAAA").unwrap();
        b.replace_contents(b"BBBB").unwrap();
        a.begin_save().unwrap();
        assert_eq!(b.begin_save(), Err(Error::Service(Status::Busy)));
        assert!(b.is_dirty());
        assert_eq!(
            a.save_next(),
            Ok(SaveProgress::Writing {
                completed: 4,
                total: 4
            })
        );
        assert_eq!(a.save_next(), Ok(SaveProgress::Saved { revision: 2 }));
        let actual = host.borrow().service.table().entries[0].current;
        assert_eq!(&actual.name[..actual.name_length], b"NameA");
        assert!(!a.is_dirty());
        assert_eq!(b.save(), Err(Error::Conflict));
        assert_eq!(b.draft().unwrap().contents, b"BBBB");
    });
}
