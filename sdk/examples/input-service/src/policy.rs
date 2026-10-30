use crate::{discard_caps, Ipc};
use nieuw_os_input_core::{
    Actor, ClientBinding, Configuration, Error as CoreError, Reply, Route, Service, MAX_CLIENTS,
};
use nieuw_os_sdk_runtime::{Error, Outgoing, ReceiveEndpoint, Received, SendEndpoint};

pub const ROUTES: usize = 3 + MAX_CLIENTS;
const APP_BUDGET: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct Client {
    pub binding: ClientBinding,
    pub reply: SendEndpoint,
}
#[derive(Clone, Copy, Debug)]
pub struct Replies {
    pub source: SendEndpoint,
    pub compositor: SendEndpoint,
    pub supervisor: SendEndpoint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fatal {
    Configuration,
    Receive(Error),
    CriticalReply(Error),
    Cleanup(Error),
    Halted,
    Message,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub received: u64,
    pub handled: u64,
    pub rejected: u64,
    pub pipelined: u64,
    pub reply_again: u64,
    pub sent: u64,
    pub stale: u64,
    pub retired: u64,
    pub recv_checked: u8,
}
struct Pending {
    ingress: ReceiveEndpoint,
    original: Received,
    reply: Reply,
}
pub struct Policy {
    core: Service,
    config: Configuration,
    replies: Replies,
    clients: [Option<Client>; MAX_CLIENTS],
    pending: [Option<Pending>; ROUTES],
    disabled: [bool; MAX_CLIENTS],
    rotation: usize,
    counts: Counts,
}
impl Policy {
    pub fn new(config: Configuration, replies: Replies, clients: &[Client]) -> Result<Self, Fatal> {
        if clients.is_empty() || clients.len() > MAX_CLIENTS {
            return Err(Fatal::Configuration);
        }
        let mut bindings = [ClientBinding { id: 0, pid: 0 }; MAX_CLIENTS];
        let mut table = [None; MAX_CLIENTS];
        for (index, client) in clients.iter().enumerate() {
            bindings[index] = client.binding;
            table[index] = Some(*client);
        }
        let core =
            Service::new(config, &bindings[..clients.len()]).map_err(|_| Fatal::Configuration)?;
        Ok(Self {
            core,
            config,
            replies,
            clients: table,
            pending: [const { None }; ROUTES],
            disabled: [false; MAX_CLIENTS],
            rotation: 0,
            counts: Counts::default(),
        })
    }
    pub const fn counts(&self) -> Counts {
        self.counts
    }
    pub fn core(&self) -> &Service {
        &self.core
    }
    pub const fn receive_rights_checked(&self) -> bool {
        self.counts.recv_checked == 15
    }
    fn route(&self, ingress: ReceiveEndpoint, pid: u64) -> Option<usize> {
        for (
            index,
            Actor {
                endpoint,
                pid: bound,
            },
        ) in [
            self.config.source,
            self.config.compositor,
            self.config.supervisor,
        ]
        .into_iter()
        .enumerate()
        {
            if ingress == endpoint {
                return (pid == bound).then_some(index);
            }
        }
        if ingress == self.config.applications {
            return self
                .clients
                .iter()
                .position(|c| c.is_some_and(|c| c.binding.pid == pid))
                .map(|i| 3 + i);
        }
        None
    }
    fn endpoint(&self, slot: usize) -> SendEndpoint {
        match slot {
            0 => self.replies.source,
            1 => self.replies.compositor,
            2 => self.replies.supervisor,
            _ => self.clients[slot - 3].unwrap().reply,
        }
    }
    fn correct_route(&self, slot: usize, reply: &Reply) -> bool {
        match reply.route() {
            Route::Source => slot == 0,
            Route::Compositor => slot == 1,
            Route::Supervisor => slot == 2,
            Route::Client(id) => {
                slot >= 3 && self.clients[slot - 3].is_some_and(|c| c.binding.id == id)
            }
        }
    }
    fn ingest(
        &mut self,
        io: &mut impl Ipc,
        ingress: ReceiveEndpoint,
        bit: u8,
    ) -> Result<(), Fatal> {
        let message = match io.receive(ingress) {
            Ok(message) => message,
            Err(error) if error.is_again() => {
                self.counts.recv_checked |= bit;
                return Ok(());
            }
            Err(error) => return Err(Fatal::Receive(error)),
        };
        self.counts.recv_checked |= bit;
        self.counts.received = self.counts.received.saturating_add(1);
        if message.cap_count() != 0 {
            discard_caps(io, &message).map_err(Fatal::Cleanup)?;
            self.counts.rejected = self.counts.rejected.saturating_add(1);
            return Ok(());
        }
        let Some(slot) = self.route(ingress, message.sender()) else {
            self.counts.rejected = self.counts.rejected.saturating_add(1);
            return Ok(());
        };
        if slot >= 3 && self.disabled[slot - 3] {
            self.counts.rejected = self.counts.rejected.saturating_add(1);
            return Ok(());
        }
        if self.pending[slot].is_some() {
            // One outstanding request per peer. Never overwrite a pending
            // response, advance the core twice or close the shared app ingress.
            self.counts.pipelined = self.counts.pipelined.saturating_add(1);
            return Ok(());
        }
        let reply = match self.core.handle(ingress, &message) {
            Ok(reply) => {
                self.counts.handled = self.counts.handled.saturating_add(1);
                reply
            }
            Err(error) => {
                self.counts.rejected = self.counts.rejected.saturating_add(1);
                if self.core.is_halted() {
                    return Err(Fatal::Halted);
                }
                match self.core.error_reply(ingress, &message, error) {
                    Ok(reply) => reply,
                    Err(_) => return Ok(()),
                }
            }
        };
        if !self.correct_route(slot, &reply) {
            return Err(Fatal::Configuration);
        }
        self.pending[slot] = Some(Pending {
            ingress,
            original: message,
            reply,
        });
        Ok(())
    }
    fn retire(&mut self, io: &mut impl Ipc, slot: usize) {
        self.pending[slot] = None;
        if slot >= 3 && !self.disabled[slot - 3] {
            self.disabled[slot - 3] = true;
            let _ = io.close(self.endpoint(slot).handle());
            self.counts.retired = self.counts.retired.saturating_add(1);
        }
    }
    fn flush(&mut self, io: &mut impl Ipc, slot: usize) -> Result<(), Fatal> {
        let Some(pending) = &self.pending[slot] else {
            return Ok(());
        };
        if !self.core.reply_is_current(&pending.reply) {
            if self.core.is_halted() {
                return Err(Fatal::Halted);
            }
            let updated =
                self.core
                    .error_reply(pending.ingress, &pending.original, CoreError::StaleRequest);
            let Ok(reply) = updated else {
                self.retire(io, slot);
                return Ok(());
            };
            self.pending[slot].as_mut().unwrap().reply = reply;
            self.counts.stale = self.counts.stale.saturating_add(1);
        }
        let reply = self.pending[slot].as_ref().unwrap().reply;
        if !self.core.reply_is_current(&reply) {
            self.retire(io, slot);
            return Ok(());
        }
        let outgoing = Outgoing::new(nieuw_os_input_core::wire::TAG, reply.payload())
            .map_err(|_| Fatal::Message)?;
        // No core mutation/callback between this validity check and real send.
        if !self.core.reply_is_current(&reply) {
            return Err(Fatal::Halted);
        }
        match io.send(self.endpoint(slot), &outgoing) {
            Ok(()) => {
                self.pending[slot] = None;
                self.counts.sent = self.counts.sent.saturating_add(1);
            }
            Err(error) if error.is_again() => {
                self.counts.reply_again = self.counts.reply_again.saturating_add(1);
            }
            Err(error) if slot < 3 => return Err(Fatal::CriticalReply(error)),
            Err(_) => self.retire(io, slot),
        }
        Ok(())
    }
    /// One bounded pass: protected ingress first, apps at fixed budget, each
    /// retained response attempted once. It never sleeps/retries inside a call.
    pub fn step(&mut self, io: &mut impl Ipc) -> Result<(), Fatal> {
        if self.core.is_halted() {
            return Err(Fatal::Halted);
        }
        for (slot, ingress, bit) in [
            (2, self.config.supervisor.endpoint, 4),
            (1, self.config.compositor.endpoint, 2),
            (0, self.config.source.endpoint, 1),
        ] {
            if self.pending[slot].is_none() {
                self.ingest(io, ingress, bit)?;
            }
        }
        for _ in 0..APP_BUDGET {
            self.ingest(io, self.config.applications, 8)?;
        }
        for offset in 0..ROUTES {
            self.flush(io, (self.rotation + offset) % ROUTES)?;
        }
        self.rotation = (self.rotation + 1) % ROUTES;
        Ok(())
    }
    pub fn shutdown(&mut self) {
        self.core.shutdown();
        self.pending.fill_with(|| None);
    }
}
