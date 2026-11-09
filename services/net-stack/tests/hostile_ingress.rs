//! Hostile-ingress regressions found by host fuzzing (VER-NET-1, 2026-11-05).
//! Hand-built frames through the public `Stack` API only; host-model evidence,
//! not native NIC or real-network proof.
use nieuw_os_net_stack::{Config, SocketStorage, Stack, State};

const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const IP: [u8; 4] = [192, 0, 2, 2];
const PEER_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];
const PEER_IP: [u8; 4] = [192, 0, 2, 1];
const BROADCAST_MAC: [u8; 6] = [0xff; 6];
const TCP: u8 = 6;
const UDP: u8 = 17;
const SYN: u8 = 0x02;
const ACK: u8 = 0x10;
const PSH_ACK: u8 = 0x18;
const SYN_ACK: u8 = 0x12;

fn csum(data: &[u8]) -> u16 {
    let mut sum: u64 = data
        .chunks(2)
        .map(|p| (u64::from(p[0]) << 8) | u64::from(*p.get(1).unwrap_or(&0)))
        .sum();
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Ethernet + IPv4 frame from the peer. TCP gets a valid pseudo-header
/// checksum; UDP uses the IPv4 "no checksum" value 0.
fn frame(dst_mac: [u8; 6], dst_ip: [u8; 4], proto: u8, mut l4: Vec<u8>) -> Vec<u8> {
    if proto == TCP {
        let mut pseudo = PEER_IP.to_vec();
        pseudo.extend_from_slice(&dst_ip);
        pseudo.extend_from_slice(&[0, TCP, (l4.len() >> 8) as u8, l4.len() as u8]);
        pseudo.extend_from_slice(&l4);
        let c = csum(&pseudo);
        l4[16..18].copy_from_slice(&c.to_be_bytes());
    }
    let total = 20 + l4.len();
    let mut ip = vec![
        0x45,
        0,
        (total >> 8) as u8,
        total as u8,
        0,
        0,
        0x40,
        0,
        64,
        proto,
        0,
        0,
    ];
    ip.extend_from_slice(&PEER_IP);
    ip.extend_from_slice(&dst_ip);
    let c = csum(&ip);
    ip[10..12].copy_from_slice(&c.to_be_bytes());
    let mut f = dst_mac.to_vec();
    f.extend_from_slice(&PEER_MAC);
    f.extend_from_slice(&[8, 0]);
    f.extend_from_slice(&ip);
    f.extend_from_slice(&l4);
    f
}

fn tcp(seq: u32, ack: u32, flags: u8, payload: &[u8]) -> Vec<u8> {
    let mut t = vec![0u8; 20];
    t[0..2].copy_from_slice(&8080u16.to_be_bytes());
    t[2..4].copy_from_slice(&40000u16.to_be_bytes());
    t[4..8].copy_from_slice(&seq.to_be_bytes());
    t[8..12].copy_from_slice(&ack.to_be_bytes());
    t[12] = 0x50;
    t[13] = flags;
    t[14..16].copy_from_slice(&4096u16.to_be_bytes());
    t.extend_from_slice(payload);
    t
}

fn udp_to_closed_port() -> Vec<u8> {
    let mut u = vec![0u8; 8];
    u[0..2].copy_from_slice(&5353u16.to_be_bytes());
    u[2..4].copy_from_slice(&9u16.to_be_bytes());
    u[4..6].copy_from_slice(&8u16.to_be_bytes());
    u
}

fn arp_reply() -> Vec<u8> {
    let mut f = vec![0u8; 42];
    f[..6].copy_from_slice(&MAC);
    f[6..12].copy_from_slice(&PEER_MAC);
    f[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 2]);
    f[22..28].copy_from_slice(&PEER_MAC);
    f[28..32].copy_from_slice(&PEER_IP);
    f[32..38].copy_from_slice(&MAC);
    f[38..42].copy_from_slice(&IP);
    f
}

/// Poll once and complete whatever the stack offered.
fn drain(stack: &mut Stack, ticks: u64) -> Option<Vec<u8>> {
    stack.poll(ticks).unwrap();
    let out = stack.transmission().map(|t| (t.id, t.bytes.to_vec()));
    out.map(|(id, bytes)| {
        stack.complete_transmission(id).unwrap();
        bytes
    })
}

