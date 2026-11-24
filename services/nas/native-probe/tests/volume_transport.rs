// Exercise the actual adapter helper against the volume service's crash model.
#[path = "../../../volume/tests/common/mod.rs"]
mod common;

use std::cell::{Cell, RefCell};

use common::{recover, SimDisk};
use nieuw_os_nas_probe::volume_transport;
use nieuw_os_sdk_runtime::Error;
use volume_core::{
    client::{Client, Transport},
    service::Service,
    table::Table,
    wire, Status,
};

const EPOCH: u64 = 9;

struct Local {
    service: Service,
    disk: SimDisk,
    commit_reply_error: Option<Error>,
    commit_requests: usize,
}

impl Local {
    fn new(commit_reply_error: Option<Error>) -> Self {
        Self {
            service: Service::new(EPOCH, Table::EMPTY, false),
            disk: SimDisk::new(),
            commit_reply_error,
            commit_requests: 0,
        }
    }
}

impl Transport for Local {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let commit = wire::header(request).operation == wire::COMMIT;
        let fault = if commit {
            self.commit_reply_error
        } else {
            None
        };
        let reply = RefCell::new(None);
        volume_transport::exchange(
            || {
                self.commit_requests += usize::from(commit);
                *reply.borrow_mut() = Some(self.service.handle(0, request, &mut self.disk));
                Ok(())
            },
            || match fault {
                Some(error) => Err(error),
                None => Ok(reply.borrow_mut().take().unwrap()),
            },
        )
    }
}

#[test]
fn a_delivered_commit_with_a_failed_reply_is_unknown() {
    // The NAS fixture builds volume-core without L2; exercise its actual WRITE
    // COMMIT. The adapter's delivery-phase handling is independent of opcode.
    for fault in [Error::Kernel(-13), Error::Kernel(-12), Error::InvalidReturn] {
        let mut client = Client::new(Local::new(Some(fault)), EPOCH, 100);
        let result = client.write(0, 0, b"kept", 3, |_, bytes| bytes.copy_from_slice(b"new"));
        assert_eq!(result, Err(Status::Unknown), "receive fault: {fault:?}");
        assert_eq!(
            client.transport.commit_requests, 1,
            "never replay the commit"
        );
        assert_eq!(client.stat(0).unwrap().revision, 1);
        let durable = recover(client.transport.disk.settled()).unwrap();
        assert_eq!(durable.entries[0].revision, 1, "the commit was durable");
    }
}

#[test]
fn failure_before_delivery_keeps_its_status_and_does_not_receive() {
    for (error, status) in [
        (Error::Kernel(-13), Status::Timeout),
        (Error::Kernel(-5), Status::Access),
        (Error::InvalidMessage, Status::Protocol),
    ] {
        let received = Cell::new(false);
        let result = volume_transport::exchange(
            || Err(error),
            || {
                received.set(true);
                Ok([0; wire::SIZE])
            },
        );
        assert_eq!(result, Err(status));
        assert!(!received.get());
    }
}

#[test]
fn received_success_and_service_refusal_keep_their_meaning() {
    let mut client = Client::new(Local::new(None), EPOCH, 100);
    let (revision, _) = client
        .write(0, 0, b"kept", 3, |_, bytes| bytes.copy_from_slice(b"new"))
        .unwrap();
    assert_eq!(revision, 1);
    assert_eq!(
        client.write(0, 0, b"retry", 0, |_, _| {}),
        Err(Status::Stale)
    );
    assert_eq!(client.transport.commit_requests, 1);
    assert_eq!(client.stat(0).unwrap().revision, 1);
}
