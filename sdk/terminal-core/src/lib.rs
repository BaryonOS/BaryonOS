#![no_std]
#![forbid(unsafe_code)]

//! Bounded, allocation-free primitives for a future native terminal app.
//!
//! This crate edits one UTF-8 command line and tokenizes a deliberately small
//! shell grammar. It does not launch processes, expand variables or globs,
//! interpret pipelines/redirections, or grant capabilities.

pub const MAX_COMMAND_BYTES: usize = 1024;
pub const MAX_ARGUMENTS: usize = 16;
pub const MAX_ARGUMENT_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LineFull,
    ControlCharacter,
    TooManyArguments,
    ArgumentTooLong,
    EmptyCommand,
    UnterminatedQuote,
    TrailingEscape,
    UnsupportedSyntax,
}

/// A fixed-capacity UTF-8 input line with a cursor positioned on a character
/// boundary. Mutations preserve UTF-8 validity and never allocate.
#[derive(Clone, PartialEq, Eq)]
pub struct CommandLine<const N: usize> {
    bytes: [u8; N],
    length: usize,
    cursor: usize,
}

impl<const N: usize> CommandLine<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            length: 0,
            cursor: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.length]).expect("CommandLine maintains valid UTF-8")
    }

    pub const fn cursor_byte(&self) -> usize {
        self.cursor
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn insert(&mut self, character: char) -> Result<(), Error> {
        if character.is_control() {
            return Err(Error::ControlCharacter);
        }
        let mut encoded = [0; 4];
        let encoded = character.encode_utf8(&mut encoded).as_bytes();
        let new_length = self
            .length
            .checked_add(encoded.len())
            .filter(|length| *length <= N)
            .ok_or(Error::LineFull)?;
        self.bytes
            .copy_within(self.cursor..self.length, self.cursor + encoded.len());
        self.bytes[self.cursor..self.cursor + encoded.len()].copy_from_slice(encoded);
        self.cursor += encoded.len();
        self.length = new_length;
        Ok(())
    }

    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        while self.bytes[self.cursor] & 0b1100_0000 == 0b1000_0000 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor == self.length {
            return;
        }
        self.cursor += 1;
        while self.cursor < self.length && self.bytes[self.cursor] & 0b1100_0000 == 0b1000_0000 {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.length;
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let end = self.cursor;
        self.move_left();
        self.remove_range(self.cursor, end);
        true
    }

    pub fn delete(&mut self) -> bool {
        if self.cursor == self.length {
            return false;
        }
        let start = self.cursor;
        self.move_right();
        self.remove_range(start, self.cursor);
        self.cursor = start;
        true
    }

    pub fn clear(&mut self) {
        self.bytes[..self.length].fill(0);
        self.length = 0;
        self.cursor = 0;
    }

    fn remove_range(&mut self, start: usize, end: usize) {
        self.bytes.copy_within(end..self.length, start);
        let new_length = self.length - (end - start);
        self.bytes[new_length..self.length].fill(0);
        self.length = new_length;
    }
}

impl<const N: usize> Default for CommandLine<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed argv stored in fixed-size slots; each returned argument borrows
/// the parser result and is valid UTF-8.
#[derive(Clone, PartialEq, Eq)]
pub struct ParsedCommand<const ARGS: usize, const BYTES: usize> {
    arguments: [[u8; BYTES]; ARGS],
    lengths: [usize; ARGS],
    count: usize,
}

pub type ShellCommand = ParsedCommand<MAX_ARGUMENTS, MAX_ARGUMENT_BYTES>;

impl<const ARGS: usize, const BYTES: usize> ParsedCommand<ARGS, BYTES> {
    pub const fn new() -> Self {
        Self {
            arguments: [[0; BYTES]; ARGS],
            lengths: [0; ARGS],
            count: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.count
    }

    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        let length = *self.lengths.get(index)?;
        if index >= self.count {
            return None;
        }
        core::str::from_utf8(&self.arguments[index][..length]).ok()
    }
}

impl<const ARGS: usize, const BYTES: usize> Default for ParsedCommand<ARGS, BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse whitespace-separated arguments with single/double quotes and
/// backslash escaping. Unquoted shell operators and expansion syntax are
/// rejected rather than being silently interpreted or passed through.
pub fn parse<const ARGS: usize, const BYTES: usize>(
    line: &str,
) -> Result<ParsedCommand<ARGS, BYTES>, Error> {
    if line.len() > MAX_COMMAND_BYTES {
        return Err(Error::LineFull);
    }
    let mut parsed = ParsedCommand::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;

    for character in line.chars() {
        if character.is_control() {
            return Err(Error::ControlCharacter);
        }
        if escaped {
            push(&mut parsed, character)?;
            escaped = false;
            started = true;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            started = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            } else {
                push(&mut parsed, character)?;
            }
            started = true;
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                started = true;
            }
            '|' | '&' | ';' | '<' | '>' | '$' | '`' => {
                return Err(Error::UnsupportedSyntax);
            }
            character if character.is_whitespace() => {
                if started {
                    finish_argument(&mut parsed)?;
                    started = false;
                }
            }
            _ => {
                push(&mut parsed, character)?;
                started = true;
            }
        }
    }

    if escaped {
        return Err(Error::TrailingEscape);
    }
    if quote.is_some() {
        return Err(Error::UnterminatedQuote);
    }
    if started {
        finish_argument(&mut parsed)?;
    }
    if parsed.count == 0 {
        return Err(Error::EmptyCommand);
    }
    Ok(parsed)
}

