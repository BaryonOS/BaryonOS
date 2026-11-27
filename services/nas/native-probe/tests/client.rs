//! `native-client` over encoded NASP frames and the real `dispatch`.
use nieuw_os_nas_client::{Client, Remote, State};
use nieuw_os_nas_core::{Entropy, Error, Service, Storage, Workspace, SNAPSHOT_BYTES};
use nieuw_os_nas_probe::{
    dispatch, Delivery, Frame, Ingress, LocalBinding, NaspRemote, Op, RequestContext, SIZE,
};

#[derive(Default)]
struct Memory {
    bytes: Option<Box<[u8; SNAPSHOT_BYTES]>>,
}
impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        Ok(self.bytes.as_ref().map(|bytes| {
            out.copy_from_slice(bytes.as_slice());
            SNAPSHOT_BYTES
        }))
    }
    fn replace(&mut self, bytes: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        self.bytes = Some(Box::new(*bytes));
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

const ADMIN: u64 = 11;
const GUEST: u64 = 12;

fn request(
    service: &mut Service<'_, Memory>,
    sender: u64,
    bytes: Option<&[u8; SIZE]>,
) -> Result<[u8; SIZE], Delivery> {
    // Receive-only with nothing queued: the bounded receive gives up.
    let Some(bytes) = bytes else {
        return Err(Delivery::Sent);
    };
    let binding = LocalBinding::new(ADMIN, GUEST).unwrap();
    let context = RequestContext {
        binding: &binding,
        ingress: if sender == ADMIN {
            Ingress::Administrator
        } else {
            Ingress::Guest
        },
        sender,
        now: 100,
        recover: false,
    };
    dispatch(service, &context, bytes, &mut Random(50))
        .map(|(reply, _)| reply)
        .map_err(|_| Delivery::Sent)
}

/// Admin invites and the guest accepts, through frames as in the native probe.
fn namespace(service: &mut Service<'_, Memory>) -> [u8; 32] {
    let mut invite = Frame::new(Op::Invite, 1);
    invite.value = 512;
    let reply = request(service, ADMIN, Some(&invite.encode(false).unwrap())).unwrap();
    let invitation = Frame::decode(&reply, true).unwrap();
    let mut accept = Frame::new(Op::Accept, 1);
    accept.namespace = invitation.namespace;
    accept.value = invitation.value;
    accept.length = 32;
    accept.data[..32].copy_from_slice(&invitation.data[..32]);
    let reply = request(service, GUEST, Some(&accept.encode(false).unwrap())).unwrap();
    assert_eq!(Frame::decode(&reply, true).unwrap().status, None);
    invitation.namespace
}

fn op(bytes: Option<&[u8; SIZE]>) -> u16 {
    let bytes = bytes.expect("a request, not a receive-only call");
    u16::from_le_bytes([bytes[6], bytes[7]])
}

#[test]
fn offline_edit_survives_a_nas_restart_and_reconciles_through_head() {
    let mut work = Box::new(Workspace::new());
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(ADMIN),
        &mut Random(10),
        &mut work,
    )
    .unwrap();
    let namespace = namespace(&mut service);
    let guest = LocalBinding::principal(GUEST);
    let mut client = Client::new();
    let first = client.enqueue(namespace, 0, 0, &[0xa5; 96]).unwrap();
    {
        let mut remote = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            request(&mut service, GUEST, b)
        });
        client.prepare_via(first, &mut remote).unwrap();
        assert_eq!(client.commit_via(first, &mut remote), Ok(1));
    }
    client.take(first).unwrap();
    // The NAS is down: nothing is delivered and the edit is retained.
    let edit = client.enqueue(namespace, 0, 1, &[0x5a; 64]).unwrap();
    let mut down = NaspRemote::new(guest, |_: Option<&[u8; SIZE]>| {
        Err(Delivery::NotSent(Error::Io))
    });
    assert_eq!(
        client.prepare_via(edit, &mut down).map_err(|f| f.operation),
        Err(Error::Io)
    );
    assert_eq!(client.edit(edit).unwrap().state(), State::NeedsReconcile);
    // Cold restart from the same storage; old sessions do not survive.
    let storage = service.into_storage();
    let mut work = Box::new(Workspace::new());
    let mut service = Service::recover(storage, &mut Random(30), &mut work).unwrap();
    let mut remote = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
        request(&mut service, GUEST, b)
    });
    assert_eq!(
        client.reconcile_latest_via(edit, &mut remote),
        Ok(State::Ready)
    );
    client.prepare_via(edit, &mut remote).unwrap();
    assert_eq!(client.commit_via(edit, &mut remote), Ok(2));
}

