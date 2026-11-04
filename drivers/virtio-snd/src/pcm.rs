//! Bounded PCM stream lifecycle, TX framing and event codec for virtio-snd
//! (OASIS virtio 1.3, sections 5.14.6.6, 5.14.6.7 and 5.14.6.8).
//!
//! Pure byte encoding, response validation and a per-stream state machine.
//! No transport, queue, DMA, MMIO, broker, app grant or audible output: the
//! kernel transport owns descriptors, slots and fences (`TRANSPORT.md`).
//! Callers pass exactly the device-written used-length bytes of a response.
//!
//! Lifecycle (spec "PCM command lifecycle"): SET_PARAMS -> {SET_PARAMS,
//! PREPARE}; PREPARE -> {SET_PARAMS, PREPARE, START, RELEASE}; START ->
//! {STOP}; STOP -> {START, RELEASE}; RELEASE -> {SET_PARAMS, PREPARE}.
//! This codec drives the conservative subset chosen in `TRANSPORT.md` §5:
//! `Ready -> Configured -> Prepared -> Running <-> Stopped -> Ready`, one
//! RELEASE per configured lifetime (F19), and never a capture stream.

use crate::{Config, StreamInfo};

pub const R_PCM_SET_PARAMS: u32 = 0x0101;
pub const R_PCM_PREPARE: u32 = 0x0102;
pub const R_PCM_RELEASE: u32 = 0x0103;
pub const R_PCM_START: u32 = 0x0104;
pub const R_PCM_STOP: u32 = 0x0105;

pub const S_OK: u32 = 0x8000;
pub const S_BAD_MSG: u32 = 0x8001;
pub const S_NOT_SUPP: u32 = 0x8002;
pub const S_IO_ERR: u32 = 0x8003;

pub const EVT_JACK_CONNECTED: u32 = 0x1000;
pub const EVT_JACK_DISCONNECTED: u32 = 0x1001;
pub const EVT_PCM_PERIOD_ELAPSED: u32 = 0x1100;
pub const EVT_PCM_XRUN: u32 = 0x1101;
pub const EVT_CTL_NOTIFY: u32 = 0x1200;

/// `virtio_snd_pcm_hdr`: code + stream_id.
pub const PCM_HDR_BYTES: usize = 8;
/// `virtio_snd_pcm_set_params`.
pub const SET_PARAMS_BYTES: usize = 24;
/// Device-writable `virtio_snd_hdr` status of every PCM control request.
pub const CONTROL_STATUS_BYTES: usize = 4;
/// `virtio_snd_pcm_xfer` ahead of the PCM bytes of one TX message.
pub const XFER_BYTES: usize = 4;
/// Device-writable `virtio_snd_pcm_status` closing one TX message.
pub const TX_STATUS_BYTES: usize = 8;
/// `virtio_snd_event`: code + data.
pub const EVENT_BYTES: usize = 8;

/// Own bounds (`TRANSPORT.md` §4): four bounce slots of 2048 bytes.
pub const MAX_PERIOD_BYTES: u32 = 2048;
pub const MAX_PERIODS: u32 = 4;
pub const MAX_BUFFER_BYTES: u32 = MAX_PERIOD_BYTES * MAX_PERIODS;
/// F17: repeated invalid events are not ignored forever.
pub const MAX_INVALID_EVENTS: u32 = 8;

/// Fixed initial playback profile: stereo, S16, 48 kHz.
pub const CHANNELS: u8 = 2;
pub const FORMAT_S16: u8 = 5;
pub const RATE_48000: u8 = 7;
/// Bytes per frame for the fixed profile; every TX message carries whole
/// frames (own rule, the specification only requires period-sized buffers).
pub const FRAME_BYTES: u32 = CHANNELS as u32 * 2;

