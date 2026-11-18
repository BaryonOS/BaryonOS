//! One-shot, polled USB 2 HID boot-interface transport for the QEMU fixture.
//!
//! Intel xHCI 1.2c §§4.3, 4.6, 6.2 and 6.4; USB HID 1.11 §7.2.6.
//! Only actual transfer completions enter the existing USB Session. The fixture
//! grants no input-policy authority and publishes no userspace device capability.
//! Rings never wrap, all waits are bounded, and every page remains pinned.
use super::*;
use nieuw_os_usb_core::{
    boot_interface::{Protocol, Speed},
    boot_session::{Changes, CompletionStatus, Lifetime, Session, Token},
};

const COUNT: usize = 15;
const COMMAND_RING: usize = 0;
const EVENT_RING: usize = 1;
const ERST: usize = 2;
const DCBAA: usize = 3;
const INPUT_CONTEXT: usize = 4;
const OUTPUT_CONTEXT: usize = 5;
const CONTROL_RING: usize = 7;
const INPUT_RING: usize = 9;
const DESCRIPTOR: usize = 11;
const REPORT: usize = 12;
const GUARD: usize = 14;
const INPUT_POLLS: usize = 30_000_000;
const PORT_CHANGES: u32 = 0x7f << 17;

fn bad(code: u64) -> Error {
    serial::hex(b"USB-DIAG:REJECT:", code);
    Error::Usb
}

fn write32(pa: u64, offset: usize, values: &[u32]) {
    assert!(offset + values.len() <= PAGE as usize / 4);
    paging::physical(pa, |p| {
        for (i, value) in values.iter().enumerate() {
            // SAFETY: Pinned page, aligned bounded writes. The caller owns these
            // unpublished TRBs or has completed the command using this context.
            unsafe { p.cast::<u32>().add(offset + i).write_volatile(*value) };
        }
    });
}

fn clear(pa: u64) {
    paging::physical(pa, |p| {
        // SAFETY: Exactly one pinned page. The caller has no outstanding device
        // operation using its previous contents; no Rust reference is retained.
        unsafe { core::ptr::write_bytes(p, 0, PAGE as usize) };
    });
}

fn bytes<const N: usize>(pa: u64) -> [u8; N] {
    assert!(N <= PAGE as usize);
    paging::physical_read(pa, |p| {
        let mut result = [0; N];
        for (i, value) in result.iter_mut().enumerate() {
            // SAFETY: Bounded volatile reads of a pinned page after the matching
            // completion. No reference aliases the DMA-visible storage.
            *value = unsafe { p.add(i).read_volatile() };
        }
        result
    })
}

#[derive(Clone, Copy)]
struct Event {
    parameter: u64,
    status: u32,
    control: u32,
}
impl Event {
    fn kind(self) -> u32 {
        (self.control >> 10) & 63
    }
    fn code(self) -> u8 {
        (self.status >> 24) as u8
    }
    fn slot(self) -> u8 {
        (self.control >> 24) as u8
    }
    fn endpoint(self) -> u8 {
        ((self.control >> 16) & 31) as u8
    }
    fn remaining(self) -> usize {
        (self.status & 0xff_ffff) as usize
    }
    fn rejected(self) -> Error {
        serial::hex(b"USB-DIAG:EVENT_PARAMETER:", self.parameter);
        serial::hex(b"USB-DIAG:EVENT_STATUS:", u64::from(self.status));
        serial::hex(b"USB-DIAG:EVENT_CONTROL:", u64::from(self.control));
        bad(1)
    }
    fn transfer(self, slot: u8, endpoint: u8, pointer: u64) -> bool {
        self.kind() == 32
            && self.slot() == slot
            && self.endpoint() == endpoint
            && self.parameter == pointer
            && self.control & 4 == 0 // no Event Data TRB is submitted
            && self.control & 0x00e0_03fa == 0
    }
}

