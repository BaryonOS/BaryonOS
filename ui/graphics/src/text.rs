//! Fixed Latin UI atlas. Glyph fallback never changes stored document bytes.
use crate::{atlas, Canvas, Rect};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Size {
    Small,
    Body,
    Title,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Weight {
    Regular,
    Semibold,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Font {
    size: Size,
    weight: Weight,
    double: bool,
    magnify: bool,
}
impl Font {
    pub const fn new(size: Size, weight: Weight, double: bool) -> Self {
        Self {
            size,
            weight,
            double,
            magnify: false,
        }
    }
    /// Fixed display face for clocks: the existing 64-pixel atlas at 100%,
    /// bounded 2x coverage expansion at 200%. No runtime font parser or asset.
    /// This is not an optically rerasterized font or general text shaping.
    pub const fn display(double: bool) -> Self {
        Self {
            size: Size::Title,
            weight: Weight::Semibold,
            double: true,
            magnify: double,
        }
    }
    const fn factor(self) -> u32 {
        if self.magnify {
            2
        } else {
            1
        }
    }
    fn index(self) -> usize {
        match (self.size, self.weight, self.double) {
            (Size::Small, _, false) => 0,
            (Size::Small, _, true) => 2,
            (Size::Body, Weight::Regular, false) => 1,
            (Size::Body, Weight::Regular, true) => 3,
            (Size::Body, Weight::Semibold, false) => 4,
            (Size::Body, Weight::Semibold, true) => 5,
            // Titles have one deliberate semibold design weight at both scales.
            (Size::Title, _, false) => 5,
            (Size::Title, _, true) => 6,
        }
    }
    pub fn height(self) -> u32 {
        u32::from(atlas::FONTS[self.index()].height) * self.factor()
    }
    pub fn em(self) -> u32 {
        u32::from(atlas::FONTS[self.index()].size) * self.factor()
    }
    /// Exact 26.6-pixel advance used by `Canvas::text`, including fallback.
    /// Consumers must accumulate before rounding; summing rounded character
    /// widths would make selection and caret drift from rendered text.
    pub fn advance_64(self, character: char) -> u32 {
        let first = usize::from(atlas::FONTS[self.index()].first);
        u32::from(atlas::GLYPHS[first + index(character)].advance) * self.factor()
    }
    pub fn measure(self, value: &str) -> u32 {
        let first = usize::from(atlas::FONTS[self.index()].first);
        let advance = value.chars().fold(0u64, |n, ch| {
            n.saturating_add(u64::from(atlas::GLYPHS[first + index(ch)].advance))
        });
        u32::try_from(
            advance
                .saturating_mul(u64::from(self.factor()))
                .div_ceil(64),
        )
        .unwrap_or(u32::MAX)
    }
}
fn index(ch: char) -> usize {
    atlas::CHARS
        .binary_search(&(ch as u32))
        .unwrap_or(atlas::GLYPH_COUNT - 1)
}
pub(crate) fn draw(
    canvas: &mut Canvas<'_>,
    x: i32,
    y: i32,
    value: &str,
    font: Font,
    color: u32,
) -> u32 {
    let first = usize::from(atlas::FONTS[font.index()].first);
    let factor = font.factor();
    let mut advance = 0u64;
    for ch in value.chars() {
        let glyph = atlas::GLYPHS[first + index(ch)];
        let gx = i64::from(x)
            + i64::try_from(advance.saturating_mul(u64::from(factor)) / 64).unwrap_or(i64::MAX / 2)
            + i64::from(glyph.left) * i64::from(factor);
        let gy = i64::from(y) + i64::from(glyph.top) * i64::from(factor);
        if let (Ok(gx), Ok(gy)) = (i32::try_from(gx), i32::try_from(gy)) {
            let r = Rect::new(
                gx,
                gy,
                u32::from(glyph.width) * factor,
                u32::from(glyph.height) * factor,
            );
            if r.intersects(canvas.bounds()) {
                for row in 0..usize::from(glyph.height) {
                    let py = gy.saturating_add(row as i32 * factor as i32);
                    if i64::from(py) + i64::from(factor) <= i64::from(canvas.bounds().y)
                        || i64::from(py)
                            >= i64::from(canvas.bounds().y) + i64::from(canvas.bounds().height)
                    {
                        continue;
                    }
                    let at = glyph.rows as usize + row;
                    let (Some(start), Some(end)) = (atlas::ROWS.get(at), atlas::ROWS.get(at + 1))
                    else {
                        continue;
                    };
                    let begin = glyph.offset as usize + usize::from(*start);
                    let finish = glyph.offset as usize + usize::from(*end);
                    let Some(encoded) = atlas::DATA.get(begin..finish) else {
                        continue;
                    };
                    let mut column = 0usize;
                    for byte in encoded {
                        let count = usize::from(byte >> 4) + 1;
                        let coverage = (byte & 15) * 17;
                        if column + count > usize::from(glyph.width) {
                            break;
                        }
                        if coverage != 0 {
                            for dx in 0..count {
                                for sy in 0..factor {
                                    for sx in 0..factor {
                                        canvas.blend_pixel(
                                            gx.saturating_add(
                                                ((column + dx) as u32 * factor + sx) as i32,
                                            ),
                                            py.saturating_add(sy as i32),
                                            color,
                                            coverage,
                                        );
                                    }
                                }
                            }
                        }
                        column += count;
                    }
                }
            }
        }
        advance = advance.saturating_add(u64::from(glyph.advance));
    }
    u32::try_from(advance.saturating_mul(u64::from(factor)).div_ceil(64)).unwrap_or(u32::MAX)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_static_rows_are_canonical_and_bounded() {
        assert_eq!(atlas::GLYPHS.len(), atlas::FONTS.len() * atlas::GLYPH_COUNT);
        for glyph in &atlas::GLYPHS {
            assert!(glyph.width <= 96 && glyph.height <= 96);
            for row in 0..usize::from(glyph.height) {
                let start =
                    usize::from(atlas::ROWS[glyph.rows as usize + row]) + glyph.offset as usize;
                let end =
                    usize::from(atlas::ROWS[glyph.rows as usize + row + 1]) + glyph.offset as usize;
                assert!(start <= end && end <= atlas::DATA.len());
                assert_eq!(
                    atlas::DATA[start..end]
                        .iter()
                        .map(|b| usize::from(b >> 4) + 1)
                        .sum::<usize>(),
                    usize::from(glyph.width)
                );
            }
        }
    }
    #[test]
    fn proportional_metrics_and_real_double_rasters() {
        let a = Font::new(Size::Body, Weight::Regular, false);
        let b = Font::new(Size::Body, Weight::Regular, true);
        assert!(a.measure("WWW") > a.measure("iii") * 2);
        assert!(b.measure("Materiaal") > a.measure("Materiaal"));
        assert_eq!(b.em(), 32);
        assert_eq!(a.measure(""), 0);
    }
    #[test]
    fn missing_characters_use_visible_fixed_replacement() {
        let f = Font::new(Size::Body, Weight::Regular, false);
        assert_eq!(f.measure("漢"), f.measure("\u{fffd}"));
    }
    #[test]
    fn display_scale_doubles_metrics_without_changing_regular_faces() {
        let a = Font::display(false);
        let b = Font::display(true);
        assert_eq!(a, Font::new(Size::Title, Weight::Semibold, true));
        assert_eq!(b.em(), 2 * a.em());
        assert_eq!(b.height(), 2 * a.height());
        assert!(b.measure("14:32").abs_diff(2 * a.measure("14:32")) <= 1);
        for glyph in &atlas::GLYPHS {
            assert!(u32::from(glyph.width) * b.factor() <= 192);
            assert!(u32::from(glyph.height) * b.factor() <= 192);
        }
        assert_eq!(Font::new(Size::Body, Weight::Regular, false).em(), 16);
        assert_eq!(Font::new(Size::Body, Weight::Regular, true).em(), 32);
    }
    #[test]
    fn display_text_clips_at_every_tile_edge_and_preserves_guards() {
        let mut full = [0u32; 192 * 192];
        let font = Font::display(true);
        let mut c = Canvas::new(Rect::new(0, 0, 192, 192), &mut full).unwrap();
        c.text(-13, -17, "8", font, 0xffffff);
        assert!(full.iter().any(|p| *p != 0));
        for (x, y) in [(0, 0), (8, 8), (24, 32), (64, 96), (128, 128)] {
            let mut tile = [0x125589; 258];
            let mut c = Canvas::new(Rect::new(x, y, 16, 16), &mut tile[1..257]).unwrap();
            c.clear(0);
            c.text(-13, -17, "8", font, 0xffffff);
            for row in 0..16usize {
                assert_eq!(
                    &tile[1 + row * 16..1 + (row + 1) * 16],
                    &full[(y as usize + row) * 192 + x as usize
                        ..(y as usize + row) * 192 + x as usize + 16]
                );
            }
            assert_eq!(tile[0], 0x125589);
            assert_eq!(tile[257], 0x125589);
        }
    }
}
