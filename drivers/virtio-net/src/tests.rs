//! Host-only register/DMA MODEL. Never executes PCI I/O or transmits packets.
extern crate std;
use super::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, vec, vec::Vec};

const COMMON: u64 = 0x1000_0000;
const NOTIFY: u64 = COMMON + 0x100;
const CONFIG: u64 = COMMON + 0x300;
const POOL: NetPool = NetPool {
    rx_ring: 0x1000,
    tx_ring: 0x2000,
    rx_bounce: 0x3000,
    tx_bounce: 0x4000,
};
const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];

#[derive(Clone)]
struct Model(Rc<RefCell<Registers>>);

struct Registers {
    bytes: BTreeMap<u64, u8>,
    offered: u64,
    feature_select: u32,
    driver_select: u32,
    negotiated: u64,
    queue_select: usize,
    queue_sizes: [u16; 2],
    queue_enabled: [u16; 2],
    queue_addresses: [[u64; 3]; 2],
    notify_offsets: [u16; 2],
    notifications: Vec<(u64, u16)>,
    bme: bool,
    status: u8,
    queue_count: u16,
    generation: u8,
    spinning_generation: bool,
    reject_features: bool,
    reject_driver_ok: bool,
    refuse_reset: bool,
}

impl Registers {
    fn load(&self, address: u64, length: usize) -> u64 {
        (0..length).fold(0, |value, index| {
            value
                | u64::from(*self.bytes.get(&(address + index as u64)).unwrap_or(&0)) << (index * 8)
        })
    }
    fn store(&mut self, address: u64, value: u64, length: usize) {
        for index in 0..length {
            self.bytes
                .insert(address + index as u64, (value >> (index * 8)) as u8);
        }
    }
}

impl Model {
    fn new() -> Self {
        let mut registers = Registers {
            bytes: BTreeMap::new(),
            offered: VERSION_1 | FEATURE_MAC | FEATURE_STATUS,
            feature_select: 0,
            driver_select: 0,
            negotiated: 0,
            queue_select: 0,
            queue_sizes: [QUEUE_SIZE; 2],
            queue_enabled: [0; 2],
            queue_addresses: [[0; 3]; 2],
            notify_offsets: [0, 1],
            notifications: Vec::new(),
            bme: false,
            status: 0,
            queue_count: 2,
            generation: 0,
            spinning_generation: false,
            reject_features: false,
            reject_driver_ok: false,
            refuse_reset: false,
        };
        for (index, byte) in MAC.iter().enumerate() {
            registers.store(CONFIG + index as u64, u64::from(*byte), 1);
        }
        registers.store(CONFIG + 6, 1, 2);
        Self(Rc::new(RefCell::new(registers)))
    }
    fn read_bytes(&self, address: u64, length: usize) -> Vec<u8> {
        let state = self.0.borrow();
        (0..length)
            .map(|index| state.load(address + index as u64, 1) as u8)
            .collect()
    }
    fn complete(&self, engine: &Engine<Self>, index: usize, delta: u16, id: u32, length: u32) {
        let queue = engine.queues[index];
        let memory = engine.pool.queue(index);
        let at = memory.used() + 4 + u64::from(queue.used % QUEUE_SIZE) * 8;
        let mut state = self.0.borrow_mut();
        state.store(at, u64::from(id), 4);
        state.store(at + 4, u64::from(length), 4);
        state.store(
            memory.used() + 2,
            u64::from(queue.used.wrapping_add(delta)),
            2,
        );
    }
    fn receive(&self, engine: &Engine<Self>, frame: &[u8]) {
        let mut state = self.0.borrow_mut();
        for offset in 0..HEADER_BYTES {
            state.store(POOL.rx_bounce + offset as u64, 0, 1);
        }
        state.store(POOL.rx_bounce + 10, 1, 2);
        for (offset, byte) in frame.iter().enumerate() {
            state.store(
                POOL.rx_bounce + HEADER_BYTES as u64 + offset as u64,
                u64::from(*byte),
                1,
            );
        }
        drop(state);
        self.complete(engine, RX, 1, 0, (HEADER_BYTES + frame.len()) as u32);
    }
}

