//! Host integration of the real capability engine and volume Service. The
//! in-memory disk below models ordered I/O; these are not native disk proofs.
use capability_core::{Cap, Engine, Message, SEND};
use nieuw_os_catalog_client::{Delivery, Transport};
use nieuw_os_document_session::{
    volume::{
        Buffers, DocumentSession, Error, HistoryPrefix, ListEntry, ListFilter, SaveProgress,
        VolumeSession,
    },
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
        let sectors = vec![[0; SECTOR]; VOLUME_SECTORS as usize];
        let mut disk = Self {
            sectors,
            writes: 0,
            fail: false,
            fail_write_at: None,
        };
        table::write_format_marker(&mut disk).unwrap();
        disk
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
    DropMutation,
    WrongObject,
    RefreshRead,
    BeforeEnqueue,
    CrashOnReceive,
    ChangedListClock,
    CorruptLateRead,
    DropRead,
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
        // Drop a late tail read only, so the first staged read succeeds.
        if matches!(self.fault, Fault::DropRead)
            && header.operation == wire::READ
            && header.aux != 0
        {
            return;
        }
        if matches!(self.fault, Fault::DropMutation)
            && matches!(
                header.operation,
                wire::META | wire::COLLECTION | wire::RESTORE
            )
        {
            return;
        }
        if matches!(self.fault, Fault::WrongObject) {
            // A valid envelope for the other domain is still not our reply.
            response[24..32].copy_from_slice(&(PER_DOMAIN as u64).to_le_bytes());
        }
        if matches!(self.fault, Fault::ChangedListClock)
            && header.operation == wire::LIST
            && header.aux != 0
        {
            let revision = volume_core::get64(&response, 32);
            response[32..40].copy_from_slice(&(revision + 1).to_le_bytes());
        }
        if matches!(self.fault, Fault::RefreshRead)
            && matches!(header.operation, wire::STAT | wire::READ)
            && self.requests.iter().any(|h| h.operation == wire::RESTORE)
        {
            response = upload::wrap(wire::error_reply(&header, EPOCH, Status::Access, 0), 0);
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
        if matches!(self.fault, Fault::CorruptLateRead)
            && header.operation == wire::READ
            && header.aux != 0
        {
            response[64] ^= 1;
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
        if matches!(self.0.borrow().fault, Fault::BeforeEnqueue) {
            return Err(RuntimeError::Kernel(capability_core::Error::Again.code()));
        }
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
        if matches!(host.fault, Fault::CrashOnReceive) {
            return Err(RuntimeError::Kernel(capability_core::Error::Closed.code()));
        }
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

fn library_case(
    mode: EditMode,
    test: impl FnOnce(&Rc<RefCell<Host>>, &mut DocumentSession<'_, Port>),
) {
    let host = Host::new();
    host.borrow_mut().seed(0, b"first");
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
    let mut editor = editor(
        &host,
        mode,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    editor.open(0).unwrap();
    test(&host, &mut editor);
}

#[test]
fn library_collections_metadata_filters_and_empty_only_delete() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            assert_eq!(e.collection_stat(1).unwrap().revision, 0);
            assert_eq!(e.update_collection(1, 0, b"Home"), Ok(1));
            let c = e.collection_stat(1).unwrap();
            assert!(c.present);
            assert_eq!(c.name(), b"Home");
            assert_ne!(c.request_id, 0);
            assert_eq!(e.update_collection(1, 1, b"Projects"), Ok(2));
            assert_eq!(e.set_metadata(1, 0x12, 7, 1), Ok(2));
            assert_eq!(e.draft().unwrap().loaded_revision, 2);
            assert!(!e.is_dirty());
            assert_eq!(e.draft().unwrap().contents, b"first");
            let stat = e.metadata().unwrap();
            assert_eq!((stat.tags, stat.kind, stat.collection), (0x12, 7, 1));
            let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
            assert_eq!(
                e.list_filtered(ListFilter::Uncollected, &mut entries),
                Ok(0)
            );
            assert_eq!(
                e.list_filtered(ListFilter::Collection(1), &mut entries),
                Ok(1)
            );
            assert_eq!((entries[0].object, entries[0].revision), (0, 2));
            assert_eq!(
                e.list_filtered(ListFilter::Collection(2), &mut entries),
                Ok(0)
            );
            assert_eq!(
                e.update_collection(1, 2, b""),
                Err(Error::Service(Status::Busy))
            );
            assert_eq!(e.set_metadata(2, 0x12, 7, 0), Ok(3));
            assert_eq!(e.update_collection(1, 2, b""), Ok(3));
            assert!(!e.collection_stat(1).unwrap().present);
            assert_eq!(
                host.borrow().service.table().entries[0].current.name_length,
                8
            );
        })
    });
}

