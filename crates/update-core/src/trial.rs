//! REL-05 trial-boot decision model (B03-REL05 step S1, ADR-B03-REL05-TPM-20261125).
//!
//! Pure model of the loader's per-boot decision in
//! `docs/implementation/B03-REL05-TRIAL-BOOT-TPM.md` section 7: the three
//! 64-byte TPM records (`TRIAL`, `CONFIRM`, `FLOOR`) and [`step`], which maps
//! one boot's observations to the ordered persistent writes, the markers and
//! the handover. No TPM, UEFI or disk I/O happens here and nothing calls it
//! yet (the loader integration is step S6; `signed-boot` keeps ADR-049).
//!
//! Executor contract (S6): perform `Plan::writes` in order, each followed by
//! the read-back of section 7.2/7.3; `IncrementGen` must read back `expect`.
//! Any failed write or read-back stops the plan: re-read the state and call
//! [`step`] again with the TPM classified as [`Tpm::Fault`]. Then extend PCR 13
//! with `Handover::extend`; if that fails call [`Plan::extend_failed`].
use core::fmt;

use crate::{sha256, Manifest};

pub const RECORD_LEN: usize = 64;
pub const TRIAL_MAGIC: [u8; 8] = *b"NOSTRL01";
pub const CONFIRM_MAGIC: [u8; 8] = *b"NOSCNF01";
pub const FLOOR_MAGIC: [u8; 8] = *b"NOSFLR01";
pub const CONFIRM_FORMAT: u8 = 1;
/// "PASS", the only verdict that confirms.
pub const VERDICT_PASS: u32 = 0x5041_5353;
/// `HEALTH-LIST-1` (section 9.1).
pub const HEALTH_LIST_1: u16 = 1;
/// O3: trial boots per candidate.
pub const MAX_ATTEMPTS: u8 = 3;
/// `TRIAL` keeps a 24-byte digest prefix (U10); `CONFIRM` the full digest.
pub const TRIAL_DIGEST_LEN: usize = 24;

