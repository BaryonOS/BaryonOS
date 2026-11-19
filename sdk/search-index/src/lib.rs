//! Bounded local search cache. See README.md for the trusted adapter contract.
#![no_std]
#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests;

/// Matches the current catalogue's two domains and two object IDs per domain.
pub const DOMAINS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Denied,
    InvalidIdentity,
    StaleRevision,
    InvalidUtf8,
    TextFull,
    TokensFull,
    EntriesFull,
    ResultsFull,
    Corrupt,
    DuplicateObject,
    EmptyQuery,
}

/// Only a trusted adapter may establish grants from authenticated session state.
/// This value is not a kernel capability and does not authenticate its holder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    domain: usize,
    epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub domain: usize,
    pub object: u64,
    pub revision: u64,
}

/// Text fields from one trusted catalogue document and its associated metadata.
#[derive(Clone, Copy)]
pub struct Document<'a> {
    pub object: u64,
    pub revision: u64,
    pub name: &'a [u8],
    pub collections: &'a [u8],
    pub metadata: &'a [u8],
    pub content: &'a [u8],
}

#[derive(Clone, Copy)]
struct Text<const B: usize, const T: usize> {
    bytes: [u8; B],
    length: usize,
    tokens: [(usize, usize); T],
    count: usize,
}

impl<const B: usize, const T: usize> Text<B, T> {
    const EMPTY: Self = Self {
        bytes: [0; B],
        length: 0,
        tokens: [(0, 0); T],
        count: 0,
    };

    fn new(fields: &[&[u8]]) -> Result<Self, Error> {
        let mut text = Self::EMPTY;
        for field in fields {
            let end = text
                .length
                .checked_add(field.len())
                .ok_or(Error::TextFull)?;
            if end > B {
                return Err(Error::TextFull);
            }
            let value = core::str::from_utf8(field).map_err(|_| Error::InvalidUtf8)?;
            let base = text.length;
            text.bytes[base..end].copy_from_slice(field);
            for token in value.split_whitespace() {
                if text.count == T {
                    return Err(Error::TokensFull);
                }
                let start = base + (token.as_ptr() as usize - value.as_ptr() as usize);
                text.tokens[text.count] = (start, start + token.len());
                text.count += 1;
            }
            text.length = end;
        }
        Ok(text)
    }

    fn matches(&self, query: &Self) -> bool {
        query.tokens[..query.count].iter().all(|&(start, end)| {
            self.tokens[..self.count]
                .iter()
                .any(|&(a, b)| self.bytes[a..b].eq_ignore_ascii_case(&query.bytes[start..end]))
        })
    }
}

#[derive(Clone, Copy)]
struct Entry<const B: usize, const T: usize> {
    hit: Hit,
    live: bool,
    text: Text<B, T>,
    digest: [u8; 32],
}

impl<const B: usize, const T: usize> Entry<B, T> {
    fn seal(&mut self) {
        self.digest = self.hash();
    }

    fn hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update((self.hit.domain as u64).to_le_bytes());
        hash.update(self.hit.object.to_le_bytes());
        hash.update(self.hit.revision.to_le_bytes());
        hash.update([u8::from(self.live)]);
        hash.update((self.text.length as u64).to_le_bytes());
        hash.update((self.text.count as u64).to_le_bytes());
        hash.update(self.text.bytes);
        for (a, b) in self.text.tokens {
            hash.update((a as u64).to_le_bytes());
            hash.update((b as u64).to_le_bytes());
        }
        hash.finalize().into()
    }

    fn validate(&self) -> Result<(), Error> {
        if self.hit.domain >= DOMAINS
            || self.hit.object / 2 != self.hit.domain as u64
            || self.hit.revision == 0
            || self.text.length > B
            || self.text.count > T
            || self.digest != self.hash()
        {
            return Err(Error::Corrupt);
        }
        let value = core::str::from_utf8(&self.text.bytes[..self.text.length])
            .map_err(|_| Error::Corrupt)?;
        for &(a, b) in &self.text.tokens[..self.text.count] {
            if a >= b || b > value.len() || !value.is_char_boundary(a) || !value.is_char_boundary(b)
            {
                return Err(Error::Corrupt);
            }
        }
        Ok(())
    }
}

