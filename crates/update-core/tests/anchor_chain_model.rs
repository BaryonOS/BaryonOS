//! M1 protected-slot HOST MODEL (B, PR #524): the cold-freshness extend chain
//! against N's attack sequences in `services/net-broker/src/durable.rs`
//! (N branch `claude/n03-n04-adr-20261125` @ `fad2a39a`). This is not evidence
//! of TPM behaviour: `Tpm` below encodes the unverified items U21–U24 of
//! COLD-FRESHNESS-ANCHOR.md, which S5 must prove on swtpm. 5A stays in force.
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
}

/// Model TPM: the anchor extend indices and the authorisation that guards
/// undefining them. `None` = defined but never extended.
struct Tpm {
    nv: BTreeMap<u32, Option<Digest>>,
    owner_auth: Vec<u8>,
    /// `None`: OVMF set a random platformAuth the OS never learns (U8, U23).
    platform_auth: Option<Vec<u8>>,
}

impl Tpm {
    /// TEST profile today: empty ownerAuth.
    fn new() -> Self {
        Tpm {
            nv: BTreeMap::new(),
            owner_auth: vec![],
            platform_auth: None,
        }
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

/// RESIDUAL in today's TEST profile (empty ownerAuth): T3 revives X.
#[test]
fn undefine_redefine_replay_revives_with_empty_owner_auth() {
    let mut a = after_m1_sequence();
    assert_eq!(
        undefine_redefine_replay(&mut a, false),
        Ok(Startup::RolledForward)
    );
}

/// Proposed mitigation (OWNER decision pending): ownerAuth set by the loader
/// to a PCR 13 = 0³²-sealed secret. Rests on U8/U23/U24 as well.
#[test]
fn undefine_redefine_replay_refused_with_protected_owner_auth() {
    let mut a = after_m1_sequence();
    a.tpm.owner_auth = vec![0x5a; 32];
    assert_eq!(undefine_redefine_replay(&mut a, false), Err(Denied::Stale));
    assert_eq!(undefine_redefine_replay(&mut a, true), Err(Denied::Stale));
    // An OS-usable platformAuth could delete the index: denial, not revival.
    let mut a = after_m1_sequence();
    a.tpm.owner_auth = vec![0x5a; 32];
    a.tpm.platform_auth = Some(vec![]);
    assert_eq!(
        undefine_redefine_replay(&mut a, true),
        Err(Denied::Anchor(AnchorError::Unavailable))
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
