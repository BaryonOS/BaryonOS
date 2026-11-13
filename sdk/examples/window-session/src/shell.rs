//! Compositor-owned navigation for the disposable VM. No account service,
//! keyring unlock, device authentication or kernel capability is fabricated.
use crate::session::{Account, AccountId, SessionFlow, UnavailableBroker};
use nieuw_os_graphics::{Canvas, Font, Icon, Rect, Size, Weight};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

// Trusted dock roles on main's dark desktop; contrast is computed in tests.
pub const DOCK_BAR: u32 = 0xf2f1ec;
pub const DOCK_HOVER: u32 = 0xe3ede8;
pub const DOCK_ACTIVE: u32 = 0xffffff;
pub const DOCK_TEXT: u32 = 0x20262d;
pub const DOCK_ICON: u32 = 0x30434b;
pub const DOCK_ACCENT: u32 = 0x14655e;
pub const DOCK_MARKER: u32 = 0x5c6570;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Lock,
    Users,
    Auth,
    Unlocking,
    Desktop,
    Search,
    Settings,
    Notifications,
    Session,
    Capability,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    None,
    Activate(usize),
    Hide(usize),
    Shutdown,
    Restart,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Effect {
    pub consumed: bool,
    pub changed: bool,
    pub command: Command,
}
impl Effect {
    const fn pass() -> Self {
        Self {
            consumed: false,
            changed: false,
            command: Command::None,
        }
    }
    const fn take(changed: bool, command: Command) -> Self {
        Self {
            consumed: true,
            changed,
            command,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Control {
    pub bounds: Rect,
    pub label: &'static str,
    pub icon: Icon,
    action: u8,
}

pub struct Shell {
    flow: SessionFlow,
    page: Page,
    focus: usize,
    query: [u8; 64],
    query_len: usize,
    notice: &'static str,
    pressed: Option<u8>,
    pub hidden: [bool; 2],
    pub reduced_motion: bool,
    clock: [u8; 5],
    clock_valid: bool,
    date: [u8; 10],
    super_chord: bool,
}
impl Shell {
    pub fn new(preview: bool) -> Self {
        Self {
            flow: SessionFlow::new(),
            page: if preview { Page::Lock } else { Page::Desktop },
            focus: 0,
            query: [0; 64],
            query_len: 0,
            notice: "",
            pressed: None,
            hidden: [false; 2],
            reduced_motion: true,
            clock: *b"--:--",
            clock_valid: false,
            date: *b"----------",
            super_chord: false,
        }
    }
    pub const fn page(&self) -> Page {
        self.page
    }
    pub fn blocks_apps(&self) -> bool {
        self.page != Page::Desktop
    }
    pub fn query(&self) -> &str {
        core::str::from_utf8(&self.query[..self.query_len]).unwrap_or("")
    }
    pub fn clock(&self) -> &str {
        core::str::from_utf8(&self.clock).unwrap_or("--:--")
    }
    pub fn date(&self) -> &str {
        core::str::from_utf8(&self.date).unwrap_or("")
    }
    /// Display-only UTC from the existing qualified RTC reading. Never used
    /// for authentication freshness, certificate validity or rate limiting.
    pub fn set_clock(&mut self, unix: Option<i64>) -> bool {
        let unix = unix.filter(|t| (0..=253_402_300_799).contains(t));
        let mut text = *b"--:--";
        if let Some(time) = unix.filter(|t| *t >= 0) {
            let minutes = (time / 60) % 1440;
            let hour = (minutes / 60) as u8;
            let minute = (minutes % 60) as u8;
            text = [
                b'0' + hour / 10,
                b'0' + hour % 10,
                b':',
                b'0' + minute / 10,
                b'0' + minute % 10,
            ];
        }
        let mut date = *b"----------";
        if let Some(time) = unix {
            let mut days = time / 86400;
            let mut year = 1970u32;
            loop {
                let length = if leap(year) { 366 } else { 365 };
                if days < length {
                    break;
                }
                days -= length;
                year += 1;
            }
            let mut month = 1u32;
            for length in [
                31,
                if leap(year) { 29 } else { 28 },
                31,
                30,
                31,
                30,
                31,
                31,
                30,
                31,
                30,
                31,
            ] {
                if days < length {
                    break;
                }
                days -= length;
                month += 1;
            }
            let day = days as u32 + 1;
            date = [
                b'0' + (year / 1000) as u8,
                b'0' + (year / 100 % 10) as u8,
                b'0' + (year / 10 % 10) as u8,
                b'0' + (year % 10) as u8,
                b'-',
                b'0' + (month / 10) as u8,
                b'0' + (month % 10) as u8,
                b'-',
                b'0' + (day / 10) as u8,
                b'0' + (day % 10) as u8,
            ];
        }
        let changed = self.clock != text || self.date != date || self.clock_valid != unix.is_some();
        self.clock = text;
        self.clock_valid = unix.is_some();
        self.date = date;
        changed
    }
    pub fn open(&mut self, page: Page) {
        self.page = page;
        self.focus = 0;
        self.pressed = None;
        self.notice = "";
        self.query.fill(0);
        self.query_len = 0;
    }
    pub fn lock(&mut self) {
        let _ = self.flow.lock();
        self.open(Page::Lock);
    }
    pub fn finish_unlock(&mut self) -> bool {
        if self.page != Page::Unlocking {
            return false;
        }
        if self.flow.activate_guest().is_ok() {
            self.open(Page::Desktop);
            true
        } else {
            self.open(Page::Lock);
            false
        }
    }
    pub fn panel(&self, width: u32, height: u32, scale: u32) -> Rect {
        let w = (width / scale).saturating_sub(48).min(568);
        let preferred_height = match self.page {
            Page::Users => 344,
            Page::Auth if self.flow.selected_account() == Some(Account::VmGuest) => 320,
            Page::Auth | Page::Settings => 368,
            Page::Notifications | Page::Unlocking => 320,
            Page::Session => 400,
            Page::Capability => 432,
            Page::Search => 448,
            _ => 476,
        };
        let h = (height / scale).saturating_sub(112).min(preferred_height);
        Rect::new(
            ((width / scale - w) / 2 * scale) as i32,
            ((height / scale - h) / 2 * scale) as i32,
            w * scale,
            h * scale,
        )
    }
    pub fn controls(&self, width: u32, height: u32, scale: u32) -> [Option<Control>; 8] {
        let p = self.panel(width, height, scale);
        let s = scale as i32;
        let mut out = [None; 8];
        let mut add = |index: usize, y: i32, label, icon, action| {
            out[index] = Some(Control {
                bounds: Rect::new(p.x + 28 * s, p.y + y * s, p.width - 56 * scale, 44 * scale),
                label,
                icon,
                action,
            });
        };
        match self.page {
            Page::Lock => add(
                0,
                (p.height / scale) as i32 - 68,
                "Aanmelden",
                Icon::User,
                1,
            ),
            Page::Users => {
                add(0, 112, "VM-gast · openbare testsessie", Icon::User, 2);
                add(
                    1,
                    176,
                    "Lokale gebruiker · nog niet gekoppeld",
                    Icon::User,
                    3,
                );
                add(2, 264, "Terug naar vergrendelscherm", Icon::ChevronLeft, 4);
            }
            Page::Auth => {
                if self.flow.selected_account() == Some(Account::VmGuest) {
                    add(0, 168, "Open testsessie", Icon::ChevronRight, 5);
                    add(1, 232, "Andere gebruiker", Icon::User, 1);
                } else {
                    add(0, 112, "Wachtwoord · dienst niet beschikbaar", Icon::Key, 6);
                    add(
                        1,
                        168,
                        "Vingerafdruk · sensor niet beschikbaar",
                        Icon::Fingerprint,
                        7,
                    );
                    add(2, 224, "FIDO2-sleutel · geen apparaatdienst", Icon::Key, 8);
                    add(3, 296, "Andere gebruiker", Icon::ChevronLeft, 1);
                }
            }
            Page::Search => {
                add(0, 104, "Zoek apps, materiaal of acties", Icon::Search, 14);
                let compact = p.height / scale < 416;
                let (first, gap) = if compact { (156, 44) } else { (164, 48) };
                for (i, (label, icon, action)) in self.results().into_iter().flatten().enumerate() {
                    add(i + 1, first + i as i32 * gap, label, icon, action);
                }
            }
            Page::Settings => {
                add(0, 112, "Beweging verminderen · aan", Icon::Check, 9);
                add(1, 168, "Verbinding · offline in deze VM", Icon::Network, 10);
                add(
                    2,
                    224,
                    "Opslag · alleen lokale testgegevens",
                    Icon::Folder,
                    10,
                );
                add(3, 296, "Terug naar werkruimte", Icon::ChevronLeft, 0);
            }
            Page::Notifications => add(0, 232, "Terug naar werkruimte", Icon::ChevronLeft, 0),
            Page::Session => {
                add(0, 96, "Scherm afschermen · VM-preview", Icon::Lock, 4);
                add(1, 152, "Gebruiker wisselen", Icon::User, 1);
                add(2, 208, "Snelle instellingen", Icon::Settings, 11);
                add(3, 264, "Afsluiten", Icon::Power, 12);
                add(
                    4,
                    320,
                    "Opnieuw starten · nog niet beschikbaar",
                    Icon::Refresh,
                    13,
                );
            }
            Page::Capability => {
                add(0, 296, "Annuleren", Icon::Close, 0);
                add(
                    1,
                    352,
                    "Toegang verlenen · broker ontbreekt",
                    Icon::Shield,
                    10,
                );
            }
            Page::Desktop | Page::Unlocking => {}
        }
        out
    }
    fn results(&self) -> [Option<(&'static str, Icon, u8)>; 5] {
        let mut out = [None; 5];
        let mut at = 0;
        for (label, terms, icon, action) in [
            (
                "Materiaal",
                "materiaal documenten bestanden",
                Icon::Folder,
                20,
            ),
            (
                "Rekenmachine",
                "rekenmachine calculator rekenen",
                Icon::Apps,
                21,
            ),
            (
                "Mijn materiaal openen",
                "mijn materiaal document openen",
                Icon::Material,
                20,
            ),
            (
                "Instellingen",
                "instellingen beeld verbinding opslag",
                Icon::Settings,
                11,
            ),
            (
                "Scherm afschermen",
                "vergrendelen afschermen lock",
                Icon::Lock,
                4,
            ),
        ] {
            let q = self.query().as_bytes();
            if q.is_empty()
                || terms
                    .as_bytes()
                    .windows(q.len())
                    .any(|v| v.eq_ignore_ascii_case(q))
            {
                out[at] = Some((label, icon, action));
                at += 1;
            }
        }
        out
    }
    fn activate(&mut self, action: u8) -> Command {
        match action {
            0 => self.open(Page::Desktop),
            1 => {
                let _ = self.flow.cancel();
                self.open(Page::Users);
            }
            2 | 3 => {
                let account = if action == 2 {
                    Account::VmGuest
                } else {
                    Account::LocalUser(AccountId::new(1).unwrap())
                };
                if self.flow.select_account(account).is_ok() {
                    self.open(Page::Auth);
                }
            }
            4 => self.lock(),
            5 => {
                if self.flow.begin_guest().is_ok() {
                    self.open(Page::Unlocking);
                }
            }
            6..=8 => {
                self.flow.clear_password();
                self.notice = "Niet beschikbaar. Er wordt geen sessie ontsloten.";
                let _ = UnavailableBroker;
            }
            9 => {
                self.reduced_motion = true;
                self.notice = "Deze native slice gebruikt geen animaties.";
            }
            10 | 13 => self.notice = "Deze functie is nog niet met een systeemdienst verbonden.",
            11 => self.open(Page::Settings),
            12 => return Command::Shutdown,
            20 | 21 => {
                self.hidden[(action - 20) as usize] = false;
                self.open(Page::Desktop);
                return Command::Activate((action - 20) as usize);
            }
            _ => {}
        }
        Command::None
    }
    pub fn key(&mut self, key: KeyboardEvent, width: u32, height: u32, scale: u32) -> Effect {
        if key.action == KeyAction::Release {
            if matches!(key.key, PhysicalKey::LeftSuper | PhysicalKey::RightSuper) {
                let open = self.page == Page::Desktop && !self.super_chord;
                self.super_chord = false;
                if open {
                    self.open(Page::Search);
                    return Effect::take(true, Command::None);
                }
            }
            return if self.blocks_apps() {
                Effect::take(false, Command::None)
            } else {
                Effect::pass()
            };
        }
        if !matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
            return Effect::pass();
        }
        if self.page == Page::Desktop {
            if key.modifiers.super_key()
                && !matches!(key.key, PhysicalKey::LeftSuper | PhysicalKey::RightSuper)
            {
                self.super_chord = true;
            }
            if key.modifiers.super_key() && key.key == PhysicalKey::L {
                self.lock();
                return Effect::take(true, Command::None);
            }
            if key.key == PhysicalKey::F3 {
                self.open(Page::Search);
                return Effect::take(true, Command::None);
            }
            if key.key == PhysicalKey::Menu {
                self.open(Page::Session);
                return Effect::take(true, Command::None);
            }
            return Effect::pass();
        }
        if key.key == PhysicalKey::Escape {
            match self.page {
                Page::Users | Page::Auth => {
                    let _ = self.flow.cancel();
                    self.lock();
                }
                // Apps may never deliver a complete frame; Escape must cancel.
                Page::Unlocking => self.lock(),
                Page::Search
                | Page::Settings
                | Page::Session
                | Page::Notifications
                | Page::Capability => self.open(Page::Desktop),
                _ => {}
            }
            return Effect::take(true, Command::None);
        }
        let count = self.controls(width, height, scale).iter().flatten().count();
        if key.key == PhysicalKey::Tab || key.key == PhysicalKey::Down || key.key == PhysicalKey::Up
        {
            if count != 0 {
                self.focus = if key.modifiers.shift() || key.key == PhysicalKey::Up {
                    (self.focus + count - 1) % count
                } else {
                    (self.focus + 1) % count
                };
            }
            return Effect::take(true, Command::None);
        }
        if key.key == PhysicalKey::Enter
            || key.key == PhysicalKey::Space && self.page != Page::Search
        {
            let control = self
                .controls(width, height, scale)
                .into_iter()
                .flatten()
                .nth(self.focus);
            let action = control.and_then(|c| {
                if c.action == 14 {
                    self.results()[0].map(|(_, _, action)| action)
                } else {
                    Some(c.action)
                }
            });
            let command = action.map_or(Command::None, |action| self.activate(action));
            return Effect::take(true, command);
        }
        if self.page == Page::Search {
            if key.key == PhysicalKey::Backspace && self.query_len > 0 {
                self.query_len -= 1;
                self.query[self.query_len] = 0;
            } else if let Some(byte) = character(key.key) {
                if self.query_len < self.query.len() {
                    self.query[self.query_len] = byte;
                    self.query_len += 1;
                }
            }
            self.focus = 0;
            return Effect::take(true, Command::None);
        }
        // No personal credential is collected before a real provider exists.
        Effect::take(false, Command::None)
    }
    pub fn mouse(
        &mut self,
        position: (u32, u32),
        down: bool,
        held: bool,
        width: u32,
        height: u32,
        scale: u32,
    ) -> Effect {
        let (x, y) = position;
        if self.page == Page::Desktop {
            if y < 44 * scale && down {
                self.open(if x < 180 * scale {
                    Page::Search
                } else if x > width - 160 * scale {
                    Page::Session
                } else {
                    Page::Notifications
                });
                return Effect::take(true, Command::None);
            }
            let bar_y = height.saturating_sub(64 * scale);
            if y >= bar_y {
                if down {
                    if x < 180 * scale {
                        self.open(Page::Search);
                    } else if x < 350 * scale {
                        self.hidden[0] = false;
                        return Effect::take(true, Command::Activate(0));
                    } else if x < 540 * scale {
                        self.hidden[1] = false;
                        return Effect::take(true, Command::Activate(1));
                    } else if x > width - 180 * scale {
                        self.open(Page::Settings);
                    }
                    return Effect::take(true, Command::None);
                }
                return Effect::take(false, Command::None);
            }
            return Effect::pass();
        }
        let controls = self.controls(width, height, scale);
        let hit = controls
            .iter()
            .flatten()
            .position(|c| c.bounds.contains(x as i32, y as i32));
        let action = hit
            .and_then(|i| controls.into_iter().flatten().nth(i))
            .map(|c| c.action);
        if down {
            self.pressed = action;
            if let Some(i) = hit {
                self.focus = i;
            }
        } else if !held {
            let pressed = self.pressed.take();
            if pressed.is_some() && pressed == action {
                return Effect::take(true, self.activate(action.unwrap()));
            }
        }
        Effect::take(down || !held, Command::None)
    }
    pub fn cancel_pointer(&mut self) {
        self.pressed = None;
    }
    pub fn render(&self, c: &mut Canvas<'_>, width: u32, height: u32, scale: u32, focused: usize) {
        self.render_with_hover(c, width, height, scale, focused, None);
    }
    pub fn render_with_hover(
        &self,
        c: &mut Canvas<'_>,
        width: u32,
        height: u32,
        scale: u32,
        focused: usize,
        hovered: Option<usize>,
    ) {
        let s = scale as i32;
        let body = Font::new(Size::Body, Weight::Regular, scale == 2);
        let bold = Font::new(Size::Body, Weight::Semibold, scale == 2);
        let small = Font::new(Size::Small, Weight::Regular, scale == 2);
        let title = Font::new(Size::Title, Weight::Semibold, scale == 2);
        let r = |x: i32, y: i32, w: u32, h: u32| Rect::new(x * s, y * s, w * scale, h * scale);
        c.fill_rect(Rect::new(0, 0, width, 44 * scale), 0x18242f);
        c.text(24 * s, 12 * s, "Nieuw OS", bold, 0xf4f4ef);
        c.text(136 * s, 12 * s, "Werkruimte", body, 0xc5ccc9);
        c.text(
            (width / scale) as i32 * s - 216 * s,
            12 * s,
            "Offline",
            small,
            0xd6d9d6,
        );
        c.text(
            (width / scale) as i32 * s - 128 * s,
            10 * s,
            self.clock(),
            bold,
            0xf4f4ef,
        );
        c.icon(
            Rect::new(width as i32 - 42 * s, 13 * s, 18 * scale, 18 * scale),
            Icon::User,
            0xf4f4ef,
            scale,
        );
        if self.page == Page::Desktop {
            let y = (height / scale) as i32 - 64;
            c.fill_rect(r(0, y, width / scale, 64), DOCK_BAR);
            for (x, w, label, icon, index) in [
                (16, 152, "Zoeken", Icon::Search, 2),
                (188, 152, "Materiaal", Icon::Folder, 0),
                (360, 172, "Rekenmachine", Icon::Apps, 1),
            ] {
                if hovered == Some(index) && focused != index {
                    c.rounded_rect(r(x, y + 8, w, 48), 8 * scale, DOCK_HOVER);
                }
                if focused == index && !self.hidden[index] {
                    c.rounded_rect(r(x, y + 8, w, 48), 8 * scale, DOCK_ACTIVE);
                    c.fill_rect(r(x + 16, y + 56, w - 32, 3), DOCK_ACCENT);
                }
                c.icon(r(x + 12, y + 20, 20, 20), icon, DOCK_ICON, scale);
                c.text((x + 42) * s, (y + 18) * s, label, bold, DOCK_TEXT);
                // Veld minimized marker: shape and position, not color alone.
                if index < self.hidden.len() && self.hidden[index] {
                    let marker = r(x + 16, y + 44, 12, 6);
                    c.fill_rect(marker, DOCK_MARKER);
                    c.fill_rect(
                        Rect::new(marker.x + s, marker.y + s, 10 * scale, 4 * scale),
                        if hovered == Some(index) {
                            DOCK_HOVER
                        } else {
                            DOCK_BAR
                        },
                    );
                }
            }
            c.icon(
                Rect::new(width as i32 - 158 * s, (y + 22) * s, 20 * scale, 20 * scale),
                Icon::Settings,
                DOCK_ICON,
                scale,
            );
            c.text(
                width as i32 - 128 * s,
                (y + 20) * s,
                "Systeem",
                body,
                DOCK_TEXT,
            );
            return;
        }
        // Opaque trusted layer is painted after all app pixels; apps never
        // receive its hit targets, labels, badge or input queue.
        c.fill_rect(Rect::new(0, 44 * s, width, height - 44 * scale), 0x18242f);
        let p = self.panel(width, height, scale);
        if self.page == Page::Lock {
            let clock = Font::display(scale == 2);
            let text = if self.clock_valid {
                self.clock()
            } else {
                "Welkom"
            };
            let clock_y = p.y + 40 * s;
            let date_y = clock_y + clock.height() as i32 + 16 * s;
            c.text(p.x + 28 * s, clock_y, text, clock, 0xf2f1ec);
            c.text(
                p.x + 28 * s,
                date_y + 30 * s,
                if self.clock_valid {
                    "Systeemtijd · UTC"
                } else {
                    "Datum en tijd nog niet beschikbaar"
                },
                body,
                0xc5ccc9,
            );
            if self.clock_valid {
                c.text(p.x + 28 * s, date_y, self.date(), body, 0xc5ccc9);
            }
            c.text(
                p.x + 28 * s,
                date_y + 74 * s,
                "Je werk heeft een eigen plek.",
                title,
                0xf2f1ec,
            );
        } else {
            c.rounded_rect(p, 12 * scale, 0xf2f1ec);
            let heading = match self.page {
                Page::Users => "Wie gaat er werken?",
                Page::Auth => "Aanmelden",
                Page::Unlocking => "Werkruimte openen",
                Page::Search => "Zoeken en starten",
                Page::Settings => "Snelle instellingen",
                Page::Notifications => "Meldingen",
                Page::Session => "Sessie",
                Page::Capability => "Toegang tot materiaal",
                _ => "Nieuw OS",
            };
            c.text(p.x + 28 * s, p.y + 24 * s, heading, title, 0x20262d);
            let detail = match self.page {
                Page::Users => "Kies een profiel op dit apparaat.",
                Page::Auth if self.flow.selected_account() == Some(Account::VmGuest) => {
                    "Openbare VM-gast · geen persoonlijke sleutels"
                }
                Page::Auth => "Nog geen accountdienst. Voer geen wachtwoord in.",
                Page::Unlocking => "De lokale testwerkruimte wordt zichtbaar.",
                Page::Search => "Apps, materiaal, instellingen en acties",
                Page::Settings => "Lokale bediening, zonder leveranciersaccount.",
                Page::Notifications => "Geen nieuwe meldingen.",
                Page::Session => "Openbare VM-testsessie",
                Page::Capability => "Systeemoppervlak · aanvraagvoorbeeld, geen grant",
                _ => "",
            };
            c.text(p.x + 28 * s, p.y + 76 * s, detail, body, 0x5c6570);
            if self.page == Page::Capability {
                for (n, label) in [
                    "Programma: Materiaal",
                    "Materiaal: gekozen document",
                    "Handeling: lezen · bestemming: dit apparaat",
                    "Duur: deze handeling · later intrekbaar",
                ]
                .iter()
                .enumerate()
                {
                    c.text(
                        p.x + 28 * s,
                        p.y + (120 + n as i32 * 34) * s,
                        label,
                        body,
                        0x20262d,
                    );
                }
            }
        }
        for (i, control) in self
            .controls(width, height, scale)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let b = control.bounds;
            c.rounded_rect(
                b,
                8 * scale,
                if i == self.focus { 0xffffff } else { 0xe6e8e7 },
            );
            if i == self.focus {
                c.stroke_rounded_rect(b, 8 * scale, 3 * scale, 0x275dad);
            }
            c.icon(
                Rect::new(b.x + 12 * s, b.y + 12 * s, 20 * scale, 20 * scale),
                control.icon,
                0x30434b,
                scale,
            );
            let mut label = if control.action == 14 && !self.query().is_empty() {
                self.query()
            } else {
                control.label
            };
            let available = b.width.saturating_sub(72 * scale);
            // Search is bounded ASCII input; keep the caret and newest query
            // visible instead of spilling text beyond the trusted input box.
            if control.action == 14 {
                while !label.is_empty() && body.measure(label) > available {
                    // Current queries are ASCII; preserve the boundary even
                    // if a future placeholder is localized.
                    let first = label.chars().next().unwrap().len_utf8();
                    label = &label[first..];
                }
            }
            c.text(
                b.x + 44 * s,
                b.y + 12 * s,
                label,
                body,
                if control.action == 14 && self.query().is_empty() {
                    0x5c6570
                } else {
                    0x20262d
                },
            );
            if control.action == 14 && i == self.focus {
                let caret = b.x + 46 * s + body.measure(label) as i32;
                if !self.query().is_empty() {
                    c.line(caret, b.y + 12 * s, caret, b.y + 32 * s, scale, 0x20262d);
                }
            }
        }
        if !self.notice.is_empty() {
            c.text(
                p.x + 28 * s,
                p.y + p.height as i32 - 22 * s,
                self.notice,
                small,
                0xa13b32,
            );
        }
        c.icon(
            r(24, (height / scale) as i32 - 34, 16, 16),
            Icon::Shield,
            0xc5ccc9,
            scale,
        );
        c.text(
            52 * s,
            height as i32 - 34 * s,
            "VM-preview · schermbediening is geen accountbeveiliging",
            small,
            0xc5ccc9,
        );
    }
}
fn leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn character(key: PhysicalKey) -> Option<u8> {
    use PhysicalKey::*;
    Some(match key {
        A => b'a',
        B => b'b',
        C => b'c',
        D => b'd',
        E => b'e',
        F => b'f',
        G => b'g',
        H => b'h',
        I => b'i',
        J => b'j',
        K => b'k',
        L => b'l',
        M => b'm',
        N => b'n',
        O => b'o',
        P => b'p',
        Q => b'q',
        R => b'r',
        S => b's',
        T => b't',
        U => b'u',
        V => b'v',
        W => b'w',
        X => b'x',
        Y => b'y',
        Z => b'z',
        Space => b' ',
        _ => return None,
    })
}

#[cfg(test)]
mod visual_interaction_tests {
    use super::*;
    fn press(s: &mut Shell, key: PhysicalKey) -> Command {
        s.key(
            KeyboardEvent {
                key,
                action: KeyAction::Press,
                modifiers: Default::default(),
            },
            1280,
            720,
            1,
        )
        .command
    }
    #[test]
    fn search_field_click_edits_and_enter_launches_current_result() {
        let mut shell = Shell::new(false);
        shell.open(Page::Search);
        let field = shell.controls(1280, 720, 1)[0].unwrap();
        let point = ((field.bounds.x + 8) as u32, (field.bounds.y + 8) as u32);
        assert_eq!(
            shell.mouse(point, true, true, 1280, 720, 1).command,
            Command::None
        );
        assert_eq!(
            shell.mouse(point, false, false, 1280, 720, 1).command,
            Command::None
        );
        assert_eq!(shell.page(), Page::Search);
        for key in [PhysicalKey::R, PhysicalKey::E, PhysicalKey::K] {
            press(&mut shell, key);
        }
        assert_eq!(shell.query(), "rek");
        assert_eq!(press(&mut shell, PhysicalKey::Enter), Command::Activate(1));
        assert_eq!(shell.page(), Page::Desktop);
    }
    #[test]
    fn search_tab_reaches_results_and_reverse_tab_returns_to_field() {
        let mut shell = Shell::new(false);
        shell.open(Page::Search);
        assert_eq!(shell.focus, 0);
        press(&mut shell, PhysicalKey::Tab);
        assert_eq!(shell.focus, 1);
        shell.key(
            KeyboardEvent {
                key: PhysicalKey::Tab,
                action: KeyAction::Press,
                modifiers: ui_core::KeyboardModifiers {
                    left_shift: true,
                    ..Default::default()
                },
            },
            1280,
            720,
            1,
        );
        assert_eq!(shell.focus, 0);
        assert_eq!(press(&mut shell, PhysicalKey::Enter), Command::Activate(0));
    }
}
