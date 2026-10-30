//! Host tests for `transport_model` (I1–I6, F7/F8/F11/F12/F18/F19).
//! Each `// I<n>` test is the red target of mutant M<n> in
//! `verification/model-mutants.py`.

use super::*;

const L0: Lease = Lease {
    epoch: 0,
    broker_pid: 7,
    cap_generation: 0,
};
const L1: Lease = Lease {
    epoch: 0,
    broker_pid: 7,
    cap_generation: 1,
};
const OK: u32 = 0x8000;
const IO_ERR: u32 = 0x8002;
const BAD_MSG: u32 = 0x8001;

fn running() -> Model {
    let mut m = Model::new(L0);
    m.init(L1).unwrap();
    m.configure().unwrap();
    m.prepare().unwrap();
    m.start().unwrap();
    m
}

fn ack(m: &mut Model, now: u64) {
    m.observe_status(now, 0).unwrap();
    m.observe_bme(now, false).unwrap();
}

fn next_lease(m: &Model) -> Lease {
    let l = m.lease();
    Lease {
        epoch: l.epoch,
        broker_pid: l.broker_pid,
        cap_generation: l.cap_generation + 1,
    }
}

#[test]
fn happy_path_submit_complete_reuse() {
    let mut m = running();
    for expect in 0..SLOTS {
        assert_eq!(m.submit(L1), Ok(expect));
    }
    assert_eq!(m.submit(L1), Err(Reject::Again));
    assert_eq!(m.complete(L1, 2, 8, OK), Ok(Completion::Ok));
    assert_eq!(m.slot(2), Some(Slot::Free));
    assert_eq!(m.submit(L1), Ok(2));
    assert_eq!(m.published(), SLOTS);
}

// I1
#[test]
fn i1_status_zero_alone_is_not_a_fence() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.fence(10, 100).unwrap();
    m.observe_status(11, 0).unwrap();
    assert_eq!(m.state(), State::Fencing);
    assert_eq!(m.slot(0), Some(Slot::Published));
    assert_eq!(m.lease().epoch, 0);
    m.observe_bme(12, false).unwrap();
    assert_eq!(m.state(), State::Offline);
    assert_eq!(m.slot(0), Some(Slot::Free));
    assert_eq!(m.lease().epoch, 1);
}

#[test]
fn i1_bme_off_alone_is_not_a_fence() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.fence(0, 100).unwrap();
    m.observe_bme(1, false).unwrap();
    assert_eq!(m.state(), State::Fencing);
    assert_eq!(m.published(), 1);
}

#[test]
fn i1_frozen_while_fencing_no_selective_release() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.fence(0, 100).unwrap();
    assert_eq!(m.complete(L1, 0, 8, OK), Err(Reject::Frozen));
    assert_eq!(m.submit(L1), Err(Reject::Frozen));
    assert_eq!(m.stop(), Err(Reject::Frozen));
    assert_eq!(m.slot(0), Some(Slot::Published));
}

// I2
#[test]
fn i2_stale_lease_completion_rejected_without_state_change() {
    let mut m = running();
    m.submit(L1).unwrap();
    for stale in [
        L0,
        Lease { epoch: 1, ..L1 },
        Lease {
            broker_pid: 8,
            ..L1
        },
    ] {
        assert_eq!(m.complete(stale, 0, 8, OK), Err(Reject::StaleLease));
        assert_eq!(m.slot(0), Some(Slot::Published));
        assert_eq!(m.state(), State::Running);
    }
    assert_eq!(m.submit(L0), Err(Reject::StaleLease));
}

#[test]
fn i2_completion_after_both_acks_quarantines() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.fence(0, 100).unwrap();
    ack(&mut m, 1);
    let l = m.lease();
    assert_eq!(m.complete(l, 0, 8, OK), Err(Reject::Quarantined));
    assert_eq!(m.state(), State::Quarantined);
}

#[test]
fn i2_used_index_moving_after_ack_quarantines() {
    let mut m = running();
    m.fence(0, 100).unwrap();
    ack(&mut m, 1);
    assert_eq!(m.observe_used_idx(0), Ok(()));
    assert_eq!(m.observe_used_idx(1), Err(Reject::Quarantined));
    assert_eq!(m.state(), State::Quarantined);
}

#[test]
fn i2_unknown_or_idle_completion_fences() {
    let mut m = running();
    assert_eq!(m.complete(L1, 0, 8, OK), Err(Reject::UnknownSlot));
    assert_eq!(m.state(), State::Fencing);
    let mut m = running();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, 4, 8, OK), Err(Reject::UnknownSlot));
    assert_eq!(m.state(), State::Fencing);
    let mut m = running();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, u32::MAX, 8, OK), Err(Reject::UnknownSlot));
}

#[test]
fn bad_len_or_status_fences_io_err_stops() {
    let mut m = running();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, 0, 7, OK), Ok(Completion::Fenced));
    assert_eq!(m.slot(0), Some(Slot::Published));
    let mut m = running();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, 0, 8, BAD_MSG), Ok(Completion::Fenced));
    assert_eq!(m.state(), State::Fencing);
    let mut m = running();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, 0, 8, IO_ERR), Ok(Completion::IoError));
    assert_eq!(m.state(), State::Stopped);
    assert_eq!(m.slot(0), Some(Slot::Free));
}

