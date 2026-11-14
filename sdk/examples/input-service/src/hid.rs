//! Boot-HID completion publication over the existing authenticated input ingress.
//! This performs no USB transport or enumeration. The owner must authenticate
//! Session lifetimes and completions; synthetic host reports are not USB I/O.
use crate::{
    discard_caps, protocol,
    source::{Configuration, Fatal, EXCHANGE_TICKS},
    Ipc,
};
use nieuw_os_input_core::wire::{self, Operation, Request};
use nieuw_os_sdk_runtime::Outgoing;
use nieuw_os_usb_core::{
    boot_interface::Protocol,
    boot_session::{CompletionStatus, Error as SessionError, Session, Token},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Pending,
    NotSynchronized,
    Session(SessionError),
    Fatal(Fatal),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Progress {
    Idle,
    Pending,
    Acknowledged(Operation),
}

struct Pending {
    request: Request,
    outgoing: Outgoing,
    sent: bool,
    started: u64,
}

/// One serialized source stream for one boot keyboard and one boot mouse.
/// Retains each exact message until its matching reply; EAGAIN never advances it.
/// The enumeration owner must prevent two live devices from sharing one role.
/// It must acknowledge the old role's detach before publishing its replacement,
/// and must never replay that old detach after replacement publication begins.
pub struct Source {
    config: Configuration,
    next_request: u64,
    last_tick: u64,
    online: bool,
    failed: bool,
    pending: Option<Pending>,
}

impl Source {
    pub fn new(config: Configuration) -> Result<Self, Error> {
        if config.epoch == 0
            || config.policy_pid == 0
            || config.request.handle() == config.response.handle()
        {
            return Err(Error::Fatal(Fatal::Configuration));
        }
        Ok(Self {
            config,
            next_request: 1,
            last_tick: 0,
            online: false,
            failed: false,
            pending: None,
        })
    }

    /// After a real loss/reset, quiesce transport work and reset its Sessions
    /// separately before synchronization. This acknowledgement does not select
    /// boot protocol or grant any device capability.
    pub fn synchronize(&mut self, now: u64) -> Result<(), Error> {
        self.available(now, false)?;
        self.online = false;
        self.queue(now, Operation::Sync, wire::HID_PROFILE, 0)
    }

    /// Publishes only a matching successful Session completion. Caller retries
    /// unchanged completion data after Pending; it must not consume it elsewhere.
    /// Failed, stale, malformed or oversized work creates no IPC request.
    pub fn complete_input(
        &mut self,
        now: u64,
        session: &mut Session,
        token: Token,
        status: CompletionStatus,
        bytes: &[u8],
    ) -> Result<(), Error> {
        self.available(now, true)?;
        session
            .complete_input(token, status, bytes)
            .map_err(Error::Session)?;
        let operation = match session.binding().protocol {
            Protocol::Keyboard => Operation::HidKeyboard,
            Protocol::Mouse => Operation::HidMouse,
        };
        let mut payload = [0; 8];
        payload[..bytes.len()].copy_from_slice(bytes);
        self.queue(
            now,
            operation,
            u64::from_le_bytes(payload),
            if operation == Operation::HidMouse {
                bytes.len() as u64
            } else {
                0
            },
        )
    }

    /// Invalidates the Session immediately, including when publication is busy.
    /// If Pending is returned, retain this detach notification and retry after
    /// the prior IPC acknowledgement. Late USB completions already remain invalid.
    pub fn detach(&mut self, now: u64, session: &mut Session) -> Result<(), Error> {
        let mask = match session.binding().protocol {
            Protocol::Keyboard => 1,
            Protocol::Mouse => 2,
        };
        session.detach();
        self.available(now, true)?;
        self.queue(now, Operation::HidDetach, mask, 0)
    }

    fn available(&mut self, now: u64, synchronized: bool) -> Result<(), Error> {
        if self.failed {
            return Err(Error::Fatal(Fatal::Protocol));
        }
        self.clock(now)?;
        if self.pending.is_some() {
            return Err(Error::Pending);
        }
        if synchronized && !self.online {
            return Err(Error::NotSynchronized);
        }
        if self.next_request == u64::MAX {
            return self.fail(Fatal::Counter);
        }
        Ok(())
    }

    fn queue(&mut self, now: u64, operation: Operation, arg0: u64, arg1: u64) -> Result<(), Error> {
        let request = Request {
            operation,
            id: self.next_request,
            epoch: self.config.epoch,
            arg0,
            arg1,
        };
        let outgoing = Outgoing::new(wire::TAG, &request.encode())
            .map_err(|_| Error::Fatal(Fatal::Protocol))?;
        self.next_request += 1; // available() excluded MAX before any Session mutation.
        self.pending = Some(Pending {
            request,
            outgoing,
            sent: false,
            started: now,
        });
        Ok(())
    }

    fn clock(&mut self, now: u64) -> Result<(), Error> {
        if now < self.last_tick {
            return self.fail(Fatal::Clock);
        }
        self.last_tick = now;
        Ok(())
    }

    fn fail<T>(&mut self, error: Fatal) -> Result<T, Error> {
        self.failed = true;
        self.online = false;
        self.pending = None;
        Err(Error::Fatal(error))
    }

    /// Uses the existing pre-granted request/reply endpoints. The native caller
    /// schedules retries and terminates/restarts its epoch on Fatal; this does
    /// not spin, create capabilities, infer a sender, or synthesize a reply.
    pub fn step(&mut self, now: u64, io: &mut impl Ipc) -> Result<Progress, Error> {
        if self.failed {
            return Err(Error::Fatal(Fatal::Protocol));
        }
        self.clock(now)?;
        let Some(pending) = self.pending.as_mut() else {
            return Ok(Progress::Idle);
        };
        if now - pending.started >= EXCHANGE_TICKS {
            return self.fail(Fatal::Deadline);
        }
        if !pending.sent {
            match io.send(self.config.request, &pending.outgoing) {
                Ok(()) => pending.sent = true,
                Err(error) if error.is_again() => return Ok(Progress::Pending),
                Err(error) => return self.fail(Fatal::Transport(error)),
            }
        }
        let request = pending.request;
        let response = match io.receive(self.config.response) {
            Ok(message) => message,
            Err(error) if error.is_again() => return Ok(Progress::Pending),
            Err(error) => return self.fail(Fatal::Transport(error)),
        };
        if response.cap_count() != 0 {
            if let Err(error) = discard_caps(io, &response) {
                return self.fail(Fatal::Cleanup(error));
            }
            return self.fail(Fatal::Protocol);
        }
        match protocol::command_reply(&response, self.config.policy_pid, request) {
            Ok(0) => {
                self.pending = None;
                if request.operation == Operation::Sync {
                    self.online = true;
                }
                Ok(Progress::Acknowledged(request.operation))
            }
            Ok(status) => self.fail(Fatal::Remote(status)),
            Err(_) => self.fail(Fatal::Protocol),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, SendEndpoint};

    #[test]
    fn source_counter_exhaustion_is_terminal_without_wrapping() {
        let mut source = Source::new(Configuration {
            epoch: 1,
            policy_pid: 10,
            request: SendEndpoint::from_handle(Handle::from_raw(0x1_0000_0001).unwrap()),
            response: ReceiveEndpoint::from_handle(Handle::from_raw(0x1_0000_0002).unwrap()),
        })
        .unwrap();
        source.next_request = u64::MAX;
        assert_eq!(source.synchronize(1), Err(Error::Fatal(Fatal::Counter)));
        assert_eq!(source.next_request, u64::MAX);
        assert!(source.pending.is_none());
        assert!(source.failed);
    }
}
