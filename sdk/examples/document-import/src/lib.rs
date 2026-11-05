//! Bounded DOCX plain-text import, not a document renderer.
//! XML handling uses an upstream namespace-aware parser, not a new XML engine.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use roxmltree::{Document, Node, ParsingOptions};
mod package;
pub use package::MAX_PACKAGE_BYTES;

pub const MAX_XML_BYTES: usize = 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_XML_NODES: u32 = 32768;
pub const MAX_PARAGRAPHS: usize = 4096;
const WORD: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    PackageLimit,
    InvalidPackage,
    UnsupportedPackage,
    InvalidCompression,
    InvalidBinding,
    XmlLimit,
    InvalidUtf8,
    InvalidXml,
    UnsupportedContent,
    TextLimit,
    ParagraphLimit,
    OutOfMemory,
}

const OPC_RELS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const OPC_TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
const OFFICE_DOCUMENT: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const DOCUMENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";

/// Read a DOCX package without writing to disk or resolving any relationship.
/// Only ordinary Transitional text paragraphs are accepted; unsupported content
/// fails instead of producing an incomplete successful plain-text document.
pub fn import_docx(bytes: &[u8]) -> Result<String, Error> {
    let parts = package::parts(bytes)?;
    let get = |name| {
        parts
            .iter()
            .find(|part| part.name == name)
            .map(|part| part.bytes.as_slice())
            .ok_or(Error::InvalidBinding)
    };
    let rels = parse_xml(get("_rels/.rels")?)?;
    if !rels
        .root_element()
        .has_tag_name((OPC_RELS, "Relationships"))
    {
        return Err(Error::InvalidBinding);
    }
    let mut main = false;
    let mut ids = Vec::new();
    ids.try_reserve_exact(512).map_err(|_| Error::OutOfMemory)?;
    for rel in rels.root_element().children().filter(Node::is_element) {
        let id = attribute(rel, "Id").ok_or(Error::InvalidBinding)?;
        if !rel.has_tag_name((OPC_RELS, "Relationship"))
            || id.is_empty()
            || ids.contains(&id)
            || ids.len() == 512
            || attribute(rel, "Target").is_none_or(str::is_empty)
            || attribute(rel, "Type").is_none_or(str::is_empty)
            || attribute(rel, "TargetMode").is_some_and(|mode| mode != "Internal")
        {
            return Err(Error::InvalidBinding);
        }
        require_empty(rel)?;
        ids.push(id);
        if attribute(rel, "Type") == Some(OFFICE_DOCUMENT) {
            if main
                || !matches!(
                    attribute(rel, "Target"),
                    Some("word/document.xml" | "/word/document.xml")
                )
            {
                return Err(Error::InvalidBinding);
            }
            main = true;
        }
    }
    if !main {
        return Err(Error::InvalidBinding);
    }
    let types = parse_xml(get("[Content_Types].xml")?)?;
    if !types.root_element().has_tag_name((OPC_TYPES, "Types")) {
        return Err(Error::InvalidBinding);
    }
    let mut main_type = false;
    for node in types.root_element().children().filter(Node::is_element) {
        if attribute(node, "ContentType")
            .is_some_and(|ty| ty.contains("macroEnabled") || ty.contains("vbaProject"))
        {
            return Err(Error::UnsupportedPackage);
        }
        if !node.has_tag_name((OPC_TYPES, "Override")) && !node.has_tag_name((OPC_TYPES, "Default"))
        {
            return Err(Error::InvalidBinding);
        }
        require_empty(node)?;
        if node.has_tag_name((OPC_TYPES, "Override"))
            && attribute(node, "PartName") == Some("/word/document.xml")
        {
            if main_type || attribute(node, "ContentType") != Some(DOCUMENT_TYPE) {
                return Err(Error::InvalidBinding);
            }
            main_type = true;
        }
    }
    if !main_type {
        return Err(Error::InvalidBinding);
    }
    extract_main_document_xml(get("word/document.xml")?)
}

fn parse_xml(bytes: &[u8]) -> Result<Document<'_>, Error> {
    if bytes.len() > MAX_XML_BYTES {
        return Err(Error::XmlLimit);
    }
    let xml = core::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?;
    Document::parse_with_options(
        xml,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_XML_NODES,
            entity_resolver: None,
        },
    )
    .map_err(|_| Error::InvalidXml)
}

fn attribute<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attr| attr.namespace().is_none() && attr.name() == name)
        .map(|attr| attr.value())
}

