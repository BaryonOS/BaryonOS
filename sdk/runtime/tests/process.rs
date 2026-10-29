use nieuw_os_sdk_runtime::{
    process::{CODE_MASK, STATE_EXITED, STATE_FAULTED, STATE_KILLED, STATE_RUNNING},
    validate_abi, ChildProcess, Error, Handle, LaunchImage, ProcessStatus,
};

#[test]
fn abi_1_3_is_accepted_and_1_4_is_not() {
    assert_eq!(validate_abi((1 << 32) | 3), Ok(()));
    assert_eq!(validate_abi((1 << 32) | 4), Err(Error::UnsupportedAbi));
}

#[test]
fn status_words_round_trip_and_reject_noncanonical_values() {
    for status in [
        ProcessStatus::Running,
        ProcessStatus::Exited(0),
        ProcessStatus::Exited(7),
        ProcessStatus::Exited(CODE_MASK),
        ProcessStatus::Faulted(6),
        ProcessStatus::Faulted(14),
        ProcessStatus::Killed,
    ] {
        assert_eq!(ProcessStatus::decode(status.encode()), Ok(status));
    }
    assert_eq!(
        ProcessStatus::decode((STATE_EXITED << 56) | CODE_MASK),
        Ok(ProcessStatus::Exited(CODE_MASK))
    );
    for bad in [
        0,
        (STATE_RUNNING << 56) | 1,
        (STATE_KILLED << 56) | 1,
        (STATE_FAULTED << 56) | 256,
        5 << 56,
        255 << 56,
        u64::MAX,
    ] {
        assert_eq!(ProcessStatus::decode(bad), Err(Error::InvalidReturn));
    }
    assert!(ProcessStatus::Running.is_running());
    assert!(!ProcessStatus::Killed.is_running());
}

#[test]
fn typed_handles_keep_their_representation() {
    let handle = Handle::from_raw((3 << 32) | 9).unwrap();
    assert_eq!(LaunchImage::from_handle(handle).handle(), handle);
    assert_eq!(ChildProcess::from_handle(handle).handle(), handle);
}
