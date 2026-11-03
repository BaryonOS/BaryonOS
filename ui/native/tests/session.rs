use nieuw_os_materiaal::session::{
    Account, AccountId, AuthBroker, AuthFailure, AuthMethod, AuthRequest, BrokerReply,
    BrokerTicket, Error, Phase, SessionFlow, SessionIdentity, UnavailableBroker,
    MAX_PASSWORD_BYTES,
};

#[derive(Default)]
struct ServiceFixture {
    begin_failure: Option<AuthFailure>,
    activation_failure: Option<AuthFailure>,
    calls: usize,
    activations: usize,
    received_password_len: usize,
}
impl AuthBroker for ServiceFixture {
    fn method_available(&self, _: AccountId, _: AuthMethod) -> bool {
        true
    }
    fn begin(&mut self, _: AuthRequest, password: &[u8]) -> Result<(), AuthFailure> {
        self.calls += 1;
        self.received_password_len = password.len();
        self.begin_failure.map_or(Ok(()), Err)
    }
    fn activate(&mut self, _: AuthRequest, ticket: &BrokerTicket) -> Result<(), AuthFailure> {
        assert_eq!(ticket.as_bytes(), &[7; 32]);
        self.activations += 1;
        self.activation_failure.map_or(Ok(()), Err)
    }
}

fn user(id: u64) -> Account {
    Account::LocalUser(AccountId::new(id).unwrap())
}
fn enter(flow: &mut SessionFlow) {
    for byte in b"synthetic-only" {
        flow.push_password_byte(*byte).unwrap();
    }
}
fn request(flow: &mut SessionFlow, broker: &mut ServiceFixture) -> AuthRequest {
    enter(flow);
    flow.submit(broker).unwrap()
}
fn grant(request: AuthRequest) -> BrokerReply {
    BrokerReply::granted(request, BrokerTicket::from_broker([7; 32]).unwrap())
}

#[test]
fn guest_is_an_explicit_separate_profile_and_never_an_authenticated_identity() {
    let mut flow = SessionFlow::new();
    assert_eq!(flow.phase(), Phase::AccountSelection);
    assert_eq!(flow.begin_guest(), Err(Error::WrongPhase));
    assert_eq!(flow.activate_guest(), Err(Error::WrongPhase));
    flow.select_account(user(1)).unwrap();
    assert_eq!(flow.begin_guest(), Err(Error::WrongPhase));
    flow.select_account(Account::VmGuest).unwrap();
    assert_eq!(
        flow.push_password_byte(b'x'),
        Err(Error::GuestHasNoCredential)
    );
    assert_eq!(
        flow.submit(&mut UnavailableBroker),
        Err(Error::GuestHasNoCredential)
    );
    flow.begin_guest().unwrap();
    assert_eq!(flow.phase(), Phase::SessionStarting);
    assert_eq!(flow.identity(), None);
    assert_eq!(
        flow.activate(&mut UnavailableBroker),
        Err(Error::WrongPhase)
    );
    flow.activate_guest().unwrap();
    assert_eq!(flow.identity(), Some(SessionIdentity::VmGuest));
    assert_eq!(flow.phase(), Phase::Desktop);
    assert_eq!(flow.activate_guest(), Err(Error::WrongPhase));
}

#[test]
fn native_default_rejects_every_password_and_clears_it() {
    for password in [
        b"password".as_slice(),
        b"anything",
        b"correct horse battery staple",
    ] {
        let mut flow = SessionFlow::new();
        flow.select_account(user(1)).unwrap();
        for byte in password {
            flow.push_password_byte(*byte).unwrap();
        }
        assert_eq!(
            flow.submit(&mut UnavailableBroker),
            Err(Error::Auth(AuthFailure::Unavailable))
        );
        assert_eq!(flow.password_len(), 0);
        assert_eq!(flow.last_failure(), Some(AuthFailure::Unavailable));
        assert_eq!(flow.phase(), Phase::CredentialEntry);
        assert_eq!(flow.identity(), None);
    }
}

