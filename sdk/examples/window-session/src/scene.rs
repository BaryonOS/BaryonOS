//! Bounded compositor scene policy and damage tracking for the window probe.
//!
//! Coordinates and rectangles use physical pixels; app surfaces are copied
//! into compositor-owned backing before rendering.

use nieuw_os_graphics::{theme, Canvas, Font, Icon, Rect, Size, Weight};
use nieuw_os_surface_protocol::{Geometry, Rectangle};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

use crate::clipboard::Notice as ClipboardNotice;
use crate::shell::Shell;

pub const MAX_PIXELS: usize = 1_400_000;
pub const WINDOWS: usize = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub body: Rect,
    pub geometry: Geometry,
    pub offset: usize,
}
const APP_NAMES: [&str; WINDOWS] = ["Materiaal", "Rekenmachine"];
const QUERY_BYTES: usize = 32;
struct ProgramPalette {
    open: bool,
    query: [u8; QUERY_BYTES],
    length: usize,
    selected: Option<usize>,
}
impl ProgramPalette {
    const fn new() -> Self {
        Self {
            open: false,
            query: [0; QUERY_BYTES],
            length: 0,
            selected: None,
        }
    }
    fn matches(&self, index: usize) -> bool {
        let query = &self.query[..self.length];
        query.is_empty()
            || APP_NAMES[index]
                .as_bytes()
                .windows(query.len())
                .any(|part| part.eq_ignore_ascii_case(query))
    }
    fn first_match(&self) -> Option<usize> {
        (0..WINDOWS).find(|index| self.matches(*index))
    }
}
pub struct Scene {
    pub shell: Shell,
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub windows: [Window; WINDOWS],
    pub focused: usize,
    pub pointer: (u32, u32),
    drag: Option<(usize, i32, i32)>,
    pub moves: u64,
    pub switches: u64,
    clipboard_notice: ClipboardNotice,
    palette: ProgramPalette,
    palette_keys: [bool; PhysicalKey::Menu as usize + 1],
    switcher: Option<usize>,
    switcher_keys: [bool; PhysicalKey::Menu as usize + 1],
    switcher_mouse_capture: bool,
}
impl Scene {
    /// Construct the two-window scene for a supported physical-pixel viewport.
    ///
    /// `scale` must be 1 or 2, the logical viewport at least 800×500, the
    /// physical viewport no larger than 1920×1080, and both surface epochs
    /// nonzero. The backing store must fit within [`MAX_PIXELS`].
    pub fn new(width: u32, height: u32, scale: u32, epochs: [u64; 2]) -> Result<Self, Invalid> {
        if !matches!(scale, 1 | 2)
            || width > 1920
            || height > 1080
            || width / scale < 800
            || height / scale < 500
            || epochs.contains(&0)
        {
            return Err(Invalid);
        }
        let (w, h) = (width / scale, height / scale);
        let mw = (w - 400).clamp(480, 960);
        let bh = (h - 210).clamp(260, 620);
        let definitions = [(28, 114, mw, bh), (w - 348, 140, 320, bh - 26)];
        let mut offset = 0;
        let windows = core::array::from_fn(|i| {
            let (x, y, ww, hh) = definitions[i];
            let window = Window {
                body: Rect::new(
                    (x * scale) as i32,
                    (y * scale) as i32,
                    ww * scale,
                    hh * scale,
                ),
                geometry: Geometry {
                    width,
                    height,
                    epoch: epochs[i],
                    assigned: Rectangle {
                        x: 0,
                        y: 0,
                        width: (ww * scale) as u16,
                        height: (hh * scale) as u16,
                    },
                },
                offset,
            };
            offset += (ww * hh * scale * scale) as usize;
            window
        });
        if offset > MAX_PIXELS {
            return Err(Invalid);
        }
        for window in windows {
            window.geometry.validate().map_err(|_| Invalid)?;
        }
        Ok(Self {
            shell: Shell::new(false),
            width,
            height,
            scale,
            windows,
            focused: 0,
            pointer: (width / 2, height / 2),
            drag: None,
            moves: 0,
            switches: 0,
            clipboard_notice: ClipboardNotice::None,
            palette: ProgramPalette::new(),
            palette_keys: [false; PhysicalKey::Menu as usize + 1],
            switcher: None,
            switcher_keys: [false; PhysicalKey::Menu as usize + 1],
            switcher_mouse_capture: false,
        })
    }

    /// Return the title-bar frame for a valid window index.
    pub fn frame(&self, index: usize) -> Rect {
        let b = self.windows[index].body;
        Rect::new(
            b.x,
            b.y - 42 * self.scale as i32,
            b.width,
            b.height + 42 * self.scale,
        )
    }

    /// Open the app palette and reset its query and selection.
    ///
    /// Returns `false` when the palette is already open.
    pub fn palette_open(&mut self) -> bool {
        if self.palette.open || self.switcher_is_open() {
            return false;
        }
        self.palette.open = true;
        self.palette.query.fill(0);
        self.palette.length = 0;
        self.palette.selected = Some(self.focused);
        self.drag = None;
        true
    }

    /// Report whether the app palette is open.
    pub fn palette_is_open(&self) -> bool {
        self.palette.open
    }

    pub fn switcher_is_open(&self) -> bool {
        self.switcher.is_some()
    }

    /// Keep the entire mouse gesture in the switcher, even after dismissal.
    /// The native input dispatcher must not route the button-up to the dock.
    pub fn switcher_owns_pointer(&self) -> bool {
        self.switcher.is_some() || self.switcher_mouse_capture
    }

    pub fn switcher_selected(&self) -> Option<usize> {
        self.switcher
    }

    pub fn switcher_bounds(&self) -> Rect {
        let s = self.scale;
        Rect::new(
            (self.width - 620 * s) as i32 / 2,
            (self.height - 292 * s) as i32 / 2,
            620 * s,
            292 * s,
        )
    }

    pub fn switcher_card(&self, index: usize) -> Rect {
        if index >= WINDOWS {
            return Rect::default();
        }
        let r = self.switcher_bounds();
        let s = self.scale;
        Rect::new(
            r.x + (26 + index as i32 * 286) * s as i32,
            r.y + 58 * s as i32,
            282 * s,
            174 * s,
        )
    }

