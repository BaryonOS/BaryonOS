//! B02 contract item 4: write-ahead GENERATE/REVOKE on the host. A crash is
//! modelled as a storage adapter that either did or did not make the blob
//! durable, after which the `Service` is dropped without using the result and
//! a fresh instance restores the last durable blob. Deterministic entropy is a
//! test fixture, not an entropy claim; no native disk or VM is exercised.

use keyring_core::{
    persistent::{KeyStateStore, SnapshotInfo, StoreError, WriteAheadError, PERSISTENT_BLOB},
    records::{RecordContext, RecordKind},
    wire::{self, Header, GENERATE, REVOKE},
    EntropySource, Service, Status, DOMAINS, MAX_PLAINTEXT, MAX_SEALED, RECOVERY_BLOB,
    RESEED_INTERVAL, SECRET_BYTES,
};

/// Entropy-accounted operations performed by `committed_base`: allocate,
/// export for the commit, and one seal. With `RESEED_INTERVAL`, this places
/// the failing reseed exactly at the export boundary in G2 and R1.
const BASE_OPS: u32 = 3;

const VOLUME: [u8; 16] = *b"write-ahead-vol1";
const SECRET: [u8; SECRET_BYTES] = [0x3c; SECRET_BYTES];
const PLAINTEXT: &[u8] = b"secret-plaintext-marker-0001";

struct Entropy {
    state: u64,
    fail: bool,
}
impl EntropySource for Entropy {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        if self.fail {
            return Err(Status::Entropy);
        }
        for byte in out {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            *byte = (self.state >> 24) as u8;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Ok,
    /// Certain failure: nothing written.
    Fail,
    /// Uncertain outcome; the write did not land.
    UnknownLost,
    /// Uncertain outcome; the write did land.
    UnknownLanded,
    /// Process dies before the blob is durable.
    CrashBeforeWrite,
    /// Process dies after the blob is durable, before `commit` returns.
    CrashAfterWrite,
}

struct Store {
    mode: Mode,
    calls: usize,
    durable: Option<[u8; PERSISTENT_BLOB]>,
    durable_generation: u64,
    offered: Vec<[u8; PERSISTENT_BLOB]>,
}
impl Store {
    fn new() -> Self {
        Self {
            mode: Mode::Ok,
            calls: 0,
            durable: None,
            durable_generation: 0,
            offered: Vec::new(),
        }
    }
    fn persist(&mut self, generation: u64, blob: &[u8; PERSISTENT_BLOB]) {
        assert!(generation > self.durable_generation || self.durable.is_none());
        self.durable = Some(*blob);
        self.durable_generation = generation;
    }
}
impl KeyStateStore for Store {
    fn commit(&mut self, generation: u64, blob: &[u8; PERSISTENT_BLOB]) -> Result<(), StoreError> {
        self.calls += 1;
        self.offered.push(*blob);
        match self.mode {
            Mode::Ok => {
                self.persist(generation, blob);
                Ok(())
            }
            Mode::Fail => Err(StoreError::Failed),
            Mode::UnknownLost | Mode::CrashBeforeWrite => Err(StoreError::Unknown),
            Mode::UnknownLanded | Mode::CrashAfterWrite => {
                self.persist(generation, blob);
                Err(StoreError::Unknown)
            }
        }
    }
}

fn ready(seed: u64) -> (Service, Entropy) {
    let mut service = Service::new();
    let mut source = Entropy {
        state: seed,
        fail: false,
    };
    service.start(&mut source).unwrap();
    (service, source)
}

fn generate(s: &mut Service, store: &mut Store, e: &mut Entropy) -> Result<u8, WriteAheadError> {
    s.generate_durable(0, &VOLUME, &SECRET, store, e)
}

fn revoke(
    s: &mut Service,
    index: u8,
    store: &mut Store,
    e: &mut Entropy,
) -> Result<(), WriteAheadError> {
    s.revoke_durable(0, index, &VOLUME, &SECRET, store, e)
}

fn seal(s: &mut Service, index: u8, e: &mut Entropy) -> Result<Vec<u8>, Status> {
    let mut out = [0xa5u8; MAX_SEALED];
    let result = s.seal(0, index, PLAINTEXT, e, &mut out);
    if result.is_err() {
        assert_eq!(out, [0xa5u8; MAX_SEALED], "refused seal must not write");
    }
    result.map(|n| out[..n].to_vec())
}

fn unseal(s: &mut Service, index: u8, sealed: &[u8], e: &mut Entropy) -> Result<Vec<u8>, Status> {
    let mut out = [0u8; MAX_PLAINTEXT];
    s.unseal(0, index, sealed, e, &mut out)
        .map(|n| out[..n].to_vec())
}

/// Crash recovery: a fresh instance restores whatever is durable.
fn reopen(store: &Store) -> (Service, Entropy, SnapshotInfo) {
    let (mut s, mut e) = ready(0xfeed_0001);
    let info = match store.durable {
        Some(blob) => s
            .restore_persistent(&VOLUME, &SECRET, &blob, 0, &mut e)
            .unwrap(),
        None => s.snapshot_info(),
    };
    (s, e, info)
}

/// The instance with one confirmed handle (index 0) and its ciphertext.
fn committed_base() -> (Service, Entropy, Store, Vec<u8>) {
    let (mut s, mut e) = ready(0x1111);
    let mut store = Store::new();
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(0));
    let sealed = seal(&mut s, 0, &mut e).unwrap();
    (s, e, store, sealed)
}

