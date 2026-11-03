//! Sole manifest-authorized display transport. No GUI/content interpretation.
//! One CPU, IF=0, immutable supervisor-only framebuffer mapping.
use super::*;
use nieuw_os_display_client::wire::{self, Blit, Geometry, SIZE};

struct Runtime {
    prepared: bool,
    owner: Option<(u64, u64, u64)>,
    next_epoch: u64,
    blits: u64,
    pixels: u64,
    denied: u64,
}
static mut DISPLAY: Runtime = Runtime {
    prepared: false,
    owner: None,
    next_epoch: 1,
    blits: 0,
    pixels: 0,
    denied: 0,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU/IF0; every borrow ends before return/scheduling, and
    // no user reference or user pointer is retained in this private state.
    unsafe { &mut *ptr::addr_of_mut!(DISPLAY) }
}
/// Called once after framebuffer capture, validation and mapping.
pub fn prepare() {
    let f = crate::framebuffer::descriptor();
    assert!(f.validate().is_ok());
    runtime().prepared = true;
}
/// Read-only preflight for the fixed window-group restart. A fresh owner must
/// negotiate a fresh geometry epoch; reject exhaustion before any teardown.
#[cfg(feature = "native-window-session")]
pub(super) fn can_bind_fresh() -> bool {
    let r = runtime();
    r.prepared && r.next_epoch < u64::MAX
}
/// PID death/full principal retirement only; not a late single-grant close.
pub fn detach(pid: u64) {
    let r = runtime();
    if r.owner.is_some_and(|o| o.0 == pid) {
        r.owner = None;
    }
}
#[cfg(any(
    feature = "native-display-probe",
    feature = "native-system-materiaal-probe"
))]
pub fn counts() -> [u64; 3] {
    let r = runtime();
    [r.blits, r.pixels, r.denied]
}

fn call(s: &mut State, slot: usize, number: u64, a: [u64; 6]) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    let key = m4::engine()
        .check_display(pid, a[0])
        .map_err(|e| e.code())?;
    if !runtime().prepared {
        return Err(-14);
    }
    match number {
        0x80 => {
            if a[2] != SIZE as u64 || a[3..].iter().any(|n| *n != 0) {
                return Err(-1);
            }
            let output = m4::plan(s, slot, a[1], SIZE as u64, true).map_err(|_| -3)?;
            let r = runtime();
            let epoch = if let Some((_, _, epoch)) = r.owner.filter(|o| o.0 == pid && o.1 == key) {
                epoch
            } else {
                let epoch = r.next_epoch;
                r.next_epoch = epoch.checked_add(1).ok_or(-7)?;
                r.owner = Some((pid, key, epoch));
                epoch
            };
            let f = crate::framebuffer::descriptor();
            let bytes = Geometry {
                width: f.width,
                height: f.height,
                epoch,
            }
            .encode()
            .map_err(|_| -14)?;
            let mut out = [0; 320];
            out[..SIZE].copy_from_slice(&bytes);
            m4::write_plan(&output, &out);
            Ok(SIZE as u64)
        }
        0x81 => {
            if a[4] != 0 || a[5] != 0 || a[3] == 0 || a[3] > wire::MAX_BYTES as u64 {
                return Err(-1);
            }
            let header = m4::plan(s, slot, a[1], SIZE as u64, false).map_err(|_| -3)?;
            let snapshot = m4::read_plan(&header);
            let mut bytes = [0; SIZE];
            bytes.copy_from_slice(&snapshot[..SIZE]);
            let blit = Blit::decode(&bytes).map_err(|_| -1)?;
            let epoch = runtime()
                .owner
                .filter(|o| o.0 == pid && o.1 == key)
                .map(|o| o.2)
                .ok_or(-5)?;
            let f = crate::framebuffer::descriptor();
            blit.validate(
                Geometry {
                    width: f.width,
                    height: f.height,
                    epoch,
                },
                a[3] as usize,
            )
            .map_err(|_| -1)?;
            // Validate every complete input range before copying or touching
            // device memory. This keeps the existing 320-byte copy policy.
            let mut plans = [None; 4];
            let mut at = 0usize;
            for plan in &mut plans {
                if at == a[3] as usize {
                    break;
                }
                let n = (a[3] as usize - at).min(320);
                *plan = Some(
                    m4::plan(
                        s,
                        slot,
                        a[2].checked_add(at as u64).ok_or(-3)?,
                        n as u64,
                        false,
                    )
                    .map_err(|_| -3)?,
                );
                at += n;
            }
            if at != a[3] as usize {
                return Err(-1);
            }
            let mut payload = [0; wire::MAX_BYTES];
            at = 0;
            for plan in plans.iter().flatten() {
                let bytes = m4::read_plan(plan);
                let n = (a[3] as usize - at).min(320);
                payload[at..at + n].copy_from_slice(&bytes[..n]);
                at += n;
            }
            wire::validate_pixels(&payload[..at]).map_err(|_| -1)?;
            let rect = blit.rectangle;
            let width = usize::from(rect.width);
            let height = usize::from(rect.height);
            let mut rows = [[0u32; wire::TILE_WIDTH as usize]; wire::TILE_HEIGHT as usize];
            for (y, row) in rows.iter_mut().enumerate().take(height) {
                for (x, pixel) in row.iter_mut().enumerate().take(width) {
                    let at = (y * width + x) * 4;
                    *pixel = u32::from_le_bytes(payload[at..at + 4].try_into().unwrap());
                }
                crate::framebuffer::validate_pixels(rect.x, rect.y + y as u32, &row[..width])
                    .map_err(|_| -1)?;
            }
            // No user/mapping/metadata mutation can intervene between the full
            // tile preflight and these synchronous stores. No partial success
            // is reported; no user pointer is dereferenced by the backend.
            for (y, row) in rows.iter().enumerate().take(height) {
                crate::framebuffer::write_pixels(rect.x, rect.y + y as u32, &row[..width])
                    .map_err(|_| -14)?;
            }
            let r = runtime();
            r.blits = r.blits.saturating_add(1);
            r.pixels = r.pixels.saturating_add((width * height) as u64);
            Ok((width * height) as u64)
        }
        _ => Err(-2),
    }
}
pub fn syscall(s: &mut State, slot: usize, number: u64, args: [u64; 6]) -> (i64, u64) {
    match call(s, slot, number, args) {
        Ok(value) => (0, value),
        Err(error) => {
            let r = runtime();
            r.denied = r.denied.saturating_add(1);
            (error, 0)
        }
    }
}
