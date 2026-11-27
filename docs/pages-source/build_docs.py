"""Render the static visitor documentation and primary project pages."""

import argparse
from html import escape
from pathlib import Path
from string import Template
from textwrap import indent

SOURCE = Path(__file__).resolve().parent
SITE = SOURCE.parent / "pages"
DOCS_OUTPUT = SITE / "docs"

# Each tuple is the URL slug, visible title, and short visitor-facing summary.
GROUPS = {
    "Decide if it fits": [
        ("project", "About Baryon OS", "Why the project exists, what the name means and what makes the OS independent."),
        ("mission", "Mission and vision", "The European research question, the mission it produced and the future experience being pursued."),
        ("fit", "Is Baryon OS for me?", "A direct suitability guide for curious visitors, testers, contributors and daily-use seekers."),
        ("capabilities", "Planned capabilities", "The intended product experience, from everyday apps to storage, networking and recovery."),
        ("choices", "Why these choices?", "The main product and technology decisions explained in everyday language and linked to their evidence."),
        ("progress", "Roadmap and evidence", "How development progress is measured and how to interpret the available evidence."),
        ("faq", "Frequently asked questions", "Short answers about Linux, installation, readiness, privacy, licensing and support."),
    ],
    "Experience and personal data": [
        ("design", "Desktop and Material", "How Orbit, familiar desktop interactions and the document model fit together."),
        ("profiles", "Profiles and accessibility", "The intended first start, local accounts, trusted sign-in and adaptive experience."),
        ("apps", "Apps and permissions", "How apps, packages, data access and replaceable tools are meant to remain separate."),
        ("storage", "Storage, sync and NAS", "Local work, encrypted remote copies, personal NAS roles, conflicts and recovery."),
        ("privacy", "Privacy and data control", "Where data may go, which choices remain explicit and what local-first actually means."),
        ("local-ai", "Local AI", "The on-device assistant direction, model boundaries and current experiment limits."),
    ],
    "Inside the system": [
        ("architecture", "System architecture", "Follow the boot chain from firmware and the Rust kernel through services to apps."),
        ("security", "Security model", "Capabilities, isolation, the Security Floor, threat boundaries and evidence limits."),
        ("packages", "Packages and updates", "Provenance, installation, permissions, system updates and remaining integration work."),
        ("research", "Research tracks", "Explore work on local AI, post-quantum protection and quantum execution."),
    ],
    "Try and verify": [
        ("try", "Try the development build", "How to start a bounded QEMU experiment without treating it as an installer."),
        ("hardware", "Hardware and release readiness", "Why virtual-machine progress differs from PC support and a maintained release."),
        ("testing", "Testing and evidence", "What host, build, QEMU, physical-hardware and human evidence can establish."),
    ],
    "Explore the project": [
        ("workflow", "Development workflow", "How a bounded change moves from current state and contracts to evidence and review."),
        ("repository", "Repository guide", "A map of the loader, kernel, services, drivers, SDK, UI, tests and research areas."),
        ("sources", "Sources and documentation", "Which project source answers which question and how the website and wiki relate."),
        ("glossary", "Glossary", "Look up terms such as kernel, capability, NAS and QEMU in plain English."),
    ],
}


def nav_state(active):
    return {
        "overview_current": ' aria-current="page"' if active == "overview" else "",
        "docs_current": ' aria-current="page"' if active == "docs" else "",
        "status_current": ' aria-current="page"' if active == "status" else "",
        "contribute_current": ' aria-current="page"' if active == "contribute" else "",
    }


def render_page(template, *, title, heading, description, breadcrumb, body, active):
    return template.substitute(
        title=title,
        heading=heading,
        description=description,
        breadcrumb=breadcrumb,
        body=body,
        **nav_state(active),
    )


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


