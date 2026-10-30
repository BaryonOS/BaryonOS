#![no_std]
#![forbid(unsafe_code)]
//! Socket authority core, not a native issuer, NIC fence or IPC service.
//!
//! All entrypoints belong to the trusted service loop, not to an app wire ABI.
//! No production issuer exists: safe external code cannot construct the
//! contexts below. Private test fixtures are not a kernel-authentication seam.
//!
//! ```compile_fail
//! use nieuw_os_net_broker::Caller;
//! let caller = Caller { instance: 1, subject: 2, incarnation: 3 };
//! ```
//! ```compile_fail
//! use nieuw_os_net_broker::ServiceInstance;
//! let instance = ServiceInstance::default();
//! ```
//! ```compile_fail
//! use nieuw_os_net_broker::PolicyAuthority;
//! let authority = PolicyAuthority::new(1, true);
//! ```
//! ```compile_fail
//! use nieuw_os_net_broker::fixtures;
//! ```

use nieuw_os_net_stack::{
    Client, Config, Error as StackError, FRAME_BYTES, MAX_CLIENTS, SocketStorage, Stack, State,
};

pub const MAX_GRANTS: usize = 8;
pub const MAX_TIMEOUT_TICKS: u64 = 6000;

/// Must eventually come from a trusted issuer bound to a unique process start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caller {
    instance: u64,
    subject: u64,
    incarnation: u64,
}

/// Separate policy authority; neither an app grant nor a request field.
pub struct PolicyAuthority {
    instance: u64,
}

/// Consumed, non-cloneable lease context. A native backend-wide issuer and
/// device fence remain missing; an exclusive Stack borrow alone cannot prove it.
pub struct ServiceInstance {
    instance: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Destination {
    ipv4: [u8; 4],
    port: u16,
}

impl Destination {
    pub fn new(ipv4: [u8; 4], port: u16) -> Result<Self, Error> {
        if ipv4[0] == 0 || ipv4[0] == 127 || ipv4[0] >= 224 || port == 0 {
            return Err(Error::InvalidDestination);
        }
        Ok(Self { ipv4, port })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rights {
    Send,
    Receive,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    instance: u64,
    slot: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connection {
    instance: u64,
    slot: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quarantine {
    Revoked,
    Cancelled,
    Expired,
    ClockFault,
    UnknownPublication,
    CompletionMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Denied,
    StaleGrant,
    StaleConnection,
    InvalidDestination,
    InvalidDeadline,
    GrantLimit,
    SocketLimit,
    GenerationExhausted,
    ClockWentBackwards,
    ClockOverflow,
    Expired,
    Busy,
    Quarantined(Quarantine),
    Backend(StackError),
    Cleanup([Option<StackError>; MAX_CLIENTS]),
}

/// Outcomes may only be supplied by the future trusted device adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Publication {
    /// Definitely not submitted, e.g. native EAGAIN. Not a TX completion.
    NotPublished,
    /// The matching frame really completed. Not a peer/application ACK.
    Completed,
    /// No retry, synthetic completion, recovery or buffer release is allowed.
    Unknown,
}

pub struct Outbound<'a> {
    pub id: u64,
    pub bytes: &'a [u8],
}

pub struct Buffers<'a> {
    pub storage: &'a mut [SocketStorage<'a>],
    pub rx_frame: &'a mut [u8; FRAME_BYTES],
    pub tx_frame: &'a mut [u8; FRAME_BYTES],
    pub rx_sockets: [&'a mut [u8]; MAX_CLIENTS],
    pub tx_sockets: [&'a mut [u8]; MAX_CLIENTS],
}

#[derive(Clone, Copy)]
struct Policy {
    caller: Caller,
    destination: Destination,
    rights: Rights,
    deadline: u64,
}

#[derive(Clone, Copy)]
struct GrantSlot {
    generation: u64,
    policy: Option<Policy>,
}

#[derive(Clone, Copy)]
struct Session {
    caller: Caller,
    grant: Grant,
    deadline: u64,
}

#[derive(Clone, Copy)]
struct SocketSlot {
    generation: u64,
    session: Option<Session>,
}

pub struct Broker<'a> {
    instance: ServiceInstance,
    local_ipv4: [u8; 4],
    prefix_len: u8,
    stack: Stack<'a>,
    clients: [Client; MAX_CLIENTS],
    grants: [GrantSlot; MAX_GRANTS],
    sockets: [SocketSlot; MAX_CLIENTS],
    last_ticks: u64,
    quarantine: Option<Quarantine>,
    offered: Option<u64>,
}

impl<'a> Broker<'a> {
    pub fn new(
        instance: ServiceInstance,
        config: Config,
        buffers: Buffers<'a>,
        ticks: u64,
    ) -> Result<Self, Error> {
        if buffers.storage.len() != MAX_CLIENTS {
            return Err(Error::Backend(StackError::InvalidConfiguration));
        }
        let mut stack = Stack::new(
            config,
            buffers.storage,
            buffers.rx_frame,
            buffers.tx_frame,
            ticks,
        )
        .map_err(Error::Backend)?;
        let [rx0, rx1] = buffers.rx_sockets;
        let [tx0, tx1] = buffers.tx_sockets;
        let clients = [
            stack.add_client(rx0, tx0).map_err(Error::Backend)?,
            stack.add_client(rx1, tx1).map_err(Error::Backend)?,
        ];
        Ok(Self {
            instance,
            local_ipv4: config.ipv4,
            prefix_len: config.prefix_len,
            stack,
            clients,
            grants: [GrantSlot {
                generation: 0,
                policy: None,
            }; MAX_GRANTS],
            sockets: [SocketSlot {
                generation: 0,
                session: None,
            }; MAX_CLIENTS],
            last_ticks: ticks,
            quarantine: None,
            offered: None,
        })
    }