#[test]
fn authentication_grant_still_requires_successful_service_activation() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    flow.select_account(user(1)).unwrap();
    let req = request(&mut flow, &mut broker);
    assert_eq!(req.account(), AccountId::new(1).unwrap());
    assert_eq!(req.method(), AuthMethod::Password);
    assert!(req.id() > 0 && req.epoch() > 0);
    assert_eq!(flow.password_len(), 0);
    assert_eq!(flow.phase(), Phase::Authenticating);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.activate(&mut broker), Err(Error::WrongPhase));
    flow.apply_reply(grant(req)).unwrap();
    assert_eq!(flow.phase(), Phase::SessionStarting);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.activate_guest(), Err(Error::WrongPhase));
    flow.activate(&mut broker).unwrap();
    assert_eq!(flow.phase(), Phase::Desktop);
    assert_eq!(
        flow.identity(),
        Some(SessionIdentity::Authenticated(req.account()))
    );
    assert_eq!(broker.activations, 1);
    assert_eq!(flow.activate(&mut broker), Err(Error::WrongPhase));
    assert_eq!(flow.apply_reply(grant(req)), Err(Error::StaleReply));
}

#[test]
fn denial_timeout_and_failed_activation_never_open_a_session() {
    for failure in [
        AuthFailure::Denied,
        AuthFailure::TimedOut,
        AuthFailure::InvalidReply,
    ] {
        let mut flow = SessionFlow::new();
        let mut broker = ServiceFixture::default();
        flow.select_account(user(1)).unwrap();
        let req = request(&mut flow, &mut broker);
        assert_eq!(
            flow.apply_reply(BrokerReply::denied(req, failure)),
            Err(Error::Auth(failure))
        );
        assert_eq!(flow.phase(), Phase::CredentialEntry);
        assert_eq!(flow.identity(), None);
        assert_eq!(flow.pending_request(), None);
        assert_eq!(flow.apply_reply(grant(req)), Err(Error::StaleReply));
    }
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture {
        activation_failure: Some(AuthFailure::Denied),
        ..Default::default()
    };
    flow.select_account(user(1)).unwrap();
    let req = request(&mut flow, &mut broker);
    flow.apply_reply(grant(req)).unwrap();
    assert_eq!(
        flow.activate(&mut broker),
        Err(Error::Auth(AuthFailure::Denied))
    );
    assert_eq!(flow.phase(), Phase::CredentialEntry);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.activate(&mut broker), Err(Error::WrongPhase));
}

#[test]
fn cancellation_account_switch_method_switch_and_lock_reject_delayed_replies() {
    let mut broker = ServiceFixture::default();
    for action in 0..4 {
        let mut flow = SessionFlow::new();
        flow.select_account(user(1)).unwrap();
        let old = request(&mut flow, &mut broker);
        match action {
            0 => flow.cancel().unwrap(),
            1 => flow.select_account(user(2)).unwrap(),
            2 => flow.select_method(AuthMethod::Fido2).unwrap(),
            3 => flow.lock().unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(flow.apply_reply(grant(old)), Err(Error::StaleReply));
        assert_eq!(flow.identity(), None);
        assert_eq!(flow.password_len(), 0);
    }
}

#[test]
fn complete_reply_binding_includes_account_and_method() {
    let mut actual = SessionFlow::new();
    let mut other = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    actual.select_account(user(1)).unwrap();
    other.select_account(user(2)).unwrap();
    let req = request(&mut actual, &mut broker);
    let wrong_account = request(&mut other, &mut broker);
    assert_eq!(req.id(), wrong_account.id());
    assert_eq!(req.epoch(), wrong_account.epoch());
    assert_eq!(
        actual.apply_reply(grant(wrong_account)),
        Err(Error::StaleReply)
    );
    assert_eq!(actual.pending_request(), Some(req));

    let mut actual = SessionFlow::new();
    let mut other = SessionFlow::new();
    actual.select_account(user(1)).unwrap();
    other.select_account(user(1)).unwrap();
    actual.select_method(AuthMethod::Password).unwrap();
    other.select_method(AuthMethod::Fingerprint).unwrap();
    let req = request(&mut actual, &mut broker);
    let wrong_method = other.submit(&mut broker).unwrap();
    assert_eq!(req.id(), wrong_method.id());
    assert_eq!(req.epoch(), wrong_method.epoch());
    assert_eq!(broker.received_password_len, 0);
    assert_eq!(
        actual.apply_reply(grant(wrong_method)),
        Err(Error::StaleReply)
    );
    actual.apply_reply(grant(req)).unwrap();
}

#[test]
fn retry_has_a_new_request_and_old_success_cannot_satisfy_it() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    flow.select_account(user(1)).unwrap();
    let old = request(&mut flow, &mut broker);
    flow.apply_reply(BrokerReply::denied(old, AuthFailure::Denied))
        .unwrap_err();
    let current = request(&mut flow, &mut broker);
    assert_ne!(old.id(), current.id());
    assert_eq!(flow.apply_reply(grant(old)), Err(Error::StaleReply));
    assert_eq!(flow.pending_request(), Some(current));
    flow.apply_reply(grant(current)).unwrap();
}

