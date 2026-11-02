//! One owner of one app->compositor endpoint pair. Positive completion means
//! this tile's validated COMMIT reply, not pageflip, persistence or GUI quality.
use crate::{
    Error as ServiceError, Geometry, Header, Operation, Rectangle, Reply, CHUNK, HEADER,
    MAX_MESSAGE, TAG,
};
use nieuw_os_catalog_client::{Delivery, Transport, MAX_POLL_BUDGET};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Closed,
    NotReady,
    Exhausted,
    InvalidTile,
    Interrupted {
        delivery: Delivery,
        runtime: Option<RuntimeError>,
        cleanup: Option<RuntimeError>,
    },
    Protocol {
        cleanup: Option<RuntimeError>,
    },
    Service {
        error: ServiceError,
        cleanup: Option<RuntimeError>,
    },
}
pub struct Session<T> {
    transport: T,
    peer: u64,
    epoch: u64,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    budget: usize,
    next_request: u64,
    geometry: Option<Geometry>,
    frame: u64,
    next_tile: u64,
    closed: bool,
}
impl<T: Transport> Session<T> {
    /// The caller transfers the endpoint pair only on successful construction.
    /// Copyable handle representations must not be owned by another session.
    pub fn new(
        transport: T,
        peer: u64,
        epoch: u64,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        budget: usize,
    ) -> Result<Self, Error> {
        if peer == 0
            || epoch == 0
            || send.handle() == receive.handle()
            || budget == 0
            || budget > MAX_POLL_BUDGET
        {
            return Err(Error::Configuration);
        }
        Ok(Self {
            transport,
            peer,
            epoch,
            send,
            receive,
            budget,
            next_request: 1,
            geometry: None,
            frame: 0,
            next_tile: 0,
            closed: false,
        })
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        self.close_owner(false)
    }
    /// Only explicit manifest retirement tolerates an already-closed recorded
    /// leaf. Normalize BadHandle separately for each close, retaining any other
    /// failure from either owner operation; no positive tile ACK is invented.
    pub fn retire(&mut self) -> Result<(), RuntimeError> {
        self.close_owner(true)
    }
    fn close_owner(&mut self, externally_retired: bool) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        let normalize = |result| match result {
            Err(RuntimeError::Kernel(-4)) if externally_retired => Ok(()),
            other => other,
        };
        let send = normalize(self.transport.close(self.send.handle()));
        let receive = normalize(self.transport.close(self.receive.handle()));
        send.and(receive)
    }
    fn fresh(&mut self) -> Result<u64, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if self.next_request == u64::MAX {
            let _ = self.close();
            return Err(Error::Exhausted);
        }
        let request = self.next_request;
        self.next_request += 1;
        Ok(request)
    }
    fn protocol(&mut self) -> Error {
        let cleanup = self.close().err();
        Error::Protocol { cleanup }
    }
    fn interrupted(&mut self, delivery: Delivery, runtime: Option<RuntimeError>) -> Error {
        let cleanup = self.close().err();
        Error::Interrupted {
            delivery,
            runtime,
            cleanup,
        }
    }
    fn send(&mut self, payload: &[u8], published: &mut bool) -> Result<(), Error> {
        let message = Outgoing::new(TAG, payload).map_err(|_| Error::InvalidTile)?;
        for _ in 0..self.budget {
            match self.transport.send(self.send, &message) {
                Ok(()) => {
                    *published = true;
                    return Ok(());
                }
                Err(error) if error.is_again() => {}
                Err(error) => {
                    let delivery = if matches!(error, RuntimeError::Kernel(_)) {
                        if *published {
                            Delivery::Enqueued
                        } else {
                            Delivery::NotEnqueued
                        }
                    } else {
                        Delivery::Unknown
                    };
                    return Err(self.interrupted(delivery, Some(error)));
                }
            }
            if let Err(error) = self.transport.yield_now() {
                return Err(self.interrupted(
                    if *published {
                        Delivery::Enqueued
                    } else {
                        Delivery::NotEnqueued
                    },
                    Some(error),
                ));
            }
        }
        Err(self.interrupted(
            if *published {
                Delivery::Enqueued
            } else {
                Delivery::NotEnqueued
            },
            None,
        ))
    }
    fn receive_reply(&mut self, expected: Header) -> Result<Reply, Error> {
        let message: Received = {
            let mut found = None;
            for _ in 0..self.budget {
                match self.transport.receive(self.receive) {
                    Ok(message) => {
                        found = Some(message);
                        break;
                    }
                    Err(error) if error.is_again() => {}
                    Err(error) => return Err(self.interrupted(Delivery::Enqueued, Some(error))),
                }
                if let Err(error) = self.transport.yield_now() {
                    return Err(self.interrupted(Delivery::Enqueued, Some(error)));
                }
            }
            match found {
                Some(message) => message,
                None => return Err(self.interrupted(Delivery::Enqueued, None)),
            }
        };
        if message.cap_count() != 0 {
            let mut cap_error = None;
            for i in 0..message.cap_count() {
                if let Some(cap) = message.cap(i) {
                    if let Err(error) = self.transport.close(cap.handle) {
                        cap_error.get_or_insert(error);
                    }
                }
            }
            let cleanup = self.close().err().or(cap_error);
            return Err(Error::Protocol { cleanup });
        }
        if message.sender() != self.peer || message.tag() != TAG {
            return Err(self.protocol());
        }
        Reply::decode(message.payload(), expected).map_err(|_| self.protocol())
    }
    fn receive(&mut self, expected: Header) -> Result<Reply, Error> {
        let reply = self.receive_reply(expected)?;
        if let Err(error) = reply.status() {
            let cleanup = self.close().err();
            return Err(Error::Service { error, cleanup });
        }
        Ok(reply)
    }
    pub fn geometry(&mut self) -> Result<Geometry, Error> {
        let request = self.fresh()?;
        let header = Header {
            operation: Operation::Info,
            request,
            epoch: self.epoch,
            frame: 0,
            tile: 0,
            arg0: 0,
            arg1: 0,
        };
        let mut published = false;
        self.send(&header.encode(), &mut published)?;
        let reply = self.receive(header)?;
        let geometry = reply.geometry().map_err(|_| self.protocol())?;
        if self.geometry.is_some_and(|old| old != geometry) {
            return Err(self.protocol());
        }
        self.geometry = Some(geometry);
        Ok(geometry)
    }
    /// Explicit native acceptance diagnostic, before the first tile. This
    /// publishes a canonical BEGIN whose x lies just outside the assigned
    /// local surface. Only the real peer's correlated COMMIT/Grid refusal
    /// permits continued use. No positive tile ACK or local paint is implied.
    /// Ordinary blit/geometry service failures still close the endpoint pair.
    pub fn verify_isolation_probe(&mut self) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let geometry = self.geometry.ok_or(Error::NotReady)?;
        if self.frame != 0 || self.next_tile != 0 {
            return Err(Error::InvalidTile);
        }
        let rectangle = geometry.rectangle(0).map_err(|_| self.protocol())?;
        let mut header = Header {
            operation: Operation::Begin,
            request: self.fresh()?,
            epoch: self.epoch,
            frame: 1,
            tile: 0,
            arg0: u32::from(geometry.assigned.width),
            arg1: rectangle.y,
        };
        let mut begin = [0; HEADER + 8];
        begin[..HEADER].copy_from_slice(&header.encode());
        begin[48..50].copy_from_slice(&rectangle.width.to_le_bytes());
        begin[50..52].copy_from_slice(&rectangle.height.to_le_bytes());
        begin[52..56].copy_from_slice(&(rectangle.bytes() as u32).to_le_bytes());
        let mut published = false;
        self.send(&begin, &mut published)?;
        header.operation = Operation::Commit;
        header.arg0 = 0;
        header.arg1 = 0;
        let reply = self.receive_reply(header)?;
        match reply.status() {
            Err(ServiceError::Grid) => Ok(()),
            Err(error) => Err(Error::Service {
                error,
                cleanup: self.close().err(),
            }),
            Ok(()) => Err(self.protocol()),
        }
    }
    pub fn blit(
        &mut self,
        frame: u64,
        tile: u64,
        rectangle: Rectangle,
        pixels: &[u8],
    ) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let geometry = self.geometry.ok_or(Error::NotReady)?;
        if frame == 0
            || geometry.rectangle(tile).ok() != Some(rectangle)
            || pixels.len() != rectangle.bytes()
            || pixels.chunks_exact(4).any(|p| p[3] != 0)
            || frame < self.frame
            || (frame == self.frame && tile != self.next_tile)
            || (frame > self.frame && tile != 0)
        {
            return Err(Error::InvalidTile);
        }
        let request = self.fresh()?;
        let mut header = Header {
            operation: Operation::Begin,
            request,
            epoch: self.epoch,
            frame,
            tile,
            arg0: rectangle.x,
            arg1: rectangle.y,
        };
        let mut begin = [0; HEADER + 8];
        begin[..HEADER].copy_from_slice(&header.encode());
        begin[48..50].copy_from_slice(&rectangle.width.to_le_bytes());
        begin[50..52].copy_from_slice(&rectangle.height.to_le_bytes());
        begin[52..56].copy_from_slice(&(pixels.len() as u32).to_le_bytes());
        let mut published = false;
        self.send(&begin, &mut published)?;
        header.operation = Operation::Chunk;
        for (index, part) in pixels.chunks(CHUNK).enumerate() {
            header.arg0 = (index * CHUNK) as u32;
            header.arg1 = part.len() as u32;
            let mut message = [0; MAX_MESSAGE];
            message[..HEADER].copy_from_slice(&header.encode());
            message[HEADER..HEADER + part.len()].copy_from_slice(part);
            self.send(&message[..HEADER + part.len()], &mut published)?;
        }
        header.operation = Operation::Commit;
        header.arg0 = 0;
        header.arg1 = 0;
        self.send(&header.encode(), &mut published)?;
        let reply = self.receive(header)?;
        if reply.pixels() as usize != pixels.len() / 4 {
            return Err(self.protocol());
        }
        self.frame = frame;
        self.next_tile = tile + 1;
        Ok(())
    }
}