const F_SHMEM_HOST: u32 = 1 << 0;
const F_SHMEM_GUEST: u32 = 1 << 1;
const F_MSG_POLLING: u32 = 1 << 2;
const F_EVT_SHMEM_PERIODS: u32 = 1 << 3;
const F_EVT_XRUNS: u32 = 1 << 4;
/// Message-based transport only: shared memory and polling are never
/// selected, so period events through shared memory cannot be either.
const SELECTABLE_FEATURES: u32 = F_EVT_XRUNS;
const NEVER_SELECTED: u32 = F_SHMEM_HOST | F_SHMEM_GUEST | F_MSG_POLLING | F_EVT_SHMEM_PERIODS;
const _: () = assert!(SELECTABLE_FEATURES & NEVER_SELECTED == 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Device wrote a different used length than the message defines.
    Length,
    /// A driver-side argument is outside its bound; nothing was sent.
    Range,
    Overflow,
    /// The stream profile cannot be driven (capture, no stereo S16 48 kHz)
    /// or the device answered NOT_SUPP; the stream state is unchanged.
    Unsupported,
    /// The request is not valid in the current lifecycle state.
    BadState,
    /// F19: a second RELEASE for the same configured lifetime.
    DoubleRelease,
    /// One control request is already outstanding for this stream.
    Busy,
    /// A control response arrived while nothing was outstanding.
    NotPending,
    /// Device status other than OK/NOT_SUPP (control) or OK/IO_ERR (TX).
    Status(u32),
    /// A TX completion for a stream that has no I/O outstanding by contract.
    LateCompletion,
    /// TX refused after an I/O error until the stream is stopped.
    Faulted,
    /// Event stream/jack id outside the device configuration.
    InvalidStream,
    InvalidJack,
    /// Unknown event code, or an event the stream cannot receive now.
    InvalidEvent,
    /// More than `MAX_INVALID_EVENTS` invalid events were observed.
    EventStorm,
}

impl Error {
    /// True when device-delivered bytes violated the contract, so the
    /// transport must fence (`TRANSPORT.md` F5/F7/F9/F17). Driver-side
    /// refusals never require a fence: nothing was published.
    pub const fn requires_fence(self) -> bool {
        matches!(
            self,
            Self::Length | Self::Status(_) | Self::LateCompletion | Self::EventStorm
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Ready,
    Configured,
    Prepared,
    Running,
    Stopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    SetParams,
    Prepare,
    Start,
    Stop,
    Release,
}

impl Op {
    const fn code(self) -> u32 {
        match self {
            Self::SetParams => R_PCM_SET_PARAMS,
            Self::Prepare => R_PCM_PREPARE,
            Self::Start => R_PCM_START,
            Self::Stop => R_PCM_STOP,
            Self::Release => R_PCM_RELEASE,
        }
    }
}

/// Accepted stream parameters; only constructible through [`Stream::set_params`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Params {
    buffer_bytes: u32,
    period_bytes: u32,
    features: u32,
}

impl Params {
    pub const fn buffer_bytes(self) -> u32 {
        self.buffer_bytes
    }

    pub const fn period_bytes(self) -> u32 {
        self.period_bytes
    }

    pub const fn features(self) -> u32 {
        self.features
    }

    pub const fn xrun_events(self) -> bool {
        self.features & F_EVT_XRUNS != 0
    }
}

/// One encoded, device-readable control request. Correlation only; the
/// transport posts it together with a `CONTROL_STATUS_BYTES` response buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request {
    bytes: [u8; SET_PARAMS_BYTES],
    len: usize,
    op: Op,
}

impl Request {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub const fn op(self) -> Op {
        self.op
    }

    /// Exact device-writable buffer size the transport must post.
    pub const fn response_len(self) -> usize {
        CONTROL_STATUS_BYTES
    }
}

/// Outcome of one completed TX message. `latency_bytes` is reported as the
/// device wrote it; it is not validated and proves nothing about playback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TxCompletion {
    Ok {
        latency_bytes: u32,
    },
    /// F9: the slot is free again, the stream must be stopped before new TX.
    IoError {
        latency_bytes: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    JackConnected(u32),
    JackDisconnected(u32),
    PeriodElapsed(u32),
    Xrun(u32),
}

/// Decodes one event buffer against the parsed device configuration.
/// `bytes` must be exactly the used length the device wrote.
pub fn parse_event(config: Config, bytes: &[u8]) -> Result<Event, Error> {
    if bytes.len() != EVENT_BYTES {
        return Err(Error::Length);
    }
    let code = u32_at(bytes, 0)?;
    let data = u32_at(bytes, 4)?;
    match code {
        EVT_JACK_CONNECTED | EVT_JACK_DISCONNECTED => {
            if data >= config.jacks() {
                return Err(Error::InvalidJack);
            }
            Ok(if code == EVT_JACK_CONNECTED {
                Event::JackConnected(data)
            } else {
                Event::JackDisconnected(data)
            })
        }
        EVT_PCM_PERIOD_ELAPSED | EVT_PCM_XRUN => {
            if data >= config.streams() {
                return Err(Error::InvalidStream);
            }
            Ok(if code == EVT_PCM_PERIOD_ELAPSED {
                Event::PeriodElapsed(data)
            } else {
                Event::Xrun(data)
            })
        }
        // CTLS is never negotiated, so a control notification is a violation.
        EVT_CTL_NOTIFY => Err(Error::Unsupported),
        _ => Err(Error::InvalidEvent),
    }
}

