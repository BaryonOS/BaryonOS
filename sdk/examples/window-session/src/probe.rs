//! Native-only acceptance diagnostics on the app's actual granted channels.
//! No expected refusal creates a capability or acknowledges an input event.
use nieuw_os_input_client::{NativeTransport, ServiceError};
use nieuw_os_input_core::wire::{self, Operation, Request};
use nieuw_os_sdk_runtime::{
    native, Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};
use nieuw_os_surface_protocol::client::{Error as SurfaceError, Session as Surface};

const BUDGET: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Surface(SurfaceError),
    Runtime(RuntimeError),
    Interrupted,
    Protocol,
    ExpectedAccess,
    ExpectedBadHandle,
}
impl Error {
    pub const fn code(self) -> u64 {
        match self {
            Self::Configuration => 1,
            Self::Surface(_) => 2,
            Self::Runtime(_) => 3,
            Self::Interrupted => 4,
            Self::Protocol => 5,
            Self::ExpectedAccess => 6,
            Self::ExpectedBadHandle => 7,
        }
    }
}

/// Old grants are values only. Do not close them again: a stale generation
/// must never close a freshly reused slot. Actual kernel lookup must refuse
/// all four before any request can be published to the replacement service.
pub fn stale(handles: &[Handle; 4]) -> Result<u64, Error> {
    if handles
        .iter()
        .enumerate()
        .any(|(i, h)| handles[..i].contains(h))
    {
        return Err(Error::Configuration);
    }
    let message = Outgoing::new(wire::TAG, &[]).map_err(Error::Runtime)?;
    for handle in handles {
        match native::send(SendEndpoint::from_handle(*handle), &message) {
            Err(RuntimeError::Kernel(-4)) => {}
            Err(error) => return Err(Error::Runtime(error)),
            Ok(()) => return Err(Error::ExpectedBadHandle),
        }
    }
    Ok(4)
}

/// Call after SUR INFO and before constructing InputSession. WrongRole must
/// leave request id1 unused so the real InputSession's first PEEK still works.
/// The caller retains ownership of both input handles on every outcome.
pub fn run(
    surface: &mut Surface<NativeTransport>,
    input_send: SendEndpoint,
    input_receive: ReceiveEndpoint,
    compositor_pid: u64,
    input_epoch: u64,
    own_client: u64,
) -> Result<u64, Error> {
    if compositor_pid == 0
        || input_epoch == 0
        || !matches!(own_client, 1 | 2)
        || input_send.handle() == input_receive.handle()
    {
        return Err(Error::Configuration);
    }
    surface.verify_isolation_probe().map_err(Error::Surface)?;
    let request = Request {
        operation: Operation::Focus,
        id: 1,
        epoch: input_epoch,
        arg0: if own_client == 1 { 2 } else { 1 },
        arg1: 0,
    };
    let outgoing = Outgoing::new(wire::TAG, &request.encode()).map_err(Error::Runtime)?;

    // m4 checks SEND before copying/publishing. Engine::require reports
    // capability_core::Error::Access=-5 for this existing RECV-only handle.
    match native::send(SendEndpoint::from_handle(input_receive.handle()), &outgoing) {
        Err(RuntimeError::Kernel(-5)) => {}
        Err(error) => return Err(Error::Runtime(error)),
        Ok(()) => return Err(Error::ExpectedAccess),
    }
    let mut sent = false;
    for _ in 0..BUDGET {
        match native::send(input_send, &outgoing) {
            Ok(()) => {
                sent = true;
                break;
            }
            Err(error) if error.is_again() => native::yield_now().map_err(Error::Runtime)?,
            Err(error) => return Err(Error::Runtime(error)),
        }
    }
    if !sent {
        return Err(Error::Interrupted);
    }
    for _ in 0..BUDGET {
        match native::receive(input_receive) {
            Ok(message) => {
                if message.cap_count() != 0 {
                    // Refuse every cap and release the unexpected grants;
                    // never let a synthetic envelope certify a native check.
                    native::close_received_caps(&message).map_err(Error::Runtime)?;
                    return Err(Error::Protocol);
                }
                if message.sender() != compositor_pid
                    || message.tag() != wire::TAG
                    || message.payload()
                        != wire::failure(request, own_client, ServiceError::WrongRole)
                {
                    return Err(Error::Protocol);
                }
                return Ok(3);
            }
            Err(error) if error.is_again() => native::yield_now().map_err(Error::Runtime)?,
            Err(error) => return Err(Error::Runtime(error)),
        }
    }
    Err(Error::Interrupted)
}
