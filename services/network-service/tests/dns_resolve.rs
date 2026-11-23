//! Host model: real DHCP/ARP/UDP/DNS bytes, modeled NIC completions, no native evidence.
use nieuw_os_net_config::service::{DriveOutcome, ProbeTiming, RandomSource};
use nieuw_os_net_protocol::{arp, build_udp_ipv4, parse, UdpChecksum, UdpIpv4Tx};
use nieuw_os_net_stack::{LinkConfig, SocketStorage, FRAME_BYTES};
use nieuw_os_network_service::{
    wire::{HeaderFields, Operation, Packet, Status},
    Buffers, Destination, DhcpConfiguration, DnsCache, DnsDriveError, Service, DNS_QUERY_BUDGET,
    DNS_TIMEOUT_TICKS,
};
use std::collections::VecDeque;

const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const SERVER_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];
const DNS_MAC: [u8; 6] = [2, 0, 0, 0, 0, 53];
const ADDRESS: [u8; 4] = [192, 0, 2, 42];
const SERVER: [u8; 4] = [192, 0, 2, 1];
const DNS: [u8; 4] = [192, 0, 2, 53];
const ROGUE: [u8; 4] = [192, 0, 2, 54];
const SECOND_DNS: [u8; 4] = [192, 0, 2, 55];
const RESOLVED: [u8; 4] = [203, 0, 113, 7];

fn grant(ipv4: [u8; 4], udp_port: u16) -> Destination {
    Destination {
        ipv4,
        tcp_first: 8080,
        tcp_count: 1,
        udp_port,
    }
}

fn with_service(destinations: [Destination; 2], f: impl FnOnce(&mut Service<'_>)) {
    with_cache(&mut DnsCache::new().unwrap(), destinations, f)
}
fn with_cache(
    dns: &mut DnsCache,
    destinations: [Destination; 2],
    f: impl FnOnce(&mut Service<'_>),
) {
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; FRAME_BYTES], [0; FRAME_BYTES]);
    let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
    let mut service = Service::new_dhcp(
        DhcpConfiguration {
            epoch: 19,
            pids: [11, 12],
            link: LinkConfig {
                mac: MAC,
                random_seed: 7,
            },
            destinations,
        },
        Buffers {
            storage: &mut storage,
            rx_frame: &mut rx,
            tx_frame: &mut tx,
            rx_sockets: [&mut ra, &mut rb],
            tx_sockets: [&mut ta, &mut tb],
        },
        ProbeTiming::new(0, 100, 100).unwrap(),
        dns,
        0,
    )
    .unwrap();
    f(&mut service);
}

struct Entropy(VecDeque<u32>);
impl RandomSource for Entropy {
    type Error = ();
    fn random_u32(&mut self) -> Result<u32, ()> {
        self.0.pop_front().ok_or(())
    }
}
fn dhcp_random(xid: u32) -> Entropy {
    Entropy([901_000, xid, 101_000, 101_000, 101_000, 201_000, 201_000].into())
}
fn dns_random() -> Entropy {
    Entropy(
        (0..256u32)
            .map(|i| 0x9e37_79b9u32.wrapping_mul(i + 1))
            .collect(),
    )
}

