#![no_std]
#![forbid(unsafe_code)]
//! A reusable userspace network service. Trusted startup binds two ingress
//! peers to fixed destination ranges; packet fields and app bodies grant nothing.
/// Native callers configure the DHCP mode without a direct net-config edge.
pub use nieuw_os_net_config::service::{ProbeTiming, RandomSource};
use nieuw_os_net_config::{
    dhcp_client::Lease,
    service::{DriveError, DriveOutcome, Error as DhcpError, Service as Dhcp},
};
use nieuw_os_net_protocol::{
    arp::{Interface, NeighborSlot},
    build_udp_ipv4,
    dns::{self, Address, Name, QueryType, Question, Resolution},
    parse,
    resolver::{Error as ResolveError, Resolver},
    UdpChecksum, UdpIpv4Tx,
};
use nieuw_os_net_stack::{
    Client, Config, Error as StackError, LinkConfig, SocketStorage, Stack, State, Transmission,
    FRAME_BYTES,
};
pub use nieuw_os_network_protocol as wire;
use wire::{ConnectionState, Operation, Packet, Status, DATA, MAX_NAME, MAX_RESOLVED};
pub const CLIENTS: usize = 2;
pub const SOCKET_BYTES: usize = 4096;
/// Upstream DNS queries one slot may start per lease configuration epoch.
/// shortcut: no refill on renewal, so a long unchanged lease can exhaust it;
/// replace with a time-based refill once native callers resolve routinely.
pub const DNS_QUERY_BUDGET: u16 = 32;
/// Monotonic ticks (`MONOTONIC_HZ`) from Resolve to Timeout, sent or not.
pub const DNS_TIMEOUT_TICKS: u64 = 3 * nieuw_os_net_stack::MONOTONIC_HZ;
/// Random DNS source ports start here; the low bit is the owner slot.
const DNS_PORT_BASE: u16 = 49152;
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
/// Trusted startup authority; DHCP supplies configuration, never destination rights.
#[derive(Clone, Copy)]
pub struct DhcpConfiguration {
    pub epoch: u64,
    pub pids: [u64; CLIENTS],
    pub link: LinkConfig,
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
    Dhcp(DhcpError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DnsDriveError<E> {
    Service(Fault),
    Random(E),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DhcpDriveError<E> {
    Service(Fault),
    Coordinator(DriveError<E>),
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
    /// Unpublished frames of a revoked owner dropped before reaching the NIC.
    pub revoked_frames_discarded: u64,
}
struct Owner {
    client: Client,
    request: u64,
    generation: u64,
    next_port: Option<u16>,
    token: u64,
    active: bool,
    closing: bool,
    deadline: u64,
    timed_out: bool,
    retired: bool,
    revoked: bool,
    udp: [u8; DATA],
    udp_len: Option<usize>,
    neighbor: Option<NeighborSlot>,
    dns: Option<DnsQuery>,
    dns_budget: u16,
}
struct DnsQuery {
    name: Name,
    deadline: u64,
    /// (DNS ID, local port) once `drive_dns` queued the query.
    sent: Option<(u16, u16)>,
    outcome: Option<Answer>,
}
/// Encoded Resolve reply: status, IPv4 bytes and their length.
type Answer = (Status, [u8; 4 * MAX_RESOLVED], usize);
/// Caller-owned resolver caches for a DHCP service, kept out of `Service` so
/// the static-mode service stays small. Private per slot: no cross-app
/// eviction, TTL probing or answers from a server the other slot is not granted.
pub struct DnsCache {
    resolvers: [Resolver; CLIENTS],
    /// Bumped by each new service using this storage and by any lease change
    /// of address, prefix, router or DNS servers; a bump flushes every entry.
    epoch: u64,
}
impl DnsCache {
    pub fn new() -> Result<Self, Fault> {
        let resolver =
            || Resolver::new(1, nieuw_os_net_stack::MONOTONIC_HZ).map_err(|_| Fault::Configuration);
        Ok(Self {
            resolvers: [resolver()?, resolver()?],
            epoch: 1,
        })
    }
    fn flush(&mut self) -> Result<(), Fault> {
        self.epoch = self.epoch.checked_add(1).ok_or(Fault::Exhausted)?;
        Ok(())
    }
}
// Both modes keep bounded storage inline: this no_std service has no allocator.
#[allow(clippy::large_enum_variant)]
enum Transport<'a> {
    Static(Stack<'a>),
    Dhcp(Dhcp<'a>),
}
impl<'a> core::ops::Deref for Transport<'a> {
    type Target = Stack<'a>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(stack) => stack,
            Self::Dhcp(service) => service.transport_snapshot(),
        }
    }
}
impl<'a> Transport<'a> {
    fn get_mut(&mut self, now: u64) -> Result<&mut Stack<'a>, Fault> {
        match self {
            Self::Static(stack) => Ok(stack),
            Self::Dhcp(service) => service.transport_mut(now).map_err(Fault::Dhcp),
        }
    }
}
pub struct Service<'a> {
    config: Configuration,
    stack: Transport<'a>,
    owners: [Owner; CLIENTS],
    last_tick: u64,
    identification: u16,
    counters: Counters,
    needs_poll: bool,
    lease: Option<Lease>,
    /// Present only in DHCP mode: static configuration has no DNS servers.
    dns_cache: Option<&'a mut DnsCache>,
}
impl<'a> Service<'a> {
    pub fn new(config: Configuration, buffers: Buffers<'a>, now: u64) -> Result<Self, Fault> {
        Self::new_inner(config, buffers, now, None)
    }
    /// `dns` may be reused storage; it is flushed before this service uses it.
    pub fn new_dhcp(
        config: DhcpConfiguration,
        buffers: Buffers<'a>,
        timing: ProbeTiming,
        dns: &'a mut DnsCache,
        now: u64,
    ) -> Result<Self, Fault> {
        Self::new_inner(
            Configuration {
                epoch: config.epoch,
                pids: config.pids,
                destinations: config.destinations,
                link: Config {
                    mac: config.link.mac,
                    random_seed: config.link.random_seed,
                    ipv4: [0; 4],
                    prefix_len: 0,
                },
            },
            buffers,
            now,
            Some((timing, dns)),
        )
    }
    fn new_inner(
        config: Configuration,
        buffers: Buffers<'a>,
        now: u64,
        dhcp: Option<(ProbeTiming, &'a mut DnsCache)>,
    ) -> Result<Self, Fault> {
        if config.epoch == 0
            || config.pids.contains(&0)
            || config.pids[0] == config.pids[1]
            || buffers.storage.len() != CLIENTS
        {
            return Err(Fault::Configuration);
        }
        let mask = if dhcp.is_some() {
            0
        } else if config.link.prefix_len == 0 || config.link.prefix_len > 32 {
            return Err(Fault::Configuration);
        } else {
            u32::MAX << (32 - config.link.prefix_len)
        };
        for d in config.destinations {
            if d.ipv4[0] == 0
                || d.ipv4[0] == 127
                || d.ipv4[0] >= 224
                || (dhcp.is_none()
                    && (d.ipv4 == config.link.ipv4
                        || (config.link.prefix_len < 31
                            && u32::from_be_bytes(d.ipv4) & !mask == 0)
                        || (config.link.prefix_len < 31
                            && u32::from_be_bytes(d.ipv4) & !mask == !mask)
                        || u32::from_be_bytes(d.ipv4) & mask
                            != u32::from_be_bytes(config.link.ipv4) & mask))
                || d.tcp_first == 0
                || d.tcp_count == 0
                || d.tcp_count > 8
                || d.tcp_first.checked_add(d.tcp_count - 1).is_none()
                || d.udp_port == 0
            {
                return Err(Fault::Configuration);
            }
        }
        let mut stack = if dhcp.is_some() {
            Stack::new_unconfigured(
                LinkConfig {
                    mac: config.link.mac,
                    random_seed: config.link.random_seed,
                },
                buffers.storage,
                buffers.rx_frame,
                buffers.tx_frame,
                now,
            )
        } else {
            Stack::new(
                config.link,
                buffers.storage,
                buffers.rx_frame,
                buffers.tx_frame,
                now,
            )
        }
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
        let dhcp_mode = dhcp.is_some();
        let make = |client, slot: u16| {
            Ok(Owner {
                client,
                request: 0,
                generation: 0,
                next_port: Some(40000 + slot),
                token: 0,
                active: false,
                closing: false,
                deadline: 0,
                timed_out: false,
                retired: false,
                revoked: false,
                udp: [0; DATA],
                udp_len: None,
                neighbor: if dhcp_mode {
                    None
                } else {
                    Some(NeighborSlot::new(local).map_err(|_| Fault::Configuration)?)
                },
                dns: None,
                dns_budget: DNS_QUERY_BUDGET,
            })
        };
        let (stack, dns_cache) = match dhcp {
            Some((timing, cache)) => {
                cache.flush()?;
                (
                    Transport::Dhcp(Dhcp::new(stack, timing, now).map_err(Fault::Dhcp)?),
                    Some(cache),
                )
            }
            None => (Transport::Static(stack), None),
        };
        Ok(Self {
            config,
            stack,
            owners: [make(clients[0], 0)?, make(clients[1], 1)?],
            last_tick: now,
            identification: 0,
            counters: Counters::default(),
            needs_poll: false,
            lease: None,
            dns_cache,
        })
    }
    fn time(&mut self, now: u64) -> Result<(), Fault> {
        if let Transport::Dhcp(service) = &mut self.stack {
            let result = service.configuration(now);
            self.sync_configuration(result.as_ref().ok().copied().flatten())?;
            result.map_err(Fault::Dhcp)?;
        }
        if now < self.last_tick || now > nieuw_os_net_stack::MAX_MONOTONIC_TICKS {
            Err(Fault::Clock)
        } else {
            if matches!(self.stack, Transport::Dhcp(_)) {
                self.last_tick = now;
            }
            Ok(())
        }
    }
    fn sync_configuration(&mut self, lease: Option<Lease>) -> Result<(), Fault> {
        let changed = self
            .lease
            .map(|l| (l.address, l.prefix_len, l.routers().first().copied()))
            != lease.map(|l| (l.address, l.prefix_len, l.routers().first().copied()));
        let dns_changed = changed
            || self.lease.as_ref().map(|l| l.dns_servers())
                != lease.as_ref().map(|l| l.dns_servers());
        if dns_changed {
            // Cache flush plus pending-query drop: no answer crosses a lease change.
            if let Some(cache) = self.dns_cache.as_deref_mut() {
                cache.flush()?;
            }
            for owner in &mut self.owners {
                owner.dns = None;
                owner.dns_budget = DNS_QUERY_BUDGET;
            }
        }
        if changed {
            for owner in &mut self.owners {
                owner.active = false;
                owner.token = 0;
                owner.closing = false;
                owner.timed_out = false;
                owner.udp.fill(0);
                owner.udp_len = None;
                owner.neighbor = lease
                    .map(|l| {
                        NeighborSlot::new(Interface {
                            mac: self.config.link.mac,
                            ipv4: l.address,
                        })
                    })
                    .transpose()
                    .map_err(|_| Fault::Configuration)?;
            }
        }
        self.config.link.ipv4 = lease.map_or([0; 4], |l| l.address);
        self.config.link.prefix_len = lease.map_or(0, |l| l.prefix_len);
        self.lease = lease;
        Ok(())
    }
    pub fn dhcp_configuration(&mut self, now: u64) -> Result<Option<Lease>, Fault> {
        self.time(now)?;
        Ok(self.lease)
    }
    pub fn link_up(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let Transport::Dhcp(service) = &mut self.stack else {
            return Err(Fault::Configuration);
        };
        service.automatic_link_up(now).map_err(Fault::Dhcp)?;
        self.time(now)
    }
    pub fn link_down(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let Transport::Dhcp(service) = &mut self.stack else {
            return Err(Fault::Configuration);
        };
        let result = service.link_down(now);
        self.time(now)?;
        result.map_err(Fault::Dhcp)
    }
    /// One bounded automatic coordinator step, with caller-provided entropy.
    /// Feed ingress and matching real completions first, then drive and poll.
    pub fn drive_dhcp<R: RandomSource>(
        &mut self,
        now: u64,
        random: &mut R,
    ) -> Result<DriveOutcome, DhcpDriveError<R::Error>> {
        self.time(now).map_err(DhcpDriveError::Service)?;
        let Transport::Dhcp(service) = &mut self.stack else {
            return Err(DhcpDriveError::Service(Fault::Configuration));
        };
        let result = service.drive(now, random);
        self.time(now).map_err(DhcpDriveError::Service)?;
        result.map_err(DhcpDriveError::Coordinator)
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
    /// Legacy static-mode snapshot; DHCP requires the timestamped API.
    pub fn transmission(&self) -> Option<Transmission<'_>> {
        match &self.stack {
            Transport::Static(stack) => stack.transmission(),
            Transport::Dhcp(_) => None,
        }
    }
    /// Legacy static-mode completion; DHCP requires the timestamped API.
    pub fn complete_transmission(&mut self, id: u64) -> Result<(), Fault> {
        if matches!(self.stack, Transport::Dhcp(_)) {
            return Err(Fault::Configuration);
        }
        self.complete_transmission_at(id, self.last_tick)
    }
    pub fn complete_transmission_at(&mut self, id: u64, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let result = match &mut self.stack {
            Transport::Static(stack) => stack
                .complete_transmission(id)
                .map_err(|_| Fault::Transport),
            Transport::Dhcp(service) => service.complete_transmission(id, now).map_err(Fault::Dhcp),
        };
        self.time(now)?;
        result
    }
    pub fn transmission_at(&mut self, now: u64) -> Result<Option<Transmission<'_>>, Fault> {
        self.time(now)?;
        Ok(self.stack.transmission())
    }
    /// Pending frame withdrawn by lease expiry or link-down while still queued.
    pub fn withheld_transmission_at(&mut self, now: u64) -> Result<Option<u64>, Fault> {
        self.time(now)?;
        let result = match &mut self.stack {
            Transport::Static(stack) => Ok(stack.withheld_transmission()),
            Transport::Dhcp(service) => service.withheld_transmission(now).map_err(Fault::Dhcp),
        };
        self.time(now)?;
        result
    }
    /// Adapter attests the withheld frame was never submitted to the broker.
    /// A submitted frame keeps its slot until `complete_transmission_at`.
    pub fn discard_withheld_transmission_at(&mut self, id: u64, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let result = match &mut self.stack {
            Transport::Static(stack) => stack
                .discard_withheld_transmission(id)
                .map_err(|_| Fault::Transport),
            Transport::Dhcp(service) => service
                .discard_withheld_transmission(id, now)
                .map_err(Fault::Dhcp),
        };
        self.time(now)?;
        result
    }
    pub fn poll(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        match &mut self.stack {
            Transport::Static(stack) => stack.poll(now).map_err(|_| Fault::Clock)?,
            Transport::Dhcp(service) => service.poll(now).map_err(Fault::Dhcp)?,
        }
        self.time(now)?;
        self.needs_poll = false;
        self.last_tick = now;
        // The OpenTcp deadline bounds connection establishment only. After the
        // handshake the stack's inactivity timeout (reset by peer traffic)
        // governs the session, so an active connection is never timed out.
        for owner in &mut self.owners {
            if owner.active
                && !owner.closing
                && now >= owner.deadline
                && matches!(
                    self.stack
                        .state(owner.client)
                        .map_err(|_| Fault::Transport)?,
                    State::SynSent | State::SynReceived
                )
            {
                self.stack
                    .get_mut(now)?
                    .cancel(owner.client)
                    .map_err(|_| Fault::Transport)?;
                owner.timed_out = true;
                owner.closing = true;
                self.needs_poll = true;
            }
        }
        Ok(())
    }
    pub fn ingress(&mut self, frame: &[u8], now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let dhcp_packet =
            parse(frame).is_ok_and(|p| p.source_port() == 67 && p.destination_port() == 68);
        if matches!(self.stack, Transport::Dhcp(_))
            && (dhcp_packet || frame.get(12..14) == Some(&[8, 6]))
        {
            let Transport::Dhcp(service) = &mut self.stack else {
                unreachable!()
            };
            let result = service.receive_frame(frame, now);
            self.time(now)?;
            result.map_err(Fault::Dhcp)?;
            if dhcp_packet {
                return Ok(());
            }
            // The coordinator already offered this ARP frame to the shared Stack.
            for owner in &mut self.owners {
                if let Some(neighbor) = &mut owner.neighbor {
                    let _ = neighbor.accept_reply(frame, now, 2000);
                }
            }
            return Ok(());
        }
        if matches!(self.stack, Transport::Dhcp(_)) && self.lease.is_none() {
            return Ok(());
        }
        if frame.len() >= 14 && frame[12..14] == [8, 6] {
            for owner in &mut self.owners {
                if let Some(neighbor) = &mut owner.neighbor {
                    let _ = neighbor.accept_reply(frame, now, 2000);
                }
            }
        }
        if frame.len() > 23 && frame[12..14] == [8, 0] && frame[23] == 17 {
            match parse(frame) {
                Ok(packet) => {
                    if self.accept_dns(&packet, now) {
                        return Ok(());
                    }
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
                            if owner.revoked {
                                // Well-formed but no longer authorized: a policy
                                // refusal, kept apart from malformed-frame evidence.
                                self.counters.rejected = self.counters.rejected.saturating_add(1);
                            } else if owner.udp_len.is_none() {
                                owner.udp[..packet.payload().len()]
                                    .copy_from_slice(packet.payload());
                                owner.udp_len = Some(packet.payload().len());
                                self.counters.udp_received =
                                    self.counters.udp_received.saturating_add(1);
                            } else {
                                self.counters.dropped_datagrams =
                                    self.counters.dropped_datagrams.saturating_add(1);
                            }
                            break;
                        }
                    }
                    if !accepted {
                        self.counters.malformed_frames =
                            self.counters.malformed_frames.saturating_add(1);
                    }
                }
                Err(_) => {
                    self.counters.malformed_frames =
                        self.counters.malformed_frames.saturating_add(1)
                }
            };
            return Ok(());
        }
        if !(14..=FRAME_BYTES).contains(&frame.len()) {
            self.counters.malformed_frames = self.counters.malformed_frames.saturating_add(1);
            return Ok(());
        }
        self.stack
            .get_mut(now)?
            .push_frame(frame)
            .map_err(|_| Fault::Transport)
    }
    pub fn retire_all(&mut self, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        for owner in &mut self.owners {
            owner.retired = true;
            owner.closing = true;
            owner.active = false;
            owner.udp.fill(0);
            owner.udp_len = None;
            owner.dns = None;
            if let Some(neighbor) = &mut owner.neighbor {
                neighbor.cancel();
            }
            self.stack
                .get_mut(now)?
                .close(owner.client)
                .map_err(|_| Fault::Transport)?;
        }
        self.needs_poll = true;
        Ok(())
    }
    /// Trusted adapter notification, never an app RPC: one peer's terminal
    /// channel failure retires only that owner's authority. The shared device
    /// transmission (possibly already published to the kernel) remains pinned
    /// until its matching real completion. A subsequent poll emits cancellation
    /// traffic when the bounded device slot becomes available.
    pub fn retire_slot(&mut self, slot: usize, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let owner = self.owners.get_mut(slot).ok_or(Fault::Configuration)?;
        owner.retired = true;
        owner.active = false;
        owner.closing = true;
        owner.token = 0;
        owner.udp.fill(0);
        owner.udp_len = None;
        owner.dns = None;
        if let Some(neighbor) = &mut owner.neighbor {
            neighbor.cancel();
        }
        self.stack
            .get_mut(now)?
            .cancel(owner.client)
            .map_err(|_| Fault::Transport)?;
        self.needs_poll = true;
        Ok(())
    }
    /// Trusted policy-authority notification, never an app RPC: withdraws one
    /// owner's destination grant for the rest of this epoch. A live connection
    /// is aborted (the stack queues a TCP RST) and buffered UDP is dropped.
    /// Afterwards every data/open operation of that owner answers `Access`;
    /// only State, CloseTcp and Retire stay available for cleanup. As with
    /// `retire_slot`, a frame already handed to the device stays pinned until
    /// its real completion; revocation never fakes a discard of DMA-live data.
    pub fn revoke(&mut self, slot: usize, now: u64) -> Result<(), Fault> {
        self.time(now)?;
        let owner = self.owners.get_mut(slot).ok_or(Fault::Configuration)?;
        owner.revoked = true;
        owner.closing = true;
        owner.udp.fill(0);
        owner.udp_len = None;
        owner.dns = None;
        if let Some(neighbor) = &mut owner.neighbor {
            neighbor.cancel();
        }
        self.stack
            .get_mut(now)?
            .cancel(owner.client)
            .map_err(|_| Fault::Transport)?;
        self.needs_poll = true;
        Ok(())
    }
    /// Revocation fence for the shared TX slot. The caller promises the
    /// current slot frame was never handed to the device (no transmission is
    /// outstanding). A frame built for a revoked owner before the revocation
    /// is dropped instead of published; only a payload-free TCP RST of that
    /// owner (the abort itself) may still leave. Returns whether it dropped.
    pub fn discard_revoked_unpublished(&mut self) -> Result<bool, Fault> {
        let Some(frame) = self.stack.transmission() else {
            return Ok(false);
        };
        let id = frame.id;
        let Some(slot) = frame_owner(frame.bytes) else {
            return Ok(false);
        };
        if !self.owners[slot].revoked {
            return Ok(false);
        }
        self.complete_transmission_at(id, self.last_tick)?;
        self.counters.revoked_frames_discarded =
            self.counters.revoked_frames_discarded.saturating_add(1);
        self.needs_poll = true;
        Ok(true)
    }
    pub fn revoked_mask(&self) -> u64 {
        u64::from(self.owners[0].revoked) | u64::from(self.owners[1].revoked) << 1
    }
    pub fn clean(&self) -> bool {
        !self.needs_poll
            && self.retired_mask() == 3
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
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(None);
        }
        let request = match Packet::decode_request(bytes) {
            Ok(p) => p,
            Err(_) => {
                self.counters.rejected = self.counters.rejected.saturating_add(1);
                return Ok(Packet::refusal_header(bytes, self.config.epoch)
                    .map(|h| Packet::response(h, Status::Malformed, 0, &[])));
            }
        };
        let mut h = request.header();
        self.counters.requests = self.counters.requests.saturating_add(1);
        let refusal = |h, status| Some(Packet::response(h, status, 0, &[]));
        if h.epoch != self.config.epoch {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Stale));
        }
        if h.request <= self.owners[slot].request {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Replay));
        }
        self.owners[slot].request = h.request;
        if self.owners[slot].retired {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Closed));
        }
        if self.owners[slot].revoked
            && !matches!(
                h.operation,
                Operation::State | Operation::CloseTcp | Operation::Retire
            )
        {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Access));
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
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Access));
        }
        if matches!(
            h.operation,
            Operation::SendTcp | Operation::ReceiveTcp | Operation::CloseTcp | Operation::State
        ) && (!self.owners[slot].active || h.connection != self.owners[slot].token)
        {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Stale));
        }
        if matches!(self.stack, Transport::Dhcp(_))
            && self.lease.is_none()
            && matches!(
                h.operation,
                Operation::OpenTcp
                    | Operation::SendUdp
                    | Operation::ReceiveUdp
                    | Operation::Resolve
            )
        {
            return Ok(refusal(h, Status::Again));
        }
        // Resolution uses only the lease's DNS servers, and only one this slot
        // already holds a UDP/53 grant for. No lease DNS option means no
        // resolver: there is no default or fallback server.
        if h.operation == Operation::Resolve
            && (d.udp_port != 53
                || !self
                    .lease
                    .is_some_and(|l| l.dns_servers().contains(&d.ipv4)))
        {
            self.counters.rejected = self.counters.rejected.saturating_add(1);
            return Ok(refusal(h, Status::Access));
        }
        if self.lease.is_some()
            && matches!(
                h.operation,
                Operation::OpenTcp | Operation::SendUdp | Operation::Resolve
            )
        {
            let mask = u32::MAX << (32 - self.config.link.prefix_len);
            if d.ipv4 == self.config.link.ipv4
                || (self.config.link.prefix_len < 31
                    && u32::from_be_bytes(d.ipv4) & mask
                        == u32::from_be_bytes(self.config.link.ipv4) & mask
                    && (u32::from_be_bytes(d.ipv4) & !mask == 0
                        || u32::from_be_bytes(d.ipv4) & !mask == !mask))
            {
                return Ok(refusal(h, Status::Access));
            }
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
                let Some(generation) = self.owners[slot]
                    .generation
                    .checked_add(1)
                    .filter(|g| *g < u64::MAX >> 8)
                else {
                    return Ok(refusal(h, Status::Exhausted));
                };
                // Each app has a disjoint bounded port namespace. Exhausting
                // one namespace is an app result, never a service fault, and
                // cannot consume the other owner's future connection budget.
                let Some(port) = self.owners[slot].next_port else {
                    return Ok(refusal(h, Status::Exhausted));
                };
                let deadline = now.checked_add(u64::from(h.argument)).ok_or(Fault::Clock)?;
                match self.stack.get_mut(now)?.connect(
                    self.owners[slot].client,
                    h.address,
                    h.port,
                    port,
                    u64::from(h.argument),
                ) {
                    Ok(()) => {
                        let owner = &mut self.owners[slot];
                        owner.generation = generation;
                        owner.next_port = port.checked_add(2);
                        owner.token = (generation << 8) | (slot as u64 + 1);
                        owner.active = true;
                        owner.closing = false;
                        owner.deadline = deadline;
                        owner.timed_out = false;
                        self.needs_poll = true;
                        h.connection = owner.token;
                        self.counters.opened = self.counters.opened.saturating_add(1);
                        (Status::Ok, ConnectionState::Connecting as u32)
                    }
                    Err(_) => (Status::Busy, 0),
                }
            }
            Operation::SendTcp => {
                if self.owners[slot].closing {
                    (Status::Closed, 0)
                } else {
                    match self
                        .stack
                        .get_mut(now)?
                        .send(self.owners[slot].client, request.data())
                    {
                        Ok(n) => {
                            self.counters.sent = self.counters.sent.saturating_add(n as u64);
                            (if n == 0 { Status::Again } else { Status::Ok }, n as u32)
                        }
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
            }
            Operation::ReceiveTcp => {
                match self
                    .stack
                    .get_mut(now)?
                    .receive(self.owners[slot].client, &mut data)
                {
                    Ok(n) => {
                        self.counters.received = self.counters.received.saturating_add(n as u64);
                        (if n == 0 { Status::Again } else { Status::Ok }, n as u32)
                    }
                    Err(StackError::Finished) => (Status::Closed, 0),
                    Err(StackError::InvalidState) => (
                        if matches!(
                            self.state(slot)?,
                            ConnectionState::Closed | ConnectionState::TimedOut
                        ) {
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
                    .get_mut(now)?
                    .close(self.owners[slot].client)
                    .map_err(|_| Fault::Transport)?;
                self.owners[slot].closing = true;
                self.needs_poll = true;
                self.counters.closed = self.counters.closed.saturating_add(1);
                (Status::Ok, 0)
            }
            Operation::State => (Status::Ok, self.state(slot)? as u32),
            Operation::SendUdp => {
                let port = d.udp_port;
                match self.send_udp(slot, 45000 + slot as u16, port, request.data(), now)? {
                    Status::Ok => (Status::Ok, h.argument),
                    status => (status, 0),
                }
            }
            Operation::Resolve => self.resolve(slot, request.data(), &mut data, now)?,
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
                owner.dns = None;
                if let Some(neighbor) = &mut owner.neighbor {
                    neighbor.cancel();
                }
                self.stack
                    .get_mut(now)?
                    .close(owner.client)
                    .map_err(|_| Fault::Transport)?;
                self.needs_poll = true;
                (Status::Ok, 0)
            }
        };
        if status != Status::Ok {
            self.counters.rejected = self.counters.rejected.saturating_add(u64::from(!matches!(
                status,
                Status::Again | Status::Busy | Status::Closed
            )));
        }
        let length = if status == Status::Ok
            && matches!(
                h.operation,
                Operation::ReceiveTcp | Operation::ReceiveUdp | Operation::Resolve
            ) {
            argument as usize
        } else {
            0
        };
        Ok(Some(Packet::response(h, status, argument, &data[..length])))
    }
}

impl Service<'_> {
    /// One IPv4 datagram from `source_port` to this slot's granted destination,
    /// with ARP resolution of the next hop. `Again` while ARP is pending.
    fn send_udp(
        &mut self,
        slot: usize,
        source_port: u16,
        destination_port: u16,
        payload: &[u8],
        now: u64,
    ) -> Result<Status, Fault> {
        let d = self.config.destinations[slot];
        let mask = u32::MAX << (32 - self.config.link.prefix_len);
        let next_hop = if u32::from_be_bytes(d.ipv4) & mask
            == u32::from_be_bytes(self.config.link.ipv4) & mask
        {
            d.ipv4
        } else if let Some(gateway) = self.lease.and_then(|l| l.routers().first().copied()) {
            gateway
        } else {
            return Ok(Status::Again);
        };
        if let Some(mac) = self.owners[slot]
            .neighbor
            .as_mut()
            .and_then(|n| n.lookup(next_hop, now))
        {
            let mut frame = [0; FRAME_BYTES];
            let id = self.identification;
            let n = build_udp_ipv4(
                UdpIpv4Tx {
                    destination_mac: mac,
                    source_mac: self.config.link.mac,
                    source_ipv4: self.config.link.ipv4,
                    destination_ipv4: d.ipv4,
                    source_port,
                    destination_port,
                    identification: id,
                    ttl: 64,
                    mtu: 1500,
                },
                payload,
                &mut frame,
            )
            .map_err(|_| Fault::Configuration)?;
            match self
                .stack
                .get_mut(now)?
                .queue_control_frame(&frame[..n], now)
            {
                Ok(_) => {
                    self.identification = self.identification.wrapping_add(1);
                    self.counters.udp_sent = self.counters.udp_sent.saturating_add(1);
                    Ok(Status::Ok)
                }
                Err(StackError::Busy) => Ok(Status::Again),
                Err(_) => Err(Fault::Transport),
            }
        } else {
            let mut frame = [0; 60];
            let n = nieuw_os_net_protocol::arp::build_request(
                Interface {
                    mac: self.config.link.mac,
                    ipv4: self.config.link.ipv4,
                },
                next_hop,
                &mut frame,
            )
            .map_err(|_| Fault::Configuration)?;
            if self.owners[slot]
                .neighbor
                .as_mut()
                .is_some_and(|n| n.begin(next_hop, now, 100).is_ok())
            {
                match self
                    .stack
                    .get_mut(now)?
                    .queue_control_frame(&frame[..n], now)
                {
                    Ok(_) => {}
                    Err(StackError::Busy) => {
                        if let Some(neighbor) = &mut self.owners[slot].neighbor {
                            neighbor.cancel();
                        }
                    }
                    Err(_) => return Err(Fault::Transport),
                }
            }
            Ok(Status::Again)
        }
    }

    /// Resolve for an authorized slot (grant and lease checked by `handle`).
    /// Answers come from this slot's cache or its single pending query; a new
    /// query is only recorded here and sent by `drive_dns` with fresh entropy.
    /// A resolved address grants nothing: OpenTcp/SendUdp still compare
    /// against the slot's fixed `Destination`, never against DNS results.
    fn resolve(
        &mut self,
        slot: usize,
        hostname: &[u8],
        out: &mut [u8; DATA],
        now: u64,
    ) -> Result<(Status, u32), Fault> {
        let Some(name) = core::str::from_utf8(hostname)
            .ok()
            .and_then(|text| Name::from_ascii(text).ok())
        else {
            return Ok((Status::Malformed, 0));
        };
        let Some(cache) = self.dns_cache.as_deref_mut() else {
            return Ok((Status::Access, 0));
        };
        let owner = &mut self.owners[slot];
        let (status, addresses, length) = match &mut owner.dns {
            Some(query) if query.name == name => {
                if let Some(outcome) = query.outcome.take() {
                    owner.dns = None;
                    outcome
                } else if now >= query.deadline {
                    owner.dns = None;
                    return Ok((Status::Timeout, 0));
                } else {
                    return Ok((Status::Again, 0));
                }
            }
            pending => {
                if let Some(cached) =
                    cache.resolvers[slot].lookup(name, QueryType::A, cache.epoch, now)
                {
                    answer(cached)
                } else if pending
                    .as_ref()
                    .is_some_and(|q| q.outcome.is_none() && now < q.deadline)
                {
                    // One live upstream query per slot; a finished or expired
                    // one for another name is abandoned and replaced.
                    return Ok((Status::Busy, 0));
                } else if owner.dns_budget == 0 {
                    return Ok((Status::Exhausted, 0));
                } else {
                    owner.dns_budget -= 1;
                    owner.dns = Some(DnsQuery {
                        name,
                        deadline: now.checked_add(DNS_TIMEOUT_TICKS).ok_or(Fault::Clock)?,
                        sent: None,
                        outcome: None,
                    });
                    return Ok((Status::Again, 0));
                }
            }
        };
        out[..length].copy_from_slice(&addresses[..length]);
        Ok((status, length as u32))
    }

    /// Consume a reply only when it matches a sent query's exact transport
    /// binding: granted lease server, port 53, our address, the random local
    /// port, a UDP checksum and the deadline. Wrong ID/question or malformed
    /// bytes keep the query pending and leave the cache untouched.
    fn accept_dns(&mut self, packet: &nieuw_os_net_protocol::UdpDatagram<'_>, now: u64) -> bool {
        let Some(cache) = self.dns_cache.as_deref_mut() else {
            return false;
        };
        for (i, owner) in self.owners.iter_mut().enumerate() {
            let Some(query) = &mut owner.dns else {
                continue;
            };
            let Some((id, port)) = query.sent else {
                continue;
            };
            if query.outcome.is_some()
                || owner.revoked
                || owner.retired
                || packet.destination_mac() != self.config.link.mac
                || packet.destination_address() != self.config.link.ipv4
                || packet.source_address() != self.config.destinations[i].ipv4
                || packet.source_port() != 53
                || packet.destination_port() != port
            {
                continue;
            }
            if packet.checksum() == UdpChecksum::Absent || now >= query.deadline {
                self.counters.malformed_frames = self.counters.malformed_frames.saturating_add(1);
                return true;
            }
            let question = Question {
                id,
                name: query.name,
                kind: QueryType::A,
            };
            match cache.resolvers[i].accept(packet.payload(), question, cache.epoch, now) {
                Ok(resolution) => query.outcome = Some(answer(Ok(resolution))),
                Err(
                    error @ (ResolveError::NxDomain
                    | ResolveError::ServFail
                    | ResolveError::Dns(
                        dns::Error::TruncatedResponse
                        | dns::Error::ResponseCode(_)
                        | dns::Error::NoAddress,
                    )),
                ) => query.outcome = Some(answer(Err(error))),
                Err(_) => {
                    self.counters.malformed_frames =
                        self.counters.malformed_frames.saturating_add(1)
                }
            }
            return true;
        }
        false
    }

    /// Queue recorded Resolve queries, each with a fresh DNS ID and source port
    /// from the caller's already-authorized entropy. No entropy, no query.
    /// Returns how many queries were queued in this bounded step.
    pub fn drive_dns<R: RandomSource>(
        &mut self,
        now: u64,
        random: &mut R,
    ) -> Result<usize, DnsDriveError<R::Error>> {
        self.time(now).map_err(DnsDriveError::Service)?;
        if self.lease.is_none() {
            return Ok(0);
        }
        let mut queued = 0;
        for slot in 0..CLIENTS {
            let owner = &self.owners[slot];
            let Some(query) = &owner.dns else {
                continue;
            };
            if query.sent.is_some() || now >= query.deadline || owner.revoked || owner.retired {
                continue;
            }
            let value = random.random_u32().map_err(DnsDriveError::Random)?;
            let id = value as u16;
            // 13 random bits; the low bit names the slot (see `frame_owner`).
            let port = DNS_PORT_BASE | (((value >> 16) as u16) & 0x3ffe) | slot as u16;
            let question = Question {
                id,
                name: query.name,
                kind: QueryType::A,
            };
            let mut message = [0; 16 + MAX_NAME + 2];
            let n = dns::build_query(question, &mut message)
                .map_err(|_| DnsDriveError::Service(Fault::Configuration))?;
            if self
                .send_udp(slot, port, 53, &message[..n], now)
                .map_err(DnsDriveError::Service)?
                == Status::Ok
            {
                if let Some(query) = &mut self.owners[slot].dns {
                    query.sent = Some((id, port));
                }
                queued += 1;
            }
        }
        Ok(queued)
    }
}

