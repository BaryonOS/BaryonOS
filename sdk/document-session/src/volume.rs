//! Streaming document editing over volume v7 upload-bound envelopes.
//! The caller supplies storage for draft/original/read staging; no allocator,
//! document-sized stack allocation, block capability or new kernel ABI.
use core::ops::Range;

use crate::{EditMode, SaveOutcome};
use nieuw_os_catalog_client::{Delivery, Transport, MAX_POLL_BUDGET};
use nieuw_os_sdk_runtime::{Error as RuntimeError, Outgoing, ReceiveEndpoint, SendEndpoint};
use volume_core::{client, upload, wire, NAME, PER_DOMAIN};
pub use volume_core::{
    client::{CollectionInfo, HistoryInfo, Stat},
    wire::ListEntry,
    Status, MAX_LENGTH,
};

/// A collection is a bounded index in the already granted volume domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListFilter {
    All,
    Uncollected,
    Collection(u8),
}

impl ListFilter {
    fn wire(self) -> Result<u32, Error> {
        match self {
            Self::All => Ok(0),
            Self::Uncollected => Ok(1),
            Self::Collection(index) if (1..=volume_core::COLLECTIONS as u8).contains(&index) => {
                Ok(u32::from(index) + 1)
            }
            Self::Collection(_) => Err(Error::InvalidRequest),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfiguration,
    InvalidRequest,
    RequestIdsExhausted,
    Closed,
    NotLoaded,
    NotFound {
        object: u64,
        revision: u64,
    },
    UnsavedChanges,
    ReadOnly,
    ContentTooLarge,
    InvalidRange,
    InvalidUtf8,
    Busy,
    Conflict,
    Uncertain,
    /// RESTORE was acknowledged, but refreshing the editor failed. The old
    /// draft remains available; explicitly reopen, never repeat the restore.
    CommittedRefreshFailed {
        revision: u64,
    },
    Service(Status),
    Interrupted {
        delivery: Delivery,
        transport: Option<RuntimeError>,
        cleanup: Option<RuntimeError>,
    },
    Protocol {
        cleanup: Option<RuntimeError>,
    },
}

/// Owns a previously granted endpoint pair. A published request is never
/// resent. Transport or protocol failure closes both endpoints; a new session
/// needs fresh grants after independent service recovery.
pub struct VolumeSession<T> {
    transport: T,
    epoch: u64,
    service: u64,
    domain: usize,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    budget: usize,
    next_id: u64,
    closed: bool,
}

impl<T: Transport> VolumeSession<T> {
    pub fn new(
        transport: T,
        epoch: u64,
        service: u64,
        domain: usize,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        budget: usize,
    ) -> Result<Self, Error> {
        if epoch == 0
            || service == 0
            || domain >= volume_core::DOMAINS
            || send.handle() == receive.handle()
            || budget == 0
            || budget > MAX_POLL_BUDGET
        {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            transport,
            epoch,
            service,
            domain,
            send,
            receive,
            budget,
            next_id: 1,
            closed: false,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        let send = self.transport.close(self.send.handle());
        let receive = self.transport.close(self.receive.handle());
        send.and(receive)
    }

    fn fresh(&mut self) -> Result<u64, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or(Error::RequestIdsExhausted)?;
        Ok(id)
    }

    fn header(&mut self, operation: u16, object: u64) -> Result<wire::Header, Error> {
        Ok(wire::Header::new(
            operation,
            self.fresh()?,
            self.epoch,
            object,
        ))
    }

    fn interrupted(&mut self, delivery: Delivery, transport: Option<RuntimeError>) -> Error {
        Error::Interrupted {
            delivery,
            transport,
            cleanup: self.close().err(),
        }
    }

    fn protocol_with(&mut self, cleanup: Option<RuntimeError>) -> Error {
        let endpoints = self.close().err();
        Error::Protocol {
            cleanup: cleanup.or(endpoints),
        }
    }

    fn protocol(&mut self) -> Error {
        self.protocol_with(None)
    }

    fn exchange(
        &mut self,
        h: &wire::Header,
        data: &[u8],
    ) -> Result<(wire::Header, [u8; wire::SIZE]), Error> {
        self.exchange_upload(h, data, 0)
    }

    fn exchange_upload(
        &mut self,
        h: &wire::Header,
        data: &[u8],
        token: u32,
    ) -> Result<(wire::Header, [u8; wire::SIZE]), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        // Validate all app-controlled sizes before wire::encode's assertions.
        if data.len() > wire::DATA || h.length as usize != data.len() {
            return Err(Error::InvalidRequest);
        }
        let request = upload::encode(h, data, token);
        upload::validate_request(&request, self.epoch, self.domain).map_err(Error::Service)?;
        let message = Outgoing::new(upload::TAG, &request).map_err(|_| Error::InvalidRequest)?;
        let mut queued = false;
        for _ in 0..self.budget {
            match self.transport.send(self.send, &message) {
                Ok(()) => {
                    queued = true;
                    break;
                }
                Err(e) if e.is_again() => {
                    if let Err(e) = self.transport.yield_now() {
                        return Err(self.interrupted(Delivery::NotEnqueued, Some(e)));
                    }
                }
                Err(e) => {
                    let delivery = if matches!(e, RuntimeError::Kernel(code) if code < 0) {
                        Delivery::NotEnqueued
                    } else {
                        Delivery::Unknown
                    };
                    return Err(self.interrupted(delivery, Some(e)));
                }
            }
        }
        if !queued {
            return Err(self.interrupted(Delivery::NotEnqueued, None));
        }
        for _ in 0..self.budget {
            match self.transport.receive(self.receive) {
                Ok(message) => {
                    let unexpected_caps = message.cap_count() != 0;
                    let mut cleanup = None;
                    for i in 0..message.cap_count() {
                        if let Some(cap) = message.cap(i) {
                            // Reject the entire envelope even after closing its
                            // unexpected delegated handles. Never strip proof.
                            if let Err(error) = self.transport.close(cap.handle) {
                                cleanup.get_or_insert(error);
                            }
                        }
                    }
                    if unexpected_caps
                        || message.sender() != self.service
                        || message.tag() != upload::TAG
                        || message.payload().len() != wire::SIZE
                    {
                        return Err(self.protocol_with(cleanup));
                    }
                    let mut body = [0; wire::SIZE];
                    body.copy_from_slice(message.payload());
                    let reply = match upload::validate_reply(h, token, &body, self.epoch) {
                        Ok(reply) => reply,
                        Err(_) => return Err(self.protocol()),
                    };
                    let status = Status::from_code(i64::from(reply.status));
                    if status != Status::Ok {
                        if matches!(
                            status,
                            Status::Unknown | Status::Timeout | Status::NoDevice | Status::Corrupt
                        ) {
                            let _ = self.close();
                        }
                        if status == Status::NotFound {
                            return Err(Error::NotFound {
                                object: h.object,
                                revision: reply.revision,
                            });
                        }
                        return Err(Error::Service(status));
                    }
                    return Ok((reply, body));
                }
                Err(e) if e.is_again() => {
                    if let Err(e) = self.transport.yield_now() {
                        return Err(self.interrupted(Delivery::Enqueued, Some(e)));
                    }
                }
                Err(e) => return Err(self.interrupted(Delivery::Enqueued, Some(e))),
            }
        }
        Err(self.interrupted(Delivery::Enqueued, None))
    }

    pub fn stat(&mut self, object: u64) -> Result<Stat, Error> {
        let h = self.header(wire::STAT, object)?;
        let (reply, body) = self.exchange(&h, &[])?;
        let data = wire::data(&body, wire::STAT_LENGTH as usize);
        let mut name = [0; NAME];
        name.copy_from_slice(&data[..NAME]);
        let mut history = [client::HistoryInfo::default(); volume_core::HISTORY];
        let history_count = usize::from(data[71]);
        for (index, version) in history.iter_mut().enumerate().take(history_count) {
            let at = 80 + index * wire::STAT_HISTORY;
            *version = client::HistoryInfo {
                revision: volume_core::get64(data, at),
                length: volume_core::get32(data, at + 8) as usize,
                content_sum: volume_core::get64(data, at + 12),
            };
        }
        Ok(Stat {
            revision: reply.revision,
            version_revision: volume_core::get64(data, 72),
            length: reply.aux as usize,
            name_length: reply.name_length as usize,
            name,
            content_sum: volume_core::get64(data, 32),
            request_id: volume_core::get64(data, 40),
            created: volume_core::get64(data, 48),
            modified: volume_core::get64(data, 56),
            tags: volume_core::get32(data, 64),
            kind: u16::from_le_bytes([data[68], data[69]]),
            collection: data[70],
            history_count,
            history,
        })
    }

    /// Paged catalog, returned only when every page has the same domain clock.
    pub fn list(&mut self, out: &mut [ListEntry; PER_DOMAIN]) -> Result<usize, Error> {
        self.list_filtered(ListFilter::All, out)
    }

    pub fn list_filtered(
        &mut self,
        filter: ListFilter,
        out: &mut [ListEntry; PER_DOMAIN],
    ) -> Result<usize, Error> {
        let filter = filter.wire()?;
        let mut staged = [ListEntry::EMPTY; PER_DOMAIN];
        let (mut cursor, mut count, mut generation) = (0, 0, None);
        while cursor < PER_DOMAIN as u64 {
            let mut h = self.header(wire::LIST, (self.domain * PER_DOMAIN) as u64)?;
            h.aux = cursor;
            h.count = filter;
            let (reply, body) = self.exchange(&h, &[])?;
            if generation.is_some_and(|g| g != reply.revision) {
                return Err(Error::Conflict);
            }
            generation = Some(reply.revision);
            for i in 0..reply.count as usize {
                if count == PER_DOMAIN {
                    return Err(self.protocol());
                }
                let at = 64 + i * wire::LIST_ENTRY;
                staged[count] = wire::decode_list_entry(&body[at..at + wire::LIST_ENTRY]);
                count += 1;
            }
            cursor = reply.aux;
        }
        *out = staged;
        Ok(count)
    }

    pub fn collection_stat(&mut self, index: usize) -> Result<CollectionInfo, Error> {
        if !(1..=volume_core::COLLECTIONS).contains(&index) {
            return Err(Error::InvalidRequest);
        }
        let mut h = self.header(wire::COLLECTION_STAT, (self.domain * PER_DOMAIN) as u64)?;
        h.aux = index as u64;
        let (reply, body) = self.exchange(&h, &[])?;
        let length = reply.length as usize;
        let mut name = [0; NAME];
        name[..length].copy_from_slice(wire::data(&body, length));
        Ok(CollectionInfo {
            revision: reply.revision,
            request_id: reply.aux,
            present: reply.count == 1,
            name_length: length,
            name,
        })
    }

    fn read(&mut self, object: u64, revision: u64, out: &mut [u8]) -> Result<(), Error> {
        self.read_version(object, revision, 0, out)
    }

    fn read_version(
        &mut self,
        object: u64,
        revision: u64,
        selector: usize,
        out: &mut [u8],
    ) -> Result<(), Error> {
        let mut offset = 0;
        while offset < out.len() {
            let mut h = self.header(wire::READ, object)?;
            h.revision = revision;
            h.aux = offset as u64;
            h.count = (out.len() - offset).min(wire::DATA) as u32;
            h.name_length = selector as u32;
            let (reply, body) = self.exchange(&h, &[])?;
            let amount = reply.length as usize;
            if amount == 0 || amount > out.len() - offset {
                return Err(self.protocol());
            }
            out[offset..offset + amount].copy_from_slice(wire::data(&body, amount));
            offset += amount;
        }
        Ok(())
    }
}

/// Three caller-owned, equally sized buffers. `staging` prevents a failed
/// multi-part read from destroying the previously loaded document.
pub struct Buffers<'a> {
    pub draft: &'a mut [u8],
    pub original: &'a mut [u8],
    pub staging: &'a mut [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draft<'a> {
    pub object: u64,
    pub loaded_revision: u64,
    pub name: &'a [u8],
    pub contents: &'a [u8],
}

/// A text preview verified against the entire retained version. `copied`
/// always ends on a UTF-8 character boundary; truncation is never implicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryPrefix {
    pub copied: usize,
    pub total: usize,
    pub truncated: bool,
}