fn dhcp_reply(kind: u8, xid: u32, dns: &[[u8; 4]]) -> Vec<u8> {
    let mut message = vec![0; 240];
    message[..3].copy_from_slice(&[2, 1, 6]);
    message[4..8].copy_from_slice(&xid.to_be_bytes());
    message[16..20].copy_from_slice(&ADDRESS);
    message[28..34].copy_from_slice(&MAC);
    message[236..240].copy_from_slice(&[99, 130, 83, 99]);
    message.extend_from_slice(&[53, 1, kind, 54, 4]);
    message.extend_from_slice(&SERVER);
    if kind != 6 {
        message.extend_from_slice(&[51, 4, 0, 0, 0, 80, 1, 4, 255, 255, 255, 0, 3, 4]);
        message.extend_from_slice(&SERVER);
        if !dns.is_empty() {
            message.extend_from_slice(&[6, 4 * dns.len() as u8]);
            message.extend_from_slice(dns.concat().as_slice());
        }
    }
    message.push(255);
    udp(SERVER_MAC, SERVER, [255; 4], 67, 68, &message)
}
fn udp(
    mac: [u8; 6],
    source: [u8; 4],
    destination: [u8; 4],
    source_port: u16,
    destination_port: u16,
    bytes: &[u8],
) -> Vec<u8> {
    let mut frame = vec![0; FRAME_BYTES];
    let n = build_udp_ipv4(
        UdpIpv4Tx {
            source_mac: mac,
            destination_mac: MAC,
            source_ipv4: source,
            destination_ipv4: destination,
            source_port,
            destination_port,
            identification: 1,
            ttl: 64,
            mtu: 1500,
        },
        bytes,
        &mut frame,
    )
    .unwrap();
    frame.truncate(n);
    frame
}
fn complete(s: &mut Service<'_>, now: u64) -> Vec<u8> {
    let tx = s.transmission_at(now).unwrap().unwrap();
    let (id, bytes) = (tx.id, tx.bytes.to_vec());
    s.complete_transmission_at(id, now).unwrap();
    bytes
}
fn acquire(s: &mut Service<'_>, start: u64, xid: u32, dns: bool) -> u64 {
    s.link_up(start).unwrap();
    let mut rng = dhcp_random(xid);
    assert_eq!(
        s.drive_dhcp(start, &mut rng),
        Ok(DriveOutcome::WaitingUntil(start + 100))
    );
    let now = start + 100;
    assert!(matches!(
        s.drive_dhcp(now, &mut rng),
        Ok(DriveOutcome::Published(_))
    ));
    complete(s, now);
    let servers: &[[u8; 4]] = if dns { &[DNS] } else { &[] };
    s.ingress(&dhcp_reply(2, xid, servers), now).unwrap();
    assert!(matches!(
        s.drive_dhcp(now, &mut rng),
        Ok(DriveOutcome::Published(_))
    ));
    complete(s, now);
    s.ingress(&dhcp_reply(5, xid, servers), now).unwrap();
    for tick in [now, now + 100, now + 200] {
        s.poll(tick).unwrap();
        complete(s, tick);
    }
    s.poll(now + 400).unwrap();
    complete(s, now + 400);
    let lease = s.dhcp_configuration(now + 400).unwrap().unwrap();
    assert_eq!(lease.dns_servers(), if dns { &[DNS][..] } else { &[] });
    now + 400
}

#[allow(clippy::too_many_arguments)]
fn call(
    s: &mut Service<'_>,
    slot: usize,
    request: u64,
    op: Operation,
    address: [u8; 4],
    port: u16,
    data: &[u8],
    now: u64,
) -> Packet {
    let packet = Packet::request(
        HeaderFields {
            operation: op,
            request,
            epoch: 19,
            connection: 0,
            argument: if op == Operation::OpenTcp {
                300
            } else {
                data.len() as u32
            },
            address,
            port,
        },
        data,
    )
    .unwrap();
    s.handle(slot, 11 + slot as u64, 0, packet.bytes(), now)
        .unwrap()
        .unwrap()
}
fn resolve(s: &mut Service<'_>, slot: usize, request: u64, name: &str, now: u64) -> Packet {
    call(
        s,
        slot,
        request,
        Operation::Resolve,
        [0; 4],
        0,
        name.as_bytes(),
        now,
    )
}

/// Resolve ARP for the DNS server through a real request/reply exchange.
fn arp_dns(s: &mut Service<'_>, rng: &mut Entropy, now: u64) {
    assert_eq!(s.drive_dns(now, rng), Ok(0));
    s.poll(now).unwrap();
    let request = complete(s, now);
    assert_eq!(&request[38..42], &DNS);
    let mut frame = [0; 60];
    let n = arp::build_reply(
        arp::Interface {
            mac: DNS_MAC,
            ipv4: DNS,
        },
        &arp::parse(&request).unwrap(),
        &mut frame,
    )
    .unwrap();
    s.ingress(&frame[..n], now).unwrap();
}
/// Send the pending query and return (DNS ID, local port, query payload).
fn sent_query(s: &mut Service<'_>, rng: &mut Entropy, now: u64) -> (u16, u16, Vec<u8>) {
    assert_eq!(s.drive_dns(now, rng), Ok(1));
    s.poll(now).unwrap();
    let frame = complete(s, now);
    let packet = parse(&frame).unwrap();
    assert_eq!(packet.destination_address(), DNS);
    assert_eq!(packet.destination_port(), 53);
    assert_eq!(packet.source_address(), ADDRESS);
    let payload = packet.payload().to_vec();
    (
        u16::from_be_bytes([payload[0], payload[1]]),
        packet.source_port(),
        payload,
    )
}

