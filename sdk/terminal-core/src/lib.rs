#![no_std]
#![forbid(unsafe_code)]

//! Bounded, allocation-free primitives for a future native terminal app.
//!
//! This crate edits one UTF-8 command line and tokenizes a deliberately small
//! shell grammar and validates a Message320 stdin/stdout protocol. It does not
//! launch processes, expand variables or globs, interpret pipelines/
//! redirections, or grant capabilities.

pub const MAX_COMMAND_BYTES: usize = 1024;
pub const MAX_ARGUMENTS: usize = 16;
pub const MAX_ARGUMENT_BYTES: usize = 256;
pub const IPC_TAG: u64 = 0x5445_524d_494f_3031;
pub const FRAME_PAYLOAD_BYTES: usize = 256;
pub const FRAME_HEADER_BYTES: usize = 28;
pub const MAX_FRAME_DATA: usize = FRAME_PAYLOAD_BYTES - FRAME_HEADER_BYTES;
const FRAME_MAGIC: [u8; 4] = *b"NOST";

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
    InvalidPeer,
    InvalidJobId,
    InvalidFrame,
    WrongPeer,
    WrongJob,
    UnexpectedMessage,
    SequenceMismatch,
    SequenceExhausted,
    StreamClosed,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Stdin = 1,
    StdinEof = 2,
    Stdout = 3,
    Stderr = 4,
    OutputEof = 5,
}

/// One canonical terminal-I/O payload. Job IDs scope messages within the
/// broker protocol; they are labels, not capabilities or process identities.
pub struct Frame {
    bytes: [u8; FRAME_PAYLOAD_BYTES],
    length: usize,
}

impl Frame {
    pub fn data(job_id: u64, sequence: u64, channel: Channel, data: &[u8]) -> Result<Self, Error> {
        let kind = match channel {
            Channel::Stdin => Kind::Stdin,
            Channel::Stdout => Kind::Stdout,
            Channel::Stderr => Kind::Stderr,
        };
        if data.is_empty() || data.len() > MAX_FRAME_DATA {
            return Err(Error::InvalidFrame);
        }
        Self::new(job_id, sequence, kind, data)
    }

    pub fn end_input(job_id: u64, sequence: u64) -> Result<Self, Error> {
        Self::new(job_id, sequence, Kind::StdinEof, &[])
    }

    pub fn end_output(job_id: u64, sequence: u64) -> Result<Self, Error> {
        Self::new(job_id, sequence, Kind::OutputEof, &[])
    }

    pub fn payload(&self) -> &[u8] {
        &self.bytes[..self.length]
    }

    fn new(job_id: u64, sequence: u64, kind: Kind, data: &[u8]) -> Result<Self, Error> {
        if job_id == 0 {
            return Err(Error::InvalidJobId);
        }
        let length = FRAME_HEADER_BYTES + data.len();
        let mut bytes = [0; FRAME_PAYLOAD_BYTES];
        bytes[..4].copy_from_slice(&FRAME_MAGIC);
        bytes[4] = 1;
        bytes[5] = kind as u8;
        bytes[8..16].copy_from_slice(&job_id.to_le_bytes());
        bytes[16..24].copy_from_slice(&sequence.to_le_bytes());
        bytes[24..26].copy_from_slice(&(data.len() as u16).to_le_bytes());
        bytes[FRAME_HEADER_BYTES..length].copy_from_slice(data);
        Ok(Self { bytes, length })
    }
}

/// A checked view into a received Message320 payload. The payload remains
/// borrowed from the caller's validated receive buffer.
pub struct DecodedFrame<'a> {
    pub job_id: u64,
    pub sequence: u64,
    kind: Kind,
    data: &'a [u8],
}

impl<'a> DecodedFrame<'a> {
    pub fn channel(&self) -> Option<Channel> {
        match self.kind {
            Kind::Stdin | Kind::Stdout => Some(if self.kind == Kind::Stdin {
                Channel::Stdin
            } else {
                Channel::Stdout
            }),
            Kind::Stderr => Some(Channel::Stderr),
            Kind::StdinEof | Kind::OutputEof => None,
        }
    }