/// Reopen and check: no confirmed or persisted index is reissued, and every
/// ciphertext released before the crash still opens under its handle.
fn check_after_crash(store: &Store, handed_out: &[u8], released: &[(u8, Vec<u8>)]) -> u8 {
    let (mut s, mut e, info) = reopen(store);
    let mut fresh = Store::new();
    let next = s
        .generate_durable(0, &VOLUME, &SECRET, &mut fresh, &mut e)
        .unwrap();
    assert!(handed_out.iter().all(|&index| next > index), "reissued");
    assert_eq!(u16::from(next), info.created[0], "persisted index reissued");
    for (index, sealed) in released {
        assert_eq!(unseal(&mut s, *index, sealed, &mut e).unwrap(), PLAINTEXT);
    }
    next
}

#[test]
fn seal_before_commit_is_refused_for_every_record_operation() {
    let (mut s, mut e) = ready(0x2222);
    let mut store = Store::new();
    store.mode = Mode::Fail;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    assert_eq!(store.calls, 1);
    assert!(store.durable.is_none());
    // The index exists in RAM (never reissued) but is not usable.
    assert_eq!(s.keys_created(0), 1);
    assert_eq!(seal(&mut s, 0, &mut e), Err(Status::NotReady));
    let fake = [0u8; 40];
    assert_eq!(unseal(&mut s, 0, &fake, &mut e), Err(Status::NotReady));
    let context = RecordContext {
        volume: VOLUME,
        domain: 0,
        object: 1,
        revision: 1,
        chunk: 0,
        length: PLAINTEXT.len() as u16,
        kind: RecordKind::Content,
        schema: 1,
    };
    let mut record = [0x77u8; MAX_SEALED];
    assert_eq!(
        s.seal_record(0, 0, context, PLAINTEXT, &mut e, &mut record),
        Err(Status::NotReady)
    );
    assert!(record.iter().all(|&b| b == 0x77));
    assert_eq!(s.counters.sealed, 0);
    // A later successful commit records both allocations; index 0 is not reissued.
    store.mode = Mode::Ok;
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    assert_eq!(store.calls, 2);
    assert!(seal(&mut s, 1, &mut e).is_ok());
    let (_, _, info) = reopen(&store);
    assert_eq!(info.created[0], 2);
}

#[test]
fn revocation_is_not_confirmed_before_commit() {
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::Fail;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    // Denied locally at once ...
    assert_eq!(seal(&mut s, 0, &mut e), Err(Status::Revoked));
    // ... but not durable: a cold instance still sees index 0 as active.
    let (mut cold, mut ce, info) = reopen(&store);
    assert_eq!(info.revoked[0], 0);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce).unwrap(), PLAINTEXT);
    // Explicit retry of the same revocation commits and confirms it.
    store.mode = Mode::Ok;
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    assert_eq!(store.calls, 3);
    let (mut cold, mut ce, info) = reopen(&store);
    assert_eq!(info.revoked[0], 1);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce), Err(Status::Revoked));
    // B02-T2: a retry of a confirmed revocation is confirmed again only by a
    // fresh commit through the passed store, at an advanced generation.
    let generation = store.durable_generation;
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    assert_eq!(store.calls, 4);
    assert!(store.durable_generation > generation, "generation reused");
}

/// M3: Failed revoke, then an unrelated successful commit makes it durable;
/// B02-T2: the retry still reports `Ok` only after its own store commit.
#[test]
fn revoke_retry_after_unrelated_commit_is_idempotent() {
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::Fail;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    store.mode = Mode::Ok;
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    assert_eq!(store.calls, 3);
    let generation = store.durable_generation;
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    assert_eq!(store.calls, 4, "Ok without a commit to the passed store");
    assert!(store.durable_generation > generation, "generation reused");
    let (mut cold, mut ce, info) = reopen(&store);
    assert_eq!(info.revoked[0], 1);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce), Err(Status::Revoked));
}

