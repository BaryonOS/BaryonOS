use nieuw_os_graphics::{theme, Canvas, Font, Icon, Rect, Size, Weight};
use nieuw_os_surface_protocol::{Geometry, Rectangle};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

use crate::clipboard::Notice as ClipboardNotice;

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
const WORK_NAMES: [&str; WINDOWS] = ["Mijn materiaal", "Berekening"];
const WORK_TYPES: [&str; WINDOWS] = ["Documenten", "Gereedschap"];
const WORK_ALIASES: [&[&str]; WINDOWS] = [
    &[
        "bestanden",
        "verkenner",
        "explorer",
        "finder",
        "files",
        "nautilus",
        "dolphin",
    ],
    &["calculator", "rekenen"],
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkspaceMode {
    #[default]
    Work,
    Focus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
}
#[derive(Clone, Copy)]
struct KeyboardMove {
    index: usize,
    origin: Rect,
}
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
            || [APP_NAMES[index], WORK_NAMES[index], WORK_TYPES[index]]
                .iter()
                .chain(WORK_ALIASES[index].iter())
                .any(|label| {
                    label
                        .as_bytes()
                        .windows(query.len())
                        .any(|part| part.eq_ignore_ascii_case(query))
                })
    }
    fn first_match(&self) -> Option<usize> {
        (0..WINDOWS).find(|index| self.matches(*index))
    }
}
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub windows: [Window; WINDOWS],
    pub focused: usize,
    pub pointer: (u32, u32),
    drag: Option<(usize, i32, i32)>,
    pub moves: u64,
    pub switches: u64,
    mode: WorkspaceMode,
    clipboard_notice: ClipboardNotice,
    palette: ProgramPalette,
    palette_keys: [bool; PhysicalKey::Menu as usize + 1],
    cancelled_keys: [bool; PhysicalKey::Menu as usize + 1],
    states: [WindowState; WINDOWS],
    window_menu_open: bool,
    window_menu_target: usize,
    window_menu_selected: usize,
    overlay_pointer: bool,
    keyboard_move: Option<KeyboardMove>,
    notifications_open: bool,
    notice_dismissed: bool,
    notification_focus_close: bool,
}
impl Scene {
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
            width,
            height,
            scale,
            windows,
            focused: 0,
            pointer: (width / 2, height / 2),
            drag: None,
            moves: 0,
            switches: 0,
            mode: WorkspaceMode::Work,
            clipboard_notice: ClipboardNotice::None,
            palette: ProgramPalette::new(),
            palette_keys: [false; PhysicalKey::Menu as usize + 1],
            cancelled_keys: [false; PhysicalKey::Menu as usize + 1],
            states: [WindowState::Normal; WINDOWS],
            window_menu_open: false,
            window_menu_target: 0,
            window_menu_selected: 0,
            overlay_pointer: false,
            keyboard_move: None,
            notifications_open: false,
            notice_dismissed: false,
            notification_focus_close: false,
        })
    }
    pub fn frame(&self, index: usize) -> Rect {
        let b = self.windows[index].body;
        Rect::new(
            b.x,
            b.y - 42 * self.scale as i32,
            b.width,
            b.height + 42 * self.scale,
        )
    }
    pub fn workspace_mode(&self) -> WorkspaceMode {
        self.mode
    }
    pub fn visible(&self, index: usize) -> bool {
        self.window_state(index) == Some(WindowState::Normal)
            && (self.mode == WorkspaceMode::Work || self.focused == index)
    }
    pub fn window_state(&self, index: usize) -> Option<WindowState> {
        self.states.get(index).copied()
    }
    pub fn focused_visible(&self) -> bool {
        self.visible(self.focused)
    }
    fn minimize_state(&mut self, index: usize) -> Result<(), Invalid> {
        if index >= WINDOWS {
            return Err(Invalid);
        }
        if self.focused == index && self.states[1 - index] == WindowState::Normal {
            self.select(1 - index)?;
        }
        self.states[index] = WindowState::Minimized;
        self.cancel_pointer();
        Ok(())
    }
    pub fn minimize(&mut self, index: usize, damage: &mut Damage) -> Result<(), Invalid> {
        if self.keyboard_move_is_active() {
            return Err(Invalid);
        }
        self.minimize_state(index)?;
        self.close_window_menu(damage);
        for index in 0..WINDOWS {
            damage.mark(self.paint_bounds(index));
        }
        damage.mark(self.dock());
        Ok(())
    }
    pub fn window_menu_is_open(&self) -> bool {
        self.window_menu_open
    }
    pub fn keyboard_move_is_active(&self) -> bool {
        self.keyboard_move.is_some()
    }
    pub fn overlay_is_open(&self) -> bool {
        self.palette_is_open()
            || self.window_menu_is_open()
            || self.keyboard_move_is_active()
            || self.notifications_are_open()
    }
    pub fn notifications_are_open(&self) -> bool {
        self.notifications_open
    }
    pub fn notification_is_present(&self) -> bool {
        self.clipboard_notice != ClipboardNotice::None && !self.notice_dismissed
    }
    pub fn notifications_bounds(&self) -> Rect {
        let s = self.scale;
        Rect::new(
            (self.width - 468 * s) as i32,
            56 * s as i32,
            440 * s,
            232 * s,
        )
    }
    pub fn notifications_paint_bounds(&self) -> Rect {
        let r = self.notifications_bounds();
        let pad = 12 * self.scale;
        Rect::new(
            r.x - pad as i32,
            r.y - pad as i32,
            r.width + 2 * pad,
            r.height + 2 * pad,
        )
    }
    pub fn notification_close_button(&self) -> Rect {
        let r = self.notifications_bounds();
        let s = self.scale;
        Rect::new(r.x + 396 * s as i32, r.y + 12 * s as i32, 32 * s, 32 * s)
    }
    pub fn notification_dismiss_button(&self) -> Rect {
        let r = self.notifications_bounds();
        let s = self.scale;
        Rect::new(r.x + 280 * s as i32, r.y + 144 * s as i32, 136 * s, 36 * s)
    }
    fn open_notifications(&mut self, damage: &mut Damage) {
        if !self.overlay_is_open() {
            self.cancel_pointer();
            self.notifications_open = true;
            self.notification_focus_close = !self.notification_is_present();
            damage.mark(self.notifications_paint_bounds());
            damage.mark(self.dock());
        }
    }
    fn close_notifications(&mut self, damage: &mut Damage) {
        if self.notifications_open {
            self.notifications_open = false;
            damage.mark(self.notifications_paint_bounds());
            damage.mark(self.dock());
        }
    }
    fn dismiss_notification(&mut self) {
        self.notice_dismissed = true;
        self.notification_focus_close = true;
    }
    fn begin_keyboard_move(&mut self, damage: &mut Damage) {
        if self.focused_visible() {
            self.cancel_pointer();
            self.close_window_menu(damage);
            self.keyboard_move = Some(KeyboardMove {
                index: self.focused,
                origin: self.windows[self.focused].body,
            });
            damage.mark(Rect::new(0, 0, self.width, 40 * self.scale));
            damage.mark(self.dock());
        }
    }
    fn finish_keyboard_move(&mut self, cancel: bool) {
        if let Some(movement) = self.keyboard_move.take() {
            if cancel {
                self.windows[movement.index].body = movement.origin;
            }
        }
    }
    fn cancel_keyboard_move(&mut self, damage: &mut Damage) {
        if let Some(movement) = self.keyboard_move {
            damage.mark(self.paint_bounds(movement.index));
            self.finish_keyboard_move(true);
            damage.mark(self.paint_bounds(movement.index));
            damage.mark(Rect::new(0, 0, self.width, 40 * self.scale));
            damage.mark(self.dock());
        }
    }
    fn position_window(&mut self, index: usize, x: i32, y: i32) -> Result<(), Invalid> {
        let body = self.windows[index].body;
        let x = x.clamp(0, (self.width - body.width) as i32);
        let y = y.clamp(
            94 * self.scale as i32,
            (self.height - body.height - 72 * self.scale) as i32,
        );
        if (body.x, body.y) != (x, y) {
            let moves = self.moves.checked_add(1).ok_or(Invalid)?;
            self.windows[index].body.x = x;
            self.windows[index].body.y = y;
            self.moves = moves;
        }
        Ok(())
    }
    pub fn window_menu_button(&self, index: usize) -> Rect {
        if index >= WINDOWS {
            return Rect::default();
        }
        let frame = self.frame(index);
        let s = self.scale;
        Rect::new(
            frame.x + frame.width as i32 - 72 * s as i32,
            frame.y + 7 * s as i32,
            28 * s,
            28 * s,
        )
    }
    pub fn minimize_button(&self, index: usize) -> Rect {
        let mut r = self.window_menu_button(index);
        if index < WINDOWS {
            r.x += 36 * self.scale as i32;
        }
        r
    }
    /// Physical pixels; the anchor survives closing for shadow damage repair.
    pub fn window_menu_bounds(&self) -> Rect {
        let s = self.scale;
        let button = self.window_menu_button(self.window_menu_target);
        Rect::new(
            button.x.clamp(12 * s as i32, (self.width - 312 * s) as i32),
            (button.y + 33 * s as i32).clamp(52 * s as i32, (self.height - 260 * s) as i32),
            300 * s,
            184 * s,
        )
    }
    fn window_menu_paint_bounds(&self) -> Rect {
        let r = self.window_menu_bounds();
        let pad = 12 * self.scale;
        Rect::new(
            r.x - pad as i32,
            r.y - pad as i32,
            r.width + 2 * pad,
            r.height + 2 * pad,
        )
    }
    fn window_menu_row(&self, index: usize) -> Rect {
        let r = self.window_menu_bounds();
        let s = self.scale;
        Rect::new(
            r.x + 8 * s as i32,
            r.y + (8 + index as i32 * 44) * s as i32,
            r.width - 16 * s,
            40 * s,
        )
    }
    fn open_window_menu(&mut self) {
        if self.focused_visible() {
            self.palette_close();
            self.cancel_pointer();
            self.window_menu_target = self.focused;
            self.window_menu_selected = 0;
            self.window_menu_open = true;
        }
    }
    fn close_window_menu(&mut self, damage: &mut Damage) {
        if self.window_menu_open {
            damage.mark(self.window_menu_paint_bounds());
            self.window_menu_open = false;
            damage.mark(self.dock());
        }
    }
    fn activate_window_menu(&mut self) -> Result<(), Invalid> {
        self.window_menu_open = false;
        match self.window_menu_selected {
            0 => self.minimize_state(self.window_menu_target)?,
            1 => self.arrange_positions(),
            _ => {
                self.cancel_pointer();
                self.mode = match self.mode {
                    WorkspaceMode::Work => WorkspaceMode::Focus,
                    WorkspaceMode::Focus => WorkspaceMode::Work,
                };
            }
        }
        Ok(())
    }
    pub fn toggle_focus(&mut self, damage: &mut Damage) {
        self.cancel_keyboard_move(damage);
        for index in 0..WINDOWS {
            damage.mark(self.paint_bounds(index));
        }
        self.mode = match self.mode {
            WorkspaceMode::Work => WorkspaceMode::Focus,
            WorkspaceMode::Focus => WorkspaceMode::Work,
        };
        self.cancel_pointer();
        damage.mark(self.mode_button());
    }
    pub fn arrange(&mut self, damage: &mut Damage) {
        self.cancel_keyboard_move(damage);
        let old = [self.paint_bounds(0), self.paint_bounds(1)];
        self.arrange_positions();
        for (index, bounds) in old.into_iter().enumerate() {
            damage.mark(bounds);
            damage.mark(self.paint_bounds(index));
        }
        damage.mark(self.mode_button());
    }
    fn arrange_positions(&mut self) {
        self.cancel_pointer();
        self.mode = WorkspaceMode::Work;
        for index in 0..WINDOWS {
            let body = &mut self.windows[index].body;
            body.x = if index == 0 {
                28 * self.scale as i32
            } else {
                (self.width - body.width - 28 * self.scale) as i32
            };
            body.y = (if index == 0 { 114 } else { 140 }) * self.scale as i32;
        }
    }
    pub fn mode_button(&self) -> Rect {
        Rect::new(
            self.width as i32 - 284 * self.scale as i32,
            0,
            136 * self.scale,
            40 * self.scale,
        )
    }
    pub fn arrange_button(&self) -> Rect {
        Rect::new(
            self.width as i32 - 144 * self.scale as i32,
            0,
            136 * self.scale,
            40 * self.scale,
        )
    }
    pub fn command_button(&self) -> Rect {
        Rect::new(
            self.width as i32 - 166 * self.scale as i32,
            self.dock().y + 4 * self.scale as i32,
            158 * self.scale,
            40 * self.scale,
        )
    }
    pub fn palette_open(&mut self) -> bool {
        if self.overlay_is_open() {
            return false;
        }
        self.palette.open = true;
        self.palette.query.fill(0);
        self.palette.length = 0;
        self.palette.selected = Some(self.focused);
        self.drag = None;
        true
    }
    pub fn palette_is_open(&self) -> bool {
        self.palette.open
    }
    pub fn palette_query(&self) -> &str {
        core::str::from_utf8(&self.palette.query[..self.palette.length]).expect("ASCII app query")
    }
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
    pub fn palette_backspace(&mut self) -> bool {
        if !self.palette.open || self.palette.length == 0 {
            return false;
        }
        self.palette.length -= 1;
        self.palette.query[self.palette.length] = 0;
        self.palette.selected = self.palette.first_match();
        true
    }
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
    pub fn palette_selected(&self) -> Option<usize> {
        self.palette
            .selected
            .filter(|index| self.palette.open && self.palette.matches(*index))
    }
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
    pub fn palette_paint_bounds(&self) -> Rect {
        let r = self.palette_bounds();
        let pad = 12 * self.scale;
        Rect::new(
            r.x - pad as i32,
            r.y - pad as i32,
            r.width + 2 * pad,
            r.height + 2 * pad,
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
    pub fn palette_hit(&self, x: u32, y: u32) -> Option<usize> {
        (0..WINDOWS).find(|index| {
            self.palette_row(*index)
                .is_some_and(|row| row.contains(x as i32, y as i32))
        })
    }
    /// Returns true only for keys owned by trusted workspace controls.
    pub fn palette_key(
        &mut self,
        key: KeyboardEvent,
        damage: &mut Damage,
    ) -> Result<bool, Invalid> {
        let index = key.key as usize;
        if key.action == KeyAction::Release
            && (self.palette_keys[index] || self.cancelled_keys[index])
        {
            self.palette_keys[index] = false;
            self.cancelled_keys[index] = false;
            return Ok(true);
        }
        if key.action == KeyAction::Press {
            self.cancelled_keys[index] = false;
        } else if self.cancelled_keys[index] {
            return Ok(true);
        }
        let frames = [self.paint_bounds(0), self.paint_bounds(1)];
        let states = self.states;
        let focus = self.focused;
        let mode = self.mode;
        if self.notifications_are_open() {
            if matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
                self.palette_keys[index] = true;
                if key.action == KeyAction::Press {
                    match key.key {
                        PhysicalKey::Escape => self.close_notifications(damage),
                        PhysicalKey::Tab | PhysicalKey::Up | PhysicalKey::Down
                            if self.notification_is_present() =>
                        {
                            self.notification_focus_close = !self.notification_focus_close
                        }
                        PhysicalKey::Delete => self.dismiss_notification(),
                        PhysicalKey::Enter | PhysicalKey::KeypadEnter | PhysicalKey::Space => {
                            if self.notification_focus_close {
                                self.close_notifications(damage);
                            } else {
                                self.dismiss_notification();
                            }
                        }
                        _ => {}
                    }
                }
            }
            damage.mark(self.notifications_paint_bounds());
            damage.mark(self.clipboard_notice_bounds());
            return Ok(true);
        }
        if key.key == PhysicalKey::N
            && key.modifiers.alt()
            && !key.modifiers.control()
            && !key.modifiers.shift()
            && !key.modifiers.super_key()
            && !self.overlay_is_open()
        {
            if key.action == KeyAction::Press && !self.palette_keys[index] {
                self.palette_keys[index] = true;
                self.open_notifications(damage);
            }
            return Ok(true);
        }
        if let Some(movement) = self.keyboard_move {
            if matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
                self.palette_keys[index] = true;
                let step = (if key.modifiers.shift() { 1 } else { 16 }) * self.scale as i32;
                let body = self.windows[movement.index].body;
                match key.key {
                    PhysicalKey::Escape => self.finish_keyboard_move(true),
                    PhysicalKey::Enter | PhysicalKey::KeypadEnter
                        if key.action == KeyAction::Press =>
                    {
                        self.finish_keyboard_move(false)
                    }
                    PhysicalKey::Left => {
                        self.position_window(movement.index, body.x - step, body.y)?
                    }
                    PhysicalKey::Right => {
                        self.position_window(movement.index, body.x + step, body.y)?
                    }
                    PhysicalKey::Up => {
                        self.position_window(movement.index, body.x, body.y - step)?
                    }
                    PhysicalKey::Down => {
                        self.position_window(movement.index, body.x, body.y + step)?
                    }
                    _ => {}
                }
            }
            if !self.keyboard_move_is_active() {
                damage.mark(Rect::new(0, 0, self.width, 40 * self.scale));
                damage.mark(self.dock());
            }
            self.mark_window_changes(frames, states, focus, mode, damage);
            return Ok(true);
        }
        let move_chord = key.key == PhysicalKey::F7
            && key.modifiers.alt()
            && !key.modifiers.control()
            && !key.modifiers.shift()
            && !key.modifiers.super_key();
        if move_chord && !self.palette_is_open() {
            if key.action == KeyAction::Press && !self.palette_keys[index] {
                self.palette_keys[index] = true;
                self.begin_keyboard_move(damage);
            }
            return Ok(true);
        }
        if self.window_menu_is_open() {
            damage.mark(self.window_menu_paint_bounds());
            if matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
                self.palette_keys[index] = true;
                match key.key {
                    PhysicalKey::Escape => self.window_menu_open = false,
                    PhysicalKey::Up | PhysicalKey::Down | PhysicalKey::Tab => {
                        let reverse = key.key == PhysicalKey::Up
                            || (key.key == PhysicalKey::Tab && key.modifiers.shift());
                        self.window_menu_selected =
                            (self.window_menu_selected + if reverse { 2 } else { 1 }) % 3;
                    }
                    PhysicalKey::Enter | PhysicalKey::KeypadEnter
                        if key.action == KeyAction::Press =>
                    {
                        self.activate_window_menu()?;
                    }
                    _ => {}
                }
            }
            if !self.window_menu_is_open() {
                damage.mark(self.dock());
            }
            self.mark_window_changes(frames, states, focus, mode, damage);
            return Ok(true);
        }
        if !self.palette_is_open() {
            if self.palette_keys[index] {
                return Ok(true);
            }
            if key.modifiers.alt()
                && !key.modifiers.control()
                && !key.modifiers.super_key()
                && matches!(key.key, PhysicalKey::F9 | PhysicalKey::Space)
            {
                if key.action == KeyAction::Press {
                    self.palette_keys[index] = true;
                    if key.key == PhysicalKey::F9 {
                        self.minimize(self.focused, damage)?;
                    } else {
                        self.open_window_menu();
                        damage.mark(self.window_menu_paint_bounds());
                        damage.mark(self.dock());
                    }
                }
                return Ok(true);
            }
            if matches!(key.key, PhysicalKey::F7 | PhysicalKey::F8)
                && !key.modifiers.alt()
                && !key.modifiers.control()
                && !key.modifiers.super_key()
            {
                if key.action == KeyAction::Press {
                    self.palette_keys[index] = true;
                    if key.key == PhysicalKey::F7 {
                        self.toggle_focus(damage);
                    } else {
                        self.arrange(damage);
                    }
                }
                return Ok(true);
            }
            if key.key != PhysicalKey::K
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
        damage.mark(self.palette_paint_bounds());
        damage.mark(self.dock());
        self.mark_window_changes(frames, states, focus, mode, damage);
        Ok(true)
    }
    pub fn hovered_task(&self) -> Option<usize> {
        (!self.overlay_is_open())
            .then(|| {
                (0..WINDOWS).find(|index| {
                    self.task_button(*index)
                        .contains(self.pointer.0 as i32, self.pointer.1 as i32)
                })
            })
            .flatten()
    }
    pub fn cancel_palette_input(&mut self, damage: &mut Damage) {
        // Keep ownership until release, even when focus loss closes an overlay.
        for (cancelled, owned) in self.cancelled_keys.iter_mut().zip(&mut self.palette_keys) {
            *cancelled |= *owned;
            *owned = false;
        }
        self.close_window_menu(damage);
        self.close_notifications(damage);
        self.cancel_keyboard_move(damage);
        if self.palette_close() {
            damage.mark(self.palette_paint_bounds());
            damage.mark(self.dock());
        }
    }
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
        let menu_open = self.window_menu_is_open();
        let move_active = self.keyboard_move_is_active();
        let notifications_open = self.notifications_are_open();
        let menu_bounds = self.window_menu_paint_bounds();
        let states = self.states;
        let mode = self.mode;
        let focus = self.focused;
        let frames = [self.paint_bounds(0), self.paint_bounds(1)];
        damage.mark(self.pointer_bounds());
        let recipient = self.mouse(x, y, down, held)?;
        damage.mark(self.pointer_bounds());
        if hover != self.hovered_task() {
            for index in [hover, self.hovered_task()].into_iter().flatten() {
                damage.mark(self.task_button(index));
            }
        }
        if palette_open || self.palette_is_open() {
            damage.mark(self.palette_paint_bounds());
            damage.mark(self.dock());
        }
        if menu_open {
            damage.mark(menu_bounds);
        }
        if self.window_menu_is_open() {
            damage.mark(self.window_menu_paint_bounds());
        }
        if notifications_open || self.notifications_are_open() {
            damage.mark(self.notifications_paint_bounds());
            damage.mark(self.clipboard_notice_bounds());
            damage.mark(self.dock());
        }
        if move_active && !self.keyboard_move_is_active() {
            damage.mark(Rect::new(0, 0, self.width, 40 * self.scale));
            damage.mark(self.dock());
        }
        self.mark_window_changes(frames, states, focus, mode, damage);
        Ok(recipient)
    }
    fn mark_window_changes(
        &self,
        frames: [Rect; WINDOWS],
        states: [WindowState; WINDOWS],
        focus: usize,
        mode: WorkspaceMode,
        damage: &mut Damage,
    ) {
        if focus != self.focused || states != self.states {
            damage.mark(self.dock());
        }
        for (index, frame) in frames.into_iter().enumerate() {
            if frame != self.paint_bounds(index)
                || focus != self.focused
                || mode != self.mode
                || states[index] != self.states[index]
            {
                damage.mark(frame);
                damage.mark(self.paint_bounds(index));
            }
            if mode != self.mode {
                damage.mark(self.mode_button());
            }
        }
    }
    pub fn clipboard_notice_bounds(&self) -> Rect {
        Rect::new(
            300 * self.scale as i32,
            0,
            self.width - 588 * self.scale,
            40 * self.scale,
        )
    }
    pub fn set_clipboard_notice(&mut self, notice: ClipboardNotice) -> bool {
        if self.clipboard_notice == notice {
            return false;
        }
        self.clipboard_notice = notice;
        self.notice_dismissed = false;
        if !self.notification_is_present() {
            self.notification_focus_close = true;
        }
        true
    }
    pub fn paint_bounds(&self, index: usize) -> Rect {
        let f = self.frame(index);
        Rect::new(
            f.x - 12 * self.scale as i32,
            f.y - 8 * self.scale as i32,
            f.width + 24 * self.scale,
            f.height + 24 * self.scale,
        )
    }
    pub fn select(&mut self, index: usize) -> Result<(), Invalid> {
        if index >= WINDOWS || self.keyboard_move_is_active() {
            return Err(Invalid);
        }
        if index != self.focused {
            let switches = self.switches.checked_add(1).ok_or(Invalid)?;
            self.drag = None;
            self.focused = index;
            self.switches = switches;
        }
        self.states[index] = WindowState::Normal;
        Ok(())
    }
    pub fn cycle(&mut self) -> Result<(), Invalid> {
        self.select(1 - self.focused)
    }
    pub fn cancel_pointer(&mut self) {
        self.drag = None;
    }
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
        if self.notifications_are_open() {
            if down {
                self.overlay_pointer = true;
                if self
                    .notification_close_button()
                    .contains(x as i32, y as i32)
                    || !self.notifications_bounds().contains(x as i32, y as i32)
                {
                    self.notifications_open = false;
                } else if self.notification_is_present()
                    && self
                        .notification_dismiss_button()
                        .contains(x as i32, y as i32)
                {
                    self.dismiss_notification();
                }
            }
            if !held && !down {
                self.overlay_pointer = false;
            }
            return Ok(None);
        }
        if self.keyboard_move_is_active() {
            if down {
                self.finish_keyboard_move(true);
                self.overlay_pointer = true;
            }
            return Ok(None);
        }
        if self.window_menu_is_open() {
            if down {
                self.overlay_pointer = true;
                if let Some(index) =
                    (0..3).find(|index| self.window_menu_row(*index).contains(x as i32, y as i32))
                {
                    self.window_menu_selected = index;
                    self.activate_window_menu()?;
                } else if !self.window_menu_bounds().contains(x as i32, y as i32) {
                    self.window_menu_open = false;
                }
            }
            if !held && !down {
                self.overlay_pointer = false;
            }
            return Ok(None);
        }
        if self.palette_is_open() {
            if down {
                self.overlay_pointer = true;
                if let Some(index) = self.palette_hit(x, y) {
                    self.select(index)?;
                    self.palette_close();
                } else if !self.palette_bounds().contains(x as i32, y as i32) {
                    self.palette_close();
                }
            }
            if !held && !down {
                self.overlay_pointer = false;
            }
            return Ok(None);
        }
        if self.overlay_pointer {
            if !held {
                self.overlay_pointer = false;
            }
            return Ok(None);
        }
        if let Some((index, dx, dy)) = self.drag {
            if !held {
                self.drag = None;
                return Ok(None);
            }
            self.position_window(index, x as i32 - dx, y as i32 - dy)?;
            return Ok(None);
        }
        if down {
            if self.clipboard_notice_bounds().contains(x as i32, y as i32) {
                self.cancel_pointer();
                self.notifications_open = true;
                self.notification_focus_close = !self.notification_is_present();
                self.overlay_pointer = true;
                return Ok(None);
            }
            if self.command_button().contains(x as i32, y as i32) {
                self.palette_open();
                return Ok(None);
            }
            if self.mode_button().contains(x as i32, y as i32) {
                self.mode = match self.mode {
                    WorkspaceMode::Work => WorkspaceMode::Focus,
                    WorkspaceMode::Focus => WorkspaceMode::Work,
                };
                self.cancel_pointer();
                return Ok(None);
            }
            if self.arrange_button().contains(x as i32, y as i32) {
                self.arrange_positions();
                return Ok(None);
            }
            for index in 0..WINDOWS {
                let button = self.task_button(index);
                if button.contains(x as i32, y as i32) {
                    self.select(index)?;
                    return Ok(None);
                }
            }
            for index in [self.focused, 1 - self.focused] {
                if !self.visible(index) {
                    continue;
                }
                let frame = self.frame(index);
                if frame.contains(x as i32, y as i32) {
                    self.select(index)?;
                    if y < self.windows[index].body.y as u32 {
                        if self.window_menu_button(index).contains(x as i32, y as i32) {
                            self.open_window_menu();
                            self.overlay_pointer = true;
                            return Ok(None);
                        }
                        if self.minimize_button(index).contains(x as i32, y as i32) {
                            self.minimize_state(index)?;
                            self.overlay_pointer = true;
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
        Ok((self.focused_visible() && b.contains(x as i32, y as i32)).then_some(self.focused))
    }
    pub fn dock(&self) -> Rect {
        Rect::new(
            16 * self.scale as i32,
            self.height as i32 - 64 * self.scale as i32,
            self.width - 32 * self.scale,
            56 * self.scale,
        )
    }
    fn dock_paint_bounds(&self) -> Rect {
        let r = self.dock();
        let pad = 6 * self.scale;
        Rect::new(
            r.x - pad as i32,
            r.y - pad as i32,
            r.width + 2 * pad,
            r.height + 2 * pad,
        )
    }
    pub fn task_button(&self, index: usize) -> Rect {
        if index >= WINDOWS {
            return Rect::default();
        }
        let s = self.scale;
        Rect::new(
            (28 + index as i32 * 224) * s as i32,
            self.dock().y + 6 * s as i32,
            216 * s,
            44 * s,
        )
    }
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
    pub fn render(&self, rect: Rect, pixels: &mut [u32], backing: &[u32]) -> Result<(), Invalid> {
        // Validate the same destination and backing extents as the composite
        // path before taking the opaque-body shortcut. No pixels are borrowed
        // from an app: this is the compositor's private, validated backing.
        Canvas::new(rect, pixels).map_err(|_| Invalid)?;
        for (index, window) in self.windows.iter().enumerate() {
            if self.visible(index)
                && self.paint_bounds(index).intersects(rect)
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
        let window = self.windows[self.focused];
        let body = window.body;
        let inside = rect.x >= body.x
            && rect.y >= body.y
            && i64::from(rect.x) + i64::from(rect.width)
                <= i64::from(body.x) + i64::from(body.width)
            && i64::from(rect.y) + i64::from(rect.height)
                <= i64::from(body.y) + i64::from(body.height);
        if !self.focused_visible()
            || !inside
            || rect.intersects(self.dock_paint_bounds())
            || rect.intersects(self.pointer_bounds())
            || (self.palette_is_open() && rect.intersects(self.palette_paint_bounds()))
            || (self.window_menu_is_open() && rect.intersects(self.window_menu_paint_bounds()))
            || (self.notifications_are_open() && rect.intersects(self.notifications_paint_bounds()))
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
        let palette = theme::MATERIAAL;
        theme::wallpaper(&mut c, Rect::new(0, 0, self.width, self.height), palette);
        let s = self.scale;
        let font = Font::new(Size::Body, Weight::Semibold, s == 2);
        c.fill_rect(Rect::new(0, 0, self.width, 40 * s), palette.desktop_start);
        c.fill_rect(
            Rect::new(0, (40 * s - 1) as i32, self.width, 1),
            theme::SHELL_LINE,
        );
        c.icon(
            Rect::new(22 * s as i32, 10 * s as i32, 20 * s, 20 * s),
            Icon::Material,
            palette.accent,
            s,
        );
        c.text(54 * s as i32, 9 * s as i32, "Veld", font, theme::SHELL_TEXT);
        c.text(
            112 * s as i32,
            10 * s as i32,
            "Werk op dit apparaat",
            Font::new(Size::Small, Weight::Regular, s == 2),
            theme::SHELL_MUTED,
        );
        c.text(
            300 * s as i32,
            10 * s as i32,
            if self.notification_is_present() {
                self.clipboard_notice.label()
            } else {
                "Meldingen · Alt+N"
            },
            Font::new(Size::Small, Weight::Regular, s == 2),
            theme::SHELL_TEXT,
        );
        for (bounds, label) in [
            (
                self.mode_button(),
                if self.mode == WorkspaceMode::Focus {
                    "F7 · Alles"
                } else {
                    "F7 · Dit werk"
                },
            ),
            (self.arrange_button(), "F8 · Ordenen"),
        ] {
            c.rounded_rect(
                Rect::new(
                    bounds.x + 2 * s as i32,
                    bounds.y + 4 * s as i32,
                    bounds.width - 4 * s,
                    bounds.height - 8 * s,
                ),
                6 * s,
                palette.background,
            );
            c.text(
                bounds.x + 12 * s as i32,
                bounds.y + 10 * s as i32,
                label,
                Font::new(Size::Small, Weight::Regular, s == 2),
                palette.text,
            );
        }
        if self.keyboard_move_is_active() {
            c.fill_rect(Rect::new(0, 0, self.width, 39 * s), palette.desktop_start);
            c.text(
                22 * s as i32,
                10 * s as i32,
                "Verplaatsen · Pijlen 16 px · Shift+pijlen 1 px · Enter bewaren · Esc terug",
                Font::new(Size::Small, Weight::Regular, s == 2),
                theme::SHELL_TEXT,
            );
        }
        for index in [1 - self.focused, self.focused] {
            let body = self.windows[index].body;
            let frame = self.frame(index);
            if !self.visible(index) || !self.paint_bounds(index).intersects(rect) {
                continue;
            }
            c.soft_shadow(
                Rect::new(frame.x, frame.y + 4 * s as i32, frame.width, frame.height),
                12 * s,
                theme::SHADOW,
                if index == self.focused { 40 } else { 22 },
            )
            .map_err(|_| Invalid)?;
            c.rounded_rect(
                Rect::new(
                    frame.x - s as i32,
                    frame.y - s as i32,
                    frame.width + 2 * s,
                    frame.height + 2 * s,
                ),
                10 * s,
                if index == self.focused {
                    palette.border
                } else {
                    theme::SHELL_LINE
                },
            );
            c.rounded_rect(
                frame,
                9 * s,
                if index == self.focused {
                    palette.surface
                } else {
                    palette.background
                },
            );
            let title = WORK_NAMES[index];
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
            let title_width = font.measure(title);
            let small = Font::new(Size::Small, Weight::Regular, s == 2);
            let tool = APP_NAMES[index];
            if title_width + small.measure(tool) + 156 * s <= frame.width {
                c.text(
                    frame.x + 60 * s as i32 + title_width as i32,
                    frame.y + 12 * s as i32,
                    tool,
                    small,
                    palette.muted,
                );
            }
            for (bounds, menu) in [
                (self.window_menu_button(index), true),
                (self.minimize_button(index), false),
            ] {
                c.rounded_rect(bounds, 5 * s, palette.background);
                let x = bounds.x + 8 * s as i32;
                let y = bounds.y + 13 * s as i32;
                for dy in if menu {
                    [-5, 0, 5].as_slice()
                } else {
                    [5].as_slice()
                } {
                    c.fill_rect(Rect::new(x, y + dy * s as i32, 12 * s, 2 * s), palette.text);
                }
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
        let dock = self.dock();
        c.soft_shadow(dock, 6 * s, theme::SHADOW, 24)
            .map_err(|_| Invalid)?;
        c.rounded_rect(dock, 12 * s, theme::SHELL_LINE);
        c.rounded_rect(
            Rect::new(
                dock.x + s as i32,
                dock.y + s as i32,
                dock.width - 2 * s,
                dock.height - 2 * s,
            ),
            11 * s,
            palette.surface,
        );
        for (index, name) in WORK_NAMES.iter().enumerate() {
            let r = self.task_button(index);
            let active = index == self.focused && self.focused_visible();
            let minimized = self.states[index] == WindowState::Minimized;
            let hovered = self.hovered_task() == Some(index);
            if active || hovered {
                c.rounded_rect(r, 6 * s, theme::SHELL_ACTIVE);
            }
            c.icon(
                Rect::new(r.x + 10 * s as i32, r.y + 9 * s as i32, 18 * s, 18 * s),
                if index == 0 {
                    Icon::Material
                } else {
                    Icon::Apps
                },
                if active {
                    palette.accent
                } else {
                    theme::SHELL_MUTED
                },
                s,
            );
            c.text(
                r.x + 38 * s as i32,
                r.y + 2 * s as i32,
                name,
                Font::new(Size::Body, Weight::Regular, s == 2),
                theme::SHELL_TEXT,
            );
            c.text(
                r.x + 38 * s as i32,
                r.y + 25 * s as i32,
                if minimized {
                    "Geminimaliseerd"
                } else {
                    APP_NAMES[index]
                },
                Font::new(Size::Small, Weight::Regular, s == 2),
                theme::SHELL_MUTED,
            );
            if active {
                c.fill_rect(
                    Rect::new(r.x + 10 * s as i32, r.y + 40 * s as i32, 20 * s, 2 * s),
                    palette.accent,
                );
            } else if minimized {
                let marker = Rect::new(r.x + 12 * s as i32, r.y + 36 * s as i32, 12 * s, 6 * s);
                c.fill_rect(marker, theme::SHELL_MUTED);
                c.fill_rect(
                    Rect::new(marker.x + s as i32, marker.y + s as i32, 10 * s, 4 * s),
                    if hovered {
                        theme::SHELL_ACTIVE
                    } else {
                        palette.surface
                    },
                );
            }
        }
        c.icon(
            Rect::new(
                self.width as i32 - 150 * s as i32,
                dock.y + 16 * s as i32,
                16 * s,
                16 * s,
            ),
            Icon::Search,
            theme::SHELL_MUTED,
            s,
        );
        c.text(
            self.width as i32 - 124 * s as i32,
            dock.y + 14 * s as i32,
            "Ctrl K · Werk",
            Font::new(Size::Small, Weight::Regular, s == 2),
            theme::SHELL_MUTED,
        );
        self.render_palette(&mut c);
        self.render_window_menu(&mut c);
        self.render_notifications(&mut c);
        let (x, y) = (self.pointer.0 as i32, self.pointer.1 as i32);
        c.line(x, y, x, y + 18 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 12 * s as i32, y + 12 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 3 * s as i32, y + 15 * s as i32, s, 0x172a3a);
        Ok(())
    }
    fn render_notifications(&self, c: &mut Canvas<'_>) {
        if !self.notifications_are_open()
            || !self.notifications_paint_bounds().intersects(c.bounds())
        {
            return;
        }
        let p = theme::MATERIAAL;
        let r = self.notifications_bounds();
        let s = self.scale;
        let small = Font::new(Size::Small, Weight::Regular, s == 2);
        let body = Font::new(Size::Body, Weight::Regular, s == 2);
        c.soft_shadow(r, 12 * s, theme::SHADOW, 40)
            .expect("validated workspace scale and nonempty notifications");
        c.rounded_rect(r, 8 * s, p.border);
        c.rounded_rect(
            Rect::new(
                r.x + s as i32,
                r.y + s as i32,
                r.width - 2 * s,
                r.height - 2 * s,
            ),
            7 * s,
            p.surface,
        );
        c.text(
            r.x + 24 * s as i32,
            r.y + 18 * s as i32,
            "Systeemmeldingen",
            Font::new(Size::Body, Weight::Semibold, s == 2),
            p.text,
        );
        let close = self.notification_close_button();
        c.icon(
            Rect::new(
                close.x + 8 * s as i32,
                close.y + 8 * s as i32,
                16 * s,
                16 * s,
            ),
            Icon::Close,
            p.text,
            s,
        );
        if self.notification_focus_close {
            c.stroke_rounded_rect(close, 4 * s, 2 * s, p.accent);
        }
        if self.notification_is_present() {
            c.text(
                r.x + 24 * s as i32,
                r.y + 70 * s as i32,
                "Klembord · Op dit apparaat",
                small,
                p.muted,
            );
            c.text(
                r.x + 24 * s as i32,
                r.y + 100 * s as i32,
                self.clipboard_notice.label(),
                body,
                p.text,
            );
            let dismiss = self.notification_dismiss_button();
            c.rounded_rect(dismiss, 4 * s, p.accent_soft);
            if !self.notification_focus_close {
                c.stroke_rounded_rect(dismiss, 4 * s, 2 * s, p.accent);
            }
            c.text(
                dismiss.x + 14 * s as i32,
                dismiss.y + 8 * s as i32,
                "Verwijderen",
                small,
                p.text,
            );
        } else {
            c.text(
                r.x + 24 * s as i32,
                r.y + 88 * s as i32,
                "Geen meldingen",
                body,
                p.text,
            );
        }
        c.text(
            r.x + 24 * s as i32,
            r.y + 202 * s as i32,
            "Tab kiezen · Delete verwijderen · Esc terug",
            small,
            p.muted,
        );
    }
    fn render_window_menu(&self, c: &mut Canvas<'_>) {
        if !self.window_menu_is_open() || !self.window_menu_paint_bounds().intersects(c.bounds()) {
            return;
        }
        let p = theme::MATERIAAL;
        let r = self.window_menu_bounds();
        let s = self.scale;
        c.soft_shadow(r, 12 * s, theme::SHADOW, 40)
            .expect("validated workspace scale and nonempty window menu");
        c.rounded_rect(r, 8 * s, p.border);
        c.rounded_rect(
            Rect::new(
                r.x + s as i32,
                r.y + s as i32,
                r.width - 2 * s,
                r.height - 2 * s,
            ),
            7 * s,
            p.surface,
        );
        for (index, label) in [
            "Minimaliseren",
            "Ordenen",
            if self.mode == WorkspaceMode::Focus {
                "Alles tonen"
            } else {
                "Dit werk"
            },
        ]
        .iter()
        .enumerate()
        {
            let row = self.window_menu_row(index);
            if index == self.window_menu_selected {
                c.fill_rect(row, p.accent_soft);
                c.fill_rect(Rect::new(row.x, row.y, 3 * s, row.height), p.accent);
            }
            c.text(
                row.x + 12 * s as i32,
                row.y + 10 * s as i32,
                label,
                Font::new(Size::Body, Weight::Regular, s == 2),
                p.text,
            );
        }
        c.text(
            r.x + 20 * s as i32,
            r.y + 152 * s as i32,
            "Alt+F7 · Verplaatsen",
            Font::new(Size::Small, Weight::Regular, s == 2),
            p.muted,
        );
    }
    fn render_palette(&self, c: &mut Canvas<'_>) {
        if !self.palette_is_open() || !self.palette_paint_bounds().intersects(c.bounds()) {
            return;
        }
        let p = theme::MATERIAAL;
        let r = self.palette_bounds();
        let s = self.scale;
        let small = Font::new(Size::Small, Weight::Regular, s == 2);
        let body = Font::new(Size::Body, Weight::Regular, s == 2);
        let strong = Font::new(Size::Body, Weight::Semibold, s == 2);
        c.soft_shadow(r, 12 * s, theme::SHADOW, 40)
            .expect("validated workspace scale and nonempty palette");
        c.rounded_rect(r, 12 * s, p.border);
        c.rounded_rect(
            Rect::new(
                r.x + s as i32,
                r.y + s as i32,
                r.width - 2 * s,
                r.height - 2 * s,
            ),
            11 * s,
            p.surface,
        );
        c.text(
            r.x + 24 * s as i32,
            r.y + 16 * s as i32,
            "VERDER MET JE WERK",
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
                "Zoek materiaal of gereedschap"
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
                "Geen open werk met deze naam",
                body,
                p.text,
            );
        }
        for (index, name) in WORK_NAMES.iter().enumerate() {
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
        self.cursor = 0;
    }
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