impl Io for Model {
    fn read8(&self, address: u64) -> u8 {
        let mut state = self.0.borrow_mut();
        if address == COMMON + 20 {
            return state.status;
        }
        if address == COMMON + 21 {
            if state.spinning_generation {
                state.generation = state.generation.wrapping_add(1);
            }
            return state.generation;
        }
        state.load(address, 1) as u8
    }
    fn read16(&self, address: u64) -> u16 {
        let state = self.0.borrow();
        match address {
            a if a == COMMON + 18 => state.queue_count,
            a if a == COMMON + 24 => state.queue_sizes[state.queue_select],
            a if a == COMMON + 28 => state.queue_enabled[state.queue_select],
            a if a == COMMON + 30 => state.notify_offsets[state.queue_select],
            _ => state.load(address, 2) as u16,
        }
    }
    fn read32(&self, address: u64) -> u32 {
        let state = self.0.borrow();
        if address == COMMON + 4 {
            (state.offered >> (state.feature_select * 32)) as u32
        } else {
            state.load(address, 4) as u32
        }
    }
    fn write8(&mut self, address: u64, value: u8) {
        let mut state = self.0.borrow_mut();
        if address == COMMON + 20 {
            if value == 0 && state.refuse_reset {
                return;
            }
            state.status = if value == 11 && state.reject_features {
                3
            } else if value == 15 && state.reject_driver_ok {
                11
            } else {
                value
            };
            if value == 0 {
                state.queue_enabled = [0; 2];
            }
        } else {
            state.store(address, u64::from(value), 1);
        }
    }
    fn write16(&mut self, address: u64, value: u16) {
        let mut state = self.0.borrow_mut();
        if address == COMMON + 22 {
            state.queue_select = usize::from(value);
        } else if address == COMMON + 24 {
            let selected = state.queue_select;
            state.queue_sizes[selected] = value;
        } else if address == COMMON + 28 {
            let selected = state.queue_select;
            state.queue_enabled[selected] = value;
        } else if (NOTIFY..NOTIFY + 32).contains(&address) {
            state.notifications.push((address, value));
        } else {
            state.store(address, u64::from(value), 2);
        }
    }
    fn write32(&mut self, address: u64, value: u32) {
        let mut state = self.0.borrow_mut();
        if address == COMMON {
            state.feature_select = value;
        } else if address == COMMON + 8 {
            state.driver_select = value;
        } else if address == COMMON + 12 {
            let shift = state.driver_select * 32;
            state.negotiated =
                (state.negotiated & !(u64::from(u32::MAX) << shift)) | u64::from(value) << shift;
        } else {
            state.store(address, u64::from(value), 4);
        }
    }
    fn write64(&mut self, address: u64, value: u64) {
        let mut state = self.0.borrow_mut();
        if (COMMON + 32..=COMMON + 48).contains(&address) {
            let selected = state.queue_select;
            state.queue_addresses[selected][((address - COMMON - 32) / 8) as usize] = value;
        } else {
            state.store(address, value, 8);
        }
    }
    fn bus_master(&mut self, enabled: bool) {
        self.0.borrow_mut().bme = enabled;
    }
    fn bus_master_enabled(&self) -> bool {
        self.0.borrow().bme
    }
}