/// B02-T2: a revocation committed to store A is never confirmed for store B
/// from A's commit; `Ok` follows only a successful commit to B, a `Failed`
/// commit to B is not `Ok`, and an `Unknown` commit to B latches.
#[test]
fn already_revoked_index_is_confirmed_only_through_the_passed_store() {
    let (mut s, mut e, mut a, sealed) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut a, &mut e), Ok(()));
    let a_calls = a.calls;
    let mut b = Store::new();
    assert_eq!(revoke(&mut s, 0, &mut b, &mut e), Ok(()));
    assert_eq!(b.calls, 1, "Ok without a commit to B");
    assert!(
        b.durable_generation > a.durable_generation,
        "generation reused"
    );
    assert_eq!(a.calls, a_calls);
    let (mut cold, mut ce, info) = reopen(&b);
    assert_eq!(info.revoked[0], 1);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce), Err(Status::Revoked));

    // B certainly fails: not Ok, and B holds no revocation.
    let (mut s, mut e, mut a, sealed) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut a, &mut e), Ok(()));
    let mut b = Store::new();
    b.mode = Mode::Fail;
    assert_eq!(
        revoke(&mut s, 0, &mut b, &mut e),
        Err(WriteAheadError::Failed)
    );
    assert_eq!(b.calls, 1);
    assert!(b.durable.is_none());
    let (_, _, info) = reopen(&b);
    assert_eq!(info.revoked[0], 0);
    assert!(!s.write_ahead_uncertain());
    // An explicit retry against B then commits and confirms it there.
    b.mode = Mode::Ok;
    assert_eq!(revoke(&mut s, 0, &mut b, &mut e), Ok(()));
    let (mut cold, mut ce, info) = reopen(&b);
    assert_eq!(info.revoked[0], 1);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce), Err(Status::Revoked));

    // B is uncertain: Unknown is returned and the instance latches.
    let (mut s, mut e, mut a, _) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut a, &mut e), Ok(()));
    let mut b = Store::new();
    b.mode = Mode::UnknownLost;
    assert_eq!(
        revoke(&mut s, 0, &mut b, &mut e),
        Err(WriteAheadError::Unknown)
    );
    assert!(s.write_ahead_uncertain());
    let a_calls = a.calls;
    assert_eq!(
        revoke(&mut s, 0, &mut a, &mut e),
        Err(WriteAheadError::Unknown)
    );
    assert_eq!(a.calls, a_calls, "latched instance committed again");
}

/// M3: no `Ok` for an already-revoked index while anything is pending, while
/// uncertain, or before any write-ahead commit.
#[test]
fn revoke_retry_is_not_ok_unless_settled() {
    // A pending allocation (Failed generate) blocks the idempotent Ok.
    let (mut s, mut e, mut store, _) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    store.mode = Mode::Fail;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    let calls = store.calls;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Revoked))
    );
    assert_eq!(store.calls, calls);

    // Uncertain: refused before any check of the index.
    let (mut s, mut e, mut store, _) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    store.mode = Mode::UnknownLost;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );

    // Legacy revoke without any durable commit is not durable.
    let (mut s, mut e) = ready(0x5555);
    let index = s.generate(0, &mut e).unwrap();
    assert_eq!(s.revoke(0, index), Ok(()));
    let mut store = Store::new();
    assert_eq!(
        revoke(&mut s, index, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Revoked))
    );
    assert_eq!(store.calls, 0);
    assert_eq!(s.generate(0, &mut e), Ok(1), "legacy API still usable");
}

/// M2: a Failed generate consumes its index; the next commit records it as
/// allocated but unreturned, and nothing was sealed under it while pending.
#[test]
fn failed_generate_consumes_index_without_ciphertext() {
    let (mut s, mut e) = ready(0x6666);
    let mut store = Store::new();
    store.mode = Mode::Fail;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    assert_eq!(seal(&mut s, 0, &mut e), Err(Status::NotReady));
    assert_eq!(s.counters.sealed, 0);
    store.mode = Mode::Ok;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Ok(1),
        "index 0 skipped"
    );
    assert_eq!(s.counters.sealed, 0, "no seal under the skipped index");
    assert_eq!(s.keys_created(0), 2);
    let (_, _, info) = reopen(&store);
    assert_eq!(info.created[0], 2, "skipped index counts against the quota");
    assert_eq!(info.revoked[0], 0);
}

#[test]
fn recovery_export_is_refused_while_write_ahead_state_is_pending() {
    let (mut s, mut e) = ready(0x6767);
    let mut store = Store::new();
    store.mode = Mode::Fail;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    let mut secret = [0x11u8; SECRET_BYTES];
    let mut blob = [0x22u8; RECOVERY_BLOB];
    assert_eq!(
        s.export_recovery(&mut e, &mut secret, &mut blob),
        Err(Status::NotReady),
        "pending allocation must not leave through recovery"
    );
    assert_eq!(
        secret, [0x11u8; SECRET_BYTES],
        "refused export must not write"
    );
    assert_eq!(
        blob, [0x22u8; RECOVERY_BLOB],
        "refused export must not write"
    );

    // Once a later commit settles the state, export and restore work again
    // and the formerly pending index is only usable because it is durable.
    store.mode = Mode::Ok;
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    s.export_recovery(&mut e, &mut secret, &mut blob).unwrap();
    let (mut fresh, mut fe) = ready(0x6768);
    fresh.restore(&secret, &blob, &mut fe).unwrap();
    assert!(seal(&mut fresh, 1, &mut fe).is_ok());

    // An uncertain commit also blocks export.
    let (mut u, mut ue) = ready(0x6769);
    let mut lost = Store::new();
    lost.mode = Mode::UnknownLost;
    assert_eq!(
        generate(&mut u, &mut lost, &mut ue),
        Err(WriteAheadError::Unknown)
    );
    assert_eq!(
        u.export_recovery(&mut ue, &mut secret, &mut blob),
        Err(Status::NotReady)
    );
}

