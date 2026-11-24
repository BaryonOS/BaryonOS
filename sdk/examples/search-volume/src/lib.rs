#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
//! Local name and collection-label search over the M6e v6 volume IPC
//! (U05-T3 S1, host-tested only). One `Search` serves the caller's own
//! kernel-bound volume domain; the volume service authorizes every request by
//! its endpoint, so this adapter holds no other rights.
//!
//! Freshness contract (U05-T3 lead decisions B1-B4):
//! - `Client::validate_reply` is the primary reply check; the checks here are
//!   defence in depth only.
//! - A clean index needs a strictly newer LIST domain generation. A dirty
//!   index may rebuild at the last applied generation, but never an older one.
//! - Rename and delete apply the revision the service returned; WRITE,
//!   COLLECTION and META force a full LIST plus COLLECTION_STAT rebuild.
//! - A mutation the service applied but the index did not follow leaves the
//!   index dirty: queries are refused until a rebuild succeeds. Only a
//!   rebuild clears it. A refused mutation changed nothing and keeps the index.
//!   A `Protocol` mutation reply is uncertain (the service may have applied
//!   it), so it also leaves the index dirty.
//! - `Unknown`/`Timeout` revokes the grant. The epoch is the volume epoch,
//!   fixed for the boot, so a revoke is terminal until a reboot. After a
//!   revoke no further request is sent.
//!
//! The grant is issued in the same untrusted app process (spec I1/L1): it
//! bounds this cache's lifecycle and carries no security value natively.

use nieuw_os_search_index::{Document, Grant, Index, Results};
use volume_core::{
    checksum_update,
    client::{Client, Transport},
    wire::ListEntry,
    Status, COLLECTIONS, FNV_OFFSET, NAME, PER_DOMAIN,
};

pub use nieuw_os_search_index::Error as IndexError;

/// Name plus collection label (B4).
pub const TEXT: usize = 2 * NAME;
/// A 32-byte name and a 32-byte label hold at most 16 tokens each (B4).
pub const TOKENS: usize = 32;
/// One domain holds at most `PER_DOMAIN` live objects, so results never overflow.
pub const RESULTS: usize = PER_DOMAIN;
/// Start value for `digest`.
pub const DIGEST_START: u64 = FNV_OFFSET;

pub type SearchIndex = Index<PER_DOMAIN, TEXT, TOKENS, PER_DOMAIN>;
pub type Hits = Results<RESULTS>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Volume(Status),
    Index(IndexError),
    /// The object is not in this adapter's domain; nothing was sent.
    Foreign,
    /// The LIST generation is older, or equal while the index is clean.
    StaleGeneration,
    /// A mutation was not applied to the index; rebuild first.
    RebuildRequired,
}

impl From<IndexError> for Error {
    fn from(error: IndexError) -> Self {
        Self::Index(error)
    }
}

type Labels = [([u8; NAME], usize); COLLECTIONS];

fn label(labels: &Labels, collection: u8) -> &[u8] {
    match usize::from(collection).checked_sub(1) {
        Some(i) if i < COLLECTIONS => &labels[i].0[..labels[i].1],
        _ => b"",
    }
}

/// Defence in depth over one LIST entry (B1): `Client::validate_reply`
/// already refuses all of this. Returns the object's slot in `domain`.
pub fn check_entry(domain: usize, entry: &ListEntry) -> Result<usize, Error> {
    if entry.object / PER_DOMAIN as u64 != domain as u64 {
        return Err(Error::Foreign);
    }
    if entry.revision == 0
        || entry.name_length > NAME
        || usize::from(entry.collection) > COLLECTIONS
    {
        return Err(Error::Volume(Status::Protocol));
    }
    Ok((entry.object % PER_DOMAIN as u64) as usize)
}

pub struct Search<T: Transport> {
    /// Private: a direct mutation would bypass the forced rebuild.
    client: Client<T>,
    domain: usize,
    grant: Grant,
    revoked: bool,
    index: SearchIndex,
    /// Domain generation of the last applied rebuild; `None` before the first.
    generation: Option<u64>,
    dirty: bool,
    /// Collection of each object and the labels, as of the last rebuild.
    collection_of: [u8; PER_DOMAIN],
    labels: Labels,
}