    /// Fit a live app image inside its card without stretching its aspect ratio.
    pub fn switcher_preview(&self, index: usize) -> Result<Rect, Invalid> {
        if index >= WINDOWS {
            return Err(Invalid);
        }
        let body = self.windows[index].body;
        if body.width == 0 || body.height == 0 {
            return Err(Invalid);
        }
        let card = self.switcher_card(index);
        let s = self.scale;
        let (maxw, maxh) = (250 * s, 112 * s);
        let (w, h) =
            if u64::from(body.width) * u64::from(maxh) > u64::from(body.height) * u64::from(maxw) {
                (
                    maxw,
                    (u64::from(body.height) * u64::from(maxw) / u64::from(body.width)) as u32,
                )
            } else {
                (
                    (u64::from(body.width) * u64::from(maxh) / u64::from(body.height)) as u32,
                    maxh,
                )
            };
        if w == 0 || h == 0 {
            return Err(Invalid);
        }
        Ok(Rect::new(
            card.x + (card.width - w) as i32 / 2,
            card.y + 10 * s as i32 + (maxh - h) as i32 / 2,
            w,
            h,
        ))
    }

    fn close_switcher(&mut self, commit: bool, damage: &mut Damage) -> Result<(), Invalid> {
        if let Some(selected) = self.switcher {
            if commit {
                self.select(selected)?;
                self.shell.hidden[selected] = false;
            }
            self.switcher = None;
            damage.mark(self.switcher_bounds());
            for index in 0..WINDOWS {
                damage.mark(self.paint_bounds(index));
            }
            damage.mark(self.dock());
        }
        Ok(())
    }

    /// Alt+Tab previews without changing focus; the last Alt release commits.
    /// Owned repeats and releases stay in the shell after dismissal.
    pub fn switcher_key(
        &mut self,
        key: KeyboardEvent,
        damage: &mut Damage,
    ) -> Result<bool, Invalid> {
        let index = key.key as usize;
        if self.switcher_is_open() {
            // Trusted shell, lock and recovery shortcuts must not be trapped
            // behind the unprivileged Alt+Tab overlay.
            if matches!(key.action, KeyAction::Press | KeyAction::Repeat)
                && ((key.key == PhysicalKey::L && key.modifiers.super_key())
                    || matches!(
                        key.key,
                        PhysicalKey::F3 | PhysicalKey::Menu | PhysicalKey::F10 | PhysicalKey::F12
                    ))
            {
                self.close_switcher(false, damage)?;
                return Ok(false);
            }
            self.switcher_keys[index] = key.action != KeyAction::Release;
            match (key.key, key.action) {
                (PhysicalKey::LeftAlt | PhysicalKey::RightAlt, KeyAction::Release)
                    if !key.modifiers.alt() =>
                {
                    self.close_switcher(true, damage)?;
                }
                (PhysicalKey::Escape, KeyAction::Press) => self.close_switcher(false, damage)?,
                (PhysicalKey::Enter | PhysicalKey::KeypadEnter, KeyAction::Press) => {
                    self.close_switcher(true, damage)?;
                }
                (PhysicalKey::Tab, KeyAction::Press)
                | (PhysicalKey::Left | PhysicalKey::Right, KeyAction::Press | KeyAction::Repeat) => {
                    self.switcher = self.switcher.map(|selected| 1 - selected);
                    damage.mark(self.switcher_bounds());
                }
                _ => {}
            }
            return Ok(true);
        }
        if self.switcher_keys[index] {
            if key.action == KeyAction::Release {
                self.switcher_keys[index] = false;
            }
            return Ok(true);
        }
        if key.key != PhysicalKey::Tab
            || key.action != KeyAction::Press
            || !key.modifiers.alt()
            || key.modifiers.control()
            || key.modifiers.super_key()
            || self.shell.blocks_apps()
        {
            return Ok(false);
        }
        if self.palette_close() {
            damage.mark(self.palette_bounds());
            damage.mark(self.dock());
        }
        self.cancel_pointer();
        self.switcher = Some(1 - self.focused);
        self.switcher_keys[index] = true;
        self.switcher_keys[PhysicalKey::LeftAlt as usize] = key.modifiers.left_alt;
        self.switcher_keys[PhysicalKey::RightAlt as usize] = key.modifiers.right_alt;
        damage.mark(self.switcher_bounds());
        damage.mark(self.dock());
        Ok(true)
    }

    /// Return the current ASCII app-name query.
    pub fn palette_query(&self) -> &str {
        core::str::from_utf8(&self.palette.query[..self.palette.length]).expect("ASCII app query")
    }

    /// Append one permitted ASCII letter, digit, or space to the open query.
    ///
    /// Returns `false` if the palette is closed, the query is full, or the
    /// byte is not accepted.
    pub fn palette_insert(&mut self, byte: u8) -> bool {
        if !self.palette.open
            || self.palette.length == QUERY_BYTES
            || !(byte.is_ascii_alphanumeric() || byte == b' ')
        {
            return false;
        }
        self.palette.query[self.palette.length] = byte;
        self.palette.length += 1;
        self.palette.selected = self.palette.first_match();
        true
    }

    /// Remove the last query byte and refresh the selected match.
    pub fn palette_backspace(&mut self) -> bool {
        if !self.palette.open || self.palette.length == 0 {
            return false;
        }
        self.palette.length -= 1;
        self.palette.query[self.palette.length] = 0;
        self.palette.selected = self.palette.first_match();
        true
    }

    /// Move selection to the next matching app, wrapping in the requested
    /// direction; return whether the selection changed.
    pub fn palette_next(&mut self, reverse: bool) -> bool {
        if !self.palette.open {
            return false;
        }
        let before = self.palette.selected;
        let start = before.unwrap_or(if reverse { 0 } else { WINDOWS - 1 });
        self.palette.selected = (1..=WINDOWS)
            .map(|step| {
                if reverse {
                    (start + WINDOWS - step) % WINDOWS
                } else {
                    (start + step) % WINDOWS
                }
            })
            .find(|index| self.palette.matches(*index));
        self.palette.selected != before
    }

