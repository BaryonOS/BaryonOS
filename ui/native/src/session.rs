//! Bounded login interaction, without an authentication implementation.
//!
//! Only a trusted adapter may implement [`AuthBroker`] or deliver [`BrokerReply`].
//! That adapter must authenticate its IPC peer and enforce service/capability
//! binding. These Rust types do not authenticate an issuer or grant kernel rights.
//! The separate VM guest path never reports authenticated user identity.
use zeroize::Zeroize;

pub const MAX_PASSWORD_BYTES: usize = 128;
pub const TICKET_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountId(u64);
impl AccountId {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Account {
    LocalUser(AccountId),
    /// Explicit disposable VM profile, with no password or user-key unlock.
    VmGuest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMethod {
    Password,
    Fingerprint,
    Fido2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    AccountSelection,
    CredentialEntry,
    Authenticating,
    SessionStarting,
    Desktop,
    Locked,
    /// Terminal fail-closed state if a generation or request counter is spent.
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthFailure {
    Denied,
    Unavailable,
    TimedOut,
    InvalidReply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    WrongPhase,
    NoAccount,
    GuestHasNoCredential,
    EmptyPassword,
    PasswordFull,
    InvalidPasswordByte,
    StaleReply,
    InvalidTicket,
    CounterExhausted,
    Auth(AuthFailure),
}

/// Public correlation metadata; never a password, challenge, or authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthRequest {
    id: u64,
    epoch: u64,
    account: AccountId,
    method: AuthMethod,
}
impl AuthRequest {
    pub const fn id(self) -> u64 {
        self.id
    }
    pub const fn epoch(self) -> u64 {
        self.epoch
    }
    pub const fn account(self) -> AccountId {
        self.account
    }
    pub const fn method(self) -> AuthMethod {
        self.method
    }
}

/// Opaque service-issued activation handle, deliberately not Debug/Clone/Copy.
///
/// Construction is for the trusted broker adapter after authenticating the
/// service response. Nonzero bytes are only format validation, not verification
/// of the issuer. The session service must validate this handle again at use.
pub struct BrokerTicket([u8; TICKET_BYTES]);
impl BrokerTicket {
    pub fn from_broker(bytes: [u8; TICKET_BYTES]) -> Result<Self, Error> {
        if bytes.iter().all(|byte| *byte == 0) {
            Err(Error::InvalidTicket)
        } else {
            Ok(Self(bytes))
        }
    }

    pub fn as_bytes(&self) -> &[u8; TICKET_BYTES] {
        &self.0
    }
}
impl Drop for BrokerTicket {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// A trusted adapter's reply, bound to the complete original request.
/// Never decode an untrusted application's claim into this type.
pub struct BrokerReply {
    request: AuthRequest,
    outcome: Result<BrokerTicket, AuthFailure>,
}
impl BrokerReply {
    pub fn granted(request: AuthRequest, ticket: BrokerTicket) -> Self {
        Self {
            request,
            outcome: Ok(ticket),
        }
    }

