//! Regression candidate for PR #92 at 1490b77; modeled host frames only.
use nieuw_os_net_stack::{AddressConfiguration, LinkConfig, SocketStorage, Stack, State};

#[test]
fn cancelled_socket_cannot_emit_old_traffic_after_gateway_change() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let (mut socket_rx, mut socket_tx) = ([0; 128], [0; 128]);
    let mut storage = [SocketStorage::EMPTY];
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
    let client = stack.add_client(&mut socket_rx, &mut socket_tx).unwrap();
    let original = AddressConfiguration {
        ipv4: [192, 0, 2, 2],
        prefix_len: 24,
        gateway: Some([192, 0, 2, 1]),
    };
    stack.apply_configuration(original, 0).unwrap();
    stack
        .connect(client, [198, 51, 100, 1], 443, 49152, 300)
        .unwrap();
    stack.cancel(client).unwrap();
    assert_eq!(stack.state(client), Ok(State::Closed));
    assert!(stack.transmission().is_none());

    // The address stays unchanged. The new gateway is a different epoch,
    // not a request to transmit anything for the canceled connection.
    let replacement = AddressConfiguration {
        gateway: Some([192, 0, 2, 254]),
        ..original
    };
    stack.apply_configuration(replacement, 1).unwrap();
    assert_eq!(stack.configuration(), Some(replacement));
    stack.poll(1).unwrap();
    assert!(
        stack.transmission().is_none(),
        "old socket emitted a frame after gateway change: {:?}",
        stack.transmission()
    );
}
