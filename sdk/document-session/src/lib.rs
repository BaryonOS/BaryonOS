#![no_std]
#![forbid(unsafe_code)]
//! One bounded app document draft above the existing catalog transport.
//! Contents are opaque bytes. This module grants no storage authority.

use core::ops::Range;
pub use nieuw_os_catalog_client::Status;
use nieuw_os_catalog_client::{
    Delivery, Error as TransportError, Operation, Reply, Session, Transport,
};
use nieuw_os_sdk_runtime::Error as RuntimeError;

pub const MAX_CONTENT_BYTES: usize = catalog_core::wire::VALUE;
const MAX_NAME_BYTES: usize = catalog_core::wire::NAME;

/// Local app editing policy only. The real service and kernel-bound endpoint
/// decide authority; ReadWrite cannot turn a denied request into a grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditMode {
    ReadOnly,
    ReadWrite,
}

/// An owned, bounded copy that callers may retain before explicit recovery.
/// `loaded_revision` remains the original CAS base until a successful ACK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draft {
    object: u64,
    loaded_revision: u64,
    name: [u8; MAX_NAME_BYTES],
    name_length: usize,
    contents: [u8; MAX_CONTENT_BYTES],
    length: usize,
}

impl Draft {
    pub fn object(&self) -> u64 {
        self.object
    }
    pub fn loaded_revision(&self) -> u64 {
        self.loaded_revision
    }
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
    pub fn contents(&self) -> &[u8] {
        &self.contents[..self.length]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotLoaded,
    NotFound {
        object: u64,
        revision: u64,
    },
    ReadOnly,
    UnsavedChanges,
    ContentTooLarge,
    InvalidRange,
    Conflict,
    /// A preceding save may have committed. No operation resets this state.
    Uncertain,
    Closed,
    RevisionExhausted,
    Service(Status),
    Transport(TransportError),
    UnexpectedReply {
        cleanup: Option<RuntimeError>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Unchanged { revision: u64 },
    Saved { revision: u64 },
}

struct Loaded {
    draft: Draft,
    original: [u8; MAX_CONTENT_BYTES],
    original_length: usize,
}

/// Owns the catalog session's two granted endpoints. Call `close` explicitly;
/// dropping this value does not perform a hidden syscall or retry. After
/// uncertainty, retain `*draft()`, close, and use independently recovered
/// service state/new grants in a new session. There is no reconnect/replay API.
pub struct DocumentSession<T> {
    catalog: Session<T>,
    mode: EditMode,
    loaded: Option<Loaded>,
    conflict: bool,
    uncertain: bool,
}

impl<T: Transport> DocumentSession<T> {
    pub fn new(catalog: Session<T>, mode: EditMode) -> Self {
        Self {
            catalog,
            mode,
            loaded: None,
            conflict: false,
            uncertain: false,
        }
    }

    pub fn draft(&self) -> Option<&Draft> {
        self.loaded.as_ref().map(|entry| &entry.draft)
    }
    pub fn catalog(&self) -> &Session<T> {
        &self.catalog
    }
    pub fn is_closed(&self) -> bool {
        self.catalog.is_closed()
    }
    pub fn is_conflicted(&self) -> bool {
        self.conflict
    }
    pub fn is_uncertain(&self) -> bool {
        self.uncertain
    }

    pub fn is_dirty(&self) -> bool {
        self.loaded.as_ref().is_some_and(|entry| {
            entry.draft.length != entry.original_length || entry.draft.contents != entry.original
        })
    }

    fn available(&self) -> Result<(), Error> {
        if self.uncertain {
            Err(Error::Uncertain)
        } else if self.catalog.is_closed() {
            Err(Error::Closed)
        } else {
            Ok(())
        }
    }

    fn unexpected(&mut self) -> Error {
        Error::UnexpectedReply {
            cleanup: self.catalog.close().err(),
        }
    }

    /// Explicitly load an existing document. No failed read discards the prior
    /// document. Dirty drafts require an explicit `discard_changes` first.
    pub fn open(&mut self, object: u64) -> Result<(), Error> {
        if self.is_dirty() {
            return Err(Error::UnsavedChanges);
        }
        self.available()?;
        match self
            .catalog
            .exchange(Operation::Get { object })
            .map_err(Error::Transport)?
        {
            Reply::Document(document) if document.object == object => {
                let mut draft = Draft {
                    object,
                    loaded_revision: document.revision,
                    name: [0; MAX_NAME_BYTES],
                    name_length: document.name().len(),
                    contents: [0; MAX_CONTENT_BYTES],
                    length: document.value().len(),
                };
                draft.name[..draft.name_length].copy_from_slice(document.name());
                draft.contents[..draft.length].copy_from_slice(document.value());
                self.loaded = Some(Loaded {
                    original: draft.contents,
                    original_length: draft.length,
                    draft,
                });
                self.conflict = false;
                Ok(())
            }
            Reply::NotFound {
                object: missing,
                revision,
            } if missing == object => Err(Error::NotFound { object, revision }),
            Reply::ServiceError(status) => Err(Error::Service(status)),
            _ => Err(self.unexpected()),
        }
    }

    pub fn reopen(&mut self) -> Result<(), Error> {
        let object = self.draft().ok_or(Error::NotLoaded)?.object;
        self.open(object)
    }

    /// Read the granted domain's catalog without changing this local draft.
    /// Dirty or conflicted edits remain intact; closed/uncertain sessions
    /// cannot use this read as an implicit reconnect or recovery operation.
    pub fn list(&mut self) -> Result<catalog_core::wire::Catalog, Error> {
        self.available()?;
        match self
            .catalog
            .exchange(Operation::List)
            .map_err(Error::Transport)?
        {
            Reply::Catalog(catalog) => Ok(catalog),
            Reply::ServiceError(status) => Err(Error::Service(status)),
            _ => Err(self.unexpected()),
        }
    }

    pub fn replace_contents(&mut self, contents: &[u8]) -> Result<(), Error> {
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if contents.len() > MAX_CONTENT_BYTES {
            return Err(Error::ContentTooLarge);
        }
        let entry = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        entry.draft.contents[..contents.len()].copy_from_slice(contents);
        entry.draft.contents[contents.len()..].fill(0);
        entry.draft.length = contents.len();
        Ok(())
    }

    /// Byte-indexed insertion/replacement/removal; imposes no text encoding.
    /// Validation completes before changing any draft byte.
    pub fn replace_range(&mut self, range: Range<usize>, replacement: &[u8]) -> Result<(), Error> {
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        let entry = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        let length = entry.draft.length;
        if range.start > range.end || range.end > length {
            return Err(Error::InvalidRange);
        }
        let new_length = (length - (range.end - range.start))
            .checked_add(replacement.len())
            .filter(|length| *length <= MAX_CONTENT_BYTES)
            .ok_or(Error::ContentTooLarge)?;
        entry
            .draft
            .contents
            .copy_within(range.end..length, range.start + replacement.len());
        entry.draft.contents[range.start..range.start + replacement.len()]
            .copy_from_slice(replacement);
        entry.draft.contents[new_length..].fill(0);
        entry.draft.length = new_length;
        Ok(())
    }

    /// Restore the last loaded/ACKed bytes locally, with no transport action.
    /// This does not refresh a stale revision or resolve an uncertain save.
    pub fn discard_changes(&mut self) -> Result<(), Error> {
        let entry = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        entry.draft.contents = entry.original;
        entry.draft.length = entry.original_length;
        Ok(())
    }

    pub fn save(&mut self) -> Result<SaveOutcome, Error> {
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        self.available()?;
        if self.conflict {
            return Err(Error::Conflict);
        }
        let draft = self.draft().ok_or(Error::NotLoaded)?;
        let (object, revision) = (draft.object, draft.loaded_revision);
        if !self.is_dirty() {
            return Ok(SaveOutcome::Unchanged { revision });
        }
        let next_revision = revision.checked_add(1).ok_or(Error::RevisionExhausted)?;
        let draft = &self.loaded.as_ref().unwrap().draft;
        let response = self.catalog.exchange(Operation::Put {
            object,
            expected_revision: revision,
            name: draft.name(),
            value: draft.contents(),
        });
        match response {
            Ok(Reply::Mutation {
                object: saved,
                revision: acknowledged,
            }) if saved == object && acknowledged == next_revision => {
                let entry = self.loaded.as_mut().unwrap();
                entry.draft.loaded_revision = acknowledged;
                entry.original = entry.draft.contents;
                entry.original_length = entry.draft.length;
                Ok(SaveOutcome::Saved {
                    revision: acknowledged,
                })
            }
            Ok(Reply::ServiceError(Status::Stale)) => {
                self.conflict = true;
                Err(Error::Conflict)
            }
            Ok(Reply::ServiceError(status)) => {
                self.uncertain |= matches!(status, Status::Unknown | Status::Timeout | Status::Io);
                Err(Error::Service(status))
            }
            Err(error) => {
                self.uncertain |= match error {
                    TransportError::Interrupted { delivery, .. } => {
                        delivery != Delivery::NotEnqueued
                    }
                    TransportError::Cleanup { .. } => true,
                    _ => false,
                };
                Err(Error::Transport(error))
            }
            _ => {
                self.uncertain = true;
                Err(self.unexpected())
            }
        }
    }

    /// Attempt both endpoint closes through the existing transport; keep draft.
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        self.catalog.close()
    }
}
