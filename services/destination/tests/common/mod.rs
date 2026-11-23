#![allow(dead_code)]
//! Host-only test doubles for the destination core.
//!
//! `SimDisk` models the per-device kernel broker of spec 2.7 over a disk with
//! an explicit volatile write cache: a completed write is visible to later
//! reads but durable only after a later completed FLUSH; a power cut keeps
//! the durable image plus any subset of the cache. It injects submission
//! results (-8 for a number of ticks or forever, -14, -13), completion
//! results (Io, Timeout, Unknown) with or without the data reaching the
//! device, pending polls, late completion of a timed-out write, and the reset
//! that follows a timed-out request. One request at most is outstanding; a
//! submission while one is pending is answered -8, as the K5 broker does.
//! `tests/sim_contract.rs` checks it against the broker contract of 2.7 (P12).
//!
//! Broker deadline (property 4 of 2.7, revision 5 Q4): with
//! [`Deadline::Enforced`] (opt-in) a request whose device result is ready
//! only after admission + `TIMEOUT` is answered -16 (WRITE, FLUSH) or -13
//! (READ) by the model itself and resets the device, however late the first
//! poll after the deadline comes. The deadline is evaluated lazily at that
//! poll, so the reset starts at the poll that consumes that result. The default,
//! [`Deadline::Tripwire`], panics instead (review 9, OPT9-7): only N26 (e),
//! N26 (f) and the "given up on" case of P12 property 2 opt out with
//! [`Deadline::Unchecked`], because they break property 4 on purpose.
//! Quarantine and offline (property 5) are device state: a broker reset
//! (after a -16 or -13 result) that ends per [`ResetEnd::Timeout`] or
//! [`ResetEnd::ReinitFails`], or a submission answered -13 or -14 by an
//! injected fault, leaves every later submission refused with that code
//! until a simulated reboot (a new `SimDisk` on the durable image). The
//! state is entered lazily, at the first submission at or after the end of
//! the reset interval. The device's own reset injected with `Fault::again`
//! (N26 (d)) always ends ready; `reset_end` does not apply to it.
//! Stray polls (2.7 poll rule, Q6): a poll with no request pending is
//! answered -1 and counted; dropping a `SimDisk` with a non-zero count fails
//! the test unless the test sets `stray_polls_allowed` (the P12 tests that
//! poll so on purpose, and the Q6 kill test in `broker.rs`).
//!
//! Whether a device reset keeps or drops the volatile cache is not known for
//! QEMU (O9), so the reset has two models (section 6, revision 3): the cache
//! kept across the reset (default), or dropped at it. Rows whose outcome
//! passes through a reset run under both with [`both_reset_models`].
//!
//! `TestVerifier` is a stand-in for `volume_core::archive::verify` over a
//! simplified archive layout (64-byte header with the v1 magic, payload,
//! 16-byte trailer with end magic and FNV-64); it is not the v1 format.
use destination_core::{
    catalog::{self, Catalog, Record},
    device::{Broker, Request, RESET_TIMEOUT, TIMEOUT},
    fnv64, get64,
    service::{Entropy, Env, Meta, Outcome, Service, Source, Summary, Verifier},
    wire::{self, Header},
    Status, FNV_OFFSET, ID, SECTOR,
};

pub const PORTAL: u64 = 7;
pub const DIALOG: u64 = 5;
pub const OTHER: u64 = 9;
pub const SECTORS: u64 = 96;
pub const TEST_ID: [u8; ID] = *b"dest-test-id-001";