// I3
#[test]
fn i3_fence_timeout_quarantines_and_keeps_pool_pinned() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.fence(100, 50).unwrap();
    m.observe_status(120, 0).unwrap();
    assert_eq!(m.tick(149), Ok(()));
    assert_eq!(m.tick(150), Err(Reject::Quarantined));
    assert_eq!(m.state(), State::Quarantined);
    assert_eq!(m.slot(0), Some(Slot::Published));
    assert_eq!(m.observe_bme(151, false), Err(Reject::Quarantined));
    let next = next_lease(&m);
    assert_eq!(m.init(next), Err(Reject::Quarantined));
}

#[test]
fn i3_late_read_back_after_deadline_quarantines() {
    let mut m = running();
    m.fence(0, 10).unwrap();
    m.observe_status(5, 0).unwrap();
    assert_eq!(m.observe_bme(10, false), Err(Reject::Quarantined));
}

#[test]
fn i3_contradicting_read_back_quarantines() {
    let mut m = running();
    m.fence(0, 10).unwrap();
    m.observe_status(1, 0).unwrap();
    assert_eq!(m.observe_status(2, 1), Err(Reject::Quarantined));
    let mut m = running();
    m.fence(0, 10).unwrap();
    m.observe_bme(1, false).unwrap();
    assert_eq!(m.observe_bme(2, true), Err(Reject::Quarantined));
}

#[test]
fn i3_unarmed_internal_fence_quarantines_on_tick() {
    let mut m = running();
    m.complete(L1, 0, 8, OK).unwrap_err();
    assert_eq!(m.tick(0), Err(Reject::Quarantined));
}

#[test]
fn i3_fence_budget_and_deadline_bounds() {
    let mut m = running();
    assert_eq!(m.fence(0, 0), Err(Reject::BadBudget));
    assert_eq!(m.fence(0, MAX_FENCE_BUDGET + 1), Err(Reject::BadBudget));
    assert_eq!(m.state(), State::Running);
    m.fence(0, 10).unwrap();
    m.fence(5, MAX_FENCE_BUDGET).unwrap();
    assert_eq!(m.tick(10), Err(Reject::Quarantined));
    let mut m = running();
    assert_eq!(m.fence(u64::MAX, 1), Err(Reject::Quarantined));
}

#[test]
fn f18_epoch_overflow_quarantines() {
    let max = Lease {
        epoch: u64::MAX,
        broker_pid: 1,
        cap_generation: 0,
    };
    let mut m = Model::new(max);
    m.init(Lease {
        cap_generation: 1,
        ..max
    })
    .unwrap();
    m.fence(0, 10).unwrap();
    m.observe_status(1, 0).unwrap();
    assert_eq!(m.observe_bme(1, false), Err(Reject::Quarantined));
    assert_eq!(m.state(), State::Quarantined);
}

// I4
#[test]
fn i4_no_submit_after_revoke() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.revoke(0, 100).unwrap();
    assert_eq!(m.submit(L1), Err(Reject::Revoked));
    assert_eq!(m.slot(0), Some(Slot::Published));
    ack(&mut m, 1);
    assert_eq!(m.submit(m.lease()), Err(Reject::Revoked));
    assert_eq!(m.start(), Err(Reject::Revoked));
}

#[test]
fn i4_reinit_requires_fresh_lease() {
    let mut m = running();
    m.revoke(0, 100).unwrap();
    ack(&mut m, 1);
    let old = Lease { epoch: 1, ..L1 };
    assert_eq!(m.init(old), Err(Reject::StaleLease));
    assert_eq!(m.init(L1), Err(Reject::StaleLease));
    let next = next_lease(&m);
    m.init(next).unwrap();
    assert_eq!(m.state(), State::Ready);
    assert_eq!(m.complete(L1, 0, 8, OK), Err(Reject::StaleLease));
}

// I5
#[test]
fn i5_rx_never_gets_buffers() {
    let mut m = running();
    assert_eq!(m.post_rx(), Err(Reject::RxForbidden));
    m.fence(0, 10).unwrap();
    assert_eq!(m.post_rx(), Err(Reject::RxForbidden));
}

// I6
#[test]
fn i6_double_completion_rejected() {
    let mut m = running();
    m.submit(L1).unwrap();
    m.submit(L1).unwrap();
    assert_eq!(m.complete(L1, 0, 8, OK), Ok(Completion::Ok));
    assert_eq!(m.complete(L1, 0, 8, OK), Err(Reject::UnknownSlot));
    assert_eq!(m.state(), State::Fencing);
    assert_eq!(m.slot(1), Some(Slot::Published));
}

// I6 (F19)
#[test]
fn i6_double_release_rejected() {
    let mut m = running();
    m.stop().unwrap();
    m.release().unwrap();
    assert_eq!(m.state(), State::Ready);
    assert_eq!(m.release(), Err(Reject::DoubleRelease));
    m.configure().unwrap();
    m.prepare().unwrap();
    m.start().unwrap();
    m.stop().unwrap();
    assert_eq!(m.release(), Ok(()));
}

#[test]
fn bad_transitions_rejected() {
    let mut m = Model::new(L0);
    assert_eq!(m.configure(), Err(Reject::BadState));
    assert_eq!(m.submit(L0), Err(Reject::BadState));
    assert_eq!(m.release(), Err(Reject::BadState));
    assert_eq!(m.observe_status(0, 0), Err(Reject::BadState));
    assert_eq!(m.init(L0), Err(Reject::StaleLease));
    m.init(L1).unwrap();
    assert_eq!(m.init(next_lease(&m)), Err(Reject::BadState));
    assert_eq!(m.start(), Err(Reject::BadState));
}