fn info() -> Discovery {
    let mut config = [0; 256];
    config[..4].copy_from_slice(&[0xf4, 0x1a, 0x41, 0x10]);
    config[6] = 0x10;
    config[0x34] = 0x40;
    config[0x10..0x14].copy_from_slice(&(COMMON as u32).to_le_bytes());
    for (index, (offset, size)) in [(0u32, 56u32), (0x100, 32), (0x200, 1), (0x300, 8)]
        .iter()
        .enumerate()
    {
        let at = 0x40 + index * 20;
        config[at] = 9;
        config[at + 1] = if index == 3 { 0 } else { (at + 20) as u8 };
        config[at + 2] = if index == 1 { 20 } else { 16 };
        config[at + 3] = (index + 1) as u8;
        config[at + 8..at + 12].copy_from_slice(&offset.to_le_bytes());
        config[at + 12..at + 16].copy_from_slice(&size.to_le_bytes());
    }
    config[0x64..0x68].copy_from_slice(&4u32.to_le_bytes());
    let mut bars = [Bar::NONE; 6];
    bars[0] = Bar {
        base: COMMON,
        size: 4096,
        is_memory: true,
    };
    let regions = parse_capabilities_for(&config, &bars, DEVICE_ID_NET).unwrap();
    Discovery {
        config,
        bars,
        regions,
    }
}

fn new_engine(model: Model) -> Engine<Model> {
    Engine::new(info(), [COMMON, 0, 0, 0, 0, 0], POOL, model).unwrap()
}
fn online() -> (Engine<Model>, Model) {
    let model = Model::new();
    let mut engine = new_engine(model.clone());
    engine.initialize().unwrap();
    (engine, model)
}

#[test]
fn model_initialization_programs_two_isolated_queues_and_features() {
    let (engine, model) = online();
    assert!(engine.ready && model.bus_master_enabled());
    assert_eq!(engine.mac, MAC);
    assert_eq!(engine.link_up(), Ok(true));
    let state = model.0.borrow();
    assert_eq!(state.negotiated, VERSION_1 | FEATURE_MAC | FEATURE_STATUS);
    assert_eq!(state.queue_enabled, [1, 1]);
    assert_eq!(
        state.queue_addresses[RX],
        [POOL.rx_ring, POOL.rx_ring + 256, POOL.rx_ring + 512]
    );
    assert_eq!(
        state.queue_addresses[TX],
        [POOL.tx_ring, POOL.tx_ring + 256, POOL.tx_ring + 512]
    );
    assert!(!engine.queues[RX].posted && !engine.queues[TX].posted);
}

#[test]
fn feature_profile_does_not_enable_offloads_or_iommu() {
    for offered in [
        0,
        VERSION_1,
        FEATURE_MAC,
        VERSION_1 | FEATURE_MAC | ACCESS_PLATFORM,
    ] {
        assert_eq!(negotiate_features(offered), Err(Error::Unsupported));
    }
    assert_eq!(
        negotiate_features(VERSION_1 | FEATURE_MAC | 0xffff),
        Ok(VERSION_1 | FEATURE_MAC)
    );
    let model = Model::new();
    model.0.borrow_mut().offered &= !FEATURE_STATUS;
    let mut engine = new_engine(model);
    engine.initialize().unwrap();
    assert_eq!(engine.link_up(), Err(Error::Unsupported));
}

#[test]
fn invalid_dma_pages_and_forged_discovery_fail_before_any_io() {
    for pool in [
        NetPool { rx_ring: 0, ..POOL },
        NetPool {
            tx_ring: POOL.rx_ring,
            ..POOL
        },
        NetPool {
            rx_bounce: 4097,
            ..POOL
        },
        NetPool {
            tx_bounce: RAM_END,
            ..POOL
        },
        NetPool {
            tx_bounce: u64::MAX - 4095,
            ..POOL
        },
    ] {
        assert_eq!(pool.validate(RAM_END), Err(Error::Range));
    }
    let model = Model::new();
    let mut forged = info();
    forged.regions.common.offset = 8;
    assert!(matches!(
        Engine::new(forged, [COMMON, 0, 0, 0, 0, 0], POOL, model.clone()),
        Err(Error::Protocol)
    ));
    assert!(matches!(
        Engine::new(info(), [0; 6], POOL, model.clone()),
        Err(Error::Range)
    ));
    assert_eq!(model.0.borrow().bytes.len(), 8);
    assert!(!model.bus_master_enabled());
}