fn header(id: u16, name: &str, flags: u16, answers: u16, authority: u16) -> Vec<u8> {
    let mut bytes = id.to_be_bytes().to_vec();
    bytes.extend_from_slice(&flags.to_be_bytes());
    bytes.extend_from_slice(&[0, 1]);
    bytes.extend_from_slice(&answers.to_be_bytes());
    bytes.extend_from_slice(&authority.to_be_bytes());
    bytes.extend_from_slice(&[0, 0]);
    for label in name.split('.') {
        bytes.push(label.len() as u8);
        bytes.extend_from_slice(label.as_bytes());
    }
    bytes.extend_from_slice(&[0, 0, 1, 0, 1]);
    bytes
}
fn record(bytes: &mut Vec<u8>, owner: &[u8], kind: u16, ttl: u32, data: &[u8]) {
    bytes.extend_from_slice(owner);
    bytes.extend_from_slice(&kind.to_be_bytes());
    bytes.extend_from_slice(&[0, 1]);
    bytes.extend_from_slice(&ttl.to_be_bytes());
    bytes.extend_from_slice(&(data.len() as u16).to_be_bytes());
    bytes.extend_from_slice(data);
}
fn positive(id: u16, name: &str, address: [u8; 4]) -> Vec<u8> {
    let mut bytes = header(id, name, 0x8180, 1, 0);
    record(&mut bytes, &[0xc0, 12], 1, 300, &address);
    bytes
}
fn nxdomain(id: u16, name: &str) -> Vec<u8> {
    let mut bytes = header(id, name, 0x8183, 0, 1);
    let mut soa = vec![0xc0, 12, 0xc0, 12];
    for value in [1u32, 3600, 600, 86400, 60] {
        soa.extend_from_slice(&value.to_be_bytes());
    }
    record(&mut bytes, &[0xc0, 12], 6, 600, &soa);
    bytes
}
fn from_dns(port: u16, bytes: &[u8]) -> Vec<u8> {
    udp(DNS_MAC, DNS, ADDRESS, 53, port, bytes)
}

#[test]
fn lease_server_answer_is_cached_per_slot_and_grants_no_destination() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 0, 1, "Example.org", now).status(), Status::Again);
        // A different name while one query is outstanding is refused, not queued.
        assert_eq!(resolve(s, 0, 2, "other.org", now).status(), Status::Busy);
        arp_dns(s, &mut rng, now);
        let (id, port, query) = sent_query(s, &mut rng, now);
        assert!(port >= 49152 && port % 2 == 0);
        assert_eq!(&query[12..25], b"\x07example\x03org\x00");
        s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
            .unwrap();
        let reply = resolve(s, 0, 3, "example.org", now);
        assert_eq!(reply.status(), Status::Ok);
        assert_eq!(reply.data(), &RESOLVED);
        // Cache hit: no new query and no budget use.
        assert_eq!(resolve(s, 0, 4, "example.org.", now + 1).data(), &RESOLVED);
        assert_eq!(s.drive_dns(now + 1, &mut rng), Ok(0));
        // The other slot has its own cache and must ask its own query.
        assert_eq!(
            resolve(s, 1, 1, "example.org", now + 1).status(),
            Status::Again
        );
        // A resolved address is not a destination grant (no amplification).
        assert_eq!(
            call(s, 0, 5, Operation::OpenTcp, RESOLVED, 8080, &[], now + 1).status(),
            Status::Access
        );
        assert_eq!(
            call(s, 0, 6, Operation::SendUdp, RESOLVED, 53, b"x", now + 1).status(),
            Status::Access
        );
        // A DNS reply never lands in the app's raw UDP buffer.
        assert_eq!(
            call(s, 0, 7, Operation::ReceiveUdp, [0; 4], 0, &[], now + 1).status(),
            Status::Again
        );
    });
}

