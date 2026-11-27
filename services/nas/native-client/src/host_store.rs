//! Host-only serialized coordinator model. This is not a native security service,
//! cryptographic implementation, durable store, or cold-freshness proof.
use crate::{Checkpoint, Client, Failure, Remote, State, CHECKPOINT_BYTES, PENDING};

pub const CHUNK_BYTES: usize = 128;
pub const CHUNKS: usize = CHECKPOINT_BYTES.div_ceil(CHUNK_BYTES);

/// Full identities from trusted local state, not the storage provider.
#[derive(Clone, PartialEq, Eq)]
pub struct Binding {
    pub user: [u8; 32],
    pub peer: [u8; 32],
    pub namespace: [u8; 32],
    pub volume: [u8; 32],
    pub grant: [u8; 32],
    pub grant_scope: [u8; 32],
    pub grant_generation: u64,
    pub checkpoint: [u8; 32],
    pub objects: [[u8; 32]; PENDING],
    pub revisions: [u64; PENDING],
}

/// Semantic plaintext returned after authentication and decryption.
#[derive(Clone, PartialEq, Eq)]
pub struct Manifest {
    pub binding: Binding,
    pub publication: u64,
    pub attempt: [u8; 32],
    pub schema: u16,
    pub length: usize,
    pub chunks: usize,
    pub commitment: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    Unauthorized,
    Stale,
    Invalid,
    Backend,
    Unknown,
}

/// The trusted local mapping must match all decoded edits, including each full
/// object identity and NAS revision. Grant changes must be serialized through
/// this coordinator via revoke. A real authority is outside this model.
pub trait Authorization {
    fn authorize(&mut self, binding: &Binding, checkpoint: &Checkpoint) -> bool;
}