    pub fn quarantine(&self) -> Option<Quarantine> {
        self.quarantine
    }

    fn ready(&self) -> Result<(), Error> {
        match self.quarantine {
            Some(reason) => Err(Error::Quarantined(reason)),
            None => Ok(()),
        }
    }

    fn authority(&self, authority: &PolicyAuthority) -> Result<(), Error> {
        if authority.instance != self.instance.instance {
            return Err(Error::Denied);
        }
        Ok(())
    }

    fn caller(&self, caller: Caller) -> Result<(), Error> {
        if caller.instance != self.instance.instance {
            return Err(Error::Denied);
        }
        Ok(())
    }

    fn policy(&self, caller: Caller, grant: Grant) -> Result<Policy, Error> {
        self.caller(caller)?;
        if grant.instance != self.instance.instance {
            return Err(Error::StaleGrant);
        }
        let slot = self.grants.get(grant.slot).ok_or(Error::StaleGrant)?;
        let policy = slot
            .policy
            .filter(|_| slot.generation == grant.generation)
            .ok_or(Error::StaleGrant)?;
        if policy.caller != caller {
            return Err(Error::Denied);
        }
        Ok(policy)
    }

    fn session(&self, caller: Caller, connection: Connection) -> Result<Session, Error> {
        self.caller(caller)?;
        if connection.instance != self.instance.instance {
            return Err(Error::StaleConnection);
        }
        let slot = self
            .sockets
            .get(connection.slot)
            .ok_or(Error::StaleConnection)?;
        let session = slot
            .session
            .filter(|_| slot.generation == connection.generation)
            .ok_or(Error::StaleConnection)?;
        if session.caller != caller {
            return Err(Error::Denied);
        }
        self.policy(caller, session.grant)?;
        Ok(session)
    }

    // The local egress fence is set before any abort/backend action. Retire
    // every session before cleanup; no buffer/completion is released here.
    fn fence(&mut self, reason: Quarantine) -> Result<(), Error> {
        self.quarantine.get_or_insert(reason);
        let active = self.sockets.map(|slot| slot.session.is_some());
        for slot in &mut self.sockets {
            slot.session = None;
        }
        let mut errors = [None; MAX_CLIENTS];
        for (index, active) in active.into_iter().enumerate() {
            if active {
                errors[index] = self.stack.cancel(self.clients[index]).err();
            }
        }
        if errors.iter().any(Option::is_some) {
            return Err(Error::Cleanup(errors));
        }
        Ok(())
    }

    /// Time is supplied by the trusted loop, never an app request.
    fn observe(&mut self, ticks: u64) -> Result<(), Error> {
        self.ready()?;
        if ticks < self.last_ticks {
            self.fence(Quarantine::ClockFault)?;
            return Err(Error::ClockWentBackwards);
        }
        if ticks
            .checked_mul(10_000)
            .is_none_or(|micros| micros > i64::MAX as u64 / 2)
        {
            self.fence(Quarantine::ClockFault)?;
            return Err(Error::ClockOverflow);
        }
        self.last_ticks = ticks;
        for slot in &mut self.grants {
            if slot.policy.is_some_and(|policy| ticks >= policy.deadline) {
                slot.policy = None;
            }
        }
        if self.sockets.iter().any(|slot| {
            slot.session.is_some_and(|session| {
                ticks >= session.deadline || self.policy(session.caller, session.grant).is_err()
            })
        }) {
            self.fence(Quarantine::Expired)?;
            return Err(Error::Expired);
        }
        Ok(())
    }

