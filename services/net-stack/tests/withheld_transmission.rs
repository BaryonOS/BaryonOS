//! Review regression for PR #96: a frame queued before withdrawal that the
//! adapter never received must have an explicit, non-completing recovery path.
use nieuw_os_net_stack::{
    AddressConfiguration, Error, FRAME_BYTES, LinkConfig, SocketStorage, Stack,
};

const ADDRESS: AddressConfiguration = AddressConfiguration {
    ipv4: [192, 0, 2, 2],
    prefix_len: 24,
    gateway: Some([192, 0, 2, 1]),
};

#[test]
fn never_offered_frame_withheld_by_withdrawal_can_be_discarded_by_exact_id() {
    let mut storage = [SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; FRAME_BYTES], [0; FRAME_BYTES]);
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
    assert_eq!(stack.withheld_transmission(), None);
    let probe = stack.queue_control_frame(&[1; 60], 0).unwrap();
    assert_eq!(
        stack.discard_withheld_transmission(probe),
        Err(Error::StaleTransmission),
        "an offered frame must not be discarded"
    );
    stack.withdraw_configuration(1).unwrap();
    assert!(stack.transmission().is_none());
    assert_eq!(stack.withheld_transmission(), Some(probe));
    assert_eq!(stack.queue_control_frame(&[2; 60], 1), Err(Error::Busy));
    assert_eq!(stack.apply_configuration(ADDRESS, 1), Err(Error::Busy));
    assert_eq!(
        stack.discard_withheld_transmission(probe + 1),
        Err(Error::StaleTransmission)
    );
    assert_eq!(stack.withheld_transmission(), Some(probe));

    stack.discard_withheld_transmission(probe).unwrap();
    assert_eq!(stack.withheld_transmission(), None);
    assert!(stack.transmission().is_none(), "discard never publishes");
    assert_eq!(
        stack.discard_withheld_transmission(probe),
        Err(Error::StaleTransmission)
    );
    assert_eq!(
        stack.complete_transmission(probe),
        Err(Error::StaleTransmission)
    );
    stack.apply_configuration(ADDRESS, 2).unwrap();
    let next = stack.queue_control_frame(&[2; 60], 2).unwrap();
    assert!(next > probe);
    assert_eq!(stack.transmission().unwrap().id, next);
}
