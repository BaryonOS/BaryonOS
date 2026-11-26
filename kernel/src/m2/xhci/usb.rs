//! One-shot, polled USB 2 HID boot-interface transport for the QEMU fixture.
//!
//! Intel xHCI 1.2c §§4.3, 4.6, 6.2 and 6.4; USB HID 1.11 §7.2.6.
//! Only actual transfer completions enter the existing USB Session. The fixture
//! grants no input-policy authority and publishes no userspace device capability.
//! Rings never wrap, all waits are bounded, and every page remains pinned.
//! Device lifetimes come from the bounded `hotplug::Ports` tracker. The opt-in
//! `native-usb-hid-reattach` profile re-enumerates one same-port keyboard as a
//! new generation and rejects generation-1 work after slot reuse. The separate
//! `native-usb-hid-mouse-reattach` profile does the same for the mouse, after
//! the keyboard is retired, so its generation 2 can reuse the keyboard's slot.
//! The separate `native-usb-hid-refusal` profile refuses an out-of-profile
//! device on one port without ending service of the other port.
use super::*;
#[cfg(any(feature = "usb-hid-reattach-core", feature = "native-usb-hid-refusal"))]
use nieuw_os_usb_core::hotplug::{Error as PortError, Phase};
use nieuw_os_usb_core::{
    boot::MouseChanges,
    boot_interface::{Error as BindingError, Protocol, Speed},
    boot_session::{Changes, CompletionStatus, Error as SessionError, Lifetime, Session, Token},
    hotplug::{Ports, Ticket},
};

// Every grant exists before the fixed domain is established; none is added
// later. The reattach profile uses the single spare page of the 16-page limit.
#[cfg(not(feature = "usb-hid-reattach-core"))]
const COUNT: usize = 15;
#[cfg(feature = "usb-hid-reattach-core")]
const COUNT: usize = 16;
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
/// Index of the device the reattach profile re-enumerates: 0 = keyboard on
/// root port 1, 1 = mouse on root port 2.
#[cfg(feature = "usb-hid-reattach-core")]
const TARGET: usize = if cfg!(feature = "native-usb-hid-mouse-reattach") {
    1
} else {
    0
};
#[cfg(feature = "usb-hid-reattach-core")]
const TARGET_PROTOCOL: Protocol = if TARGET == 1 {
    Protocol::Mouse
} else {
    Protocol::Keyboard
};
/// Generation-2 device page, readable and writable by the device: its output
/// device context at offset 0 (1 KiB with the required 32-byte contexts) and
/// its report buffer at `SPARE_REPORT`. It holds only zeroed, xHC-maintained
/// context and report bytes.
#[cfg(feature = "usb-hid-reattach-core")]
const SPARE: usize = 15;
#[cfg(feature = "usb-hid-reattach-core")]
const SPARE_REPORT: usize = 0x800;
/// Fill for retired generation-1 pages that stay mapped in the fixed domain.
/// Comparing it at the end detects a stray device write; it cannot prevent one.
#[cfg(feature = "usb-hid-reattach-core")]
const RETIRED: u8 = 0x3c;
#[cfg(feature = "usb-hid-reattach-core")]
const QUIET_POLLS: usize = 3_000_000;
const REPORT_BYTES: usize = 8;
const INPUT_POLLS: usize = 30_000_000;
const PORT_CHANGES: u32 = 0x7f << 17;

fn bad(code: u64) -> Error {
    serial::hex(b"USB-DIAG:REJECT:", code);
    Error::Usb
}

/// Why the device on one root port is outside this low/full-speed boot-HID
/// profile. Every reason is decided before SET_CONFIGURATION, before any
/// interrupt endpoint context, and without a Session.
#[derive(Clone, Copy)]
enum Refusal {
    /// PORTSC speed is neither low nor full (high-speed devices, storage).
    Speed,
    /// USB 2.0 §9.6.1/§11.23.1: bDeviceClass 9 is a hub.
    Hub,
    /// Interface 0 is not a HID boot keyboard or mouse (USB HID 1.11 §4.2).
    NotBoot,
    /// Any other configuration the boot-interface binding rejects.
    Binding,
}

#[cfg_attr(not(feature = "native-usb-hid-refusal"), allow(dead_code))]
impl Refusal {
    fn marker(self) -> &'static [u8] {
        match self {
            Refusal::Speed => b"USB:REFUSE:SPEED\n",
            Refusal::Hub => b"USB:REFUSE:HUB\n",
            Refusal::NotBoot => b"USB:REFUSE:NOT_BOOT\n",
            Refusal::Binding => b"USB:REFUSE:BINDING\n",
        }
    }
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

/// Clears only one report region; the rest of its page may be a live context.
fn clear_bytes(pa: u64, offset: usize, length: usize) {
    assert!(offset
        .checked_add(length)
        .is_some_and(|end| end <= PAGE as usize));
    paging::physical(pa, |p| {
        // SAFETY: Bounded range inside one pinned page. The caller has no
        // outstanding device operation using this range; no reference is kept.
        unsafe { core::ptr::write_bytes(p.add(offset), 0, length) };
    });
}

