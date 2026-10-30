use crate::{
    dhcp_client::{self, Client, Lease, Phase},
    time::{self, Clock},
};
use nieuw_os_net_protocol::{UdpIpv4Tx, build_udp_ipv4, parse};
use nieuw_os_net_stack::{
    AddressArp, AddressConfiguration, ArpObservation, Error as StackError, FRAME_BYTES, Stack,
    Transmission,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Stack(StackError),
    Client(dhcp_client::Error),
    Packet(nieuw_os_net_protocol::Error),
    Encode(nieuw_os_net_protocol::EncodeError),
    ClockFault {
        clock: time::Error,
        withdrawal: Option<StackError>,
    },
    InvalidTiming,
    AddressConflict,
    ConflictRateLimit,
    MissingServerNeighbor,
    TransportUncertain,
}

impl From<StackError> for Error {
    fn from(error: StackError) -> Self {
        Self::Stack(error)
    }
}

impl From<dhcp_client::Error> for Error {
    fn from(error: dhcp_client::Error) -> Self {
        Self::Client(error)
    }
}

/// Caller samples uniformly using its authorized entropy source. These are
/// bounded delays, NOT a trusted-RNG token or a hidden native RNG fallback.
#[derive(Clone, Copy, Debug)]
pub struct ProbeTiming {
    initial: u64,
    between: [u64; 2],
}

