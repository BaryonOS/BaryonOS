//! Host tests for the PCM lifecycle codec: exact wire layouts, the
//! conservative lifecycle subset, device-status handling and fence policy.

use super::*;
use crate::{negotiate_features, Config};

const VERSION_1: u64 = 1 << 32;
const FMT_S16_BIT: u64 = 1 << 5;
const RATE_48000_BIT: u64 = 1 << 7;
const D_INPUT: u8 = 1;

fn config(jacks: u32, streams: u32) -> Config {
    let mut bytes = [0; 12];
    bytes[..4].copy_from_slice(&jacks.to_le_bytes());
    bytes[4..8].copy_from_slice(&streams.to_le_bytes());
    Config::parse(negotiate_features(VERSION_1).unwrap(), &bytes).unwrap()
}

fn info_bytes(direction: u8, features: u32) -> [u8; 32] {
    let mut item = [0; 32];
    item[4..8].copy_from_slice(&features.to_le_bytes());
    item[8..16].copy_from_slice(&FMT_S16_BIT.to_le_bytes());
    item[16..24].copy_from_slice(&RATE_48000_BIT.to_le_bytes());
    item[24] = direction;
    item[25] = 1;
    item[26] = 2;
    item
}

fn info(direction: u8, features: u32) -> StreamInfo {
    let mut bytes = [0u8; 4 + 32];
    bytes[..4].copy_from_slice(&S_OK.to_le_bytes());
    bytes[4..].copy_from_slice(&info_bytes(direction, features));
    let infos = config(0, 2)
        .query_stream(1)
        .unwrap()
        .parse_response(&bytes)
        .unwrap();
    infos.as_slice()[0]
}

fn stream() -> Stream {
    Stream::new(info(0, 0)).unwrap()
}

fn ok(stream: &mut Stream) {
    stream.complete_control(&S_OK.to_le_bytes()).unwrap();
}

fn configured() -> Stream {
    let mut s = stream();
    s.set_params(4, 1920).unwrap();
    ok(&mut s);
    s
}

fn prepared() -> Stream {
    let mut s = configured();
    s.prepare().unwrap();
    ok(&mut s);
    s
}

fn running() -> Stream {
    let mut s = prepared();
    s.start().unwrap();
    ok(&mut s);
    s
}

fn stopped() -> Stream {
    let mut s = running();
    s.stop().unwrap();
    ok(&mut s);
    s
}

fn tx_status(status: u32, latency: u32) -> [u8; 8] {
    let mut bytes = [0; 8];
    bytes[..4].copy_from_slice(&status.to_le_bytes());
    bytes[4..].copy_from_slice(&latency.to_le_bytes());
    bytes
}

fn event_bytes(code: u32, data: u32) -> [u8; 8] {
    let mut bytes = [0; 8];
    bytes[..4].copy_from_slice(&code.to_le_bytes());
    bytes[4..].copy_from_slice(&data.to_le_bytes());
    bytes
}

#[test]
fn constants_match_the_specification_header() {
    assert_eq!(
        [
            R_PCM_SET_PARAMS,
            R_PCM_PREPARE,
            R_PCM_RELEASE,
            R_PCM_START,
            R_PCM_STOP
        ],
        [0x0101, 0x0102, 0x0103, 0x0104, 0x0105]
    );
    assert_eq!(
        [S_OK, S_BAD_MSG, S_NOT_SUPP, S_IO_ERR],
        [0x8000, 0x8001, 0x8002, 0x8003]
    );
    assert_eq!(
        [
            EVT_JACK_CONNECTED,
            EVT_JACK_DISCONNECTED,
            EVT_PCM_PERIOD_ELAPSED,
            EVT_PCM_XRUN,
            EVT_CTL_NOTIFY
        ],
        [0x1000, 0x1001, 0x1100, 0x1101, 0x1200]
    );
    assert_eq!(
        (
            PCM_HDR_BYTES,
            SET_PARAMS_BYTES,
            CONTROL_STATUS_BYTES,
            XFER_BYTES,
            TX_STATUS_BYTES,
            EVENT_BYTES
        ),
        (8, 24, 4, 4, 8, 8)
    );
    assert_eq!(
        (CHANNELS, FORMAT_S16, RATE_48000, FRAME_BYTES),
        (2, 5, 7, 4)
    );
    assert_eq!(MAX_BUFFER_BYTES, 8192);
}