/// A live snapshot with a pending index, restored into a fresh instance, would
/// lose the pending gate and let that index seal without any store commit.
#[test]
fn persistent_export_is_refused_while_write_ahead_state_is_unsettled() {
    let export = |s: &mut Service, e: &mut Entropy| {
        let mut out = [0x5au8; PERSISTENT_BLOB];
        let result = s.export_persistent(&VOLUME, &SECRET, e, &mut out);
        if result.is_err() {
            assert_eq!(out, [0x5au8; PERSISTENT_BLOB], "refused export wrote");
        }
        result.map(|info| (info, out))
    };

    // Failed allocation: index 0 is pending and must not leave as a blob.
    let (mut s, mut e) = ready(0x6868);
    let mut store = Store::new();
    store.mode = Mode::Fail;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    assert_eq!(export(&mut s, &mut e), Err(Status::NotReady));
    assert_eq!(seal(&mut s, 0, &mut e), Err(Status::NotReady));

    // After a successful commit the export equals the durable state.
    store.mode = Mode::Ok;
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    let (info, blob) = export(&mut s, &mut e).unwrap();
    let (_, _, durable) = reopen(&store);
    assert_eq!(info, durable);
    let (mut fresh, mut fe) = ready(0x6869);
    let restored = fresh
        .restore_persistent(&VOLUME, &SECRET, &blob, 0, &mut fe)
        .unwrap();
    assert_eq!(restored, durable);
    assert!(seal(&mut fresh, 1, &mut fe).is_ok());

    // Failed revocation: pending until a retry commits it.
    let (mut s, mut e, mut store, _) = committed_base();
    store.mode = Mode::Fail;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Failed)
    );
    assert_eq!(export(&mut s, &mut e), Err(Status::NotReady));
    store.mode = Mode::Ok;
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    let (info, _) = export(&mut s, &mut e).unwrap();
    assert_eq!(info.revoked[0] & 1, 1);

    // Unknown latches the refusal permanently.
    let (mut s, mut e, mut store, _) = committed_base();
    store.mode = Mode::UnknownLost;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );
    store.mode = Mode::Ok;
    for _ in 0..2 {
        assert_eq!(export(&mut s, &mut e), Err(Status::NotReady));
        assert_eq!(
            generate(&mut s, &mut store, &mut e),
            Err(WriteAheadError::Unknown)
        );
    }
}

fn unknown_latches(mode: Mode) -> Store {
    let (mut s, mut e, mut store, _) = committed_base();
    store.mode = mode;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );
    assert!(s.write_ahead_uncertain());
    assert_eq!(store.calls, 2);
    assert_eq!(seal(&mut s, 1, &mut e), Err(Status::NotReady));
    // No automatic or caller-driven retry reaches the store.
    store.mode = Mode::Ok;
    for _ in 0..3 {
        assert_eq!(
            generate(&mut s, &mut store, &mut e),
            Err(WriteAheadError::Unknown)
        );
        assert_eq!(
            revoke(&mut s, 0, &mut store, &mut e),
            Err(WriteAheadError::Unknown)
        );
    }
    assert_eq!(store.calls, 2);
    assert_eq!(s.keys_created(0), 2, "refused calls allocate nothing");
    assert_eq!(seal(&mut s, 1, &mut e), Err(Status::NotReady));
    // Legacy mutators cannot bypass the write-ahead state.
    assert_eq!(s.generate(0, &mut e), Err(Status::Invalid));
    assert_eq!(s.revoke(0, 0), Err(Status::Invalid));
    assert_eq!(
        s.restore(&SECRET, &[0u8; RECOVERY_BLOB], &mut e),
        Err(Status::Invalid)
    );
    // Committed handles stay usable; only durability work is refused.
    assert!(seal(&mut s, 0, &mut e).is_ok());
    store
}

#[test]
fn unknown_is_never_confirmed_or_retried_when_write_was_lost() {
    let store = unknown_latches(Mode::UnknownLost);
    // Recovery reads durable state: index 1 was never confirmed nor persisted.
    assert_eq!(check_after_crash(&store, &[0], &[]), 1);
}

#[test]
fn unknown_is_never_confirmed_or_retried_when_write_landed() {
    let store = unknown_latches(Mode::UnknownLanded);
    // Index 1 is durable though never confirmed: it is not reissued.
    assert_eq!(check_after_crash(&store, &[0], &[]), 2);
}

