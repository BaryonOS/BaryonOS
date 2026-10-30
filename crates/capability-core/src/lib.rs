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
pub const BLOCK_WRITE: u64 = 32;
pub const BLOCK_FLUSH: u64 = 64;
pub const BLOCK_READ_RIGHTS: u64 = BLOCK_READ | DERIVE | TRANSFER | REVOKE;
pub const BLOCK_RIGHTS: u64 = BLOCK_READ_RIGHTS | BLOCK_WRITE | BLOCK_FLUSH;
/// K05 process management (ADR-042). A `Launch` object names one built-in
/// image; a `Process` object names one spawned child. Neither type knows
/// DERIVE, TRANSFER or REVOKE, so this authority can never leave its holder.
pub const LAUNCH: u64 = 128;
pub const OBSERVE: u64 = 256;
pub const KILL: u64 = 512;
pub const LAUNCH_RIGHTS: u64 = LAUNCH;
pub const PROCESS_RIGHTS: u64 = OBSERVE | KILL;
/// B01 (ADR-043): read raw bytes from the kernel's entropy device broker.
/// Only the Key Service holds this; it cannot be derived or transferred.
pub const ENTROPY: u64 = 1024;
pub const ENTROPY_RIGHTS: u64 = ENTROPY;
/// Raw NIC transport authority for the isolated native network service.
/// Manifest-only, non-delegable; closing it or owner exit ends access.
pub const NETWORK: u64 = 2048;
pub const NETWORK_RIGHTS: u64 = NETWORK;
/// Sole manifest-only framebuffer transport holder; no delegation or raw VRAM.
pub const DISPLAY: u64 = 4096;
pub const DISPLAY_RIGHTS: u64 = DISPLAY;
/// Sole manifest-only PS/2 hardware source; no arbitrary port-I/O interface.
pub const INPUT: u64 = 8192;
pub const INPUT_RIGHTS: u64 = INPUT;
pub const KNOWN_RIGHTS: u64 = ALL_RIGHTS
    | BLOCK_RIGHTS
    | LAUNCH_RIGHTS
    | PROCESS_RIGHTS
    | ENTROPY_RIGHTS
    | NETWORK_RIGHTS
    | DISPLAY_RIGHTS
    | INPUT_RIGHTS;
pub const MAX_LAUNCH_OBJECTS: usize = 8;
pub const MAX_PROCESS_OBJECTS: usize = MAX_PROCESSES;
pub const MAX_ENTROPY_OBJECTS: usize = 4;
pub const MAX_NETWORK_OBJECTS: usize = 4;
const LAUNCH_BASE: usize = MAX_ENDPOINTS + MAX_BLOCK_DEVICES;
const PROCESS_BASE: usize = LAUNCH_BASE + MAX_LAUNCH_OBJECTS;
const ENTROPY_BASE: usize = PROCESS_BASE + MAX_PROCESS_OBJECTS;
const NETWORK_BASE: usize = ENTROPY_BASE + MAX_ENTROPY_OBJECTS;
pub const MAX_DISPLAY_OBJECTS: usize = 1;
pub const MAX_INPUT_OBJECTS: usize = 1;
const DISPLAY_BASE: usize = NETWORK_BASE + MAX_NETWORK_OBJECTS;
const INPUT_BASE: usize = DISPLAY_BASE + MAX_DISPLAY_OBJECTS;
const OBJECT_COUNT: usize = INPUT_BASE + MAX_INPUT_OBJECTS;
/// Lifecycle state of a spawned child as reported through `OBSERVE`.
pub const PROCESS_RUNNING: u8 = 1;
pub const PROCESS_EXITED: u8 = 2;
pub const PROCESS_FAULTED: u8 = 3;
pub const PROCESS_KILLED: u8 = 4;
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

