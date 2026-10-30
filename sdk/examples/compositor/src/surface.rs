use crate::{chrome::Chrome, control::State};
use nieuw_os_display_client::{
    wire::{Geometry as NativeGeometry, Rectangle as NativeRectangle},
    DisplayClient, Transport,
};
use nieuw_os_native_input_service::{discard_caps, Ipc};
use nieuw_os_sdk_runtime::{Outgoing, ReceiveEndpoint, SendEndpoint};
use nieuw_os_surface_protocol::{self as sur, Action, Error, Receiver, Reply};

const TIMEOUT: u64 = 200;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fatal {
    Configuration,
    Clock,
    Receive,
    Reply,
    Expired,
    Display,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub received: u64,
    pub committed: u64,
    pub pixels: u64,
    pub rejected: u64,
    pub reply_again: u64,
    pub border: u64,
    pub cleared: u64,
}
pub struct Surface<D> {
    display: DisplayClient<D>,
    receiver: Receiver,
    peer: u64,
    receive: ReceiveEndpoint,
    send: SendEndpoint,
    pending: Option<Reply>,
    pending_at: u64,
    retired: bool,
    display_failed: bool,
    counts: Counts,
}
impl<D: Transport> Surface<D> {
    /// Caller has obtained actual geometry and performed the mandatory initial
    /// trusted write before publishing boot Ready. This type never fabricates it.
    pub fn new(
        display: DisplayClient<D>,
        native: NativeGeometry,
        epoch: u64,
        peer: u64,
        receive: ReceiveEndpoint,
        send: SendEndpoint,
    ) -> Result<Self, Fatal> {
        let geometry = Chrome::new(native)
            .and_then(|c| c.geometry(epoch))
            .map_err(|_| Fatal::Configuration)?;
        let receiver = Receiver::new(peer, geometry, TIMEOUT).map_err(|_| Fatal::Configuration)?;
        Ok(Self {
            display,
            receiver,
            peer,
            receive,
            send,
            pending: None,
            pending_at: 0,
            retired: false,
            display_failed: false,
            counts: Counts::default(),
        })
    }
    pub const fn counts(&self) -> Counts {
        self.counts
    }
    pub fn completed_frame(&self) -> Option<u64> {
        self.receiver.completed_frame()
    }
    pub const fn is_retired(&self) -> bool {
        self.retired
    }
    pub fn retire(&mut self, io: &mut impl Ipc) {
        self.receiver.retire();
        self.pending = None;
        if !self.retired {
            let _ = io.close(self.receive.handle());
            let _ = io.close(self.send.handle());
        }
        self.retired = true;
    }
    pub fn chrome(
        &mut self,
        chrome: Chrome,
        tile: NativeRectangle,
        state: State,
        bytes: &mut [u8; 1024],
    ) -> Result<(), Fatal> {
        let length = chrome
            .render(tile, state, bytes)
            .map_err(|_| Fatal::Configuration)?;
        self.display
            .blit(tile, &bytes[..length])
            .map_err(|_| Fatal::Display)?;
        self.counts.border = self.counts.border.saturating_add(1);
        Ok(())
    }
    pub fn clear(
        &mut self,
        chrome: Chrome,
        tile: NativeRectangle,
        bytes: &mut [u8; 1024],
    ) -> Result<(), Fatal> {
        if !self.retired {
            return Err(Fatal::Configuration);
        }
        let length = chrome
            .render_wipe(tile, bytes)
            .map_err(|_| Fatal::Configuration)?;
        self.display
            .blit(tile, &bytes[..length])
            .map_err(|_| Fatal::Display)?;
        self.counts.cleared = self.counts.cleared.saturating_add(1);
        Ok(())
    }
    fn pending(&mut self, reply: Reply, now: u64) {
        self.pending = Some(reply);
        self.pending_at = now;
    }
    pub fn step(
        &mut self,
        now: u64,
        io: &mut impl Ipc,
        fresh_tick: impl FnOnce() -> Result<u64, Fatal>,
    ) -> Result<(), Fatal> {
        if self.retired {
            return Ok(());
        }
        if self.pending.is_none() {
            if let Some(reply) = self.receiver.expire(now).map_err(|_| Fatal::Clock)? {
                self.pending(reply, now);
            } else {
                let message = match io.receive(self.receive) {
                    Ok(message) => Some(message),
                    Err(error) if error.is_again() => None,
                    Err(_) => return Err(Fatal::Receive),
                };
                if let Some(message) = message {
                    self.counts.received = self.counts.received.saturating_add(1);
                    let has_caps = message.cap_count() != 0;
                    if has_caps {
                        discard_caps(io, &message).map_err(|_| Fatal::Receive)?;
                    }
                    if message.sender() != self.peer || message.tag() != sur::TAG || has_caps {
                        self.counts.rejected = self.counts.rejected.saturating_add(1);
                        if message.sender() == self.peer && message.tag() == sur::TAG {
                            if let Some(reply) = self.receiver.reject(
                                message.sender(),
                                message.payload(),
                                Error::Access,
                            ) {
                                self.pending(reply, now);
                            }
                        }
                    } else {
                        match self
                            .receiver
                            .accept(message.sender(), now, message.payload())
                        {
                            Ok(Action::Staged) => {}
                            Ok(Action::Info(reply)) => self.pending(reply, now),
                            Ok(Action::Commit(token)) => {
                                let tick = fresh_tick()?;
                                let display = &mut self.display;
                                let reply = self
                                    .receiver
                                    .commit_with(token, tick, |rect, pixels| {
                                        display.blit(
                                            NativeRectangle {
                                                x: rect.x,
                                                y: rect.y,
                                                width: rect.width,
                                                height: rect.height,
                                            },
                                            pixels,
                                        )
                                    })
                                    .map_err(|_| Fatal::Expired)?;
                                if reply.status().is_ok() {
                                    self.counts.committed = self.counts.committed.saturating_add(1);
                                    self.counts.pixels = self
                                        .counts
                                        .pixels
                                        .saturating_add(u64::from(reply.pixels()));
                                } else {
                                    self.counts.rejected = self.counts.rejected.saturating_add(1);
                                    if reply.status() == Err(Error::Blit) {
                                        self.display_failed = true;
                                        self.receiver.retire();
                                    }
                                }
                                self.pending(reply, now);
                            }
                            Err(error) => {
                                self.counts.rejected = self.counts.rejected.saturating_add(1);
                                if let Some(reply) =
                                    self.receiver
                                        .reject(message.sender(), message.payload(), error)
                                {
                                    self.pending(reply, now);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(reply) = &self.pending {
            if now.saturating_sub(self.pending_at) > TIMEOUT {
                return Err(Fatal::Reply);
            }
            let message = Outgoing::new(sur::TAG, reply.bytes()).map_err(|_| Fatal::Reply)?;
            match io.send(self.send, &message) {
                Ok(()) => self.pending = None,
                Err(error) if error.is_again() => {
                    self.counts.reply_again = self.counts.reply_again.saturating_add(1)
                }
                Err(_) => return Err(Fatal::Reply),
            }
        }
        if self.display_failed && self.pending.is_none() {
            return Err(Fatal::Display);
        }
        Ok(())
    }
}