#[test]
fn set_params_wire_layout_is_exact() {
    let mut s = stream();
    let request = s.set_params(4, 1920).unwrap();
    assert_eq!(request.op(), Op::SetParams);
    assert_eq!(request.response_len(), 4);
    let bytes = request.as_bytes();
    assert_eq!(bytes.len(), SET_PARAMS_BYTES);
    assert_eq!(u32_at(bytes, 0), Ok(R_PCM_SET_PARAMS));
    assert_eq!(u32_at(bytes, 4), Ok(1));
    assert_eq!(u32_at(bytes, 8), Ok(7680));
    assert_eq!(u32_at(bytes, 12), Ok(1920));
    assert_eq!(u32_at(bytes, 16), Ok(0));
    assert_eq!(&bytes[20..], &[CHANNELS, FORMAT_S16, RATE_48000, 0]);
    assert_eq!(s.pending(), Some(Op::SetParams));
    assert_eq!(s.params(), None);
    ok(&mut s);
    let params = s.params().unwrap();
    assert_eq!(
        (
            params.buffer_bytes(),
            params.period_bytes(),
            params.features()
        ),
        (7680, 1920, 0)
    );
    assert!(!params.xrun_events());
    assert_eq!(s.state(), State::Configured);
}

#[test]
fn only_advertised_xrun_events_are_selected_never_shared_memory() {
    let all = (1 << 5) - 1;
    let mut s = Stream::new(info(0, all)).unwrap();
    let request = s.set_params(1, 4).unwrap();
    assert_eq!(u32_at(request.as_bytes(), 16), Ok(1 << 4));
    ok(&mut s);
    assert!(s.params().unwrap().xrun_events());
    let mut none = Stream::new(info(0, all & !(1 << 4))).unwrap();
    let request = none.set_params(1, 4).unwrap();
    assert_eq!(u32_at(request.as_bytes(), 16), Ok(0));
}

#[test]
fn simple_requests_are_eight_bytes_with_stream_id() {
    let mut s = configured();
    for (request, code) in [
        (s.prepare().unwrap(), R_PCM_PREPARE),
        (
            {
                ok(&mut s);
                s.start().unwrap()
            },
            R_PCM_START,
        ),
        (
            {
                ok(&mut s);
                s.stop().unwrap()
            },
            R_PCM_STOP,
        ),
        (
            {
                ok(&mut s);
                s.release().unwrap()
            },
            R_PCM_RELEASE,
        ),
    ] {
        let bytes = request.as_bytes();
        assert_eq!(bytes.len(), PCM_HDR_BYTES);
        assert_eq!(u32_at(bytes, 0), Ok(code));
        assert_eq!(u32_at(bytes, 4), Ok(1));
        assert_eq!(request.response_len(), CONTROL_STATUS_BYTES);
    }
}

#[test]
fn capture_and_unsupported_profiles_are_never_driven() {
    assert_eq!(Stream::new(info(D_INPUT, 0)), Err(Error::Unsupported));
    let mut bytes = [0u8; 36];
    bytes[..4].copy_from_slice(&S_OK.to_le_bytes());
    bytes[4..].copy_from_slice(&info_bytes(0, 0));
    bytes[12..20].copy_from_slice(&1u64.to_le_bytes());
    let mono_format = config(0, 2)
        .query_stream(1)
        .unwrap()
        .parse_response(&bytes)
        .unwrap()
        .as_slice()[0];
    assert_eq!(Stream::new(mono_format), Err(Error::Unsupported));
}