#[derive(Clone, Copy)]
struct Loaded {
    object: u64,
    revision: u64,
    name: [u8; NAME],
    name_length: usize,
    original_name: [u8; NAME],
    original_name_length: usize,
    length: usize,
    original_length: usize,
    created: bool,
}

#[derive(Clone, Copy)]
struct Upload {
    id: u64,
    token: u32,
    offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveProgress {
    Writing { completed: usize, total: usize },
    Saved { revision: u64 },
}

/// Local editor state over the real volume service, independent of UI.
/// ReadWrite is an app policy; only kernel/service capabilities grant access.
pub struct DocumentSession<'a, T> {
    session: VolumeSession<T>,
    mode: EditMode,
    buffers: Buffers<'a>,
    loaded: Option<Loaded>,
    upload: Option<Upload>,
    conflict: bool,
    uncertain: bool,
}

impl<'a, T: Transport> DocumentSession<'a, T> {
    pub fn new(
        mut session: VolumeSession<T>,
        mode: EditMode,
        buffers: Buffers<'a>,
    ) -> Result<Self, Error> {
        let size = buffers.draft.len();
        if size == 0
            || size > MAX_LENGTH
            || buffers.original.len() != size
            || buffers.staging.len() != size
        {
            let _ = session.close();
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            session,
            mode,
            buffers,
            loaded: None,
            upload: None,
            conflict: false,
            uncertain: false,
        })
    }

    pub fn draft(&self) -> Option<Draft<'_>> {
        let loaded = self.loaded.as_ref()?;
        Some(Draft {
            object: loaded.object,
            loaded_revision: loaded.revision,
            name: &loaded.name[..loaded.name_length],
            contents: &self.buffers.draft[..loaded.length],
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.loaded.is_some_and(|l| {
            l.created
                || l.length != l.original_length
                || self.buffers.draft[..l.length] != self.buffers.original[..l.original_length]
                || l.name != l.original_name
                || l.name_length != l.original_name_length
        })
    }
    pub fn is_closed(&self) -> bool {
        self.session.is_closed()
    }
    pub fn is_conflicted(&self) -> bool {
        self.conflict
    }
    pub fn is_uncertain(&self) -> bool {
        self.uncertain
    }
    pub fn is_saving(&self) -> bool {
        self.upload.is_some()
    }
    pub fn volume(&self) -> &VolumeSession<T> {
        &self.session
    }

    fn available(&self) -> Result<(), Error> {
        if self.uncertain {
            Err(Error::Uncertain)
        } else if self.session.is_closed() {
            Err(Error::Closed)
        } else {
            Ok(())
        }
    }

    fn idle(&self) -> Result<(), Error> {
        if self.upload.is_some() {
            Err(Error::Busy)
        } else {
            Ok(())
        }
    }

    fn failed(&mut self, error: Error, committing: bool) -> Error {
        if error == Error::Service(Status::Stale) {
            self.conflict = true;
        }
        self.uncertain |= committing
            && matches!(
                error,
                Error::Service(
                    Status::Unknown | Status::Timeout | Status::NoDevice | Status::Corrupt
                ) | Error::Protocol { .. }
                    | Error::Interrupted {
                        delivery: Delivery::Enqueued | Delivery::Unknown,
                        ..
                    }
            );
        self.upload = None;
        if self.conflict {
            Error::Conflict
        } else {
            error
        }
    }

    pub fn open(&mut self, object: u64) -> Result<(), Error> {
        self.load(object, None)
    }

    fn load(&mut self, object: u64, expected: Option<u64>) -> Result<(), Error> {
        self.available()?;
        self.idle()?;
        if self.is_dirty() {
            return Err(Error::UnsavedChanges);
        }
        let stat = self.session.stat(object)?;
        if expected.is_some_and(|revision| stat.revision != revision) {
            return Err(Error::Conflict);
        }
        if stat.length > self.buffers.draft.len() {
            return Err(Error::ContentTooLarge);
        }
        self.session.read(
            object,
            stat.revision,
            &mut self.buffers.staging[..stat.length],
        )?;
        // Content checksums are accidental-corruption checks, not authenticity.
        if volume_core::checksum_update(
            volume_core::FNV_OFFSET,
            &self.buffers.staging[..stat.length],
        ) != stat.content_sum
        {
            return Err(self.session.protocol());
        }
        self.buffers.draft.fill(0);
        self.buffers.original.fill(0);
        self.buffers.draft[..stat.length].copy_from_slice(&self.buffers.staging[..stat.length]);
        self.buffers.original[..stat.length].copy_from_slice(&self.buffers.staging[..stat.length]);
        self.loaded = Some(Loaded {
            object,
            revision: stat.revision,
            name: stat.name,
            name_length: stat.name_length,
            original_name: stat.name,
            original_name_length: stat.name_length,
            length: stat.length,
            original_length: stat.length,
            created: false,
        });
        self.conflict = false;
        Ok(())
    }

    /// Start a local draft only after STAT proves absence and supplies the
    /// tombstone CAS revision. No disk mutation occurs until save.
    pub fn create(&mut self, object: u64, name: &[u8]) -> Result<(), Error> {
        self.available()?;
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if self.is_dirty() {
            return Err(Error::UnsavedChanges);
        }
        if name.len() > NAME || !volume_core::valid_name(name) {
            return Err(Error::InvalidRequest);
        }
        let revision = match self.session.stat(object) {
            Err(Error::NotFound { revision, .. }) => revision,
            Err(e) => return Err(e),
            Ok(_) => return Err(Error::Conflict),
        };
        let mut title = [0; NAME];
        title[..name.len()].copy_from_slice(name);
        self.buffers.draft.fill(0);
        self.buffers.original.fill(0);
        self.loaded = Some(Loaded {
            object,
            revision,
            name: title,
            name_length: name.len(),
            original_name: title,
            original_name_length: name.len(),
            length: 0,
            original_length: 0,
            created: true,
        });
        self.conflict = false;
        Ok(())
    }

    pub fn replace_contents(&mut self, contents: &[u8]) -> Result<(), Error> {
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if contents.len() > self.buffers.draft.len() {
            return Err(Error::ContentTooLarge);
        }
        let loaded = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        self.buffers.draft[..contents.len()].copy_from_slice(contents);
        self.buffers.draft[contents.len()..].fill(0);
        loaded.length = contents.len();
        Ok(())
    }

    pub fn replace_range(&mut self, range: Range<usize>, replacement: &[u8]) -> Result<(), Error> {
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        let loaded = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        if range.start > range.end || range.end > loaded.length {
            return Err(Error::InvalidRange);
        }
        let length = (loaded.length - (range.end - range.start))
            .checked_add(replacement.len())
            .filter(|n| *n <= self.buffers.draft.len())
            .ok_or(Error::ContentTooLarge)?;
        self.buffers
            .draft
            .copy_within(range.end..loaded.length, range.start + replacement.len());
        self.buffers.draft[range.start..range.start + replacement.len()]
            .copy_from_slice(replacement);
        self.buffers.draft[length..].fill(0);
        loaded.length = length;
        Ok(())
    }

    pub fn rename_draft(&mut self, name: &[u8]) -> Result<(), Error> {
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if name.len() > NAME || !volume_core::valid_name(name) {
            return Err(Error::InvalidRequest);
        }
        let loaded = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        loaded.name.fill(0);
        loaded.name[..name.len()].copy_from_slice(name);
        loaded.name_length = name.len();
        Ok(())
    }

    pub fn discard_changes(&mut self) -> Result<(), Error> {
        self.idle()?;
        let loaded = self.loaded.as_mut().ok_or(Error::NotLoaded)?;
        if loaded.created {
            self.loaded = None;
            return Ok(());
        }
        self.buffers.draft.copy_from_slice(self.buffers.original);
        loaded.length = loaded.original_length;
        loaded.name = loaded.original_name;
        loaded.name_length = loaded.original_name_length;
        Ok(())
    }

    pub fn metadata(&mut self) -> Result<Stat, Error> {
        self.available()?;
        self.idle()?;
        self.session
            .stat(self.loaded.ok_or(Error::NotLoaded)?.object)
    }

    pub fn list(&mut self, out: &mut [ListEntry; PER_DOMAIN]) -> Result<usize, Error> {
        self.list_filtered(ListFilter::All, out)
    }

    pub fn list_filtered(
        &mut self,
        filter: ListFilter,
        out: &mut [ListEntry; PER_DOMAIN],
    ) -> Result<usize, Error> {
        self.available()?;
        self.idle()?;
        self.session.list_filtered(filter, out)
    }

    pub fn collection_stat(&mut self, index: usize) -> Result<CollectionInfo, Error> {
        self.available()?;
        self.idle()?;
        self.session.collection_stat(index)
    }

    fn clean_write(&self) -> Result<(), Error> {
        self.available()?;
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if self.conflict {
            return Err(Error::Conflict);
        }
        if self.is_dirty() {
            return Err(Error::UnsavedChanges);
        }
        Ok(())
    }

    fn expected_document(&self, expected: u64) -> Result<Loaded, Error> {
        let loaded = self.loaded.ok_or(Error::NotLoaded)?;
        if expected == u64::MAX {
            return Err(Error::InvalidRequest);
        }
        if expected != loaded.revision {
            return Err(Error::Conflict);
        }
        Ok(loaded)
    }

    /// Create/rename a collection, or delete it with an empty name. The
    /// existing service rejects deleting a non-empty collection. Selection
    /// and collection reads never call this explicit mutation.
    pub fn update_collection(
        &mut self,
        index: usize,
        expected: u64,
        name: &[u8],
    ) -> Result<u64, Error> {
        self.clean_write()?;
        if !(1..=volume_core::COLLECTIONS).contains(&index)
            || expected == u64::MAX
            || name.len() > NAME
            || !name.is_empty() && !volume_core::valid_name(name)
        {
            return Err(Error::InvalidRequest);
        }
        let mut h = self
            .session
            .header(wire::COLLECTION, (self.session.domain * PER_DOMAIN) as u64)?;
        h.revision = expected;
        h.aux = index as u64;
        h.length = name.len() as u32;
        h.name_length = h.length;
        self.session
            .exchange(&h, name)
            .map(|(reply, _)| reply.revision)
            .map_err(|error| self.failed(error, true))
    }

    /// Update the loaded object's metadata with its exact CAS revision.
    /// Content/name and the editor's clean original bytes stay unchanged.
    pub fn set_metadata(
        &mut self,
        expected_revision: u64,
        tags: u32,
        kind: u16,
        collection: u8,
    ) -> Result<u64, Error> {
        self.clean_write()?;
        let loaded = self.expected_document(expected_revision)?;
        if collection as usize > volume_core::COLLECTIONS {
            return Err(Error::InvalidRequest);
        }
        let mut h = self.session.header(wire::META, loaded.object)?;
        h.revision = expected_revision;
        h.aux = wire::meta(tags, kind, collection);
        match self.session.exchange(&h, &[]) {
            Ok((reply, _)) => {
                self.loaded.as_mut().unwrap().revision = reply.revision;
                Ok(reply.revision)
            }
            Err(error) => Err(self.failed(error, true)),
        }
    }

    fn history_stat(&mut self, expected: u64, selector: usize) -> Result<Stat, Error> {
        let loaded = self.expected_document(expected)?;
        if !(1..=volume_core::HISTORY).contains(&selector) {
            return Err(Error::InvalidRequest);
        }
        let stat = self.session.stat(loaded.object)?;
        if stat.revision != expected {
            return Err(self.failed(Error::Service(Status::Stale), false));
        }
        if selector > stat.history_count {
            return Err(Error::NotFound {
                object: loaded.object,
                revision: stat.revision,
            });
        }
        Ok(stat)
    }

    /// Preview a retained version without modifying the draft or disk. The
    /// caller's output changes only after the entire bound read is verified.
    /// Dirty drafts may preview history, but an active upload may not.
    pub fn read_history(
        &mut self,
        expected_revision: u64,
        selector: usize,
        out: &mut [u8],
    ) -> Result<usize, Error> {
        let info = self.load_history(expected_revision, selector, out.len())?;
        out[..info.length].copy_from_slice(&self.buffers.staging[..info.length]);
        Ok(info.length)
    }

    /// Verify the entire historical version using the existing staging
    /// buffer, then copy a UTF-8-safe prefix. No output byte changes on any
    /// failure, including malformed UTF-8 beyond the displayed prefix. This
    /// does not write, select a new current version or discard a dirty draft.
    pub fn read_history_prefix(
        &mut self,
        expected_revision: u64,
        selector: usize,
        out: &mut [u8],
    ) -> Result<HistoryPrefix, Error> {
        let info = self.load_history(expected_revision, selector, usize::MAX)?;
        let text = core::str::from_utf8(&self.buffers.staging[..info.length])
            .map_err(|_| Error::InvalidUtf8)?;
        let mut copied = out.len().min(info.length);
        while !text.is_char_boundary(copied) {
            copied -= 1;
        }
        out[..copied].copy_from_slice(&self.buffers.staging[..copied]);
        Ok(HistoryPrefix {
            copied,
            total: info.length,
            truncated: copied < info.length,
        })
    }

    fn load_history(
        &mut self,
        expected_revision: u64,
        selector: usize,
        output_limit: usize,
    ) -> Result<HistoryInfo, Error> {
        self.available()?;
        self.idle()?;
        let stat = self.history_stat(expected_revision, selector)?;
        let info = stat.history[selector - 1];
        if info.length > self.buffers.staging.len() || info.length > output_limit {
            return Err(Error::ContentTooLarge);
        }
        let object = self.loaded.unwrap().object;
        if let Err(error) = self.session.read_version(
            object,
            expected_revision,
            selector,
            &mut self.buffers.staging[..info.length],
        ) {
            return Err(self.failed(error, false));
        }
        if volume_core::checksum_update(
            volume_core::FNV_OFFSET,
            &self.buffers.staging[..info.length],
        ) != info.content_sum
        {
            return Err(self.session.protocol());
        }
        Ok(info)
    }

    /// Restore history as a new durable revision, then load that exact
    /// revision. A lost mutation reply is sticky uncertainty. A known commit
    /// followed by failed reading is reported separately and blocks saving
    /// the retained old draft until an explicit successful open.
    pub fn restore_history(
        &mut self,
        expected_revision: u64,
        selector: usize,
    ) -> Result<u64, Error> {
        self.clean_write()?;
        let stat = self.history_stat(expected_revision, selector)?;
        if stat.history[selector - 1].length > self.buffers.draft.len() {
            return Err(Error::ContentTooLarge);
        }
        let object = self.loaded.unwrap().object;
        let mut h = self.session.header(wire::RESTORE, object)?;
        h.revision = expected_revision;
        h.aux = selector as u64;
        let revision = match self.session.exchange(&h, &[]) {
            Ok((reply, _)) => reply.revision,
            Err(error) => return Err(self.failed(error, true)),
        };
        if self.load(object, Some(revision)).is_err() {
            self.conflict = true;
            return Err(Error::CommittedRefreshFailed { revision });
        }
        Ok(revision)
    }

    pub fn begin_save(&mut self) -> Result<(), Error> {
        self.available()?;
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if self.conflict {
            return Err(Error::Conflict);
        }
        let loaded = self.loaded.ok_or(Error::NotLoaded)?;
        if loaded.revision == u64::MAX {
            return Err(Error::InvalidRequest);
        }
        let mut h = self.session.header(wire::BEGIN, loaded.object)?;
        h.revision = loaded.revision;
        h.length = loaded.name_length as u32;
        h.name_length = h.length;
        h.aux = loaded.length as u64;
        match self
            .session
            .exchange(&h, &loaded.name[..loaded.name_length])
        {
            Ok((_, body)) => {
                self.upload = Some(Upload {
                    id: h.request_id,
                    token: upload::token(&body),
                    offset: 0,
                });
                Ok(())
            }
            Err(e) => Err(self.failed(e, false)),
        }
    }

    /// At most one APPEND or COMMIT per call. The caller may service input and
    /// cancel between calls. Draft edits are refused while an upload is live.
    pub fn save_next(&mut self) -> Result<SaveProgress, Error> {
        self.available()?;
        let loaded = self.loaded.ok_or(Error::NotLoaded)?;
        let upload = self.upload.ok_or(Error::InvalidRequest)?;
        let committing = upload.offset == loaded.length;
        let mut h = wire::Header::new(
            if committing {
                wire::COMMIT
            } else {
                wire::APPEND
            },
            upload.id,
            self.session.epoch,
            loaded.object,
        );
        h.revision = loaded.revision;
        h.aux = upload.offset as u64;
        let end = (upload.offset + wire::DATA).min(loaded.length);
        let data = if committing {
            &[][..]
        } else {
            &self.buffers.draft[upload.offset..end]
        };
        h.length = data.len() as u32;
        match self.session.exchange_upload(&h, data, upload.token) {
            Ok((reply, _)) if committing => {
                let loaded = self.loaded.as_mut().unwrap();
                loaded.revision = reply.revision;
                loaded.original_length = loaded.length;
                loaded.original_name = loaded.name;
                loaded.original_name_length = loaded.name_length;
                loaded.created = false;
                self.buffers.original.copy_from_slice(self.buffers.draft);
                self.upload = None;
                Ok(SaveProgress::Saved {
                    revision: reply.revision,
                })
            }
            Ok(_) => {
                self.upload.as_mut().unwrap().offset = end;
                Ok(SaveProgress::Writing {
                    completed: end,
                    total: loaded.length,
                })
            }
            Err(e) => Err(self.failed(e, committing)),
        }
    }

    pub fn cancel_save(&mut self) -> Result<(), Error> {
        self.available()?;
        let loaded = self.loaded.ok_or(Error::NotLoaded)?;
        let upload = self.upload.ok_or(Error::InvalidRequest)?;
        let mut h = wire::Header::new(wire::ABORT, upload.id, self.session.epoch, loaded.object);
        h.revision = loaded.revision;
        match self.session.exchange_upload(&h, &[], upload.token) {
            Ok(_) => {
                self.upload = None;
                Ok(())
            }
            Err(e) => Err(self.failed(e, false)),
        }
    }

    pub fn save(&mut self) -> Result<SaveOutcome, Error> {
        self.available()?;
        self.idle()?;
        if self.mode == EditMode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if self.conflict {
            return Err(Error::Conflict);
        }
        let loaded = self.loaded.ok_or(Error::NotLoaded)?;
        if !self.is_dirty() {
            return Ok(SaveOutcome::Unchanged {
                revision: loaded.revision,
            });
        }
        self.begin_save()?;
        loop {
            if let SaveProgress::Saved { revision } = self.save_next()? {
                return Ok(SaveOutcome::Saved { revision });
            }
        }
    }

    /// Explicit endpoint cleanup, retaining draft bytes. Closing an active
    /// upload does not imply rollback; no COMMIT is sent by close/drop.
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        self.upload = None;
        self.session.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::{Handle, Received};

    struct NoSend;
    impl Transport for NoSend {
        fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
            panic!("exhausted request IDs must not send")
        }
        fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
            panic!("exhausted request IDs must not receive")
        }
        fn yield_now(&mut self) -> Result<(), RuntimeError> {
            Ok(())
        }
        fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
            Ok(())
        }
    }

    #[test]
    fn request_id_exhaustion_never_wraps_or_publishes() {
        let mut session = VolumeSession::new(
            NoSend,
            7,
            2,
            0,
            SendEndpoint::from_handle(Handle::from_raw((1 << 32) | 1).unwrap()),
            ReceiveEndpoint::from_handle(Handle::from_raw((1 << 32) | 2).unwrap()),
            1,
        )
        .unwrap();
        session.next_id = u64::MAX;
        assert_eq!(session.stat(0), Err(Error::RequestIdsExhausted));
        assert_eq!(session.stat(0), Err(Error::RequestIdsExhausted));
        assert_eq!(session.next_id, u64::MAX);
    }
}
