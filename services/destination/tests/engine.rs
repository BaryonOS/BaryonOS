//! Rows that need the real capability engine: N5, N6, N22 (reply queue),
//! N27. The manifest of 4.3 is built with `capability-core`'s `Engine`; a
//! small shell plays the native service loop (receive, `Service::handle`,
//! close attached handles, send the reply).
mod common;

use capability_core::{Cap, Engine, Error, Message, RECV, REVOKE, SEND, TRANSFER};
use common::*;
use destination_core::{
    service::Meta,
    wire::{self, Header},
    Status,
};

const SERVICE: u64 = 1;

struct World {
    e: Box<Engine>,
    h: H,
    /// Service RECV handles: control lane, then lanes 1-3.
    recv: [u64; 4],
    /// Dialog handles: control-lane SEND, then the lane roots.
    dialog: [u64; 4],
    dialog_reply: u64,
    svc_dialog_reply: u64,
    portal_reply: u64,
    svc_portal_reply: u64,
    portal_intake: u64,
    dialog_intake: u64,
}

impl World {
    fn new() -> Self {
        let mut e = Box::new(Engine::new());
        for pid in [SERVICE, DIALOG, PORTAL] {
            e.register(pid).unwrap();
        }
        let mut recv = [0; 4];
        let mut dialog = [0; 4];
        for lane in 0..4 {
            let root = e.create_endpoint(SERVICE).unwrap();
            recv[lane] = e.derive(SERVICE, root, RECV).unwrap();
            let rights = if lane == 0 {
                SEND
            } else {
                SEND | TRANSFER | REVOKE
            };
            dialog[lane] = e.bootstrap_delegate(SERVICE, root, DIALOG, rights).unwrap();
            e.close(SERVICE, root).unwrap();
        }
        let mut channel = |owner: u64, to: u64| {
            let root = e.create_endpoint(owner).unwrap();
            let own = e.derive(owner, root, RECV).unwrap();
            let delegated = e.bootstrap_delegate(owner, root, to, SEND).unwrap();
            e.close(owner, root).unwrap();
            (own, delegated)
        };
        let (dialog_reply, svc_dialog_reply) = channel(DIALOG, SERVICE);
        let (portal_reply, svc_portal_reply) = channel(PORTAL, SERVICE);
        let (portal_intake, dialog_intake) = channel(PORTAL, DIALOG);
        Self {
            e,
            h: H::formatted(),
            recv,
            dialog,
            dialog_reply,
            svc_dialog_reply,
            portal_reply,
            svc_portal_reply,
            portal_intake,
            dialog_intake,
        }
    }

    fn message(&mut self, lane: usize, header: Header, data: &[u8], caps: &[Cap]) -> Message {
        let mut header = header;
        self.h.next_id[lane] += 1;
        header.request_id = self.h.next_id[lane];
        header.epoch = self.h.generation[lane];
        header.length = data.len() as u32;
        let body = wire::encode(&header, data);
        let mut m = Message::new();
        m.len = wire::SIZE as u32;
        m.data[..wire::SIZE].copy_from_slice(&body);
        m.cap_count = caps.len() as u32;
        m.caps[..caps.len()].copy_from_slice(caps);
        m
    }

    /// One pass of the service loop over all four lanes.
    fn serve(&mut self) -> Vec<Status> {
        let mut statuses = Vec::new();
        for lane in 0..4 {
            while let Ok(m) = self.e.receive(SERVICE, self.recv[lane]) {
                let mut body = [0; wire::SIZE];
                body.copy_from_slice(&m.data[..wire::SIZE]);
                let meta = Meta {
                    lane,
                    sender_pid: m.sender_pid,
                    cap_count: m.cap_count,
                };
                let outcome = self.h.send(meta, &body);
                if outcome.close_caps {
                    for cap in &m.caps[..m.cap_count as usize] {
                        self.e.close(SERVICE, cap.handle).unwrap();
                    }
                }
                if let Some(reply) = outcome.reply {
                    let mut r = Message::new();
                    r.len = wire::SIZE as u32;
                    r.data[..wire::SIZE].copy_from_slice(&reply);
                    let to = if lane == 0 {
                        self.svc_dialog_reply
                    } else {
                        self.svc_portal_reply
                    };
                    // Never block on a reply: a full queue drops it.
                    let _ = self.e.send(SERVICE, to, &r);
                }
                statuses.push(outcome.status);
            }
        }
        statuses
    }

    fn replies(&mut self, pid: u64, endpoint: u64) -> Vec<[u8; wire::SIZE]> {
        let mut out = Vec::new();
        while let Ok(m) = self.e.receive(pid, endpoint) {
            let mut body = [0; wire::SIZE];
            body.copy_from_slice(&m.data[..wire::SIZE]);
            out.push(body);
        }
        out
    }

