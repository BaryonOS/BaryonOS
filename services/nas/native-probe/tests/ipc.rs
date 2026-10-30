use nieuw_os_nas_core::{
    Entropy, Error, NamespaceStatus, Service, Storage, Workspace, SNAPSHOT_BYTES,
};
use nieuw_os_nas_probe::{dispatch, Frame, Ingress, LocalBinding, Op, RequestContext, SIZE};

#[derive(Default)]
struct Memory {
    bytes: Option<Box<[u8; SNAPSHOT_BYTES]>>,
    writes: usize,
}
impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        if let Some(bytes) = &self.bytes {
            out.copy_from_slice(bytes.as_slice());
            Ok(Some(SNAPSHOT_BYTES))
        } else {
            Ok(None)
        }
    }
    fn replace(&mut self, bytes: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        self.bytes = Some(Box::new(*bytes));
        self.writes += 1;
        Ok(())
    }
}
struct Random(u8);
impl Entropy for Random {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), Error> {
        self.0 += 1;
        out.fill(self.0);
        Ok(())
    }
}

fn context(binding: &LocalBinding, admin: bool) -> RequestContext<'_> {
    RequestContext {
        binding,
        ingress: if admin {
            Ingress::Administrator
        } else {
            Ingress::Guest
        },
        sender: if admin {
            binding.admin_pid
        } else {
            binding.guest_pid
        },
        now: 100,
        recover: false,
    }
}
fn call(
    service: &mut Service<'_, Memory>,
    context: &RequestContext<'_>,
    request: Frame,
    status: Option<Error>,
    entropy: &mut Random,
) -> Frame {
    let (bytes, shutdown) =
        dispatch(service, context, &request.encode(false).unwrap(), entropy).unwrap();
    let reply = Frame::decode(&bytes, true).unwrap();
    assert_eq!(
        (reply.op, reply.request, reply.status),
        (request.op, request.request, status)
    );
    assert_eq!(shutdown, request.op == Op::Shutdown && status.is_none());
    if status.is_some() {
        assert_eq!(&bytes[20..], &[0; SIZE - 20]);
    }
    reply
}
fn named(op: Op, namespace: [u8; 32]) -> Frame {
    let mut request = Frame::new(op, 1);
    request.namespace = namespace;
    request
}

#[test]
fn malformed_envelopes_and_all_oversize_payload_lengths_fail_closed() {
    let original = Frame::new(Op::Shutdown, 1).encode(false).unwrap();
    for offset in [0, 4, 6, 16, 22, 24, 56, 96, 136, 144, 152, 255] {
        let mut bytes = original;
        bytes[offset] ^= 0x40;
        assert!(matches!(Frame::decode(&bytes, false), Err(Error::Invalid)));
    }
    for length in 105..=u16::MAX {
        let mut bytes = original;
        bytes[20..22].copy_from_slice(&length.to_le_bytes());
        assert!(matches!(Frame::decode(&bytes, false), Err(Error::Invalid)));
    }
    assert!(matches!(
        Frame::decode(&original, true),
        Err(Error::Invalid)
    ));
    let mut invalid = Frame::new(Op::Shutdown, 1);
    invalid.length = usize::MAX;
    assert_eq!(invalid.encode(false), Err(Error::Invalid));
}

#[test]
fn malformed_success_and_error_replies_never_pass_shape_validation() {
    let original = Frame::new(Op::Shutdown, 1).encode(true).unwrap();
    for offset in [20, 24, 56, 96, 136, 144, 152, 255] {
        let mut bytes = original;
        bytes[offset] = 1;
        assert!(matches!(Frame::decode(&bytes, true), Err(Error::Invalid)));
    }
    let mut error = Frame::new(Op::Read, 1);
    error.status = Some(Error::Unknown);
    let bytes = error.encode(true).unwrap();
    assert_eq!(
        Frame::decode(&bytes, true).unwrap().status,
        Some(Error::Unknown)
    );
    error.namespace[0] = 1;
    assert_eq!(error.encode(true), Err(Error::Invalid));
    let mut bytes_with_content = bytes;
    bytes_with_content[24] = 1;
    assert!(matches!(
        Frame::decode(&bytes_with_content, true),
        Err(Error::Invalid)
    ));
    assert_eq!(Frame::new(Op::Open, 1).encode(true), Err(Error::Invalid));
    let mut usage = Frame::new(Op::Usage, 1);
    usage.namespace = [1; 32];
    usage.revision = 1;
    usage.value = 128;
    usage.length = 16;
    usage.data[0] = NamespaceStatus::Active as u8;
    let valid = usage.encode(true).unwrap();
    assert!(Frame::decode(&valid, true).is_ok());
    for (offset, value) in [(152, 5), (153, 1), (156, 129), (164, 1)] {
        let mut bytes = valid;
        bytes[offset] = value;
        assert!(matches!(Frame::decode(&bytes, true), Err(Error::Invalid)));
    }
}

#[test]
fn actual_sender_and_ingress_cannot_be_substituted_by_frame_contents() {
    let mut work = Box::new(Workspace::new());
    let binding = LocalBinding::new(11, 12).unwrap();
    let mut entropy = Random(10);
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(11),
        &mut entropy,
        &mut work,
    )
    .unwrap();
    let mut ctx = context(&binding, true);
    ctx.sender = 12;
    assert!(matches!(
        dispatch(
            &mut service,
            &ctx,
            &Frame::new(Op::Shutdown, 1).encode(false).unwrap(),
            &mut entropy
        ),
        Err(Error::Denied)
    ));
    call(
        &mut service,
        &context(&binding, false),
        Frame::new(Op::Shutdown, 1),
        Some(Error::Denied),
        &mut entropy,
    );
    let mut forged = Frame::new(Op::Shutdown, 1).encode(false).unwrap();
    forged[24..32].copy_from_slice(&11u64.to_le_bytes());
    assert!(matches!(
        dispatch(
            &mut service,
            &context(&binding, false),
            &forged,
            &mut entropy
        ),
        Err(Error::Invalid)
    ));
    assert_eq!(service.into_storage().writes, 1);
}

