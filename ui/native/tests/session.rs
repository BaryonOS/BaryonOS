use nieuw_os_materiaal::session::{
    Account, AccountId, AuthBroker, AuthContext, AuthFailure, AuthMethod, AuthPurpose, AuthRequest,
    BrokerReply, BrokerTicket, Error, Phase, RetryPolicy, SessionFlow, SessionIdentity,
    UnavailableBroker, AUTH_TIMEOUT_MS, MAX_PASSWORD_BYTES, MAX_RETRY_ACCOUNTS,
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

// These exercise a synthetic service boundary, never real credentials or keys.
fn context(boot: u64, session: u64, purpose: AuthPurpose) -> AuthContext {
    AuthContext::new(boot, session, purpose).unwrap()
}

#[test]
fn reply_binding_rejects_another_boot_session_or_purpose() {
    let original = context(11, 22, AuthPurpose::Login);
    for wrong in [
        context(12, 22, AuthPurpose::Login),
        context(11, 23, AuthPurpose::Login),
        context(11, 22, AuthPurpose::Unlock),
    ] {
        let mut actual = SessionFlow::with_context(original);
        let mut other = SessionFlow::with_context(wrong);
        let mut broker = ServiceFixture::default();
        actual.select_account(user(1)).unwrap();
        other.select_account(user(1)).unwrap();
        enter(&mut actual);
        enter(&mut other);
        let current = actual.submit_at(&mut broker, 100).unwrap();
        let foreign = other.submit_at(&mut broker, 100).unwrap();
        assert_eq!(current.id(), foreign.id());
        assert_eq!(current.epoch(), foreign.epoch());
        assert_eq!(current.context(), original);
        assert_eq!(
            actual.apply_reply_at(grant(foreign), 101),
            Err(Error::StaleReply)
        );
        assert_eq!(actual.pending_request(), Some(current));
        assert_eq!(actual.identity(), None);
    }
    assert_eq!(AuthContext::new(0, 1, AuthPurpose::Login), None);
    assert_eq!(AuthContext::new(1, 0, AuthPurpose::Login), None);
}

#[test]
fn deadline_expires_pending_reply_and_already_granted_activation() {
    for scenario in 0..3 {
        let mut flow = SessionFlow::with_context(context(1, 2, AuthPurpose::Unlock));
        let mut broker = ServiceFixture::default();
        flow.select_account(user(1)).unwrap();
        enter(&mut flow);
        let req = flow.submit_at(&mut broker, 10).unwrap();
        assert_eq!(req.deadline_ms(), 10 + AUTH_TIMEOUT_MS);
        let result = match scenario {
            0 => flow.advance_time(req.deadline_ms()),
            1 => flow.apply_reply_at(grant(req), req.deadline_ms()),
            2 => {
                flow.apply_reply_at(grant(req), 11).unwrap();
                flow.activate_at(&mut broker, req.deadline_ms())
            }
            _ => unreachable!(),
        };
        assert_eq!(result, Err(Error::Auth(AuthFailure::TimedOut)));
        assert_eq!(flow.phase(), Phase::CredentialEntry);
        assert_eq!(flow.identity(), None);
        assert_eq!(flow.pending_request(), None);
        assert_eq!(
            flow.apply_reply_at(grant(req), req.deadline_ms()),
            Err(Error::StaleReply)
        );
        assert_eq!(
            flow.activate_at(&mut broker, req.deadline_ms()),
            Err(Error::WrongPhase)
        );
        assert_eq!(broker.activations, 0);
    }
}

#[test]
fn clock_regression_and_deadline_overflow_fail_closed() {
    let mut broker = ServiceFixture::default();
    let mut flow = SessionFlow::with_context(context(1, 2, AuthPurpose::Login));
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    let req = flow.submit_at(&mut broker, 50).unwrap();
    assert_eq!(flow.apply_reply_at(grant(req), 49), Err(Error::InvalidTime));
    assert_eq!(flow.phase(), Phase::Unavailable);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.pending_request(), None);
    let mut flow = SessionFlow::new();
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    assert_eq!(
        flow.submit_at(&mut broker, u64::MAX),
        Err(Error::InvalidTime)
    );
    assert_eq!(flow.phase(), Phase::Unavailable);
    assert_eq!(flow.password_len(), 0);
}