/// E bounds live records plus revision tombstones per domain; B bounds total UTF-8 bytes
/// per record/query (and each token); T bounds tokens per record/query.
/// Grant epochs and tombstones are retained until a strictly newer grant.
pub struct Index<const E: usize, const B: usize, const T: usize> {
    epochs: [u64; DOMAINS],
    granted: [bool; DOMAINS],
    entries: [[Option<Entry<B, T>>; E]; DOMAINS],
}

impl<const E: usize, const B: usize, const T: usize> Default for Index<E, B, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const E: usize, const B: usize, const T: usize> Index<E, B, T> {
    pub const fn new() -> Self {
        Self {
            epochs: [0; DOMAINS],
            granted: [false; DOMAINS],
            entries: [[None; E]; DOMAINS],
        }
    }

    /// Trusted adapter only: install a strictly newer authenticated epoch.
    /// Replaying any installed or revoked epoch is refused, including after revoke.
    pub fn grant(&mut self, domain: usize, epoch: u64) -> Result<Grant, Error> {
        if domain >= DOMAINS || epoch == 0 || epoch <= self.epochs[domain] {
            return Err(Error::Denied);
        }
        self.epochs[domain] = epoch;
        self.granted[domain] = true;
        // Do not inspect possibly corrupt cache identities while purging.
        self.entries[domain] = [None; E];
        Ok(Grant { domain, epoch })
    }

    fn authorize(&self, grant: Grant) -> Result<(), Error> {
        if grant.domain >= DOMAINS
            || !self.granted[grant.domain]
            || self.epochs[grant.domain] != grant.epoch
        {
            return Err(Error::Denied);
        }
        Ok(())
    }

    pub fn revoke(&mut self, grant: Grant) -> Result<(), Error> {
        self.authorize(grant)?;
        self.granted[grant.domain] = false;
        self.entries[grant.domain] = [None; E];
        Ok(())
    }

    fn identity(&self, grant: Grant, object: u64, revision: u64) -> Result<(), Error> {
        self.authorize(grant)?;
        if object / 2 != grant.domain as u64 || revision == 0 {
            return Err(Error::InvalidIdentity);
        }
        Ok(())
    }

    fn validate_domain(&self, domain: usize) -> Result<(), Error> {
        for entry in self.entries[domain].iter().flatten() {
            entry.validate()?;
            if entry.hit.domain != domain {
                return Err(Error::Corrupt);
            }
        }
        Ok(())
    }

    pub fn update(&mut self, grant: Grant, doc: Document<'_>) -> Result<(), Error> {
        self.identity(grant, doc.object, doc.revision)?;
        self.validate_domain(grant.domain)?;
        let entries = &mut self.entries[grant.domain];
        let slot = entries
            .iter()
            .position(|e| e.is_some_and(|e| e.hit.object == doc.object));
        if slot.is_some_and(|i| entries[i].is_some_and(|e| doc.revision <= e.hit.revision)) {
            return Err(Error::StaleRevision);
        }
        let index = slot
            .or_else(|| entries.iter().position(Option::is_none))
            .ok_or(Error::EntriesFull)?;
        let text = Text::new(&[doc.name, doc.collections, doc.metadata, doc.content])?;
        let mut entry = Entry {
            hit: Hit {
                domain: grant.domain,
                object: doc.object,
                revision: doc.revision,
            },
            live: true,
            text,
            digest: [0; 32],
        };
        entry.seal();
        entries[index] = Some(entry);
        Ok(())
    }