#[test]
fn malformed_mismatched_and_spoofed_replies_never_poison_the_cache() {
    with_service([grant(DNS, 53), grant(ROGUE, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        let (id, port, _) = sent_query(s, &mut rng, now);
        let forged = positive(id, "example.org", [6, 6, 6, 6]);
        let mut truncated_body = forged.clone();
        truncated_body.truncate(forged.len() - 2);
        let mut trailing = forged.clone();
        trailing.push(0);
        let spoofs = [
            // Non-lease server, wrong source port, wrong local port.
            udp(DNS_MAC, ROGUE, ADDRESS, 53, port, &forged),
            udp(DNS_MAC, DNS, ADDRESS, 5353, port, &forged),
            from_dns(port ^ 2, &forged),
            // ID and question mismatch, malformed bodies.
            from_dns(
                port,
                &positive(id.wrapping_add(1), "example.org", [6, 6, 6, 6]),
            ),
            from_dns(port, &positive(id, "example.com", [6, 6, 6, 6])),
            // Matching ID with TC but another question is no final outcome.
            from_dns(port, &header(id, "example.com", 0x8380, 0, 0)),
            from_dns(port, &truncated_body),
            from_dns(port, &trailing),
            from_dns(port, &[0; 11]),
        ];
        let mut request = 2;
        for frame in &spoofs {
            s.ingress(frame, now).unwrap();
            assert_eq!(
                resolve(s, 0, request, "example.org", now).status(),
                Status::Again
            );
            request += 1;
        }
        // The genuine reply still completes the query with its own data.
        s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
            .unwrap();
        assert_eq!(resolve(s, 0, request, "example.org", now).data(), &RESOLVED);
        // Slot 1 holds UDP/53 only to a non-lease server: no resolution at all.
        assert_eq!(
            resolve(s, 1, 1, "example.org", now).status(),
            Status::Access
        );
    });
}

#[test]
fn tc_servfail_and_refusals_fail_uncached_while_nxdomain_is_negatively_cached() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        let mut request = 1;
        let mut ask = |s: &mut Service<'_>, name: &str| {
            request += 1;
            resolve(s, 0, request, name, now).status()
        };
        assert_eq!(ask(s, "example.org"), Status::Again);
        arp_dns(s, &mut rng, now);
        for (flags, status) in [
            (0x8380, Status::ServerFailure), // TC: no silent TCP fallback.
            (0x8182, Status::ServerFailure), // SERVFAIL.
            (0x8185, Status::ServerFailure), // REFUSED.
        ] {
            let (id, port, _) = sent_query(s, &mut rng, now);
            s.ingress(
                &from_dns(port, &header(id, "example.org", flags, 0, 0)),
                now,
            )
            .unwrap();
            assert_eq!(ask(s, "example.org"), status);
            // Not cached: the next Resolve starts a fresh query.
            assert_eq!(ask(s, "example.org"), Status::Again);
        }
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &nxdomain(id, "example.org")), now)
            .unwrap();
        assert_eq!(ask(s, "example.org"), Status::NotFound);
        // Negative answer served from cache without a new query.
        assert_eq!(ask(s, "example.org"), Status::NotFound);
        assert_eq!(s.drive_dns(now, &mut rng), Ok(0));
        // NOERROR without an address is NotFound but never cached.
        assert_eq!(ask(s, "empty.org"), Status::Again);
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &header(id, "empty.org", 0x8180, 0, 0)), now)
            .unwrap();
        assert_eq!(ask(s, "empty.org"), Status::NotFound);
        assert_eq!(ask(s, "empty.org"), Status::Again);
    });
}