#[test]
fn set_params_bounds_are_enforced_before_encoding() {
    for (periods, period) in [
        (0, 1920),
        (5, 1920),
        (1, 0),
        (1, 2049),
        (1, 2050),
        (1, 1919),
        (1, 2),
        (4, 2052),
    ] {
        let mut s = stream();
        assert_eq!(
            s.set_params(periods, period),
            Err(Error::Range),
            "{periods}x{period}"
        );
        assert_eq!(s.pending(), None);
        assert_eq!(s.state(), State::Ready);
    }
    let mut s = stream();
    let request = s.set_params(MAX_PERIODS, MAX_PERIOD_BYTES).unwrap();
    assert_eq!(u32_at(request.as_bytes(), 8), Ok(MAX_BUFFER_BYTES));
    assert_eq!(s.set_params(1, 4), Err(Error::Busy));
}

#[test]
fn lifecycle_subset_matrix_is_exact() {
    use State::*;
    let states = [
        (Ready, stream()),
        (Configured, configured()),
        (Prepared, prepared()),
        (Running, running()),
        (Stopped, stopped()),
    ];
    for (state, template) in states {
        assert_eq!(template.state(), state);
        let allowed = |op: Op| {
            matches!(
                (state, op),
                (Ready | Configured, Op::SetParams)
                    | (Configured, Op::Prepare)
                    | (Prepared | Stopped, Op::Start)
                    | (Running, Op::Stop)
                    | (Stopped, Op::Release)
            )
        };
        for op in [Op::SetParams, Op::Prepare, Op::Start, Op::Stop, Op::Release] {
            let mut s = template;
            let result = match op {
                Op::SetParams => s.set_params(2, 1920),
                Op::Prepare => s.prepare(),
                Op::Start => s.start(),
                Op::Stop => s.stop(),
                Op::Release => s.release(),
            };
            if allowed(op) {
                assert_eq!(result.unwrap().op(), op, "{state:?} {op:?}");
                assert_eq!(s.pending(), Some(op));
            } else {
                assert_eq!(result, Err(Error::BadState), "{state:?} {op:?}");
                assert_eq!(s.pending(), None);
                assert_eq!(s.state(), state);
            }
        }
    }
}

#[test]
fn busy_stream_refuses_a_second_request_until_completion() {
    let mut s = stream();
    s.set_params(1, 4).unwrap();
    assert_eq!(s.set_params(1, 4), Err(Error::Busy));
    assert_eq!(s.prepare(), Err(Error::Busy));
    assert_eq!(s.release(), Err(Error::Busy));
    ok(&mut s);
    assert!(s.prepare().is_ok());
}

#[test]
fn f19_second_release_is_refused_before_encoding() {
    let mut s = stopped();
    s.release().unwrap();
    ok(&mut s);
    assert_eq!(s.state(), State::Ready);
    assert_eq!(s.params(), None);
    assert_eq!(s.release(), Err(Error::DoubleRelease));
    assert_eq!(s.pending(), None);
    // A fresh SET_PARAMS starts a new configured lifetime.
    s.set_params(2, 1920).unwrap();
    ok(&mut s);
    s.prepare().unwrap();
    ok(&mut s);
    s.start().unwrap();
    ok(&mut s);
    s.stop().unwrap();
    ok(&mut s);
    assert!(s.release().is_ok());
}

#[test]
fn control_completion_requires_pending_exact_length_and_status() {
    let mut s = stream();
    assert_eq!(
        s.complete_control(&S_OK.to_le_bytes()),
        Err(Error::NotPending)
    );
    s.set_params(1, 4).unwrap();
    for length in [0, 1, 3, 5, 8] {
        let bytes = [0u8; 8];
        assert_eq!(s.complete_control(&bytes[..length]), Err(Error::Length));
        assert_eq!(s.pending(), Some(Op::SetParams));
    }
    assert_eq!(
        s.complete_control(&S_NOT_SUPP.to_le_bytes()),
        Err(Error::Unsupported)
    );
    assert_eq!(s.pending(), None);
    assert_eq!(s.state(), State::Ready);
    assert_eq!(s.params(), None);
    for status in [0, S_BAD_MSG, S_IO_ERR, 0x8004, u32::MAX] {
        let mut s = configured();
        s.prepare().unwrap();
        assert_eq!(
            s.complete_control(&status.to_le_bytes()),
            Err(Error::Status(status))
        );
        assert_eq!(s.pending(), None);
        assert_eq!(s.state(), State::Configured);
    }
}

