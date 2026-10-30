use capability_core::{Cap, Engine, Error, Message, DISPLAY, INPUT};
use nieuw_os_native_display_input_probe::contract::SOURCE_RESTRICTED_ERROR;

#[test]
fn fixture_source_restrictions_match_real_engine_and_are_atomic() {
    let mut engine = Engine::new();
    engine.register(1).unwrap();
    let endpoint = engine.create_endpoint(1).unwrap();
    let display = engine.create_display(1).unwrap();
    let input = engine.create_input(1).unwrap();
    for (handle, rights) in [(display, DISPLAY), (input, INPUT)] {
        let before = engine.clone();
        assert_eq!(
            engine.derive(1, handle, rights).unwrap_err().code(),
            SOURCE_RESTRICTED_ERROR
        );
        assert_eq!(engine, before);
        assert_eq!(
            engine.derive(1, handle, 0).unwrap_err().code(),
            SOURCE_RESTRICTED_ERROR
        );
        assert_eq!(engine, before);
        assert_eq!(
            engine.revoke(1, handle).unwrap_err().code(),
            SOURCE_RESTRICTED_ERROR
        );
        assert_eq!(engine, before);
        let mut message = Message::new();
        message.cap_count = 1;
        message.caps[0] = Cap { handle, rights };
        assert_eq!(
            engine.send(1, endpoint, &message).unwrap_err().code(),
            SOURCE_RESTRICTED_ERROR
        );
        assert_eq!(engine, before);
        assert!(engine.check_invariants());
    }
}

#[test]
fn fixture_manifest_integers_cannot_cross_pid_type_or_lifetime_boundaries() {
    let mut engine = Engine::new();
    for pid in 1..=3 {
        engine.register(pid).unwrap();
    }
    let display_wrong = engine.create_endpoint(1).unwrap();
    let display = engine.create_display(1).unwrap();
    let input_wrong = engine.create_endpoint(2).unwrap();
    engine.create_endpoint(2).unwrap();
    let input = engine.create_input(2).unwrap();
    let intruder_wrong = engine.create_endpoint(3).unwrap();
    assert_eq!(engine.check_display(1, display_wrong), Err(Error::Type));
    assert_eq!(engine.check_input(1, input), Err(Error::BadHandle));
    assert_eq!(engine.check_display(2, display), Err(Error::Type));
    assert_eq!(engine.check_input(2, input_wrong), Err(Error::Type));
    assert_eq!(engine.check_display(3, display), Err(Error::BadHandle));
    assert_eq!(engine.check_input(3, input), Err(Error::BadHandle));
    assert_eq!(engine.check_display(3, intruder_wrong), Err(Error::Type));
    let mut old_sources = [display, input];
    for pid in 1..=3 {
        engine.exit(pid).unwrap();
    }
    for pid in 4..=6 {
        engine.register(pid).unwrap();
    }
    engine.create_endpoint(4).unwrap();
    let fresh_display = engine.create_display(4).unwrap();
    engine.create_endpoint(5).unwrap();
    engine.create_endpoint(5).unwrap();
    let fresh_input = engine.create_input(5).unwrap();
    engine.create_endpoint(6).unwrap();
    assert_eq!(
        engine.check_display(4, old_sources[0]),
        Err(Error::BadHandle)
    );
    assert_eq!(engine.check_input(5, old_sources[1]), Err(Error::BadHandle));
    old_sources = [fresh_display, fresh_input];
    engine.close(4, fresh_display).unwrap();
    engine.close(5, fresh_input).unwrap();
    assert_eq!(
        engine.check_display(4, old_sources[0]),
        Err(Error::BadHandle)
    );
    assert_eq!(engine.check_input(5, old_sources[1]), Err(Error::BadHandle));
    assert!(engine.check_invariants());
}
