//! Register/protocol host models only. No adapter accesses physical TPM/MMIO;
//! fabricated protocol test data never counts as a native rollback anchor.
use nieuw_os_tpm_crb::{
    codec::{self, NvIndex, Password, Request, Response, Startup},
    transport::{
        self as crb, Effect, Failure, Fault, Io, IoError, Phase, Poll, Register, Ticket, Transport,
        BASE, BUFFER_SIZE,
    },
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
enum Access {
    Read(Register),
    Write(Register, u32),
    ReadBuffer(usize, usize),
    WriteBuffer(usize, usize),
    Quarantine,
}
struct State {
    regs: [u32; 28],
    buffer: [u8; BUFFER_SIZE],
    trace: Vec<Access>,
    locality_ack: bool,
    ready_ack: bool,
    idle_ack: bool,
    release_ack: bool,
    auto_response: Option<Vec<u8>>,
    empty_completion: bool,
    cancel_ack: bool,
    cancel_clear_ack: bool,
    start_write_error: bool,
    header_change: bool,
}
impl State {
    fn new() -> Self {
        let mut s = Self {
            regs: [0; 28],
            buffer: [0xa5; BUFFER_SIZE],
            trace: vec![],
            locality_ack: true,
            ready_ack: true,
            idle_ack: true,
            release_ack: true,
            auto_response: None,
            empty_completion: false,
            cancel_ack: false,
            cancel_clear_ack: true,
            start_write_error: false,
            header_change: false,
        };
        s.set(Register::InterfaceId, 0x25811);
        s.set(Register::LocalityState, 0x80);
        s.set(Register::ControlStatus, 2);
        s.set(Register::CommandAddressLow, (BASE + 0x80) as u32);
        s.set(Register::ResponseAddressLow, (BASE + 0x80) as u32);
        s.set(Register::CommandSize, BUFFER_SIZE as u32);
        s.set(Register::ResponseSize, BUFFER_SIZE as u32);
        s
    }
    fn set(&mut self, r: Register, v: u32) {
        self.regs[r as usize / 4] = v;
    }
    fn get(&self, r: Register) -> u32 {
        self.regs[r as usize / 4]
    }
    fn writes(&self, r: Register, value: u32) -> usize {
        self.trace
            .iter()
            .filter(|a| matches!(a,Access::Write(reg,v)if*reg==r&&*v==value))
            .count()
    }
    fn buffer_writes(&self) -> usize {
        self.trace
            .iter()
            .filter(|a| matches!(a, Access::WriteBuffer(..)))
            .count()
    }
    fn complete(&mut self, response: &[u8]) {
        self.buffer[..response.len()].copy_from_slice(response);
        self.set(Register::Start, 0);
    }
}
struct Model(Rc<RefCell<State>>);
impl Io for Model {
    fn read32(&mut self, r: Register) -> Result<u32, IoError> {
        let mut s = self.0.borrow_mut();
        s.trace.push(Access::Read(r));
        Ok(s.get(r))
    }
    fn write32(&mut self, r: Register, v: u32) -> Result<(), IoError> {
        assert!(
            matches!(
                (r, v),
                (Register::LocalityControl, 1 | 2)
                    | (Register::ControlRequest, 1 | 2)
                    | (Register::Cancel, 0 | 1)
                    | (Register::Start, 1)
            ),
            "forbidden register write"
        );
        let mut s = self.0.borrow_mut();
        s.trace.push(Access::Write(r, v));
        match (r, v) {
            (Register::LocalityControl, 1) if s.locality_ack => {
                s.set(Register::LocalityState, 0x82);
                s.set(Register::LocalityStatus, 1);
            }
            (Register::LocalityControl, 2) if s.release_ack => {
                s.set(Register::LocalityState, 0x80);
                s.set(Register::LocalityStatus, 0);
            }
            (Register::ControlRequest, 1) => {
                if s.ready_ack {
                    s.set(Register::ControlStatus, 0);
                    s.set(r, 0);
                } else {
                    s.set(r, 1);
                }
            }
            (Register::ControlRequest, 2) => {
                if s.idle_ack {
                    s.set(Register::ControlStatus, 2);
                    s.set(r, 0);
                } else {
                    s.set(r, 2);
                }
            }
            (Register::Start, 1) => {
                s.set(r, 1);
                if let Some(bytes) = s.auto_response.take() {
                    s.complete(&bytes);
                } else if s.empty_completion {
                    s.set(r, 0);
                }
                if s.start_write_error {
                    return Err(IoError);
                }
            }
            (Register::Cancel, 1) => {
                s.set(r, 1);
                if s.cancel_ack {
                    s.set(Register::Start, 0);
                }
            }
            (Register::Cancel, 0) if !s.cancel_clear_ack => {}
            _ => s.set(r, v),
        }
        Ok(())
    }
    fn read_buffer(&mut self, o: usize, out: &mut [u8]) -> Result<(), IoError> {
        assert!(out.len() <= 128 && o + out.len() <= BUFFER_SIZE);
        let mut s = self.0.borrow_mut();
        s.trace.push(Access::ReadBuffer(o, out.len()));
        out.copy_from_slice(&s.buffer[o..o + out.len()]);
        if s.header_change {
            s.header_change = false;
            s.buffer[0] ^= 1;
        }
        Ok(())
    }
    fn write_buffer(&mut self, o: usize, b: &[u8]) -> Result<(), IoError> {
        assert!(b.len() <= 128 && o + b.len() <= BUFFER_SIZE);
        let mut s = self.0.borrow_mut();
        s.trace.push(Access::WriteBuffer(o, b.len()));
        s.buffer[o..o + b.len()].copy_from_slice(b);
        Ok(())
    }
    fn quarantine(&mut self) {
        self.0.borrow_mut().trace.push(Access::Quarantine);
    }
}
fn model() -> (Transport<Model>, Rc<RefCell<State>>) {
    let state = Rc::new(RefCell::new(State::new()));
    (Transport::new(Model(state.clone())), state)
}
fn ready(t: &mut Transport<Model>) -> Ticket {
    let ticket = t.acquire(0, 100).unwrap();
    assert!(!t.poll_ready(ticket, 1).unwrap());
    assert!(t.poll_ready(ticket, 2).unwrap());
    ticket
}
fn header(tag: u16, len: u32, rc: u32) -> Vec<u8> {
    [
        tag.to_be_bytes().as_slice(),
        len.to_be_bytes().as_slice(),
        rc.to_be_bytes().as_slice(),
    ]
    .concat()
}
fn empty() -> Vec<u8> {
    header(codec::NO_SESSIONS, 10, 0)
}
fn session(body: &[u8]) -> Vec<u8> {
    let mut b = header(codec::SESSIONS, (19 + body.len()) as u32, 0);
    b.extend((body.len() as u32).to_be_bytes());
    b.extend(body);
    b.extend([0, 0, 1, 0, 0]);
    b
}
fn startup<'a>(out: &'a mut [u8]) -> codec::Command<'a> {
    codec::encode(Request::Startup(Startup::Clear), out).unwrap()
}
#[test]
fn explicit_commands_are_big_endian_and_errors_preserve_output() {
    let mut b = [0xa5; 128];
    let c = startup(&mut b);
    assert_eq!(c.bytes(), [0x80, 1, 0, 0, 0, 12, 0, 0, 1, 0x44, 0, 0]);
    assert!(!c.is_nv_mutation());
    assert!(b[12..].iter().all(|x| *x == 0xa5));
    let before = b;
    assert!(codec::encode(
        Request::Properties {
            first: 0x100,
            count: 0
        },
        &mut b
    )
    .is_err());
    assert_eq!(b, before);
    assert!(codec::encode(Request::Startup(Startup::State), &mut b[..11]).is_err());
    assert_eq!(b, before);
    assert!(Password::new(&[1; 33]).is_err());
    assert!(NvIndex::new(0x2000000).is_err());
}
#[test]
fn nv_encoders_require_explicit_authorization_and_counter_type() {
    let index = NvIndex::new(0x1000042).unwrap();
    let auth = Password::new(&[0x7d; 32]).unwrap();
    let owner = Password::new(&[]).unwrap();
    let mut b = [0; 128];
    let c = codec::encode(
        Request::NvDefineCounter {
            index,
            owner_auth: owner,
            index_auth: auth,
        },
        &mut b,
    )
    .unwrap();
    assert!(c.is_nv_mutation());
    assert_eq!(c.bytes().len(), 77);
    assert_eq!(&c.bytes()[6..10], &0x12au32.to_be_bytes());
    assert_eq!(&c.bytes()[14..18], &9u32.to_be_bytes());
    assert_eq!(&c.bytes()[18..22], &0x40000009u32.to_be_bytes());
    let end = c.bytes().len();
    assert_eq!(
        &c.bytes()[end - 8..end - 4],
        &codec::COUNTER_ATTRIBUTES.to_be_bytes()
    );
    assert_eq!(&c.bytes()[end - 2..], &8u16.to_be_bytes());
}
#[test]
fn exact_completion_and_release_ack_are_required_before_detach() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().auto_response = Some(empty());
    let mut out = [0; 128];
    let ticket = t.execute(&startup(&mut out), 3, 20, 5).unwrap();
    assert_eq!(t.poll(ticket, 4), Ok(Poll::Complete(Response::Empty)));
    assert!(s.borrow().buffer.iter().all(|x| *x == 0));
    assert!(t.detach().is_err());
    let release = t.release(5, 20).unwrap();
    assert!(!t.poll_released(release, 6).unwrap());
    assert!(t.poll_released(release, 7).unwrap());
    assert!(t.detach().is_ok());
    assert_eq!(t.phase(), Phase::Detached);
    assert_eq!(s.borrow().writes(Register::Start, 1), 1);
    assert!(!s
        .borrow()
        .trace
        .iter()
        .any(|a| matches!(a, Access::Write(Register::Start, 0))));
}
#[test]
fn instant_completion_does_not_require_observing_busy() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().auto_response = Some(empty());
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    assert_eq!(t.poll(id, 3), Ok(Poll::Complete(Response::Empty)));
}
#[test]
fn stale_success_is_poisoned_before_start_and_cannot_complete() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().complete(&empty());
    s.borrow_mut().empty_completion = true;
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    let error = t.poll(id, 4).unwrap_err();
    assert_eq!(
        error,
        Failure {
            fault: Fault::Protocol,
            effect: Effect::Unknown
        }
    );
    assert!(t.is_quarantined());
}
#[test]
fn unsupported_fifo_and_external_buffers_are_never_accessed() {
    for bad in [0, 1, 2] {
        let (mut t, s) = model();
        match bad {
            0 => s.borrow_mut().set(Register::InterfaceId, 0x25810),
            1 => s.borrow_mut().set(Register::CommandAddressLow, 0x2000000),
            _ => s.borrow_mut().set(Register::ResponseSize, 4096),
        }
        assert!(t.acquire(0, 100).is_err());
        assert!(t.is_quarantined());
        assert!(!s.borrow().trace.iter().any(|a| matches!(
            a,
            Access::ReadBuffer(..)
                | Access::WriteBuffer(..)
                | Access::Write(Register::LocalityControl, _)
        )));
    }
}
#[test]
fn lost_locality_and_unacknowledged_reset_never_free_or_complete() {
    let (mut t, s) = model();
    ready(&mut t);
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    s.borrow_mut().set(Register::Start, 0);
    s.borrow_mut().set(Register::ControlStatus, 2);
    let writes = s.borrow().buffer_writes();
    assert_eq!(t.poll(id, 4).unwrap_err().effect, Effect::Unknown);
    assert!(t.is_quarantined());
    assert!(t.detach().is_err());
    assert_eq!(s.borrow().buffer_writes(), writes);
    assert!(!s
        .borrow()
        .trace
        .iter()
        .any(|a| matches!(a, Access::ReadBuffer(..))));
}
#[test]
fn timeout_cancel_never_replays_or_wipes_while_tpm_owns_buffer() {
    let (mut t, s) = model();
    ready(&mut t);
    let mut b = [0; 128];
    let index = NvIndex::new(0x1000042).unwrap();
    let auth = Password::new(&[0x55; 32]).unwrap();
    let c = codec::encode(Request::NvIncrement { index, auth }, &mut b).unwrap();
    let id = t.execute(&c, 3, 10, 5).unwrap();
    let writes = s.borrow().buffer_writes();
    assert_eq!(t.poll(id, 10), Ok(Poll::Pending));
    assert_eq!(t.phase(), Phase::Cancelling);
    assert_eq!(s.borrow().writes(Register::Cancel, 1), 1);
    assert_eq!(s.borrow().buffer_writes(), writes);
    assert_eq!(
        t.poll(id, 15),
        Err(Failure {
            fault: Fault::CancelTimeout,
            effect: Effect::Unknown
        })
    );
    assert!(t.is_quarantined());
    assert_eq!(s.borrow().writes(Register::Start, 1), 1);
    assert_eq!(s.borrow().buffer_writes(), writes);
}
#[test]
fn acknowledged_cancel_still_returns_unknown_effect_without_output() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().cancel_ack = true;
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 10, 5).unwrap();
    t.cancel(id, 4).unwrap();
    t.cancel(id, 5).unwrap();
    assert_eq!(s.borrow().writes(Register::Cancel, 1), 1);
    assert_eq!(
        t.poll(id, 6),
        Err(Failure {
            fault: Fault::Cancelled,
            effect: Effect::Unknown
        })
    );
    assert_eq!(t.phase(), Phase::Ready);
    assert!(s.borrow().buffer.iter().all(|x| *x == 0));
    assert_eq!(s.borrow().writes(Register::Start, 1), 1);
}
#[test]
fn failed_cancel_clear_is_not_permission_to_reuse_buffers() {
    let (mut t, s) = model();
    ready(&mut t);
    {
        let mut m = s.borrow_mut();
        m.cancel_ack = true;
        m.cancel_clear_ack = false;
    }
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 10, 5).unwrap();
    t.cancel(id, 4).unwrap();
    let writes = s.borrow().buffer_writes();
    assert_eq!(
        t.poll(id, 5),
        Err(Failure {
            fault: Fault::Protocol,
            effect: Effect::Unknown
        })
    );
    assert!(t.is_quarantined());
    assert_eq!(s.borrow().buffer_writes(), writes);
}
#[test]
fn partial_start_failure_quarantines_and_drop_notifies_only_once() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().start_write_error = true;
    let mut b = [0; 128];
    assert_eq!(
        t.execute(&startup(&mut b), 3, 20, 5),
        Err(Failure {
            fault: Fault::Io,
            effect: Effect::Unknown
        })
    );
    assert!(t.is_quarantined());
    drop(t);
    assert_eq!(
        s.borrow()
            .trace
            .iter()
            .filter(|a| matches!(a, Access::Quarantine))
            .count(),
        1
    );
    assert_eq!(s.borrow().writes(Register::Start, 1), 1);
}
#[test]
fn dropping_an_executing_owner_quarantines_without_hidden_reset() {
    let (mut t, s) = model();
    ready(&mut t);
    let mut b = [0; 128];
    t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    let trace = s.borrow().trace.len();
    drop(t);
    assert!(matches!(s.borrow().trace[trace], Access::Quarantine));
    assert_eq!(s.borrow().trace.len(), trace + 1);
}
#[test]
fn stale_ticket_performs_no_io_and_does_not_consume_active_command() {
    let (mut t, s) = model();
    let old = ready(&mut t);
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    let n = s.borrow().trace.len();
    assert_eq!(
        t.poll(old, 4),
        Err(Failure {
            fault: Fault::Ticket,
            effect: Effect::NotStarted
        })
    );
    assert_eq!(s.borrow().trace.len(), n);
    s.borrow_mut().complete(&empty());
    assert_eq!(t.poll(id, 5), Ok(Poll::Complete(Response::Empty)));
}
#[test]
fn response_lengths_are_bounded_before_second_read() {
    for size in [0, 9, 513, u32::MAX] {
        let (mut t, s) = model();
        ready(&mut t);
        s.borrow_mut().auto_response = Some(header(codec::NO_SESSIONS, size, 0));
        let mut b = [0; 128];
        let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
        assert_eq!(t.poll(id, 4).unwrap_err().fault, Fault::Protocol);
        let reads: Vec<_> = s
            .borrow()
            .trace
            .iter()
            .filter_map(|a| match a {
                Access::ReadBuffer(o, n) => Some((*o, *n)),
                _ => None,
            })
            .collect();
        assert_eq!(reads, [(0, 10)]);
    }
}
#[test]
fn changed_header_or_device_address_is_never_accepted() {
    for mode in [0, 1] {
        let (mut t, s) = model();
        ready(&mut t);
        s.borrow_mut().auto_response = Some(empty());
        let mut b = [0; 128];
        let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
        if mode == 0 {
            s.borrow_mut().header_change = true;
        } else {
            s.borrow_mut().set(Register::ResponseAddressHigh, 1);
        }
        assert!(t.poll(id, 4).is_err());
        assert!(t.is_quarantined());
    }
}
#[test]
fn password_response_requires_exact_session_auth_and_counter_size() {
    for mode in [0, 1, 2, 3] {
        let (mut t, s) = model();
        ready(&mut t);
        let mut body = 8u16.to_be_bytes().to_vec();
        body.extend(42u64.to_be_bytes());
        let mut response = session(&body);
        match mode {
            0 => {}
            1 => {
                let len = response.len();
                response[len - 3] = 0;
            }
            2 => response[14] = 0xff,
            _ => {
                response.push(0);
                let len = response.len() as u32;
                response[2..6].copy_from_slice(&len.to_be_bytes());
            }
        }
        s.borrow_mut().auto_response = Some(response);
        let mut b = [0; 128];
        let c = codec::encode(
            Request::NvReadCounter {
                index: NvIndex::new(0x1000042).unwrap(),
                auth: Password::new(&[0x44; 32]).unwrap(),
            },
            &mut b,
        )
        .unwrap();
        let id = t.execute(&c, 3, 20, 5).unwrap();
        let result = t.poll(id, 4);
        if mode == 0 {
            assert_eq!(result, Ok(Poll::Complete(Response::Counter(42))));
        } else {
            assert!(result.is_err());
        }
    }
}
#[test]
fn tpm_error_is_exposed_without_success_normalization_or_retry() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().auto_response = Some(header(codec::NO_SESSIONS, 10, 0x100));
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    assert_eq!(
        t.poll(id, 4),
        Err(Failure {
            fault: Fault::Tpm(0x100),
            effect: Effect::Completed
        })
    );
    assert_eq!(s.borrow().writes(Register::Start, 1), 1);
    assert_eq!(t.phase(), Phase::Ready);
}
#[test]
fn locality_and_idle_deadlines_never_fake_ack() {
    let (mut t, s) = model();
    s.borrow_mut().locality_ack = false;
    let id = t.acquire(0, 5).unwrap();
    assert!(!t.poll_ready(id, 1).unwrap());
    assert_eq!(t.poll_ready(id, 5).unwrap_err().fault, Fault::Timeout);
    assert_eq!(s.borrow().buffer_writes(), 0);
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().idle_ack = false;
    let id = t.release(3, 8).unwrap();
    assert!(!t.poll_released(id, 4).unwrap());
    assert_eq!(t.poll_released(id, 8).unwrap_err().fault, Fault::Timeout);
    assert_eq!(s.borrow().writes(Register::LocalityControl, 2), 0);
    assert!(t.detach().is_err());
}
#[test]
fn clock_rollback_and_invalid_deadlines_do_not_enable_work() {
    let (mut t, s) = model();
    let n = s.borrow().trace.len();
    assert!(t.acquire(0, 0).is_err());
    assert!(t.acquire(0, crb::MAX_TICKS + 1).is_err());
    assert_eq!(s.borrow().trace.len(), n);
    ready(&mut t);
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    assert_eq!(
        t.poll(id, 2),
        Err(Failure {
            fault: Fault::Clock,
            effect: Effect::Unknown
        })
    );
    assert!(t.is_quarantined());
}
#[test]
fn all_model_buffer_accesses_stay_in_fixed_device_aperture() {
    let (mut t, s) = model();
    ready(&mut t);
    s.borrow_mut().auto_response = Some(empty());
    let mut b = [0; 128];
    let id = t.execute(&startup(&mut b), 3, 20, 5).unwrap();
    t.poll(id, 4).unwrap();
    for a in &s.borrow().trace {
        match a {
            Access::WriteBuffer(o, n) | Access::ReadBuffer(o, n) => {
                assert!(*n <= 128 && o + n <= BUFFER_SIZE)
            }
            Access::Read(r) => assert!((*r as usize) < 0x80),
            _ => {}
        }
    }
}
#[test]
fn properties_validate_capability_count_order_and_reserved_more_bit() {
    for mode in 0..5 {
        let (mut t, s) = model();
        ready(&mut t);
        let mut body = vec![0];
        body.extend(6u32.to_be_bytes());
        body.extend(2u32.to_be_bytes());
        for (p, v) in [(0x100u32, 0x322e3000u32), (0x11e, 4096)] {
            body.extend(p.to_be_bytes());
            body.extend(v.to_be_bytes());
        }
        match mode {
            0 => {}
            1 => body[0] = 2,
            2 => body[1..5].copy_from_slice(&5u32.to_be_bytes()),
            3 => body[5..9].copy_from_slice(&17u32.to_be_bytes()),
            _ => body[17..21].copy_from_slice(&0x100u32.to_be_bytes()),
        }
        let mut response = header(codec::NO_SESSIONS, (10 + body.len()) as u32, 0);
        response.extend(body);
        s.borrow_mut().auto_response = Some(response);
        let mut b = [0; 128];
        let command = codec::encode(
            Request::Properties {
                first: 0x100,
                count: 2,
            },
            &mut b,
        )
        .unwrap();
        let id = t.execute(&command, 3, 20, 5).unwrap();
        let result = t.poll(id, 4);
        if mode == 0 {
            let Poll::Complete(Response::Properties {
                count,
                values,
                more,
            }) = result.unwrap()
            else {
                panic!()
            };
            assert_eq!(count, 2);
            assert!(!more);
            assert_eq!(values[0].value, 0x322e3000);
            assert_eq!(values[1].property, 0x11e);
        } else {
            assert_eq!(result.unwrap_err().fault, Fault::Protocol);
        }
    }
}
fn public_body(index: u32) -> Vec<u8> {
    let mut b = 14u16.to_be_bytes().to_vec();
    b.extend(index.to_be_bytes());
    b.extend(0xbu16.to_be_bytes());
    b.extend((codec::COUNTER_ATTRIBUTES | 0x20000000).to_be_bytes());
    b.extend(0u16.to_be_bytes());
    b.extend(8u16.to_be_bytes());
    b.extend(34u16.to_be_bytes());
    b.extend(0xbu16.to_be_bytes());
    b.extend([0xa5; 32]);
    b
}
#[test]
fn nv_public_binds_index_counter_attributes_sizes_and_name_algorithm() {
    for mode in 0..6 {
        let (mut t, s) = model();
        ready(&mut t);
        let index = NvIndex::new(0x1000042).unwrap();
        let mut body = public_body(index.raw());
        match mode {
            0 => {}
            1 => body[2..6].copy_from_slice(&0x1000043u32.to_be_bytes()),
            2 => body[8..12].copy_from_slice(&0x40004u32.to_be_bytes()),
            3 => body[12..14].copy_from_slice(&1u16.to_be_bytes()),
            4 => body[14..16].copy_from_slice(&7u16.to_be_bytes()),
            _ => body[18..20].copy_from_slice(&4u16.to_be_bytes()),
        }
        let mut response = header(codec::NO_SESSIONS, (10 + body.len()) as u32, 0);
        response.extend(body);
        s.borrow_mut().auto_response = Some(response);
        let mut b = [0; 128];
        let command = codec::encode(Request::NvReadPublic { index }, &mut b).unwrap();
        let id = t.execute(&command, 3, 20, 5).unwrap();
        let result = t.poll(id, 4);
        if mode == 0 {
            let Poll::Complete(Response::NvPublic {
                index: actual,
                name,
                ..
            }) = result.unwrap()
            else {
                panic!()
            };
            assert_eq!(actual, index);
            assert_eq!(&name[2..], &[0xa5; 32]); /* Name bytes are opaque; this codec deliberately performs no digest authentication. */
        } else {
            assert_eq!(result.unwrap_err().fault, Fault::Protocol);
        }
    }
}
#[test]
fn nv_increment_success_requires_empty_session_parameters() {
    for nonempty in [false, true] {
        let (mut t, s) = model();
        ready(&mut t);
        s.borrow_mut().auto_response = Some(session(if nonempty { &[0] } else { &[] }));
        let mut b = [0; 128];
        let command = codec::encode(
            Request::NvIncrement {
                index: NvIndex::new(0x1000042).unwrap(),
                auth: Password::new(&[0x55; 32]).unwrap(),
            },
            &mut b,
        )
        .unwrap();
        let id = t.execute(&command, 3, 20, 5).unwrap();
        let result = t.poll(id, 4);
        if nonempty {
            assert_eq!(result.unwrap_err().fault, Fault::Protocol);
        } else {
            assert_eq!(result, Ok(Poll::Complete(Response::Empty)));
        }
    }
}