/// F17 counter: invalid events are ignored up to a bound, then fenced.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EventBudget {
    invalid: u32,
}

impl EventBudget {
    pub const fn new() -> Self {
        Self { invalid: 0 }
    }

    pub const fn invalid(self) -> u32 {
        self.invalid
    }

    /// Records one invalid event; the bound itself is the fence trigger.
    pub fn note_invalid(&mut self) -> Result<(), Error> {
        self.invalid = self.invalid.saturating_add(1);
        if self.invalid > MAX_INVALID_EVENTS {
            Err(Error::EventStorm)
        } else {
            Ok(())
        }
    }
}

/// Per-stream lifecycle for one advertised output stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stream {
    info: StreamInfo,
    state: State,
    pending: Option<Op>,
    params: Option<Params>,
    candidate: Option<Params>,
    released: bool,
    io_error: bool,
    periods: u32,
    xruns: u32,
}

impl Stream {
    /// Only an output stream advertising stereo S16 at 48 kHz is driven.
    pub fn new(info: StreamInfo) -> Result<Self, Error> {
        info.stereo_s16_48000().map_err(|_| Error::Unsupported)?;
        Ok(Self {
            info,
            state: State::Ready,
            pending: None,
            params: None,
            candidate: None,
            released: false,
            io_error: false,
            periods: 0,
            xruns: 0,
        })
    }

    pub const fn id(self) -> u32 {
        self.info.id()
    }

    pub const fn state(self) -> State {
        self.state
    }

    pub const fn pending(self) -> Option<Op> {
        self.pending
    }

    pub const fn params(self) -> Option<Params> {
        self.params
    }

    pub const fn periods_elapsed(self) -> u32 {
        self.periods
    }

    pub const fn xruns(self) -> u32 {
        self.xruns
    }

    /// TX publication is allowed only here (`TRANSPORT.md` §4) and never
    /// after an I/O error until the stream has been stopped.
    pub const fn tx_allowed(self) -> bool {
        matches!(self.state, State::Prepared | State::Running) && !self.io_error
    }

    /// SET_PARAMS for `periods` periods of `period_bytes`: `buffer_bytes`
    /// is their product, so the specification's divisibility rule holds by
    /// construction. XRUN events are selected only when the device advertises
    /// them; shared memory and polling are never selected.
    pub fn set_params(&mut self, periods: u32, period_bytes: u32) -> Result<Request, Error> {
        self.check_idle()?;
        if !matches!(self.state, State::Ready | State::Configured) {
            return Err(Error::BadState);
        }
        if period_bytes == 0
            || period_bytes > MAX_PERIOD_BYTES
            || !period_bytes.is_multiple_of(FRAME_BYTES)
            || periods == 0
            || periods > MAX_PERIODS
        {
            return Err(Error::Range);
        }
        let buffer_bytes = period_bytes.checked_mul(periods).ok_or(Error::Overflow)?;
        if buffer_bytes > MAX_BUFFER_BYTES {
            return Err(Error::Range);
        }
        let features = self.info.features() & SELECTABLE_FEATURES;
        let params = Params {
            buffer_bytes,
            period_bytes,
            features,
        };
        let mut bytes = [0; SET_PARAMS_BYTES];
        self.header(Op::SetParams, &mut bytes);
        bytes[8..12].copy_from_slice(&buffer_bytes.to_le_bytes());
        bytes[12..16].copy_from_slice(&period_bytes.to_le_bytes());
        bytes[16..20].copy_from_slice(&features.to_le_bytes());
        bytes[20] = CHANNELS;
        bytes[21] = FORMAT_S16;
        bytes[22] = RATE_48000;
        bytes[23] = 0;
        self.candidate = Some(params);
        self.pending = Some(Op::SetParams);
        Ok(Request {
            bytes,
            len: SET_PARAMS_BYTES,
            op: Op::SetParams,
        })
    }

    pub fn prepare(&mut self) -> Result<Request, Error> {
        self.check_idle()?;
        if self.state != State::Configured {
            return Err(Error::BadState);
        }
        Ok(self.simple(Op::Prepare))
    }

    pub fn start(&mut self) -> Result<Request, Error> {
        self.check_idle()?;
        if !matches!(self.state, State::Prepared | State::Stopped) {
            return Err(Error::BadState);
        }
        Ok(self.simple(Op::Start))
    }

    pub fn stop(&mut self) -> Result<Request, Error> {
        self.check_idle()?;
        if self.state != State::Running {
            return Err(Error::BadState);
        }
        Ok(self.simple(Op::Stop))
    }