    /// Return the selected app only while it still matches the open query.
    pub fn palette_selected(&self) -> Option<usize> {
        self.palette
            .selected
            .filter(|index| self.palette.open && self.palette.matches(*index))
    }

    /// Close the palette and clear its query and selection.
    pub fn palette_close(&mut self) -> bool {
        if !self.palette.open {
            return false;
        }
        self.palette.open = false;
        self.palette.query.fill(0);
        self.palette.length = 0;
        self.palette.selected = None;
        true
    }
    /// Physical pixels, stable while open and after close for damage repair.
    pub fn palette_bounds(&self) -> Rect {
        let width = 540 * self.scale;
        Rect::new(
            (self.width - width) as i32 / 2,
            84 * self.scale as i32,
            width,
            256 * self.scale,
        )
    }
    fn palette_row(&self, index: usize) -> Option<Rect> {
        if !self.palette.open || index >= WINDOWS || !self.palette.matches(index) {
            return None;
        }
        let row = (0..index)
            .filter(|other| self.palette.matches(*other))
            .count();
        let bounds = self.palette_bounds();
        let s = self.scale;
        Some(Rect::new(
            bounds.x + 16 * s as i32,
            bounds.y + (96 + row as i32 * 56) * s as i32,
            bounds.width - 32 * s,
            48 * s,
        ))
    }