#[test]
fn no_dns_option_no_grant_and_oversize_names_are_refused() {
    // DHCP without option 6: no default or fallback resolver.
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        assert_eq!(resolve(s, 0, 1, "example.org", 0).status(), Status::Again);
        let now = acquire(s, 0, 0x1234, false);
        assert_eq!(
            resolve(s, 0, 2, "example.org", now).status(),
            Status::Access
        );
        let mut rng = dns_random();
        assert_eq!(s.drive_dns(now, &mut rng), Ok(0));
        assert!(s.transmission_at(now).unwrap().is_none());
    });
    // Grant to the lease server but not for UDP/53.
    with_service([grant(DNS, 9000), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        assert_eq!(
            resolve(s, 0, 1, "example.org", now).status(),
            Status::Access
        );
        // Wrong PID for the slot is silently dropped, as for every operation.
        let packet = Packet::request(
            HeaderFields {
                operation: Operation::Resolve,
                request: 2,
                epoch: 19,
                connection: 0,
                argument: 11,
                address: [0; 4],
                port: 0,
            },
            b"example.org",
        )
        .unwrap();
        assert!(s.handle(1, 11, 0, packet.bytes(), now).unwrap().is_none());
        // Oversize and malformed names are refused before any query state.
        let mut bytes = *packet.bytes();
        bytes[36..40].copy_from_slice(&209u32.to_le_bytes());
        bytes[48..48 + 208].fill(b'a');
        let refusal = s.handle(1, 12, 0, &bytes, now).unwrap().unwrap();
        assert_eq!(refusal.status(), Status::Malformed);
        let mut bytes = *packet.bytes();
        bytes[48 + 7] = b'_';
        bytes[8] = 3;
        let refusal = s.handle(1, 12, 0, &bytes, now).unwrap().unwrap();
        assert_eq!(refusal.status(), Status::Malformed);
        let mut rng = dns_random();
        assert_eq!(s.drive_dns(now, &mut rng), Ok(0));
    });
}

#[test]
fn lease_change_flushes_cache_and_drops_stale_pending_replies() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
            .unwrap();
        assert_eq!(resolve(s, 0, 2, "example.org", now).data(), &RESOLVED);
        // A second query is in flight when the lease is lost.
        assert_eq!(resolve(s, 0, 3, "stale.org", now).status(), Status::Again);
        let (stale_id, stale_port, _) = sent_query(s, &mut rng, now);
        s.link_down(now + 1).unwrap();
        assert_eq!(
            resolve(s, 0, 4, "example.org", now + 1).status(),
            Status::Again
        );
        let ready = acquire(s, now + 1, 0xabcd, true);
        // The old reply arrives after re-acquisition: it binds to nothing.
        s.ingress(
            &from_dns(stale_port, &positive(stale_id, "stale.org", [6, 6, 6, 6])),
            ready,
        )
        .unwrap();
        // The new epoch has no cached answer: example.org needs a new query,
        // and the stale reply above did not complete or cache stale.org.
        assert_eq!(
            resolve(s, 0, 5, "example.org", ready).status(),
            Status::Again
        );
        assert_eq!(resolve(s, 0, 6, "stale.org", ready).status(), Status::Busy);
        assert_eq!(resolve(s, 1, 1, "stale.org", ready).status(), Status::Again);
    });
}

#[test]
fn per_slot_budget_timeout_entropy_and_revocation_are_bounded() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        // Unsent and unanswered queries time out.
        assert_eq!(resolve(s, 0, 1, "late.org", now).status(), Status::Again);
        assert_eq!(
            resolve(s, 0, 2, "late.org", now + DNS_TIMEOUT_TICKS).status(),
            Status::Timeout
        );
        let mut request = 3;
        let mut now = now + DNS_TIMEOUT_TICKS;
        assert_eq!(resolve(s, 0, request, "e.org", now).status(), Status::Again);
        request += 1;
        arp_dns(s, &mut rng, now);
        // No entropy, no query.
        assert_eq!(
            s.drive_dns(now, &mut Entropy(VecDeque::new())),
            Err(DnsDriveError::Random(()))
        );
        assert!(s.transmission_at(now).unwrap().is_none());
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &header(id, "e.org", 0x8182, 0, 0)), now)
            .unwrap();
        assert_eq!(
            resolve(s, 0, request, "e.org", now).status(),
            Status::ServerFailure
        );
        request += 1;
        // Budget: late.org and e.org already used two of slot 0's queries.
        for i in 2..DNS_QUERY_BUDGET {
            let name = format!("n{i}.org");
            assert_eq!(resolve(s, 0, request, &name, now).status(), Status::Again);
            let (id, port, _) = sent_query(s, &mut rng, now);
            s.ingress(&from_dns(port, &header(id, &name, 0x8182, 0, 0)), now)
                .unwrap();
            assert_eq!(
                resolve(s, 0, request + 1, &name, now).status(),
                Status::ServerFailure
            );
            request += 2;
            now += 1;
        }
        assert_eq!(
            resolve(s, 0, request, "x.org", now).status(),
            Status::Exhausted
        );
        // Slot 0's exhaustion does not consume slot 1's budget.
        assert_eq!(resolve(s, 1, 1, "x.org", now).status(), Status::Again);
        // Revocation drops slot 1's pending query and blocks Resolve.
        s.revoke(1, now).unwrap();
        assert_eq!(resolve(s, 1, 2, "x.org", now).status(), Status::Access);
        assert_eq!(s.drive_dns(now, &mut rng), Ok(0));
    });
}

