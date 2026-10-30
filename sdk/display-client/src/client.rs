use crate::wire::{self, Blit, Geometry, Rectangle, Reset, Sample, SIZE};
use nieuw_os_sdk_runtime::{Error as RuntimeError, Handle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplaySource(Handle);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputSource(Handle);
impl DisplaySource {
    /// Representation only; only the kernel validates type, holder and rights.
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}
impl InputSource {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}
pub trait Transport {
    fn display_info(
        &mut self,
        source: DisplaySource,
        out: &mut [u8; SIZE],
    ) -> Result<u64, RuntimeError>;
    fn display_blit(
        &mut self,
        source: DisplaySource,
        descriptor: &[u8; SIZE],
        pixels: &[u8],
    ) -> Result<u64, RuntimeError>;
    fn input_reset_begin(&mut self, source: InputSource) -> Result<u64, RuntimeError>;
    fn input_reset_poll(
        &mut self,
        source: InputSource,
        ticket: u64,
        out: &mut [u8; SIZE],
    ) -> Result<u64, RuntimeError>;
    fn input_read(
        &mut self,
        source: InputSource,
        out: &mut [u8; SIZE],
    ) -> Result<u64, RuntimeError>;
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub struct NativeTransport;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl Transport for NativeTransport {
    fn display_info(
        &mut self,
        s: DisplaySource,
        out: &mut [u8; SIZE],
    ) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::display_info(s.handle(), out)
    }
    fn display_blit(
        &mut self,
        s: DisplaySource,
        d: &[u8; SIZE],
        p: &[u8],
    ) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::display_blit(s.handle(), d, p)
    }
    fn input_reset_begin(&mut self, s: InputSource) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::input_reset_begin(s.handle())
    }
    fn input_reset_poll(
        &mut self,
        s: InputSource,
        t: u64,
        out: &mut [u8; SIZE],
    ) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::input_reset_poll(s.handle(), t, out)
    }
    fn input_read(&mut self, s: InputSource, out: &mut [u8; SIZE]) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::input_read(s.handle(), out)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    NotReady,
    Busy,
    Poisoned,
    Protocol,
    SequenceLost,
    Runtime(RuntimeError),
}

/// No capability is created or enlarged. The caller owns the manifest grant
/// and closes it explicitly through the SDK; these wrappers never share it.
pub struct DisplayClient<T> {
    transport: T,
    source: DisplaySource,
    geometry: Option<Geometry>,
    poisoned: bool,
}
impl<T: Transport> DisplayClient<T> {
    pub const fn new(transport: T, source: DisplaySource) -> Self {
        Self {
            transport,
            source,
            geometry: None,
            poisoned: false,
        }
    }
    pub fn geometry(&mut self) -> Result<Geometry, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let mut bytes = [0; SIZE];
        let count = self
            .transport
            .display_info(self.source, &mut bytes)
            .map_err(Error::Runtime)?;
        if count != SIZE as u64 {
            self.poisoned = true;
            return Err(Error::Protocol);
        }
        let geometry = Geometry::decode(&bytes).map_err(|_| {
            self.poisoned = true;
            Error::Protocol
        })?;
        if self.geometry.is_some_and(|old| old != geometry) {
            self.poisoned = true;
            return Err(Error::Protocol);
        }
        self.geometry = Some(geometry);
        Ok(geometry)
    }
    /// A syscall success means this bounded tile was written, not vsync or
    /// durable presentation. Errors do not clear caller-owned dirty work.
    pub fn blit(&mut self, rectangle: Rectangle, pixels: &[u8]) -> Result<(), Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let g = self.geometry.ok_or(Error::NotReady)?;
        let blit = Blit {
            epoch: g.epoch,
            rectangle,
        };
        blit.validate(g, pixels.len())
            .map_err(|_| Error::InvalidInput)?;
        wire::validate_pixels(pixels).map_err(|_| Error::InvalidInput)?;
        let descriptor = blit.encode().map_err(|_| Error::InvalidInput)?;
        let count = self
            .transport
            .display_blit(self.source, &descriptor, pixels)
            .map_err(Error::Runtime)?;
        if count != (pixels.len() / 4) as u64 {
            self.poisoned = true;
            return Err(Error::Protocol);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetProgress {
    Pending,
    Ready(Reset),
}
pub struct InputClient<T> {
    transport: T,
    source: InputSource,
    pending: Option<u64>,
    last_ticket: u64,
    last_epoch: u64,
    sequence: u64,
    ready: bool,
    poisoned: bool,
}
impl<T: Transport> InputClient<T> {
    pub const fn new(transport: T, source: InputSource) -> Self {
        Self {
            transport,
            source,
            pending: None,
            last_ticket: 0,
            last_epoch: 0,
            sequence: 0,
            ready: false,
            poisoned: false,
        }
    }
    pub const fn is_ready(&self) -> bool {
        self.ready && !self.poisoned
    }
    pub fn begin_reset(&mut self) -> Result<u64, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        self.ready = false;
        let ticket = self.transport.input_reset_begin(self.source).map_err(|e| {
            // Malformed/unknown completion must not trigger an automatic second
            // reset. Kernel error codes remain explicit, without retry here.
            if !matches!(e, RuntimeError::Kernel(_)) {
                self.poisoned = true;
            }
            Error::Runtime(e)
        })?;
        if ticket == 0 || ticket <= self.last_ticket {
            self.poisoned = true;
            return Err(Error::Protocol);
        }
        self.last_ticket = ticket;
        self.pending = Some(ticket);
        Ok(ticket)
    }
    pub fn poll_reset(&mut self) -> Result<ResetProgress, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let ticket = self.pending.ok_or(Error::NotReady)?;
        let mut bytes = [0; SIZE];
        let count = match self
            .transport
            .input_reset_poll(self.source, ticket, &mut bytes)
        {
            Ok(n) => n,
            Err(e) if e.is_again() => return Ok(ResetProgress::Pending),
            Err(e) => {
                self.pending = None;
                self.ready = false;
                if !matches!(e, RuntimeError::Kernel(_)) {
                    self.poisoned = true;
                }
                return Err(Error::Runtime(e));
            }
        };
        let report = Reset::decode(&bytes).map_err(|_| {
            self.poisoned = true;
            Error::Protocol
        })?;
        if count != SIZE as u64 || report.ticket != ticket || report.epoch <= self.last_epoch {
            self.poisoned = true;
            return Err(Error::Protocol);
        }
        self.last_epoch = report.epoch;
        self.sequence = 0;
        self.pending = None;
        self.ready = true;
        Ok(ResetProgress::Ready(report))
    }
    pub fn read(&mut self) -> Result<Option<Sample>, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        if !self.ready {
            return Err(Error::NotReady);
        }
        let mut bytes = [0; SIZE];
        let count = match self.transport.input_read(self.source, &mut bytes) {
            Ok(n) => n,
            Err(e) if e.is_again() => return Ok(None),
            Err(e) => {
                self.ready = false;
                return Err(Error::Runtime(e));
            }
        };
        let sample = Sample::decode(&bytes).map_err(|_| {
            self.poisoned = true;
            self.ready = false;
            Error::Protocol
        })?;
        if count != SIZE as u64 {
            self.poisoned = true;
            self.ready = false;
            return Err(Error::Protocol);
        }
        if sample.epoch != self.last_epoch || self.sequence.checked_add(1) != Some(sample.sequence)
        {
            self.ready = false;
            return Err(Error::SequenceLost);
        }
        self.sequence = sample.sequence;
        if sample.needs_reset() {
            self.ready = false;
        }
        Ok(Some(sample))
    }
}
