//! Scalar private-RAM tiles. No framebuffer mapping, device I/O or syscall.
use crate::{
    controller::{DocumentTool, Error as UiError, Focus, Screen, Status, View},
    design as d, text, Rect, Scale, Viewport, TILE_HEIGHT, TILE_WIDTH,
};
use nieuw_os_graphics::theme::{self, Palette};
use nieuw_os_graphics::{Canvas, Font, Icon, Rect as GraphicRect, Size, Weight};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

pub const MAX_TEXT_ROWS: usize = 48;
/// Fixed material nodes, pagination controls and one entry per domain object.
pub const MAX_SEMANTIC_NODES: usize = 8 + volume_core::PER_DOMAIN;

// Explicit paint roles, never a post-render inversion or luminance filter.
// Cyan selection uses black foreground; yellow primary controls also use black.
const HIGH_CONTRAST: Palette = Palette {
    desktop_start: 0x000000,
    desktop_end: 0x000000,
    glow: 0x000000,
    surface: 0x000000,
    background: 0x000000,
    text: 0xffffff,
    muted: 0xb8b8b8,
    accent: 0xffff00,
    accent_soft: 0x00ffff,
    border: 0xffffff,
};
const HIGH_CONTRAST_DANGER: u32 = 0xffb4b4;

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
    pub tool: Rect,
    pub context_menu: Rect,
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
    content_left: u32,
    sidebar_width: u32,
    character_width: usize,
}
impl Layout {
    pub fn new(viewport: Viewport) -> Self {
        let width = viewport.logical_width();
        let height = viewport.logical_height();
        let compact = height < 360;
        let editor_top = if compact { 120 } else { 144 };
        let controls_y = if compact { 64 } else { 104 };
        let sidebar_width = if width >= 960 {
            208
        } else if width >= 720 {
            168
        } else {
            0
        };
        let content_left = if sidebar_width > 0 {
            sidebar_width + 24
        } else {
            d::PAD
        };
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
                x: content_left,
                y: controls_y,
                width: if width >= 560 { 164 } else { button_width },
                height: d::CONTROL_HEIGHT,
            },
            refresh: Rect {
                x: content_left
                    + if width >= 560 {
                        176
                    } else {
                        button_width + 2 * d::SPACE
                    },
                y: controls_y,
                width: if width >= 560 { 44 } else { button_width },
                height: d::CONTROL_HEIGHT,
            },
            back: Rect {
                x: d::PAD,
                y: d::PAD,
                width: if width < 480 { 44 } else { 100 },
                height: d::CONTROL_HEIGHT,
            },
            save: Rect {
                x: width - d::PAD - if width < 480 { 100 } else { 112 },
                y: d::PAD,
                width: if width < 480 { 100 } else { 112 },
                height: d::CONTROL_HEIGHT,
            },
            tool: Rect {
                x: d::PAD + if width < 480 { 52 } else { 112 },
                y: d::PAD,
                width: if width < 480 { 108 } else { 156 },
                height: d::CONTROL_HEIGHT,
            },
            context_menu: Rect {
                x: width - d::PAD - 44,
                y: 68,
                width: 44,
                height: 44,
            },
            editor: Rect {
                x: d::PAD,
                y: editor_top,
                width: width - 2 * d::PAD,
                height: height - editor_top - 64,
            },
            status: Rect {
                x: content_left,
                y: height - 48,
                width: width - content_left - d::PAD,
                height: 48,
            },
            list_bottom: height - 56,
            list_top: if compact { 120 } else { 168 },
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
            content_left,
            sidebar_width,
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
    pub const fn content_left(self) -> u32 {
        self.content_left
    }
    pub fn object(self, row: usize) -> Rect {
        Rect {
            x: self.content_left,
            y: self
                .list_top
                .saturating_add((row as u32).saturating_mul(72)),
            width: self.width - self.content_left - d::PAD,
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
        let p = if self.view.high_contrast {
            HIGH_CONTRAST
        } else {
            theme::MATERIAAL
        };
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
            palette: p,
            high_contrast: self.view.high_contrast,
            rect,
            pixels: &mut out[..count],
            scale: viewport.scale(),
            cursor: self
                .view
                .cursor
                .map(|(x, y)| (x / viewport.scale().factor(), y / viewport.scale().factor())),
        };
        paint.pixels.fill(p.background);
        if let Some(desktop) = self.view.desktop.filter(|desktop| desktop.overlay()) {
            paint.desktop(desktop, viewport);
            paint.pointer(self.view.cursor.filter(|_| self.view.draw_pointer));
            return Ok(rect);
        }
        let layout = self.layout;
        match self.view.screen {
            Screen::Material => {
                paint.material_chrome(layout, viewport);
                paint.title(layout.content_left, layout.title_y, "Documenten");
                if viewport.logical_height() >= 360 {
                    paint.small(layout.content_left, 62, "Op dit apparaat", p.muted);
                }
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
                    if viewport.logical_width() < 560 {
                        "Nieuw"
                    } else {
                        "Nieuw document"
                    },
                    self.view.focus == Focus::New,
                    self.view.editable,
                    ButtonStyle::Primary,
                );
                if viewport.logical_width() >= 560 {
                    paint.icon_button(
                        layout.refresh,
                        Icon::Refresh,
                        self.view.focus == Focus::Refresh,
                        true,
                    );
                } else {
                    paint.button(
                        layout.refresh,
                        "Vernieuwen",
                        self.view.focus == Focus::Refresh,
                        true,
                        ButtonStyle::Secondary,
                    );
                }
                if !self.view.list_loaded {
                    paint.wrapped(
                        layout.content_left,
                        layout.list_top,
                        "Je materiaal wordt geladen.",
                        viewport.logical_width() - layout.content_left - d::PAD,
                        p.muted,
                    );
                } else if self.view.objects.is_empty() {
                    let available = viewport.logical_width() - layout.content_left - d::PAD;
                    let icon_x = layout.content_left;
                    let top = layout.list_top
                        + if viewport.logical_height() > 480 {
                            40
                        } else {
                            8
                        };
                    paint.icon(
                        Rect {
                            x: icon_x,
                            y: top,
                            width: 32,
                            height: 32,
                        },
                        Icon::Folder,
                        p.accent,
                        1,
                    );
                    let title = "Nog geen documenten";
                    paint.medium(layout.content_left + 52, top, title, p.text);
                    let body = if available >= 360 {
                        "Maak een document om aan de slag te gaan."
                    } else {
                        "Maak je eerste document."
                    };
                    paint.wrapped(
                        layout.content_left + 52,
                        top + 32,
                        body,
                        available.saturating_sub(52),
                        p.muted,
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
                        let focused = self.view.focus == Focus::Object(index);
                        let hovered = paint.cursor.is_some_and(|(x, y)| row.contains(x, y));
                        let selected = focused || hovered;
                        paint.fill(row, if selected { p.accent_soft } else { p.surface });
                        paint.fill(
                            Rect {
                                x: row.x,
                                y: row.y + row.height,
                                width: row.width,
                                height: 1,
                            },
                            if self.view.high_contrast && selected {
                                p.background
                            } else {
                                paint.divider()
                            },
                        );
                        if focused {
                            paint.fill(
                                Rect {
                                    x: row.x,
                                    y: row.y + 12,
                                    width: 3,
                                    height: row.height - 24,
                                },
                                paint.selected(selected, p.accent),
                            );
                        }
                        paint.icon(
                            Rect {
                                x: row.x + 14,
                                y: row.y + 20,
                                width: 20,
                                height: 24,
                            },
                            Icon::Material,
                            paint.selected(selected, p.accent),
                            1,
                        );
                        let name = object
                            .name
                            .get(..object.name_length)
                            .and_then(|b| core::str::from_utf8(b).ok())
                            .unwrap_or("Naam niet leesbaar");
                        paint.wrapped(
                            row.x + 56,
                            row.y + 6,
                            name,
                            row.width.saturating_sub(96),
                            paint.selected(selected, p.text),
                        );
                        if paint.measure(name, Size::Body, Weight::Regular)
                            <= row.width.saturating_sub(96)
                        {
                            paint.small(
                                row.x + 56,
                                row.y + 36,
                                "Document · Op dit apparaat",
                                paint.selected(selected, p.muted),
                            );
                        }
                        paint.icon(
                            Rect {
                                x: row.x + row.width - 32,
                                y: row.y + 24,
                                width: 16,
                                height: 16,
                            },
                            Icon::ChevronRight,
                            paint.selected(selected, p.muted),
                            1,
                        );
                        if focused {
                            paint.focus(row);
                        }
                    }
                }
            }
            Screen::Editor | Screen::ConfirmClose => {
                paint.back_button(layout.back, self.view.focus == Focus::Back);
                paint.button(
                    layout.tool,
                    if layout.tool.width < 156 {
                        match self.view.tool {
                            DocumentTool::Write => "Lezen",
                            DocumentTool::Read => "Schrijven",
                        }
                    } else {
                        tool_action(self.view.tool)
                    },
                    self.view.focus == Focus::Tool,
                    !self.view.saving && self.view.screen == Screen::Editor,
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
                paint.clipped_medium(
                    Rect {
                        x: d::PAD,
                        y: 84,
                        width: layout.context_menu.x - d::PAD - 12,
                        height: 24,
                    },
                    title,
                    p.text,
                );
                paint.button(
                    layout.context_menu,
                    "···",
                    self.view.focus == Focus::ContextMenu && self.view.document_menu.is_none(),
                    !self.view.saving && self.view.screen == Screen::Editor,
                    ButtonStyle::Secondary,
                );
                if viewport.logical_height() >= 360 {
                    paint.small(
                        d::PAD,
                        116,
                        match self.view.tool {
                            DocumentTool::Write => "Schrijven · Op dit apparaat",
                            DocumentTool::Read => "Lezen · Dezelfde tekst, zonder bewerken",
                        },
                        p.muted,
                    );
                }
                paint.fill(layout.editor, p.surface);
                paint.fill(
                    Rect {
                        x: layout.editor.x,
                        y: layout.editor.y,
                        width: layout.editor.width,
                        height: 1,
                    },
                    paint.divider(),
                );
                if self.view.focus == Focus::Document && self.view.screen == Screen::Editor {
                    paint.fill(
                        Rect {
                            x: layout.editor.x.saturating_sub(8),
                            y: layout.editor.y + 8,
                            width: 3,
                            height: 24,
                        },
                        p.accent,
                    );
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
                            let is_selected =
                                selected.start < bytes.end && selected.end > bytes.start;
                            if is_selected {
                                paint.fill(
                                    Rect {
                                        x,
                                        y,
                                        width: (width.max(1) * layout.character_width()) as u32,
                                        height: 16,
                                    },
                                    p.accent_soft,
                                );
                            }
                            if !matches!(ch, '\n' | '\r' | '\t') {
                                paint.character(x, y, ch, paint.selected(is_selected, p.text));
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
                            paint.selected(selected.contains(&self.view.caret), p.accent),
                        );
                    }
                }
            }
        }
        let status_x = if self.view.screen == Screen::Material {
            layout.status.x
        } else {
            d::PAD
        };
        let status_width = viewport.logical_width() - status_x - d::PAD;
        paint.fill(
            Rect {
                x: status_x,
                y: layout.status.y.saturating_sub(8),
                width: status_width,
                height: 1,
            },
            paint.divider(),
        );
        paint.wrapped(
            status_x,
            layout.status.y,
            status_label(self.view.status),
            status_width,
            if matches!(self.view.status, Status::Failed(_)) {
                paint.danger()
            } else {
                p.muted
            },
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
                paint.divider(),
            );
            paint.fill(
                Rect {
                    x: layout.status.x,
                    y: layout.status.y + 24,
                    width,
                    height: 4,
                },
                p.accent,
            );
        }
        if self.view.screen == Screen::ConfirmClose {
            // Opaque application-local backdrop: no half-obscured controls or
            // text remain visible behind this modal. This grants no system-
            // dialog privilege and does not read a framebuffer/backbuffer.
            paint.pixels.fill(if self.view.high_contrast {
                p.background
            } else {
                blend(p.text, p.background, 24)
            });
            paint.outlined(layout.modal, p.surface, p.border, d::SURFACE_RADIUS);
            paint.label(
                layout.modal.x + 16,
                layout.modal.y + 16,
                "Nog niet opgeslagen",
                p.text,
            );
            paint.wrapped(
                layout.modal.x + 16,
                layout.modal.y + 48,
                "Bewaar je wijzigingen of ga terug.",
                layout.modal.width - 32,
                p.text,
            );
            paint.button(
                layout.modal_save,
                "Bewaren",
                self.view.focus == Focus::SaveAndBack,
                self.view.can_save,
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
        if let Some(menu) = self
            .view
            .document_menu
            .filter(|_| self.view.screen == Screen::Editor && !self.view.saving)
        {
            let r = menu.bounds.physical(viewport.scale());
            // Distance falloff over this private tile only, not backdrop blur.
            if !self.view.high_contrast {
                paint
                    .canvas()
                    .soft_shadow(
                        GraphicRect::new(r.x as i32, r.y as i32, r.width, r.height),
                        8 * viewport.scale().factor(),
                        p.text,
                        52,
                    )
                    .map_err(|_| Error::Tile)?;
            }
            paint.outlined(menu.bounds, p.surface, p.border, d::SURFACE_RADIUS);
            for index in 0..4 {
                paint.button(
                    menu.item(index),
                    menu_label(index, self.view.tool),
                    menu.selected == index,
                    index != 2 || self.view.can_save,
                    ButtonStyle::Secondary,
                );
            }
        }
        if let Some((x, y)) = self.view.cursor.filter(|(x, y)| {
            self.view.draw_pointer && *x < viewport.width() && *y < viewport.height()
        }) {
            // Pointer uses physical pixels and contrasting outline, independent
            // of control color. A native compositor can own this in its layer.
            for dy in 0..14 {
                for dx in 0..=dy / 2 {
                    paint.pixel(
                        i64::from(x + dx),
                        i64::from(y + dy),
                        if dx == 0 || dx == dy / 2 || dy == 13 {
                            p.surface
                        } else {
                            p.text
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
        if let Some(desktop) = self.view.desktop.filter(|desktop| desktop.overlay()) {
            visit(SemanticNode {
                id: SemanticId::Window,
                parent: None,
                role: Role::Window,
                label: "Nieuw OS",
                bounds: self.view.viewport.bounds(),
                enabled: true,
                ..SemanticNode::EMPTY
            });
            for (target, bounds, label) in
                desktop.controls(self.view.viewport).into_iter().flatten()
            {
                if target == crate::desktop::Target::Password && desktop.is_guest() {
                    continue;
                }
                // Password bytes never enter the semantic tree. The native
                // profile cannot grant authenticated access to a real account.
                visit(SemanticNode {
                    id: SemanticId::Desktop(target),
                    parent: Some(SemanticId::Window),
                    role: if matches!(
                        target,
                        crate::desktop::Target::Password | crate::desktop::Target::SearchField
                    ) {
                        Role::TextEditor
                    } else {
                        Role::Button
                    },
                    label,
                    value: if target == crate::desktop::Target::SearchField {
                        Some(desktop.query())
                    } else {
                        None
                    },
                    bounds: bounds.physical(self.view.viewport.scale()),
                    enabled: true,
                    focused: desktop.focus() == target,
                    actions: SemanticActions {
                        activate: !matches!(
                            target,
                            crate::desktop::Target::Password | crate::desktop::Target::SearchField
                        ),
                        set_value: matches!(
                            target,
                            crate::desktop::Target::Password | crate::desktop::Target::SearchField
                        ),
                    },
                    ..SemanticNode::EMPTY
                });
            }
            return;
        }
        if let Some(menu) = self
            .view
            .document_menu
            .filter(|_| self.view.screen == Screen::Editor && !self.view.saving)
        {
            visit(SemanticNode {
                id: SemanticId::Window,
                role: Role::Window,
                label: "Materiaal",
                bounds: self.view.viewport.bounds(),
                enabled: true,
                ..SemanticNode::EMPTY
            });
            visit(SemanticNode {
                id: SemanticId::DocumentMenuDialog,
                parent: Some(SemanticId::Window),
                role: Role::Dialog,
                label: "Documentacties",
                bounds: menu.bounds.physical(self.view.viewport.scale()),
                enabled: true,
                ..SemanticNode::EMPTY
            });
            for (index, id) in [
                SemanticId::SelectAll,
                SemanticId::DocumentTool,
                SemanticId::Save,
                SemanticId::CancelDocumentMenu,
            ]
            .into_iter()
            .enumerate()
            {
                let enabled = index != 2 || self.view.can_save;
                visit(SemanticNode {
                    id,
                    parent: Some(SemanticId::DocumentMenuDialog),
                    role: Role::Button,
                    label: menu_label(index, self.view.tool),
                    bounds: menu.item(index).physical(self.view.viewport.scale()),
                    enabled,
                    focused: menu.selected == index,
                    actions: SemanticActions {
                        activate: enabled,
                        set_value: false,
                    },
                    ..SemanticNode::EMPTY
                });
            }
            visit(SemanticNode {
                id: SemanticId::Status,
                parent: Some(SemanticId::Window),
                role: Role::Status,
                label: status_label(self.view.status),
                bounds: self.layout.status.physical(self.view.viewport.scale()),
                enabled: true,
                ..SemanticNode::EMPTY
            });
            return;
        }
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
                        x: l.content_left,
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
                    SemanticId::DocumentTool,
                    Some(SemanticId::Window),
                    Role::Button,
                    tool_action(self.view.tool),
                    None,
                    l.tool,
                    Some(Focus::Tool),
                    !self.view.saving,
                );
                push(
                    SemanticId::DocumentMenu,
                    Some(SemanticId::Window),
                    Role::Button,
                    "Documentacties openen",
                    None,
                    l.context_menu,
                    Some(Focus::ContextMenu),
                    !self.view.saving,
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
                    self.view.can_save,
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
    Desktop(crate::desktop::Target),
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
    DocumentTool,
    DocumentMenu,
    DocumentMenuDialog,
    SelectAll,
    CancelDocumentMenu,
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

fn tool_action(tool: DocumentTool) -> &'static str {
    match tool {
        DocumentTool::Write => "Lezen · F4",
        DocumentTool::Read => "Schrijven · F4",
    }
}

fn menu_label(index: usize, tool: DocumentTool) -> &'static str {
    match index {
        0 => "Alles selecteren",
        1 => match tool {
            DocumentTool::Write => "Lezen",
            DocumentTool::Read => "Schrijven",
        },
        2 => "Opslaan",
        _ => "Annuleren",
    }
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
        Status::ClipboardCopied => "Tekst gekopieerd.",
        Status::ClipboardEmpty => "Het klembord is leeg.",
        Status::ClipboardSelection => "Selecteer eerst tekst.",
        Status::ClipboardTooLong => "Deze selectie past nog niet op het klembord.",
        Status::ClipboardUnavailable => "Kopiëren of plakken onderbroken. Probeer opnieuw.",
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
enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
}
struct Paint<'a> {
    palette: Palette,
    high_contrast: bool,
    rect: Rect,
    pixels: &'a mut [u32],
    scale: Scale,
    cursor: Option<(u32, u32)>,
}
impl Paint<'_> {
    fn divider(&self) -> u32 {
        if self.high_contrast {
            self.palette.border
        } else {
            d::LINE
        }
    }
    fn danger(&self) -> u32 {
        if self.high_contrast {
            HIGH_CONTRAST_DANGER
        } else {
            d::DANGER
        }
    }
    fn desktop_text(&self) -> u32 {
        if self.high_contrast {
            self.palette.text
        } else {
            d::DESKTOP_TEXT
        }
    }
    fn selected(&self, selected: bool, normal: u32) -> u32 {
        if self.high_contrast && selected {
            self.palette.background
        } else {
            normal
        }
    }
    fn control_text(&self, r: Rect, enabled: bool) -> u32 {
        if enabled {
            self.selected(
                self.cursor.is_some_and(|(x, y)| r.contains(x, y)),
                self.palette.text,
            )
        } else {
            self.palette.muted
        }
    }
    fn surface(&mut self, r: Rect, normal: u32, radius: u32) {
        if self.high_contrast {
            self.outlined(r, self.palette.surface, self.palette.border, radius);
        } else {
            self.rounded(r, normal, radius);
        }
    }
    fn material_chrome(&mut self, layout: Layout, viewport: Viewport) {
        let p = self.palette;
        let h = viewport.logical_height();
        if layout.sidebar_width > 0 {
            self.fill(
                Rect {
                    x: 0,
                    y: 0,
                    width: layout.sidebar_width,
                    height: h,
                },
                if self.high_contrast {
                    p.background
                } else {
                    0xf0efe9
                },
            );
            self.fill(
                Rect {
                    x: layout.sidebar_width - 1,
                    y: 0,
                    width: 1,
                    height: h,
                },
                self.divider(),
            );
            self.small(24, 28, "MIJN WERK", p.muted);
            self.fill(
                Rect {
                    x: 0,
                    y: 80,
                    width: layout.sidebar_width,
                    height: 44,
                },
                p.accent_soft,
            );
            self.fill(
                Rect {
                    x: 0,
                    y: 80,
                    width: 3,
                    height: 44,
                },
                self.selected(true, p.accent),
            );
            self.icon(
                Rect {
                    x: 24,
                    y: 92,
                    width: 20,
                    height: 20,
                },
                Icon::Folder,
                self.selected(true, p.accent),
                1,
            );
            self.medium(56, 90, "Documenten", self.selected(true, p.text));
            self.small(24, h.saturating_sub(34), "Dit apparaat", p.muted);
        }
    }
    fn icon_button(&mut self, r: Rect, icon: Icon, focused: bool, enabled: bool) {
        self.button(r, "", focused, enabled, ButtonStyle::Secondary);
        self.icon(
            Rect {
                x: r.x + (r.width - 20) / 2,
                y: r.y + (r.height - 20) / 2,
                width: 20,
                height: 20,
            },
            icon,
            self.control_text(r, enabled),
            1,
        );
    }
    fn pointer(&mut self, cursor: Option<(u32, u32)>) {
        let p = self.palette;
        if let Some((x, y)) = cursor {
            for dy in 0..14 {
                for dx in 0..=dy / 2 {
                    self.pixel(
                        i64::from(x + dx),
                        i64::from(y + dy),
                        if dx == 0 || dx == dy / 2 || dy == 13 {
                            p.surface
                        } else {
                            p.text
                        },
                    );
                }
            }
        }
    }
    fn desktop(&mut self, desktop: &crate::desktop::Desktop, viewport: Viewport) {
        let p = self.palette;
        use crate::desktop::{Page, Target};
        let w = viewport.logical_width();
        let h = viewport.logical_height();
        let compact = h < 420;
        let factor = self.scale.factor();
        theme::wallpaper(
            &mut self.canvas(),
            GraphicRect::new(0, 0, viewport.width(), viewport.height()),
            p,
        );
        let width = w.saturating_sub(48).min(440);
        let x = (w - width) / 2;
        let y = if compact {
            0
        } else {
            h.saturating_sub(440) / 2
        };
        let controls = desktop.controls(viewport);
        match desktop.page() {
            Page::Accounts | Page::Login => {
                let panel = Rect {
                    x: x.saturating_sub(24),
                    y: y.saturating_sub(16),
                    width: width + 48,
                    height: if compact { h } else { 424 },
                };
                if !self.high_contrast {
                    self.rounded(
                        Rect {
                            x: panel.x + 2,
                            y: panel.y + 9,
                            ..panel
                        },
                        0x122636,
                        24,
                    );
                }
                self.surface(panel, p.surface, 24);
                if desktop.page() == Page::Accounts {
                    self.title(x, y + if compact { 12 } else { 24 }, "Welkom terug");
                    self.label(
                        x,
                        y + if compact { 54 } else { 78 },
                        "Kies je werkruimte om verder te gaan.",
                        p.muted,
                    );
                } else {
                    self.medium(
                        x + 60,
                        y + if compact { 12 } else { 26 },
                        if desktop.is_guest() {
                            "Gast"
                        } else {
                            "Eigen gebruiker"
                        },
                        p.text,
                    );
                    if !compact {
                        self.title(
                            x,
                            y + 72,
                            if desktop.is_guest() {
                                "Even aan de slag"
                            } else {
                                "Inloggen"
                            },
                        );
                    }
                    self.small(
                        x,
                        y + if compact { 54 } else { 126 },
                        if desktop.is_guest() {
                            "Een openbare werkruimte voor deze test."
                        } else {
                            "Voorbeeldaccount · gebruik alleen testinvoer"
                        },
                        p.muted,
                    );
                }
                for (target, r, label) in controls.into_iter().flatten() {
                    match target {
                        Target::Guest | Target::Personal => {
                            let selected = desktop.focus() == target;
                            self.outlined(
                                r,
                                if selected {
                                    p.accent_soft
                                } else {
                                    p.background
                                },
                                if self.high_contrast && selected {
                                    p.accent_soft
                                } else if selected {
                                    p.accent
                                } else {
                                    self.divider()
                                },
                                14,
                            );
                            let avatar = Rect {
                                x: r.x + 12,
                                y: r.y + (r.height - 44) / 2,
                                width: 44,
                                height: 44,
                            };
                            if self.high_contrast && selected {
                                self.rounded(avatar, p.background, 14);
                            } else {
                                self.surface(
                                    avatar,
                                    if selected { p.accent } else { 0xe2e8ec },
                                    14,
                                );
                            }
                            self.icon(
                                Rect {
                                    x: r.x + 22,
                                    y: r.y + (r.height - 24) / 2,
                                    width: 24,
                                    height: 24,
                                },
                                Icon::User,
                                if self.high_contrast {
                                    if selected {
                                        p.accent_soft
                                    } else {
                                        p.text
                                    }
                                } else if selected {
                                    p.surface
                                } else {
                                    p.muted
                                },
                                1,
                            );
                            self.medium(
                                r.x + 72,
                                r.y + if compact { 7 } else { 14 },
                                if target == Target::Guest {
                                    "Gast"
                                } else {
                                    label
                                },
                                self.selected(selected, p.text),
                            );
                            self.small(
                                r.x + 72,
                                r.y + if compact { 30 } else { 43 },
                                if target == Target::Guest {
                                    "Tijdelijke VM-werkruimte"
                                } else {
                                    "Aanmelden nog niet ingesteld"
                                },
                                self.selected(selected, p.muted),
                            );
                            self.icon(
                                Rect {
                                    x: r.x + r.width - 32,
                                    y: r.y + (r.height - 16) / 2,
                                    width: 16,
                                    height: 16,
                                },
                                Icon::ChevronRight,
                                self.selected(selected, p.muted),
                                1,
                            );
                            if self.high_contrast && selected {
                                self.focus(r);
                            }
                        }
                        Target::Back => {
                            self.icon_button(r, Icon::ChevronLeft, desktop.focus() == target, true)
                        }
                        Target::Password => {
                            if desktop.is_guest() {
                                self.label(
                                    r.x,
                                    r.y + 8,
                                    "Geen wachtwoord nodig voor de gast.",
                                    p.muted,
                                );
                            } else {
                                self.outlined(r, p.background, p.border, 10);
                                self.icon(
                                    Rect {
                                        x: r.x + 12,
                                        y: r.y + 12,
                                        width: 20,
                                        height: 20,
                                    },
                                    Icon::Lock,
                                    p.muted,
                                    1,
                                );
                                if desktop.password_len() == 0 {
                                    self.label(
                                        r.x + 44,
                                        r.y + 10,
                                        if desktop.unavailable() {
                                            "Aanmelden niet beschikbaar"
                                        } else {
                                            "Wachtwoord"
                                        },
                                        if desktop.unavailable() {
                                            self.danger()
                                        } else {
                                            p.muted
                                        },
                                    );
                                } else {
                                    for index in 0..desktop
                                        .password_len()
                                        .min(((r.width - 60) / 10) as usize)
                                    {
                                        self.canvas().ellipse(
                                            GraphicRect::new(
                                                ((r.x + 48 + index as u32 * 10) * factor) as i32,
                                                ((r.y + 20) * factor) as i32,
                                                5 * factor,
                                                5 * factor,
                                            ),
                                            p.text,
                                        );
                                    }
                                }
                                if desktop.focus() == target {
                                    self.focus(r);
                                }
                            }
                        }
                        Target::Submit => self.button(
                            r,
                            label,
                            desktop.focus() == target,
                            true,
                            ButtonStyle::Primary,
                        ),
                        Target::Fingerprint | Target::Fido2 => {
                            self.button(
                                r,
                                "",
                                desktop.focus() == target,
                                true,
                                ButtonStyle::Secondary,
                            );
                            self.icon(
                                Rect {
                                    x: r.x + 12,
                                    y: r.y + 12,
                                    width: 20,
                                    height: 20,
                                },
                                if target == Target::Fingerprint {
                                    Icon::Fingerprint
                                } else {
                                    Icon::Key
                                },
                                p.muted,
                                1,
                            );
                            self.small(
                                r.x + 40,
                                r.y + 14,
                                if target == Target::Fingerprint {
                                    "Vingerafdruk"
                                } else {
                                    "FIDO2-sleutel"
                                },
                                p.muted,
                            );
                        }
                        _ => {}
                    }
                }
                if !compact {
                    self.small(
                        x,
                        y + 370,
                        if desktop.page() == Page::Accounts {
                            "Nieuw OS · lokale werkruimte"
                        } else if desktop.is_guest() {
                            "Gast (VM-test) · uitsluitend testgegevens"
                        } else {
                            "Authenticatie is in deze testversie niet beschikbaar."
                        },
                        p.muted,
                    );
                }
            }
            Page::Home | Page::Search | Page::Material => {
                if desktop.page() == Page::Home {
                    self.title_color(
                        32,
                        if compact { 6 } else { 48 },
                        "Je werkruimte",
                        self.desktop_text(),
                    );
                    if !compact {
                        self.label(
                            32,
                            106,
                            "Open je documenten of zoek een programma.",
                            p.muted,
                        );
                    }
                    self.small(
                        w.saturating_sub(140),
                        16,
                        "Gast · VM-test",
                        self.desktop_text(),
                    );
                } else {
                    let panel_width = w.saturating_sub(48).min(620);
                    let top = if compact { 40 } else { 132 };
                    self.surface(
                        Rect {
                            x: (w - panel_width) / 2,
                            y: top,
                            width: panel_width,
                            height: if compact { 140 } else { 176 },
                        },
                        p.surface,
                        20,
                    );
                }
                for (target, r, _) in controls.into_iter().flatten() {
                    if target == Target::Material {
                        if desktop.page() == Page::Search {
                            self.rounded(r, p.accent_soft, 12);
                            self.icon(
                                Rect {
                                    x: r.x + 16,
                                    y: r.y + (r.height - 32) / 2,
                                    width: 28,
                                    height: 32,
                                },
                                Icon::Material,
                                self.selected(true, p.accent),
                                1,
                            );
                            self.medium(
                                r.x + 60,
                                r.y + 8,
                                "Materiaal",
                                self.selected(true, p.text),
                            );
                            if !compact {
                                self.small(
                                    r.x + 60,
                                    r.y + 40,
                                    "Documenten op dit apparaat",
                                    self.selected(true, p.muted),
                                );
                            }
                            self.icon(
                                Rect {
                                    x: r.x + r.width - 32,
                                    y: r.y + (r.height - 16) / 2,
                                    width: 16,
                                    height: 16,
                                },
                                Icon::ChevronRight,
                                self.selected(true, p.muted),
                                1,
                            );
                        } else {
                            self.surface(r, 0xeff7f7, 20);
                            self.surface(
                                Rect {
                                    x: r.x + 16,
                                    y: r.y + 16,
                                    width: if compact { 48 } else { 64 },
                                    height: if compact { 48 } else { 64 },
                                },
                                0xd2eae3,
                                16,
                            );
                            self.icon(
                                Rect {
                                    x: r.x + 28,
                                    y: r.y + 25,
                                    width: if compact { 24 } else { 40 },
                                    height: if compact { 32 } else { 46 },
                                },
                                Icon::Material,
                                p.accent,
                                2,
                            );
                            self.medium(
                                r.x + if compact { 80 } else { 18 },
                                r.y + if compact { 17 } else { 96 },
                                "Materiaal",
                                p.text,
                            );
                            self.small(
                                r.x + if compact { 80 } else { 18 },
                                r.y + if compact { 44 } else { 124 },
                                "Documenten",
                                p.muted,
                            );
                        }
                        if desktop.focus() == target {
                            self.focus(r);
                        }
                    } else if target == Target::SearchField {
                        self.outlined(r, p.background, p.border, 12);
                        self.icon(
                            Rect {
                                x: r.x + 12,
                                y: r.y + 12,
                                width: 20,
                                height: 20,
                            },
                            Icon::Search,
                            p.muted,
                            1,
                        );
                        self.label(
                            r.x + 44,
                            r.y + 10,
                            if desktop.query().is_empty() {
                                "Zoek een programma…"
                            } else {
                                desktop.query()
                            },
                            p.text,
                        );
                        if desktop.focus() == target {
                            self.focus(r);
                        }
                    }
                }
                if desktop.page() == Page::Search && !desktop.matches_material() {
                    let panel_width = w.saturating_sub(48).min(620);
                    self.label(
                        (w - panel_width) / 2 + 24,
                        if compact { 120 } else { 228 },
                        "Geen programma gevonden",
                        p.muted,
                    );
                }
                let bar_width = w.saturating_sub(24).min(680);
                let bar = Rect {
                    x: (w - bar_width) / 2,
                    y: h.saturating_sub(68),
                    width: bar_width,
                    height: 60,
                };
                if !self.high_contrast {
                    self.rounded(
                        Rect {
                            x: bar.x,
                            y: bar.y + 4,
                            ..bar
                        },
                        0x122c39,
                        18,
                    );
                }
                self.surface(bar, 0xf5fafb, 18);
                for (target, r, label) in desktop.controls(viewport).into_iter().flatten().take(3) {
                    if target == Target::Start {
                        self.icon_button(r, Icon::Apps, desktop.focus() == target, true);
                    } else if target == Target::Search {
                        self.surface(r, 0xe7eff2, 10);
                        self.icon(
                            Rect {
                                x: r.x + 12,
                                y: r.y + 12,
                                width: 20,
                                height: 20,
                            },
                            Icon::Search,
                            p.muted,
                            1,
                        );
                        if r.width > 96 {
                            self.label(r.x + 44, r.y + 10, "Zoeken", p.muted);
                        }
                        if desktop.focus() == target {
                            self.focus(r);
                        }
                    } else {
                        self.button(
                            r,
                            label,
                            desktop.focus() == target,
                            true,
                            ButtonStyle::Secondary,
                        );
                    }
                }
            }
        }
    }
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
        let f = self.scale.factor();
        self.canvas().rounded_rect(
            GraphicRect::new(r.x as i32, r.y as i32, r.width, r.height),
            radius * f,
            color,
        );
    }
    fn focus(&mut self, r: Rect) {
        let p = self.palette;
        let gap = d::FOCUS_OFFSET;
        let r = Rect {
            x: r.x.saturating_sub(gap),
            y: r.y.saturating_sub(gap),
            width: r.width + 2 * gap,
            height: r.height + 2 * gap,
        }
        .physical(self.scale);
        let factor = self.scale.factor();
        self.canvas().stroke_rounded_rect(
            GraphicRect::new(r.x as i32, r.y as i32, r.width, r.height),
            (d::CONTROL_RADIUS + gap) * factor,
            2 * factor,
            p.accent,
        );
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
        let p = self.palette;
        let margin = if focused { d::FOCUS_OFFSET + 2 } else { 0 };
        let paint_bounds = Rect {
            x: r.x.saturating_sub(margin),
            y: r.y.saturating_sub(margin),
            width: r.width + 2 * margin,
            height: r.height + 2 * margin,
        };
        if !paint_bounds.physical(self.scale).intersects(self.rect) {
            return;
        }
        let hovered = enabled && self.cursor.is_some_and(|(x, y)| r.contains(x, y));
        let (background, border, foreground) = if !enabled {
            if self.high_contrast {
                (p.surface, p.muted, p.muted)
            } else {
                (self.divider(), self.divider(), p.muted)
            }
        } else {
            match style {
                ButtonStyle::Primary => (
                    p.accent,
                    if hovered && !self.high_contrast {
                        p.text
                    } else {
                        p.accent
                    },
                    p.surface,
                ),
                ButtonStyle::Danger => (p.surface, self.danger(), self.danger()),
                _ => (
                    if hovered { p.accent_soft } else { p.surface },
                    if self.high_contrast && hovered {
                        p.accent_soft
                    } else {
                        p.border
                    },
                    self.selected(hovered, p.text),
                ),
            }
        };
        self.outlined(r, background, border, d::CONTROL_RADIUS);
        if self.high_contrast && !enabled {
            // A dashed boundary distinguishes unavailable controls without color.
            for x in (r.x + d::CONTROL_RADIUS..r.x + r.width - d::CONTROL_RADIUS).step_by(8) {
                for y in [r.y, r.y + r.height - 1] {
                    self.fill(
                        Rect {
                            x,
                            y,
                            width: 4,
                            height: 1,
                        },
                        background,
                    );
                }
            }
            for y in (r.y + d::CONTROL_RADIUS..r.y + r.height - d::CONTROL_RADIUS).step_by(8) {
                for x in [r.x, r.x + r.width - 1] {
                    self.fill(
                        Rect {
                            x,
                            y,
                            width: 1,
                            height: 4,
                        },
                        background,
                    );
                }
            }
        }
        if focused {
            self.focus(r);
        }
        let width = self.measure(label, Size::Body, Weight::Semibold);
        let x = r.x + r.width.saturating_sub(width) / 2;
        let line = self.font(Size::Body, Weight::Semibold).height() / self.scale.factor();
        self.medium(
            x,
            r.y + r.height.saturating_sub(line) / 2,
            label,
            foreground,
        );
    }
    fn back_button(&mut self, r: Rect, focused: bool) {
        let p = self.palette;
        if self.cursor.is_some_and(|(x, y)| r.contains(x, y)) {
            self.rounded(r, p.accent_soft, 6);
        }
        self.icon(
            Rect {
                x: r.x + 4,
                y: r.y + 12,
                width: 20,
                height: 20,
            },
            Icon::ChevronLeft,
            self.control_text(r, true),
            1,
        );
        if r.width >= 100 {
            self.medium(r.x + 30, r.y + 10, "Terug", self.control_text(r, true));
        }
        if focused {
            self.focus(r);
        }
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
                self.control_text(r, enabled),
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
    fn canvas(&mut self) -> Canvas<'_> {
        Canvas::new(
            GraphicRect::new(
                self.rect.x as i32,
                self.rect.y as i32,
                self.rect.width,
                self.rect.height,
            ),
            self.pixels,
        )
        .expect("validated private tile")
    }
    fn font(&self, size: Size, weight: Weight) -> Font {
        Font::new(size, weight, self.scale == Scale::Percent200)
    }
    fn measure(&self, label: &str, size: Size, weight: Weight) -> u32 {
        self.font(size, weight)
            .measure(label)
            .div_ceil(self.scale.factor())
    }
    fn ui_text(&mut self, x: u32, y: u32, label: &str, size: Size, weight: Weight, color: u32) {
        let font = self.font(size, weight);
        let f = self.scale.factor();
        self.canvas()
            .text((x * f) as i32, (y * f) as i32, label, font, color);
    }
    fn label(&mut self, x: u32, y: u32, label: &str, color: u32) {
        self.ui_text(x, y, label, Size::Body, Weight::Regular, color);
    }
    fn small(&mut self, x: u32, y: u32, label: &str, color: u32) {
        self.ui_text(x, y, label, Size::Small, Weight::Regular, color);
    }
    fn medium(&mut self, x: u32, y: u32, label: &str, color: u32) {
        self.ui_text(x, y, label, Size::Body, Weight::Semibold, color);
    }
    fn clipped_medium(&mut self, bounds: Rect, label: &str, color: u32) {
        let mut end = 0;
        for (start, ch) in label.char_indices() {
            let next = start + ch.len_utf8();
            if ch.is_control()
                || self.measure(&label[..next], Size::Body, Weight::Semibold) > bounds.width
            {
                break;
            }
            end = next;
        }
        self.medium(bounds.x, bounds.y, &label[..end], color);
    }
    fn icon(&mut self, r: Rect, icon: Icon, color: u32, stroke: u32) {
        let r = r.physical(self.scale);
        if !r.intersects(self.rect) {
            return;
        }
        let f = self.scale.factor();
        self.canvas().icon(
            GraphicRect::new(r.x as i32, r.y as i32, r.width, r.height),
            icon,
            color,
            stroke * f,
        );
    }
    fn wrapped(&mut self, x: u32, mut y: u32, label: &str, width: u32, color: u32) {
        let mut rest = label;
        while !rest.is_empty() {
            if y * self.scale.factor() >= self.rect.y + self.rect.height {
                return;
            }
            let mut take = rest.len();
            let mut space = None;
            for (index, ch) in rest.char_indices() {
                if ch == '\n' {
                    take = index;
                    break;
                }
                if ch.is_whitespace() {
                    space = Some(index);
                }
                if self.measure(&rest[..index + ch.len_utf8()], Size::Body, Weight::Regular) > width
                {
                    take = space.filter(|i| *i > 0).unwrap_or(index.max(ch.len_utf8()));
                    break;
                }
            }
            self.label(x, y, rest[..take].trim_end(), color);
            rest = rest[take..].trim_start_matches(char::is_whitespace);
            y = y.saturating_add(d::LINE_HEIGHT);
        }
    }
    fn title(&mut self, x: u32, y: u32, title: &str) {
        self.title_color(x, y, title, self.palette.text);
    }
    fn title_color(&mut self, x: u32, y: u32, title: &str, color: u32) {
        self.ui_text(x, y, title, Size::Title, Weight::Semibold, color);
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