#[test]
fn queued_query_of_a_revoked_slot_is_discarded_unpublished() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 1, 1, "example.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        assert_eq!(s.drive_dns(now, &mut rng), Ok(1));
        s.poll(now).unwrap();
        s.revoke(1, now).unwrap();
        assert_eq!(s.discard_revoked_unpublished(), Ok(true));
        assert_eq!(s.counters().revoked_frames_discarded, 1);
    });
}

#[test]
fn static_configuration_has_no_resolver() {
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; FRAME_BYTES], [0; FRAME_BYTES]);
    let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
    let mut s = Service::new(
        nieuw_os_network_service::Configuration {
            epoch: 19,
            pids: [11, 12],
            link: nieuw_os_net_stack::Config {
                mac: MAC,
                ipv4: ADDRESS,
                prefix_len: 24,
                random_seed: 7,
            },
            destinations: [grant(DNS, 53), grant(DNS, 53)],
        },
        Buffers {
            storage: &mut storage,
            rx_frame: &mut rx,
            tx_frame: &mut tx,
            rx_sockets: [&mut ra, &mut rb],
            tx_sockets: [&mut ta, &mut tb],
        },
        0,
    )
    .unwrap();
    // A UDP/53 grant alone never makes a server a resolver without a lease.
    assert_eq!(
        resolve(&mut s, 0, 1, "example.org", 0).status(),
        Status::Access
    );
    assert_eq!(s.drive_dns(0, &mut dns_random()), Ok(0));
}

/// Mirrors the native service static: a runtime call
/// would return ~22 KiB by value on its 64 KiB stack. Fails to compile if
/// `DnsCache::new` or `Resolver::new` stops being `const`.
static STATIC_CACHE: Option<DnsCache> = match DnsCache::new() {
    Ok(cache) => Some(cache),
    Err(_) => None,
};

#[test]
fn dns_cache_constructs_in_const_context() {
    assert!(STATIC_CACHE.is_some());
}

/// End-to-end: no answer crosses into a new service on reused storage. Every
/// Resolve needs a lease and the first lease always bumps the cache epoch, so
/// this observes that lease-acquisition flush; the extra flush in `new_inner`
/// is defence in depth with no path observable through the public API.
#[test]
fn reused_cache_storage_serves_no_answer_to_a_new_service() {
    let mut dns = DnsCache::new().unwrap();
    for round in 0..2 {
        with_cache(&mut dns, [grant(DNS, 53), grant(DNS, 53)], |s| {
            let now = acquire(s, 0, 0x1234, true);
            let mut rng = dns_random();
            // The second service must not see the first service's answer.
            assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
            if round == 0 {
                arp_dns(s, &mut rng, now);
                let (id, port, _) = sent_query(s, &mut rng, now);
                s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
                    .unwrap();
                assert_eq!(resolve(s, 0, 2, "example.org", now).data(), &RESOLVED);
            }
        });
    }
}