/// Every write boundary of generate -> commit -> seal.
#[test]
fn crash_at_every_generate_boundary_never_reissues() {
    // G0: crash before the first allocation; nothing durable.
    let store = Store::new();
    assert_eq!(check_after_crash(&store, &[], &[]), 0);

    // G1: crash after the in-memory allocation, before export or store call.
    // The store never sees a blob, which is the same durable state as G0 for
    // a later instance; modelled by allocating and dropping before commit via
    // an export refused by the key service (G2) and by CrashBeforeWrite (G3).
    // G1 equals G3 because in both the allocation exists only in RAM, nothing
    // new reaches durable storage, and the instance is dropped.

    // G2: export refused (entropy reseed fails at the export boundary). The
    // seals fill the interval so allocate is the last free op and export hits
    // the reseed.
    let (mut s, mut e, mut store, sealed) = committed_base();
    for _ in 0..RESEED_INTERVAL - BASE_OPS - 2 {
        seal(&mut s, 0, &mut e).unwrap();
    }
    e.fail = true;
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    assert_eq!(store.calls, 1, "store not called after a refused export");
    assert_eq!(s.keys_created(0), 2);
    drop(s);
    assert_eq!(check_after_crash(&store, &[0], &[(0, sealed)]), 1);

    // G3: crash before the blob is durable.
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::CrashBeforeWrite;
    let _ = generate(&mut s, &mut store, &mut e);
    drop(s);
    assert_eq!(check_after_crash(&store, &[0], &[(0, sealed)]), 1);

    // G4: crash after the blob is durable, before commit returns.
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::CrashAfterWrite;
    let _ = generate(&mut s, &mut store, &mut e);
    drop(s);
    assert_eq!(check_after_crash(&store, &[0], &[(0, sealed)]), 2);

    // G5: crash after the confirmed handle is returned, before any seal.
    let (mut s, mut e, mut store, sealed) = committed_base();
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    drop(s);
    assert_eq!(check_after_crash(&store, &[0, 1], &[(0, sealed)]), 2);

    // G6: crash after sealing under the confirmed handle; the released
    // ciphertext opens after recovery.
    let (mut s, mut e, mut store, sealed0) = committed_base();
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    let sealed1 = seal(&mut s, 1, &mut e).unwrap();
    drop(s);
    assert_eq!(
        check_after_crash(&store, &[0, 1], &[(0, sealed0), (1, sealed1)]),
        2
    );
}

/// Every write boundary of revoke -> commit.
#[test]
fn crash_at_every_revoke_boundary_keeps_confirmed_revocations() {
    let revoked_after = |store: &Store, sealed: &[u8]| {
        let (mut s, mut e, info) = reopen(store);
        let open = unseal(&mut s, 0, sealed, &mut e);
        (info.revoked[0] & 1 != 0, open)
    };

    // R1: export refused; revocation local only, never confirmed.
    // Revoke itself is not accounted; only the export reaches the reseed.
    let (mut s, mut e, mut store, sealed) = committed_base();
    for _ in 0..RESEED_INTERVAL - BASE_OPS - 1 {
        seal(&mut s, 0, &mut e).unwrap();
    }
    e.fail = true;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    assert_eq!(store.calls, 1);
    drop(s);
    let (durable, open) = revoked_after(&store, &sealed);
    assert!(!durable);
    assert_eq!(open.unwrap(), PLAINTEXT);

    // R2: crash before the blob is durable; unconfirmed and not durable.
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::CrashBeforeWrite;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );
    drop(s);
    assert!(!revoked_after(&store, &sealed).0);

    // R3: crash after the blob is durable; unconfirmed but durable.
    let (mut s, mut e, mut store, sealed) = committed_base();
    store.mode = Mode::CrashAfterWrite;
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Unknown)
    );
    drop(s);
    assert_eq!(revoked_after(&store, &sealed), (true, Err(Status::Revoked)));

    // R4: crash after the confirmed revocation; it must survive.
    let (mut s, mut e, mut store, sealed) = committed_base();
    assert_eq!(revoke(&mut s, 0, &mut store, &mut e), Ok(()));
    drop(s);
    assert_eq!(revoked_after(&store, &sealed), (true, Err(Status::Revoked)));
    assert_eq!(check_after_crash(&store, &[0], &[]), 1);
}

#[test]
fn errors_and_blobs_leak_no_secret_or_plaintext() {
    let (mut s, mut e, mut store, _) = committed_base();
    store.mode = Mode::Fail;
    let failed = generate(&mut s, &mut store, &mut e).unwrap_err();
    store.mode = Mode::UnknownLost;
    let unknown = generate(&mut s, &mut store, &mut e).unwrap_err();
    for error in [failed, unknown, WriteAheadError::Key(Status::NotReady)] {
        let text = format!("{error:?}");
        assert!(text.len() < 32, "{text}");
        assert!(!text.contains("3c"));
    }
    for blob in &store.offered {
        assert!(!blob.windows(8).any(|w| w == &SECRET[..8]));
        assert!(!blob.windows(8).any(|w| w == &PLAINTEXT[..8]));
    }
}

#[test]
fn legacy_api_is_unchanged_until_write_ahead_is_used() {
    let (mut s, mut e) = ready(0x3333);
    let index = s.generate(0, &mut e).unwrap();
    assert!(seal(&mut s, index, &mut e).is_ok());
    assert_eq!(s.revoke(0, index), Ok(()));
    let mut store = Store::new();
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    assert_eq!(s.generate(0, &mut e), Err(Status::Invalid));
    assert_eq!(s.revoke(0, 1), Err(Status::Invalid));
}

