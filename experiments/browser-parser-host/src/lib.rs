use scraper::{ElementRef, Html, Selector};
use std::str;

pub const MAX_DOCUMENT_BYTES: usize = 64 * 1024;
pub const MAX_LINKS: usize = 64;
const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

fn is_html_title(element: ElementRef<'_>) -> bool {
    let name = element.value();
    name.name() == "title" && &*name.name.ns == HTML_NAMESPACE
}

fn is_outside_template(element: ElementRef<'_>) -> bool {
    !element
        .ancestors()
        .filter_map(ElementRef::wrap)
        .any(|ancestor| ancestor.value().name() == "template")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub href: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSummary {
    pub title: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    DocumentTooLarge,
    InvalidUtf8,
    TooManyLinks,
    InvalidSelector,
}

pub fn parse_document(input: &[u8]) -> Result<PageSummary, ParseError> {
    if input.len() > MAX_DOCUMENT_BYTES {
        return Err(ParseError::DocumentTooLarge);
    }

    let markup = str::from_utf8(input).map_err(|_| ParseError::InvalidUtf8)?;
    let document = Html::parse_document(markup);
    let title_selector = Selector::parse("title").map_err(|_| ParseError::InvalidSelector)?;
    let link_selector = Selector::parse("a[href]").map_err(|_| ParseError::InvalidSelector)?;

    let title = document
        .select(&title_selector)
        .find(|element| is_html_title(*element) && is_outside_template(*element))
        .map(|element| element.text().collect::<String>().trim().to_owned());

    let mut links = Vec::new();
    for element in document.select(&link_selector) {
        if !is_outside_template(element) {
            continue;
        }
        if links.len() == MAX_LINKS {
            return Err(ParseError::TooManyLinks);
        }

        let Some(href) = element.value().attr("href") else {
            continue;
        };
        links.push(Link {
            href: href.to_owned(),
            text: element.text().collect::<String>().trim().to_owned(),
        });
    }

    Ok(PageSummary { title, links })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture_title_and_selector_matches() {
        let page = parse_document(include_bytes!("../fixtures/page.html")).unwrap();

        assert_eq!(page.title.as_deref(), Some("Nieuw OS & browserproef"));
        assert_eq!(
            page.links,
            [Link {
                href: "/materiaal?soort=tekst&weergave=lokaal".to_owned(),
                text: "Open materiaal".to_owned(),
            }]
        );
    }

    #[test]
    fn rejects_invalid_utf8() {
        assert_eq!(parse_document(b"<p>\xff</p>"), Err(ParseError::InvalidUtf8));
    }

    #[test]
    fn rejects_documents_above_the_input_limit() {
        let input = vec![b'a'; MAX_DOCUMENT_BYTES + 1];

        assert_eq!(parse_document(&input), Err(ParseError::DocumentTooLarge));
    }

    #[test]
    fn rejects_link_output_above_the_limit() {
        let markup = "<a href='/x'>x</a>".repeat(MAX_LINKS + 1);

        assert_eq!(
            parse_document(markup.as_bytes()),
            Err(ParseError::TooManyLinks)
        );
    }

    #[test]
    fn returns_script_like_links_as_untrusted_data() {
        let page = parse_document(b"<a href='javascript:alert(1)'>text</a>").unwrap();

        assert_eq!(page.links[0].href, "javascript:alert(1)");
    }

    #[test]
    fn ignores_svg_titles_when_finding_the_html_document_title() {
        let page = parse_document(
            b"<html><head></head><body><svg><title>Grafiek</title></svg></body></html>",
        )
        .unwrap();

        assert_eq!(page.title, None);
    }

    #[test]
    fn excludes_template_contents_from_the_page_inventory() {
        let template_links = "<a href='/template'>x</a>".repeat(MAX_LINKS + 1);
        let markup = format!(
            "<html><head><title>Pagina</title></head><body><template><title>Template</title>{template_links}</template><a href='/page'>Pagina-link</a></body></html>"
        );

        let page = parse_document(markup.as_bytes()).unwrap();

        assert_eq!(page.title.as_deref(), Some("Pagina"));
        assert_eq!(
            page.links,
            [Link {
                href: "/page".to_owned(),
                text: "Pagina-link".to_owned(),
            }]
        );
    }
}
