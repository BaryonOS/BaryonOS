//! One owner of one app->compositor endpoint pair. Positive completion means
//! a validated COMMIT reply or explicit retained-backing coverage reply, not
//! pageflip, persistence or GUI quality. Retain does not claim a new blit.
use crate::{
    Error as ServiceError, Geometry, Header, Operation, Rectangle, Reply, CHUNK, HEADER,
    MAX_MESSAGE, MAX_RETAIN_TILES, TAG,
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
/// Actual transport attempts and successfully acknowledged surface work.
/// `sent_bytes` counts published SUR1 payload bytes, excluding the IPC envelope.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SurfaceCounts {
    pub send_calls: u64,
    pub receive_calls: u64,
    pub sent_messages: u64,
    pub sent_bytes: u64,
    pub committed_tiles: u64,
    pub solid_tiles: u64,
    pub retained_runs: u64,
    pub retained_tiles: u64,
    pub completed_frames: u64,
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
    /// Mirrors `Receiver`: sparse frames only after one complete frame.
    complete: bool,
    closed: bool,
    retained: bool,
    retained_base: Option<u64>,
    counts: SurfaceCounts,
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
            complete: false,
            closed: false,
            retained: false,
            retained_base: None,
            counts: SurfaceCounts::default(),
        })
    }
    pub const fn counts(&self) -> SurfaceCounts {
        self.counts
    }
    pub const fn retained_base(&self) -> Option<u64> {
        self.retained_base
    }
    /// Explicit peer-profile opt-in, after authenticated INFO and before paint.
    /// Unsupported or ambiguous peer responses still close the session.
    pub fn enable_retained(&mut self) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        self.geometry.ok_or(Error::NotReady)?;
        if self.frame != 0 || self.next_tile != 0 {
            return Err(Error::Configuration);
        }
        self.retained = true;
        Ok(())
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
        self.retained_base = None;
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
    fn exhausted(&mut self) -> Error {
        let _ = self.close();
        Error::Exhausted
    }
    fn add_count(&mut self, value: u64, amount: u64) -> Result<u64, Error> {
        value.checked_add(amount).ok_or_else(|| self.exhausted())
    }
    fn advance(&mut self, frame: u64, next_tile: u64, geometry: Geometry) -> Result<(), Error> {
        if next_tile == geometry.tiles() {
            self.counts.completed_frames = self.add_count(self.counts.completed_frames, 1)?;
            if self.retained {
                self.retained_base = Some(frame);
            }
            self.complete = true;
        }
        self.frame = frame;
        self.next_tile = next_tile;
        Ok(())
    }
    fn preflight_advance(&mut self, next_tile: u64, geometry: Geometry) -> Result<(), Error> {
        if next_tile == geometry.tiles() {
            self.add_count(self.counts.completed_frames, 1)?;
        }
        Ok(())
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
            let send_calls = self.add_count(self.counts.send_calls, 1)?;
            let sent_messages = self.add_count(self.counts.sent_messages, 1)?;
            let sent_bytes = self.add_count(self.counts.sent_bytes, payload.len() as u64)?;
            self.counts.send_calls = send_calls;
            match self.transport.send(self.send, &message) {
                Ok(()) => {
                    self.counts.sent_messages = sent_messages;
                    self.counts.sent_bytes = sent_bytes;
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
                self.counts.receive_calls = self.add_count(self.counts.receive_calls, 1)?;
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
    /// Retain only already confirmed live backing, with complete grid coverage.
    /// The caller must preserve the complete baseline and account for every
    /// tile changed since it; the peer independently refuses such changed
    /// tiles. A failed/unknown reply closes this session; there is no replay.
    pub fn retain(&mut self, frame: u64, tile: u64, count: u32) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let geometry = self.geometry.ok_or(Error::NotReady)?;
        if !self.retained {
            return Err(Error::Configuration);
        }
        let base = self.retained_base.ok_or(Error::NotReady)?;
        let next_tile = tile
            .checked_add(u64::from(count))
            .ok_or(Error::InvalidTile)?;
        if frame == 0
            || !(1..=MAX_RETAIN_TILES).contains(&count)
            || next_tile > geometry.tiles()
            || frame < self.frame
            || (frame == self.frame && tile != self.next_tile)
            || (frame > self.frame && tile != 0)
        {
            return Err(Error::InvalidTile);
        }
        let retained_runs = self.add_count(self.counts.retained_runs, 1)?;
        let retained_tiles = self.add_count(self.counts.retained_tiles, u64::from(count))?;
        self.preflight_advance(next_tile, geometry)?;
        let request = self.fresh()?;
        let header = Header {
            operation: Operation::Retain,
            request,
            epoch: self.epoch,
            frame,
            tile,
            arg0: count,
            arg1: 0,
        };
        let mut bytes = [0; HEADER + 8];
        bytes[..HEADER].copy_from_slice(&header.encode());
        bytes[HEADER..].copy_from_slice(&base.to_le_bytes());
        let mut published = false;
        self.send(&bytes, &mut published)?;
        let reply = self.receive(header)?;
        if reply.retained_tiles() != count || reply.pixels() != 0 {
            return Err(self.protocol());
        }
        self.counts.retained_runs = retained_runs;
        self.counts.retained_tiles = retained_tiles;
        self.advance(frame, next_tile, geometry)
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
            || (frame == self.frame
                && if self.complete {
                    tile < self.next_tile
                } else {
                    tile != self.next_tile
                })
            || (frame > self.frame && !self.complete && tile != 0)
        {
            return Err(Error::InvalidTile);
        }
        let solid = self.retained && pixels.chunks_exact(4).all(|pixel| pixel == &pixels[..4]);
        let committed_tiles = self.add_count(self.counts.committed_tiles, 1)?;
        let solid_tiles = if solid {
            self.add_count(self.counts.solid_tiles, 1)?
        } else {
            self.counts.solid_tiles
        };
        self.preflight_advance(tile + 1, geometry)?;
        let request = self.fresh()?;
        let mut header = Header {
            operation: if solid {
                Operation::SolidBegin
            } else {
                Operation::Begin
            },
            request,
            epoch: self.epoch,
            frame,
            tile,
            arg0: rectangle.x,
            arg1: rectangle.y,
        };
        let mut begin = [0; HEADER + 12];
        begin[..HEADER].copy_from_slice(&header.encode());
        begin[48..50].copy_from_slice(&rectangle.width.to_le_bytes());
        begin[50..52].copy_from_slice(&rectangle.height.to_le_bytes());
        begin[52..56].copy_from_slice(&(pixels.len() as u32).to_le_bytes());
        if solid {
            begin[56..60].copy_from_slice(&pixels[..4]);
        }
        let mut published = false;
        self.send(
            &begin[..if solid { HEADER + 12 } else { HEADER + 8 }],
            &mut published,
        )?;
        if !solid {
            header.operation = Operation::Chunk;
            for (index, part) in pixels.chunks(CHUNK).enumerate() {
                header.arg0 = (index * CHUNK) as u32;
                header.arg1 = part.len() as u32;
                let mut message = [0; MAX_MESSAGE];
                message[..HEADER].copy_from_slice(&header.encode());
                message[HEADER..HEADER + part.len()].copy_from_slice(part);
                self.send(&message[..HEADER + part.len()], &mut published)?;
            }
        }
        header.operation = Operation::Commit;
        header.arg0 = 0;
        header.arg1 = 0;
        self.send(&header.encode(), &mut published)?;
        let reply = self.receive(header)?;
        if reply.pixels() as usize != pixels.len() / 4 {
            return Err(self.protocol());
        }
        self.counts.committed_tiles = committed_tiles;
        self.counts.solid_tiles = solid_tiles;
        self.advance(frame, tile + 1, geometry)
    }
}

#[cfg(test)]
mod counter_tests {
    use super::*;
    use nieuw_os_sdk_runtime::Handle;

    struct NoIo {
        closed: usize,
    }
    impl Transport for NoIo {
        fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
            panic!("overflow must fail before send")
        }
        fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
            panic!("overflow must fail before receive")
        }
        fn yield_now(&mut self) -> Result<(), RuntimeError> {
            panic!("overflow must not yield")
        }
        fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
            self.closed += 1;
            Ok(())
        }
    }
    fn session() -> Session<NoIo> {
        let mut session = Session::new(
            NoIo { closed: 0 },
            2,
            11,
            SendEndpoint::from_handle(Handle::from_raw(0x1_0000_0001).unwrap()),
            ReceiveEndpoint::from_handle(Handle::from_raw(0x1_0000_0002).unwrap()),
            1,
        )
        .unwrap();
        session.geometry = Some(Geometry {
            width: 800,
            height: 600,
            epoch: 11,
            assigned: Rectangle {
                x: 8,
                y: 24,
                width: 32,
                height: 8,
            },
        });
        session.retained = true;
        session.retained_base = Some(7);
        session.frame = 7;
        session.next_tile = 1;
        session
    }
    fn closed(session: &Session<NoIo>) {
        assert!(session.is_closed());
        assert_eq!(session.retained_base(), None);
        assert_eq!(session.transport.closed, 2);
    }
    #[test]
    fn transport_counter_overflow_fails_before_calls_and_clears_baseline() {
        for field in 0..3 {
            let mut session = session();
            match field {
                0 => session.counts.send_calls = u64::MAX,
                1 => session.counts.sent_messages = u64::MAX,
                _ => session.counts.sent_bytes = u64::MAX,
            }
            assert_eq!(session.geometry(), Err(Error::Exhausted));
            closed(&session);
        }
        let mut session = session();
        session.counts.receive_calls = u64::MAX;
        assert!(matches!(
            session.receive_reply(Header {
                operation: Operation::Info,
                request: 1,
                epoch: 11,
                frame: 0,
                tile: 0,
                arg0: 0,
                arg1: 0,
            }),
            Err(Error::Exhausted)
        ));
        closed(&session);
    }
    #[test]
    fn work_counter_overflow_fails_before_publication_and_clears_baseline() {
        for field in 0..3 {
            let mut session = session();
            match field {
                0 => session.counts.committed_tiles = u64::MAX,
                1 => session.counts.solid_tiles = u64::MAX,
                _ => session.counts.completed_frames = u64::MAX,
            }
            let rect = session.geometry.unwrap().rectangle(0).unwrap();
            assert_eq!(session.blit(8, 0, rect, &[0; 1024]), Err(Error::Exhausted));
            closed(&session);
            assert_eq!(session.counts.send_calls, 0);
        }
        for field in 0..3 {
            let mut session = session();
            match field {
                0 => session.counts.retained_runs = u64::MAX,
                1 => session.counts.retained_tiles = u64::MAX,
                _ => session.counts.completed_frames = u64::MAX,
            }
            assert_eq!(session.retain(8, 0, 1), Err(Error::Exhausted));
            closed(&session);
            assert_eq!(session.counts.send_calls, 0);
        }
    }
}
