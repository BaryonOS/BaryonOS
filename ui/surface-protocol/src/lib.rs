//! SUR1: bounded assigned-surface transport. This crate owns no capabilities
//! and performs no device I/O. Sender PID must come from kernel IPC metadata.
#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "client")]
pub mod client;

pub const TAG: u64 = 0x5355_5246_4143_0001;
pub const VERSION: u16 = 1;
pub const HEADER: usize = 48;
pub const CHUNK: usize = 208;
pub const TILE_BYTES: usize = 1024;
pub const MAX_MESSAGE: usize = HEADER + CHUNK;
pub const REPLY: u16 = 0x8000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Operation {
    Info = 1,
    Begin = 2,
    Chunk = 3,
    Commit = 4,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Malformed = 1,
    Access = 2,
    Epoch = 3,
    Replay = 4,
    Busy = 5,
    Grid = 6,
    Length = 7,
    Pixels = 8,
    Timeout = 9,
    Clock = 10,
    Closed = 11,
    Token = 12,
    Blit = 13,
    Exhausted = 14,
}

fn u16at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn u32at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn u64at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub operation: Operation,
    pub request: u64,
    pub epoch: u64,
    pub frame: u64,
    pub tile: u64,
    pub arg0: u32,
    pub arg1: u32,
}
impl Header {
    pub fn encode(self) -> [u8; HEADER] {
        let mut bytes = [0; HEADER];
        bytes[..4].copy_from_slice(b"SUR1");
        bytes[4..6].copy_from_slice(&VERSION.to_le_bytes());
        bytes[6..8].copy_from_slice(&(self.operation as u16).to_le_bytes());
        for (at, value) in [
            (8, self.request),
            (16, self.epoch),
            (24, self.frame),
            (32, self.tile),
        ] {
            bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes[40..44].copy_from_slice(&self.arg0.to_le_bytes());
        bytes[44..48].copy_from_slice(&self.arg1.to_le_bytes());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER
            || bytes.len() > MAX_MESSAGE
            || &bytes[..4] != b"SUR1"
            || u16at(bytes, 4) != VERSION
        {
            return Err(Error::Malformed);
        }
        let operation = match u16at(bytes, 6) {
            1 => Operation::Info,
            2 => Operation::Begin,
            3 => Operation::Chunk,
            4 => Operation::Commit,
            _ => return Err(Error::Malformed),
        };
        let header = Self {
            operation,
            request: u64at(bytes, 8),
            epoch: u64at(bytes, 16),
            frame: u64at(bytes, 24),
            tile: u64at(bytes, 32),
            arg0: u32at(bytes, 40),
            arg1: u32at(bytes, 44),
        };
        if header.request == 0 || header.epoch == 0 {
            return Err(Error::Malformed);
        }
        let valid = match operation {
            Operation::Info => {
                bytes.len() == HEADER
                    && header.frame == 0
                    && header.tile == 0
                    && header.arg0 == 0
                    && header.arg1 == 0
            }
            Operation::Begin => bytes.len() == HEADER + 8 && header.frame != 0,
            Operation::Chunk => {
                header.frame != 0
                    && header.arg1 != 0
                    && header.arg1 as usize <= CHUNK
                    && bytes.len() == HEADER + header.arg1 as usize
            }
            Operation::Commit => {
                bytes.len() == HEADER && header.frame != 0 && header.arg0 == 0 && header.arg1 == 0
            }
        };
        if !valid {
            return Err(Error::Malformed);
        }
        Ok(header)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u16,
    pub height: u16,
}
impl Rectangle {
    pub fn bytes(self) -> usize {
        self.width as usize * self.height as usize * 4
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Geometry {
    pub width: u32,
    pub height: u32,
    pub assigned: Rectangle,
    pub epoch: u64,
}
impl Geometry {
    pub fn validate(self) -> Result<(), Error> {
        let r = self.assigned;
        if self.width == 0
            || self.height == 0
            || self.width > 1920
            || self.height > 1080
            || self.epoch == 0
            || r.width == 0
            || r.height == 0
            || u64::from(r.x) + u64::from(r.width) > u64::from(self.width)
            || u64::from(r.y) + u64::from(r.height) > u64::from(self.height)
        {
            return Err(Error::Grid);
        }
        Ok(())
    }
    pub fn columns(self) -> u64 {
        u64::from(self.assigned.width).div_ceil(32)
    }
    pub fn tiles(self) -> u64 {
        self.columns() * u64::from(self.assigned.height).div_ceil(8)
    }
    pub fn rectangle(self, index: u64) -> Result<Rectangle, Error> {
        self.validate()?;
        if index >= self.tiles() {
            return Err(Error::Grid);
        }
        let x = (index % self.columns()) as u32 * 32;
        let y = (index / self.columns()) as u32 * 8;
        Ok(Rectangle {
            x,
            y,
            width: 32.min(u32::from(self.assigned.width) - x) as u16,
            height: 8.min(u32::from(self.assigned.height) - y) as u16,
        })
    }
    fn body(self) -> [u8; 32] {
        let mut out = [0; 32];
        for (at, value) in [
            (0, self.width),
            (4, self.height),
            (8, self.assigned.x),
            (12, self.assigned.y),
            (16, u32::from(self.assigned.width)),
            (20, u32::from(self.assigned.height)),
        ] {
            out[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        out[24..26].copy_from_slice(&32u16.to_le_bytes());
        out[26..28].copy_from_slice(&8u16.to_le_bytes());
        out[28..32].copy_from_slice(&1u32.to_le_bytes());
        out
    }
}

/// Received bytes only; native callers must separately authenticate the IPC
/// sender/tag/no-caps and bind expected request/epoch/frame/tile before use.
pub struct Reply {
    bytes: [u8; 80],
    length: usize,
}
impl Reply {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    pub fn decode(bytes: &[u8], expected: Header) -> Result<Self, Error> {
        if !matches!(expected.operation, Operation::Info | Operation::Commit) {
            return Err(Error::Malformed);
        }
        let length = if expected.operation == Operation::Info {
            80
        } else {
            HEADER
        };
        if bytes.len() != length
            || &bytes[..4] != b"SUR1"
            || u16at(bytes, 4) != VERSION
            || u16at(bytes, 6) != (expected.operation as u16 | REPLY)
            || u64at(bytes, 8) != expected.request
            || u64at(bytes, 16) != expected.epoch
            || u64at(bytes, 24) != expected.frame
            || u64at(bytes, 32) != expected.tile
        {
            return Err(Error::Malformed);
        }
        let status = u32at(bytes, 40);
        if status > 14 {
            return Err(Error::Malformed);
        }
        if status != 0 && (u32at(bytes, 44) != 0 || bytes[HEADER..].iter().any(|b| *b != 0)) {
            return Err(Error::Malformed);
        }
        let mut out = [0; 80];
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            bytes: out,
            length: bytes.len(),
        })
    }
    pub fn status(&self) -> Result<(), Error> {
        match u32at(&self.bytes, 40) {
            0 => Ok(()),
            1 => Err(Error::Malformed),
            2 => Err(Error::Access),
            3 => Err(Error::Epoch),
            4 => Err(Error::Replay),
            5 => Err(Error::Busy),
            6 => Err(Error::Grid),
            7 => Err(Error::Length),
            8 => Err(Error::Pixels),
            9 => Err(Error::Timeout),
            10 => Err(Error::Clock),
            11 => Err(Error::Closed),
            12 => Err(Error::Token),
            13 => Err(Error::Blit),
            _ => Err(Error::Exhausted),
        }
    }
    pub fn pixels(&self) -> u32 {
        u32at(&self.bytes, 44)
    }
    pub fn geometry(&self) -> Result<Geometry, Error> {
        self.status()?;
        if self.length != 80
            || u16at(&self.bytes, 6) != (Operation::Info as u16 | REPLY)
            || u32at(&self.bytes, 44) != 0
            || u16at(&self.bytes, 72) != 32
            || u16at(&self.bytes, 74) != 8
            || u32at(&self.bytes, 76) != 1
        {
            return Err(Error::Malformed);
        }
        let width = u32at(&self.bytes, 64);
        let height = u32at(&self.bytes, 68);
        if width > u16::MAX as u32 || height > u16::MAX as u32 {
            return Err(Error::Grid);
        }
        let geometry = Geometry {
            width: u32at(&self.bytes, 48),
            height: u32at(&self.bytes, 52),
            assigned: Rectangle {
                x: u32at(&self.bytes, 56),
                y: u32at(&self.bytes, 60),
                width: width as u16,
                height: height as u16,
            },
            epoch: u64at(&self.bytes, 16),
        };
        geometry.validate()?;
        Ok(geometry)
    }
}

fn reply(header: Header, result: Result<u32, Error>, geometry: Option<Geometry>) -> Reply {
    let mut bytes = [0; 80];
    bytes[..HEADER].copy_from_slice(&header.encode());
    bytes[6..8].copy_from_slice(&(header.operation as u16 | REPLY).to_le_bytes());
    let (status, pixels) = match result {
        Ok(pixels) => (0, pixels),
        Err(e) => (e as u32, 0),
    };
    bytes[40..44].copy_from_slice(&status.to_le_bytes());
    bytes[44..48].copy_from_slice(&pixels.to_le_bytes());
    let length = if header.operation == Operation::Info {
        80
    } else {
        HEADER
    };
    if status == 0 {
        if let Some(g) = geometry {
            bytes[48..80].copy_from_slice(&g.body());
        }
    }
    Reply { bytes, length }
}

#[derive(Clone, Copy)]
struct Stage {
    header: Header,
    rectangle: Rectangle,
    length: usize,
    received: usize,
    deadline: u64,
    ready: bool,
}
/// Opaque single-use completion ticket. No public success-response constructor.
pub struct CommitReady {
    request: u64,
    epoch: u64,
    frame: u64,
    tile: u64,
}
pub enum Action {
    Staged,
    Info(Reply),
    Commit(CommitReady),
}
pub struct Receiver {
    peer: u64,
    geometry: Geometry,
    timeout: u64,
    last_tick: u64,
    last_request: u64,
    last_rejected: u64,
    frame: u64,
    next_tile: u64,
    stage: Option<Stage>,
    pixels: [u8; TILE_BYTES],
    closed: bool,
}
impl Receiver {
    pub fn new(peer: u64, geometry: Geometry, timeout: u64) -> Result<Self, Error> {
        geometry.validate()?;
        if peer == 0 || timeout == 0 || timeout > 10000 {
            return Err(Error::Malformed);
        }
        Ok(Self {
            peer,
            geometry,
            timeout,
            last_tick: 0,
            last_request: 0,
            last_rejected: 0,
            frame: 0,
            next_tile: 0,
            stage: None,
            pixels: [0; TILE_BYTES],
            closed: false,
        })
    }
    fn clear(&mut self) {
        self.stage = None;
        self.pixels.fill(0);
    }
    pub fn retire(&mut self) {
        self.clear();
        self.closed = true;
    }
    pub fn completed_frame(&self) -> Option<u64> {
        if !self.closed && self.next_tile == self.geometry.tiles() {
            Some(self.frame)
        } else {
            None
        }
    }
    fn clock(&mut self, now: u64) -> Result<(), Error> {
        if now < self.last_tick {
            self.retire();
            return Err(Error::Clock);
        }
        self.last_tick = now;
        Ok(())
    }
    /// Expiry never creates a positive tile acknowledgement.
    pub fn expire(&mut self, now: u64) -> Result<Option<Reply>, Error> {
        self.clock(now)?;
        if let Some(stage) = self.stage {
            if now >= stage.deadline {
                self.last_rejected = stage.header.request;
                self.clear();
                return Ok(Some(reply(
                    Header {
                        operation: Operation::Commit,
                        arg0: 0,
                        arg1: 0,
                        ..stage.header
                    },
                    Err(Error::Timeout),
                    None,
                )));
            }
        }
        Ok(None)
    }
    /// Bounded negative response only to the configured actor and a canonical
    /// current-epoch transaction. Retire its staging and suppress error storms
    /// from already queued fragments. There is deliberately no public positive
    /// response builder; successful tiles go through commit_with's callback.
    pub fn reject(&mut self, sender: u64, bytes: &[u8], error: Error) -> Option<Reply> {
        if sender != self.peer || self.closed {
            return None;
        }
        let mut header = Header::decode(bytes).ok()?;
        if header.epoch != self.geometry.epoch || header.request <= self.last_rejected {
            return None;
        }
        if self.stage.is_none() && header.request <= self.last_request {
            return None;
        }
        if self
            .stage
            .is_some_and(|stage| stage.header.request != header.request)
        {
            return None;
        }
        self.last_rejected = header.request;
        self.last_request = self.last_request.max(header.request);
        self.clear();
        if header.operation != Operation::Info {
            header.operation = Operation::Commit;
            header.arg0 = 0;
            header.arg1 = 0;
        }
        Some(reply(header, Err(error), None))
    }
    pub fn accept(&mut self, sender: u64, now: u64, bytes: &[u8]) -> Result<Action, Error> {
        if sender != self.peer {
            return Err(Error::Access);
        }
        if self.closed {
            return Err(Error::Closed);
        }
        self.clock(now)?;
        // Expiry precedes decoding: even malformed traffic must not preserve
        // a token whose deadline is now known to have elapsed.
        if self.stage.is_some_and(|s| now >= s.deadline) {
            self.clear();
            return Err(Error::Timeout);
        }
        let header = Header::decode(bytes)?;
        if header.epoch != self.geometry.epoch {
            return Err(Error::Epoch);
        }
        match header.operation {
            Operation::Info | Operation::Begin => {
                if self.stage.is_some() {
                    return Err(Error::Busy);
                }
                if header.request <= self.last_request {
                    return Err(Error::Replay);
                }
                if header.operation == Operation::Info {
                    self.last_request = header.request;
                    return Ok(Action::Info(reply(header, Ok(0), Some(self.geometry))));
                }
                let rectangle = self.geometry.rectangle(header.tile)?;
                if header.frame < self.frame
                    || (header.frame == self.frame && header.tile != self.next_tile)
                    || (header.frame > self.frame && header.tile != 0)
                {
                    return Err(Error::Grid);
                }
                if header.arg0 != rectangle.x
                    || header.arg1 != rectangle.y
                    || u16at(bytes, 48) != rectangle.width
                    || u16at(bytes, 50) != rectangle.height
                    || u32at(bytes, 52) as usize != rectangle.bytes()
                {
                    return Err(Error::Grid);
                }
                let deadline = now.checked_add(self.timeout).ok_or(Error::Clock)?;
                if header.frame > self.frame {
                    self.frame = header.frame;
                    self.next_tile = 0;
                }
                self.last_request = header.request;
                self.stage = Some(Stage {
                    header,
                    rectangle,
                    length: rectangle.bytes(),
                    received: 0,
                    deadline,
                    ready: false,
                });
                Ok(Action::Staged)
            }
            Operation::Chunk | Operation::Commit => {
                let stage = self.stage.as_mut().ok_or(Error::Token)?;
                if header.request != stage.header.request
                    || header.frame != stage.header.frame
                    || header.tile != stage.header.tile
                    || stage.ready
                {
                    return Err(Error::Token);
                }
                if header.operation == Operation::Chunk {
                    let count = header.arg1 as usize;
                    if header.arg0 as usize != stage.received
                        || count != (stage.length - stage.received).min(CHUNK)
                    {
                        return Err(Error::Length);
                    }
                    self.pixels[stage.received..stage.received + count]
                        .copy_from_slice(&bytes[HEADER..]);
                    stage.received += count;
                    return Ok(Action::Staged);
                }
                if stage.received != stage.length {
                    return Err(Error::Length);
                }
                if self.pixels[..stage.length]
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] != 0)
                {
                    return Err(Error::Pixels);
                }
                stage.ready = true;
                Ok(Action::Commit(CommitReady {
                    request: header.request,
                    epoch: header.epoch,
                    frame: header.frame,
                    tile: header.tile,
                }))
            }
        }
    }
    /// Root's compositor supplies the real DisplayClient::blit callback. The
    /// callback is mandatory and receives only a validated global rectangle
    /// and complete canonical tile. A failed callback cannot encode success.
    /// This is sequencing support, not proof that an arbitrary callback is I/O.
    pub fn commit_with<E>(
        &mut self,
        token: CommitReady,
        now: u64,
        blit: impl FnOnce(Rectangle, &[u8]) -> Result<(), E>,
    ) -> Result<Reply, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        self.clock(now)?;
        let stage = self.stage.ok_or(Error::Token)?;
        if !stage.ready
            || stage.header.request != token.request
            || stage.header.epoch != token.epoch
            || stage.header.frame != token.frame
            || stage.header.tile != token.tile
        {
            return Err(Error::Token);
        }
        // Fresh native monotone time is required from the trusted adapter at
        // admission, before the first blit. This is not a vsync/deadline promise
        // across later preemption, and does not undo already written pixels.
        if now >= stage.deadline {
            self.last_rejected = stage.header.request;
            let header = Header {
                operation: Operation::Commit,
                arg0: 0,
                arg1: 0,
                ..stage.header
            };
            self.clear();
            return Ok(reply(header, Err(Error::Timeout), None));
        }
        let global = Rectangle {
            x: self.geometry.assigned.x + stage.rectangle.x,
            y: self.geometry.assigned.y + stage.rectangle.y,
            ..stage.rectangle
        };
        let result = blit(global, &self.pixels[..stage.length])
            .map(|()| stage.length as u32 / 4)
            .map_err(|_| Error::Blit);
        if result.is_ok() {
            self.frame = stage.header.frame;
            self.next_tile = stage.header.tile.checked_add(1).ok_or(Error::Exhausted)?;
        }
        let header = Header {
            operation: Operation::Commit,
            arg0: 0,
            arg1: 0,
            ..stage.header
        };
        self.clear();
        Ok(reply(header, result, None))
    }
}
