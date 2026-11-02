//! UTF-8 byte boundaries and bounded visual lines. No normalization
//! or lossy replacement is applied to document bytes. This is not grapheme
//! shaping, an input method, or full Unicode font coverage.
use core::ops::Range;
use nieuw_os_graphics::Font;

/// One shared layout for rendering, caret, pointer hits and navigation. Values
/// are exact 26.6 physical pixels from the renderer's fixed atlas. This is a
/// bounded Latin/fallback layout, not grapheme shaping or a Unicode input method.
#[derive(Clone, Copy, Debug)]
pub struct Proportional {
    font: Font,
    width_64: u64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VisualPosition {
    pub row: usize,
    pub x_64: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct GlyphCell {
    pub character: char,
    pub start: usize,
    pub end: usize,
    pub position: VisualPosition,
    pub advance_64: u64,
}
pub struct Glyphs<'a> {
    characters: core::iter::Peekable<core::str::CharIndices<'a>>,
    metrics: Proportional,
    position: VisualPosition,
}
impl Proportional {
    pub fn new(font: Font, physical_width: u32) -> Self {
        Self {
            font,
            width_64: u64::from(physical_width.max(1)) * 64,
        }
    }
    pub fn cells(self, text: &str) -> Glyphs<'_> {
        Glyphs {
            characters: text.char_indices().peekable(),
            metrics: self,
            position: VisualPosition::default(),
        }
    }
    pub fn position(self, text: &str, byte: usize) -> VisualPosition {
        let byte = boundary(text, byte);
        let mut cells = self.cells(text);
        for cell in cells.by_ref() {
            if cell.start >= byte {
                return cell.position;
            }
        }
        cells.position
    }
    /// A soft wrap has two visual sides of the same UTF-8 boundary. Keep
    /// End/pointer affinity without adding or changing a document character.
    pub fn position_with_affinity(self, text: &str, byte: usize, trailing: bool) -> VisualPosition {
        if trailing {
            let byte = boundary(text, byte);
            let mut previous: Option<GlyphCell> = None;
            for cell in self.cells(text) {
                if cell.start >= byte {
                    if let Some(before) = previous {
                        if before.end == byte
                            && before.position.row < cell.position.row
                            && !matches!(before.character, '\n' | '\r')
                        {
                            return VisualPosition {
                                row: before.position.row,
                                x_64: before.position.x_64 + before.advance_64,
                            };
                        }
                    }
                    return cell.position;
                }
                previous = Some(cell);
            }
        }
        self.position(text, byte)
    }
    /// Nearest insertion boundary on a visual row, clamped before a hard
    /// newline or to that row's end. Every result is an original UTF-8 boundary.
    pub fn byte_at(self, text: &str, target: VisualPosition) -> usize {
        let mut end = 0;
        for cell in self.cells(text) {
            if cell.position.row > target.row {
                return cell.start;
            }
            end = cell.end;
            if cell.position.row == target.row {
                if matches!(cell.character, '\n' | '\r') {
                    return cell.start;
                }
                if target.x_64 < cell.position.x_64 + cell.advance_64.div_ceil(2) {
                    return cell.start;
                }
            }
        }
        end
    }
    pub fn line_boundary(self, text: &str, byte: usize, end: bool) -> usize {
        let p = self.position(text, byte);
        self.byte_at(
            text,
            VisualPosition {
                row: p.row,
                x_64: if end { u64::MAX } else { 0 },
            },
        )
    }
}
impl Iterator for Glyphs<'_> {
    type Item = GlyphCell;
    fn next(&mut self) -> Option<Self::Item> {
        let (start, character) = self.characters.next()?;
        let tab = u64::from(self.metrics.font.advance_64(' ')).max(1) * 4;
        let width = |x| match character {
            '\n' | '\r' => 0,
            '\t' => tab - x % tab,
            _ => u64::from(self.metrics.font.advance_64(character)),
        };
        let mut advance_64 = width(self.position.x_64);
        if self.position.x_64 > 0
            && self.position.x_64.saturating_add(advance_64) > self.metrics.width_64
        {
            self.position.row += 1;
            self.position.x_64 = 0;
            advance_64 = width(0);
        }
        let cell = GlyphCell {
            character,
            start,
            end: start + character.len_utf8(),
            position: self.position,
            advance_64,
        };
        if character == '\n'
            || (character == '\r' && self.characters.peek().is_none_or(|(_, next)| *next != '\n'))
        {
            self.position.row += 1;
            self.position.x_64 = 0;
        } else {
            self.position.x_64 += advance_64;
        }
        Some(cell)
    }
}

pub fn boundary(text: &str, position: usize) -> usize {
    let mut position = position.min(text.len());
    while !text.is_char_boundary(position) {
        position -= 1;
    }
    position
}
pub fn previous(text: &str, position: usize) -> usize {
    text[..boundary(text, position)]
        .char_indices()
        .next_back()
        .map_or(0, |(at, _)| at)
}
pub fn next(text: &str, position: usize) -> usize {
    let position = boundary(text, position);
    text[position..]
        .chars()
        .next()
        .map_or(position, |ch| position + ch.len_utf8())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub character: char,
    pub start: usize,
    pub end: usize,
    pub position: Position,
    pub width: usize,
}

pub struct Cells<'a> {
    characters: core::iter::Peekable<core::str::CharIndices<'a>>,
    position: Position,
    columns: usize,
}
impl<'a> Cells<'a> {
    pub fn new(text: &'a str, columns: usize) -> Self {
        Self {
            characters: text.char_indices().peekable(),
            position: Position::default(),
            columns: columns.max(4),
        }
    }
    pub fn end(&self) -> Position {
        self.position
    }
}
impl Iterator for Cells<'_> {
    type Item = Cell;
    fn next(&mut self) -> Option<Cell> {
        let (start, character) = self.characters.next()?;
        let width = match character {
            '\n' | '\r' => 0,
            '\t' => 4 - self.position.column % 4,
            _ => 1,
        };
        if character != '\n' && character != '\r' && self.position.column + width > self.columns {
            self.position.row += 1;
            self.position.column = 0;
        }
        let width = if character == '\t' {
            4 - self.position.column % 4
        } else {
            width
        };
        let cell = Cell {
            character,
            start,
            end: start + character.len_utf8(),
            position: self.position,
            width,
        };
        if character == '\n'
            || (character == '\r' && self.characters.peek().is_none_or(|(_, next)| *next != '\n'))
        {
            self.position.row += 1;
            self.position.column = 0;
        } else {
            self.position.column += width;
        }
        Some(cell)
    }
}

pub fn position(text: &str, byte: usize, columns: usize) -> Position {
    let mut cells = Cells::new(text, columns);
    for cell in cells.by_ref() {
        if cell.start >= byte {
            return cell.position;
        }
    }
    cells.end()
}

pub fn byte_at(text: &str, target: Position, columns: usize) -> usize {
    let cells = Cells::new(text, columns);
    for cell in cells {
        if cell.position.row > target.row
            || (cell.position.row == target.row
                && cell.position.column + cell.width > target.column)
        {
            return cell.start;
        }
        if cell.character == '\n' && cell.position.row >= target.row {
            return cell.start;
        }
    }
    text.len()
}

pub fn selection(anchor: usize, caret: usize) -> Range<usize> {
    anchor.min(caret)..anchor.max(caret)
}
