//! Local calculator state. Only authenticated INP1 events reach this model in
//! the native adapter. No input envelope, PID or grant can be forged here.
use crate::decimal::{self, Evaluation, MAX_INPUT};
use crate::render::{Layout, Rect, Viewport};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

pub const LABELS: [&str; 20] = [
    "7", "8", "9", "/", "C", "4", "5", "6", "*", "Wis", "1", "2", "3", "-", "(", "0", ",", "=",
    "+", ")",
];
pub const RECENT_CAPACITY: usize = 8;
pub const UNDO_CAPACITY: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Insert(u8),
    Evaluate,
    Reset,
    Cancel,
    Backspace,
    Delete,
    RecallOlder,
    RecallNewer,
    Undo,
    Redo,
    Left(bool),
    Right(bool),
    Home(bool),
    End(bool),
    SelectAll,
    SelectResult,
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
    result_bytes: [u8; 32],
    result_length: usize,
    result_selected: bool,
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
    recent: Recent,
    history: History,
    paint: PaintState,
    damage: Damage,
}

#[derive(Clone, Debug)]
struct PaintState {
    bytes: [u8; MAX_INPUT],
    length: usize,
    caret: usize,
    anchor: usize,
    answer: Option<Evaluation>,
    result_selected: bool,
    current: bool,
    notice: Notice,
    focus: usize,
    active: bool,
    hover: Option<usize>,
    pressed: Option<usize>,
    status: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Formula {
    bytes: [u8; MAX_INPUT],
    length: usize,
}
impl Formula {
    const EMPTY: Self = Self {
        bytes: [0; MAX_INPUT],
        length: 0,
    };
}
#[derive(Clone, Copy, Debug)]
struct RecallDraft {
    formula: Formula,
    caret: usize,
    anchor: usize,
}
#[derive(Clone, Debug)]
struct Recent {
    entries: [Formula; RECENT_CAPACITY],
    length: usize,
    browsing: Option<usize>,
    draft: Option<RecallDraft>,
}
impl Recent {
    const fn new() -> Self {
        Self {
            entries: [Formula::EMPTY; RECENT_CAPACITY],
            length: 0,
            browsing: None,
            draft: None,
        }
    }
    fn leave(&mut self) {
        self.browsing = None;
        self.draft = None;
    }
    fn record(&mut self, formula: Formula) {
        self.leave();
        if self.length != 0 && self.entries[self.length - 1] == formula {
            return;
        }
        if self.length == RECENT_CAPACITY {
            self.entries.copy_within(1..RECENT_CAPACITY, 0);
            self.length -= 1;
        }
        self.entries[self.length] = formula;
        self.length += 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EditState {
    bytes: [u8; MAX_INPUT],
    length: usize,
    caret: usize,
    anchor: usize,
}
impl EditState {
    const EMPTY: Self = Self {
        bytes: [0; MAX_INPUT],
        length: 0,
        caret: 0,
        anchor: 0,
    };
}
#[derive(Clone, Debug)]
struct History {
    entries: [EditState; UNDO_CAPACITY],
    length: usize,
    cursor: usize,
}
impl History {
    const fn new() -> Self {
        Self {
            entries: [EditState::EMPTY; UNDO_CAPACITY],
            length: 0,
            cursor: 0,
        }
    }
    fn record(&mut self, before: EditState) {
        self.entries[self.cursor..].fill(EditState::EMPTY);
        if self.cursor == UNDO_CAPACITY {
            self.entries.copy_within(1..UNDO_CAPACITY, 0);
            self.cursor -= 1;
        }
        self.entries[self.cursor] = before;
        self.cursor += 1;
        self.length = self.cursor;
    }
    fn restore(&mut self, mut current: EditState, redo: bool) -> Option<EditState> {
        if redo {
            if self.cursor == self.length {
                return None;
            }
            core::mem::swap(&mut current, &mut self.entries[self.cursor]);
            self.cursor += 1;
        } else {
            if self.cursor == 0 {
                return None;
            }
            self.cursor -= 1;
            core::mem::swap(&mut current, &mut self.entries[self.cursor]);
        }
        Some(current)
    }
}

#[derive(Clone, Debug)]
struct Damage {
    full: bool,
    expression_full: bool,
    span: core::ops::Range<usize>,
    caret_min: usize,
    caret_max: usize,
    result: bool,
    status: bool,
    buttons: u32,
}
impl Damage {
    const fn new(full: bool, caret: usize) -> Self {
        Self {
            full,
            expression_full: false,
            span: 0..0,
            caret_min: caret,
            caret_max: caret,
            result: false,
            status: false,
            buttons: 0,
        }
    }
    fn include(&mut self, start: usize, end: usize) {
        if self.span.is_empty() {
            self.span = start..end;
        } else {
            self.span.start = self.span.start.min(start);
            self.span.end = self.span.end.max(end);
        }
    }
}

/// Physical-pixel bounds; querying damage never acknowledges presentation.
#[derive(Clone, Debug)]
pub struct PaintDamage {
    rects: [Rect; 24],
    length: usize,
}
impl PaintDamage {
    pub fn intersects(&self, x: u32, y: u32, width: u32, height: u32) -> bool {
        width != 0
            && height != 0
            && self.rects[..self.length].iter().any(|r| {
                u64::from(x) < u64::from(r.x) + u64::from(r.width)
                    && u64::from(r.x) < u64::from(x) + u64::from(width)
                    && u64::from(y) < u64::from(r.y) + u64::from(r.height)
                    && u64::from(r.y) < u64::from(y) + u64::from(height)
            })
    }
    fn push(&mut self, rect: Rect, scale: u32) {
        if rect.width != 0 && rect.height != 0 {
            self.rects[self.length] = Rect {
                x: rect.x * scale,
                y: rect.y * scale,
                width: rect.width * scale,
                height: rect.height * scale,
            };
            self.length += 1;
        }
    }
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
            result_bytes: [0; 32],
            result_length: 0,
            result_selected: false,
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
            recent: Recent::new(),
            history: History::new(),
            paint: PaintState {
                bytes: [0; MAX_INPUT],
                length: 0,
                caret: 0,
                anchor: 0,
                answer: None,
                result_selected: false,
                current: false,
                notice: Notice::None,
                focus: 0,
                active: true,
                hover: None,
                pressed: None,
                status: "",
            },
            damage: Damage::new(true, 0),
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
        if self.active && self.result_selected && self.current && self.answer.is_some() {
            core::str::from_utf8(&self.result_bytes[..self.result_length])
                .expect("formatted calculator result is ASCII")
        } else if self.active && self.focus == 0 {
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
        let error = if self.exhausted || !self.active || self.focus != 0 || self.result_selected {
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
        let before = self.edit_state();
        self.erase(self.selection());
        self.bytes
            .copy_within(self.caret..self.length, self.caret + value.len());
        self.bytes[self.caret..self.caret + value.len()].copy_from_slice(value.as_bytes());
        self.length += value.len();
        self.caret += value.len();
        self.anchor = self.caret;
        self.selecting = false;
        self.edited(before);
        Ok(())
    }
    pub const fn answer(&self) -> Option<Evaluation> {
        self.answer
    }
    pub const fn current(&self) -> bool {
        self.current
    }
    pub const fn result_selected(&self) -> bool {
        self.result_selected
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
    pub const fn recent_count(&self) -> usize {
        self.recent.length
    }
    pub const fn recalling(&self) -> bool {
        self.recent.browsing.is_some()
    }
    pub const fn can_undo(&self) -> bool {
        self.history.cursor != 0
    }
    pub const fn can_redo(&self) -> bool {
        self.history.cursor < self.history.length
    }
    /// The single status line: notices first, then the most relevant hint.
    pub fn status(&self) -> &'static str {
        if self.notice != Notice::None {
            self.notice.label()
        } else if self.result_selected {
            "Resultaat: Ctrl+C om te kopiëren"
        } else if self.answer.is_some_and(|answer| answer.rounded) && self.current {
            "Afgerond op 9 decimalen."
        } else if self.current {
            "Klik resultaat of Ctrl+R: selecteren"
        } else if self.recalling() && self.focus == 0 {
            "Omlaag: volgende of eigen invoer"
        } else if self.recent_count() != 0 && self.length == 0 {
            "Omhoog: eerdere berekening"
        } else {
            match (self.can_undo(), self.can_redo()) {
                (true, true) => "Ctrl+Z: ongedaan | Ctrl+Y: opnieuw",
                (true, false) => "Ctrl+Z: ongedaan",
                (false, true) => "Ctrl+Y: opnieuw",
                (false, false) => "",
            }
        }
    }
    pub fn paint_damage(&self, viewport: Viewport) -> PaintDamage {
        let mut out = PaintDamage {
            rects: [Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }; 24],
            length: 0,
        };
        if self.damage.full {
            out.push(
                Rect {
                    x: 0,
                    y: 0,
                    width: viewport.width(),
                    height: viewport.height(),
                },
                1,
            );
            return out;
        }
        let layout = Layout::new(viewport);
        let scale = viewport.scale();
        // Proportional glyphs and intermediate scrolls share the whole field.
        if self.damage.expression_full
            || self.damage.caret_min != self.damage.caret_max
            || self.damage.span.start < self.damage.span.end
        {
            out.push(layout.expression, scale);
        }
        for (dirty, bounds) in [
            (self.damage.result, layout.result_bounds(scale)),
            (self.damage.status, layout.status_bounds(scale)),
        ] {
            if dirty {
                out.push(bounds, scale);
            }
        }
        for index in 0..20 {
            if self.damage.buttons & (1 << index) != 0 {
                out.push(layout.button(index), scale);
            }
        }
        out
    }
    /// Only the adapter's complete, acknowledged current frame can clear damage.
    pub fn presented(&mut self, generation: u64) -> bool {
        if self.exhausted || generation != self.generation {
            return false;
        }
        self.damage = Damage::new(false, self.caret);
        true
    }
    pub fn force_full_repaint(&mut self) {
        self.damage.full = true;
        self.changed();
    }
    fn changed(&mut self) {
        let old = &self.paint;
        let length = old.length.max(self.length);
        let first = (0..length).find(|&i| old.bytes[i] != self.bytes[i]);
        if let Some(first) = first {
            let last = (first..length)
                .rfind(|&i| old.bytes[i] != self.bytes[i])
                .unwrap();
            self.damage.include(first, last + 1);
        }
        if old.caret != self.caret || old.anchor != self.anchor {
            self.damage.include(old.caret, old.caret + 1);
            self.damage.include(self.caret, self.caret + 1);
            if old.anchor != old.caret || self.anchor != self.caret {
                self.damage.include(
                    old.anchor.min(old.caret).min(self.anchor.min(self.caret)),
                    old.anchor.max(old.caret).max(self.anchor.max(self.caret)),
                );
            }
        }
        self.damage.caret_min = self.damage.caret_min.min(self.caret);
        self.damage.caret_max = self.damage.caret_max.max(self.caret);
        if (old.focus == 0 && old.active) != (self.focus == 0 && self.active)
            || old.result_selected != self.result_selected
        {
            self.damage.expression_full = true;
        }
        self.damage.result |= old.answer != self.answer
            || old.current != self.current
            || old.result_selected != self.result_selected;
        // Every hint (selection, recall, undo, rounding) is derived from status().
        self.damage.status |= old.notice != self.notice || old.status != self.status();
        for index in 0..20 {
            if (old.focus == index + 1 && old.active) != (self.focus == index + 1 && self.active)
                || (old.hover == Some(index)) != (self.hover == Some(index))
                || (old.pressed == Some(index)) != (self.pressed == Some(index))
            {
                self.damage.buttons |= 1 << index;
            }
        }
        self.paint = PaintState {
            bytes: self.bytes,
            length: self.length,
            caret: self.caret,
            anchor: self.anchor,
            answer: self.answer,
            result_selected: self.result_selected,
            current: self.current,
            notice: self.notice,
            focus: self.focus,
            active: self.active,
            hover: self.hover,
            pressed: self.pressed,
            status: self.status(),
        };
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next;
        } else {
            self.exhausted = true;
        }
    }
    fn edit_state(&self) -> EditState {
        EditState {
            bytes: self.bytes,
            length: self.length,
            caret: self.caret,
            anchor: self.anchor,
        }
    }
    fn remember(&mut self, before: EditState) {
        if before != self.edit_state() {
            self.history.record(before);
        }
    }
    /// Leaving recall with a recalled formula in place turns that replacement
    /// into one undoable edit, so undo can always reach the unfinished draft.
    fn leave_recall(&mut self) {
        if let Some(draft) = self.recent.draft {
            self.remember(EditState {
                bytes: draft.formula.bytes,
                length: draft.formula.length,
                caret: draft.caret,
                anchor: draft.anchor,
            });
        }
        self.recent.leave();
    }
    fn edited(&mut self, before: EditState) {
        self.leave_recall();
        self.remember(before);
        self.result_selected = false;
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
    fn recall(&mut self, older: bool) {
        if self.focus != 0 || self.recent.length == 0 {
            return;
        }
        let next = match (self.recent.browsing, older) {
            (None, false) | (Some(0), true) => return,
            (None, true) => {
                self.recent.draft = Some(RecallDraft {
                    formula: Formula {
                        bytes: self.bytes,
                        length: self.length,
                    },
                    caret: self.caret,
                    anchor: self.anchor,
                });
                Some(self.recent.length - 1)
            }
            (Some(index), true) => Some(index - 1),
            (Some(index), false) if index + 1 < self.recent.length => Some(index + 1),
            (Some(_), false) => None,
        };
        let state = match next {
            Some(index) => {
                let formula = self.recent.entries[index];
                RecallDraft {
                    formula,
                    caret: formula.length,
                    anchor: formula.length,
                }
            }
            None => self
                .recent
                .draft
                .take()
                .expect("recall retains its starting draft"),
        };
        self.recent.browsing = next;
        self.bytes = state.formula.bytes;
        self.length = state.formula.length;
        self.caret = state.caret;
        self.anchor = state.anchor;
        self.pressed = None;
        self.current = false;
        self.notice = Notice::None;
        self.changed();
    }
    pub fn command(&mut self, command: Command) {
        if self.exhausted {
            return;
        }
        self.focus_gained();
        self.selecting = false;
        if command != Command::SelectResult && self.result_selected {
            self.result_selected = false;
            self.changed();
        }
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
                let before = self.edit_state();
                self.erase(range);
                self.bytes
                    .copy_within(self.caret..self.length, self.caret + 1);
                self.bytes[self.caret] = byte;
                self.length += 1;
                self.caret += 1;
                self.anchor = self.caret;
                self.focus = 0;
                self.edited(before);
            }
            Command::Evaluate => {
                self.leave_recall();
                match decimal::evaluate(self.expression()) {
                    Ok(answer) => {
                        let formatted = answer.value.format();
                        self.result_bytes.fill(0);
                        self.result_length = formatted.as_str().len();
                        self.result_bytes[..self.result_length]
                            .copy_from_slice(formatted.as_str().as_bytes());
                        self.recent.record(Formula {
                            bytes: self.bytes,
                            length: self.length,
                        });
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
                self.leave_recall();
                let before = self.edit_state();
                self.bytes.fill(0);
                self.length = 0;
                self.caret = 0;
                self.anchor = 0;
                self.answer = None;
                self.result_bytes.fill(0);
                self.result_length = 0;
                self.current = false;
                self.notice = Notice::None;
                self.focus = 0;
                self.pressed = None;
                self.remember(before);
                self.changed();
            }
            Command::Cancel => {
                self.leave_recall();
                let before = self.edit_state();
                self.bytes.fill(0);
                self.length = 0;
                self.caret = 0;
                self.anchor = 0;
                self.current = false;
                self.notice = Notice::None;
                self.focus = 0;
                self.pressed = None;
                self.remember(before);
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
                    let before = self.edit_state();
                    self.erase(range);
                    self.edited(before);
                }
            }
            Command::Undo | Command::Redo => {
                let recalling = self.recalling();
                if command == Command::Redo && recalling && self.can_redo() {
                    // Redo continues the existing branch from the unfinished
                    // draft; recording the recall here would truncate it.
                    if let Some(draft) = self.recent.draft {
                        self.bytes = draft.formula.bytes;
                        self.length = draft.formula.length;
                        self.caret = draft.caret;
                        self.anchor = draft.anchor;
                    }
                    self.recent.leave();
                } else {
                    self.leave_recall();
                }
                if let Some(state) = self
                    .history
                    .restore(self.edit_state(), command == Command::Redo)
                {
                    self.bytes = state.bytes;
                    self.length = state.length;
                    self.caret = state.caret;
                    self.anchor = state.anchor;
                    self.focus = 0;
                    self.pressed = None;
                    // History restores formula edits, never a cached calculation.
                    self.current = false;
                    self.notice = Notice::None;
                    self.changed();
                } else if recalling {
                    self.changed();
                }
            }
            Command::RecallOlder => self.recall(true),
            Command::RecallNewer => self.recall(false),
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
            Command::SelectResult => {
                if self.current && self.answer.is_some() {
                    self.anchor = self.caret;
                    self.result_selected = true;
                    self.pressed = None;
                    self.focus = 0;
                    self.notice = Notice::None;
                    self.changed();
                }
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
        if self.focus != focus {
            self.focus = focus;
            self.changed();
        }
    }
    pub fn pointer(&mut self, button: Option<usize>, down: bool, up: bool) {
        if self.exhausted {
            return;
        }
        self.focus_gained();
        if down && self.result_selected {
            self.result_selected = false;
            self.changed();
        }
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
        if self.exhausted {
            return;
        }
        if down || self.selecting {
            self.focus_gained();
        }
        if down {
            self.result_selected = false;
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
        if self.result_selected {
            self.result_selected = false;
            self.changed();
        }
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
            let command = match key {
                PhysicalKey::A => Command::SelectAll,
                PhysicalKey::R if !m.shift() => Command::SelectResult,
                PhysicalKey::Z if !m.shift() => Command::Undo,
                PhysicalKey::Z => Command::Redo,
                PhysicalKey::Y if !m.shift() => Command::Redo,
                _ => return,
            };
            if event.action == KeyAction::Press || command == Command::SelectAll {
                self.command(command);
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
            Up if !m.shift() => Command::RecallOlder,
            Down if !m.shift() => Command::RecallNewer,
            Keypad8 if !numeric && !m.shift() => Command::RecallOlder,
            Keypad2 if !numeric && !m.shift() => Command::RecallNewer,
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
                    | Command::RecallOlder
                    | Command::RecallNewer
            )
        {
            return;
        }
        self.command(command);
    }
}
