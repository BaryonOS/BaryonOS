#![no_std]
#![forbid(unsafe_code)]
//! Bounded ring-3 Ethernet/IPv4 active TCP client engine. No NIC, syscall,
//! allocator, host socket, authority grant or simulated clock lives here.
//! smoltcp writes a local TX slot; only the native caller can observe NIC
//! completion, and neither event means the remote application accepted data.

mod device;
pub use device::{FRAME_BYTES, PacketDevice, Transmission};
pub use smoltcp::iface::SocketStorage;
pub use smoltcp::socket::tcp::State;
use smoltcp::{
    iface::{Config as InterfaceConfig, Interface, SocketHandle, SocketSet},
    socket::tcp::{self, CongestionControl, Socket, SocketBuffer},
    time::{Duration, Instant},
    wire::{EthernetAddress, HardwareAddress, IpCidr, IpEndpoint, Ipv4Address},
};

pub const MONOTONIC_HZ: u64 = 100;
pub const MAX_CLIENTS: usize = 2;
pub const MAX_SOCKET_BYTES: usize = 8192;
const MAX_TIMEOUT_TICKS: u64 = 60 * MONOTONIC_HZ;

// smoltcp build.rs permits environment overrides. A changed compile-time
// resource contract fails the build, even when not using our build wrapper.
const _: () = {
    assert!(smoltcp::config::IFACE_MAX_ADDR_COUNT == 1);
    assert!(smoltcp::config::IFACE_NEIGHBOR_CACHE_COUNT == 4);
    assert!(smoltcp::config::IFACE_MAX_ROUTE_COUNT == 1);
    assert!(smoltcp::config::ASSEMBLER_MAX_SEGMENT_COUNT == 4);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfiguration,
    InvalidFrame,
    Busy,
    StaleTransmission,
    IdentifierExhausted,
    ClockWentBackwards,
    ClockOverflow,
    SocketLimit,
    InvalidHandle,
    InvalidState,
    Unaddressable,
    Finished,
}

/// A process-local index into this engine, not a kernel capability or IPC token.
/// Keep it with its creating engine; no slot removal/reuse is exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Client(SocketHandle);

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub mac: [u8; 6],
    pub ipv4: [u8; 4],
    pub prefix_len: u8,
    /// Production users must seed from their separately authorized entropy
    /// source. A fixed test seed is not suitable for Internet-facing ISNs.
    pub random_seed: u64,
}

/// All packet and socket bytes are borrowed from caller-owned fixed storage.
/// At most one ingress packet and one pending egress packet exist at a time.
pub struct Stack<'a> {
    interface: Interface,
    device: PacketDevice<'a>,
    sockets: SocketSet<'a>,
    slots: usize,
    clients: usize,
    last_ticks: u64,
}

fn instant(ticks: u64) -> Result<Instant, Error> {
    let micros = ticks
        .checked_mul(1_000_000 / MONOTONIC_HZ)
        .and_then(|value| i64::try_from(value).ok())
        // Leave headroom for upstream timer additions, not just the initial
        // conversion. Network input must not drive an Instant near i64::MAX.
        .filter(|value| *value <= i64::MAX / 2)
        .ok_or(Error::ClockOverflow)?;
    Ok(Instant::from_micros(micros))
}

