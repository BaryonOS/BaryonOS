use nieuw_os_window_session::retirement::record;

#[test]
fn retirement_mask_is_bounded_monotonic_and_idempotent() {
    assert_eq!(record(0, 0), Some(1));
    assert_eq!(record(0, 1), Some(2));
    assert_eq!(record(1, 0), Some(1));
    assert_eq!(record(1, 1), Some(3));
    assert_eq!(record(2, 0), Some(3));
    assert_eq!(record(3, 1), Some(3));
    assert_eq!(record(0, 2), None);
    assert_eq!(record(4, 0), None);
}
