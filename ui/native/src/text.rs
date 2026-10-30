//! UTF-8 byte boundaries and bounded monospace visual lines. No normalization
//! or lossy replacement is applied to document bytes. This is not grapheme
//! shaping, an input method, or full Unicode font coverage.
use core::ops::Range;

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