#[test]
fn native_probe_workload_preserves_policy_through_encoded_requests_and_recovery() {
    let mut work = Box::new(Workspace::new());
    let binding = LocalBinding::new(11, 12).unwrap();
    let admin = context(&binding, true);
    let guest = context(&binding, false);
    let mut entropy = Random(10);
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(11),
        &mut entropy,
        &mut work,
    )
    .unwrap();
    let mut request = Frame::new(Op::Invite, 1);
    request.value = 128;
    let invitation = call(&mut service, &admin, request, None, &mut entropy);
    let namespace = invitation.namespace;
    let mut accept = named(Op::Accept, namespace);
    accept.value = invitation.value;
    accept.length = 32;
    accept.data[..32].copy_from_slice(&invitation.data[..32]);
    call(&mut service, &guest, accept, None, &mut entropy);
    let session = call(
        &mut service,
        &guest,
        named(Op::Open, namespace),
        None,
        &mut entropy,
    )
    .session;
    let mut request = Frame::new(Op::Stage, 1);
    request.session = session;
    request.length = 96;
    request.data[..96].fill(0xa5);
    let upload = call(&mut service, &guest, request, None, &mut entropy).upload;
    let mut commit = Frame::new(Op::Commit, 1);
    commit.session = session;
    commit.upload = upload;
    assert_eq!(
        call(&mut service, &guest, commit, None, &mut entropy).revision,
        1
    );
    let mut read = Frame::new(Op::Read, 1);
    read.session = session;
    read.revision = 1;
    let bytes = call(&mut service, &guest, read, None, &mut entropy);
    assert_eq!((bytes.length, &bytes.data[..96]), (96, &[0xa5; 96][..]));
    let mut request = Frame::new(Op::Stage, 1);
    request.session = session;
    request.revision = 1;
    request.length = 32;
    request.data[..32].fill(0x55);
    let pending = call(&mut service, &guest, request, None, &mut entropy).upload;
    let usage = call(
        &mut service,
        &admin,
        Frame::new(Op::UsageAt, 1),
        None,
        &mut entropy,
    );
    assert_eq!(usage.namespace, namespace);
    assert_eq!(&usage.data[4..12], &[96, 0, 0, 0, 32, 0, 0, 0]);
    let mut quota = named(Op::SetQuota, namespace);
    quota.value = 127;
    call(
        &mut service,
        &admin,
        quota,
        Some(Error::Quota),
        &mut entropy,
    );
    call(
        &mut service,
        &guest,
        named(Op::Revoke, namespace),
        Some(Error::Denied),
        &mut entropy,
    );
    call(
        &mut service,
        &admin,
        named(Op::Revoke, namespace),
        None,
        &mut entropy,
    );
    let mut old = Frame::new(Op::Commit, 1);
    old.session = session;
    old.upload = pending;
    call(&mut service, &guest, old, Some(Error::Stale), &mut entropy);
    call(
        &mut service,
        &guest,
        named(Op::Open, namespace),
        Some(Error::Denied),
        &mut entropy,
    );
    call(
        &mut service,
        &admin,
        named(Op::Evict, namespace),
        None,
        &mut entropy,
    );
    let storage = service.into_storage();
    let writes = storage.writes;
    let mut service = Service::recover(storage, &mut entropy, &mut work).unwrap();
    let usage = call(
        &mut service,
        &admin,
        Frame::new(Op::UsageAt, 1),
        None,
        &mut entropy,
    );
    assert_eq!(usage.data[0], NamespaceStatus::Evicted as u8);
    assert_eq!(&usage.data[4..12], &[0; 8]);
    assert_eq!(usage.revision, 2);
    call(
        &mut service,
        &guest,
        named(Op::Open, namespace),
        Some(Error::Denied),
        &mut entropy,
    );
    let mut old = Frame::new(Op::Read, 1);
    old.session = session;
    old.revision = 1;
    call(&mut service, &guest, old, Some(Error::Stale), &mut entropy);
    call(
        &mut service,
        &admin,
        Frame::new(Op::Shutdown, 1),
        None,
        &mut entropy,
    );
    assert_eq!(service.into_storage().writes, writes);
}

#[test]
fn reset_monotonic_ticks_never_authorize_an_old_offer_in_recovery_probe() {
    let mut work = Box::new(Workspace::new());
    let binding = LocalBinding::new(11, 12).unwrap();
    let mut entropy = Random(10);
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(11),
        &mut entropy,
        &mut work,
    )
    .unwrap();
    let mut invite = Frame::new(Op::Invite, 1);
    invite.value = 128;
    let invite = call(
        &mut service,
        &context(&binding, true),
        invite,
        None,
        &mut entropy,
    );
    let mut service = Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
    let mut ctx = context(&binding, false);
    ctx.recover = true;
    ctx.now = 0;
    let mut accept = named(Op::Accept, invite.namespace);
    accept.value = invite.value;
    accept.length = 32;
    accept.data[..32].copy_from_slice(&invite.data[..32]);
    call(
        &mut service,
        &ctx,
        accept,
        Some(Error::Expired),
        &mut entropy,
    );
    call(
        &mut service,
        &ctx,
        named(Op::Open, invite.namespace),
        Some(Error::Denied),
        &mut entropy,
    );
}