/// Interpret an already extracted main-document part.
/// Plain-text is an explicit lossy format choice, not a visual roundtrip.
pub fn extract_main_document_xml(bytes: &[u8]) -> Result<String, Error> {
    let document = parse_xml(bytes)?;
    let root = document.root_element();
    if !root.has_tag_name((WORD, "document")) {
        return Err(Error::UnsupportedContent);
    }
    let mut elements = root.children().filter(Node::is_element);
    let body = elements
        .next()
        .filter(|node| node.has_tag_name((WORD, "body")))
        .ok_or(Error::UnsupportedContent)?;
    if elements.next().is_some() {
        return Err(Error::UnsupportedContent);
    }
    for node in root.children().filter(|node| !node.is_element()) {
        reject_literal_text(node)?;
    }
    let mut text = String::new();
    text.try_reserve_exact(MAX_TEXT_BYTES)
        .map_err(|_| Error::OutOfMemory)?;
    let mut paragraphs = 0;
    let mut section = false;
    for node in body.children() {
        if !node.is_element() {
            reject_literal_text(node)?;
            continue;
        }
        if node.has_tag_name((WORD, "sectPr")) {
            if section {
                return Err(Error::UnsupportedContent);
            }
            for property in node.children() {
                if property.is_element() {
                    if property.tag_name().namespace() != Some(WORD)
                        || !matches!(
                            property.tag_name().name(),
                            "pgSz" | "pgMar" | "cols" | "docGrid" | "type" | "titlePg"
                        )
                    {
                        return Err(Error::UnsupportedContent);
                    }
                    require_empty(property)?;
                } else {
                    reject_literal_text(property)?;
                }
            }
            section = true;
            continue;
        }
        if section || !node.has_tag_name((WORD, "p")) {
            return Err(Error::UnsupportedContent);
        }
        if paragraphs == MAX_PARAGRAPHS {
            return Err(Error::ParagraphLimit);
        }
        if paragraphs > 0 {
            append(&mut text, "\n")?;
        }
        paragraphs += 1;
        for child in node.children() {
            if child.has_tag_name((WORD, "pPr")) {
                validate_formatting(child)?;
                continue;
            }
            if !child.is_element() {
                reject_literal_text(child)?;
                continue;
            }
            if !child.has_tag_name((WORD, "r")) {
                return Err(Error::UnsupportedContent);
            }
            for content in child.children() {
                if content.has_tag_name((WORD, "rPr")) {
                    validate_formatting(content)?;
                    continue;
                }
                if !content.is_element() {
                    reject_literal_text(content)?;
                } else if content.has_tag_name((WORD, "t")) {
                    if content.children().any(|child| child.is_element()) {
                        return Err(Error::UnsupportedContent);
                    }
                    for child in content.children().filter(Node::is_text) {
                        append(&mut text, child.text().unwrap_or(""))?;
                    }
                } else if content.has_tag_name((WORD, "tab")) {
                    require_empty(content)?;
                    append(&mut text, "\t")?;
                } else if content.has_tag_name((WORD, "br")) {
                    if content
                        .attribute((WORD, "type"))
                        .is_some_and(|kind| kind != "textWrapping")
                    {
                        return Err(Error::UnsupportedContent);
                    }
                    require_empty(content)?;
                    append(&mut text, "\n")?;
                } else {
                    return Err(Error::UnsupportedContent);
                }
            }
        }
    }
    Ok(text)
}

fn validate_formatting(node: Node<'_, '_>) -> Result<(), Error> {
    for property in node.children() {
        if !property.is_element() {
            reject_literal_text(property)?;
            continue;
        }
        if property.tag_name().namespace() != Some(WORD)
            || !matches!(
                property.tag_name().name(),
                "pStyle"
                    | "rStyle"
                    | "jc"
                    | "spacing"
                    | "ind"
                    | "keepNext"
                    | "keepLines"
                    | "pageBreakBefore"
                    | "contextualSpacing"
                    | "widowControl"
                    | "rFonts"
                    | "b"
                    | "bCs"
                    | "i"
                    | "iCs"
                    | "u"
                    | "strike"
                    | "dstrike"
                    | "color"
                    | "highlight"
                    | "sz"
                    | "szCs"
                    | "lang"
                    | "kern"
                    | "vertAlign"
                    | "noProof"
                    | "rtl"
                    | "cs"
                    | "snapToGrid"
            )
        {
            return Err(Error::UnsupportedContent);
        }
        require_empty(property)?;
    }
    Ok(())
}

fn append(text: &mut String, value: &str) -> Result<(), Error> {
    if value.len() > MAX_TEXT_BYTES - text.len() {
        return Err(Error::TextLimit);
    }
    text.push_str(value);
    Ok(())
}

fn reject_literal_text(node: Node<'_, '_>) -> Result<(), Error> {
    if node.is_text()
        && !node
            .text()
            .unwrap_or("")
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    {
        return Err(Error::UnsupportedContent);
    }
    Ok(())
}

fn require_empty(node: Node<'_, '_>) -> Result<(), Error> {
    if node
        .children()
        .any(|child| child.is_element() || child.is_text())
    {
        return Err(Error::UnsupportedContent);
    }
    Ok(())
}
