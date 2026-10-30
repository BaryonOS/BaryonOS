//! Durable NAS snapshots using the existing volume client's bounded streaming.
//! The trusted startup controller owns an exclusive, kernel-scoped volume
//! endpoint. Object numbers and caller-provided identity are not capabilities.
#![no_std]
#![forbid(unsafe_code)]

use nieuw_os_nas_core::{validate_snapshot, Error, Storage, SNAPSHOT_BYTES};
use volume_core::{
    checksum_update,
    client::{Client, Stat, Transport},
    Status, FNV_OFFSET, OBJECTS,
};

const NAME: &[u8] = b"nas-authority-v1";

pub struct VolumeStorage<T: Transport> {
    client: Client<T>,
    object: u64,
    revision: Option<u64>,
}

impl<T: Transport> VolumeStorage<T> {
    /// The caller must exclusively own this designated object and transport.
    /// This does not grant volume access; the volume service checks its ingress.
    pub fn new(client: Client<T>, object: u64) -> Result<Self, Error> {
        if object >= OBJECTS as u64 {
            return Err(Error::Invalid);
        }
        Ok(Self {
            client,
            object,
            revision: None,
        })
    }

    pub fn into_client(self) -> Client<T> {
        self.client
    }

    fn stat(&mut self) -> Result<Option<Stat>, Error> {
        match self.client.stat(self.object) {
            Ok(stat) => {
                if stat.length != SNAPSHOT_BYTES
                    || &stat.name[..stat.name_length] != NAME
                    || stat.tags != 0
                    || stat.kind != 0
                    || stat.collection != 0
                {
                    return Err(Error::Corrupt);
                }
                Ok(Some(stat))
            }
            Err((Status::NotFound, 0)) => Ok(None),
            // A tombstone is not a never-initialized store.
            Err((Status::NotFound, _)) => Err(Error::Corrupt),
            Err((status, _)) => Err(read_error(status)),
        }
    }

    fn read(&mut self, stat: &Stat, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        self.client
            .read(
                self.object,
                stat.revision,
                SNAPSHOT_BYTES,
                |offset, bytes| {
                    out[offset..offset + bytes.len()].copy_from_slice(bytes);
                },
            )
            .map_err(read_error)?;
        if checksum_update(FNV_OFFSET, out) != stat.content_sum {
            return Err(Error::Corrupt);
        }
        validate_snapshot(out)
    }

    /// Explicit startup-repair action, never an implicit retry in `load`.
    /// A snapshot may have committed while its history purge failed. Validate
    /// the current version before retiring that history, then reopen the core.
    /// This cannot repair corrupt content or choose an older authority state.
    pub fn finish_cleanup(&mut self) -> Result<(), Error> {
        self.revision = None;
        let stat = self.stat()?.ok_or(Error::Corrupt)?;
        let mut bytes = [0; SNAPSHOT_BYTES];
        self.read(&stat, &mut bytes)?;
        if stat.history_count != 0 {
            if stat.revision == u64::MAX {
                return Err(Error::Full);
            }
            self.client
                .purge(self.object, stat.revision)
                .map_err(|(status, _)| write_error(status))?;
        }
        Ok(())
    }
}

impl<T: Transport> Storage for VolumeStorage<T> {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        self.revision = None;
        let Some(stat) = self.stat()? else {
            self.revision = Some(0);
            return Ok(None);
        };
        if stat.history_count != 0 {
            return Err(Error::ReopenRequired);
        }
        self.read(&stat, out)?;
        self.revision = Some(stat.revision);
        Ok(Some(SNAPSHOT_BYTES))
    }

    fn replace(&mut self, snapshot: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        let expected = self.revision.take().ok_or(Error::ReopenRequired)?;
        validate_snapshot(snapshot)?;
        // Both WRITE and history PURGE increment the volume revision.
        if expected >= u64::MAX - 1 {
            return Err(Error::Full);
        }
        let (mut revision, _) = self
            .client
            .write(
                self.object,
                expected,
                NAME,
                SNAPSHOT_BYTES,
                |offset, out| out.copy_from_slice(&snapshot[offset..offset + out.len()]),
            )
            .map_err(write_error)?;
        if expected != 0 {
            revision = self
                .client
                .purge(self.object, revision)
                .map_err(|(status, _)| write_error(status))?;
        }
        self.revision = Some(revision);
        Ok(())
    }
}

fn read_error(status: Status) -> Error {
    match status {
        Status::Access => Error::Denied,
        Status::Stale => Error::Conflict,
        Status::NotFound => Error::NotFound,
        Status::Full => Error::Full,
        Status::Quota => Error::Quota,
        Status::Invalid | Status::Range | Status::NoSys => Error::Invalid,
        Status::Corrupt | Status::Protocol | Status::Ok => Error::Corrupt,
        Status::Io | Status::Timeout | Status::NoDevice => Error::Io,
        Status::Unknown => Error::Unknown,
        Status::Busy => Error::ReopenRequired,
    }
}

fn write_error(status: Status) -> Error {
    match status {
        // Transport failure may hide a successful COMMIT or PURGE reply.
        Status::Io | Status::Timeout | Status::NoDevice | Status::Protocol => Error::Unknown,
        other => read_error(other),
    }
}