fn bytes<const N: usize>(pa: u64, offset: usize) -> [u8; N] {
    assert!(offset
        .checked_add(N)
        .is_some_and(|end| end <= PAGE as usize));
    paging::physical_read(pa, |p| {
        let mut result = [0; N];
        for (i, value) in result.iter_mut().enumerate() {
            // SAFETY: Bounded volatile reads of a pinned page after the matching
            // completion. No reference aliases the DMA-visible storage.
            *value = unsafe { p.add(offset + i).read_volatile() };
        }
        result
    })
}

#[cfg(feature = "usb-hid-reattach-core")]
fn fill(pa: u64, value: u8) {
    paging::physical(pa, |p| {
        // SAFETY: One pinned page whose device work is retired by a completed
        // Disable Slot; exactly PAGE bytes, no reference retained.
        unsafe { core::ptr::write_bytes(p, value, PAGE as usize) };
    });
}

#[cfg(feature = "usb-hid-reattach-core")]
fn filled(pa: u64, value: u8) -> bool {
    paging::physical_read(pa, |p| {
        // SAFETY: Retained pinned page, exactly one page of bounded volatile
        // reads after controller halt, bus-master clear and domain denial.
        (0..PAGE as usize).all(|i| unsafe { p.add(i).read_volatile() } == value)
    })
}

/// A device-written report region: pinned page, byte offset and its IOVA.
#[derive(Clone, Copy)]
struct Buffer {
    page: u64,
    offset: usize,
    io: u64,
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

