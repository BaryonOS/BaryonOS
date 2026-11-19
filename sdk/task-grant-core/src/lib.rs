#![no_std]
#![forbid(unsafe_code)]
//! Bounded policy model owned exclusively by a trusted adapter. See the README
//! for identity, lifecycle and check/use assumptions; this is not an IPC API.

/// Authenticated process incarnation, supplied by the trusted execution adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub app: u64,
    pub pid: u64,
    pub epoch: u64,
}

/// Authoritative object identity and revision, never an authorization by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub id: u64,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights {
    pub read: bool,
    pub write: bool,
    pub export: bool,
}

impl Rights {
    fn permits(self, operation: Operation) -> bool {
        match operation {
            Operation::Read => self.read,
            Operation::Write => self.write,
            Operation::Export => self.export,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duration {
    UntilClose,
    Session,
}

/// Internal model reference, retained only by the trusted adapter, not a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    session: u64,
    slot: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    SessionEnded,
    UnknownObject,
    StaleRevision,
    AlreadyRegistered,
    ObjectTableFull,
    GrantTableFull,
    GenerationExhausted,
    EmptyRights,
    StaleHandle,
    WrongActor,
    WrongObject,
    Denied,
}

#[derive(Clone, Copy)]
struct Grant {
    actor: Actor,
    object: Object,
    rights: Rights,
    duration: Duration,
    generation: u64,
}

/// One non-reusable session with fixed object and grant capacities.
/// All mutation methods are trusted model operations, not app requests.
pub struct Grants<const OBJECTS: usize, const GRANTS: usize> {
    session: u64,
    ended: bool,
    generation: u64,
    objects: [Option<Object>; OBJECTS],
    grants: [Option<Grant>; GRANTS],
}

impl<const OBJECTS: usize, const GRANTS: usize> Grants<OBJECTS, GRANTS> {
    /// `session` must be unique across all table instances and retained handles.
    pub const fn new(session: u64) -> Self {
        Self {
            session,
            ended: false,
            generation: 0,
            objects: [None; OBJECTS],
            grants: [None; GRANTS],
        }
    }

    fn active(&self) -> Result<(), Error> {
        if self.ended {
            Err(Error::SessionEnded)
        } else {
            Ok(())
        }
    }

    fn object_slot(&self, id: u64) -> Result<usize, Error> {
        self.active()?;
        self.objects
            .iter()
            .position(|entry| entry.is_some_and(|object| object.id == id))
            .ok_or(Error::UnknownObject)
    }

    fn known(&self, object: Object) -> Result<(), Error> {
        let slot = self.object_slot(object.id)?;
        if self.objects[slot] != Some(object) {
            return Err(Error::StaleRevision);
        }
        Ok(())
    }

    pub fn register_object(&mut self, object: Object) -> Result<(), Error> {
        self.active()?;
        if self.object_slot(object.id).is_ok() {
            return Err(Error::AlreadyRegistered);
        }
        let slot = self
            .objects
            .iter_mut()
            .find(|entry| entry.is_none())
            .ok_or(Error::ObjectTableFull)?;
        *slot = Some(object);
        Ok(())
    }

    /// Advance an existing authoritative revision, retiring all its grants.
    pub fn advance_revision(&mut self, object: Object) -> Result<(), Error> {
        let slot = self.object_slot(object.id)?;
        if self.objects[slot].is_some_and(|old| old.revision >= object.revision) {
            return Err(Error::StaleRevision);
        }
        self.objects[slot] = Some(object);
        self.retire(|grant| grant.object.id == object.id);
        Ok(())
    }

    /// Issue only after the trusted permission decision; never delegate a handle.
    pub fn grant(
        &mut self,
        actor: Actor,
        object: Object,
        rights: Rights,
        duration: Duration,
    ) -> Result<Handle, Error> {
        self.known(object)?;
        if !rights.read && !rights.write && !rights.export {
            return Err(Error::EmptyRights);
        }
        let slot = self
            .grants
            .iter()
            .position(Option::is_none)
            .ok_or(Error::GrantTableFull)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::GenerationExhausted)?;
        self.generation = generation;
        self.grants[slot] = Some(Grant {
            actor,
            object,
            rights,
            duration,
            generation,
        });
        Ok(Handle {
            session: self.session,
            slot,
            generation,
        })
    }

    fn live(&self, handle: Handle) -> Result<&Grant, Error> {
        self.active()?;
        if handle.session != self.session {
            return Err(Error::StaleHandle);
        }
        self.grants
            .get(handle.slot)
            .and_then(Option::as_ref)
            .filter(|grant| grant.generation == handle.generation)
            .ok_or(Error::StaleHandle)
    }

    /// Check every protected operation using authenticated actor and object data.
    pub fn authorize(
        &self,
        handle: Handle,
        actor: Actor,
        object: Object,
        operation: Operation,
    ) -> Result<(), Error> {
        let grant = self.live(handle)?;
        if grant.actor != actor {
            return Err(Error::WrongActor);
        }
        self.known(object)?;
        if grant.object != object {
            return Err(Error::WrongObject);
        }
        if !grant.rights.permits(operation) {
            return Err(Error::Denied);
        }
        Ok(())
    }

    /// A stale revoke must never retire the replacement occupying the same slot.
    pub fn revoke(&mut self, handle: Handle) -> Result<(), Error> {
        self.live(handle)?;
        self.grants[handle.slot] = None;
        Ok(())
    }

    fn retire(&mut self, predicate: impl Fn(&Grant) -> bool) {
        for entry in &mut self.grants {
            if entry.as_ref().is_some_and(&predicate) {
                *entry = None;
            }
        }
    }

    pub fn close(&mut self, actor: Actor, object: Object) -> Result<(), Error> {
        self.known(object)?;
        self.retire(|grant| {
            grant.actor == actor && grant.object == object && grant.duration == Duration::UntilClose
        });
        Ok(())
    }

    /// Called on authenticated process exit; no grant is inherited by a helper.
    pub fn retire_process(&mut self, actor: Actor) -> Result<(), Error> {
        self.active()?;
        self.retire(|grant| grant.actor == actor);
        Ok(())
    }

    pub fn end_session(&mut self) {
        self.ended = true;
        self.grants.fill(None);
    }
}

#[cfg(test)]
mod tests;