#[test]
fn lock_and_unlock_do_not_restore_guest_or_user_identity() {
    let mut flow = SessionFlow::new();
    flow.select_account(Account::VmGuest).unwrap();
    flow.begin_guest().unwrap();
    flow.activate_guest().unwrap();
    flow.lock().unwrap();
    assert_eq!(flow.phase(), Phase::Locked);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.activate_guest(), Err(Error::WrongPhase));
    flow.unlock().unwrap();
    assert_eq!(flow.phase(), Phase::CredentialEntry);
    assert_eq!(flow.identity(), None);
    flow.sign_out().unwrap();
    assert_eq!(flow.phase(), Phase::AccountSelection);
    assert_eq!(flow.selected_account(), None);
}

#[test]
fn input_is_bounded_masks_only_length_and_method_switch_discards_password() {
    assert_eq!(AccountId::new(0), None);
    assert!(matches!(
        BrokerTicket::from_broker([0; 32]),
        Err(Error::InvalidTicket)
    ));
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    flow.select_account(user(1)).unwrap();
    assert_eq!(flow.submit(&mut broker), Err(Error::EmptyPassword));
    assert_eq!(broker.calls, 0);
    for byte in [0, b'\n', b'\t', 0x7f] {
        assert_eq!(
            flow.push_password_byte(byte),
            Err(Error::InvalidPasswordByte)
        );
    }
    for _ in 0..MAX_PASSWORD_BYTES {
        flow.push_password_byte(b'x').unwrap();
    }
    assert_eq!(flow.push_password_byte(b'x'), Err(Error::PasswordFull));
    assert_eq!(flow.password_len(), MAX_PASSWORD_BYTES);
    flow.clear_password();
    for byte in "é".as_bytes() {
        flow.push_password_byte(*byte).unwrap();
    }
    flow.backspace_password().unwrap();
    assert_eq!(flow.password_len(), 0);
    enter(&mut flow);
    flow.select_method(AuthMethod::Fido2).unwrap();
    assert_eq!(flow.password_len(), 0);
    assert_eq!(flow.push_password_byte(b'x'), Err(Error::WrongPhase));
    let req = flow.submit(&mut broker).unwrap();
    assert_eq!(req.method(), AuthMethod::Fido2);
    assert_eq!(broker.received_password_len, 0);
}

#[test]
fn submission_error_clears_password_and_never_records_a_pending_request() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture {
        begin_failure: Some(AuthFailure::Unavailable),
        ..Default::default()
    };
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    assert_eq!(
        flow.submit(&mut broker),
        Err(Error::Auth(AuthFailure::Unavailable))
    );
    assert_eq!(flow.password_len(), 0);
    assert_eq!(flow.pending_request(), None);
    assert_eq!(flow.phase(), Phase::CredentialEntry);
    assert_eq!(flow.identity(), None);
}

#[test]
fn malformed_utf8_password_is_cleared_before_the_auth_broker_is_called() {
    for password in [
        &[0xc3][..],
        &[0x80][..],
        &[0xc0, 0xaf][..],
        &[0xed, 0xa0, 0x80][..],
    ] {
        let mut flow = SessionFlow::new();
        let mut broker = ServiceFixture::default();
        flow.select_account(user(1)).unwrap();
        for byte in password {
            flow.push_password_byte(*byte).unwrap();
        }

        assert_eq!(
            flow.submit(&mut broker),
            Err(Error::InvalidPasswordEncoding)
        );
        assert_eq!(broker.calls, 0);
        assert_eq!(flow.password_len(), 0);
        assert_eq!(flow.phase(), Phase::CredentialEntry);
        assert_eq!(flow.identity(), None);
        assert_eq!(flow.pending_request(), None);
    }
}