#[test]
fn retry_policy_survives_method_account_cancel_and_lock_switches() {
    let retry = RetryPolicy::new(1_000);
    let mut flow = SessionFlow::with_context(context(1, 2, AuthPurpose::Login));
    let mut broker = ServiceFixture::default();
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    let req = flow.submit_at(&mut broker, 100).unwrap();
    assert_eq!(
        flow.apply_reply_at(
            BrokerReply::denied(req, AuthFailure::TemporarilyLocked(retry)),
            101
        ),
        Err(Error::Auth(AuthFailure::TemporarilyLocked(retry)))
    );
    for action in 0..4 {
        match action {
            0 => flow.select_method(AuthMethod::Fido2SecurityKey).unwrap(),
            1 => {
                flow.select_account(user(2)).unwrap();
                flow.select_account(user(1)).unwrap();
            }
            2 => {
                flow.cancel().unwrap();
                flow.select_account(user(1)).unwrap();
            }
            3 => {
                flow.lock().unwrap();
                flow.unlock().unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(flow.retry_policy(), Some(retry));
        assert_eq!(
            flow.submit_at(&mut broker, 500),
            Err(Error::Auth(AuthFailure::TemporarilyLocked(retry)))
        );
        assert_eq!(flow.identity(), None);
        assert_eq!(broker.calls, 1);
    }
    flow.select_method(AuthMethod::Fido2SecurityKey).unwrap();
    flow.submit_at(&mut broker, 1_000).unwrap();
    assert_eq!(broker.calls, 2);
    assert_eq!(flow.retry_policy(), None);
}

#[test]
fn retry_cache_does_not_evict_a_still_locked_account() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    for id in 1..=MAX_RETRY_ACCOUNTS as u64 {
        flow.select_account(user(id)).unwrap();
        enter(&mut flow);
        let req = flow.submit_at(&mut broker, 0).unwrap();
        assert_eq!(
            flow.apply_reply(BrokerReply::denied(
                req,
                AuthFailure::TemporarilyLocked(RetryPolicy::new(10_000))
            )),
            Err(Error::Auth(AuthFailure::TemporarilyLocked(
                RetryPolicy::new(10_000)
            )))
        );
    }
    flow.select_account(user(MAX_RETRY_ACCOUNTS as u64 + 1))
        .unwrap();
    enter(&mut flow);
    let req = flow.submit_at(&mut broker, 0).unwrap();
    assert_eq!(
        flow.apply_reply(BrokerReply::denied(
            req,
            AuthFailure::TemporarilyLocked(RetryPolicy::new(10_000))
        )),
        Err(Error::RetryCapacity)
    );
    assert_eq!(flow.phase(), Phase::Unavailable);
    assert_eq!(flow.identity(), None);
}

#[test]
fn missing_removed_cancelled_sensor_and_service_failure_never_become_success() {
    for method in [AuthMethod::Fingerprint, AuthMethod::Fido2SecurityKey] {
        for failure in [
            AuthFailure::HardwareUnavailable,
            AuthFailure::HardwareRemoved,
            AuthFailure::Cancelled,
            AuthFailure::TimedOut,
            AuthFailure::ServiceFailure,
        ] {
            let mut flow = SessionFlow::new();
            let mut broker = ServiceFixture::default();
            flow.select_account(user(1)).unwrap();
            flow.select_method(method).unwrap();
            let req = flow.submit_at(&mut broker, 0).unwrap();
            assert_eq!(
                flow.apply_reply_at(BrokerReply::denied(req, failure), 1),
                Err(Error::Auth(failure))
            );
            assert_eq!(flow.identity(), None);
            assert_eq!(flow.pending_request(), None);
            assert_eq!(flow.apply_reply_at(grant(req), 2), Err(Error::StaleReply));
            assert_eq!(broker.activations, 0);
        }
    }
}

#[test]
fn pin_and_recovery_require_their_own_bounded_secret_entry() {
    for method in [AuthMethod::LocalPin, AuthMethod::RecoveryMethod] {
        let mut flow = SessionFlow::new();
        let mut broker = ServiceFixture::default();
        flow.select_account(user(1)).unwrap();
        flow.select_method(method).unwrap();
        assert_eq!(flow.submit_at(&mut broker, 0), Err(Error::EmptyCredential));
        assert_eq!(flow.push_password_byte(b'x'), Err(Error::WrongPhase));
        for byte in b"123456" {
            flow.push_secret_byte(*byte).unwrap();
        }
        assert_eq!(flow.password_len(), 6);
        flow.backspace_secret().unwrap();
        assert_eq!(flow.password_len(), 5);
        let req = flow.submit_at(&mut broker, 1).unwrap();
        assert_eq!(req.method(), method);
        assert_eq!(broker.received_password_len, 5);
        assert_eq!(flow.password_len(), 0);
        assert_eq!(flow.identity(), None);
    }
    let mut flow = SessionFlow::new();
    flow.select_account(user(1)).unwrap();
    flow.select_method(AuthMethod::LocalPin).unwrap();
    assert_eq!(
        flow.push_secret_byte(b'a'),
        Err(Error::InvalidCredentialByte)
    );
    flow.push_secret_byte(b'1').unwrap();
    flow.select_method(AuthMethod::RecoveryMethod).unwrap();
    assert_eq!(flow.password_len(), 0);
}

#[test]
fn wrong_password_and_default_all_methods_are_closed() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture {
        begin_failure: Some(AuthFailure::Denied),
        ..Default::default()
    };
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    assert_eq!(
        flow.submit_at(&mut broker, 0),
        Err(Error::Auth(AuthFailure::Denied))
    );
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.password_len(), 0);
    for method in [
        AuthMethod::Password,
        AuthMethod::LocalPin,
        AuthMethod::Fingerprint,
        AuthMethod::Fido2SecurityKey,
        AuthMethod::RecoveryMethod,
    ] {
        flow.select_method(method).unwrap();
        assert_eq!(
            flow.submit_at(&mut UnavailableBroker, 1),
            Err(Error::Auth(AuthFailure::Unavailable))
        );
        assert_eq!(flow.identity(), None);
    }
}