    /// `lifetime` must be the identity of the current Ports ticket. `output`
    /// is a zeroed output-context IOVA and `report` a buffer that no earlier
    /// generation of this port used.
    fn enumerate(
        &mut self,
        index: usize,
        port: u8,
        lifetime: Lifetime,
        output: u64,
        report: Buffer,
    ) -> Result<Hid, Error> {
        // A later generation continues each per-device ring. Old TRBs keep
        // cycle=1 and must never become a new dequeue position.
        if index >= 2 || self.control_next[index] >= 255 || self.input_next[index] >= 255 {
            return Err(bad(37));
        }
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
            // No slot exists yet: nothing to undo.
            _ => return Err(self.refuse(Refusal::Speed, None, 8)),
        };
        if state & 3 != 3 {
            return Err(bad(9));
        }
        let slot = self.command(9, 0, 0, 0)?; // Enable Slot
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
        let ep0 = self.io[CONTROL_RING + index] + self.control_next[index] as u64 * 16;
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
        let descriptor = bytes::<18>(self.pages[DESCRIPTOR], 0);
        if actual != 18
            || descriptor[0] != 18
            || descriptor[1] != 1
            || descriptor[7] != 8
            || descriptor[17] != 1
        {
            return Err(self.refuse(Refusal::Binding, Some(slot), 10));
        }
        // Refused before its configuration is read; other profiles keep code 14.
        if descriptor[4] == 9 {
            return Err(self.refuse(Refusal::Hub, Some(slot), 14));
        }
        if self.control(index, slot, [0x80, 6, 0, 2, 0, 0, 9, 0], 9)? != 9 {
            return Err(self.refuse(Refusal::Binding, Some(slot), 11));
        }
        let header = bytes::<9>(self.pages[DESCRIPTOR], 0);
        let total = u16::from_le_bytes([header[2], header[3]]) as usize;
        if header[0] != 9 || header[1] != 2 || !(9..=256).contains(&total) || header[4] != 1 {
            return Err(self.refuse(Refusal::Binding, Some(slot), 12));
        }
        let setup = [0x80, 6, 0, 2, 0, 0, total as u8, (total >> 8) as u8];
        if self.control(index, slot, setup, total)? != total {
            return Err(self.refuse(Refusal::Binding, Some(slot), 13));
        }
        let configuration = bytes::<256>(self.pages[DESCRIPTOR], 0);
        let mut session = match Session::new(lifetime, &configuration[..total], 0, speed) {
            Ok(session) => session,
            Err(SessionError::Binding(BindingError::NotBoot)) => {
                return Err(self.refuse(Refusal::NotBoot, Some(slot), 14))
            }
            Err(_) => return Err(self.refuse(Refusal::Binding, Some(slot), 14)),
        };
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
        let ring = self.io[INPUT_RING + index] + self.input_next[index] as u64 * 16;
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
        Ok(Hid {
            index,
            port,
            slot,
            dci,
            session,
            pending: None,
            report,
        })
    }

    /// Refuses the device on this port. In the refusal profile a slot that
    /// Address Device created (EP0 only, no transfer pending) is disabled and
    /// its DCBAA entry cleared before `Refused` returns; a failed Disable Slot
    /// stays terminal. Every other profile keeps its terminal reject `code`.
    fn refuse(&mut self, reason: Refusal, slot: Option<u8>, code: u64) -> Error {
        #[cfg(feature = "native-usb-hid-refusal")]
        {
            let _ = code;
            if let Some(slot) = slot {
                if let Err(error) = self.command(10, slot, 0, 0) {
                    return error;
                }
                write32(self.pages[DCBAA], usize::from(slot) * 2, &[0, 0]);
                fence(Ordering::SeqCst);
                if let Err(error) = self.slot_disabled(slot) {
                    return error;
                }
            }
            serial::write(reason.marker());
            Error::Refused
        }
        #[cfg(not(feature = "native-usb-hid-refusal"))]
        {
            let _ = (reason, slot);
            bad(code)
        }
    }

    /// Negative check after Disable Slot: a Reset Device command for the same
    /// slot must complete with Slot Not Enabled (xHCI 1.2c §4.6.11, code 11).
    #[cfg(feature = "native-usb-hid-refusal")]
    fn slot_disabled(&mut self, slot: u8) -> Result<(), Error> {
        let pointer = self.trb(
            COMMAND_RING,
            self.command_next,
            0,
            0,
            17 << 10 | u32::from(slot) << 24,
        )?;
        self.command_next += 1;
        self.controller
            .write(self.controller.caps.doorbell_offset, 0);
        let event = self.next()?;
        if event.kind() != 33
            || event.parameter != pointer
            || event.code() != 11
            // QEMU reports Slot ID 0 for a slot that is no longer enabled.
            || (event.slot() != slot && event.slot() != 0)
        {
            return Err(event.rejected());
        }
        // The controller never reads DCBAA for a disabled slot, so check the clear.
        if bytes::<8>(self.pages[DCBAA], usize::from(slot) * 8) != [0; 8] {
            return Err(bad(62));
        }
        serial::write(b"USB:REFUSE:SLOT_DISABLED\n");
        Ok(())
    }

    /// One Ports-correlated attach. Only a real, matched enumeration result
    /// completes the ticket; a failure is recorded before the error returns.
    /// `retired` is the previous generation's ticket for this port, if any.
    fn attach(
        &mut self,
        ports: &mut Ports<2>,
        index: usize,
        port: u8,
        output: u64,
        report: Buffer,
        retired: Option<Ticket>,
    ) -> Result<(Hid, Ticket), Error> {
        let ticket = ports.attach(index).map_err(|_| bad(38))?;
        // While the new generation is Enumerating, the phase check passes, so
        // only the lifetime and attempt can refuse the retired ticket. The
        // new ticket is accepted below in the same phase.
        #[cfg(feature = "usb-hid-reattach-core")]
        {
            if let Some(old) = retired {
                if ports.complete(old, true) != Err(PortError::Stale)
                    || ports.phase(index) != Ok(Phase::Enumerating)
                {
                    let _ = ports.complete(ticket, false);
                    return Err(bad(51));
                }
            }
        }
        #[cfg(not(feature = "usb-hid-reattach-core"))]
        let _ = retired;
        let result = self.enumerate(index, port, ticket.lifetime(), output, report);
        let completed = ports.complete(ticket, result.is_ok());
        let hid = result?;
        if completed != Ok(Some(ticket.lifetime())) {
            return Err(bad(39));
        }
        Ok((hid, ticket))
    }

    fn submit(&mut self, hid: &mut Hid) -> Result<(), Error> {
        if hid.pending.is_some() {
            return Err(bad(19));
        }
        let submission = hid.session.begin_input().map_err(|_| bad(20))?;
        if submission.endpoint != hid.session.binding().input_endpoint
            || submission.max_length > REPORT_BYTES
        {
            return Err(bad(21));
        }
        clear_bytes(hid.report.page, hid.report.offset, REPORT_BYTES);
        let pointer = self.trb(
            INPUT_RING + hid.index,
            self.input_next[hid.index],
            hid.report.io,
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

    /// Exact correlation of a Transfer Event with this device's submitted TRB.
    fn accepts(hid: &Hid, pending: Pending, event: Event) -> bool {
        event.transfer(hid.slot, hid.dci, pending.pointer)
            && [1, 13].contains(&event.code())
            && event.remaining() <= pending.length
    }

    fn complete(&mut self, hid: &mut Hid, event: Event) -> Result<Changes, Error> {
        let pending = hid.pending.ok_or_else(|| bad(22))?;
        if !Self::accepts(hid, pending, event) {
            return Err(event.rejected());
        }
        hid.pending = None;
        let actual = pending.length - event.remaining();
        let report = bytes::<REPORT_BYTES>(hid.report.page, hid.report.offset);
        hid.session
            .complete_input(pending.token, CompletionStatus::Success, &report[..actual])
            .map_err(|_| bad(23))
    }

    /// After an observed disconnect: correlated Stop Endpoint, then Disable
    /// Slot, then terminal Session detach releasing exactly the held `keys`
    /// (keyboard) or `buttons` (mouse) and nothing else.
    /// Returns the detached work so its token can be checked as stale.
    #[cfg_attr(feature = "native-usb-hid-refusal", allow(dead_code))]
    fn disconnect(&mut self, hid: &mut Hid, keys: &[u8], buttons: u8) -> Result<Pending, Error> {
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
        let released = match hid.session.detach() {
            Changes::Keyboard(changes) => {
                buttons == 0
                    && changes.released() == keys
                    && changes.pressed().is_empty()
                    && changes.modifiers_released == 0
                    && changes.modifiers_pressed == 0
            }
            Changes::Mouse(changes) => {
                keys.is_empty()
                    && changes
                        == MouseChanges {
                            buttons_released: buttons,
                            ..Default::default()
                        }
            }
        };
        if !released {
            return Err(bad(27));
        }
        Ok(pending)
    }

    /// The detached token cannot complete, and repeated detach releases nothing.
    #[cfg_attr(feature = "native-usb-hid-refusal", allow(dead_code))]
    fn deny_stale(hid: &mut Hid, pending: Pending) -> Result<(), Error> {
        let idle = match hid.session.binding().protocol {
            Protocol::Keyboard => Changes::Keyboard(Default::default()),
            Protocol::Mouse => Changes::Mouse(Default::default()),
        };
        if hid
            .session
            .complete_input(pending.token, CompletionStatus::Success, &[0; REPORT_BYTES])
            .is_ok()
            || hid.session.detach() != idle
        {
            return Err(bad(28));
        }
        hid.pending = None;
        Ok(())
    }

    #[cfg_attr(feature = "native-usb-hid-refusal", allow(dead_code))]
    fn reports(&mut self, keyboard: &mut Hid, mouse: &mut Hid) -> Result<Retired, Error> {
        if keyboard.session.binding().protocol != Protocol::Keyboard
            || mouse.session.binding().protocol != Protocol::Mouse
        {
            return Err(bad(29));
        }
        self.submit(keyboard)?;
        self.submit(mouse)?;
        serial::write(b"USB:READY\n");
        let mut phase = 0;
        let mut captured = None;
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
                    let pending = self.disconnect(keyboard, &[5], 0)?;
                    serial::write(b"USB:UNPLUG_CLEAR:PASS\n");
                    Self::deny_stale(keyboard, pending)?;
                    serial::write(b"USB:STALE_TOKEN:DENIED\n");
                    serial::write(b"USB:REPORTS:PASS\n");
                    return Ok(Retired {
                        pending,
                        event: captured,
                    });
                }
                if cfg!(feature = "usb-hid-reattach-core") {
                    // This profile admits a connect only in its Detached wait.
                    return Err(bad(40));
                }
                continue;
            }
            if event.slot() == keyboard.slot {
                let changes = self.complete(keyboard, event)?;
                captured = Some(event);
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
                self.submit(keyboard)?;
            } else if event.slot() == mouse.slot {
                let changes = self.complete(mouse, event)?;
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
                self.submit(mouse)?;
            } else {
                return Err(event.rejected());
            }
        }
        Err(Error::Timeout)
    }

    /// Ports detach after the completed Disable Slot. The xHC no longer owns
    /// the slot, so its DCBAA entry is cleared before any later Enable Slot.
    #[cfg(feature = "usb-hid-reattach-core")]
    fn retire(&mut self, ports: &mut Ports<2>, hid: &Hid) -> Result<(), Error> {
        ports.detach(hid.index).map_err(|_| bad(41))?;
        write32(self.pages[DCBAA], usize::from(hid.slot) * 2, &[0, 0]);
        fence(Ordering::SeqCst);
        Ok(())
    }

    /// While Detached, only a connect on `port` with PED=0 may start attach.
    #[cfg(any(feature = "usb-hid-reattach-core", feature = "native-usb-hid-refusal"))]
    fn await_connect(&mut self, ports: &Ports<2>, index: usize, port: u8) -> Result<(), Error> {
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            if event.kind() != 34 {
                return Err(event.rejected());
            }
            let changed = self.port_event(event)?;
            let state = self.controller.read(self.port_offset(changed)?);
            if changed != port
                || state & 1 == 0
                || state & 2 != 0
                || ports.phase(index) != Ok(Phase::Detached)
            {
                return Err(bad(42));
            }
            return Ok(());
        }
        Err(Error::Timeout)
    }

    /// QEMU makes a USB HID device the host input target only once the xHC
    /// first polls its interrupt endpoint, so READY waits until MFINDEX
    /// (xHCI 1.2c §5.5.1: 14 bits of 125 µs microframes) has advanced by two of
    /// this endpoint's periods plus one frame. Bounded and read-only.
    #[cfg(feature = "native-usb-hid-refusal")]
    fn settle(&self, hid: &Hid) -> Result<(), Error> {
        let target = 16 * u32::from(hid.session.binding().interval_ms) + 8;
        let mfindex = self.controller.caps.runtime_offset;
        let mut last = self.controller.read(mfindex) & 0x3fff;
        let mut elapsed = 0;
        for _ in 0..POLLS {
            let now = self.controller.read(mfindex) & 0x3fff;
            elapsed += now.wrapping_sub(last) & 0x3fff;
            last = now;
            if elapsed >= target {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout)
    }

    /// Refusal profile, after the port-1 refusal and the port-2 mouse attach:
    /// the mouse keeps delivering reports; the refused device's unplug moves
    /// only port 1 from Failed to Detached; a later in-profile keyboard on port
    /// 1 enumerates as a new generation into `output`/`report` (the refused
    /// slot was disabled and its DCBAA entry cleared) and delivers one key.
    #[cfg(feature = "native-usb-hid-refusal")]
    fn refusal(
        &mut self,
        ports: &mut Ports<2>,
        mouse: &mut Hid,
        output: u64,
        report: Buffer,
    ) -> Result<(), Error> {
        if mouse.session.binding().protocol != Protocol::Mouse || mouse.port != 2 {
            return Err(bad(57));
        }
        self.submit(mouse)?;
        self.settle(mouse)?;
        serial::write(b"USB:REFUSE:READY\n");
        let mut phase = 0;
        let mut unplugged = false;
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            if event.kind() == 34 {
                // Only the refused device's unplug, after the mouse sequence.
                let port = self.port_event(event)?;
                let connected = self.controller.read(self.port_offset(port)?) & 1 != 0;
                if phase != 2 || port != 1 || connected || ports.phase(0) != Ok(Phase::Failed) {
                    return Err(bad(58));
                }
                ports.detach(0).map_err(|_| bad(58))?;
                serial::write(b"USB:REFUSE:UNPLUG:PASS\n");
                unplugged = true;
                break;
            }
            if event.slot() != mouse.slot {
                return Err(event.rejected());
            }
            let Changes::Mouse(change) = self.complete(mouse, event)? else {
                return Err(bad(59));
            };
            if change != MouseChanges::default() {
                let down = MouseChanges {
                    dx: 7,
                    dy: -3,
                    buttons_pressed: 1,
                    buttons_released: 0,
                };
                let up = MouseChanges {
                    buttons_released: 1,
                    ..Default::default()
                };
                match phase {
                    0 if change == down => {
                        phase = 1;
                        serial::write(b"USB:REFUSE:MOUSE_DOWN\n");
                    }
                    1 if change == up => {
                        phase = 2;
                        serial::write(b"USB:REFUSE:MOUSE_UP\n");
                    }
                    _ => return Err(bad(59)),
                }
            }
            self.submit(mouse)?;
        }
        if !unplugged {
            return Err(Error::Timeout);
        }
        self.await_connect(ports, 0, 1)?;
        // xHCI §4.3.3: the refused device's slot and EP0 contexts must not
        // survive into the next Address Device.
        // `output` is an IOVA; clear the physical frame behind it.
        let page = self
            .io
            .iter()
            .position(|&io| io == output)
            .ok_or_else(|| bad(63))?;
        clear(self.pages[page]);
        let (mut keyboard, _) = self.attach(ports, 0, 1, output, report, None)?;
        if keyboard.session.binding().protocol != Protocol::Keyboard
            || ports.phase(0) != Ok(Phase::Ready)
            || keyboard.slot == mouse.slot
        {
            return Err(bad(60));
        }
        serial::write(b"USB:REFUSE:RECOVER:KEYBOARD\n");
        self.submit(&mut keyboard)?;
        self.settle(&keyboard)?;
        serial::write(b"USB:REFUSE:RECOVER:READY\n");
        let mut pressed = false;
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            // The mouse stays idle and no further port change is admitted.
            if event.kind() == 34 || event.slot() != keyboard.slot {
                return Err(event.rejected());
            }
            let Changes::Keyboard(keys) = self.complete(&mut keyboard, event)? else {
                return Err(bad(61));
            };
            if keys.status.is_some() || keys.modifiers_pressed != 0 || keys.modifiers_released != 0
            {
                return Err(bad(61));
            }
            if !keys.pressed().is_empty() || !keys.released().is_empty() {
                if !pressed && keys.pressed() == [4] && keys.released().is_empty() {
                    pressed = true;
                    serial::write(b"USB:REFUSE:RECOVER:KEY_DOWN\n");
                } else if pressed && keys.released() == [4] && keys.pressed().is_empty() {
                    serial::write(b"USB:REFUSE:RECOVER:KEY_UP\n");
                    return Ok(());
                } else {
                    return Err(bad(61));
                }
            }
            self.submit(&mut keyboard)?;
        }
        Err(Error::Timeout)
    }

    /// Mouse profile, after `reports()`: wait for the generation-1 left press,
    /// then for the port-2 unplug while it is held. Detach must release exactly
    /// that button. The mouse is not retired here; `reattach()` owns that.
    #[cfg(feature = "native-usb-hid-mouse-reattach")]
    fn hold_and_unplug_mouse(&mut self, mouse: &mut Hid) -> Result<Retired, Error> {
        let mut held = false;
        let mut captured = None;
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            if event.kind() == 34 {
                let port = self.port_event(event)?;
                let connected = self.controller.read(self.port_offset(port)?) & 1 != 0;
                if !held || port != mouse.port || connected {
                    return Err(bad(52));
                }
                let pending = self.disconnect(mouse, &[], 1)?;
                Self::deny_stale(mouse, pending)?;
                return Ok(Retired {
                    pending,
                    event: captured,
                });
            }
            if event.slot() != mouse.slot {
                return Err(bad(54));
            }
            let Changes::Mouse(change) = self.complete(mouse, event)? else {
                return Err(bad(53));
            };
            captured = Some(event);
            if change != MouseChanges::default() {
                let press = MouseChanges {
                    buttons_pressed: 1,
                    ..Default::default()
                };
                if held || change != press {
                    return Err(bad(53));
                }
                held = true;
                serial::write(b"USB:REATTACH:HELD_BUTTON\n");
            }
            self.submit(mouse)?;
        }
        Err(Error::Timeout)
    }

    /// Same-port reattach of the `TARGET` device after its generation-1
    /// unplug. `old` is the detached generation-1 device; `spare` is the
    /// (page, IOVA) pair of the generation-2 page granted before domain
    /// establishment. Each `replay` entry is a real generation-1 completion and
    /// the slot it came from; `stale` lists generation-1 Session tokens beyond
    /// `retired`'s own, all of which the generation-2 Session must refuse.
    #[cfg(feature = "usb-hid-reattach-core")]
    #[allow(clippy::too_many_arguments)]
    fn reattach(
        &mut self,
        ports: &mut Ports<2>,
        old_ticket: Ticket,
        old: &Hid,
        retired: Retired,
        replay: &[(Event, u8)],
        stale: &[Token],
        spare: (u64, u64),
    ) -> Result<(), Error> {
        if old.index != TARGET || old.session.binding().protocol != TARGET_PROTOCOL {
            return Err(bad(43));
        }
        self.retire(ports, old)?;
        fill(self.pages[OUTPUT_CONTEXT + old.index], RETIRED);
        fill(old.report.page, RETIRED);
        serial::write(b"USB:REATTACH:DETACHED\n");
        self.await_connect(ports, old.index, old.port)?;
        let (page, io) = spare;
        let report = Buffer {
            page,
            offset: SPARE_REPORT,
            io: io + SPARE_REPORT as u64,
        };
        let (mut hid, ticket) =
            self.attach(ports, old.index, old.port, io, report, Some(old_ticket))?;
        let (before, after) = (old_ticket.lifetime(), ticket.lifetime());
        if hid.session.binding().protocol != TARGET_PROTOCOL
            || after.device != before.device
            || before.generation.checked_add(1) != Some(after.generation)
        {
            return Err(bad(44));
        }
        serial::write(match TARGET_PROTOCOL {
            Protocol::Keyboard => b"USB:REATTACH:ENUM:KEYBOARD\n".as_slice(),
            Protocol::Mouse => b"USB:REATTACH:ENUM:MOUSE\n".as_slice(),
        });
        #[cfg(not(feature = "native-usb-hid-mouse-reattach"))]
        serial::write(if hid.slot == old.slot {
            b"USB:REATTACH:SLOT_REUSED\n".as_slice()
        } else {
            b"USB:REATTACH:SLOT_NEW\n".as_slice()
        });
        // The mouse profile's replay lists the generation-1 keyboard first.
        #[cfg(feature = "native-usb-hid-mouse-reattach")]
        serial::write(
            match replay.iter().position(|&(_, source)| source == hid.slot) {
                Some(0) => b"USB:REATTACH:SLOT_REUSED:KEYBOARD\n".as_slice(),
                Some(_) => b"USB:REATTACH:SLOT_REUSED:MOUSE\n".as_slice(),
                None => b"USB:REATTACH:SLOT_NEW\n".as_slice(),
            },
        );
        self.submit(&mut hid)?;
        let pending = hid.pending.ok_or_else(|| bad(45))?;
        // Replayed captured completions, not hardware-generated events: each
        // real generation-1 Transfer Event must not match generation-2 work.
        // The source whose slot was reused must pass every other accepts()
        // field once its TRB address is substituted, so only the never-reused
        // TRB address rejects it. Generation-1 tokens and the ticket are
        // rejected without consuming the generation-2 pending transfer, which
        // must later accept its real event.
        let replayed = replay.iter().all(|&(event, source)| {
            event.slot() == source
                && !Self::accepts(&hid, pending, event)
                && (source != hid.slot
                    || Self::accepts(
                        &hid,
                        Pending {
                            pointer: event.parameter,
                            ..pending
                        },
                        event,
                    ))
        });
        let refused = core::iter::once(&retired.pending.token)
            .chain(stale)
            .all(|&token| {
                hid.session
                    .complete_input(token, CompletionStatus::Success, &[0; REPORT_BYTES])
                    == Err(SessionError::UnexpectedCompletion)
            });
        if !replayed
            || !refused
            || ports.complete(old_ticket, true) != Err(PortError::Stale)
            || hid.pending != Some(pending)
        {
            return Err(bad(45));
        }
        serial::write(b"USB:REATTACH:STALE_DENIED\nUSB:REATTACH:READY\n");
        let mut phase = 0;
        for _ in 0..INPUT_POLLS {
            let Some(event) = self.poll()? else {
                core::hint::spin_loop();
                continue;
            };
            if event.kind() == 34 {
                let port = self.port_event(event)?;
                let state = self.controller.read(self.port_offset(port)?);
                match (phase, state & 1 != 0) {
                    (2, false) if port == hid.port => {
                        let detached = self.disconnect(&mut hid, &[], 0)?;
                        Self::deny_stale(&mut hid, detached)?;
                        self.retire(ports, &hid)?;
                        phase = 3;
                        serial::write(b"USB:REATTACH:UNPLUG:PASS\n");
                    }
                    (3, true) if port == hid.port => {
                        // The instance attach budget refuses a third identity:
                        // no port reset, no Enable Slot, and the port stays PED=0.
                        if ports.phase(hid.index) != Ok(Phase::Detached)
                            || ports.attach(hid.index) != Err(PortError::Exhausted)
                            || state & 2 != 0
                        {
                            return Err(bad(46));
                        }
                        // Bounded observation: no further event arrives and the
                        // refused port stays connected but unenabled. It also
                        // lets QEMU answer the device_add before the guest exits.
                        for _ in 0..QUIET_POLLS {
                            if let Some(event) = self.poll()? {
                                return Err(event.rejected());
                            }
                            core::hint::spin_loop();
                        }
                        if self.controller.read(self.port_offset(port)?) & 3 != 1 {
                            return Err(bad(46));
                        }
                        serial::write(b"USB:ATTACH:BUDGET_DENIED\n");
                        return Ok(());
                    }
                    _ => return Err(bad(47)),
                }
                continue;
            }
            if phase >= 3 || event.slot() != hid.slot {
                return Err(event.rejected());
            }
            let changes = self.complete(&mut hid, event)?;
            #[cfg(not(feature = "native-usb-hid-mouse-reattach"))]
            {
                let Changes::Keyboard(keys) = changes else {
                    return Err(bad(48));
                };
                if keys.status.is_some()
                    || keys.modifiers_pressed != 0
                    || keys.modifiers_released != 0
                {
                    return Err(bad(48));
                }
                if !keys.pressed().is_empty() || !keys.released().is_empty() {
                    match phase {
                        0 if keys.pressed() == [6] && keys.released().is_empty() => {
                            phase = 1;
                            serial::write(b"USB:REATTACH:KEY_DOWN\n");
                        }
                        1 if keys.released() == [6] && keys.pressed().is_empty() => {
                            phase = 2;
                            serial::write(b"USB:REATTACH:KEY_UP\n");
                        }
                        _ => return Err(bad(49)),
                    }
                }
            }
            #[cfg(feature = "native-usb-hid-mouse-reattach")]
            {
                let Changes::Mouse(change) = changes else {
                    return Err(bad(55));
                };
                let down = MouseChanges {
                    dx: -4,
                    dy: 6,
                    buttons_pressed: 1,
                    buttons_released: 0,
                };
                let up = MouseChanges {
                    buttons_released: 1,
                    ..Default::default()
                };
                if change != MouseChanges::default() {
                    match phase {
                        0 if change == down => {
                            phase = 1;
                            serial::write(b"USB:REATTACH:MOUSE_DOWN\n");
                        }
                        1 if change == up => {
                            phase = 2;
                            serial::write(b"USB:REATTACH:MOUSE_UP\n");
                        }
                        _ => return Err(bad(55)),
                    }
                }
            }
            self.submit(&mut hid)?;
        }
        Err(Error::Timeout)
    }
}