    /// Return the matching app row under a physical-pixel position.
    pub fn palette_hit(&self, x: u32, y: u32) -> Option<usize> {
        (0..WINDOWS).find(|index| {
            self.palette_row(*index)
                .is_some_and(|row| row.contains(x as i32, y as i32))
        })
    }
    /// Returns true only for keys owned by the compositor's app palette.
    /// Route a keyboard event to the palette when it owns the event.
    ///
    /// Handled keys mark the palette and dock in `damage`. Returns `Ok(true)`
    /// when consumed and `Ok(false)` for unrelated keys.
    pub fn palette_key(
        &mut self,
        key: KeyboardEvent,
        damage: &mut Damage,
    ) -> Result<bool, Invalid> {
        // A userspace app launcher must not swallow the trusted lock/menu or
        // the native shutdown/fault-injection routes owned by the outer loop.
        if self.palette_is_open()
            && matches!(key.action, KeyAction::Press | KeyAction::Repeat)
            && ((key.key == PhysicalKey::L && key.modifiers.super_key())
                || matches!(
                    key.key,
                    PhysicalKey::F3 | PhysicalKey::Menu | PhysicalKey::F10 | PhysicalKey::F12
                ))
        {
            self.palette_close();
            damage.mark(self.palette_bounds());
            damage.mark(self.dock());
            return Ok(false);
        }
        let index = key.key as usize;
        // Keep ownership through key-up, even if closing the palette happens first.
        if key.action == KeyAction::Release && self.palette_keys[index] {
            self.palette_keys[index] = false;
            return Ok(true);
        }
        if !self.palette_is_open() {
            if self.palette_keys[index] {
                return Ok(true);
            }
            if self.shell.blocks_apps()
                || key.key != PhysicalKey::K
                || !key.modifiers.control()
                || key.modifiers.alt()
                || key.modifiers.super_key()
                || key.action != KeyAction::Press
            {
                return Ok(false);
            }
            self.palette_keys[index] = true;
            self.cancel_pointer();
            self.palette_open();
        } else if matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
            self.palette_keys[index] = true;
            match key.key {
                PhysicalKey::Escape => {
                    self.palette_close();
                }
                PhysicalKey::Enter | PhysicalKey::KeypadEnter if key.action == KeyAction::Press => {
                    if let Some(index) = self.palette_selected() {
                        self.shell.hidden[index] = false;
                        self.select(index)?;
                        self.palette_close();
                    }
                }
                PhysicalKey::Tab if key.action == KeyAction::Press => {
                    self.palette_next(key.modifiers.shift());
                }
                PhysicalKey::Up | PhysicalKey::Down => {
                    self.palette_next(key.key == PhysicalKey::Up);
                }
                PhysicalKey::Backspace => {
                    self.palette_backspace();
                }
                _ if !key.modifiers.control()
                    && !key.modifiers.alt()
                    && !key.modifiers.super_key() =>
                {
                    use PhysicalKey::*;
                    let keys = [
                        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y,
                        Z, Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8,
                        Digit9, Space,
                    ];
                    if let Some(index) = keys.iter().position(|candidate| *candidate == key.key) {
                        let mut byte = b"abcdefghijklmnopqrstuvwxyz0123456789 "[index];
                        if key.modifiers.shift() ^ key.modifiers.caps_lock {
                            byte = byte.to_ascii_uppercase();
                        }
                        self.palette_insert(byte);
                    }
                }
                _ => {}
            }
        }
        damage.mark(self.palette_bounds());
        damage.mark(self.dock());
        Ok(true)
    }

    /// Return the dock task button currently under the pointer, unless the
    /// palette is open.
    pub fn hovered_task(&self) -> Option<usize> {
        (!self.palette_is_open() && !self.switcher_is_open() && !self.shell.blocks_apps())
            .then(|| {
                (0..WINDOWS).find(|index| {
                    self.task_button(*index)
                        .contains(self.pointer.0 as i32, self.pointer.1 as i32)
                })
            })
            .flatten()
    }

    /// Forget palette key ownership and close it, marking affected regions.
    pub fn cancel_palette_input(&mut self, damage: &mut Damage) {
        self.palette_keys.fill(false);
        self.switcher_keys.fill(false);
        self.switcher_mouse_capture = false;
        self.close_switcher(false, damage)
            .expect("cancellation cannot select");
        if self.palette_close() {
            damage.mark(self.palette_bounds());
            damage.mark(self.dock());
        }
    }

    /// Apply pointer input and mark every old and new region that needs repaint.
    pub fn mouse_with_damage(
        &mut self,
        x: u32,
        y: u32,
        down: bool,
        held: bool,
        damage: &mut Damage,
    ) -> Result<Option<usize>, Invalid> {
        let hover = self.hovered_task();
        let palette_open = self.palette_is_open();
        let switcher_open = self.switcher_is_open();
        let frames = [self.paint_bounds(0), self.paint_bounds(1)];
        damage.mark(self.pointer_bounds());
        let recipient = self.mouse(x, y, down, held)?;
        damage.mark(self.pointer_bounds());
        // Both task buttons can change appearance when hover moves or vanishes.
        if hover != self.hovered_task() {
            for index in [hover, self.hovered_task()].into_iter().flatten() {
                damage.mark(self.task_button(index));
            }
        }
        if palette_open {
            damage.mark(self.palette_bounds());
            damage.mark(self.dock());
        }
        if switcher_open {
            damage.mark(self.switcher_bounds());
            if !self.switcher_is_open() {
                for index in 0..WINDOWS {
                    damage.mark(self.paint_bounds(index));
                }
                damage.mark(self.dock());
            }
        }
        for (index, frame) in frames.into_iter().enumerate() {
            if frame != self.paint_bounds(index) {
                damage.mark(frame);
                damage.mark(self.paint_bounds(index));
            }
        }
        Ok(recipient)
    }
    pub fn update_pointer(&mut self, x: u32, y: u32, damage: &mut Damage) {
        let hover = self.hovered_task();
        damage.mark(self.pointer_bounds());
        self.pointer = (x, y);
        damage.mark(self.pointer_bounds());
        if hover != self.hovered_task() {
            for index in [hover, self.hovered_task()].into_iter().flatten() {
                damage.mark(self.task_button(index));
            }
        }
    }

    /// Return the physical-pixel bounds reserved for clipboard status text.
    pub fn clipboard_notice_bounds(&self) -> Rect {
        Rect::new(
            300 * self.scale as i32,
            0,
            self.width.saturating_sub(540 * self.scale),
            44 * self.scale,
        )
    }

    /// Set the clipboard notice and report whether its visible state changed.
    pub fn set_clipboard_notice(&mut self, notice: ClipboardNotice) -> bool {
        if self.clipboard_notice == notice {
            return false;
        }
        self.clipboard_notice = notice;
        true
    }

    /// Return a window's repaint bounds, including its one-pixel contour.
    pub fn paint_bounds(&self, index: usize) -> Rect {
        let f = self.frame(index);
        Rect::new(
            f.x - self.scale as i32,
            f.y - self.scale as i32,
            f.width + 2 * self.scale,
            f.height + 2 * self.scale,
        )
    }
    /// A shell page change can retain the already opaque surround. App/window
    /// visibility changes and entering/leaving the cover need a full repaint.
    /// Header, footer and pointer damage are tracked separately by the caller.
    pub fn shell_damage(
        &self,
        was_blocked: bool,
        was_hidden: [bool; WINDOWS],
        previous_panel: Rect,
    ) -> Rect {
        if was_blocked && self.shell.blocks_apps() && was_hidden == self.shell.hidden {
            let panel = self.shell.panel(self.width, self.height, self.scale);
            // Retain old bounds as well when a page shrinks. All rectangles
            // come from this compositor, and the union is clipped to viewport.
            let left = previous_panel.x.min(panel.x).clamp(0, self.width as i32);
            let top = previous_panel.y.min(panel.y).clamp(0, self.height as i32);
            let right = (i64::from(previous_panel.x) + i64::from(previous_panel.width))
                .max(i64::from(panel.x) + i64::from(panel.width))
                .clamp(0, i64::from(self.width));
            let bottom = (i64::from(previous_panel.y) + i64::from(previous_panel.height))
                .max(i64::from(panel.y) + i64::from(panel.height))
                .clamp(0, i64::from(self.height));
            Rect::new(
                left,
                top,
                (right - i64::from(left)) as u32,
                (bottom - i64::from(top)) as u32,
            )
        } else {
            Rect::new(0, 0, self.width, self.height)
        }
    }
    /// UTC is also visible in the lock panel, independently of page changes.
    pub fn mark_clock_damage(&self, damage: &mut Damage) {
        damage.mark(Rect::new(0, 0, self.width, 44 * self.scale));
        if self.shell.page() == crate::shell::Page::Lock {
            damage.mark(self.shell.panel(self.width, self.height, self.scale));
        }
    }
    /// Reveal only complete app views. This is presentation readiness, never
    /// account authentication or a grant of storage/keyring/kernel authority.
    pub fn finish_unlock(
        &mut self,
        damage_pending: bool,
        completed_frames: [Option<u64>; WINDOWS],
    ) -> bool {
        if damage_pending
            || completed_frames
                .iter()
                .any(|frame| frame.is_none_or(|n| n == 0))
        {
            return false;
        }
        self.shell.finish_unlock()
    }

    /// Focus a valid window and cancel any drag; reject invalid indices.
    pub fn select(&mut self, index: usize) -> Result<(), Invalid> {
        if index >= WINDOWS {
            return Err(Invalid);
        }
        if index != self.focused {
            let switches = self.switches.checked_add(1).ok_or(Invalid)?;
            self.drag = None;
            self.focused = index;
            self.switches = switches;
        }
        Ok(())
    }

    /// Switch focus to the other window.
    pub fn cycle(&mut self) -> Result<(), Invalid> {
        self.shell.hidden[1 - self.focused] = false;
        self.select(1 - self.focused)
    }

    /// Cancel an active window drag.
    pub fn cancel_pointer(&mut self) {
        self.drag = None;
    }

    /// Return the physical-pixel bounds that the pointer renderer can affect.
    pub fn pointer_bounds(&self) -> Rect {
        let pad = 4 * self.scale as i32;
        Rect::new(
            self.pointer.0 as i32 - pad,
            self.pointer.1 as i32 - pad,
            24 * self.scale,
            28 * self.scale,
        )
    }
    /// Mouse policy receives the already decoded global position. Only a
    /// trusted title bar starts movement; application pixels cannot start it.
    /// Returns a body recipient, never a caller-chosen process identifier.
    pub fn mouse(
        &mut self,
        x: u32,
        y: u32,
        down: bool,
        held: bool,
    ) -> Result<Option<usize>, Invalid> {
        if x >= self.width || y >= self.height {
            return Err(Invalid);
        }
        self.pointer = (x, y);
        if self.switcher_is_open() {
            if down {
                self.switcher_mouse_capture = true;
                let hit = (0..WINDOWS)
                    .find(|index| self.switcher_card(*index).contains(x as i32, y as i32));
                if let Some(index) = hit {
                    self.select(index)?;
                    self.shell.hidden[index] = false;
                    self.switcher = None;
                } else if !self.switcher_bounds().contains(x as i32, y as i32) {
                    self.switcher = None;
                }
            }
            // A click on the surround leaves the overlay open; release its
            // capture on button-up so the following app click is delivered.
            if !held {
                self.switcher_mouse_capture = false;
            }
            return Ok(None);
        }
        if self.switcher_mouse_capture {
            self.switcher_mouse_capture = held;
            return Ok(None);
        }
        if self.shell.blocks_apps() {
            self.drag = None;
            return Ok(None);
        }
        if self.palette_is_open() {
            if down {
                if let Some(index) = self.palette_hit(x, y) {
                    self.shell.hidden[index] = false;
                    self.select(index)?;
                    self.palette_close();
                } else if !self.palette_bounds().contains(x as i32, y as i32) {
                    self.palette_close();
                }
            }
            return Ok(None);
        }
        if let Some((index, dx, dy)) = self.drag {
            if !held {
                self.drag = None;
                return Ok(None);
            }
            let window = &mut self.windows[index];
            let maxx = self.width - window.body.width;
            let maxy = self.height - window.body.height - 72 * self.scale;
            let nx = (x as i32 - dx).clamp(0, maxx as i32);
            let ny = (y as i32 - dy).clamp(70 * self.scale as i32, maxy as i32);
            if window.body.x != nx || window.body.y != ny {
                window.body.x = nx;
                window.body.y = ny;
                self.moves = self.moves.checked_add(1).ok_or(Invalid)?;
            }
            return Ok(None);
        }
        if down {
            let s = self.scale;
            for index in 0..WINDOWS {
                let button = self.task_button(index);
                if button.contains(x as i32, y as i32) {
                    self.shell.hidden[index] = false;
                    self.select(index)?;
                    return Ok(None);
                }
            }
            for index in [self.focused, 1 - self.focused] {
                if self.shell.hidden[index] {
                    continue;
                }
                let frame = self.frame(index);
                if frame.contains(x as i32, y as i32) {
                    self.select(index)?;
                    if y < self.windows[index].body.y as u32 {
                        if x >= (frame.x + frame.width as i32 - 84 * s as i32) as u32 {
                            self.shell.hidden[index] = true;
                            self.drag = None;
                            self.select(1 - index)?;
                            return Ok(None);
                        }
                        let b = self.windows[index].body;
                        self.drag = Some((index, x as i32 - b.x, y as i32 - b.y));
                        return Ok(None);
                    }
                    return Ok(Some(index));
                }
            }
            return Ok(None);
        }
        let b = self.windows[self.focused].body;
        Ok(
            (!self.shell.hidden[self.focused] && b.contains(x as i32, y as i32))
                .then_some(self.focused),
        )
    }

    /// Return the physical-pixel rectangle occupied by the dock.
    pub fn dock(&self) -> Rect {
        Rect::new(
            0,
            self.height as i32 - 64 * self.scale as i32,
            self.width,
            64 * self.scale,
        )
    }

    /// Return a task-button rectangle, or an empty rectangle for an invalid index.
    pub fn task_button(&self, index: usize) -> Rect {
        if index >= WINDOWS {
            return Rect::default();
        }
        let s = self.scale;
        let (x, width) = if index == 0 { (188, 152) } else { (360, 172) };
        Rect::new(
            x * s as i32,
            self.dock().y + 8 * s as i32,
            width * s,
            48 * s,
        )
    }

    /// Validate and copy one tile of 32-bit pixels into compositor backing.
    ///
    /// The pixel buffer must exactly match the tile dimensions and its high
    /// byte must be zero. Returns the tile's destination rectangle.
    pub fn copy_tile(
        &self,
        index: usize,
        rect: Rectangle,
        pixels: &[u8],
        backing: &mut [u32],
    ) -> Result<Rect, Invalid> {
        let window = self.windows.get(index).ok_or(Invalid)?;
        let (w, h) = (window.body.width, window.body.height);
        if rect.width == 0
            || rect.height == 0
            || rect
                .x
                .checked_add(u32::from(rect.width))
                .is_none_or(|v| v > w)
            || rect
                .y
                .checked_add(u32::from(rect.height))
                .is_none_or(|v| v > h)
            || pixels.len() != usize::from(rect.width) * usize::from(rect.height) * 4
            || window.offset + w as usize * h as usize > backing.len()
        {
            return Err(Invalid);
        }
        for row in 0..usize::from(rect.height) {
            for col in 0..usize::from(rect.width) {
                let at = (row * usize::from(rect.width) + col) * 4;
                let color = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
                if color >> 24 != 0 {
                    return Err(Invalid);
                }
            }
        }
        for row in 0..usize::from(rect.height) {
            for col in 0..usize::from(rect.width) {
                let at = (row * usize::from(rect.width) + col) * 4;
                backing[window.offset
                    + (rect.y as usize + row) * w as usize
                    + rect.x as usize
                    + col] = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
            }
        }
        Ok(Rect::new(
            window.body.x + rect.x as i32,
            window.body.y + rect.y as i32,
            u32::from(rect.width),
            u32::from(rect.height),
        ))
    }

    /// Render a viewport, using the opaque-body fast path when overlays do not
    /// intersect it and the reference compositor otherwise.
    pub fn render(&self, rect: Rect, pixels: &mut [u32], backing: &[u32]) -> Result<(), Invalid> {
        // Validate the same destination and backing extents as the composite
        // path before taking the opaque-body shortcut. No pixels are borrowed
        // from an app: this is the compositor's private, validated backing.
        Canvas::new(rect, pixels).map_err(|_| Invalid)?;
        for (index, window) in self.windows.iter().enumerate() {
            if self.paint_bounds(index).intersects(rect)
                && window
                    .body
                    .width
                    .checked_mul(window.body.height)
                    .and_then(|n| window.offset.checked_add(n as usize))
                    .is_none_or(|end| end > backing.len())
            {
                return Err(Invalid);
            }
        }
        // Every non-desktop shell page covers the entire viewport opaquely.
        // Keep the destination/backing validation above and the original
        // composite oracle below. Outside-viewport requests retain that oracle
        // because moved window shadows can extend beyond the visible screen.
        if self.shell.blocks_apps()
            && rect.x >= 0
            && rect.y >= 0
            && i64::from(rect.x) + i64::from(rect.width) <= i64::from(self.width)
            && i64::from(rect.y) + i64::from(rect.height) <= i64::from(self.height)
        {
            let mut c = Canvas::new(rect, pixels).map_err(|_| Invalid)?;
            self.shell
                .render(&mut c, self.width, self.height, self.scale, self.focused);
            self.system_foreground(&mut c);
            self.pointer_foreground(&mut c);
            return Ok(());
        }
        let window = self.windows[self.focused];
        let body = window.body;
        let inside = rect.x >= body.x
            && rect.y >= body.y
            && i64::from(rect.x) + i64::from(rect.width)
                <= i64::from(body.x) + i64::from(body.width)
            && i64::from(rect.y) + i64::from(rect.height)
                <= i64::from(body.y) + i64::from(body.height);
        if !inside
            || self.shell.blocks_apps()
            || self.shell.hidden[self.focused]
            || rect.intersects(self.dock())
            || rect.intersects(self.pointer_bounds())
            || (self.palette_is_open() && rect.intersects(self.palette_bounds()))
            || (self.switcher_is_open() && rect.intersects(self.switcher_bounds()))
        {
            return self.render_reference(rect, pixels, backing);
        }
        let left = (rect.x - body.x) as usize;
        let top = (rect.y - body.y) as usize;
        let width = rect.width as usize;
        for row in 0..rect.height as usize {
            let source = window.offset + (top + row) * body.width as usize + left;
            for (out, color) in pixels[row * width..(row + 1) * width]
                .iter_mut()
                .zip(&backing[source..source + width])
            {
                // Canvas::pixel also ignores the unused high color byte.
                *out = color & 0x00ff_ffff;
            }
        }
        Ok(())
    }
    /// Full composite path, retained as the original raster oracle for the
    /// opaque-body optimization. It performs the same normal bounds checks.
    pub fn render_reference(
        &self,
        rect: Rect,
        pixels: &mut [u32],
        backing: &[u32],
    ) -> Result<(), Invalid> {
        let mut c = Canvas::new(rect, pixels).map_err(|_| Invalid)?;
        if self.switcher_is_open() && rect.intersects(self.switcher_bounds()) {
            for window in self.windows {
                if window.body.width == 0
                    || window.body.height == 0
                    || window
                        .body
                        .width
                        .checked_mul(window.body.height)
                        .and_then(|n| window.offset.checked_add(n as usize))
                        .is_none_or(|end| end > backing.len())
                {
                    return Err(Invalid);
                }
            }
        }
        let palette = theme::MATERIAAL;
        theme::wallpaper(&mut c, Rect::new(0, 0, self.width, self.height), palette);
        let s = self.scale;
        let font = Font::new(Size::Body, Weight::Semibold, s == 2);
        for index in [1 - self.focused, self.focused] {
            if self.shell.hidden[index] {
                continue;
            }
            let body = self.windows[index].body;
            let frame = self.frame(index);
            if !self.paint_bounds(index).intersects(rect) {
                continue;
            }
            // A one-pixel structural contour outside the assigned app body.
            // There is no offset shadow or fake elevation behind the window.
            c.fill_rect(
                self.paint_bounds(index),
                if index == self.focused {
                    palette.border
                } else {
                    0x565861
                },
            );
            c.fill_rect(
                frame,
                if index == self.focused {
                    palette.surface
                } else {
                    palette.background
                },
            );
            let title = APP_NAMES[index];
            c.icon(
                Rect::new(
                    frame.x + 15 * s as i32,
                    frame.y + 12 * s as i32,
                    18 * s,
                    18 * s,
                ),
                if index == 0 {
                    Icon::Material
                } else {
                    Icon::Apps
                },
                if index == self.focused {
                    palette.accent
                } else {
                    palette.muted
                },
                s,
            );
            c.text(
                frame.x + 44 * s as i32,
                frame.y + 10 * s as i32,
                title,
                font,
                if index == self.focused {
                    palette.text
                } else {
                    palette.muted
                },
            );
            for (offset, icon) in [(76, Icon::Minimize), (38, Icon::Close)] {
                c.icon(
                    Rect::new(
                        frame.x + frame.width as i32 - offset * s as i32,
                        frame.y + 12 * s as i32,
                        20 * s,
                        20 * s,
                    ),
                    icon,
                    0x5c6570,
                    s,
                );
            }
            // Trusted app identity and focus marker live outside app bounds.
            if index == self.focused {
                c.fill_rect(
                    Rect::new(
                        frame.x + 44 * s as i32,
                        body.y - 3 * s as i32,
                        40 * s,
                        2 * s,
                    ),
                    palette.accent,
                );
            }
            c.fill_rect(body, palette.background);
            let left = rect.x.max(body.x);
            let top = rect.y.max(body.y);
            let right = (rect.x + rect.width as i32).min(body.x + body.width as i32);
            let bottom = (rect.y + rect.height as i32).min(body.y + body.height as i32);
            if self.windows[index].offset + body.width as usize * body.height as usize
                > backing.len()
            {
                return Err(Invalid);
            }
            for y in top..bottom {
                for x in left..right {
                    let at = self.windows[index].offset
                        + (y - body.y) as usize * body.width as usize
                        + (x - body.x) as usize;
                    c.pixel(x, y, backing[at]);
                }
            }
        }
        self.shell.render_with_hover(
            &mut c,
            self.width,
            self.height,
            self.scale,
            self.focused,
            self.hovered_task(),
        );
        self.system_foreground(&mut c);
        self.render_switcher(&mut c, backing)?;
        self.pointer_foreground(&mut c);
        Ok(())
    }
    fn system_foreground(&self, c: &mut Canvas<'_>) {
        let s = self.scale;
        if !self.shell.blocks_apps() {
            let font = Font::new(Size::Small, Weight::Regular, s == 2);
            let label = self.clipboard_notice.label();
            // A status label cannot overwrite Offline, time or system identity.
            if font.measure(label).saturating_add(8 * s) <= self.clipboard_notice_bounds().width {
                c.text(300 * s as i32, 12 * s as i32, label, font, 0xc5ccc9);
            }
        }
        if !self.shell.blocks_apps() {
            self.render_palette(c);
        }
    }
    fn pointer_foreground(&self, c: &mut Canvas<'_>) {
        let s = self.scale;
        let (x, y) = (self.pointer.0 as i32, self.pointer.1 as i32);
        c.line(x, y, x, y + 18 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 12 * s as i32, y + 12 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 3 * s as i32, y + 15 * s as i32, s, 0x172a3a);
    }
    fn render_switcher(&self, c: &mut Canvas<'_>, backing: &[u32]) -> Result<(), Invalid> {
        if !self.switcher_is_open() || !self.switcher_bounds().intersects(c.bounds()) {
            return Ok(());
        }
        let r = self.switcher_bounds();
        let p = theme::MATERIAAL;
        let s = self.scale;
        let small = Font::new(Size::Small, Weight::Regular, s == 2);
        c.fill_rect(r, p.border);
        c.fill_rect(
            Rect::new(
                r.x + s as i32,
                r.y + s as i32,
                r.width - 2 * s,
                r.height - 2 * s,
            ),
            p.surface,
        );
        c.text(
            r.x + 26 * s as i32,
            r.y + 20 * s as i32,
            "WISSEL VAN VENSTER",
            small,
            p.text,
        );
        for (index, name) in APP_NAMES.iter().enumerate() {
            let card = self.switcher_card(index);
            let selected = self.switcher == Some(index);
            c.fill_rect(card, if selected { p.accent } else { p.border });
            c.fill_rect(
                Rect::new(
                    card.x + 2 * s as i32,
                    card.y + 2 * s as i32,
                    card.width - 4 * s,
                    card.height - 4 * s,
                ),
                if selected {
                    p.accent_soft
                } else {
                    p.background
                },
            );
            let preview = self.switcher_preview(index)?;
            let body = self.windows[index].body;
            let bounds = c.bounds();
            for y in preview.y.max(bounds.y)
                ..(preview.y + preview.height as i32).min(bounds.y + bounds.height as i32)
            {
                for x in preview.x.max(bounds.x)
                    ..(preview.x + preview.width as i32).min(bounds.x + bounds.width as i32)
                {
                    let sx = (u64::from((x - preview.x) as u32) * u64::from(body.width)
                        / u64::from(preview.width)) as usize;
                    let sy = (u64::from((y - preview.y) as u32) * u64::from(body.height)
                        / u64::from(preview.height)) as usize;
                    c.pixel(
                        x,
                        y,
                        backing[self.windows[index].offset + sy * body.width as usize + sx],
                    );
                }
            }
            c.text(
                card.x + 16 * s as i32,
                card.y + 130 * s as i32,
                name,
                Font::new(Size::Body, Weight::Semibold, s == 2),
                p.text,
            );
            c.text(
                card.x + 16 * s as i32,
                card.y + 152 * s as i32,
                if selected {
                    "Geselecteerd"
                } else {
                    "Open venster"
                },
                small,
                p.muted,
            );
        }
        c.text(
            r.x + 26 * s as i32,
            r.y + 250 * s as i32,
            "Tab kiezen · Alt los / Enter openen · Esc terug",
            small,
            p.text,
        );
        Ok(())
    }
    fn render_palette(&self, c: &mut Canvas<'_>) {
        if !self.palette_is_open() || !self.palette_bounds().intersects(c.bounds()) {
            return;
        }
        let p = theme::MATERIAAL;
        let r = self.palette_bounds();
        let s = self.scale;
        let small = Font::new(Size::Small, Weight::Regular, s == 2);
        let body = Font::new(Size::Body, Weight::Regular, s == 2);
        let strong = Font::new(Size::Body, Weight::Semibold, s == 2);
        c.fill_rect(r, p.border);
        c.fill_rect(
            Rect::new(
                r.x + s as i32,
                r.y + s as i32,
                r.width - 2 * s,
                r.height - 2 * s,
            ),
            p.surface,
        );
        c.text(
            r.x + 24 * s as i32,
            r.y + 16 * s as i32,
            "OPEN APPS",
            small,
            p.muted,
        );
        c.text(
            r.x + r.width as i32 - 74 * s as i32,
            r.y + 16 * s as i32,
            "Esc terug",
            small,
            p.muted,
        );
        c.icon(
            Rect::new(r.x + 24 * s as i32, r.y + 50 * s as i32, 20 * s, 20 * s),
            Icon::Search,
            p.accent,
            s,
        );
        c.text(
            r.x + 56 * s as i32,
            r.y + 48 * s as i32,
            if self.palette_query().is_empty() {
                "Zoek een app"
            } else {
                self.palette_query()
            },
            body,
            if self.palette_query().is_empty() {
                p.muted
            } else {
                p.text
            },
        );
        c.fill_rect(
            Rect::new(
                r.x + 24 * s as i32,
                r.y + 80 * s as i32,
                r.width - 48 * s,
                s,
            ),
            p.accent,
        );
        if self.palette.first_match().is_none() {
            c.text(
                r.x + 24 * s as i32,
                r.y + 112 * s as i32,
                "Geen app met deze naam",
                body,
                p.text,
            );
        }
        for (index, name) in APP_NAMES.iter().enumerate() {
            let Some(row) = self.palette_row(index) else {
                continue;
            };
            if self.palette_selected() == Some(index) {
                c.fill_rect(row, p.accent_soft);
                c.fill_rect(Rect::new(row.x, row.y, 3 * s, row.height), p.accent);
            }
            c.icon(
                Rect::new(row.x + 14 * s as i32, row.y + 14 * s as i32, 20 * s, 20 * s),
                if index == 0 {
                    Icon::Material
                } else {
                    Icon::Apps
                },
                p.accent,
                s,
            );
            c.text(
                row.x + 46 * s as i32,
                row.y + 12 * s as i32,
                name,
                strong,
                p.text,
            );
            c.text(
                row.x + row.width as i32 - 86 * s as i32,
                row.y + 16 * s as i32,
                "Activeren",
                small,
                p.muted,
            );
        }
        c.text(
            r.x + 24 * s as i32,
            r.y + 226 * s as i32,
            "Pijltjes kiezen · Enter activeren",
            small,
            p.muted,
        );
    }
}

