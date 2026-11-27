//! Host-only tokenizer oracle for L01: prints `{text, ids}` cases from a
//! verified local copy of the pinned `tokenizer.json` (tokenizers 0.23.2 with
//! Oniguruma). Each case records the split pieces and the ids. Edge cases target every
//! split alternative; the sweep puts each code point accepted by the no_std
//! tokenizer into several contexts, so a Unicode-class drift on either side
//! changes the pieces even where the ids happen to agree. No model is loaded.

use std::{env, path::Path};

use serde_json::json;
use tokenizers::{
    NormalizedString, Normalizer, OffsetReferential, OffsetType, PreTokenizedString, PreTokenizer,
    Tokenizer,
};

const EDGE: &[&str] = &[
    "",
    " ",
    "  ",
    "   ",
    "\n",
    "\r\n",
    "\n\n",
    "\r\r\n",
    " \n",
    "\n ",
    " \n ",
    "a  b",
    "a   b",
    "a \n b",
    "a\t\tb",
    "a\t \nb",
    "x  ",
    "x \n\n  ",
    "hello   \n\n  world",
    "It's",
    "IT'S",
    "we'LL",
    "they'Re",
    "I'd",
    "you'VE",
    "I'M",
    "'sound",
    "'Sound",
    "'TIS",
    "'LLama",
    "'VEry",
    "'Mix",
    "'REd",
    "'Dad",
    "x'Sy",
    "''s",
    "'ß",
    "'ßa",
    "rock'n'roll",
    "don't'll've",
    "aaaa",
    "aaaaaaaa",
    "        ",
    "!!!!",
    "!!!\n\n",
    " ?!\r\n\r\nx",
    "x\r\n\r\ny",
    "abc123def",
    "1,234.5",
    "0.000001",
    "x²+y³=z¹",
    "¼ ½",
    "(hello) [world] {x}",
    "a.b,c;d:e",
    "  hello",
    "\thello",
    "\u{a0}hello",
    "\u{2003}hello",
    "e-mail — test",
    "€100 and $5",
    "東京 is 京都",
    "naïve café",
    "Ærøskøbing",
    "Łódź",
    "ǅ ǈ ǋ",
    "µ ª º",
    "\u{200b}zero\u{200d}width",
    "a\u{2028}b\u{2029}c",
    "<|im_x|>",
    "<tool_call",
    "\u{0}\u{1f}\u{7f}",
    "\u{85}x",
    "x \u{85} y",
    " .\n",
    "..\r\n\r\n",
    "a\n\n\nb",
    "  \n  \n",
    "end   ",
    "tab\t",
    "#include <stdio.h>\nint main() { return 0; }\n",
    "Summarize the following text in one sentence:\n\n",
];

/// Inclusive ranges the no_std tokenizer accepts (kept independent of it).
const SWEEP: &[(u32, u32)] = &[
    (0x00, 0x17E),
    (0x180, 0x24F),
    (0x2002, 0x2064),
    (0x2066, 0x206F),
    (0x20A0, 0x20C0),
];
const CJK: (u32, u32) = (0x4E00, 0x9FFF);
const CJK_CHUNK: usize = 128;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = env::args()
        .nth(1)
        .ok_or("usage: tokenize <model-dir> [--cjk]")?;
    let tokenizer =
        Tokenizer::from_file(Path::new(&dir).join("tokenizer.json")).map_err(|e| e.to_string())?;
    let mut texts: Vec<String> = EDGE.iter().map(|s| s.to_string()).collect();
    long_runs(&mut texts);
    for &(lo, hi) in SWEEP {
        for c in (lo..=hi).filter_map(char::from_u32) {
            texts.push(format!("a{c}b {c}1{c}{c} '{c} {c}.\n{c}"));
        }
    }
    // The full CJK sweep is large; `--cjk` adds it for an uncommitted run.
    let cjk: Vec<char> = (CJK.0..=CJK.1).filter_map(char::from_u32).collect();
    let chunks = if env::args().any(|a| a == "--cjk") {
        cjk.len()
    } else {
        0
    };
    for chunk in cjk[..chunks].chunks(CJK_CHUNK) {
        let run: String = chunk.iter().collect();
        texts.push(format!("{} x{}1", run, chunk[0]));
    }
    println!("{{\"oracle\": \"tokenizers 0.23.2 (onig), pinned tokenizer.json\", \"cases\": [");
    for (i, text) in texts.iter().enumerate() {
        let ids = tokenizer
            .encode(text.as_str(), false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();
        let pieces = pieces(&tokenizer, text)?;
        let comma = if i + 1 < texts.len() { "," } else { "" };
        println!(
            "{}{comma}",
            json!({ "text": text, "pieces": pieces, "ids": ids })
        );
    }
    println!("]}}");
    Ok(())
}

/// Split pieces as slices of the original text (normalized, then pre-tokenized).
fn pieces(tokenizer: &Tokenizer, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut normalized = NormalizedString::from(text);
    tokenizer
        .get_normalizer()
        .ok_or("no normalizer")?
        .normalize(&mut normalized)
        .map_err(|e| e.to_string())?;
    let mut split = PreTokenizedString::from(normalized);
    tokenizer
        .get_pre_tokenizer()
        .ok_or("no pre-tokenizer")?
        .pre_tokenize(&mut split)
        .map_err(|e| e.to_string())?;
    Ok(split
        .get_splits(OffsetReferential::Original, OffsetType::Byte)
        .into_iter()
        .map(|(_, (start, end), _)| text[start..end].to_string())
        .collect())
}

fn long_runs(texts: &mut Vec<String>) {
    texts.push("a".repeat(300));
    texts.push("supercalifragilistic".repeat(12));
    texts.push(format!("x{}y", " ".repeat(64)));
    texts.push("!?".repeat(100));
    texts.push("1234567890".repeat(20));
}