#[test]
fn model_initialization_refusals_are_bounded_and_never_report_ready() {
    for case in 0..8 {
        let model = Model::new();
        {
            let mut state = model.0.borrow_mut();
            match case {
                0 => state.offered &= !FEATURE_MAC,
                1 => state.reject_features = true,
                2 => state.queue_count = 1,
                3 => state.queue_sizes[TX] = 4,
                4 => state.notify_offsets[TX] = u16::MAX,
                5 => state.spinning_generation = true,
                6 => state.store(CONFIG, 1, 1),
                _ => state.reject_driver_ok = true,
            }
        }
        let mut engine = new_engine(model.clone());
        assert!(engine.initialize().is_err(), "case {case}");
        // Mirrors the public wrapper's error path: failed init always resets.
        engine.begin_reset();
        assert!(!engine.ready && !model.bus_master_enabled());
        assert!(engine.reset_acknowledged());
    }
}

#[test]
fn tx_publication_has_real_bytes_zero_header_padding_and_no_early_completion() {
    let (mut engine, model) = online();
    let frame = [0xa5; 42];
    engine.send(&frame, 5, 10).unwrap();
    assert_eq!(
        model.read_bytes(POOL.tx_bounce, HEADER_BYTES),
        vec![0; HEADER_BYTES]
    );
    assert_eq!(
        model.read_bytes(POOL.tx_bounce + HEADER_BYTES as u64, 42),
        frame
    );
    assert_eq!(
        model.read_bytes(POOL.tx_bounce + HEADER_BYTES as u64 + 42, 18),
        vec![0; 18]
    );
    assert_eq!(model.0.borrow().load(POOL.tx_ring + 8, 4), 72);
    assert_eq!(model.0.borrow().load(POOL.tx_ring + 12, 2), 0);
    assert_eq!(
        model.0.borrow().notifications.last(),
        Some(&(NOTIFY + 4, 1))
    );
    assert_eq!(engine.poll_tx(6), Ok(None));
    assert_eq!(engine.send(&frame, 6, 10), Err(Error::Busy));
    model.complete(&engine, TX, 1, 0, 0);
    assert_eq!(engine.poll_tx(14), Ok(Some(60)));
    assert_eq!(engine.poll_tx(14), Ok(None));
}

#[test]
fn tx_errors_never_publish_or_change_bounce_data() {
    let (mut engine, model) = online();
    let before = model.read_bytes(POOL.tx_bounce, BUFFER_BYTES);
    for (frame, now, timeout) in [
        (vec![0; 13], 0, 1),
        (vec![0; MAX_FRAME_BYTES + 1], 0, 1),
        (vec![0; 60], 0, 0),
        (vec![0; 60], u64::MAX, 1),
    ] {
        assert_eq!(engine.send(&frame, now, timeout), Err(Error::Range));
        assert!(!engine.queues[TX].posted);
        assert_eq!(model.read_bytes(POOL.tx_bounce, BUFFER_BYTES), before);
    }
    engine.send(&[0x7c; MAX_FRAME_BYTES], 0, 1).unwrap();
    assert_eq!(
        model.0.borrow().load(POOL.tx_ring + 8, 4),
        BUFFER_BYTES as u64
    );
}

#[test]
fn rx_copies_only_valid_completed_frame_and_preserves_output_tail() {
    let (mut engine, model) = online();
    engine.post_rx().unwrap();
    assert_eq!(engine.post_rx(), Err(Error::Busy));
    assert_eq!(
        model.0.borrow().load(POOL.rx_ring + 12, 2),
        u64::from(WRITE)
    );
    let mut output = [0xa5; 70];
    assert_eq!(engine.poll_rx(&mut output), Ok(None));
    let frame = [0x73; 60];
    model.receive(&engine, &frame);
    assert_eq!(engine.poll_rx(&mut output[..59]), Err(Error::Range));
    assert_eq!(output, [0xa5; 70]);
    assert!(engine.queues[RX].posted);
    assert_eq!(engine.poll_rx(&mut output), Ok(Some(60)));
    assert_eq!(output[..60], frame);
    assert_eq!(output[60..], [0xa5; 10]);
    assert_eq!(engine.poll_rx(&mut output), Ok(None));
    engine.post_rx().unwrap();
}

