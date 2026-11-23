//! Request handling: the total check order of 2.2, the operations of 2.3, the
//! entry state machine of 2.5 and the write path and flush order of 2.7.
//!
//! One instance serves the control lane and the three operation lanes from
//! one loop, one request at a time; the native shell (R5) receives a message,
//! passes its kernel metadata as [`Meta`], closes every attached handle when
//! [`Outcome::close_caps`] is set and sends [`Outcome::reply`] if present.
use crate::{
    catalog::{self, Catalog, Mount, Record, Selection, AREA_SECTORS, COPY, COPY_SECTORS},
    device::{self, Broker},
    put64, sectors_for,
    wire::{self, Header, CONTROL, DATA, LANES, SIZE},
    Status, ID, LABEL, MAX_ARCHIVE, MAX_ENTRIES, SECTOR,
};

/// Kernel-provided metadata of one received message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meta {
    /// Endpoint the message arrived on: 0 control, 1..=3 operation lanes.
    pub lane: usize,
    /// Kernel-stamped sender process ID (message offset 0).
    pub sender_pid: u64,
    /// Number of capabilities `recv` attached to the message.
    pub cap_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Handler status, also where 2.2 sends no reply.
    pub status: Status,
    pub reply: Option<[u8; SIZE]>,
    /// Check 1: the shell closes every handle `recv` attached, never keeps one.
    pub close_caps: bool,
}

/// Byte stream over a stored extent, shaped like `volume_core::archive::Source`:
/// returns the number of bytes copied, 0 at the end of the extent.
pub trait Source {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub generation: u64,
    pub objects: u64,
}

/// Archive check run on a stored extent (SEAL step 3 and VERIFY). The native
/// service plugs in `volume_core::archive::verify`; a source error must be
/// returned unchanged.
pub trait Verifier {
    fn verify(&mut self, source: &mut dyn Source) -> Result<Summary, Status>;
}

/// Entropy for the destination ID drawn by FORMAT (2.6).
pub trait Entropy {
    fn fill(&mut self, id: &mut [u8; ID]) -> Result<(), Status>;
}

pub struct Env<'a, B, V, E> {
    pub broker: &'a mut B,
    pub verifier: &'a mut V,
    pub entropy: &'a mut E,
}

/// Mount state of the instance (2.2 check 7). "Remount pending" exists only
/// inside one call of [`Service::handle`], which receives no other request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Mounted,
    /// Failed mount or remount, a failed FORMAT, or a remount that contradicts
    /// the committed catalog (O16); terminal for the instance.
    Failed {
        no_device: bool,
    },
    /// Terminal for the instance; never formatted or repaired.
    Corrupt,
    Blank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Export,
    Verify,
    Restore,
}

