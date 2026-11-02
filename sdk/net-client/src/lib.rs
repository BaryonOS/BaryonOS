#![no_std]
#![forbid(unsafe_code)]
//! Two owned IPC endpoints to a separately authorized native service. A token
//! is owner/epoch-bound by that service; constructing an integer grants nothing.
use nieuw_os_catalog_client::Transport;
pub use nieuw_os_network_protocol::{ConnectionState, Status, DATA};
use nieuw_os_network_protocol::{HeaderFields, Operation, Packet, TAG};
use nieuw_os_sdk_runtime::{Error as RuntimeError, Outgoing, ReceiveEndpoint, SendEndpoint};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Connection(u64);
impl Connection {
    pub fn from_id(id: u64) -> Option<Self> {
        (id != 0).then_some(Self(id))
    }
    pub const fn id(self) -> u64 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Buffer,
    Closed,
    Exhausted,
    Protocol,
    Service(Status),
    Transport(RuntimeError),
    Uncertain,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Read {
    Data(usize),
    End,
}
pub struct Session<T> {
    transport: T,
    epoch: u64,
    peer: u64,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    budget: usize,
    request: u64,
    closed: bool,
}
impl<T: Transport> Session<T> {
    pub fn new(
        transport: T,
        epoch: u64,
        peer: u64,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        budget: usize,
    ) -> Result<Self, Error> {
        if epoch == 0
            || peer == 0
            || send.handle() == receive.handle()
            || budget == 0
            || budget > 1_000_000
        {
            return Err(Error::Configuration);
        }
        Ok(Self {
            transport,
            epoch,
            peer,
            send,
            receive,
            budget,
            request: 0,
            closed: false,
        })
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn transport(&self) -> &T {
        &self.transport
    }
    pub fn close_endpoints(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        let a = self.transport.close(self.send.handle());
        let b = self.transport.close(self.receive.handle());
        a.and(b)
    }
    fn terminal(&mut self, error: Error) -> Error {
        let _ = self.close_endpoints();
        error
    }
    fn call(
        &mut self,
        operation: Operation,
        connection: u64,
        argument: u32,
        address: [u8; 4],
        port: u16,
        data: &[u8],
    ) -> Result<Packet, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        self.request = self
            .request
            .checked_add(1)
            .ok_or_else(|| self.terminal(Error::Exhausted))?;
        let expected = HeaderFields {
            operation,
            request: self.request,
            epoch: self.epoch,
            connection,
            argument,
            address,
            port,
        };
        let packet = Packet::request(expected, data).map_err(|_| Error::Buffer)?;
        let outgoing = Outgoing::new(TAG, packet.bytes()).map_err(|_| Error::Buffer)?;
        let mut enqueued = false;
        for _ in 0..self.budget {
            match self.transport.send(self.send, &outgoing) {
                Ok(()) => {
                    enqueued = true;
                    break;
                }
                Err(e) if e.is_again() => {}
                Err(e) => return Err(self.terminal(Error::Transport(e))),
            }
            self.transport
                .yield_now()
                .map_err(|e| self.terminal(Error::Transport(e)))?;
        }
        if !enqueued {
            return Err(self.terminal(Error::Uncertain));
        }
        for _ in 0..self.budget {
            match self.transport.receive(self.receive) {
                Ok(message) => {
                    if message.cap_count() != 0 {
                        for i in 0..message.cap_count() {
                            if let Some(cap) = message.cap(i) {
                                let _ = self.transport.close(cap.handle);
                            }
                        }
                        return Err(self.terminal(Error::Protocol));
                    }
                    if message.sender() != self.peer || message.tag() != TAG {
                        return Err(self.terminal(Error::Protocol));
                    }
                    let reply = Packet::decode_response(message.payload(), expected)
                        .map_err(|_| self.terminal(Error::Protocol))?;
                    return if reply.status() == Status::Ok {
                        Ok(reply)
                    } else {
                        Err(Error::Service(reply.status()))
                    };
                }
                Err(e) if e.is_again() => {}
                Err(e) => return Err(self.terminal(Error::Transport(e))),
            }
            self.transport
                .yield_now()
                .map_err(|e| self.terminal(Error::Transport(e)))?;
        }
        // A successfully queued request is never automatically resent. Stream or
        // datagram publication may have occurred; a lost reply is uncertain.
        Err(self.terminal(Error::Uncertain))
    }
    pub fn open(
        &mut self,
        address: [u8; 4],
        port: u16,
        timeout_ticks: u32,
    ) -> Result<Connection, Error> {
        let reply = self.call(Operation::OpenTcp, 0, timeout_ticks, address, port, &[])?;
        Connection::from_id(reply.header().connection).ok_or_else(|| self.terminal(Error::Protocol))
    }
    pub fn state(&mut self, connection: Connection) -> Result<ConnectionState, Error> {
        let reply = self.call(Operation::State, connection.0, 0, [0; 4], 0, &[])?;
        ConnectionState::decode(reply.header().argument).map_err(|_| self.terminal(Error::Protocol))
    }
    pub fn send(&mut self, connection: Connection, data: &[u8]) -> Result<usize, Error> {
        if data.is_empty() || data.len() > DATA {
            return Err(Error::Buffer);
        }
        let reply = self.call(
            Operation::SendTcp,
            connection.0,
            data.len() as u32,
            [0; 4],
            0,
            data,
        )?;
        Ok(reply.header().argument as usize)
    }
    pub fn receive(&mut self, connection: Connection, out: &mut [u8]) -> Result<Read, Error> {
        if out.len() < DATA {
            return Err(Error::Buffer);
        }
        match self.call(Operation::ReceiveTcp, connection.0, 0, [0; 4], 0, &[]) {
            Ok(reply) => {
                let n = reply.data().len();
                out[..n].copy_from_slice(reply.data());
                Ok(Read::Data(n))
            }
            Err(Error::Service(Status::Closed)) => Ok(Read::End),
            Err(e) => Err(e),
        }
    }
    pub fn close(&mut self, connection: Connection) -> Result<(), Error> {
        self.call(Operation::CloseTcp, connection.0, 0, [0; 4], 0, &[])
            .map(|_| ())
    }
    /// Success means queued in the service, not NIC completion or remote receipt.
    pub fn send_udp(&mut self, address: [u8; 4], port: u16, data: &[u8]) -> Result<usize, Error> {
        if data.len() > DATA {
            return Err(Error::Buffer);
        }
        let reply = self.call(
            Operation::SendUdp,
            0,
            data.len() as u32,
            address,
            port,
            data,
        )?;
        Ok(reply.header().argument as usize)
    }
    pub fn receive_udp(&mut self, out: &mut [u8]) -> Result<usize, Error> {
        if out.len() < DATA {
            return Err(Error::Buffer);
        }
        let reply = self.call(Operation::ReceiveUdp, 0, 0, [0; 4], 0, &[])?;
        let n = reply.data().len();
        out[..n].copy_from_slice(reply.data());
        Ok(n)
    }
    pub fn retire(&mut self) -> Result<(), Error> {
        self.call(Operation::Retire, 0, 0, [0; 4], 0, &[])?;
        self.close_endpoints().map_err(Error::Transport)
    }
}
