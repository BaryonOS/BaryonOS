//! Bounded paper calculator surface. The immutable proportional UI atlas is
//! shared with the compositor; no parser, device, allocator or model mutation.
use crate::{
    decimal::Value,
    model::{Model, Notice, LABELS},
};
use nieuw_os_graphics::{Canvas, Font, Rect as GraphicRect, Size, Weight};
#[path = "../../../../ui/native/src/design.rs"]
pub mod design;
use design as d;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Rect {
    pub fn contains(self, x: u32, y: u32) -> bool {
        x >= self.x
            && y >= self.y
            && u64::from(x) < u64::from(self.x) + u64::from(self.width)
            && u64::from(y) < u64::from(self.y) + u64::from(self.height)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale: u32,
}
impl Viewport {
    pub const fn width(self) -> u32 {
        self.width
    }
    pub const fn height(self) -> u32 {
        self.height
    }
    pub const fn scale(self) -> u32 {
        self.scale
    }
    pub fn new(width: u32, height: u32, scale: u32) -> Option<Self> {
        if !matches!(scale, 1 | 2)
            || width > 1920
            || height > 1080
            || width / scale < 320
            || height / scale < 240
        {
            None
        } else {
            Some(Self {
                width,
                height,
                scale,
            })
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub expression: Rect,
    pub title_y: u32,
    pub result_y: u32,
    pub status_y: u32,
    pub grid_y: u32,
    pub gap: u32,
    pub button_width: u32,
    pub button_height: u32,
    pub x: u32,
    pub width: u32,
    pub compact: bool,
}
impl Layout {
    pub fn new(view: Viewport) -> Self {
        let (w, h) = (view.width / view.scale, view.height / view.scale);
        let compact = h < 360;
        let pad = if compact { 16 } else { 24 };
        let width = (w - 2 * pad).min(520);
        let x = (w - width) / 2;
        let expression = Rect {
            x,
            y: if compact { 8 } else { 24 },
            width,
            height: if compact { 26 } else { 32 },
        };
        let result_y = if compact { 42 } else { 80 };
        let status_y = if compact { 90 } else { 136 };
        let grid_y = if compact { 116 } else { 184 };
        let gap = 1;
        let bottom = if compact { 12 } else { 24 };
        let button_height = ((h - bottom - grid_y - 3 * gap) / 4).min(72);
        Self {
            expression,
            title_y: pad,
            result_y,
            status_y,
            grid_y,
            gap,
            button_width: (width - 4 * gap) / 5,
            button_height,
            x,
            width,
            compact,
        }
    }
    pub fn button(self, index: usize) -> Rect {
        Rect {
            x: self.x + (index as u32 % 5) * (self.button_width + self.gap),
            y: self.grid_y + (index as u32 / 5) * (self.button_height + self.gap),
            width: self.button_width,
            height: self.button_height,
        }
    }
    pub fn hit(self, x: u32, y: u32) -> Option<usize> {
        (0..20).find(|i| self.button(*i).contains(x, y))
    }
}
/// Compatibility metric for legacy callers; editable formula hit testing must
/// use expression_caret_at, whose positions follow the actual glyph advances.
pub fn character_width(scale: u32) -> u32 {
    formula_font(scale).measure("0").div_ceil(scale.max(1))
}
fn formula_font(scale: u32) -> Font {
    Font::new(Size::Body, Weight::Regular, scale == 2)
}
fn available_formula_width(layout: Layout, scale: u32) -> u64 {
    u64::from(layout.expression.width.saturating_sub(16) * scale).saturating_sub(u64::from(scale))
}
/// The expression is bounded ASCII. Keep the caret inside the same viewport
/// used for rendering, selection and pointer hit testing, without mono cells.
pub fn expression_start(model: &Model, layout: Layout, scale: u32) -> usize {
    let font = formula_font(scale);
    let bytes = model.expression().as_bytes();
    let mut start = model.caret();
    let mut advance = 0u64;
    while start != 0 {
        let next = advance + u64::from(font.advance(char::from(bytes[start - 1])));
        if next / 64 > available_formula_width(layout, scale) {
            break;
        }
        advance = next;
        start -= 1;
    }
    start
}
fn expression_end(model: &Model, layout: Layout, scale: u32, start: usize) -> usize {
    let font = formula_font(scale);
    let mut advance = 0u64;
    let mut end = start;
    for byte in &model.expression().as_bytes()[start..] {
        let next = advance + u64::from(font.advance(char::from(*byte)));
        if next / 64 > available_formula_width(layout, scale) {
            break;
        }
        advance = next;
        end += 1;
    }
    end
}
/// `logical_x` is the app-local pointer coordinate after scale conversion.
/// Choose the nearest insertion boundary using the exact same 26.6 advances
/// and cropped viewport as the painted formula. No model state is changed.
pub fn expression_caret_at(model: &Model, layout: Layout, scale: u32, logical_x: u32) -> usize {
    let start = expression_start(model, layout, scale);
    let end = expression_end(model, layout, scale, start);
    let pointer = u64::from(logical_x.saturating_sub(layout.expression.x + 8)) * u64::from(scale);
    let font = formula_font(scale);
    let mut advance = 0u64;
    for index in start..end {
        let next =
            advance + u64::from(font.advance(char::from(model.expression().as_bytes()[index])));
        if pointer * 2 < advance / 64 + next / 64 {
            return index;
        }
        advance = next;
    }
    end
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Tile,
    Capacity,
}

pub fn render_tile(
    model: &Model,
    view: Viewport,
    tile: Rect,
    out: &mut [u32],
) -> Result<(), Error> {
    if tile.width == 0
        || tile.height == 0
        || tile.width > 32
        || tile.height > 8
        || u64::from(tile.x) + u64::from(tile.width) > u64::from(view.width)
        || u64::from(tile.y) + u64::from(tile.height) > u64::from(view.height)
    {
        return Err(Error::Tile);
    }
    let count = (tile.width * tile.height) as usize;
    if out.len() < count {
        return Err(Error::Capacity);
    }
    let mut canvas = Canvas::new(
        GraphicRect::new(tile.x as i32, tile.y as i32, tile.width, tile.height),
        &mut out[..count],
    )
    .map_err(|_| Error::Tile)?;
    canvas.clear(d::BACKGROUND);
    let mut paint = Paint {
        canvas,
        scale: view.scale,
    };
    let layout = Layout::new(view);
    let field = paint.physical(layout.expression);
    if field.intersects(paint.canvas.bounds()) {
        paint.formula(model, layout);
    }
    let title = Font::new(Size::Title, Weight::Semibold, view.scale == 2);
    let result_area = GraphicRect::new(
        (layout.x * view.scale) as i32,
        ((layout.result_y - 18) * view.scale) as i32,
        layout.width * view.scale,
        title.height() + 18 * view.scale,
    );
    if result_area.intersects(paint.canvas.bounds()) {
        let formatted = model
            .answer()
            .map_or(Value::default(), |answer| answer.value)
            .format();
        let result = formatted.as_str();
        let body = Font::new(Size::Body, Weight::Semibold, view.scale == 2);
        let font = if title.measure(result) <= layout.width * view.scale {
            title
        } else {
            body
        };
        let stale = model.answer().is_some() && !model.current();
        let value_x = (layout.x + layout.width) * view.scale - font.measure(result);
        paint.canvas.text(
            value_x as i32,
            (layout.result_y * view.scale) as i32,
            result,
            font,
            if stale || model.answer().is_none() {
                d::MUTED
            } else {
                d::TEXT
            },
        );
        if stale && !layout.compact {
            paint.text(
                layout.x,
                layout.result_y - 18,
                "Vorig resultaat",
                Size::Small,
                Weight::Regular,
                d::MUTED,
            );
        }
    }
    let status = if model.notice() != Notice::None {
        model.notice().label()
    } else if model.answer().is_some_and(|answer| answer.rounded) && model.current() {
        "Afgerond op 9 decimalen."
    } else {
        ""
    };
    let color = match model.notice() {
        Notice::ClipboardCopied => d::ACCENT,
        Notice::None => d::MUTED,
        _ => d::DANGER,
    };
    paint.notice(layout, status, color);
    paint.fill(
        Rect {
            x: layout.x,
            y: layout.grid_y - 12,
            width: layout.width,
            height: 1,
        },
        d::LINE,
    );
    for (index, original_label) in LABELS.iter().enumerate() {
        let rect = layout.button(index);
        if !paint.physical(rect).intersects(paint.canvas.bounds()) {
            continue;
        }
        let primary = index == 17;
        let operator = matches!(index, 3 | 8 | 13 | 18);
        let hover = model.hover() == Some(index);
        let pressed = model.pressed() == Some(index);
        let background = if primary {
            if hover || pressed {
                d::ACCENT_SOFT
            } else {
                d::ACCENT
            }
        } else if hover || pressed || operator {
            d::ACCENT_SOFT
        } else {
            d::SURFACE
        };
        paint.fill(rect, background);
        if model.focus() == index + 1 && model.active() {
            let physical = paint.physical(rect);
            paint
                .canvas
                .stroke_rounded_rect(physical, 0, 2 * view.scale, d::ACCENT);
        }
        let label = match index {
            3 => "÷",
            8 => "×",
            _ => original_label,
        };
        let large = Font::new(Size::Title, Weight::Semibold, view.scale == 2);
        let font = if large.height() + 8 * view.scale <= rect.height * view.scale
            && large.measure(label) + 12 * view.scale <= rect.width * view.scale
        {
            large
        } else {
            Font::new(Size::Body, Weight::Regular, view.scale == 2)
        };
        let x =
            rect.x * view.scale + (rect.width * view.scale).saturating_sub(font.measure(label)) / 2;
        let y = rect.y * view.scale + (rect.height * view.scale).saturating_sub(font.height()) / 2;
        let color = if primary && !hover && !pressed {
            d::SURFACE
        } else if primary || operator {
            d::ACCENT
        } else {
            d::TEXT
        };
        paint.canvas.text(x as i32, y as i32, label, font, color);
    }
    Ok(())
}
struct Paint<'a> {
    canvas: Canvas<'a>,
    scale: u32,
}
impl Paint<'_> {
    fn physical(&self, rect: Rect) -> GraphicRect {
        GraphicRect::new(
            (rect.x * self.scale) as i32,
            (rect.y * self.scale) as i32,
            rect.width * self.scale,
            rect.height * self.scale,
        )
    }
    fn fill(&mut self, rect: Rect, color: u32) {
        self.canvas.fill_rect(self.physical(rect), color);
    }
    fn text(&mut self, x: u32, y: u32, text: &str, size: Size, weight: Weight, color: u32) {
        self.canvas.text(
            (x * self.scale) as i32,
            (y * self.scale) as i32,
            text,
            Font::new(size, weight, self.scale == 2),
            color,
        );
    }
    fn formula(&mut self, model: &Model, layout: Layout) {
        let font = formula_font(self.scale);
        let start = expression_start(model, layout, self.scale);
        let end = expression_end(model, layout, self.scale, start);
        let origin = (layout.expression.x + 8) * self.scale;
        let top = layout.expression.y * self.scale;
        let height = layout.expression.height * self.scale;
        let selection = model.selection();
        let mut advance = 0u64;
        let mut caret = 0u32;
        for index in start..end {
            let next =
                advance + u64::from(font.advance(char::from(model.expression().as_bytes()[index])));
            if index == model.caret() {
                caret = (advance / 64) as u32;
            }
            if selection.contains(&index) {
                self.canvas.fill_rect(
                    GraphicRect::new(
                        (origin + (advance / 64) as u32) as i32,
                        (top + 4 * self.scale) as i32,
                        ((next / 64) - (advance / 64)) as u32,
                        height - 8 * self.scale,
                    ),
                    d::ACCENT_SOFT,
                );
            }
            advance = next;
        }
        if model.caret() == end {
            caret = (advance / 64) as u32;
        }
        let value = &model.expression()[start..end];
        let y = top + height.saturating_sub(font.height()) / 2;
        if value.is_empty() {
            self.canvas
                .text(origin as i32, y as i32, "0", font, d::MUTED);
        } else {
            self.canvas
                .text(origin as i32, y as i32, value, font, d::TEXT);
        }
        if model.focus() == 0 && model.active() {
            self.canvas.fill_rect(
                GraphicRect::new(
                    (origin + caret) as i32,
                    (top + 4 * self.scale) as i32,
                    self.scale,
                    height - 8 * self.scale,
                ),
                d::TEXT,
            );
        }
        self.canvas.fill_rect(
            GraphicRect::new(
                (layout.expression.x * self.scale) as i32,
                (top + height - self.scale) as i32,
                layout.expression.width * self.scale,
                self.scale,
            ),
            if model.active() && model.focus() == 0 {
                d::ACCENT
            } else {
                d::BORDER
            },
        );
    }
    fn notice(&mut self, layout: Layout, value: &str, color: u32) {
        let font = Font::new(Size::Small, Weight::Regular, self.scale == 2);
        let area = GraphicRect::new(
            (layout.x * self.scale) as i32,
            (layout.status_y * self.scale) as i32,
            layout.width * self.scale,
            font.height(),
        );
        if value.is_empty() || !area.intersects(self.canvas.bounds()) {
            return;
        }
        let mut end = 0;
        for (index, ch) in value.char_indices() {
            if font.measure(&value[..index + ch.len_utf8()]) > layout.width * self.scale {
                break;
            }
            end = index + ch.len_utf8();
        }
        self.canvas.text(
            (layout.x * self.scale) as i32,
            (layout.status_y * self.scale) as i32,
            &value[..end],
            font,
            color,
        );
    }
}
