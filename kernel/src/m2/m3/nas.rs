//! Local NAS acceptance manifest, not a network/E2EE or release manifest.
use super::*;
use capability_core::{
    BlockScope, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, PROCESS_EXITED, RECV, SEND,
};

const ADMIN: usize = 0;
const NAS: usize = 1;
const VOLUME: usize = 2;
const GUEST: usize = 3;
const RECOVER: bool = cfg!(feature = "test-system-nas-recover");
const WORDS: [u64; 8] = [40, 240, 400, 408, 416, 424, 448, 192];

/// Exact boot-1 token replay (#110): private test handoff, never a product
/// interface. Token bytes leave the normal VM only on a dedicated
/// `isa-debugcon` port that the runner binds to a private 0600 file outside
/// the checkout, and enter the cold VM only as a QEMU `fw_cfg` file that the
/// runner creates read-only from the validated handoff. COM1 serial, reports
/// and CI artifacts never carry token bytes. No process gets port I/O or a new
/// capability; only this kernel acceptance harness touches these ports.
mod replay {
    use core::arch::asm;

    /// Guest startup bytes 256..336: exact session (40) and upload (40).
    pub const OFFSET: u64 = 256;
    pub const BYTES: usize = 80;
    /// Private output frame: magic, then exactly 80 payload bytes.
    pub const MAGIC: &[u8; 8] = b"NOSRPL01";
    /// Dedicated debugcon port; firmware has no device or driver there.
    const OUT_PORT: u16 = 0x4f8;
    const FW_CFG_SELECTOR: u16 = 0x510;
    const FW_CFG_DATA: u16 = 0x511;
    const FW_CFG_SIGNATURE: u16 = 0x0000;
    const FW_CFG_FILE_DIR: u16 = 0x0019;
    /// QEMU `-fw_cfg name=...`; the guest kernel only reads it.
    pub const FW_CFG_NAME: &[u8] = b"opt/nieuw-os/nas-replay";

    fn out8(port: u16, value: u8) {
        // SAFETY: CPL0 acceptance kernel, KERNEL held with IF=0. The debugcon
        // port is a write-only byte sink; no memory is accessed.
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
        }
    }
    fn out16(port: u16, value: u16) {
        // SAFETY: Same contract; selects a QEMU fw_cfg item, no memory access.
        unsafe {
            asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
        }
    }
    fn in8(port: u16) -> u8 {
        let value: u8;
        // SAFETY: Same contract; reads one fw_cfg data byte, no memory access.
        unsafe {
            asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
        }
        value
    }
    fn read_into(out: &mut [u8]) {
        for byte in out {
            *byte = in8(FW_CFG_DATA);
        }
    }

    /// Publish the exact pair on the private channel.
    pub fn publish(payload: &[u8; BYTES]) {
        for byte in MAGIC.iter().chain(payload.iter()) {
            out8(OUT_PORT, *byte);
        }
    }

    /// Locate the replay file. `Ok(None)`: absent. `Err`: no QEMU fw_cfg or a
    /// malformed directory. Sizes other than exactly 80 bytes are returned so
    /// the caller can reject them.
    fn find() -> Result<Option<(u16, u32)>, ()> {
        out16(FW_CFG_SELECTOR, FW_CFG_SIGNATURE);
        let mut signature = [0; 4];
        read_into(&mut signature);
        if &signature != b"QEMU" {
            return Err(());
        }
        out16(FW_CFG_SELECTOR, FW_CFG_FILE_DIR);
        let mut count = [0; 4];
        read_into(&mut count);
        let count = u32::from_be_bytes(count);
        if count > 4096 {
            return Err(());
        }
        let mut found = None;
        for _ in 0..count {
            let mut entry = [0; 64];
            read_into(&mut entry);
            let size = u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let select = u16::from_be_bytes([entry[4], entry[5]]);
            let name = &entry[8..];
            let length = name.iter().position(|b| *b == 0).ok_or(())?;
            if &name[..length] == FW_CFG_NAME {
                if found.is_some() {
                    return Err(());
                }
                found = Some((select, size));
            }
        }
        Ok(found)
    }

    /// Present: exactly 80 bytes read. Absent: `Ok(None)`.
    pub fn read() -> Result<Option<[u8; BYTES]>, ()> {
        let Some((select, size)) = find()? else {
            return Ok(None);
        };
        if size as usize != BYTES {
            return Err(());
        }
        out16(FW_CFG_SELECTOR, select);
        let mut payload = [0; BYTES];
        read_into(&mut payload);
        Ok(Some(payload))
    }
}

