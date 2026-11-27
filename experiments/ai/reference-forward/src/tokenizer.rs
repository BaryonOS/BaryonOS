//! Byte-level BPE tokenizer for the pinned Qwen2.5-0.5B-Instruct tokenizer (L01).
//!
//! The table is a borrowed, validated byte blob; nothing is allocated. Token
//! ids are `256 + merge rank` (bytes first), which the host builder checks
//! against the pinned `tokenizer.json`, so no vocabulary strings are stored.
//! Text containing a control-token literal (`<|im_start|>`, `<tool_call>`, ...)
//! is refused: untrusted text never yields a control id. Host evidence only.

use crate::{Budget, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenError {
    /// The text holds a character outside the supported set or a control-token
    /// literal, or an id is outside the vocabulary. Nothing was guessed.
    Unsupported,
    /// The tokenizer table failed validation.
    Malformed,
    /// Output capacity (`ContextLimit`), work budget or cancellation; no ids
    /// are published.
    Limit(Error),
}

/// Blob layout (little-endian u32 unless noted): `MAGIC`, merge count `n`,
/// 256 byte ids, `n` merge pairs `(left, right)` in rank order, then `n` ranks
/// sorted by their pair.
pub const MAGIC: [u8; 4] = *b"BTK1";

/// Pinned added tokens in id order; their ids start at `256 + n`.
pub const ADDED_TOKENS: [&str; 22] = [
    "<|endoftext|>",
    "<|im_start|>",
    "<|im_end|>",
    "<|object_ref_start|>",
    "<|object_ref_end|>",
    "<|box_start|>",
    "<|box_end|>",
    "<|quad_start|>",
    "<|quad_end|>",
    "<|vision_start|>",
    "<|vision_end|>",
    "<|vision_pad|>",
    "<|image_pad|>",
    "<|video_pad|>",
    "<tool_call>",
    "</tool_call>",
    "<|fim_prefix|>",
    "<|fim_middle|>",
    "<|fim_suffix|>",
    "<|fim_pad|>",
    "<|repo_name|>",
    "<|file_sep|>",
];

const BYTE_TABLE: usize = 8;
const MERGES: usize = BYTE_TABLE + 256 * 4;
/// Explicit decode stack; the deepest pinned token is far shallower.
const DECODE_STACK: usize = 256;

pub struct Tokenizer<'a> {
    merges: &'a [u8],
    sorted: &'a [u8],
    count: u32,
    byte_ids: [u32; 256],
    byte_of: [u8; 256],
}

