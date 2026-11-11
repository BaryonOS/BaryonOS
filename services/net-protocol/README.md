# Bounded Ethernet/IPv4/UDP, ARP and DNS protocols

`nieuw-os-net-protocol` is a standalone Rust workspace with one
allocator-free `no_std` library, `forbid(unsafe_code)` and no dependencies.
It validates received bytes and encodes outgoing frames, but opens no
device, socket, connection or network right and sends nothing.

## Interface and invariant

```rust
use nieuw_os_net_protocol::{parse, Error};

fn payload_length(frame: &[u8]) -> Result<usize, Error> {
    let packet = parse(frame)?;
    Ok(packet.payload().len())
}
```

`parse(&[u8]) -> Result<UdpDatagram<'_>, Error>` returns an immutable view
exclusively after all checks. `frame`, `ethernet_header`,
`ipv4_packet`, `udp_packet`, `payload` and `ethernet_padding` borrow the original
bytes; the parser does not modify or copy the payload. MAC/IPv4 addresses,
ports, TTL, DF and the checksum status are readable via getters. The private
result fields cannot be constructed with unvalidated slices.

The caller must keep the buffer stable and immutable for the duration of the borrow.
A future DMA driver must therefore first actually transfer buffer ownership;
this library manages no DMA, MMIO or device memory.

## Accepted subset

| Layer | Checked behaviour |
|---|---|
| Ethernet II | 14-byte header, EtherType IPv4 (`0x0800`). VLAN C/S tags and other EtherTypes explicitly yield `Unsupported`. |
| IPv4 | Version 4, IHL exactly 5, total_length including the 20-byte header within the available bytes, correct header checksum, reserved flag zero. |
| Fragmentation/options | MF or any fragment offset is rejected as `Unsupported`; DF may be on or off. IHL above 5 is explicitly unsupported, below 5 invalid. |
| Upper-layer protocol | Only protocol 17 (UDP). Other protocols are not interpreted as UDP. |
| UDP | At least eight bytes; the UDP length must be exactly equal to the entire IPv4 payload. A present checksum field is checked with the IPv4 pseudo-header. |
| UDP checksum zero | `UdpChecksum::Absent`, never `Valid`. A correctly computed zero value is recognized and checked as `0xffff` on the wire. |

The input begins at the Ethernet destination MAC and contains **no FCS**.
The caller may have stripped Ethernet padding. If padding is present,
this first profile accepts only the complete Ethernet minimum length
of 60 bytes without FCS. Other trailers or partial padding are
rejected. The padding bytes remain separately visible and fall outside
the IP and UDP length/checksum. The parser treats their content as opaque;
they are never returned as UDP payload.

All checksum words are big-endian. The implementation follows the standard
Internet checksum with end-around carry and a zero pad after an odd last
octet. Folding after every word keeps the accumulator bounded. The IPv4 length
bounds the read/checksum work to 65.535 bytes; the extra Ethernet offset
is checked for overflow. The tests at this maximum length are
synthetic parser boundary checks, not a claim that the later Ethernet driver
supports such frames or a particular jumbo MTU.

`Error` distinguishes truncated Ethernet/IP/UDP, unsupported features,
invalid IP header/total length/flags/checksum, invalid or mismatching
UDP length/checksum, unexpected Ethernet length and arithmetic overflow.
The library repairs no checksum, length, fragment or payload.

## Outgoing packets (N01 preparation)

Alongside the existing parser, the library can now use `build_udp_ipv4(tx, payload, output)`
to encode an IPv4/UDP datagram into a caller-managed buffer.
`UdpIpv4Tx` contains MAC and IP addresses, ports, TTL, identification and
explicit effective IPv4 MTU. The encoder places IPv4 and UDP checksums
(including the RFC 768 zero-as-`0xffff` rule), sets DF, writes exactly the
60-byte Ethernet minimum length where needed, and returns the length to send.

**No error whatsoever writes partial data:** a buffer that is too small, a
payload that is too large, TTL zero or exceeded MTU is rejected in advance.
A successful call leaves everything after the returned length untouched.
The network service must itself enforce MTU, source addresses, destination rights, route and
neighbouring MAC identity. This function does no ARP,
fragmentation, transmission, device management, IP/TCP stack or privilege check.

Fixed, independently derived existing RX vectors are now also used as a TX oracle.
Negative tests cover MTU, output too small, payload overrun,
zero TTL, byte preservation on error, maximum IPv4 size and checksum corruption.
The tests are host tests; only a separate kernel/NIC/QEMU gate can substantiate the
native network requirements of N01. The progress meter remains
unchanged.

