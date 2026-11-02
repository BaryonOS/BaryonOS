#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Sole raw NIC owner. Actual kernel PIDs and fixed ingress/reply endpoints
//! authenticate each owner; NET1 never accepts a caller-supplied reply handle.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_net_stack::{Config, SocketStorage};
use nieuw_os_network_service::{wire, Buffers, Configuration, Destination, Service};
use nieuw_os_sdk_runtime::{
    native, EntropySource, Handle, NetworkSource, Outgoing, ReceiveEndpoint, SendEndpoint,
};
const MAGIC: u64 = u64::from_le_bytes(*b"NETSVC01");
const CONTROL: u64 = u64::from_le_bytes(*b"NETCTL01");
fn fail(code: u64) -> ! {
    support::put(240, code);
    support::put(504, code);
    native::exit(1)
}
fn handle(offset: u64) -> Handle {
    Handle::from_raw(support::word(offset)).unwrap_or_else(|_| fail(2))
}
fn bump(offset: u64) {
    support::put(offset, support::word(offset).saturating_add(1))
}
fn now() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(3))
}
struct Pending {
    id: u64,
    ticket: u64,
    length: usize,
}
struct Bridge {
    source: NetworkSource,
    pending: Option<Pending>,
    rx: [u8; 1514],
}
impl Bridge {
    fn step(&mut self, service: &mut Service<'_>, tick: u64) {
        if let Some(p) = &self.pending {
            match native::net_poll_tx(self.source, p.ticket) {
                Ok(n) => {
                    if n != p.length.max(60) {
                        fail(10)
                    }
                    service
                        .complete_transmission(p.id)
                        .unwrap_or_else(|_| fail(11));
                    self.pending = None;
                    bump(416)
                }
                Err(e) if e.is_again() => {}
                Err(_) => fail(12),
            }
        }
        if self.pending.is_none() {
            if let Some(frame) = service.transmission() {
                match native::net_send(self.source, frame.bytes, 100) {
                    Ok(ticket) => {
                        self.pending = Some(Pending {
                            id: frame.id,
                            ticket,
                            length: frame.bytes.len(),
                        })
                    }
                    Err(e) if e.is_again() => {}
                    Err(_) => fail(13),
                }
            }
        }
        if service.can_receive_frame() {
            match native::net_receive(self.source, &mut self.rx) {
                Ok(n) => {
                    service
                        .ingress(&self.rx[..n], tick)
                        .unwrap_or_else(|_| fail(14));
                    bump(424)
                }
                Err(e) if e.is_again() => {}
                Err(_) => fail(15),
            }
        }
        service.poll(tick).unwrap_or_else(|_| fail(16));
    }
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(1));
    if support::word(32) != MAGIC {
        fail(2)
    }
    let epoch = support::word(40);
    let source = NetworkSource::from_handle(handle(48));
    let entropy = EntropySource::from_handle(handle(104));
    let service_pid = support::word(112);
    let receives = [
        ReceiveEndpoint::from_handle(handle(56)),
        ReceiveEndpoint::from_handle(handle(80)),
    ];
    let replies = [
        SendEndpoint::from_handle(handle(64)),
        SendEndpoint::from_handle(handle(88)),
    ];
    let pids = [support::word(72), support::word(96)];
    let all = [
        source.handle(),
        entropy.handle(),
        receives[0].handle(),
        receives[1].handle(),
        replies[0].handle(),
        replies[1].handle(),
    ];
    if epoch == 0
        || service_pid == 0
        || pids.contains(&service_pid)
        || pids.contains(&0)
        || pids[0] == pids[1]
        || all.iter().enumerate().any(|(i, h)| all[..i].contains(h))
    {
        fail(4)
    }
    let control = match (support::word(152), support::word(160)) {
        (0, 0) => None,
        (raw, pid) if pid != 0 && pid != service_pid && !pids.contains(&pid) => {
            let cap = Handle::from_raw(raw).unwrap_or_else(|_| fail(5));
            if all.contains(&cap) {
                fail(5)
            }
            Some((ReceiveEndpoint::from_handle(cap), pid))
        }
        _ => fail(5),
    };
    let ticket = native::entropy_request(entropy, 8).unwrap_or_else(|_| fail(6));
    let mut seed = [0; 8];
    native::entropy_wait(entropy, ticket, &mut seed, 100_000).unwrap_or_else(|_| fail(7));
    let random_seed = u64::from_le_bytes(seed);
    seed.fill(0);
    let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
    let port = |offset: u64| {
        u16::try_from(support::word(offset))
            .ok()
            .filter(|p| *p != 0)
            .unwrap_or_else(|| fail(8))
    };
    let mut service = Service::new(
        Configuration {
            epoch,
            pids,
            link: Config {
                mac: [2, 0, 0, 0, 0, 7],
                ipv4: [192, 0, 2, 2],
                prefix_len: 24,
                random_seed,
            },
            destinations: [
                Destination {
                    ipv4: [192, 0, 2, 1],
                    tcp_first: port(120),
                    tcp_count: 5,
                    udp_port: port(136),
                },
                Destination {
                    ipv4: [192, 0, 2, 1],
                    tcp_first: port(128),
                    tcp_count: 5,
                    udp_port: port(144),
                },
            ],
        },
        Buffers {
            storage: &mut storage,
            rx_frame: &mut rx,
            tx_frame: &mut tx,
            rx_sockets: [&mut ra, &mut rb],
            tx_sockets: [&mut ta, &mut tb],
        },
        now(),
    )
    .unwrap_or_else(|_| fail(9));
    let mut bridge = Bridge {
        source,
        pending: None,
        rx: [0; 1514],
    };
    let mut pending: [Option<wire::Packet>; 2] = [None, None];
    let mut drain = false;
    support::put(400, 1);
    loop {
        let tick = now();
        bridge.step(&mut service, tick);
        bump(408);
        if let Some((endpoint, pid)) = control {
            match native::receive(endpoint) {
                Ok(message) => {
                    if message.cap_count() != 0 {
                        let _ = native::close_received_caps(&message);
                        fail(17)
                    }
                    if message.sender() != pid
                        || message.tag() != CONTROL
                        || message.payload().len() != 16
                        || message.payload()[..8] != *b"NETSTOP1"
                        || u64::from_le_bytes(message.payload()[8..16].try_into().unwrap()) != epoch
                    {
                        fail(18)
                    }
                    drain = true;
                    service.retire_all(tick).unwrap_or_else(|_| fail(19));
                }
                Err(e) if e.is_again() => {}
                Err(_) => fail(20),
            }
        }
        for slot in 0..2 {
            if let Some(reply) = &pending[slot] {
                let message = Outgoing::new(wire::TAG, reply.bytes()).unwrap_or_else(|_| fail(21));
                match native::send(replies[slot], &message) {
                    Ok(()) => pending[slot] = None,
                    Err(e) if e.is_again() => {}
                    Err(_) => {
                        if drain {
                            pending[slot] = None
                        } else {
                            fail(22)
                        }
                    }
                }
            }
            if pending[slot].is_none() && !drain {
                match native::receive(receives[slot]) {
                    Ok(message) => {
                        if message.cap_count() != 0 {
                            native::close_received_caps(&message).unwrap_or_else(|_| fail(23));
                        }
                        // The syscall envelope is kernel-authenticated; neither
                        // an unrelated tag nor transferred caps invoke NET1.
                        if message.tag() != wire::TAG {
                            continue;
                        }
                        pending[slot] = service
                            .handle(
                                slot,
                                message.sender(),
                                message.cap_count(),
                                message.payload(),
                                tick,
                            )
                            .unwrap_or_else(|_| fail(24));
                    }
                    Err(e) if e.is_again() => {}
                    Err(_) => fail(25),
                }
            }
        }
        let c = service.counters();
        for (at, value) in [
            (432, c.requests),
            (440, c.rejected),
            (448, c.udp_sent),
            (456, c.udp_received),
            (464, c.opened),
            (472, c.sent),
            (480, c.received),
            (488, c.closed),
            (496, service.retired_mask()),
            (512, c.malformed_frames),
            (520, c.dropped_datagrams),
        ] {
            support::put(at, value)
        }
        if service.clean() && pending.iter().all(Option::is_none) && bridge.pending.is_none() {
            for h in all {
                native::close(h).unwrap_or_else(|_| fail(26));
            }
            if let Some((h, _)) = control {
                native::close(h.handle()).unwrap_or_else(|_| fail(27));
            }
            support::put(528, 1);
            native::exit(0)
        }
        native::yield_now().unwrap_or_else(|_| fail(28));
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