#[test]
fn rejected_write_ahead_call_changes_nothing() {
    let (mut s, mut e, mut store, sealed) = committed_base();
    let other = *b"other-volume-002";
    assert_eq!(
        s.generate_durable(0, &other, &SECRET, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Invalid))
    );
    assert_eq!(
        s.revoke_durable(0, 0, &other, &SECRET, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Invalid))
    );
    assert_eq!(store.calls, 1);
    assert_eq!(s.keys_created(0), 1);
    assert_eq!(unseal(&mut s, 0, &sealed, &mut e).unwrap(), PLAINTEXT);
    // A fresh instance that never used write-ahead keeps the legacy API after
    // a rejected durable call.
    let (mut fresh, mut fe) = ready(0x4444);
    let mut store = Store::new();
    assert_eq!(
        fresh.revoke_durable(0, 0, &VOLUME, &SECRET, &mut store, &mut fe),
        Err(WriteAheadError::Key(Status::BadHandle))
    );
    assert_eq!(fresh.generate(0, &mut fe), Ok(0));
    assert_eq!(store.calls, 0);
}

/// A snapshot never committed through this store must not let `revoke_durable`
/// confirm a revocation it merged: restore is refused once write-ahead mode is
/// engaged, so the revocation is committed here before `Ok`.
#[test]
fn restore_in_write_ahead_mode_cannot_fake_a_durable_revocation() {
    let (mut s, mut e, mut store, _) = committed_base();
    assert_eq!(generate(&mut s, &mut store, &mut e), Ok(1));
    // A twin of the same lineage revokes index 1 against a different store.
    let (mut twin, mut te, _) = reopen(&store);
    let mut other = Store::new();
    assert_eq!(revoke(&mut twin, 1, &mut other, &mut te), Ok(()));
    let foreign = other.durable.unwrap();

    let before = s.snapshot_info();
    let restores = s.counters.restores;
    let calls = store.calls;
    assert_eq!(
        s.restore_persistent(&VOLUME, &SECRET, &foreign, 0, &mut e),
        Err(Status::Invalid)
    );
    assert_eq!(s.snapshot_info(), before, "refused restore changed state");
    assert_eq!(s.counters.restores, restores);
    assert_eq!(store.calls, calls);

    assert_eq!(revoke(&mut s, 1, &mut store, &mut e), Ok(()));
    assert_eq!(store.calls, calls + 1, "Ok without a store commit");
    let (_, _, info) = reopen(&store);
    assert_ne!(info.revoked[0] & (1 << 1), 0, "revocation not durable");

    // Before write-ahead mode a restore is still allowed, and the merged
    // revocation is not confirmed without a commit through this store.
    let (mut fresh, mut fe) = ready(0x7777);
    fresh
        .restore_persistent(&VOLUME, &SECRET, &foreign, 0, &mut fe)
        .unwrap();
    let mut fresh_store = Store::new();
    assert_eq!(
        revoke(&mut fresh, 1, &mut fresh_store, &mut fe),
        Err(WriteAheadError::Key(Status::Revoked))
    );
    assert_eq!(fresh_store.calls, 0);
}

// ---- B02-T2: durable KEY1 domain dispatch (host only; no native store) ----

const FOREIGN: [u8; 16] = *b"other-volume-003";

fn key_op(op: u8, index: u8) -> Header {
    Header {
        op,
        index,
        length: 0,
        request: 7,
        status: 0,
    }
}

/// One request through `dispatch_domain_durable` with the adapter-held
/// volume, secret and store; returns the reply status and body.
fn durable(
    s: &mut Service,
    e: &mut Entropy,
    domain: usize,
    header: Header,
    body: &[u8],
    store: &mut Store,
) -> (Status, Vec<u8>) {
    let mut ctx = wire::Durable {
        volume: &VOLUME,
        secret: &SECRET,
        store,
    };
    let out = wire::dispatch_domain_durable(s, e, domain, &header, body, &mut ctx);
    let reply = Header::decode(&out).unwrap();
    assert_eq!(reply.op, header.op | wire::REPLY);
    let length = usize::from(reply.length);
    (
        Status::from_code(reply.status),
        out[wire::HEADER..wire::HEADER + length].to_vec(),
    )
}

/// Drive `n` entropy-accounted operations on index 0 of a legacy instance.
fn spend(s: &mut Service, e: &mut Entropy, n: u32) {
    for _ in 0..n {
        seal(s, 0, e).unwrap();
    }
}

