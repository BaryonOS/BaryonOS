#![forbid(unsafe_code)]

//! Bounded coordination of one selected HID boot interface, without device I/O.
//!
//! The trusted enumeration owner supplies a nonreused device/attach identity,
//! verifies configuration and alternate-setting completion, and authenticates
//! the actual transport completions. Tokens match submitted work; they are not
//! capabilities. DMA, IRQs, buffer ownership, timeouts, focus and event delivery
//! remain the owner's responsibility. No numeric identity proves those rights.
//!
//! This initial profile accepts one input packet at a time, at most eight bytes.
//! It does not aggregate reports across packets or implement SET_IDLE/repeat.

use crate::{
    boot::{self, KeyboardChanges, KeyboardState, MouseChanges, MouseState},
    boot_interface::{self, BootInterface, Protocol, Speed},
    hid_control::Request,
};

/// Owner-assigned device identity and attach generation, not a USB address alone.
/// The pair must be unique across controllers and must not be reused while any
/// completion from that lifetime can still exist, including after session drop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Lifetime {
    pub device: u64,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Protocol,
    Input,
}

/// Opaque correlation metadata to retain alongside the actual submitted work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    lifetime: Lifetime,
    interface: u8,
    sequence: u64,
    kind: Kind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlSubmission {
    pub token: Token,
    pub request: Request,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputSubmission {
    pub token: Token,
    pub endpoint: u8,
    pub max_length: usize,
}

/// Must describe the authenticated transport result, including the control
/// status stage for SET_PROTOCOL. Preparing or queueing work is not success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionStatus {
    Success,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Changes {
    Keyboard(KeyboardChanges),
    Mouse(MouseChanges),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Binding(boot_interface::Error),
    PacketTooSmall,
    Detached,
    Pending,
    NotReady,
    AlreadyReady,
    UnexpectedCompletion,
    TransferFailed,
    Length,
    Report(boot::Error),
    TokenExhausted,
}

#[derive(Debug)]
enum State {
    Keyboard(KeyboardState),
    Mouse(MouseState),
}

#[derive(Debug, Eq, PartialEq)]
enum Phase {
    NeedsProtocol,
    Ready,
    Detached,
}

/// One owner, one interface, and at most one outstanding transfer. This state is
/// deliberately not Clone/Copy: duplicating it would duplicate live tokens.
#[derive(Debug)]
pub struct Session {
    lifetime: Lifetime,
    binding: BootInterface,
    sequence: u64,
    pending: Option<Token>,
    phase: Phase,
    state: State,
}

impl Session {
    /// Selects from the complete configuration snapshot rather than trusting an
    /// arbitrary public BootInterface value. The owner must independently bind
    /// this snapshot to its live, successfully configured device/alternate zero.
    pub fn new(
        lifetime: Lifetime,
        configuration: &[u8],
        interface: u8,
        speed: Speed,
    ) -> Result<Self, Error> {
        let binding =
            boot_interface::select(configuration, interface, speed).map_err(Error::Binding)?;
        let (minimum, state) = match binding.protocol {
            Protocol::Keyboard => (8, State::Keyboard(KeyboardState::new())),
            Protocol::Mouse => (3, State::Mouse(MouseState::new())),
        };
        if binding.max_packet_size < minimum {
            return Err(Error::PacketTooSmall);
        }
        Ok(Self {
            lifetime,
            binding,
            sequence: 0,
            pending: None,
            phase: Phase::NeedsProtocol,
            state,
        })
    }

    /// Selected metadata, not device/transfer authority.
    pub const fn binding(&self) -> BootInterface {
        self.binding
    }

    pub fn begin_protocol(&mut self) -> Result<ControlSubmission, Error> {
        self.idle()?;
        if self.phase == Phase::Ready {
            return Err(Error::AlreadyReady);
        }
        Ok(ControlSubmission {
            token: self.begin(Kind::Protocol)?,
            request: Request::set_boot_protocol(self.binding.interface_number),
        })
    }

    /// A matching completion is consumed even on failure. SET_PROTOCOL has no
    /// data stage; nonzero actual data length cannot make the interface ready.
    pub fn complete_protocol(
        &mut self,
        token: Token,
        status: CompletionStatus,
        actual_length: usize,
    ) -> Result<(), Error> {
        self.take(token, Kind::Protocol)?;
        if status == CompletionStatus::Failed {
            return Err(Error::TransferFailed);
        }
        if actual_length != 0 {
            return Err(Error::Length);
        }
        self.phase = Phase::Ready;
        Ok(())
    }

    pub fn begin_input(&mut self) -> Result<InputSubmission, Error> {
        self.idle()?;
        if self.phase != Phase::Ready {
            return Err(Error::NotReady);
        }
        Ok(InputSubmission {
            token: self.begin(Kind::Input)?,
            endpoint: self.binding.input_endpoint,
            max_length: usize::from(self.binding.max_packet_size.min(8)),
        })
    }

    /// Pass only the exact successfully completed bytes, never buffer capacity
    /// or an advertised report length. Matching failed/malformed work is consumed
    /// without changing held state; the owner must reset/detach on terminal loss.
    pub fn complete_input(
        &mut self,
        token: Token,
        status: CompletionStatus,
        bytes: &[u8],
    ) -> Result<Changes, Error> {
        self.take(token, Kind::Input)?;
        if status == CompletionStatus::Failed {
            return Err(Error::TransferFailed);
        }
        if bytes.len() > usize::from(self.binding.max_packet_size.min(8)) {
            return Err(Error::Length);
        }
        match &mut self.state {
            State::Keyboard(state) => state.update(bytes).map(Changes::Keyboard),
            State::Mouse(state) => state.update(bytes).map(Changes::Mouse),
        }
        .map_err(Error::Report)
    }

    /// Invalidates work and releases held input. The owner must quiesce/cancel
    /// transport work separately. Protocol selection is required again, and the
    /// sequence counter never restarts, even if the device retains its address.
    /// After a bus reset, re-establish configuration/alternate zero and verify
    /// the retained binding before submitting protocol work. Changed descriptors
    /// or device identity require detach and a newly selected session instead.
    pub fn reset(&mut self) -> Result<Changes, Error> {
        if self.phase == Phase::Detached {
            return Err(Error::Detached);
        }
        self.pending = None;
        self.phase = Phase::NeedsProtocol;
        Ok(self.clear())
    }

    /// Permanently invalidates this session. Repeated calls yield no releases.
    /// Reattach requires a fresh owner-assigned generation and a new session.
    pub fn detach(&mut self) -> Changes {
        self.pending = None;
        self.phase = Phase::Detached;
        self.clear()
    }

    fn clear(&mut self) -> Changes {
        match &mut self.state {
            State::Keyboard(state) => Changes::Keyboard(state.clear()),
            State::Mouse(state) => Changes::Mouse(state.clear()),
        }
    }

    fn idle(&self) -> Result<(), Error> {
        if self.phase == Phase::Detached {
            Err(Error::Detached)
        } else if self.pending.is_some() {
            Err(Error::Pending)
        } else {
            Ok(())
        }
    }

    fn begin(&mut self, kind: Kind) -> Result<Token, Error> {
        self.sequence = self.sequence.checked_add(1).ok_or(Error::TokenExhausted)?;
        let token = Token {
            lifetime: self.lifetime,
            interface: self.binding.interface_number,
            sequence: self.sequence,
            kind,
        };
        self.pending = Some(token);
        Ok(token)
    }

    fn take(&mut self, token: Token, kind: Kind) -> Result<(), Error> {
        // Wrong, old and duplicate completions cannot consume current work.
        if token.kind != kind || self.pending != Some(token) {
            return Err(Error::UnexpectedCompletion);
        }
        self.pending = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_exhaustion_never_wraps_or_restarts_after_reset() {
        let config = [
            9, 2, 34, 0, 1, 1, 0, 0x80, 50, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 0x21, 0x11, 1, 0, 1,
            0x22, 63, 0, 7, 5, 0x81, 3, 8, 0, 10,
        ];
        let mut session = Session::new(
            Lifetime {
                device: 1,
                generation: 1,
            },
            &config,
            0,
            Speed::Low,
        )
        .unwrap();
        session.sequence = u64::MAX - 1;
        let final_request = session.begin_protocol().unwrap();
        assert_eq!(final_request.token.sequence, u64::MAX);
        session
            .complete_protocol(final_request.token, CompletionStatus::Success, 0)
            .unwrap();
        assert_eq!(session.begin_input(), Err(Error::TokenExhausted));
        assert_eq!(session.pending, None);
        session.reset().unwrap();
        assert_eq!(session.begin_protocol(), Err(Error::TokenExhausted));
        assert_eq!(session.sequence, u64::MAX);
        assert_eq!(session.pending, None);
    }
}
