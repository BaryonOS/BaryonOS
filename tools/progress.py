#!/usr/bin/env python3
"""Render the fixed 100-point OS roadmap; check by default, never run OS tests."""
from __future__ import annotations

import argparse
import datetime as dt
import html
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "docs/progress.json"
README_MARKERS = ("<!-- OS-PROGRESS:BEGIN -->", "<!-- OS-PROGRESS:END -->")
RUBRIC_MARKERS = ("<!-- OS-RUBRIC:BEGIN -->", "<!-- OS-RUBRIC:END -->")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def integer(value: object, low: int, high: int) -> bool:
    return type(value) is int and low <= value <= high


def label(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and not any(c in value for c in "\n\r|<>")


def load_progress() -> dict:
    data = json.loads(DATA.read_text(encoding="utf-8"))
    require(data.get("schema_version") == 1, "Onbekende progress.json-schemaversie")
    require(integer(data.get("rubric_version"), 1, 1000), "Ongeldige rubricversie")
    assessment = data["assessment"]
    require(dt.date.fromisoformat(assessment["date"]).isoformat() == assessment["date"], "Datum moet YYYY-MM-DD zijn")
    require(re.fullmatch(r"[0-9a-f]{40}", assessment["source_commit"]) is not None, "Volledige broncommit ontbreekt")
    for key in ("branch", "task", "summary"):
        require(label(assessment[key]), f"Ongeldig assessment-veld: {key}")
    require(assessment["previous_score"] is None or integer(assessment["previous_score"], 0, 100), "Ongeldige vorige score")
    tracks = data["tracks"]
    require(isinstance(tracks, list) and bool(tracks), "Geen rubricsporen")
    identifiers: set[str] = set()
    total = 0
    for track in tracks:
        require(label(track["title"]), "Ongeldige spoortitel")
        require(re.fullmatch(r"[A-Z]", track["id"]) is not None, "Ongeldig spoor-ID")
        require(track["id"] not in identifiers, "Dubbel spoor-ID")
        identifiers.add(track["id"])
        require(integer(track["points"], 1, 100), "Ongeldig spoorgewicht")
        subtotal = 0
        for gate in track["gates"]:
            require(re.fullmatch(track["id"] + r"[0-9]{2}", gate["id"]) is not None, "Ongeldig criterium-ID")
            require(gate["id"] not in identifiers, "Dubbel criterium-ID")
            identifiers.add(gate["id"])
            require(integer(gate["points"], 1, 100), "Ongeldig criteriumgewicht")
            require(label(gate["title"]) and label(gate["acceptance"]), "Ongeldige criteriumtekst")
            require(gate["status"] in ("open", "accepted"), "Status moet open of accepted zijn")
            evidence = gate["evidence"]
            require(isinstance(evidence, list), "Bewijs moet een lijst zijn")
            require(gate["status"] != "accepted" or bool(evidence), f"{gate['id']}: geen punten zonder bewijsverwijzing")
            for path in evidence:
                require(isinstance(path, str) and re.fullmatch(r"[A-Za-z0-9_./-]+", path) is not None, "Bewijs moet een lokaal repositorypad zijn")
                target = (ROOT / path).resolve()
                require(target.is_relative_to(ROOT) and target.is_file(), f"Ontbrekend/ongeldig bewijsbestand: {path}")
            subtotal += gate["points"]
        require(subtotal == track["points"], f"{track['id']}: criteriagewichten tellen niet op tot spoorgewicht")
        total += subtotal
    require(total == 100, f"Rubric telt op tot {total}; vereist exact 100")
    return data


def score(track: dict) -> int:
    return sum(g["points"] for g in track["gates"] if g["status"] == "accepted")


def total_score(data: dict) -> int:
    return sum(score(track) for track in data["tracks"])


def assessment_delta(data: dict) -> str:
    previous = data["assessment"]["previous_score"]
    current = total_score(data)
    if previous is None:
        return f"Eerste meting: **{current}%**."
    delta = current - previous
    return f"Deze opdracht: **{previous}% → {current}% ({delta:+d} procentpunt)**."


def readme_block(data: dict) -> str:
    current = total_score(data)
    a = data["assessment"]
    rows = "\n".join(f"| {t['title']} | {t['points']} | {score(t)} |" for t in data["tracks"])
    return f"""{README_MARKERS[0]}
## Voortgang naar een volwaardig zelfstandig OS

![Nieuw OS: {current}% van de vaste productroadmap](docs/assets/progress.svg)

**{current}% — {current} van 100 gewogen acceptatiepunten.** Dit is een
planningsmeter voor aantoonbaar werkende functies van het eigen OS, geen
percentage verstreken ontwikkeltijd, veiligheidswaarborg of gemeten gelijkheid
met Windows, macOS of Linux. De eigen kernel blijft het fundament; geen van
die besturingssystemen is de basis.

**0%:** nog geen criterium gerealiseerd. **100%:** een volwaardig, dagelijks
bruikbaar en onderhoudbaar zelfstandig OS op gedocumenteerde ondersteunde
hardware, met native Materiaal, nuttige apps, internet, veilige opslag,
updates, herstel en de afgesproken privacyfuncties. Een bootende demo of
alleen een nette desktop haalt die eindgrens niet.

| Productspoor | Totaal punten | Behaald |
|---|---:|---:|
{rows}
| **Totaal** | **100** | **{current}** |

**Herbeoordeeld:** {a['date']} · **Rubric:** v{data['rubric_version']} ·
**Bron:** [`{a['source_commit'][:8]}`](https://github.com/simonvanieperen/nieuw-os/commit/{a['source_commit']}) ·
**Ontwikkelbranch:** `{a['branch']}`.
{assessment_delta(data)} **Opdracht:** {a['task']}.
{a['summary']}

De meter geldt voor de beoordeelde ontwikkelstand; opname in `main` en een
productrelease zijn afzonderlijke gebeurtenissen. Hostprototypes van Materiaal,
NAS/TLS en qsim staan zichtbaar in de projectstatus, maar krijgen geen native
productpunten. Elk criterium telt volledig of niet; onbewezen werk blijft open.

[Criteria, bewijs en berekening](docs/PROGRESS.md) ·
[Volledige roadmap](docs/ROADMAP.md) ·
[Volledige releaseopdracht en acceptatie](docs/FULL-TEST-RELEASE.md) ·
[Actuele uitvoering](docs/PROJECT-STATE.md).
Bij **iedere opdracht** worden deze meter, datum, bewijs en wijziging opnieuw
beoordeeld; de score mag gelijk blijven of dalen. Instructie: [AGENTS.md](AGENTS.md).
{README_MARKERS[1]}"""


def rubric_block(data: dict) -> str:
    chunks = [RUBRIC_MARKERS[0]]
    for track in data["tracks"]:
        chunks.append(f"### {track['id']} — {track['title']} ({score(track)}/{track['points']})\n")
        chunks.append("| Criterium | Punten | Status | Acceptatie en bewijs |\n|---|---:|---|---|")
        for gate in track["gates"]:
            state = "Behaald" if gate["status"] == "accepted" else "Open"
            links = " · ".join(f"[bewijs {i + 1}](../{path})" for i, path in enumerate(gate["evidence"]))
            acceptance = gate["acceptance"] + (" " + links if links else "")
            chunks.append(f"| **{gate['id']} — {gate['title']}** | {gate['points']} | {state} | {acceptance} |")
        chunks.append("")
    chunks.append(RUBRIC_MARKERS[1])
    return "\n".join(chunks)


def svg(data: dict) -> str:
    current = total_score(data)
    date = html.escape(data["assessment"]["date"])
    width = 828 * current / 100
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 226" role="img" aria-labelledby="title description">
  <title id="title">Nieuw OS: {current}% van de vaste productroadmap</title>
  <desc id="description">{current} van 100 gewogen acceptatiepunten. Nul is niet begonnen; honderd is een volwaardig zelfstandig OS op ondersteunde hardware. Beoordeeld op {date}. Geen tijds- of veiligheidsscore.</desc>
  <rect width="960" height="226" rx="16" fill="#f4f1eb"/>
  <text x="48" y="40" fill="#45504c" font-family="system-ui,sans-serif" font-size="14" font-weight="600" letter-spacing="2">NIEUW OS · EIGEN KERNEL · EIGEN PRODUCT</text>
  <text x="48" y="104" fill="#173e34" font-family="system-ui,sans-serif" font-size="56" font-weight="700">{current}%</text>
  <text x="238" y="78" fill="#173e34" font-family="system-ui,sans-serif" font-size="21" font-weight="600">Van eigen kern naar dagelijks bruikbaar OS</text>
  <text x="238" y="105" fill="#45504c" font-family="system-ui,sans-serif" font-size="15">{current} / 100 acceptatiepunten · herbeoordeeld {date}</text>
  <rect x="66" y="136" width="828" height="19" rx="9.5" fill="#d5ddd5"/>
  <rect x="66" y="136" width="{width:.2f}" height="19" rx="9.5" fill="#207459"/>
  <g fill="#45504c" font-family="system-ui,sans-serif" font-size="13" text-anchor="middle">
    <text x="66" y="177">0%</text><text x="273" y="177">25%</text><text x="480" y="177">50%</text><text x="687" y="177">75%</text><text x="894" y="177">100%</text>
  </g>
  <text x="48" y="207" fill="#45504c" font-family="system-ui,sans-serif" font-size="13">Niet begonnen</text>
  <text x="912" y="207" fill="#45504c" font-family="system-ui,sans-serif" font-size="13" text-anchor="end">Volwaardig zelfstandig OS</text>
</svg>
'''


def replace_block(path: Path, markers: tuple[str, str], block: str) -> str:
    text = path.read_text(encoding="utf-8")
    begin, end = markers
    require(text.count(begin) == 1 and text.count(end) == 1, f"{path.name}: verwacht precies één markerpaar")
    start = text.index(begin)
    stop = text.index(end)
    require(start < stop, f"{path.name}: omgekeerde markers")
    return text[:start] + block + text[stop + len(end):]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="Controleer rubric, bewijsverwijzingen en gegenereerde weergaven (standaard)")
    mode.add_argument("--write", action="store_true", help="Werk README, criteriatabel en lokale SVG bij")
    args = parser.parse_args()
    try:
        data = load_progress()
        outputs = {
            ROOT / "README.md": replace_block(ROOT / "README.md", README_MARKERS, readme_block(data)),
            ROOT / "docs/PROGRESS.md": replace_block(ROOT / "docs/PROGRESS.md", RUBRIC_MARKERS, rubric_block(data)),
            ROOT / "docs/assets/progress.svg": svg(data),
        }
        stale = [path for path, content in outputs.items() if not path.is_file() or path.read_text(encoding="utf-8") != content]
        if args.write:
            for path in stale:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(outputs[path], encoding="utf-8")
        elif stale:
            print("Verouderde voortgangsweergave: " + ", ".join(str(p.relative_to(ROOT)) for p in stale), file=sys.stderr)
            print("Beoordeel docs/progress.json en voer tools/progress.py --write uit.", file=sys.stderr)
            return 1
        print(f"Nieuw OS: {total_score(data)}/100 punten ({total_score(data)}%); rubric v{data['rubric_version']}; {'bijgewerkt' if args.write else 'consistent'}.")
        print("Dit controleert berekening, verwijzingen en weergave; geen nieuwe native tests of beoordeling van bewijsinhoud.")
        return 0
    except (ValueError, KeyError, TypeError, OSError) as exc:
        print(f"Voortgangscontrole afgewezen: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