#[test]
fn history_preview_is_read_only_and_restore_is_a_new_revision() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.rename_draft(b"Current name").unwrap();
            e.replace_contents(b"second").unwrap();
            assert_eq!(e.save(), Ok(SaveOutcome::Saved { revision: 2 }));
            let stat = e.metadata().unwrap();
            assert_eq!(stat.history[0].revision, 1);
            let writes = host.borrow().disk.writes;
            let mut preview = [0x55; 16];
            assert_eq!(e.read_history(2, 1, &mut preview), Ok(5));
            assert_eq!(&preview[..5], b"first");
            assert_eq!(&preview[5..], &[0x55; 11]);
            assert_eq!(host.borrow().disk.writes, writes);
            assert_eq!(e.draft().unwrap().contents, b"second");
            assert_eq!(e.draft().unwrap().name, b"Current name");
            assert_eq!(e.restore_history(2, 1), Ok(3));
            assert_eq!(e.draft().unwrap().loaded_revision, 3);
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.draft().unwrap().name, b"Document");
            assert!(!e.is_dirty());
            let stat = e.metadata().unwrap();
            assert_eq!(stat.history_count, 1);
            assert_eq!(stat.history[0].revision, 2);
            assert_eq!(e.read_history(3, 1, &mut preview), Ok(6));
            assert_eq!(&preview[..6], b"second");
            assert_eq!(e.save(), Ok(SaveOutcome::Unchanged { revision: 3 }));
        })
    });
}

#[test]
fn readonly_library_reads_but_never_mutates() {
    run(|| {
        library_case(EditMode::ReadOnly, |host, e| {
            let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
            assert_eq!(e.list_filtered(ListFilter::All, &mut entries), Ok(1));
            assert!(!e.collection_stat(1).unwrap().present);
            let sent = host.borrow().requests.len();
            assert_eq!(e.update_collection(1, 0, b"Home"), Err(Error::ReadOnly));
            assert_eq!(e.set_metadata(1, 0, 0, 0), Err(Error::ReadOnly));
            assert_eq!(e.restore_history(1, 1), Err(Error::ReadOnly));
            assert_eq!(host.borrow().requests.len(), sent);
            assert_eq!(e.draft().unwrap().loaded_revision, 1);
            assert!(!e.is_dirty());
            host.borrow_mut().seed(0, b"second");
            e.open(0).unwrap();
            let mut preview = [0; 16];
            assert_eq!(e.read_history(2, 1, &mut preview), Ok(5));
            assert_eq!(&preview[..5], b"first");
        })
    });
}