    pub fn grant(
        &mut self,
        authority: &PolicyAuthority,
        caller: Caller,
        destination: Destination,
        rights: Rights,
        deadline: u64,
        ticks: u64,
    ) -> Result<Grant, Error> {
        self.authority(authority)?;
        self.caller(caller)?;
        let mask = u32::MAX << (32 - self.prefix_len);
        let target = u32::from_be_bytes(destination.ipv4);
        let local = u32::from_be_bytes(self.local_ipv4);
        if self.prefix_len < 31
            && target & mask == local & mask
            && (target & !mask == 0 || target & !mask == !mask)
        {
            return Err(Error::InvalidDestination);
        }
        self.observe(ticks)?;
        if deadline <= ticks || deadline - ticks > MAX_TIMEOUT_TICKS {
            return Err(Error::InvalidDeadline);
        }
        let index = self
            .grants
            .iter()
            .position(|slot| slot.policy.is_none() && slot.generation < u64::MAX)
            .ok_or_else(|| {
                if self.grants.iter().any(|slot| slot.policy.is_none()) {
                    Error::GenerationExhausted
                } else {
                    Error::GrantLimit
                }
            })?;
        let slot = &mut self.grants[index];
        slot.generation += 1;
        slot.policy = Some(Policy {
            caller,
            destination,
            rights,
            deadline,
        });
        Ok(Grant {
            instance: self.instance.instance,
            slot: index,
            generation: slot.generation,
        })
    }

    pub fn revoke(&mut self, authority: &PolicyAuthority, grant: Grant) -> Result<(), Error> {
        self.authority(authority)?;
        if grant.instance != self.instance.instance {
            return Err(Error::StaleGrant);
        }
        let slot = self.grants.get_mut(grant.slot).ok_or(Error::StaleGrant)?;
        if slot.generation != grant.generation || slot.policy.is_none() {
            return Err(Error::StaleGrant);
        }
        // Local revocation linearizes here. Fence before any backend cleanup.
        slot.policy = None;
        if self
            .sockets
            .iter()
            .any(|slot| slot.session.is_some_and(|session| session.grant == grant))
        {
            self.fence(Quarantine::Revoked)?;
        }
        Ok(())
    }

    pub fn connect(
        &mut self,
        caller: Caller,
        grant: Grant,
        destination: Destination,
        timeout_ticks: u64,
        ticks: u64,
    ) -> Result<Connection, Error> {
        let policy = self.policy(caller, grant)?;
        if destination != policy.destination {
            return Err(Error::Denied);
        }
        self.observe(ticks)?;
        self.policy(caller, grant)?;
        if timeout_ticks == 0 || timeout_ticks > MAX_TIMEOUT_TICKS {
            return Err(Error::InvalidDeadline);
        }
        let deadline = ticks
            .checked_add(timeout_ticks)
            .ok_or(Error::InvalidDeadline)?
            .min(policy.deadline);
        let index = self
            .sockets
            .iter()
            .position(|slot| slot.session.is_none() && slot.generation < u64::MAX)
            .ok_or_else(|| {
                if self.sockets.iter().any(|slot| slot.session.is_none()) {
                    Error::GenerationExhausted
                } else {
                    Error::SocketLimit
                }
            })?;
        let generation = self.sockets[index].generation + 1;
        // connect() uses the existing stack context's time. Advance it only
        // after authorization, expiry and resource checks have succeeded.
        self.stack.poll(ticks).map_err(Error::Backend)?;
        self.stack
            .connect(
                self.clients[index],
                destination.ipv4,
                destination.port,
                49152 + index as u16,
                timeout_ticks,
            )
            .map_err(Error::Backend)?;
        self.sockets[index] = SocketSlot {
            generation,
            session: Some(Session {
                caller,
                grant,
                deadline,
            }),
        };
        Ok(Connection {
            instance: self.instance.instance,
            slot: index,
            generation,
        })
    }

