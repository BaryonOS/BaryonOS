//! Configuration lifecycle on synthetic time/frames, not native DHCP.
use nieuw_os_net_stack::{
    AddressConfiguration, Error, LinkConfig, SocketStorage, Stack, State,
};

const ADDRESS: AddressConfiguration = AddressConfiguration {
    ipv4: [192, 0, 2, 2],
    prefix_len: 24,
    gateway: Some([192, 0, 2, 1]),
};

#[test]
fn unconfigured_apply_withdraw_and_reapply_do_not_reuse_old_socket_authority() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let (mut srx, mut stx) = ([0; 128], [0; 128]);
    let mut storage = [SocketStorage::EMPTY];
    let mut stack = Stack::new_unconfigured(
        LinkConfig { mac: [2, 0, 0, 0, 0, 7], random_seed: 7 },
        &mut storage, &mut rx, &mut tx, 0,
    ).unwrap();
    let client = stack.add_client(&mut srx, &mut stx).unwrap();
    assert_eq!(stack.configuration(), None);
    assert_eq!(
        stack.connect(client, [198, 51, 100, 1], 443, 49152, 300),
        Err(Error::NotConfigured)
    );
    stack.apply_configuration(ADDRESS, 0).unwrap();
    assert_eq!(stack.configuration(), Some(ADDRESS));
    stack.connect(client, [198, 51, 100, 1], 443, 49152, 300).unwrap();
    assert_eq!(stack.state(client), Ok(State::SynSent));
    stack.withdraw_configuration(1).unwrap();
    assert_eq!(stack.configuration(), None);
    assert_eq!(stack.state(client), Ok(State::Closed));
    assert_eq!(
        stack.connect(client, [198, 51, 100, 1], 443, 49152, 300),
        Err(Error::NotConfigured)
    );
    assert!(stack.transmission().is_none());
    stack.apply_configuration(ADDRESS, 2).unwrap();
    stack.connect(client, [198, 51, 100, 1], 443, 49152, 300).unwrap();
}
