#![no_std]
#![forbid(unsafe_code)]
//! Fixed-capacity M4 authority engine. Callers serialize operations and validate
//! user output before receive; this crate never sees pointers or user memory.

pub const SEND: u64 = 1;
pub const RECV: u64 = 2;
pub const DERIVE: u64 = 4;
pub const TRANSFER: u64 = 8;
pub const REVOKE: u64 = 16;
pub const ALL_RIGHTS: u64 = SEND | RECV | DERIVE | TRANSFER | REVOKE;
pub const MESSAGE_SIZE: usize = 320;
pub const MAX_PROCESSES: usize = 16;
pub const MAX_SLOTS: usize = 256;
pub const MAX_NODES: usize = 1024;
pub const MAX_ENDPOINTS: usize = 64;
pub const MAX_BLOCK_DEVICES: usize = 4;
pub const BLOCK_READ: u64 = 1;
pub const BLOCK_RIGHTS: u64 = BLOCK_READ | DERIVE | TRANSFER | REVOKE;
pub const MAX_OWNED_ENDPOINTS: usize = 4;
pub const QUEUE_CAPACITY: usize = 16;
pub const MAX_DEPTH: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockScope {
    pub device: u32,
    pub first_sector: u64,
    pub sector_count: u64,
}
impl BlockScope {
    pub fn validate(self) -> Result<(), Error> {
        if self.sector_count == 0 || self.first_sector.checked_add(self.sector_count).is_none() {
            Err(Error::Invalid)
        } else {
            Ok(())
        }
    }
    pub fn sector(self, relative: u64) -> Result<u64, Error> {
        self.validate()?;
        if relative >= self.sector_count {
            return Err(Error::Invalid);
        }
        self.first_sector
            .checked_add(relative)
            .ok_or(Error::Invalid)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockAuthorization {
    object_key: u64,
    scope: BlockScope,
}
impl BlockAuthorization {
    pub const fn object_key(self) -> u64 {
        self.object_key
    }
    pub const fn scope(self) -> BlockScope {
        self.scope
    }
    pub fn sector(self, relative: u64) -> Result<u64, Error> {
        self.scope.sector(relative)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum Error {
    Invalid = -1,
    NoSys = -2,
    Fault = -3,
    BadHandle = -4,
    Access = -5,
    Type = -6,
    Quota = -7,
    Again = -8,
    Closed = -9,
    Revoked = -10,
    MessageSize = -11,
}
impl Error {
    pub const fn code(self) -> i64 {
        self as i64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cap {
    pub handle: u64,
    pub rights: u64,
}
impl Cap {
    pub const EMPTY: Self = Self {
        handle: 0,
        rights: 0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub sender_pid: u64,
    pub tag: u64,
    pub len: u32,
    pub cap_count: u32,
    pub reserved: u64,
    pub caps: [Cap; 2],
    pub data: [u8; 256],
}
pub type Message320 = Message;
impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}
impl Message {
    pub const fn new() -> Self {
        Self {
            sender_pid: 0,
            tag: 0,
            len: 0,
            cap_count: 0,
            reserved: 0,
            caps: [Cap::EMPTY; 2],
            data: [0; 256],
        }
    }
    /// Decode a send request, including its canonical zero fields. Byte copies
    /// deliberately avoid repr(C), alignment assumptions and untrusted casts.
    pub fn decode(bytes: &[u8; MESSAGE_SIZE]) -> Result<Self, Error> {
        let mut m = Self::new();
        m.sender_pid = get64(bytes, 0);
        m.tag = get64(bytes, 8);
        m.len = get32(bytes, 16);
        m.cap_count = get32(bytes, 20);
        m.reserved = get64(bytes, 24);
        for (i, cap) in m.caps.iter_mut().enumerate() {
            cap.handle = get64(bytes, 32 + i * 16);
            cap.rights = get64(bytes, 40 + i * 16);
        }
        m.data.copy_from_slice(&bytes[64..]);
        m.validate_send()?;
        Ok(m)
    }
    pub fn encode(&self) -> [u8; MESSAGE_SIZE] {
        let mut b = [0; MESSAGE_SIZE];
        put64(&mut b, 0, self.sender_pid);
        put64(&mut b, 8, self.tag);
        b[16..20].copy_from_slice(&self.len.to_le_bytes());
        b[20..24].copy_from_slice(&self.cap_count.to_le_bytes());
        put64(&mut b, 24, self.reserved);
        for (i, cap) in self.caps.iter().enumerate() {
            put64(&mut b, 32 + i * 16, cap.handle);
            put64(&mut b, 40 + i * 16, cap.rights);
        }
        b[64..].copy_from_slice(&self.data);
        b
    }
    pub fn validate_send(&self) -> Result<(), Error> {
        if self.len > 256 || self.cap_count > 2 {
            return Err(Error::MessageSize);
        }
        if self.sender_pid != 0
            || self.reserved != 0
            || self.caps[self.cap_count as usize..]
                .iter()
                .any(|x| *x != Cap::EMPTY)
            || self.data[self.len as usize..].iter().any(|x| *x != 0)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
fn get64(b: &[u8; MESSAGE_SIZE], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}
fn get32(b: &[u8; MESSAGE_SIZE], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn put64(b: &mut [u8; MESSAGE_SIZE], o: usize, x: u64) {
    b[o..o + 8].copy_from_slice(&x.to_le_bytes());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Id {
    index: u16,
    generation: u32,
}
impl Id {
    const NONE: Self = Self {
        index: u16::MAX,
        generation: 0,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    generation: u32,
    retired: bool,
    node: Id,
}
impl Slot {
    const EMPTY: Self = Self {
        generation: 1,
        retired: false,
        node: Id::NONE,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Process {
    pid: u64,
    live: bool,
    slots: [Slot; MAX_SLOTS],
}
impl Process {
    const EMPTY: Self = Self {
        pid: 0,
        live: false,
        slots: [Slot::EMPTY; MAX_SLOTS],
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Node {
    generation: u32,
    live: bool,
    retired: bool,
    revoked: bool,
    object: Id,
    parent: Id,
    issuer: u64,
    rights: u64,
    refs: u32,
    depth: u8,
}
impl Node {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        revoked: false,
        object: Id::NONE,
        parent: Id::NONE,
        issuer: 0,
        rights: 0,
        refs: 0,
        depth: 0,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Queued {
    message: Message,
    caps: [Id; 2],
}
impl Queued {
    const EMPTY: Self = Self {
        message: Message::new(),
        caps: [Id::NONE; 2],
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Endpoint {
    generation: u32,
    live: bool,
    retired: bool,
    open: bool,
    owner: u64,
    nodes: u32,
    head: u8,
    len: u8,
    queue: [Queued; QUEUE_CAPACITY],
}
impl Endpoint {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        open: false,
        owner: 0,
        nodes: 0,
        head: 0,
        len: 0,
        queue: [Queued::EMPTY; QUEUE_CAPACITY],
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockObject {
    generation: u32,
    live: bool,
    retired: bool,
    open: bool,
    owner: u64,
    nodes: u32,
    scope: BlockScope,
}
impl BlockObject {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        open: false,
        owner: 0,
        nodes: 0,
        scope: BlockScope {
            device: 0,
            first_sector: 0,
            sector_count: 0,
        },
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub processes: usize,
    pub handles: usize,
    pub nodes: usize,
    pub endpoints: usize,
    pub open_endpoints: usize,
    pub queued_messages: usize,
    pub revoked_nodes: usize,
    pub retired_slots: usize,
    pub retired_nodes: usize,
    pub retired_endpoints: usize,
    pub block_objects: usize,
    pub retired_block_objects: usize,
    pub state_digest: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engine {
    processes: [Process; MAX_PROCESSES],
    nodes: [Node; MAX_NODES],
    endpoints: [Endpoint; MAX_ENDPOINTS],
    blocks: [BlockObject; MAX_BLOCK_DEVICES],
    last_pid: u64,
}
impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
impl Engine {
    /// Use directly in a static initializer in the kernel, not as a stack local.
    pub const fn new() -> Self {
        Self {
            processes: [Process::EMPTY; MAX_PROCESSES],
            nodes: [Node::EMPTY; MAX_NODES],
            endpoints: [Endpoint::EMPTY; MAX_ENDPOINTS],
            blocks: [BlockObject::EMPTY; MAX_BLOCK_DEVICES],
            last_pid: 0,
        }
    }
    pub fn register(&mut self, pid: u64) -> Result<(), Error> {
        if pid == 0 || pid <= self.last_pid {
            return Err(Error::Invalid);
        }
        let p = self
            .processes
            .iter()
            .position(|p| !p.live)
            .ok_or(Error::Quota)?;
        self.processes[p].pid = pid;
        self.processes[p].live = true;
        self.last_pid = pid;
        Ok(())
    }
    pub fn create_endpoint(&mut self, owner: u64) -> Result<u64, Error> {
        let p = self.process(owner)?;
        // Closed tombstones still occupy object capacity until their last node
        // reference disappears. This prevents stale handles reaching a new object.
        if self
            .endpoints
            .iter()
            .filter(|e| e.live && e.owner == owner)
            .count()
            >= MAX_OWNED_ENDPOINTS
        {
            return Err(Error::Quota);
        }
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .endpoints
            .iter()
            .position(|e| !e.live && !e.retired)
            .ok_or(Error::Quota)?;
        let generation = self.endpoints[object].generation;
        self.endpoints[object] = Endpoint {
            generation,
            live: true,
            open: true,
            owner,
            ..Endpoint::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: object as u16,
                generation,
            },
            Id::NONE,
            owner,
            ALL_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_endpoint(&self, pid: u64, handle: u64, required: u64) -> Result<(), Error> {
        let node = self.resolve(pid, handle)?;
        if self.nodes[node].object.index as usize >= MAX_ENDPOINTS {
            return Err(Error::Type);
        }
        self.require(node, required)?;
        if required & RECV != 0
            && self.endpoints[self.nodes[node].object.index as usize].owner != pid
        {
            return Err(Error::Access);
        }
        Ok(())
    }
    /// Boot-manifest constructor only; no user syscall grants object creation.
    pub fn create_block_device(&mut self, owner: u64, scope: BlockScope) -> Result<u64, Error> {
        let p = self.process(owner)?;
        scope.validate()?;
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .blocks
            .iter()
            .position(|b| !b.live && !b.retired)
            .ok_or(Error::Quota)?;
        let generation = self.blocks[object].generation;
        self.blocks[object] = BlockObject {
            generation,
            live: true,
            open: true,
            owner,
            scope,
            ..BlockObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (MAX_ENDPOINTS + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            BLOCK_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_block(
        &self,
        pid: u64,
        handle: u64,
        required: u64,
    ) -> Result<BlockAuthorization, Error> {
        let node = self.resolve(pid, handle)?;
        let object = self.nodes[node].object;
        if (object.index as usize) < MAX_ENDPOINTS {
            return Err(Error::Type);
        }
        self.require(node, required)?;
        Ok(BlockAuthorization {
            object_key: (u64::from(object.generation) << 32) | u64::from(object.index),
            scope: self.blocks[object.index as usize - MAX_ENDPOINTS].scope,
        })
    }
    /// Close is deliberately valid for revoked nodes and closed objects.
    pub fn close(&mut self, pid: u64, handle: u64) -> Result<(), Error> {
        let p = self.process(pid)?;
        let s = self.handle_slot(p, handle)?;
        let id = self.processes[p].slots[s].node;
        self.detach(p, s);
        self.nodes[id.index as usize].refs -= 1;
        self.maintain();
        Ok(())
    }
    pub fn derive(&mut self, pid: u64, handle: u64, rights: u64) -> Result<u64, Error> {
        let n = self.resolve(pid, handle)?;
        self.require(n, DERIVE)?;
        if rights & !ALL_RIGHTS != 0 {
            return Err(Error::Invalid);
        }
        self.subset(n, rights, false)?;
        let p = self.process(pid)?;
        let slot = self.slots(p, 1)?[0];
        let child = self.free_nodes(1)?[0];
        let parent = self.nodes[n];
        if parent.depth >= MAX_DEPTH {
            return Err(Error::Quota);
        }
        self.make_node(
            child,
            parent.object,
            self.node_id(n),
            pid,
            rights,
            parent.depth + 1,
        );
        Ok(self.attach(p, slot, self.node_id(child)))
    }
    /// Boot-manifest delegation uses the same authority rules as IPC transfer.
    pub fn bootstrap_delegate(
        &mut self,
        from: u64,
        handle: u64,
        to: u64,
        rights: u64,
    ) -> Result<u64, Error> {
        let n = self.resolve(from, handle)?;
        self.require(n, TRANSFER)?;
        if rights & !ALL_RIGHTS != 0 {
            return Err(Error::Invalid);
        }
        self.subset(n, rights, true)?;
        let p = self.process(to)?;
        let slot = self.slots(p, 1)?[0];
        let child = self.free_nodes(1)?[0];
        let parent = self.nodes[n];
        if parent.depth >= MAX_DEPTH {
            return Err(Error::Quota);
        }
        self.make_node(
            child,
            parent.object,
            self.node_id(n),
            from,
            rights,
            parent.depth + 1,
        );
        Ok(self.attach(p, slot, self.node_id(child)))
    }
    pub fn revoke(&mut self, pid: u64, handle: u64) -> Result<(), Error> {
        let n = self.resolve(pid, handle)?;
        self.require(n, REVOKE)?;
        let root = self.node_id(n);
        // Mark first, collect afterwards: ancestry never depends on array order.
        let mut marks = [false; MAX_NODES];
        for (i, mark) in marks.iter_mut().enumerate() {
            *mark = self.nodes[i].live && self.descends_from(i, root);
        }
        for (i, mark) in marks.into_iter().enumerate() {
            if mark {
                self.nodes[i].revoked = true;
            }
        }
        self.maintain();
        Ok(())
    }
    pub fn send(&mut self, pid: u64, endpoint: u64, message: &Message) -> Result<(), Error> {
        self.check_endpoint(pid, endpoint, SEND)?;
        message.validate_send()?;
        let count = message.cap_count as usize;
        let mut parents = [0; 2];
        for (i, parent) in parents.iter_mut().enumerate().take(count) {
            *parent = self.resolve(pid, message.caps[i].handle)?;
            self.require(*parent, TRANSFER)?;
            if message.caps[i].rights & !ALL_RIGHTS != 0 {
                return Err(Error::Invalid);
            }
            self.subset(*parent, message.caps[i].rights, true)?;
        }
        // Every transferred handle/right is validated before any resource
        // failure, including a preceding descriptor's delegation-depth quota.
        if parents[..count]
            .iter()
            .any(|n| self.nodes[*n].depth >= MAX_DEPTH)
        {
            return Err(Error::Quota);
        }
        let en = self.resolve(pid, endpoint)?;
        let ep = self.nodes[en].object.index as usize;
        if self.endpoints[ep].len as usize == QUEUE_CAPACITY {
            return Err(Error::Again);
        }
        let reserved = self.free_nodes(count)?;
        // No fallible operation follows this point. Resources were only observed,
        // never partially reserved, before this serialized transaction commits.
        let mut q = Queued {
            message: *message,
            caps: [Id::NONE; 2],
        };
        q.message.sender_pid = pid;
        for i in 0..count {
            let parent = self.nodes[parents[i]];
            self.make_node(
                reserved[i],
                parent.object,
                self.node_id(parents[i]),
                pid,
                message.caps[i].rights,
                parent.depth + 1,
            );
            q.caps[i] = self.node_id(reserved[i]);
            q.message.caps[i].handle = 0;
        }
        let e = &mut self.endpoints[ep];
        let tail = (usize::from(e.head) + usize::from(e.len)) % QUEUE_CAPACITY;
        e.queue[tail] = q;
        e.len += 1;
        Ok(())
    }
    /// The kernel must validate the entire output buffer BEFORE calling this.
    /// All receive resources are checked before queue/slot state is changed.
    pub fn receive(&mut self, pid: u64, endpoint: u64) -> Result<Message, Error> {
        self.check_endpoint(pid, endpoint, RECV)?;
        let n = self.resolve(pid, endpoint)?;
        let ep = self.nodes[n].object.index as usize;
        let e = &self.endpoints[ep];
        if e.len == 0 {
            return Err(Error::Again);
        }
        let q = e.queue[e.head as usize];
        let count = q.message.cap_count as usize;
        let p = self.process(pid)?;
        let slots = self.slots(p, count)?;
        for id in q.caps.iter().take(count) {
            if !self.node_live(*id) || self.nodes[id.index as usize].revoked {
                return Err(Error::Revoked);
            }
        }
        let mut result = q.message;
        for (i, slot) in slots.into_iter().enumerate().take(count) {
            result.caps[i].handle = self.attach(p, slot, q.caps[i]);
        }
        // Queue refs become local-slot refs, so counts do not change.
        let e = &mut self.endpoints[ep];
        e.queue[e.head as usize] = Queued::EMPTY;
        e.head = ((usize::from(e.head) + 1) % QUEUE_CAPACITY) as u8;
        e.len -= 1;
        Ok(result)
    }
    pub fn exit(&mut self, pid: u64) -> Result<(), Error> {
        let p = self.process(pid)?;
        let mut roots = [false; MAX_NODES];
        for (i, root) in roots.iter_mut().enumerate() {
            *root = self.nodes[i].live && self.nodes[i].issuer == pid;
        }
        for slot in &self.processes[p].slots[1..] {
            if self.node_live(slot.node) {
                roots[slot.node.index as usize] = true;
            }
        }
        let mut marks = roots;
        for (i, mark) in marks.iter_mut().enumerate() {
            if !self.nodes[i].live {
                continue;
            }
            let mut ancestor = self.node_id(i);
            for _ in 0..=MAX_DEPTH {
                if !self.node_live(ancestor) {
                    break;
                }
                if roots[ancestor.index as usize] {
                    *mark = true;
                    break;
                }
                ancestor = self.nodes[ancestor.index as usize].parent;
            }
        }
        for (i, mark) in marks.into_iter().enumerate() {
            if mark {
                self.nodes[i].revoked = true;
            }
        }
        for s in 1..MAX_SLOTS {
            let id = self.processes[p].slots[s].node;
            if id != Id::NONE {
                self.detach(p, s);
                self.nodes[id.index as usize].refs -= 1;
            }
        }
        self.processes[p].live = false;
        for e in &mut self.endpoints {
            if e.live && e.owner == pid {
                e.open = false;
            }
        }
        for b in &mut self.blocks {
            if b.live && b.owner == pid {
                b.open = false;
            }
        }
        self.maintain();
        Ok(())
    }
    fn process(&self, pid: u64) -> Result<usize, Error> {
        self.processes
            .iter()
            .position(|p| p.live && p.pid == pid)
            .ok_or(Error::BadHandle)
    }
    fn handle_slot(&self, p: usize, h: u64) -> Result<usize, Error> {
        let s = (h & 0xffff_ffff) as usize;
        let g = (h >> 32) as u32;
        if s == 0 || s >= MAX_SLOTS {
            return Err(Error::BadHandle);
        }
        let slot = self.processes[p].slots[s];
        if slot.retired || slot.generation != g || !self.node_live(slot.node) {
            return Err(Error::BadHandle);
        }
        Ok(s)
    }
    fn resolve(&self, pid: u64, h: u64) -> Result<usize, Error> {
        let p = self.process(pid)?;
        let s = self.handle_slot(p, h)?;
        let n = self.processes[p].slots[s].node.index as usize;
        if self.nodes[n].revoked {
            return Err(Error::Revoked);
        }
        let o = self.nodes[n].object;
        if !self.object_live(o) || !self.object_open(o) {
            return Err(Error::Closed);
        }
        Ok(n)
    }
    fn require(&self, n: usize, rights: u64) -> Result<(), Error> {
        if rights & !self.allowed_rights(self.nodes[n].object) != 0 {
            return Err(Error::Invalid);
        }
        if rights & !self.nodes[n].rights != 0 {
            return Err(Error::Access);
        }
        Ok(())
    }
    fn subset(&self, n: usize, rights: u64, transfer: bool) -> Result<(), Error> {
        self.require(n, rights)?;
        if transfer && rights & RECV != 0 {
            return Err(Error::Access);
        }
        Ok(())
    }
    fn slots(&self, p: usize, count: usize) -> Result<[usize; 2], Error> {
        let mut result = [0; 2];
        let mut found = 0;
        for s in 1..MAX_SLOTS {
            if found == count {
                break;
            }
            let slot = self.processes[p].slots[s];
            if !slot.retired && slot.node == Id::NONE {
                result[found] = s;
                found += 1;
            }
        }
        if found == count {
            Ok(result)
        } else {
            Err(Error::Quota)
        }
    }
    fn free_nodes(&self, count: usize) -> Result<[usize; 2], Error> {
        let mut result = [0; 2];
        let mut found = 0;
        for (i, node) in self.nodes.iter().enumerate() {
            if found == count {
                break;
            }
            if !node.live && !node.retired {
                result[found] = i;
                found += 1;
            }
        }
        if found == count {
            Ok(result)
        } else {
            Err(Error::Quota)
        }
    }
    fn node_id(&self, n: usize) -> Id {
        Id {
            index: n as u16,
            generation: self.nodes[n].generation,
        }
    }
    fn node_live(&self, id: Id) -> bool {
        (id.index as usize) < MAX_NODES
            && self.nodes[id.index as usize].live
            && self.nodes[id.index as usize].generation == id.generation
    }
    fn object_live(&self, id: Id) -> bool {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            self.endpoints[i].live && self.endpoints[i].generation == id.generation
        } else if i < MAX_ENDPOINTS + MAX_BLOCK_DEVICES {
            let b = self.blocks[i - MAX_ENDPOINTS];
            b.live && b.generation == id.generation
        } else {
            false
        }
    }
    fn object_open(&self, id: Id) -> bool {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            self.endpoints[i].open
        } else {
            self.blocks[i - MAX_ENDPOINTS].open
        }
    }
    fn allowed_rights(&self, id: Id) -> u64 {
        if (id.index as usize) < MAX_ENDPOINTS {
            ALL_RIGHTS
        } else {
            BLOCK_RIGHTS
        }
    }
    fn object_refs(&mut self, id: Id) -> &mut u32 {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            &mut self.endpoints[i].nodes
        } else {
            &mut self.blocks[i - MAX_ENDPOINTS].nodes
        }
    }
    fn attach(&mut self, p: usize, s: usize, node: Id) -> u64 {
        self.processes[p].slots[s].node = node;
        (u64::from(self.processes[p].slots[s].generation) << 32) | s as u64
    }
    fn detach(&mut self, p: usize, s: usize) {
        let slot = &mut self.processes[p].slots[s];
        slot.node = Id::NONE;
        if let Some(g) = slot.generation.checked_add(1) {
            slot.generation = g;
        } else {
            slot.retired = true;
        }
    }
    fn make_node(&mut self, i: usize, object: Id, parent: Id, issuer: u64, rights: u64, depth: u8) {
        let generation = self.nodes[i].generation;
        self.nodes[i] = Node {
            generation,
            live: true,
            retired: false,
            revoked: false,
            object,
            parent,
            issuer,
            rights,
            refs: 1,
            depth,
        };
        *self.object_refs(object) += 1;
        if parent != Id::NONE {
            self.nodes[parent.index as usize].refs += 1;
        }
    }
    fn descends_from(&self, n: usize, root: Id) -> bool {
        let mut parent = self.nodes[n].parent;
        for _ in 0..MAX_DEPTH {
            if parent == root {
                return true;
            }
            if !self.node_live(parent) {
                return false;
            }
            parent = self.nodes[parent.index as usize].parent;
        }
        false
    }
    fn remove_queued(&mut self, ep: usize, pos: usize) {
        let e = &mut self.endpoints[ep];
        let at = (usize::from(e.head) + pos) % QUEUE_CAPACITY;
        let q = e.queue[at];
        for i in 0..q.message.cap_count as usize {
            self.nodes[q.caps[i].index as usize].refs -= 1;
        }
        for j in pos..usize::from(e.len) - 1 {
            let a = (usize::from(e.head) + j) % QUEUE_CAPACITY;
            let b = (a + 1) % QUEUE_CAPACITY;
            e.queue[a] = e.queue[b];
        }
        let tail = (usize::from(e.head) + usize::from(e.len) - 1) % QUEUE_CAPACITY;
        e.queue[tail] = Queued::EMPTY;
        e.len -= 1;
    }
    fn maintain(&mut self) {
        // At most 64 endpoints, 4096 local slots and 1024 queue positions. All
        // authority is revoked first; no user-visible partial transaction exists.
        for ep in 0..MAX_ENDPOINTS {
            if !self.endpoints[ep].live {
                continue;
            }
            if self.endpoints[ep].open {
                let id = Id {
                    index: ep as u16,
                    generation: self.endpoints[ep].generation,
                };
                let receiver = self
                    .processes
                    .iter()
                    .filter(|p| p.live && p.pid == self.endpoints[ep].owner)
                    .any(|p| {
                        p.slots[1..].iter().any(|s| {
                            self.node_live(s.node) && {
                                let n = self.nodes[s.node.index as usize];
                                !n.revoked && n.object == id && n.rights & RECV != 0
                            }
                        })
                    });
                if !receiver {
                    self.endpoints[ep].open = false;
                }
            }
            let mut pos = 0;
            while pos < self.endpoints[ep].len as usize {
                let e = &self.endpoints[ep];
                let q = e.queue[(usize::from(e.head) + pos) % QUEUE_CAPACITY];
                let cancel = !e.open
                    || q.caps[..q.message.cap_count as usize]
                        .iter()
                        .any(|id| !self.node_live(*id) || self.nodes[id.index as usize].revoked);
                if cancel {
                    self.remove_queued(ep, pos);
                } else {
                    pos += 1;
                }
            }
        }
        // One array scan, with at most eight ancestor steps per reclaimed leaf.
        // Every node can be reclaimed only once, regardless of array ordering.
        for first in 0..MAX_NODES {
            let mut i = first;
            for _ in 0..=MAX_DEPTH {
                let n = self.nodes[i];
                if !n.live || n.refs != 0 {
                    break;
                }
                if n.parent != Id::NONE {
                    self.nodes[n.parent.index as usize].refs -= 1;
                }
                *self.object_refs(n.object) -= 1;
                let (generation, retired) = advance(n.generation);
                self.nodes[i] = Node {
                    generation,
                    retired,
                    ..Node::EMPTY
                };
                if n.parent == Id::NONE {
                    break;
                }
                i = n.parent.index as usize;
            }
        }
        for e in &mut self.endpoints {
            if e.live && !e.open && e.nodes == 0 {
                let (generation, retired) = advance(e.generation);
                *e = Endpoint {
                    generation,
                    retired,
                    ..Endpoint::EMPTY
                };
            }
        }
        // Block objects have no receiver/queue. Their last node, not a RECV
        // handle, controls reclamation; owner exit closes surviving tombstones.
        for b in &mut self.blocks {
            if b.live && b.nodes == 0 {
                let (generation, retired) = advance(b.generation);
                *b = BlockObject {
                    generation,
                    retired,
                    ..BlockObject::EMPTY
                };
            }
        }
    }
    pub fn stats(&self) -> Stats {
        let mut s = Stats {
            processes: 0,
            handles: 0,
            nodes: 0,
            endpoints: 0,
            open_endpoints: 0,
            queued_messages: 0,
            revoked_nodes: 0,
            retired_slots: 0,
            retired_nodes: 0,
            retired_endpoints: 0,
            block_objects: 0,
            retired_block_objects: 0,
            state_digest: self.state_digest(),
        };
        for p in &self.processes {
            s.processes += usize::from(p.live);
            for x in &p.slots[1..] {
                s.handles += usize::from(x.node != Id::NONE);
                s.retired_slots += usize::from(x.retired);
            }
        }
        for n in &self.nodes {
            s.nodes += usize::from(n.live);
            s.revoked_nodes += usize::from(n.live && n.revoked);
            s.retired_nodes += usize::from(n.retired);
        }
        for e in &self.endpoints {
            s.endpoints += usize::from(e.live);
            s.open_endpoints += usize::from(e.live && e.open);
            s.queued_messages += usize::from(e.len);
            s.retired_endpoints += usize::from(e.retired);
        }
        for b in &self.blocks {
            s.block_objects += usize::from(b.live);
            s.retired_block_objects += usize::from(b.retired);
        }
        s
    }
    /// Diagnostic fingerprint only, not cryptography or proof against collisions.
    pub fn state_digest(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        fn add(h: &mut u64, v: u64) {
            // Fixed typed word boundaries: this is a fast diagnostic mixer,
            // never an authentication/integrity primitive or a crypto hash.
            *h = (*h ^ v).wrapping_mul(0x100000001b3);
        }
        fn id(h: &mut u64, x: Id) {
            add(h, u64::from(x.index));
            add(h, u64::from(x.generation));
        }
        add(&mut h, self.last_pid);
        for p in &self.processes {
            add(&mut h, p.pid);
            add(&mut h, u64::from(p.live));
            for s in &p.slots {
                add(&mut h, u64::from(s.generation));
                add(&mut h, u64::from(s.retired));
                id(&mut h, s.node);
            }
        }
        for n in &self.nodes {
            for x in [
                u64::from(n.generation),
                u64::from(n.live),
                u64::from(n.retired),
                u64::from(n.revoked),
                n.issuer,
                n.rights,
                u64::from(n.refs),
                u64::from(n.depth),
            ] {
                add(&mut h, x);
            }
            id(&mut h, n.object);
            id(&mut h, n.parent);
        }
        for e in &self.endpoints {
            for x in [
                u64::from(e.generation),
                u64::from(e.live),
                u64::from(e.retired),
                u64::from(e.open),
                e.owner,
                u64::from(e.nodes),
                u64::from(e.head),
                u64::from(e.len),
            ] {
                add(&mut h, x);
            }
            for q in &e.queue {
                for word in q.message.encode().chunks_exact(8) {
                    add(&mut h, u64::from_le_bytes(word.try_into().unwrap()));
                }
                for x in q.caps {
                    id(&mut h, x);
                }
            }
        }
        for b in &self.blocks {
            for x in [
                u64::from(b.generation),
                u64::from(b.live),
                u64::from(b.retired),
                u64::from(b.open),
                b.owner,
                u64::from(b.nodes),
                u64::from(b.scope.device),
                b.scope.first_sector,
                b.scope.sector_count,
            ] {
                add(&mut h, x);
            }
        }
        h
    }
    /// Full structural/refcount check for observers/tests, not a syscall hook.
    pub fn check_invariants(&self) -> bool {
        let mut refs = [0u32; MAX_NODES];
        let mut objects = [0u32; MAX_ENDPOINTS + MAX_BLOCK_DEVICES];
        for p in &self.processes {
            if p.slots[0].node != Id::NONE {
                return false;
            }
            if p.live && (p.pid == 0 || p.pid > self.last_pid) {
                return false;
            }
            for slot in &p.slots[1..] {
                if slot.generation == 0
                    || slot.retired && (slot.node != Id::NONE || slot.generation != u32::MAX)
                {
                    return false;
                }
                if slot.node != Id::NONE {
                    if !p.live || !self.node_live(slot.node) {
                        return false;
                    }
                    refs[slot.node.index as usize] += 1;
                    let n = self.nodes[slot.node.index as usize];
                    if !self.object_live(n.object)
                        || n.rights & RECV != 0
                            && ((n.object.index as usize) >= MAX_ENDPOINTS
                                || self.endpoints[n.object.index as usize].owner != p.pid)
                    {
                        return false;
                    }
                }
            }
        }
        for n in &self.nodes {
            if n.generation == 0 || n.retired && (n.live || n.generation != u32::MAX) {
                return false;
            }
            if !n.live {
                continue;
            }
            if n.retired
                || n.generation == 0
                || n.rights & !self.allowed_rights(n.object) != 0
                || n.depth > MAX_DEPTH
                || n.issuer == 0
                || n.issuer > self.last_pid
                || !self.object_live(n.object)
            {
                return false;
            }
            objects[n.object.index as usize] += 1;
            if n.parent == Id::NONE {
                if n.depth != 0 {
                    return false;
                }
            } else {
                if !self.node_live(n.parent) {
                    return false;
                }
                let p = self.nodes[n.parent.index as usize];
                if p.object != n.object
                    || p.depth + 1 != n.depth
                    || n.rights & !p.rights != 0
                    || p.revoked && !n.revoked
                {
                    return false;
                }
                refs[n.parent.index as usize] += 1;
            }
        }
        for (i, e) in self.endpoints.iter().enumerate() {
            if e.generation == 0
                || e.retired && e.generation != u32::MAX
                || e.len as usize > QUEUE_CAPACITY
                || e.head as usize >= QUEUE_CAPACITY
            {
                return false;
            }
            if e.nodes != objects[i]
                || !e.live && (e.nodes != 0 || e.len != 0)
                || e.live && e.retired
                || !e.open && e.len != 0
                || e.open && !e.live
            {
                return false;
            }
            if e.live
                && self
                    .endpoints
                    .iter()
                    .filter(|x| x.live && x.owner == e.owner)
                    .count()
                    > MAX_OWNED_ENDPOINTS
            {
                return false;
            }
            if e.open {
                let Ok(owner) = self.process(e.owner) else {
                    return false;
                };
                let id = Id {
                    index: i as u16,
                    generation: e.generation,
                };
                if !self.processes[owner].slots[1..].iter().any(|s| {
                    self.node_live(s.node) && {
                        let n = self.nodes[s.node.index as usize];
                        !n.revoked && n.object == id && n.rights & RECV != 0
                    }
                }) {
                    return false;
                }
            }
            for p in 0..e.len as usize {
                let q = e.queue[(e.head as usize + p) % QUEUE_CAPACITY];
                if q.message.len > 256
                    || q.message.cap_count > 2
                    || q.message.sender_pid == 0
                    || q.message.reserved != 0
                    || q.message.data[q.message.len as usize..]
                        .iter()
                        .any(|x| *x != 0)
                {
                    return false;
                }
                for (j, id) in q.caps.iter().enumerate() {
                    if j < q.message.cap_count as usize {
                        if !self.node_live(*id) {
                            return false;
                        }
                        let n = self.nodes[id.index as usize];
                        if n.revoked
                            || n.rights & RECV != 0
                            || n.issuer != q.message.sender_pid
                            || n.parent == Id::NONE
                            || n.rights != q.message.caps[j].rights
                            || q.message.caps[j].handle != 0
                        {
                            return false;
                        }
                        refs[id.index as usize] += 1;
                    } else if *id != Id::NONE || q.message.caps[j] != Cap::EMPTY {
                        return false;
                    }
                }
            }
        }
        for (i, b) in self.blocks.iter().enumerate() {
            if b.generation == 0
                || b.retired && (b.live || b.generation != u32::MAX)
                || b.nodes != objects[MAX_ENDPOINTS + i]
                || !b.live && (b.open || b.nodes != 0)
            {
                return false;
            }
            if b.live && (b.nodes == 0 || b.owner == 0 || b.scope.validate().is_err()) {
                return false;
            }
            if b.open && self.process(b.owner).is_err() {
                return false;
            }
        }
        self.nodes.iter().enumerate().all(|(i, n)| {
            if n.live {
                n.refs == refs[i] && n.refs != 0
            } else {
                refs[i] == 0
            }
        })
    }
}
fn advance(g: u32) -> (u32, bool) {
    match g.checked_add(1) {
        Some(n) => (n, false),
        None => (g, true),
    }
}

#[cfg(test)]
mod tests;