/// Progress of an export binding. Anything other than `Fresh` and `Writing`
/// limits the binding to INFO and LIST (2.2 check 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Export {
    Fresh,
    Writing(u64),
    Sealed(u64),
    /// Aborted, discarded, or resolved as not sealed after a remount.
    NotSealed,
    /// SEAL answered `Unknown` and no completed remount FLUSH resolved it;
    /// also after a remount that contradicted the committed catalog (O16).
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub kind: Kind,
    /// Bound sealed entry (verify, restore); 0 for export.
    pub entry: u64,
    pub generation: u64,
    pub export: Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lane {
    binding: Option<Binding>,
    high_water: u64,
    /// Highest generation accepted by BIND for the life of the instance.
    max_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Writing {
    lane: usize,
    id: u64,
    first: u64,
    appended: u64,
    /// Buffered partial last sector, zero beyond `appended % SECTOR`.
    tail: [u8; SECTOR],
    label_length: u8,
    label: [u8; LABEL],
}

struct Reply {
    header: Header,
    data: [u8; DATA],
}

pub struct Service {
    scope: u64,
    portal: u64,
    state: State,
    selection: Selection,
    /// Read-only latch of 2.7, set by any device error on a destination
    /// write or flush; it ends only with the instance (once-per-boot rule).
    safe_path: bool,
    lanes: [Lane; LANES],
    writing: Option<Writing>,
    /// Requests refused by checks 1-6 (`DEST:DENIALS=`, a service report).
    pub denials: u64,
    /// Test hook for N29: alters the encoded catalog copy before the
    /// pre-write check of 2.7 step 4. Exists only with the test-only
    /// `test-hooks` feature; a product build has no such field.
    #[cfg(feature = "test-hooks")]
    #[doc(hidden)]
    pub tamper: Option<fn(&mut [u8; COPY])>,
}

impl Service {
    /// Boot mount: the service receives no request before it has ended in
    /// one of the states of 2.2 check 7. `portal_pid` comes from the start
    /// context written by the manifest (4.3).
    pub fn boot<B: Broker>(scope_sectors: u64, portal_pid: u64, broker: &mut B) -> Self {
        let mut service = Self {
            scope: scope_sectors,
            portal: portal_pid,
            state: State::Blank,
            selection: Selection::committed(Catalog::formatted([0; ID], scope_sectors)),
            safe_path: false,
            lanes: [Lane {
                binding: None,
                high_water: 0,
                max_generation: 0,
            }; LANES],
            writing: None,
            denials: 0,
            #[cfg(feature = "test-hooks")]
            tamper: None,
        };
        service.mount(broker);
        service
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn safe_path(&self) -> bool {
        self.safe_path
    }
    pub fn binding(&self, lane: usize) -> Option<Binding> {
        self.lanes[lane].binding
    }
    /// The mounted catalog (meaningful only in [`State::Mounted`]).
    pub fn catalog(&self) -> &Catalog {
        &self.selection.catalog
    }

    /// Mount rule of 2.6: FLUSH first, then read sectors 0-15; a failed
    /// FLUSH or read gives the failed-mount state and writes nothing.
    fn mount<B: Broker>(&mut self, broker: &mut B) {
        let mut broker = device::Watch::new(broker);
        if device::flush(&mut broker).is_err() {
            self.state = State::Failed {
                no_device: broker.no_device,
            };
            return;
        }
        let mut area = [[0; SECTOR]; AREA_SECTORS];
        for (sector, buffer) in area.iter_mut().enumerate() {
            if device::read(&mut broker, sector as u64, buffer).is_err() {
                self.state = State::Failed {
                    no_device: broker.no_device,
                };
                return;
            }
        }
        self.state = match catalog::mount_rule(&area, self.scope) {
            Mount::Mounted(selection) => {
                self.selection = selection;
                State::Mounted
            }
            Mount::Blank => State::Blank,
            Mount::Corrupt => State::Corrupt,
        };
    }

    pub fn handle<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        meta: Meta,
        body: &[u8; SIZE],
        env: &mut Env<'_, B, V, E>,
    ) -> Outcome {
        let close_caps = meta.cap_count != 0;
        if meta.lane >= LANES {
            return Outcome {
                status: Status::Invalid,
                reply: None,
                close_caps,
            };
        }
        let echo = wire::header(body);
        let mut reply = Reply {
            header: Header::new(echo.operation, echo.request_id, echo.epoch, echo.entry),
            data: [0; DATA],
        };
        let status = match self.checks(meta, body) {
            Err(status) => {
                self.denials += 1;
                status
            }
            Ok(h) => match self.gate(meta.lane, &h, body, env, &mut reply) {
                Ok(()) => Status::Ok,
                Err(status) => {
                    // An error reply echoes identity only, never data, except
                    // the zero-record `Unknown` LIST and state-coded INFO.
                    if h.operation != wire::INFO {
                        let keep = reply.header;
                        reply.header =
                            Header::new(keep.operation, keep.request_id, keep.epoch, keep.entry);
                    }
                    status
                }
            },
        };
        reply.header.status = status as i32;
        let length = reply.header.length as usize;
        // 2.2 Replies: an operation-lane request from anyone but the portal
        // gets no reply at all, whatever check refused it.
        let suppressed = meta.lane != CONTROL && meta.sender_pid != self.portal;
        Outcome {
            status,
            reply: (!suppressed).then(|| wire::encode(&reply.header, &reply.data[..length])),
            close_caps,
        }
    }

    /// Checks 1-6 of 2.2, in order. A request that passes them advances the
    /// lane's high-water mark, whatever check 7 or the operation answers.
    fn checks(&mut self, meta: Meta, body: &[u8; SIZE]) -> Result<Header, Status> {
        if meta.cap_count != 0 {
            return Err(Status::Invalid);
        }
        let h = wire::validate(body)?;
        let lane = &self.lanes[meta.lane];
        if meta.lane == CONTROL {
            if h.epoch != 0 || h.request_id <= lane.high_water {
                return Err(Status::Stale);
            }
            if !matches!(
                h.operation,
                wire::INFO | wire::LIST | wire::ABORT | wire::BIND | wire::UNBIND | wire::FORMAT
            ) {
                return Err(Status::Access);
            }
        } else {
            let Some(binding) = lane.binding else {
                return Err(Status::Access);
            };
            if meta.sender_pid != self.portal {
                return Err(Status::Access);
            }
            if h.epoch != binding.generation || h.request_id <= lane.high_water {
                return Err(Status::Stale);
            }
            if !permitted(&binding, &h) {
                return Err(Status::Access);
            }
        }
        self.lanes[meta.lane].high_water = h.request_id;
        Ok(h)
    }

    /// Check 7: the mount state decides the reply; the operation runs only
    /// where the table of 2.2 says so.
    fn gate<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        lane: usize,
        h: &Header,
        body: &[u8; SIZE],
        env: &mut Env<'_, B, V, E>,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        match self.state {
            State::Mounted => {
                if self.safe_path
                    && matches!(
                        h.operation,
                        wire::CREATE | wire::APPEND | wire::SEAL | wire::FORMAT
                    )
                {
                    return Err(Status::Access);
                }
                self.run(lane, h, body, env, reply)
            }
            State::Failed { no_device } => match h.operation {
                wire::INFO => {
                    self.info(reply);
                    Err(if no_device {
                        Status::NoDevice
                    } else {
                        Status::Io
                    })
                }
                wire::LIST
                    if self.lanes[lane]
                        .binding
                        .is_some_and(|b| b.export == Export::Unknown) =>
                {
                    Err(Status::Unknown)
                }
                _ => Err(Status::Io),
            },
            State::Corrupt => {
                if h.operation == wire::INFO {
                    self.info(reply);
                }
                Err(Status::Corrupt)
            }
            State::Blank => match h.operation {
                // Unreachable since O16: the latch is set only in a mounted
                // instance and a remount never ends blank (2.7). Kept as a
                // defence: the latch forbids every write for the boot.
                wire::FORMAT if self.safe_path => Err(Status::Access),
                wire::FORMAT | wire::UNBIND => self.run(lane, h, body, env, reply),
                wire::INFO => {
                    self.info(reply);
                    Err(Status::NotFound)
                }
                _ => Err(Status::NotFound),
            },
        }
    }

    fn run<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        lane: usize,
        h: &Header,
        body: &[u8; SIZE],
        env: &mut Env<'_, B, V, E>,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        match h.operation {
            wire::INFO => {
                self.info(reply);
                Ok(())
            }
            wire::LIST => self.list(lane, h, reply),
            wire::CREATE => self.create(lane, wire::data(body), reply),
            wire::APPEND => self.append(env.broker, h, wire::data(body), reply),
            wire::SEAL => self.seal(lane, h, env, reply),
            wire::ABORT => self.abort(h),
            wire::VERIFY => self.verify(h, env, reply),
            wire::READ => self.read(env.broker, h, reply),
            wire::BIND => self.bind(h, wire::data(body)),
            wire::UNBIND => {
                self.unbind(h.count as usize);
                Ok(())
            }
            wire::FORMAT => self.format(env, reply),
            _ => Err(Status::NoSys),
        }
    }

    /// INFO data: destination ID, formatted sectors, then the highest
    /// accepted generation of lanes 1-3. Outside the mounted states the ID,
    /// formatted count, `aux` and `count` are zero.
    fn info(&self, reply: &mut Reply) {
        let mounted = self.state == State::Mounted;
        let catalog = &self.selection.catalog;
        reply.header.length = wire::INFO_LENGTH as u32;
        if mounted {
            reply.header.aux = catalog.formatted - catalog.next_free;
            reply.header.count = catalog.count as u32;
            reply.data[..ID].copy_from_slice(&catalog.id);
            put64(&mut reply.data, 16, catalog.formatted);
        }
        for (lane, state) in self.lanes.iter().enumerate().skip(1) {
            put64(&mut reply.data, 16 + lane * 8, state.max_generation);
        }
    }

    fn visible(&self, lane: usize, record: &Record) -> bool {
        let Some(binding) = self.lanes[lane].binding.filter(|_| lane != CONTROL) else {
            return lane == CONTROL;
        };
        match binding.kind {
            Kind::Export => binding.export == Export::Sealed(record.id),
            Kind::Verify | Kind::Restore => record.id == binding.entry,
        }
    }

    fn list(&self, lane: usize, h: &Header, reply: &mut Reply) -> Result<(), Status> {
        let entries = self.selection.catalog.entries();
        let mut returned = 0;
        let mut next = 0;
        for (index, record) in entries.iter().enumerate().skip(h.count as usize) {
            if !self.visible(lane, record) {
                continue;
            }
            if returned == wire::LIST_MAX {
                next = index as u32;
                break;
            }
            record.encode(&mut reply.data[returned * wire::LIST_RECORD..]);
            returned += 1;
        }
        reply.header.length = (returned * wire::LIST_RECORD) as u32;
        reply.header.count = next;
        Ok(())
    }

    fn create(&mut self, lane: usize, label: &[u8], reply: &mut Reply) -> Result<(), Status> {
        if !crate::valid_name(label) {
            return Err(Status::Invalid);
        }
        if self.writing.is_some() {
            return Err(Status::Busy);
        }
        let catalog = &self.selection.catalog;
        if catalog.count >= MAX_ENTRIES || catalog.next_free >= self.scope {
            return Err(Status::Full);
        }
        let mut stored = [0; LABEL];
        stored[..label.len()].copy_from_slice(label);
        let id = catalog.next_entry;
        self.writing = Some(Writing {
            lane,
            id,
            first: catalog.next_free,
            appended: 0,
            tail: [0; SECTOR],
            label_length: label.len() as u8,
            label: stored,
        });
        self.set_export(lane, Export::Writing(id));
        reply.header.entry = id;
        Ok(())
    }

    fn set_export(&mut self, lane: usize, export: Export) {
        if let Some(binding) = self.lanes[lane].binding.as_mut() {
            binding.export = export;
        }
    }

    /// Discard the writing entry; its binding is then limited to INFO and LIST.
    fn discard(&mut self) {
        if let Some(writing) = self.writing.take() {
            self.set_export(writing.lane, Export::NotSealed);
        }
    }

    /// Any device error on a destination write or flush: discard the writing
    /// entry and latch the read-only safe path for the rest of the boot.
    fn device_failure(&mut self, status: Status) -> Status {
        self.discard();
        if status != Status::Access {
            self.safe_path = true;
        }
        status
    }

    fn append<B: Broker>(
        &mut self,
        broker: &mut B,
        h: &Header,
        data: &[u8],
        reply: &mut Reply,
    ) -> Result<(), Status> {
        let (scope, committed) = (self.scope, self.selection.catalog.next_free);
        let Some(writing) = self.writing.as_mut() else {
            return Err(Status::Access);
        };
        if h.offset != writing.appended {
            return Err(Status::Stale);
        }
        let total = writing.appended + data.len() as u64;
        // Order of 2.3: offset (`Stale`), archive bound (`Range`), scope end
        // (`Full`); each refusal appends nothing and leaves the entry writing.
        if total > MAX_ARCHIVE {
            return Err(Status::Range);
        }
        if writing.first + sectors_for(total) > scope {
            return Err(Status::Full);
        }
        for byte in data {
            let within = (writing.appended % SECTOR as u64) as usize;
            writing.tail[within] = *byte;
            writing.appended += 1;
            if within + 1 == SECTOR {
                let sector = writing.first + writing.appended / SECTOR as u64 - 1;
                let tail = writing.tail;
                writing.tail = [0; SECTOR];
                if let Err(status) = device::write_entry(broker, committed, scope, sector, &tail) {
                    return Err(self.device_failure(status));
                }
            }
        }
        reply.header.offset = total;
        Ok(())
    }

    fn seal<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        lane: usize,
        h: &Header,
        env: &mut Env<'_, B, V, E>,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        let Some(writing) = self.writing else {
            return Err(Status::Access);
        };
        if h.offset != writing.appended {
            return Err(Status::Invalid);
        }
        let committed = self.selection.catalog;
        // Step 3: padded last sector, barrier 1, read back and verify.
        if writing.appended % SECTOR as u64 != 0 {
            let sector = writing.first + writing.appended / SECTOR as u64;
            let written = device::write_entry(
                env.broker,
                committed.next_free,
                self.scope,
                sector,
                &writing.tail,
            );
            if let Err(status) = written {
                return Err(self.device_failure(status));
            }
        }
        if let Err(status) = device::flush(env.broker) {
            return Err(self.device_failure(status));
        }
        let trailer = match check_extent(env.broker, env.verifier, writing.first, writing.appended)
        {
            Ok((_, trailer)) => trailer,
            Err(status) => {
                // A failed readback starts no safe path: no write is
                // outstanding after barrier 1 has completed.
                self.discard();
                return Err(status);
            }
        };
        // Step 4: encode, then validate before any write. The sequence and
        // next entry ID come from disk bytes that the validator accepts up to
        // u64::MAX, so their increments are checked (no panic, no wrap).
        let (Some(sequence), Some(next_entry)) = (
            committed.sequence.checked_add(1),
            committed.next_entry.checked_add(1),
        ) else {
            self.discard();
            return Err(Status::Corrupt);
        };
        if committed.count >= MAX_ENTRIES {
            self.discard();
            return Err(Status::Corrupt);
        }
        let mut next = committed;
        next.records[next.count] = Record {
            id: writing.id,
            first: writing.first,
            length: writing.appended,
            trailer,
            sealed_sequence: sequence,
            label_length: writing.label_length,
            label: writing.label,
        };
        next.count += 1;
        next.sequence = sequence;
        next.next_free = writing.first + sectors_for(writing.appended);
        next.next_entry = next_entry;
        #[cfg_attr(not(feature = "test-hooks"), allow(unused_mut))]
        let mut bytes = next.encode();
        #[cfg(feature = "test-hooks")]
        if let Some(tamper) = self.tamper {
            tamper(&mut bytes);
        }
        let Some(checked) = catalog::decode(&bytes, self.scope)
            .filter(|copy| catalog::extends_by_one(&committed, copy))
        else {
            self.discard();
            return Err(Status::Corrupt);
        };
        // Steps 4-5: target slot, barrier 2, other slot, barrier 3.
        let target = self.selection.target();
        let written = write_copy(env.broker, target, &bytes)
            .and_then(|()| device::flush(env.broker))
            .and_then(|()| write_copy(env.broker, 1 - target, &bytes))
            .and_then(|()| device::flush(env.broker));
        if written.is_err() {
            return Err(self.catalog_failure(env.broker, lane, writing.id, &committed));
        }
        self.selection = Selection::committed(checked);
        self.writing = None;
        self.set_export(lane, Export::Sealed(writing.id));
        reply.header.entry = writing.id;
        reply.header.offset = writing.appended;
        reply.header.aux = trailer;
        Ok(())
    }

    /// A device result other than success in SEAL steps 4-5: reply `Unknown`,
    /// drop in-memory catalog and writing state, latch the safe path and
    /// remount before the next request. Only a remount whose FLUSH completed
    /// and that mounted `committed` or `committed` plus exactly the
    /// provisional record resolves the export binding, once (O16, 2.7).
    fn catalog_failure<B: Broker>(
        &mut self,
        broker: &mut B,
        lane: usize,
        id: u64,
        committed: &Catalog,
    ) -> Status {
        self.writing = None;
        self.safe_path = true;
        self.set_export(lane, Export::Unknown);
        self.mount(broker);
        let remounted = self.selection.catalog;
        let resolved = match self.state {
            State::Mounted if remounted == *committed => Some(Export::NotSealed),
            State::Mounted
                if catalog::extends_by_one(committed, &remounted)
                    && remounted.records[committed.count].id == id =>
            {
                Some(Export::Sealed(id))
            }
            // O16: a blank candidate or another catalog contradicts the
            // catalog mounted earlier in this boot: failed mount, binding
            // unknown. `Corrupt` and a failed FLUSH or read stay as they are.
            State::Mounted | State::Blank => {
                self.state = State::Failed { no_device: false };
                None
            }
            State::Corrupt | State::Failed { .. } => None,
        };
        if let Some(export) = resolved {
            self.set_export(lane, export);
        }
        Status::Unknown
    }

    fn abort(&mut self, h: &Header) -> Result<(), Status> {
        if self.writing.map(|writing| writing.id) != Some(h.entry) {
            return Err(Status::NotFound);
        }
        self.discard();
        Ok(())
    }

    fn verify<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        h: &Header,
        env: &mut Env<'_, B, V, E>,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        let record = *self
            .selection
            .catalog
            .find(h.entry)
            .ok_or(Status::NotFound)?;
        let (summary, _) = check_extent(env.broker, env.verifier, record.first, record.length)?;
        reply.header.aux = summary.generation;
        reply.header.count = summary.objects as u32;
        Ok(())
    }

    fn read<B: Broker>(
        &mut self,
        broker: &mut B,
        h: &Header,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        let record = *self
            .selection
            .catalog
            .find(h.entry)
            .ok_or(Status::NotFound)?;
        let end = h.offset.checked_add(h.aux).ok_or(Status::Range)?;
        if end > record.length {
            return Err(Status::Range);
        }
        let mut position = h.offset;
        let mut buffer = [0; SECTOR];
        while position < end {
            let index = position / SECTOR as u64;
            let within = (position % SECTOR as u64) as usize;
            device::read(broker, record.first + index, &mut buffer)?;
            let count = (SECTOR - within).min((end - position) as usize);
            let at = (position - h.offset) as usize;
            reply.data[at..at + count].copy_from_slice(&buffer[within..within + count]);
            position += count as u64;
        }
        reply.header.length = h.aux as u32;
        reply.header.offset = end;
        Ok(())
    }

    fn bind(&mut self, h: &Header, data: &[u8]) -> Result<(), Status> {
        let lane = h.count as usize;
        if h.aux <= self.lanes[lane].max_generation {
            return Err(Status::Stale);
        }
        if data[..ID] != self.selection.catalog.id {
            return Err(Status::NoDevice);
        }
        let kind = match h.name_length {
            wire::BIND_EXPORT => Kind::Export,
            wire::BIND_VERIFY => Kind::Verify,
            _ => Kind::Restore,
        };
        if kind != Kind::Export && self.selection.catalog.find(h.entry).is_none() {
            return Err(Status::NotFound);
        }
        if self.lanes[lane].binding.is_some() {
            return Err(Status::Busy);
        }
        self.lanes[lane] = Lane {
            binding: Some(Binding {
                kind,
                entry: h.entry,
                generation: h.aux,
                export: Export::Fresh,
            }),
            high_water: 0,
            max_generation: h.aux,
        };
        Ok(())
    }

    /// Idempotent: an unbound lane is `Ok` with no state change.
    fn unbind(&mut self, lane: usize) {
        if self.writing.is_some_and(|writing| writing.lane == lane) {
            self.discard();
        }
        self.lanes[lane].binding = None;
    }

    /// FORMAT of a blank scope only (2.6): full-scope read first, then copy A,
    /// flush, copy B, flush. A write or flush failure is the failed-mount state.
    fn format<B: Broker, V: Verifier, E: Entropy>(
        &mut self,
        env: &mut Env<'_, B, V, E>,
        reply: &mut Reply,
    ) -> Result<(), Status> {
        let mut buffer = [0; SECTOR];
        for sector in 0..self.scope {
            device::read(env.broker, sector, &mut buffer)?;
            if buffer.iter().any(|byte| *byte != 0) {
                return Err(Status::Access);
            }
        }
        let mut id = [0; ID];
        env.entropy.fill(&mut id).map_err(|_| Status::Io)?;
        let fresh = Catalog::formatted(id, self.scope);
        let bytes = fresh.encode();
        let mut broker = device::Watch::new(&mut *env.broker);
        let written = write_copy(&mut broker, 0, &bytes)
            .and_then(|()| device::flush(&mut broker))
            .and_then(|()| write_copy(&mut broker, 1, &bytes))
            .and_then(|()| device::flush(&mut broker));
        if let Err(status) = written {
            self.state = State::Failed {
                no_device: broker.no_device,
            };
            // The write-or-flush status of "Device results" (2.5).
            return Err(status);
        }
        self.selection = Selection::committed(fresh);
        self.state = State::Mounted;
        reply.header.length = ID as u32;
        reply.data[..ID].copy_from_slice(&id);
        Ok(())
    }
}

