//! A bounded plain-text DOCX export, not an Office parser or native file portal.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::vec::Vec;

pub const MAX_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PARAGRAPHS: usize = 4096;

const TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
const RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
const START: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#;
const END: &[u8] = b"</w:body></w:document>";
const PARAGRAPH: &[u8] = br#"<w:p><w:r><w:t xml:space="preserve">"#;
const CLOSE: &[u8] = b"</w:t></w:r></w:p>";
const TAB: &[u8] = br#"</w:t><w:tab/><w:t xml:space="preserve">"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InputLimit,
    InvalidUtf8,
    UnsupportedCharacter { byte: usize },
    ParagraphLimit,
    OutputLimit,
    OutOfMemory,
}

fn document_size(text: &str) -> Result<usize, Error> {
    let paragraphs = 1 + text.bytes().filter(|byte| *byte == b'\n').count();
    if paragraphs > MAX_PARAGRAPHS {
        return Err(Error::ParagraphLimit);
    }
    for (byte, ch) in text.char_indices() {
        if !matches!(ch, '\t' | '\n' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
        {
            return Err(Error::UnsupportedCharacter { byte });
        }
    }
    let escaped = text
        .bytes()
        .map(|byte| match byte {
            b'\n' => 0,
            b'\t' => TAB.len(),
            b'&' => 5,
            b'<' | b'>' => 4,
            _ => 1,
        })
        .sum::<usize>();
    Ok(START.len() + END.len() + paragraphs * (PARAGRAPH.len() + CLOSE.len()) + escaped)
}

/// One LF becomes one paragraph boundary, including empty/trailing paragraphs.
/// Tabs become w:tab. Whitespace, Unicode and literal markup remain text.
/// Fixed part paths only; no macros, links, fields, comments or author metadata.
pub fn export(input: &[u8], output_limit: usize) -> Result<Vec<u8>, Error> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(Error::InputLimit);
    }
    let text = core::str::from_utf8(input).map_err(|_| Error::InvalidUtf8)?;
    let document_bytes = document_size(text)?;
    let names = ["[Content_Types].xml", "_rels/.rels", "word/document.xml"];
    let total = 22
        + names
            .iter()
            .map(|name| 30 + 46 + name.len() * 2)
            .sum::<usize>()
        + TYPES.len()
        + RELS.len()
        + document_bytes;
    if total > output_limit.min(MAX_OUTPUT_BYTES) {
        return Err(Error::OutputLimit);
    }
    let mut document = Vec::new();
    document
        .try_reserve_exact(document_bytes)
        .map_err(|_| Error::OutOfMemory)?;
    document.extend_from_slice(START);
    for line in text.split('\n') {
        document.extend_from_slice(PARAGRAPH);
        for &byte in line.as_bytes() {
            match byte {
                b'\t' => document.extend_from_slice(TAB),
                b'&' => document.extend_from_slice(b"&amp;"),
                b'<' => document.extend_from_slice(b"&lt;"),
                b'>' => document.extend_from_slice(b"&gt;"),
                _ => document.push(byte),
            }
        }
        document.extend_from_slice(CLOSE);
    }
    document.extend_from_slice(END);
    debug_assert_eq!(document.len(), document_bytes);
    let parts: [&[u8]; 3] = [TYPES, RELS, &document];
    let mut output = Vec::new();
    output
        .try_reserve_exact(total)
        .map_err(|_| Error::OutOfMemory)?;
    let mut offsets = [0u32; 3];
    let checksums = parts.map(crc32);
    for index in 0..3 {
        offsets[index] = output.len() as u32;
        push32(&mut output, 0x04034b50);
        push16(&mut output, 20);
        push16(&mut output, 0);
        push16(&mut output, 0); // Stored: no compression or decompression engine.
        push16(&mut output, 0);
        push16(&mut output, 0x21); // 1980-01-01, deterministic; not a user timestamp.
        push32(&mut output, checksums[index]);
        push32(&mut output, parts[index].len() as u32);
        push32(&mut output, parts[index].len() as u32);
        push16(&mut output, names[index].len() as u16);
        push16(&mut output, 0);
        output.extend_from_slice(names[index].as_bytes());
        output.extend_from_slice(parts[index]);
    }
    let directory = output.len() as u32;
    for index in 0..3 {
        push32(&mut output, 0x02014b50);
        push16(&mut output, 20);
        push16(&mut output, 20);
        push16(&mut output, 0);
        push16(&mut output, 0);
        push16(&mut output, 0);
        push16(&mut output, 0x21);
        push32(&mut output, checksums[index]);
        push32(&mut output, parts[index].len() as u32);
        push32(&mut output, parts[index].len() as u32);
        push16(&mut output, names[index].len() as u16);
        push16(&mut output, 0);
        push16(&mut output, 0);
        push16(&mut output, 0);
        push16(&mut output, 0);
        push32(&mut output, 0);
        push32(&mut output, offsets[index]);
        output.extend_from_slice(names[index].as_bytes());
    }
    let directory_bytes = output.len() as u32 - directory;
    push32(&mut output, 0x06054b50);
    push16(&mut output, 0);
    push16(&mut output, 0);
    push16(&mut output, 3);
    push16(&mut output, 3);
    push32(&mut output, directory_bytes);
    push32(&mut output, directory);
    push16(&mut output, 0);
    debug_assert_eq!(output.len(), total);
    Ok(output)
}

fn push16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn push32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

// ZIP's standard CRC-32/ISO-HDLC detects accidental corruption, not authenticity.
fn crc32(input: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in input {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    #[test]
    fn zip_crc_matches_standard_check_value() {
        assert_eq!(super::crc32(b"123456789"), 0xcbf43926);
        assert_eq!(super::crc32(b""), 0);
    }
}
