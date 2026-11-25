//! M1 protected-slot HOST MODEL (B, PR #524): the cold-freshness extend chain
//! against N's attack sequences in `services/net-broker/src/durable.rs`
//! (N branch `claude/n03-n04-adr-20261125` @ `fad2a39a`). This is not evidence
//! of TPM behaviour: `Tpm` below encodes U21–U24 of COLD-FRESHNESS-ANCHOR.md
//! (U21, the hierarchy-auth rules and the squatting behaviour observed on
//! swtpm 0.10.1 in docs/verification/b03-rel05-hauth-probe-20261125 and
//! b03-rel05-squat-probe-20261125; native S5 still open).
//! 5A stays in force.
//!
//! Mapping to N's tests: there is no `Pending` (one extend is the only anchor
//! write), and N's `Rollback`/`Fork`/`Ahead` collapse into one `Stale`
//! denial, because the chain value cannot say which of them applies.
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
    /// Owner undefine of a platform-created index (probe S3).
    NvAuthorization,
}

/// Loader-only record (open design item U26: an owner index written under
/// `PolicyPCR(13 = 0³²) + CC_NV_Write`). It binds the `HAUTH` contents (the
/// real record holds their SHA-256) and holds the TPM copy of the accepted
/// pair (serial, instance) of the anchored record (U13c).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Mark {
    hauth: [u8; 64],
    accepted: Option<(u32, Digest)>,
}

use tpm::{Commands, Index, Public, Tpm};

/// Model TPM. The loader below is generic over `Commands`, the commands any
/// software can send, so the compiler keeps it to what a real loader observes
/// (permanent flags, public areas, policy reads), never an authValue. Tests
/// and attackers may also use the `oracle_*` accessors.
mod tpm {
    use super::{chain_step, Digest, Mark, TpmError};
    use std::collections::BTreeMap;

