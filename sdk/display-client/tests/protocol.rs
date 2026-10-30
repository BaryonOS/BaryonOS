//! Protocol fixtures only: actual capability/hardware behavior has separate
//! engine tests and the required native integration gate.
use nieuw_os_display_client::{
    wire::*, DisplayClient, DisplaySource, Error, InputClient, InputSource, ResetProgress,
    Transport,
};
use nieuw_os_sdk_runtime::{Error as RuntimeError, Handle};
use std::{cell::RefCell, rc::Rc};

fn geometry() -> Geometry {
    Geometry {
        width: 1280,
        height: 720,
        epoch: 7,
    }
}
fn rectangle() -> Rectangle {
    Rectangle {
        x: 1248,
        y: 712,
        width: 32,
        height: 8,
    }
}
#[test]
fn fixed_wire_layouts_have_exact_fields_and_padding() {
    let g = geometry();
    let b = g.encode().unwrap();
    assert_eq!(&b[0..8], &[0, 5, 0, 0, 208, 2, 0, 0]);
    assert_eq!(&b[8..16], &7u64.to_le_bytes());
    assert_eq!(&b[16..24], &[32, 0, 8, 0, 1, 0, 0, 0]);
    assert!(b[24..].iter().all(|x| *x == 0));
    assert_eq!(Geometry::decode(&b), Ok(g));
    let blit = Blit {
        epoch: 7,
        rectangle: rectangle(),
    };
    let b = blit.encode().unwrap();
    assert_eq!(&b[16..20], &[32, 0, 8, 0]);
    assert_eq!(&b[24..28], &1024u32.to_le_bytes());
    assert_eq!(Blit::decode(&b), Ok(blit));
    let sample = Sample {
        epoch: 8,
        sequence: 15,
        status: 0xe1,
        byte: 0xff,
    };
    let b = sample.encode().unwrap();
    assert_eq!(&b[16..20], &[0xe1, 0xff, 0, 0]);
    assert_eq!(Sample::decode(&b), Ok(sample));
    assert!(sample.needs_reset() && sample.is_mouse());
    let reset = Reset {
        ticket: 9,
        epoch: 8,
        discarded: 3,
        operations: 100,
    };
    let b = reset.encode().unwrap();
    assert_eq!(&b[24..32], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(Reset::decode(&b), Ok(reset));
}
#[test]
fn malformed_geometry_descriptor_color_reset_and_sample_are_never_canonical() {
    for at in [16, 18, 20, 24, 31] {
        let mut b = geometry().encode().unwrap();
        b[at] ^= 128;
        assert!(Geometry::decode(&b).is_err());
    }
    for at in [20, 23, 24, 25, 28, 31] {
        let mut b = Blit {
            epoch: 7,
            rectangle: rectangle(),
        }
        .encode()
        .unwrap();
        b[at] ^= 1;
        assert!(Blit::decode(&b).is_err());
    }
    for r in [
        Rectangle {
            x: u32::MAX,
            ..rectangle()
        },
        Rectangle {
            y: u32::MAX,
            ..rectangle()
        },
        Rectangle {
            width: 33,
            ..rectangle()
        },
        Rectangle {
            height: 9,
            ..rectangle()
        },
        Rectangle {
            width: 0,
            ..rectangle()
        },
        Rectangle {
            x: 1249,
            ..rectangle()
        },
    ] {
        assert!(Blit {
            epoch: 7,
            rectangle: r
        }
        .validate(geometry(), 1024)
        .is_err());
    }
    assert!(Blit {
        epoch: 6,
        rectangle: rectangle()
    }
    .validate(geometry(), 1024)
    .is_err());
    let mut pixels = [0; 1024];
    pixels[1023] = 1;
    assert!(validate_pixels(&pixels).is_err());
    assert!(validate_pixels(&pixels[..1023]).is_err());
    assert!(validate_pixels(&[]).is_err());
    let reset = Reset {
        ticket: 1,
        epoch: 1,
        discarded: 0,
        operations: 100,
    };
    for at in [24, 28, 31] {
        let mut b = reset.encode().unwrap();
        b[at] ^= 1;
        assert!(Reset::decode(&b).is_err());
    }
    for r in [
        Reset { ticket: 0, ..reset },
        Reset { epoch: 0, ..reset },
        Reset {
            operations: 0,
            ..reset
        },
        Reset {
            operations: 8193,
            ..reset
        },
        Reset {
            discarded: 101,
            ..reset
        },
    ] {
        assert!(r.encode().is_err());
    }
    let sample = Sample {
        epoch: 1,
        sequence: 1,
        status: 1,
        byte: 0x1e,
    };
    for at in [18, 20, 31] {
        let mut b = sample.encode().unwrap();
        b[at] = 1;
        assert!(Sample::decode(&b).is_err());
    }
    assert!(Sample {
        status: 0,
        ..sample
    }
    .encode()
    .is_err());
    assert!(Sample {
        sequence: 0,
        ..sample
    }
    .encode()
    .is_err());
}

struct Server {
    g: Geometry,
    blits: usize,
    begin: usize,
    polls: usize,
    reads: usize,
    pending: bool,
    denied: bool,
    bad_count: bool,
    bad_reset: bool,
    sample: Sample,
}
impl Server {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            g: geometry(),
            blits: 0,
            begin: 0,
            polls: 0,
            reads: 0,
            pending: false,
            denied: false,
            bad_count: false,
            bad_reset: false,
            sample: Sample {
                epoch: 1,
                sequence: 1,
                status: 1,
                byte: 0x1e,
            },
        }))
    }
}
struct Port(Rc<RefCell<Server>>);
impl Transport for Port {
    fn display_info(&mut self, _: DisplaySource, out: &mut [u8; 32]) -> Result<u64, RuntimeError> {
        *out = self.0.borrow().g.encode().unwrap();
        Ok(32)
    }
    fn display_blit(
        &mut self,
        _: DisplaySource,
        d: &[u8; 32],
        p: &[u8],
    ) -> Result<u64, RuntimeError> {
        let mut s = self.0.borrow_mut();
        s.blits += 1;
        assert!(Blit::decode(d).unwrap().validate(s.g, p.len()).is_ok());
        if s.denied {
            Err(RuntimeError::Kernel(-5))
        } else {
            Ok(if s.bad_count { 0 } else { p.len() as u64 / 4 })
        }
    }
    fn input_reset_begin(&mut self, _: InputSource) -> Result<u64, RuntimeError> {
        let mut s = self.0.borrow_mut();
        s.begin += 1;
        Ok(s.begin as u64)
    }
    fn input_reset_poll(
        &mut self,
        _: InputSource,
        t: u64,
        out: &mut [u8; 32],
    ) -> Result<u64, RuntimeError> {
        let mut s = self.0.borrow_mut();
        s.polls += 1;
        if s.pending {
            return Err(RuntimeError::Kernel(-8));
        }
        *out = Reset {
            ticket: if s.bad_reset { t + 1 } else { t },
            epoch: t,
            discarded: 0,
            operations: 100,
        }
        .encode()
        .unwrap();
        Ok(32)
    }
    fn input_read(&mut self, _: InputSource, out: &mut [u8; 32]) -> Result<u64, RuntimeError> {
        let mut s = self.0.borrow_mut();
        s.reads += 1;
        if s.denied {
            return Err(RuntimeError::Kernel(-5));
        }
        *out = s.sample.encode().unwrap();
        Ok(if s.bad_count { 31 } else { 32 })
    }
}
fn source() -> Handle {
    Handle::from_raw(0x1_00000001).unwrap()
}
fn ready(s: &Rc<RefCell<Server>>) -> InputClient<Port> {
    let mut c = InputClient::new(Port(s.clone()), InputSource::from_handle(source()));
    c.begin_reset().unwrap();
    assert!(matches!(c.poll_reset(), Ok(ResetProgress::Ready(_))));
    c
}

