//! Scalar assigned-surface tiles. One existing token source and the approved
//! Noto font; no document model, display mapping, allocator or host rendering.
use crate::{
    decimal::Value,
    model::{Model, Notice, LABELS},
};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
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
        let pad = if compact { d::PAD / 2 } else { d::PAD };
        let width = (w - 2 * pad).min(520);
        let x = (w - width) / 2;
        let expression = Rect {
            x,
            y: pad + if compact { 22 } else { 40 },
            width,
            height: if compact { 24 } else { 36 },
        };
        let result_y = expression.y + expression.height + if compact { 4 } else { 12 };
        let status_y = result_y + if compact { 20 } else { 24 };
        let grid_y = status_y + if compact { 22 } else { 32 };
        let gap = if compact { d::SPACE / 4 * 3 } else { d::SPACE };
        let button_height = d::CONTROL_HEIGHT.min((h - pad - grid_y - 3 * gap) / 4);
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
pub fn character_width(scale: u32) -> u32 {
    get_raster_width(FontWeight::Regular, height(scale)).div_ceil(scale as usize) as u32
}
fn height(scale: u32) -> RasterHeight {
    if scale == 1 {
        RasterHeight::Size16
    } else {
        RasterHeight::Size32
    }
}
pub fn expression_start(model: &Model, layout: Layout, scale: u32) -> usize {
    let columns =
        (layout.expression.width.saturating_sub(16) / character_width(scale)).max(1) as usize;
    model.caret().saturating_sub(columns.saturating_sub(1))
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
    let mut paint = Paint {
        tile,
        pixels: &mut out[..count],
        scale: view.scale,
    };
    paint.pixels.fill(d::BACKGROUND);
    let layout = Layout::new(view);
    if layout.compact {
        paint.label(
            layout.x,
            layout.title_y,
            "Rekenmachine",
            d::TEXT,
            layout.width,
        );
    } else {
        paint.title(layout.x, layout.title_y, "Rekenmachine");
    }
    paint.label(
        layout.x
            + layout
                .width
                .saturating_sub(11 * character_width(view.scale)),
        layout.title_y + if layout.compact { 0 } else { 8 },
        "9 decimalen",
        d::MUTED,
        layout.width,
    );
    paint.boxed(
        layout.expression,
        d::SURFACE,
        if model.focus() == 0 && model.active() {
            d::ACCENT
        } else {
            d::BORDER
        },
    );
    let start = expression_start(model, layout, view.scale);
    let step = character_width(view.scale);
    let columns = ((layout.expression.width - 16) / step) as usize;
    let selection = model.selection();
    for (column, byte) in model
        .expression()
        .as_bytes()
        .iter()
        .enumerate()
        .skip(start)
        .take(columns)
    {
        let x = layout.expression.x + 8 + (column - start) as u32 * step;
        if selection.contains(&column) {
            paint.fill(
                Rect {
                    x,
                    y: layout.expression.y + 3,
                    width: step,
                    height: 18,
                },
                d::ACCENT_SOFT,
            );
        }
        paint.glyph(
            x,
            layout.expression.y + (layout.expression.height - 16) / 2,
            *byte as char,
            d::TEXT,
        );
    }
    if model.focus() == 0 && model.active() {
        paint.fill(
            Rect {
                x: layout.expression.x + 8 + (model.caret() - start) as u32 * step,
                y: layout.expression.y + 4,
                width: 1,
                height: layout.expression.height - 8,
            },
            d::TEXT,
        );
    }
    let formatted = model
        .answer()
        .map_or(Value::default(), |a| a.value)
        .format();
    let prefix = if model.answer().is_some() && !model.current() {
        "Vorig: "
    } else {
        "= "
    };
    paint.label(layout.x, layout.result_y, prefix, d::MUTED, layout.width);
    paint.label(
        layout.x + prefix.len() as u32 * step,
        layout.result_y,
        formatted.as_str(),
        d::TEXT,
        layout.width - prefix.len() as u32 * step,
    );
    let status = if model.notice() != Notice::None {
        model.notice().label()
    } else if model.answer().is_some_and(|a| a.rounded) && model.current() {
        "Afgerond op 9 decimalen."
    } else {
        "Punt of komma is decimaal."
    };
    paint.label(
        layout.x,
        layout.status_y,
        status,
        if model.notice() == Notice::ClipboardCopied {
            d::ACCENT
        } else if model.notice() != Notice::None {
            d::DANGER
        } else {
            d::MUTED
        },
        layout.width,
    );
    for (index, label) in LABELS.iter().enumerate() {
        let button = layout.button(index);
        let primary = index == 17;
        let hovered = model.hover() == Some(index);
        let pressed = model.pressed() == Some(index);
        paint.boxed(
            button,
            if pressed || hovered {
                d::ACCENT_SOFT
            } else if primary {
                d::ACCENT
            } else {
                d::SURFACE
            },
            if model.focus() == index + 1 && model.active() {
                d::ACCENT
            } else {
                d::BORDER
            },
        );
        if model.focus() == index + 1 && model.active() {
            paint.rounded(
                Rect {
                    x: button.x + 2,
                    y: button.y + 2,
                    width: button.width - 4,
                    height: button.height - 4,
                },
                d::ACCENT,
                d::CONTROL_RADIUS.saturating_sub(2),
            );
            paint.rounded(
                Rect {
                    x: button.x + 4,
                    y: button.y + 4,
                    width: button.width - 8,
                    height: button.height - 8,
                },
                if pressed || hovered {
                    d::ACCENT_SOFT
                } else if primary {
                    d::ACCENT
                } else {
                    d::SURFACE
                },
                d::CONTROL_RADIUS.saturating_sub(4),
            );
        }
        let text_width = label.len() as u32 * step;
        paint.label(
            button.x + (button.width - text_width) / 2,
            button.y + (button.height - 16) / 2,
            label,
            if primary && !hovered && !pressed {
                d::SURFACE
            } else {
                d::TEXT
            },
            button.width,
        );
    }
    Ok(())
}
struct Paint<'a> {
    tile: Rect,
    pixels: &'a mut [u32],
    scale: u32,
}
impl Paint<'_> {
    fn pixel(&mut self, x: u32, y: u32, color: u32) {
        if self.tile.contains(x, y) {
            let at = ((y - self.tile.y) * self.tile.width + x - self.tile.x) as usize;
            self.pixels[at] = color & 0xffffff;
        }
    }
    fn fill(&mut self, r: Rect, color: u32) {
        let (left, top) = (r.x * self.scale, r.y * self.scale);
        let (right, bottom) = ((r.x + r.width) * self.scale, (r.y + r.height) * self.scale);
        for y in top.max(self.tile.y)..bottom.min(self.tile.y + self.tile.height) {
            for x in left.max(self.tile.x)..right.min(self.tile.x + self.tile.width) {
                self.pixel(x, y, color);
            }
        }
    }
    fn rounded(&mut self, r: Rect, color: u32, radius: u32) {
        let (left, top) = (r.x * self.scale, r.y * self.scale);
        let (width, height) = (r.width * self.scale, r.height * self.scale);
        let radius = (radius * self.scale).min(width / 2).min(height / 2);
        for y in top.max(self.tile.y)..(top + height).min(self.tile.y + self.tile.height) {
            for x in left.max(self.tile.x)..(left + width).min(self.tile.x + self.tile.width) {
                let dx = if x < left + radius {
                    left + radius - x
                } else {
                    x.saturating_sub(left + width - radius - 1)
                };
                let dy = if y < top + radius {
                    top + radius - y
                } else {
                    y.saturating_sub(top + height - radius - 1)
                };
                if dx * dx + dy * dy <= radius * radius {
                    self.pixel(x, y, color);
                }
            }
        }
    }
    fn boxed(&mut self, r: Rect, fill: u32, border: u32) {
        self.rounded(r, border, d::CONTROL_RADIUS);
        if r.width > 2 && r.height > 2 {
            self.rounded(
                Rect {
                    x: r.x + 1,
                    y: r.y + 1,
                    width: r.width - 2,
                    height: r.height - 2,
                },
                fill,
                d::CONTROL_RADIUS.saturating_sub(1),
            );
        }
    }
    fn glyph(&mut self, x: u32, y: u32, ch: char, color: u32) {
        let Some(glyph) = get_raster(ch, FontWeight::Regular, height(self.scale)) else {
            return;
        };
        let (x, y) = (x * self.scale, y * self.scale);
        if x >= self.tile.x + self.tile.width
            || y >= self.tile.y + self.tile.height
            || x + glyph.width() as u32 <= self.tile.x
            || y + glyph.height() as u32 <= self.tile.y
        {
            return;
        }
        for (dy, row) in glyph.raster().iter().enumerate() {
            for (dx, alpha) in row.iter().copied().enumerate() {
                let (px, py) = (x + dx as u32, y + dy as u32);
                if alpha != 0 && self.tile.contains(px, py) {
                    let index = ((py - self.tile.y) * self.tile.width + px - self.tile.x) as usize;
                    let old = self.pixels[index];
                    let mut new = 0;
                    for shift in [0, 8, 16] {
                        new |= ((((color >> shift) & 255) * u32::from(alpha)
                            + ((old >> shift) & 255) * (255 - u32::from(alpha))
                            + 127)
                            / 255)
                            << shift;
                    }
                    self.pixels[index] = new;
                }
            }
        }
    }
    fn title(&mut self, x: u32, y: u32, text: &str) {
        let step = get_raster_width(FontWeight::Regular, RasterHeight::Size32) as u32;
        for (index, ch) in text.chars().enumerate() {
            let Some(glyph) = get_raster(ch, FontWeight::Regular, RasterHeight::Size32) else {
                continue;
            };
            let gx = (x + index as u32 * step) * self.scale;
            let gy = y * self.scale;
            if gx >= self.tile.x + self.tile.width
                || gy >= self.tile.y + self.tile.height
                || gx + glyph.width() as u32 * self.scale <= self.tile.x
                || gy + glyph.height() as u32 * self.scale <= self.tile.y
            {
                continue;
            }
            for (dy, row) in glyph.raster().iter().enumerate() {
                for (dx, alpha) in row.iter().copied().enumerate() {
                    if alpha == 0 {
                        continue;
                    }
                    for sy in 0..self.scale {
                        for sx in 0..self.scale {
                            let px = (x + index as u32 * step + dx as u32) * self.scale + sx;
                            let py = (y + dy as u32) * self.scale + sy;
                            if self.tile.contains(px, py) {
                                let at = ((py - self.tile.y) * self.tile.width + px - self.tile.x)
                                    as usize;
                                let old = self.pixels[at];
                                let mut color = 0;
                                for shift in [0, 8, 16] {
                                    color |= ((((d::TEXT >> shift) & 255) * u32::from(alpha)
                                        + ((old >> shift) & 255) * (255 - u32::from(alpha))
                                        + 127)
                                        / 255)
                                        << shift;
                                }
                                self.pixels[at] = color;
                            }
                        }
                    }
                }
            }
        }
    }
    fn label(&mut self, x: u32, y: u32, label: &str, color: u32, width: u32) {
        let step = character_width(self.scale);
        for (index, ch) in label.chars().take((width / step) as usize).enumerate() {
            self.glyph(x + index as u32 * step, y, ch, color);
        }
    }
}
