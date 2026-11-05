use nieuw_os_document_export::{export, Error, MAX_INPUT_BYTES, MAX_OUTPUT_BYTES, MAX_PARAGRAPHS};

fn document(output: &[u8]) -> &str {
    let start = output
        .windows(b"<?xml".len())
        .enumerate()
        .filter(|(_, bytes)| *bytes == b"<?xml")
        .nth(2)
        .unwrap()
        .0;
    let end = output[start..]
        .windows(b"</w:document>".len())
        .position(|bytes| bytes == b"</w:document>")
        .unwrap()
        + start
        + b"</w:document>".len();
    std::str::from_utf8(&output[start..end]).unwrap()
}

#[test]
fn genuine_package_preserves_unicode_whitespace_paragraphs_and_tabs() {
    let input = " Caf\u{e9} & <x> \"quote\"\t漢字 😀 \n\nlaatste\n";
    let before = input.as_bytes().to_vec();
    let output = export(input.as_bytes(), MAX_OUTPUT_BYTES).unwrap();
    assert_eq!(&output[..4], b"PK\x03\x04");
    let xml = document(&output);
    assert_eq!(xml.matches("<w:p>").count(), 4);
    assert_eq!(xml.matches("<w:tab/>").count(), 1);
    assert!(xml.contains(" Caf\u{e9} &amp; &lt;x&gt; \"quote\""));
    assert!(xml.contains("漢字 😀 "));
    assert!(xml.contains("xml:space=\"preserve\""));
    assert_eq!(input.as_bytes(), before);
    assert_eq!(export(input.as_bytes(), MAX_OUTPUT_BYTES).unwrap(), output);
}

#[test]
fn apparent_fields_links_or_markup_never_become_active_content() {
    let input = b"</w:t><w:fldSimple w:instr=\"INCLUDETEXT https://example.invalid\"/><w:t>";
    let output = export(input, MAX_OUTPUT_BYTES).unwrap();
    let xml = document(&output);
    assert!(!xml.contains("<w:fldSimple"));
    assert!(xml.contains("&lt;/w:t&gt;"));
    assert!(!output
        .windows(b"TargetMode=".len())
        .any(|bytes| bytes == b"TargetMode="));
    assert!(!output
        .windows(b"vbaProject".len())
        .any(|bytes| bytes == b"vbaProject"));
}

#[test]
fn exact_budgets_and_empty_document_are_explicit() {
    let empty = export(b"", MAX_OUTPUT_BYTES).unwrap();
    assert_eq!(document(&empty).matches("<w:p>").count(), 1);
    assert_eq!(export(b"", empty.len()).unwrap(), empty);
    assert_eq!(export(b"", empty.len() - 1), Err(Error::OutputLimit));
    assert_eq!(export(b"a", 0), Err(Error::OutputLimit));
    assert!(export(&vec![b'&'; MAX_INPUT_BYTES], MAX_OUTPUT_BYTES).is_ok());
    assert_eq!(
        export(&vec![b'a'; MAX_INPUT_BYTES + 1], MAX_OUTPUT_BYTES),
        Err(Error::InputLimit)
    );
    assert!(export(&vec![b'\n'; MAX_PARAGRAPHS - 1], MAX_OUTPUT_BYTES).is_ok());
    assert_eq!(
        export(&vec![b'\n'; MAX_PARAGRAPHS], MAX_OUTPUT_BYTES),
        Err(Error::ParagraphLimit)
    );
}

#[test]
fn invalid_utf8_xml_controls_and_carriage_returns_are_never_silently_removed() {
    assert_eq!(export(&[0xff], MAX_OUTPUT_BYTES), Err(Error::InvalidUtf8));
    for input in ["a\0", "a\r\n", "a\u{b}", "a\u{fffe}", "a\u{ffff}"] {
        assert_eq!(
            export(input.as_bytes(), MAX_OUTPUT_BYTES),
            Err(Error::UnsupportedCharacter { byte: 1 })
        );
    }
    assert_eq!(
        export("é\0".as_bytes(), MAX_OUTPUT_BYTES),
        Err(Error::UnsupportedCharacter { byte: 2 })
    );
}
