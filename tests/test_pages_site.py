"""Regression checks for the visitor-site navigation and generated pages."""

import importlib.util
import unittest
from html.parser import HTMLParser
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "docs" / "pages"
BUILDER = ROOT / "docs" / "pages-source" / "build_docs.py"


def load_builder():
    spec = importlib.util.spec_from_file_location("baryonos_pages_builder", BUILDER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class LinkParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            href = dict(attrs).get("href")
            if href:
                self.links.append(href)


class PagesSiteTests(unittest.TestCase):
    """The primary destination, generated output and local-link invariants."""

    @classmethod
    def setUpClass(cls):
        cls.builder = load_builder()
        cls.pages = cls.builder.render()

    def test_primary_destination_has_one_active_navigation_item(self):
        cases = {
            SITE / "docs" / "index.html": 'href="/docs/" aria-current="page"',
            SITE / "docs" / "progress" / "index.html": 'href="/docs/" aria-current="page"',
            SITE / "status" / "index.html": 'href="/status/" aria-current="page"',
            SITE / "contribute" / "index.html": 'href="/contribute/" aria-current="page"',
            SITE / "search" / "index.html": 'href="/search/" aria-current="page"',
        }
        for path, expected in cases.items():
            with self.subTest(path=path):
                content = self.pages[path]
                header = content.split("</header>", 1)[0]
                self.assertEqual(header.count('aria-current="page"'), 1)
                self.assertIn(expected, header)

    def test_primary_links_do_not_route_status_or_contribute_through_docs(self):
        pages = list(self.pages.values()) + [(SITE / "index.html").read_text(encoding="utf-8")]
        for content in pages:
            header = content.split("</header>", 1)[0]
            self.assertIn('href="/status/"', header)
            self.assertIn('>Status</a>', header)
            self.assertIn('href="/contribute/"', header)
            self.assertIn('>Contribute</a>', header)
            self.assertIn('href="/search/"', header)
            self.assertIn('>Search</a>', header)
            self.assertNotIn('href="/docs/progress/">Status</a>', header)
            self.assertNotIn('href="/docs/contribute/">Contribute</a>', header)

    def test_docs_directory_contains_every_documentation_topic(self):
        directory = self.pages[SITE / "docs" / "index.html"]
        topics = [topic for group in self.builder.GROUPS.values() for topic in group]
        self.assertGreaterEqual(len(topics), 20)
        self.assertNotIn('/docs/contribute/"', directory)
        for slug, title, _ in topics:
            with self.subTest(slug=slug):
                self.assertIn(f'href="/docs/{slug}/"', directory)
                self.assertIn(title, directory)

    def test_mission_page_preserves_eurobarometer_evidence_boundary(self):
        mission = self.pages[SITE / "docs" / "mission" / "index.html"]
        self.assertIn("58%", mission)
        self.assertIn("EU-based <em>digital service provider</em>", mission)
        self.assertIn("did not ask whether they would install a European operating system", mission)
        self.assertIn("not an OS adoption forecast", mission)

    def test_plain_language_choices_page_covers_visitor_facing_decisions(self):
        choices = self.pages[SITE / "docs" / "choices" / "index.html"]
        for heading in (
            "Why keep the desktop familiar?",
            "Why build an independent kernel?",
            "Why organize work around Material?",
            "Why are app permissions narrow?",
            "Why is local-first still connected?",
            "Why is accessibility part of the identity?",
            "Why is AI optional and on-device?",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, choices)

    def test_search_page_uses_a_local_index_and_minimum_csp(self):
        search = self.pages[SITE / "search" / "index.html"]
        status = self.pages[SITE / "status" / "index.html"]
        script = self.builder.render_assets()[SITE / "assets" / "search.js"]

        self.assertIn('<form class="site-search" role="search"', search)
        self.assertIn('<label for="site-search-input">', search)
        self.assertIn('id="search-result-status" aria-live="polite"', search)
        self.assertIn('<noscript>', search)
        self.assertIn('href="/docs/#topics"', search)
        self.assertIn("script-src 'self'", search)
        self.assertIn("form-action 'self'", search)
        self.assertIn('<script src="/assets/search.js?v=site-search" defer></script>', search)
        self.assertIn("script-src 'none'", status)
        self.assertIn("form-action 'none'", status)
        self.assertNotIn("fetch(", script)

        for group in self.builder.GROUPS.values():
            for slug, title, _ in group:
                with self.subTest(slug=slug):
                    self.assertIn(f'"route":"/docs/{slug}/"', script)
                    self.assertIn(f'"title":"{title}"', script)

    def test_faq_and_search_tips_use_native_disclosures(self):
        faq = self.pages[SITE / "docs" / "faq" / "index.html"]
        search = self.pages[SITE / "search" / "index.html"]
        self.assertEqual(faq.count('<details class="disclosure">'), 7)
        self.assertIn('<summary>Can it be installed on a personal computer?</summary>', faq)
        self.assertIn('<details class="disclosure search-tips">', search)
        self.assertIn('<summary>Search tips</summary>', search)

    def test_committed_generated_output_matches_rendered_pages(self):
        for path, content in self.pages.items():
            with self.subTest(path=path):
                self.assertTrue(path.is_file())
                self.assertEqual(path.read_text(encoding="utf-8"), content)

    def test_root_relative_page_links_resolve(self):
        pages = dict(self.pages)
        pages[SITE / "index.html"] = (SITE / "index.html").read_text(encoding="utf-8")
        for source, content in pages.items():
            parser = LinkParser()
            parser.feed(content)
            for href in parser.links:
                if not href.startswith("/"):
                    continue
                route = href.split("#", 1)[0].split("?", 1)[0]
                if not route:
                    continue
                candidate = SITE / route.lstrip("/")
                target = candidate / "index.html" if route.endswith("/") else candidate
                with self.subTest(source=source, href=href):
                    self.assertTrue(target.is_file(), f"missing local page target {target}")


if __name__ == "__main__":
    unittest.main()