/// Read-only view of a `Process` object for the holder of an OBSERVE/KILL node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessView {
    pub child: u64,
    pub state: u8,
    pub code: u64,
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
/// Hash the same forty little-endian wire words as Message::encode without
/// materialising a 320-byte temporary. Every field and all 256 payload bytes
/// participate even when len/cap_count/live metadata are invalid or zero.
const DIAGNOSTIC_PRIME: u64 = 0x100000001b3;
struct DiagnosticHasher {
    value: u64,
    zeros: usize,
}
impl DiagnosticHasher {
    const FACTORS: [u64; 65] = {
        let mut out = [1u64; 65];
        let mut i = 1;
        while i < 65 {
            out[i] = out[i - 1].wrapping_mul(DIAGNOSTIC_PRIME);
            i += 1;
        }
        out
    };
    fn new(value: u64) -> Self {
        Self { value, zeros: 0 }
    }
    #[inline(always)]
    fn flush(&mut self) {
        if self.zeros != 0 {
            let factor = if self.zeros < 65 {
                Self::FACTORS[self.zeros]
            } else {
                Self::long_factor(self.zeros)
            };
            self.value = self.value.wrapping_mul(factor);
            self.zeros = 0;
        }
    }
    // Canonical empty-message runs fit the small table. Keep the exact
    // arbitrary-state fallback out of each inlined hot-path word operation.
    #[cold]
    #[inline(never)]
    fn long_factor(mut n: usize) -> u64 {
        let mut base = DIAGNOSTIC_PRIME;
        let mut factor = 1u64;
        while n != 0 {
            if n & 1 != 0 {
                factor = factor.wrapping_mul(base);
            }
            n >>= 1;
            base = base.wrapping_mul(base);
        }
        factor
    }
    #[inline(always)]
    fn word(&mut self, value: u64) {
        if value == 0 {
            self.zeros += 1;
        } else {
            self.flush();
            self.value = (self.value ^ value).wrapping_mul(DIAGNOSTIC_PRIME);
        }
    }
    fn finish(mut self) -> u64 {
        self.flush();
        self.value
    }
}
fn digest_message(h: &mut DiagnosticHasher, m: &Message) {
    // Exactly forty zero words compose to h * PRIME^40 modulo 2^64. This
    // branch is allowed only after checking every typed header/cap field and
    // all payload words; inactive/noncanonical data are never assumed empty.
    let header = m.sender_pid
        | m.tag
        | u64::from(m.len)
        | u64::from(m.cap_count)
        | m.reserved
        | m.caps[0].handle
        | m.caps[0].rights
        | m.caps[1].handle
        | m.caps[1].rights;
    if header == 0
        && m.data
            .chunks_exact(8)
            .all(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()) == 0)
    {
        h.zeros += MESSAGE_SIZE / 8;
        return;
    }

    fn word(h: &mut DiagnosticHasher, value: u64) {
        h.word(value);
    }
    word(h, m.sender_pid);
    word(h, m.tag);
    word(h, u64::from(m.len) | (u64::from(m.cap_count) << 32));
    word(h, m.reserved);
    for cap in &m.caps {
        word(h, cap.handle);
        word(h, cap.rights);
    }
    for bytes in m.data.chunks_exact(8) {
        word(h, u64::from_le_bytes(bytes.try_into().unwrap()));
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
struct LaunchObject {
    generation: u32,
    live: bool,
    retired: bool,
    open: bool,
    owner: u64,
    nodes: u32,
    image: u32,
}
impl LaunchObject {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        open: false,
        owner: 0,
        nodes: 0,
        image: 0,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntropyObject {
    generation: u32,
    live: bool,
    retired: bool,
    open: bool,
    owner: u64,
    nodes: u32,
}
impl EntropyObject {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        open: false,
        owner: 0,
        nodes: 0,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProcessObject {
    generation: u32,
    live: bool,
    retired: bool,
    open: bool,
    owner: u64,
    nodes: u32,
    child: u64,
    state: u8,
    code: u64,
}
impl ProcessObject {
    const EMPTY: Self = Self {
        generation: 1,
        live: false,
        retired: false,
        open: false,
        owner: 0,
        nodes: 0,
        child: 0,
        state: 0,
        code: 0,
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
    pub launch_objects: usize,
    pub retired_launch_objects: usize,
    pub process_objects: usize,
    pub retired_process_objects: usize,
    pub entropy_objects: usize,
    pub retired_entropy_objects: usize,
    pub network_objects: usize,
    pub retired_network_objects: usize,
    pub display_objects: usize,
    pub retired_display_objects: usize,
    pub input_objects: usize,
    pub retired_input_objects: usize,
    pub state_digest: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engine {
    processes: [Process; MAX_PROCESSES],
    nodes: [Node; MAX_NODES],
    endpoints: [Endpoint; MAX_ENDPOINTS],
    blocks: [BlockObject; MAX_BLOCK_DEVICES],
    launches: [LaunchObject; MAX_LAUNCH_OBJECTS],
    children: [ProcessObject; MAX_PROCESS_OBJECTS],
    entropy: [EntropyObject; MAX_ENTROPY_OBJECTS],
    network: [EntropyObject; MAX_NETWORK_OBJECTS],
    display: [EntropyObject; MAX_DISPLAY_OBJECTS],
    input: [EntropyObject; MAX_INPUT_OBJECTS],
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
            launches: [LaunchObject::EMPTY; MAX_LAUNCH_OBJECTS],
            children: [ProcessObject::EMPTY; MAX_PROCESS_OBJECTS],
            entropy: [EntropyObject::EMPTY; MAX_ENTROPY_OBJECTS],
            network: [EntropyObject::EMPTY; MAX_NETWORK_OBJECTS],
            display: [EntropyObject::EMPTY; MAX_DISPLAY_OBJECTS],
            input: [EntropyObject::EMPTY; MAX_INPUT_OBJECTS],
            last_pid: 0,
        }
    }
    /// Manifest-only: the holder may ask the kernel broker for raw entropy
    /// bytes. Like `Launch`, the type knows no derive/transfer/revoke.
    pub fn create_entropy(&mut self, owner: u64) -> Result<u64, Error> {
        let p = self.process(owner)?;
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .entropy
            .iter()
            .position(|e| !e.live && !e.retired)
            .ok_or(Error::Quota)?;
        let generation = self.entropy[object].generation;
        self.entropy[object] = EntropyObject {
            generation,
            live: true,
            open: true,
            owner,
            ..EntropyObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (ENTROPY_BASE + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            ENTROPY_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_entropy(&self, pid: u64, handle: u64) -> Result<(), Error> {
        let node = self.resolve(pid, handle)?;
        let i = self.nodes[node].object.index as usize;
        if !(ENTROPY_BASE..NETWORK_BASE).contains(&i) {
            return Err(Error::Type);
        }
        self.require(node, ENTROPY)
    }
    /// Manifest-only raw network authority. No process can manufacture, derive
    /// or transfer this handle; closing it and owner exit invalidate it.
    pub fn create_network(&mut self, owner: u64) -> Result<u64, Error> {
        let p = self.process(owner)?;
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .network
            .iter()
            .position(|e| !e.live && !e.retired)
            .ok_or(Error::Quota)?;
        let generation = self.network[object].generation;
        self.network[object] = EntropyObject {
            generation,
            live: true,
            open: true,
            owner,
            ..EntropyObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (NETWORK_BASE + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            NETWORK_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_network(&self, pid: u64, handle: u64) -> Result<u64, Error> {
        let node = self.resolve(pid, handle)?;
        let i = self.nodes[node].object.index as usize;
        if !(NETWORK_BASE..DISPLAY_BASE).contains(&i) {
            return Err(Error::Type);
        }
        self.require(node, NETWORK)?;
        let object = self.nodes[node].object;
        Ok(u64::from(object.index) | u64::from(object.generation) << 32)
    }
    /// Manifest-only display/input creation; neither type knows DERIVE,
    /// TRANSFER or REVOKE. One live object per physical source globally.
    pub fn create_display(&mut self, owner: u64) -> Result<u64, Error> {
        self.create_console_source(owner, true)
    }
    pub fn create_input(&mut self, owner: u64) -> Result<u64, Error> {
        self.create_console_source(owner, false)
    }
    fn create_console_source(&mut self, owner: u64, display: bool) -> Result<u64, Error> {
        let p = self.process(owner)?;
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let (objects, base, rights) = if display {
            (self.display.as_mut_slice(), DISPLAY_BASE, DISPLAY_RIGHTS)
        } else {
            (self.input.as_mut_slice(), INPUT_BASE, INPUT_RIGHTS)
        };
        let object = objects
            .iter()
            .position(|e| !e.live && !e.retired)
            .ok_or(Error::Quota)?;
        let generation = objects[object].generation;
        objects[object] = EntropyObject {
            generation,
            live: true,
            open: true,
            owner,
            ..EntropyObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (base + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            rights,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_display(&self, pid: u64, handle: u64) -> Result<u64, Error> {
        self.check_console_source(pid, handle, DISPLAY_BASE..INPUT_BASE, DISPLAY)
    }
    pub fn check_input(&self, pid: u64, handle: u64) -> Result<u64, Error> {
        self.check_console_source(pid, handle, INPUT_BASE..OBJECT_COUNT, INPUT)
    }
    fn check_console_source(
        &self,
        pid: u64,
        handle: u64,
        range: core::ops::Range<usize>,
        right: u64,
    ) -> Result<u64, Error> {
        let node = self.resolve(pid, handle)?;
        let object = self.nodes[node].object;
        if !range.contains(&(object.index as usize)) {
            return Err(Error::Type);
        }
        self.require(node, right)?;
        Ok(u64::from(object.index) | u64::from(object.generation) << 32)
    }
    /// Capacity probe for a spawner: true when one more process can register.
    pub fn can_register(&self) -> bool {
        self.processes.iter().any(|p| !p.live)
    }
    /// Free node count, so a caller can check all resources before committing.
    pub fn available_nodes(&self) -> usize {
        self.nodes.iter().filter(|n| !n.live && !n.retired).count()
    }
    /// Free local slots of a live process (0 for an unknown pid).
    pub fn available_slots(&self, pid: u64) -> usize {
        match self.process(pid) {
            Ok(p) => self.processes[p].slots[1..]
                .iter()
                .filter(|s| !s.retired && s.node == Id::NONE)
                .count(),
            Err(_) => 0,
        }
    }
    /// Read-only precheck that `bootstrap_delegate(from, handle, _, rights)`
    /// would pass its authority rules; it reserves nothing.
    pub fn check_delegation(&self, from: u64, handle: u64, rights: u64) -> Result<(), Error> {
        let n = self.resolve(from, handle)?;
        self.require(n, TRANSFER)?;
        if rights & !KNOWN_RIGHTS != 0 {
            return Err(Error::Invalid);
        }
        self.subset(n, rights, true)?;
        if self.nodes[n].depth >= MAX_DEPTH {
            return Err(Error::Quota);
        }
        Ok(())
    }
    /// Manifest-only: one `Launch` object for a built-in image. Apps cannot
    /// create, derive, transfer or revoke it; they can only use or close it.
    pub fn create_launch(&mut self, owner: u64, image: u32) -> Result<u64, Error> {
        let p = self.process(owner)?;
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .launches
            .iter()
            .position(|l| !l.live && !l.retired)
            .ok_or(Error::Quota)?;
        let generation = self.launches[object].generation;
        self.launches[object] = LaunchObject {
            generation,
            live: true,
            open: true,
            owner,
            image,
            ..LaunchObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (LAUNCH_BASE + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            LAUNCH_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_launch(&self, pid: u64, handle: u64) -> Result<u32, Error> {
        let node = self.resolve(pid, handle)?;
        let i = self.nodes[node].object.index as usize;
        if !(LAUNCH_BASE..PROCESS_BASE).contains(&i) {
            return Err(Error::Type);
        }
        self.require(node, LAUNCH)?;
        Ok(self.launches[i - LAUNCH_BASE].image)
    }
    /// True when one more `Process` object can be created.
    pub fn can_create_process(&self) -> bool {
        self.children.iter().any(|c| !c.live && !c.retired)
    }
    /// Kernel-only, after the child is registered: the spawner receives the
    /// only handle to a fresh `Process` object; the child itself holds none.
    pub fn create_process(&mut self, owner: u64, child: u64) -> Result<u64, Error> {
        let p = self.process(owner)?;
        self.process(child)?;
        if child == owner || self.children.iter().any(|c| c.live && c.child == child) {
            return Err(Error::Invalid);
        }
        let slot = self.slots(p, 1)?[0];
        let node = self.free_nodes(1)?[0];
        let object = self
            .children
            .iter()
            .position(|c| !c.live && !c.retired)
            .ok_or(Error::Quota)?;
        let generation = self.children[object].generation;
        self.children[object] = ProcessObject {
            generation,
            live: true,
            open: true,
            owner,
            child,
            state: PROCESS_RUNNING,
            code: 0,
            ..ProcessObject::EMPTY
        };
        self.make_node(
            node,
            Id {
                index: (PROCESS_BASE + object) as u16,
                generation,
            },
            Id::NONE,
            owner,
            PROCESS_RIGHTS,
            0,
        );
        Ok(self.attach(p, slot, self.node_id(node)))
    }
    pub fn check_process(
        &self,
        pid: u64,
        handle: u64,
        required: u64,
    ) -> Result<ProcessView, Error> {
        let node = self.resolve(pid, handle)?;
        let i = self.nodes[node].object.index as usize;
        if !(PROCESS_BASE..ENTROPY_BASE).contains(&i) {
            return Err(Error::Type);
        }
        self.require(node, required)?;
        let c = self.children[i - PROCESS_BASE];
        Ok(ProcessView {
            child: c.child,
            state: c.state,
            code: c.code,
        })
    }
    /// Record how a child ended. Call before `exit(child)`. Returns whether a
    /// running `Process` object named this child.
    pub fn process_died(&mut self, child: u64, state: u8, code: u64) -> bool {
        if !matches!(state, PROCESS_EXITED | PROCESS_FAULTED | PROCESS_KILLED) {
            return false;
        }
        let mut found = false;
        for c in &mut self.children {
            if c.live && c.child == child && c.state == PROCESS_RUNNING {
                c.state = state;
                c.code = code;
                found = true;
            }
        }
        self.maintain();
        found
    }
    /// Running children whose spawner is no longer live; the kernel must end
    /// them (invariant 5 of K05). Returns the count written into `out`.
    pub fn orphans(&self, out: &mut [u64; MAX_PROCESS_OBJECTS]) -> usize {
        let mut n = 0;
        for c in &self.children {
            if c.live && c.state == PROCESS_RUNNING && self.process(c.owner).is_err() {
                out[n] = c.child;
                n += 1;
            }
        }
        n
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
        self.create_block_device_with_rights(owner, scope, BLOCK_READ_RIGHTS)
    }
    /// Manifest-only authority creation. The legacy constructor remains readonly.
    /// FLUSH explicitly authorizes a device-wide cache effect, not a sector write.
    pub fn create_block_device_with_rights(
        &mut self,
        owner: u64,
        scope: BlockScope,
        rights: u64,
    ) -> Result<u64, Error> {
        let p = self.process(owner)?;
        scope.validate()?;
        if rights & !BLOCK_RIGHTS != 0 {
            return Err(Error::Invalid);
        }
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
            rights,
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
        if !(MAX_ENDPOINTS..LAUNCH_BASE).contains(&(object.index as usize)) {
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
        if rights & !KNOWN_RIGHTS != 0 {
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
        if rights & !KNOWN_RIGHTS != 0 {
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
            if message.caps[i].rights & !KNOWN_RIGHTS != 0 {
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
        for l in &mut self.launches {
            if l.live && l.owner == pid {
                l.open = false;
            }
        }
        for c in &mut self.children {
            if c.live && c.owner == pid {
                c.open = false;
            }
        }
        for e in self
            .entropy
            .iter_mut()
            .chain(self.network.iter_mut())
            .chain(self.display.iter_mut())
            .chain(self.input.iter_mut())
        {
            if e.live && e.owner == pid {
                e.open = false;
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
        } else if i < LAUNCH_BASE {
            let b = self.blocks[i - MAX_ENDPOINTS];
            b.live && b.generation == id.generation
        } else if i < PROCESS_BASE {
            let l = self.launches[i - LAUNCH_BASE];
            l.live && l.generation == id.generation
        } else if i < ENTROPY_BASE {
            let c = self.children[i - PROCESS_BASE];
            c.live && c.generation == id.generation
        } else if i < NETWORK_BASE {
            let e = self.entropy[i - ENTROPY_BASE];
            e.live && e.generation == id.generation
        } else if i < DISPLAY_BASE {
            let n = self.network[i - NETWORK_BASE];
            n.live && n.generation == id.generation
        } else if i < INPUT_BASE {
            let n = self.display[i - DISPLAY_BASE];
            n.live && n.generation == id.generation
        } else if i < OBJECT_COUNT {
            let n = self.input[i - INPUT_BASE];
            n.live && n.generation == id.generation
        } else {
            false
        }
    }
    fn object_open(&self, id: Id) -> bool {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            self.endpoints[i].open
        } else if i < LAUNCH_BASE {
            self.blocks[i - MAX_ENDPOINTS].open
        } else if i < PROCESS_BASE {
            self.launches[i - LAUNCH_BASE].open
        } else if i < ENTROPY_BASE {
            self.children[i - PROCESS_BASE].open
        } else if i < NETWORK_BASE {
            self.entropy[i - ENTROPY_BASE].open
        } else if i < DISPLAY_BASE {
            self.network[i - NETWORK_BASE].open
        } else if i < INPUT_BASE {
            self.display[i - DISPLAY_BASE].open
        } else {
            self.input[i - INPUT_BASE].open
        }
    }
    fn allowed_rights(&self, id: Id) -> u64 {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            ALL_RIGHTS
        } else if i < LAUNCH_BASE {
            BLOCK_RIGHTS
        } else if i < PROCESS_BASE {
            LAUNCH_RIGHTS
        } else if i < ENTROPY_BASE {
            PROCESS_RIGHTS
        } else if i < NETWORK_BASE {
            ENTROPY_RIGHTS
        } else if i < DISPLAY_BASE {
            NETWORK_RIGHTS
        } else if i < INPUT_BASE {
            DISPLAY_RIGHTS
        } else {
            INPUT_RIGHTS
        }
    }
    fn object_refs(&mut self, id: Id) -> &mut u32 {
        let i = id.index as usize;
        if i < MAX_ENDPOINTS {
            &mut self.endpoints[i].nodes
        } else if i < LAUNCH_BASE {
            &mut self.blocks[i - MAX_ENDPOINTS].nodes
        } else if i < PROCESS_BASE {
            &mut self.launches[i - LAUNCH_BASE].nodes
        } else if i < ENTROPY_BASE {
            &mut self.children[i - PROCESS_BASE].nodes
        } else if i < NETWORK_BASE {
            &mut self.entropy[i - ENTROPY_BASE].nodes
        } else if i < DISPLAY_BASE {
            &mut self.network[i - NETWORK_BASE].nodes
        } else if i < INPUT_BASE {
            &mut self.display[i - DISPLAY_BASE].nodes
        } else {
            &mut self.input[i - INPUT_BASE].nodes
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
        for l in &mut self.launches {
            if l.live && l.nodes == 0 {
                let (generation, retired) = advance(l.generation);
                *l = LaunchObject {
                    generation,
                    retired,
                    ..LaunchObject::EMPTY
                };
            }
        }
        // A running child keeps its object alive even without handles: the
        // kernel still needs the spawner link to end orphans (K05 invariant 5).
        for c in &mut self.children {
            if c.live && c.nodes == 0 && c.state != PROCESS_RUNNING {
                let (generation, retired) = advance(c.generation);
                *c = ProcessObject {
                    generation,
                    retired,
                    ..ProcessObject::EMPTY
                };
            }
        }
        for e in self
            .entropy
            .iter_mut()
            .chain(self.network.iter_mut())
            .chain(self.display.iter_mut())
            .chain(self.input.iter_mut())
        {
            if e.live && e.nodes == 0 {
                let (generation, retired) = advance(e.generation);
                *e = EntropyObject {
                    generation,
                    retired,
                    ..EntropyObject::EMPTY
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
            launch_objects: 0,
            retired_launch_objects: 0,
            process_objects: 0,
            retired_process_objects: 0,
            entropy_objects: 0,
            retired_entropy_objects: 0,
            network_objects: 0,
            retired_network_objects: 0,
            display_objects: 0,
            retired_display_objects: 0,
            input_objects: 0,
            retired_input_objects: 0,
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
        for l in &self.launches {
            s.launch_objects += usize::from(l.live);
            s.retired_launch_objects += usize::from(l.retired);
        }
        for c in &self.children {
            s.process_objects += usize::from(c.live);
            s.retired_process_objects += usize::from(c.retired);
        }
        for e in &self.entropy {
            s.entropy_objects += usize::from(e.live);
            s.retired_entropy_objects += usize::from(e.retired);
        }
        for n in &self.network {
            s.network_objects += usize::from(n.live);
            s.retired_network_objects += usize::from(n.retired);
        }
        for e in &self.display {
            s.display_objects += usize::from(e.live);
            s.retired_display_objects += usize::from(e.retired);
        }
        for e in &self.input {
            s.input_objects += usize::from(e.live);
            s.retired_input_objects += usize::from(e.retired);
        }
        s
    }
    /// Diagnostic fingerprint only, not cryptography or proof against collisions.
    pub fn state_digest(&self) -> u64 {
        let mut h = DiagnosticHasher::new(0xcbf29ce484222325);
        fn add(h: &mut DiagnosticHasher, v: u64) {
            // Fixed typed word boundaries: this is a fast diagnostic mixer,
            // never an authentication/integrity primitive or a crypto hash.
            h.word(v);
        }
        fn id(h: &mut DiagnosticHasher, x: Id) {
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
                digest_message(&mut h, &q.message);
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
        for l in &self.launches {
            for x in [
                u64::from(l.generation),
                u64::from(l.live),
                u64::from(l.retired),
                u64::from(l.open),
                l.owner,
                u64::from(l.nodes),
                u64::from(l.image),
            ] {
                add(&mut h, x);
            }
        }
        for c in &self.children {
            for x in [
                u64::from(c.generation),
                u64::from(c.live),
                u64::from(c.retired),
                u64::from(c.open),
                c.owner,
                u64::from(c.nodes),
                c.child,
                u64::from(c.state),
                c.code,
            ] {
                add(&mut h, x);
            }
        }
        for e in self
            .entropy
            .iter()
            .chain(self.network.iter())
            .chain(self.display.iter())
            .chain(self.input.iter())
        {
            for x in [
                u64::from(e.generation),
                u64::from(e.live),
                u64::from(e.retired),
                u64::from(e.open),
                e.owner,
                u64::from(e.nodes),
            ] {
                add(&mut h, x);
            }
        }
        h.finish()
    }
    #[cfg(test)]
    fn state_digest_reference(&self) -> u64 {
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
        for l in &self.launches {
            for x in [
                u64::from(l.generation),
                u64::from(l.live),
                u64::from(l.retired),
                u64::from(l.open),
                l.owner,
                u64::from(l.nodes),
                u64::from(l.image),
            ] {
                add(&mut h, x);
            }
        }
        for c in &self.children {
            for x in [
                u64::from(c.generation),
                u64::from(c.live),
                u64::from(c.retired),
                u64::from(c.open),
                c.owner,
                u64::from(c.nodes),
                c.child,
                u64::from(c.state),
                c.code,
            ] {
                add(&mut h, x);
            }
        }
        for e in self
            .entropy
            .iter()
            .chain(self.network.iter())
            .chain(self.display.iter())
            .chain(self.input.iter())
        {
            for x in [
                u64::from(e.generation),
                u64::from(e.live),
                u64::from(e.retired),
                u64::from(e.open),
                e.owner,
                u64::from(e.nodes),
            ] {
                add(&mut h, x);
            }
        }
        h
    }
    /// Full structural/refcount check for observers/tests, not a syscall hook.
    pub fn check_invariants(&self) -> bool {
        let mut refs = [0u32; MAX_NODES];
        let mut objects = [0u32; OBJECT_COUNT];
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
        for (i, l) in self.launches.iter().enumerate() {
            if l.generation == 0
                || l.retired && (l.live || l.generation != u32::MAX)
                || l.nodes != objects[LAUNCH_BASE + i]
                || !l.live && (l.open || l.nodes != 0)
            {
                return false;
            }
            if l.live && (l.nodes == 0 || l.owner == 0) {
                return false;
            }
            if l.open && self.process(l.owner).is_err() {
                return false;
            }
        }
        for (i, c) in self.children.iter().enumerate() {
            if c.generation == 0
                || c.retired && (c.live || c.generation != u32::MAX)
                || c.nodes != objects[PROCESS_BASE + i]
                || !c.live && (c.open || c.nodes != 0 || c.state != 0)
            {
                return false;
            }
            if !c.live {
                continue;
            }
            if c.owner == 0
                || c.child == 0
                || c.child == c.owner
                || !(PROCESS_RUNNING..=PROCESS_KILLED).contains(&c.state)
                || c.nodes == 0 && c.state != PROCESS_RUNNING
                || c.state == PROCESS_RUNNING && self.process(c.child).is_err()
                || c.open && self.process(c.owner).is_err()
                || self
                    .children
                    .iter()
                    .filter(|x| x.live && x.child == c.child)
                    .count()
                    != 1
            {
                return false;
            }
        }
        for (base, device) in [
            (ENTROPY_BASE, self.entropy.as_slice()),
            (NETWORK_BASE, self.network.as_slice()),
            (DISPLAY_BASE, self.display.as_slice()),
            (INPUT_BASE, self.input.as_slice()),
        ] {
            for (i, e) in device.iter().enumerate() {
                if e.generation == 0
                    || e.retired && (e.live || e.generation != u32::MAX)
                    || e.nodes != objects[base + i]
                    || !e.live && (e.open || e.nodes != 0)
                    || e.live && (e.nodes == 0 || e.owner == 0)
                    || e.open && self.process(e.owner).is_err()
                {
                    return false;
                }
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

#[cfg(test)]
mod digest_differential {
    extern crate std;
    use super::*;
    use std::{hint::black_box, thread, time::Instant, vec::Vec};
    fn run(f: impl FnOnce() + Send + 'static) {
        thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap();
    }
    fn same(e: &Engine) {
        assert_eq!(e.state_digest(), e.state_digest_reference());
        assert_eq!(e.stats().state_digest, e.state_digest_reference());
    }
    fn random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }
    fn raw_message(bytes: &[u8; MESSAGE_SIZE]) -> Message {
        let mut m = Message::new();
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
        m
    }
    fn reference_message(mut h: u64, message: &Message) -> u64 {
        for bytes in message.encode().chunks_exact(8) {
            h = (h ^ u64::from_le_bytes(bytes.try_into().unwrap())).wrapping_mul(0x100000001b3);
        }
        h
    }
    #[test]
    fn every_message_wire_bit_including_invalid_and_trailing_bytes_is_preserved() {
        for seed in [0, 1, u64::MAX, 0xcbf29ce484222325, 0x123456789abcdef0] {
            let mut empty = DiagnosticHasher::new(seed);
            digest_message(&mut empty, &Message::new());
            assert_eq!(empty.finish(), reference_message(seed, &Message::new()));
            for byte in 0..MESSAGE_SIZE {
                for bit in 0..8 {
                    let mut wire = [0; MESSAGE_SIZE];
                    wire[byte] = 1 << bit;
                    let message = raw_message(&wire);
                    assert_eq!(message.encode(), wire);
                    let mut h = DiagnosticHasher::new(seed);
                    digest_message(&mut h, &message);
                    assert_eq!(
                        h.finish(),
                        reference_message(seed, &message),
                        "byte={byte} bit={bit}"
                    );
                }
            }
        }
        let mut seed = 0x5a8279993c6ef372;
        for _ in 0..256 {
            let mut wire = [0; MESSAGE_SIZE];
            for bytes in wire.chunks_exact_mut(8) {
                bytes.copy_from_slice(&random(&mut seed).to_le_bytes());
            }
            let message = raw_message(&wire);
            let mut h = DiagnosticHasher::new(seed);
            digest_message(&mut h, &message);
            assert_eq!(h.finish(), reference_message(seed, &message));
        }
    }
    #[test]
    fn zero_runs_compose_exactly_across_boundaries_for_all_table_lengths() {
        for seed in [0, 1, u64::MAX, 0xcbf29ce484222325, 0x8000000000000000] {
            for count in [
                0, 1, 2, 7, 8, 31, 32, 39, 40, 41, 63, 64, 65, 66, 127, 128, 255, 1024, 4096,
                65535, 74669,
            ] {
                let mut h = DiagnosticHasher::new(seed);
                h.zeros = count;
                let mut reference = seed;
                for _ in 0..count {
                    reference = reference.wrapping_mul(0x100000001b3);
                }
                assert_eq!(h.finish(), reference, "count={count}");
            }
        }
        let mut seed = 17;
        let mut h = DiagnosticHasher::new(seed);
        let mut reference = seed;
        for _ in 0..10000 {
            let value = if random(&mut seed) & 15 < 13 { 0 } else { seed };
            h.word(value);
            reference = (reference ^ value).wrapping_mul(0x100000001b3);
        }
        assert_eq!(h.finish(), reference);
    }
    #[test]
    fn real_lifecycle_failures_revoke_transfer_and_queue_wrap_match_reference() {
        run(|| {
            let mut e = Engine::new();
            same(&e);
            for pid in 1..=7 {
                e.register(pid).unwrap();
                same(&e);
            }
            let registry = e.create_endpoint(1).unwrap();
            let delegated = e.bootstrap_delegate(1, registry, 2, SEND).unwrap();
            same(&e);
            let root = e.create_endpoint(2).unwrap();
            let child = e.derive(2, root, SEND | DERIVE | TRANSFER).unwrap();
            let grandchild = e.derive(2, child, SEND).unwrap();
            same(&e);
            for round in 0..4 {
                for i in 0..QUEUE_CAPACITY {
                    let mut message = Message::new();
                    message.tag = (round * QUEUE_CAPACITY + i) as u64;
                    message.len = 3;
                    message.data[..3].copy_from_slice(&[1, 2, 3]);
                    e.send(2, delegated, &message).unwrap();
                    same(&e);
                }
                let before = e.clone();
                assert_eq!(e.send(2, delegated, &Message::new()), Err(Error::Again));
                assert!(e == before);
                same(&e);
                for _ in 0..QUEUE_CAPACITY {
                    e.receive(1, registry).unwrap();
                    same(&e);
                }
                assert!(e.check_invariants());
            }
            let mut m = Message::new();
            m.cap_count = 1;
            m.caps[0] = Cap {
                handle: child,
                rights: SEND,
            };
            e.send(2, delegated, &m).unwrap();
            same(&e);
            let message = e.receive(1, registry).unwrap();
            same(&e);
            e.close(1, message.caps[0].handle).unwrap();
            same(&e);
            e.revoke(2, root).unwrap();
            same(&e);
            assert_eq!(e.send(2, grandchild, &Message::new()), Err(Error::Revoked));
            same(&e);
            e.close(2, grandchild).unwrap();
            e.close(2, child).unwrap();
            e.close(2, root).unwrap();
            same(&e);
            let block = e
                .create_block_device_with_rights(
                    3,
                    BlockScope {
                        device: 9,
                        first_sector: 64,
                        sector_count: 100,
                    },
                    BLOCK_RIGHTS,
                )
                .unwrap();
            let entropy = e.create_entropy(4).unwrap();
            let network = e.create_network(5).unwrap();
            let display = e.create_display(6).unwrap();
            let input = e.create_input(7).unwrap();
            let launch = e.create_launch(1, 3).unwrap();
            let process = e.create_process(1, 7).unwrap();
            same(&e);
            let before = e.clone();
            assert_eq!(e.derive(4, entropy, ENTROPY), Err(Error::Invalid));
            assert!(e == before);
            same(&e);
            assert!(e.process_died(7, PROCESS_FAULTED, 6));
            same(&e);
            for (pid, handle) in [
                (3, block),
                (4, entropy),
                (5, network),
                (6, display),
                (7, input),
                (1, launch),
                (1, process),
            ] {
                e.close(pid, handle).unwrap();
                same(&e);
            }
            for pid in (1..=7).rev() {
                e.exit(pid).unwrap();
                same(&e);
            }
            assert!(e.check_invariants());
        });
    }
    fn id(seed: &mut u64) -> Id {
        Id {
            index: random(seed) as u16,
            generation: random(seed) as u32,
        }
    }
    fn bit(seed: &mut u64) -> bool {
        random(seed) & 1 != 0
    }
    #[test]
    fn dense_private_corruptions_cover_every_table_field_and_inactive_queue() {
        run(|| {
            for initial in [1, 0x123456789abcdef, 0xfedcba9876543210] {
                let mut seed = initial;
                let mut e = Engine::new();
                e.last_pid = random(&mut seed);
                for p in &mut e.processes {
                    p.pid = random(&mut seed);
                    p.live = bit(&mut seed);
                    for s in &mut p.slots {
                        s.generation = random(&mut seed) as u32;
                        s.retired = bit(&mut seed);
                        s.node = id(&mut seed);
                    }
                }
                for n in &mut e.nodes {
                    n.generation = random(&mut seed) as u32;
                    n.live = bit(&mut seed);
                    n.retired = bit(&mut seed);
                    n.revoked = bit(&mut seed);
                    n.object = id(&mut seed);
                    n.parent = id(&mut seed);
                    n.issuer = random(&mut seed);
                    n.rights = random(&mut seed);
                    n.refs = random(&mut seed) as u32;
                    n.depth = random(&mut seed) as u8;
                }
                for ep in &mut e.endpoints {
                    ep.generation = random(&mut seed) as u32;
                    ep.live = bit(&mut seed);
                    ep.retired = bit(&mut seed);
                    ep.open = bit(&mut seed);
                    ep.owner = random(&mut seed);
                    ep.nodes = random(&mut seed) as u32;
                    ep.head = random(&mut seed) as u8;
                    ep.len = random(&mut seed) as u8;
                    for q in &mut ep.queue {
                        let mut bytes = [0; MESSAGE_SIZE];
                        for word in bytes.chunks_exact_mut(8) {
                            word.copy_from_slice(&random(&mut seed).to_le_bytes());
                        }
                        q.message = raw_message(&bytes);
                        q.caps = [id(&mut seed), id(&mut seed)];
                    }
                }
                for b in &mut e.blocks {
                    b.generation = random(&mut seed) as u32;
                    b.live = bit(&mut seed);
                    b.retired = bit(&mut seed);
                    b.open = bit(&mut seed);
                    b.owner = random(&mut seed);
                    b.nodes = random(&mut seed) as u32;
                    b.scope = BlockScope {
                        device: random(&mut seed) as u32,
                        first_sector: random(&mut seed),
                        sector_count: random(&mut seed),
                    };
                }
                for l in &mut e.launches {
                    l.generation = random(&mut seed) as u32;
                    l.live = bit(&mut seed);
                    l.retired = bit(&mut seed);
                    l.open = bit(&mut seed);
                    l.owner = random(&mut seed);
                    l.nodes = random(&mut seed) as u32;
                    l.image = random(&mut seed) as u32;
                }
                for c in &mut e.children {
                    c.generation = random(&mut seed) as u32;
                    c.live = bit(&mut seed);
                    c.retired = bit(&mut seed);
                    c.open = bit(&mut seed);
                    c.owner = random(&mut seed);
                    c.nodes = random(&mut seed) as u32;
                    c.child = random(&mut seed);
                    c.state = random(&mut seed) as u8;
                    c.code = random(&mut seed);
                }
                for x in e
                    .entropy
                    .iter_mut()
                    .chain(e.network.iter_mut())
                    .chain(e.display.iter_mut())
                    .chain(e.input.iter_mut())
                {
                    x.generation = random(&mut seed) as u32;
                    x.live = bit(&mut seed);
                    x.retired = bit(&mut seed);
                    x.open = bit(&mut seed);
                    x.owner = random(&mut seed);
                    x.nodes = random(&mut seed) as u32;
                }
                // Deliberately do not call authority methods/invariant walkers on
                // corrupt indices/counts: the diagnostic itself must hash them.
                same(&e);
            }
        });
    }
    #[test]
    fn zero_corrupt_states_and_single_inactive_trailing_bytes_match_reference() {
        run(|| {
            let mut e = Engine::new();
            e.last_pid = 0;
            for p in &mut e.processes {
                p.pid = 0;
                p.live = false;
                for s in &mut p.slots {
                    s.generation = 0;
                    s.retired = false;
                    s.node = Id {
                        index: 0,
                        generation: 0,
                    };
                }
            }
            for n in &mut e.nodes {
                *n = Node {
                    generation: 0,
                    live: false,
                    retired: false,
                    revoked: false,
                    object: Id {
                        index: 0,
                        generation: 0,
                    },
                    parent: Id {
                        index: 0,
                        generation: 0,
                    },
                    issuer: 0,
                    rights: 0,
                    refs: 0,
                    depth: 0,
                };
            }
            for ep in &mut e.endpoints {
                ep.generation = 0;
                for q in &mut ep.queue {
                    q.caps = [Id {
                        index: 0,
                        generation: 0,
                    }; 2];
                }
            }
            for b in &mut e.blocks {
                b.generation = 0;
            }
            for l in &mut e.launches {
                l.generation = 0;
            }
            for c in &mut e.children {
                c.generation = 0;
            }
            for x in e
                .entropy
                .iter_mut()
                .chain(e.network.iter_mut())
                .chain(e.display.iter_mut())
                .chain(e.input.iter_mut())
            {
                x.generation = 0;
            }
            same(&e);
            let baseline = e.state_digest();
            for ep in [0, 31, 63] {
                for q in [0, 7, 15] {
                    for at in [0, 7, 8, 127, 128, 247, 248, 255] {
                        e.endpoints[ep].queue[q].message.data[at] = 0x80;
                        same(&e);
                        assert_ne!(e.state_digest(), baseline);
                        e.endpoints[ep].queue[q].message.data[at] = 0;
                    }
                }
            }
            let snapshot = e.clone();
            for set in 0..18 {
                e = snapshot.clone();
                match set {
                    0 => e.processes[15].slots[255].generation = u32::MAX,
                    1 => e.processes[0].slots[0].retired = true,
                    2 => e.nodes[1023].generation = u32::MAX,
                    3 => e.nodes[512].depth = u8::MAX,
                    4 => e.nodes[0].object = Id::NONE,
                    5 => e.nodes[0].parent = Id::NONE,
                    6 => e.endpoints[63].head = u8::MAX,
                    7 => e.endpoints[31].len = u8::MAX,
                    8 => e.endpoints[0].queue[0].message.reserved = u64::MAX,
                    9 => e.endpoints[63].queue[15].message.len = u32::MAX,
                    10 => e.endpoints[63].queue[15].message.cap_count = u32::MAX,
                    11 => e.endpoints[63].queue[15].caps[1] = Id::NONE,
                    12 => e.blocks[3].scope.sector_count = u64::MAX,
                    13 => e.launches[7].image = u32::MAX,
                    14 => e.children[15].state = u8::MAX,
                    15 => e.entropy[3].owner = u64::MAX,
                    16 => e.display[0].retired = true,
                    _ => e.input[0].nodes = u32::MAX,
                };
                same(&e);
                assert_ne!(e.state_digest(), baseline);
            }
        });
    }
    #[test]
    #[ignore = "host cost/ratio only; invoke explicitly, never native performance evidence"]
    fn host_microbenchmark_reference_and_optimized() {
        run(|| {
            let mut e = Engine::new();
            for pid in 1..=7 {
                e.register(pid).unwrap();
                for _ in 0..3 {
                    e.create_endpoint(pid).unwrap();
                }
            }
            same(&e);
            let iterations = 1000u32;
            for reference in [true, false] {
                let mut samples = Vec::new();
                let mut digest = 0;
                for _ in 0..7 {
                    let start = Instant::now();
                    for _ in 0..iterations {
                        digest = if reference {
                            black_box(black_box(&e).state_digest_reference())
                        } else {
                            black_box(black_box(&e).state_digest())
                        };
                    }
                    samples.push(start.elapsed().as_nanos() / u128::from(iterations));
                }
                samples.sort();
                std::println!(
                    "reference={reference} median_ns={} digest={digest:016x} samples={samples:?}",
                    samples[3]
                );
            }
        });
    }
}