#[test]
fn durable_dispatch_generate_is_ok_only_after_store_commit() {
    let (mut s, mut e) = ready(0x8001);
    let mut store = Store::new();
    let ok = durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
    assert_eq!(ok, (Status::Ok, vec![0]));
    assert_eq!(store.calls, 1);
    assert_eq!(reopen(&store).2.created[0], 1);

    store.mode = Mode::Fail;
    let failed = durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
    assert_eq!(failed, (Status::NotReady, vec![]));
    assert_eq!(store.calls, 2);
    assert_eq!(
        seal(&mut s, 1, &mut e),
        Err(Status::NotReady),
        "pending gate"
    );
    assert_eq!(reopen(&store).2.created[0], 1, "failed allocation durable");

    for mode in [Mode::UnknownLost, Mode::UnknownLanded] {
        let (mut s, mut e) = ready(0x8011);
        let mut store = Store::new();
        store.mode = mode;
        let unknown = durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
        assert_eq!(unknown, (Status::Degraded, vec![]));
        assert_eq!(store.calls, 1);
        store.mode = Mode::Ok;
        for op in [GENERATE, REVOKE] {
            let latched = durable(&mut s, &mut e, 0, key_op(op, 0), &[], &mut store);
            assert_eq!(latched, (Status::Degraded, vec![]));
        }
        assert_eq!(store.calls, 1, "latched instance reached the store");
    }
}

#[test]
fn durable_dispatch_revoke_reply_matches_durable_state() {
    let (mut s, mut e) = ready(0x8002);
    let mut store = Store::new();
    let ok = durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
    assert_eq!(ok, (Status::Ok, vec![0]));
    let sealed = seal(&mut s, 0, &mut e).unwrap();

    store.mode = Mode::Fail;
    let failed = durable(&mut s, &mut e, 0, key_op(REVOKE, 0), &[], &mut store);
    assert_eq!(failed, (Status::NotReady, vec![]));
    assert_eq!(unseal(&mut s, 0, &sealed, &mut e), Err(Status::Revoked));
    assert_eq!(reopen(&store).2.revoked[0], 0, "Failed revocation durable");

    store.mode = Mode::Ok;
    let retried = durable(&mut s, &mut e, 0, key_op(REVOKE, 0), &[], &mut store);
    assert_eq!(retried, (Status::Ok, vec![]));
    assert_eq!(store.calls, 3);
    let (mut cold, mut ce, info) = reopen(&store);
    assert_eq!(info.revoked[0], 1);
    assert_eq!(unseal(&mut cold, 0, &sealed, &mut ce), Err(Status::Revoked));

    let (mut s, mut e) = ready(0x8012);
    let mut store = Store::new();
    durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
    store.mode = Mode::UnknownLost;
    let unknown = durable(&mut s, &mut e, 0, key_op(REVOKE, 0), &[], &mut store);
    assert_eq!(unknown, (Status::Degraded, vec![]));
    store.mode = Mode::Ok;
    for op in [REVOKE, GENERATE] {
        let latched = durable(&mut s, &mut e, 0, key_op(op, 0), &[], &mut store);
        assert_eq!(latched, (Status::Degraded, vec![]));
    }
    assert_eq!(store.calls, 2);
}

#[test]
fn durable_dispatch_refuses_malformed_key_mutation_without_store_call() {
    let (mut s, mut e) = ready(0x8003);
    let mut store = Store::new();
    let before = s.snapshot_info();
    for op in [GENERATE, REVOKE] {
        let mut with_body = key_op(op, 0);
        with_body.length = 1;
        let mut short = key_op(op, 0);
        short.length = 1;
        let mut status = key_op(op, 0);
        status.status = 1;
        for (header, body) in [(with_body, &[0u8][..]), (short, &[][..]), (status, &[][..])] {
            let refused = durable(&mut s, &mut e, 0, header, body, &mut store);
            assert_eq!(refused, (Status::Invalid, vec![]));
        }
        let foreign = durable(&mut s, &mut e, DOMAINS, key_op(op, 0), &[], &mut store);
        assert_eq!(foreign, (Status::Invalid, vec![]));
    }
    assert_eq!(store.calls, 0);
    assert_eq!(s.snapshot_info(), before);
    // Write-ahead was never engaged: the legacy API still works.
    assert_eq!(s.generate(0, &mut e), Ok(0));
}

#[test]
fn durable_dispatch_never_reaches_persistence_or_legacy() {
    let (mut s, mut e) = ready(0x8004);
    let mut store = Store::new();
    let blob = [0u8; PERSISTENT_BLOB];
    let requests = [
        wire::export_persistent_request(1, &VOLUME, &SECRET).unwrap(),
        wire::restore_persistent_request(2, &VOLUME, &SECRET, 0, &blob).unwrap(),
        wire::request(wire::EXPORT_RECOVERY, 0, 3, &[]).unwrap(),
        wire::request(wire::RESTORE, 0, 4, &[0u8; SECRET_BYTES + RECOVERY_BLOB]).unwrap(),
    ];
    let before = s.snapshot_info();
    for message in requests {
        let header = Header::decode(&message).unwrap();
        let body = &message[wire::HEADER..wire::HEADER + usize::from(header.length)];
        let refused = durable(&mut s, &mut e, 0, header, body, &mut store);
        assert_eq!(refused, (Status::Invalid, vec![]));
    }
    assert_eq!(store.calls, 0);
    assert_eq!(s.snapshot_info(), before);
    assert_eq!(s.counters.restores, 0);

    let ok = durable(&mut s, &mut e, 0, key_op(GENERATE, 0), &[], &mut store);
    assert_eq!(ok, (Status::Ok, vec![0]));
    // No silent downgrade to the legacy, non-durable mutators.
    for op in [GENERATE, REVOKE] {
        let out = wire::dispatch_domain(&mut s, &mut e, 0, &key_op(op, 0), &[]);
        let reply = Header::decode(&out).unwrap();
        assert_eq!(Status::from_code(reply.status), Status::Invalid);
        assert_eq!(reply.length, 0);
    }
    assert_eq!(store.calls, 1);
    let info = s.snapshot_info();
    assert_eq!((info.created[0], info.revoked[0]), (1, 0));
}