## Authority and functions still missing

The Internet checksum is error detection, **not authentication or cryptography**.
Packet addresses and ports provide no reliable sender identity and
grant no capability. Not even a `Valid` checksum may determine a socket, app,
NAS or destination right. The zero-checksum status must remain visible
for a future explicit service choice.

Driver, MTU/FCS/offload policy, receive buffers, route/subnet policy,
socket/process rights, DHCP client states, native DNS/ARP integration, TLS
and real connections remain open.
This limited parser is not a complete IPv4 host implementation and is not yet
called by a native network service. No new
network, privacy, hardware or roadmap criterion has been accepted.

## FR-N01: ARP and DNS as real protocol building blocks

Task contract: build on top of the existing codec a dependency-free, allocator-free
ARP/DNS foundation for the future native network service. Only this standalone
workspace changes; no kernel ABI, new authority or root lockfile. The
main agent owns NIC/kernel integration. There are no host network operations or
simulated connection successes. This does **not** close N01/N02/N03.

| Interface | Behaviour and boundary |
|---|---|
| `arp::parse` | Ethernet/IPv4 request/reply, source MAC bound to Ethernet header; reply target equal in both headers; 42 bytes without padding or a full 60-byte frame, no FCS. |
| `arp::build_request/build_reply` | Writes a real RFC 826 frame with zero padding; all errors before writing; reply only for the configured local interface. |
| `arp::NeighborSlot` | One explicitly started request; only a matching reply before the monotonic deadline fills the slot. Cancellation, exclusive expiry time and checked clock overflow. |
| `dns::Name/Question/build_query` | Canonical ASCII hostname, IN/A or IN/AAAA, one question and RD; at most 255 wire bytes. Unicode/IDNA conversion remains a separate input layer. |
| `dns::parse_response` | Exact transaction ID, question name/type/class, QR/opcode/flags, error code and all declared record lengths. TC is an explicit error, not a partial success. |
| `dns::Response::resolve` | Only IN answers for the requested name or its checked CNAME chain; additional glue and foreign names yield no address. CNAME cycles, contradictory alias/address answers and oversized sets are rejected. |
| `dns::PendingQuery` | Binds the request to both IPv4 address/port pairs, checksum and deadline. Accepted answers consume the request; foreign/malformed answers do not. Cancelling and expiry yield no answer. |

DNS processes at most 4.096 message bytes, 32 records across all sections,
32 backward compression pointers per name, eight alias steps and eight unique
addresses. Pointers to the header, to themselves, forward or outside the message,
label/pointer cycles and names above 255 wire bytes are rejected. A CNAME
must not read beyond its RDLENGTH. TTLs with the high bit set count
as zero per RFC 2181; the cache duration is at most 86.400 seconds and the
minimum of relevant alias/address TTLs. A zero TTL must not be cached.
No OPT/EDNS is requested; an unsolicited OPT record is explicitly
rejected, so that an extended error code does not count as ordinary successful DNS.

These limits are an explicit initial codec profile, not lowered
release requirements. DNS-over-TCP after TC, EDNS, DNSSEC validation, full resolver
cache and IPv6 transport remain visibly open. Decoding an AAAA record
does not prove a working IPv6 stack.

ARP processes probes with source IP zero as recognizable requests, but does not yet perform
RFC 5227 address conflict detection and does not answer them via `build_reply`.
The resolver stores no unsolicited or gratuitous ARP replies. ARP is
**not authenticated**: an attacker on the same link segment can forge a matching
solicited reply. No MAC/IP pair grants identity, TLS trust
or app rights. The network service must enforce interface, netmask, route and broadcast
policy; DNS likewise cannot grant destination capabilities.

Acceptance for this building block: independent fixed request/reply/DNS wire
fixtures, truncation at every offset, invalid headers/identities, wrong
targets, expiry, cancellation, overflow, DNS pointer and alias attacks,
RDLENGTH limits, TTL and endpoint/ID rejection; in addition Clippy, fmt and real
freestanding target compilation. Execution and hashes:
[`verification/fr-n01-report.json`](verification/fr-n01-report.json).
These are host tests and a target build; **no NIC, VM or internet evidence**.

Next native coupling: modern `virtio-net-pci` with pinned RX/TX
queues, explicit buffer ownership and rights; an isolated network service
uses these codecs with configured interface and monotonic time. The kernel
may only mediate bounded I/O/capability boundaries. An own test peer can
answer ARP and deliver DNS errors on a delimited VM network. Only real
frames through the Nieuw OS client count for native network acceptance.