struct Engine<'a> {
    controller: &'a Controller,
    pages: [u64; COUNT],
    io: [u64; COUNT],
    command_next: usize,
    event_next: usize,
    control_next: [usize; 2],
    input_next: [usize; 2],
    port_events: u32,
}

impl Engine<'_> {
    fn trb(
        &self,
        ring: usize,
        index: usize,
        parameter: u64,
        status: u32,
        control: u32,
    ) -> Result<u64, Error> {
        // Leave the final entry cycle=0. This bounded fixture never wraps or
        // reuses a TRB, so a stale pointer cannot identify newly submitted work.
        if index >= 255 || control & 1 != 0 {
            return Err(bad(2));
        }
        write32(
            self.pages[ring],
            index * 4,
            &[parameter as u32, (parameter >> 32) as u32, status],
        );
        fence(Ordering::Release);
        write32(self.pages[ring], index * 4 + 3, &[control | 1]);
        Ok(self.io[ring] + index as u64 * 16)
    }

    fn poll(&mut self) -> Result<Option<Event>, Error> {
        if self.event_next >= 255 {
            return Err(bad(3));
        }
        let event = paging::physical_read(self.pages[EVENT_RING], |p| {
            // SAFETY: Pinned event page; bounded aligned volatile access. The
            // xHC publishes the cycle bit last. Read payload only after acquire.
            unsafe {
                let entry = p.cast::<u64>().add(self.event_next * 2);
                let header = entry.add(1).read_volatile();
                if header >> 32 & 1 == 0 {
                    return None;
                }
                fence(Ordering::Acquire);
                Some(Event {
                    parameter: entry.read_volatile(),
                    status: header as u32,
                    control: (header >> 32) as u32,
                })
            }
        });
        if event.is_some() {
            self.event_next += 1;
            let intr = self.controller.caps.runtime_offset + 0x20;
            self.controller.write64(
                intr + 0x18,
                (self.io[EVENT_RING] + self.event_next as u64 * 16) | 8,
            );
        }
        Ok(event)
    }

    fn port_offset(&self, port: u8) -> Result<u64, Error> {
        if port == 0 || port > self.controller.caps.max_ports || port > 31 {
            return Err(bad(4));
        }
        Ok(self.controller.caps.operational_offset + 0x400 + u64::from(port - 1) * 16)
    }

    fn port_event(&mut self, event: Event) -> Result<u8, Error> {
        let port = (event.parameter >> 24) as u8;
        if event.kind() != 34
            || event.code() != 1
            || event.parameter != u64::from(port) << 24
            || event.status != 1 << 24
            || event.control != (34 << 10) | 1
        {
            return Err(event.rejected());
        }
        let offset = self.port_offset(port)?;
        let current = self.controller.read(offset);
        // Preserve power, acknowledge only W1C changes; do not write PED=1.
        self.controller
            .write(offset, (current & (1 << 9)) | (current & PORT_CHANGES));
        self.port_events |= 1 << port;
        Ok(port)
    }

    fn next(&mut self) -> Result<Event, Error> {
        for _ in 0..POLLS {
            if let Some(event) = self.poll()? {
                if event.kind() == 34 {
                    self.port_event(event)?;
                } else {
                    return Ok(event);
                }
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout)
    }

    fn command(&mut self, kind: u32, slot: u8, parameter: u64, endpoint: u8) -> Result<u8, Error> {
        let pointer = self.trb(
            COMMAND_RING,
            self.command_next,
            parameter,
            0,
            kind << 10 | u32::from(endpoint) << 16 | u32::from(slot) << 24,
        )?;
        self.command_next += 1;
        self.controller
            .write(self.controller.caps.doorbell_offset, 0);
        let event = self.next()?;
        if event.kind() != 33
            || event.parameter != pointer
            || event.status != 1 << 24
            || event.control & 0x00ff_03fe != 0
            || (slot != 0 && event.slot() != slot)
            || event.slot() == 0
            || event.slot() > 2
        {
            return Err(event.rejected());
        }
        Ok(event.slot())
    }

    fn control(
        &mut self,
        index: usize,
        slot: u8,
        setup: [u8; 8],
        length: usize,
    ) -> Result<usize, Error> {
        if index >= 2
            || length > 256
            || u16::from_le_bytes([setup[6], setup[7]]) as usize != length
            || (length != 0 && setup[0] & 0x80 == 0)
        {
            return Err(bad(5));
        }
        if length != 0 {
            clear(self.pages[DESCRIPTOR]);
        }
        let ring = CONTROL_RING + index;
        let start = self.control_next[index];
        let trt = if length == 0 { 0 } else { 3 << 16 };
        self.trb(
            ring,
            start,
            u64::from_le_bytes(setup),
            8,
            2 << 10 | 1 << 6 | trt,
        )?;
        let data = if length == 0 {
            None
        } else {
            Some(self.trb(
                ring,
                start + 1,
                self.io[DESCRIPTOR],
                length as u32,
                3 << 10 | 1 << 16 | 1 << 2,
            )?) // ISP, no IOC or CH
        };
        let status_index = start + if data.is_some() { 2 } else { 1 };
        let status = self.trb(
            ring,
            status_index,
            0,
            0,
            4 << 10 | 1 << 5 | if length == 0 { 1 << 16 } else { 0 },
        )?;
        self.control_next[index] = status_index + 1;
        fence(Ordering::SeqCst);
        self.controller.write(
            self.controller.caps.doorbell_offset + u64::from(slot) * 4,
            1,
        );
        let mut actual = length;
        let mut short = false;
        // Intel §6.4.1.2: IOC only Status. A short Data TD with ISP can publish
        // a separate event, but never substitutes for successful Status.
        for _ in 0..2 {
            let event = self.next()?;
            if data.is_some_and(|pointer| event.transfer(slot, 1, pointer)) {
                if short || event.code() != 13 || event.remaining() > length {
                    return Err(event.rejected());
                }
                actual = length - event.remaining();
                short = true;
            } else if event.transfer(slot, 1, status) && event.code() == 1 && event.remaining() == 0
            {
                return Ok(actual);
            } else {
                return Err(event.rejected());
            }
        }
        Err(bad(6))
    }

    fn enumerate(&mut self, index: usize, port: u8, generation: u64) -> Result<Hid, Error> {
        let offset = self.port_offset(port)?;
        let state = self.controller.read(offset);
        if state & 1 == 0 {
            return Err(bad(7));
        }
        // USB 2 port reset; only PP and PR are written, avoiding W1C/PED aliases.
        self.controller.write(offset, (state & (1 << 9)) | 1 << 4);
        self.controller.wait(offset, 1 << 4, 0)?;
        let state = self.controller.read(offset);
        let speed_id = ((state >> 10) & 15) as u8;
        let speed = match speed_id {
            1 => Speed::Full,
            2 => Speed::Low,
            _ => return Err(bad(8)),
        };
        if state & 3 != 3 {
            return Err(bad(9));
        }
        let slot = self.command(9, 0, 0, 0)?; // Enable Slot
        let output = self.io[OUTPUT_CONTEXT + index];
        write32(
            self.pages[DCBAA],
            usize::from(slot) * 2,
            &[output as u32, (output >> 32) as u32],
        );
        clear(self.pages[INPUT_CONTEXT]);
        write32(self.pages[INPUT_CONTEXT], 1, &[3]); // Add Slot + EP0
        write32(
            self.pages[INPUT_CONTEXT],
            8,
            &[u32::from(speed_id) << 20 | 1 << 27, u32::from(port) << 16],
        );
        let ep0 = self.io[CONTROL_RING + index];
        write32(
            self.pages[INPUT_CONTEXT],
            17,
            &[
                3 << 1 | 4 << 3 | 8 << 16,
                ep0 as u32 | 1,
                (ep0 >> 32) as u32,
                8,
            ],
        );
        fence(Ordering::SeqCst);
        self.command(11, slot, self.io[INPUT_CONTEXT], 0)?; // Address Device performs SET_ADDRESS
        let actual = self.control(index, slot, [0x80, 6, 0, 1, 0, 0, 18, 0], 18)?;
        let descriptor = bytes::<18>(self.pages[DESCRIPTOR]);
        if actual != 18
            || descriptor[0] != 18
            || descriptor[1] != 1
            || descriptor[7] != 8
            || descriptor[17] != 1
        {
            return Err(bad(10));
        }
        if self.control(index, slot, [0x80, 6, 0, 2, 0, 0, 9, 0], 9)? != 9 {
            return Err(bad(11));
        }
        let header = bytes::<9>(self.pages[DESCRIPTOR]);
        let total = u16::from_le_bytes([header[2], header[3]]) as usize;
        if header[0] != 9 || header[1] != 2 || !(9..=256).contains(&total) || header[4] != 1 {
            return Err(bad(12));
        }
        let setup = [0x80, 6, 0, 2, 0, 0, total as u8, (total >> 8) as u8];
        if self.control(index, slot, setup, total)? != total {
            return Err(bad(13));
        }
        let configuration = bytes::<256>(self.pages[DESCRIPTOR]);
        let mut session = Session::new(
            Lifetime {
                device: u64::from(port),
                generation,
            },
            &configuration[..total],
            0,
            speed,
        )
        .map_err(|_| bad(14))?;
        let binding = session.binding();
        if self.control(
            index,
            slot,
            [0, 9, binding.configuration_value, 0, 0, 0, 0, 0],
            0,
        )? != 0
        {
            return Err(bad(15));
        }
        let dci = (binding.input_endpoint & 15) * 2 + 1;
        if !(3..=31).contains(&dci) || binding.interval_ms == 0 {
            return Err(bad(16));
        }
        // Intel Table 6-12 footnote 113: round FS/LS interrupt periods down.
        let interval = 3 + (7 - binding.interval_ms.leading_zeros());
        clear(self.pages[INPUT_CONTEXT]);
        write32(self.pages[INPUT_CONTEXT], 1, &[1 | 1 << dci]);
        write32(
            self.pages[INPUT_CONTEXT],
            8,
            &[
                u32::from(speed_id) << 20 | u32::from(dci) << 27,
                u32::from(port) << 16,
            ],
        );
        let ring = self.io[INPUT_RING + index];
        let packet = u32::from(binding.max_packet_size);
        write32(
            self.pages[INPUT_CONTEXT],
            (usize::from(dci) + 1) * 8,
            &[
                interval << 16,
                3 << 1 | 7 << 3 | packet << 16,
                ring as u32 | 1,
                (ring >> 32) as u32,
                packet | packet << 16,
            ],
        );
        fence(Ordering::SeqCst);
        self.command(12, slot, self.io[INPUT_CONTEXT], 0)?;
        let protocol = session.begin_protocol().map_err(|_| bad(17))?;
        let actual = self.control(index, slot, protocol.request.setup(), 0)?;
        session
            .complete_protocol(protocol.token, CompletionStatus::Success, actual)
            .map_err(|_| bad(18))?;
        match binding.protocol {
            Protocol::Keyboard => serial::write(b"USB:ENUM:KEYBOARD\n"),
            Protocol::Mouse => serial::write(b"USB:ENUM:MOUSE\n"),
        }
        Ok(Hid {
            index,
            port,
            slot,
            dci,
            session,
            pending: None,
        })
    }

    fn submit(&mut self, hid: &mut Hid) -> Result<(), Error> {
        if hid.pending.is_some() {
            return Err(bad(19));
        }
        let submission = hid.session.begin_input().map_err(|_| bad(20))?;
        if submission.endpoint != hid.session.binding().input_endpoint || submission.max_length > 8
        {
            return Err(bad(21));
        }
        clear(self.pages[REPORT + hid.index]);
        let pointer = self.trb(
            INPUT_RING + hid.index,
            self.input_next[hid.index],
            self.io[REPORT + hid.index],
            submission.max_length as u32,
            1 << 10 | 1 << 5 | 1 << 2,
        )?;
        self.input_next[hid.index] += 1;
        hid.pending = Some(Pending {
            pointer,
            token: submission.token,
            length: submission.max_length,
        });
        fence(Ordering::SeqCst);
        self.controller.write(
            self.controller.caps.doorbell_offset + u64::from(hid.slot) * 4,
            u32::from(hid.dci),
        );
        Ok(())
    }

    fn complete(&mut self, hid: &mut Hid, event: Event) -> Result<Changes, Error> {
        let pending = hid.pending.ok_or_else(|| bad(22))?;
        if !event.transfer(hid.slot, hid.dci, pending.pointer)
            || ![1, 13].contains(&event.code())
            || event.remaining() > pending.length
        {
            return Err(event.rejected());
        }
        hid.pending = None;
        let actual = pending.length - event.remaining();
        let report = bytes::<8>(self.pages[REPORT + hid.index]);
        hid.session
            .complete_input(pending.token, CompletionStatus::Success, &report[..actual])
            .map_err(|_| bad(23))
    }

    fn disconnect(&mut self, hid: &mut Hid) -> Result<(), Error> {
        let offset = self.port_offset(hid.port)?;
        if self.controller.read(offset) & 1 != 0 {
            return Err(bad(24));
        }
        let pending = hid.pending.ok_or_else(|| bad(25))?;
        // Stop Endpoint may additionally publish a stopped Transfer Event. Keep
        // its old pointer/slot/endpoint bound until the command itself completes.
        let pointer = self.trb(
            COMMAND_RING,
            self.command_next,
            0,
            0,
            15 << 10 | u32::from(hid.dci) << 16 | u32::from(hid.slot) << 24,
        )?;
        self.command_next += 1;
        self.controller
            .write(self.controller.caps.doorbell_offset, 0);
        let mut stopped = false;
        for _ in 0..2 {
            let event = self.next()?;
            if event.kind() == 33
                && event.parameter == pointer
                && event.status == 1 << 24
                && event.control == (u32::from(hid.slot) << 24 | 33 << 10 | 1)
            {
                stopped = true;
                break;
            }
            if !event.transfer(hid.slot, hid.dci, pending.pointer)
                || ![26, 27, 28].contains(&event.code())
            {
                return Err(event.rejected());
            }
        }
        if !stopped {
            return Err(bad(26));
        }
        self.command(10, hid.slot, 0, 0)?; // Disable Slot after endpoint quiescence
        let cleared = hid.session.detach();
        if !matches!(cleared, Changes::Keyboard(changes) if changes.released() == [5]
            && changes.pressed().is_empty() && changes.modifiers_released == 0 && changes.modifiers_pressed == 0)
        {
            return Err(bad(27));
        }
        serial::write(b"USB:UNPLUG_CLEAR:PASS\n");
        if hid
            .session
            .complete_input(pending.token, CompletionStatus::Success, &[0; 8])
            .is_ok()
            || !matches!(hid.session.detach(), Changes::Keyboard(changes) if changes == Default::default())
        {
            return Err(bad(28));
        }
        hid.pending = None;
        serial::write(b"USB:STALE_TOKEN:DENIED\n");
        Ok(())
    }

    fn reports(&mut self, mut keyboard: Hid, mut mouse: Hid) -> Result<(), Error> {
        if keyboard.session.binding().protocol != Protocol::Keyboard
            || mouse.session.binding().protocol != Protocol::Mouse
        {
            return Err(bad(29));
        }
        self.submit(&mut keyboard)?;
        self.submit(&mut mouse)?;
        serial::write(b"USB:READY\n");
        let mut phase = 0;
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            if event.kind() == 34 {
                let port = self.port_event(event)?;
                let connected = self.controller.read(self.port_offset(port)?) & 1 != 0;
                if !connected {
                    if phase != 5 || port != keyboard.port {
                        return Err(bad(30));
                    }
                    self.disconnect(&mut keyboard)?;
                    serial::write(b"USB:REPORTS:PASS\n");
                    return Ok(());
                }
                continue;
            }
            if event.slot() == keyboard.slot {
                let changes = self.complete(&mut keyboard, event)?;
                let Changes::Keyboard(keys) = changes else {
                    return Err(bad(31));
                };
                if keys.status.is_some()
                    || keys.modifiers_pressed != 0
                    || keys.modifiers_released != 0
                {
                    return Err(bad(32));
                }
                if !keys.pressed().is_empty() || !keys.released().is_empty() {
                    match phase {
                        0 if keys.pressed() == [4] && keys.released().is_empty() => {
                            phase = 1;
                            serial::write(b"USB:KEY_DOWN\n");
                        }
                        1 if keys.released() == [4] && keys.pressed().is_empty() => {
                            phase = 2;
                            serial::write(b"USB:KEY_UP\n");
                        }
                        4 if keys.pressed() == [5] && keys.released().is_empty() => {
                            phase = 5;
                            serial::write(b"USB:HELD_KEY\n");
                        }
                        _ => return Err(bad(33)),
                    }
                }
                self.submit(&mut keyboard)?;
            } else if event.slot() == mouse.slot {
                let changes = self.complete(&mut mouse, event)?;
                let Changes::Mouse(mouse_change) = changes else {
                    return Err(bad(34));
                };
                if mouse_change != Default::default() {
                    match phase {
                        2 if mouse_change.dx == 7
                            && mouse_change.dy == -3
                            && mouse_change.buttons_pressed == 1
                            && mouse_change.buttons_released == 0 =>
                        {
                            phase = 3;
                            serial::write(b"USB:MOUSE_DOWN\n");
                        }
                        3 if mouse_change.dx == 0
                            && mouse_change.dy == 0
                            && mouse_change.buttons_pressed == 0
                            && mouse_change.buttons_released == 1 =>
                        {
                            phase = 4;
                            serial::write(b"USB:MOUSE_UP\n");
                        }
                        _ => return Err(bad(35)),
                    }
                }
                self.submit(&mut mouse)?;
            } else {
                return Err(event.rejected());
            }
        }
        Err(Error::Timeout)
    }
}