#[test]
fn library_mutations_preserve_dirty_draft_and_async_upload() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"draft retained").unwrap();
            let sent = host.borrow().requests.len();
            assert_eq!(
                e.update_collection(1, 0, b"Home"),
                Err(Error::UnsavedChanges)
            );
            assert_eq!(e.set_metadata(1, 0, 0, 0), Err(Error::UnsavedChanges));
            assert_eq!(e.restore_history(1, 1), Err(Error::UnsavedChanges));
            assert_eq!(host.borrow().requests.len(), sent);
            e.begin_save().unwrap();
            let sent = host.borrow().requests.len();
            let mut preview = [0; 16];
            assert_eq!(e.update_collection(1, 0, b"Home"), Err(Error::Busy));
            assert_eq!(e.set_metadata(1, 0, 0, 0), Err(Error::Busy));
            assert_eq!(e.restore_history(1, 1), Err(Error::Busy));
            assert_eq!(e.read_history(1, 1, &mut preview), Err(Error::Busy));
            assert_eq!(e.collection_stat(1), Err(Error::Busy));
            assert_eq!(host.borrow().requests.len(), sent);
            assert!(e.is_saving());
            assert_eq!(e.draft().unwrap().contents, b"draft retained");
            e.cancel_save().unwrap();
            assert_eq!(e.draft().unwrap().contents, b"draft retained");
            assert!(e.is_dirty());
        })
    });
}

#[test]
fn stale_metadata_cas_is_sticky_without_automatic_retry() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            host.borrow_mut().seed(0, b"other editor");
            assert_eq!(e.set_metadata(1, 42, 0, 0), Err(Error::Conflict));
            assert!(e.is_conflicted());
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.draft().unwrap().loaded_revision, 1);
            let sent = host.borrow().requests.len();
            assert_eq!(e.set_metadata(1, 42, 0, 0), Err(Error::Conflict));
            assert_eq!(e.update_collection(1, 0, b"Home"), Err(Error::Conflict));
            assert_eq!(host.borrow().requests.len(), sent);
            e.open(0).unwrap();
            assert_eq!(e.draft().unwrap().contents, b"other editor");
            assert_eq!(e.set_metadata(2, 42, 0, 0), Ok(3));
        })
    });
}

#[test]
fn library_invalid_indices_revisions_and_domain_do_not_publish() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            let sent = host.borrow().requests.len();
            let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
            let mut preview = [0x55; 16];
            assert_eq!(
                e.list_filtered(ListFilter::Collection(0), &mut entries),
                Err(Error::InvalidRequest)
            );
            assert_eq!(e.collection_stat(0), Err(Error::InvalidRequest));
            assert_eq!(
                e.collection_stat(volume_core::COLLECTIONS + 1),
                Err(Error::InvalidRequest)
            );
            assert_eq!(
                e.update_collection(1, u64::MAX, b"Home"),
                Err(Error::InvalidRequest)
            );
            assert_eq!(
                e.update_collection(1, 0, b"bad/name"),
                Err(Error::InvalidRequest)
            );
            assert_eq!(e.set_metadata(2, 0, 0, 0), Err(Error::Conflict));
            assert_eq!(
                e.set_metadata(u64::MAX, 0, 0, 0),
                Err(Error::InvalidRequest)
            );
            assert_eq!(e.set_metadata(1, 0, 0, 255), Err(Error::InvalidRequest));
            assert_eq!(
                e.read_history(1, 0, &mut preview),
                Err(Error::InvalidRequest)
            );
            assert_eq!(
                e.restore_history(1, volume_core::HISTORY + 1),
                Err(Error::InvalidRequest)
            );
            assert_eq!(
                e.open(PER_DOMAIN as u64),
                Err(Error::Service(Status::Access))
            );
            assert_eq!(host.borrow().requests.len(), sent);
            assert_eq!(preview, [0x55; 16]);
            assert_eq!(e.draft().unwrap().contents, b"first");
        })
    });
}