#[test]
fn tx_header_only_while_publication_is_allowed() {
    for (template, expected) in [
        (stream(), Err(Error::BadState)),
        (configured(), Err(Error::BadState)),
        (prepared(), Ok(1u32.to_le_bytes())),
        (running(), Ok(1u32.to_le_bytes())),
        (stopped(), Err(Error::BadState)),
    ] {
        assert_eq!(template.tx_header(1920), expected, "{:?}", template.state());
        assert_eq!(template.tx_allowed(), expected.is_ok());
    }
    let s = running();
    for len in [0, 1, 2, 3, 1921, 1924, 4096, usize::MAX] {
        assert_eq!(s.tx_header(len), Err(Error::Range), "{len}");
    }
    assert!(s.tx_header(4).is_ok());
    assert!(s.tx_header(1916).is_ok());
}

#[test]
fn tx_completion_statuses_follow_f9() {
    let mut s = running();
    assert_eq!(
        s.tx_completed(&tx_status(S_OK, 1920)),
        Ok(TxCompletion::Ok {
            latency_bytes: 1920
        })
    );
    assert!(s.tx_allowed());
    assert_eq!(
        s.tx_completed(&tx_status(S_IO_ERR, 7)),
        Ok(TxCompletion::IoError { latency_bytes: 7 })
    );
    assert!(!s.tx_allowed());
    assert_eq!(s.tx_header(4), Err(Error::Faulted));
    assert_eq!(s.state(), State::Running);
    // Only STOP (device-confirmed) clears the fault.
    s.stop().unwrap();
    assert_eq!(s.tx_header(4), Err(Error::Faulted));
    ok(&mut s);
    assert_eq!(s.state(), State::Stopped);
    s.start().unwrap();
    ok(&mut s);
    assert!(s.tx_allowed());
    for status in [0, S_BAD_MSG, S_NOT_SUPP, 0x8004, u32::MAX] {
        let mut s = running();
        let error = s.tx_completed(&tx_status(status, 0)).unwrap_err();
        assert_eq!(error, Error::Status(status));
        assert!(error.requires_fence());
    }
    for length in [0, 4, 7, 9, 16] {
        let mut s = running();
        let bytes = [0u8; 16];
        assert_eq!(s.tx_completed(&bytes[..length]), Err(Error::Length));
    }
}

#[test]
fn tx_completion_after_stop_is_allowed_after_release_it_is_late() {
    let mut s = stopped();
    assert!(s.tx_completed(&tx_status(S_OK, 0)).is_ok());
    s.release().unwrap();
    assert!(s.tx_completed(&tx_status(S_OK, 0)).is_ok());
    ok(&mut s);
    assert_eq!(
        s.tx_completed(&tx_status(S_OK, 0)),
        Err(Error::LateCompletion)
    );
    for mut idle in [stream(), configured()] {
        assert_eq!(
            idle.tx_completed(&tx_status(S_OK, 0)),
            Err(Error::LateCompletion)
        );
    }
}

