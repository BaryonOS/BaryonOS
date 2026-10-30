use nieuw_os_net_stack::{Error, FRAME_BYTES, LinkConfig, SocketStorage, Stack};

#[test]
fn control_frames_share_the_same_slot_identity_and_real_completion_path() {
    let mut storage = [SocketStorage::EMPTY];
    let mut rx = [0; FRAME_BYTES];
    let mut tx = [0; FRAME_BYTES];
    let mut stack = Stack::new_unconfigured(
        LinkConfig {
            mac: [2, 0, 0, 0, 0, 7],
            random_seed: 7,
        },
        &mut storage,
        &mut rx,
        &mut tx,
        0,
    )
    .unwrap();
    assert_eq!(
        stack.queue_control_frame(&[1; 13], 0),
        Err(Error::InvalidFrame)
    );
    let first = stack.queue_control_frame(&[1; 60], 1).unwrap();
    assert_eq!(stack.queue_control_frame(&[2; 60], 2), Err(Error::Busy));
    assert_eq!(stack.transmission().unwrap().bytes, &[1; 60]);
    assert_eq!(stack.transmission().unwrap().id, first);
    assert_eq!(
        stack.complete_transmission(first + 1),
        Err(Error::StaleTransmission)
    );
    stack.withdraw_configuration(2).unwrap();
    assert!(stack.transmission().is_none());
    assert_eq!(stack.queue_control_frame(&[2; 60], 3), Err(Error::Busy));
    stack.complete_transmission(first).unwrap();
    let second = stack.queue_control_frame(&[2; 60], 3).unwrap();
    assert!(second > first);
    assert_eq!(stack.transmission().unwrap().id, second);
    assert_eq!(stack.transmission().unwrap().bytes, &[2; 60]);
    assert_eq!(
        stack.queue_control_frame(&[3; 60], 2),
        Err(Error::ClockWentBackwards)
    );
    assert_eq!(
        stack.queue_control_frame(&[3; 60], u64::MAX),
        Err(Error::ClockOverflow)
    );
}