fn announce(hid: &Hid) {
    match hid.session.binding().protocol {
        Protocol::Keyboard => serial::write(b"USB:ENUM:KEYBOARD\n"),
        Protocol::Mouse => serial::write(b"USB:ENUM:MOUSE\n"),
    }
}

#[derive(Clone, Copy, PartialEq)]
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
    report: Buffer,
}

/// Generation-1 device evidence retained after its detach.
#[cfg_attr(not(feature = "usb-hid-reattach-core"), allow(dead_code))]
#[derive(Clone, Copy)]
struct Retired {
    /// Submitted work whose token belongs to the detached session.
    pending: Pending,
    /// The last real completion the controller published for that device.
    event: Option<Event>,
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
            #[cfg(feature = "usb-hid-reattach-core")]
            SPARE => device::DmaDirection::DeviceReadsWrites,
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
        // Device identities are the two fixed root ports. Three attaches last
        // for this whole instance; each attach gets one enumeration attempt.
        let mut ports = Ports::<2>::new([1, 2], 3, 1).map_err(|_| bad(50))?;
        let report = |index: usize| Buffer {
            page: pages[REPORT + index],
            offset: 0,
            io: io[REPORT + index],
        };
        #[cfg(feature = "native-usb-hid-refusal")]
        {
            // H02-T5: port 1 holds an out-of-profile device; only its refusal
            // is accepted. One attempt per attach: the refused port stays
            // Failed and cannot be re-enumerated until that device is unplugged.
            match engine.attach(&mut ports, 0, 1, io[OUTPUT_CONTEXT], report(0), None) {
                Err(Error::Refused) => {}
                Ok(_) => return Err(bad(56)),
                Err(error) => return Err(error),
            }
            if ports.phase(0) != Ok(Phase::Failed) || ports.attach(0) != Err(PortError::Busy) {
                return Err(bad(56));
            }
            serial::write(b"USB:REFUSE:PORT_HELD\n");
            let (mut mouse, _) =
                engine.attach(&mut ports, 1, 2, io[OUTPUT_CONTEXT + 1], report(1), None)?;
            announce(&mouse);
            engine.refusal(&mut ports, &mut mouse, io[OUTPUT_CONTEXT], report(0))?;
        }
        #[cfg(not(feature = "native-usb-hid-refusal"))]
        {
            let (mut keyboard, keyboard_ticket) =
                engine.attach(&mut ports, 0, 1, io[OUTPUT_CONTEXT], report(0), None)?;
            announce(&keyboard);
            let (mut mouse, mouse_ticket) =
                engine.attach(&mut ports, 1, 2, io[OUTPUT_CONTEXT + 1], report(1), None)?;
            announce(&mouse);
            serial::write(b"USB:PROTOCOL:PASS\n");
            let retired = engine.reports(&mut keyboard, &mut mouse)?;
            #[cfg(feature = "native-usb-hid-reattach")]
            {
                let _ = mouse_ticket;
                let captured = retired.event.ok_or_else(|| bad(43))?;
                engine.reattach(
                    &mut ports,
                    keyboard_ticket,
                    &keyboard,
                    retired,
                    &[(captured, keyboard.slot)],
                    &[],
                    (pages[SPARE], io[SPARE]),
                )?;
            }
            #[cfg(feature = "native-usb-hid-mouse-reattach")]
            {
                let _ = keyboard_ticket;
                let keyboard_event = retired.event.ok_or_else(|| bad(43))?;
                // QEMU may hand the keyboard's slot to a later Enable Slot, so its
                // DCBAA entry is cleared and its pages filled before any attach.
                engine.retire(&mut ports, &keyboard)?;
                fill(pages[OUTPUT_CONTEXT], RETIRED);
                fill(pages[REPORT], RETIRED);
                let mouse_retired = engine.hold_and_unplug_mouse(&mut mouse)?;
                let mouse_event = mouse_retired.event.ok_or_else(|| bad(43))?;
                engine.reattach(
                    &mut ports,
                    mouse_ticket,
                    &mouse,
                    mouse_retired,
                    &[(keyboard_event, keyboard.slot), (mouse_event, mouse.slot)],
                    &[retired.pending.token],
                    (pages[SPARE], io[SPARE]),
                )?;
            }
            #[cfg(not(feature = "usb-hid-reattach-core"))]
            let _ = (keyboard_ticket, mouse_ticket, retired);
        }
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
    // Detection only: the retired generation-1 output context and report pages
    // stayed mapped in the fixed domain until bus mastering was cleared.
    #[cfg(feature = "usb-hid-reattach-core")]
    {
        let retired: &[usize] = if TARGET == 1 {
            &[OUTPUT_CONTEXT, REPORT, OUTPUT_CONTEXT + 1, REPORT + 1]
        } else {
            &[OUTPUT_CONTEXT, REPORT]
        };
        if retired.iter().any(|&page| !filled(pages[page], RETIRED)) {
            return Err(Error::Guard);
        }
        serial::write(b"USB:RETIRED:UNCHANGED\n");
    }
    if !guard_unchanged(pages[GUARD]) {
        return Err(Error::Guard);
    }
    serial::write(b"USB:GUARD:UNCHANGED\nUSB:QUIESCE:PASS\nUSB:PAGES:RETAINED\nUSB:PASS\n");
    Ok(())
}