impl<'a> Stack<'a> {
    pub fn new(
        config: Config,
        storage: &'a mut [SocketStorage<'a>],
        rx_frame: &'a mut [u8; FRAME_BYTES],
        tx_frame: &'a mut [u8; FRAME_BYTES],
        ticks: u64,
    ) -> Result<Self, Error> {
        let mac = EthernetAddress(config.mac);
        let ip = Ipv4Address::from(config.ipv4);
        if storage.is_empty()
            || storage.len() > MAX_CLIENTS
            || !mac.is_unicast()
            || mac.0 == [0; 6]
            || config.ipv4[0] == 0
            || config.ipv4[0] == 127
            || config.ipv4[0] >= 224
            || config.prefix_len == 0
            || config.prefix_len > 32
        {
            return Err(Error::InvalidConfiguration);
        }
        let now = instant(ticks)?;
        let slots = storage.len();
        let sockets = SocketSet::new(storage);
        if sockets.iter().next().is_some() {
            return Err(Error::InvalidConfiguration);
        }
        let mut device = PacketDevice::new(rx_frame, tx_frame);
        let mut settings = InterfaceConfig::new(HardwareAddress::Ethernet(mac));
        settings.random_seed = config.random_seed;
        let mut interface = Interface::new(settings, &mut device, now);
        interface.update_ip_addrs(|addresses| {
            // Capacity is asserted above at compile time and the list is empty.
            addresses
                .push(IpCidr::new(ip.into(), config.prefix_len))
                .unwrap();
        });
        Ok(Self {
            interface,
            device,
            sockets,
            slots,
            clients: 0,
            last_ticks: ticks,
        })
    }

    pub fn add_client(&mut self, rx: &'a mut [u8], tx: &'a mut [u8]) -> Result<Client, Error> {
        if rx.is_empty()
            || tx.is_empty()
            || rx.len() > MAX_SOCKET_BYTES
            || tx.len() > MAX_SOCKET_BYTES
        {
            return Err(Error::InvalidConfiguration);
        }
        if self.clients == self.slots {
            return Err(Error::SocketLimit);
        }
        let mut socket = Socket::new(SocketBuffer::new(rx), SocketBuffer::new(tx));
        socket.set_congestion_control(CongestionControl::Reno);
        // Fewer latency-dependent fixtures; still actual TCP sequencing/ACKs.
        socket.set_nagle_enabled(false);
        socket.set_ack_delay(None);
        let handle = self.sockets.add(socket);
        self.clients += 1;
        Ok(Client(handle))
    }

    pub fn connect(
        &mut self,
        client: Client,
        remote_ipv4: [u8; 4],
        remote_port: u16,
        local_port: u16,
        timeout_ticks: u64,
    ) -> Result<(), Error> {
        if timeout_ticks == 0 || timeout_ticks > MAX_TIMEOUT_TICKS {
            return Err(Error::InvalidConfiguration);
        }
        if remote_ipv4[0] == 0 || remote_ipv4[0] == 127 || remote_ipv4[0] >= 224 {
            return Err(Error::Unaddressable);
        }
        // An ambiguous four-tuple must not let a second socket capture the
        // first socket's replies. Distinct remote peers may share a local port.
        if self
            .sockets
            .iter()
            .any(|(handle, smoltcp::socket::Socket::Tcp(socket))| {
                handle != client.0
                    && socket.state() != State::Closed
                    && socket
                        .local_endpoint()
                        .is_some_and(|endpoint| endpoint.port == local_port)
                    && socket.remote_endpoint().is_some_and(|endpoint| {
                        endpoint.port == remote_port
                            && endpoint.addr == Ipv4Address::from(remote_ipv4).into()
                    })
            })
        {
            return Err(Error::Busy);
        }
        let socket = self
            .sockets
            .iter_mut()
            .find(|(handle, _)| *handle == client.0)
            .map(|(_, smoltcp::socket::Socket::Tcp(socket))| socket)
            .ok_or(Error::InvalidHandle)?;
        socket.set_timeout(Some(Duration::from_micros(timeout_ticks * 10_000)));
        let remote = IpEndpoint::new(Ipv4Address::from(remote_ipv4).into(), remote_port);
        socket
            .connect(self.interface.context(), remote, local_port)
            .map_err(|error| match error {
                tcp::ConnectError::InvalidState => Error::InvalidState,
                tcp::ConnectError::Unaddressable => Error::Unaddressable,
            })
    }

    fn socket(&self, client: Client) -> Result<&Socket<'a>, Error> {
        self.sockets
            .iter()
            .find(|(handle, _)| *handle == client.0)
            .map(|(_, smoltcp::socket::Socket::Tcp(socket))| socket)
            .ok_or(Error::InvalidHandle)
    }

    fn socket_mut(&mut self, client: Client) -> Result<&mut Socket<'a>, Error> {
        self.sockets
            .iter_mut()
            .find(|(handle, _)| *handle == client.0)
            .map(|(_, smoltcp::socket::Socket::Tcp(socket))| socket)
            .ok_or(Error::InvalidHandle)
    }

    pub fn state(&self, client: Client) -> Result<State, Error> {
        Ok(self.socket(client)?.state())
    }

    pub fn send(&mut self, client: Client, bytes: &[u8]) -> Result<usize, Error> {
        self.socket_mut(client)?
            .send_slice(bytes)
            .map_err(|_| Error::InvalidState)
    }

    pub fn receive(&mut self, client: Client, out: &mut [u8]) -> Result<usize, Error> {
        self.socket_mut(client)?
            .recv_slice(out)
            .map_err(|error| match error {
                tcp::RecvError::Finished => Error::Finished,
                tcp::RecvError::InvalidState => Error::InvalidState,
            })
    }

    pub fn close(&mut self, client: Client) -> Result<(), Error> {
        self.socket_mut(client)?.close();
        Ok(())
    }

    /// Explicit cancellation. The stack may still queue a TCP RST; the caller
    /// must drain or reset the native transport before releasing its authority.
    pub fn cancel(&mut self, client: Client) -> Result<(), Error> {
        self.socket_mut(client)?.abort();
        Ok(())
    }

    /// A bounded step: maintenance, at most one ingress, one bounded egress pass
    /// over at most two sockets. Caller must yield between steps and supply
    /// actual boot-monotonic ticks; repeated equal ticks never advance timers.
    pub fn poll(&mut self, ticks: u64) -> Result<(), Error> {
        if ticks < self.last_ticks {
            return Err(Error::ClockWentBackwards);
        }
        let now = instant(ticks)?;
        self.last_ticks = ticks;
        self.interface.poll_maintenance(now);
        self.interface
            .poll_ingress_single(now, &mut self.device, &mut self.sockets);
        self.interface
            .poll_egress(now, &mut self.device, &mut self.sockets);
        Ok(())
    }

    pub fn can_receive_frame(&self) -> bool {
        self.device.can_receive()
    }
    pub fn push_frame(&mut self, frame: &[u8]) -> Result<(), Error> {
        self.device.push(frame)
    }
    pub fn transmission(&self) -> Option<Transmission<'_>> {
        self.device.transmission()
    }
    /// Caller promises the exact queued frame completed at the native broker.
    /// No implicit retry/release on timeout; an uncertain device outcome must
    /// terminate/recover the caller rather than reusing a potentially DMA-live slot.
    pub fn complete_transmission(&mut self, id: u64) -> Result<(), Error> {
        self.device.complete(id)
    }
}