#[test]
fn dropped_library_mutation_replies_are_uncertain_and_never_retried() {
    run(|| {
        for operation in [wire::META, wire::COLLECTION, wire::RESTORE] {
            library_case(EditMode::ReadWrite, |host, e| {
                let expected = if operation == wire::RESTORE {
                    e.replace_contents(b"second").unwrap();
                    e.save().unwrap();
                    2
                } else {
                    1
                };
                let prior = e.draft().unwrap().contents.to_vec();
                host.borrow_mut().fault = Fault::DropMutation;
                let error = match operation {
                    wire::META => e.set_metadata(expected, 42, 0, 0),
                    wire::COLLECTION => e.update_collection(1, 0, b"Home"),
                    _ => e.restore_history(expected, 1),
                }
                .unwrap_err();
                assert!(matches!(
                    error,
                    Error::Interrupted {
                        delivery: Delivery::Enqueued,
                        ..
                    }
                ));
                assert!(e.is_uncertain());
                assert!(e.is_closed());
                assert_eq!(e.draft().unwrap().contents, prior);
                assert_eq!(e.draft().unwrap().loaded_revision, expected);
                let sent = host.borrow().requests.len();
                e.discard_changes().unwrap();
                assert_eq!(e.set_metadata(expected, 42, 0, 0), Err(Error::Uncertain));
                assert_eq!(e.update_collection(1, 0, b"Home"), Err(Error::Uncertain));
                assert_eq!(e.restore_history(expected, 1), Err(Error::Uncertain));
                assert_eq!(host.borrow().requests.len(), sent);
                let host = host.borrow();
                if operation == wire::COLLECTION {
                    assert!(host.service.table().collection(0, 1).present);
                } else {
                    assert_eq!(host.service.table().entries[0].revision, expected + 1);
                }
                assert_eq!(host.closed.len(), 2);
            });
        }
    });
}

#[test]
fn library_wrong_domain_or_malformed_mutation_reply_is_not_success() {
    run(|| {
        for fault in [
            Fault::WrongObject,
            Fault::WrongPeer,
            Fault::BadReply,
            Fault::CapReply,
        ] {
            library_case(EditMode::ReadWrite, |host, e| {
                host.borrow_mut().fault = fault;
                assert!(matches!(
                    e.set_metadata(1, 42, 0, 0),
                    Err(Error::Protocol { .. })
                ));
                assert!(e.is_uncertain());
                assert!(e.is_closed());
                assert_eq!(e.draft().unwrap().loaded_revision, 1);
                assert_eq!(e.draft().unwrap().contents, b"first");
                assert_eq!(host.borrow().service.table().entries[0].revision, 2);
            });
        }
    });
}

#[test]
fn library_pre_enqueue_timeout_does_not_claim_publication_or_retry() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            host.borrow_mut().fault = Fault::BeforeEnqueue;
            let sent = host.borrow().requests.len();
            assert!(matches!(
                e.set_metadata(1, 0, 0, 0),
                Err(Error::Interrupted {
                    delivery: Delivery::NotEnqueued,
                    ..
                })
            ));
            assert!(!e.is_uncertain());
            assert!(e.is_closed());
            assert_eq!(e.set_metadata(1, 0, 0, 0), Err(Error::Closed));
            assert_eq!(host.borrow().requests.len(), sent);
            assert_eq!(host.borrow().service.table().entries[0].revision, 1);
        })
    });
}

#[test]
fn library_service_crash_after_enqueue_blocks_all_mutations() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            host.borrow_mut().fault = Fault::CrashOnReceive;
            assert!(matches!(
                e.set_metadata(1, 0, 0, 0),
                Err(Error::Interrupted {
                    delivery: Delivery::Enqueued,
                    ..
                })
            ));
            assert!(e.is_uncertain());
            assert!(e.is_closed());
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.set_metadata(1, 0, 0, 0), Err(Error::Uncertain));
        })
    });
}

#[test]
fn acknowledged_restore_with_failed_refresh_retains_old_draft_and_blocks_save() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"second").unwrap();
            e.save().unwrap();
            host.borrow_mut().fault = Fault::RefreshRead;
            assert_eq!(
                e.restore_history(2, 1),
                Err(Error::CommittedRefreshFailed { revision: 3 })
            );
            assert!(!e.is_uncertain());
            assert!(e.is_conflicted());
            assert_eq!(e.draft().unwrap().contents, b"second");
            assert_eq!(e.draft().unwrap().loaded_revision, 2);
            assert_eq!(host.borrow().service.table().entries[0].revision, 3);
            let sent = host.borrow().requests.len();
            assert_eq!(e.save(), Err(Error::Conflict));
            assert_eq!(e.restore_history(2, 1), Err(Error::Conflict));
            assert_eq!(host.borrow().requests.len(), sent);
            host.borrow_mut().fault = Fault::None;
            e.open(0).unwrap();
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.draft().unwrap().loaded_revision, 3);
        })
    });
}