/// Fixed-capacity set of 32-by-8 physical-pixel regions awaiting repaint.
pub struct Damage {
    bits: [u64; 128],
    width: u32,
    height: u32,
    cursor: usize,
}
impl Damage {
    /// Inspect outstanding chrome/cursor tiles without consuming one.
    pub fn has_pending(&self) -> bool {
        self.bits.iter().any(|word| *word != 0)
    }

    /// Create an empty damage set for a nonzero viewport no larger than 1920×1080.
    pub fn new(width: u32, height: u32) -> Result<Self, Invalid> {
        if width == 0 || height == 0 || width > 1920 || height > 1080 {
            return Err(Invalid);
        }
        Ok(Self {
            bits: [0; 128],
            width,
            height,
            cursor: 0,
        })
    }

    /// Mark every damage tile intersecting `r`; clip the rectangle to the viewport.
    pub fn mark(&mut self, r: Rect) {
        let columns = self.width.div_ceil(32);
        let left = r.x.clamp(0, self.width as i32) as u32 / 32;
        let top = r.y.clamp(0, self.height as i32) as u32 / 8;
        let right = (i64::from(r.x) + i64::from(r.width)).clamp(0, i64::from(self.width)) as u32;
        let bottom = (i64::from(r.y) + i64::from(r.height)).clamp(0, i64::from(self.height)) as u32;
        for y in top..bottom.div_ceil(8) {
            for x in left..right.div_ceil(32) {
                let at = (y * columns + x) as usize;
                if at < 8192 {
                    self.bits[at / 64] |= 1 << (at % 64);
                }
            }
        }
        // New damage may precede the scan cursor, so restart enumeration.
        self.cursor = 0;
    }