impl<T: Transport> Search<T> {
    /// Grants the index with the client's volume epoch, never a caller value.
    pub fn new(client: Client<T>, domain: usize) -> Result<Self, Error> {
        let mut index = SearchIndex::new();
        let grant = index.grant(domain, client.epoch)?;
        Ok(Self {
            client,
            domain,
            grant,
            revoked: false,
            index,
            generation: None,
            dirty: false,
            collection_of: [0; PER_DOMAIN],
            labels: [([0; NAME], 0); COLLECTIONS],
        })
    }

    /// Grant again with the same volume epoch. The index refuses any epoch it
    /// has seen (`Denied`), so after a revoke this cannot restore search.
    pub fn regrant(&mut self) -> Result<(), Error> {
        self.grant = self.index.grant(self.domain, self.client.epoch)?;
        self.revoked = false;
        self.generation = None;
        Ok(())
    }

    /// `Unknown`/`Timeout` revoke; every failure is returned unchanged.
    fn fail(&mut self, status: Status) -> Error {
        if matches!(status, Status::Unknown | Status::Timeout) && !self.revoked {
            self.revoked = true;
            // Cannot fail: the grant is live until this point.
            let _ = self.index.revoke(self.grant);
        }
        Error::Volume(status)
    }

    /// For mutation replies: `Protocol` may follow an applied change, so the
    /// index waits for a rebuild; otherwise as `fail`.
    fn uncertain(&mut self, status: Status) -> Error {
        if status == Status::Protocol {
            self.dirty = true;
        }
        self.fail(status)
    }

    /// Refuses every operation after a revoke, before any request is sent.
    fn live(&self) -> Result<(), Error> {
        if self.revoked {
            return Err(Error::Index(IndexError::Denied));
        }
        Ok(())
    }

    /// Names longer than the protocol limit are refused before encoding.
    fn named(name: &[u8]) -> Result<(), Error> {
        if name.len() > NAME {
            return Err(Error::Volume(Status::Range));
        }
        Ok(())
    }

    fn own(&self, object: u64) -> Result<usize, Error> {
        self.live()?;
        if object / PER_DOMAIN as u64 != self.domain as u64 {
            return Err(Error::Foreign);
        }
        Ok((object % PER_DOMAIN as u64) as usize)
    }

    fn list(&mut self, out: &mut [ListEntry; PER_DOMAIN]) -> Result<(usize, u64), Error> {
        let listed = self.client.list(self.domain, out);
        listed.map_err(|status| self.fail(status))
    }

    /// Full snapshot: LIST, all COLLECTION_STATs, then LIST again. Equal
    /// generations bind the labels to the listing, because collection
    /// revisions are part of the domain generation. Nothing is published on
    /// any error.
    pub fn rebuild(&mut self) -> Result<(), Error> {
        self.live()?;
        let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
        let (count, generation) = self.list(&mut entries)?;
        let mut labels: Labels = [([0; NAME], 0); COLLECTIONS];
        for (i, label) in labels.iter_mut().enumerate() {
            let info = self.client.collection_stat(self.domain, i + 1);
            let info = info.map_err(|status| self.fail(status))?;
            if info.present {
                *label = (info.name, info.name_length);
            }
        }
        let (_, again) = self.list(&mut [ListEntry::EMPTY; PER_DOMAIN])?;
        if again != generation {
            return Err(Error::Volume(Status::Stale));
        }
        // A dirty index may rebuild at the last applied generation: an
        // uncertain mutation reply need not have advanced it.
        if self
            .generation
            .is_some_and(|last| generation < last || (generation == last && !self.dirty))
        {
            return Err(Error::StaleGeneration);
        }
        let mut collection_of = [0; PER_DOMAIN];
        let mut documents = [Document {
            object: 0,
            revision: 0,
            name: b"",
            collections: b"",
            metadata: b"",
            content: b"",
        }; PER_DOMAIN];
        for (document, entry) in documents.iter_mut().zip(&entries[..count]) {
            // Defence in depth only: `Client::validate_reply` already refused these.
            let slot = check_entry(self.domain, entry)?;
            collection_of[slot] = entry.collection;
            *document = Document {
                object: entry.object,
                revision: entry.revision,
                name: &entry.name[..entry.name_length],
                collections: label(&labels, entry.collection),
                metadata: b"",
                content: b"",
            };
        }
        self.index.rebuild(self.grant, &documents[..count])?;
        self.generation = Some(generation);
        self.collection_of = collection_of;
        self.labels = labels;
        self.dirty = false;
        Ok(())
    }