/// Map one resolver result to the Resolve reply: IPv4 addresses only.
fn answer(result: Result<Resolution, ResolveError>) -> Answer {
    let mut out = [0; 4 * MAX_RESOLVED];
    let mut length = 0;
    let status = match result {
        Ok(resolution) => {
            for address in resolution.addresses() {
                if let (Address::Ipv4(ipv4), true) = (address, length < out.len()) {
                    out[length..length + 4].copy_from_slice(ipv4);
                    length += 4;
                }
            }
            if length == 0 {
                Status::NotFound
            } else {
                Status::Ok
            }
        }
        Err(ResolveError::NxDomain | ResolveError::Dns(dns::Error::NoAddress)) => Status::NotFound,
        Err(_) => Status::ServerFailure,
    };
    (status, out, length)
}

/// Which owner a locally built IPv4 frame speaks for, from its source port:
/// TCP ports are 40000 + slot + 2k up to u16::MAX (see `next_port`), UDP
/// ports 45000 + slot, DNS query ports 49152.. with the slot in the low bit;
/// the protocol byte keeps the TCP and UDP ranges apart.
/// A payload-free TCP RST belongs to nobody: it only ends a connection.
fn frame_owner(frame: &[u8]) -> Option<usize> {
    if frame.len() < 34 || frame[12..14] != [8, 0] {
        return None;
    }
    let ip = &frame[14..];
    let header = usize::from(ip[0] & 15) * 4;
    let total = usize::from(u16::from_be_bytes([ip[2], ip[3]]));
    if ip[0] >> 4 != 4 || header < 20 || total > ip.len() || total < header + 8 {
        return None;
    }
    let l4 = &ip[header..total];
    let port = u16::from_be_bytes([l4[0], l4[1]]);
    match ip[9] {
        6 if l4.len() >= 20 => {
            let data = usize::from(l4[12] >> 4) * 4;
            if l4[13] & 4 != 0 && l4.len() <= data {
                return None;
            }
            (port >= 40000).then(|| usize::from(port - 40000) % CLIENTS)
        }
        17 if port >= DNS_PORT_BASE => Some(usize::from(port - DNS_PORT_BASE) % CLIENTS),
        17 => (45000..45000 + CLIENTS as u16)
            .contains(&port)
            .then(|| usize::from(port - 45000)),
        _ => None,
    }
}
