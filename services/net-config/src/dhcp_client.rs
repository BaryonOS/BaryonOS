//! One direct-link DHCPv4 exchange and lease. This is not a network grant.
//! The caller owns NIC I/O, randomized retries, entropy and RFC 5227 probing.

use crate::{UdpDatagram, dhcp, unicast_ipv4};

const BROADCAST: [u8; 4] = [255; 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Selecting,
    Requesting,
    CheckingAddress,
    Bound,
    RenewalDue,
    Renewing,
    Rebinding,
    Expired,
    TimedOut,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidState,
    InvalidTimeout,
    InvalidMac,
    ClockWentBackwards,
    ClockOverflow,
    WrongEndpoint,
    RelayUnsupported,
    WrongServer,
    WrongMessage,
    WrongAddress,
    MissingConfiguration,
    InvalidConfiguration,
    InfiniteLeaseUnsupported,
    Codec(dhcp::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lease {
    pub address: [u8; 4],
    pub prefix_len: u8,
    pub server: [u8; 4],
    pub renew_at: u64,
    pub rebind_at: u64,
    pub expires_at: u64,
    configuration: dhcp::Reply,
}

impl Lease {
    pub fn routers(&self) -> &[[u8; 4]] {
        self.configuration.routers()
    }

    pub fn dns_servers(&self) -> &[[u8; 4]] {
        self.configuration.dns_servers()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outbound {
    pub source: [u8; 4],
    pub destination: [u8; 4],
    pub length: usize,
}

/// All times are caller-supplied monotonic seconds, not poll counts or UTC.
/// One client instance belongs to one interface; never feed another NIC here.
#[derive(Clone, Debug)]
pub struct Client {
    identity: dhcp::Client,
    phase: Phase,
    last_now: u64,
    started_at: u64,
    request_at: Option<u64>,
    timeout_at: u64,
    offer: Option<dhcp::Reply>,
    lease: Option<Lease>,
    sent: bool,
}

impl Client {
    pub fn new(mac: [u8; 6], now: u64) -> Result<Self, Error> {
        if mac == [0; 6] || mac[0] & 1 != 0 {
            return Err(Error::InvalidMac);
        }
        Ok(Self {
            identity: dhcp::Client {
                transaction_id: 0,
                mac,
            },
            phase: Phase::Idle,
            last_now: now,
            started_at: now,
            request_at: None,
            timeout_at: now,
            offer: None,
            lease: None,
            sent: false,
        })
    }

    /// XID must come from the caller's authorized entropy source. Reusing an
    /// old XID after restart can admit delayed replies; no RNG is hidden here.
    pub fn start(&mut self, xid: u32, now: u64, timeout: u64) -> Result<(), Error> {
        self.check_time(now)?;
        if !matches!(
            self.phase,
            Phase::Idle | Phase::Expired | Phase::TimedOut | Phase::Invalidated
        ) {
            return Err(Error::InvalidState);
        }
        if timeout == 0 || timeout > 120 {
            return Err(Error::InvalidTimeout);
        }
        let deadline = now.checked_add(timeout).ok_or(Error::ClockOverflow)?;
        self.clear(Phase::Selecting);
        self.identity.transaction_id = xid;
        self.last_now = now;
        self.started_at = now;
        self.timeout_at = deadline;
        Ok(())
    }

    fn check_time(&self, now: u64) -> Result<(), Error> {
        if now < self.last_now {
            return Err(Error::ClockWentBackwards);
        }
        Ok(())
    }

    fn clear(&mut self, phase: Phase) {
        self.phase = phase;
        self.offer = None;
        self.lease = None;
        self.request_at = None;
        self.sent = false;
    }

    /// Apply deadlines even if no packet arrives. At the exact expiry boundary
    /// no lease may be used, including while address probing is incomplete.
    pub fn tick(&mut self, now: u64) -> Result<Phase, Error> {
        self.check_time(now)?;
        self.last_now = now;
        if let Some(lease) = self.lease {
            if now >= lease.expires_at {
                self.clear(Phase::Expired);
            } else if self.phase == Phase::Bound && now >= lease.renew_at {
                self.phase = Phase::RenewalDue;
            } else if self.phase == Phase::Renewing && now >= lease.rebind_at {
                self.phase = Phase::Rebinding;
                self.sent = false;
            }
        } else if matches!(self.phase, Phase::Selecting | Phase::Requesting)
            && now >= self.timeout_at
        {
            self.clear(Phase::TimedOut);
        }
        Ok(self.phase)
    }

    /// A due renewal starts a fresh transaction; T2 may already have passed.
    pub fn begin_renewal(&mut self, xid: u32, now: u64) -> Result<(), Error> {
        self.tick(now)?;
        if self.phase != Phase::RenewalDue {
            return Err(Error::InvalidState);
        }
        let lease = self.lease.ok_or(Error::InvalidState)?;
        self.identity.transaction_id = xid;
        self.started_at = now;
        self.request_at = None;
        self.sent = false;
        self.phase = if now >= lease.rebind_at {
            Phase::Rebinding
        } else {
            Phase::Renewing
        };
        Ok(())
    }

    /// Encoding starts a conservative lease clock, not a TX-completion claim.
    /// Retries keep the first request time. Failed encodes publish no message.
    pub fn build(&mut self, output: &mut [u8], now: u64) -> Result<Outbound, Error> {
        self.tick(now)?;
        let elapsed = now.saturating_sub(self.started_at).min(u64::from(u16::MAX)) as u16;
        let id = self.identity;
        let (source, destination, length) = match self.phase {
            Phase::Selecting => (
                [0; 4],
                BROADCAST,
                dhcp::build_discover(output, id.transaction_id, id.mac, elapsed),
            ),
            Phase::Requesting => {
                let offer = self.offer.ok_or(Error::InvalidState)?;
                (
                    [0; 4],
                    BROADCAST,
                    dhcp::build_selecting_request(
                        output,
                        id.transaction_id,
                        id.mac,
                        elapsed,
                        offer.offered_address,
                        offer.server_identifier,
                    ),
                )
            }
            Phase::Renewing | Phase::Rebinding => {
                let lease = self.lease.ok_or(Error::InvalidState)?;
                (
                    lease.address,
                    if self.phase == Phase::Renewing {
                        lease.server
                    } else {
                        BROADCAST
                    },
                    dhcp::build_renewal_request(
                        output,
                        id.transaction_id,
                        id.mac,
                        elapsed,
                        lease.address,
                    ),
                )
            }
            _ => return Err(Error::InvalidState),
        };
        let length = length.map_err(Error::Codec)?;
        if self.phase != Phase::Selecting && self.request_at.is_none() {
            self.request_at = Some(now);
        }
        self.sent = true;
        Ok(Outbound {
            source,
            destination,
            length,
        })
    }

    /// No relay support in this profile. DHCP's permitted zero UDP checksum
    /// remains permitted; even a present checksum provides no authentication.
    pub fn accept(&mut self, packet: &UdpDatagram<'_>, now: u64) -> Result<Phase, Error> {
        self.tick(now)?;
        if !self.sent
            || !matches!(
                self.phase,
                Phase::Selecting | Phase::Requesting | Phase::Renewing | Phase::Rebinding
            )
        {
            return Err(Error::InvalidState);
        }
        let reply = dhcp::parse_reply(packet.payload(), self.identity).map_err(Error::Codec)?;
        let target = if self.phase == Phase::Selecting {
            reply.offered_address
        } else if let Some(lease) = self.lease {
            lease.address
        } else {
            self.offer.ok_or(Error::InvalidState)?.offered_address
        };
        if packet.source_port() != 67
            || packet.destination_port() != 68
            || !unicast_ipv4(packet.source_address())
            || (packet.destination_address() != BROADCAST && packet.destination_address() != target)
            || (packet.destination_mac() != BROADCAST_MAC
                && packet.destination_mac() != self.identity.mac)
        {
            return Err(Error::WrongEndpoint);
        }
        if packet.payload()[24..28] != [0; 4] {
            return Err(Error::RelayUnsupported);
        }
        if packet.source_address() != reply.server_identifier {
            return Err(Error::WrongServer);
        }
        if self.phase == Phase::Selecting {
            if reply.message_type != dhcp::MessageType::Offer {
                return Err(Error::WrongMessage);
            }
            self.validate_config(reply)?;
            self.offer = Some(reply);
            self.phase = Phase::Requesting;
            self.sent = false;
            return Ok(self.phase);
        }
        let expected_server = self
            .lease
            .map(|lease| lease.server)
            .or(self.offer.map(|offer| offer.server_identifier))
            .ok_or(Error::InvalidState)?;
        if self.phase != Phase::Rebinding && reply.server_identifier != expected_server {
            return Err(Error::WrongServer);
        }
        match reply.message_type {
            dhcp::MessageType::Nak => {
                self.clear(Phase::Invalidated);
            }
            dhcp::MessageType::Ack => {
                if reply.offered_address != target {
                    return Err(Error::WrongAddress);
                }
                let prefix_len = self.validate_config(reply)?;
                let duration = u64::from(reply.lease_seconds.ok_or(Error::MissingConfiguration)?);
                if duration == u64::from(u32::MAX) {
                    return Err(Error::InfiniteLeaseUnsupported);
                }
                let start = self.request_at.ok_or(Error::InvalidState)?;
                let renew = u64::from(reply.renewal_seconds.unwrap_or((duration / 2) as u32));
                let rebind =
                    u64::from(reply.rebinding_seconds.unwrap_or((duration * 7 / 8) as u32));
                let lease = Lease {
                    address: target,
                    prefix_len,
                    server: reply.server_identifier,
                    renew_at: start.checked_add(renew).ok_or(Error::ClockOverflow)?,
                    rebind_at: start.checked_add(rebind).ok_or(Error::ClockOverflow)?,
                    expires_at: start.checked_add(duration).ok_or(Error::ClockOverflow)?,
                    configuration: reply,
                };
                if now >= lease.expires_at {
                    return Err(Error::InvalidConfiguration);
                }
                let renewing = self.lease.is_some();
                self.lease = Some(lease);
                self.offer = None;
                self.sent = false;
                self.phase = if renewing {
                    Phase::Bound
                } else {
                    Phase::CheckingAddress
                };
                self.tick(now)?;
            }
            _ => return Err(Error::WrongMessage),
        }
        Ok(self.phase)
    }

    fn validate_config(&self, reply: dhcp::Reply) -> Result<u8, Error> {
        if !unicast_ipv4(reply.offered_address) {
            return Err(Error::WrongAddress);
        }
        let mask = u32::from_be_bytes(reply.subnet_mask.ok_or(Error::MissingConfiguration)?);
        let prefix = mask.count_ones() as u8;
        if prefix == 0 {
            return Err(Error::InvalidConfiguration);
        }
        let address = u32::from_be_bytes(reply.offered_address);
        let host = address & !mask;
        if prefix <= 30 && (host == 0 || host == !mask) {
            return Err(Error::WrongAddress);
        }
        for router in reply.routers() {
            let router = u32::from_be_bytes(*router);
            if router == address
                || router & mask != address & mask
                || (prefix <= 30 && (router & !mask == 0 || router & !mask == !mask))
            {
                return Err(Error::InvalidConfiguration);
            }
        }
        Ok(prefix)
    }

    /// Candidate to pass to the caller's real address-conflict detector.
    /// It is not yet a usable interface/route configuration.
    pub fn candidate(&mut self, now: u64) -> Result<Option<Lease>, Error> {
        self.tick(now)?;
        Ok((self.phase == Phase::CheckingAddress)
            .then_some(self.lease)
            .flatten())
    }

    /// Call only after actual conflict detection on this interface, never just
    /// because DHCPACK arrived. This library cannot observe ARP probes.
    pub fn confirm_address_available(&mut self, now: u64) -> Result<Phase, Error> {
        self.tick(now)?;
        if self.phase != Phase::CheckingAddress {
            return Err(Error::InvalidState);
        }
        self.phase = Phase::Bound;
        self.tick(now)
    }

    pub fn reject_address_conflict(&mut self, now: u64) -> Result<(), Error> {
        self.tick(now)?;
        if self.phase != Phase::CheckingAddress {
            return Err(Error::InvalidState);
        }
        self.clear(Phase::Invalidated);
        Ok(())
    }

    pub fn lease(&mut self, now: u64) -> Result<Option<Lease>, Error> {
        self.tick(now)?;
        Ok(if self.phase == Phase::CheckingAddress {
            None
        } else {
            self.lease
        })
    }

    pub fn link_down(&mut self, now: u64) -> Result<(), Error> {
        self.check_time(now)?;
        self.last_now = now;
        self.clear(Phase::Idle);
        Ok(())
    }
}

const BROADCAST_MAC: [u8; 6] = [255; 6];