#[test]
fn malformed_used_ring_snapshots_reset_without_partial_rx_copy() {
    for (delta, id, length) in [
        (2, 0, 72),
        (1, 1, 72),
        (1, 8, 72),
        (1, 0, 0),
        (1, 0, 25),
        (1, 0, BUFFER_BYTES as u32 + 1),
        (1, u32::MAX, u32::MAX),
    ] {
        let (mut engine, model) = online();
        engine.post_rx().unwrap();
        model.complete(&engine, RX, delta, id, length);
        let mut output = [0xa5; MAX_FRAME_BYTES];
        assert_eq!(engine.poll_rx(&mut output), Err(Error::Protocol));
        assert_eq!(output, [0xa5; MAX_FRAME_BYTES]);
        assert!(!engine.ready && !model.bus_master_enabled());
        assert!(engine.reset_acknowledged());
    }
}

#[test]
fn unsupported_rx_offloads_and_multibuffer_headers_are_not_accepted() {
    for offset in 0..HEADER_BYTES {
        let mut header = [0; HEADER_BYTES];
        header[offset] = if offset == 10 { 2 } else { 1 };
        assert_eq!(validate_rx_header(&header), Err(Error::Protocol));
    }
    let mut header = [0; HEADER_BYTES];
    header[0] = 2;
    header[10] = 1;
    assert_eq!(validate_rx_header(&header), Ok(()));
    let (mut engine, model) = online();
    engine.post_rx().unwrap();
    model.receive(&engine, &[0; 60]);
    model.0.borrow_mut().store(POOL.rx_bounce, 1, 1);
    let mut output = [0xa5; 60];
    assert_eq!(engine.poll_rx(&mut output), Err(Error::Protocol));
    assert_eq!(output, [0xa5; 60]);
    assert!(engine.reset_acknowledged());
}

#[test]
fn wrapping_queue_indices_do_not_confuse_stale_or_duplicate_completions() {
    let (mut engine, model) = online();
    for index in [RX, TX] {
        engine.queues[index].available = u16::MAX;
        engine.queues[index].used = u16::MAX;
        model
            .0
            .borrow_mut()
            .store(engine.pool.queue(index).used() + 2, u64::from(u16::MAX), 2);
    }
    engine.post_rx().unwrap();
    engine.send(&[0; 60], 0, 10).unwrap();
    assert_eq!(engine.queues[RX].available, 0);
    assert_eq!(engine.queues[TX].available, 0);
    model.receive(&engine, &[0x42; 60]);
    model.complete(&engine, TX, 1, 0, 0);
    assert_eq!(engine.poll_rx(&mut [0; 60]), Ok(Some(60)));
    assert_eq!(engine.poll_tx(1), Ok(Some(60)));
    assert_eq!(engine.queues[RX].used, 0);
    assert_eq!(engine.queues[TX].used, 0);
    engine.post_rx().unwrap();
    assert_eq!(engine.poll_rx(&mut [0; 60]), Ok(None));
}

#[test]
fn tx_timeout_and_device_reset_request_fail_closed_with_no_replay() {
    let (mut engine, model) = online();
    engine.send(&[0; 60], 0, 5).unwrap();
    assert_eq!(engine.poll_tx(4), Ok(None));
    assert_eq!(engine.poll_tx(5), Err(Error::Timeout));
    assert!(engine.reset_acknowledged());
    assert_eq!(engine.send(&[0; 60], 6, 10), Err(Error::Offline));
    assert_eq!(model.0.borrow().notifications.len(), 1);
    let (mut engine, model) = online();
    model.0.borrow_mut().status |= 64;
    assert_eq!(engine.post_rx(), Err(Error::Io));
    assert!(engine.reset_acknowledged());
}