#[test]
fn history_preview_keeps_dirty_draft_and_failed_read_keeps_output() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"second").unwrap();
            e.save().unwrap();
            e.rename_draft(b"Unsaved title").unwrap();
            e.replace_contents(b"unsaved work").unwrap();
            let writes = host.borrow().disk.writes;
            let mut preview = [0x55; 16];
            assert_eq!(e.read_history(2, 1, &mut preview), Ok(5));
            assert_eq!(e.draft().unwrap().name, b"Unsaved title");
            assert_eq!(e.draft().unwrap().contents, b"unsaved work");
            assert!(e.is_dirty());
            assert_eq!(host.borrow().disk.writes, writes);
        });
        library_case(EditMode::ReadWrite, |host, e| {
            host.borrow_mut().seed(0, &[4; 500]);
            e.open(0).unwrap();
            e.replace_contents(&[7; 500]).unwrap();
            e.save().unwrap();
            let expected = e.draft().unwrap().loaded_revision;
            let mut preview = [0x55; 600];
            assert_eq!(
                e.read_history(expected, 1, &mut preview[..20]),
                Err(Error::ContentTooLarge)
            );
            assert_eq!(preview, [0x55; 600]);
            host.borrow_mut().fault = Fault::ShortRead;
            assert_eq!(
                e.read_history(expected, 1, &mut preview),
                Err(Error::Service(Status::Access))
            );
            assert_eq!(preview, [0x55; 600]);
            assert_eq!(e.draft().unwrap().contents, &[7; 500]);
            assert_eq!(e.draft().unwrap().loaded_revision, expected);
        });
    });
}

#[test]
fn stale_history_selection_cannot_restore_a_different_current_revision() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"second").unwrap();
            e.save().unwrap();
            host.borrow_mut().seed(0, b"third");
            let sent = host.borrow().requests.len();
            assert_eq!(e.restore_history(2, 1), Err(Error::Conflict));
            assert!(e.is_conflicted());
            assert_eq!(host.borrow().requests.len(), sent + 1); // STAT only
            assert!(!host.borrow().requests[sent..]
                .iter()
                .any(|h| h.operation == wire::RESTORE));
            assert_eq!(e.draft().unwrap().contents, b"second");
            assert_eq!(host.borrow().service.table().entries[0].revision, 3);
        })
    });
}

#[test]
fn filtered_list_never_publishes_a_partial_or_cross_generation_result() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            for object in 1..PER_DOMAIN as u64 {
                host.borrow_mut().seed(object, b"other");
            }
            let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
            entries[0].revision = 99;
            let before = entries;
            host.borrow_mut().fault = Fault::ChangedListClock;
            assert_eq!(
                e.list_filtered(ListFilter::Uncollected, &mut entries),
                Err(Error::Conflict)
            );
            assert_eq!(entries, before);
            assert!(!e.is_closed());
            host.borrow_mut().fault = Fault::WrongObject;
            assert!(matches!(
                e.list_filtered(ListFilter::All, &mut entries),
                Err(Error::Protocol { .. })
            ));
            assert_eq!(entries, before);
            assert!(e.is_closed());
            assert_eq!(e.draft().unwrap().contents, b"first");
        })
    });
}

