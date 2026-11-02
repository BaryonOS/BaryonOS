//! Controlled IPC transport model; not a native NIC/kernel authentication test.
use nieuw_os_catalog_client::Transport;
use nieuw_os_net_client::{Connection, Error, Session, Status, DATA};
use nieuw_os_network_protocol::{Operation, Packet, TAG};
use nieuw_os_sdk_runtime::{
    Capability, Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, Rights,
    SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Copy)]
enum Fault {
    None,
    Lost,
    Peer,
    Tag,
    Request,
    Caps,
    Receive,
    SendAgain,
    UnknownSend,
}
struct Model {
    fault: Fault,
    replies: Vec<[u8; 320]>,
    sends: usize,
    yields: usize,
    closed: Vec<u64>,
}
struct Port(Rc<RefCell<Model>>);
fn handle(slot: u64) -> Handle {
    Handle::from_raw(1 << 32 | slot).unwrap()
}
impl Transport for Port {
    fn send(&mut self, _: SendEndpoint, m: &Outgoing) -> Result<(), RuntimeError> {
        let mut model = self.0.borrow_mut();
        model.sends += 1;
        if matches!(model.fault, Fault::SendAgain) {
            return Err(RuntimeError::Kernel(-8));
        }
        let request = Packet::decode_request(&m.as_bytes()[64..320]).unwrap();
        let mut h = request.header();
        let arg = match h.operation {
            Operation::OpenTcp => {
                h.connection = 257;
                1
            }
            Operation::State => 2,
            Operation::SendTcp | Operation::SendUdp => h.argument,
            _ => 0,
        };
        let p = Packet::response(h, Status::Ok, arg, &[]);
        let mut message = Outgoing::new(TAG, p.bytes()).unwrap();
        if matches!(model.fault, Fault::Caps) {
            message
                .push_cap(Capability {
                    handle: handle(9),
                    rights: Rights::SEND,
                })
                .unwrap();
        }
        let mut bytes = *message.as_bytes();
        bytes[..8].copy_from_slice(&3u64.to_le_bytes());
        match model.fault {
            Fault::Peer => bytes[..8].copy_from_slice(&4u64.to_le_bytes()),
            Fault::Tag => bytes[8] ^= 1,
            Fault::Request => bytes[64 + 8] ^= 1,
            _ => {}
        }
        if !matches!(model.fault, Fault::Lost) {
            model.replies.push(bytes);
        }
        if matches!(model.fault, Fault::UnknownSend) {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut m = self.0.borrow_mut();
        if matches!(m.fault, Fault::Receive) {
            return Err(RuntimeError::InvalidReturn);
        }
        if m.replies.is_empty() {
            return Err(RuntimeError::Kernel(-8));
        }
        let b = m.replies.remove(0);
        Received::from_bytes(b, 256)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        self.0.borrow_mut().yields += 1;
        Ok(())
    }
    fn close(&mut self, h: Handle) -> Result<(), RuntimeError> {
        self.0.borrow_mut().closed.push(h.raw());
        Ok(())
    }
}
fn make(fault: Fault) -> (Session<Port>, Rc<RefCell<Model>>) {
    let model = Rc::new(RefCell::new(Model {
        fault,
        replies: Vec::new(),
        sends: 0,
        yields: 0,
        closed: Vec::new(),
    }));
    let s = Session::new(
        Port(model.clone()),
        19,
        3,
        SendEndpoint::from_handle(handle(1)),
        ReceiveEndpoint::from_handle(handle(2)),
        3,
    )
    .unwrap();
    (s, model)
}
#[test]
fn queued_mutation_is_never_resent_after_lost_reply() {
    let (mut s, m) = make(Fault::Lost);
    assert_eq!(s.open([192, 0, 2, 1], 8080, 300), Err(Error::Uncertain));
    assert_eq!(m.borrow().sends, 1);
    assert!(s.is_closed());
    assert_eq!(m.borrow().closed.len(), 2);
    assert_eq!(s.open([192, 0, 2, 1], 8080, 300), Err(Error::Closed));
    assert_eq!(m.borrow().sends, 1);
}
#[test]
fn peer_tag_request_and_received_caps_are_not_trusted() {
    for f in [Fault::Peer, Fault::Tag, Fault::Request, Fault::Caps] {
        let (mut s, m) = make(f);
        assert_eq!(s.open([192, 0, 2, 1], 8080, 300), Err(Error::Protocol));
        assert!(s.is_closed());
        assert!(m.borrow().closed.contains(&handle(1).raw()));
        assert!(m.borrow().closed.contains(&handle(2).raw()));
        if matches!(f, Fault::Caps) {
            assert!(m.borrow().closed.contains(&handle(9).raw()));
        }
    }
}
#[test]
fn malformed_syscall_and_unknown_send_stop_the_session() {
    for f in [Fault::Receive, Fault::UnknownSend] {
        let (mut s, m) = make(f);
        assert!(s.open([192, 0, 2, 1], 8080, 300).is_err());
        assert_eq!(m.borrow().sends, 1);
        assert!(s.is_closed());
    }
}
#[test]
fn eagain_is_bounded_and_never_becomes_success() {
    let (mut s, m) = make(Fault::SendAgain);
    assert_eq!(s.open([192, 0, 2, 1], 8080, 300), Err(Error::Uncertain));
    assert_eq!(m.borrow().sends, 3);
    assert!(s.is_closed());
}
#[test]
fn buffers_are_rejected_before_any_exchange() {
    let (mut s, m) = make(Fault::None);
    let c = Connection::from_id(257).unwrap();
    assert_eq!(s.send(c, &[0; DATA + 1]), Err(Error::Buffer));
    assert_eq!(s.receive(c, &mut [0; DATA - 1]), Err(Error::Buffer));
    assert_eq!(s.receive_udp(&mut [0; DATA - 1]), Err(Error::Buffer));
    assert_eq!(m.borrow().sends, 0);
}
#[test]
fn connection_and_explicit_retirement_use_the_bound_service() {
    let (mut s, m) = make(Fault::None);
    let c = s.open([192, 0, 2, 1], 8080, 300).unwrap();
    assert_eq!(c.id(), 257);
    assert_eq!(s.send(c, b"hello"), Ok(5));
    s.retire().unwrap();
    assert!(s.is_closed());
    assert_eq!(m.borrow().sends, 3);
    assert_eq!(m.borrow().closed.len(), 2);
}
