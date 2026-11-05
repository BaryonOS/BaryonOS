use nieuw_os_document_import::{extract_main_document_xml, Error, MAX_TEXT_BYTES, MAX_XML_BYTES};

fn document(body: &str) -> Vec<u8> {
    format!("<a:document xmlns:a=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><a:body>{body}</a:body></a:document>").into_bytes()
}

#[test]
fn namespace_not_prefix_controls_real_text_paragraph_tab_and_linebreak_content() {
    let xml = document("<a:p><a:r><a:t xml:space=\"preserve\"> Café &amp; 漢字 😀 </a:t><a:tab/><a:t>x</a:t><a:br/><a:t>y</a:t></a:r></a:p><a:p/><a:p><a:r><a:t>einde</a:t></a:r></a:p>");
    assert_eq!(
        extract_main_document_xml(&xml).unwrap(),
        " Café & 漢字 😀 \tx\ny\n\neinde"
    );
}

#[test]
fn fields_foreign_namespaces_tables_changes_and_hidden_content_are_not_silently_lost() {
    for body in [
        "<a:p>\u{a0}<a:r><a:t>x</a:t></a:r></a:p>",
        "<a:p><a:r><a:t xmlns:a=\"urn:not-word\">fake</a:t></a:r></a:p>",
        "<a:tbl/>",
        "<a:p><a:ins/></a:p>",
        "<a:p><a:r><a:instrText>INCLUDETEXT something</a:instrText></a:r></a:p>",
        "<a:p><a:r><a:rPr><a:vanish/></a:rPr><a:t>hidden</a:t></a:r></a:p>",
        "<a:p><a:r><a:br a:type=\"page\"/></a:r></a:p>",
        "<a:p><a:r><a:tab>hidden</a:tab></a:r></a:p>",
        "<a:p><a:r><a:t><a:t>nested</a:t></a:t></a:r></a:p>",
    ] {
        assert_eq!(
            extract_main_document_xml(&document(body)),
            Err(Error::UnsupportedContent),
            "{body}"
        );
    }
}

#[test]
fn paragraph_limit_is_exact_and_formatting_cannot_hide_unknown_content() {
    let maximum = document(&"<a:p/>".repeat(4096));
    assert_eq!(
        extract_main_document_xml(&maximum).unwrap(),
        "\n".repeat(4095)
    );
    assert_eq!(
        extract_main_document_xml(&document(&"<a:p/>".repeat(4097))),
        Err(Error::ParagraphLimit)
    );
    for body in [
        "<a:p><a:pPr><a:numPr/></a:pPr></a:p>",
        "<a:p><a:pPr><a:t>not formatting</a:t></a:pPr></a:p>",
        "<a:p><a:r><a:rPr><a:b><a:t>hidden</a:t></a:b></a:rPr></a:r></a:p>",
        "<a:sectPr/><a:p/>",
        "<a:sectPr><a:footerReference/></a:sectPr>",
    ] {
        assert_eq!(
            extract_main_document_xml(&document(body)),
            Err(Error::UnsupportedContent),
            "{body}"
        );
    }
}

#[test]
fn xml_dtd_external_entities_malformed_input_and_limits_fail_explicitly() {
    assert_eq!(extract_main_document_xml(&[0xff]), Err(Error::InvalidUtf8));
    assert_eq!(
        extract_main_document_xml(b"<unclosed>"),
        Err(Error::InvalidXml)
    );
    let dtd = b"<!DOCTYPE x [<!ENTITY ext SYSTEM 'file:///untrusted'>]><x>&ext;</x>";
    assert_eq!(extract_main_document_xml(dtd), Err(Error::InvalidXml));
    assert_eq!(
        extract_main_document_xml(&vec![b'a'; MAX_XML_BYTES + 1]),
        Err(Error::XmlLimit)
    );
    let long = document(&format!(
        "<a:p><a:r><a:t>{}</a:t></a:r></a:p>",
        "a".repeat(MAX_TEXT_BYTES)
    ));
    assert_eq!(
        extract_main_document_xml(&long).unwrap().len(),
        MAX_TEXT_BYTES
    );
    let long = document(&format!(
        "<a:p><a:r><a:t>{}</a:t></a:r></a:p>",
        "a".repeat(MAX_TEXT_BYTES + 1)
    ));
    assert_eq!(extract_main_document_xml(&long), Err(Error::TextLimit));
}

#[test]
fn deeply_nested_input_uses_patched_parser_and_rejects_unsupported_structure() {
    let nested = document(&format!(
        "{}x{}",
        "<a:p>".repeat(12000),
        "</a:p>".repeat(12000)
    ));
    assert_eq!(
        extract_main_document_xml(&nested),
        Err(Error::UnsupportedContent)
    );
    let nodes = document(&"<a:p/>".repeat(33000));
    assert_eq!(extract_main_document_xml(&nodes), Err(Error::InvalidXml));
}
