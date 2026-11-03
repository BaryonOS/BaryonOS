//! App-local UTF-8 CSV editing. Cells are literal text, never executable formulas.
//! No storage, network, process or capability authority is provided here.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub text_bytes: usize,
    pub field_bytes: usize,
    pub rows: usize,
    pub columns: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: 4 * 1024 * 1024,
            output_bytes: 8 * 1024 * 1024,
            text_bytes: 4 * 1024 * 1024,
            field_bytes: 64 * 1024,
            rows: 4096,
            columns: 256,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    InvalidUtf8,
    InvalidText,
    InputLimit,
    OutputLimit,
    TextLimit,
    FieldLimit,
    RowLimit,
    ColumnLimit,
    RaggedRow,
    UnexpectedQuote,
    UnterminatedQuote,
    TrailingQuotedText,
    BareCarriageReturn,
    CellOutOfBounds,
    OutOfMemory,
}

fn reserve<T>(vector: &mut Vec<T>, additional: usize) -> Result<(), Error> {
    vector
        .try_reserve(additional)
        .map_err(|_| Error::OutOfMemory)
}

fn text(value: &str, limit: usize) -> Result<String, Error> {
    if value.contains('\0') {
        return Err(Error::InvalidText);
    }
    if value.len() > limit {
        return Err(Error::FieldLimit);
    }
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| Error::OutOfMemory)?;
    result.push_str(value);
    Ok(result)
}

#[derive(Debug)]
pub struct Table {
    rows: Vec<Vec<String>>,
    columns: usize,
    text_bytes: usize,
    limits: Limits,
}

impl Table {
    /// Accept comma-separated UTF-8, optional UTF-8 BOM, CRLF or LF records,
    /// doubled quotes and quoted newlines. Every row must have the same width.
    /// An empty input is a zero-row table; a blank record is one empty cell.
    pub fn import(input: &[u8], limits: Limits) -> Result<Self, Error> {
        let maximum = Limits::default();
        if limits.input_bytes > maximum.input_bytes
            || limits.output_bytes > maximum.output_bytes
            || limits.text_bytes > maximum.text_bytes
            || limits.field_bytes > maximum.field_bytes
            || limits.rows == 0
            || limits.rows > maximum.rows
            || limits.columns == 0
            || limits.columns > maximum.columns
        {
            return Err(Error::InvalidLimits);
        }
        if input.len() > limits.input_bytes {
            return Err(Error::InputLimit);
        }
        let input = input.strip_prefix(b"\xef\xbb\xbf").unwrap_or(input);
        core::str::from_utf8(input).map_err(|_| Error::InvalidUtf8)?;
        if input.contains(&0) {
            return Err(Error::InvalidText);
        }
        let mut table = Self {
            rows: Vec::new(),
            columns: 0,
            text_bytes: 0,
            limits,
        };
        let mut cursor = 0;
        while cursor < input.len() {
            if table.rows.len() == limits.rows {
                return Err(Error::RowLimit);
            }
            let mut row = Vec::new();
            loop {
                if row.len() == limits.columns {
                    return Err(Error::ColumnLimit);
                }
                let field = parse_field(input, &mut cursor, limits.field_bytes)?;
                table.text_bytes = table
                    .text_bytes
                    .checked_add(field.len())
                    .ok_or(Error::TextLimit)?;
                if table.text_bytes > limits.text_bytes {
                    return Err(Error::TextLimit);
                }
                reserve(&mut row, 1)?;
                row.push(field);
                if input.get(cursor) == Some(&b',') {
                    cursor += 1;
                } else {
                    break;
                }
            }
            if table.rows.is_empty() {
                table.columns = row.len();
            } else if row.len() != table.columns {
                return Err(Error::RaggedRow);
            }
            reserve(&mut table.rows, 1)?;
            table.rows.push(row);
            match input.get(cursor) {
                None => {}
                Some(b'\n') => cursor += 1,
                Some(b'\r') if input.get(cursor + 1) == Some(&b'\n') => cursor += 2,
                Some(b'\r') => return Err(Error::BareCarriageReturn),
                _ => return Err(Error::TrailingQuotedText),
            }
        }
        Ok(table)
    }