#[test]
fn reset_request_is_not_acknowledgement_and_pinned_memory_is_not_reused() {
    let (mut engine, model) = online();
    engine.post_rx().unwrap();
    model.0.borrow_mut().refuse_reset = true;
    let before = model.read_bytes(POOL.rx_bounce, BUFFER_BYTES);
    engine.begin_reset();
    assert!(!engine.reset_acknowledged());
    assert!(!model.bus_master_enabled());
    assert_eq!(engine.post_rx(), Err(Error::Offline));
    assert_eq!(model.read_bytes(POOL.rx_bounce, BUFFER_BYTES), before);
    model.0.borrow_mut().refuse_reset = false;
    engine.begin_reset();
    assert!(engine.reset_acknowledged());
}

#[test]
fn tx_writable_length_and_max_rx_frame_are_validated() {
    let (mut engine, model) = online();
    engine.send(&[0; 60], 0, 10).unwrap();
    model.complete(&engine, TX, 1, 0, 1);
    assert_eq!(engine.poll_tx(1), Err(Error::Protocol));
    let (mut engine, model) = online();
    engine.post_rx().unwrap();
    model.receive(&engine, &[0x8b; MAX_FRAME_BYTES]);
    let mut output = [0; MAX_FRAME_BYTES];
    assert_eq!(engine.poll_rx(&mut output), Ok(Some(MAX_FRAME_BYTES)));
    assert_eq!(output, [0x8b; MAX_FRAME_BYTES]);
}

#[test]
fn late_completions_before_reset_ack_cannot_copy_replay_or_reuse_pages() {
    for timeout in [false, true] {
        let (mut engine, model) = online();
        engine.post_rx().unwrap();
        engine.send(&[0x31; 60], 0, 5).unwrap();
        model.0.borrow_mut().refuse_reset = true;
        if timeout {
            assert_eq!(engine.poll_tx(5), Err(Error::Timeout));
        } else {
            engine.begin_reset();
        }
        assert!(!engine.reset_acknowledged());
        model.receive(&engine, &[0x92; 60]);
        model.complete(&engine, TX, 1, 0, 0);
        let pages = [POOL.rx_ring, POOL.tx_ring, POOL.rx_bounce, POOL.tx_bounce]
            .map(|page| model.read_bytes(page, 4096));
        let notifications = model.0.borrow().notifications.clone();
        let mut output = [0xa5; 70];
        for _ in 0..3 {
            assert_eq!(engine.poll_rx(&mut output), Err(Error::Offline));
            assert_eq!(engine.poll_tx(6), Err(Error::Offline));
            assert_eq!(engine.post_rx(), Err(Error::Offline));
            assert_eq!(engine.send(&[0x53; 60], 6, 5), Err(Error::Offline));
            assert!(!engine.reset_acknowledged());
        }
        assert_eq!(output, [0xa5; 70]);
        assert!(!engine.ready && !model.bus_master_enabled());
        assert!(engine.queues.iter().all(|queue| !queue.posted));
        let mut candidate = new_engine(model.clone());
        assert_eq!(candidate.initialize(), Err(Error::Timeout));
        assert!(!candidate.ready && !model.bus_master_enabled());
        for (page, before) in [POOL.rx_ring, POOL.tx_ring, POOL.rx_bounce, POOL.tx_bounce]
            .into_iter()
            .zip(pages)
        {
            assert_eq!(model.read_bytes(page, 4096), before);
        }
        assert_eq!(model.0.borrow().notifications, notifications);
    }
}

