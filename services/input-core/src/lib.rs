#![no_std]
#![forbid(unsafe_code)]
//! Allocation-free input service policy over existing PS/2 and IPC primitives.
//! No device I/O or kernel authority is implemented here. `Received` must come
//! from the real kernel adapter; parsing bytes on a host proves no provenance.

pub mod wire;
use nieuw_os_sdk_runtime::{ReceiveEndpoint, Received};
use ui_core::{
    Cursor, FocusOwner, FocusToken, InputRouter, KeyboardEvent, MouseEvent, Ps2Mouse, Set1Keyboard,
};
use wire::{Operation, Request};

pub const MAX_CLIENTS: usize = 8;
pub const QUEUE_CAPACITY: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Error {
    Configuration = 2,
    Access = 3,
    UnexpectedCaps = 4,
    Malformed = 5,
    WrongEpoch = 6,
    WrongRole = 7,
    StaleRequest = 8,
    UnknownClient = 9,
    ActorDead = 10,
    NotSynchronized = 11,
    StreamLost = 12,
    ControllerFault = 13,
    DecodeFault = 14,
    QueueFull = 15,
    StaleAck = 16,
    CounterExhausted = 17,
    Halted = 18,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub endpoint: ReceiveEndpoint,
    pub pid: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientBinding {
    pub id: u64,
    pub pid: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub epoch: u64,
    pub source: Actor,
    pub compositor: Actor,
    pub supervisor: Actor,
    pub applications: ReceiveEndpoint,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Source,
    Compositor,
    Supervisor,
    Client(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelReason {
    FocusLost,
    SourceLost,
    QueueFull,
    Resync,
    SourceDead,
    CompositorDead,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Key(KeyboardEvent),
    Mouse { input: MouseEvent, x: u32, y: u32 },
    Cancel(CancelReason),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Queued {
    sequence: u64,
    token: FocusToken,
    event: Event,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cancellation {
    sequence: u64,
    through: u64,
    reason: CancelReason,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Client {
    binding: ClientBinding,
    alive: bool,
    last_request: u64,
    offered: Option<u64>,
    cancel: Option<Cancellation>,
}

/// Route is resolved from bootstrap identity and kernel metadata, never from a
/// caller-chosen reply handle. Check `reply_is_current` immediately before send.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    route: Route,
    payload: [u8; wire::SIZE],
    event_sequence: Option<u64>,
}
impl Reply {
    pub const fn route(&self) -> Route {
        self.route
    }
    pub const fn payload(&self) -> &[u8; wire::SIZE] {
        &self.payload
    }
}

pub struct Service {
    configuration: Configuration,
    clients: [Option<Client>; MAX_CLIENTS],
    keyboard: Set1Keyboard,
    mouse: Ps2Mouse,
    cursor: Cursor,
    router: InputRouter,
    queue: [Option<Queued>; QUEUE_CAPACITY],
    head: usize,
    length: usize,
    next_event: u64,
    last_source: u64,
    last_control: u64,
    last_lifecycle: u64,
    online: bool,
    source_alive: bool,
    compositor_alive: bool,
    halted: bool,
}

impl Service {
    pub fn new(configuration: Configuration, bindings: &[ClientBinding]) -> Result<Self, Error> {
        let c = configuration;
        let endpoints = [
            c.source.endpoint,
            c.compositor.endpoint,
            c.supervisor.endpoint,
            c.applications,
        ];
        let pids = [c.source.pid, c.compositor.pid, c.supervisor.pid];
        if c.epoch == 0
            || pids.contains(&0)
            || bindings.is_empty()
            || bindings.len() > MAX_CLIENTS
            || endpoints
                .iter()
                .enumerate()
                .any(|(i, e)| endpoints[..i].contains(e))
            || pids.iter().enumerate().any(|(i, p)| pids[..i].contains(p))
        {
            return Err(Error::Configuration);
        }
        let mut clients = [None; MAX_CLIENTS];
        for (index, binding) in bindings.iter().enumerate() {
            if binding.id == 0
                || binding.pid == 0
                || binding.pid == c.source.pid
                || bindings[..index]
                    .iter()
                    .any(|previous| previous.id == binding.id || previous.pid == binding.pid)
            {
                return Err(Error::Configuration);
            }
            clients[index] = Some(Client {
                binding: *binding,
                alive: true,
                last_request: 0,
                offered: None,
                cancel: None,
            });
        }
        Ok(Self {
            configuration,
            clients,
            keyboard: Set1Keyboard::new(),
            mouse: Ps2Mouse::new(),
            cursor: Cursor::new(c.width, c.height).ok_or(Error::Configuration)?,
            router: InputRouter::new(),
            queue: [None; QUEUE_CAPACITY],
            head: 0,
            length: 0,
            next_event: 1,
            last_source: 0,
            last_control: 0,
            last_lifecycle: 0,
            online: false,
            source_alive: true,
            compositor_alive: true,
            halted: false,
        })
    }
    pub const fn is_online(&self) -> bool {
        self.online && !self.halted
    }
    pub const fn is_halted(&self) -> bool {
        self.halted
    }
    pub fn focus(&self) -> Option<FocusToken> {
        self.router.current()
    }
    pub const fn cursor(&self) -> (u32, u32) {
        self.cursor.position()
    }
    pub const fn queued(&self) -> usize {
        self.length
    }

    /// Trusted adapter uses this after losing its authority/lifecycle channel.
    /// It must close its endpoints and tell consumers to cancel input.
    pub fn shutdown(&mut self) {
        self.halted = true;
        self.online = false;
        self.clear_queue();
        let _ = self.router.set_focus(None);
    }

    fn client_id(&self, id: u64) -> Result<usize, Error> {
        self.clients
            .iter()
            .position(|client| client.is_some_and(|c| c.binding.id == id))
            .ok_or(Error::UnknownClient)
    }
    fn principal(&self, ingress: ReceiveEndpoint, sender: u64) -> Result<Route, Error> {
        let c = self.configuration;
        for (actor, route) in [
            (c.source, Route::Source),
            (c.compositor, Route::Compositor),
            (c.supervisor, Route::Supervisor),
        ] {
            if actor.endpoint == ingress {
                return if actor.pid == sender {
                    Ok(route)
                } else {
                    Err(Error::Access)
                };
            }
        }
        if ingress == c.applications {
            return self
                .clients
                .iter()
                .flatten()
                .find(|client| client.binding.pid == sender)
                .map(|client| Route::Client(client.binding.id))
                .ok_or(Error::Access);
        }
        Err(Error::Access)
    }

    /// No returned error may be converted to an application success. The
    /// adapter explicitly closes any unexpected received capabilities even
    /// when this method rejects the envelope before reading its payload.
    pub fn handle(&mut self, ingress: ReceiveEndpoint, message: &Received) -> Result<Reply, Error> {
        if self.halted {
            return Err(Error::Halted);
        }
        let route = self.principal(ingress, message.sender())?;
        if message.cap_count() != 0 {
            return Err(Error::UnexpectedCaps);
        }
        if message.tag() != wire::TAG {
            return Err(Error::Malformed);
        }
        let request = Request::decode(message.payload(), self.configuration.epoch)?;
        let allowed = matches!(
            (route, request.operation),
            (Route::Source, Operation::Sync | Operation::Sample)
                | (Route::Compositor, Operation::Focus)
                | (
                    Route::Supervisor,
                    Operation::ClientDead | Operation::SourceDead | Operation::CompositorDead
                )
                | (Route::Client(_), Operation::Peek | Operation::Ack)
        );
        if !allowed {
            return Err(Error::WrongRole);
        }
        let mut event = None;
        match route {
            Route::Source => self.source(request)?,
            Route::Compositor => self.control(request)?,
            Route::Supervisor => self.lifecycle(request)?,
            Route::Client(id) => event = self.application(id, request)?,
        }
        Ok(Reply {
            route,
            payload: wire::response(
                request,
                match route {
                    Route::Client(id) => id,
                    _ => 0,
                },
                event,
            ),
            event_sequence: event.map(|e| e.0),
        })
    }

    /// Negative replies require a bound principal and a canonical current-epoch
    /// request. Other protocol failures close/report through the native adapter.
    pub fn error_reply(
        &self,
        ingress: ReceiveEndpoint,
        message: &Received,
        error: Error,
    ) -> Result<Reply, Error> {
        if self.halted {
            return Err(Error::Halted);
        }
        let route = self.principal(ingress, message.sender())?;
        if message.cap_count() != 0 {
            return Err(Error::UnexpectedCaps);
        }
        if message.tag() != wire::TAG {
            return Err(Error::Malformed);
        }
        let request = Request::decode(message.payload(), self.configuration.epoch)?;
        Ok(Reply {
            route,
            payload: wire::failure(
                request,
                match route {
                    Route::Client(id) => id,
                    _ => 0,
                },
                error,
            ),
            event_sequence: None,
        })
    }

    fn advance(last: &mut u64, id: u64) -> Result<(), Error> {
        if id <= *last {
            return Err(Error::StaleRequest);
        }
        *last = id;
        Ok(())
    }
    fn sequence(&mut self) -> Result<u64, Error> {
        let id = self.next_event;
        let Some(next) = id.checked_add(1) else {
            self.shutdown();
            return Err(Error::CounterExhausted);
        };
        self.next_event = next;
        Ok(id)
    }
    fn clear_queue(&mut self) {
        self.queue.fill(None);
        self.head = 0;
        self.length = 0;
    }
    fn cancellation(&mut self, reason: CancelReason) -> Result<(), Error> {
        self.clear_queue();
        if let Some(token) = self.router.current() {
            let slot = self.client_id(token.owner().identity())?;
            let sequence = self.sequence()?;
            let client = self.clients[slot].as_mut().unwrap();
            let through = client.cancel.map_or(token.generation(), |cancel| {
                cancel.through.max(token.generation())
            });
            client.cancel = Some(Cancellation {
                sequence,
                through,
                reason,
            });
            client.offered = None;
        }
        Ok(())
    }
    fn reset_input(&mut self, reason: CancelReason, reset_decoder: bool) -> Result<(), Error> {
        self.cancellation(reason)?;
        if self.router.reset_input().is_err() {
            self.shutdown();
            return Err(Error::CounterExhausted);
        }
        if reset_decoder {
            self.keyboard.reset();
            self.mouse.reset();
        }
        Ok(())
    }
    fn source_fault(&mut self, error: Error) -> Result<(), Error> {
        self.online = false;
        self.reset_input(CancelReason::SourceLost, true)?;
        Err(error)
    }
    fn source(&mut self, request: Request) -> Result<(), Error> {
        if !self.source_alive {
            return Err(Error::ActorDead);
        }
        if request.id == u64::MAX {
            self.shutdown();
            return Err(Error::CounterExhausted);
        }
        if request.id <= self.last_source {
            return Err(Error::StaleRequest);
        }
        if request.operation == Operation::Sync {
            self.last_source = request.id;
            self.reset_input(CancelReason::Resync, true)?;
            self.online = true;
            return Ok(());
        }
        let contiguous = self.last_source.checked_add(1) == Some(request.id);
        self.last_source = request.id;
        if !self.online {
            return Err(Error::NotSynchronized);
        }
        if !contiguous {
            return self.source_fault(Error::StreamLost);
        }
        let (status, byte) = (request.arg0 as u8, request.arg1 as u8);
        if status & 1 == 0 || status & 0xc0 != 0 {
            return self.source_fault(Error::ControllerFault);
        }
        if status & 0x20 == 0 {
            match self.keyboard.feed(byte) {
                Ok(Some(input)) => {
                    if let Some(routed) = self.router.route_key(input) {
                        self.enqueue(routed.token(), Event::Key(routed.into_event()))?;
                    }
                }
                Ok(None) => {}
                Err(_) => return self.source_fault(Error::DecodeFault),
            }
        } else {
            match self.mouse.feed(byte) {
                Ok(Some(input)) => {
                    let (x, y) = self.cursor.apply(input);
                    if let Some(routed) = self.router.route_mouse(input) {
                        self.enqueue(
                            routed.token(),
                            Event::Mouse {
                                input: routed.into_event(),
                                x,
                                y,
                            },
                        )?;
                    }
                }
                Ok(None) => {}
                Err(_) => return self.source_fault(Error::DecodeFault),
            }
        }
        Ok(())
    }
    fn enqueue(&mut self, token: FocusToken, event: Event) -> Result<(), Error> {
        if self.length == QUEUE_CAPACITY {
            self.reset_input(CancelReason::QueueFull, false)?;
            return Err(Error::QueueFull);
        }
        let sequence = self.sequence()?;
        self.queue[(self.head + self.length) % QUEUE_CAPACITY] = Some(Queued {
            sequence,
            token,
            event,
        });
        self.length += 1;
        Ok(())
    }
    fn control(&mut self, request: Request) -> Result<(), Error> {
        if !self.compositor_alive {
            return Err(Error::ActorDead);
        }
        Self::advance(&mut self.last_control, request.id)?;
        if request.id == u64::MAX {
            self.shutdown();
            return Err(Error::CounterExhausted);
        }
        let owner = if request.arg0 == 0 {
            None
        } else {
            let client = self.clients[self.client_id(request.arg0)?]
                .as_ref()
                .unwrap();
            if !client.alive {
                return Err(Error::ActorDead);
            }
            FocusOwner::new(client.binding.id)
        };
        if self.router.current().map(|token| token.owner()) == owner {
            return Ok(());
        }
        self.cancellation(CancelReason::FocusLost)?;
        if self.router.set_focus(owner).is_err() {
            self.shutdown();
            return Err(Error::CounterExhausted);
        }
        Ok(())
    }
    fn lifecycle(&mut self, request: Request) -> Result<(), Error> {
        Self::advance(&mut self.last_lifecycle, request.id)?;
        if request.id == u64::MAX {
            self.shutdown();
            return Err(Error::CounterExhausted);
        }
        match request.operation {
            Operation::ClientDead => {
                let slot = self.client_id(request.arg0)?;
                if self
                    .router
                    .current()
                    .is_some_and(|token| token.owner().identity() == request.arg0)
                {
                    self.clear_queue();
                    if self.router.set_focus(None).is_err() {
                        self.shutdown();
                        return Err(Error::CounterExhausted);
                    }
                }
                let client = self.clients[slot].as_mut().unwrap();
                client.alive = false;
                client.cancel = None;
                client.offered = None;
            }
            Operation::SourceDead | Operation::CompositorDead => {
                let reason = if request.operation == Operation::SourceDead {
                    self.source_alive = false;
                    self.online = false;
                    CancelReason::SourceDead
                } else {
                    self.compositor_alive = false;
                    CancelReason::CompositorDead
                };
                self.cancellation(reason)?;
                if self.router.set_focus(None).is_err() {
                    self.shutdown();
                    return Err(Error::CounterExhausted);
                }
            }
            _ => return Err(Error::WrongRole),
        }
        Ok(())
    }
    fn application(
        &mut self,
        id: u64,
        request: Request,
    ) -> Result<Option<(u64, u64, Event)>, Error> {
        let slot = self.client_id(id)?;
        let client = self.clients[slot].as_mut().unwrap();
        if !client.alive {
            return Err(Error::ActorDead);
        }
        Self::advance(&mut client.last_request, request.id)?;
        if request.operation == Operation::Ack {
            if client.offered != Some(request.arg0) {
                return Err(Error::StaleAck);
            }
            if client
                .cancel
                .is_some_and(|cancel| cancel.sequence == request.arg0)
            {
                client.cancel = None;
            } else if self.length != 0
                && self.queue[self.head].is_some_and(|entry| {
                    entry.sequence == request.arg0
                        && entry.token.owner().identity() == id
                        && self.router.is_current(entry.token)
                })
            {
                self.queue[self.head] = None;
                self.head = (self.head + 1) % QUEUE_CAPACITY;
                self.length -= 1;
            } else {
                return Err(Error::StaleAck);
            }
            client.offered = None;
            return Ok(None);
        }
        if let Some(cancel) = client.cancel {
            client.offered = Some(cancel.sequence);
            return Ok(Some((
                cancel.sequence,
                cancel.through,
                Event::Cancel(cancel.reason),
            )));
        }
        let result = if self.length != 0 {
            self.queue[self.head]
                .filter(|entry| {
                    entry.token.owner().identity() == id && self.router.is_current(entry.token)
                })
                .map(|entry| (entry.sequence, entry.token.generation(), entry.event))
        } else {
            None
        };
        client.offered = result.map(|entry| entry.0);
        Ok(result)
    }

    /// Required immediately before kernel send in the serialized adapter loop.
    /// A focus/death/overflow/cancel ACK between prepare and send invalidates
    /// the old response. PEEK itself never consumes a pending event.
    pub fn reply_is_current(&self, reply: &Reply) -> bool {
        if self.halted {
            return false;
        }
        let Route::Client(id) = reply.route else {
            return true;
        };
        let Ok(slot) = self.client_id(id) else {
            return false;
        };
        let client = self.clients[slot].as_ref().unwrap();
        if !client.alive {
            return false;
        }
        let Some(sequence) = reply.event_sequence else {
            return true;
        };
        if client
            .cancel
            .is_some_and(|cancel| cancel.sequence == sequence)
        {
            return true;
        }
        self.length != 0
            && self.queue[self.head].is_some_and(|entry| {
                entry.sequence == sequence
                    && entry.token.owner().identity() == id
                    && self.router.is_current(entry.token)
            })
    }
}

#[cfg(test)]
mod exhaustion_tests {
    // Counter saturation cannot be exercised by a realistic public loop.
    // Integration tests cover all ordinary transitions over real IPC metadata.
    use super::*;
    #[test]
    fn event_counter_exhaustion_never_wraps_or_retains_focus() {
        let endpoint = |slot: u64| {
            ReceiveEndpoint::from_handle(
                nieuw_os_sdk_runtime::Handle::from_raw((1 << 32) | slot).unwrap(),
            )
        };
        let mut service = Service::new(
            Configuration {
                epoch: 1,
                source: Actor {
                    endpoint: endpoint(1),
                    pid: 1,
                },
                compositor: Actor {
                    endpoint: endpoint(2),
                    pid: 2,
                },
                supervisor: Actor {
                    endpoint: endpoint(3),
                    pid: 3,
                },
                applications: endpoint(4),
                width: 800,
                height: 600,
            },
            &[ClientBinding { id: 10, pid: 4 }],
        )
        .unwrap();
        service.router.set_focus(FocusOwner::new(10)).unwrap();
        service.next_event = u64::MAX;
        assert_eq!(service.sequence(), Err(Error::CounterExhausted));
        assert!(service.is_halted());
        assert_eq!(service.focus(), None);
        assert_eq!(service.next_event, u64::MAX);
    }
}
