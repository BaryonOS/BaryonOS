use nieuw_os_usb_core::{
    boot_session::Lifetime,
    hotplug::{Error, Phase, Ports},
};

#[test]
fn detach_during_enumeration_retires_old_completion_and_releases_port() {
    let mut ports = Ports::new([10, 20], 4, 2).unwrap();
    let old = ports.attach(0).unwrap();
    assert_eq!(ports.attach(0), Err(Error::Busy));
    ports.detach(0).unwrap();
    assert_eq!(ports.phase(0), Ok(Phase::Detached));
    assert_eq!(ports.complete(old, true), Err(Error::Stale));
    let new = ports.attach(0).unwrap();
    assert_eq!(ports.complete(old, true), Err(Error::Stale));
    assert_eq!(ports.phase(0), Ok(Phase::Enumerating));
    assert_eq!(
        ports.complete(new, true),
        Ok(Some(Lifetime {
            device: 10,
            generation: 2
        }))
    );
    let other = ports.attach(1).unwrap();
    assert_eq!(
        ports.complete(other, true),
        Ok(Some(Lifetime {
            device: 20,
            generation: 1
        }))
    );
    assert_eq!(ports.complete(new, true), Err(Error::Stale));
}

#[test]
fn retry_requires_a_matched_failure_and_invalidates_the_earlier_attempt() {
    let mut ports = Ports::new([10], 1, u8::MAX).unwrap();
    let mut ticket = ports.attach(0).unwrap();
    assert_eq!(ports.retry(0), Err(Error::Busy));
    for attempt in 1..=u8::MAX {
        assert_eq!(ports.complete(ticket, false), Ok(None));
        assert_eq!(ports.complete(ticket, true), Err(Error::Stale));
        if attempt < u8::MAX {
            let next = ports.retry(0).unwrap();
            assert_eq!(ports.complete(ticket, true), Err(Error::Stale));
            ticket = next;
        }
    }
    assert_eq!(ports.phase(0), Ok(Phase::Failed));
    assert_eq!(ports.retry(0), Err(Error::Busy));
    ports.detach(0).unwrap();
    assert_eq!(ports.attach(0), Err(Error::Exhausted));
}

#[test]
fn storm_and_identity_bounds_do_not_reset_on_detach_or_cross_instances() {
    assert!(matches!(Ports::new([0], 4, 2), Err(Error::Invalid)));
    assert!(matches!(Ports::new([10, 10], 4, 2), Err(Error::Invalid)));
    assert!(matches!(Ports::new([10], 4, 0), Err(Error::Invalid)));
    let mut ports = Ports::new([10], 2, 1).unwrap();
    assert_eq!(ports.attach(1), Err(Error::Invalid));
    let old = ports.attach(0).unwrap();
    let mut foreign = Ports::new([30], 2, 1).unwrap();
    foreign.attach(0).unwrap();
    assert_eq!(foreign.complete(old, true), Err(Error::Stale));
    ports.detach(0).unwrap();
    ports.attach(0).unwrap();
    ports.detach(0).unwrap();
    assert_eq!(ports.attach(0), Err(Error::Exhausted));
    assert_eq!(ports.phase(0), Ok(Phase::Detached));
}