    /// Run a mutation that the index cannot follow incrementally, then rebuild.
    fn forced<R>(
        &mut self,
        mutation: impl FnOnce(&mut Client<T>) -> Result<R, Status>,
    ) -> Result<R, Error> {
        self.live()?;
        let value = mutation(&mut self.client).map_err(|status| self.uncertain(status))?;
        self.dirty = true;
        self.rebuild()?;
        Ok(value)
    }

    /// Write a whole object, then rebuild. Returns the new revision.
    pub fn write(
        &mut self,
        object: u64,
        expected: u64,
        name: &[u8],
        content: &[u8],
    ) -> Result<u64, Error> {
        self.own(object)?;
        Self::named(name)?;
        self.forced(|client| {
            client
                .write(object, expected, name, content.len(), |offset, buffer| {
                    buffer.copy_from_slice(&content[offset..offset + buffer.len()]);
                })
                .map(|(revision, _)| revision)
        })
    }

    /// Create, rename (non-empty `name`) or delete (empty) a collection.
    pub fn collection(&mut self, index: usize, expected: u64, name: &[u8]) -> Result<u64, Error> {
        Self::named(name)?;
        let domain = self.domain;
        self.forced(|client| {
            client
                .collection(domain, index, expected, name)
                .map_err(|(status, _)| status)
        })
    }

    pub fn meta(
        &mut self,
        object: u64,
        expected: u64,
        tags: u32,
        kind: u16,
        collection: u8,
    ) -> Result<u64, Error> {
        self.own(object)?;
        self.forced(|client| {
            client
                .meta(object, expected, tags, kind, collection)
                .map_err(|(status, _)| status)
        })
    }

    /// Rename, then index the new name at the revision RENAME returned.
    pub fn rename(&mut self, object: u64, expected: u64, name: &[u8]) -> Result<u64, Error> {
        let slot = self.own(object)?;
        Self::named(name)?;
        let revision = self.client.rename(object, expected, name);
        let revision = revision.map_err(|status| self.uncertain(status))?;
        let document = Document {
            object,
            revision,
            name,
            collections: label(&self.labels, self.collection_of[slot]),
            metadata: b"",
            content: b"",
        };
        let applied = self.index.update(self.grant, document);
        self.applied(applied)?;
        Ok(revision)
    }

    /// Delete, then record the tombstone at the revision DELETE returned.
    pub fn delete(&mut self, object: u64, expected: u64) -> Result<u64, Error> {
        let slot = self.own(object)?;
        let revision = self.client.delete(object, expected);
        let revision = revision.map_err(|(status, _)| self.uncertain(status))?;
        let applied = self.index.delete(self.grant, object, revision);
        self.applied(applied)?;
        self.collection_of[slot] = 0;
        Ok(revision)
    }

    /// The outcome of an incremental change the service already made; on
    /// failure the index no longer matches the volume and waits for a rebuild.
    fn applied(&mut self, result: Result<(), IndexError>) -> Result<(), Error> {
        result.map_err(|error| {
            self.dirty = true;
            Error::Index(error)
        })
    }

    /// Answers only from a complete, applied snapshot of a live grant.
    pub fn query(&self, query: &[u8]) -> Result<Hits, Error> {
        self.live()?;
        if self.dirty || self.generation.is_none() {
            return Err(Error::RebuildRequired);
        }
        Ok(self.index.query(&[self.grant], query)?)
    }
}

/// FNV-64 over the hits sorted by object, each as little-endian
/// `(object, revision)`; chain calls to digest a fixed query set.
/// tests/test_search_oracle.py implements the same function.
pub fn digest(value: u64, hits: &Hits) -> u64 {
    digest_pairs(value, hits.iter().map(|hit| (hit.object, hit.revision)))
}

/// At most `RESULTS` pairs.
fn digest_pairs(value: u64, pairs: impl Iterator<Item = (u64, u64)>) -> u64 {
    let mut sorted = [(0u64, 0u64); RESULTS];
    let mut count = 0;
    for pair in pairs {
        sorted[count] = pair;
        count += 1;
    }
    sorted[..count].sort_unstable();
    sorted[..count]
        .iter()
        .fold(value, |value, (object, revision)| {
            let value = checksum_update(value, &object.to_le_bytes());
            checksum_update(value, &revision.to_le_bytes())
        })
}

#[cfg(test)]
mod tests;