pub fn parse_command(line: &str) -> Result<ShellCommand, Error> {
    parse(line)
}

fn finish_argument<const ARGS: usize, const BYTES: usize>(
    parsed: &mut ParsedCommand<ARGS, BYTES>,
) -> Result<(), Error> {
    if parsed.count >= ARGS {
        return Err(Error::TooManyArguments);
    }
    parsed.count += 1;
    Ok(())
}

fn push<const ARGS: usize, const BYTES: usize>(
    parsed: &mut ParsedCommand<ARGS, BYTES>,
    character: char,
) -> Result<(), Error> {
    let index = parsed.count;
    if index >= ARGS {
        return Err(Error::TooManyArguments);
    }
    let current = parsed.lengths[index];
    let mut encoded = [0; 4];
    let encoded = character.encode_utf8(&mut encoded).as_bytes();
    let end = current
        .checked_add(encoded.len())
        .filter(|length| *length <= BYTES)
        .ok_or(Error::ArgumentTooLong)?;
    parsed.arguments[index][current..end].copy_from_slice(encoded);
    parsed.lengths[index] = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    type Args = ParsedCommand<6, 16>;

    #[test]
    fn line_editor_moves_and_deletes_whole_utf8_characters() {
        let mut line = CommandLine::<16>::new();
        line.insert('a').unwrap();
        line.insert('é').unwrap();
        line.insert('🙂').unwrap();
        line.move_left();
        assert_eq!(line.cursor_byte(), 3);
        line.backspace();
        assert_eq!(line.as_str(), "a🙂");
        line.move_home();
        line.delete();
        assert_eq!(line.as_str(), "🙂");
        assert_eq!(line.cursor_byte(), 0);
    }

    #[test]
    fn line_editor_rejects_controls_and_keeps_state_on_overflow() {
        let mut line = CommandLine::<2>::new();
        line.insert('é').unwrap();
        assert_eq!(line.insert('a'), Err(Error::LineFull));
        assert_eq!(line.insert('\n'), Err(Error::ControlCharacter));
        assert_eq!(line.as_str(), "é");
        line.clear();
        assert!(line.is_empty());
        assert_eq!(line.cursor_byte(), 0);
    }

    #[test]
    fn parser_handles_quotes_escapes_and_empty_arguments() {
        let parsed: Args = parse(r#"open "two words" 'literal $value' "" escaped\ space"#).unwrap();
        assert_eq!(parsed.len(), 5);
        assert_eq!(parsed.get(0), Some("open"));
        assert_eq!(parsed.get(1), Some("two words"));
        assert_eq!(parsed.get(2), Some("literal $value"));
        assert_eq!(parsed.get(3), Some(""));
        assert_eq!(parsed.get(4), Some("escaped space"));
        assert_eq!(parsed.get(5), None);
    }

    #[test]
    fn parser_rejects_operators_expansion_and_malformed_quotes() {
        for line in [
            "run | next",
            "run > file",
            "run && next",
            "run $HOME",
            "run `x`",
        ] {
            assert!(matches!(
                parse::<4, 16>(line),
                Err(Error::UnsupportedSyntax)
            ));
        }
        assert!(matches!(
            parse::<4, 16>("run 'unfinished"),
            Err(Error::UnterminatedQuote)
        ));
        assert!(matches!(
            parse::<4, 16>("run trailing\\"),
            Err(Error::TrailingEscape)
        ));
    }

    #[test]
    fn parser_enforces_argument_and_buffer_bounds() {
        assert!(matches!(
            parse::<1, 16>("one two"),
            Err(Error::TooManyArguments)
        ));
        assert!(matches!(
            parse::<4, 3>("tool four"),
            Err(Error::ArgumentTooLong)
        ));
        assert!(matches!(parse::<4, 16>("   "), Err(Error::EmptyCommand)));
        assert!(matches!(
            parse::<4, 16>("a\tb"),
            Err(Error::ControlCharacter)
        ));
        assert!(matches!(parse::<0, 16>("''"), Err(Error::TooManyArguments)));
        let too_long = "x".repeat(MAX_COMMAND_BYTES + 1);
        assert!(matches!(parse_command(&too_long), Err(Error::LineFull)));
    }

    #[test]
    fn quoted_operators_are_literal_arguments() {
        let parsed: Args = parse(r#"echo "a|b" 'c>d'"#).unwrap();
        assert_eq!(parsed.get(1), Some("a|b"));
        assert_eq!(parsed.get(2), Some("c>d"));
    }
}
