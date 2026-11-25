//! M1 protected-slot HOST MODEL (B, PR #524): the cold-freshness extend chain
//! against N's attack sequences in `services/net-broker/src/durable.rs`
//! (N branch `claude/n03-n04-adr-20261125` @ `fad2a39a`). This is not evidence
//! of TPM behaviour: `Tpm` below encodes U21–U24 of COLD-FRESHNESS-ANCHOR.md
//! (U21 and the hierarchy-auth rules observed on swtpm 0.10.1 in
//! docs/verification/b03-rel05-hauth-probe-20261125; native S5 still open).
//! 5A stays in force.
//!
//! Mapping to N's tests: there is no `Pending` (one extend is the only anchor
//! write), and N's `Rollback`/`Fork`/`Ahead` collapse into one `Stale`
//! denial, because the chain value cannot say which of them applies.
use std::collections::BTreeMap;
use update_core::anchor::*;
use update_core::trial_tpm::anchor_domain;

const DOM: u8 = 0;

fn hex(d: &Digest) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TpmError {
    Handle,
    Uninitialized,
    Auth,
    Exists,
    Policy,
}

/// Model TPM: the anchor extend indices, the hierarchy authorisations that
/// guard undefining them, `HAUTH`, PCR 13 and one loader-only record (`mark`,
/// standing for the policy-written REL-05 records of section 6.1: writable
/// only at PCR 13 = 0³²). `None` = defined but never extended.
struct Tpm {
    nv: BTreeMap<u32, Option<Digest>>,
    owner_auth: Vec<u8>,
    lockout_auth: Vec<u8>,
    /// `None`: OVMF set a random platformAuth the OS never learns (U8, U23).
    platform_auth: Option<Vec<u8>>,
    /// PCR 13 still all-zero: the loader phase of this boot.
    pcr13_zero: bool,
    hauth: Option<[u8; 64]>,
    hauth_writes_with_owner_set: u32,
    mark: Option<Digest>,
}