def topic_navigation(current_slug):
    items = []
    for heading, topics in GROUPS.items():
        if not any(slug == current_slug for slug, _, _ in topics):
            continue
        items.append(f'          <li class="guide-topic-heading">{escape(heading)}</li>')
        for slug, title, _ in topics:
            current = ' aria-current="page"' if slug == current_slug else ""
            items.append(f'          <li><a href="/docs/{slug}/"{current}>{escape(title)}</a></li>')
        break
    items.extend([
        '          <li class="guide-topic-heading">Explore</li>',
        '          <li><a href="/docs/#topics">All Docs topics</a></li>',
    ])
    return "\n".join(items)


def chapter(slug):
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
{topic_navigation(slug)}
        </ul>
      </nav>
    </div>"""


def standalone(name):
    fragment = (SOURCE / f"{name}.html").read_text(encoding="utf-8")
    return indent(fragment.strip(), "    ")


def legacy_contribute():
    return """    <div class="wrap standalone-layout">
      <article class="guide-topic section-divider" aria-labelledby="guide-title">
        <h2>Contribute has moved</h2>
        <p>Contribution routes now have their own primary page, separate from the documentation library.</p>
        <p><a class="button button-primary" href="/contribute/">Open Contribute <span aria-hidden="true">→</span></a></p>
      </article>
    </div>"""


def render():
    template = Template((SOURCE / "layout.html").read_text(encoding="utf-8"))
    topics = [topic for group in GROUPS.values() for topic in group]
    pages = {
        DOCS_OUTPUT / "index.html": render_page(
            template,
            title="Baryon OS Docs",
            heading="Baryon OS Docs",
            description="Choose the question that matters to you: suitability, experience, architecture, evidence or development.",
            active="docs",
            breadcrumb='<p class="breadcrumb"><a href="/">Home</a><span aria-hidden="true">/</span>Docs</p>',
            body=directory(),
        ),
        SITE / "status" / "index.html": render_page(
            template,
            title="Status · Baryon OS",
            heading="Status",
            description="A direct view of accepted progress, open product boundaries and release readiness.",
            active="status",
            breadcrumb='<p class="breadcrumb"><a href="/">Home</a><span aria-hidden="true">/</span>Status</p>',
            body=standalone("status"),
        ),
        SITE / "contribute" / "index.html": render_page(
            template,
            title="Contribute · Baryon OS",
            heading="Contribute",
            description="Help through questions, product feedback, reproducible reports, documentation, testing or code.",
            active="contribute",
            breadcrumb='<p class="breadcrumb"><a href="/">Home</a><span aria-hidden="true">/</span>Contribute</p>',
            body=standalone("contribute"),
        ),
        DOCS_OUTPUT / "contribute" / "index.html": render_page(
            template,
            title="Contribute has moved · Baryon OS",
            heading="Contribute has moved",
            description="Contribution routes now have a primary page outside the Docs library.",
            active=None,
            breadcrumb='<p class="breadcrumb"><a href="/">Home</a><span aria-hidden="true">/</span>Contribute</p>',
            body=legacy_contribute(),
        ),
    }
    for slug, title, summary in topics:
        heading = escape(title)
        pages[DOCS_OUTPUT / slug / "index.html"] = render_page(
            template,
            title=f"{heading} · Baryon OS Docs",
            heading=heading,
            description=escape(summary),
            active="docs",
            breadcrumb=f'<p class="breadcrumb"><a href="/docs/">Docs</a><span aria-hidden="true">/</span>{heading}</p>',
            body=chapter(slug),
        )
    return pages


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Check committed pages without writing")
    args = parser.parse_args()
    pages = render()
    if args.check:
        stale = [path for path, content in pages.items() if not path.is_file() or path.read_text(encoding="utf-8") != content]
        expected_docs = {path for path in pages if DOCS_OUTPUT in path.parents}
        unexpected = set(DOCS_OUTPUT.rglob("index.html")) - expected_docs
        if stale or unexpected:
            for path in sorted(set(stale) | unexpected):
                print(f"Stale or unexpected visitor page: {path.relative_to(SITE)}")
            return 1
    else:
        for path, content in pages.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    print(f"{len(pages)} static visitor pages {'match the sources' if args.check else 'rendered'}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
