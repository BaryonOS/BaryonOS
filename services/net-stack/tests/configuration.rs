//! Configuration lifecycle on synthetic time/frames, not native DHCP.
use nieuw_os_net_stack::{
    AddressConfiguration, Client, Error, LinkConfig, SocketStorage, Stack, State,
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
    let client = stack.add_client(&mut srx, &mut stx).unwrap();
    assert_eq!(stack.configuration(), None);
    assert_eq!(
        stack.connect(client, [198, 51, 100, 1], 443, 49152, 300),
        Err(Error::NotConfigured)
    );
    stack.apply_configuration(ADDRESS, 0).unwrap();
    assert_eq!(stack.configuration(), Some(ADDRESS));
    stack
        .connect(client, [198, 51, 100, 1], 443, 49152, 300)
        .unwrap();
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
    stack.poll(2).unwrap();
    assert!(
        stack.transmission().is_none(),
        "no old abort-RST or neighbor request under reapplied address"
    );
    stack
        .connect(client, [198, 51, 100, 1], 443, 49152, 300)
        .unwrap();
}

fn with_stack(test: impl FnOnce(&mut Stack<'_>, Client)) {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let (mut srx, mut stx) = ([0; 128], [0; 128]);
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
    let client = stack.add_client(&mut srx, &mut stx).unwrap();
    test(&mut stack, client);
}

#[test]
fn offlink_connection_uses_real_default_gateway_neighbor_request() {
    with_stack(|stack, client| {
        stack.apply_configuration(ADDRESS, 0).unwrap();
        stack
            .connect(client, [198, 51, 100, 1], 443, 49152, 300)
            .unwrap();
        stack.poll(0).unwrap();
        let frame = stack.transmission().unwrap();
        assert_eq!(&frame.bytes[12..14], &[8, 6]);
        assert_eq!(&frame.bytes[28..32], &ADDRESS.ipv4);
        assert_eq!(&frame.bytes[38..42], &ADDRESS.gateway.unwrap());
    });
}

#[test]
fn withdrawal_with_pending_tx_retains_id_and_blocks_reapply_until_true_completion() {
    with_stack(|stack, client| {
        stack.apply_configuration(ADDRESS, 0).unwrap();
        stack
            .connect(client, [198, 51, 100, 1], 443, 49152, 300)
            .unwrap();
        stack.poll(0).unwrap();
        let id = stack.transmission().unwrap().id;
        stack.withdraw_configuration(1).unwrap();
        assert!(stack.transmission().is_none());
        assert_eq!(stack.apply_configuration(ADDRESS, 1), Err(Error::Busy));
        assert_eq!(
            stack.complete_transmission(id + 1),
            Err(Error::StaleTransmission)
        );
        assert_eq!(stack.apply_configuration(ADDRESS, 1), Err(Error::Busy));
        stack.poll(2).unwrap();
        assert!(stack.transmission().is_none());
        // Model matching completion of already-published work, not permission
        // to complete an unsubmitted frame in a native adapter.
        stack.complete_transmission(id).unwrap();
        stack.apply_configuration(ADDRESS, 2).unwrap();
        stack.poll(2).unwrap();
        assert!(
            stack.transmission().is_none(),
            "old RST cleared without publication"
        );
        stack
            .connect(client, [198, 51, 100, 1], 443, 49152, 300)
            .unwrap();
        // Upstream retains interface-wide ARP rate limiting across cache
        // flushes. Advance real model ticks rather than require an instant TX.
        for ticks in 3..150 {
            stack.poll(ticks).unwrap();
            if let Some(frame) = stack.transmission() {
                assert!(frame.id > id);
                assert_eq!(&frame.bytes[12..14], &[8, 6]);
                return;
            }
        }
        panic!("no new ARP after bounded neighbor backoff");
    });
}

#[test]
fn rollback_or_overflow_on_withdraw_still_removes_address_route_and_fences() {
    for (now, error) in [
        (9, Error::ClockWentBackwards),
        (u64::MAX, Error::ClockOverflow),
    ] {
        with_stack(|stack, client| {
            stack.apply_configuration(ADDRESS, 10).unwrap();
            stack
                .connect(client, [198, 51, 100, 1], 443, 49152, 300)
                .unwrap();
            assert_eq!(stack.withdraw_configuration(now), Err(error));
            assert_eq!(stack.configuration(), None);
            assert_eq!(stack.state(client), Ok(State::Closed));
            assert!(stack.transmission().is_none());
            assert_eq!(stack.send(client, b"x"), Err(Error::NotConfigured));
            let mut out = [0x5a; 32];
            assert_eq!(stack.receive(client, &mut out), Err(Error::NotConfigured));
            assert_eq!(out, [0x5a; 32]);
            stack.apply_configuration(ADDRESS, 10).unwrap();
            stack.poll(10).unwrap();
            assert!(stack.transmission().is_none());
        });
    }
}

#[test]
fn invalid_address_or_gateway_never_changes_existing_configuration() {
    with_stack(|stack, _| {
        stack.apply_configuration(ADDRESS, 0).unwrap();
        for invalid in [
            AddressConfiguration {
                prefix_len: 0,
                ..ADDRESS
            },
            AddressConfiguration {
                prefix_len: 33,
                ..ADDRESS
            },
            AddressConfiguration {
                ipv4: [0; 4],
                ..ADDRESS
            },
            AddressConfiguration {
                ipv4: [127, 0, 0, 1],
                ..ADDRESS
            },
            AddressConfiguration {
                ipv4: [224, 0, 0, 1],
                ..ADDRESS
            },
            AddressConfiguration {
                ipv4: [192, 0, 2, 0],
                ..ADDRESS
            },
            AddressConfiguration {
                ipv4: [192, 0, 2, 255],
                ..ADDRESS
            },
            AddressConfiguration {
                gateway: Some(ADDRESS.ipv4),
                ..ADDRESS
            },
            AddressConfiguration {
                gateway: Some([192, 0, 3, 1]),
                ..ADDRESS
            },
            AddressConfiguration {
                gateway: Some([192, 0, 2, 0]),
                ..ADDRESS
            },
            AddressConfiguration {
                gateway: Some([192, 0, 2, 255]),
                ..ADDRESS
            },
        ] {
            assert_eq!(
                stack.apply_configuration(invalid, 0),
                Err(Error::InvalidConfiguration)
            );
            assert_eq!(stack.configuration(), Some(ADDRESS));
        }
    });
}

#[test]
fn active_socket_blocks_address_change_without_mutation() {
    with_stack(|stack, client| {
        stack.apply_configuration(ADDRESS, 0).unwrap();
        stack
            .connect(client, [198, 51, 100, 1], 443, 49152, 300)
            .unwrap();
        let next = AddressConfiguration {
            ipv4: [192, 0, 2, 3],
            ..ADDRESS
        };
        assert_eq!(stack.apply_configuration(next, 1), Err(Error::Busy));
        assert_eq!(stack.configuration(), Some(ADDRESS));
        assert_eq!(stack.state(client), Ok(State::SynSent));
        assert_eq!(
            stack.apply_configuration(next, u64::MAX),
            Err(Error::ClockOverflow)
        );
        assert_eq!(stack.configuration(), Some(ADDRESS));
    });
}

#[test]
fn queued_ingress_is_discarded_and_neighbor_cache_not_inherited() {
    with_stack(|stack, client| {
        stack.apply_configuration(ADDRESS, 0).unwrap();
        let mut reply = [0; 42];
        reply[..6].copy_from_slice(&[2, 0, 0, 0, 0, 7]);
        reply[6..12].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
        reply[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 2]);
        reply[22..28].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
        reply[28..32].copy_from_slice(&[192, 0, 2, 1]);
        reply[32..38].copy_from_slice(&[2, 0, 0, 0, 0, 7]);
        reply[38..42].copy_from_slice(&ADDRESS.ipv4);
        stack.push_frame(&reply).unwrap();
        assert!(!stack.can_receive_frame());
        stack.withdraw_configuration(0).unwrap();
        assert!(stack.can_receive_frame());
        stack.apply_configuration(ADDRESS, 0).unwrap();
        stack
            .connect(client, [198, 51, 100, 1], 443, 49152, 300)
            .unwrap();
        stack.poll(0).unwrap();
        let frame = stack.transmission().unwrap();
        assert_eq!(&frame.bytes[12..14], &[8, 6], "must ARP, not trust old RX");
    });
}