/// All confidentiality, authentication, collision resistance and durability
/// remain explicit backend assumptions. No native implementation is supplied.
pub trait Backend {
    fn begin_attempt(&mut self, attempt: &[u8; 32]) -> Result<(), StoreError>;
    fn commitment(&mut self, bytes: &[u8; CHECKPOINT_BYTES]) -> [u8; 32];
    fn stage_chunk(
        &mut self,
        manifest: &Manifest,
        index: usize,
        bytes: &[u8],
    ) -> Result<(), StoreError>;
    fn flush_chunks(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
    /// Confirm this exact candidate remains complete and durable at decision time.
    fn verify_candidate(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
    /// Publish only the exact fully staged and flushed candidate bound to this
    /// manifest. On uncertainty retain the prior bytes until resolution.
    fn publish(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
    fn open_manifest(&mut self) -> Result<Manifest, StoreError>;
    fn open_chunk(
        &mut self,
        manifest: &Manifest,
        index: usize,
        out: &mut [u8],
    ) -> Result<(), StoreError>;
    fn check_chunk_count(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
}

/// This authority models separately trusted state; a real independent anchor is
/// not implemented here. An unavailable authoritative read returns an error.
pub trait IndependentAuthority {
    fn observe(
        &mut self,
        prior: &Option<Manifest>,
        candidate: &Manifest,
    ) -> Result<Option<Manifest>, StoreError>;
}

/// An unexposed staged checkpoint; its expected anchor prevents stale writers.
pub struct Candidate {
    manifest: Manifest,
    expected: Option<Manifest>,
}
/// Unexposed decoded state, rechecked at its later handoff decision.
pub struct Prepared {
    manifest: Manifest,
    checkpoint: Checkpoint,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    instance: [u8; 32],
    generation: u64,
}

/// A single mutable coordinator serializes model decisions. Independent anchor
/// state is represented by `anchor`; production must implement it independently
/// of the rewindable backend. The model assumes attempts are globally unique.
pub struct Coordinator {
    instance: [u8; 32],
    binding: Binding,
    active: bool,
    generation: u64,
    anchor: Option<Manifest>,
    pending: Option<Manifest>,
    client: Option<Client>,
}
impl Coordinator {
    /// Instance must be unique and allocated by the trusted local supervisor.
    /// Predictable bytes do not provide cryptographic unforgeability.
    pub fn new(binding: Binding, instance: [u8; 32]) -> Self {
        Self {
            instance,
            binding,
            active: true,
            generation: 1,
            anchor: None,
            pending: None,
            client: None,
        }
    }
    pub fn revoke(&mut self) {
        self.active = false;
        self.client = None;
        self.generation = self.generation.saturating_add(1);
    }
    fn live(&self, binding: &Binding) -> Result<(), StoreError> {
        if !self.active || self.binding != *binding {
            Err(StoreError::Unauthorized)
        } else {
            Ok(())
        }
    }
    pub fn stage<A: Authorization, B: Backend>(
        &self,
        client: &Client,
        publication: u64,
        attempt: [u8; 32],
        auth: &mut A,
        backend: &mut B,
    ) -> Result<Candidate, StoreError> {
        self.live(&self.binding)?;
        if self.pending.is_some() {
            return Err(StoreError::Unknown);
        }
        if attempt == [0; 32]
            || self
                .anchor
                .as_ref()
                .is_some_and(|m| publication <= m.publication)
        {
            return Err(StoreError::Invalid);
        }
        let cp = client.checkpoint();
        if !auth.authorize(&self.binding, &cp) {
            return Err(StoreError::Unauthorized);
        }
        backend.begin_attempt(&attempt)?;
        let mut bytes = [0; CHECKPOINT_BYTES];
        cp.encode(&mut bytes);
        let manifest = Manifest {
            binding: self.binding.clone(),
            publication,
            attempt,
            schema: 1,
            length: CHECKPOINT_BYTES,
            chunks: CHUNKS,
            commitment: backend.commitment(&bytes),
        };
        for (i, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
            backend.stage_chunk(&manifest, i, chunk)?;
        }
        backend.flush_chunks(&manifest)?;
        Ok(Candidate {
            manifest,
            expected: self.anchor.clone(),
        })
    }
    /// The model's publication linearization point; revoke uses the same owner.
    pub fn publish<B: Backend>(
        &mut self,
        candidate: Candidate,
        backend: &mut B,
    ) -> Result<(), StoreError> {
        self.live(&candidate.manifest.binding)?;
        if self.pending.is_some() {
            return Err(StoreError::Unknown);
        }
        if self.anchor != candidate.expected {
            return Err(StoreError::Stale);
        }
        backend.verify_candidate(&candidate.manifest)?;
        match backend.publish(&candidate.manifest) {
            Ok(()) => {
                self.client = None;
                self.anchor = Some(candidate.manifest);
                Ok(())
            }
            Err(StoreError::Unknown) => {
                self.client = None;
                self.pending = Some(candidate.manifest);
                Err(StoreError::Unknown)
            }
            Err(e) => Err(e),
        }
    }
    /// Reads a trusted independent authority, never a provider observation.
    pub fn resolve_unknown(
        &mut self,
        authority: &mut impl IndependentAuthority,
    ) -> Result<(), StoreError> {
        self.live(&self.binding)?;
        let candidate = self.pending.as_ref().ok_or(StoreError::Invalid)?;
        let observed = authority.observe(&self.anchor, candidate)?;
        if observed.as_ref() == Some(candidate) {
            self.anchor = observed;
            self.pending = None;
            Ok(())
        } else if observed == self.anchor {
            self.pending = None;
            Ok(())
        } else {
            Err(StoreError::Stale)
        }
    }
    pub fn prepare<A: Authorization, B: Backend>(
        &self,
        auth: &mut A,
        backend: &mut B,
    ) -> Result<Prepared, StoreError> {
        self.live(&self.binding)?;
        if self.pending.is_some() {
            return Err(StoreError::Unknown);
        }
        let manifest = backend.open_manifest()?;
        if Some(&manifest) != self.anchor.as_ref()
            || manifest.binding != self.binding
            || manifest.schema != 1
            || manifest.length != CHECKPOINT_BYTES
            || manifest.chunks != CHUNKS
            || manifest.attempt == [0; 32]
        {
            return Err(StoreError::Stale);
        }
        backend.check_chunk_count(&manifest)?;
        let mut bytes = [0; CHECKPOINT_BYTES];
        for (i, chunk) in bytes.chunks_mut(CHUNK_BYTES).enumerate() {
            backend.open_chunk(&manifest, i, chunk)?;
        }
        if backend.commitment(&bytes) != manifest.commitment {
            return Err(StoreError::Invalid);
        }
        let checkpoint = Checkpoint::decode(&bytes).map_err(|_| StoreError::Invalid)?;
        if !auth.authorize(&self.binding, &checkpoint) {
            return Err(StoreError::Unauthorized);
        }
        Ok(Prepared {
            manifest,
            checkpoint,
        })
    }
    /// Restore-handoff linearization: no decoded data is returned to the caller.
    pub fn handoff<A: Authorization>(
        &mut self,
        prepared: Prepared,
        auth: &mut A,
    ) -> Result<Handle, StoreError> {
        self.live(&prepared.manifest.binding)?;
        if self.pending.is_some() || self.anchor.as_ref() != Some(&prepared.manifest) {
            return Err(StoreError::Stale);
        }
        if !auth.authorize(&self.binding, &prepared.checkpoint) {
            return Err(StoreError::Unauthorized);
        }
        self.generation = self.generation.checked_add(1).ok_or(StoreError::Invalid)?;
        self.client = Some(Client::restore(prepared.checkpoint));
        Ok(Handle {
            instance: self.instance,
            generation: self.generation,
        })
    }
    fn client(&mut self, handle: Handle) -> Result<&mut Client, StoreError> {
        self.live(&self.binding)?;
        if self.instance != handle.instance || self.generation != handle.generation {
            return Err(StoreError::Unauthorized);
        }
        self.client.as_mut().ok_or(StoreError::Unauthorized)
    }
    /// Copy locally held ciphertext only while the handle is valid.
    pub fn read(
        &mut self,
        handle: Handle,
        slot: usize,
        out: &mut [u8],
    ) -> Result<usize, StoreError> {
        let bytes = self
            .client(handle)?
            .edit(slot)
            .map_err(|_| StoreError::Invalid)?
            .local_ciphertext();
        if out.len() < bytes.len() {
            return Err(StoreError::Invalid);
        }
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }
    /// Authorized read-only reconciliation. This model intentionally supplies no
    /// Commit: the existing client does not yet enforce durable pre-send storage.
    pub fn reconcile(
        &mut self,
        handle: Handle,
        slot: usize,
        remote: &mut impl Remote,
    ) -> Result<Result<State, Failure>, StoreError> {
        Ok(self.client(handle)?.reconcile_latest_via(slot, remote))
    }
}