#[test]
fn durable_dispatch_takes_domain_from_adapter_not_payload() {
    let (mut s, mut e) = ready(0x8005);
    let mut store = Store::new();
    let message = wire::request(GENERATE, 0, 9, &[]).unwrap();
    let header = Header::decode(&message).unwrap();
    let ok = durable(&mut s, &mut e, 1, header, &[], &mut store);
    assert_eq!(ok, (Status::Ok, vec![0]));
    assert_eq!(s.snapshot_info().created, [0, 1]);
    // Index 0 exists only in domain 1; the domain-0 ingress cannot revoke it.
    let wrong = durable(&mut s, &mut e, 0, key_op(REVOKE, 0), &[], &mut store);
    assert_eq!(wrong, (Status::BadHandle, vec![]));
    assert_eq!(store.calls, 1);
    assert_eq!(s.snapshot_info().revoked, [0, 0]);
}

/// I5: the first durable commit binds its volume even when sealing fails, so
/// a later call with another volume is refused instead of reaching the state.
#[test]
fn first_durable_commit_failing_before_binding_still_binds_volume() {
    let (mut s, mut e) = ready(0x8006);
    assert_eq!(s.generate(0, &mut e), Ok(0));
    // 1 + 61 accounted ops: allocate is op 63, the snapshot seal reseeds at 64.
    spend(&mut s, &mut e, RESEED_INTERVAL - 3);
    e.fail = true;
    let mut store = Store::new();
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    let before = s.snapshot_info();
    assert_eq!(
        s.generate_durable(0, &FOREIGN, &SECRET, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Invalid))
    );
    assert_eq!(
        s.revoke_durable(0, 0, &FOREIGN, &SECRET, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Invalid))
    );
    assert_eq!(s.snapshot_info(), before, "foreign volume changed state");
    assert_eq!(store.calls, 0);
}

/// I1/I5 on the reachable Degraded path: `revoke_entry` accepts `Degraded`,
/// so a durable REVOKE engages write-ahead but can never be confirmed.
#[test]
fn degraded_durable_revoke_binds_volume_and_never_confirms() {
    let (mut s, mut e) = ready(0x8007);
    assert_eq!(s.generate(0, &mut e), Ok(0));
    assert_eq!(s.generate(0, &mut e), Ok(1));
    spend(&mut s, &mut e, RESEED_INTERVAL - 3);
    e.fail = true;
    assert_eq!(seal(&mut s, 0, &mut e), Err(Status::Degraded));
    let mut store = Store::new();
    assert_eq!(
        revoke(&mut s, 0, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    assert_eq!(s.snapshot_info().revoked[0], 1, "revoked locally at once");
    let before = s.snapshot_info();
    assert_eq!(
        s.revoke_durable(0, 1, &FOREIGN, &SECRET, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Invalid))
    );
    assert_eq!(s.snapshot_info(), before, "foreign volume revoked index 1");
    assert_eq!(
        revoke(&mut s, 1, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    assert_eq!(
        generate(&mut s, &mut store, &mut e),
        Err(WriteAheadError::Key(Status::Degraded))
    );
    assert_eq!(store.calls, 0, "no commit, so no confirmation");
}

/// I6: a failed persistent export outside write-ahead mode binds nothing.
#[test]
fn failed_export_before_binding_leaves_instance_unbound() {
    let (mut s, mut e) = ready(0x8008);
    assert_eq!(s.generate(0, &mut e), Ok(0));
    // 1 + 62 accounted ops: the export's accounting reseeds at op 64.
    spend(&mut s, &mut e, RESEED_INTERVAL - 2);
    e.fail = true;
    let before = s.snapshot_info();
    let mut out = [0x5au8; PERSISTENT_BLOB];
    assert_eq!(
        s.export_persistent(&VOLUME, &SECRET, &mut e, &mut out),
        Err(Status::Degraded)
    );
    assert_eq!(out, [0x5au8; PERSISTENT_BLOB], "failed export wrote");
    assert_eq!(
        s.export_persistent(&FOREIGN, &SECRET, &mut e, &mut out),
        Err(Status::Degraded),
        "failed export bound the volume"
    );
    assert_eq!(s.snapshot_info(), before);
}