#[test]
fn renewal_dns_server_change_and_nak_flush_the_cache() {
    for nak in [false, true] {
        with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
            let now = acquire(s, 0, 0x1234, true);
            let mut rng = dns_random();
            assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
            arp_dns(s, &mut rng, now);
            let (id, port, _) = sent_query(s, &mut rng, now);
            s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
                .unwrap();
            assert_eq!(resolve(s, 0, 2, "example.org", now).data(), &RESOLVED);
            // Real renewal REQUEST at T1; same address, router and grant.
            let renew = 4100;
            let mut dhcp = Entropy([0x5678, 101_000].into());
            assert!(matches!(
                s.drive_dhcp(renew, &mut dhcp),
                Ok(DriveOutcome::Published(_))
            ));
            complete(s, renew);
            if nak {
                s.ingress(&dhcp_reply(6, 0x5678, &[]), renew).unwrap();
                assert!(s.dhcp_configuration(renew).unwrap().is_none());
                assert_eq!(
                    resolve(s, 0, 3, "example.org", renew).status(),
                    Status::Again
                );
                let ready = acquire(s, renew, 0xabcd, true);
                assert_eq!(
                    resolve(s, 0, 4, "example.org", ready).status(),
                    Status::Again
                );
            } else {
                // Only the DNS server list changes; it still names our server.
                s.ingress(&dhcp_reply(5, 0x5678, &[SECOND_DNS, DNS]), renew)
                    .unwrap();
                let lease = s.dhcp_configuration(renew).unwrap().unwrap();
                assert_eq!(lease.address, ADDRESS);
                assert_eq!(lease.dns_servers(), &[SECOND_DNS, DNS]);
                assert_eq!(
                    resolve(s, 0, 3, "example.org", renew).status(),
                    Status::Again
                );
            }
        });
    }
}

#[test]
fn abandoned_query_does_not_block_cache_hits_or_later_names() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
            .unwrap();
        assert_eq!(resolve(s, 0, 2, "example.org", now).data(), &RESOLVED);
        // Ask once for another name, then never come back for it.
        assert_eq!(resolve(s, 0, 3, "a.org", now).status(), Status::Again);
        // Cached names are still served while that query is live.
        assert_eq!(resolve(s, 0, 4, "example.org", now).data(), &RESOLVED);
        assert_eq!(resolve(s, 0, 5, "b.org", now).status(), Status::Busy);
        // After the deadline the abandoned query no longer blocks the slot.
        let later = now + DNS_TIMEOUT_TICKS;
        assert_eq!(resolve(s, 0, 6, "b.org", later).status(), Status::Again);
        assert_eq!(resolve(s, 0, 7, "a.org", later).status(), Status::Busy);
    });
}

#[test]
fn reply_without_udp_checksum_is_dropped_and_not_cached() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        let (id, port, _) = sent_query(s, &mut rng, now);
        // Otherwise exact reply, but the UDP checksum field (frame[40..42]) is 0.
        let mut frame = from_dns(port, &positive(id, "example.org", [6, 6, 6, 6]));
        frame[40..42].fill(0);
        assert_eq!(parse(&frame).unwrap().checksum(), UdpChecksum::Absent);
        let malformed = s.counters().malformed_frames;
        s.ingress(&frame, now).unwrap();
        assert_eq!(s.counters().malformed_frames, malformed + 1);
        assert_eq!(resolve(s, 0, 2, "example.org", now).status(), Status::Again);
        // The checksummed reply still completes; the dropped one was never cached.
        s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), now)
            .unwrap();
        assert_eq!(resolve(s, 0, 3, "example.org", now).data(), &RESOLVED);
        assert_eq!(resolve(s, 0, 4, "example.org", now).data(), &RESOLVED);
    });
}

#[test]
fn reply_at_or_after_the_deadline_is_dropped_and_not_cached() {
    for late in [0, 1] {
        with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
            let now = acquire(s, 0, 0x1234, true);
            let mut rng = dns_random();
            assert_eq!(resolve(s, 0, 1, "example.org", now).status(), Status::Again);
            arp_dns(s, &mut rng, now);
            let (id, port, _) = sent_query(s, &mut rng, now);
            let at = now + DNS_TIMEOUT_TICKS + late;
            let malformed = s.counters().malformed_frames;
            s.ingress(&from_dns(port, &positive(id, "example.org", RESOLVED)), at)
                .unwrap();
            assert_eq!(s.counters().malformed_frames, malformed + 1, "late {late}");
            assert_eq!(
                resolve(s, 0, 2, "example.org", at).status(),
                Status::Timeout,
                "late {late}"
            );
            // Not cached: the next Resolve starts a fresh query.
            assert_eq!(
                resolve(s, 0, 3, "example.org", at).status(),
                Status::Again,
                "late {late}"
            );
        });
    }
}