std::thread_local! {
    /// Reset model of every `SimDisk` created on this thread (each test runs
    /// on its own thread): true drops the volatile cache at a reset.
    static RESET_DROPS_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `body` once per reset model (cache kept, then cache dropped); every
/// `SimDisk` the body creates, cold boots included, uses that model.
pub fn both_reset_models(mut body: impl FnMut(bool)) {
    for drops in [false, true] {
        RESET_DROPS_CACHE.with(|model| model.set(drops));
        body(drops);
    }
    RESET_DROPS_CACHE.with(|model| model.set(false));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Read(u64),
    Write(u64),
    Flush,
}

impl Op {
    fn of(request: &Request<'_>) -> Self {
        match request {
            Request::Read(sector) => Op::Read(*sector),
            Request::Write(sector, _) => Op::Write(*sector),
            Request::Flush => Op::Flush,
        }
    }
    pub fn mutating(self) -> bool {
        !matches!(self, Op::Read(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Submission admitted.
    Submit(Op),
    /// Submission answered -8 (consecutive refusals are logged once).
    Refused(Op),
    /// Completion consumed with this kernel result.
    Complete(Op, i64),
    ResetAck,
    LateLand(u64),
    /// The service's handler returned (logged by the harness).
    Handled(u16, i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Late {
    /// Lands when the reset that follows the timeout is acknowledged.
    AtResetAck,
    /// Lands right after the next completed FLUSH.
    AfterNextFlush,
    /// Lands when the test calls `land_late`.
    Manual,
}

/// Broker deadline of property 4 (2.7) for a request whose device result is
/// ready only after admission + `TIMEOUT`, decided at the first poll at or
/// after the deadline whether or not that result is ready by then.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Deadline {
    /// Default: such a request is a test defect and panics (OPT9-7).
    #[default]
    Tripwire,
    /// Opt-in: the model answers it -16 (mutating) or -13 (read) and resets.
    Enforced,
    /// Opt-out for N26 (e), N26 (f) and the P12 property-2 "given up on"
    /// case, which keep a request pending past the deadline on purpose.
    Unchecked,
}

/// How a device reset ends (property 5 of 2.7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResetEnd {
    /// Acknowledged after `RESET_TIMEOUT`; the device is ready again.
    #[default]
    Ready,
    /// Not acknowledged: quarantined, every submission -13 until reboot.
    Timeout,
    /// Acknowledged, but re-initialisation fails: offline, -14 until reboot.
    ReinitFails,
}

/// (request, fault, written data, tick its result is ready, admission tick).
type Inflight = (Op, Fault, [u8; SECTOR], u64, u64);

/// Broker state besides the medium (P12 snapshots): the admitted request,
/// the request being submitted, the end of the reset and of an own-reset
/// refusal, refusal forever, a late write, and quarantine or offline.
pub type BrokerState = (
    Option<Inflight>,
    Option<(Op, Fault, [u8; SECTOR])>,
    Option<u64>,
    Option<u64>,
    bool,
    Option<(u64, [u8; SECTOR], Late)>,
    Option<i64>,
);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fault {
    /// Kernel error on submission (-14 absent/offline, -13 quarantined);
    /// -13 and -14 are then held as device state until a simulated reboot.
    pub submit: i64,
    /// -8 on submission: `Some(None)` forever for this and every later
    /// request, `Some(Some(t))` for `t` ticks, which models the device's own
    /// reset (N26 (d)) and so drops the cache under the dropping model.
    pub again: Option<Option<u64>>,
    /// The completion poll returns -8 for this many ticks.
    pub pending: u64,
    /// Device result at completion (0 success, -12, -13, -16, -14).
    pub code: i64,
    /// A failed write still reaches the cache; a failed flush still drains it.
    pub lands: bool,
    pub late: Option<Late>,
    /// A write that reports success but stores nothing (N37 (b)).
    pub silent: bool,
}

impl Fault {
    pub fn code(code: i64) -> Self {
        Self {
            code,
            ..Self::default()
        }
    }
}

/// Request counts since the last `arm`, as seen by the fault hook.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub reads: usize,
    pub writes: usize,
    pub catalog_writes: usize,
    pub entry_writes: usize,
    pub flushes: usize,
    pub mutating: usize,
}

type Hook = Box<dyn FnMut(Op, &Counts) -> Option<Fault>>;

pub struct SimDisk {
    pub durable: Vec<[u8; SECTOR]>,
    pub cache: Vec<(u64, [u8; SECTOR])>,
    pub tick: u64,
    pub log: Vec<Event>,
    pub counts: Counts,
    /// Totals over the disk's life: admitted writes and flushes.
    pub writes: usize,
    pub flushes: usize,
    pub fault: Option<Hook>,
    /// Power cut at the n-th mutating request since `arm`: it and every later
    /// request fail and change nothing.
    pub crash_at: Option<usize>,
    pub crashed: bool,
    /// Reset model: a reset drops the volatile cache instead of keeping it.
    pub reset_drops_cache: bool,
    /// Cached writes lost at resets under the dropping model (non-vacuity).
    pub dropped: usize,
    /// Property 4 (2.7): see [`Deadline`].
    pub deadline: Deadline,
    /// Property 5 (2.7): how every later broker reset ends (after a -16 or
    /// -13 result); not the own reset injected with `Fault::again`.
    pub reset_end: ResetEnd,
    /// Polls with no request pending (2.7 poll rule).
    pub stray_polls: usize,
    /// Set only by tests that poll with nothing pending on purpose (P12) or
    /// that inject such polls to show the counter works (Q6 kill test).
    pub stray_polls_allowed: bool,
    current: Option<(Op, Fault, [u8; SECTOR])>,
    again_until: Option<u64>,
    again_forever: bool,
    inflight: Option<Inflight>,
    resetting_until: Option<u64>,
    late: Option<(u64, [u8; SECTOR], Late)>,
    /// Quarantined (-13) or offline (-14) until a simulated reboot.
    down: Option<i64>,
    calls: u64,
}

impl Drop for SimDisk {
    fn drop(&mut self) {
        // The 2.7 poll rule, asserted for every service-driven host test.
        if !std::thread::panicking() && !self.stray_polls_allowed {
            assert_eq!(
                self.stray_polls, 0,
                "stray poll: polled with no request pending (2.7 poll rule)"
            );
        }
    }
}

impl SimDisk {
    pub fn new(sectors: u64) -> Self {
        Self::from_image(vec![[0; SECTOR]; sectors as usize])
    }
    pub fn from_image(durable: Vec<[u8; SECTOR]>) -> Self {
        Self {
            durable,
            cache: Vec::new(),
            tick: 0,
            log: Vec::new(),
            counts: Counts::default(),
            writes: 0,
            flushes: 0,
            fault: None,
            crash_at: None,
            crashed: false,
            reset_drops_cache: RESET_DROPS_CACHE.with(|model| model.get()),
            dropped: 0,
            deadline: Deadline::default(),
            reset_end: ResetEnd::default(),
            stray_polls: 0,
            stray_polls_allowed: false,
            current: None,
            again_until: None,
            again_forever: false,
            inflight: None,
            resetting_until: None,
            late: None,
            down: None,
            calls: 0,
        }
    }
    /// Simulated reboot: a power cycle that keeps the durable image and
    /// loses the cache and every broker state, quarantine and offline too.
    pub fn rebooted(&self) -> Self {
        Self::from_image(self.durable.clone())
    }
    /// Quarantined (-13) or offline (-14), held until a simulated reboot.
    pub fn down(&self) -> Option<i64> {
        self.down
    }
    pub fn broker_state(&self) -> BrokerState {
        (
            self.inflight,
            self.current,
            self.resetting_until,
            self.again_until,
            self.again_forever,
            self.late,
            self.down,
        )
    }
    pub fn sectors(&self) -> u64 {
        self.durable.len() as u64
    }
    /// Reset the counts the fault hook and `crash_at` see.
    pub fn arm(&mut self) {
        self.counts = Counts::default();
    }
    pub fn fault(&mut self, hook: impl FnMut(Op, &Counts) -> Option<Fault> + 'static) {
        self.fault = Some(Box::new(hook));
    }
    fn current(&self, sector: u64) -> [u8; SECTOR] {
        self.cache
            .iter()
            .rev()
            .find(|(s, _)| *s == sector)
            .map(|(_, data)| *data)
            .unwrap_or(self.durable[sector as usize])
    }
    /// Visible content (durable plus cache), what a read returns now.
    pub fn visible(&self) -> Vec<[u8; SECTOR]> {
        (0..self.sectors()).map(|s| self.current(s)).collect()
    }
    /// Image after a power cut that kept the cached writes selected by `keep`
    /// (index into the cache in submission order).
    pub fn crash_subset(&self, keep: impl Fn(usize) -> bool) -> Vec<[u8; SECTOR]> {
        let mut image = self.durable.clone();
        for (index, (sector, data)) in self.cache.iter().enumerate() {
            if keep(index) {
                image[*sector as usize] = *data;
            }
        }
        image
    }
    /// Image keeping a prefix of the cache and tearing the next write.
    pub fn crash_torn(&self, keep: usize, bytes: usize) -> Vec<[u8; SECTOR]> {
        let mut image = self.crash_subset(|index| index < keep);
        if let Some((sector, data)) = self.cache.get(keep) {
            image[*sector as usize][..bytes].copy_from_slice(&data[..bytes]);
        }
        image
    }
    /// Every cache-loss image: all subsets when the cache is small, else
    /// prefixes, suffixes and single omissions; plus one torn image per prefix.
    pub fn crash_images(&self) -> Vec<Vec<[u8; SECTOR]>> {
        let n = self.cache.len();
        let mut images = Vec::new();
        if n <= 12 {
            for mask in 0u32..(1 << n) {
                images.push(self.crash_subset(|index| mask & (1 << index) != 0));
            }
        } else {
            for keep in 0..=n {
                images.push(self.crash_subset(|index| index < keep));
                images.push(self.crash_subset(|index| index >= keep));
                images.push(self.crash_subset(|index| index != keep));
            }
        }
        for keep in 0..n {
            images.push(self.crash_torn(keep, 100));
        }
        images
    }
    pub fn land_late(&mut self) {
        if let Some((sector, data, _)) = self.late.take() {
            self.log.push(Event::LateLand(sector));
            self.cache.push((sector, data));
        }
    }
    pub fn has_late(&self) -> bool {
        self.late.is_some()
    }
    fn budget(&mut self) {
        self.calls += 1;
        assert!(
            self.calls < 2_000_000,
            "SimDisk call budget exhausted (unbounded loop)"
        );
    }
    fn drop_cache_at_reset(&mut self) {
        if self.reset_drops_cache {
            self.dropped += self.cache.len();
            self.cache.clear();
        }
    }
    fn refuse(&mut self, op: Op) -> i64 {
        if self.log.last() != Some(&Event::Refused(op)) {
            self.log.push(Event::Refused(op));
        }
        -8
    }
}

impl Broker for SimDisk {
    fn submit(&mut self, request: Request<'_>) -> i64 {
        self.budget();
        self.tick += 1;
        let op = Op::of(&request);
        // As the K5 broker: no request is admitted while one is pending.
        if self.inflight.is_some() {
            return self.refuse(op);
        }
        if let Some(until) = self.resetting_until {
            if self.tick < until {
                return self.refuse(op);
            }
            // Property 5: the reset ends at the first submission after it.
            self.resetting_until = None;
            match self.reset_end {
                ResetEnd::Timeout => self.down = Some(-13),
                ResetEnd::ReinitFails => {
                    self.log.push(Event::ResetAck);
                    self.down = Some(-14);
                }
                ResetEnd::Ready => {
                    self.log.push(Event::ResetAck);
                    if matches!(self.late, Some((_, _, Late::AtResetAck))) {
                        self.land_late();
                    }
                }
            }
        }
        if self.current.map(|(o, _, _)| o) != Some(op) {
            let mut data = [0; SECTOR];
            if let Request::Write(_, bytes) = request {
                data = *bytes;
            }
            let mutating_index = self.counts.mutating;
            match op {
                Op::Read(_) => self.counts.reads += 1,
                Op::Write(sector) => {
                    self.counts.writes += 1;
                    if sector < 8 {
                        self.counts.catalog_writes += 1;
                    } else {
                        self.counts.entry_writes += 1;
                    }
                }
                Op::Flush => self.counts.flushes += 1,
            }
            if op.mutating() {
                self.counts.mutating += 1;
                if self.crash_at == Some(mutating_index) {
                    self.crashed = true;
                }
            }
            // The hook sees the counts before this request was counted.
            let mut before = self.counts;
            match op {
                Op::Read(_) => before.reads -= 1,
                Op::Write(sector) => {
                    before.writes -= 1;
                    if sector < 8 {
                        before.catalog_writes -= 1;
                    } else {
                        before.entry_writes -= 1;
                    }
                }
                Op::Flush => before.flushes -= 1,
            }
            if op.mutating() {
                before.mutating -= 1;
            }
            let fault = match self.fault.as_mut() {
                Some(hook) => hook(op, &before).unwrap_or_default(),
                None => Fault::default(),
            };
            // A device that is down refuses before its own reset could start.
            match fault.again.filter(|_| self.down.is_none()) {
                Some(None) => self.again_forever = true,
                Some(Some(ticks)) => {
                    self.again_until = Some(self.tick + ticks);
                    self.drop_cache_at_reset();
                }
                None => {}
            }
            self.current = Some((op, fault, data));
        }
        // Counted above like any refused attempt, so count assertions still
        // see a request the service tries on a quarantined or offline device.
        if let Some(code) = self.down {
            self.current = None;
            return code;
        }
        if self.again_forever || self.again_until.is_some_and(|until| self.tick < until) {
            return self.refuse(op);
        }
        self.again_until = None;
        let (op, fault, data) = self.current.take().expect("current request");
        if fault.submit != 0 {
            if matches!(fault.submit, -13 | -14) {
                self.down = Some(fault.submit);
            }
            return fault.submit;
        }
        self.log.push(Event::Submit(op));
        match op {
            Op::Write(_) => self.writes += 1,
            Op::Flush => self.flushes += 1,
            Op::Read(_) => {}
        }
        self.inflight = Some((op, fault, data, self.tick + fault.pending, self.tick));
        0
    }

    fn poll(&mut self, buffer: &mut [u8; SECTOR]) -> i64 {
        self.budget();
        self.tick += 1;
        // Property 3 (2.7): a delivered result is not delivered again. With
        // no request outstanding the M3 broker answers `WrongTicket` -1.
        let Some((op, mut fault, data, ready, admitted)) = self.inflight else {
            self.stray_polls += 1;
            return -1;
        };
        // Property 4 (2.7): the deadline runs from admission. It is decided by
        // the ready time, not the poll tick, so a sparse poll cannot skip it.
        let past_deadline = self.tick >= admitted + TIMEOUT && ready > admitted + TIMEOUT;
        if past_deadline {
            match self.deadline {
                Deadline::Tripwire => panic!(
                    "{op:?} pending past TIMEOUT from admission at tick {admitted} \
                     without an explicit Deadline opt-out (property 4, 2.7)"
                ),
                // Answered by the broker, not the device. Model choice, not
                // property 4: the request's own `lands` and `late` are
                // dropped, so nothing it carried reaches the cache.
                Deadline::Enforced => {
                    fault = Fault {
                        code: if op.mutating() { -16 } else { -13 },
                        lands: false,
                        late: None,
                        ..fault
                    };
                }
                Deadline::Unchecked => {}
            }
        }
        if self.tick < ready && !(past_deadline && self.deadline == Deadline::Enforced) {
            return -8;
        }
        self.inflight = None;
        let code = if self.crashed { -12 } else { fault.code };
        let effect = !self.crashed && (code == 0 || fault.lands);
        match op {
            Op::Read(sector) if code == 0 => *buffer = self.current(sector),
            Op::Read(_) => {}
            Op::Write(sector) => {
                if let (Some(late), false) = (fault.late, self.crashed) {
                    self.late = Some((sector, data, late));
                } else if effect && !fault.silent {
                    self.cache.push((sector, data));
                }
            }
            Op::Flush => {
                if effect {
                    for (sector, bytes) in self.cache.drain(..) {
                        self.durable[sector as usize] = bytes;
                    }
                    if code == 0 && matches!(self.late, Some((_, _, Late::AfterNextFlush))) {
                        self.log.push(Event::Complete(op, code));
                        self.land_late();
                        return code;
                    }
                }
            }
        }
        self.log.push(Event::Complete(op, code));
        if code == -16 || code == -13 {
            // A timed-out request resets the device; nothing is admitted
            // before the reset is acknowledged (the 2.7 completion fence).
            self.resetting_until = Some(self.tick + RESET_TIMEOUT);
            self.drop_cache_at_reset();
        }
        code
    }

    fn now(&mut self) -> u64 {
        self.budget();
        self.tick
    }
}

/// Simplified archive (see module comment).
pub fn archive(generation: u64, objects: u64, payload: usize, seed: u8) -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[..8].copy_from_slice(b"NOSBAK\x00\x01");
    bytes[16..24].copy_from_slice(&generation.to_le_bytes());
    bytes[24..32].copy_from_slice(&objects.to_le_bytes());
    for index in 0..payload {
        bytes.push(seed.wrapping_add((index * 7 + index / 251) as u8));
    }
    bytes.extend_from_slice(b"NOSBAKZ\x00");
    let sum = fnv64(FNV_OFFSET, &bytes);
    bytes.extend_from_slice(&sum.to_le_bytes());
    bytes
}

#[derive(Default)]
pub struct TestVerifier;

impl Verifier for TestVerifier {
    fn verify(&mut self, source: &mut dyn Source) -> Result<Summary, Status> {
        let mut bytes = Vec::new();
        let mut buffer = [0; 200];
        loop {
            let count = source.get(&mut buffer)?;
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        let n = bytes.len();
        if n < 80
            || bytes[..8] != *b"NOSBAK\x00\x01"
            || bytes[n - 16..n - 8] != *b"NOSBAKZ\x00"
            || fnv64(FNV_OFFSET, &bytes[..n - 8]) != get64(&bytes, n - 8)
        {
            return Err(Status::Corrupt);
        }
        Ok(Summary {
            generation: get64(&bytes, 16),
            objects: get64(&bytes, 24),
        })
    }
}

pub struct TestEntropy(pub [u8; ID]);

impl Entropy for TestEntropy {
    fn fill(&mut self, id: &mut [u8; ID]) -> Result<(), Status> {
        *id = self.0;
        Ok(())
    }
}

pub fn status_of(body: &[u8; wire::SIZE]) -> Status {
    let code = wire::header(body).status;
    [
        Status::Ok,
        Status::Invalid,
        Status::NoSys,
        Status::Range,
        Status::Access,
        Status::Stale,
        Status::Io,
        Status::Timeout,
        Status::NoDevice,
        Status::Protocol,
        Status::Unknown,
        Status::Full,
        Status::Corrupt,
        Status::NotFound,
        Status::Busy,
    ]
    .into_iter()
    .find(|status| *status as i32 == code)
    .expect("known status")
}

pub fn req(operation: u16) -> Header {
    Header::new(operation, 0, 0, 0)
}

/// Driver for one service instance over a `SimDisk`.
pub struct H {
    pub disk: SimDisk,
    pub svc: Service,
    pub verifier: TestVerifier,
    pub entropy: TestEntropy,
    pub next_id: [u64; 4],
    pub generation: [u64; 4],
}

impl H {
    pub fn boot(disk: SimDisk) -> Self {
        let mut disk = disk;
        let svc = Service::boot(disk.sectors(), PORTAL, &mut disk);
        Self {
            disk,
            svc,
            verifier: TestVerifier,
            entropy: TestEntropy(TEST_ID),
            next_id: [0; 4],
            generation: [0; 4],
        }
    }
    pub fn blank() -> Self {
        Self::boot(SimDisk::new(SECTORS))
    }
    pub fn formatted() -> Self {
        let mut h = Self::blank();
        assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Ok);
        h
    }
    /// Cold boot of a new instance on `image` (durable bytes only).
    pub fn cold(image: Vec<[u8; SECTOR]>) -> Self {
        Self::boot(SimDisk::from_image(image))
    }
    /// Cold boot after a power cut that lost the whole cache.
    pub fn reboot(&self) -> Self {
        Self::cold(self.disk.durable.clone())
    }

    pub fn send(&mut self, meta: Meta, body: &[u8; wire::SIZE]) -> Outcome {
        let mut env = Env {
            broker: &mut self.disk,
            verifier: &mut self.verifier,
            entropy: &mut self.entropy,
        };
        let outcome = self.svc.handle(meta, body, &mut env);
        self.disk.log.push(Event::Handled(
            wire::header(body).operation,
            outcome.status as i32,
        ));
        outcome
    }

    /// Send with the next request ID of the lane and its epoch.
    pub fn on(&mut self, lane: usize, sender: u64, mut header: Header, data: &[u8]) -> Outcome {
        self.next_id[lane] += 1;
        header.request_id = self.next_id[lane];
        header.epoch = self.generation[lane];
        header.length = data.len() as u32;
        let body = wire::encode(&header, data);
        self.send(
            Meta {
                lane,
                sender_pid: sender,
                cap_count: 0,
            },
            &body,
        )
    }
    pub fn control(&mut self, header: Header, data: &[u8]) -> Outcome {
        self.on(0, DIALOG, header, data)
    }
    pub fn portal(&mut self, lane: usize, header: Header, data: &[u8]) -> Outcome {
        self.on(lane, PORTAL, header, data)
    }

    pub fn bind(&mut self, lane: usize, operation: u32, entry: u64) -> Status {
        let generation = self.generation[lane] + 1;
        let id = self.svc.catalog().id;
        let status = self.bind_with(lane, operation, entry, generation, &id);
        if status == Status::Ok {
            self.generation[lane] = generation;
            self.next_id[lane] = 0;
        }
        status
    }
    pub fn bind_with(
        &mut self,
        lane: usize,
        operation: u32,
        entry: u64,
        generation: u64,
        id: &[u8; ID],
    ) -> Status {
        let mut h = req(wire::BIND);
        h.entry = entry;
        h.aux = generation;
        h.count = lane as u32;
        h.name_length = operation;
        self.control(h, id).status
    }
    pub fn unbind(&mut self, lane: usize) -> Status {
        let mut h = req(wire::UNBIND);
        h.count = lane as u32;
        self.control(h, &[]).status
    }

    pub fn create(&mut self, lane: usize, label: &[u8]) -> Outcome {
        let mut h = req(wire::CREATE);
        h.name_length = label.len() as u32;
        self.portal(lane, h, label)
    }
    pub fn append(&mut self, lane: usize, entry: u64, offset: u64, data: &[u8]) -> Outcome {
        let mut h = req(wire::APPEND);
        h.entry = entry;
        h.offset = offset;
        self.portal(lane, h, data)
    }
    pub fn seal(&mut self, lane: usize, entry: u64, total: u64) -> Outcome {
        let mut h = req(wire::SEAL);
        h.entry = entry;
        h.offset = total;
        self.portal(lane, h, &[])
    }
    /// CREATE and every APPEND; returns the provisional ID.
    pub fn write_all(&mut self, lane: usize, bytes: &[u8]) -> u64 {
        let created = self.create(lane, b"backup");
        assert_eq!(created.status, Status::Ok);
        let entry = wire::header(&created.reply.unwrap()).entry;
        for (index, chunk) in bytes.chunks(wire::DATA).enumerate() {
            let outcome = self.append(lane, entry, (index * wire::DATA) as u64, chunk);
            assert_eq!(outcome.status, Status::Ok, "append {index}");
        }
        entry
    }
    /// Bind an export lane, write and seal; returns the sealed ID.
    pub fn export(&mut self, lane: usize, bytes: &[u8]) -> u64 {
        assert_eq!(self.bind(lane, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = self.write_all(lane, bytes);
        assert_eq!(
            self.seal(lane, entry, bytes.len() as u64).status,
            Status::Ok
        );
        assert_eq!(self.unbind(lane), Status::Ok);
        entry
    }

    pub fn list_on(&mut self, lane: usize, sender: u64) -> (Status, Vec<Record>) {
        let mut records = Vec::new();
        let mut cursor = 0;
        loop {
            let mut h = req(wire::LIST);
            h.count = cursor;
            let outcome = self.on(lane, sender, h, &[]);
            let Some(body) = outcome.reply else {
                return (outcome.status, records);
            };
            let reply = wire::header(&body);
            for chunk in wire::data(&body).chunks(wire::LIST_RECORD) {
                records.push(decode_record(chunk));
            }
            if outcome.status != Status::Ok || reply.count == 0 {
                return (outcome.status, records);
            }
            cursor = reply.count;
        }
    }
    pub fn list_ids(&mut self, lane: usize) -> (Status, Vec<u64>) {
        let sender = if lane == 0 { DIALOG } else { PORTAL };
        let (status, records) = self.list_on(lane, sender);
        (status, records.iter().map(|r| r.id).collect())
    }
    pub fn info(&mut self) -> (Status, Header, Vec<u8>) {
        let body = self.control(req(wire::INFO), &[]).reply.unwrap();
        (
            status_of(&body),
            wire::header(&body),
            wire::data(&body).to_vec(),
        )
    }
    /// Read a whole sealed entry through a restore lane.
    pub fn read_entry(&mut self, lane: usize, entry: u64, length: u64) -> Result<Vec<u8>, Status> {
        let mut bytes = Vec::new();
        while (bytes.len() as u64) < length {
            let mut h = req(wire::READ);
            h.entry = entry;
            h.offset = bytes.len() as u64;
            h.aux = (length - bytes.len() as u64).min(wire::DATA as u64);
            let outcome = self.portal(lane, h, &[]);
            if outcome.status != Status::Ok {
                return Err(outcome.status);
            }
            bytes.extend_from_slice(wire::data(&outcome.reply.unwrap()));
        }
        Ok(bytes)
    }
    pub fn verify_entry(&mut self, lane: usize, entry: u64) -> Status {
        let mut h = req(wire::VERIFY);
        h.entry = entry;
        self.portal(lane, h, &[]).status
    }
    /// Device writes and flushes admitted so far.
    pub fn mutations(&self) -> usize {
        self.disk.writes + self.disk.flushes
    }
}

pub fn decode_record(raw: &[u8]) -> Record {
    let mut label = [0; 32];
    label.copy_from_slice(&raw[48..80]);
    Record {
        id: get64(raw, 0),
        first: get64(raw, 8),
        length: get64(raw, 16),
        trailer: get64(raw, 24),
        sealed_sequence: get64(raw, 32),
        label_length: raw[40],
        label,
    }
}

/// Entry bytes as stored in an image.
pub fn extent(image: &[[u8; SECTOR]], record: &Record) -> Vec<u8> {
    let mut bytes = Vec::new();
    for sector in record.first..record.end() {
        bytes.extend_from_slice(&image[sector as usize]);
    }
    bytes.truncate(record.length as usize);
    bytes
}

/// Catalog copy bytes of slot 0 or 1 in an image.
pub fn copy_of(image: &[[u8; SECTOR]], slot: usize) -> [u8; catalog::COPY] {
    let mut copy = [0; catalog::COPY];
    for index in 0..4 {
        copy[index * SECTOR..(index + 1) * SECTOR].copy_from_slice(&image[slot * 4 + index]);
    }
    copy
}

pub fn put_copy(image: &mut [[u8; SECTOR]], slot: usize, copy: &[u8; catalog::COPY]) {
    for index in 0..4 {
        image[slot * 4 + index].copy_from_slice(&copy[index * SECTOR..(index + 1) * SECTOR]);
    }
}

pub fn catalog_of(image: &[[u8; SECTOR]], slot: usize, sectors: u64) -> Option<Catalog> {
    catalog::decode(&copy_of(image, slot), sectors)
}
