//! Bounded client queue against the existing local NAS core, not a transport.
//! Inputs are already-encrypted opaque objects. This module does not establish
//! identity, encrypt objects, persist checkpoints or authenticate their freshness.
#![no_std]
#![forbid(unsafe_code)]

use core::fmt;
use nieuw_os_nas_core::{
    Error, Principal, Service, Session, Storage, Upload, MAX_CIPHERTEXT, OBJECTS,
};

pub const PENDING: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ready,
    Staged,
    NeedsReconcile,
    Conflict,
    Blocked(Error),
    Published(u64),
    /// Matching bytes were read back; this is not our operation's commit ACK.
    Observed(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failure {
    pub operation: Error,
    pub cleanup: Option<Error>,
}

impl From<Error> for Failure {
    fn from(operation: Error) -> Self {
        Self {
            operation,
            cleanup: None,
        }
    }
}

#[derive(Clone)]
struct Version {
    revision: u64,
    length: usize,
    bytes: [u8; MAX_CIPHERTEXT],
}

/// No Debug of namespace, identifiers or ciphertext.
#[derive(Clone)]
pub struct Edit {
    namespace: [u8; 32],
    object: usize,
    local: Version,
    remote: Option<Version>,
    base: u64,
    state: State,
}

impl Edit {
    pub fn namespace(&self) -> &[u8; 32] {
        &self.namespace
    }

    pub fn object(&self) -> usize {
        self.object
    }

    pub fn base_revision(&self) -> u64 {
        self.base
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn local_ciphertext(&self) -> &[u8] {
        &self.local.bytes[..self.local.length]
    }

    pub fn remote_ciphertext(&self) -> Option<(u64, &[u8])> {
        self.remote
            .as_ref()
            .map(|v| (v.revision, &v.bytes[..v.length]))
    }
}

impl fmt::Debug for Edit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Edit")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

/// An in-memory model, not an on-disk schema or a durable-save acknowledgement.
/// It contains no Session or Upload. An adapter still needs authenticated,
/// atomic, rollback-aware local persistence.
#[derive(Clone)]
pub struct Checkpoint {
    edits: [Option<Edit>; PENDING],
}

pub struct Client {
    edits: [Option<Edit>; PENDING],
    tickets: [Option<(Session, Upload)>; PENDING],
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            edits: core::array::from_fn(|_| None),
            tickets: [None; PENDING],
        }
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            edits: self.edits.clone(),
        }
    }

    pub fn restore(checkpoint: Checkpoint) -> Self {
        let mut edits = checkpoint.edits;
        // Even a pre-dispatch checkpoint may precede an unobserved commit.
        for edit in edits.iter_mut().flatten() {
            if !matches!(edit.state, State::Published(_) | State::Observed(_)) {
                edit.state = State::NeedsReconcile;
            }
        }
        Self {
            edits,
            tickets: [None; PENDING],
        }
    }

    pub fn edit(&self, slot: usize) -> Result<&Edit, Failure> {
        self.edits
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(Error::Invalid.into())
    }

    fn edit_mut(&mut self, slot: usize) -> Result<&mut Edit, Failure> {
        self.edits
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(Error::Invalid.into())
    }

    /// Queue ciphertext without contacting a server. A full/duplicate queue
    /// rejects the new edit rather than replacing an unsynchronised edit.
    pub fn enqueue(
        &mut self,
        namespace: [u8; 32],
        object: usize,
        base: u64,
        ciphertext: &[u8],
    ) -> Result<usize, Failure> {
        let revision = base.checked_add(1).ok_or(Error::Invalid)?;
        if namespace == [0; 32]
            || object >= OBJECTS
            || ciphertext.is_empty()
            || ciphertext.len() > MAX_CIPHERTEXT
        {
            return Err(Error::Invalid.into());
        }
        if self
            .edits
            .iter()
            .flatten()
            .any(|e| e.namespace == namespace && e.object == object)
        {
            return Err(Error::Conflict.into());
        }
        let slot = self
            .edits
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let mut bytes = [0; MAX_CIPHERTEXT];
        bytes[..ciphertext.len()].copy_from_slice(ciphertext);
        self.edits[slot] = Some(Edit {
            namespace,
            object,
            local: Version {
                revision,
                length: ciphertext.len(),
                bytes,
            },
            remote: None,
            base,
            state: State::Ready,
        });
        Ok(slot)
    }

    /// Caller is supplied only by trusted ingress, as required by nas-core.
    /// Always open the queued namespace; never accept an unrelated session.
    pub fn prepare<S: Storage>(
        &mut self,
        slot: usize,
        service: &mut Service<'_, S>,
        caller: Principal,
    ) -> Result<(), Failure> {
        let edit = self.edit_mut(slot)?;
        if edit.state != State::Ready {
            return Err(Error::ReopenRequired.into());
        }
        let session = match service.open(caller, &edit.namespace) {
            Ok(session) => session,
            Err(error) => {
                edit.state = failed(error);
                return Err(error.into());
            }
        };
        match service.stage(
            caller,
            session,
            edit.object,
            edit.base,
            edit.local_ciphertext(),
        ) {
            Ok(upload) => {
                edit.state = State::Staged;
                self.tickets[slot] = Some((session, upload));
                Ok(())
            }
            Err(operation) => {
                edit.state = failed(operation);
                Err(Failure {
                    operation,
                    cleanup: service.close(caller, session).err(),
                })
            }
        }
    }

    pub fn commit<S: Storage>(
        &mut self,
        slot: usize,
        service: &mut Service<'_, S>,
        caller: Principal,
    ) -> Result<u64, Failure> {
        if self.edit(slot)?.state != State::Staged {
            return Err(Error::ReopenRequired.into());
        }
        let (session, upload) = self.tickets[slot].take().ok_or(Error::ReopenRequired)?;
        let edit = self.edit_mut(slot)?;
        edit.state = State::NeedsReconcile;
        let result = service.commit(caller, session, upload);
        let cleanup = service.close(caller, session).err();
        match result {
            Ok(revision) => {
                edit.state = State::Published(revision);
                match cleanup {
                    Some(error) => Err(Failure {
                        operation: error,
                        cleanup: Some(error),
                    }),
                    None => Ok(revision),
                }
            }
            Err(operation) => {
                edit.state = failed(operation);
                Err(Failure { operation, cleanup })
            }
        }
    }

    pub fn cancel<S: Storage>(
        &mut self,
        slot: usize,
        service: &mut Service<'_, S>,
        caller: Principal,
    ) -> Result<(), Failure> {
        if self.edit(slot)?.state != State::Staged {
            return Err(Error::ReopenRequired.into());
        }
        let (session, upload) = self.tickets[slot].take().ok_or(Error::ReopenRequired)?;
        let edit = self.edit_mut(slot)?;
        let result = service.cancel(caller, session, upload);
        let cleanup = service.close(caller, session).err();
        match result {
            Ok(()) => match cleanup {
                None => {
                    edit.state = State::Ready;
                    Ok(())
                }
                Some(error) => {
                    edit.state = State::NeedsReconcile;
                    Err(Failure {
                        operation: error,
                        cleanup: Some(error),
                    })
                }
            },
            Err(operation) => {
                edit.state = failed(operation);
                Err(Failure { operation, cleanup })
            }
        }
    }

    /// Explicit read-only reconciliation. The revision is a hint, NOT trusted
    /// evidence: the existing service validates caller/session/object/revision.
    /// There is deliberately no new "head/latest" protocol here.
    pub fn reconcile<S: Storage>(
        &mut self,
        slot: usize,
        service: &mut Service<'_, S>,
        caller: Principal,
        revision: u64,
    ) -> Result<State, Failure> {
        let edit = self.edit_mut(slot)?;
        if matches!(edit.state, State::Published(_) | State::Observed(_)) {
            return Err(Error::Invalid.into());
        }
        if edit.state == State::Staged {
            return Err(Error::ReopenRequired.into());
        }
        let session = match service.open(caller, &edit.namespace) {
            Ok(session) => session,
            Err(error) => {
                edit.state = failed(error);
                return Err(error.into());
            }
        };
        let mut bytes = [0; MAX_CIPHERTEXT];
        let read = service.read(caller, session, edit.object, revision, &mut bytes);
        let cleanup = service.close(caller, session).err();
        let length = match read {
            Ok(length) => length,
            Err(Error::NotFound) if revision == 0 => 0,
            Err(operation) => {
                edit.state = failed(operation);
                return Err(Failure { operation, cleanup });
            }
        };
        if let Some(error) = cleanup {
            edit.state = State::NeedsReconcile;
            return Err(Failure {
                operation: error,
                cleanup: Some(error),
            });
        }
        edit.remote = Some(Version {
            revision,
            length,
            bytes,
        });
        if revision < edit.base {
            edit.state = State::Blocked(Error::Stale);
            return Err(Error::Stale.into());
        }
        edit.state = if revision == edit.base {
            State::Ready
        } else if revision == edit.local.revision && &bytes[..length] == edit.local_ciphertext() {
            State::Observed(revision)
        } else {
            State::Conflict
        };
        Ok(edit.state)
    }

    /// Transfer the retained edit to the caller; never silently discard it.
    /// A staged operation must first finish or be recovered without its ticket.
    pub fn take(&mut self, slot: usize) -> Result<Edit, Failure> {
        if self.edit(slot)?.state == State::Staged {
            return Err(Error::ReopenRequired.into());
        }
        self.edits[slot].take().ok_or(Error::Invalid.into())
    }
}

fn failed(error: Error) -> State {
    match error {
        Error::Unknown | Error::ReopenRequired | Error::Io => State::NeedsReconcile,
        Error::Conflict => State::Conflict,
        error => State::Blocked(error),
    }
}