#[test]
fn missing_collection_and_stale_collection_cas_preserve_the_document() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            assert_eq!(
                e.set_metadata(1, 42, 0, 1),
                Err(Error::Service(Status::Range))
            );
            assert_eq!(e.draft().unwrap().loaded_revision, 1);
            assert_eq!(e.update_collection(1, 0, b"Home"), Ok(1));
            assert_eq!(e.update_collection(1, 0, b"Replaced"), Err(Error::Conflict));
            let sent = host.borrow().requests.len();
            assert_eq!(e.update_collection(1, 0, b"Replaced"), Err(Error::Conflict));
            assert_eq!(host.borrow().requests.len(), sent);
            assert_eq!(e.collection_stat(1).unwrap().name(), b"Home");
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.draft().unwrap().loaded_revision, 1);
            assert_eq!(host.borrow().service.table().entries[0].collection, 0);
        })
    });
}

#[test]
fn writable_app_cannot_bypass_readonly_volume_service() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"second").unwrap();
            e.save().unwrap();
            let table = *host.borrow().service.table();
            host.borrow_mut().service = Service::new(EPOCH, table, true);
            let writes = host.borrow().disk.writes;
            assert_eq!(
                e.set_metadata(2, 42, 0, 0),
                Err(Error::Service(Status::Access))
            );
            assert_eq!(
                e.update_collection(1, 0, b"Home"),
                Err(Error::Service(Status::Access))
            );
            assert_eq!(e.restore_history(2, 1), Err(Error::Service(Status::Access)));
            assert_eq!(host.borrow().disk.writes, writes);
            assert_eq!(e.draft().unwrap().contents, b"second");
            assert_eq!(e.draft().unwrap().loaded_revision, 2);
            assert!(!e.is_dirty());
            assert!(!e.is_uncertain());
        })
    });
}

#[test]
fn collection_table_io_failure_is_unknown_without_retry() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            let writes = host.borrow().disk.writes;
            host.borrow_mut().disk.fail_write_at = Some(writes);
            assert_eq!(
                e.update_collection(1, 0, b"Home"),
                Err(Error::Service(Status::Unknown))
            );
            assert!(e.is_uncertain());
            assert!(e.is_closed());
            assert_eq!(e.draft().unwrap().contents, b"first");
            assert_eq!(e.draft().unwrap().loaded_revision, 1);
            let sent = host.borrow().requests.len();
            assert_eq!(e.update_collection(1, 0, b"Home"), Err(Error::Uncertain));
            assert_eq!(host.borrow().requests.len(), sent);
        })
    });
}

#[test]
fn verified_history_prefix_uses_existing_staging_and_labels_truncation() {
    run(|| {
        let host = Host::new();
        let historic = "een rustige reis langs de kust. ".repeat(100) + "één";
        host.borrow_mut().seed(0, historic.as_bytes());
        host.borrow_mut().seed(0, b"current");
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
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
        e.replace_contents(b"keep this draft").unwrap();
        let writes = host.borrow().disk.writes;
        let sent = host.borrow().requests.len();
        let mut preview = [0x55; 2048];
        assert_eq!(
            e.read_history_prefix(2, 1, &mut preview),
            Ok(HistoryPrefix {
                copied: 2048,
                total: historic.len(),
                truncated: true
            })
        );
        assert_eq!(&preview, &historic.as_bytes()[..2048]);
        let h = host.borrow();
        assert_eq!(h.disk.writes, writes);
        let reads: Vec<_> = h.requests[sent..]
            .iter()
            .filter(|h| h.operation == wire::READ)
            .collect();
        assert_eq!(reads.len(), historic.len().div_ceil(wire::DATA));
        assert!(reads.iter().all(|h| h.revision == 2 && h.name_length == 1));
        assert!(reads.last().unwrap().aux >= 2048); // Tail verified beyond visible prefix.
        drop(h);
        assert_eq!(e.draft().unwrap().contents, b"keep this draft");
        assert!(e.is_dirty());
    });
}

