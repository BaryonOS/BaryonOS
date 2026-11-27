"""Render the static visitor documentation for the Baryon OS project site."""

import argparse
from html import escape
from pathlib import Path
from string import Template
from textwrap import indent

SOURCE = Path(__file__).resolve().parent
OUTPUT = SOURCE.parent / "pages" / "docs"

# Each tuple is the URL slug, visible title, and short visitor-facing summary.
GROUPS = {
    "Start here": [
        ("project", "About Baryon OS", "Why the project exists, what the name means and what makes the OS independent."),
        ("progress", "Roadmap and evidence", "How far development has progressed and how to read the test evidence."),
        ("try", "Try the development build", "How to start a QEMU experiment and what hardware support is available."),
    ],
    "How it works": [
        ("architecture", "System architecture", "Follow the boot chain from firmware and the Rust kernel through services to apps."),
        ("design", "Desktop and Material", "How Orbit, familiar desktop interactions and the document model fit together."),
        ("privacy", "Privacy and security", "Understand app permissions, local storage, optional sync and the security requirements."),
    ],
    "Go further": [
        ("research", "Research tracks", "Explore work on local AI, post-quantum protection and quantum execution."),
        ("contribute", "Contribute", "Find a way to help through feedback, reproducible reports, documentation or code."),
        ("glossary", "Glossary", "Look up terms such as kernel, capability, NAS and QEMU in plain English."),
    ],
}


def directory():
    groups = []
    for heading, topics in GROUPS.items():
        cards = "\n".join(
            f'          <li id="{slug}"><a class="resource-card topic-card" href="/docs/{slug}/">'
            f'<strong>{escape(title)} <span aria-hidden="true">→</span></strong>'
            f'<span>{escape(summary)}</span></a></li>'
            for slug, title, summary in topics
        )
        groups.append(f"""      <section class="topic-group section-divider">
        <h2>{escape(heading)}</h2>
        <ul class="topic-grid">
{cards}
        </ul>
      </section>""")
    sections = "\n".join(groups)
    return f"""    <div class="wrap docs-directory" id="topics" tabindex="-1">
{sections}
    </div>"""


def chapter(slug, topics):
    links = []
    for route, title, _ in topics:
        current = ' aria-current="page"' if route == slug else ""
        links.append(f'          <li><a href="/docs/{route}/"{current}>{escape(title)}</a></li>')
    navigation = "\n".join(links)
    fragment = (SOURCE / "topics" / f"{slug}.html").read_text(encoding="utf-8")
    content = indent(fragment.strip(), "        ")
    return f"""    <div class="wrap guide-layout">
      <article class="guide-topic section-divider" aria-labelledby="guide-title">
{content}
        <div class="guide-return">
          <a class="button button-secondary" href="/docs/#topics"><span aria-hidden="true">←</span> All Docs topics</a>
        </div>
      </article>
      <nav class="guide-topics" aria-label="Documentation topics">
        <p class="guide-topics-title">Docs topics</p>
        <ul>
{navigation}
        </ul>
      </nav>
    </div>"""


def render():
    template = Template((SOURCE / "layout.html").read_text(encoding="utf-8"))
    topics = [topic for group in GROUPS.values() for topic in group]
    pages = {
        OUTPUT / "index.html": template.substitute(
            title="Baryon OS Docs",
            heading="Baryon OS Docs",
            description="Choose a topic to learn about the project, follow development, try a build or contribute.",
            docs_current="page",
            breadcrumb='<p class="breadcrumb"><a href="/">Home</a><span aria-hidden="true">/</span>Docs</p>',
            body=directory(),
        )
    }
    for slug, title, summary in topics:
        heading = escape(title)
        pages[OUTPUT / slug / "index.html"] = template.substitute(
            title=f"{heading} · Baryon OS Docs",
            heading=heading,
            description=escape(summary),
            docs_current="location",
            breadcrumb=f'<p class="breadcrumb"><a href="/docs/">Docs</a><span aria-hidden="true">/</span>{heading}</p>',
            body=chapter(slug, topics),
        )
    return pages


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Check committed pages without writing")
    args = parser.parse_args()
    pages = render()
    if args.check:
        stale = [path for path, content in pages.items() if not path.is_file() or path.read_text(encoding="utf-8") != content]
        unexpected = set(OUTPUT.rglob("index.html")) - pages.keys()
        if stale or unexpected:
            for path in sorted(set(stale) | unexpected):
                print(f"Stale or unexpected Docs output: {path.relative_to(OUTPUT.parent)}")
            return 1
    else:
        for path, content in pages.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    print(f"{len(pages)} static Docs pages {'match the sources' if args.check else 'rendered'}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