    /// Record the confirmed catalogue deletion revision, retaining a tombstone.
    pub fn delete(&mut self, grant: Grant, object: u64, revision: u64) -> Result<(), Error> {
        self.update(
            grant,
            Document {
                object,
                revision,
                name: b"",
                collections: b"",
                metadata: b"",
                content: b"",
            },
        )?;
        let entry = self.entries[grant.domain]
            .iter_mut()
            .flatten()
            .find(|e| e.hit.object == object)
            .ok_or(Error::Corrupt)?;
        entry.live = false;
        entry.seal();
        Ok(())
    }

    /// Atomic complete snapshot replacement for one domain. A trusted adapter
    /// must authenticate snapshot freshness; this is also the corruption recovery
    /// route and therefore does not trust old cached revisions or identities.
    pub fn rebuild(&mut self, grant: Grant, snapshot: &[Document<'_>]) -> Result<(), Error> {
        self.authorize(grant)?;
        if snapshot.len() > E {
            return Err(Error::EntriesFull);
        }
        let mut replacement = Self::new();
        replacement.epochs = self.epochs;
        replacement.granted = self.granted;
        for (i, doc) in snapshot.iter().enumerate() {
            if snapshot[..i].iter().any(|other| other.object == doc.object) {
                return Err(Error::DuplicateObject);
            }
            replacement.update(grant, *doc)?;
        }
        // Validate old entries individually: a corrupt record is replaced from
        // the trusted snapshot, while every intact record keeps its revision
        // floor or tombstone.
        for old in self.entries[grant.domain]
            .iter()
            .flatten()
            .filter(|old| old.validate().is_ok() && old.hit.domain == grant.domain)
        {
            if let Some(doc) = snapshot.iter().find(|doc| doc.object == old.hit.object) {
                if doc.revision < old.hit.revision {
                    return Err(Error::StaleRevision);
                }
                if !old.live && doc.revision == old.hit.revision {
                    return Err(Error::StaleRevision);
                }
            } else {
                let slot = replacement.entries[grant.domain]
                    .iter_mut()
                    .find(|e| e.is_none())
                    .ok_or(Error::EntriesFull)?;
                let mut tombstone = *old;
                tombstone.live = false;
                tombstone.text = Text::EMPTY;
                tombstone.seal();
                *slot = Some(tombstone);
            }
        }
        self.entries[grant.domain] = replacement.entries[grant.domain];
        Ok(())
    }

    /// Results are returned by value only after complete validation. R bounds
    /// result capacity; overflow is an error, never partial output or eviction.
    pub fn query<const R: usize>(
        &self,
        grants: &[Grant],
        query: &[u8],
    ) -> Result<Results<R>, Error> {
        if grants.len() > DOMAINS {
            return Err(Error::Denied);
        }
        for (i, grant) in grants.iter().enumerate() {
            self.authorize(*grant)?;
            if grants[..i].iter().any(|other| other.domain == grant.domain) {
                return Err(Error::Denied);
            }
        }
        let query = Text::<B, T>::new(&[query])?;
        if query.count == 0 {
            return Err(Error::EmptyQuery);
        }
        for grant in grants {
            self.validate_domain(grant.domain)?;
        }
        let mut results = Results {
            hits: [None; R],
            length: 0,
        };
        for (domain, entries) in self.entries.iter().enumerate() {
            if !grants.iter().any(|g| g.domain == domain) {
                continue;
            }
            for entry in entries.iter().flatten() {
                if entry.live && entry.text.matches(&query) {
                    if results.length == R {
                        return Err(Error::ResultsFull);
                    }
                    results.hits[results.length] = Some(entry.hit);
                    results.length += 1;
                }
            }
        }
        Ok(results)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Results<const R: usize> {
    hits: [Option<Hit>; R],
    length: usize,
}
impl<const R: usize> Results<R> {
    pub fn iter(&self) -> impl Iterator<Item = &Hit> {
        self.hits[..self.length].iter().flatten()
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}
