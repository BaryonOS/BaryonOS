//! Algorithm/model tests only: timestamps and Ethernet frames are fixtures.
//! They do not exercise the Nieuw OS kernel, NIC, clock or capability boundary.
use nieuw_os_net_stack::{Config, Error, PacketDevice, SocketStorage, Stack, State};
use smoltcp::{
    phy::{Device, RxToken, TxToken},
    time::Instant,
};

const CONFIG: Config = Config {
    mac: [2, 0, 0, 0, 0, 7],
    ipv4: [192, 0, 2, 2],
    prefix_len: 24,
    random_seed: 7,
};

fn arp_reply() -> [u8; 42] {
    let mut frame = [0u8; 42];
    frame[..6].copy_from_slice(&CONFIG.mac);
    frame[6..12].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
    frame[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 2]);
    frame[22..28].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
    frame[28..32].copy_from_slice(&[192, 0, 2, 1]);
    frame[32..38].copy_from_slice(&CONFIG.mac);
    frame[38..42].copy_from_slice(&CONFIG.ipv4);
    frame
}

#[test]
fn malformed_or_busy_rx_does_not_overwrite_queued_frame() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut device = PacketDevice::new(&mut rx, &mut tx);
    assert_eq!(device.push(&[1; 13]), Err(Error::InvalidFrame));
    assert_eq!(device.push(&[1; 1515]), Err(Error::InvalidFrame));
    device.push(&[7; 1514]).unwrap();
    assert_eq!(device.push(&[2; 60]), Err(Error::Busy));
    let (receive, _) = device.receive(Instant::ZERO).unwrap();
    receive.consume(|bytes| assert_eq!(bytes, &[7; 1514]));
    assert!(device.can_receive());
}

#[test]
fn tx_backpressure_and_stale_completions_preserve_current_packet() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut device = PacketDevice::new(&mut rx, &mut tx);
    assert_eq!(device.complete(0), Err(Error::StaleTransmission));
    device
        .transmit(Instant::ZERO)
        .unwrap()
        .consume(60, |out| out.fill(3));
    let first = device.transmission().unwrap().id;
    assert!(device.transmit(Instant::ZERO).is_none());
    assert_eq!(device.complete(first + 1), Err(Error::StaleTransmission));
    assert_eq!(device.transmission().unwrap().bytes, &[3; 60]);
    device.complete(first).unwrap();
    device
        .transmit(Instant::ZERO)
        .unwrap()
        .consume(1514, |out| out.fill(4));
    assert_eq!(device.complete(first), Err(Error::StaleTransmission));
    assert_eq!(device.transmission().unwrap().id, first + 1);
    assert_eq!(device.transmission().unwrap().bytes, &[4; 1514]);
}

#[test]
fn rx_is_not_consumed_while_tx_cannot_answer() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut device = PacketDevice::new(&mut rx, &mut tx);
    device
        .transmit(Instant::ZERO)
        .unwrap()
        .consume(60, |out| out.fill(3));
    let id = device.transmission().unwrap().id;
    device.push(&[7; 60]).unwrap();
    assert!(device.receive(Instant::ZERO).is_none());
    assert!(!device.can_receive());
    device.complete(id).unwrap();
    let (receive, _) = device.receive(Instant::ZERO).unwrap();
    receive.consume(|bytes| assert_eq!(bytes, &[7; 60]));
}

#[test]
fn bounds_and_invalid_initial_identity_are_rejected() {
    for config in [
        Config {
            mac: [0; 6],
            ..CONFIG
        },
        Config {
            mac: [1, 0, 0, 0, 0, 7],
            ..CONFIG
        },
        Config {
            ipv4: [0; 4],
            ..CONFIG
        },
        Config {
            ipv4: [224, 0, 0, 1],
            ..CONFIG
        },
        Config {
            ipv4: [127, 0, 0, 1],
            ..CONFIG
        },
        Config {
            prefix_len: 33,
            ..CONFIG
        },
    ] {
        let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
        let mut storage = [SocketStorage::EMPTY];
        assert!(matches!(
            Stack::new(config, &mut storage, &mut rx, &mut tx, 0),
            Err(Error::InvalidConfiguration)
        ));
    }
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut storage = [
        SocketStorage::EMPTY,
        SocketStorage::EMPTY,
        SocketStorage::EMPTY,
    ];
    assert!(matches!(
        Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0),
        Err(Error::InvalidConfiguration)
    ));
}