Specification basis: RFC 826 (ARP fields and opcodes), RFC 1035 §§3.1, 4.1 and
4.1.4 (DNS questions, records and compression), RFC 2181 §8 (TTL). Local
knowledge orientation: OSDev *Address Resolution Protocol*, W002539, revision 11170
(5 februari 2011). No tutorial code taken over; no dependency added.

## DHCPv4 — packet codec for future configuration service

`dhcp` adds fixed, allocator-free BOOTP/DHCP messages. `build_discover`,
`build_selecting_request` and `build_renewal_request` encode respectively
DISCOVER, the chosen OFFER request and an existing lease request. The
caller supplies MAC, transaction ID and elapsed seconds; this crate
generates no entropy, sends no frame and manages no timer.
All three builders return at least 300 UDP payload bytes, with
zero fill after END, for the BOOTP consistency check from RFC 1542 §2.1.
A buffer smaller than the full message length yields `OutputTooSmall`
without modification; bytes beyond the returned length remain intact.

`parse_reply` bounds the received BOOTP/DHCP datagram length to 576 bytes and
checks BOOTREPLY, Ethernet type/length, transaction ID, client MAC,
cookie, option boundaries, end/padding, message type, server ID and lease fields.
The first profile subset reads subnet mask, router(s), DNS, lease duration and
optional T1/T2; lists are bounded to four addresses. Invalid masks,
addresses, lease timers, duplicate relevant options and DHCP option overload
are rejected. RFC 3396 concatenation and option overload are not yet
supported; this can limit compatibility with valid extended server answers
and is not a complete DHCP client.

The parser does not check the UDP/IP source or destination endpoint; the caller must
do that before use. A matching transaction ID or DHCP server ID is no
server authentication: DHCP cannot securely validate route/DNS configuration
against a malicious link peer. The parser activates no lease, route or
DNS server. Discovery, selection, ARP address conflict checking, DHCPDECLINE/NAK
recovery, lease persistence, T1 renewal, T2 rebinding, link transition and
reconnect are not yet implemented. The kernel ABI and capability boundary
do not change; DHCP gets no independent authority.

This is a host/freestanding codec building block for FR-N01, not an N01 sign-off
or Internet evidence. Only a native test with a protocol-conformant DHCP peer,
real virtio-net frames, applied configuration, lease renewal/expiry,
wrong server answers and cold/hot link recovery can substantiate network
progress.

## Execution and evidence

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 check --manifest-path services/net-protocol/Cargo.toml --tests --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

The fixed synthetic packet vectors are not generated by the library under
test. Their separate Python derivation, RFC checksum reference,
exact execution and source/artifact hashes are in
`docs/verification/parallel-b11/`. Host tests and compilation for the native
target remain separately labelled; no VM or network is used.

## Primary sources

Accessed 28 October 2026:

- J. Postel / ISI, *User Datagram Protocol*, RFC 768, 28 augustus 1980,
  Fields: <https://www.rfc-editor.org/rfc/rfc768.html>.
- DARPA / ISI, *Internet Protocol*, RFC 791, september 1981, §3.1:
  <https://www.rfc-editor.org/rfc/rfc791.html>.
- R. Braden, D. Borman en C. Partridge, *Computing the Internet Checksum*,
  RFC 1071, september 1988, §§1 en 3:
  <https://www.rfc-editor.org/rfc/rfc1071.html>. The fixed example sum is
  `0xddf2`; the complement checksum is `0x220d`.
- C. Hornig, *A Standard for the Transmission of IP Datagrams over Ethernet
  Networks*, RFC 894, april 1984, Frame Format:
  <https://www.rfc-editor.org/rfc/rfc894.html>, with verified errata
  570/5141: <https://www.rfc-editor.org/errata_search.php?rfc=894>.
- IANA, *IEEE 802 Numbers*, versie 3 november 2024:
  <https://www.iana.org/assignments/ieee-802-numbers/>; labels of the two
  explicitly rejected VLAN tag types.
- R. Droms, *Dynamic Host Configuration Protocol*, RFC 2131, maart 1997,
  §§3.1, 3.2, 4.1 en 4.4.5: <https://www.rfc-editor.org/rfc/rfc2131.html>.
- S. Alexander en R. Droms, *DHCP Options and BOOTP Vendor Extensions*,
  RFC 2132, maart 1997, §§3.3, 3.9, 3.11 en 9:
  <https://www.rfc-editor.org/rfc/rfc2132.html>.