#[derive(Clone, Copy)]
struct Pending {
    pointer: u64,
    token: Token,
    length: usize,
}
struct Hid {
    index: usize,
    port: u8,
    slot: u8,
    dci: u8,
    session: Session,
    pending: Option<Pending>,
}

pub(super) fn execute(
    controller: &Controller,
    discovery: &boot_protocol::acpi::Discovery,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    function: device::PciFunction,
) -> Result<(), Error> {
    serial::write(b"USB:BEGIN\n");
    if controller.read(0x10) & 4 != 0
        || controller.caps.max_ports != 2
        || controller.caps.max_slots < 2
    {
        return Err(bad(36));
    }
    let driver = device::DriverId::new(0x4e4f_0004).map_err(|_| Error::Dma)?;
    let mut registry = device::LeaseRegistry::<1>::new();
    let mut lease = registry
        .lease(
            function,
            device::DriverMatch::new(driver, 0x1b36, 0x000d, 0x0c, 3, 0x30),
        )
        .map_err(|_| Error::Dma)?;
    lease.authorize(driver, 1).map_err(|_| Error::Dma)?;
    lease.begin_initialization().map_err(|_| Error::Dma)?;
    let mut grants: [Option<device::DmaGrant>; COUNT] = core::array::from_fn(|_| None);
    let mut pages = [0; COUNT];
    for index in 0..COUNT {
        let direction = match index {
            EVENT_RING | DESCRIPTOR | REPORT | 13 => device::DmaDirection::DeviceWrites,
            OUTPUT_CONTEXT | 6 => device::DmaDirection::DeviceReadsWrites,
            _ => device::DmaDirection::DeviceReads,
        };
        let grant = lease
            .allocate_dma(frames, PAGE as usize, direction, paging::zero_dynamic)
            .map_err(|_| Error::Dma)?;
        pages[index] = lease.dma_pages(&grant).map_err(|_| Error::Dma)?[0];
        grants[index] = Some(grant);
    }
    paging::physical(pages[GUARD], |p| {
        // SAFETY: Exclusive pinned guard before bus mastering; full page bounded.
        unsafe { core::ptr::write_bytes(p, 0xa5, PAGE as usize) };
    });
    let references: [&device::DmaGrant; COUNT] =
        core::array::from_fn(|i| grants[i].as_ref().unwrap());
    let mut domain = iommu::establish(discovery, frames, paging, &lease, &references)
        .map_err(|_| Error::Isolation)?;
    let mut io = [0; COUNT];
    for (index, value) in io.iter_mut().enumerate() {
        *value = domain.iova(index, 0).map_err(|_| Error::Dma)?;
    }
    let readonly = cfg!(feature = "test-usb-hid-readonly-fault");
    let event_io = io[if readonly { GUARD } else { EVENT_RING }];
    page_write(pages[ERST], &[(0, event_io), (1, 256)]);
    fence(Ordering::SeqCst);
    paging.audit(frames);
    serial::write(b"USB:PAGING:PASS\nUSB:DOMAIN:PASS\n");
    let mut engine = Engine {
        controller,
        pages,
        io,
        command_next: 0,
        event_next: 0,
        control_next: [0; 2],
        input_next: [0; 2],
        port_events: 0,
    };
    let outcome = (|| {
        controller.enable_dma(&domain)?;
        serial::write(b"USB:BUS_MASTER:CONTAINED\n");
        let op = controller.caps.operational_offset;
        let intr = controller.caps.runtime_offset + 0x20;
        controller.write(op + 0x38, 2);
        controller.write64(op + 0x30, io[DCBAA]);
        controller.write64(op + 0x18, io[COMMAND_RING] | 1);
        controller.write(intr, 0);
        controller.write(intr + 8, 1);
        controller.write64(intr + 0x18, event_io);
        controller.write64(intr + 0x10, io[ERST]);
        controller.write(op, 1); // R/S=1, INTE=0
        controller.wait(op + 4, 1, 0)?;
        if readonly {
            engine.trb(COMMAND_RING, 0, 0, 0, 23 << 10)?;
            controller.write(controller.caps.doorbell_offset, 0);
            for _ in 0..POLLS {
                if let Some(fault) = domain.fault().map_err(|_| Error::Isolation)? {
                    let requester = u16::from(controller.address.bus) << 8
                        | u16::from(controller.address.device) << 3
                        | u16::from(controller.address.function);
                    if fault.requester != requester
                        || fault.address != io[GUARD]
                        || !fault.write
                        || fault.reason != 5
                    {
                        return Err(Error::Isolation);
                    }
                    serial::write(b"USB:READONLY:WRITE_DENIED\n");
                    return Ok(());
                }
                core::hint::spin_loop();
            }
            return Err(Error::Timeout);
        }
        let keyboard = engine.enumerate(0, 1, 1)?;
        let mouse = engine.enumerate(1, 2, 2)?;
        serial::write(b"USB:PROTOCOL:PASS\n");
        engine.reports(keyboard, mouse)?;
        if domain.fault().map_err(|_| Error::Isolation)?.is_some() {
            return Err(Error::Isolation);
        }
        Ok(())
    })();
    // Quarantine on every outcome, including protocol errors or bounded timeout.
    // Clear BME before invalidating, and retain all pages even if either fails.
    let halted = controller.stop();
    let denied = domain.deny_all().map_err(|_| Error::Isolation);
    outcome?;
    halted?;
    denied?;
    if !guard_unchanged(pages[GUARD]) {
        return Err(Error::Guard);
    }
    serial::write(b"USB:GUARD:UNCHANGED\nUSB:QUIESCE:PASS\nUSB:PAGES:RETAINED\nUSB:PASS\n");
    Ok(())
}