impl Tpm {
    /// Fresh TPM: empty hierarchy authValues, loader phase.
    fn new() -> Self {
        Tpm {
            nv: BTreeMap::new(),
            owner_auth: vec![],
            lockout_auth: vec![],
            platform_auth: None,
            pcr13_zero: true,
            hauth: None,
            hauth_writes_with_owner_set: 0,
            mark: None,
        }
    }
    fn reboot(&mut self) {
        self.pcr13_zero = true;
    }
    /// Loader extends PCR 13 before handover: the kernel phase begins.
    fn close(&mut self) {
        self.pcr13_zero = false;
    }
    fn write_hauth(&mut self, owner: &[u8], v: [u8; 64]) -> Result<(), TpmError> {
        if owner != self.owner_auth {
            return Err(TpmError::Auth);
        }
        if !self.owner_auth.is_empty() {
            self.hauth_writes_with_owner_set += 1;
        }
        self.hauth = Some(v);
        Ok(())
    }
    fn undefine_hauth(&mut self, owner: &[u8]) -> Result<(), TpmError> {
        if owner != self.owner_auth {
            return Err(TpmError::Auth);
        }
        self.hauth.take().map(|_| ()).ok_or(TpmError::Handle)
    }
    /// `PolicyPCR(13 = 0³²) + CC_NV_Read` (H2 on swtpm).
    fn read_hauth(&self) -> Result<[u8; 64], TpmError> {
        if !self.pcr13_zero {
            return Err(TpmError::Policy);
        }
        self.hauth.ok_or(TpmError::Handle)
    }
    fn change_auth(&mut self, lockout: bool, old: &[u8], new: &[u8]) -> Result<(), TpmError> {
        let slot = if lockout {
            &mut self.lockout_auth
        } else {
            &mut self.owner_auth
        };
        if old != slot.as_slice() {
            return Err(TpmError::Auth);
        }
        *slot = new.to_vec();
        Ok(())
    }
    /// `TPM2_Clear` by lockout or platform: owner NV, `HAUTH`, the loader
    /// records and both authValues are gone (probe U24: platform clear).
    fn clear(&mut self, platform: bool, auth: &[u8]) -> Result<(), TpmError> {
        let ok = if platform {
            self.platform_auth.as_deref() == Some(auth)
        } else {
            auth == self.lockout_auth
        };
        if !ok {
            return Err(TpmError::Auth);
        }
        let platform_auth = self.platform_auth.take();
        let pcr13_zero = self.pcr13_zero;
        *self = Tpm::new();
        self.platform_auth = platform_auth;
        self.pcr13_zero = pcr13_zero;
        Ok(())
    }
    fn write_mark(&mut self, v: Digest) -> Result<(), TpmError> {
        if !self.pcr13_zero {
            return Err(TpmError::Policy);
        }
        self.mark = Some(v);
        Ok(())
    }
    fn define(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError> {
        if auth != self.owner_auth {
            return Err(TpmError::Auth);
        }
        if self.nv.contains_key(&h) {
            return Err(TpmError::Exists);
        }
        self.nv.insert(h, None);
        Ok(())
    }
    fn undefine_owner(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError> {
        if auth != self.owner_auth {
            return Err(TpmError::Auth);
        }
        self.nv.remove(&h).map(|_| ()).ok_or(TpmError::Handle)
    }
    /// Platform may undefine an owner-created index (U24).
    fn undefine_platform(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError> {
        if self.platform_auth.as_deref() != Some(auth) {
            return Err(TpmError::Auth);
        }
        self.nv.remove(&h).map(|_| ()).ok_or(TpmError::Handle)
    }
    /// `AUTHWRITE` with empty authValue: anyone who can submit commands (T3).
    /// The first extend starts from zeros (U21); one extend is atomic (U22).
    fn nv_extend(&mut self, h: u32, data: &Digest) -> Result<(), TpmError> {
        let v = self.nv.get_mut(&h).ok_or(TpmError::Handle)?;
        *v = Some(chain_step(&v.unwrap_or([0; 32]), data));
        Ok(())
    }
    fn nv_read(&self, h: u32) -> Result<Digest, TpmError> {
        self.nv
            .get(&h)
            .ok_or(TpmError::Handle)?
            .ok_or(TpmError::Uninitialized)
    }
}

fn handle() -> u32 {
    anchor_domain(DOM).unwrap().handle
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnchorError {
    Unavailable,
    Corrupt,
    Conflict,
    RateLimited,
}

/// Kernel anchor service for one domain (serialised per domain, C87).
struct Anchor {
    tpm: Tpm,
    instance: Digest,
    budget: u32,
    lose_next_response: bool,
}

impl Anchor {
    fn read(&self) -> Result<Digest, AnchorError> {
        self.tpm
            .nv_read(handle())
            .map_err(|_| AnchorError::Unavailable)
    }
    /// Compare-and-extend: the kernel computes `E` from the capability's
    /// domain, never from the caller.
    fn extend(&mut self, expected: &Digest, store: &Digest) -> Result<(), AnchorError> {
        if self.read()? != *expected {
            return Err(AnchorError::Conflict);
        }
        if self.budget == 0 {
            return Err(AnchorError::RateLimited);
        }
        self.budget -= 1;
        let e = extend_event(DOM, store);
        self.tpm
            .nv_extend(handle(), &e)
            .map_err(|_| AnchorError::Unavailable)?;
        if std::mem::take(&mut self.lose_next_response) {
            return Err(AnchorError::Unavailable);
        }
        if self.read()? != chain_step(expected, &e) {
            return Err(AnchorError::Corrupt);
        }
        Ok(())
    }
}

/// TEST provisioning: define, then one genesis extend. Re-runnable after a cut
/// between the two steps.
fn provision(tpm: &mut Tpm, instance: &Digest) {
    match tpm.define(&tpm.owner_auth.clone(), handle()) {
        Ok(()) | Err(TpmError::Exists) => {}
        Err(e) => panic!("{e:?}"),
    }
    if tpm.nv_read(handle()) == Err(TpmError::Uninitialized) {
        tpm.nv_extend(handle(), &genesis_event(DOM, instance))
            .unwrap();
    }
}

const I0: Digest = [0xa0; 32];
const I1: Digest = [0xa1; 32];

fn anchor() -> Anchor {
    let mut tpm = Tpm::new();
    provision(&mut tpm, &I0);
    Anchor {
        tpm,
        instance: I0,
        budget: 16,
        lose_next_response: false,
    }
}

/// An authenticated store state as N04 keeps it: `prev` is the chain value it
/// was committed on top of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Store {
    instance: Digest,
    prev: Digest,
    digest: Digest,
}

impl Store {
    fn anchored(&self) -> Digest {
        chain_after(DOM, &self.prev, &self.digest)
    }
    fn child(&self, tag: u8) -> Store {
        Store {
            prev: self.anchored(),
            digest: [tag; 32],
            ..*self
        }
    }
}

fn first(instance: Digest, tag: u8) -> Store {
    Store {
        instance,
        prev: genesis_chain(DOM, &instance),
        digest: [tag; 32],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Startup {
    Accept,
    RolledForward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Denied {
    Anchor(AnchorError),
    Foreign,
    Stale,
}

/// N's agreed consumer rules (reference only; N owns the implementation).
fn startup(a: &mut Anchor, s: &Store) -> Result<Startup, Denied> {
    if a.instance != s.instance {
        return Err(Denied::Foreign);
    }
    let chain = a.read().map_err(Denied::Anchor)?;
    if chain == s.anchored() {
        return Ok(Startup::Accept);
    }
    if chain != s.prev {
        return Err(Denied::Stale);
    }
    a.extend(&s.prev, &s.digest).map_err(Denied::Anchor)?;
    Ok(Startup::RolledForward)
}

fn acknowledge(a: &mut Anchor, next: &Store) -> Result<(), Denied> {
    if a.instance != next.instance {
        return Err(Denied::Foreign);
    }
    a.extend(&next.prev, &next.digest).map_err(Denied::Anchor)
}

/// Anchor with store `A` acknowledged on top of genesis.
fn at_a() -> (Anchor, Store) {
    let mut a = anchor();
    let s_a = first(I0, 1);
    assert_eq!(acknowledge(&mut a, &s_a), Ok(()));
    (a, s_a)
}

/// N: `hostile_slot_write_cannot_revive_a_stale_store_at_the_same_generation`
/// and `protected_slot_refuses_the_single_boot_m1_attack`.
#[test]
fn hostile_extend_cannot_revive_a_stale_store_at_the_same_position() {
    let (mut a, s_a) = at_a();
    let x = s_a.child(3); // committed to disk, crash before the extend
                          // Disk attacker replays A inside the rule (c) window: accepted.
    assert_eq!(startup(&mut a, &s_a), Ok(Startup::Accept));
    let y = s_a.child(2); // carries an acknowledged revocation
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
    assert_eq!(startup(&mut a, &x), Err(Denied::Stale));
    assert_eq!(startup(&mut a, &y), Ok(Startup::Accept));
    // T3 can only extend forward; no input it holds leads back to X.
    for data in [
        extend_event(DOM, &x.digest),
        extend_event(DOM, &s_a.digest),
        x.digest,
        x.prev,
        s_a.anchored(),
    ] {
        let chain = chain_step(&a.read().unwrap(), &data);
        assert_ne!(chain, x.anchored());
        assert_ne!(chain, x.prev);
    }
    a.tpm
        .nv_extend(handle(), &extend_event(DOM, &x.digest))
        .unwrap();
    assert_eq!(startup(&mut a, &x), Err(Denied::Stale));
    // A stray extend is permanent denial of every store until genesis
    // recovery: stronger than the old rewritable slot, stated in the contract.
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
}

/// N: `protected_slot_refuses_both_multi_boot_attacks` and the two
/// `reseal_multi_boot_*` residuals, which need no reseal under the chain.
#[test]
fn crashed_sibling_cannot_displace_a_rolled_forward_store() {
    let (mut a, s_a) = at_a();
    let x = s_a.child(3);
    let z = s_a.child(4);
    assert_eq!(startup(&mut a, &x), Ok(Startup::RolledForward));
    assert_eq!(startup(&mut a, &z), Err(Denied::Stale));
    assert_eq!(startup(&mut a, &s_a), Err(Denied::Stale));
    let y = x.child(2);
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
    for stale in [s_a, x, z, z.child(5)] {
        assert_eq!(startup(&mut a, &stale), Err(Denied::Stale));
    }
    assert_eq!(startup(&mut a, &y), Ok(Startup::Accept));
}

/// N: `protected_slot_interrupted_commit_completes_only_once`. Every write
/// prefix of one commit (A1 = store write, then the extend).
#[test]
fn every_crash_prefix_of_a_commit() {
    // Cut before or during A1 (torn A1 rejected by N's authenticator): A.
    let (mut a, s_a) = at_a();
    assert_eq!(startup(&mut a, &s_a), Ok(Startup::Accept));
    // Cut after A1: X rolls forward exactly once.
    let x = s_a.child(3);
    assert_eq!(startup(&mut a, &x), Ok(Startup::RolledForward));
    assert_eq!(startup(&mut a, &x), Ok(Startup::Accept));
    assert_eq!(a.extend(&x.prev, &x.digest), Err(AnchorError::Conflict));
    // Cut after the extend: accept.
    let (mut a, s_a) = at_a();
    let x = s_a.child(3);
    assert_eq!(acknowledge(&mut a, &x), Ok(()));
    assert_eq!(startup(&mut a, &x), Ok(Startup::Accept));
}

#[test]
fn lost_response_is_settled_by_the_next_read_never_by_a_blind_retry() {
    let (mut a, s_a) = at_a();
    let y = s_a.child(2);
    a.lose_next_response = true;
    assert_eq!(
        acknowledge(&mut a, &y),
        Err(Denied::Anchor(AnchorError::Unavailable))
    );
    // The compare step turns a retry into Conflict instead of a second extend.
    assert_eq!(
        acknowledge(&mut a, &y),
        Err(Denied::Anchor(AnchorError::Conflict))
    );
    assert_eq!(startup(&mut a, &y), Ok(Startup::Accept));
    // Without the compare, a blind retry double-extends: permanent denial.
    a.tpm
        .nv_extend(handle(), &extend_event(DOM, &y.digest))
        .unwrap();
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
}

#[test]
fn spent_budget_writes_nothing() {
    let (mut a, s_a) = at_a();
    let before = a.read().unwrap();
    a.budget = 0;
    let y = s_a.child(2);
    assert_eq!(
        acknowledge(&mut a, &y),
        Err(Denied::Anchor(AnchorError::RateLimited))
    );
    assert_eq!(a.read(), Ok(before));
    a.budget = 1; // next boot
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
}

#[test]
fn genesis_only_through_provisioning() {
    // Missing index, or defined but never extended: Unavailable, never genesis.
    let mut a = anchor();
    a.tpm.nv.clear();
    let s = first(I0, 1);
    assert_eq!(
        startup(&mut a, &s),
        Err(Denied::Anchor(AnchorError::Unavailable))
    );
    a.tpm.define(&[], handle()).unwrap();
    assert_eq!(
        startup(&mut a, &s),
        Err(Denied::Anchor(AnchorError::Unavailable))
    );
    // Cut between define and the genesis extend: provisioning re-runs.
    provision(&mut a.tpm, &I0);
    assert_eq!(a.read(), Ok(genesis_chain(DOM, &I0)));
    provision(&mut a.tpm, &I0);
    assert_eq!(a.read(), Ok(genesis_chain(DOM, &I0)), "idempotent");
    assert_eq!(startup(&mut a, &s), Ok(Startup::RolledForward));
    // On a provisioned index no extend reproduces the genesis value.
    let g = genesis_event(DOM, &I0);
    assert_ne!(chain_step(&a.read().unwrap(), &g), genesis_chain(DOM, &I0));
}

/// Owner-authorised recovery on the same TPM: undefine, define, genesis.
fn regenesis(a: &mut Anchor, instance: Digest) {
    let auth = a.tpm.owner_auth.clone();
    a.tpm.undefine_owner(&auth, handle()).unwrap();
    provision(&mut a.tpm, &instance);
    a.instance = instance;
}

/// A stray extend kills the domain; same-TPM recovery needs a fresh instance.
#[test]
fn same_tpm_regenesis_after_a_stray_extend_needs_a_fresh_instance() {
    let (mut a, s_a) = at_a();
    let y = s_a.child(2);
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
    a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap();
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
    // Provisioning alone does not revive a written index.
    provision(&mut a.tpm, &I0);
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
    // Fresh instance (new nonce from the signed recovery record): old stores
    // are Foreign, the new genesis store rolls forward.
    regenesis(&mut a, I1);
    for old in [s_a, y] {
        assert_eq!(startup(&mut a, &old), Err(Denied::Foreign));
    }
    let g = first(I1, 7);
    assert_eq!(startup(&mut a, &g), Ok(Startup::RolledForward));
    // Reusing the old instance would let a disk attacker walk the old history
    // forward one boot at a time, choosing any branch (here the stale X):
    // every genesis therefore requires a fresh nonce.
    let mut a = at_a().0;
    let x = s_a.child(3);
    a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap();
    regenesis(&mut a, I0);
    assert_eq!(startup(&mut a, &s_a), Ok(Startup::RolledForward));
    assert_eq!(startup(&mut a, &x), Ok(Startup::RolledForward));
}

/// N: `replaced_tpm_rejects_the_store_until_authorised_genesis`.
#[test]
fn replaced_tpm_rejects_the_store_until_authorised_genesis() {
    let (_, s_a) = at_a();
    let store = s_a.child(2);
    let mut tpm = Tpm::new();
    provision(&mut tpm, &I1); // authorised recovery: new instance
    let mut a = Anchor {
        tpm,
        instance: I1,
        budget: 16,
        lose_next_response: false,
    };
    assert_eq!(startup(&mut a, &store), Err(Denied::Foreign));
    assert_eq!(acknowledge(&mut a, &store), Err(Denied::Foreign));
    assert_eq!(a.budget, 16, "no write");
    // Same store digest under the new instance still cannot ride the old chain.
    let moved = Store {
        instance: I1,
        ..store
    };
    assert_eq!(startup(&mut a, &moved), Err(Denied::Stale));
    let genesis = first(I1, 7);
    assert_eq!(startup(&mut a, &genesis), Ok(Startup::RolledForward));
    assert_eq!(startup(&mut a, &genesis), Ok(Startup::Accept));
}

/// Tries to revive X over acknowledged Y by undefine + redefine + replay of
/// the public extend events, then presents X.
fn undefine_redefine_replay(a: &mut Anchor, by_platform: bool) -> Result<Startup, Denied> {
    let s_a = first(I0, 1);
    let x = s_a.child(3);
    let h = handle();
    let gone = if by_platform {
        a.tpm.undefine_platform(&[], h)
    } else {
        a.tpm.undefine_owner(&[], h)
    };
    // Redefinition needs ownerAuth; a PLATFORMCREATE index would fail the
    // public-area check and read as Unavailable.
    if gone.is_ok() && a.tpm.define(&[], h).is_ok() {
        a.tpm.nv_extend(h, &genesis_event(DOM, &I0)).unwrap();
        a.tpm.nv_extend(h, &extend_event(DOM, &s_a.digest)).unwrap();
    }
    startup(a, &x)
}

fn after_m1_sequence() -> Anchor {
    let (mut a, s_a) = at_a();
    assert_eq!(startup(&mut a, &s_a), Ok(Startup::Accept));
    assert_eq!(acknowledge(&mut a, &s_a.child(2)), Ok(()));
    a
}

/// Retained M1 regression: the pre-HAUTH TEST profile (empty ownerAuth)
/// lets a runtime attacker revive X.
#[test]
fn undefine_redefine_replay_revives_with_empty_owner_auth() {
    let mut a = after_m1_sequence();
    assert_eq!(
        undefine_redefine_replay(&mut a, false),
        Ok(Startup::RolledForward)
    );
}

// ---- M1-HAUTH (owner decision, ADR-B03-REL05-TPM-20261125 amendment) ----

/// Boot record as the loader sees it after the accepted-serial check (U13c,
/// open): `fresh` = this record's nonce has never been anchored, so it may
/// authorise one genesis.
#[derive(Clone, Copy)]
struct Record {
    instance: Digest,
    fresh: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Halt {
    /// TPM cleared, replaced or never provisioned and no fresh record (C665).
    NeedsRecovery,
    /// ownerAuth set but `HAUTH` unreadable, or an authorisation failed.
    Corrupt,
}

/// Loader provisioning steps; `cut` stops after that many TPM writes.
const LOADER_STEPS: u32 = 7;

/// TEST-profile loader, re-run from the start after any cut. Order: remove
/// leftovers, `HAUTH`, ownerAuth, lockoutAuth, anchor genesis, mark, close.
/// Secrets come from loader entropy (`rng`), never from the record. The
/// provisioning state is read from the permanent flags (ownerAuthSet,
/// lockoutAuthSet), never by trying an authorisation.
fn loader(tpm: &mut Tpm, rec: Record, rng: &mut u8, cut: u32) -> Result<bool, Halt> {
    tpm.reboot();
    let mut writes = 0;
    let mut step = |tpm: &mut Tpm| -> bool {
        writes += 1;
        if writes > cut {
            tpm.reboot(); // power loss: nothing after this write
            return false;
        }
        true
    };
    let c = |r: Result<(), TpmError>| r.map_err(|_| Halt::Corrupt);
    if tpm.owner_auth.is_empty() {
        if !rec.fresh {
            return Err(Halt::NeedsRecovery);
        }
        // Leftovers of a cut or of a runtime attacker after a clear.
        let _ = tpm.undefine_owner(&[], handle());
        let _ = tpm.undefine_hauth(&[]);
        *rng = rng.wrapping_add(1);
        let secret = [*rng; 64];
        c(tpm.write_hauth(&[], secret))?;
        if !step(tpm) {
            return Ok(false);
        }
        c(tpm.change_auth(false, &[], &secret[..32]))?;
        if !step(tpm) {
            return Ok(false);
        }
    }
    let secret = tpm.read_hauth().map_err(|_| Halt::Corrupt)?;
    let (owner, lockout) = secret.split_at(32);
    if tpm.owner_auth != owner {
        return Err(Halt::Corrupt);
    }
    if tpm.lockout_auth.is_empty() {
        c(tpm.change_auth(true, &[], lockout))?;
        if !step(tpm) {
            return Ok(false);
        }
    }
    if tpm.mark != Some(rec.instance) {
        if !rec.fresh {
            return Err(Halt::NeedsRecovery);
        }
        let _ = tpm.undefine_owner(owner, handle());
        if !step(tpm) {
            return Ok(false);
        }
        c(tpm.define(owner, handle()))?;
        if !step(tpm) {
            return Ok(false);
        }
        c(tpm.nv_extend(handle(), &genesis_event(DOM, &rec.instance)))?;
        if !step(tpm) {
            return Ok(false);
        }
        c(tpm.write_mark(rec.instance))?;
        if !step(tpm) {
            return Ok(false);
        }
    }
    tpm.close(); // secret zeroised before the PCR 13 extend
    Ok(true)
}

fn boot(tpm: Tpm, rec: Record, rng: &mut u8) -> Result<Anchor, Halt> {
    let mut tpm = tpm;
    assert!(loader(&mut tpm, rec, rng, u32::MAX)?);
    Ok(Anchor {
        tpm,
        instance: rec.instance,
        budget: 16,
        lose_next_response: false,
    })
}

const ENROLL: Record = Record {
    instance: I0,
    fresh: true,
};
const SAME: Record = Record {
    instance: I0,
    fresh: false,
};
const RECOVERY: Record = Record {
    instance: I1,
    fresh: true,
};

/// Enrolled machine after the M1 sequence: A, then Y acknowledged; X is the
/// stale sibling of Y that the attacker wants back.
fn provisioned() -> (Anchor, Store, Store, u8) {
    let mut rng = 0;
    let mut a = boot(Tpm::new(), ENROLL, &mut rng).unwrap();
    let s_a = first(I0, 1);
    assert_eq!(acknowledge(&mut a, &s_a), Ok(()));
    let x = s_a.child(3);
    let y = s_a.child(2);
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
    (a, x, y, rng)
}

/// Next boot with an honest loader and N's startup rule on store `s`.
fn honest_reboot(a: Anchor, rec: Record, rng: &mut u8, s: &Store) -> Result<Startup, Denied> {
    match boot(a.tpm, rec, rng) {
        Ok(mut a) => startup(&mut a, s),
        Err(Halt::NeedsRecovery) => Err(Denied::Foreign),
        Err(h) => panic!("{h:?}"),
    }
}

/// Runtime attacker (kernel phase): every hierarchy path it can try.
fn runtime_attack(tpm: &mut Tpm, x: &Store) {
    assert!(!tpm.pcr13_zero);
    assert_eq!(tpm.read_hauth(), Err(TpmError::Policy));
    for guess in [&[][..], &[0x5a; 32][..]] {
        assert_eq!(tpm.undefine_owner(guess, handle()), Err(TpmError::Auth));
        assert_eq!(tpm.undefine_hauth(guess), Err(TpmError::Auth));
        assert_eq!(tpm.clear(false, guess), Err(TpmError::Auth));
        assert_eq!(tpm.write_mark(x.prev), Err(TpmError::Policy));
    }
    assert_eq!(tpm.clear(true, &[]), Err(TpmError::Auth), "U23");
}

/// The targeted M1 revival is refused once HAUTH protects the hierarchies:
/// at runtime and on the next honest boot.
#[test]
fn undefine_redefine_replay_refused_with_hauth_provisioning() {
    let (mut a, x, y, mut rng) = provisioned();
    runtime_attack(&mut a.tpm, &x);
    assert_eq!(undefine_redefine_replay(&mut a, false), Err(Denied::Stale));
    assert_eq!(undefine_redefine_replay(&mut a, true), Err(Denied::Stale));
    let mut a = boot(a.tpm, SAME, &mut rng).unwrap();
    assert_eq!(startup(&mut a, &x), Err(Denied::Stale));
    assert_eq!(startup(&mut a, &y), Ok(Startup::Accept));
    assert_eq!(a.tpm.hauth_writes_with_owner_set, 0);
}

#[test]
fn hauth_readable_only_in_the_loader_phase() {
    let (mut a, ..) = provisioned();
    assert_eq!(a.tpm.read_hauth(), Err(TpmError::Policy));
    a.tpm.reboot();
    let s = a.tpm.read_hauth().unwrap();
    assert_eq!(&s[..32], a.tpm.owner_auth.as_slice());
    assert_eq!(&s[32..], a.tpm.lockout_auth.as_slice());
    assert_ne!(&s[..32], &I0[..], "not derived from the record");
}

/// Every power cut during first provisioning converges on the next boot,
/// and `HAUTH` is never rewritten once ownerAuth is set.
#[test]
fn every_provisioning_cut_converges() {
    for cut in 0..LOADER_STEPS {
        let mut rng = 0;
        let mut tpm = Tpm::new();
        assert_eq!(
            loader(&mut tpm, ENROLL, &mut rng, cut),
            Ok(false),
            "cut {cut}"
        );
        // Kernel never ran; the next boot repeats with the same record.
        let mut a = boot(tpm, ENROLL, &mut rng).unwrap();
        assert_eq!(a.read(), Ok(genesis_chain(DOM, &I0)), "cut {cut}");
        assert_eq!(a.tpm.hauth_writes_with_owner_set, 0, "cut {cut}");
        assert!(!a.tpm.owner_auth.is_empty() && !a.tpm.lockout_auth.is_empty());
        let s = first(I0, 1);
        assert_eq!(startup(&mut a, &s), Ok(Startup::RolledForward));
    }
}

/// Clear needs lockoutAuth (or a platformAuth the OS lacks, U23). After any
/// clear the next boot halts until a fresh recovery record; the old stores
/// stay Foreign, so a clear is denial, never revival.
#[test]
fn tpm_clear_is_denial_until_authorised_recovery() {
    // Lockout clear by someone holding the secret (loader-phase code).
    let (mut a, x, y, mut rng) = provisioned();
    a.tpm.reboot();
    let lockout = a.tpm.read_hauth().unwrap()[32..].to_vec();
    a.tpm.close();
    a.tpm.clear(false, &lockout).unwrap();
    assert_eq!(honest_reboot(a, SAME, &mut rng, &x), Err(Denied::Foreign));
    // OS-usable platformAuth (U23 broken): clear, then the attacker rebuilds
    // the old chain and even provisions its own HAUTH and hierarchy auth.
    let (mut a, x, _, mut rng) = provisioned();
    a.tpm.platform_auth = Some(vec![]);
    a.tpm.clear(true, &[]).unwrap();
    let s_a = first(I0, 1);
    a.tpm.define(&[], handle()).unwrap();
    a.tpm.nv_extend(handle(), &genesis_event(DOM, &I0)).unwrap();
    a.tpm
        .nv_extend(handle(), &extend_event(DOM, &s_a.digest))
        .unwrap();
    a.tpm.write_hauth(&[], [0x77; 64]).unwrap();
    a.tpm.change_auth(false, &[], &[0x77; 32]).unwrap();
    a.tpm.change_auth(true, &[], &[0x77; 32]).unwrap();
    assert_eq!(a.tpm.write_mark(I0), Err(TpmError::Policy));
    // Honest loader: ownerAuth set, HAUTH consistent, but no loader mark.
    assert_eq!(honest_reboot(a, SAME, &mut rng, &x), Err(Denied::Foreign));
    // Authorised recovery: fresh instance; old stores stay Foreign.
    let (mut a, x, y2, mut rng) = provisioned();
    a.tpm.reboot();
    let lockout = a.tpm.read_hauth().unwrap()[32..].to_vec();
    a.tpm.clear(false, &lockout).unwrap();
    let mut a = boot(a.tpm, RECOVERY, &mut rng).unwrap();
    for old in [x, y, y2] {
        assert_eq!(startup(&mut a, &old), Err(Denied::Foreign));
    }
    assert_eq!(startup(&mut a, &first(I1, 7)), Ok(Startup::RolledForward));
}

/// Authorised same-TPM recovery keeps the hierarchy secret and needs a fresh
/// record; replaying an old record halts.
#[test]
fn authorised_recovery_needs_a_fresh_record() {
    let (mut a, x, y, mut rng) = provisioned();
    a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap(); // stray extend
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
    let owner = a.tpm.owner_auth.clone();
    let mut a = boot(a.tpm, RECOVERY, &mut rng).unwrap();
    assert_eq!(a.tpm.owner_auth, owner, "no new secret on recovery");
    assert_eq!(a.tpm.hauth_writes_with_owner_set, 0);
    for old in [x, y] {
        assert_eq!(startup(&mut a, &old), Err(Denied::Foreign));
    }
    // Back to the old record (not fresh): halt, never a second I0 genesis.
    assert_eq!(boot(a.tpm, SAME, &mut rng).err(), Some(Halt::NeedsRecovery));
}

/// Replaced or foreign TPM with the old record: halt until recovery.
#[test]
fn replaced_tpm_halts_with_the_old_record() {
    let (_, x, _, mut rng) = provisioned();
    assert_eq!(
        boot(Tpm::new(), SAME, &mut rng).err(),
        Some(Halt::NeedsRecovery)
    );
    let mut a = boot(Tpm::new(), RECOVERY, &mut rng).unwrap();
    assert_eq!(startup(&mut a, &x), Err(Denied::Foreign));
}

/// RESIDUAL until B04: code that runs at PCR 13 = 0³² (a replaced loader on
/// the ESP, a BootNext to another OS, a foreign OS on the same machine) reads
/// HAUTH and revives X one reboot later. PCR 13 phase separation does not
/// authenticate the boot path.
#[test]
fn reboot_path_attacker_at_pcr13_zero_still_revives() {
    let (mut a, x, _, mut rng) = provisioned();
    a.tpm.reboot(); // attacker-controlled loader-phase code
    let owner = a.tpm.read_hauth().unwrap()[..32].to_vec();
    let s_a = first(I0, 1);
    a.tpm.undefine_owner(&owner, handle()).unwrap();
    a.tpm.define(&owner, handle()).unwrap();
    a.tpm.nv_extend(handle(), &genesis_event(DOM, &I0)).unwrap();
    a.tpm
        .nv_extend(handle(), &extend_event(DOM, &s_a.digest))
        .unwrap();
    assert_eq!(
        honest_reboot(a, SAME, &mut rng, &x),
        Ok(Startup::RolledForward)
    );
}

#[test]
fn domains_and_instances_are_separated() {
    let d = [0x33; 32];
    assert_ne!(extend_event(0, &d), extend_event(1, &d));
    assert_ne!(genesis_event(0, &I0), genesis_event(1, &I0));
    assert_ne!(instance(&I0, 0, &[0; 32]), instance(&I0, 1, &[0; 32]));
    assert_ne!(instance(&I0, 0, &[0; 32]), instance(&I0, 0, &[1; 32]));
    assert_ne!(genesis_chain(DOM, &I0), genesis_chain(DOM, &I1));
}

/// Golden vectors, recomputed independently with Python `hashlib`; N's fake
/// and the S5 swtpm run must produce the same bytes.
#[test]
fn golden_vectors() {
    let inst = instance(&[0x11; 32], 0, &[0x22; 32]);
    let store = [0x33; 32];
    let g = genesis_chain(0, &inst);
    for (got, want) in [
        (
            inst,
            "bfe1bb14b6e07bf6c176d752ebaa1425c915d3b9e54b567fadbe2d3bd6b15a71",
        ),
        (
            genesis_event(0, &inst),
            "695716ce7f9f7ca92945c51f3425cfbd4d88ed4976dad3a09a1717c015247cf9",
        ),
        (
            g,
            "f5f47a2563aae97e4a3b116dbdf449bb78b89889475fa9a2c6995f9183f2261f",
        ),
        (
            extend_event(0, &store),
            "a0e8380d73d61df74dc648499898091350f03a83e59de072a234188679bb011c",
        ),
        (
            chain_after(0, &g, &store),
            "ba929ee819c99d8f0fb01bcec7e0ff999f4e8b3d651079ad127add6b75cd7d96",
        ),
    ] {
        assert_eq!(hex(&got), want);
    }
}
