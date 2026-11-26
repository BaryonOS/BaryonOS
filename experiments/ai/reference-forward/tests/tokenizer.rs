//! Tokenizer behaviour on synthetic tables; runs without the pinned model.
//! Expected pieces follow the pinned split pattern; the opt-in
//! `tokenizer_oracle` test checks the real table against the reference ids.

mod common;

use baryon_ai_reference_forward::{
    pre_tokenize, Budget, Cancel, Error, NoCancel, TokenError, Tokenizer, ADDED_TOKENS,
};
use common::blob;

const IDENTITY: [u32; 256] = {
    let mut ids = [0; 256];
    let mut i = 0;
    while i < 256 {
        ids[i] = i as u32;
        i += 1;
    }
    ids
};

/// he=256, ll=257, hell=258, hello=259, aa=260, aaa=261, aaaa=262.
fn table() -> Vec<u8> {
    let (h, e, l, o, a) = (104, 101, 108, 111, 97);
    blob(
        &IDENTITY,
        &[
            (h, e),
            (l, l),
            (256, 257),
            (258, o),
            (a, a),
            (260, a),
            (260, 260),
        ],
    )
}

fn pieces(text: &str) -> Vec<&str> {
    pre_tokenize(text).unwrap().collect()
}

fn encode(t: &Tokenizer<'_>, text: &str) -> Result<Vec<u32>, TokenError> {
    let mut out = vec![0; text.len()];
    let n = t.encode(text, &mut out, &mut Budget::new(u64::MAX, &NoCancel))?;
    Ok(out[..n].to_vec())
}

#[test]
fn split_follows_each_alternative() {
    let cases: &[(&str, &[&str])] = &[
        ("Hello world", &["Hello", " world"]),
        ("It's  fine", &["It", "'s", " ", " fine"]),
        ("WE'LL'sound", &["WE", "'LL", "'s", "ound"]),
        ("a1b22", &["a", "1", "b", "2", "2"]),
        ("(hi \u{a0}x", &["(hi", " ", "\u{a0}x"]),
        (" ?!\r\n\r\nx", &[" ?!\r\n\r\n", "x"]),
        ("x \n\n  y", &["x", " \n\n", " ", " y"]),
        ("end   ", &["end", "   "]),
        ("a \tb", &["a", " ", "\tb"]),
        ("1 \n", &["1", " \n"]),
        ("東京 is", &["東京", " is"]),
        ("€5", &["€", "5"]),
        ("", &[]),
    ];
    for (text, expected) in cases {
        assert_eq!(pieces(text), *expected, "{text:?}");
    }
}

#[test]
fn refuses_unsupported_text() {
    for text in [
        "e\u{301}",
        "\u{2000}",
        "'\u{17f}",
        "\u{ac00}",
        "\u{2065}",
        "\u{1f600}",
        "a<|im_start|>b",
        "<tool_call>",
    ] {
        assert!(
            matches!(pre_tokenize(text), Err(TokenError::Unsupported)),
            "{text:?}"
        );
    }
    assert_eq!(pieces("<|im_x|>").concat(), "<|im_x|>");
}

#[test]
fn merges_lowest_rank_leftmost_first() {
    let data = table();
    let t = Tokenizer::new(&data).unwrap();
    assert_eq!(encode(&t, "hello hello").unwrap(), [259, 32, 259]);
    assert_eq!(encode(&t, "aaaa").unwrap(), [262]);
    // A rightmost tie-break would give [97, 260].
    assert_eq!(encode(&t, "aaa").unwrap(), [261]);
    // Pieces "he", "'ll", "o": merges stay inside each piece.
    assert_eq!(encode(&t, "he'llo").unwrap(), [256, 39, 257, 111]);
}

#[test]
fn decodes_merges_and_added_tokens() {
    let data = table();
    let t = Tokenizer::new(&data).unwrap();
    let mut out = [0; 64];
    let n = t.decode(&[259, 32, 262, t.vocab() + 1], &mut out).unwrap();
    assert_eq!(&out[..n], b"hello aaaa<|im_start|>");
    let last = t.vocab() + ADDED_TOKENS.len() as u32;
    assert_eq!(t.decode(&[last], &mut out), Err(TokenError::Unsupported));
    assert_eq!(
        t.decode(&[259], &mut out[..4]),
        Err(TokenError::Limit(Error::ContextLimit))
    );
}

struct Always;
impl Cancel for Always {
    fn cancelled(&self) -> bool {
        true
    }
}

#[test]
fn limits_publish_nothing() {
    let data = table();
    let t = Tokenizer::new(&data).unwrap();
    let mut out = [7; 16];
    let r = t.encode("hello", &mut out, &mut Budget::new(3, &NoCancel));
    assert_eq!(r, Err(TokenError::Limit(Error::WorkLimit)));
    assert_eq!(out, [0; 16]);
    out = [7; 16];
    let r = t.encode("hello", &mut out, &mut Budget::new(u64::MAX, &Always));
    assert_eq!(r, Err(TokenError::Limit(Error::Cancelled)));
    assert_eq!(out, [0; 16]);
    let mut short = [7; 4];
    let r = t.encode("hello", &mut short, &mut Budget::new(u64::MAX, &NoCancel));
    assert_eq!(r, Err(TokenError::Limit(Error::ContextLimit)));
    assert_eq!(short, [0; 4]);
    let r = t.encode("é\u{301}", &mut out, &mut Budget::new(u64::MAX, &NoCancel));
    assert_eq!(r, Err(TokenError::Unsupported));
}

#[test]
fn rejects_malformed_tables() {
    let good = table();
    assert!(Tokenizer::new(&good).is_ok());
    let mut bad_magic = good.clone();
    bad_magic[0] = b'X';
    let mut duplicate_byte = good.clone();
    duplicate_byte[8..12].copy_from_slice(&1u32.to_le_bytes());
    let mut future_merge = good.clone();
    // Rank 0's left child may not be token 256 (itself).
    let merges = 8 + 1024;
    future_merge[merges..merges + 4].copy_from_slice(&256u32.to_le_bytes());
    let mut unsorted = good.clone();
    let sorted = merges + 7 * 8;
    unsorted.swap(sorted, sorted + 4);
    let mut out_of_range = good.clone();
    out_of_range[sorted..sorted + 4].copy_from_slice(&7u32.to_le_bytes());
    for (name, data) in [
        ("magic", bad_magic),
        ("truncated", good[..good.len() - 1].to_vec()),
        ("duplicate byte id", duplicate_byte),
        ("future merge", future_merge),
        ("unsorted index", unsorted),
        ("rank out of range", out_of_range),
        ("empty", Vec::new()),
    ] {
        assert!(
            matches!(Tokenizer::new(&data), Err(TokenError::Malformed)),
            "{name}"
        );
    }
}
