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
}
impl Font {
    pub const fn new(size: Size, weight: Weight, double: bool) -> Self {
        Self {
            size,
            weight,
            double,
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
        u32::from(atlas::FONTS[self.index()].height)
    }
    pub fn em(self) -> u32 {
        u32::from(atlas::FONTS[self.index()].size)
    }
    /// Immutable glyph advance in 26.6 fixed-point pixels. Summing these and
    /// flooring a prefix matches the exact placement used by the atlas drawer.
    pub fn advance(self, character: char) -> u32 {
        let first = usize::from(atlas::FONTS[self.index()].first);
        u32::from(atlas::GLYPHS[first + index(character)].advance)
    }
    pub fn measure(self, value: &str) -> u32 {
        let first = usize::from(atlas::FONTS[self.index()].first);
        let advance = value.chars().fold(0u64, |n, ch| {
            n.saturating_add(u64::from(atlas::GLYPHS[first + index(ch)].advance))
        });
        u32::try_from(advance.div_ceil(64)).unwrap_or(u32::MAX)
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
    let mut advance = 0u64;
    for ch in value.chars() {
        let glyph = atlas::GLYPHS[first + index(ch)];
        let gx = i64::from(x)
            + i64::try_from(advance / 64).unwrap_or(i64::MAX / 2)
            + i64::from(glyph.left);
        let gy = i64::from(y) + i64::from(glyph.top);
        if let (Ok(gx), Ok(gy)) = (i32::try_from(gx), i32::try_from(gy)) {
            let r = Rect::new(gx, gy, u32::from(glyph.width), u32::from(glyph.height));
            if r.intersects(canvas.bounds()) {
                for row in 0..usize::from(glyph.height) {
                    let py = gy.saturating_add(row as i32);
                    if i64::from(py) < i64::from(canvas.bounds().y)
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
                                canvas.blend_pixel(
                                    gx.saturating_add((column + dx) as i32),
                                    py,
                                    color,
                                    coverage,
                                );
                            }
                        }
                        column += count;
                    }
                }
            }
        }
        advance = advance.saturating_add(u64::from(glyph.advance));
    }
    u32::try_from(advance.div_ceil(64)).unwrap_or(u32::MAX)
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
}
