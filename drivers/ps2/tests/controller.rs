//! Scripted controller/device fixtures; not native hardware execution.
use nieuw_os_ps2::*;
use std::collections::VecDeque;

#[derive(Default)]
struct Io {
    mode: u8,
    queue: VecDeque<(u8, u8)>,
    writes: Vec<(bool, u8)>,
    operations: usize,
    write_mode: bool,
    mouse: bool,
    stuck: bool,
    silence: bool,
    bad_mode: bool,
    resend: u8,
    resend_mouse: bool,
    wrong_reply: bool,
    wrong_bat: bool,
    mouse_id: u8,
    opposite: bool,
    bad_health: Option<(u8, u8)>,
    missing_health: Option<u8>,
    parity_health: Option<u8>,
    reset_mode_on_test: bool,
    parity_device: bool,
}
impl Io {
    fn byte(&mut self, mouse: bool, byte: u8) {
        self.queue.push_back((if mouse { 0x21 } else { 1 }, byte));
    }
}
impl Ports for Io {
    fn status(&mut self) -> u8 {
        self.operations += 1;
        if self.stuck {
            2
        } else {
            self.queue.front().map_or(0, |e| e.0)
        }
    }
    fn read_data(&mut self) -> u8 {
        self.operations += 1;
        self.queue.pop_front().expect("FIFO read without OBF").1
    }
    fn command(&mut self, v: u8) {
        self.operations += 1;
        self.writes.push((true, v));
        match v {
            0xad => self.mode |= 0x10,
            0xa7 => self.mode |= 0x20,
            0xae => self.mode &= !0x10,
            0xa8 => self.mode &= !0x20,
            0xa9..=0xab => {
                if v == 0xaa && self.reset_mode_on_test {
                    self.mode = 3;
                }
                if self.missing_health != Some(v) {
                    let reply = self
                        .bad_health
                        .filter(|(command, _)| *command == v)
                        .map_or(if v == 0xaa { 0x55 } else { 0 }, |(_, reply)| reply);
                    self.byte(false, reply);
                    if self.parity_health == Some(v) {
                        self.queue.back_mut().unwrap().0 |= 0x80;
                    }
                }
            }
            0x20 => self.byte(
                false,
                if self.bad_mode {
                    self.mode ^ 1
                } else {
                    self.mode
                },
            ),
            0x60 => self.write_mode = true,
            0xd4 => self.mouse = true,
            _ => panic!("unexpected controller command"),
        }
    }
    fn data(&mut self, v: u8) {
        self.operations += 1;
        self.writes.push((false, v));
        if self.write_mode {
            self.write_mode = false;
            self.mode = v;
            return;
        }
        let mouse = core::mem::take(&mut self.mouse);
        if self.silence {
            return;
        }
        if self.resend > 0 && mouse == self.resend_mouse {
            self.resend -= 1;
            self.byte(mouse, 0xfe);
            return;
        }
        if self.opposite && mouse {
            self.opposite = false;
            self.byte(false, 0x1e);
        }
        self.byte(mouse, if self.wrong_reply { 0xfc } else { 0xfa });
        if self.parity_device {
            self.queue.back_mut().unwrap().0 |= 0xc0;
        }
        if v == 0xff {
            self.byte(mouse, if self.wrong_bat { 0xfc } else { 0xaa });
            if mouse {
                self.byte(true, self.mouse_id);
            }
        }
    }
}
fn finish(c: &mut Controller, io: &mut Io, ticket: u64, start: u64) -> Result<ResetReport, Fault> {
    for now in start..start + 1100 {
        let before = io.operations;
        let result = c.poll_reset(ticket, now, io);
        assert!(io.operations - before <= PER_CALL_ACTIONS * 2);
        match result? {
            Progress::Pending => {}
            Progress::Ready(r) => return Ok(r),
        }
    }
    panic!("unbounded reset")
}

#[test]
fn controller_and_both_port_tests_gate_all_device_commands() {
    for command in [0xaa, 0xab, 0xa9] {
        let expected = if command == 0xaa { 0x55 } else { 0 };
        for reply in 0..=u8::MAX {
            if reply == expected {
                continue;
            }
            let mut c = Controller::new();
            let mut io = Io {
                bad_health: Some((command, reply)),
                ..Io::default()
            };
            let ticket = c.begin_reset(0).unwrap();
            assert_eq!(finish(&mut c, &mut io, ticket, 0), Err(Fault::Controller));
            assert!(io.writes.contains(&(true, command)));
            assert!(io.writes.iter().all(|(is_command, _)| *is_command));
            assert_eq!(c.read_sample(&mut io), Err(Fault::NeedsReset));
            let operations = io.operations;
            assert_eq!(c.poll_reset(ticket, 1001, &mut io), Err(Fault::Controller));
            assert_eq!(io.operations, operations);
        }
    }
}