#[test]
fn client_display_needs_geometry_rejects_bad_input_and_never_acks_denial_or_short_blit() {
    let s = Server::new();
    let mut c = DisplayClient::new(Port(s.clone()), DisplaySource::from_handle(source()));
    assert_eq!(c.blit(rectangle(), &[0; 1024]), Err(Error::NotReady));
    c.geometry().unwrap();
    let mut bad = [0; 1024];
    bad[1023] = 1;
    assert_eq!(c.blit(rectangle(), &bad), Err(Error::InvalidInput));
    assert_eq!(s.borrow().blits, 0);
    s.borrow_mut().denied = true;
    assert_eq!(
        c.blit(rectangle(), &[0; 1024]),
        Err(Error::Runtime(RuntimeError::Kernel(-5)))
    );
    s.borrow_mut().denied = false;
    s.borrow_mut().bad_count = true;
    assert_eq!(c.blit(rectangle(), &[0; 1024]), Err(Error::Protocol));
    assert_eq!(c.blit(rectangle(), &[0; 1024]), Err(Error::Poisoned));
    assert_eq!(s.borrow().blits, 2);
    let s = Server::new();
    let mut c = DisplayClient::new(Port(s.clone()), DisplaySource::from_handle(source()));
    c.geometry().unwrap();
    c.blit(rectangle(), &[0; 1024]).unwrap();
    assert_eq!(s.borrow().blits, 1);
    s.borrow_mut().g.epoch += 1;
    assert_eq!(c.geometry(), Err(Error::Protocol));
}
#[test]
fn input_pending_never_becomes_ready_and_ticket_mismatch_never_acks() {
    let s = Server::new();
    s.borrow_mut().pending = true;
    let mut c = InputClient::new(Port(s.clone()), InputSource::from_handle(source()));
    assert_eq!(c.read(), Err(Error::NotReady));
    assert_eq!(c.begin_reset(), Ok(1));
    assert_eq!(c.begin_reset(), Err(Error::Busy));
    for _ in 0..10 {
        assert_eq!(c.poll_reset(), Ok(ResetProgress::Pending));
        assert!(!c.is_ready());
    }
    assert_eq!(s.borrow().reads, 0);
    assert_eq!(s.borrow().begin, 1);
    s.borrow_mut().pending = false;
    s.borrow_mut().bad_reset = true;
    assert_eq!(c.poll_reset(), Err(Error::Protocol));
    assert!(!c.is_ready());
    assert_eq!(c.read(), Err(Error::Poisoned));
}
#[test]
fn input_replay_gap_epoch_error_and_parity_force_explicit_resync() {
    for mode in 0..5 {
        let s = Server::new();
        let mut c = ready(&s);
        let first = c.read().unwrap().unwrap();
        assert_eq!(first.sequence, 1);
        match mode {
            0 => {}
            1 => s.borrow_mut().sample.sequence = 3,
            2 => s.borrow_mut().sample.epoch = 2,
            3 => {
                s.borrow_mut().sample.sequence = 2;
                s.borrow_mut().sample.status = 0xc1;
            }
            _ => {
                s.borrow_mut().sample.sequence = 2;
                s.borrow_mut().bad_count = true;
            }
        }
        if mode == 3 {
            assert!(c.read().unwrap().unwrap().needs_reset());
            assert_eq!(c.read(), Err(Error::NotReady));
        } else if mode == 4 {
            assert_eq!(c.read(), Err(Error::Protocol));
            assert_eq!(c.read(), Err(Error::Poisoned));
        } else {
            assert_eq!(c.read(), Err(Error::SequenceLost));
            assert_eq!(c.read(), Err(Error::NotReady));
        }
        assert!(!c.is_ready());
    }
}