/// Runs `body` on a stack whose client has resolved the peer and sent its SYN.
/// Passes the client's initial sequence number to `body`.
fn with_syn_sent(body: impl FnOnce(&mut Stack, nieuw_os_net_stack::Client, u32)) {
    let mut storage = [SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let (mut rxb, mut txb) = ([0; 4096], [0; 4096]);
    let config = Config {
        mac: MAC,
        ipv4: IP,
        prefix_len: 24,
        random_seed: 7,
    };
    let mut stack = Stack::new(config, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let client = stack.add_client(&mut rxb, &mut txb).unwrap();
    stack.connect(client, PEER_IP, 8080, 40000, 300).unwrap();
    let arp = drain(&mut stack, 0).expect("ARP request");
    assert_eq!(&arp[12..14], &[8, 6]);
    stack.push_frame(&arp_reply()).unwrap();
    let syn = drain(&mut stack, 1).expect("SYN");
    assert_eq!(syn[47] & SYN_ACK, SYN);
    let isn = u32::from_be_bytes(syn[38..42].try_into().unwrap());
    body(&mut stack, client, isn);
}

#[test]
fn unicast_syn_ack_to_own_address_still_establishes() {
    with_syn_sent(|stack, client, isn| {
        let syn_ack = tcp(0x1000_0000, isn.wrapping_add(1), SYN_ACK, &[]);
        stack.push_frame(&frame(MAC, IP, TCP, syn_ack)).unwrap();
        assert!(drain(stack, 2).is_some(), "ACK of SYN-ACK");
        assert_eq!(stack.state(client).unwrap(), State::Established);
    });
}

/// smoltcp 0.14.0 answered these with a RST whose *source* is the broadcast or
/// foreign destination: reflection from an address this host does not own.
#[test]
fn tcp_to_an_address_we_do_not_own_gets_no_reply() {
    for dst_ip in [[192, 0, 2, 255], [255, 255, 255, 255], [192, 0, 2, 99]] {
        for dst_mac in [MAC, BROADCAST_MAC] {
            with_syn_sent(|stack, _, isn| {
                for flags in [SYN, ACK] {
                    let segment = tcp(0x1000_0000, isn.wrapping_add(1), flags, &[]);
                    stack
                        .push_frame(&frame(dst_mac, dst_ip, TCP, segment))
                        .unwrap();
                    assert_eq!(
                        drain(stack, 2),
                        None,
                        "reply to TCP flags {flags:#x} for {dst_ip:?} via {dst_mac:x?}"
                    );
                }
            });
        }
    }
}

/// RFC 1122 3.2.2: no ICMP error for a datagram sent as link-layer broadcast.
#[test]
fn link_broadcast_udp_to_own_address_gets_no_icmp() {
    with_syn_sent(|stack, _, _| {
        stack
            .push_frame(&frame(BROADCAST_MAC, IP, UDP, udp_to_closed_port()))
            .unwrap();
        assert_eq!(drain(stack, 2), None);
    });
}

/// KNOWN DEFECT, not desired behaviour. Upstream smoltcp 0.14.0 (also current
/// upstream main) sets `remote_last_ack = Some(seq)` on SYN-ACK while
/// `remote_last_win` is still 0, so until the first ACK is sent the receive
/// window is inverted (end = start - 1). A segment about 2^31 ahead then
/// passes the wrapping in-window test and `SeqNumber::sub` panics
/// unconditionally (release too; this crate aborts on panic). Here the ACK
/// cannot leave because the peer is not ARP-resolved.
/// When this test starts failing, the defect is fixed: invert it into a
/// no-panic regression and drop this note.
#[test]
#[should_panic(expected = "attempt to subtract sequence numbers with underflow")]
fn known_defect_half_space_segment_before_first_ack_panics_smoltcp() {
    let mut storage = [SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let (mut rxb, mut txb) = ([0; 4096], [0; 4096]);
    let config = Config {
        mac: MAC,
        ipv4: IP,
        prefix_len: 24,
        random_seed: 7,
    };
    let mut stack = Stack::new(config, &mut storage, &mut rx, &mut tx, 0).unwrap();
    let client = stack.add_client(&mut rxb, &mut txb).unwrap();
    stack.connect(client, PEER_IP, 8080, 40000, 300).unwrap();
    let _arp_request = drain(&mut stack, 0);
    // No ARP reply: SYN and the later ACK stay unsent. ISN is the value
    // smoltcp derives from random_seed 7 (asserted by the state check below).
    let isn = 0x8824_eb91u32;
    let peer_isn = 0x1000_0000u32;
    let syn_ack = tcp(peer_isn, isn.wrapping_add(1), SYN_ACK, &[]);
    stack.push_frame(&frame(MAC, IP, TCP, syn_ack)).unwrap();
    let _ = drain(&mut stack, 1);
    assert_eq!(stack.state(client).unwrap(), State::Established);
    let far = tcp(
        peer_isn.wrapping_add(1).wrapping_add(0x8000_0000),
        isn.wrapping_add(1),
        PSH_ACK,
        b"x",
    );
    stack.push_frame(&frame(MAC, IP, TCP, far)).unwrap();
    let _ = drain(&mut stack, 2);
}