fn word(data: &[u8], index: usize) -> u32 {
    let at = index * 4;
    u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

impl<'a> Tokenizer<'a> {
    /// Validate the whole table before use: exact length, byte ids a
    /// permutation of 0..256, each merge built only from earlier tokens, and
    /// a strictly increasing pair index (so every pair is unique).
    pub fn new(data: &'a [u8]) -> Result<Self, TokenError> {
        if data.len() < MERGES || data[..4] != MAGIC {
            return Err(TokenError::Malformed);
        }
        let count = word(data, 1);
        let n = count as usize;
        let sorted_at = n
            .checked_mul(8)
            .and_then(|m| m.checked_add(MERGES))
            .ok_or(TokenError::Malformed)?;
        let end = n
            .checked_mul(4)
            .and_then(|s| s.checked_add(sorted_at))
            .ok_or(TokenError::Malformed)?;
        if end != data.len() || count > u32::MAX - 256 - ADDED_TOKENS.len() as u32 {
            return Err(TokenError::Malformed);
        }
        let mut byte_ids = [0; 256];
        let mut byte_of = [0; 256];
        let mut seen = [false; 256];
        for (b, id) in byte_ids.iter_mut().enumerate() {
            *id = word(&data[BYTE_TABLE..MERGES], b);
            let slot = seen.get_mut(*id as usize).ok_or(TokenError::Malformed)?;
            if *slot {
                return Err(TokenError::Malformed);
            }
            *slot = true;
            byte_of[*id as usize] = b as u8;
        }
        let tokenizer = Self {
            merges: &data[MERGES..sorted_at],
            sorted: &data[sorted_at..],
            count,
            byte_ids,
            byte_of,
        };
        for rank in 0..count {
            let (left, right) = tokenizer.pair(rank);
            if left >= 256 + rank || right >= 256 + rank {
                return Err(TokenError::Malformed);
            }
        }
        let mut previous = None;
        for i in 0..n {
            let rank = word(tokenizer.sorted, i);
            if rank >= count {
                return Err(TokenError::Malformed);
            }
            let key = tokenizer.pair(rank);
            if previous.is_some_and(|p| p >= key) {
                return Err(TokenError::Malformed);
            }
            previous = Some(key);
        }
        Ok(tokenizer)
    }

    /// Ordinary vocabulary size (bytes plus merges); added ids follow it.
    pub fn vocab(&self) -> u32 {
        256 + self.count
    }

    fn pair(&self, rank: u32) -> (u32, u32) {
        let i = rank as usize * 2;
        (word(self.merges, i), word(self.merges, i + 1))
    }

    fn rank(&self, left: u32, right: u32) -> Option<u32> {
        let (mut lo, mut hi) = (0, self.count as usize);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let rank = word(self.sorted, mid);
            match self.pair(rank).cmp(&(left, right)) {
                core::cmp::Ordering::Less => lo = mid + 1,
                core::cmp::Ordering::Greater => hi = mid,
                core::cmp::Ordering::Equal => return Some(rank),
            }
        }
        None
    }

    /// Encode ordinary text into `out` and return the id count. Every pair
    /// lookup costs one budget unit. A piece's raw bytes must fit in `out`
    /// before merging (conservative). On any error `out` is zeroed.
    pub fn encode(
        &self,
        text: &str,
        out: &mut [u32],
        budget: &mut Budget<'_>,
    ) -> Result<usize, TokenError> {
        let result = self.encode_into(text, out, budget);
        if result.is_err() {
            out.fill(0);
        }
        result
    }

    fn encode_into(
        &self,
        text: &str,
        out: &mut [u32],
        budget: &mut Budget<'_>,
    ) -> Result<usize, TokenError> {
        let mut len = 0;
        for piece in pre_tokenize(text)? {
            // Polls cancellation even for one-byte pieces, which skip the pair loop.
            budget.charge(0).map_err(TokenError::Limit)?;
            let end = len + piece.len();
            let ids = out
                .get_mut(len..end)
                .ok_or(TokenError::Limit(Error::ContextLimit))?;
            for (id, &b) in ids.iter_mut().zip(piece.as_bytes()) {
                *id = self.byte_ids[b as usize];
            }
            let mut n = ids.len();
            // Lowest rank first, leftmost on ties; new pairs always rank
            // higher (validated), so this equals the rank-ordered queue.
            while n > 1 {
                budget.charge(n as u64 - 1).map_err(TokenError::Limit)?;
                let mut best: Option<(u32, usize)> = None;
                for p in 0..n - 1 {
                    if let Some(rank) = self.rank(ids[p], ids[p + 1]) {
                        if best.is_none_or(|(b, _)| rank < b) {
                            best = Some((rank, p));
                        }
                    }
                }
                let Some((rank, p)) = best else { break };
                ids[p] = 256 + rank;
                ids.copy_within(p + 2..n, p + 1);
                n -= 1;
            }
            len += n;
        }
        Ok(len)
    }

    /// Decode ids to bytes (added tokens as their literal text) and return the
    /// byte count. The bytes need not end on a UTF-8 boundary.
    pub fn decode(&self, ids: &[u32], out: &mut [u8]) -> Result<usize, TokenError> {
        let full = TokenError::Limit(Error::ContextLimit);
        let mut written = 0;
        for &id in ids {
            if let Some(text) = id
                .checked_sub(self.vocab())
                .and_then(|i| ADDED_TOKENS.get(i as usize))
            {
                let dest = out.get_mut(written..written + text.len()).ok_or(full)?;
                dest.copy_from_slice(text.as_bytes());
                written += text.len();
                continue;
            }
            if id >= self.vocab() {
                return Err(TokenError::Unsupported);
            }
            let mut stack = [0u32; DECODE_STACK];
            stack[0] = id;
            let mut depth = 1;
            while depth > 0 {
                depth -= 1;
                let token = stack[depth];
                if token < 256 {
                    *out.get_mut(written).ok_or(full)? = self.byte_of[token as usize];
                    written += 1;
                } else {
                    let (left, right) = self.pair(token - 256);
                    if depth + 2 > DECODE_STACK {
                        return Err(full);
                    }
                    stack[depth] = right;
                    stack[depth + 1] = left;
                    depth += 2;
                }
            }
        }
        Ok(written)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Letter,
    Number,
    Space,
    Other,
}

// shortcut: a fixed whitelist of assigned, combining-class-0, NFC-stable code
// points (ASCII, Latin-1, Latin Extended-A/B without U+017F, General
// Punctuation without U+2000/U+2001/U+2065, Currency Symbols to U+20C0, CJK
// Unified Ideographs), so NFC is the identity and no composition tables are
// needed; extend it only with new differential oracle evidence.
fn class(c: char) -> Option<Class> {
    Some(match c as u32 {
        0x09..=0x0D | 0x20 | 0x85 | 0xA0 | 0x2002..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F => {
            Class::Space
        }
        0x30..=0x39 | 0xB2 | 0xB3 | 0xB9 | 0xBC..=0xBE => Class::Number,
        0x41..=0x5A
        | 0x61..=0x7A
        | 0xAA
        | 0xB5
        | 0xBA
        | 0xC0..=0xD6
        | 0xD8..=0xF6
        | 0xF8..=0x17E
        | 0x180..=0x24F
        | 0x4E00..=0x9FFF => Class::Letter,
        0x00..=0xFF | 0x200B..=0x2064 | 0x2066..=0x206F | 0x20A0..=0x20C0 => Class::Other,
        _ => return None,
    })
}

/// Pieces of the pinned split pattern
/// `(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+`
/// (leftmost alternative, greedy with backtracking). Every character is
/// covered, so the pieces concatenate to the text.
pub struct Pieces<'t> {
    text: &'t str,
    at: usize,
}