fn u16_at(b: &[u8; RECORD_LEN], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}
fn u32_at(b: &[u8; RECORD_LEN], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64_at(b: &[u8; RECORD_LEN], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrialState {
    Idle = 0,
    Pending = 1,
    Rejected = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrialReason {
    None = 0,
    Exhausted = 1,
    Abandoned = 2,
}

/// `TRIAL` (section 6.2): loader-only, written only while PCR 13 is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrialRecord {
    pub state: TrialState,
    pub slot: u8,
    pub attempts: u8,
    pub reason: TrialReason,
    pub health_list: u16,
    pub generation: u64,
    pub version: u64,
    /// Monotonic: no version at or below it is ever trialled again.
    pub max_rejected: u64,
    pub digest: [u8; TRIAL_DIGEST_LEN],
}

impl TrialRecord {
    /// The record the TEST provisioning tool writes (section 10.3).
    pub const fn provisioned() -> Self {
        Self {
            state: TrialState::Idle,
            slot: 0,
            attempts: 0,
            reason: TrialReason::None,
            health_list: 0,
            generation: 0,
            version: 0,
            max_rejected: 0,
            digest: [0; TRIAL_DIGEST_LEN],
        }
    }

    pub fn encode(&self) -> [u8; RECORD_LEN] {
        let mut b = [0u8; RECORD_LEN];
        b[..8].copy_from_slice(&TRIAL_MAGIC);
        b[8] = self.state as u8;
        b[9] = self.slot;
        b[10] = self.attempts;
        b[11] = self.reason as u8;
        b[12..14].copy_from_slice(&self.health_list.to_le_bytes());
        b[16..24].copy_from_slice(&self.generation.to_le_bytes());
        b[24..32].copy_from_slice(&self.version.to_le_bytes());
        b[32..40].copy_from_slice(&self.max_rejected.to_le_bytes());
        b[40..].copy_from_slice(&self.digest);
        b
    }

    /// Strict decode; any deviation is `None` (anchor loss for the caller).
    pub fn decode(b: &[u8; RECORD_LEN]) -> Option<Self> {
        let state = match b[8] {
            0 => TrialState::Idle,
            1 => TrialState::Pending,
            2 => TrialState::Rejected,
            _ => return None,
        };
        let reason = match b[11] {
            0 => TrialReason::None,
            1 => TrialReason::Exhausted,
            2 => TrialReason::Abandoned,
            _ => return None,
        };
        if b[..8] != TRIAL_MAGIC || b[9] > 1 || b[10] > MAX_ATTEMPTS || b[14..16] != [0, 0] {
            return None;
        }
        Some(Self {
            state,
            slot: b[9],
            attempts: b[10],
            reason,
            health_list: u16_at(b, 12),
            generation: u64_at(b, 16),
            version: u64_at(b, 24),
            max_rejected: u64_at(b, 32),
            digest: b[40..].try_into().unwrap(),
        })
    }
}

/// `CONFIRM` (section 6.2): written by the trial kernel from the BootInfo binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConfirmRecord {
    pub slot: u8,
    pub health_list: u16,
    pub verdict: u32,
    pub generation: u64,
    pub version: u64,
    pub digest: [u8; 32],
}

impl ConfirmRecord {
    pub fn encode(&self) -> [u8; RECORD_LEN] {
        let mut b = [0u8; RECORD_LEN];
        b[..8].copy_from_slice(&CONFIRM_MAGIC);
        b[8] = CONFIRM_FORMAT;
        b[9] = self.slot;
        b[10..12].copy_from_slice(&self.health_list.to_le_bytes());
        b[12..16].copy_from_slice(&self.verdict.to_le_bytes());
        b[16..24].copy_from_slice(&self.generation.to_le_bytes());
        b[24..32].copy_from_slice(&self.version.to_le_bytes());
        b[32..].copy_from_slice(&self.digest);
        b
    }

    pub fn decode(b: &[u8; RECORD_LEN]) -> Option<Self> {
        if b[..8] != CONFIRM_MAGIC || b[8] != CONFIRM_FORMAT || b[9] > 1 {
            return None;
        }
        Some(Self {
            slot: b[9],
            health_list: u16_at(b, 10),
            verdict: u32_at(b, 12),
            generation: u64_at(b, 16),
            version: u64_at(b, 24),
            digest: b[32..].try_into().unwrap(),
        })
    }
}

/// `FLOOR` (section 6.2): the authoritative floor; the UEFI variable is a copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloorRecord {
    pub revision: u64,
    pub floor: u64,
    pub last_generation: u64,
    pub last_digest: [u8; 32],
}

impl FloorRecord {
    pub fn encode(&self) -> [u8; RECORD_LEN] {
        let mut b = [0u8; RECORD_LEN];
        b[..8].copy_from_slice(&FLOOR_MAGIC);
        b[8..16].copy_from_slice(&self.revision.to_le_bytes());
        b[16..24].copy_from_slice(&self.floor.to_le_bytes());
        b[24..32].copy_from_slice(&self.last_generation.to_le_bytes());
        b[32..].copy_from_slice(&self.last_digest);
        b
    }

    pub fn decode(b: &[u8; RECORD_LEN]) -> Option<Self> {
        if b[..8] != FLOOR_MAGIC {
            return None;
        }
        Some(Self {
            revision: u64_at(b, 8),
            floor: u64_at(b, 16),
            last_generation: u64_at(b, 24),
            last_digest: b[32..].try_into().unwrap(),
        })
    }
}

/// What the loader read from the TPM during `INIT` (section 7.1 step 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorRead {
    /// Every `NV_ReadPublic` matched the frozen constants.
    pub public_ok: bool,
    /// `None`: `TPMA_NV_WRITTEN` clear.
    pub trial: Option<[u8; RECORD_LEN]>,
    pub confirm: Option<[u8; RECORD_LEN]>,
    pub floor: Option<[u8; RECORD_LEN]>,
    /// `None`: `GEN` unreadable.
    pub generation: Option<u64>,
}

