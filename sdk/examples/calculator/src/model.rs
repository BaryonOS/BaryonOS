//! Local calculator state. Only authenticated INP1 events reach this model in
//! the native adapter. No input envelope, PID or grant can be forged here.
use crate::decimal::{self, Evaluation, MAX_INPUT};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

pub const LABELS: [&str; 20] = [
    "7", "8", "9", "/", "C", "4", "5", "6", "*", "Wis", "1", "2", "3", "-", "(", "0", ",", "=",
    "+", ")",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Insert(u8),
    Evaluate,
    Reset,
    Cancel,
    Backspace,
    Delete,
    Left(bool),
    Right(bool),
    Home(bool),
    End(bool),
    SelectAll,
    Tab(bool),
    Activate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    None,
    Arithmetic(decimal::Error),
    TooLong,
    InputLost,
    ClipboardCopied,
    ClipboardEmpty,
    ClipboardSelection,
    ClipboardTooLong,
    ClipboardUnavailable,
    ClipboardUnsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteError {
    Empty,
    TooLong,
    Unsupported,
    Inactive,
}
impl Notice {
    pub const fn label(self) -> &'static str {
        use decimal::Kind;
        match self {
            Self::None => "",
            Self::TooLong => "Maximaal 256 tekens.",
            Self::InputLost => "Invoerverbinding verbroken.",
            Self::ClipboardCopied => "Tekst gekopieerd.",
            Self::ClipboardEmpty => "Het klembord is leeg.",
            Self::ClipboardSelection => "Selecteer eerst tekst.",
            Self::ClipboardTooLong => "Deze selectie past nog niet op het klembord.",
            Self::ClipboardUnavailable => "Kopiëren of plakken onderbroken. Probeer opnieuw.",
            Self::ClipboardUnsupported => "Plak cijfers, + - * / ( ) en een komma of punt.",
            Self::Arithmetic(error) => match error.kind {
                Kind::Range => "Maximaal 10 gehele cijfers.",
                Kind::Nesting => "Maximaal 32 geneste tekens.",
                Kind::Unsupported => "Dit teken kan hier niet.",
                Kind::TooLong => "Maximaal 256 tekens.",
                _ => error.label(),
            },
        }
    }
}
#[derive(Clone, Debug)]
pub struct Model {
    bytes: [u8; MAX_INPUT],
    length: usize,
    caret: usize,
    anchor: usize,
    answer: Option<Evaluation>,
    current: bool,
    notice: Notice,
    focus: usize,
    active: bool,
    hover: Option<usize>,
    pressed: Option<usize>,
    generation: u64,
    evaluations: u64,
    failures: u64,
    exhausted: bool,
    selecting: bool,
}
impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
impl Model {
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_INPUT],
            length: 0,
            caret: 0,
            anchor: 0,
            answer: None,
            current: false,
            notice: Notice::None,
            focus: 0,
            active: true,
            hover: None,
            pressed: None,
            generation: 1,
            evaluations: 0,
            failures: 0,
            exhausted: false,
            selecting: false,
        }
    }
    pub fn expression(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.length]).expect("calculator input is ASCII")
    }
    pub const fn caret(&self) -> usize {
        self.caret
    }
    pub fn selection(&self) -> core::ops::Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }
    /// App-local selection only; input authority is checked by the broker.
    /// An old answer or the whole expression is never a selection fallback.
    pub fn clipboard_selection(&self) -> &str {
        if self.active && self.focus == 0 {
            &self.expression()[self.selection()]
        } else {
            ""
        }
    }
    pub fn clipboard_notice(&mut self, notice: Notice) {
        if !self.exhausted {
            self.notice = notice;
            self.changed();
        }
    }
    /// Validate every byte and the final size before erasing a selection.
    /// This edits text only; pasted operators never trigger evaluation.
    pub fn paste_clipboard(&mut self, value: &str) -> Result<(), PasteError> {
        let error = if self.exhausted || !self.active || self.focus != 0 {
            Some((PasteError::Inactive, Notice::ClipboardUnavailable))
        } else if value.is_empty() {
            Some((PasteError::Empty, Notice::ClipboardEmpty))
        } else if value.len() > MAX_INPUT
            || self.length - self.selection().len() > MAX_INPUT - value.len()
        {
            Some((PasteError::TooLong, Notice::TooLong))
        } else if !value.bytes().all(|byte| {
            matches!(
                byte,
                b'0'..=b'9' | b'+' | b'-' | b'*' | b'/' | b'(' | b')' | b'.' | b',' | b' '
            )
        }) {
            Some((PasteError::Unsupported, Notice::ClipboardUnsupported))
        } else {
            None
        };
        if let Some((error, notice)) = error {
            self.clipboard_notice(notice);
            return Err(error);
        }
        self.erase(self.selection());
        self.bytes
            .copy_within(self.caret..self.length, self.caret + value.len());
        self.bytes[self.caret..self.caret + value.len()].copy_from_slice(value.as_bytes());
        self.length += value.len();
        self.caret += value.len();
        self.anchor = self.caret;
        self.selecting = false;
        self.edited();
        Ok(())
    }
    pub const fn answer(&self) -> Option<Evaluation> {
        self.answer
    }
    pub const fn current(&self) -> bool {
        self.current
    }
    pub const fn notice(&self) -> Notice {
        self.notice
    }
    pub const fn focus(&self) -> usize {
        self.focus
    }
    pub const fn active(&self) -> bool {
        self.active
    }
    pub const fn hover(&self) -> Option<usize> {
        self.hover
    }
    pub const fn pressed(&self) -> Option<usize> {
        self.pressed
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn evaluations(&self) -> u64 {
        self.evaluations
    }
    pub const fn failures(&self) -> u64 {
        self.failures
    }
    pub const fn exhausted(&self) -> bool {
        self.exhausted
    }
    fn changed(&mut self) {
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next;
        } else {
            self.exhausted = true;
        }
    }
    fn edited(&mut self) {
        self.current = false;
        self.notice = Notice::None;
        self.changed();
    }
    fn erase(&mut self, range: core::ops::Range<usize>) {
        self.bytes.copy_within(range.end..self.length, range.start);
        self.length -= range.end - range.start;
        self.bytes[self.length..].fill(0);
        self.caret = range.start;
        self.anchor = self.caret;
    }
    fn move_to(&mut self, position: usize, select: bool) {
        self.caret = position.min(self.length);
        if !select {
            self.anchor = self.caret;
        }
        self.focus = 0;
        self.changed();
    }
    pub fn command(&mut self, command: Command) {
        if self.exhausted {
            return;
        }
        self.active = true;
        self.selecting = false;
        match command {
            Command::Insert(byte) => {
                if !matches!(
                    byte,
                    b'0'..=b'9' | b'+' | b'-' | b'*' | b'/' | b'(' | b')' | b'.' | b',' | b' '
                ) {
                    return;
                }
                let range = self.selection();
                if self.length - (range.end - range.start) == MAX_INPUT {
                    self.notice = Notice::TooLong;
                    self.changed();
                    return;
                }
                self.erase(range);
                self.bytes
                    .copy_within(self.caret..self.length, self.caret + 1);
                self.bytes[self.caret] = byte;
                self.length += 1;
                self.caret += 1;
                self.anchor = self.caret;
                self.focus = 0;
                self.edited();
            }
            Command::Evaluate => {
                match decimal::evaluate(self.expression()) {
                    Ok(answer) => {
                        self.answer = Some(answer);
                        self.current = true;
                        self.notice = Notice::None;
                        if let Some(next) = self.evaluations.checked_add(1) {
                            self.evaluations = next;
                        } else {
                            self.exhausted = true;
                        }
                    }
                    Err(error) => {
                        self.notice = Notice::Arithmetic(error);
                        self.current = false;
                        if let Some(next) = self.failures.checked_add(1) {
                            self.failures = next;
                        } else {
                            self.exhausted = true;
                        }
                    }
                }
                self.changed();
            }
            Command::Reset => {
                self.bytes.fill(0);
                self.length = 0;
                self.caret = 0;
                self.anchor = 0;
                self.answer = None;
                self.current = false;
                self.notice = Notice::None;
                self.focus = 0;
                self.pressed = None;
                self.changed();
            }
            Command::Cancel => {
                self.bytes.fill(0);
                self.length = 0;
                self.caret = 0;
                self.anchor = 0;
                self.current = false;
                self.notice = Notice::None;
                self.focus = 0;
                self.pressed = None;
                self.changed();
            }
            Command::Backspace | Command::Delete => {
                let mut range = self.selection();
                if range.is_empty() {
                    if command == Command::Backspace {
                        range.start = range.start.saturating_sub(1);
                    } else {
                        range.end = (range.end + 1).min(self.length);
                    }
                }
                if !range.is_empty() {
                    self.erase(range);
                    self.edited();
                }
            }
            Command::Left(select) => self.move_to(
                if !select && !self.selection().is_empty() {
                    self.selection().start
                } else {
                    self.caret.saturating_sub(1)
                },
                select,
            ),
            Command::Right(select) => self.move_to(
                if !select && !self.selection().is_empty() {
                    self.selection().end
                } else {
                    (self.caret + 1).min(self.length)
                },
                select,
            ),
            Command::Home(select) => self.move_to(0, select),
            Command::End(select) => self.move_to(self.length, select),
            Command::SelectAll => {
                self.caret = self.length;
                self.anchor = 0;
                self.focus = 0;
                self.changed();
            }
            Command::Tab(backwards) => {
                self.focus = if backwards {
                    (self.focus + 20) % 21
                } else {
                    (self.focus + 1) % 21
                };
                self.pressed = None;
                self.changed();
            }
            Command::Activate => {
                if self.focus == 0 {
                    self.command(Command::Evaluate)
                } else {
                    self.button(self.focus - 1)
                }
            }
        }
    }
    fn button(&mut self, index: usize) {
        let command = match index {
            4 => Command::Reset,
            9 => Command::Backspace,
            17 => Command::Evaluate,
            _ => match LABELS.get(index) {
                Some(label) => Command::Insert(label.as_bytes()[0]),
                None => return,
            },
        };
        let focus = self.focus;
        self.command(command);
        self.focus = focus;
    }
    pub fn pointer(&mut self, button: Option<usize>, down: bool, up: bool) {
        if self.exhausted {
            return;
        }
        self.active = true;
        self.selecting = false;
        let before = (self.hover, self.pressed, self.focus);
        let button = button.filter(|index| *index < 20);
        self.hover = button;
        if down {
            self.pressed = button;
            if let Some(index) = button {
                self.focus = index + 1;
            }
        }
        if up {
            let activate = self.pressed.take();
            if let Some(index) = activate.filter(|_| activate == button) {
                self.button(index);
            }
        }
        if before != (self.hover, self.pressed, self.focus) {
            self.changed();
        }
    }
    pub fn caret_at(&mut self, position: usize, down: bool, held: bool) {
        if down || self.selecting {
            self.active = true;
        }
        if down {
            self.move_to(position, false);
            self.selecting = true;
        } else if self.selecting {
            self.move_to(position, true);
            if !held {
                self.selecting = false;
            }
        }
        let pressed = self.pressed.take();
        let hover = self.hover.take();
        if pressed.is_some() || hover.is_some() {
            self.changed();
        }
    }
    pub fn cancel_pointer(&mut self) {
        self.selecting = false;
        let pressed = self.pressed.take();
        let hover = self.hover.take();
        if pressed.is_some() || hover.is_some() {
            self.changed();
        }
    }
    pub fn cancel_focus(&mut self) {
        self.cancel_pointer();
        if self.active {
            self.active = false;
            self.changed();
        }
    }
    /// Called only after the typed client authenticates a fresh non-cancel
    /// offer. Empty PEEK is not evidence that this window has input focus.
    pub fn focus_gained(&mut self) {
        if !self.active {
            self.active = true;
            self.changed();
        }
    }
    pub fn input_lost(&mut self) {
        self.cancel_focus();
        self.notice = Notice::InputLost;
        self.changed();
    }
    pub fn keyboard(&mut self, event: KeyboardEvent) {
        if !matches!(event.action, KeyAction::Press | KeyAction::Repeat) {
            return;
        }
        let m = event.modifiers;
        if m.alt() || m.super_key() {
            return;
        }
        let key = event.key;
        if m.control() {
            if key == PhysicalKey::A {
                self.command(Command::SelectAll);
            }
            return;
        }
        use PhysicalKey::*;
        let numeric = m.num_lock != m.shift();
        let command = match key {
            Escape => Command::Cancel,
            C => Command::Reset,
            Tab => Command::Tab(m.shift()),
            Enter | KeypadEnter => Command::Activate,
            Space if self.focus != 0 => Command::Activate,
            Backspace => Command::Backspace,
            Delete => Command::Delete,
            Equal if !m.shift() => Command::Evaluate,
            Left => Command::Left(m.shift()),
            Right => Command::Right(m.shift()),
            Home => Command::Home(m.shift()),
            End => Command::End(m.shift()),
            Keypad4 if !numeric => Command::Left(false),
            Keypad6 if !numeric => Command::Right(false),
            Keypad7 if !numeric => Command::Home(false),
            Keypad1 if !numeric => Command::End(false),
            KeypadDecimal if !numeric => Command::Delete,
            _ => {
                let byte = match key {
                    Digit0 => {
                        if m.shift() {
                            b')'
                        } else {
                            b'0'
                        }
                    }
                    Digit1 if !m.shift() => b'1',
                    Digit2 if !m.shift() => b'2',
                    Digit3 if !m.shift() => b'3',
                    Digit4 if !m.shift() => b'4',
                    Digit5 if !m.shift() => b'5',
                    Digit6 if !m.shift() => b'6',
                    Digit7 if !m.shift() => b'7',
                    Digit8 => {
                        if m.shift() {
                            b'*'
                        } else {
                            b'8'
                        }
                    }
                    Digit9 => {
                        if m.shift() {
                            b'('
                        } else {
                            b'9'
                        }
                    }
                    Minus if !m.shift() => b'-',
                    Equal if m.shift() => b'+',
                    Slash if !m.shift() => b'/',
                    KeypadDivide => b'/',
                    KeypadMultiply => b'*',
                    KeypadSubtract => b'-',
                    KeypadAdd => b'+',
                    Period if !m.shift() => b'.',
                    Comma if !m.shift() => b',',
                    KeypadDecimal if numeric => b',',
                    Space => b' ',
                    Keypad0 if numeric => b'0',
                    Keypad1 if numeric => b'1',
                    Keypad2 if numeric => b'2',
                    Keypad3 if numeric => b'3',
                    Keypad4 if numeric => b'4',
                    Keypad5 if numeric => b'5',
                    Keypad6 if numeric => b'6',
                    Keypad7 if numeric => b'7',
                    Keypad8 if numeric => b'8',
                    Keypad9 if numeric => b'9',
                    _ => return,
                };
                Command::Insert(byte)
            }
        };
        // A repeated Enter/Tab is a real input repeat, but cannot generate an
        // unbounded set of evaluations or jump focus without deliberate presses.
        if event.action == KeyAction::Repeat
            && matches!(
                command,
                Command::Activate
                    | Command::Evaluate
                    | Command::Tab(_)
                    | Command::Cancel
                    | Command::Reset
            )
        {
            return;
        }
        self.command(command);
    }
}
