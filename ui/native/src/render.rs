//! Scalar private-RAM tiles. No framebuffer mapping, device I/O or syscall.
use crate::{
    controller::{Error as UiError, Focus, Screen, Status, View},
    design as d, text, Rect, Scale, Viewport, TILE_HEIGHT, TILE_WIDTH,
};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

pub const MAX_TEXT_ROWS: usize = 48;
/// Fixed material nodes, a paging pair, and one node per volume-domain object.
pub const MAX_SEMANTIC_NODES: usize = 8 + volume_core::PER_DOMAIN;

fn raster_height(scale: Scale) -> RasterHeight {
    match scale {
        Scale::Percent100 => RasterHeight::Size16,
        Scale::Percent200 => RasterHeight::Size32,
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub new: Rect,
    pub refresh: Rect,
    pub back: Rect,
    pub save: Rect,
    pub editor: Rect,
    pub status: Rect,
    pub list_bottom: u32,
    pub list_top: u32,
    pub title_y: u32,
    pub list_previous: Rect,
    pub list_next: Rect,
    pub modal: Rect,
    pub modal_save: Rect,
    pub modal_cancel: Rect,
    pub modal_discard: Rect,
    width: u32,
    character_width: usize,
}
impl Layout {
    pub fn new(viewport: Viewport) -> Self {
        let width = viewport.logical_width();
        let height = viewport.logical_height();
        let compact = height < 360;
        let controls_y = if compact { 64 } else { 80 };
        let button_width = (width - 2 * d::PAD - 2 * d::SPACE) / 2;
        let modal = Rect {
            x: d::PAD,
            y: (height - 208) / 2,
            width: width - 2 * d::PAD,
            height: 208,
        };
        let modal_button = (modal.width - 40) / 2;
        Self {
            new: Rect {
                x: d::PAD,
                y: controls_y,
                width: button_width,
                height: d::CONTROL_HEIGHT,
            },
            refresh: Rect {
                x: d::PAD + button_width + 2 * d::SPACE,
                y: controls_y,
                width: button_width,
                height: d::CONTROL_HEIGHT,
            },
            back: Rect {
                x: d::PAD,
                y: d::PAD,
                width: 100,
                height: d::CONTROL_HEIGHT,
            },
            save: Rect {
                x: width - d::PAD - 112,
                y: d::PAD,
                width: 112,
                height: d::CONTROL_HEIGHT,
            },
            editor: Rect {
                x: d::PAD,
                y: 144,
                width: width - 2 * d::PAD,
                height: height - 208,
            },
            status: Rect {
                x: d::PAD,
                y: height - 48,
                width: width - 2 * d::PAD,
                height: 48,
            },
            list_bottom: height - 56,
            list_top: if compact { 120 } else { 144 },
            title_y: if compact { 16 } else { 20 },
            list_previous: Rect {
                x: width - d::PAD - 96,
                y: if compact { 8 } else { 20 },
                width: 44,
                height: 44,
            },
            list_next: Rect {
                x: width - d::PAD - 44,
                y: if compact { 8 } else { 20 },
                width: 44,
                height: 44,
            },
            modal,
            modal_save: Rect {
                x: modal.x + 16,
                y: modal.y + 104,
                width: modal_button,
                height: d::CONTROL_HEIGHT,
            },
            modal_cancel: Rect {
                x: modal.x + 24 + modal_button,
                y: modal.y + 104,
                width: modal_button,
                height: d::CONTROL_HEIGHT,
            },
            modal_discard: Rect {
                x: modal.x + 16,
                y: modal.y + 156,
                width: modal.width - 32,
                height: d::CONTROL_HEIGHT,
            },
            width,
            character_width: get_raster_width(FontWeight::Regular, raster_height(viewport.scale()))
                .div_ceil(viewport.scale().factor() as usize),
        }
    }
    pub fn character_width(self) -> usize {
        self.character_width
    }
    pub fn columns(self) -> usize {
        ((self.editor.width - 18) as usize / self.character_width).max(4)
    }
    pub fn rows(self) -> usize {
        ((self.editor.height.saturating_sub(16)) / d::LINE_HEIGHT).max(1) as usize
    }
    pub fn list_rows(self) -> usize {
        ((self.list_bottom.saturating_sub(self.list_top) + 8) / 72).max(1) as usize
    }
    pub fn object(self, row: usize) -> Rect {
        Rect {
            x: d::PAD,
            y: self
                .list_top
                .saturating_add((row as u32).saturating_mul(72)),
            width: self.width - 2 * d::PAD,
            height: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Tile,
    Buffer,
}

/// Snapshot borrows immutable data owned by the process. Construct it once per
/// frame: visible-line indexing is O(document bytes), not repeated per tile.
pub struct Frame<'a> {
    view: View<'a>,
    layout: Layout,
    rows: [Option<(usize, usize)>; MAX_TEXT_ROWS],
    caret: text::Position,
}
impl<'a> Frame<'a> {
    pub fn new(view: View<'a>) -> Self {
        let layout = Layout::new(view.viewport);
        let mut frame = Self {
            view,
            layout,
            rows: [None; MAX_TEXT_ROWS],
            caret: text::Position::default(),
        };
        if let Some(contents) = view.contents {
            frame.caret = text::position(contents, view.caret, layout.columns());
            if view.scroll == 0 {
                frame.rows[0] = Some((0, 0));
            }
            let mut cells = text::Cells::new(contents, layout.columns());
            for cell in cells.by_ref() {
                if cell.position.row < view.scroll {
                    continue;
                }
                let row = cell.position.row - view.scroll;
                if row >= layout.rows().min(MAX_TEXT_ROWS) {
                    break;
                }
                let start = frame.rows[row].map_or(cell.start, |range| range.0);
                frame.rows[row] = Some((start, cell.end));
            }
            let end = text::position(contents, contents.len(), layout.columns());
            if end.row >= view.scroll && end.row - view.scroll < layout.rows().min(MAX_TEXT_ROWS) {
                frame.rows[end.row - view.scroll].get_or_insert((contents.len(), contents.len()));
            }
        }
        frame
    }
    pub fn generation(&self) -> u64 {
        self.view.generation
    }
    pub fn display_epoch(&self) -> u64 {
        self.view.viewport.epoch()
    }
    pub fn tiles(&self) -> Tiles {
        Tiles {
            viewport: self.view.viewport,
            x: 0,
            y: if self.view.dirty {
                0
            } else {
                self.view.viewport.height()
            },
        }
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Produces tightly packed canonical 0x00RRGGBB pixels. Everything after
    /// rect.width*rect.height in `out` remains unchanged, including on errors.
    pub fn render_tile(&self, x: u32, y: u32, out: &mut [u32]) -> Result<Rect, Error> {
        let viewport = self.view.viewport;
        if x >= viewport.width() || y >= viewport.height() {
            return Err(Error::Tile);
        }
        let rect = Rect {
            x,
            y,
            width: TILE_WIDTH.min(viewport.width() - x),
            height: TILE_HEIGHT.min(viewport.height() - y),
        };
        let count = (rect.width * rect.height) as usize;
        if out.len() < count {
            return Err(Error::Buffer);
        }
        let mut paint = Paint {
            rect,
            pixels: &mut out[..count],
            scale: viewport.scale(),
            cursor: self
                .view
                .cursor
                .map(|(x, y)| (x / viewport.scale().factor(), y / viewport.scale().factor())),
        };
        paint.pixels.fill(d::BACKGROUND);
        let layout = self.layout;
        match self.view.screen {
            Screen::Material => {
                paint.title(d::PAD, layout.title_y, "Materiaal");
                if self.view.list_scroll > 0 || self.view.objects.len() > layout.list_rows() {
                    paint.arrow(
                        layout.list_previous,
                        true,
                        self.view.focus == Focus::Previous,
                        self.view.list_scroll > 0,
                    );
                    paint.arrow(
                        layout.list_next,
                        false,
                        self.view.focus == Focus::Next,
                        self.view.list_scroll + layout.list_rows() < self.view.objects.len(),
                    );
                }
                paint.button(
                    layout.new,
                    "Nieuw",
                    self.view.focus == Focus::New,
                    self.view.editable,
                    ButtonStyle::Primary,
                );
                paint.button(
                    layout.refresh,
                    "Vernieuwen",
                    self.view.focus == Focus::Refresh,
                    true,
                    ButtonStyle::Secondary,
                );
                if !self.view.list_loaded {
                    paint.wrapped(
                        d::PAD,
                        layout.list_top,
                        "Open je materiaal met Vernieuwen.",
                        layout.editor.width,
                        d::MUTED,
                    );
                } else if self.view.objects.is_empty() {
                    paint.wrapped(
                        d::PAD,
                        layout.list_top,
                        "Hier begint je materiaal. Maak een nieuw document.",
                        layout.editor.width,
                        d::MUTED,
                    );
                } else {
                    for (index, object) in self
                        .view
                        .objects
                        .iter()
                        .enumerate()
                        .skip(self.view.list_scroll)
                        .take(layout.list_rows())
                    {
                        let row = layout.object(index - self.view.list_scroll);
                        if row.y + row.height > layout.list_bottom {
                            break;
                        }
                        paint.rounded(row, d::SURFACE, d::SURFACE_RADIUS);
                        if self.view.focus == Focus::Object(index) {
                            paint.focus(row);
                        }
                        let name = object
                            .name
                            .get(..object.name_length)
                            .and_then(|b| core::str::from_utf8(b).ok())
                            .unwrap_or("Naam niet leesbaar");
                        paint.wrapped(row.x + 12, row.y + 8, name, row.width - 24, d::TEXT);
                        // Preserve the full bounded title on narrow layouts.
                        // Location is already stated by the page status; do
                        // not paint metadata through a wrapped second line.
                        if name.chars().count() * layout.character_width()
                            <= (row.width - 24) as usize
                        {
                            paint.label(row.x + 12, row.y + 36, "Op dit apparaat", d::MUTED);
                        }
                    }
                }
            }
            Screen::Editor | Screen::ConfirmClose => {
                paint.button(
                    layout.back,
                    "Terug",
                    self.view.focus == Focus::Back,
                    true,
                    ButtonStyle::Secondary,
                );
                let label = if self.view.saving {
                    "Annuleren"
                } else {
                    "Opslaan"
                };
                let focus = if self.view.saving {
                    Focus::CancelSave
                } else {
                    Focus::Save
                };
                paint.button(
                    layout.save,
                    label,
                    self.view.focus == focus,
                    self.view.can_save || self.view.saving,
                    if self.view.saving {
                        ButtonStyle::Secondary
                    } else {
                        ButtonStyle::Primary
                    },
                );
                let title = core::str::from_utf8(self.view.title).unwrap_or("Naam niet leesbaar");
                paint.wrapped(d::PAD, 80, title, layout.editor.width, d::TEXT);
                paint.outlined(layout.editor, d::SURFACE, d::BORDER, d::CONTROL_RADIUS);
                if self.view.focus == Focus::Document && self.view.screen == Screen::Editor {
                    paint.focus(layout.editor);
                }
                if let Some(contents) = self.view.contents {
                    let selected = text::selection(self.view.anchor, self.view.caret);
                    for (row, range) in self
                        .rows
                        .iter()
                        .enumerate()
                        .take(layout.rows().min(MAX_TEXT_ROWS))
                    {
                        let Some((start, end)) = *range else {
                            continue;
                        };
                        let mut column = 0usize;
                        for (offset, ch) in contents[start..end].char_indices() {
                            let width = if ch == '\t' {
                                4 - column % 4
                            } else if ch == '\r' {
                                0
                            } else {
                                1
                            };
                            let x =
                                layout.editor.x + 8 + (column * layout.character_width()) as u32;
                            let y = layout.editor.y + 8 + row as u32 * d::LINE_HEIGHT;
                            let bytes = start + offset..start + offset + ch.len_utf8();
                            if selected.start < bytes.end && selected.end > bytes.start {
                                paint.fill(
                                    Rect {
                                        x,
                                        y,
                                        width: (width.max(1) * layout.character_width()) as u32,
                                        height: 16,
                                    },
                                    d::ACCENT_SOFT,
                                );
                            }
                            if !matches!(ch, '\n' | '\r' | '\t') {
                                paint.character(x, y, ch, d::TEXT);
                            }
                            if ch != '\n' {
                                column += width;
                            }
                        }
                    }
                    if self.view.focus == Focus::Document
                        && self.view.screen == Screen::Editor
                        && self.caret.row >= self.view.scroll
                        && self.caret.row - self.view.scroll < layout.rows()
                    {
                        paint.fill(
                            Rect {
                                x: layout.editor.x
                                    + 8
                                    + (self.caret.column * layout.character_width()) as u32,
                                y: layout.editor.y
                                    + 8
                                    + (self.caret.row - self.view.scroll) as u32 * d::LINE_HEIGHT,
                                width: 2,
                                height: 16,
                            },
                            d::ACCENT,
                        );
                    }
                }
            }
        }
        paint.wrapped(
            layout.status.x,
            layout.status.y,
            status_label(self.view.status),
            layout.status.width,
            status_color(self.view.status),
        );
        if let Status::Saving { completed, total } = self.view.status {
            let width = if total == 0 {
                0
            } else {
                ((layout.status.width as u64 * completed.min(total) as u64) / total as u64) as u32
            };
            paint.fill(
                Rect {
                    x: layout.status.x,
                    y: layout.status.y + 24,
                    width: layout.status.width,
                    height: 4,
                },
                d::LINE,
            );
            paint.fill(
                Rect {
                    x: layout.status.x,
                    y: layout.status.y + 24,
                    width,
                    height: 4,
                },
                d::ACCENT,
            );
        }
        if self.view.screen == Screen::ConfirmClose {
            // Opaque application-local backdrop: no half-obscured controls or
            // text remain visible behind this modal. This grants no system-
            // dialog privilege and does not read a framebuffer/backbuffer.
            paint.pixels.fill(blend(d::TEXT, d::BACKGROUND, 24));
            paint.outlined(layout.modal, d::SURFACE, d::BORDER, d::SURFACE_RADIUS);
            paint.label(
                layout.modal.x + 16,
                layout.modal.y + 16,
                "Nog niet opgeslagen",
                d::TEXT,
            );
            paint.wrapped(
                layout.modal.x + 16,
                layout.modal.y + 48,
                "Bewaar je wijzigingen of ga terug.",
                layout.modal.width - 32,
                d::TEXT,
            );
            paint.button(
                layout.modal_save,
                "Bewaren",
                self.view.focus == Focus::SaveAndBack,
                true,
                ButtonStyle::Primary,
            );
            paint.button(
                layout.modal_cancel,
                "Annuleren",
                self.view.focus == Focus::KeepEditing,
                true,
                ButtonStyle::Secondary,
            );
            paint.button(
                layout.modal_discard,
                "Wijzigingen verwerpen",
                self.view.focus == Focus::Discard,
                true,
                ButtonStyle::Danger,
            );
        }
        if let Some((x, y)) = self
            .view
            .cursor
            .filter(|(x, y)| *x < viewport.width() && *y < viewport.height())
        {
            // Pointer uses physical pixels and contrasting outline, independent
            // of control color. A native compositor can own this in its layer.
            for dy in 0..14 {
                for dx in 0..=dy / 2 {
                    paint.pixel(
                        i64::from(x + dx),
                        i64::from(y + dy),
                        if dx == 0 || dx == dy / 2 || dy == 13 {
                            d::SURFACE
                        } else {
                            d::TEXT
                        },
                    );
                }
            }
        }
        Ok(rect)
    }

    pub fn semantics(&self, out: &mut [SemanticNode<'a>]) -> Result<usize, Error> {
        let mut required = 0;
        self.visit_semantics(|_| required += 1);
        if required > MAX_SEMANTIC_NODES || out.len() < required {
            return Err(Error::Buffer);
        }

        let mut count = 0;
        self.visit_semantics(|node| {
            out[count] = node;
            count += 1;
        });
        Ok(count)
    }

    fn visit_semantics(&self, mut visit: impl FnMut(SemanticNode<'a>)) {
        let mut push =
            |id, parent, role, label, value, bounds: Rect, focus: Option<Focus>, enabled| {
                visit(SemanticNode {
                    id,
                    parent,
                    role,
                    label,
                    value,
                    bounds: if id == SemanticId::Window {
                        // The viewport is already in physical pixels; layout
                        // rectangles below are logical and need one conversion.
                        bounds
                    } else {
                        bounds.physical(self.view.viewport.scale())
                    },
                    focused: focus == Some(self.view.focus),
                    enabled,
                    actions: SemanticActions {
                        activate: enabled && matches!(role, Role::Button | Role::ListItem),
                        set_value: enabled && role == Role::TextEditor && self.view.editable,
                    },
                    read_only: role == Role::TextEditor && !self.view.editable,
                    selection: if role == Role::TextEditor {
                        Some((self.view.anchor, self.view.caret))
                    } else {
                        None
                    },
                    progress: if role == Role::Status {
                        if let Status::Saving { completed, total } = self.view.status {
                            Some((completed, total))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                });
            };
        let l = self.layout;
        push(
            SemanticId::Window,
            None,
            Role::Window,
            "Materiaal",
            None,
            self.view.viewport.bounds(),
            None,
            true,
        );
        match self.view.screen {
            Screen::Material => {
                push(
                    SemanticId::Heading,
                    Some(SemanticId::Window),
                    Role::Heading,
                    "Materiaal",
                    None,
                    Rect {
                        x: d::PAD,
                        y: l.title_y,
                        width: 200,
                        height: 40,
                    },
                    None,
                    true,
                );
                push(
                    SemanticId::NewDocument,
                    Some(SemanticId::Window),
                    Role::Button,
                    "Nieuw document",
                    None,
                    l.new,
                    Some(Focus::New),
                    self.view.editable,
                );
                push(
                    SemanticId::Refresh,
                    Some(SemanticId::Window),
                    Role::Button,
                    "Vernieuwen",
                    None,
                    l.refresh,
                    Some(Focus::Refresh),
                    true,
                );
                push(
                    SemanticId::MaterialList,
                    Some(SemanticId::Window),
                    Role::List,
                    "Materialen",
                    None,
                    Rect {
                        x: d::PAD,
                        y: l.list_top,
                        width: l.width - 2 * d::PAD,
                        height: l.list_bottom.saturating_sub(l.list_top),
                    },
                    None,
                    true,
                );
                if self.view.list_scroll > 0 || self.view.objects.len() > l.list_rows() {
                    push(
                        SemanticId::PreviousPage,
                        Some(SemanticId::Window),
                        Role::Button,
                        "Vorige materialen",
                        None,
                        l.list_previous,
                        Some(Focus::Previous),
                        self.view.list_scroll > 0,
                    );
                    push(
                        SemanticId::NextPage,
                        Some(SemanticId::Window),
                        Role::Button,
                        "Volgende materialen",
                        None,
                        l.list_next,
                        Some(Focus::Next),
                        self.view.list_scroll + l.list_rows() < self.view.objects.len(),
                    );
                }
                for (index, entry) in self
                    .view
                    .objects
                    .iter()
                    .enumerate()
                    .skip(self.view.list_scroll)
                    .take(l.list_rows())
                {
                    let bounds = l.object(index - self.view.list_scroll);
                    if bounds.y + bounds.height > l.list_bottom {
                        break;
                    }
                    let label = entry
                        .name
                        .get(..entry.name_length)
                        .and_then(|b| core::str::from_utf8(b).ok())
                        .unwrap_or("Naam niet leesbaar");
                    push(
                        SemanticId::Object(entry.object),
                        Some(SemanticId::MaterialList),
                        Role::ListItem,
                        label,
                        None,
                        bounds,
                        Some(Focus::Object(index)),
                        true,
                    );
                }
            }
            Screen::Editor => {
                push(
                    SemanticId::Back,
                    Some(SemanticId::Window),
                    Role::Button,
                    "Terug naar materiaal",
                    None,
                    l.back,
                    Some(Focus::Back),
                    true,
                );
                push(
                    SemanticId::Save,
                    Some(SemanticId::Window),
                    Role::Button,
                    if self.view.saving {
                        "Opslaan annuleren"
                    } else {
                        "Opslaan"
                    },
                    None,
                    l.save,
                    Some(if self.view.saving {
                        Focus::CancelSave
                    } else {
                        Focus::Save
                    }),
                    self.view.can_save || self.view.saving,
                );
                push(
                    SemanticId::Document,
                    Some(SemanticId::Window),
                    Role::TextEditor,
                    "Document",
                    self.view.contents,
                    l.editor,
                    Some(Focus::Document),
                    !self.view.saving,
                );
            }
            Screen::ConfirmClose => {
                push(
                    SemanticId::CloseDialog,
                    Some(SemanticId::Window),
                    Role::Dialog,
                    "Onopgeslagen wijzigingen",
                    None,
                    l.modal,
                    None,
                    true,
                );
                push(
                    SemanticId::SaveAndBack,
                    Some(SemanticId::CloseDialog),
                    Role::Button,
                    "Bewaren",
                    None,
                    l.modal_save,
                    Some(Focus::SaveAndBack),
                    true,
                );
                push(
                    SemanticId::KeepEditing,
                    Some(SemanticId::CloseDialog),
                    Role::Button,
                    "Annuleren",
                    None,
                    l.modal_cancel,
                    Some(Focus::KeepEditing),
                    true,
                );
                push(
                    SemanticId::Discard,
                    Some(SemanticId::CloseDialog),
                    Role::Button,
                    "Wijzigingen verwerpen",
                    None,
                    l.modal_discard,
                    Some(Focus::Discard),
                    true,
                );
            }
        }
        push(
            SemanticId::Status,
            Some(SemanticId::Window),
            Role::Status,
            status_label(self.view.status),
            None,
            l.status,
            None,
            true,
        );
    }
}

pub struct Tiles {
    viewport: Viewport,
    x: u32,
    y: u32,
}
impl Iterator for Tiles {
    type Item = Rect;
    fn next(&mut self) -> Option<Rect> {
        if self.y >= self.viewport.height() {
            return None;
        }
        let rect = Rect {
            x: self.x,
            y: self.y,
            width: TILE_WIDTH.min(self.viewport.width() - self.x),
            height: TILE_HEIGHT.min(self.viewport.height() - self.y),
        };
        self.x += TILE_WIDTH;
        if self.x >= self.viewport.width() {
            self.x = 0;
            self.y += TILE_HEIGHT;
        }
        Some(rect)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Window,
    Heading,
    List,
    Button,
    ListItem,
    TextEditor,
    Status,
    Dialog,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticId {
    Empty,
    Window,
    Heading,
    NewDocument,
    Refresh,
    MaterialList,
    PreviousPage,
    NextPage,
    Object(u64),
    Back,
    Save,
    Document,
    CloseDialog,
    SaveAndBack,
    KeepEditing,
    Discard,
    Status,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SemanticActions {
    pub activate: bool,
    pub set_value: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct SemanticNode<'a> {
    pub id: SemanticId,
    pub parent: Option<SemanticId>,
    pub role: Role,
    pub label: &'a str,
    pub value: Option<&'a str>,
    pub bounds: Rect,
    pub focused: bool,
    pub enabled: bool,
    pub actions: SemanticActions,
    pub read_only: bool,
    pub selection: Option<(usize, usize)>,
    pub progress: Option<(usize, usize)>,
}
impl SemanticNode<'_> {
    pub const EMPTY: Self = Self {
        id: SemanticId::Empty,
        parent: None,
        role: Role::Status,
        label: "",
        value: None,
        bounds: Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        focused: false,
        enabled: false,
        actions: SemanticActions {
            activate: false,
            set_value: false,
        },
        read_only: false,
        selection: None,
        progress: None,
    };
}

pub fn status_label(status: Status) -> &'static str {
    use nieuw_os_document_session::volume::Error as S;
    match status {
        Status::Ready => "Je materiaal op dit apparaat.",
        Status::ReadOnly => "Alleen lezen op dit apparaat.",
        Status::Opened => "Document geopend.",
        Status::Dirty => "Wijzigingen nog niet opgeslagen.",
        Status::Saving { .. } => "Opslaan...",
        Status::Saved { .. } => "Op dit apparaat opgeslagen.",
        Status::Cancelled => "Opslaan geannuleerd. Je tekst blijft open.",
        Status::InputLost => "Invoer onderbroken. Je tekst blijft open.",
        Status::Failed(UiError::InvalidUtf8) => "Dit bestand is geen geldige UTF-8-tekst.",
        Status::Failed(UiError::NoFreeObject) => "Er is geen ruimte voor een nieuw document.",
        Status::Failed(UiError::Storage(S::ReadOnly)) => {
            "Alleen lezen: wijzigen is niet toegestaan."
        }
        Status::Failed(UiError::Storage(S::ContentTooLarge)) => {
            "Tekst past niet in de beschikbare ruimte."
        }
        Status::Failed(UiError::Storage(S::Conflict)) => {
            "Er is een nieuwe versie. Je tekst blijft open."
        }
        Status::Failed(UiError::Storage(S::Uncertain)) => "Opslag onzeker. Je tekst blijft open.",
        Status::Failed(UiError::Storage(S::Closed)) => {
            "Opslagverbinding verbroken. Tekst blijft open."
        }
        Status::Failed(UiError::Storage(S::Busy)) => "Opslaan loopt al. Je tekst blijft open.",
        Status::Failed(_) => "Handeling mislukt. Je tekst blijft open.",
    }
}
fn status_color(status: Status) -> u32 {
    if matches!(status, Status::Failed(_)) {
        d::DANGER
    } else {
        d::MUTED
    }
}

enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
}
struct Paint<'a> {
    rect: Rect,
    pixels: &'a mut [u32],
    scale: Scale,
    cursor: Option<(u32, u32)>,
}
impl Paint<'_> {
    fn pixel(&mut self, x: i64, y: i64, color: u32) {
        if x < i64::from(self.rect.x)
            || y < i64::from(self.rect.y)
            || x >= i64::from(self.rect.x + self.rect.width)
            || y >= i64::from(self.rect.y + self.rect.height)
        {
            return;
        }
        self.pixels[(y as usize - self.rect.y as usize) * self.rect.width as usize + x as usize
            - self.rect.x as usize] = color & 0xffffff;
    }
    fn fill(&mut self, rect: Rect, color: u32) {
        let r = rect.physical(self.scale);
        let left = r.x.max(self.rect.x);
        let top = r.y.max(self.rect.y);
        let right =
            (u64::from(r.x) + u64::from(r.width)).min(u64::from(self.rect.x + self.rect.width));
        let bottom =
            (u64::from(r.y) + u64::from(r.height)).min(u64::from(self.rect.y + self.rect.height));
        for y in u64::from(top)..bottom {
            for x in u64::from(left)..right {
                self.pixel(x as i64, y as i64, color);
            }
        }
    }
    fn rounded(&mut self, rect: Rect, color: u32, radius: u32) {
        let r = rect.physical(self.scale);
        if !r.intersects(self.rect) {
            return;
        }
        let radius = (radius * self.scale.factor())
            .min(r.width / 2)
            .min(r.height / 2);
        let left = r.x.max(self.rect.x);
        let top = r.y.max(self.rect.y);
        let right = (r.x + r.width).min(self.rect.x + self.rect.width);
        let bottom = (r.y + r.height).min(self.rect.y + self.rect.height);
        for y in top..bottom {
            for x in left..right {
                let dx = if x < r.x + radius {
                    r.x + radius - x
                } else {
                    x.saturating_sub(r.x + r.width - radius - 1)
                };
                let dy = if y < r.y + radius {
                    r.y + radius - y
                } else {
                    y.saturating_sub(r.y + r.height - radius - 1)
                };
                if dx * dx + dy * dy <= radius * radius {
                    self.pixel(i64::from(x), i64::from(y), color);
                }
            }
        }
    }
    fn border(&mut self, r: Rect, color: u32) {
        self.fill(
            Rect {
                x: r.x,
                y: r.y,
                width: r.width,
                height: 1,
            },
            color,
        );
        self.fill(
            Rect {
                x: r.x,
                y: r.y + r.height - 1,
                width: r.width,
                height: 1,
            },
            color,
        );
        self.fill(
            Rect {
                x: r.x,
                y: r.y,
                width: 1,
                height: r.height,
            },
            color,
        );
        self.fill(
            Rect {
                x: r.x + r.width - 1,
                y: r.y,
                width: 1,
                height: r.height,
            },
            color,
        );
    }
    fn focus(&mut self, r: Rect) {
        let gap = d::FOCUS_WIDTH + d::FOCUS_OFFSET;
        for offset in d::FOCUS_OFFSET..gap {
            self.border(
                Rect {
                    x: r.x.saturating_sub(offset),
                    y: r.y.saturating_sub(offset),
                    width: r.width + 2 * offset,
                    height: r.height + 2 * offset,
                },
                d::ACCENT,
            );
        }
    }
    fn outlined(&mut self, r: Rect, fill: u32, border: u32, radius: u32) {
        self.rounded(r, border, radius);
        if r.width > 2 && r.height > 2 {
            self.rounded(
                Rect {
                    x: r.x + 1,
                    y: r.y + 1,
                    width: r.width - 2,
                    height: r.height - 2,
                },
                fill,
                radius.saturating_sub(1),
            );
        }
    }
    fn button(&mut self, r: Rect, label: &str, focused: bool, enabled: bool, style: ButtonStyle) {
        let hovered = enabled && self.cursor.is_some_and(|(x, y)| r.contains(x, y));
        let (background, border, foreground) = if !enabled {
            (d::LINE, d::LINE, d::MUTED)
        } else {
            match style {
                ButtonStyle::Primary => (
                    d::ACCENT,
                    if hovered { d::TEXT } else { d::ACCENT },
                    d::SURFACE,
                ),
                ButtonStyle::Danger => (d::SURFACE, d::DANGER, d::DANGER),
                _ => (
                    if hovered { d::ACCENT_SOFT } else { d::SURFACE },
                    d::BORDER,
                    d::TEXT,
                ),
            }
        };
        self.outlined(r, background, border, d::CONTROL_RADIUS);
        if focused {
            self.focus(r);
        }
        let width = get_raster_width(FontWeight::Regular, raster_height(self.scale))
            .div_ceil(self.scale.factor() as usize) as u32;
        let x = r.x + r.width.saturating_sub(label.chars().count() as u32 * width) / 2;
        self.label(x, r.y + 14, label, foreground);
    }
    fn arrow(&mut self, r: Rect, previous: bool, focused: bool, enabled: bool) {
        self.button(r, "", focused, enabled, ButtonStyle::Secondary);
        for row in 0..13u32 {
            let delta = row.abs_diff(6);
            let x = if previous {
                r.x + 19 + delta
            } else {
                r.x + 25 - delta
            };
            self.fill(
                Rect {
                    x,
                    y: r.y + 16 + row,
                    width: 2,
                    height: 2,
                },
                if enabled { d::TEXT } else { d::MUTED },
            );
        }
    }
    fn character(&mut self, x: u32, y: u32, character: char, color: u32) {
        let height = raster_height(self.scale);
        let Some(glyph) = get_raster(character, FontWeight::Regular, height)
            .or_else(|| get_raster('\u{fffd}', FontWeight::Regular, height))
        else {
            return;
        };
        let x = x * self.scale.factor();
        let y = y * self.scale.factor();
        if !(Rect {
            x,
            y,
            width: glyph.width() as u32,
            height: glyph.height() as u32,
        })
        .intersects(self.rect)
        {
            return;
        }
        for (dy, row) in glyph.raster().iter().enumerate() {
            for (dx, alpha) in row.iter().copied().enumerate() {
                let px = x + dx as u32;
                let py = y + dy as u32;
                if alpha != 0 && self.rect.contains(px, py) {
                    let index = ((py - self.rect.y) * self.rect.width + px - self.rect.x) as usize;
                    let old = self.pixels[index];
                    self.pixels[index] = blend(color, old, alpha);
                }
            }
        }
    }
    fn label(&mut self, x: u32, y: u32, label: &str, color: u32) {
        let width = get_raster_width(FontWeight::Regular, raster_height(self.scale))
            .div_ceil(self.scale.factor() as usize) as u32;
        for (index, ch) in label.chars().enumerate() {
            self.character(x + (index as u32) * width, y, ch, color);
        }
    }
    fn wrapped(&mut self, x: u32, y: u32, label: &str, width: u32, color: u32) {
        let step = get_raster_width(FontWeight::Regular, raster_height(self.scale))
            .div_ceil(self.scale.factor() as usize) as u32;
        let columns = (width / step).max(1) as usize;
        for (index, ch) in label.chars().enumerate() {
            self.character(
                x + (index % columns) as u32 * step,
                y + (index / columns) as u32 * d::LINE_HEIGHT,
                ch,
                color,
            );
        }
    }
    fn title(&mut self, x: u32, y: u32, title: &str) {
        // 32px title at100%, doubled retained raster at200%. Body uses a real
        // 32px raster at200%; this is not a general scalable shaping engine.
        let scale = self.scale.factor();
        let step = get_raster_width(FontWeight::Regular, RasterHeight::Size32) as u32;
        for (index, ch) in title.chars().enumerate() {
            if let Some(glyph) = get_raster(ch, FontWeight::Regular, RasterHeight::Size32) {
                for (dy, row) in glyph.raster().iter().enumerate() {
                    for (dx, alpha) in row.iter().copied().enumerate() {
                        if alpha == 0 {
                            continue;
                        }
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = (x + index as u32 * step + dx as u32) * scale + sx;
                                let py = (y + dy as u32) * scale + sy;
                                if self.rect.contains(px, py) {
                                    let at = ((py - self.rect.y) * self.rect.width + px
                                        - self.rect.x)
                                        as usize;
                                    self.pixels[at] = blend(d::TEXT, self.pixels[at], alpha);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
fn blend(front: u32, back: u32, alpha: u8) -> u32 {
    let a = u32::from(alpha);
    let mut result = 0;
    for shift in [0, 8, 16] {
        let channel =
            (((front >> shift) & 255) * a + ((back >> shift) & 255) * (255 - a) + 127) / 255;
        result |= channel << shift;
    }
    result
}
