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
    /// Across this backend, block all new attempts while any prior publication
    /// outcome is unresolved, including across replacement coordinators.
    /// A failed or unavailable fence must retain that backend-wide gate.
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
    /// manifest. Block publication while any earlier attempt is unresolved,
    /// including direct calls and across coordinators. Refuse terminal attempts.
    /// On uncertainty atomically set the backend-wide gate and retain prior bytes.
    fn publish(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
    /// Make the uncertain attempt terminal before a fresh authority read. Once
    /// this succeeds no delayed completion can modify published bytes, even
    /// after a later attempt publishes. Repeated fences are idempotent, because
    /// the later authority read may fail. Fencing alone never releases the
    /// backend-wide gate. Failure or uncertainty must return Err and retain it.
    fn fence_attempt(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
    /// Trusted modeled decision after fencing and exact independent selection.
    /// Preserve both the exact prior checkpoint (if any) and the entire
    /// candidate, even when prior is selected, before atomically releasing the
    /// gate. Failure or uncertainty retains both the gate and candidate data.
    /// This is a host-only contract, not a native recovery implementation.
    fn finalize_attempt(
        &mut self,
        candidate: &Manifest,
        prior: &Option<Manifest>,
        selected: &Option<Manifest>,
    ) -> Result<(), StoreError>;
    fn open_manifest(&mut self) -> Result<Manifest, StoreError>;
    fn open_chunk(
        &mut self,
        manifest: &Manifest,
        index: usize,
        out: &mut [u8],
    ) -> Result<(), StoreError>;
    fn check_chunk_count(&mut self, manifest: &Manifest) -> Result<(), StoreError>;
}

/// Coordinator-owned trusted host-model route factory: it must bind the returned
/// connection to the authorized full peer, namespace, grant, scope and current
/// grant generation, independently of Remote::caller (local ingress principal).
/// It must keep this route stable for the entire exchange and fail closed when
/// verification is unavailable. No native authentication or transport is
/// implemented here. Only trusted setup may construct or supply the factory;
/// a request or handle holder may not choose or replace it.
pub trait TrustedPeerConnection {
    type Remote: Remote;
    fn verified_remote(&mut self, expected: &Binding) -> Result<&mut Self::Remote, StoreError>;
}

/// This authority models separately trusted state; a real independent anchor is
/// not implemented here. An unavailable authoritative read returns an error.
/// Observe only after a successful backend fence; return the current exact
/// authoritative state after that fence, never a stale observation.
pub trait IndependentAuthority {
    fn observe(
        &mut self,
        prior: &Option<Manifest>,
        candidate: &Manifest,
    ) -> Result<Option<Manifest>, StoreError>;
}

/// An unexposed staged checkpoint; its expected anchor prevents stale writers.
pub struct Candidate {
    instance: [u8; 32],
    checkpoint: Checkpoint,
    manifest: Manifest,
    expected: Option<Manifest>,
}
/// Unexposed decoded state, rechecked at its later handoff decision.
pub struct Prepared {
    instance: [u8; 32],
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
pub struct Coordinator<C: TrustedPeerConnection> {
    connection: C,
    instance: [u8; 32],
    binding: Binding,
    active: bool,
    generation: u64,
    anchor: Option<Manifest>,
    pending: Option<Manifest>,
    client: Option<Client>,
}
impl<C: TrustedPeerConnection> Coordinator<C> {
    /// Instance must be unique and allocated by the trusted local supervisor.
    /// Predictable bytes do not provide cryptographic unforgeability.
    pub fn new(binding: Binding, instance: [u8; 32], connection: C) -> Self {
        Self {
            connection,
            instance,
            binding,
            active: true,
            generation: 1,
            anchor: None,
            pending: None,
            client: None,
        }
    }
    /// Disable access immediately; success additionally means any uncertain
    /// publication is terminally fenced. On fence failure access stays revoked,
    /// pending is retained, and this method may be called again for cleanup.
    pub fn revoke(&mut self, backend: &mut impl Backend) -> Result<(), StoreError> {
        self.active = false;
        self.client = None;
        self.generation = self.generation.saturating_add(1);
        if let Some(candidate) = &self.pending {
            backend.fence_attempt(candidate)?;
        }
        Ok(())
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
            instance: self.instance,
            checkpoint: cp,
            manifest,
            expected: self.anchor.clone(),
        })
    }
    /// The model's publication linearization point; revoke uses the same owner.
    pub fn publish<A: Authorization, B: Backend>(
        &mut self,
        candidate: Candidate,
        auth: &mut A,
        backend: &mut B,
    ) -> Result<(), StoreError> {
        self.live(&candidate.manifest.binding)?;
        if candidate.instance != self.instance {
            return Err(StoreError::Unauthorized);
        }
        if self.pending.is_some() {
            return Err(StoreError::Unknown);
        }
        if self.anchor != candidate.expected {
            return Err(StoreError::Stale);
        }
        if !auth.authorize(&self.binding, &candidate.checkpoint) {
            return Err(StoreError::Unauthorized);
        }
        backend.verify_candidate(&candidate.manifest)?;
        if !auth.authorize(&self.binding, &candidate.checkpoint) {
            return Err(StoreError::Unauthorized);
        }
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
        backend: &mut impl Backend,
    ) -> Result<(), StoreError> {
        self.live(&self.binding)?;
        let candidate = self.pending.as_ref().ok_or(StoreError::Invalid)?;
        backend.fence_attempt(candidate)?;
        let observed = authority.observe(&self.anchor, candidate)?;
        if observed.as_ref() != Some(candidate) && observed != self.anchor {
            return Err(StoreError::Stale);
        }
        backend.finalize_attempt(candidate, &self.anchor, &observed)?;
        self.anchor = observed;
        self.pending = None;
        Ok(())
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
            instance: self.instance,
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
        if prepared.instance != self.instance {
            return Err(StoreError::Unauthorized);
        }
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
    /// Observe a queue state only while the handle is valid.
    pub fn state(&mut self, handle: Handle, slot: usize) -> Result<State, StoreError> {
        Ok(self
            .client(handle)?
            .edit(slot)
            .map_err(|_| StoreError::Invalid)?
            .state())
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
    ) -> Result<Result<State, Failure>, StoreError> {
        self.live(&self.binding)?;
        if self.instance != handle.instance
            || self.generation != handle.generation
            || self.client.is_none()
        {
            return Err(StoreError::Unauthorized);
        }
        let remote = self.connection.verified_remote(&self.binding)?;
        Ok(self
            .client
            .as_mut()
            .ok_or(StoreError::Unauthorized)?
            .reconcile_latest_via(slot, remote))
    }
}