    fn authorize(
        &mut self,
        caller: Caller,
        connection: Connection,
        ticks: u64,
    ) -> Result<Policy, Error> {
        let session = self.session(caller, connection)?;
        self.observe(ticks)?;
        self.policy(caller, session.grant)
    }

    pub fn state(
        &mut self,
        caller: Caller,
        connection: Connection,
        ticks: u64,
    ) -> Result<State, Error> {
        self.authorize(caller, connection, ticks)?;
        self.stack
            .state(self.clients[connection.slot])
            .map_err(Error::Backend)
    }

    pub fn send(
        &mut self,
        caller: Caller,
        connection: Connection,
        bytes: &[u8],
        ticks: u64,
    ) -> Result<usize, Error> {
        let policy = self.session(caller, connection)?;
        if self.policy(caller, policy.grant)?.rights == Rights::Receive {
            return Err(Error::Denied);
        }
        self.authorize(caller, connection, ticks)?;
        self.stack
            .send(self.clients[connection.slot], bytes)
            .map_err(Error::Backend)
    }

    pub fn receive(
        &mut self,
        caller: Caller,
        connection: Connection,
        out: &mut [u8],
        ticks: u64,
    ) -> Result<usize, Error> {
        let session = self.session(caller, connection)?;
        if self.policy(caller, session.grant)?.rights == Rights::Send {
            return Err(Error::Denied);
        }
        self.authorize(caller, connection, ticks)?;
        self.stack
            .receive(self.clients[connection.slot], out)
            .map_err(Error::Backend)
    }

    pub fn close(
        &mut self,
        caller: Caller,
        connection: Connection,
        ticks: u64,
    ) -> Result<(), Error> {
        self.authorize(caller, connection, ticks)?;
        self.stack
            .close(self.clients[connection.slot])
            .map_err(Error::Backend)
    }

    pub fn cancel(
        &mut self,
        caller: Caller,
        connection: Connection,
        ticks: u64,
    ) -> Result<(), Error> {
        self.authorize(caller, connection, ticks)?;
        self.fence(Quarantine::Cancelled)
    }

    /// Reuse only after closure with no queued/offered TX frame. Explicitly
    /// relinquishes unread RX; the existing socket's connect resets its queues.
    pub fn finish(
        &mut self,
        caller: Caller,
        connection: Connection,
        ticks: u64,
    ) -> Result<(), Error> {
        self.authorize(caller, connection, ticks)?;
        if self.offered.is_some()
            || self.stack.transmission().is_some()
            || self
                .stack
                .state(self.clients[connection.slot])
                .map_err(Error::Backend)?
                != State::Closed
        {
            return Err(Error::Busy);
        }
        self.sockets[connection.slot].session = None;
        Ok(())
    }

    pub fn poll(&mut self, ticks: u64) -> Result<(), Error> {
        self.observe(ticks)?;
        self.stack.poll(ticks).map_err(Error::Backend)
    }

    pub fn ingress(&mut self, bytes: &[u8], ticks: u64) -> Result<(), Error> {
        self.observe(ticks)?;
        self.stack.push_frame(bytes).map_err(Error::Backend)
    }

    /// One offer at a time: an outstanding/unknown publication is not replayed.
    pub fn offer(&mut self, ticks: u64) -> Result<Option<Outbound<'_>>, Error> {
        self.observe(ticks)?;
        if self.offered.is_some() {
            return Err(Error::Busy);
        }
        Ok(self.stack.transmission().map(|frame| {
            self.offered = Some(frame.id);
            Outbound {
                id: frame.id,
                bytes: frame.bytes,
            }
        }))
    }

    pub fn publication_result(&mut self, id: u64, result: Publication) -> Result<(), Error> {
        self.ready()?;
        if self.offered != Some(id) {
            self.fence(Quarantine::CompletionMismatch)?;
            return Err(Error::Quarantined(Quarantine::CompletionMismatch));
        }
        match result {
            Publication::NotPublished => {
                self.offered = None;
                Ok(())
            }
            Publication::Completed => {
                if let Err(error) = self.stack.complete_transmission(id) {
                    self.fence(Quarantine::CompletionMismatch)?;
                    return Err(Error::Backend(error));
                }
                self.offered = None;
                Ok(())
            }
            Publication::Unknown => {
                self.fence(Quarantine::UnknownPublication)?;
                Err(Error::Quarantined(Quarantine::UnknownPublication))
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/rights.rs"]
mod rights;