    fn control(&mut self, header: Header, data: &[u8]) -> Status {
        let m = self.message(0, header, data, &[]);
        self.e.send(DIALOG, self.dialog[0], &m).unwrap();
        self.serve();
        let replies = self.replies(DIALOG, self.dialog_reply);
        assert_eq!(replies.len(), 1);
        status_of(&replies[0])
    }

    /// BIND on the control lane, then transfer SEND on the lane to the portal.
    fn grant(&mut self, lane: usize, operation: u32, entry: u64) -> u64 {
        let generation = self.h.generation[lane] + 1;
        let mut bind = req(wire::BIND);
        bind.count = lane as u32;
        bind.name_length = operation;
        bind.entry = entry;
        bind.aux = generation;
        assert_eq!(self.control(bind, &TEST_ID), Status::Ok);
        self.h.generation[lane] = generation;
        self.h.next_id[lane] = 0;
        let mut m = Message::new();
        m.cap_count = 1;
        m.caps[0] = Cap {
            handle: self.dialog[lane],
            rights: SEND,
        };
        self.e.send(DIALOG, self.dialog_intake, &m).unwrap();
        let received = self.e.receive(PORTAL, self.portal_intake).unwrap();
        received.caps[0].handle
    }

    fn portal_send(
        &mut self,
        handle: u64,
        lane: usize,
        header: Header,
        data: &[u8],
    ) -> Result<(), Error> {
        let m = self.message(lane, header, data, &[]);
        self.e.send(PORTAL, handle, &m)
    }
}

/// N5: the dialog revoked the lane before the operation: the portal's send
/// fails in the kernel with `Revoked` -10 (syscall return, not a service
/// status). Kills: a revoke that does not reach the transferred child node.
#[test]
fn n5_revoked_before_operation() {
    let mut w = World::new();
    let lane = w.grant(1, wire::BIND_EXPORT, 0);
    assert_eq!(w.portal_send(lane, 1, req(wire::INFO), &[]), Ok(()));
    w.serve();
    assert_eq!(status_of(&w.replies(PORTAL, w.portal_reply)[0]), Status::Ok);
    w.e.revoke(DIALOG, w.dialog[1]).unwrap();
    assert_eq!(
        w.portal_send(lane, 1, req(wire::INFO), &[]),
        Err(Error::Revoked)
    );
    // The dialog keeps its own node for the next grant.
    assert!(w
        .e
        .check_endpoint(DIALOG, w.dialog[1], SEND | TRANSFER)
        .is_ok());
}

