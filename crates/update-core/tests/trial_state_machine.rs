//! Host model checks for the REL-05 trial-boot decision (`update_core::trial`,
//! step S1 of docs/implementation/B03-REL05-TRIAL-BOOT-TPM.md). A simulated
//! TPM/UEFI store applies the planned writes; power loss is injected after
//! every write prefix. Pure host evidence: no loader, kernel, TPM or QEMU.
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;
use std::thread;

use update_core::trial::*;

const MAX_VERSION: u64 = 4;
const GEN_CAP: u64 = 5;

type Disk = [Option<(u64, u8)>; 2];

fn digest(version: u64, variant: u8) -> [u8; 32] {
    static TABLE: OnceLock<Vec<[u8; 32]>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        (0..=MAX_VERSION + 1)
            .flat_map(|v| (0..2u8).map(move |w| update_core::sha256(&[v as u8, w, 0x5a])))
            .collect()
    });
    table[(version as usize) * 2 + usize::from(variant)]
}

fn view(slot: Option<(u64, u8)>) -> Option<SlotView> {
    slot.map(|(version, variant)| SlotView {
        version,
        digest: digest(version, variant),
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Store {
    uefi: u64,
    public_ok: bool,
    trial: Option<[u8; RECORD_LEN]>,
    confirm: Option<[u8; RECORD_LEN]>,
    floor: Option<[u8; RECORD_LEN]>,
    generation: Option<u64>,
}

fn provisioned(floor: u64, generation: u64) -> Store {
    Store {
        uefi: floor,
        public_ok: true,
        trial: Some(TrialRecord::provisioned().encode()),
        confirm: None,
        floor: Some(
            FloorRecord {
                revision: 1,
                floor,
                last_generation: 0,
                last_digest: [0; 32],
            }
            .encode(),
        ),
        generation: Some(generation),
    }
}

fn cleared() -> Store {
    Store {
        uefi: 0,
        public_ok: false,
        trial: None,
        confirm: None,
        floor: None,
        generation: None,
    }
}

impl Store {
    fn trial(&self) -> Option<TrialRecord> {
        self.trial.as_ref().and_then(TrialRecord::decode)
    }
    fn floor_rec(&self) -> Option<FloorRecord> {
        self.floor.as_ref().and_then(FloorRecord::decode)
    }
    /// I1's quantity: max(UEFI copy, TPM FLOOR).
    fn effective(&self) -> u64 {
        self.uefi.max(self.floor_rec().map_or(0, |f| f.floor))
    }
    fn apply(&mut self, write: &Write) {
        match *write {
            Write::UefiFloor(v) => self.uefi = v,
            Write::Floor(r) => self.floor = Some(r.encode()),
            Write::IncrementGen { expect } => {
                let g = self.generation.expect("GEN readable") + 1;
                assert_eq!(g, expect, "GEN read-back");
                self.generation = Some(g);
            }
            Write::Trial(r) => self.trial = Some(r.encode()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dev {
    Ready,
    Absent,
    Fault,
    PcrDirty,
    /// NV works, the PCR 13 extend fails (C17b after the reads).
    ExtendBroken,
}

fn observe(store: &Store, disk: &Disk, hint: u8, dev: Dev) -> Observation {
    let tpm = match dev {
        Dev::Absent => Tpm::Absent,
        Dev::Fault => Tpm::Fault,
        Dev::PcrDirty => Tpm::PcrDirty,
        Dev::Ready | Dev::ExtendBroken => Tpm::Ready(AnchorRead {
            public_ok: store.public_ok,
            trial: store.trial,
            confirm: store.confirm,
            floor: store.floor,
            generation: store.generation,
        }),
    };
    Observation {
        tpm,
        uefi_floor: store.uefi,
        slots: [view(disk[0]), view(disk[1])],
        hint,
    }
}

fn boot(store: &Store, disk: &Disk, hint: u8, dev: Dev) -> Plan {
    let mut plan = step(&observe(store, disk, hint, dev), HEALTH_LIST_1);
    if dev == Dev::ExtendBroken {
        plan.extend_failed();
    }
    plan
}

fn writes(plan: &Plan) -> Vec<Write> {
    plan.writes.iter().copied().collect()
}

fn trial_binding(plan: &Plan) -> Option<TrialBinding> {
    match plan.outcome {
        Outcome::Handover(Handover {
            mode: Mode::Trial(b),
            ..
        }) => Some(b),
        _ => None,
    }
}

/// Per-boot safety (I1, I2, I3 ordering, I4, I6, I7). Returns the store after
/// all writes.
fn check_plan(pre: &Store, disk: &Disk, dev: Dev, plan: &Plan) -> Store {
    let ctx = || format!("pre={pre:?} disk={disk:?} dev={dev:?} plan={plan:?}");
    let pre_trial = pre.trial();
    let pre_eff = pre.effective();
    let mut cur = pre.clone();
    for w in plan.writes.iter() {
        match *w {
            Write::UefiFloor(v) => assert!(v >= cur.uefi, "UEFI lowered: {}", ctx()),
            Write::Floor(r) => {
                let old = cur.floor_rec().unwrap_or_else(|| panic!("{}", ctx()));
                assert!(r.floor >= old.floor, "FLOOR lowered: {}", ctx());
                assert_eq!(r.revision, old.revision + 1, "{}", ctx());
            }
            Write::Trial(r) => {
                if let Some(old) = cur.trial() {
                    assert!(
                        r.max_rejected >= old.max_rejected,
                        "max_rejected: {}",
                        ctx()
                    );
                }
            }
            Write::IncrementGen { .. } => {}
        }
        cur.apply(w);
        assert!(cur.effective() >= pre_eff, "I1 within plan: {}", ctx());
    }
    let post = cur;
    let post_eff = post.effective();
    assert!(post_eff >= pre_eff, "I1: {}", ctx());
    let degraded = plan.path.iter().any(|s| *s == State::Degraded);
    // I7: DEGRADED never writes and never trials.
    if degraded || dev != Dev::Ready && dev != Dev::ExtendBroken {
        assert!(degraded, "non-ready TPM must degrade: {}", ctx());
        assert!(plan.writes.is_empty(), "I7 write: {}", ctx());
        assert!(trial_binding(plan).is_none(), "I7 trial: {}", ctx());
    }
    // I2: a raise needs a matching durable confirmation for the pending trial.
    if post_eff > pre_eff {
        let t = pre_trial.unwrap_or_else(|| panic!("I2 no trial: {}", ctx()));
        let c = pre
            .confirm
            .as_ref()
            .and_then(ConfirmRecord::decode)
            .unwrap_or_else(|| panic!("I2 no confirm: {}", ctx()));
        let slot = view(disk[usize::from(t.slot)]).unwrap_or_else(|| panic!("{}", ctx()));
        assert!(
            t.state == TrialState::Pending
                && t.version == post_eff
                && c.verdict == VERDICT_PASS
                && c.generation == t.generation
                && c.slot == t.slot
                && c.version == t.version
                && c.health_list == HEALTH_LIST_1
                && c.digest[..TRIAL_DIGEST_LEN] == t.digest
                && slot.version == t.version
                && slot.digest == c.digest,
            "I2 binding: {}",
            ctx()
        );
    }
    match plan.outcome {
        Outcome::Handover(h) => {
            // I6: only verified images at or above the effective floor.
            let slot = view(disk[usize::from(h.slot)]).unwrap_or_else(|| panic!("{}", ctx()));
            assert_eq!(
                (slot.version, slot.digest),
                (h.version, h.digest),
                "{}",
                ctx()
            );
            // DEGRADED trusts only the UEFI copy (section 10.2); the BFS
            // proves UEFI >= FLOOR in every reachable state (F1 before F2).
            let visible = if degraded { pre.uefi } else { post_eff };
            assert!(h.version >= visible, "I6 below floor: {}", ctx());
            // I7: DEGRADED hands over only the confirmed version (spec 10.2).
            assert!(!degraded || h.version == pre.uefi, "I7 version: {}", ctx());
            match h.mode {
                Mode::Trial(b) => {
                    assert_eq!(h.extend, Some(Phase::Open), "{}", ctx());
                    assert!(matches!(dev, Dev::Ready), "{}", ctx());
                    // I3: the counted attempt is durable before handover.
                    let t = post.trial().unwrap_or_else(|| panic!("{}", ctx()));
                    assert_eq!(t.state, TrialState::Pending, "{}", ctx());
                    assert_eq!(
                        (t.generation, t.version, t.slot, t.attempts, t.digest),
                        (
                            b.generation,
                            b.version,
                            b.slot,
                            b.attempt,
                            b.digest[..TRIAL_DIGEST_LEN].try_into().unwrap()
                        ),
                        "{}",
                        ctx()
                    );
                    assert_eq!(post.generation, Some(b.generation), "{}", ctx());
                    assert!((1..=MAX_ATTEMPTS).contains(&b.attempt), "{}", ctx());
                    // I4: never a version at or below a rejection.
                    if let Some(old) = pre_trial {
                        assert!(b.version > old.max_rejected, "I4: {}", ctx());
                        let carried = old.state == TrialState::Pending
                            && old.version == b.version
                            && old.digest == t.digest;
                        let expect = if carried { old.attempts + 1 } else { 1 };
                        assert_eq!(b.attempt, expect, "attempt count: {}", ctx());
                    }
                    assert!(b.version > post_eff, "{}", ctx());
                }
                Mode::KnownGood => {
                    assert_eq!(h.version, post_eff, "{}", ctx());
                    assert_eq!(h.extend, Some(Phase::Closed), "{}", ctx());
                }
                Mode::Degraded(class) => {
                    let want = (class != TpmClass::Absent).then_some(Phase::Closed);
                    assert_eq!(h.extend, want, "{}", ctx());
                }
            }
        }
        Outcome::Halt(Halt::TpmExtend) => assert_eq!(dev, Dev::ExtendBroken, "{}", ctx()),
        Outcome::Halt(Halt::NoValidSlot) => {
            // I5: a valid known-good slot is never refused.
            assert!(
                disk.iter().all(|s| s.is_none_or(|(v, _)| v != plan.floor)),
                "I5: {}",
                ctx()
            );
        }
    }
    if dev == Dev::ExtendBroken {
        assert!(matches!(plan.outcome, Outcome::Halt(_)), "{}", ctx());
    }
    post
}

/// Every write prefix k (including 0 and the full plan): the persisted state
/// after a power loss must again yield a safe plan.
fn check_crash_prefixes(pre: &Store, disk: &Disk, hint: u8, plan: &Plan) -> usize {
    let ws = writes(plan);
    let mut s = pre.clone();
    for k in 0..=ws.len() {
        if k > 0 {
            s.apply(&ws[k - 1]);
        }
        let again = boot(&s, disk, hint, Dev::Ready);
        check_plan(&s, disk, Dev::Ready, &again);
    }
    ws.len() + 1
}

// ---------------------------------------------------------------------------
// 1. Exhaustive single-step invariants over arbitrary (also unreachable) states.

#[derive(Clone, Copy, Debug)]
enum ConfirmKind {
    None,
    Valid,
    WrongSlot,
    WrongDigest,
    WrongVersion,
    StaleGen,
    FutureGen,
    BadVerdict,
    WrongList,
    Garbage,
}
const CONFIRM_KINDS: [ConfirmKind; 10] = [
    ConfirmKind::None,
    ConfirmKind::Valid,
    ConfirmKind::WrongSlot,
    ConfirmKind::WrongDigest,
    ConfirmKind::WrongVersion,
    ConfirmKind::StaleGen,
    ConfirmKind::FutureGen,
    ConfirmKind::BadVerdict,
    ConfirmKind::WrongList,
    ConfirmKind::Garbage,
];

fn confirm_bytes(kind: ConfirmKind, t: &TrialRecord, disk: &Disk) -> Option<[u8; RECORD_LEN]> {
    let full = view(disk[usize::from(t.slot)])
        .filter(|s| s.version == t.version)
        .map_or(digest(t.version, 0), |s| s.digest);
    let mut c = ConfirmRecord {
        slot: t.slot,
        health_list: HEALTH_LIST_1,
        verdict: VERDICT_PASS,
        generation: t.generation,
        version: t.version,
        digest: full,
    };
    match kind {
        ConfirmKind::None => return None,
        ConfirmKind::Valid => {}
        ConfirmKind::WrongSlot => c.slot ^= 1,
        ConfirmKind::WrongDigest => c.digest = digest(t.version, 1),
        ConfirmKind::WrongVersion => c.version += 1,
        ConfirmKind::StaleGen => c.generation = c.generation.wrapping_sub(1),
        ConfirmKind::FutureGen => c.generation += 1,
        ConfirmKind::BadVerdict => c.verdict = 0x4641_494c,
        ConfirmKind::WrongList => c.health_list = 2,
        ConfirmKind::Garbage => return Some([0xA5; RECORD_LEN]),
    }
    Some(c.encode())
}

fn disk_options() -> Vec<Option<(u64, u8)>> {
    let mut v = vec![None];
    v.extend((0..=MAX_VERSION).map(|x| Some((x, 0))));
    v.push(Some((2, 1)));
    v
}

#[test]
fn exhaustive_single_step_and_every_crash_prefix() {
    let jobs: Vec<(u64, u64)> = (0..=MAX_VERSION)
        .flat_map(|u| (0..=MAX_VERSION).map(move |f| (u, f)))
        .collect();
    let handles: Vec<_> = jobs
        .into_iter()
        .map(|(uefi, floor)| {
            thread::spawn(move || {
                let options = disk_options();
                let (mut states, mut prefixes) = (0usize, 0usize);
                for state in [TrialState::Idle, TrialState::Pending, TrialState::Rejected] {
                    for attempts in 0..=MAX_ATTEMPTS {
                        for version in 0..=MAX_VERSION {
                            for max_rejected in [0, 2, MAX_VERSION] {
                                for slot in 0..2u8 {
                                    let t = TrialRecord {
                                        state,
                                        slot,
                                        attempts,
                                        reason: TrialReason::None,
                                        health_list: HEALTH_LIST_1,
                                        generation: 2,
                                        version,
                                        max_rejected,
                                        digest: digest(version, 0)[..TRIAL_DIGEST_LEN]
                                            .try_into()
                                            .unwrap(),
                                    };
                                    for a in &options {
                                        for b in &options {
                                            let disk = [*a, *b];
                                            for kind in CONFIRM_KINDS {
                                                let mut s = provisioned(floor, 3);
                                                s.uefi = uefi;
                                                s.trial = Some(t.encode());
                                                s.confirm = confirm_bytes(kind, &t, &disk);
                                                for hint in 0..2u8 {
                                                    let plan = boot(&s, &disk, hint, Dev::Ready);
                                                    check_plan(&s, &disk, Dev::Ready, &plan);
                                                    prefixes += check_crash_prefixes(
                                                        &s, &disk, hint, &plan,
                                                    );
                                                    states += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                (states, prefixes)
            })
        })
        .collect();
    let (mut states, mut prefixes) = (0, 0);
    for h in handles {
        let (s, p) = h.join().expect("worker");
        states += s;
        prefixes += p;
    }
    eprintln!("single-step: {states} states, {prefixes} crash prefixes re-booted");
    assert!(states > 1_000_000);
}

#[test]
fn every_tpm_fault_class_degrades_without_writes() {
    let options = disk_options();
    let mut garbage = provisioned(1, 3);
    garbage.trial = Some([0x11; RECORD_LEN]);
    let mut bad_floor = provisioned(1, 3);
    bad_floor.floor = Some([0; RECORD_LEN]);
    let mut no_gen = provisioned(1, 3);
    no_gen.generation = None;
    let mut unwritten = provisioned(1, 3);
    unwritten.trial = None;
    let mut squatted = provisioned(1, 3);
    squatted.public_ok = false;
    let mut future = provisioned(1, 3);
    future.trial = Some(
        TrialRecord {
            generation: 4,
            ..TrialRecord::provisioned()
        }
        .encode(),
    );
    let mut zero_attempt = provisioned(1, 3);
    zero_attempt.trial = Some(
        TrialRecord {
            state: TrialState::Pending,
            ..TrialRecord::provisioned()
        }
        .encode(),
    );
    let stores = [
        provisioned(1, 3),
        cleared(),
        garbage,
        bad_floor,
        no_gen,
        unwritten,
        squatted,
        future,
        zero_attempt,
    ];
    let mut n = 0;
    for (i, base) in stores.iter().enumerate() {
        for uefi in 0..=MAX_VERSION {
            let mut s = base.clone();
            s.uefi = uefi;
            for a in &options {
                for b in &options {
                    let disk = [*a, *b];
                    for dev in [
                        Dev::Ready,
                        Dev::Absent,
                        Dev::Fault,
                        Dev::PcrDirty,
                        Dev::ExtendBroken,
                    ] {
                        let plan = boot(&s, &disk, 0, dev);
                        check_plan(&s, &disk, dev, &plan);
                        if i > 0 && matches!(dev, Dev::Ready | Dev::ExtendBroken) {
                            // Every corrupted anchor is ANCHOR_LOST (or BEHIND).
                            assert!(plan.path.iter().any(|p| *p == State::Degraded));
                            assert!(plan.writes.is_empty());
                        }
                        n += 1;
                    }
                }
            }
        }
    }
    eprintln!("fault classes: {n} boots");
}

// ---------------------------------------------------------------------------
// 2. Reachability: BFS over every boot outcome, power cut and T1/T6 action.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct World {
    store: Store,
    disk: Disk,
    hint: u8,
    /// Ghost: trial executions per version (I3, I4 across alternation).
    execs: [u8; MAX_VERSION as usize + 1],
    /// Ghost: TPM cleared (I3/I4 are claimed only while the anchor is intact).
    tpm_lost: bool,
}

/// Crash-only continuation (no adversary, kernel always crashes): must reach a
/// boot without writes within a few boots, every boot safe.
fn converges(store: &Store, disk: &Disk, hint: u8) {
    let mut s = store.clone();
    for _ in 0..8 {
        let plan = boot(&s, disk, hint, Dev::Ready);
        let post = check_plan(&s, disk, Dev::Ready, &plan);
        if plan.writes.is_empty() {
            assert!(
                trial_binding(&plan).is_none(),
                "a trial always writes: {s:?} {disk:?}"
            );
            return;
        }
        s = post;
    }
    panic!("no convergence from {store:?} {disk:?}");
}

/// The next boot after a cut after `k` writes of `plan`.
fn check_resume(cut: &Store, disk: &Disk, hint: u8, plan: &Plan, k: usize) {
    let ws = writes(plan);
    let done = &ws[..k];
    let next = boot(cut, disk, hint, Dev::Ready);
    // A cut between F1 and F2 resumes FINALIZE; it never ends DEGRADED.
    let f1 = done.iter().any(|w| matches!(w, Write::UefiFloor(_)))
        && plan.path.iter().any(|p| *p == State::Finalize);
    let f2 = done.iter().any(|w| matches!(w, Write::Floor(_)));
    if f1 && !f2 {
        assert!(
            next.path.iter().any(|p| *p == State::Finalize),
            "F1→F2 resume: {cut:?} {next:?}"
        );
        assert!(!next.path.iter().any(|p| *p == State::Degraded));
    }
    // A durable T2 counts: the next boot continues with attempt n+1 or exhausts.
    if let Some(Write::Trial(t)) = done.last() {
        if t.state == TrialState::Pending {
            match trial_binding(&next) {
                Some(b) => assert_eq!(
                    (b.version, b.attempt),
                    (t.version, t.attempts + 1),
                    "{next:?}"
                ),
                None => assert!(
                    t.attempts == MAX_ATTEMPTS && next.path.iter().any(|p| *p == State::Exhausted),
                    "{next:?}"
                ),
            }
        }
    }
}

#[test]
fn reachable_states_crash_consistency_and_attempt_bounds() {
    let mut seen: HashSet<World> = HashSet::new();
    let mut queue = VecDeque::new();
    // Provisioned at floor 1 with v1 in slot A; the adversary installs the rest.
    let start = World {
        store: provisioned(1, 1),
        disk: [Some((1, 0)), None],
        hint: 0,
        execs: [0; MAX_VERSION as usize + 1],
        tpm_lost: false,
    };
    seen.insert(start.clone());
    queue.push_back(start);
    let slot_options: Vec<Option<(u64, u8)>> = {
        let mut v = vec![None];
        v.extend((0..=MAX_VERSION).map(|x| Some((x, 0))));
        v.push(Some((3, 1)));
        v
    };
    let (mut boots, mut cuts, mut trials, mut exhausted, mut finalized) = (0, 0, 0, 0, 0);
    let push = |w: World, seen: &mut HashSet<World>, queue: &mut VecDeque<World>| {
        if w.store.generation.is_none_or(|g| g <= GEN_CAP) && seen.insert(w.clone()) {
            queue.push_back(w);
            true
        } else {
            false
        }
    };
    while let Some(w) = queue.pop_front() {
        if let Some(f) = w.store.floor_rec() {
            assert!(w.store.uefi >= f.floor, "UEFI copy behind FLOOR: {w:?}");
        }
        if seen.len() > 3_000_000 {
            panic!("state space larger than expected");
        }
        // Boots under every TPM condition.
        // Fault and PcrDirty take the same no-write path as Absent (covered over
        // arbitrary states by every_tpm_fault_class_degrades_without_writes).
        for dev in [Dev::Ready, Dev::Absent, Dev::ExtendBroken] {
            let plan = boot(&w.store, &w.disk, w.hint, dev);
            let post = check_plan(&w.store, &w.disk, dev, &plan);
            boots += 1;
            if plan.path.iter().any(|p| *p == State::Exhausted) {
                exhausted += 1;
            }
            if plan.path.iter().any(|p| *p == State::Finalize) {
                finalized += 1;
            }
            let ws = writes(&plan);
            // Power loss after k writes, k < all (k == all is the halt/no-kernel case below).
            for k in 0..ws.len() {
                let mut s = w.store.clone();
                for x in &ws[..k] {
                    s.apply(x);
                }
                cuts += 1;
                check_resume(&s, &w.disk, w.hint, &plan, k);
                let next = World {
                    store: s,
                    ..w.clone()
                };
                if push(next.clone(), &mut seen, &mut queue) {
                    converges(&next.store, &next.disk, next.hint);
                }
            }
            let mut after = w.clone();
            after.store = post;
            match (plan.outcome, trial_binding(&plan)) {
                (Outcome::Handover(_), Some(b)) => {
                    trials += 1;
                    let n = &mut after.execs[b.version as usize];
                    *n += 1;
                    if !w.tpm_lost {
                        assert!(*n <= MAX_ATTEMPTS, "I3/I4 ghost: {w:?} {plan:?}");
                    }
                    push(after.clone(), &mut seen, &mut queue); // kernel crash/unhealthy
                    after.store.confirm = Some(b.confirmation().encode());
                    push(after, &mut seen, &mut queue); // healthy confirm
                }
                _ => {
                    push(after, &mut seen, &mut queue);
                }
            }
        }
        // T1: disk rewrites, swaps, replays; header hint.
        for i in 0..2 {
            for o in &slot_options {
                let mut n = w.clone();
                n.disk[i] = *o;
                push(n, &mut seen, &mut queue);
            }
        }
        let mut swapped = w.clone();
        swapped.disk.swap(0, 1);
        push(swapped, &mut seen, &mut queue);
        // T6: TPM cleared or replaced (UEFI copy kept; T7 excluded).
        if !w.tpm_lost {
            let mut lost = w.clone();
            lost.store = Store {
                uefi: w.store.uefi,
                ..cleared()
            };
            lost.tpm_lost = true;
            push(lost, &mut seen, &mut queue);
        }
    }
    eprintln!(
        "reachable: {} worlds, {boots} boots, {cuts} power cuts, {trials} trial handovers, \
         {exhausted} exhaustions, {finalized} finalizations",
        seen.len()
    );
    assert!(exhausted > 0 && finalized > 0 && cuts > 0);
}

// ---------------------------------------------------------------------------
// 3. Scenarios (owner categories, spec section 11 cases).

struct Machine {
    store: Store,
    disk: Disk,
    hint: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum Kernel {
    Crash,
    Healthy,
}

impl Machine {
    fn new(floor: u64) -> Self {
        Self {
            store: provisioned(floor, 1),
            disk: [Some((floor, 0)), None],
            hint: 0,
        }
    }
    fn boot_with(&mut self, dev: Dev, kernel: Kernel) -> (Plan, Vec<String>) {
        let plan = boot(&self.store, &self.disk, self.hint, dev);
        self.store = check_plan(&self.store, &self.disk, dev, &plan);
        if kernel == Kernel::Healthy {
            if let (Outcome::Handover(_), Some(b)) = (plan.outcome, trial_binding(&plan)) {
                self.store.confirm = Some(b.confirmation().encode());
            }
        }
        let markers = plan.markers.iter().map(ToString::to_string).collect();
        (plan, markers)
    }
    fn boot(&mut self, kernel: Kernel) -> Vec<String> {
        self.boot_with(Dev::Ready, kernel).1
    }
}

fn has(markers: &[String], m: &str) -> bool {
    markers.iter().any(|x| x == m)
}

#[test]
fn healthy_confirm_raises_floor_on_next_boot_and_old_version_is_refused() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    let b1 = m.boot(Kernel::Healthy);
    assert!(has(&b1, "BOOT:TRIAL:START:2:B:2:1"), "{b1:?}");
    assert!(has(&b1, "BOOT:FLOOR:1"));
    assert_eq!(
        m.store.effective(),
        1,
        "no raise at selection time (REL-05)"
    );
    let b2 = m.boot(Kernel::Crash);
    assert!(has(&b2, "BOOT:CONFIRM:FINALIZED:2:2"), "{b2:?}");
    assert!(has(&b2, "BOOT:FLOOR:2") && has(&b2, "BOOT:SLOT:B:VERSION:2"));
    assert_eq!((m.store.uefi, m.store.floor_rec().unwrap().floor), (2, 2));
    // Old release: the existing ADR-049 selection now refuses it as a downgrade.
    let v1 = update_core::Manifest {
        target: update_core::TARGET_KERNEL,
        image_length: 1,
        version: 1,
        min_version: 0,
        image_sha256: [0; 32],
        ed25519_key_id: [0; 8],
        mldsa_key_id: [0; 8],
    };
    let (choice, reasons) = update_core::choose(
        &update_core::Header {
            active: 0,
            generation: 0,
        },
        [Ok(v1), Err(update_core::Reject::Read)],
        m.store.uefi,
    );
    assert!(choice.is_none());
    assert_eq!(reasons[0], Some(update_core::Reject::Downgrade));
    // C14 disk replay: the pre-update disk image alone no longer boots.
    m.disk = [Some((1, 0)), None];
    let b3 = m.boot(Kernel::Crash);
    assert!(
        has(&b3, "BOOT:FAIL:NO_VALID_SLOT") && has(&b3, "BOOT:FLOOR:2"),
        "{b3:?}"
    );
    // C15: old v1 next to v2 is ignored, v2 boots.
    m.disk = [Some((1, 0)), Some((2, 0))];
    let b4 = m.boot(Kernel::Crash);
    assert!(has(&b4, "BOOT:SLOT:B:VERSION:2") && has(&b4, "BOOT:MODE:KNOWN_GOOD"));
}

#[test]
fn kernel_crash_three_times_falls_back_to_known_good() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    for (n, g) in [(1, 2), (2, 3), (3, 4)] {
        let b = m.boot(Kernel::Crash);
        assert!(has(&b, &format!("BOOT:TRIAL:START:{g}:B:2:{n}")), "{b:?}");
    }
    let b4 = m.boot(Kernel::Crash);
    assert!(has(&b4, "BOOT:TRIAL:EXHAUSTED:2"), "{b4:?}");
    assert!(has(&b4, "BOOT:FLOOR:1") && has(&b4, "BOOT:SLOT:A:VERSION:1"));
    // C4: header hint pointing at the rejected slot changes nothing.
    m.hint = 1;
    let b5 = m.boot(Kernel::Healthy);
    assert!(has(&b5, "BOOT:TRIAL:REFUSED:2") && has(&b5, "BOOT:SLOT:A:VERSION:1"));
    assert!(!b5.iter().any(|x| x.starts_with("BOOT:TRIAL:START")));
    // The same version in the other slot (T1 swap) is still refused.
    m.disk.swap(0, 1);
    let b6 = m.boot(Kernel::Crash);
    assert!(has(&b6, "BOOT:TRIAL:REFUSED:2") && has(&b6, "BOOT:SLOT:B:VERSION:1"));
}

#[test]
fn extend_failure_halts_but_counts_the_attempt() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    for _ in 0..3 {
        let (plan, b) = m.boot_with(Dev::ExtendBroken, Kernel::Healthy);
        assert_eq!(plan.outcome, Outcome::Halt(Halt::TpmExtend));
        assert!(has(&b, "BOOT:FAIL:TPM_EXTEND"));
    }
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:TRIAL:EXHAUSTED:2") && has(&b, "BOOT:SLOT:A:VERSION:1"));
}

#[test]
fn forged_confirmations_are_inert() {
    let forgeries: [fn(&mut ConfirmRecord); 6] = [
        |c| c.slot ^= 1,
        |c| c.version += 1,
        |c| c.digest[31] ^= 1,
        |c| c.verdict = 0,
        |c| c.health_list = 9,
        |c| c.generation += 1,
    ];
    for (i, forge) in forgeries.iter().enumerate() {
        let mut m = Machine::new(1);
        m.disk[1] = Some((2, 0));
        let (plan, _) = m.boot_with(Dev::Ready, Kernel::Crash);
        let mut c = trial_binding(&plan).unwrap().confirmation();
        forge(&mut c);
        m.store.confirm = Some(c.encode());
        let b = m.boot(Kernel::Crash);
        assert!(
            b.iter().any(|x| x.starts_with("BOOT:CONFIRM:REJECTED:")),
            "{i}: {b:?}"
        );
        assert!(has(&b, "BOOT:TRIAL:START:3:B:2:2"), "{i}: {b:?}");
        assert_eq!(m.store.effective(), 1);
    }
    // Garbage bytes in CONFIRM: malformed, ignored, boot continues.
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Crash);
    m.store.confirm = Some([0x41; RECORD_LEN]);
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:CONFIRM:REJECTED:MALFORMED"), "{b:?}");
}

#[test]
fn replayed_confirmation_does_not_confirm_a_new_candidate() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Healthy);
    m.boot(Kernel::Crash); // finalizes v2
    m.disk[0] = Some((3, 0));
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:TRIAL:START:3:A:3:1"), "{b:?}");
    // CONFIRM for generation 2 is still in NV: inert for generation 3 (C10).
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:CONFIRM:REJECTED:GENERATION"), "{b:?}");
    assert_eq!(m.store.effective(), 2);
    // Copying the old record with the new generation still fails the binding.
    let mut c = ConfirmRecord::decode(&m.store.confirm.unwrap()).unwrap();
    c.generation = m.store.trial().unwrap().generation;
    m.store.confirm = Some(c.encode());
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:CONFIRM:REJECTED:BINDING"), "{b:?}");
    assert_eq!(m.store.effective(), 2);
}

#[test]
fn swapped_slots_reject_the_confirmation_and_carry_attempts() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Healthy);
    m.disk.swap(0, 1); // C11
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:CONFIRM:REJECTED:BINDING"), "{b:?}");
    assert!(has(&b, "BOOT:TRIAL:START:3:A:2:2"), "{b:?}");
    assert_eq!(m.store.effective(), 1);
}

#[test]
fn alternating_candidates_never_reset_attempts() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Crash);
    m.boot(Kernel::Crash);
    m.disk[1] = Some((3, 0)); // C13
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:TRIAL:ABANDONED:2") && has(&b, "BOOT:TRIAL:START:4:B:3:1"));
    m.disk[1] = Some((2, 0));
    let b = m.boot(Kernel::Crash);
    // v2 is refused; pending v3 is absent from disk, so it stays pending.
    assert!(
        has(&b, "BOOT:TRIAL:REFUSED:2") && has(&b, "BOOT:SLOT:A:VERSION:1"),
        "{b:?}"
    );
    assert_eq!(m.store.trial().unwrap().state, TrialState::Pending);
    // Same version, different signed manifest: a different candidate, so v3
    // is abandoned and both manifests of version 3 are refused from now on.
    m.disk[1] = Some((3, 1));
    let b = m.boot(Kernel::Crash);
    assert!(
        has(&b, "BOOT:TRIAL:ABANDONED:3") && has(&b, "BOOT:TRIAL:REFUSED:3"),
        "{b:?}"
    );
    assert!(has(&b, "BOOT:SLOT:A:VERSION:1"));
}

#[test]
fn ambiguous_equal_versions_are_not_trialled() {
    let mut m = Machine::new(1);
    m.disk = [Some((3, 0)), Some((3, 1))];
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:FAIL:NO_VALID_SLOT"), "{b:?}");
    assert_eq!(m.store, provisioned(1, 1));
}

#[test]
fn downgrade_candidates_never_run() {
    let mut m = Machine::new(2);
    m.disk[1] = Some((1, 0));
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:SLOT:A:VERSION:2") && has(&b, "BOOT:MODE:KNOWN_GOOD"));
    m.disk[0] = None;
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:FAIL:NO_VALID_SLOT"));
    assert_eq!(m.store.effective(), 2);
}

#[test]
fn power_cut_between_f1_and_f2_resumes_finalization() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Healthy);
    let plan = boot(&m.store, &m.disk, 0, Dev::Ready);
    let ws = writes(&plan);
    assert!(matches!(ws[0], Write::UefiFloor(2)) && matches!(ws[1], Write::Floor(_)));
    m.store.apply(&ws[0]);
    let b = m.boot(Kernel::Crash);
    assert!(has(&b, "BOOT:CONFIRM:FINALIZED:2:2") && !has(&b, "BOOT:MODE:DEGRADED"));
    assert_eq!(m.store.floor_rec().unwrap().floor, 2);
    // TPM lost in that window: the raise survives in UEFI, boot degrades.
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    m.boot(Kernel::Healthy);
    m.store.apply(&Write::UefiFloor(2));
    m.store = Store {
        uefi: 2,
        ..cleared()
    };
    let b = m.boot(Kernel::Crash);
    assert!(
        has(&b, "BOOT:TPM:ANCHOR_LOST") && has(&b, "BOOT:FLOOR:2"),
        "{b:?}"
    );
    assert!(has(&b, "BOOT:SLOT:B:VERSION:2"));
}

#[test]
fn tpm_missing_faulty_cleared_replaced_or_rolled_back() {
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    let snapshot = m.store.clone();
    m.boot(Kernel::Healthy);
    m.boot(Kernel::Crash); // floor 2
    for (dev, marker) in [
        (Dev::Absent, "BOOT:TPM:ABSENT"),
        (Dev::Fault, "BOOT:TPM:FAULT"),
        (Dev::PcrDirty, "BOOT:TPM:FAULT:PCR_DIRTY"),
    ] {
        let (plan, b) = m.boot_with(dev, Kernel::Healthy);
        assert!(has(&b, marker) && has(&b, "BOOT:MODE:DEGRADED") && has(&b, "BOOT:FLOOR:2"));
        assert!(plan.writes.is_empty());
    }
    // C18 cleared, C21 squatted/replaced: anchor lost, UEFI floor kept.
    let kept = m.store.clone();
    m.store = Store {
        uefi: 2,
        ..cleared()
    };
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:TPM:ANCHOR_LOST") && has(&b, "BOOT:FLOOR:2"));
    m.store = kept.clone();
    m.store.public_ok = false;
    let b = m.boot(Kernel::Healthy);
    assert!(has(&b, "BOOT:TPM:ANCHOR_LOST"));
    // C19 rolled back to the pre-update TPM state with the UEFI copy at 2.
    m.store = Store {
        uefi: 2,
        ..snapshot
    };
    m.disk[0] = Some((3, 0));
    let b = m.boot(Kernel::Healthy);
    assert!(
        has(&b, "BOOT:TPM:ANCHOR_BEHIND") && has(&b, "BOOT:FLOOR:2"),
        "{b:?}"
    );
    assert!(has(&b, "BOOT:SLOT:B:VERSION:2") && !b.iter().any(|x| x.contains("TRIAL")));
    // Degraded with only a rejected version above the floor: never booted.
    let mut m = Machine::new(1);
    m.disk[1] = Some((2, 0));
    for _ in 0..4 {
        m.boot(Kernel::Crash);
    }
    m.disk[0] = None;
    m.store.public_ok = false;
    let (plan, b) = m.boot_with(Dev::Ready, Kernel::Crash);
    // Spec 10.2: no slot at the UEFI floor, so halt rather than boot the
    // rejected v2 as known-good.
    assert!(
        has(&b, "BOOT:MODE:DEGRADED")
            && has(&b, "BOOT:FAIL:NO_VALID_SLOT")
            && !b.iter().any(|x| x.starts_with("BOOT:SLOT:")),
        "{b:?}"
    );
    assert!(trial_binding(&plan).is_none() && plan.writes.is_empty());
}

#[test]
fn marker_strings_match_the_spec() {
    let cases = [
        (
            Marker::TrialStart {
                generation: 7,
                slot: 1,
                version: 2,
                attempt: 1,
            },
            "BOOT:TRIAL:START:7:B:2:1",
        ),
        (
            Marker::ConfirmFinalized {
                generation: 7,
                version: 2,
            },
            "BOOT:CONFIRM:FINALIZED:7:2",
        ),
        (
            Marker::Slot {
                slot: 0,
                version: 1,
            },
            "BOOT:SLOT:A:VERSION:1",
        ),
        (
            Marker::Tpm(TpmClass::AnchorBehind),
            "BOOT:TPM:ANCHOR_BEHIND",
        ),
        (Marker::FailTpmExtend, "BOOT:FAIL:TPM_EXTEND"),
        (Marker::TrialRefused(2), "BOOT:TRIAL:REFUSED:2"),
    ];
    for (m, s) in cases {
        assert_eq!(m.to_string(), s);
    }
}