#[test]
fn consumed_assertion_cannot_activate_twice_or_survive_logout_or_lock() {
    for logout in [false, true] {
        let mut flow = SessionFlow::with_context(context(1, 2, AuthPurpose::Login));
        let mut broker = ServiceFixture::default();
        flow.select_account(user(1)).unwrap();
        enter(&mut flow);
        let req = flow.submit_at(&mut broker, 0).unwrap();
        flow.apply_reply_at(grant(req), 1).unwrap();
        assert_eq!(flow.apply_reply_at(grant(req), 1), Err(Error::StaleReply));
        flow.activate_at(&mut broker, 2).unwrap();
        assert_eq!(flow.activate_at(&mut broker, 2), Err(Error::WrongPhase));
        if logout {
            flow.sign_out().unwrap();
        } else {
            flow.lock().unwrap();
        }
        assert_eq!(flow.identity(), None);
        assert_eq!(flow.apply_reply_at(grant(req), 3), Err(Error::StaleReply));
        assert_eq!(broker.activations, 1);
    }
}

#[test]
fn service_failure_or_removed_hardware_during_begin_or_activation_is_closed() {
    for failure in [
        AuthFailure::ServiceFailure,
        AuthFailure::HardwareRemoved,
        AuthFailure::HardwareUnavailable,
        AuthFailure::Cancelled,
        AuthFailure::TimedOut,
    ] {
        for at_activation in [false, true] {
            let mut flow = SessionFlow::new();
            let mut broker = ServiceFixture::default();
            flow.select_account(user(1)).unwrap();
            flow.select_method(AuthMethod::Fido2SecurityKey).unwrap();
            if at_activation {
                let req = flow.submit_at(&mut broker, 0).unwrap();
                flow.apply_reply_at(grant(req), 1).unwrap();
                broker.activation_failure = Some(failure);
                assert_eq!(flow.activate_at(&mut broker, 2), Err(Error::Auth(failure)));
                assert_eq!(flow.apply_reply_at(grant(req), 3), Err(Error::StaleReply));
            } else {
                broker.begin_failure = Some(failure);
                assert_eq!(flow.submit_at(&mut broker, 0), Err(Error::Auth(failure)));
            }
            assert_eq!(flow.phase(), Phase::CredentialEntry);
            assert_eq!(flow.identity(), None);
            assert_eq!(flow.pending_request(), None);
            assert_eq!(flow.password_len(), 0);
        }
    }
}

#[test]
fn malformed_retry_deadline_does_not_enable_immediate_retry() {
    let mut flow = SessionFlow::new();
    let mut broker = ServiceFixture::default();
    flow.select_account(user(1)).unwrap();
    enter(&mut flow);
    let req = flow.submit_at(&mut broker, 50).unwrap();
    assert_eq!(
        flow.apply_reply_at(
            BrokerReply::denied(req, AuthFailure::TemporarilyLocked(RetryPolicy::new(50))),
            50
        ),
        Err(Error::Auth(AuthFailure::InvalidReply))
    );
    assert_eq!(flow.phase(), Phase::Unavailable);
    assert_eq!(flow.identity(), None);
    assert_eq!(flow.pending_request(), None);
}