/// N6: revoked during an export at about 50 %: the dialog sends UNBIND,
/// waits for `Ok`, then revokes. The writing entry is discarded, LIST is
/// unchanged, earlier entries verify, the portal's next send is `Revoked`.
#[test]
fn n6_revoked_during_export() {
    let mut w = World::new();
    let e1_bytes = archive(1, 1, 400, 1);
    let e1 = w.h.export(1, &e1_bytes);
    let lane = w.grant(1, wire::BIND_EXPORT, 0);
    let mut create = req(wire::CREATE);
    create.name_length = 1;
    w.portal_send(lane, 1, create, b"x").unwrap();
    w.serve();
    let entry = wire::header(&w.replies(PORTAL, w.portal_reply)[0]).entry;
    let bytes = archive(2, 1, 900, 2);
    let half = bytes.len() / 2;
    for (index, chunk) in bytes[..half].chunks(wire::DATA).enumerate() {
        let mut append = req(wire::APPEND);
        append.entry = entry;
        append.offset = (index * wire::DATA) as u64;
        w.portal_send(lane, 1, append, chunk).unwrap();
        assert_eq!(w.serve(), vec![Status::Ok]);
    }
    w.replies(PORTAL, w.portal_reply);
    // A SEAL already queued behind the APPENDs when UNBIND arrives.
    let mut seal = req(wire::SEAL);
    seal.entry = entry;
    seal.offset = half as u64;
    let queued = w.message(1, seal, &[], &[]);
    let mut unbind = req(wire::UNBIND);
    unbind.count = 1;
    let unbind_message = w.message(0, unbind, &[], &[]);
    w.e.send(DIALOG, w.dialog[0], &unbind_message).unwrap();
    w.e.send(PORTAL, lane, &queued).unwrap();
    // The control lane is served first in this loop pass: UNBIND is ordered
    // before the queued SEAL, which then gets `Access` and cannot commit.
    assert_eq!(w.serve(), vec![Status::Ok, Status::Access]);
    assert_eq!(status_of(&w.replies(DIALOG, w.dialog_reply)[0]), Status::Ok);
    w.e.revoke(DIALOG, w.dialog[1]).unwrap();
    assert_eq!(
        w.portal_send(lane, 1, req(wire::INFO), &[]),
        Err(Error::Revoked)
    );
    assert!(w.h.svc.binding(1).is_none());
    assert_eq!(w.h.list_ids(0).1, vec![e1]);
    assert_eq!(w.h.bind(2, wire::BIND_VERIFY, e1), Status::Ok);
    assert_eq!(w.h.verify_entry(2, e1), Status::Ok);
    // UNBIND discarded the writing entry: a new export lane may CREATE (not
    // `Busy`). Kills: an UNBIND that keeps the writing entry (M30).
    assert_eq!(w.h.bind(3, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(w.h.create(3, b"z").status, Status::Ok);
}

/// N6 (b): the service exits instead. The dialog's UNBIND `send` returns
/// `Revoked` -10, which permits the revoke step; the revoke then succeeds or
/// is a no-op, and the portal's send is `Revoked`. The kill named by the spec
/// (a dialog that waits for `Closed` only) targets dialog code, which is not
/// part of this crate; only these engine facts are tested here.
#[test]
fn n6b_service_exit() {
    let mut w = World::new();
    let lane = w.grant(1, wire::BIND_EXPORT, 0);
    w.e.exit(SERVICE).unwrap();
    let mut unbind = req(wire::UNBIND);
    unbind.count = 1;
    let m = w.message(0, unbind, &[], &[]);
    let sent = w.e.send(DIALOG, w.dialog[0], &m);
    assert!(
        matches!(sent, Err(Error::Revoked) | Err(Error::Closed)),
        "{sent:?}"
    );
    let revoked = w.e.revoke(DIALOG, w.dialog[1]);
    assert!(
        matches!(revoked, Ok(()) | Err(Error::Revoked) | Err(Error::Closed)),
        "{revoked:?}"
    );
    let portal = w.portal_send(lane, 1, req(wire::INFO), &[]);
    assert!(
        matches!(portal, Err(Error::Revoked) | Err(Error::Closed)),
        "{portal:?}"
    );
}

/// N27: a request carrying a transferred handle, on the control lane and on
/// a bound operation lane. Handler status `Invalid`; the control-lane reply
/// goes to the dialog, the operation-lane request gets none; the service's
/// handle count is unchanged; no state change; the high-water mark holds.
/// Kills: a service that keeps the received handle (count grows).
#[test]
fn n27_transferred_capability_closed() {
    let mut w = World::new();
    let _lane = w.grant(1, wire::BIND_EXPORT, 0);
    let handles = w.e.process_usage(SERVICE).unwrap().handles;
    let cap = Cap {
        handle: w.dialog[2],
        rights: SEND,
    };
    // Control lane.
    let request_id = w.h.next_id[0] + 1;
    let m = w.message(0, req(wire::INFO), &[], &[cap]);
    w.e.send(DIALOG, w.dialog[0], &m).unwrap();
    assert_eq!(w.serve(), vec![Status::Invalid]);
    assert_eq!(w.e.process_usage(SERVICE).unwrap().handles, handles);
    let replies = w.replies(DIALOG, w.dialog_reply);
    assert_eq!(replies.len(), 1);
    assert_eq!(status_of(&replies[0]), Status::Invalid);
    // The same request ID is still fresh: the mark did not move.
    w.h.next_id[0] = request_id - 1;
    assert_eq!(w.control(req(wire::INFO), &[]), Status::Ok);
    // Bound operation lane, sent by the dialog with its own root.
    let m = w.message(1, req(wire::INFO), &[], &[cap]);
    w.e.send(DIALOG, w.dialog[1], &m).unwrap();
    assert_eq!(w.serve(), vec![Status::Invalid]);
    assert_eq!(w.e.process_usage(SERVICE).unwrap().handles, handles);
    assert!(
        w.replies(PORTAL, w.portal_reply).is_empty(),
        "no reply to the portal"
    );
    assert!(w.h.svc.binding(1).is_some());
}

/// N22 (reply queue): the dialog's requests on a lane bound to the portal
/// never put a reply in the portal's reply queue.
/// Kills: a service that replies to the portal for a non-portal sender.
#[test]
fn n22_no_reply_to_portal_for_dialog_requests() {
    let mut w = World::new();
    let e1 = w.h.export(3, &archive(1, 1, 200, 1));
    let _lane = w.grant(1, wire::BIND_EXPORT, 0);
    let _restore = w.grant(2, wire::BIND_RESTORE, e1);
    let mut read = req(wire::READ);
    read.entry = e1;
    read.aux = 5;
    for (lane, header) in [(1, req(wire::INFO)), (2, read)] {
        let m = w.message(lane, header, &[], &[]);
        w.e.send(DIALOG, w.dialog[lane], &m).unwrap();
        assert_eq!(w.serve(), vec![Status::Access]);
        assert!(w.replies(PORTAL, w.portal_reply).is_empty());
    }
}
