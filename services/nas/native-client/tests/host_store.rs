//! Deliberately non-cryptographic fake; tests ordering and trust boundaries only.
use nieuw_os_nas_client::{
    host_store::{
        self, Authorization, Backend, Binding, Coordinator, IndependentAuthority, Manifest,
        StoreError, TrustedPeerConnection, CHUNKS,
    },
    Checkpoint, Client, Remote, State, CHECKPOINT_BYTES,
};
use nieuw_os_nas_core::{Error, Principal, Session, Upload};
use std::{cell::Cell, rc::Rc};

fn binding() -> Binding {
    Binding {
        user: [1; 32],
        peer: [2; 32],
        namespace: [3; 32],
        volume: [4; 32],
        grant: [5; 32],
        grant_scope: [6; 32],
        grant_generation: 7,
        checkpoint: [8; 32],
        objects: [[9; 32]; 4],
        revisions: [0; 4],
    }
}
struct Auth {
    allowed: bool,
}
impl Authorization for Auth {
    fn authorize(&mut self, binding: &Binding, cp: &Checkpoint) -> bool {
        let client = Client::restore(cp.clone());
        self.allowed
            && (0..4).all(|i| match client.edit(i) {
                Ok(edit) => {
                    edit.namespace() == &binding.namespace
                        && edit.base_revision() == binding.revisions[i]
                        && binding.objects[i] == [9 + edit.object() as u8; 32]
                }
                Err(_) => true,
            })
    }
}
#[derive(Clone)]
struct Chunk {
    manifest: Manifest,
    index: usize,
    bytes: Vec<u8>,
}
#[derive(Default)]
struct Fake {
    staged: Vec<Chunk>,
    published: Option<(Manifest, Vec<Chunk>)>,
    preserved: Option<(Manifest, Vec<Chunk>)>,
    fail_at: Option<usize>,
    fail_flush: bool,
    unknown: bool,
    unknown_applied: bool,
    attempts: Vec<[u8; 32]>,
    unresolved: Option<[u8; 32]>,
    fenced: Vec<[u8; 32]>,
    fail_fence: bool,
    fail_finalize: bool,
    retained: Vec<(Manifest, Vec<Chunk>)>,
    verifies: usize,
    delayed: Option<(Manifest, Vec<Chunk>)>,
}
impl Fake {
    fn snapshot_valid(&mut self, snapshot: &(Manifest, Vec<Chunk>)) -> bool {
        let (manifest, chunks) = snapshot;
        if chunks.len() != CHUNKS
            || chunks.iter().enumerate().any(|(i, c)| {
                c.manifest != *manifest
                    || c.index != i
                    || c.bytes.len() != if i + 1 == CHUNKS { 16 } else { 128 }
            })
        {
            return false;
        }
        let mut bytes = [0; CHECKPOINT_BYTES];
        for chunk in chunks {
            let start = chunk.index * host_store::CHUNK_BYTES;
            bytes[start..start + chunk.bytes.len()].copy_from_slice(&chunk.bytes);
        }
        self.commitment(&bytes) == manifest.commitment
    }
    fn complete_delayed(&mut self) -> bool {
        let Some((manifest, chunks)) = self.delayed.take() else {
            return false;
        };
        if self.fenced.contains(&manifest.attempt) {
            return false;
        }
        self.published = Some((manifest, chunks));
        true
    }
}
impl Backend for Fake {
    fn begin_attempt(&mut self, attempt: &[u8; 32]) -> Result<(), StoreError> {
        if self.unresolved.is_some() {
            return Err(StoreError::Unknown);
        }
        if self.attempts.contains(attempt) {
            return Err(StoreError::Invalid);
        }
        self.attempts.push(*attempt);
        Ok(())
    }
    fn commitment(&mut self, bytes: &[u8; CHECKPOINT_BYTES]) -> [u8; 32] {
        // Not cryptography: this deliberately weak fake detects test mutations only.
        let mut out = [0u8; 32];
        for (i, b) in bytes.iter().enumerate() {
            out[i % 32] = out[i % 32].wrapping_add(*b);
        }
        out
    }
    fn stage_chunk(&mut self, m: &Manifest, index: usize, bytes: &[u8]) -> Result<(), StoreError> {
        if self.fail_at == Some(index) {
            return Err(StoreError::Backend);
        }
        if index == 0 {
            self.staged.clear();
        }
        self.staged.push(Chunk {
            manifest: m.clone(),
            index,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }
    fn flush_chunks(&mut self, _: &Manifest) -> Result<(), StoreError> {
        if self.fail_flush {
            Err(StoreError::Backend)
        } else {
            Ok(())
        }
    }
    fn verify_candidate(&mut self, m: &Manifest) -> Result<(), StoreError> {
        self.verifies += 1;
        if self.staged.len() != CHUNKS
            || self.staged.iter().enumerate().any(|(i, c)| {
                c.manifest != *m
                    || c.index != i
                    || c.bytes.len() != if i + 1 == CHUNKS { 16 } else { 128 }
            })
        {
            return Err(StoreError::Invalid);
        }
        let mut bytes = [0; CHECKPOINT_BYTES];
        for chunk in &self.staged {
            let start = chunk.index * host_store::CHUNK_BYTES;
            bytes[start..start + chunk.bytes.len()].copy_from_slice(&chunk.bytes);
        }
        if self.commitment(&bytes) != m.commitment {
            return Err(StoreError::Invalid);
        }
        Ok(())
    }
    fn publish(&mut self, m: &Manifest) -> Result<(), StoreError> {
        if self.unresolved.is_some() {
            return Err(StoreError::Unknown);
        }
        if self.fenced.contains(&m.attempt) {
            return Err(StoreError::Invalid);
        }
        if self.staged.len() != CHUNKS || self.staged.iter().any(|c| c.manifest != *m) {
            return Err(StoreError::Invalid);
        }
        if self.unknown {
            self.unresolved = Some(m.attempt);
            if !self.unknown_applied {
                self.delayed = Some((m.clone(), self.staged.clone()));
            }
            if self.unknown_applied {
                self.preserved = self.published.clone();
                self.published = Some((m.clone(), self.staged.clone()));
            }
            return Err(StoreError::Unknown);
        }
        self.published = Some((m.clone(), self.staged.clone()));
        Ok(())
    }
    fn fence_attempt(&mut self, m: &Manifest) -> Result<(), StoreError> {
        if self.fail_fence {
            return Err(StoreError::Backend);
        }
        if self.fenced.contains(&m.attempt) {
            return Ok(());
        }
        if self.unresolved != Some(m.attempt) {
            return Err(StoreError::Invalid);
        }
        self.fenced.push(m.attempt);
        Ok(())
    }
    fn finalize_attempt(
        &mut self,
        candidate: &Manifest,
        prior: &Option<Manifest>,
        selected: &Option<Manifest>,
    ) -> Result<(), StoreError> {
        if self.fail_finalize {
            return Err(StoreError::Backend);
        }
        if self.unresolved != Some(candidate.attempt) || !self.fenced.contains(&candidate.attempt) {
            return Err(StoreError::Invalid);
        }
        if self.published.as_ref().map(|p| &p.0) != selected.as_ref() {
            return Err(StoreError::Stale);
        }
        let candidate_data = if self.published.as_ref().is_some_and(|p| &p.0 == candidate) {
            self.published.clone().unwrap()
        } else {
            (candidate.clone(), self.staged.clone())
        };
        let prior_data = match prior {
            None => None,
            Some(m) if self.published.as_ref().is_some_and(|p| &p.0 == m) => self.published.clone(),
            Some(m) if self.preserved.as_ref().is_some_and(|p| &p.0 == m) => self.preserved.clone(),
            Some(_) => return Err(StoreError::Stale),
        };
        if !self.snapshot_valid(&candidate_data)
            || prior_data.as_ref().is_some_and(|p| !self.snapshot_valid(p))
        {
            return Err(StoreError::Invalid);
        }
        if let Some(data) = prior_data {
            self.retained.push(data);
        }
        self.retained.push(candidate_data);
        self.unresolved = None;
        Ok(())
    }
    fn open_manifest(&mut self) -> Result<Manifest, StoreError> {
        self.published
            .as_ref()
            .map(|p| p.0.clone())
            .ok_or(StoreError::Backend)
    }
    fn check_chunk_count(&mut self, _: &Manifest) -> Result<(), StoreError> {
        if self.published.as_ref().unwrap().1.len() == CHUNKS {
            Ok(())
        } else {
            Err(StoreError::Invalid)
        }
    }
    fn open_chunk(&mut self, m: &Manifest, index: usize, out: &mut [u8]) -> Result<(), StoreError> {
        let c = &self.published.as_ref().unwrap().1[index];
        if c.manifest != *m || c.index != index || c.bytes.len() != out.len() {
            return Err(StoreError::Invalid);
        }
        out.copy_from_slice(&c.bytes);
        Ok(())
    }
}
#[derive(Clone, Default)]
struct CountingRemote {
    opened: Rc<Cell<usize>>,
    heads: Rc<Cell<usize>>,
    reads: Rc<Cell<usize>>,
    closes: Rc<Cell<usize>>,
    stages: Rc<Cell<usize>>,
    commits: Rc<Cell<usize>>,
}
impl Remote for CountingRemote {
    fn caller(&self) -> Principal {
        Principal([37; 32])
    }
    fn open(&mut self, namespace: &[u8; 32]) -> Result<Session, Error> {
        assert_eq!(namespace, &[3; 32]);
        self.opened.set(self.opened.get() + 1);
        Session::from_bytes(&[1; 40])
    }
    fn stage(&mut self, _: Session, _: usize, _: u64, _: &[u8]) -> Result<Upload, Error> {
        self.stages.set(self.stages.get() + 1);
        Err(Error::Invalid)
    }
    fn commit(&mut self, _: Session, _: Upload) -> Result<u64, Error> {
        self.commits.set(self.commits.get() + 1);
        Err(Error::Invalid)
    }
    fn cancel(&mut self, _: Session, _: Upload) -> Result<(), Error> {
        Err(Error::Invalid)
    }
    fn close(&mut self, _: Session) -> Result<(), Error> {
        self.closes.set(self.closes.get() + 1);
        Ok(())
    }
    fn head(&mut self, _: Session, object: usize) -> Result<u64, Error> {
        assert_eq!(object, 0);
        self.heads.set(self.heads.get() + 1);
        Ok(1)
    }
    fn read(
        &mut self,
        _: Session,
        object: usize,
        revision: u64,
        out: &mut [u8],
    ) -> Result<usize, Error> {
        assert_eq!((object, revision), (0, 1));
        self.reads.set(self.reads.get() + 1);
        let bytes = b"encrypted object";
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }
}

struct UnreachableAuthority;
impl IndependentAuthority for UnreachableAuthority {
    fn observe(
        &mut self,
        _: &Option<Manifest>,
        _: &Manifest,
    ) -> Result<Option<Manifest>, StoreError> {
        panic!("authority must not be read when fencing fails")
    }
}
/// Test-only trusted fixture: identity is held outside the Remote.
struct Connection {
    route: Option<Binding>,
    remote: CountingRemote,
}
impl TrustedPeerConnection for Connection {
    type Remote = CountingRemote;
    fn verified_remote(&mut self, expected: &Binding) -> Result<&mut Self::Remote, StoreError> {
        match &self.route {
            Some(route) if route == expected => Ok(&mut self.remote),
            Some(_) => Err(StoreError::Unauthorized),
            None => Err(StoreError::Backend),
        }
    }
}

fn valid_connection() -> Connection {
    Connection {
        route: Some(binding()),
        remote: CountingRemote::default(),
    }
}
struct Independent {
    result: Result<Option<Manifest>, StoreError>,
}
impl IndependentAuthority for Independent {
    fn observe(
        &mut self,
        _: &Option<Manifest>,
        _: &Manifest,
    ) -> Result<Option<Manifest>, StoreError> {
        self.result.clone()
    }
}

fn client() -> Client {
    let mut client = Client::new();
    client.enqueue([3; 32], 0, 0, b"encrypted object").unwrap();
    client
}
fn stage(
    coordinator: &Coordinator<Connection>,
    fake: &mut Fake,
    publication: u64,
    attempt: u8,
) -> host_store::Candidate {
    coordinator
        .stage(
            &client(),
            publication,
            [attempt; 32],
            &mut Auth { allowed: true },
            fake,
        )
        .unwrap()
}
fn setup() -> (Coordinator<Connection>, Fake) {
    let mut coordinator = Coordinator::new(binding(), [11; 32], valid_connection());
    let mut fake = Fake::default();
    let candidate = stage(&coordinator, &mut fake, 1, 1);
    coordinator
        .publish(candidate, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    (coordinator, fake)
}
fn prepared(coordinator: &Coordinator<Connection>, fake: &mut Fake) -> host_store::Prepared {
    coordinator
        .prepare(&mut Auth { allowed: true }, fake)
        .unwrap()
}

#[test]
fn restored_queue_reconciles_via_head_without_implicit_commit() {
    let remote = CountingRemote::default();
    let mut coordinator = Coordinator::new(
        binding(),
        [11; 32],
        Connection {
            route: Some(binding()),
            remote: remote.clone(),
        },
    );
    let mut fake = Fake::default();
    let candidate = stage(&coordinator, &mut fake, 1, 1);
    coordinator
        .publish(candidate, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    let prep = prepared(&coordinator, &mut fake);
    let handle = coordinator
        .handoff(prep, &mut Auth { allowed: true })
        .unwrap();
    assert_eq!(coordinator.reconcile(handle, 0), Ok(Ok(State::Observed(1))));
    assert_eq!(
        (
            remote.opened.get(),
            remote.heads.get(),
            remote.reads.get(),
            remote.closes.get()
        ),
        (1, 1, 1, 1)
    );
    assert_eq!((remote.stages.get(), remote.commits.get()), (0, 0));
}

#[test]
fn revocation_serializes_with_publication_in_both_orders() {
    let (mut coordinator, mut fake) = setup();
    let staged = stage(&coordinator, &mut fake, 2, 2);
    coordinator.revoke(&mut fake).unwrap();
    assert!(matches!(
        coordinator.publish(staged, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unauthorized)
    ));
    assert_eq!(fake.published.as_ref().unwrap().0.publication, 1);
    let (mut coordinator, mut fake) = setup();
    let staged = stage(&coordinator, &mut fake, 2, 2);
    coordinator
        .publish(staged, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    coordinator.revoke(&mut fake).unwrap();
    assert!(matches!(
        coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unauthorized)
    ));
    assert_eq!(fake.published.as_ref().unwrap().0.publication, 2);
}

#[test]
fn restore_handoff_races_revoke_and_invalidates_stale_handles() {
    let (mut coordinator, mut fake) = setup();
    let prep = prepared(&coordinator, &mut fake);
    coordinator.revoke(&mut fake).unwrap();
    assert!(matches!(
        coordinator.handoff(prep, &mut Auth { allowed: true }),
        Err(StoreError::Unauthorized)
    ));
    let (mut coordinator, mut fake) = setup();
    let prep = prepared(&coordinator, &mut fake);
    let handle = coordinator
        .handoff(prep, &mut Auth { allowed: true })
        .unwrap();
    let mut out = [0; 32];
    assert_eq!(coordinator.read(handle, 0, &mut out), Ok(16));
    coordinator.revoke(&mut fake).unwrap();
    assert!(matches!(
        coordinator.read(handle, 0, &mut out),
        Err(StoreError::Unauthorized)
    ));
    assert!(matches!(
        coordinator.reconcile(handle, 0),
        Err(StoreError::Unauthorized)
    ));
}

#[test]
fn replay_with_same_number_but_different_attempt_and_commitment_fails() {
    let (mut coordinator, mut fake) = setup();
    let old = fake.published.clone().unwrap();
    let staged = stage(&coordinator, &mut fake, 2, 2);
    coordinator
        .publish(staged, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    let valid = fake.published.clone().unwrap();
    let mut forged = old;
    forged.0.publication = valid.0.publication;
    forged.0.commitment[0] ^= 1;
    // Simulate an independently authenticated old value at the same number.
    for chunk in &mut forged.1 {
        chunk.manifest = forged.0.clone();
    }
    fake.published = Some(forged);
    assert!(matches!(
        coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Stale)
    ));
    fake.published = Some(valid);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

#[test]
fn restore_handoff_rechecks_exact_anchor_and_full_object_mapping() {
    let (mut coordinator, mut fake) = setup();
    let prep = prepared(&coordinator, &mut fake);
    let staged = stage(&coordinator, &mut fake, 2, 2);
    coordinator
        .publish(staged, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    assert!(matches!(
        coordinator.handoff(prep, &mut Auth { allowed: true }),
        Err(StoreError::Stale)
    ));
    let mut other = binding();
    other.objects[0][31] ^= 1;
    let foreign = Coordinator::new(other, [12; 32], valid_connection());
    assert!(matches!(
        foreign.stage(
            &client(),
            1,
            [3; 32],
            &mut Auth { allowed: true },
            &mut fake
        ),
        Err(StoreError::Unauthorized)
    ));
}

#[test]
fn chunk_substitution_reorder_truncation_and_corruption_are_rejected() {
    let (coordinator, mut fake) = setup();
    let original = fake.published.clone().unwrap();
    fake.published.as_mut().unwrap().1.swap(0, 1);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_err());
    fake.published = Some(original.clone());
    fake.published.as_mut().unwrap().1.pop();
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_err());
    fake.published = Some(original.clone());
    fake.published.as_mut().unwrap().1[0].bytes[0] ^= 1;
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_err());
    fake.published = Some(original.clone());
    fake.published.as_mut().unwrap().1[0].bytes.pop();
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_err());
    fake.published = Some(original);
    fake.published.as_mut().unwrap().1[0].manifest.attempt = [42; 32];
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_err());
}

#[test]
fn unknown_publication_requires_exact_independent_resolution_of_both_outcomes() {
    for applied in [false, true] {
        let (mut coordinator, mut fake) = setup();
        let prior = fake.published.as_ref().unwrap().0.clone();
        let staged = stage(&coordinator, &mut fake, 2, 2);
        let candidate = fake.staged[0].manifest.clone();
        fake.unknown = true;
        fake.unknown_applied = applied;
        assert!(matches!(
            coordinator.publish(staged, &mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        assert!(matches!(
            coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        assert!(matches!(
            coordinator.resolve_unknown(
                &mut Independent {
                    result: Err(StoreError::Backend)
                },
                &mut fake
            ),
            Err(StoreError::Backend)
        ));
        assert!(matches!(
            coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        let mut wrong = candidate.clone();
        wrong.attempt[0] ^= 1;
        assert!(matches!(
            coordinator.resolve_unknown(
                &mut Independent {
                    result: Ok(Some(wrong))
                },
                &mut fake
            ),
            Err(StoreError::Stale)
        ));
        assert!(matches!(
            coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        if applied {
            assert!(fake.preserved.is_some());
        }
        coordinator
            .resolve_unknown(
                &mut Independent {
                    result: Ok(Some(if applied { candidate } else { prior })),
                },
                &mut fake,
            )
            .unwrap();
        let prep = prepared(&coordinator, &mut fake);
        assert!(coordinator
            .handoff(prep, &mut Auth { allowed: true })
            .is_ok());
    }
}

#[test]
fn new_publication_invalidates_old_handles_and_old_candidates() {
    let (mut coordinator, mut fake) = setup();
    let p = prepared(&coordinator, &mut fake);
    let h = coordinator.handoff(p, &mut Auth { allowed: true }).unwrap();
    let stale = stage(&coordinator, &mut fake, 2, 2);
    let newer = stage(&coordinator, &mut fake, 3, 3);
    coordinator
        .publish(newer, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    assert!(matches!(
        coordinator.publish(stale, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Stale)
    ));
    assert!(matches!(
        coordinator.read(h, 0, &mut [0; 32]),
        Err(StoreError::Unauthorized)
    ));
}

#[test]
fn staging_or_flush_failure_never_changes_authoritative_checkpoint() {
    let (coordinator, mut fake) = setup();
    for (i, cut) in [0, 33, 66].into_iter().enumerate() {
        fake.fail_at = Some(cut);
        assert!(matches!(
            coordinator.stage(
                &client(),
                2,
                [2 + i as u8; 32],
                &mut Auth { allowed: true },
                &mut fake
            ),
            Err(StoreError::Backend)
        ));
        assert!(coordinator
            .prepare(&mut Auth { allowed: true }, &mut fake)
            .is_ok());
    }
    fake.fail_at = None;
    fake.fail_flush = true;
    assert!(matches!(
        coordinator.stage(
            &client(),
            2,
            [5; 32],
            &mut Auth { allowed: true },
            &mut fake
        ),
        Err(StoreError::Backend)
    ));
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

#[test]
fn competing_candidates_cannot_publish_another_attempts_chunks() {
    // The fake has only one staging area; replacement must fail closed.
    for first_a in [true, false] {
        let (mut coordinator, mut fake) = setup();
        let a = stage(&coordinator, &mut fake, 2, 2);
        let b = stage(&coordinator, &mut fake, 3, 3);
        if first_a {
            assert!(matches!(
                coordinator.publish(a, &mut Auth { allowed: true }, &mut fake),
                Err(StoreError::Invalid)
            ));
            assert_eq!(fake.published.as_ref().unwrap().0.publication, 1);
            assert!(coordinator
                .prepare(&mut Auth { allowed: true }, &mut fake)
                .is_ok());
            coordinator
                .publish(b, &mut Auth { allowed: true }, &mut fake)
                .unwrap();
            assert_eq!(fake.published.as_ref().unwrap().0.publication, 3);
        } else {
            coordinator
                .publish(b, &mut Auth { allowed: true }, &mut fake)
                .unwrap();
            assert!(matches!(
                coordinator.publish(a, &mut Auth { allowed: true }, &mut fake),
                Err(StoreError::Stale)
            ));
            assert_eq!(fake.published.as_ref().unwrap().0.publication, 3);
        }
    }
}

#[test]
fn foreign_coordinator_handle_is_not_accepted() {
    let (mut a, mut fake_a) = setup();
    let prep_a = prepared(&a, &mut fake_a);
    let handle_a = a.handoff(prep_a, &mut Auth { allowed: true }).unwrap();
    let mut b = Coordinator::new(binding(), [12; 32], valid_connection());
    let mut fake_b = Fake::default();
    let staged = stage(&b, &mut fake_b, 1, 1);
    b.publish(staged, &mut Auth { allowed: true }, &mut fake_b)
        .unwrap();
    let prep_b = prepared(&b, &mut fake_b);
    let _handle_b = b.handoff(prep_b, &mut Auth { allowed: true }).unwrap();
    assert!(matches!(
        b.read(handle_a, 0, &mut [0; 32]),
        Err(StoreError::Unauthorized)
    ));
}

#[test]
fn changed_candidate_after_flush_cannot_replace_previous_checkpoint() {
    let (mut coordinator, mut fake) = setup();
    let candidate = stage(&coordinator, &mut fake, 2, 2);
    fake.staged[1].bytes[3] ^= 1;
    assert!(matches!(
        coordinator.publish(candidate, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Invalid)
    ));
    assert_eq!(fake.published.as_ref().unwrap().0.publication, 1);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

#[test]
fn foreign_candidate_cannot_cross_coordinator_after_revoke() {
    let mut a = Coordinator::new(binding(), [11; 32], valid_connection());
    let mut b = Coordinator::new(binding(), [12; 32], valid_connection());
    let mut fake = Fake::default();
    let candidate = stage(&a, &mut fake, 1, 1);
    a.revoke(&mut fake).unwrap();
    assert!(matches!(
        b.publish(candidate, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unauthorized)
    ));
    assert!(fake.published.is_none());
}

#[test]
fn uncertain_attempt_must_be_fenced_before_prior_resolution_and_later_publish() {
    let (mut coordinator, mut fake) = setup();
    let prior = fake.published.as_ref().unwrap().0.clone();
    let candidate = stage(&coordinator, &mut fake, 2, 2);
    fake.unknown = true;
    fake.unknown_applied = false;
    assert!(matches!(
        coordinator.publish(candidate, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unknown)
    ));
    let mut authority = Independent {
        result: Ok(Some(prior.clone())),
    };
    fake.fail_fence = true;
    assert!(matches!(
        coordinator.resolve_unknown(&mut UnreachableAuthority, &mut fake),
        Err(StoreError::Backend)
    ));
    assert!(matches!(
        coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unknown)
    ));
    fake.fail_fence = false;
    coordinator
        .resolve_unknown(&mut authority, &mut fake)
        .unwrap();
    fake.unknown = false;
    let later = stage(&coordinator, &mut fake, 3, 3);
    coordinator
        .publish(later, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    // A late backend completion is rejected after the attempt became terminal.
    assert!(!fake.complete_delayed());
    assert_eq!(fake.published.as_ref().unwrap().0.publication, 3);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

#[test]
fn prepared_restore_cannot_cross_coordinator_after_revoke() {
    let (mut a, mut fake_a) = setup();
    let prepared_a = prepared(&a, &mut fake_a);
    let mut b = Coordinator::new(binding(), [12; 32], valid_connection());
    let mut fake_b = Fake::default();
    let candidate_b = stage(&b, &mut fake_b, 1, 1);
    b.publish(candidate_b, &mut Auth { allowed: true }, &mut fake_b)
        .unwrap();
    a.revoke(&mut fake_a).unwrap();
    assert!(matches!(
        b.handoff(prepared_a, &mut Auth { allowed: true }),
        Err(StoreError::Unauthorized)
    ));
}

#[test]
fn publication_reauthorizes_at_decision_without_backend_or_anchor_change() {
    let (mut coordinator, mut fake) = setup();
    let old = fake.published.as_ref().unwrap().0.clone();
    let candidate = stage(&coordinator, &mut fake, 2, 2);
    let verifies_before = fake.verifies;
    assert!(matches!(
        coordinator.publish(candidate, &mut Auth { allowed: false }, &mut fake),
        Err(StoreError::Unauthorized)
    ));
    assert_eq!(fake.verifies, verifies_before);
    assert!(fake.published.as_ref().unwrap().0 == old);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

struct DenySecondAuthorization(usize);
impl Authorization for DenySecondAuthorization {
    fn authorize(&mut self, _: &Binding, _: &Checkpoint) -> bool {
        self.0 += 1;
        self.0 == 1
    }
}
#[test]
fn publication_rechecks_authority_after_candidate_verification() {
    let (mut coordinator, mut fake) = setup();
    let old = fake.published.as_ref().unwrap().0.clone();
    let candidate = stage(&coordinator, &mut fake, 2, 2);
    assert!(matches!(
        coordinator.publish(candidate, &mut DenySecondAuthorization(0), &mut fake),
        Err(StoreError::Unauthorized)
    ));
    assert!(fake.published.as_ref().unwrap().0 == old);
    assert!(coordinator
        .prepare(&mut Auth { allowed: true }, &mut fake)
        .is_ok());
}

#[test]
fn revoke_fences_unknown_attempt_and_retries_without_restoring_access() {
    for applied in [false, true] {
        let (mut coordinator, mut fake) = setup();
        let prior = fake.published.clone().unwrap();
        let staged = stage(&coordinator, &mut fake, 2, 2);
        fake.unknown = true;
        fake.unknown_applied = applied;
        assert!(matches!(
            coordinator.publish(staged, &mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        fake.fail_fence = true;
        assert!(matches!(
            coordinator.revoke(&mut fake),
            Err(StoreError::Backend)
        ));
        assert!(matches!(
            coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unauthorized)
        ));
        assert!(matches!(
            coordinator.resolve_unknown(&mut UnreachableAuthority, &mut fake),
            Err(StoreError::Unauthorized)
        ));
        if applied {
            assert!(fake.preserved.as_ref().unwrap().0 == prior.0);
        } else {
            assert!(fake.published.as_ref().unwrap().0 == prior.0);
        }
        fake.fail_fence = false;
        assert_eq!(coordinator.revoke(&mut fake), Ok(()));
        // The fenced delayed write must not become visible after revoke success.
        assert!(!fake.complete_delayed());
        assert!(matches!(
            coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unauthorized)
        ));
        assert_eq!(
            fake.published.as_ref().unwrap().0.publication,
            if applied { 2 } else { 1 }
        );
        // Repeated cleanup is safe and does not reauthorize or replay.
        assert_eq!(coordinator.revoke(&mut fake), Ok(()));
    }
}

#[test]
fn revoked_uncertain_checkpoint_never_releases_gate_after_fence() {
    for applied in [false, true] {
        let mut original = Coordinator::new(binding(), [11; 32], valid_connection());
        let mut fake = Fake::default();
        let mut prior_client = Client::new();
        prior_client.enqueue([3; 32], 0, 0, b"prior bytes").unwrap();
        let prior = original
            .stage(
                &prior_client,
                1,
                [1; 32],
                &mut Auth { allowed: true },
                &mut fake,
            )
            .unwrap();
        original
            .publish(prior, &mut Auth { allowed: true }, &mut fake)
            .unwrap();
        let prior_bytes = fake.published.as_ref().unwrap().1[0].bytes.clone();
        let mut candidate_client = Client::new();
        candidate_client
            .enqueue([3; 32], 0, 0, b"different candidate bytes")
            .unwrap();
        let candidate = original
            .stage(
                &candidate_client,
                2,
                [2; 32],
                &mut Auth { allowed: true },
                &mut fake,
            )
            .unwrap();
        let candidate_bytes = fake.staged[0].bytes.clone();
        assert_ne!(prior_bytes, candidate_bytes);
        let candidate_manifest = fake.staged[0].manifest.clone();
        fake.unknown = true;
        fake.unknown_applied = applied;
        assert!(matches!(
            original.publish(candidate, &mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        fake.fail_fence = true;
        assert!(matches!(
            original.revoke(&mut fake),
            Err(StoreError::Backend)
        ));
        let replacement = Coordinator::new(binding(), [12; 32], valid_connection());
        let mut replacement_client = Client::new();
        replacement_client
            .enqueue([3; 32], 0, 0, b"distinct replacement bytes")
            .unwrap();
        assert!(matches!(
            replacement.stage(
                &replacement_client,
                3,
                [3; 32],
                &mut Auth { allowed: true },
                &mut fake
            ),
            Err(StoreError::Unknown)
        ));
        assert!(matches!(
            Backend::publish(&mut fake, &candidate_manifest),
            Err(StoreError::Unknown)
        ));
        fake.fail_fence = false;
        original.revoke(&mut fake).unwrap();
        assert_eq!(fake.unresolved, Some([2; 32]));
        assert!(matches!(
            replacement.stage(
                &replacement_client,
                3,
                [3; 32],
                &mut Auth { allowed: true },
                &mut fake
            ),
            Err(StoreError::Unknown)
        ));
        assert!(matches!(
            Backend::publish(&mut fake, &candidate_manifest),
            Err(StoreError::Unknown)
        ));
        assert!(!fake.complete_delayed());
        if applied {
            assert_eq!(fake.published.as_ref().unwrap().1[0].bytes, candidate_bytes);
            assert_eq!(fake.preserved.as_ref().unwrap().1[0].bytes, prior_bytes);
        } else {
            assert_eq!(fake.published.as_ref().unwrap().1[0].bytes, prior_bytes);
            assert_eq!(fake.staged[0].bytes, candidate_bytes);
        }
        assert!(matches!(
            original.resolve_unknown(&mut UnreachableAuthority, &mut fake),
            Err(StoreError::Unauthorized)
        ));
    }
}

#[test]
fn live_unknown_finalization_retains_exact_selected_bytes_before_gate_release() {
    for applied in [false, true] {
        let mut coordinator = Coordinator::new(binding(), [11; 32], valid_connection());
        let mut fake = Fake::default();
        let mut prior_client = Client::new();
        prior_client.enqueue([3; 32], 0, 0, b"prior bytes").unwrap();
        let prior_stage = coordinator
            .stage(
                &prior_client,
                1,
                [1; 32],
                &mut Auth { allowed: true },
                &mut fake,
            )
            .unwrap();
        coordinator
            .publish(prior_stage, &mut Auth { allowed: true }, &mut fake)
            .unwrap();
        let prior = fake.published.clone().unwrap();
        let mut candidate_client = Client::new();
        candidate_client
            .enqueue([3; 32], 0, 0, b"different candidate bytes")
            .unwrap();
        let candidate_stage = coordinator
            .stage(
                &candidate_client,
                2,
                [2; 32],
                &mut Auth { allowed: true },
                &mut fake,
            )
            .unwrap();
        let candidate = (fake.staged[0].manifest.clone(), fake.staged.clone());
        fake.unknown = true;
        fake.unknown_applied = applied;
        assert!(matches!(
            coordinator.publish(candidate_stage, &mut Auth { allowed: true }, &mut fake),
            Err(StoreError::Unknown)
        ));
        let chosen = if applied { &candidate } else { &prior };
        fake.fail_finalize = true;
        let mut authority = Independent {
            result: Ok(Some(chosen.0.clone())),
        };
        assert!(matches!(
            coordinator.resolve_unknown(&mut authority, &mut fake),
            Err(StoreError::Backend)
        ));
        assert_eq!(fake.unresolved, Some([2; 32]));
        fake.fail_finalize = false;
        coordinator
            .resolve_unknown(&mut authority, &mut fake)
            .unwrap();
        assert_eq!(fake.unresolved, None);
        assert!(fake
            .retained
            .iter()
            .any(|p| p.0 == prior.0 && p.1[0].bytes == prior.1[0].bytes));
        assert!(fake
            .retained
            .iter()
            .any(|p| p.0 == candidate.0 && p.1[0].bytes == candidate.1[0].bytes));
        assert!(!fake.complete_delayed());
        assert!(coordinator
            .prepare(&mut Auth { allowed: true }, &mut fake)
            .is_ok());
    }
}

#[test]
fn prior_resolution_retains_unapplied_candidate_after_client_is_dropped_and_new_publish() {
    let mut coordinator = Coordinator::new(binding(), [11; 32], valid_connection());
    let mut fake = Fake::default();
    let mut prior_client = Client::new();
    prior_client.enqueue([3; 32], 0, 0, b"prior bytes").unwrap();
    let p = coordinator
        .stage(
            &prior_client,
            1,
            [1; 32],
            &mut Auth { allowed: true },
            &mut fake,
        )
        .unwrap();
    coordinator
        .publish(p, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    let prior = fake.published.clone().unwrap();
    let candidate = {
        let mut candidate_client = Client::new();
        candidate_client
            .enqueue([3; 32], 0, 0, b"unsent unique candidate bytes")
            .unwrap();
        coordinator
            .stage(
                &candidate_client,
                2,
                [2; 32],
                &mut Auth { allowed: true },
                &mut fake,
            )
            .unwrap()
    };
    let candidate_bytes: Vec<u8> = fake
        .staged
        .iter()
        .flat_map(|c| c.bytes.iter().copied())
        .collect();
    let candidate_manifest = fake.staged[0].manifest.clone();
    fake.unknown = true;
    fake.unknown_applied = false;
    assert!(matches!(
        coordinator.publish(candidate, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unknown)
    ));
    coordinator
        .resolve_unknown(
            &mut Independent {
                result: Ok(Some(prior.0.clone())),
            },
            &mut fake,
        )
        .unwrap();
    fake.unknown = false;
    let mut next_client = Client::new();
    next_client
        .enqueue([3; 32], 0, 0, b"next distinct data")
        .unwrap();
    let next = coordinator
        .stage(
            &next_client,
            3,
            [3; 32],
            &mut Auth { allowed: true },
            &mut fake,
        )
        .unwrap();
    coordinator
        .publish(next, &mut Auth { allowed: true }, &mut fake)
        .unwrap();
    let kept = fake
        .retained
        .iter()
        .find(|p| p.0 == candidate_manifest)
        .unwrap();
    let kept_bytes: Vec<u8> = kept
        .1
        .iter()
        .flat_map(|c| c.bytes.iter().copied())
        .collect();
    assert_eq!(kept_bytes, candidate_bytes);
    assert!(fake.retained.iter().any(|p| p.0 == prior.0));
}

#[test]
fn missing_or_damaged_candidate_keeps_gate_closed_even_when_prior_was_selected() {
    let (mut coordinator, mut fake) = setup();
    let prior = fake.published.as_ref().unwrap().0.clone();
    let candidate = stage(&coordinator, &mut fake, 2, 2);
    fake.unknown = true;
    fake.unknown_applied = false;
    assert!(matches!(
        coordinator.publish(candidate, &mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unknown)
    ));
    fake.staged[0].bytes[8] ^= 1;
    assert!(matches!(
        coordinator.resolve_unknown(
            &mut Independent {
                result: Ok(Some(prior))
            },
            &mut fake
        ),
        Err(StoreError::Invalid)
    ));
    assert_eq!(fake.unresolved, Some([2; 32]));
    assert!(matches!(
        coordinator.prepare(&mut Auth { allowed: true }, &mut fake),
        Err(StoreError::Unknown)
    ));
}

#[test]
fn foreign_or_unavailable_peer_fails_before_any_remote_operation() {
    let mut wrong_peer = binding();
    wrong_peer.peer = [99; 32];
    let mut wrong_namespace = binding();
    wrong_namespace.namespace = [99; 32];
    let mut wrong_generation = binding();
    wrong_generation.grant_generation += 1;
    let mut wrong_scope = binding();
    wrong_scope.grant_scope = [99; 32];
    for route in [
        Some(wrong_peer),
        Some(wrong_namespace),
        Some(wrong_generation),
        Some(wrong_scope),
        None,
    ] {
        let remote = CountingRemote::default();
        let mut coordinator = Coordinator::new(
            binding(),
            [11; 32],
            Connection {
                route,
                remote: remote.clone(),
            },
        );
        let mut fake = Fake::default();
        let candidate = stage(&coordinator, &mut fake, 1, 1);
        coordinator
            .publish(candidate, &mut Auth { allowed: true }, &mut fake)
            .unwrap();
        let prep = prepared(&coordinator, &mut fake);
        let handle = coordinator
            .handoff(prep, &mut Auth { allowed: true })
            .unwrap();
        let mut before = [0; 32];
        let len = coordinator.read(handle, 0, &mut before).unwrap();
        assert!(matches!(
            coordinator.reconcile(handle, 0),
            Err(StoreError::Unauthorized | StoreError::Backend)
        ));
        assert_eq!(
            (
                remote.opened.get(),
                remote.heads.get(),
                remote.reads.get(),
                remote.stages.get(),
                remote.commits.get()
            ),
            (0, 0, 0, 0, 0)
        );
        let mut after = [0; 32];
        assert_eq!(coordinator.read(handle, 0, &mut after), Ok(len));
        assert_eq!(after, before);
        assert_eq!(coordinator.state(handle, 0), Ok(State::NeedsReconcile));
    }
}
