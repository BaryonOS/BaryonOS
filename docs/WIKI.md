# English GitHub Wiki source

The Markdown pages in `docs/wiki/` are the reviewable English source for the
GitHub Wiki requested in [issue #277](https://github.com/simonvanieperen/BaryonOS/issues/277).
They explain concepts and link to canonical repository contracts; they do not
maintain a second roadmap, status log, or acceptance register.

## Review locally

- [Baryon OS](wiki/Home.md)
- [What is Baryon OS?](wiki/What-is-Baryon-OS.md)
- [Why Baryon?](wiki/Why-Baryon.md)
- [Architecture Overview](wiki/Architecture-Overview.md)
- [Material](wiki/Material.md)
- [Security Model](wiki/Security-Model.md)
- [Roadmap Explained](wiki/Roadmap-Explained.md)
- [Build and Run](wiki/Build-and-Run.md)
- [Testing and Evidence](wiki/Testing-and-Evidence.md)
- [Privacy and Local-first](wiki/Privacy-and-Local-first.md)
- [Research Tracks](wiki/Research-Tracks.md)
- [Repository Guide](wiki/Repository-Guide.md)
- [Glossary](wiki/Glossary.md)
- [FAQ](wiki/FAQ.md)

## Publication after review

GitHub keeps Wiki history in a separate Git repository. Merging this PR does
not publish pages there. A maintainer can copy the reviewed `docs/wiki/*.md`
files to the root of the BaryonOS Wiki checkout, preserving filenames, review
the diff and publish that Wiki commit separately. Do not copy this guide or
canonical specifications into the Wiki. This change adds no automatic publisher.

Page-to-page links use the final GitHub Wiki URLs so they work after publication.
The list above provides repository previews before publication. Repository links
use `main` so readers reach active contracts; evidence remains version-bound in
the repository's verification reports. When updating a technical explanation,
check its linked source and current project state rather than treating an older
Wiki revision as authority. Keep all fourteen pages in English.