#[test]
fn missing_or_parity_faulted_health_replies_fail_then_require_fresh_reset() {
    for command in [0xaa, 0xab, 0xa9] {
        for parity in [false, true] {
            let mut c = Controller::new();
            let mut io = Io {
                missing_health: (!parity).then_some(command),
                parity_health: parity.then_some(command),
                ..Io::default()
            };
            let ticket = c.begin_reset(0).unwrap();
            assert_eq!(
                finish(&mut c, &mut io, ticket, 0),
                Err(if parity {
                    Fault::Controller
                } else {
                    Fault::Timeout
                })
            );
            assert!(io.writes.iter().all(|(is_command, _)| *is_command));
            io.missing_health = None;
            io.parity_health = None;
            let next = c.begin_reset(1001).unwrap();
            assert_ne!(ticket, next);
            let before = io.operations;
            assert_eq!(c.poll_reset(ticket, 1001, &mut io), Err(Fault::WrongTicket));
            assert_eq!(io.operations, before);
            let report = finish(&mut c, &mut io, next, 1001).unwrap();
            assert_eq!(report.epoch, 1);
        }
    }
}

#[test]
fn self_test_state_reset_preserves_saved_mode_and_reasserts_port_disables() {
    let mut c = Controller::new();
    let mut io = Io {
        mode: 0x0f,
        reset_mode_on_test: true,
        ..Io::default()
    };
    let ticket = c.begin_reset(0).unwrap();
    finish(&mut c, &mut io, ticket, 0).unwrap();
    assert_eq!(io.mode, 0x4c);
    let test = io
        .writes
        .iter()
        .position(|write| *write == (true, 0xaa))
        .unwrap();
    assert_eq!(
        &io.writes[test..test + 5],
        &[
            (true, 0xaa),
            (true, 0xad),
            (true, 0xa7),
            (true, 0xab),
            (true, 0xa9)
        ]
    );
    io.byte(false, 0x1e);
    let sample = c.read_sample(&mut io).unwrap().unwrap();
    assert_eq!((sample.epoch, sample.sequence, sample.byte), (1, 1, 0x1e));
}

#[test]
fn real_command_shape_requires_both_bat_id_scanmode_and_irq_disable_before_ready() {
    let mut c = Controller::new();
    let mut io = Io::default();
    assert_eq!(c.read_sample(&mut io), Err(Fault::NeedsReset));
    let ticket = c.begin_reset(0).unwrap();
    assert_eq!(io.operations, 0);
    assert_eq!(c.begin_reset(0), Err(Fault::Busy));
    assert_eq!(
        c.poll_reset(ticket + 1, 0, &mut io),
        Err(Fault::WrongTicket)
    );
    assert_eq!(io.operations, 0);
    let report = finish(&mut c, &mut io, ticket, 0).unwrap();
    assert_eq!(
        (report.ticket, report.epoch, report.discarded),
        (ticket, 1, 0)
    );
    assert_eq!(io.mode & 0x73, 0x40);
    assert_eq!(
        io.writes,
        vec![
            (true, 0xad),
            (true, 0xa7),
            (true, 0x20),
            (true, 0xaa),
            (true, 0xad),
            (true, 0xa7),
            (true, 0xab),
            (true, 0xa9),
            (true, 0x60),
            (false, 0),
            (true, 0xae),
            (true, 0xa8),
            (false, 0xff),
            (false, 0xf5),
            (false, 0xf0),
            (false, 2),
            (true, 0xd4),
            (false, 0xff),
            (true, 0xd4),
            (false, 0xf6),
            (true, 0x60),
            (false, 0x40),
            (true, 0x20),
            (true, 0xd4),
            (false, 0xf4),
            (false, 0xf4)
        ]
    );
    let before = io.operations;
    assert_eq!(
        c.poll_reset(ticket, 9999, &mut io),
        Ok(Progress::Ready(report))
    );
    assert_eq!(io.operations, before);
}

#[test]
fn raw_input_preserves_aux_errors_and_bytes_with_no_keyboard_sentinel_rewrite() {
    let mut c = Controller::new();
    let mut io = Io::default();
    let t = c.begin_reset(0).unwrap();
    finish(&mut c, &mut io, t, 0).unwrap();
    let before = io.operations;
    assert_eq!(c.read_sample(&mut io), Ok(None));
    assert_eq!(io.operations - before, 1);
    for (seq, mouse, byte) in [(1, false, 0xe0), (2, true, 0xff), (3, true, 0x80)] {
        io.byte(mouse, byte);
        assert_eq!(
            c.read_sample(&mut io),
            Ok(Some(Sample {
                epoch: 1,
                sequence: seq,
                status: if mouse { 0x21 } else { 1 },
                byte
            }))
        );
    }
    io.queue.push_back((0xc1, 0x37));
    assert_eq!(
        c.read_sample(&mut io),
        Ok(Some(Sample {
            epoch: 1,
            sequence: 4,
            status: 0xc1,
            byte: 0x37
        }))
    );
    let before = io.operations;
    assert_eq!(c.read_sample(&mut io), Err(Fault::NeedsReset));
    assert_eq!(io.operations, before);
}