    pub const fn data(&self) -> &'a [u8] {
        self.data
    }

    pub const fn is_end_of_input(&self) -> bool {
        matches!(self.kind, Kind::StdinEof)
    }

    pub const fn is_end_of_output(&self) -> bool {
        matches!(self.kind, Kind::OutputEof)
    }
}

/// Accepts frames from one kernel-identified peer for one job and one stream
/// direction. Call `accept` with the sender PID and payload from native IPC,
/// not values copied from the payload itself. If a rejected received message
/// carries capabilities, the caller must close them before discarding it.
pub struct Receiver {
    peer: u64,
    job_id: u64,
    direction: Direction,
    next_sequence: u64,
    closed: bool,
}

impl Receiver {
    pub fn new(peer: u64, job_id: u64, direction: Direction) -> Result<Self, Error> {
        if peer == 0 {
            return Err(Error::InvalidPeer);
        }
        if job_id == 0 {
            return Err(Error::InvalidJobId);
        }
        Ok(Self {
            peer,
            job_id,
            direction,
            next_sequence: 0,
            closed: false,
        })
    }

    pub fn accept<'a>(
        &mut self,
        sender: u64,
        tag: u64,
        cap_count: usize,
        payload: &'a [u8],
    ) -> Result<DecodedFrame<'a>, Error> {
        if self.closed {
            return Err(Error::StreamClosed);
        }
        if sender != self.peer {
            return Err(Error::WrongPeer);
        }
        if tag != IPC_TAG || cap_count != 0 {
            return Err(Error::UnexpectedMessage);
        }
        let frame = decode(payload)?;
        if frame.job_id != self.job_id {
            return Err(Error::WrongJob);
        }
        if frame.sequence != self.next_sequence {
            return Err(Error::SequenceMismatch);
        }
        if frame.sequence == u64::MAX {
            return Err(Error::SequenceExhausted);
        }
        match (self.direction, frame.kind) {
            (Direction::Input, Kind::Stdin | Kind::StdinEof)
            | (Direction::Output, Kind::Stdout | Kind::Stderr | Kind::OutputEof) => {}
            _ => return Err(Error::UnexpectedMessage),
        }
        self.next_sequence = frame.sequence + 1;
        if matches!(frame.kind, Kind::StdinEof | Kind::OutputEof) {
            self.closed = true;
        }
        Ok(frame)
    }
}