#[test]
fn events_decode_against_configuration_bounds() {
    let cfg = config(1, 2);
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_JACK_CONNECTED, 0)),
        Ok(Event::JackConnected(0))
    );
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_JACK_DISCONNECTED, 0)),
        Ok(Event::JackDisconnected(0))
    );
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_JACK_CONNECTED, 1)),
        Err(Error::InvalidJack)
    );
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_PCM_PERIOD_ELAPSED, 1)),
        Ok(Event::PeriodElapsed(1))
    );
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_PCM_XRUN, 0)),
        Ok(Event::Xrun(0))
    );
    for data in [2, u32::MAX] {
        assert_eq!(
            parse_event(cfg, &event_bytes(EVT_PCM_XRUN, data)),
            Err(Error::InvalidStream)
        );
    }
    assert_eq!(
        parse_event(cfg, &event_bytes(EVT_CTL_NOTIFY, 0)),
        Err(Error::Unsupported)
    );
    for code in [0, 0x1002, 0x1102, 0x1201, 0x8000, u32::MAX] {
        assert_eq!(
            parse_event(cfg, &event_bytes(code, 0)),
            Err(Error::InvalidEvent)
        );
    }
    for length in [0, 4, 7, 9, 12] {
        let bytes = [0u8; 12];
        assert_eq!(parse_event(cfg, &bytes[..length]), Err(Error::Length));
    }
    assert_eq!(
        parse_event(config(0, 0), &event_bytes(EVT_JACK_CONNECTED, 0)),
        Err(Error::InvalidJack)
    );
}

#[test]
fn stream_events_count_only_while_running_for_this_stream() {
    let mut s = running();
    assert_eq!(s.event(Event::PeriodElapsed(1)), Ok(()));
    assert_eq!(s.event(Event::Xrun(1)), Ok(()));
    assert_eq!((s.periods_elapsed(), s.xruns()), (1, 1));
    assert_eq!(s.event(Event::PeriodElapsed(0)), Err(Error::InvalidStream));
    assert_eq!(s.event(Event::JackConnected(0)), Err(Error::InvalidEvent));
    assert_eq!((s.periods_elapsed(), s.xruns()), (1, 1));
    for mut idle in [stream(), configured(), prepared(), stopped()] {
        assert_eq!(
            idle.event(Event::PeriodElapsed(1)),
            Err(Error::InvalidEvent)
        );
        assert_eq!(idle.event(Event::Xrun(1)), Err(Error::InvalidEvent));
    }
    // A new SET_PARAMS resets the counters of the previous lifetime.
    s.stop().unwrap();
    ok(&mut s);
    s.release().unwrap();
    ok(&mut s);
    s.set_params(1, 4).unwrap();
    ok(&mut s);
    assert_eq!((s.periods_elapsed(), s.xruns()), (0, 0));
}

#[test]
fn f17_invalid_event_budget_fences_after_bound() {
    let mut budget = EventBudget::new();
    for _ in 0..MAX_INVALID_EVENTS {
        assert_eq!(budget.note_invalid(), Ok(()));
    }
    assert_eq!(budget.invalid(), MAX_INVALID_EVENTS);
    assert_eq!(budget.note_invalid(), Err(Error::EventStorm));
    assert_eq!(budget.note_invalid(), Err(Error::EventStorm));
    assert_eq!(budget.invalid(), MAX_INVALID_EVENTS + 2);
    assert_eq!(EventBudget::default(), EventBudget::new());
}

#[test]
fn fence_policy_only_for_device_violations() {
    for error in [
        Error::Length,
        Error::Status(0),
        Error::Status(S_BAD_MSG),
        Error::LateCompletion,
        Error::EventStorm,
    ] {
        assert!(error.requires_fence(), "{error:?}");
    }
    for error in [
        Error::Range,
        Error::Overflow,
        Error::Unsupported,
        Error::BadState,
        Error::DoubleRelease,
        Error::Busy,
        Error::NotPending,
        Error::Faulted,
        Error::InvalidStream,
        Error::InvalidJack,
        Error::InvalidEvent,
    ] {
        assert!(!error.requires_fence(), "{error:?}");
    }
}

#[test]
fn readers_never_panic_on_bad_offsets() {
    assert_eq!(u32_at(&[0; 3], 0), Err(Error::Length));
    assert_eq!(u32_at(&[0; 4], usize::MAX), Err(Error::Overflow));
}