    pub fn rows(&self) -> usize {
        self.rows.len()
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn cell(&self, row: usize, column: usize) -> Option<&str> {
        self.rows.get(row)?.get(column).map(String::as_str)
    }

    /// Failed edits preserve the old cell, text budget and exported contents.
    pub fn set_cell(&mut self, row: usize, column: usize, value: &str) -> Result<(), Error> {
        let old = self.cell(row, column).ok_or(Error::CellOutOfBounds)?;
        let next_bytes = self
            .text_bytes
            .checked_sub(old.len())
            .and_then(|bytes| bytes.checked_add(value.len()))
            .filter(|bytes| *bytes <= self.limits.text_bytes)
            .ok_or(Error::TextLimit)?;
        let replacement = text(value, self.limits.field_bytes)?;
        self.rows[row][column] = replacement;
        self.text_bytes = next_bytes;
        Ok(())
    }

    /// Canonical comma CSV with CRLF records and no BOM. Values including
    /// formula-like prefixes remain literal and unchanged, not spreadsheet-safe.
    /// Size and allocation are checked before any caller-visible output exists.
    pub fn export(&self) -> Result<Vec<u8>, Error> {
        let mut size = 0usize;
        for row in &self.rows {
            for (index, field) in row.iter().enumerate() {
                let quoted = needs_quotes(field);
                let escapes = if quoted {
                    field.bytes().filter(|byte| *byte == b'"').count()
                } else {
                    0
                };
                size = size
                    .checked_add(usize::from(index != 0))
                    .and_then(|n| n.checked_add(field.len()))
                    .and_then(|n| n.checked_add(escapes))
                    .and_then(|n| n.checked_add(if quoted { 2 } else { 0 }))
                    .ok_or(Error::OutputLimit)?;
            }
            size = size.checked_add(2).ok_or(Error::OutputLimit)?;
        }
        if size > self.limits.output_bytes {
            return Err(Error::OutputLimit);
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(size)
            .map_err(|_| Error::OutOfMemory)?;
        for row in &self.rows {
            for (index, field) in row.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                if needs_quotes(field) {
                    output.push(b'"');
                    for byte in field.bytes() {
                        output.push(byte);
                        if byte == b'"' {
                            output.push(byte);
                        }
                    }
                    output.push(b'"');
                } else {
                    output.extend_from_slice(field.as_bytes());
                }
            }
            output.extend_from_slice(b"\r\n");
        }
        Ok(output)
    }
}

fn needs_quotes(field: &str) -> bool {
    field.is_empty()
        || field
            .bytes()
            .any(|byte| matches!(byte, b',' | b'"' | b'\r' | b'\n'))
}

fn parse_field(input: &[u8], cursor: &mut usize, limit: usize) -> Result<String, Error> {
    let mut bytes = Vec::new();
    let quoted = input.get(*cursor) == Some(&b'"');
    if quoted {
        *cursor += 1;
    }
    loop {
        match input.get(*cursor).copied() {
            None if quoted => return Err(Error::UnterminatedQuote),
            None => break,
            Some(b'"') if quoted => {
                *cursor += 1;
                if input.get(*cursor) != Some(&b'"') {
                    if input
                        .get(*cursor)
                        .is_some_and(|byte| !matches!(byte, b',' | b'\r' | b'\n'))
                    {
                        return Err(Error::TrailingQuotedText);
                    }
                    break;
                }
            }
            Some(b'"') => return Err(Error::UnexpectedQuote),
            Some(b',' | b'\r' | b'\n') if !quoted => break,
            _ => {}
        }
        if bytes.len() == limit {
            return Err(Error::FieldLimit);
        }
        reserve(&mut bytes, 1)?;
        bytes.push(input[*cursor]);
        *cursor += 1;
    }
    String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
}