/// One fresh draw per query actually queued: none while ARP for the server
/// is pending or while an earlier frame still holds the shared TX slot.
#[test]
fn entropy_is_drawn_only_for_a_query_that_can_be_queued() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut none = Entropy(VecDeque::new());
        assert_eq!(resolve(s, 0, 1, "a.org", now).status(), Status::Again);
        // ARP pending: `arp_dns` requires Ok(0), so no draw was attempted.
        arp_dns(s, &mut none, now);
        let mut one = Entropy([0x1234_5678].into());
        assert_eq!(s.drive_dns(now, &mut one), Ok(1));
        assert!(one.0.is_empty());
        // Answer it while the query frame still holds the TX slot.
        s.poll(now).unwrap();
        let frame = s.transmission_at(now).unwrap().unwrap().bytes.to_vec();
        let packet = parse(&frame).unwrap();
        let (port, id) = (packet.source_port(), 0x5678);
        s.ingress(&from_dns(port, &positive(id, "a.org", RESOLVED)), now)
            .unwrap();
        assert_eq!(resolve(s, 0, 2, "a.org", now).data(), &RESOLVED);
        assert_eq!(resolve(s, 0, 3, "b.org", now).status(), Status::Again);
        // TX busy: no draw and no error, the query just waits.
        assert_eq!(s.drive_dns(now, &mut none), Ok(0));
        complete(s, now);
        let mut one = Entropy([0x9abc_def0].into());
        assert_eq!(s.drive_dns(now, &mut one), Ok(1));
        assert!(one.0.is_empty());
    });
}

/// A stored outcome answers only its own lookup until the query deadline: an
/// abandoned one is never handed to a later Resolve, and a zero-TTL answer
/// reaches the lookup that asked for it once and is never reused.
#[test]
fn stored_outcome_is_bounded_by_the_query_deadline() {
    with_service([grant(DNS, 53), grant(DNS, 53)], |s| {
        let now = acquire(s, 0, 0x1234, true);
        let mut rng = dns_random();
        let zero_ttl = |id: u16| {
            let mut bytes = header(id, "a.org", 0x8180, 1, 0);
            record(&mut bytes, &[0xc0, 12], 1, 0, &RESOLVED);
            bytes
        };
        // Uncached SERVFAIL, then the app never asks again in time.
        assert_eq!(resolve(s, 0, 1, "a.org", now).status(), Status::Again);
        arp_dns(s, &mut rng, now);
        let (id, port, _) = sent_query(s, &mut rng, now);
        s.ingress(&from_dns(port, &header(id, "a.org", 0x8182, 0, 0)), now)
            .unwrap();
        let later = now + DNS_TIMEOUT_TICKS;
        assert_eq!(resolve(s, 0, 2, "a.org", later).status(), Status::Again);
        // Zero TTL: delivered to its own lookup inside the window, then gone.
        let (id, port, _) = sent_query(s, &mut rng, later);
        s.ingress(&from_dns(port, &zero_ttl(id)), later).unwrap();
        assert_eq!(resolve(s, 0, 3, "a.org", later + 1).data(), &RESOLVED);
        assert_eq!(resolve(s, 0, 4, "a.org", later + 1).status(), Status::Again);
        // An abandoned zero-TTL answer is not served after the deadline.
        let (id, port, _) = sent_query(s, &mut rng, later + 1);
        s.ingress(&from_dns(port, &zero_ttl(id)), later + 1)
            .unwrap();
        let expired = later + 1 + DNS_TIMEOUT_TICKS;
        assert_eq!(resolve(s, 0, 5, "a.org", expired).status(), Status::Again);
    });
}
