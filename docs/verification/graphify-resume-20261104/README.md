# Graphify hervatten zonder bestaande chunks te verliezen

**Status 2026-11-04:** uitsluitend een herstelhulpmiddel; geen volledige code-/documentgraaf of OS-acceptatie.

## Herstelpunt uit de Claude-sessie

- Lokale map: `/home/simon/projects/nieuw-os/graphify-out` (niet uit deze connector uit te lezen).
- Eerder gerapporteerd: AST-extractie 11.674 nodes en 31.175 kanten.
- Gepland: 20 tekstchunks en 35 afbeeldingschunks.
- Laatste gebruikerscontrole: 14 `.graphify_chunk_*.json` bestanden aanwezig (hoogste gemelde suffix `_18`); definitieve bestanden ontbraken. Deze status is niet opnieuw gemeten.
- Eindproducten: `graph.json`, `graph.html` en `GRAPH_REPORT.md` zijn niet door dit hulpmiddel gegenereerd.

## Veilig lokaal uitvoeren (WSL)

```bash
cd /home/simon/projects/nieuw-os
git fetch origin
# Het script staat op de branch van deze PR: eerst die branch (of main na de merge) uitchecken.
git switch tools/graphify-resume-checkpoints-20261104   # of: git switch main
python3 tools/graphify_resume.py --output-dir graphify-out --expected-text 20 --expected-images 35
python3 tools/graphify_resume.py --output-dir graphify-out --write-manifest graphify-out/resume-manifest-20261104.json
```

Een manifest wordt alleen aangemaakt als het nog niet bestaat. De extractor-chunks worden nooit aangepast of verwijderd. Als de AST-bestandsnaam bekend is, geef die op met `--ast PATH`; een ontbrekend AST-bestand is dan een fout. Bijnaamgeving van afbeeldingschunks is onbekend: de telling is slechts een eerste bestandsinventaris, geen verificatie van de volledige beeldanalyse.

Test lokaal met `python3 -m unittest discover -s tests -p test_graphify_resume.py -v`.

## Grenzen en vervolg

1. Controleer de getoonde ontbrekende indices, dubbele nummers en JSON-syntaxis; bekijk per chunk de gerapporteerde top-level keys en SHA-256 in het optionele manifest.
2. Lokaliseer de voltooide AST-data en verifieer het werkelijke node/edge-schema; maak nog geen nieuwe volledige scan en beschouw tekst- en afbeeldingschunks niet als onderling uitwisselbaar.
3. Maak pas na volledige schemacontrole en integriteitscontrole een afzonderlijke merge-/cluster-/HTML-generator. Zonder die controle mogen bestaande bestanden niet met een gegokt schema worden samengevoegd.
4. De lokale Graphify-output is niet automatisch GitHub-data. `graphify-out/` is genegeerd om onbedoelde publicatie van corpusinhoud, afbeeldingen of analyse te voorkomen.

**Bewijsgrens:** vijf geïsoleerde Python-tests voor een read-only inventarisatie slaagden lokaal bij voorbereiding van deze PR. Geen test op Simons daadwerkelijke 14 chunks, geen Graphify-workspace-query, geen native QEMU-test; productscore blijft 41/100.