#[test]
fn resend_is_bounded_and_mouse_retry_reissues_d4_prefix() {
    for mouse in [false, true] {
        let mut c = Controller::new();
        let mut io = Io {
            resend: 2,
            resend_mouse: mouse,
            ..Io::default()
        };
        let t = c.begin_reset(0).unwrap();
        finish(&mut c, &mut io, t, 0).unwrap();
        assert_eq!(
            io.writes.iter().filter(|v| **v == (true, 0xd4)).count(),
            if mouse { 5 } else { 3 }
        );
        let mut c = Controller::new();
        let mut io = Io {
            resend: 3,
            resend_mouse: mouse,
            ..Io::default()
        };
        let t = c.begin_reset(0).unwrap();
        assert_eq!(finish(&mut c, &mut io, t, 0), Err(Fault::Response));
        assert_eq!(c.read_sample(&mut io), Err(Fault::NeedsReset));
    }
}

#[test]
fn missing_ack_wrong_bat_mouse_id_and_changed_controller_mode_fail_closed() {
    for fault in [0, 1, 2, 3, 4] {
        let mut io = Io {
            silence: fault == 0,
            wrong_reply: fault == 1,
            wrong_bat: fault == 2,
            mouse_id: if fault == 3 { 3 } else { 0 },
            bad_mode: fault == 4,
            ..Io::default()
        };
        let mut c = Controller::new();
        let t = c.begin_reset(0).unwrap();
        let error = finish(&mut c, &mut io, t, 0).unwrap_err();
        assert_eq!(
            error,
            match fault {
                0 => Fault::Timeout,
                1 | 2 => Fault::Response,
                3 => Fault::UnsupportedDevice,
                _ => Fault::Controller,
            }
        );
        let before = io.operations;
        assert_eq!(c.poll_reset(t, 2000, &mut io), Err(error));
        assert_eq!(io.operations, before);
    }
}

#[test]
fn parity_timeout_status_and_clock_rollback_do_not_publish_ready() {
    let mut c = Controller::new();
    let mut io = Io::default();
    let t = c.begin_reset(0).unwrap();
    assert_eq!(c.poll_reset(t, 5, &mut io), Ok(Progress::Pending));
    let before = io.operations;
    assert_eq!(c.poll_reset(t, 4, &mut io), Err(Fault::ClockRollback));
    assert_eq!(io.operations, before);
    let mut c = Controller::new();
    let mut io = Io {
        stuck: true,
        ..Io::default()
    };
    let t = c.begin_reset(0).unwrap();
    assert_eq!(c.poll_reset(t, 0, &mut io), Ok(Progress::Pending));
    assert_eq!(
        c.poll_reset(t, STEP_TICKS + 1, &mut io),
        Err(Fault::Timeout)
    );
    let mut c = Controller::new();
    let mut io = Io {
        parity_device: true,
        ..Io::default()
    };
    let t = c.begin_reset(0).unwrap();
    assert_eq!(finish(&mut c, &mut io, t, 0), Err(Fault::Controller));
}

#[test]
fn drain_overflow_and_fixed_io_budget_terminate_without_a_success_fallback() {
    let mut c = Controller::new();
    let mut io = Io::default();
    for _ in 0..MAX_DRAIN + 1 {
        io.byte(false, 0x1e);
    }
    let t = c.begin_reset(0).unwrap();
    assert_eq!(finish(&mut c, &mut io, t, 0), Err(Fault::DrainOverflow));
    assert_eq!(io.queue.len(), 1);
    let mut c = Controller::new();
    let mut io = Io {
        stuck: true,
        ..Io::default()
    };
    let t = c.begin_reset(0).unwrap();
    for _ in 0..MAX_OPERATIONS {
        assert_eq!(c.poll_reset(t, 0, &mut io), Ok(Progress::Pending));
    }
    assert_eq!(c.poll_reset(t, 0, &mut io), Err(Fault::Budget));
    assert_eq!(io.operations, MAX_OPERATIONS as usize);
}

#[test]
fn detach_and_new_reset_invalidate_old_ticket_epoch_and_all_pre_ready_data() {
    let mut c = Controller::new();
    let mut io = Io::default();
    let old = c.begin_reset(0).unwrap();
    finish(&mut c, &mut io, old, 0).unwrap();
    c.detach();
    let before = io.operations;
    assert_eq!(c.read_sample(&mut io), Err(Fault::NeedsReset));
    assert_eq!(io.operations, before);
    io.byte(false, 0x9e);
    io.opposite = true;
    let new = c.begin_reset(20).unwrap();
    assert_ne!(new, old);
    assert_eq!(c.poll_reset(old, 20, &mut io), Err(Fault::WrongTicket));
    let r = finish(&mut c, &mut io, new, 20).unwrap();
    assert_eq!(r.epoch, 2);
    assert_eq!(r.discarded, 2);
    io.byte(true, 7);
    assert_eq!(c.read_sample(&mut io).unwrap().unwrap().sequence, 1);
    let third = c.begin_reset(40).unwrap();
    c.detach();
    assert_eq!(c.poll_reset(third, 40, &mut io), Err(Fault::WrongTicket));
}