#[test]
fn a_lost_commit_reply_is_unknown_then_observed_without_replay() {
    let mut work = Box::new(Workspace::new());
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(ADMIN),
        &mut Random(10),
        &mut work,
    )
    .unwrap();
    let namespace = namespace(&mut service);
    let guest = LocalBinding::principal(GUEST);
    let mut client = Client::new();
    let edit = client.enqueue(namespace, 0, 0, &[0x33; 40]).unwrap();
    let mut commits = 0;
    {
        // Delivered and applied, but the reply is lost on the way back.
        let mut lossy = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            let reply = request(&mut service, GUEST, b)?;
            if op(b) == Op::Commit as u16 {
                commits += 1;
                return Err(Delivery::Sent);
            }
            Ok(reply)
        });
        client.prepare_via(edit, &mut lossy).unwrap();
        assert_eq!(
            client.commit_via(edit, &mut lossy).map_err(|f| f.operation),
            Err(Error::Unknown)
        );
    }
    assert_eq!(client.edit(edit).unwrap().state(), State::NeedsReconcile);
    let mut sent = [0u16; 16];
    let mut count = 0;
    {
        let mut remote = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            sent[count] = op(b);
            count += 1;
            request(&mut service, GUEST, b)
        });
        assert_eq!(
            client.reconcile_latest_via(edit, &mut remote),
            Ok(State::Observed(1))
        );
    }
    assert_eq!(commits, 1);
    assert_eq!(
        &sent[..count],
        &[
            Op::Open as u16,
            Op::Head as u16,
            Op::Read as u16,
            Op::Close as u16
        ]
    );
}

#[test]
fn oversize_payloads_and_mismatched_replies_fail_closed() {
    let mut work = Box::new(Workspace::new());
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(ADMIN),
        &mut Random(10),
        &mut work,
    )
    .unwrap();
    let namespace = namespace(&mut service);
    let guest = LocalBinding::principal(GUEST);
    let mut client = Client::new();
    let edit = client.enqueue(namespace, 0, 0, &[1; 105]).unwrap();
    let mut stages = 0;
    {
        let mut remote = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            stages += usize::from(op(b) == Op::Stage as u16);
            request(&mut service, GUEST, b)
        });
        assert_eq!(
            client
                .prepare_via(edit, &mut remote)
                .map_err(|f| f.operation),
            Err(Error::Invalid)
        );
    }
    assert_eq!(stages, 0);
    // A reply for another request id is not this exchange's answer. Id 0 is
    // older, but without an earlier `Unknown` it is not dropped as late.
    let mut calls = 0;
    {
        let mut confused = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            calls += 1;
            let mut reply = request(&mut service, GUEST, b)?;
            reply[8] ^= 1;
            Ok(reply)
        });
        assert!(matches!(confused.open(&namespace), Err(Error::Unknown)));
    }
    assert_eq!(calls, 1);
}

#[test]
fn a_late_reply_after_a_timeout_is_dropped_without_resending() {
    let mut work = Box::new(Workspace::new());
    let mut service = Service::initialize(
        Memory::default(),
        LocalBinding::principal(ADMIN),
        &mut Random(10),
        &mut work,
    )
    .unwrap();
    let namespace = namespace(&mut service);
    let guest = LocalBinding::principal(GUEST);
    let mut sent = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    let mut late = true;
    {
        let mut remote = NaspRemote::new(guest, |b: Option<&[u8; SIZE]>| {
            if let Some(bytes) = b {
                sent.push(op(b));
                queue.push_back(request(&mut service, GUEST, Some(bytes))?);
                // The first reply is still in flight when the receive gives up.
                if std::mem::take(&mut late) {
                    return Err(Delivery::Sent);
                }
            }
            queue.pop_front().ok_or(Delivery::Sent)
        });
        // Request 1 was applied, but its reply arrives only after the timeout.
        assert!(matches!(remote.open(&namespace), Err(Error::Unknown)));
        // Request 2 first receives request 1's late Open reply, drops it and
        // then receives its own reply; nothing is sent twice.
        let session = remote.open(&namespace).unwrap();
        assert_eq!(remote.head(session, 0), Ok(0));
        remote.close(session).unwrap();
    }
    assert_eq!(
        sent,
        [
            Op::Open as u16,
            Op::Open as u16,
            Op::Head as u16,
            Op::Close as u16
        ]
    );
    assert!(queue.is_empty());
}