    pub fn denied(request: AuthRequest, failure: AuthFailure) -> Self {
        Self {
            request,
            outcome: Err(failure),
        }
    }
}

/// Implement only at the trusted IPC boundary to separate authentication and
/// session services. `begin` must not retain the borrowed password. `activate`
/// must validate the handle, request binding, revocation and granted scope at
/// the service, then atomically start the authorized session or return failure.
/// A UI caller cannot make itself authenticated by setting a boolean.
pub trait AuthBroker {
    fn method_available(&self, account: AccountId, method: AuthMethod) -> bool;
    fn begin(&mut self, request: AuthRequest, password: &[u8]) -> Result<(), AuthFailure>;
    fn activate(&mut self, request: AuthRequest, ticket: &BrokerTicket) -> Result<(), AuthFailure>;
}

/// Safe native default until a real authenticated service adapter exists.
pub struct UnavailableBroker;
impl AuthBroker for UnavailableBroker {
    fn method_available(&self, _: AccountId, _: AuthMethod) -> bool {
        false
    }
    fn begin(&mut self, _: AuthRequest, _: &[u8]) -> Result<(), AuthFailure> {
        Err(AuthFailure::Unavailable)
    }
    fn activate(&mut self, _: AuthRequest, _: &BrokerTicket) -> Result<(), AuthFailure> {
        Err(AuthFailure::Unavailable)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionIdentity {
    Authenticated(AccountId),
    VmGuest,
}

// No Debug/Clone/Copy; renderers can observe only a length for masking.
struct Password {
    bytes: [u8; MAX_PASSWORD_BYTES],
    len: usize,
}
impl Password {
    const fn new() -> Self {
        Self {
            bytes: [0; MAX_PASSWORD_BYTES],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.bytes.zeroize();
        self.len = 0;
    }
}
impl Drop for Password {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Allocator-free interaction state. The already pinned zeroize implementation
/// clears owned password/ticket buffers; this does not erase temporary copies,
/// registers, crash images or another process's storage.
pub struct SessionFlow {
    phase: Phase,
    account: Option<Account>,
    method: AuthMethod,
    password: Password,
    epoch: u64,
    next_request: u64,
    pending: Option<AuthRequest>,
    ticket: Option<BrokerTicket>,
    identity: Option<SessionIdentity>,
    last_failure: Option<AuthFailure>,
}
impl Default for SessionFlow {
    fn default() -> Self {
        Self::new()
    }
}
impl SessionFlow {
    pub const fn new() -> Self {
        Self {
            phase: Phase::AccountSelection,
            account: None,
            method: AuthMethod::Password,
            password: Password::new(),
            epoch: 1,
            next_request: 1,
            pending: None,
            ticket: None,
            identity: None,
            last_failure: None,
        }
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }
    pub const fn selected_account(&self) -> Option<Account> {
        self.account
    }
    pub const fn method(&self) -> AuthMethod {
        self.method
    }
    pub const fn password_len(&self) -> usize {
        self.password.len
    }
    pub const fn pending_request(&self) -> Option<AuthRequest> {
        self.pending
    }
    pub const fn identity(&self) -> Option<SessionIdentity> {
        self.identity
    }
    pub const fn last_failure(&self) -> Option<AuthFailure> {
        self.last_failure
    }

    pub fn select_account(&mut self, account: Account) -> Result<(), Error> {
        self.invalidate()?;
        self.account = Some(account);
        self.method = AuthMethod::Password;
        self.phase = Phase::CredentialEntry;
        Ok(())
    }

    pub fn select_method(&mut self, method: AuthMethod) -> Result<(), Error> {
        if !matches!(self.phase, Phase::CredentialEntry | Phase::Authenticating) {
            return Err(Error::WrongPhase);
        }
        self.local_account()?;
        self.invalidate()?;
        self.method = method;
        self.phase = Phase::CredentialEntry;
        Ok(())
    }

    pub fn push_password_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.require_password_entry()?;
        if byte == 0 || byte.is_ascii_control() {
            return Err(Error::InvalidPasswordByte);
        }
        if self.password.len == MAX_PASSWORD_BYTES {
            return Err(Error::PasswordFull);
        }
        self.password.bytes[self.password.len] = byte;
        self.password.len += 1;
        Ok(())
    }

    /// Removes one UTF-8 scalar's bytes if the input adapter supplied UTF-8.
    pub fn backspace_password(&mut self) -> Result<(), Error> {
        self.require_password_entry()?;
        if self.password.len > 0 {
            let mut start = self.password.len - 1;
            while start > 0 && self.password.bytes[start] & 0xc0 == 0x80 {
                start -= 1;
            }
            self.password.bytes[start..self.password.len].zeroize();
            self.password.len = start;
        }
        Ok(())
    }

    pub fn clear_password(&mut self) {
        self.password.clear();
    }

    pub fn submit(&mut self, broker: &mut impl AuthBroker) -> Result<AuthRequest, Error> {
        if self.phase != Phase::CredentialEntry {
            return Err(Error::WrongPhase);
        }
        let account = self.local_account()?;
        if !broker.method_available(account, self.method) {
            self.password.clear();
            self.last_failure = Some(AuthFailure::Unavailable);
            return Err(Error::Auth(AuthFailure::Unavailable));
        }
        if self.method == AuthMethod::Password && self.password.len == 0 {
            return Err(Error::EmptyPassword);
        }
        let id = self.next_request;
        let Some(next) = self.next_request.checked_add(1) else {
            return self.exhausted();
        };
        self.next_request = next;
        let request = AuthRequest {
            id,
            epoch: self.epoch,
            account,
            method: self.method,
        };
        let result = broker.begin(request, &self.password.bytes[..self.password.len]);
        self.password.clear();
        match result {
            Ok(()) => {
                self.pending = Some(request);
                self.last_failure = None;
                self.phase = Phase::Authenticating;
                Ok(request)
            }
            Err(failure) => {
                self.last_failure = Some(failure);
                Err(Error::Auth(failure))
            }
        }
    }

    pub fn apply_reply(&mut self, reply: BrokerReply) -> Result<(), Error> {
        if self.phase != Phase::Authenticating || self.pending != Some(reply.request) {
            return Err(Error::StaleReply);
        }
        match reply.outcome {
            Ok(ticket) => {
                self.ticket = Some(ticket);
                self.phase = Phase::SessionStarting;
                Ok(())
            }
            Err(failure) => {
                self.pending = None;
                self.last_failure = Some(failure);
                self.phase = Phase::CredentialEntry;
                Err(Error::Auth(failure))
            }
        }
    }

    pub fn activate(&mut self, broker: &mut impl AuthBroker) -> Result<(), Error> {
        if self.phase != Phase::SessionStarting {
            return Err(Error::WrongPhase);
        }
        let request = self.pending.ok_or(Error::WrongPhase)?;
        let ticket = self.ticket.take().ok_or(Error::InvalidTicket)?;
        self.pending = None;
        match broker.activate(request, &ticket) {
            Ok(()) => {
                self.identity = Some(SessionIdentity::Authenticated(request.account));
                self.phase = Phase::Desktop;
                Ok(())
            }
            Err(failure) => {
                self.last_failure = Some(failure);
                self.phase = Phase::CredentialEntry;
                Err(Error::Auth(failure))
            }
        }
    }

    /// Explicit opt-in to a VM fixture. Does not call an authentication broker.
    pub fn begin_guest(&mut self) -> Result<(), Error> {
        if self.phase != Phase::CredentialEntry || self.account != Some(Account::VmGuest) {
            return Err(Error::WrongPhase);
        }
        self.invalidate()?;
        self.phase = Phase::SessionStarting;
        Ok(())
    }

    /// Activates only the selected disposable guest workspace, with no user keys.
    pub fn activate_guest(&mut self) -> Result<(), Error> {
        if self.phase != Phase::SessionStarting || self.account != Some(Account::VmGuest) {
            return Err(Error::WrongPhase);
        }
        self.identity = Some(SessionIdentity::VmGuest);
        self.phase = Phase::Desktop;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        self.invalidate()?;
        self.account = None;
        self.phase = Phase::AccountSelection;
        Ok(())
    }

    /// The native adapter must revoke/cover the real session before returning
    /// input to an untrusted surface. This model cannot revoke kernel grants.
    pub fn lock(&mut self) -> Result<(), Error> {
        self.invalidate()?;
        self.phase = Phase::Locked;
        Ok(())
    }

    /// Shows credentials again; does not restore any previous session identity.
    pub fn unlock(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Locked {
            return Err(Error::WrongPhase);
        }
        self.invalidate()?;
        self.phase = if self.account.is_some() {
            Phase::CredentialEntry
        } else {
            Phase::AccountSelection
        };
        Ok(())
    }

    pub fn sign_out(&mut self) -> Result<(), Error> {
        self.cancel()
    }

    fn local_account(&self) -> Result<AccountId, Error> {
        match self.account {
            Some(Account::LocalUser(account)) => Ok(account),
            Some(Account::VmGuest) => Err(Error::GuestHasNoCredential),
            None => Err(Error::NoAccount),
        }
    }

    fn require_password_entry(&self) -> Result<(), Error> {
        if self.phase != Phase::CredentialEntry || self.method != AuthMethod::Password {
            return Err(Error::WrongPhase);
        }
        self.local_account().map(|_| ())
    }

    fn invalidate(&mut self) -> Result<(), Error> {
        self.password.clear();
        self.pending = None;
        self.ticket = None;
        self.identity = None;
        self.last_failure = None;
        if self.phase == Phase::Unavailable {
            return Err(Error::CounterExhausted);
        }
        let Some(epoch) = self.epoch.checked_add(1) else {
            return self.exhausted();
        };
        self.epoch = epoch;
        Ok(())
    }

    fn exhausted<T>(&mut self) -> Result<T, Error> {
        self.password.clear();
        self.pending = None;
        self.ticket = None;
        self.identity = None;
        self.account = None;
        self.phase = Phase::Unavailable;
        Err(Error::CounterExhausted)
    }
}