/// Check 6 for a bound operation lane (2.2, 2.3 "Permitted on", 4.1).
fn permitted(binding: &Binding, h: &Header) -> bool {
    match (binding.kind, h.operation) {
        (_, wire::INFO | wire::LIST) => true,
        (Kind::Export, wire::CREATE) => binding.export == Export::Fresh,
        (Kind::Export, wire::APPEND | wire::SEAL | wire::ABORT) => {
            binding.export == Export::Writing(h.entry)
        }
        (Kind::Verify, wire::VERIFY) | (Kind::Restore, wire::READ) => h.entry == binding.entry,
        _ => false,
    }
}

fn write_copy<B: Broker>(broker: &mut B, slot: usize, bytes: &[u8; COPY]) -> Result<(), Status> {
    for index in 0..COPY_SECTORS as usize {
        let mut sector = [0; SECTOR];
        sector.copy_from_slice(&bytes[index * SECTOR..(index + 1) * SECTOR]);
        device::write_catalog(broker, slot as u64 * COPY_SECTORS + index as u64, &sector)?;
    }
    Ok(())
}

/// Stream an extent through the verifier. Returns the summary and the last
/// eight bytes of the extent (the archive trailer FNV). A device error is
/// returned as such; any other failure, including an archive that does not
/// end exactly at the extent end, is `Corrupt`.
fn check_extent<B: Broker, V: Verifier>(
    broker: &mut B,
    verifier: &mut V,
    first: u64,
    length: u64,
) -> Result<(Summary, u64), Status> {
    let mut extent = Extent {
        broker,
        first,
        length,
        position: 0,
        buffer: [0; SECTOR],
        error: None,
        last: [0; 8],
    };
    let verified = verifier.verify(&mut extent);
    if let Some(status) = extent.error {
        return Err(status);
    }
    match verified {
        Ok(summary) if extent.position == length => Ok((summary, u64::from_le_bytes(extent.last))),
        _ => Err(Status::Corrupt),
    }
}

struct Extent<'a, B> {
    broker: &'a mut B,
    first: u64,
    length: u64,
    position: u64,
    buffer: [u8; SECTOR],
    error: Option<Status>,
    last: [u8; 8],
}

impl<B: Broker> Source for Extent<'_, B> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
        if let Some(status) = self.error {
            return Err(status);
        }
        if self.position >= self.length || buffer.is_empty() {
            return Ok(0);
        }
        let within = (self.position % SECTOR as u64) as usize;
        if within == 0 {
            let sector = self.first + self.position / SECTOR as u64;
            if let Err(status) = device::read(self.broker, sector, &mut self.buffer) {
                self.error = Some(status);
                return Err(status);
            }
        }
        let count = buffer
            .len()
            .min(SECTOR - within)
            .min((self.length - self.position) as usize);
        let bytes = &self.buffer[within..within + count];
        buffer[..count].copy_from_slice(bytes);
        if count >= 8 {
            self.last.copy_from_slice(&bytes[count - 8..]);
        } else {
            self.last.copy_within(count.., 0);
            self.last[8 - count..].copy_from_slice(bytes);
        }
        self.position += count as u64;
        Ok(count)
    }
}