    /// F19: one RELEASE per configured lifetime, refused before encoding.
    pub fn release(&mut self) -> Result<Request, Error> {
        self.check_idle()?;
        if self.released {
            return Err(Error::DoubleRelease);
        }
        if self.state != State::Stopped {
            return Err(Error::BadState);
        }
        Ok(self.simple(Op::Release))
    }

    /// Applies the device status of the outstanding control request.
    /// `bytes` must be exactly the used length the device wrote.
    pub fn complete_control(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let op = self.pending.ok_or(Error::NotPending)?;
        if bytes.len() != CONTROL_STATUS_BYTES {
            return Err(Error::Length);
        }
        let status = u32_at(bytes, 0)?;
        self.pending = None;
        match status {
            S_OK => {
                self.apply(op);
                Ok(())
            }
            S_NOT_SUPP => {
                self.candidate = None;
                Err(Error::Unsupported)
            }
            other => Err(Error::Status(other)),
        }
    }

    /// Header of one TX message carrying `len` PCM bytes: whole frames, at
    /// most one period, only while publication is allowed.
    pub fn tx_header(&self, len: usize) -> Result<[u8; XFER_BYTES], Error> {
        if !self.tx_allowed() {
            return Err(if self.io_error {
                Error::Faulted
            } else {
                Error::BadState
            });
        }
        let period = self.params.ok_or(Error::BadState)?.period_bytes;
        let len = u32::try_from(len).map_err(|_| Error::Range)?;
        if len == 0 || len > period || !len.is_multiple_of(FRAME_BYTES) {
            return Err(Error::Range);
        }
        Ok(self.id().to_le_bytes())
    }

    /// Validates the device-written status of one TX message. Pending I/O
    /// may complete after STOP, but never once RELEASE succeeded.
    pub fn tx_completed(&mut self, bytes: &[u8]) -> Result<TxCompletion, Error> {
        if !matches!(
            self.state,
            State::Prepared | State::Running | State::Stopped
        ) {
            return Err(Error::LateCompletion);
        }
        if bytes.len() != TX_STATUS_BYTES {
            return Err(Error::Length);
        }
        let status = u32_at(bytes, 0)?;
        let latency_bytes = u32_at(bytes, 4)?;
        match status {
            S_OK => Ok(TxCompletion::Ok { latency_bytes }),
            S_IO_ERR => {
                self.io_error = true;
                Ok(TxCompletion::IoError { latency_bytes })
            }
            other => Err(Error::Status(other)),
        }
    }

    /// Routes a decoded PCM event to this stream; jack events and events for
    /// another stream or an idle stream are invalid here.
    pub fn event(&mut self, event: Event) -> Result<(), Error> {
        let (id, counter) = match event {
            Event::PeriodElapsed(id) => (id, &mut self.periods),
            Event::Xrun(id) => (id, &mut self.xruns),
            Event::JackConnected(_) | Event::JackDisconnected(_) => {
                return Err(Error::InvalidEvent)
            }
        };
        if id != self.info.id() {
            return Err(Error::InvalidStream);
        }
        if self.state != State::Running {
            return Err(Error::InvalidEvent);
        }
        *counter = counter.saturating_add(1);
        Ok(())
    }

    /// At most one control request is outstanding per stream (controlq owner
    /// posts one request at a time, `TRANSPORT.md` §3).
    fn check_idle(&self) -> Result<(), Error> {
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        Ok(())
    }

    fn header(&self, op: Op, bytes: &mut [u8; SET_PARAMS_BYTES]) {
        bytes[..4].copy_from_slice(&op.code().to_le_bytes());
        bytes[4..8].copy_from_slice(&self.id().to_le_bytes());
    }

    fn simple(&mut self, op: Op) -> Request {
        let mut bytes = [0; SET_PARAMS_BYTES];
        self.header(op, &mut bytes);
        self.pending = Some(op);
        Request {
            bytes,
            len: PCM_HDR_BYTES,
            op,
        }
    }

    fn apply(&mut self, op: Op) {
        match op {
            Op::SetParams => {
                self.params = self.candidate.take();
                self.state = State::Configured;
                self.released = false;
                self.io_error = false;
                self.periods = 0;
                self.xruns = 0;
            }
            Op::Prepare => self.state = State::Prepared,
            Op::Start => self.state = State::Running,
            Op::Stop => {
                self.state = State::Stopped;
                self.io_error = false;
            }
            Op::Release => {
                self.state = State::Ready;
                self.released = true;
                self.params = None;
            }
        }
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Overflow)?;
    let raw = bytes.get(offset..end).ok_or(Error::Length)?;
    Ok(u32::from_le_bytes(
        raw.try_into().map_err(|_| Error::Length)?,
    ))
}

#[cfg(test)]
#[path = "pcm_tests.rs"]
mod tests;