#[test]
fn acknowledged_reset_clears_late_completions_before_fresh_rx_and_tx() {
    for initial_used in [0, u16::MAX] {
        let (mut engine, model) = online();
        for index in [RX, TX] {
            engine.queues[index].used = initial_used;
            engine.queues[index].available = initial_used;
            model.0.borrow_mut().store(
                engine.pool.queue(index).used() + 2,
                u64::from(initial_used),
                2,
            );
        }
        engine.post_rx().unwrap();
        engine.send(&[0x31; 60], 0, 5).unwrap();
        model.0.borrow_mut().refuse_reset = true;
        engine.begin_reset();
        model.receive(&engine, &[0x92; 60]);
        model.complete(&engine, TX, 1, 0, 0);
        assert!(!engine.reset_acknowledged());
        model.0.borrow_mut().refuse_reset = false;
        engine.begin_reset();
        assert!(engine.reset_acknowledged());
        drop(engine);

        let notifications = model.0.borrow().notifications.len();
        let mut fresh = new_engine(model.clone());
        fresh.initialize().unwrap();
        assert_eq!(model.0.borrow().notifications.len(), notifications);
        for index in [RX, TX] {
            let memory = fresh.pool.queue(index);
            assert_eq!(model.read_bytes(memory.bounce, 4096), vec![0; 4096]);
            assert_eq!(model.read_bytes(memory.used(), 68), vec![0; 68]);
            assert_eq!(fresh.queues[index].used, 0);
            assert_eq!(fresh.queues[index].available, 0);
            assert!(!fresh.queues[index].posted);
        }
        let mut output = [0xa5; 70];
        assert_eq!(fresh.poll_rx(&mut output), Ok(None));
        assert_eq!(fresh.poll_tx(10), Ok(None));
        fresh.post_rx().unwrap();
        fresh.send(&[0x64; 42], 10, 5).unwrap();
        assert_eq!(fresh.poll_rx(&mut output), Ok(None));
        assert_eq!(fresh.poll_tx(11), Ok(None));
        assert_eq!(output, [0xa5; 70]);
        assert_eq!(
            model.read_bytes(POOL.tx_bounce + HEADER_BYTES as u64, 60),
            [vec![0x64; 42], vec![0; 18]].concat()
        );
        model.receive(&fresh, &[0x75; 60]);
        model.complete(&fresh, TX, 1, 0, 0);
        assert_eq!(fresh.poll_rx(&mut output), Ok(Some(60)));
        assert_eq!(output[..60], [0x75; 60]);
        assert_eq!(output[60..], [0xa5; 10]);
        assert_eq!(fresh.poll_tx(12), Ok(Some(60)));
        assert_eq!(fresh.poll_rx(&mut output), Ok(None));
        assert_eq!(fresh.poll_tx(12), Ok(None));
        assert_eq!(model.0.borrow().notifications.len(), notifications + 2);
    }
}

#[test]
fn status_generation_and_link_changes_are_observed_with_a_bound() {
    let (engine, model) = online();
    model.0.borrow_mut().store(CONFIG + 6, 0, 2);
    assert_eq!(engine.link_up(), Ok(false));
    model.0.borrow_mut().spinning_generation = true;
    assert_eq!(engine.link_up(), Err(Error::Busy));
    assert_eq!(engine.read_mac(), Err(Error::Busy));
    model.0.borrow_mut().status |= 64;
    assert_eq!(engine.link_up(), Err(Error::Io));
}

#[test]
fn stale_used_entry_while_idle_cannot_become_a_new_request_completion() {
    for index in [RX, TX] {
        let (mut engine, model) = online();
        model.complete(&engine, index, 1, 0, if index == RX { 72 } else { 0 });
        let before = model.read_bytes(engine.pool.queue(index).bounce, BUFFER_BYTES);
        let result = if index == RX {
            engine.post_rx()
        } else {
            engine.send(&[0x51; 60], 0, 10)
        };
        assert_eq!(result, Err(Error::Protocol));
        assert_eq!(
            model.read_bytes(engine.pool.queue(index).bounce, BUFFER_BYTES),
            before
        );
        assert!(engine.reset_acknowledged());
        assert!(model.0.borrow().notifications.is_empty());
    }
}