#[test]
fn history_prefix_never_cuts_a_utf8_character_and_readonly_can_preview() {
    run(|| {
        library_case(EditMode::ReadOnly, |host, e| {
            host.borrow_mut().seed(0, "abc€".as_bytes());
            host.borrow_mut().seed(0, b"current");
            e.open(0).unwrap();
            let mut out = [0x55; 8];
            assert_eq!(
                e.read_history_prefix(3, 1, &mut out[..5]),
                Ok(HistoryPrefix {
                    copied: 3,
                    total: 6,
                    truncated: true
                })
            );
            assert_eq!(&out[..3], b"abc");
            assert_eq!(&out[3..], &[0x55; 5]);
            assert_eq!(
                e.read_history_prefix(3, 1, &mut out),
                Ok(HistoryPrefix {
                    copied: 6,
                    total: 6,
                    truncated: false
                })
            );
            assert_eq!(&out[..6], "abc€".as_bytes());
            assert_eq!(
                e.read_history_prefix(3, 1, &mut []),
                Ok(HistoryPrefix {
                    copied: 0,
                    total: 6,
                    truncated: true
                })
            );
            assert_eq!(e.restore_history(3, 1), Err(Error::ReadOnly));
        })
    });
}

#[test]
fn history_prefix_rejects_invalid_utf8_even_after_the_visible_prefix() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            let mut binary = [b'a'; 500];
            binary[499] = 0xff;
            host.borrow_mut().seed(0, &binary);
            e.open(0).unwrap();
            e.replace_contents(b"current").unwrap();
            e.save().unwrap();
            let mut out = [0x55; 8];
            assert_eq!(
                e.read_history_prefix(3, 1, &mut out),
                Err(Error::InvalidUtf8)
            );
            assert_eq!(out, [0x55; 8]);
            assert!(!e.is_closed()); // Binary content is not a protocol violation.
            let mut raw = [0; 500];
            assert_eq!(e.read_history(3, 1, &mut raw), Ok(500));
            assert_eq!(raw, binary); // Existing opaque-byte API is unchanged.
            assert_eq!(e.draft().unwrap().contents, b"current");
        })
    });
}

#[test]
fn history_prefix_tail_errors_and_timeouts_never_publish_partial_output() {
    run(|| {
        for fault in [Fault::ShortRead, Fault::CorruptLateRead, Fault::DropRead] {
            library_case(EditMode::ReadWrite, |host, e| {
                host.borrow_mut().seed(0, &[b'a'; 500]);
                e.open(0).unwrap();
                e.replace_contents(b"current").unwrap();
                e.save().unwrap();
                host.borrow_mut().fault = fault;
                let mut out = [0x55; 8];
                let error = e.read_history_prefix(3, 1, &mut out).unwrap_err();
                match fault {
                    Fault::ShortRead => assert_eq!(error, Error::Service(Status::Access)),
                    Fault::CorruptLateRead => assert!(matches!(error, Error::Protocol { .. })),
                    Fault::DropRead => assert!(matches!(
                        error,
                        Error::Interrupted {
                            delivery: Delivery::Enqueued,
                            ..
                        }
                    )),
                    _ => unreachable!(),
                }
                assert_eq!(out, [0x55; 8]);
                assert_eq!(e.draft().unwrap().contents, b"current");
                assert_eq!(e.draft().unwrap().loaded_revision, 3);
                assert!(!e.is_uncertain());
                assert_eq!(host.borrow().service.table().entries[0].revision, 3);
            });
        }
    });
}

#[test]
fn history_prefix_stale_revision_or_busy_upload_preserves_output() {
    run(|| {
        library_case(EditMode::ReadWrite, |host, e| {
            e.replace_contents(b"second").unwrap();
            e.save().unwrap();
            let mut out = [0x55; 8];
            e.replace_contents(b"draft").unwrap();
            e.begin_save().unwrap();
            assert_eq!(e.read_history_prefix(2, 1, &mut out), Err(Error::Busy));
            e.cancel_save().unwrap();
            host.borrow_mut().seed(0, b"third");
            assert_eq!(e.read_history_prefix(2, 1, &mut out), Err(Error::Conflict));
            assert_eq!(out, [0x55; 8]);
            assert_eq!(e.draft().unwrap().contents, b"draft");
            assert!(e.is_conflicted());
        })
    });
}