    /// Consume and return the next marked tile in scan order, if any.
    pub fn next_tile(&mut self) -> Option<Rect> {
        let columns = self.width.div_ceil(32) as usize;
        let count = columns * self.height.div_ceil(8) as usize;
        while self.cursor < count {
            let at = self.cursor;
            self.cursor += 1;
            if self.bits[at / 64] & (1 << (at % 64)) != 0 {
                self.bits[at / 64] &= !(1 << (at % 64));
                let (x, y) = ((at % columns) as u32 * 32, (at / columns) as u32 * 8);
                return Some(Rect::new(
                    x as i32,
                    y as i32,
                    32.min(self.width - x),
                    8.min(self.height - y),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_backing_at_both_actual_scales() {
        for (w, h, s) in [(1280, 720, 1), (1920, 1080, 2), (1920, 1080, 1)] {
            let scene = Scene::new(w, h, s, [1, 2]).unwrap();
            let last = scene.windows[1];
            assert!(
                last.offset + last.body.width as usize * last.body.height as usize <= MAX_PIXELS
            );
            for i in 0..2 {
                let f = scene.frame(i);
                assert!(f.x >= 0 && f.y >= 0);
                assert!(f.x + f.width as i32 <= w as i32);
            }
        }
    }
    #[test]
    fn body_input_cannot_start_title_drag_and_focus_tracks_z_order() {
        let mut s = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let b = s.windows[1].body;
        assert_eq!(
            s.mouse(b.x as u32 + 20, b.y as u32 + 20, true, true)
                .unwrap(),
            Some(1)
        );
        assert_eq!(s.focused, 1);
        assert!(s.drag.is_none());
        assert_eq!(
            s.mouse(b.x as u32 + 20, b.y as u32 - 20, true, true)
                .unwrap(),
            None
        );
        s.mouse(800, 100, false, true).unwrap();
        assert_eq!(s.moves, 1);
        s.mouse(800, 100, false, false).unwrap();
        assert!(s.drag.is_none());
    }
    #[test]
    fn tile_rejection_is_atomic_and_scoped_to_own_backing() {
        let s = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut backing = std::vec![0x123456u32;MAX_PIXELS];
        let mut p = [0u8; 8];
        p[7] = 1;
        let r = Rectangle {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        assert_eq!(s.copy_tile(0, r, &p, &mut backing), Err(Invalid));
        assert_eq!(backing[0], 0x123456);
        p[7] = 0;
        p[0] = 0x77;
        s.copy_tile(1, r, &p, &mut backing).unwrap();
        assert_eq!(backing[0], 0x123456);
        assert_eq!(backing[s.windows[1].offset], 0x77);
    }
    #[test]
    fn damage_clips_coordinates_and_visits_each_tile_once() {
        let mut d = Damage::new(1280, 720).unwrap();
        d.mark(Rect::new(-50, -50, 90, 60));
        assert_eq!(d.next_tile(), Some(Rect::new(0, 0, 32, 8)));
        let mut count = 1;
        while d.next_tile().is_some() {
            count += 1;
        }
        assert_eq!(count, 4);
        d.mark(Rect::new(1279, 719, u32::MAX, u32::MAX));
        assert_eq!(d.next_tile(), Some(Rect::new(1248, 712, 32, 8)));
        assert_eq!(d.next_tile(), None);
    }
}