/// Validate the whole text first, then split it lazily.
pub fn pre_tokenize(text: &str) -> Result<Pieces<'_>, TokenError> {
    if text.chars().any(|c| class(c).is_none()) || ADDED_TOKENS.iter().any(|t| text.contains(t)) {
        return Err(TokenError::Unsupported);
    }
    Ok(Pieces { text, at: 0 })
}

impl<'t> Iterator for Pieces<'t> {
    type Item = &'t str;

    fn next(&mut self) -> Option<&'t str> {
        if self.at >= self.text.len() {
            return None;
        }
        let end = piece_end(self.text, self.at);
        let piece = &self.text[self.at..end];
        self.at = end;
        Some(piece)
    }
}

fn at(t: &str, i: usize) -> Option<(char, Class, usize)> {
    let c = t[i..].chars().next()?;
    Some((c, class(c)?, i + c.len_utf8()))
}

fn run(t: &str, mut i: usize, keep: impl Fn(char, Class) -> bool) -> usize {
    while let Some((c, k, next)) = at(t, i) {
        if !keep(c, k) {
            break;
        }
        i = next;
    }
    i
}

fn piece_end(t: &str, i: usize) -> usize {
    let Some((c, k, next)) = at(t, i) else {
        return t.len();
    };
    let letter = |_, k| k == Class::Letter;
    if c == '\'' {
        let rest = &t.as_bytes()[next..];
        for s in ["s", "t", "re", "ve", "m", "ll", "d"] {
            if rest.len() >= s.len() && rest[..s.len()].eq_ignore_ascii_case(s.as_bytes()) {
                return next + s.len();
            }
        }
    }
    if k == Class::Letter {
        return run(t, i, letter);
    }
    if c != '\r' && c != '\n' && k != Class::Number {
        if let Some((_, Class::Letter, _)) = at(t, next) {
            return run(t, next, letter);
        }
    }
    if k == Class::Number {
        return next;
    }
    let start = if c == ' ' { next } else { i };
    if let Some((_, Class::Other, _)) = at(t, start) {
        let others = run(t, start, |_, k| k == Class::Other);
        return run(t, others, |c, _| c == '\r' || c == '\n');
    }
    // Whitespace from here on.
    let end = run(t, i, |_, k| k == Class::Space);
    if let Some(p) = t[i..end].rfind(['\r', '\n']) {
        return i + p + 1;
    }
    if end < t.len() {
        if let Some((last, _)) = t[i..end].char_indices().last().filter(|&(p, _)| p > 0) {
            return i + last;
        }
    }
    end
}