impl ProbeTiming {
    pub fn new(initial: u64, second: u64, third: u64) -> Result<Self, Error> {
        if initial > 100 || !(100..=200).contains(&second) || !(100..=200).contains(&third) {
            return Err(Error::InvalidTiming);
        }
        Ok(Self {
            initial,
            between: [second, third],
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Dhcp,
    Probe,
    Announcement,
    Reply,
    Retired,
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    id: u64,
    kind: Kind,
}

#[derive(Clone, Copy, Debug)]
struct Probe {
    lease: Lease,
    sent: u8,
    announcements: u8,
    due: u64,
}

/// One trusted service-loop owner. No raw device, root/SDK ABI, caller issuer,
/// transport authentication or app authority is created by this source API.
pub struct Service<'a> {
    stack: Stack<'a>,
    client: Client,
    clock: Clock,
    timing: ProbeTiming,
    pending: Option<Pending>,
    probing: Option<Probe>,
    active: Option<Lease>,
    apply_pending: Option<Lease>,
    server_neighbor: Option<([u8; 4], [u8; 6])>,
    fenced: bool,
    terminal: Option<Error>,
    conflicts: u8,
    retry_after: u64,
}

impl<'a> Service<'a> {
    pub fn new(mut stack: Stack<'a>, timing: ProbeTiming, ticks: u64) -> Result<Self, Error> {
        if stack.configuration().is_some() || !stack.can_queue_control_frame() {
            return Err(StackError::Busy.into());
        }
        let clock = Clock::new(ticks).map_err(|clock| Error::ClockFault {
            clock,
            withdrawal: None,
        })?;
        stack.poll(ticks)?;
        let client = Client::new(stack.hardware_address(), time::seconds(ticks).unwrap())?;
        Ok(Self {
            stack,
            client,
            clock,
            timing,
            pending: None,
            probing: None,
            active: None,
            apply_pending: None,
            server_neighbor: None,
            fenced: true,
            terminal: None,
            conflicts: 0,
            retry_after: 0,
        })
    }

    fn fence(&mut self, ticks: u64) -> Result<(), StackError> {
        self.active = None;
        self.probing = None;
        self.apply_pending = None;
        self.server_neighbor = None;
        if let Some(pending) = &mut self.pending {
            pending.kind = Kind::Retired;
        }
        self.fenced = true;
        self.stack.withdraw_configuration(ticks)
    }

    fn now(&mut self, ticks: u64) -> Result<u64, Error> {
        if let Some(error) = self.terminal {
            return Err(error);
        }
        let now = match self.clock.observe(ticks) {
            Ok(now) => now,
            Err(clock) => {
                let withdrawal = self.fence(ticks).err();
                let error = Error::ClockFault { clock, withdrawal };
                self.terminal = Some(error);
                return Err(error);
            }
        };
        let phase = self.client.tick(now)?;
        if !self.fenced
            && matches!(
                phase,
                Phase::Idle | Phase::Expired | Phase::TimedOut | Phase::Invalidated
            )
        {
            self.fence(ticks)?;
        }
        Ok(now)
    }

    /// Caller supplies fresh authorized entropy, not a persisted/reused XID.
    /// Retry scheduling remains with the service loop; Busy never builds a
    /// second frame or changes the conservative first-request lease clock.
    pub fn start(&mut self, xid: u32, ticks: u64, timeout_seconds: u64) -> Result<(), Error> {
        let now = self.now(ticks)?;
        if ticks < self.retry_after {
            return Err(Error::ConflictRateLimit);
        }
        self.client.start(xid, now, timeout_seconds)?;
        self.fenced = false;
        Ok(())
    }

    pub fn begin_renewal(&mut self, xid: u32, ticks: u64) -> Result<(), Error> {
        let now = self.now(ticks)?;
        self.client.begin_renewal(xid, now)?;
        Ok(())
    }

    pub fn phase(&mut self, ticks: u64) -> Result<Phase, Error> {
        let now = self.now(ticks)?;
        Ok(self.client.tick(now)?)
    }

    /// DNS and route information are usable only together with the actual
    /// applied address epoch. Absence of DNS is not a fallback-server grant.
    pub fn configuration(&mut self, ticks: u64) -> Result<Option<Lease>, Error> {
        self.now(ticks)?;
        Ok(self.active)
    }

    pub fn send_dhcp(&mut self, ticks: u64) -> Result<u64, Error> {
        let now = self.now(ticks)?;
        if !self.stack.can_queue_control_frame() {
            return Err(StackError::Busy.into());
        }
        let mut staged = self.client.clone();
        let mut payload = [0; FRAME_BYTES - 42];
        let outbound = staged.build(&mut payload, now)?;
        let destination_mac = if outbound.destination == [255; 4] {
            [255; 6]
        } else {
            self.server_neighbor
                .filter(|(ip, _)| *ip == outbound.destination)
                .map(|(_, mac)| mac)
                .ok_or(Error::MissingServerNeighbor)?
        };
        let mut frame = [0; FRAME_BYTES];
        let length = build_udp_ipv4(
            UdpIpv4Tx {
                source_mac: self.stack.hardware_address(),
                destination_mac,
                source_ipv4: outbound.source,
                destination_ipv4: outbound.destination,
                source_port: 68,
                destination_port: 67,
                identification: 0,
                ttl: 64,
                mtu: 1500,
            },
            &payload[..outbound.length],
            &mut frame,
        )
        .map_err(Error::Encode)?;
        let id = self.stack.queue_control_frame(&frame[..length], ticks)?;
        self.client = staged;
        self.pending = Some(Pending {
            id,
            kind: Kind::Dhcp,
        });
        Ok(id)
    }

    pub fn receive_frame(&mut self, frame: &[u8], ticks: u64) -> Result<(), Error> {
        let now = self.now(ticks)?;
        if frame.get(12..14) == Some(&[8, 6]) {
            let address = self
                .probing
                .map(|probe| probe.lease.address)
                .or(self.active.map(|lease| lease.address));
            if let Some(address) = address {
                let observation = self.stack.observe_address_arp(frame, address)?;
                let conflict = observation == ArpObservation::ForeignClaim
                    || (observation == ArpObservation::ForeignProbe && self.active.is_none());
                if conflict {
                    if self.client.candidate(now)?.is_some() {
                        self.client.reject_address_conflict(now)?;
                    } else {
                        self.client.link_down(now)?;
                    }
                    self.conflicts = self.conflicts.saturating_add(1);
                    self.retry_after =
                        self.deadline(ticks, if self.conflicts >= 10 { 6000 } else { 1000 })?;
                    self.fence(ticks)?;
                    return Err(Error::AddressConflict);
                }
                if observation == ArpObservation::ForeignProbe && self.active.is_some() {
                    let id = self.stack.queue_address_probe_reply(frame, ticks)?;
                    self.pending = Some(Pending {
                        id,
                        kind: Kind::Reply,
                    });
                    return Ok(());
                }
            }
            if self.active.is_some() {
                self.stack.push_frame(frame)?;
            }
            return Ok(());
        }
        if frame.get(12..14) == Some(&[8, 0]) && frame.get(23) == Some(&6) {
            if self.active.is_none() {
                return Err(StackError::NotConfigured.into());
            }
            self.stack.push_frame(frame)?;
            return Ok(());
        }
        let packet = parse(frame).map_err(Error::Packet)?;
        let mac = packet.source_mac();
        if mac == [0; 6] || mac[0] & 1 != 0 {
            return Err(StackError::InvalidFrame.into());
        }
        let phase = self.client.accept(&packet, now)?;
        if phase == Phase::Invalidated {
            self.fence(ticks)?;
        } else {
            self.server_neighbor = Some((packet.source_address(), mac));
            if phase == Phase::CheckingAddress {
                let lease = self
                    .client
                    .candidate(now)?
                    .ok_or(StackError::InvalidState)?;
                self.check_lease_times(lease, ticks)?;
                self.probing = Some(Probe {
                    lease,
                    sent: 0,
                    announcements: 0,
                    due: self.deadline(ticks, self.timing.initial)?,
                });
            } else if matches!(phase, Phase::Bound | Phase::RenewalDue) {
                let lease = self.client.lease(now)?.ok_or(StackError::InvalidState)?;
                self.check_lease_times(lease, ticks)?;
                let desired = Self::address(lease);
                if self.stack.configuration() == Some(desired) {
                    self.active = Some(lease);
                } else {
                    // A changed route/prefix is a new applied epoch. Retire
                    // old sockets/TX first; do not inherit DNS prematurely.
                    self.active = None;
                    self.probing = None;
                    if let Some(pending) = &mut self.pending {
                        pending.kind = Kind::Retired;
                    }
                    self.stack.withdraw_configuration(ticks)?;
                    self.apply_pending = Some(lease);
                }
            }
        }
        Ok(())
    }

    fn address(lease: Lease) -> AddressConfiguration {
        AddressConfiguration {
            ipv4: lease.address,
            prefix_len: lease.prefix_len,
            gateway: lease.routers().first().copied(),
        }
    }

    fn deadline(&mut self, ticks: u64, delay: u64) -> Result<u64, Error> {
        match ticks
            .checked_add(delay)
            .filter(|sum| time::seconds(*sum).is_ok())
        {
            Some(deadline) => Ok(deadline),
            None => {
                let withdrawal = self.fence(ticks).err();
                let error = Error::ClockFault {
                    clock: time::Error::ClockOverflow,
                    withdrawal,
                };
                self.terminal = Some(error);
                Err(error)
            }
        }
    }

    fn check_lease_times(&mut self, lease: Lease, ticks: u64) -> Result<(), Error> {
        for seconds in [lease.renew_at, lease.rebind_at, lease.expires_at] {
            if let Err(clock) = time::ticks(seconds) {
                let withdrawal = self.fence(ticks).err();
                let error = Error::ClockFault { clock, withdrawal };
                self.terminal = Some(error);
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn poll(&mut self, ticks: u64) -> Result<(), Error> {
        self.now(ticks)?;
        if self.stack.can_queue_control_frame() {
            if let Some(lease) = self.apply_pending {
                self.stack
                    .apply_configuration(Self::address(lease), ticks)?;
                self.active = Some(lease);
                self.apply_pending = None;
            }
            if let Some(probe) = self.probing
                && ticks >= probe.due
            {
                let (arp, kind) = if probe.sent < 3 {
                    (AddressArp::Probe, Kind::Probe)
                } else {
                    (AddressArp::Announcement, Kind::Announcement)
                };
                let id = self
                    .stack
                    .queue_address_arp(probe.lease.address, arp, ticks)?;
                self.pending = Some(Pending { id, kind });
            }
        }
        self.stack.poll(ticks)?;
        Ok(())
    }

    /// Only actual matching backend completion belongs here. Encoding/offer,
    /// EAGAIN or an unknown outcome is NEVER completion. No IPC exposes this.
    pub fn complete_transmission(&mut self, id: u64, ticks: u64) -> Result<(), Error> {
        let now = self.now(ticks)?;
        self.stack.complete_transmission(id)?;
        let pending = self.pending.take();
        if let Some(Pending { id: expected, kind }) = pending {
            assert_eq!(expected, id);
            match kind {
                Kind::Dhcp | Kind::Reply | Kind::Retired => {}
                Kind::Probe => {
                    let mut probe = self.probing.ok_or(StackError::InvalidState)?;
                    probe.sent += 1;
                    let delay = if probe.sent < 3 {
                        self.timing.between[usize::from(probe.sent - 1)]
                    } else {
                        200
                    };
                    probe.due = self.deadline(ticks, delay)?;
                    self.probing = Some(probe);
                }
                Kind::Announcement => {
                    let mut probe = self.probing.ok_or(StackError::InvalidState)?;
                    probe.announcements += 1;
                    if probe.announcements == 1 {
                        self.stack
                            .apply_configuration(Self::address(probe.lease), ticks)?;
                        self.client.confirm_address_available(now)?;
                        self.active = self.client.lease(now)?;
                        probe.due = self.deadline(ticks, 200)?;
                        self.probing = Some(probe);
                    } else {
                        self.probing = None;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn transmission(&mut self, ticks: u64) -> Result<Option<Transmission<'_>>, Error> {
        self.now(ticks)?;
        Ok(self.stack.transmission())
    }

    pub fn link_down(&mut self, ticks: u64) -> Result<(), Error> {
        let now = self.now(ticks)?;
        self.client.link_down(now)?;
        self.fence(ticks)?;
        Ok(())
    }

    /// Terminal until an external exclusive-device reset/epoch handoff exists.
    /// No automatic retry, completion, buffer release or recovery constructor.
    pub fn transport_uncertain(&mut self, ticks: u64) -> Result<(), Error> {
        self.terminal = Some(Error::TransportUncertain);
        self.fence(ticks)?;
        Err(Error::TransportUncertain)
    }
}