#[test]
fn socket_buffers_and_count_remain_bounded_after_rejection() {
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut storage = [SocketStorage::EMPTY];
    let (mut too_big, mut empty, mut invalid_tx) = ([0; 8193], [], [0; 8]);
    let (mut srx, mut stx, mut more_rx, mut more_tx) = ([0; 8192], [0; 8192], [0; 8], [0; 8]);
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0).unwrap();
    assert_eq!(
        engine.add_client(&mut too_big, &mut invalid_tx),
        Err(Error::InvalidConfiguration)
    );
    // Rejected borrows remain scoped to engine lifetime, so use another TX buffer.
    assert_eq!(
        engine.add_client(&mut empty, &mut more_tx),
        Err(Error::InvalidConfiguration)
    );
    let client = engine.add_client(&mut srx, &mut stx).unwrap();
    let mut final_tx = [0; 8];
    assert_eq!(
        engine.add_client(&mut more_rx, &mut final_tx),
        Err(Error::SocketLimit)
    );
    assert_eq!(engine.state(client).unwrap(), State::Closed);
    assert_eq!(
        engine.connect(client, [192, 0, 2, 1], 8080, 40000, 0),
        Err(Error::InvalidConfiguration)
    );
    assert_eq!(
        engine.connect(client, [192, 0, 2, 1], 8080, 40000, 6001),
        Err(Error::InvalidConfiguration)
    );
    engine
        .connect(client, [192, 0, 2, 1], 8080, 40000, 300)
        .unwrap();
    assert_eq!(engine.state(client).unwrap(), State::SynSent);
}

#[test]
fn actual_syn_is_generated_only_after_arp_and_tx_is_completion_gated() {
    let (mut rx, mut tx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 4096], [0; 4096]);
    let mut storage = [SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let client = engine.add_client(&mut srx, &mut stx).unwrap();
    engine
        .connect(client, [192, 0, 2, 1], 8080, 40000, 300)
        .unwrap();
    engine.poll(0).unwrap();
    let arp = engine.transmission().unwrap();
    assert_eq!(&arp.bytes[12..14], &[8, 6]);
    assert_eq!(&arp.bytes[38..42], &[192, 0, 2, 1]);
    let id = arp.id;
    engine.push_frame(&arp_reply()).unwrap();
    engine.poll(1).unwrap();
    assert!(!engine.can_receive_frame());
    assert_eq!(engine.transmission().unwrap().id, id);
    engine.complete_transmission(id).unwrap();
    // smoltcp backs off neighbor discovery for one second after an ARP request.
    engine.poll(101).unwrap();
    let syn = engine.transmission().unwrap();
    assert_eq!(&syn.bytes[12..14], &[8, 0]);
    assert_eq!(syn.bytes[23], 6);
    assert_eq!(&syn.bytes[34..38], &[0x9c, 0x40, 0x1f, 0x90]);
    assert_eq!(syn.bytes[47] & 0x17, 2);
    assert_eq!(engine.state(client).unwrap(), State::SynSent);
}

#[test]
fn rejected_clock_does_not_mutate_time_and_equal_ticks_do_not_timeout() {
    let (mut rx, mut tx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 256], [0; 256]);
    let mut storage = [SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 10).unwrap();
    let client = engine.add_client(&mut srx, &mut stx).unwrap();
    engine
        .connect(client, [192, 0, 2, 1], 8084, 40000, 100)
        .unwrap();
    assert_eq!(engine.poll(9), Err(Error::ClockWentBackwards));
    assert_eq!(engine.poll(u64::MAX), Err(Error::ClockOverflow));
    assert_eq!(
        engine.poll((i64::MAX / 10_000) as u64),
        Err(Error::ClockOverflow)
    );
    for _ in 0..1000 {
        engine.poll(10).unwrap();
    }
    assert_eq!(engine.state(client).unwrap(), State::SynSent);
    if let Some(frame) = engine.transmission() {
        let id = frame.id;
        engine.complete_transmission(id).unwrap();
    }
    engine.push_frame(&arp_reply()).unwrap();
    engine.poll(110).unwrap();
    if let Some(frame) = engine.transmission() {
        let id = frame.id;
        engine.complete_transmission(id).unwrap();
    }
    engine.poll(211).unwrap();
    assert_eq!(engine.state(client).unwrap(), State::Closed);
}

#[test]
fn malformed_ethernet_corpus_never_creates_socket_data() {
    let (mut rx, mut tx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 64], [0; 64]);
    let mut storage = [SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let client = engine.add_client(&mut srx, &mut stx).unwrap();
    for size in 14..=1514 {
        let mut frame = [0; 1514];
        frame[..6].copy_from_slice(&CONFIG.mac);
        frame[6..12].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
        frame[12..14].copy_from_slice(&[8, 0]);
        // Version4/IHL0 was a historically fixed parser panic path.
        if size > 14 {
            frame[14] = 0x40;
        }
        engine.push_frame(&frame[..size]).unwrap();
        engine.poll(size as u64).unwrap();
        assert!(engine.can_receive_frame());
        assert!(engine.transmission().is_none());
        assert_eq!(engine.state(client).unwrap(), State::Closed);
    }
}

