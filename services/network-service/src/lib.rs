#![no_std]
#![forbid(unsafe_code)]
//! A reusable userspace network service. Trusted startup binds two ingress
//! peers to fixed destination ranges; packet fields and app bodies grant nothing.
use nieuw_os_net_protocol::{
    arp::{Interface, NeighborSlot},
    build_udp_ipv4, parse, UdpIpv4Tx,
};
use nieuw_os_net_stack::{
    Client, Config, Error as StackError, SocketStorage, Stack, State, Transmission, FRAME_BYTES,
};
pub use nieuw_os_network_protocol as wire;
use wire::{ConnectionState, Operation, Packet, Status, DATA};
pub const CLIENTS: usize = 2;
pub const SOCKET_BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Destination {
    pub ipv4: [u8; 4],
    pub tcp_first: u16,
    pub tcp_count: u16,
    pub udp_port: u16,
}
#[derive(Clone, Copy)]
pub struct Configuration {
    pub epoch: u64,
    pub pids: [u64; CLIENTS],
    pub link: Config,
    pub destinations: [Destination; CLIENTS],
}
pub struct Buffers<'a> {
    pub storage: &'a mut [SocketStorage<'a>],
    pub rx_frame: &'a mut [u8; FRAME_BYTES],
    pub tx_frame: &'a mut [u8; FRAME_BYTES],
    pub rx_sockets: [&'a mut [u8]; CLIENTS],
    pub tx_sockets: [&'a mut [u8]; CLIENTS],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fault {
    Configuration,
    Clock,
    Transport,
    Exhausted,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Counters {
    pub requests: u64,
    pub rejected: u64,
    pub opened: u64,
    pub sent: u64,
    pub received: u64,
    pub closed: u64,
    pub udp_sent: u64,
    pub udp_received: u64,
    pub malformed_frames: u64,
    pub dropped_datagrams: u64,
}
struct Owner {
    client: Client,
    request: u64,
    generation: u64,
    token: u64,
    active: bool,
    closing: bool,
    deadline: u64,
    timed_out: bool,
    retired: bool,
    udp: [u8; DATA],
    udp_len: Option<usize>,
    neighbor: NeighborSlot,
}
pub struct Service<'a> {
    config: Configuration,
    stack: Stack<'a>,
    owners: [Owner; CLIENTS],
    last_tick: u64,
    next_port: u16,
    identification: u16,
    counters: Counters,
}
impl<'a> Service<'a> {
    pub fn new(config: Configuration, buffers: Buffers<'a>, now: u64) -> Result<Self, Fault> {
        if config.epoch == 0
            || config.pids.contains(&0)
            || config.pids[0] == config.pids[1]
            || buffers.storage.len() != CLIENTS
        {
            return Err(Fault::Configuration);
        }
        let mask = if config.link.prefix_len == 0 || config.link.prefix_len > 32 {
            return Err(Fault::Configuration);
        } else {
            u32::MAX << (32 - config.link.prefix_len)
        };
        for d in config.destinations {
            if d.ipv4[0] == 0
                || d.ipv4[0] == 127
                || d.ipv4[0] >= 224
                || d.ipv4 == config.link.ipv4
                || u32::from_be_bytes(d.ipv4) & mask != u32::from_be_bytes(config.link.ipv4) & mask
                || d.tcp_first == 0
                || d.tcp_count == 0
                || d.tcp_count > 8
                || d.tcp_first.checked_add(d.tcp_count - 1).is_none()
                || d.udp_port == 0
            {
                return Err(Fault::Configuration);
            }
        }
        let mut stack = Stack::new(
            config.link,
            buffers.storage,
            buffers.rx_frame,
            buffers.tx_frame,
            now,
        )
        .map_err(|_| Fault::Configuration)?;
        let [ra, rb] = buffers.rx_sockets;
        let [ta, tb] = buffers.tx_sockets;
        let clients = [
            stack.add_client(ra, ta).map_err(|_| Fault::Configuration)?,
            stack.add_client(rb, tb).map_err(|_| Fault::Configuration)?,
        ];
        let local = Interface {
            mac: config.link.mac,
            ipv4: config.link.ipv4,
        };
        let make = |client| {
            Ok(Owner {
                client,
                request: 0,
                generation: 0,
                token: 0,
                active: false,
                closing: false,
                deadline: 0,
                timed_out: false,
                retired: false,
                udp: [0; DATA],
                udp_len: None,
                neighbor: NeighborSlot::new(local).map_err(|_| Fault::Configuration)?,
            })
        };
        Ok(Self {
            config,
            stack,
            owners: [make(clients[0])?, make(clients[1])?],
            last_tick: now,
            next_port: 40000,
            identification: 0,
            counters: Counters::default(),
        })
    }
    fn time(&self, now: u64) -> Result<(), Fault> {
        if now < self.last_tick || now > nieuw_os_net_stack::MAX_MONOTONIC_TICKS {
            Err(Fault::Clock)
        } else {
            Ok(())
        }
    }
    pub fn counters(&self) -> Counters {
        self.counters
    }
    pub fn retired_mask(&self) -> u64 {
        u64::from(self.owners[0].retired) | u64::from(self.owners[1].retired) << 1
    }
    pub fn can_receive_frame(&self) -> bool {
        self.stack.can_receive_frame()
    }
    pub fn transmission(&self) -> Option<Transmission<'_>> {
        self.stack.transmission()
    }
    pub fn complete_transmission(&mut self, id: u64) -> Result<(), Fault> {
        self.stack
            .complete_transmission(id)
            .map_err(|_| Fault::Transport)
    }
    pub fn poll(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        self.stack.poll(now).map_err(|_| Fault::Clock)?;
        self.last_tick = now;
        for owner in &mut self.owners {
            if owner.active
                && !owner.closing
                && now >= owner.deadline
                && self
                    .stack
                    .state(owner.client)
                    .map_err(|_| Fault::Transport)?
                    != State::Closed
            {
                self.stack
                    .cancel(owner.client)
                    .map_err(|_| Fault::Transport)?;
                owner.timed_out = true;
                owner.closing = true;
            }
        }
        Ok(())
    }
    pub fn ingress(&mut self, frame: &[u8], now: u64) -> Result<(), Fault> {
        self.time(now)?;
        if frame.len() >= 14 && frame[12..14] == [8, 6] {
            for owner in &mut self.owners {
                let _ = owner.neighbor.accept_reply(frame, now, 2000);
            }
        }
        if frame.len() > 23 && frame[12..14] == [8, 0] && frame[23] == 17 {
            match parse(frame) {
                Ok(packet) => {
                    let mut accepted = false;
                    for (i, owner) in self.owners.iter_mut().enumerate() {
                        let d = self.config.destinations[i];
                        if !owner.retired
                            && packet.destination_mac() == self.config.link.mac
                            && packet.destination_address() == self.config.link.ipv4
                            && packet.source_address() == d.ipv4
                            && packet.source_port() == d.udp_port
                            && packet.destination_port() == 45000 + i as u16
                            && packet.payload().len() <= DATA
                        {
                            accepted = true;
                            if owner.udp_len.is_none() {
                                owner.udp[..packet.payload().len()]
                                    .copy_from_slice(packet.payload());
                                owner.udp_len = Some(packet.payload().len());
                                self.counters.udp_received += 1;
                            } else {
                                self.counters.dropped_datagrams += 1;
                            }
                            break;
                        }
                    }
                    if !accepted {
                        self.counters.malformed_frames += 1;
                    }
                }
                Err(_) => self.counters.malformed_frames += 1,
            };
            return Ok(());
        }
        if !(14..=FRAME_BYTES).contains(&frame.len()) {
            self.counters.malformed_frames += 1;
            return Ok(());
        }
        self.stack.push_frame(frame).map_err(|_| Fault::Transport)
    }
    pub fn retire_all(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        for owner in &mut self.owners {
            owner.retired = true;
            owner.closing = true;
            owner.active = false;
            owner.udp.fill(0);
            owner.udp_len = None;
            owner.neighbor.cancel();
            self.stack
                .close(owner.client)
                .map_err(|_| Fault::Transport)?;
        }
        Ok(())
    }
    pub fn clean(&self) -> bool {
        self.retired_mask() == 3
            && self.stack.transmission().is_none()
            && self
                .owners
                .iter()
                .all(|o| self.stack.state(o.client) == Ok(State::Closed))
    }
    fn state(&self, index: usize) -> Result<ConnectionState, Fault> {
        let owner = &self.owners[index];
        if owner.timed_out {
            return Ok(ConnectionState::TimedOut);
        }
        Ok(
            match self
                .stack
                .state(owner.client)
                .map_err(|_| Fault::Transport)?
            {
                State::Closed => ConnectionState::Closed,
                State::SynSent | State::SynReceived | State::Listen => ConnectionState::Connecting,
                State::Established => ConnectionState::Established,
                State::CloseWait => ConnectionState::PeerClosed,
                _ => ConnectionState::Closing,
            },
        )
    }
    /// `slot` is selected by the trusted native ingress, and `sender` must be the
    /// actual kernel envelope PID. Passing app-supplied identities is forbidden.
    pub fn handle(
        &mut self,
        slot: usize,
        sender: u64,
        cap_count: usize,
        bytes: &[u8],
        now: u64,
    ) -> Result<Option<Packet>, Fault> {
        self.time(now)?;
        if slot >= CLIENTS {
            return Err(Fault::Configuration);
        }
        if sender != self.config.pids[slot] || cap_count != 0 {
            self.counters.rejected += 1;
            return Ok(None);
        }
        let request = match Packet::decode_request(bytes) {
            Ok(p) => p,
            Err(_) => {
                self.counters.rejected += 1;
                return Ok(Packet::refusal_header(bytes, self.config.epoch)
                    .map(|h| Packet::response(h, Status::Malformed, 0, &[])));
            }
        };
        let mut h = request.header();
        self.counters.requests += 1;
        let refusal = |h, status| Some(Packet::response(h, status, 0, &[]));
        if h.epoch != self.config.epoch {
            self.counters.rejected += 1;
            return Ok(refusal(h, Status::Stale));
        }
        if h.request <= self.owners[slot].request {
            self.counters.rejected += 1;
            return Ok(refusal(h, Status::Replay));
        }
        self.owners[slot].request = h.request;
        if self.owners[slot].retired {
            self.counters.rejected += 1;
            return Ok(refusal(h, Status::Closed));
        }
        let d = self.config.destinations[slot];
        if matches!(h.operation, Operation::OpenTcp | Operation::SendUdp)
            && (h.address != d.ipv4
                || if h.operation == Operation::OpenTcp {
                    h.port < d.tcp_first
                        || u32::from(h.port) >= u32::from(d.tcp_first) + u32::from(d.tcp_count)
                } else {
                    h.port != d.udp_port
                })
        {
            self.counters.rejected += 1;
            return Ok(refusal(h, Status::Access));
        }
        if matches!(
            h.operation,
            Operation::SendTcp | Operation::ReceiveTcp | Operation::CloseTcp | Operation::State
        ) && (!self.owners[slot].active || h.connection != self.owners[slot].token)
        {
            self.counters.rejected += 1;
            return Ok(refusal(h, Status::Stale));
        }
        let mut data = [0; DATA];
        let (status, argument) = match h.operation {
            Operation::OpenTcp => {
                if self
                    .stack
                    .state(self.owners[slot].client)
                    .map_err(|_| Fault::Transport)?
                    != State::Closed
                {
                    return Ok(refusal(h, Status::Busy));
                }
                let generation = self.owners[slot]
                    .generation
                    .checked_add(1)
                    .filter(|g| *g < u64::MAX >> 8)
                    .ok_or(Fault::Exhausted)?;
                let port = self.next_port;
                self.next_port = self.next_port.checked_add(1).ok_or(Fault::Exhausted)?;
                let deadline = now.checked_add(u64::from(h.argument)).ok_or(Fault::Clock)?;
                match self.stack.connect(
                    self.owners[slot].client,
                    h.address,
                    h.port,
                    port,
                    u64::from(h.argument),
                ) {
                    Ok(()) => {
                        let owner = &mut self.owners[slot];
                        owner.generation = generation;
                        owner.token = (generation << 8) | (slot as u64 + 1);
                        owner.active = true;
                        owner.closing = false;
                        owner.deadline = deadline;
                        owner.timed_out = false;
                        h.connection = owner.token;
                        self.counters.opened += 1;
                        (Status::Ok, ConnectionState::Connecting as u32)
                    }
                    Err(_) => (Status::Busy, 0),
                }
            }
            Operation::SendTcp => {
                if self.owners[slot].closing {
                    (Status::Closed, 0)
                } else {
                    match self.stack.send(self.owners[slot].client, request.data()) {
                        Ok(n) => {
                            self.counters.sent += n as u64;
                            (if n == 0 { Status::Again } else { Status::Ok }, n as u32)
                        }
                        Err(StackError::InvalidState) => (Status::Again, 0),
                        Err(_) => (Status::Closed, 0),
                    }
                }
            }
            Operation::ReceiveTcp => {
                match self.stack.receive(self.owners[slot].client, &mut data) {
                    Ok(n) => {
                        self.counters.received += n as u64;
                        (if n == 0 { Status::Again } else { Status::Ok }, n as u32)
                    }
                    Err(StackError::Finished) => (Status::Closed, 0),
                    Err(StackError::InvalidState) => (
                        if self.state(slot)? == ConnectionState::Closed {
                            Status::Closed
                        } else {
                            Status::Again
                        },
                        0,
                    ),
                    Err(_) => (Status::Closed, 0),
                }
            }
            Operation::CloseTcp => {
                self.stack
                    .close(self.owners[slot].client)
                    .map_err(|_| Fault::Transport)?;
                self.owners[slot].closing = true;
                self.counters.closed += 1;
                (Status::Ok, 0)
            }
            Operation::State => (Status::Ok, self.state(slot)? as u32),
            Operation::SendUdp => {
                if let Some(mac) = self.owners[slot].neighbor.lookup(d.ipv4, now) {
                    let mut frame = [0; FRAME_BYTES];
                    let id = self.identification;
                    let n = build_udp_ipv4(
                        UdpIpv4Tx {
                            destination_mac: mac,
                            source_mac: self.config.link.mac,
                            source_ipv4: self.config.link.ipv4,
                            destination_ipv4: d.ipv4,
                            source_port: 45000 + slot as u16,
                            destination_port: d.udp_port,
                            identification: id,
                            ttl: 64,
                            mtu: 1500,
                        },
                        request.data(),
                        &mut frame,
                    )
                    .map_err(|_| Fault::Configuration)?;
                    match self.stack.queue_control_frame(&frame[..n], now) {
                        Ok(_) => {
                            self.identification = self.identification.wrapping_add(1);
                            self.counters.udp_sent += 1;
                            (Status::Ok, h.argument)
                        }
                        Err(StackError::Busy) => (Status::Again, 0),
                        Err(_) => return Err(Fault::Transport),
                    }
                } else {
                    let mut frame = [0; 60];
                    let n = nieuw_os_net_protocol::arp::build_request(
                        Interface {
                            mac: self.config.link.mac,
                            ipv4: self.config.link.ipv4,
                        },
                        d.ipv4,
                        &mut frame,
                    )
                    .map_err(|_| Fault::Configuration)?;
                    if self.owners[slot].neighbor.begin(d.ipv4, now, 100).is_ok() {
                        if self.stack.queue_control_frame(&frame[..n], now).is_err() {
                            self.owners[slot].neighbor.cancel();
                        }
                    }
                    (Status::Again, 0)
                }
            }
            Operation::ReceiveUdp => {
                if let Some(n) = self.owners[slot].udp_len.take() {
                    data[..n].copy_from_slice(&self.owners[slot].udp[..n]);
                    self.owners[slot].udp.fill(0);
                    (Status::Ok, n as u32)
                } else {
                    (Status::Again, 0)
                }
            }
            Operation::Retire => {
                let owner = &mut self.owners[slot];
                owner.retired = true;
                owner.closing = true;
                owner.active = false;
                owner.udp.fill(0);
                owner.udp_len = None;
                owner.neighbor.cancel();
                self.stack
                    .close(owner.client)
                    .map_err(|_| Fault::Transport)?;
                (Status::Ok, 0)
            }
        };
        if status != Status::Ok {
            self.counters.rejected += u64::from(!matches!(
                status,
                Status::Again | Status::Busy | Status::Closed
            ));
        }
        let length = if status == Status::Ok
            && matches!(h.operation, Operation::ReceiveTcp | Operation::ReceiveUdp)
        {
            argument as usize
        } else {
            0
        };
        Ok(Some(Packet::response(h, status, argument, &data[..length])))
    }
}
