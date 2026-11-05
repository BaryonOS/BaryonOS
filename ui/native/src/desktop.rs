//! Optional VM desktop. This is app-local navigation, never a lock or a grant.
//! Real accounts remain closed until a separate authenticated broker exists.
use crate::{
    controller::us_character,
    session::{Account, AccountId, SessionFlow, UnavailableBroker},
    Rect, Viewport,
};
use nieuw_os_input_core::Event;
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Accounts,
    Login,
    Home,
    Search,
    Material,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    Guest,
    Personal,
    Back,
    Password,
    Submit,
    Fingerprint,
    Fido2,
    Start,
    Search,
    SearchField,
    Material,
    /// The taskbar's pinned Materiaal button; opens the same app as the card.
    Pinned,
    SignOut,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    None,
    OpenMaterial,
}

/// The floating command dock: one centred capsule, never a full-width bar.
pub fn dock(width: u32, height: u32) -> Rect {
    let dock_width = width.saturating_sub(32).min(760);
    Rect {
        x: (width - dock_width) / 2,
        y: height.saturating_sub(64),
        width: dock_width,
        height: 56,
    }
}

pub struct Desktop {
    flow: SessionFlow,
    page: Page,
    focus: Target,
    pressed: Option<Target>,
    query: [u8; 64],
    query_len: usize,
    unavailable: bool,
}
impl Default for Desktop {
    fn default() -> Self {
        Self::new()
    }
}
impl Desktop {
    pub fn new() -> Self {
        Self {
            flow: SessionFlow::new(),
            page: Page::Accounts,
            focus: Target::Guest,
            pressed: None,
            query: [0; 64],
            query_len: 0,
            unavailable: false,
        }
    }
    pub const fn page(&self) -> Page {
        self.page
    }
    pub const fn focus(&self) -> Target {
        self.focus
    }
    pub fn overlay(&self) -> bool {
        self.page != Page::Material
    }
    pub fn is_guest(&self) -> bool {
        self.flow.selected_account() == Some(Account::VmGuest)
    }
    pub fn password_len(&self) -> usize {
        self.flow.password_len()
    }
    pub const fn unavailable(&self) -> bool {
        self.unavailable
    }
    pub fn query(&self) -> &str {
        core::str::from_utf8(&self.query[..self.query_len]).unwrap_or("")
    }
    pub fn matches_material(&self) -> bool {
        let needle = self.query();
        "materiaal bestanden documenten"
            .as_bytes()
            .windows(needle.len().max(1))
            .any(|candidate| candidate.eq_ignore_ascii_case(needle.as_bytes()))
            || needle.is_empty()
    }
    pub fn cancel_input(&mut self) {
        self.pressed = None;
        self.flow.clear_password();
    }
    pub fn set_query(&mut self, value: &str) -> bool {
        if self.page != Page::Search
            || value.len() > self.query.len()
            || !value.bytes().all(|b| b.is_ascii() && !b.is_ascii_control())
        {
            return false;
        }
        self.query.fill(0);
        self.query[..value.len()].copy_from_slice(value.as_bytes());
        self.query_len = value.len();
        true
    }
    pub fn set_password(&mut self, value: &str) -> bool {
        if self.page != Page::Login
            || self.is_guest()
            || self.flow.phase() != crate::session::Phase::CredentialEntry
            || self.flow.method() != crate::session::AuthMethod::Password
            || value.len() > crate::session::MAX_PASSWORD_BYTES
            || value
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return false;
        }
        self.flow.clear_password();
        for byte in value.bytes() {
            if self.flow.push_password_byte(byte).is_err() {
                self.flow.clear_password();
                return false;
            }
        }
        self.focus = Target::Password;
        true
    }
    pub fn cancel_pointer(&mut self) {
        self.pressed = None;
    }
    pub fn home(&mut self) {
        self.page = Page::Home;
        self.focus = Target::Material;
        self.pressed = None;
        self.query.fill(0);
        self.query_len = 0;
    }
    pub fn controls(&self, viewport: Viewport) -> [Option<(Target, Rect, &'static str)>; 7] {
        let w = viewport.logical_width();
        let h = viewport.logical_height();
        let compact = h < 420 || w < 640;
        let width = w.saturating_sub(48).min(440);
        let x = (w - width) / 2;
        let y = if compact {
            0
        } else {
            h.saturating_sub(440) / 2
        };
        let r = |dy, height| Rect {
            x,
            y: y + dy,
            width,
            height,
        };
        match self.page {
            Page::Accounts => [
                Some((
                    Target::Guest,
                    r(
                        if compact { 84 } else { 140 },
                        if compact { 56 } else { 76 },
                    ),
                    "Gast (VM-test)",
                )),
                Some((
                    Target::Personal,
                    r(
                        if compact { 152 } else { 232 },
                        if compact { 56 } else { 76 },
                    ),
                    "Eigen gebruiker",
                )),
                None,
                None,
                None,
                None,
                None,
            ],
            Page::Login => [
                Some((
                    Target::Back,
                    Rect {
                        x,
                        y: y + if compact { 4 } else { 12 },
                        width: 44,
                        height: 44,
                    },
                    "Terug",
                )),
                Some((
                    Target::Password,
                    r(if compact { 76 } else { 158 }, 44),
                    "Wachtwoord",
                )),
                Some((
                    Target::Submit,
                    r(if compact { 132 } else { 224 }, 48),
                    if self.is_guest() {
                        "Testsessie starten"
                    } else {
                        "Inloggen"
                    },
                )),
                Some((
                    Target::Fingerprint,
                    Rect {
                        x,
                        y: y + if compact { 188 } else { 296 },
                        width: width / 2 - 4,
                        height: 44,
                    },
                    "Vingerafdruk",
                )),
                Some((
                    Target::Fido2,
                    Rect {
                        x: x + width / 2 + 4,
                        y: y + if compact { 188 } else { 296 },
                        width: width / 2 - 4,
                        height: 44,
                    },
                    "Beveiligingssleutel",
                )),
                None,
                None,
            ],
            Page::Home | Page::Search | Page::Material => {
                let dock = dock(w, h);
                let roomy = dock.width >= 560;
                let power_x = dock.x + dock.width - 50;
                let pinned_x = power_x - 8 - if roomy { 112 } else { 0 } - 44;
                let panel_width = w.saturating_sub(48).min(620);
                let panel_x = (w - panel_width) / 2;
                let top = if compact { 40 } else { 132 };
                let card_width = w.saturating_sub(64).min(704);
                let button = |x| Rect {
                    x,
                    y: dock.y + 6,
                    width: 44,
                    height: 44,
                };
                [
                    // The horizon mark opens the overview; deliberately no "Start".
                    Some((Target::Start, button(dock.x + 6), "Overzicht")),
                    Some((
                        Target::Search,
                        Rect {
                            x: dock.x + 58,
                            y: dock.y + 6,
                            width: pinned_x - 8 - (dock.x + 58),
                            height: 44,
                        },
                        "Zoeken",
                    )),
                    Some((Target::Pinned, button(pinned_x), "Materiaal")),
                    Some((Target::SignOut, button(power_x), "Afmelden")),
                    if self.page == Page::Search {
                        Some((
                            Target::SearchField,
                            Rect {
                                x: panel_x + 16,
                                y: top + 16,
                                width: panel_width - 32,
                                height: 44,
                            },
                            "Zoek programma's",
                        ))
                    } else {
                        None
                    },
                    if self.page != Page::Search || self.matches_material() {
                        Some((
                            Target::Material,
                            if self.page == Page::Search {
                                Rect {
                                    x: panel_x + 16,
                                    y: top + 76,
                                    width: panel_width - 32,
                                    height: if compact { 52 } else { 72 },
                                }
                            } else {
                                Rect {
                                    x: (w - card_width) / 2,
                                    // Centred composition on large screens.
                                    y: if compact {
                                        64
                                    } else {
                                        h.saturating_sub(264).div_ceil(2).max(170)
                                    },
                                    width: card_width,
                                    height: if compact {
                                        96
                                    } else {
                                        h.saturating_sub(238).min(200)
                                    },
                                }
                            },
                            "Materiaal",
                        ))
                    } else {
                        None
                    },
                    None,
                ]
            }
        }
    }

    pub fn activate(&mut self, target: Target) -> Action {
        self.unavailable = false;
        self.pressed = None;
        match target {
            Target::Guest | Target::Personal if self.page == Page::Accounts => {
                let account = if target == Target::Guest {
                    Account::VmGuest
                } else {
                    Account::LocalUser(AccountId::new(1).unwrap())
                };
                if self.flow.select_account(account).is_ok() {
                    self.page = Page::Login;
                    self.focus = if target == Target::Guest {
                        Target::Submit
                    } else {
                        Target::Password
                    };
                }
            }
            Target::Back if self.page == Page::Login => {
                let _ = self.flow.cancel();
                self.page = Page::Accounts;
                self.focus = Target::Guest;
            }
            Target::Submit if self.page == Page::Login => {
                if self.is_guest() {
                    if self.flow.begin_guest().is_ok() && self.flow.activate_guest().is_ok() {
                        self.home();
                    }
                } else {
                    let _ = self.flow.submit(&mut UnavailableBroker);
                    self.unavailable = true;
                }
            }
            Target::Fingerprint | Target::Fido2 if self.page == Page::Login => {
                self.flow.clear_password();
                self.unavailable = true;
            }
            Target::Start if matches!(self.page, Page::Home | Page::Search) => self.home(),
            Target::Search if matches!(self.page, Page::Home | Page::Search) => {
                self.page = Page::Search;
                self.focus = Target::SearchField;
            }
            Target::Material
                if matches!(self.page, Page::Home | Page::Search) && self.matches_material() =>
            {
                self.page = Page::Material;
                return Action::OpenMaterial;
            }
            Target::Pinned if matches!(self.page, Page::Home | Page::Search) => {
                self.page = Page::Material;
                return Action::OpenMaterial;
            }
            Target::SignOut if matches!(self.page, Page::Home | Page::Search) => {
                let _ = self.flow.sign_out();
                self.query.fill(0);
                self.query_len = 0;
                self.page = Page::Accounts;
                self.focus = Target::Guest;
            }
            _ => {}
        }
        Action::None
    }
    fn key(&mut self, key: KeyboardEvent, viewport: Viewport) -> Action {
        if !matches!(key.action, KeyAction::Press | KeyAction::Repeat) {
            return Action::None;
        }
        if key.key == PhysicalKey::Escape {
            if self.page == Page::Login {
                return self.activate(Target::Back);
            }
            if self.page == Page::Search {
                self.home();
            }
            return Action::None;
        }
        if key.key == PhysicalKey::Tab && key.action == KeyAction::Press {
            let controls = self.controls(viewport);
            let mut targets = [Target::Guest; 7];
            let mut count = 0;
            for (target, _, _) in controls.into_iter().flatten() {
                if target == Target::Password && self.is_guest() {
                    continue;
                }
                if !targets[..count].contains(&target) {
                    targets[count] = target;
                    count += 1;
                }
            }
            if count != 0 {
                let index = targets[..count]
                    .iter()
                    .position(|t| *t == self.focus)
                    .unwrap_or(0);
                self.focus =
                    targets[(index + if key.modifiers.shift() { count - 1 } else { 1 }) % count];
            }
            return Action::None;
        }
        if matches!(key.key, PhysicalKey::Enter | PhysicalKey::KeypadEnter)
            && key.action == KeyAction::Press
        {
            return self.activate(if self.focus == Target::Password {
                Target::Submit
            } else if self.page == Page::Search && self.focus == Target::SearchField {
                Target::Material
            } else {
                self.focus
            });
        }
        if key.modifiers.control() || key.modifiers.alt() {
            return Action::None;
        }
        if self.page == Page::Login && self.focus == Target::Password && !self.is_guest() {
            if key.key == PhysicalKey::Backspace {
                let _ = self.flow.backspace_password();
            } else if let Some(byte) =
                us_character(key.key, key.modifiers.shift(), key.modifiers.caps_lock)
            {
                let _ = self.flow.push_password_byte(byte);
            }
        } else if self.page == Page::Search && self.focus == Target::SearchField {
            if key.key == PhysicalKey::Backspace {
                self.query_len = self.query_len.saturating_sub(1);
                self.query[self.query_len] = 0;
            } else if let Some(byte) =
                us_character(key.key, key.modifiers.shift(), key.modifiers.caps_lock)
            {
                if self.query_len < self.query.len() {
                    self.query[self.query_len] = byte;
                    self.query_len += 1;
                }
            }
        }
        Action::None
    }
    pub fn deliver(&mut self, event: Event, viewport: Viewport) -> Action {
        match event {
            Event::Key(key) => self.key(key, viewport),
            Event::Cancel(_) => {
                self.cancel_pointer();
                Action::None
            }
            Event::Mouse { input, x, y } => {
                let hit =
                    self.controls(viewport)
                        .into_iter()
                        .flatten()
                        .find_map(|(target, rect, _)| {
                            rect.physical(viewport.scale())
                                .contains(x, y)
                                .then_some(target)
                        });
                if input.changed.left() && input.buttons.left() {
                    self.pressed = hit;
                    if let Some(target) = hit {
                        self.focus = target;
                    }
                }
                if input.changed.left() && !input.buttons.left() {
                    let pressed = self.pressed.take();
                    if let Some(target) = pressed.filter(|pressed| Some(*pressed) == hit) {
                        return self.activate(target);
                    }
                }
                Action::None
            }
        }
    }
}