fn decode(payload: &[u8]) -> Result<DecodedFrame<'_>, Error> {
    if payload.len() < FRAME_HEADER_BYTES
        || payload[..4] != FRAME_MAGIC
        || payload[4] != 1
        || payload[6..8].iter().any(|byte| *byte != 0)
        || payload[26..28].iter().any(|byte| *byte != 0)
    {
        return Err(Error::InvalidFrame);
    }
    let kind = match payload[5] {
        1 => Kind::Stdin,
        2 => Kind::StdinEof,
        3 => Kind::Stdout,
        4 => Kind::Stderr,
        5 => Kind::OutputEof,
        _ => return Err(Error::InvalidFrame),
    };
    let job_id = read_u64(payload, 8);
    let sequence = read_u64(payload, 16);
    let data_length = u16::from_le_bytes([payload[24], payload[25]]) as usize;
    if job_id == 0
        || data_length > MAX_FRAME_DATA
        || payload.len() != FRAME_HEADER_BYTES + data_length
    {
        return Err(Error::InvalidFrame);
    }
    match kind {
        Kind::Stdin | Kind::Stdout | Kind::Stderr if data_length == 0 => {
            return Err(Error::InvalidFrame);
        }
        Kind::StdinEof | Kind::OutputEof if data_length != 0 => {
            return Err(Error::InvalidFrame);
        }
        _ => {}
    }
    Ok(DecodedFrame {
        job_id,
        sequence,
        kind,
        data: &payload[FRAME_HEADER_BYTES..],
    })
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("validated terminal frame header"),
    )
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

    #[test]
    fn io_frame_round_trips_data_and_enforces_exact_lengths() {
        let frame = Frame::data(7, 0, Channel::Stdout, b"ready\n").unwrap();
        let decoded = decode(frame.payload()).unwrap();
        assert_eq!(decoded.job_id, 7);
        assert_eq!(decoded.sequence, 0);
        assert_eq!(decoded.channel(), Some(Channel::Stdout));
        assert_eq!(decoded.data(), b"ready\n");
        assert!(Frame::data(7, 0, Channel::Stdin, &[]).is_err());
        assert!(Frame::data(7, 0, Channel::Stdin, &[1; MAX_FRAME_DATA + 1]).is_err());
        assert!(matches!(
            Frame::data(0, 0, Channel::Stdin, b"x"),
            Err(Error::InvalidJobId)
        ));

        let mut malformed = frame.payload().to_vec();
        malformed.push(0);
        assert_eq!(decode(&malformed).err(), Some(Error::InvalidFrame));
    }

    #[test]
    fn io_receiver_binds_peer_job_tag_direction_and_sequence() {
        let frame = Frame::data(11, 0, Channel::Stdin, b"help\n").unwrap();
        let mut receiver = Receiver::new(22, 11, Direction::Input).unwrap();
        assert_eq!(
            receiver.accept(21, IPC_TAG, 0, frame.payload()).err(),
            Some(Error::WrongPeer)
        );
        assert_eq!(
            receiver.accept(22, IPC_TAG + 1, 0, frame.payload()).err(),
            Some(Error::UnexpectedMessage)
        );
        assert_eq!(
            receiver.accept(22, IPC_TAG, 1, frame.payload()).err(),
            Some(Error::UnexpectedMessage)
        );
        let accepted = receiver.accept(22, IPC_TAG, 0, frame.payload()).unwrap();
        assert_eq!(accepted.data(), b"help\n");
        assert_eq!(
            receiver.accept(22, IPC_TAG, 0, frame.payload()).err(),
            Some(Error::SequenceMismatch)
        );
        let wrong_job = Frame::data(12, 1, Channel::Stdin, b"help\n").unwrap();
        assert_eq!(
            receiver.accept(22, IPC_TAG, 0, wrong_job.payload()).err(),
            Some(Error::WrongJob)
        );
    }

    #[test]
    fn io_end_markers_close_only_the_bound_stream() {
        let wrong_direction = Frame::data(3, 0, Channel::Stdout, b"x").unwrap();
        let mut input = Receiver::new(4, 3, Direction::Input).unwrap();
        assert_eq!(
            input.accept(4, IPC_TAG, 0, wrong_direction.payload()).err(),
            Some(Error::UnexpectedMessage)
        );
        let eof = Frame::end_input(3, 0).unwrap();
        let accepted = input.accept(4, IPC_TAG, 0, eof.payload()).unwrap();
        assert!(accepted.is_end_of_input());
        assert_eq!(
            input.accept(4, IPC_TAG, 0, eof.payload()).err(),
            Some(Error::StreamClosed)
        );

        let output_eof = Frame::end_output(3, 0).unwrap();
        let mut output = Receiver::new(4, 3, Direction::Output).unwrap();
        let accepted = output.accept(4, IPC_TAG, 0, output_eof.payload()).unwrap();
        assert!(accepted.is_end_of_output());
    }

    #[test]
    fn io_receiver_rejects_sequence_wraparound() {
        let frame = Frame::data(3, u64::MAX, Channel::Stdin, b"x").unwrap();
        let mut receiver = Receiver::new(4, 3, Direction::Input).unwrap();
        receiver.next_sequence = u64::MAX;
        assert_eq!(
            receiver.accept(4, IPC_TAG, 0, frame.payload()).err(),
            Some(Error::SequenceExhausted)
        );
    }

    #[test]
    fn io_decoder_rejects_noncanonical_header_and_empty_data_frame() {
        let frame = Frame::data(5, 0, Channel::Stdout, b"x").unwrap();
        let mut malformed = frame.payload().to_vec();
        malformed[6] = 1;
        assert_eq!(decode(&malformed).err(), Some(Error::InvalidFrame));
        let mut malformed = frame.payload().to_vec();
        malformed[4] = 2;
        assert_eq!(decode(&malformed).err(), Some(Error::InvalidFrame));
        let mut malformed = frame.payload().to_vec();
        malformed[24] = 0;
        assert_eq!(decode(&malformed).err(), Some(Error::InvalidFrame));
    }
}
