#![no_std]
#![forbid(unsafe_code)]
//! DHCP/configuration service core, not a native caller, device lease or grant.

pub use nieuw_os_net_protocol::{UdpDatagram, dhcp};
pub mod dhcp_client;
pub mod time;

// Exact predicate from the codec's crate-private arp helper. Moving the client
// here must not widen the protocol crate's public API.
fn unicast_ipv4(ip: [u8; 4]) -> bool {
    ip[0] != 0 && ip[0] != 127 && ip[0] < 224 && ip != [255; 4]
}
