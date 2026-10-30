use nieuw_os_net_tls::time::{checked_time, CertificateClock, Error};
use nieuw_os_sdk_runtime::{
    clock::WallClockReading, tls_time::TimeUnavailable, Error as RuntimeError,
};
use rustls::time_provider::TimeProvider;

#[test]
fn missing_time_is_not_substituted() {
    let clock = CertificateClock::new(|| Ok(None));
    assert_eq!(
        clock.checked_now(),
        Err(Error::Unavailable(TimeUnavailable::MissingSource))
    );
    assert_eq!(clock.current_time(), None);
}

#[test]
fn rtc_extremes_and_plausible_times_never_become_certificate_time() {
    for seconds in [i64::MIN, -1, 0, 1_793_361_600, i64::MAX] {
        assert_eq!(
            checked_time(Some(WallClockReading::unauthenticated_rtc(seconds))),
            Err(TimeUnavailable::UnauthenticatedRtc)
        );
    }
    let clock =
        CertificateClock::new(|| Ok(Some(WallClockReading::unauthenticated_rtc(1_793_361_600))));
    assert_eq!(clock.current_time(), None);
    assert_eq!(
        clock.checked_now(),
        Err(Error::Unavailable(TimeUnavailable::UnauthenticatedRtc))
    );
}

#[test]
fn source_error_survives_the_checked_interface_and_denies_rustls_time() {
    let clock = CertificateClock::new(|| Err(RuntimeError::Kernel(-9)));
    assert_eq!(
        clock.checked_now(),
        Err(Error::Source(RuntimeError::Kernel(-9)))
    );
    assert_eq!(clock.current_time(), None);
}