    /// Loader-owned handles besides the anchor indices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Index {
        Hauth,
        /// The U26 loader record (handle open).
        Record,
    }

    /// Public area at a loader-owned handle, as observed on swtpm 0.10.1
    /// (docs/verification/b03-rel05-squat-probe-20261125).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Public {
        /// The specified attributes and policy: the Name matches.
        Expected,
        /// Owner-created squat with `AUTHREAD | AUTHWRITE` (S1, S4): anyone
        /// reads and writes it; an owner undefine or `TPM2_Clear` removes it.
        /// An owner cannot set `POLICY_DELETE` (S2, `TPM_RC_ATTRIBUTES`).
        Owner,
        /// `PLATFORMCREATE` squat (S3; needs platformAuth, U23 broken): owner
        /// undefine fails with `NV_AUTHORIZATION`, and it survives `TPM2_Clear`.
        Platform,
    }

    /// `TPM_PT_PERMANENT` as the loader reads it.
    pub struct Permanent {
        pub owner_auth_set: bool,
        pub lockout_auth_set: bool,
    }

    /// Anchor extend indices (`None` = defined, never extended), hierarchy
    /// authValues, PCR 13, `HAUTH` and the loader record.
    #[derive(Clone)]
    pub struct Tpm {
        nv: BTreeMap<u32, Option<Digest>>,
        owner_auth: Vec<u8>,
        lockout_auth: Vec<u8>,
        /// `None`: OVMF set a random platformAuth the OS never learns (U8, U23).
        platform_auth: Option<Vec<u8>>,
        /// PCR 13 still all-zero: the loader phase of this boot.
        pcr13_zero: bool,
        publics: BTreeMap<Index, Public>,
        hauth: Option<[u8; 64]>,
        record: Option<Mark>,
        hauth_writes_with_owner_set: u32,
        lockout_failures: u32,
    }

    impl Tpm {
        /// Fresh TPM: empty hierarchy authValues, loader phase.
        pub fn new() -> Self {
            Tpm {
                nv: BTreeMap::new(),
                owner_auth: vec![],
                lockout_auth: vec![],
                platform_auth: None,
                pcr13_zero: true,
                publics: BTreeMap::new(),
                hauth: None,
                record: None,
                hauth_writes_with_owner_set: 0,
                lockout_failures: 0,
            }
        }
        /// platformAuth usable by the OS (U23 broken), as on raw swtpm.
        pub fn break_u23(&mut self) {
            self.platform_auth = Some(vec![]);
        }
        pub fn oracle_owner_auth(&self) -> &[u8] {
            &self.owner_auth
        }
        pub fn oracle_lockout_auth(&self) -> &[u8] {
            &self.lockout_auth
        }
        pub fn oracle_kernel_phase(&self) -> bool {
            !self.pcr13_zero
        }
        /// `HAUTH` writes while ownerAuth was set (must stay 0).
        pub fn oracle_hauth_rewrites(&self) -> u32 {
            self.hauth_writes_with_owner_set
        }
        /// Failed lockoutAuth checks; each counts toward DA lockout.
        pub fn oracle_lockout_failures(&self) -> u32 {
            self.lockout_failures
        }
        fn remove(&mut self, i: Index) {
            self.publics.remove(&i);
            match i {
                Index::Hauth => self.hauth = None,
                Index::Record => self.record = None,
            }
        }
    }

    pub trait Commands {
        /// Power cycle: PCR 13 back to 0³², the loader phase begins.
        fn reboot(&mut self);
        /// Loader extends PCR 13 before handover: the kernel phase begins.
        fn close(&mut self);
        /// `GetCapability(TPM_PT_PERMANENT)`.
        fn permanent(&self) -> Permanent;
        /// `NV_ReadPublic`: which public area occupies the handle (`None`:
        /// not defined). The real loader compares the Name.
        fn read_public(&self, i: Index) -> Option<Public>;
        /// `NV_DefineSpace` by owner, or by platform for a `Platform` area.
        fn define_slot(&mut self, i: Index, auth: &[u8], public: Public) -> Result<(), TpmError>;
        /// `NV_UndefineSpace` by owner.
        fn undefine_slot(&mut self, i: Index, owner: &[u8]) -> Result<(), TpmError>;
        /// `NV_Write` of `HAUTH` by owner (`OWNERWRITE`; the probe's squats
        /// carry it too).
        fn write_hauth(&mut self, owner: &[u8], v: [u8; 64]) -> Result<(), TpmError>;
        /// `PolicyPCR(13 = 0³²) + CC_NV_Read` (H2 on swtpm).
        fn read_hauth(&self) -> Result<[u8; 64], TpmError>;
        /// Password read with empty auth: only a squat has `AUTHREAD` (S1).
        fn read_hauth_password(&self) -> Result<[u8; 64], TpmError>;
        /// Record write under `PolicyPCR(13 = 0³²) + CC_NV_Write` (U26).
        fn write_record(&mut self, v: Mark) -> Result<(), TpmError>;
        /// Password write with empty auth: only a squat has `AUTHWRITE` (S4).
        fn write_record_password(&mut self, v: Mark) -> Result<(), TpmError>;
        fn read_record(&self) -> Result<Mark, TpmError>;
        /// `HierarchyChangeAuth`; a wrong old lockoutAuth counts toward DA.
        fn change_auth(&mut self, lockout: bool, old: &[u8], new: &[u8]) -> Result<(), TpmError>;
        /// `TPM2_Clear` by lockout or platform: owner-created NV, the loader
        /// record and both authValues are gone (probe U24); a platform-created
        /// index survives (S3).
        fn clear(&mut self, platform: bool, auth: &[u8]) -> Result<(), TpmError>;
        /// Define an anchor index (owner).
        fn define(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError>;
        fn undefine_owner(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError>;
        /// Platform may undefine an owner-created index (U24).
        fn undefine_platform(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError>;
        /// `AUTHWRITE` with empty authValue: anyone who can submit commands
        /// (T3). The first extend starts from zeros (U21); one extend is
        /// atomic (U22).
        fn nv_extend(&mut self, h: u32, data: &Digest) -> Result<(), TpmError>;
        fn nv_read(&self, h: u32) -> Result<Digest, TpmError>;
    }

    impl Commands for Tpm {
        fn reboot(&mut self) {
            self.pcr13_zero = true;
        }
        fn close(&mut self) {
            self.pcr13_zero = false;
        }
        fn permanent(&self) -> Permanent {
            Permanent {
                owner_auth_set: !self.owner_auth.is_empty(),
                lockout_auth_set: !self.lockout_auth.is_empty(),
            }
        }
        fn read_public(&self, i: Index) -> Option<Public> {
            self.publics.get(&i).copied()
        }
        fn define_slot(&mut self, i: Index, auth: &[u8], public: Public) -> Result<(), TpmError> {
            let ok = if public == Public::Platform {
                self.platform_auth.as_deref() == Some(auth)
            } else {
                auth == self.owner_auth
            };
            if !ok {
                return Err(TpmError::Auth);
            }
            if self.publics.contains_key(&i) {
                return Err(TpmError::Exists);
            }
            self.publics.insert(i, public);
            Ok(())
        }
        fn undefine_slot(&mut self, i: Index, owner: &[u8]) -> Result<(), TpmError> {
            if owner != self.owner_auth {
                return Err(TpmError::Auth);
            }
            match self.publics.get(&i) {
                None => Err(TpmError::Handle),
                Some(Public::Platform) => Err(TpmError::NvAuthorization),
                Some(_) => {
                    self.remove(i);
                    Ok(())
                }
            }
        }
        fn write_hauth(&mut self, owner: &[u8], v: [u8; 64]) -> Result<(), TpmError> {
            if owner != self.owner_auth {
                return Err(TpmError::Auth);
            }
            if !self.publics.contains_key(&Index::Hauth) {
                return Err(TpmError::Handle);
            }
            if !self.owner_auth.is_empty() {
                self.hauth_writes_with_owner_set += 1;
            }
            self.hauth = Some(v);
            Ok(())
        }
        fn read_hauth(&self) -> Result<[u8; 64], TpmError> {
            if !self.publics.contains_key(&Index::Hauth) {
                return Err(TpmError::Handle);
            }
            if !self.pcr13_zero {
                return Err(TpmError::Policy);
            }
            self.hauth.ok_or(TpmError::Uninitialized)
        }
        fn read_hauth_password(&self) -> Result<[u8; 64], TpmError> {
            match self.publics.get(&Index::Hauth) {
                None => Err(TpmError::Handle),
                Some(Public::Expected) => Err(TpmError::Auth),
                Some(_) => self.hauth.ok_or(TpmError::Uninitialized),
            }
        }
        fn write_record(&mut self, v: Mark) -> Result<(), TpmError> {
            if !self.pcr13_zero {
                return Err(TpmError::Policy);
            }
            if !self.publics.contains_key(&Index::Record) {
                return Err(TpmError::Handle);
            }
            self.record = Some(v);
            Ok(())
        }
        fn write_record_password(&mut self, v: Mark) -> Result<(), TpmError> {
            match self.publics.get(&Index::Record) {
                None => Err(TpmError::Handle),
                Some(Public::Expected) => Err(TpmError::Auth),
                Some(_) => {
                    self.record = Some(v);
                    Ok(())
                }
            }
        }
        fn read_record(&self) -> Result<Mark, TpmError> {
            if !self.publics.contains_key(&Index::Record) {
                return Err(TpmError::Handle);
            }
            self.record.ok_or(TpmError::Uninitialized)
        }
        fn change_auth(&mut self, lockout: bool, old: &[u8], new: &[u8]) -> Result<(), TpmError> {
            let slot = if lockout {
                &mut self.lockout_auth
            } else {
                &mut self.owner_auth
            };
            if old != slot.as_slice() {
                self.lockout_failures += u32::from(lockout);
                return Err(TpmError::Auth);
            }
            *slot = new.to_vec();
            Ok(())
        }
        fn clear(&mut self, platform: bool, auth: &[u8]) -> Result<(), TpmError> {
            let ok = if platform {
                self.platform_auth.as_deref() == Some(auth)
            } else {
                auth == self.lockout_auth
            };
            if !ok {
                self.lockout_failures += u32::from(!platform);
                return Err(TpmError::Auth);
            }
            let mut t = Tpm::new();
            t.platform_auth = self.platform_auth.take();
            t.pcr13_zero = self.pcr13_zero;
            t.lockout_failures = self.lockout_failures;
            for (i, p) in &self.publics {
                if *p == Public::Platform {
                    t.publics.insert(*i, *p);
                }
            }
            if t.publics.contains_key(&Index::Hauth) {
                t.hauth = self.hauth;
            }
            if t.publics.contains_key(&Index::Record) {
                t.record = self.record;
            }
            *self = t;
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
        fn undefine_platform(&mut self, auth: &[u8], h: u32) -> Result<(), TpmError> {
            if self.platform_auth.as_deref() != Some(auth) {
                return Err(TpmError::Auth);
            }
            self.nv.remove(&h).map(|_| ()).ok_or(TpmError::Handle)
        }
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
    let owner = tpm.oracle_owner_auth().to_vec();
    match tpm.define(&owner, handle()) {
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
    a.tpm.undefine_owner(&[], handle()).unwrap();
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
    let auth = a.tpm.oracle_owner_auth().to_vec();
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

/// Signed `ENROLL` or recovery record as the loader finds it. The issuer gives
/// every record a unique serial, greater than every serial before (REL-05
/// section 10.5); the model uses the anchored instance as the record digest.
#[derive(Clone, Copy)]
struct Record {
    instance: Digest,
    serial: u32,
}

/// Accepted pair (serial, record digest) of U13c.
type Pair = (u32, Digest);

/// Per-machine state outside the TPM: loader entropy (`EFI_RNG_PROTOCOL`,
/// U25) and the UEFI copy of the accepted pair (U13c). The model assumes the
/// variable survives TPM clear and replacement and cannot be written after
/// `ExitBootServices` (U28, unverified).
#[derive(Clone, Default)]
struct Firmware {
    rng: u8,
    accepted: Option<Pair>,
}

impl Firmware {
    fn random(&mut self) -> [u8; 64] {
        self.rng = self.rng.wrapping_add(1);
        [self.rng; 64]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Halt {
    /// An older or reused record, or a TPM cleared, replaced or never
    /// provisioned without a newer record (C665, C28).
    NeedsRecovery,
    /// A squatted `HAUTH`, `HAUTH` unreadable, a failed command, or two
    /// copies of the accepted pair that disagree at one serial.
    Corrupt,
}

/// The loader record, trusted only with the expected public area: anyone can
/// write a squatted one at run time (probe S4).
fn trusted_record<T: Commands>(tpm: &T) -> Option<Mark> {
    if tpm.read_public(Index::Record) != Some(Public::Expected) {
        return None;
    }
    tpm.read_record().ok()
}

/// The surviving copy with the higher serial (U13c).
fn accepted<T: Commands>(tpm: &T, fw: &Firmware) -> Result<Option<Pair>, Halt> {
    match (trusted_record(tpm).and_then(|m| m.accepted), fw.accepted) {
        (Some(t), Some(u)) if t.0 == u.0 && t != u => Err(Halt::Corrupt),
        (t, u) => Ok(t.max(u)),
    }
}

/// `Ok(true)`: newer than every accepted record, so it may authorise one
/// genesis. `Ok(false)`: the current record on a restart.
fn judge(rec: Record, accepted: Option<Pair>) -> Result<bool, Halt> {
    match accepted {
        Some(p) if p == (rec.serial, rec.instance) => Ok(false),
        Some((serial, _)) if rec.serial <= serial => Err(Halt::NeedsRecovery),
        _ => Ok(true),
    }
}

/// Writes of a first provisioning on a fresh TPM.
const LOADER_STEPS: u32 = 10;

/// TEST-profile loader, re-run from the start after any cut (`cut` = writes
/// before the power loss). It sees only `Commands` and `Firmware`. Order:
/// remove leftovers and owner squats, define and write `HAUTH`, define and
/// write the record binding it, ownerAuth, lockoutAuth, the anchor genesis and
/// the record naming the accepted pair (TPM copy), the UEFI copy, then the
/// PCR 13 extend. The provisioning state comes from the permanent flags, never
/// from trying an authorisation. A `HAUTH` that no trusted record binds is
/// never adopted: with a newer record the loader clears it and provisions new
/// secrets, otherwise it halts.
fn loader<T: Commands>(
    tpm: &mut T,
    fw: &mut Firmware,
    rec: Record,
    cut: u32,
) -> Result<bool, Halt> {
    tpm.reboot();
    let mut writes = 0;
    // One state-changing command; a power cut may follow any of them.
    macro_rules! write {
        ($e:expr) => {{
            $e.map_err(|_| Halt::Corrupt)?;
            writes += 1;
            if writes > cut {
                tpm.reboot(); // power loss: nothing after this write
                return Ok(false);
            }
        }};
    }
    let fresh = judge(rec, accepted(tpm, fw)?)?;
    loop {
        if !tpm.permanent().owner_auth_set {
            if !fresh {
                return Err(Halt::NeedsRecovery);
            }
            // Leftovers of a cut, and squats made while ownerAuth was empty;
            // only "not defined" is tolerated (a platform squat halts).
            for i in [Index::Hauth, Index::Record] {
                match tpm.undefine_slot(i, &[]) {
                    Err(TpmError::Handle) => {}
                    r => write!(r),
                }
            }
            let secret = fw.random();
            write!(tpm.define_slot(Index::Hauth, &[], Public::Expected));
            write!(tpm.write_hauth(&[], secret));
            write!(tpm.define_slot(Index::Record, &[], Public::Expected));
            write!(tpm.write_record(Mark {
                hauth: secret,
                accepted: None,
            }));
            write!(tpm.change_auth(false, &[], &secret[..32]));
        }
        // Every boot: a squatted HAUTH is never read into an authorisation.
        if tpm.read_public(Index::Hauth) != Some(Public::Expected) {
            return Err(Halt::Corrupt);
        }
        let secret = tpm.read_hauth().map_err(|_| Halt::Corrupt)?;
        if trusted_record(tpm).map(|m| m.hauth) == Some(secret) {
            break;
        }
        // Foreign hierarchy state: halt, or clear it under a newer record.
        if !fresh {
            return Err(Halt::NeedsRecovery);
        }
        let lockout: &[u8] = if tpm.permanent().lockout_auth_set {
            &secret[32..]
        } else {
            &[]
        };
        write!(tpm.clear(false, lockout));
    }
    let secret = tpm.read_hauth().map_err(|_| Halt::Corrupt)?;
    if !tpm.permanent().lockout_auth_set {
        write!(tpm.change_auth(true, &[], &secret[32..]));
    }
    let pair = (rec.serial, rec.instance);
    if trusted_record(tpm).and_then(|m| m.accepted) != Some(pair) {
        if !fresh {
            return Err(Halt::NeedsRecovery);
        }
        let owner = &secret[..32];
        match tpm.undefine_owner(owner, handle()) {
            Err(TpmError::Handle) => {}
            r => write!(r),
        }
        write!(tpm.define(owner, handle()));
        write!(tpm.nv_extend(handle(), &genesis_event(DOM, &rec.instance)));
        write!(tpm.write_record(Mark {
            hauth: secret,
            accepted: Some(pair),
        }));
    }
    // The UEFI copy follows the TPM copy, so a cut between them converges.
    if fw.accepted != Some(pair) {
        write!({
            fw.accepted = Some(pair);
            Ok::<_, TpmError>(())
        });
    }
    tpm.close(); // secret zeroised before the PCR 13 extend
    Ok(true)
}

fn boot(tpm: Tpm, rec: Record, fw: &mut Firmware) -> Result<Anchor, Halt> {
    let mut tpm = tpm;
    assert!(loader(&mut tpm, fw, rec, u32::MAX)?);
    Ok(Anchor {
        tpm,
        instance: rec.instance,
        budget: 16,
        lose_next_response: false,
    })
}

const ENROLL: Record = Record {
    instance: I0,
    serial: 1,
};
/// The enrolment record on a later boot.
const SAME: Record = ENROLL;
const RECOVERY: Record = Record {
    instance: I1,
    serial: 2,
};

/// Enrolled machine after the M1 sequence: A, then Y acknowledged; X is the
/// stale sibling of Y that the attacker wants back.
fn provisioned() -> (Anchor, Store, Store, Firmware) {
    let mut fw = Firmware::default();
    let mut a = boot(Tpm::new(), ENROLL, &mut fw).unwrap();
    let s_a = first(I0, 1);
    assert_eq!(acknowledge(&mut a, &s_a), Ok(()));
    let x = s_a.child(3);
    let y = s_a.child(2);
    assert_eq!(acknowledge(&mut a, &y), Ok(()));
    (a, x, y, fw)
}

/// Next boot with an honest loader and N's startup rule on store `s`.
fn honest_reboot(a: Anchor, rec: Record, fw: &mut Firmware, s: &Store) -> Result<Startup, Denied> {
    match boot(a.tpm, rec, fw) {
        Ok(mut a) => startup(&mut a, s),
        Err(Halt::NeedsRecovery) => Err(Denied::Foreign),
        Err(h) => panic!("{h:?}"),
    }
}

/// Runtime attacker (kernel phase): every hierarchy path it can try.
fn runtime_attack(tpm: &mut Tpm, x: &Store) {
    assert!(tpm.oracle_kernel_phase());
    assert_eq!(tpm.read_hauth(), Err(TpmError::Policy));
    assert_eq!(tpm.read_hauth_password(), Err(TpmError::Auth));
    for guess in [&[][..], &[0x5a; 32][..]] {
        assert_eq!(tpm.undefine_owner(guess, handle()), Err(TpmError::Auth));
        assert_eq!(tpm.undefine_slot(Index::Hauth, guess), Err(TpmError::Auth));
        assert_eq!(tpm.clear(false, guess), Err(TpmError::Auth));
    }
    assert_eq!(tpm.clear(true, &[]), Err(TpmError::Auth), "U23");
    let m = tpm.read_record();
    let forged = Mark {
        hauth: [0x77; 64],
        accepted: Some((u32::MAX, x.instance)),
    };
    assert_eq!(tpm.write_record(forged), Err(TpmError::Policy));
    assert_eq!(tpm.write_record_password(forged), Err(TpmError::Auth));
    assert_eq!(tpm.read_record(), m);
}

/// Attacker after a clear (ownerAuth empty): the old I0 chain up to A.
fn rebuild_old_chain(tpm: &mut Tpm) {
    tpm.define(&[], handle()).unwrap();
    tpm.nv_extend(handle(), &genesis_event(DOM, &I0)).unwrap();
    tpm.nv_extend(handle(), &extend_event(DOM, &first(I0, 1).digest))
        .unwrap();
}

/// Attacker after a clear provisions its own `HAUTH` (`0x77…`, expected
/// public area) and ownerAuth, and lockoutAuth if `lockout`.
fn foreign_hierarchy(tpm: &mut Tpm, lockout: bool) {
    tpm.define_slot(Index::Hauth, &[], Public::Expected)
        .unwrap();
    tpm.write_hauth(&[], [0x77; 64]).unwrap();
    tpm.change_auth(false, &[], &[0x77; 32]).unwrap();
    if lockout {
        tpm.change_auth(true, &[], &[0x77; 32]).unwrap();
    }
}

/// The targeted M1 revival is refused once HAUTH protects the hierarchies:
/// at runtime and on the next honest boot.
#[test]
fn undefine_redefine_replay_refused_with_hauth_provisioning() {
    let (mut a, x, y, mut fw) = provisioned();
    runtime_attack(&mut a.tpm, &x);
    assert_eq!(undefine_redefine_replay(&mut a, false), Err(Denied::Stale));
    assert_eq!(undefine_redefine_replay(&mut a, true), Err(Denied::Stale));
    let mut a = boot(a.tpm, SAME, &mut fw).unwrap();
    assert_eq!(startup(&mut a, &x), Err(Denied::Stale));
    assert_eq!(startup(&mut a, &y), Ok(Startup::Accept));
    assert_eq!(a.tpm.oracle_hauth_rewrites(), 0);
}

#[test]
fn hauth_readable_only_in_the_loader_phase() {
    let (mut a, ..) = provisioned();
    assert_eq!(a.tpm.read_hauth(), Err(TpmError::Policy));
    a.tpm.reboot();
    let s = a.tpm.read_hauth().unwrap();
    assert_eq!(&s[..32], a.tpm.oracle_owner_auth());
    assert_eq!(&s[32..], a.tpm.oracle_lockout_auth());
    assert_ne!(&s[..32], &I0[..], "not derived from the record");
}

/// Every power cut during first provisioning converges on the next boot,
/// `HAUTH` is never rewritten once ownerAuth is set, and no lockoutAuth is
/// ever tried and failed.
#[test]
fn every_provisioning_cut_converges() {
    for cut in 0.. {
        let mut fw = Firmware::default();
        let mut tpm = Tpm::new();
        let done = loader(&mut tpm, &mut fw, ENROLL, cut).unwrap();
        // Kernel never ran; the next boot repeats with the same record.
        let mut a = boot(tpm, ENROLL, &mut fw).unwrap();
        assert_eq!(a.read(), Ok(genesis_chain(DOM, &I0)), "cut {cut}");
        assert_eq!(a.tpm.oracle_hauth_rewrites(), 0, "cut {cut}");
        assert_eq!(a.tpm.oracle_lockout_failures(), 0, "cut {cut}");
        let p = a.tpm.permanent();
        assert!(p.owner_auth_set && p.lockout_auth_set, "cut {cut}");
        assert_eq!(fw.accepted, Some((ENROLL.serial, I0)), "cut {cut}");
        assert_eq!(startup(&mut a, &first(I0, 1)), Ok(Startup::RolledForward));
        if done {
            assert_eq!(cut, LOADER_STEPS, "writes of a first provisioning");
            break;
        }
    }
}

/// Clear needs lockoutAuth (or a platformAuth the OS lacks, U23). After any
/// clear the next boot halts until a newer recovery record; the old stores
/// stay Foreign, so a clear is denial, never revival.
#[test]
fn tpm_clear_is_denial_until_authorised_recovery() {
    // Lockout clear by someone holding the secret (loader-phase code).
    let (mut a, x, y, mut fw) = provisioned();
    a.tpm.reboot();
    let lockout = a.tpm.read_hauth().unwrap()[32..].to_vec();
    a.tpm.close();
    a.tpm.clear(false, &lockout).unwrap();
    assert_eq!(honest_reboot(a, SAME, &mut fw, &x), Err(Denied::Foreign));
    // OS-usable platformAuth (U23 broken): clear, then the attacker rebuilds
    // the old chain and even provisions its own HAUTH and hierarchy auth.
    let (mut a, x, _, mut fw) = provisioned();
    a.tpm.break_u23();
    a.tpm.clear(true, &[]).unwrap();
    rebuild_old_chain(&mut a.tpm);
    foreign_hierarchy(&mut a.tpm, true);
    let forged = Mark {
        hauth: [0x77; 64],
        accepted: Some((ENROLL.serial, I0)),
    };
    assert_eq!(a.tpm.write_record(forged), Err(TpmError::Policy));
    // Honest loader: ownerAuth set, HAUTH consistent, but no loader record.
    assert_eq!(
        boot(a.tpm.clone(), SAME, &mut fw.clone()).err(),
        Some(Halt::NeedsRecovery)
    );
    // Recovery never adopts the attacker's secret: it clears and re-provisions,
    // so a runtime replay with the old secret fails afterwards.
    let mut r = boot(a.tpm, RECOVERY, &mut fw).unwrap();
    assert_ne!(r.tpm.oracle_owner_auth(), &[0x77; 32]);
    assert_eq!(r.tpm.oracle_hauth_rewrites(), 0);
    assert_eq!(r.tpm.oracle_lockout_failures(), 0);
    assert_eq!(
        r.tpm.undefine_owner(&[0x77; 32], handle()),
        Err(TpmError::Auth)
    );
    let g = first(I1, 7);
    assert_eq!(startup(&mut r, &g), Ok(Startup::RolledForward));
    assert_eq!(startup(&mut r, &x), Err(Denied::Foreign));
    // Authorised recovery: fresh instance; old stores stay Foreign.
    let (mut a, x, y2, mut fw) = provisioned();
    a.tpm.reboot();
    let lockout = a.tpm.read_hauth().unwrap()[32..].to_vec();
    a.tpm.clear(false, &lockout).unwrap();
    let mut a = boot(a.tpm, RECOVERY, &mut fw).unwrap();
    for old in [x, y, y2] {
        assert_eq!(startup(&mut a, &old), Err(Denied::Foreign));
    }
    assert_eq!(startup(&mut a, &first(I1, 7)), Ok(Startup::RolledForward));
}

/// Security review (squatting, U26): after a platform clear (U23 broken) a
/// runtime attacker squats the record handle writable (probe S4), forges a
/// record that binds its own HAUTH and names the accepted pair of ENROLL, and
/// rebuilds the old chain. The loader trusts only a record with the expected
/// public area, so it halts instead of reviving X.
#[test]
fn forged_record_after_a_platform_clear_is_never_adopted() {
    let (mut a, x, _, mut fw) = provisioned();
    a.tpm.break_u23();
    a.tpm.clear(true, &[]).unwrap();
    rebuild_old_chain(&mut a.tpm);
    a.tpm
        .define_slot(Index::Record, &[], Public::Owner)
        .unwrap();
    let forged = Mark {
        hauth: [0x77; 64],
        accepted: Some((ENROLL.serial, I0)),
    };
    a.tpm.write_record_password(forged).unwrap();
    foreign_hierarchy(&mut a.tpm, true);
    assert_eq!(honest_reboot(a, SAME, &mut fw, &x), Err(Denied::Foreign));
}

/// Security review (squatting, probe S1/S3): while ownerAuth is empty (a
/// cleared TPM that reached a kernel; REL-05 §7.9 open) a runtime attacker
/// pre-defines `HAUTH` readable by password. Recovery removes an owner squat
/// before the secret is written; a platform squat (U23 broken) halts before
/// any write. With ownerAuth set, a platform squat halts before its contents
/// are tried as lockoutAuth.
#[test]
fn squatted_hauth_never_receives_the_secret() {
    let cleared = || {
        let (mut a, x, _, fw) = provisioned();
        a.tpm.reboot();
        let lockout = a.tpm.read_hauth().unwrap()[32..].to_vec();
        a.tpm.clear(false, &lockout).unwrap();
        a.tpm.close();
        (a.tpm, fw, x)
    };
    let (mut tpm, mut fw, x) = cleared();
    tpm.define_slot(Index::Hauth, &[], Public::Owner).unwrap();
    assert_eq!(loader(&mut tpm, &mut fw, RECOVERY, u32::MAX), Ok(true));
    runtime_attack(&mut tpm, &x);
    let (mut tpm, mut fw, _) = cleared();
    tpm.break_u23();
    tpm.define_slot(Index::Hauth, &[], Public::Platform)
        .unwrap();
    assert_eq!(
        loader(&mut tpm, &mut fw, RECOVERY, u32::MAX),
        Err(Halt::Corrupt)
    );
    assert_eq!(tpm.read_hauth_password(), Err(TpmError::Uninitialized));
    let (mut tpm, mut fw, _) = cleared();
    tpm.break_u23();
    tpm.define_slot(Index::Hauth, &[], Public::Platform)
        .unwrap();
    tpm.write_hauth(&[], [0x77; 64]).unwrap();
    tpm.change_auth(false, &[], &[0x77; 32]).unwrap();
    tpm.change_auth(true, &[], &[0x66; 32]).unwrap();
    assert_eq!(
        loader(&mut tpm, &mut fw, RECOVERY, u32::MAX),
        Err(Halt::Corrupt)
    );
    assert_eq!(tpm.oracle_lockout_failures(), 0);
}

/// Authorised same-TPM recovery keeps the hierarchy secret and needs a newer
/// record; replaying an old record, or another record that reuses the
/// accepted serial, halts.
#[test]
fn authorised_recovery_needs_a_fresh_record() {
    let (mut a, x, y, mut fw) = provisioned();
    a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap(); // stray extend
    assert_eq!(startup(&mut a, &y), Err(Denied::Stale));
    let owner = a.tpm.oracle_owner_auth().to_vec();
    let mut a = boot(a.tpm, RECOVERY, &mut fw).unwrap();
    assert_eq!(
        a.tpm.oracle_owner_auth(),
        owner,
        "no new secret on recovery"
    );
    assert_eq!(a.tpm.oracle_hauth_rewrites(), 0);
    for old in [x, y] {
        assert_eq!(startup(&mut a, &old), Err(Denied::Foreign));
    }
    let reused = Record {
        instance: [0xa2; 32],
        serial: RECOVERY.serial,
    };
    for rec in [SAME, reused] {
        assert_eq!(
            boot(a.tpm.clone(), rec, &mut fw.clone()).err(),
            Some(Halt::NeedsRecovery)
        );
    }
}

/// Every power cut during a recovery converges: after a stray extend (same
/// secret), and from a hierarchy provisioned by someone else after a platform
/// clear, with and without its lockoutAuth (clear, then new secrets). The
/// clear follows the permanent flags, so no lockoutAuth is tried and failed.
#[test]
fn every_recovery_cut_converges() {
    for case in ["stray", "foreign", "foreign-no-lockout"] {
        for cut in 0.. {
            let (mut a, _, _, mut fw) = provisioned();
            let foreign = case != "stray";
            if foreign {
                a.tpm.break_u23();
                a.tpm.clear(true, &[]).unwrap();
                foreign_hierarchy(&mut a.tpm, case == "foreign");
            } else {
                a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap();
            }
            let owner = a.tpm.oracle_owner_auth().to_vec();
            let mut tpm = a.tpm;
            let done = loader(&mut tpm, &mut fw, RECOVERY, cut).unwrap();
            let mut r = boot(tpm, RECOVERY, &mut fw).unwrap();
            assert_eq!(r.read(), Ok(genesis_chain(DOM, &I1)), "{case} cut {cut}");
            assert_eq!(r.tpm.oracle_hauth_rewrites(), 0, "{case} cut {cut}");
            assert_eq!(r.tpm.oracle_lockout_failures(), 0, "{case} cut {cut}");
            assert_eq!(
                r.tpm.oracle_owner_auth() == owner,
                !foreign,
                "{case} cut {cut}"
            );
            assert_eq!(fw.accepted, Some((RECOVERY.serial, I1)), "{case} cut {cut}");
            assert_eq!(startup(&mut r, &first(I1, 7)), Ok(Startup::RolledForward));
            if done {
                break;
            }
        }
    }
}

/// Replaced or foreign TPM with the old record: halt until recovery.
#[test]
fn replaced_tpm_halts_with_the_old_record() {
    let (_, x, _, mut fw) = provisioned();
    assert_eq!(
        boot(Tpm::new(), SAME, &mut fw.clone()).err(),
        Some(Halt::NeedsRecovery)
    );
    let mut a = boot(Tpm::new(), RECOVERY, &mut fw).unwrap();
    assert_eq!(startup(&mut a, &x), Err(Denied::Foreign));
}

/// U13c: losing one copy of the accepted pair keeps older records refused,
/// and the lost UEFI copy is rewritten; two copies that disagree at one serial
/// halt. RESIDUAL: losing both (a TPM clear with U23 broken or by firmware,
/// plus a UEFI NVRAM reset; boot-path class, before B04) lets the old ENROLL
/// authorise a second I0 genesis and revives the first I0 store. Reported,
/// not accepted.
#[test]
fn accepted_pair_survives_one_lost_copy() {
    let (mut a, _, y, mut fw) = provisioned();
    a.tpm.nv_extend(handle(), &[0xee; 32]).unwrap(); // stray extend
    let a = boot(a.tpm, RECOVERY, &mut fw).unwrap();
    let pair = Some((RECOVERY.serial, I1));
    // UEFI NVRAM reset: the TPM copy refuses ENROLL and restores the UEFI copy.
    let mut reset = Firmware {
        accepted: None,
        ..fw.clone()
    };
    assert_eq!(
        boot(a.tpm.clone(), ENROLL, &mut reset).err(),
        Some(Halt::NeedsRecovery)
    );
    assert!(boot(a.tpm.clone(), RECOVERY, &mut reset).is_ok());
    assert_eq!(reset.accepted, pair);
    let mut clash = Firmware {
        accepted: Some((RECOVERY.serial, I0)),
        ..fw.clone()
    };
    assert_eq!(
        boot(a.tpm.clone(), RECOVERY, &mut clash).err(),
        Some(Halt::Corrupt)
    );
    // TPM clear: the UEFI copy refuses ENROLL; RECOVERY, now current, halts
    // until a newer record.
    let mut cleared = a.tpm;
    cleared.break_u23();
    cleared.clear(true, &[]).unwrap();
    for rec in [ENROLL, RECOVERY] {
        assert_eq!(
            boot(cleared.clone(), rec, &mut fw.clone()).err(),
            Some(Halt::NeedsRecovery)
        );
    }
    // Both copies lost: the first I0 store revives; Y stays stale.
    let mut lost = Firmware {
        accepted: None,
        ..fw
    };
    let mut r = boot(cleared, ENROLL, &mut lost).unwrap();
    assert_eq!(startup(&mut r, &y), Err(Denied::Stale));
    assert_eq!(startup(&mut r, &first(I0, 1)), Ok(Startup::RolledForward));
}

/// RESIDUAL until B04: code that runs at PCR 13 = 0³² (a replaced loader on
/// the ESP, a BootNext to another OS, a foreign OS on the same machine) reads
/// HAUTH and revives X one reboot later. PCR 13 phase separation does not
/// authenticate the boot path.
#[test]
fn reboot_path_attacker_at_pcr13_zero_still_revives() {
    let (mut a, x, _, mut fw) = provisioned();
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
        honest_reboot(a, SAME, &mut fw, &x),
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