struct Evidence {
    roles: [Option<[u64; 8]>; 4],
    kernel_tables: usize,
}
static mut EVIDENCE: Evidence = Evidence {
    roles: [None; 4],
    kernel_tables: 0,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, interrupts disabled; borrow ends before scheduling.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"NAS:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason);
    }
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
pub(super) fn image_elf(_: u32) -> Option<&'static [u8]> {
    None
}
fn channel(e: &mut capability_core::Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e.create_endpoint(owner).unwrap();
    let receive = e.derive(owner, root, RECV).unwrap();
    let send = e.bootstrap_delegate(owner, root, sender, SEND).unwrap();
    e.close(owner, root).unwrap();
    (receive, send)
}
pub(super) fn manifest(s: &mut State) {
    require(
        s.amount == MAX
            && s.processes[..4].iter().all(|p| p.status != Status::Dead)
            && s.processes[4..].iter().all(|p| p.status == Status::Dead),
        b"MANIFEST_STATE",
    );
    let [admin, nas, volume, guest] = [
        s.processes[ADMIN].pid,
        s.processes[NAS].pid,
        s.processes[VOLUME].pid,
        s.processes[GUEST].pid,
    ];
    let e = m4::engine();
    for pid in [admin, nas, volume, guest] {
        e.register(pid).unwrap();
    }
    let (admin_ingress, admin_send) = channel(e, nas, admin);
    let (guest_ingress, guest_send) = channel(e, nas, guest);
    let (admin_reply, nas_admin_reply) = channel(e, admin, nas);
    let (guest_reply, nas_guest_reply) = channel(e, guest, nas);
    let (admin_coord, guest_coord_send) = channel(e, admin, guest);
    let (guest_coord, admin_coord_send) = channel(e, guest, admin);
    let (volume_ingress, nas_volume) = channel(e, volume, nas);
    let (volume_reply, volume_reply_send) = channel(e, nas, volume);
    let (volume_other, unused_send) = channel(e, volume, admin);
    e.close(admin, unused_send).unwrap();
    let (unused_reply, volume_other_reply) = channel(e, admin, volume);
    e.close(admin, unused_reply).unwrap();
    let entropy = e.create_entropy(nas).unwrap();
    let block = e
        .create_block_device_with_rights(
            volume,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 580,
            },
            if RECOVER {
                BLOCK_READ
            } else {
                BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH
            },
        )
        .unwrap();
    for (offset, value) in [
        (32, 1),
        (48, entropy),
        (56, admin_ingress),
        (64, guest_ingress),
        (72, nas_admin_reply),
        (80, nas_guest_reply),
        (88, nas_volume),
        (96, volume_reply),
        (112, u64::from(RECOVER)),
        (120, 1),
        (160, admin),
        (168, guest),
        (176, nas),
        (184, volume),
    ] {
        put(s, NAS, offset, value);
    }
    for (slot, role, send, reply, coord_send, coord_recv, peer) in [
        (
            ADMIN,
            0,
            admin_send,
            admin_reply,
            admin_coord_send,
            admin_coord,
            guest,
        ),
        (
            GUEST,
            3,
            guest_send,
            guest_reply,
            guest_coord_send,
            guest_coord,
            admin,
        ),
    ] {
        for (offset, value) in [
            (32, 1),
            (56, send),
            (64, reply),
            (72, coord_send),
            (80, coord_recv),
            (112, u64::from(RECOVER)),
            (120, role),
            (160, nas),
            (168, peer),
            (176, s.processes[slot].pid),
        ] {
            put(s, slot, offset, value);
        }
        for handle in [send, reply, coord_send, coord_recv] {
            let pid = s.processes[slot].pid;
            require(
                e.check_entropy(pid, handle).is_err()
                    && e.check_block(pid, handle, BLOCK_READ).is_err()
                    && e.check_launch(pid, handle).is_err()
                    && e.check_network(pid, handle).is_err(),
                b"CLIENT_AUTHORITY",
            );
        }
    }
    // #110: the cold guest receives the exact boot-1 pair; a normal guest
    // must start without one, so a normal run can never be fed tokens.
    match (RECOVER, replay::read()) {
        (true, Ok(Some(payload))) => {
            require(payload.iter().any(|b| *b != 0), b"REPLAY_INPUT_ZERO");
            for (index, chunk) in payload.chunks_exact(8).enumerate() {
                let mut word = [0; 8];
                word.copy_from_slice(chunk);
                put(
                    s,
                    GUEST,
                    replay::OFFSET + index as u64 * 8,
                    u64::from_le_bytes(word),
                );
            }
            serial::write(b"NAS:REPLAY_INPUT:PASS\n");
        }
        (false, Ok(None)) => {}
        (true, _) => fail(b"REPLAY_INPUT"),
        (false, _) => fail(b"REPLAY_INPUT_IN_NORMAL"),
    }
    for (offset, value) in [
        (32, 1),
        (48, block),
        (56, volume_ingress),
        (80, volume_other),
        (88, volume_reply_send),
        (72, volume_other_reply),
        (120, u64::from(RECOVER)),
        (160, nas),
        (168, admin),
        (176, volume),
    ] {
        put(s, VOLUME, offset, value);
    }
    require(e.check_invariants(), b"MANIFEST_ENGINE");
    let releasable: usize = s.processes[..4]
        .iter()
        .map(|p| p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count())
        .sum();
    s.free_baseline = s.free_count().checked_add(releasable).unwrap();
    evidence().kernel_tables = s.kernel.owned_table_count();
    print_number(b"NAS:ADMIN_PID:", admin);
    print_number(b"NAS:GUEST_PID:", guest);
    serial::write(b"NAS:MANIFEST:PASS\n");
}
pub(super) fn observe_exit(s: &mut State, slot: usize, _: u64) {
    require([ADMIN, NAS, GUEST].contains(&slot), b"UNEXPECTED_EXIT");
    let words = WORDS.map(|offset| word(s, slot, offset));
    if s.last_event.code != 0 || words[0] != 1 {
        print_number(b"NAS:ERROR:SLOT:", slot as u64);
        print_number(b"NAS:ERROR:STEP:", words[1]);
        fail(b"ROLE_RESULT");
    }
    require(evidence().roles[slot].is_none(), b"DUPLICATE_EXIT");
    if slot == GUEST && !RECOVER {
        // Snapshot before process cleanup: the guest's own published pair.
        let mut payload = [0; replay::BYTES];
        for (index, chunk) in payload.chunks_exact_mut(8).enumerate() {
            chunk.copy_from_slice(&word(s, slot, replay::OFFSET + index as u64 * 8).to_le_bytes());
        }
        require(
            payload[..40].iter().any(|b| *b != 0) && payload[40..].iter().any(|b| *b != 0),
            b"REPLAY_EXPORT",
        );
        replay::publish(&payload);
        serial::write(b"NAS:REPLAY_EXPORT:PASS\n");
    }
    evidence().roles[slot] = Some(words);
    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
}
pub(super) fn observe_fault(_: &mut State, slot: usize, _: &[u64; 22], _: u64) {
    print_number(b"NAS:ERROR:SLOT:", slot as u64);
    fail(b"PROCESS_FAULT")
}
pub(super) fn poll(s: &mut State) -> Option<*const u64> {
    keyring::broker_tick(s);
    m5::broker_tick(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    if s.ticks >= 20_000 {
        for slot in 0..4 {
            if s.processes[slot].status != Status::Dead {
                print_number(b"NAS:TIMEOUT:SLOT:", slot as u64);
                print_number(b"NAS:TIMEOUT:STEP:", word(s, slot, 240));
            }
        }
        fail(b"TIMEOUT");
    }
    if [ADMIN, NAS, GUEST]
        .iter()
        .any(|slot| evidence().roles[*slot].is_none())
    {
        return None;
    }
    // Cold: the guest's exact-token COMMIT replay adds one Stale reply.
    let expected = if RECOVER {
        [(4, 0), (4, 5), (0, 5)]
    } else {
        [(6, 1), (12, 6), (6, 5)]
    };
    for (slot, (ok, errors)) in [ADMIN, NAS, GUEST].into_iter().zip(expected) {
        let words = evidence().roles[slot].unwrap();
        require(
            words[1] == 0
                && words[2] == ok
                && words[3] == errors
                && words[4] == u64::from(slot == GUEST)
                && words[5] == 0
                && (1..=65536).contains(&words[6]),
            b"ROLE_EVIDENCE",
        );
        print_number(b"NAS:ROLE_SLOT:", slot as u64);
        print_number(b"NAS:RPC_OK:", words[2]);
        print_number(b"NAS:RPC_ERRORS:", words[3]);
        print_number(b"NAS:RIGHTS_DENIED:", words[4]);
        print_number(b"NAS:STACK_TOUCHED:", words[6]);
    }
    require(evidence().roles[NAS].unwrap()[7] == 1, b"NAS_READY");
    require(
        s.processes[VOLUME].status != Status::Dead
            && word(s, VOLUME, 192) == 1
            && word(s, VOLUME, 376) == 0,
        b"VOLUME_STATE",
    );
    let (entropy, _, entropy_idle) = keyring::broker_summary();
    let (published, completed, block_idle) = m5::broker_summary();
    require(
        entropy > 0 && entropy_idle && block_idle && published == completed && published[0] > 0,
        b"BROKER_STATE",
    );
    require(
        if RECOVER {
            published[1..] == [0, 0]
        } else {
            published[1] > 0 && published[2] > 0
        },
        b"DISK_MODE",
    );
    for (label, value) in [
        (&b"NAS:ENTROPY_BYTES:"[..], entropy),
        (b"NAS:DISK_READS:", published[0]),
        (b"NAS:DISK_WRITES:", published[1]),
        (b"NAS:DISK_FLUSHES:", published[2]),
    ] {
        print_number(label, value);
    }
    serial::write(b"NAS:BROKERS:PASS\n");
    s.processes[VOLUME].status = Status::Dying;
    s.cleanup(VOLUME);
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.nodes == 0
            && stats.endpoints == 0
            && stats.queued_messages == 0
            && stats.block_objects == 0
            && stats.entropy_objects == 0
            && stats.launch_objects == 0
            && stats.process_objects == 0
            && stats.network_objects == 0
            && m4::engine().check_invariants(),
        b"ENGINE_CLEANUP",
    );
    let grown = s
        .kernel
        .owned_table_count()
        .checked_sub(evidence().kernel_tables)
        .unwrap();
    require(
        s.free_count().checked_add(grown) == Some(s.free_baseline),
        b"FRAME_CLEANUP",
    );
    serial::write(b"NAS:CLEANUP:PASS\n");
    serial::write(if RECOVER {
        b"NAS:CASE:RECOVER:PASS\n"
    } else {
        b"NAS:CASE:NORMAL:PASS\n"
    });
    serial::write(b"M1:PASS\n");
    serial::exit(0x10)
}
