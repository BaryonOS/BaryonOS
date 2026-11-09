# Issue #277 — documentatiecontrole

Basis: `a9b6d3c5ed2938b3740f9874a579f47f6d348ac5`,
branch `coderabbit/issue-277-english-wiki`.

- Gerichte Markdown-controle (tijdelijk `/tmp/check_wiki.py`): exit 0.
  Veertien Engelse pagina’s; 170 links, waarvan 112 bestaande canonieke
  repositorydoelen en 31 Wiki-links naar voorbereide pagina’s. Home bereikt
  alle vervolgpagina’s. De overige links zijn bronpreview- en issueverwijzingen.
  Dit controleert lokale doelen en namen, geen gepubliceerde Wiki of HTTP-status.
- `python3 tools/progress.py --write`: exit 0.
- `python3 tools/progress.py --check`: exit 0; 41 → 41/100.
- `git diff --check`: exit 0.
- Afzonderlijke AI-bronreview: geen concrete problemen gevonden; geen menselijke audit.

Geen Rust-, QEMU- of fysieke-hardwaretests: uitsluitend verklarende documentatie
met bestaande contracten als bron. Wiki-publicatie is niet uitgevoerd; de
reviewbare bronpagina’s en publicatieroute staan in `docs/WIKI.md`.