#[test]
fn duplicate_tuple_cannot_take_first_clients_replies_and_cancel_closes() {
    let (mut rx, mut tx, mut arx, mut atx, mut brx, mut btx) =
        ([0; 1514], [0; 1514], [0; 64], [0; 64], [0; 64], [0; 64]);
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let first = engine.add_client(&mut arx, &mut atx).unwrap();
    let second = engine.add_client(&mut brx, &mut btx).unwrap();
    engine
        .connect(first, [192, 0, 2, 1], 8080, 40000, 300)
        .unwrap();
    assert_eq!(
        engine.connect(second, [192, 0, 2, 1], 8080, 40000, 300),
        Err(Error::Busy)
    );
    assert_eq!(engine.state(first).unwrap(), State::SynSent);
    assert_eq!(engine.state(second).unwrap(), State::Closed);
    engine
        .connect(second, [192, 0, 2, 1], 8081, 40000, 300)
        .unwrap();
    engine.cancel(first).unwrap();
    assert_eq!(engine.state(first).unwrap(), State::Closed);
    assert_eq!(engine.state(second).unwrap(), State::SynSent);
}

#[test]
fn listener_bounds_duplicates_cancel_and_relisten_preserve_other_sockets() {
    let (mut rx, mut tx, mut arx, mut atx, mut brx, mut btx) =
        ([0; 1514], [0; 1514], [0; 64], [0; 64], [0; 64], [0; 64]);
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let first = engine.add_client(&mut arx, &mut atx).unwrap();
    let second = engine.add_client(&mut brx, &mut btx).unwrap();
    assert_eq!(
        engine.listen(first, 8080, 0),
        Err(Error::InvalidConfiguration)
    );
    assert_eq!(
        engine.listen(first, 8080, 6001),
        Err(Error::InvalidConfiguration)
    );
    assert_eq!(engine.listen(first, 0, 100), Err(Error::Unaddressable));
    assert_eq!(engine.state(first), Ok(State::Closed));
    engine.listen(first, 8080, 100).unwrap();
    assert_eq!(engine.remote_endpoint(first), Ok(None));
    assert_eq!(engine.listen(first, 8081, 100), Err(Error::InvalidState));
    assert_eq!(engine.listen(second, 8080, 100), Err(Error::Busy));
    assert_eq!(engine.state(second), Ok(State::Closed));
    assert_eq!(
        engine.send(first, b"not connected"),
        Err(Error::InvalidState)
    );
    assert_eq!(engine.receive(first, &mut [0; 8]), Err(Error::InvalidState));
    engine.listen(second, 8081, 200).unwrap();
    engine.cancel(first).unwrap();
    assert_eq!(engine.state(first), Ok(State::Closed));
    assert_eq!(engine.state(second), Ok(State::Listen));
    engine.listen(first, 8080, 100).unwrap();
    engine.close(first).unwrap();
    engine.listen(first, 8080, 100).unwrap();
    engine.poll(100).unwrap();
    assert_eq!(engine.state(first), Ok(State::Closed));
    assert_eq!(engine.state(second), Ok(State::Listen));
}

#[test]
fn listener_deadline_uses_monotonic_time_and_expires_at_exact_boundary() {
    let (mut rx, mut tx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 64], [0; 64]);
    let mut storage = [SocketStorage::EMPTY];
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, 10).unwrap();
    let socket = engine.add_client(&mut srx, &mut stx).unwrap();
    engine.listen(socket, 8080, 100).unwrap();
    assert_eq!(engine.poll(9), Err(Error::ClockWentBackwards));
    assert_eq!(engine.poll(u64::MAX), Err(Error::ClockOverflow));
    for _ in 0..1000 {
        engine.poll(10).unwrap();
    }
    assert_eq!(engine.state(socket), Ok(State::Listen));
    engine.poll(109).unwrap();
    assert_eq!(engine.state(socket), Ok(State::Listen));
    engine.poll(110).unwrap();
    assert_eq!(engine.state(socket), Ok(State::Closed));
    engine.listen(socket, 8080, 100).unwrap();
    assert_eq!(engine.state(socket), Ok(State::Listen));
}

#[test]
fn listener_deadline_overflow_does_not_open_a_socket() {
    let (mut rx, mut tx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 64], [0; 64]);
    let mut storage = [SocketStorage::EMPTY];
    let limit = (i64::MAX / 2 / 10_000) as u64;
    let mut engine = Stack::new(CONFIG, &mut storage, &mut rx, &mut tx, limit).unwrap();
    let socket = engine.add_client(&mut srx, &mut stx).unwrap();
    assert_eq!(engine.listen(socket, 8080, 1), Err(Error::ClockOverflow));
    assert_eq!(engine.state(socket), Ok(State::Closed));
}
