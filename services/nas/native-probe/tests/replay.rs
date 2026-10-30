use std::{cell::RefCell, rc::Rc};

use nieuw_os_nas_core::{
    Entropy, Error, Invitation, Principal, Service, Storage, Workspace, MAX_QUOTA, SNAPSHOT_BYTES,
    TOKEN_BYTES,
};

const ADMIN: Principal = Principal([1; 32]);
const GUEST: Principal = Principal([2; 32]);

struct TestEntropy(u8);
impl Entropy for TestEntropy {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), Error> {
        self.0 = self.0.checked_add(1).ok_or(Error::Entropy)?;
        out.fill(self.0);
        Ok(())
    }
}

#[derive(Default)]
struct Disk {
    snapshot: Option<Vec<u8>>,
    writes: usize,
}

#[derive(Clone, Default)]
struct Memory(Rc<RefCell<Disk>>);
impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        let disk = self.0.borrow();
        let Some(snapshot) = &disk.snapshot else {
            return Ok(None);
        };
        out.copy_from_slice(snapshot);
        Ok(Some(snapshot.len()))
    }

    fn replace(&mut self, snapshot: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        let mut disk = self.0.borrow_mut();
        disk.snapshot = Some(snapshot.to_vec());
        disk.writes += 1;
        Ok(())
    }
}

struct HostReplayArtifact {
    origin: &'static str,
    role: &'static str,
    stage: &'static str,
    completed: bool,
    payload: [u8; TOKEN_BYTES * 2],
}

impl HostReplayArtifact {
    fn validate_shape(&self) -> bool {
        !self.origin.is_empty()
            && self.role == "guest"
            && self.stage == "normal-after-stage"
            && self.completed
            && self.payload.len() == 80
    }
}

#[test]
fn genuine_core_issued_handles_are_rejected_after_recovery_without_ack_or_write() {
    let mut work = Box::new(Workspace::new());
    let disk = Memory::default();
    let mut entropy = TestEntropy(10);
    let mut service = Service::initialize(disk.clone(), ADMIN, &mut entropy, &mut work).unwrap();
    let invitation: Invitation = service
        .invite(ADMIN, GUEST, MAX_QUOTA, 100, 10, &mut entropy)
        .unwrap();
    service.accept(GUEST, &invitation, 100).unwrap();
    let session = service.open(GUEST, &invitation.namespace).unwrap();
    let upload = service
        .stage(GUEST, session, 0, 0, b"opaque staged test bytes")
        .unwrap();

    let mut payload = [0; TOKEN_BYTES * 2];
    payload[..TOKEN_BYTES].copy_from_slice(&session.to_bytes());
    payload[TOKEN_BYTES..].copy_from_slice(&upload.to_bytes());
    let artifact = HostReplayArtifact {
        origin: "host-core-normal-run",
        role: "guest",
        stage: "normal-after-stage",
        completed: true,
        payload,
    };
    assert!(artifact.validate_shape());
    assert_eq!(&artifact.payload[..40], &session.to_bytes());
    assert_eq!(&artifact.payload[40..], &upload.to_bytes());
    let storage = service.into_storage();
    let writes = disk.0.borrow().writes;

    let mut recovered = Service::recover(storage, &mut TestEntropy(90), &mut work).unwrap();

    // The genuine normal-run session is rejected in the recovered instance.
    let mut read = [0; 1024];
    assert_eq!(
        recovered.read(GUEST, session, 0, 1, &mut read),
        Err(Error::Stale)
    );

    // A new, valid session isolates the stale-upload check from stale-session
    // rejection. The namespace remains active in this host-only fixture.
    let new_session = recovered.open(GUEST, &invitation.namespace).unwrap();
    assert_eq!(
        recovered.commit(GUEST, new_session, upload),
        Err(Error::Stale)
    );
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(
        recovered.usage(ADMIN, &invitation.namespace).unwrap().used,
        0
    );
    assert_eq!(
        recovered
            .usage(ADMIN, &invitation.namespace)
            .unwrap()
            .reserved,
        0
    );
}

#[test]
fn incomplete_or_wrong_origin_host_artifact_metadata_is_not_accepted() {
    let payload = [0; TOKEN_BYTES * 2];
    for (origin, role, stage, completed) in [
        ("", "guest", "normal-after-stage", true),
        ("host-core-normal-run", "admin", "normal-after-stage", true),
        ("host-core-normal-run", "guest", "cold", true),
        ("host-core-normal-run", "guest", "normal-after-stage", false),
    ] {
        let artifact = HostReplayArtifact {
            origin,
            role,
            stage,
            completed,
            payload,
        };
        assert!(!artifact.validate_shape());
    }
}
