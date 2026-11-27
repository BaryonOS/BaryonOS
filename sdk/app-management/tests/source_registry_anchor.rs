//! Host-only state machine: authentication, storage and anchor I/O are fixtures.
use app_package_core::{SourceId, SourceRecord, SourceRegistry, SourceStatus, TrustAnchors};
use sha2::{Digest, Sha256};
use update_core::anchor::{chain_after, genesis_chain, instance};

const DOMAIN: u8 = 2;
type Hash = [u8; 32];

#[derive(Clone)]
struct Snapshot {
    domain: u8,
    instance: Hash,
    prev: Hash,
    records: Vec<SourceRecord>,
    authenticated: bool, // Test fixture, not a MAC or signature.
}
impl Snapshot {
    fn digest(&self) -> Hash {
        let mut h = Sha256::new();
        h.update(b"TEST-SOURCE-SNAPSHOT-1");
        h.update([self.domain]);
        h.update(self.instance);
        h.update(self.prev);
        h.update((self.records.len() as u64).to_le_bytes());
        for r in &self.records {
            h.update(r.id.as_bytes());
            h.update(r.anchors.ed25519);
            h.update(r.anchors.mldsa65);
            h.update(r.generation.to_le_bytes());
            h.update([match r.state {
                SourceStatus::Active => 1,
                SourceStatus::Revoked => 2,
            }]);
        }
        h.finalize().into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Accepted,
    Closed,
    RateLimited,
}

struct Model {
    chain: Hash,
    instance: Hash,
    disk: Option<Snapshot>,
    published: Option<Vec<SourceRecord>>,
    budget: u32,
    closed: bool,
    known_unsubmitted: bool,
    pending_revoke: bool,
}
impl Model {
    fn new(budget: u32) -> Self {
        let instance = instance(&[3; 32], DOMAIN, &[4; 32]);
        Self {
            chain: genesis_chain(DOMAIN, &instance),
            instance,
            disk: None,
            published: None,
            budget,
            closed: true,
            known_unsubmitted: false,
            pending_revoke: false,
        }
    }
    fn candidate(&self, records: Vec<SourceRecord>) -> Snapshot {
        Snapshot {
            domain: DOMAIN,
            instance: self.instance,
            prev: self.chain,
            records,
            authenticated: true,
        }
    }
    // A successful readback is represented by retaining the exact fixture snapshot.
    fn write(&mut self, s: Snapshot) {
        self.disk = Some(s);
    }
    fn extend(&mut self, s: &Snapshot) -> Outcome {
        if self.budget == 0 {
            return Outcome::RateLimited;
        }
        if s.prev != self.chain {
            self.closed = true;
            return Outcome::Closed;
        }
        self.budget -= 1;
        self.chain = chain_after(DOMAIN, &s.prev, &s.digest());
        Outcome::Accepted
    }
    fn valid_transition(&self, next: &[SourceRecord]) -> bool {
        let Some(current) = &self.published else {
            return true;
        };
        current.iter().all(|old| {
            next.iter().find(|new| new.id == old.id).is_some_and(|new| {
                if old == new {
                    return true;
                }
                old.state == SourceStatus::Active
                    && old.generation.checked_add(1) == Some(new.generation)
                    && match new.state {
                        SourceStatus::Active => new.anchors != old.anchors,
                        SourceStatus::Revoked => new.anchors == old.anchors,
                    }
            })
        }) && next
            .iter()
            .filter(|new| !current.iter().any(|old| old.id == new.id))
            .all(|new| new.generation == 1 && new.state == SourceStatus::Active)
            && next.iter().filter(|new| !current.contains(new)).count() == 1
    }
    fn commit(&mut self, s: Snapshot, revoke: bool, lose_reply: bool) -> Outcome {
        if self.closed && self.published.is_some() {
            return Outcome::Closed;
        }
        if !s.authenticated
            || s.domain != DOMAIN
            || s.instance != self.instance
            || SourceRegistry::from_snapshot(&s.records).is_err()
            || s.prev != self.chain
            || !self.valid_transition(&s.records)
            || (self.published.is_some()
                && revoke
                    != s.records.iter().any(|new| {
                        new.state == SourceStatus::Revoked
                            && self
                                .published
                                .as_ref()
                                .unwrap()
                                .iter()
                                .any(|old| old.id == new.id && old.state == SourceStatus::Active)
                    }))
        {
            return Outcome::Closed;
        }
        if self.budget == 0 {
            return Outcome::RateLimited;
        }
        if revoke {
            self.closed = true;
            self.pending_revoke = true;
        }
        self.write(s.clone());
        self.known_unsubmitted = false;
        if self.extend(&s) != Outcome::Accepted {
            return Outcome::Closed;
        }
        if lose_reply {
            self.closed = true;
            return Outcome::Closed;
        }
        self.published = Some(s.records);
        self.pending_revoke = false;
        self.closed = false;
        Outcome::Accepted
    }
    fn recover(&mut self) -> Outcome {
        self.closed = true;
        let Some(s) = self.disk.clone() else {
            return Outcome::Closed;
        };
        if !s.authenticated
            || s.domain != DOMAIN
            || s.instance != self.instance
            || SourceRegistry::from_snapshot(&s.records).is_err()
        {
            return Outcome::Closed;
        }
        if self.pending_revoke && s.records.iter().any(|r| r.state == SourceStatus::Active) {
            return Outcome::Closed;
        }
        let after = chain_after(DOMAIN, &s.prev, &s.digest());
        if self.chain == s.prev {
            if !self.known_unsubmitted {
                return Outcome::Closed;
            }
            self.known_unsubmitted = false;
            if self.extend(&s) != Outcome::Accepted {
                return Outcome::Closed;
            }
        } else if self.chain != after {
            return Outcome::Closed;
        }
        self.published = Some(s.records);
        self.pending_revoke = false;
        self.closed = false;
        Outcome::Accepted
    }
    fn accepts(&self, id: SourceId, generation: u32) -> bool {
        !self.closed
            && self.published.as_ref().is_some_and(|records| {
                records.iter().any(|r| {
                    r.id == id && r.generation == generation && r.state == SourceStatus::Active
                })
            })
    }
}

fn record() -> SourceRecord {
    SourceRecord {
        id: SourceId::from_bytes([1; 32]),
        anchors: TrustAnchors {
            ed25519: *include_bytes!("fixtures/test-ed25519.bin"),
            mldsa65: *include_bytes!("fixtures/test-mldsa65.bin"),
        },
        generation: 1,
        state: SourceStatus::Active,
    }
}
fn installed() -> Model {
    let mut m = Model::new(4);
    let s = m.candidate(vec![record()]);
    assert_eq!(m.commit(s, false, false), Outcome::Accepted);
    m
}
fn revoked() -> SourceRecord {
    let mut registry = SourceRegistry::from_snapshot(&[record()]).unwrap();
    registry.revoke(record().id, 1).unwrap();
    let mut out = [record()];
    assert_eq!(registry.export_snapshot(&mut out), Ok(1));
    out[0]
}

#[test]
fn crash_cuts_reconcile_once_without_acknowledging_early() {
    let mut m = installed();
    let s = m.candidate(vec![revoked()]);
    m.closed = true; // revoke closes the source before durable work
    m.pending_revoke = true;
    m.write(s.clone()); // A1, no ACK yet
    m.known_unsubmitted = true; // fixture proves no request was submitted
    assert!(!m.accepts(record().id, 1));
    assert_eq!(m.recover(), Outcome::Accepted); // roll forward once
    assert!(!m.accepts(record().id, 1));
    let chain = m.chain;
    assert_eq!(m.recover(), Outcome::Accepted);
    assert_eq!(chain, m.chain);
    assert_eq!(m.published.as_ref().unwrap()[0], revoked());
}

#[test]
fn uncertain_extend_never_retries_and_reconciles_exact_candidate() {
    let mut m = installed();
    let s = m.candidate(vec![revoked()]);
    assert_eq!(m.commit(s, true, true), Outcome::Closed);
    assert!(!m.accepts(record().id, 1));
    let (chain, budget) = (m.chain, m.budget);
    assert_eq!(m.recover(), Outcome::Accepted);
    assert_eq!((m.chain, m.budget), (chain, budget));
    assert!(!m.accepts(record().id, 1));
}

#[test]
fn stale_foreign_and_unauthenticated_snapshots_fail_closed() {
    let mut m = installed();
    let old = m.disk.clone().unwrap();
    let s = m.candidate(vec![revoked()]);
    assert_eq!(m.commit(s, true, false), Outcome::Accepted);
    m.disk = Some(old);
    assert_eq!(m.recover(), Outcome::Closed);
    assert!(!m.accepts(record().id, 1));
    let mut foreign = m.candidate(vec![record()]);
    foreign.instance[0] ^= 1;
    m.write(foreign);
    assert_eq!(m.recover(), Outcome::Closed);
    let mut corrupt = m.candidate(vec![record()]);
    corrupt.authenticated = false;
    m.write(corrupt);
    assert_eq!(m.recover(), Outcome::Closed);
}

#[test]
fn exhausted_budget_keeps_confirmed_registry_readable() {
    let mut m = installed();
    m.budget = 0;
    let s = m.candidate(vec![revoked()]);
    assert_eq!(m.commit(s, true, false), Outcome::RateLimited);
    assert!(m.accepts(record().id, 1));
}

#[test]
fn full_snapshot_is_bound_and_cross_domain_substitution_fails() {
    let m = installed();
    let s = m.disk.unwrap();
    let original = s.digest();
    let mut changed = s.clone();
    changed.records[0].anchors.mldsa65[0] ^= 1;
    assert_ne!(original, changed.digest());
    changed = s.clone();
    changed.records[0].state = SourceStatus::Revoked;
    assert_ne!(original, changed.digest());
    assert_ne!(
        chain_after(DOMAIN, &s.prev, &original),
        chain_after(1, &s.prev, &original)
    );
}

#[test]
fn missing_torn_and_invalid_candidates_never_become_authority() {
    let mut m = Model::new(4);
    assert_eq!(m.recover(), Outcome::Closed);
    let old = installed();
    let confirmed = old.disk.clone().unwrap();
    let mut m = old;
    let mut torn = m.candidate(vec![revoked()]);
    torn.authenticated = false;
    m.write(torn);
    assert_eq!(m.recover(), Outcome::Closed);
    // An independently retained authenticated old copy can be recovered only
    // while the authoritative chain still matches it.
    m.write(confirmed);
    assert_eq!(m.recover(), Outcome::Accepted);
    assert!(m.accepts(record().id, 1));
    let mut invalid = m.candidate(vec![record()]);
    invalid.records[0].generation = 0;
    assert_eq!(m.commit(invalid, false, false), Outcome::Closed);
    assert!(m.accepts(record().id, 1));
}

#[test]
fn mutation_generation_and_tombstone_are_preserved_through_recovery() {
    let mut registry = SourceRegistry::from_snapshot(&[record()]).unwrap();
    assert!(registry.revoke(record().id, 2).is_err());
    registry.revoke(record().id, 1).unwrap();
    assert!(registry.add(record().id, record().anchors).is_err());
    let mut records = [record()];
    registry.export_snapshot(&mut records).unwrap();
    let mut m = installed();
    let s = m.candidate(records.to_vec());
    assert_eq!(m.commit(s, true, false), Outcome::Accepted);
    assert_eq!(m.recover(), Outcome::Accepted);
    assert_eq!(m.published.unwrap()[0], records[0]);
}

#[test]
fn changed_domain_and_stray_extend_cannot_restore_old_snapshot() {
    let mut m = installed();
    let mut foreign = m.disk.clone().unwrap();
    foreign.domain = 1;
    m.write(foreign);
    assert_eq!(m.recover(), Outcome::Closed);
    let mut m = installed();
    m.chain = chain_after(DOMAIN, &m.chain, &[9; 32]);
    assert_eq!(m.recover(), Outcome::Closed);
    assert!(!m.accepts(record().id, 1));
}

#[test]
fn unknown_unsettled_extend_never_retries_or_reopens_revoke() {
    let mut m = installed();
    let s = m.candidate(vec![revoked()]);
    m.closed = true;
    m.pending_revoke = true;
    m.write(s); // request status is unknown; chain has not advanced yet
    m.budget -= 1; // uncertain submissions consume the fixture budget
    let (chain, budget) = (m.chain, m.budget);
    assert_eq!(m.recover(), Outcome::Closed);
    assert_eq!((m.chain, m.budget), (chain, budget));
    assert!(!m.accepts(record().id, 1));
}

#[test]
fn transition_and_predecessor_checks_preserve_confirmed_snapshot() {
    let mut m = installed();
    let old = m.disk.clone().unwrap().digest();
    let mut forged = m.candidate(vec![]); // removing an occupied slot is forbidden
    assert_eq!(m.commit(forged.clone(), true, false), Outcome::Closed);
    assert_eq!(m.disk.as_ref().unwrap().digest(), old);
    assert!(m.accepts(record().id, 1));
    forged = m.candidate(vec![revoked()]);
    forged.prev[0] ^= 1;
    assert_eq!(m.commit(forged, true, false), Outcome::Closed);
    assert_eq!(m.disk.as_ref().unwrap().digest(), old);
    assert!(m.accepts(record().id, 1));
    let mut bad = revoked();
    bad.generation += 1;
    let s = m.candidate(vec![bad]);
    assert_eq!(m.commit(s, true, false), Outcome::Closed);
    assert_eq!(m.disk.as_ref().unwrap().digest(), old);
}

#[test]
fn pending_revoke_does_not_reopen_confirmed_old_snapshot() {
    let mut m = installed();
    m.closed = true;
    m.pending_revoke = true;
    assert_eq!(m.recover(), Outcome::Closed);
    assert!(!m.accepts(record().id, 1));
}