// No allocator in the loader; one value per boot, so the size is irrelevant.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpm {
    /// No TCG2 protocol or `GetCapability` failed: no extend possible.
    Absent,
    /// Command error, timeout or self-test failure; the extend is still attempted.
    Fault,
    /// `PCR_Read(13) != 0` at loader start.
    PcrDirty,
    Ready(AnchorRead),
}

/// A slot whose manifest and image verified (`verify_manifest`), pre-hashed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotView {
    pub version: u64,
    /// SHA-256 over the 84 signed manifest bytes.
    pub digest: [u8; 32],
}

impl SlotView {
    pub fn of(manifest: &Manifest) -> Self {
        Self {
            version: manifest.version,
            digest: sha256(&manifest.signed_bytes()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub tpm: Tpm,
    pub uefi_floor: u64,
    /// `None`: the slot did not verify.
    pub slots: [Option<SlotView>; 2],
    /// Header `active`; used only to break a tie between equal slots.
    pub hint: u8,
}

/// One persistent write, in plan order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Write {
    /// `SetVariable(NieuwOsMinVersion)` (heal or F1); never lower.
    UefiFloor(u64),
    /// F2.
    Floor(FloorRecord),
    /// T1: `NV_Increment(GEN)`; `NV_Read(GEN)` must return `expect`.
    IncrementGen { expect: u64 },
    /// T2, E1, F3 or abandonment.
    Trial(TrialRecord),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Init,
    Degraded,
    Finalize,
    Abandon,
    Exhausted,
    IdleSelect,
    TrialStart,
    HandoverTrial,
    HandoverKnownGood,
    Halt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TpmClass {
    Absent,
    Fault,
    PcrDirty,
    AnchorLost,
    AnchorBehind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmReject {
    Malformed,
    Verdict,
    Generation,
    Binding,
    /// Valid binding, but the trial version is no longer above the floor.
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Open,
    Closed,
}

/// BootInfo trial extension contents (section 8), copied by the kernel into `CONFIRM`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrialBinding {
    pub generation: u64,
    pub version: u64,
    pub slot: u8,
    pub attempt: u8,
    pub health_list: u16,
    pub digest: [u8; 32],
}

impl TrialBinding {
    /// The record a healthy trial kernel writes (K2).
    pub fn confirmation(&self) -> ConfirmRecord {
        ConfirmRecord {
            slot: self.slot,
            health_list: self.health_list,
            verdict: VERDICT_PASS,
            generation: self.generation,
            version: self.version,
            digest: self.digest,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    KnownGood,
    Degraded(TpmClass),
    Trial(TrialBinding),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handover {
    pub slot: u8,
    pub version: u64,
    pub digest: [u8; 32],
    pub mode: Mode,
    /// `None` only without TCG2.
    pub extend: Option<Phase>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    NoValidSlot,
    TpmExtend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handover(Handover),
    Halt(Halt),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Tpm(TpmClass),
    ModeDegraded,
    ModeKnownGood,
    ModeTrial,
    ConfirmFinalized {
        generation: u64,
        version: u64,
    },
    ConfirmRejected(ConfirmReject),
    TrialAbandoned(u64),
    TrialExhausted(u64),
    TrialRefused(u64),
    TrialStart {
        generation: u64,
        slot: u8,
        version: u64,
        attempt: u8,
    },
    Floor(u64),
    Slot {
        slot: u8,
        version: u64,
    },
    FailNoValidSlot,
    FailTpmExtend,
}

fn slot_name(slot: u8) -> char {
    if slot == 0 {
        'A'
    } else {
        'B'
    }
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Marker::Tpm(class) => f.write_str(match class {
                TpmClass::Absent => "BOOT:TPM:ABSENT",
                TpmClass::Fault => "BOOT:TPM:FAULT",
                TpmClass::PcrDirty => "BOOT:TPM:FAULT:PCR_DIRTY",
                TpmClass::AnchorLost => "BOOT:TPM:ANCHOR_LOST",
                TpmClass::AnchorBehind => "BOOT:TPM:ANCHOR_BEHIND",
            }),
            Marker::ModeDegraded => f.write_str("BOOT:MODE:DEGRADED"),
            Marker::ModeKnownGood => f.write_str("BOOT:MODE:KNOWN_GOOD"),
            Marker::ModeTrial => f.write_str("BOOT:MODE:TRIAL"),
            Marker::ConfirmFinalized {
                generation,
                version,
            } => write!(f, "BOOT:CONFIRM:FINALIZED:{generation}:{version}"),
            Marker::ConfirmRejected(reason) => f.write_str(match reason {
                ConfirmReject::Malformed => "BOOT:CONFIRM:REJECTED:MALFORMED",
                ConfirmReject::Verdict => "BOOT:CONFIRM:REJECTED:VERDICT",
                ConfirmReject::Generation => "BOOT:CONFIRM:REJECTED:GENERATION",
                ConfirmReject::Binding => "BOOT:CONFIRM:REJECTED:BINDING",
                ConfirmReject::Stale => "BOOT:CONFIRM:REJECTED:STALE",
            }),
            Marker::TrialAbandoned(v) => write!(f, "BOOT:TRIAL:ABANDONED:{v}"),
            Marker::TrialExhausted(v) => write!(f, "BOOT:TRIAL:EXHAUSTED:{v}"),
            Marker::TrialRefused(v) => write!(f, "BOOT:TRIAL:REFUSED:{v}"),
            Marker::TrialStart {
                generation,
                slot,
                version,
                attempt,
            } => write!(
                f,
                "BOOT:TRIAL:START:{generation}:{}:{version}:{attempt}",
                slot_name(slot)
            ),
            Marker::Floor(v) => write!(f, "BOOT:FLOOR:{v}"),
            Marker::Slot { slot, version } => {
                write!(f, "BOOT:SLOT:{}:VERSION:{version}", slot_name(slot))
            }
            Marker::FailNoValidSlot => f.write_str("BOOT:FAIL:NO_VALID_SLOT"),
            Marker::FailTpmExtend => f.write_str("BOOT:FAIL:TPM_EXTEND"),
        }
    }
}

/// Fixed-capacity list (no allocator in the loader). Capacities are proven by
/// the exhaustive tests; overflow panics, which in the loader is a halt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct List<T: Copy, const N: usize> {
    items: [Option<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> List<T, N> {
    const fn new() -> Self {
        Self {
            items: [None; N],
            len: 0,
        }
    }
    fn push(&mut self, item: T) {
        self.items[self.len] = Some(item);
        self.len += 1;
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items[..self.len].iter().flatten()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub path: List<State, 8>,
    pub writes: List<Write, 8>,
    pub markers: List<Marker, 12>,
    /// Effective floor after confirmation processing (`BOOT:FLOOR`).
    pub floor: u64,
    pub outcome: Outcome,
}

impl Plan {
    /// Section 7.1 step 9: TCG2 present but the PCR 13 extend failed. The
    /// writes already performed stay; no kernel runs.
    pub fn extend_failed(&mut self) {
        if let Outcome::Handover(Handover {
            extend: Some(_), ..
        }) = self.outcome
        {
            self.outcome = Outcome::Halt(Halt::TpmExtend);
            self.path.push(State::Halt);
            self.markers.push(Marker::FailTpmExtend);
        }
    }
}

enum ConfirmCheck {
    Absent,
    Valid,
    Rejected(ConfirmReject),
}

fn check_confirm(
    trial: &TrialRecord,
    confirm: Option<[u8; RECORD_LEN]>,
    slots: &[Option<SlotView>; 2],
    health_list: u16,
) -> ConfirmCheck {
    let Some(bytes) = confirm else {
        return ConfirmCheck::Absent;
    };
    let Some(c) = ConfirmRecord::decode(&bytes) else {
        return ConfirmCheck::Rejected(ConfirmReject::Malformed);
    };
    if c.verdict != VERDICT_PASS {
        return ConfirmCheck::Rejected(ConfirmReject::Verdict);
    }
    if c.generation != trial.generation {
        return ConfirmCheck::Rejected(ConfirmReject::Generation);
    }
    let slot_ok = slots[usize::from(trial.slot)]
        .is_some_and(|s| s.version == trial.version && s.digest == c.digest);
    if c.slot != trial.slot
        || c.version != trial.version
        || c.health_list != trial.health_list
        || c.health_list != health_list
        || c.digest[..TRIAL_DIGEST_LEN] != trial.digest
        || !slot_ok
    {
        return ConfirmCheck::Rejected(ConfirmReject::Binding);
    }
    ConfirmCheck::Valid
}

/// Pick between two qualifying slots: `lowest` or highest version first;
/// equal versions use `prefer`, except that for a trial candidate (`strict`)
/// equal versions with different digests are ambiguous and select nothing
/// (fail closed; release versions are never reused).
fn pick(
    slots: &[Option<SlotView>; 2],
    qualifies: impl Fn(&SlotView) -> bool,
    prefer: u8,
    lowest: bool,
    strict: bool,
) -> Option<(u8, SlotView)> {
    let a = slots[0].filter(&qualifies);
    let b = slots[1].filter(&qualifies);
    match (a, b) {
        (Some(a), None) => Some((0, a)),
        (None, Some(b)) => Some((1, b)),
        (Some(a), Some(b)) if a.version == b.version => {
            if strict && a.digest != b.digest {
                None
            } else if prefer == 1 {
                Some((1, b))
            } else {
                Some((0, a))
            }
        }
        (Some(a), Some(b)) => {
            if (a.version < b.version) == lowest {
                Some((0, a))
            } else {
                Some((1, b))
            }
        }
        (None, None) => None,
    }
}

fn prefix(digest: &[u8; 32]) -> [u8; TRIAL_DIGEST_LEN] {
    digest[..TRIAL_DIGEST_LEN].try_into().unwrap()
}

fn handover(
    plan: &mut Plan,
    slot: u8,
    view: SlotView,
    mode: Mode,
    extend: Option<Phase>,
    state: State,
) {
    plan.path.push(state);
    plan.markers.push(Marker::Slot {
        slot,
        version: view.version,
    });
    plan.outcome = Outcome::Handover(Handover {
        slot,
        version: view.version,
        digest: view.digest,
        mode,
        extend,
    });
}

fn halt_no_slot(plan: &mut Plan) {
    plan.path.push(State::Halt);
    plan.markers.push(Marker::FailNoValidSlot);
    plan.outcome = Outcome::Halt(Halt::NoValidSlot);
}

/// Section 10.2. Never writes. The floor is the UEFI copy: TPM content is
/// untrusted here (a squatted index must not be able to raise it).
fn degraded(obs: &Observation, mut plan: Plan, class: TpmClass) -> Plan {
    plan.path.push(State::Degraded);
    plan.markers.push(Marker::Tpm(class));
    plan.markers.push(Marker::ModeDegraded);
    let floor = obs.uefi_floor;
    plan.floor = floor;
    plan.markers.push(Marker::Floor(floor));
    let extend = (class != TpmClass::Absent).then_some(Phase::Closed);
    let hint = u8::from(obs.hint == 1);
    // Exact floor first; else the lowest verified slot above it.
    let chosen = pick(&obs.slots, |s| s.version == floor, hint, true, false)
        .or_else(|| pick(&obs.slots, |s| s.version > floor, hint, true, false));
    match chosen {
        Some((slot, view)) => handover(
            &mut plan,
            slot,
            view,
            Mode::Degraded(class),
            extend,
            State::HandoverKnownGood,
        ),
        None => halt_no_slot(&mut plan),
    }
    plan
}

/// One boot's decision (section 7.1). Pure and total.
pub fn step(obs: &Observation, health_list: u16) -> Plan {
    let mut plan = Plan {
        path: List::new(),
        writes: List::new(),
        markers: List::new(),
        floor: 0,
        outcome: Outcome::Halt(Halt::NoValidSlot),
    };
    plan.path.push(State::Init);
    let anchor = match obs.tpm {
        Tpm::Absent => return degraded(obs, plan, TpmClass::Absent),
        Tpm::Fault => return degraded(obs, plan, TpmClass::Fault),
        Tpm::PcrDirty => return degraded(obs, plan, TpmClass::PcrDirty),
        Tpm::Ready(anchor) => anchor,
    };
    let floor_rec = anchor.floor.as_ref().and_then(FloorRecord::decode);
    let trial_rec = anchor.trial.as_ref().and_then(TrialRecord::decode);
    let (Some(fr), Some(mut t), Some(gen)) = (floor_rec, trial_rec, anchor.generation) else {
        return degraded(obs, plan, TpmClass::AnchorLost);
    };
    // Unspecified inconsistencies, all fail closed as anchor loss: a record
    // generation from the future, a PENDING record without a counted attempt,
    // and a FLOOR revision that can no longer be incremented.
    if !anchor.public_ok
        || t.generation > gen
        || fr.last_generation > gen
        || (t.state == TrialState::Pending && t.attempts == 0)
        || fr.revision == u64::MAX
    {
        return degraded(obs, plan, TpmClass::AnchorLost);
    }

    let check = if t.state == TrialState::Pending {
        check_confirm(&t, anchor.confirm, &obs.slots, health_list)
    } else {
        ConfirmCheck::Absent
    };
    let valid = matches!(check, ConfirmCheck::Valid);
    let mut uefi = obs.uefi_floor;
    // Step 2: UEFI ahead of FLOOR is only the interrupted finalization (F1 done).
    if uefi > fr.floor && !(valid && t.version == uefi) {
        return degraded(obs, plan, TpmClass::AnchorBehind);
    }
    if fr.floor > uefi {
        plan.writes.push(Write::UefiFloor(fr.floor));
        uefi = fr.floor;
    }
    let mut floor = uefi.max(fr.floor);

    // Step 4: confirmation before anything else.
    match check {
        ConfirmCheck::Valid if t.version >= floor => {
            plan.path.push(State::Finalize);
            if uefi < t.version {
                plan.writes.push(Write::UefiFloor(t.version)); // F1, commit point
            }
            if fr.floor < t.version {
                plan.writes.push(Write::Floor(FloorRecord {
                    revision: fr.revision + 1,
                    floor: t.version,
                    last_generation: t.generation,
                    last_digest: obs.slots[usize::from(t.slot)].map_or([0; 32], |s| s.digest),
                })); // F2
            }
            t.state = TrialState::Idle;
            plan.writes.push(Write::Trial(t)); // F3
            plan.markers.push(Marker::ConfirmFinalized {
                generation: t.generation,
                version: t.version,
            });
            floor = t.version;
        }
        ConfirmCheck::Valid => plan
            .markers
            .push(Marker::ConfirmRejected(ConfirmReject::Stale)),
        ConfirmCheck::Rejected(reason) => plan.markers.push(Marker::ConfirmRejected(reason)),
        ConfirmCheck::Absent => {}
    }

    // Step 5: classify. A candidate needs a fresh generation.
    let candidate = |t: &TrialRecord| {
        let above = floor.max(t.max_rejected);
        let prefer = if t.state == TrialState::Pending {
            t.slot
        } else {
            u8::from(obs.hint == 1)
        };
        if gen == u64::MAX {
            return None;
        }
        pick(&obs.slots, |s| s.version > above, prefer, false, true)
    };
    let same =
        |t: &TrialRecord, c: &SlotView| c.version == t.version && prefix(&c.digest) == t.digest;
    let mut c = candidate(&t);

    // Steps 6-7: abandonment, then exhaustion. Both raise max_rejected, so C
    // is recomputed (a lower replacement must not become eligible).
    if t.state == TrialState::Pending {
        if let Some((_, view)) = c {
            if !same(&t, &view) {
                plan.path.push(State::Abandon);
                t.state = TrialState::Rejected;
                t.reason = TrialReason::Abandoned;
                t.max_rejected = t.max_rejected.max(t.version);
                plan.writes.push(Write::Trial(t));
                plan.markers.push(Marker::TrialAbandoned(t.version));
                c = candidate(&t);
            } else if t.attempts >= MAX_ATTEMPTS {
                plan.path.push(State::Exhausted);
                t.state = TrialState::Rejected;
                t.reason = TrialReason::Exhausted;
                t.max_rejected = t.max_rejected.max(t.version);
                plan.writes.push(Write::Trial(t));
                plan.markers.push(Marker::TrialExhausted(t.version));
                c = candidate(&t);
            }
        }
    }
    for s in obs.slots.iter().flatten() {
        if s.version > floor && s.version <= t.max_rejected {
            plan.markers.push(Marker::TrialRefused(s.version));
        }
    }
    plan.floor = floor;
    plan.markers.push(Marker::Floor(floor));

    // Step 8.
    if let Some((slot, view)) = c {
        plan.path.push(State::TrialStart);
        let attempt = if t.state == TrialState::Pending && same(&t, &view) {
            t.attempts + 1
        } else {
            1
        };
        let generation = gen + 1;
        plan.writes.push(Write::IncrementGen { expect: generation }); // T1
        plan.writes.push(Write::Trial(TrialRecord {
            state: TrialState::Pending,
            slot,
            attempts: attempt,
            reason: TrialReason::None,
            health_list,
            generation,
            version: view.version,
            max_rejected: t.max_rejected,
            digest: prefix(&view.digest),
        })); // T2
        plan.markers.push(Marker::ModeTrial);
        plan.markers.push(Marker::TrialStart {
            generation,
            slot,
            version: view.version,
            attempt,
        });
        let binding = TrialBinding {
            generation,
            version: view.version,
            slot,
            attempt,
            health_list,
            digest: view.digest,
        };
        handover(
            &mut plan,
            slot,
            view,
            Mode::Trial(binding),
            Some(Phase::Open),
            State::HandoverTrial,
        );
    } else {
        plan.path.push(State::IdleSelect);
        plan.markers.push(Marker::ModeKnownGood);
        match pick(
            &obs.slots,
            |s| s.version == floor,
            u8::from(obs.hint == 1),
            true,
            false,
        ) {
            Some((slot, view)) => handover(
                &mut plan,
                slot,
                view,
                Mode::KnownGood,
                Some(Phase::Closed),
                State::HandoverKnownGood,
            ),
            None => halt_no_slot(&mut plan),
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_round_trip_and_reject_bad_bytes() {
        let t = TrialRecord {
            state: TrialState::Pending,
            slot: 1,
            attempts: 2,
            reason: TrialReason::None,
            health_list: HEALTH_LIST_1,
            generation: 7,
            version: 3,
            max_rejected: 2,
            digest: [9; TRIAL_DIGEST_LEN],
        };
        let bytes = t.encode();
        assert_eq!(TrialRecord::decode(&bytes), Some(t));
        for (at, value) in [(0, b'X'), (8, 3), (9, 2), (10, 4), (11, 3), (14, 1)] {
            let mut bad = bytes;
            bad[at] = value;
            assert_eq!(TrialRecord::decode(&bad), None, "byte {at}");
        }
        let c = ConfirmRecord {
            slot: 1,
            health_list: 1,
            verdict: VERDICT_PASS,
            generation: 7,
            version: 3,
            digest: [5; 32],
        };
        assert_eq!(ConfirmRecord::decode(&c.encode()), Some(c));
        let mut bad = c.encode();
        bad[8] = 2;
        assert_eq!(ConfirmRecord::decode(&bad), None);
        let f = FloorRecord {
            revision: 1,
            floor: 2,
            last_generation: 3,
            last_digest: [1; 32],
        };
        assert_eq!(FloorRecord::decode(&f.encode()), Some(f));
        assert_eq!(FloorRecord::decode(&[0; RECORD_LEN]), None);
        assert_eq!(TrialRecord::decode(&[0; RECORD_LEN]), None);
        assert_eq!(ConfirmRecord::decode(&[0; RECORD_LEN]), None);
    }

    #[test]
    fn verdict_bytes_spell_pass() {
        assert_eq!(&VERDICT_PASS.to_be_bytes(), b"PASS");
    }
}
