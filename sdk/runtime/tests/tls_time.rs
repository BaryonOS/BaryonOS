use nieuw_os_sdk_runtime::{
    clock::WallClockReading,
    tls_time::{certificate_time, TimeUnavailable},
};

#[test]
fn absent_source_is_an_explicit_failure() {
    assert_eq!(certificate_time(None), Err(TimeUnavailable::MissingSource));
}

#[test]
fn plausible_and_extreme_rtc_values_never_become_certificate_time() {
    for unix_seconds in [
        i64::MIN,
        -62_135_596_800,
        -1,
        0,
        1,
        1_793_361_600,
        253_402_300_799,
        i64::MAX,
    ] {
        assert_eq!(
            certificate_time(Some(WallClockReading::unauthenticated_rtc(unix_seconds))),
            Err(TimeUnavailable::UnauthenticatedRtc)
        );
    }
}

#[test]
fn stable_forward_and_backward_sequences_do_not_establish_trust() {
    for unix_seconds in [1_793_361_600, 1_793_361_600, 1_793_361_601, 1_793_361_599] {
        assert_eq!(
            certificate_time(Some(WallClockReading::unauthenticated_rtc(unix_seconds))),
            Err(TimeUnavailable::UnauthenticatedRtc)
        );
    }
    assert_eq!(certificate_time(None), Err(TimeUnavailable::MissingSource));
}

#[test]
fn none_cannot_be_substituted_by_a_successful_rtc_fallback() {
    let rtc_fallback = certificate_time(None)
        .or_else(|_| certificate_time(Some(WallClockReading::unauthenticated_rtc(1_793_361_600))));
    assert_eq!(rtc_fallback, Err(TimeUnavailable::UnauthenticatedRtc));
}
