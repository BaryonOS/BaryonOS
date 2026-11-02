# Actuele projectstatus

## NEXT AGENT HANDOFF — herstel NAS-tokenarchivering — 2 november 2026

**Bron:** branch-head `8736e06c5298ce9581531fb02a3b88deb5c82d3e` van de
concept-PR #125; implementatie-/reviewfixcommit `05f97cc92c638f2d94ddc4d4a00e73683ac3dfbc`.
De fixcommit voegt PR #120-head `562be4d13b9d039d7e8fece2f0fc26e767dccb15`
toe aan basis `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. PR #120 bevat de
implementaties voor #109 en #110 plus de overige componentdelta's; de volledige
broninventaris staat in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).

**Reviewherstel geïmplementeerd:** controleer de ruwe stdout en stderr vóór
archivering op het volledige paar en elk van beide 40-bytetokens, inclusief
hexweergave. `serial.txt` bewaart alleen allowlist-markers; stderr wordt
achtergehouden als de tokencontrole niet slaagt. Regressies testen beide
tokenhelften, het paar en stderr. De oude PR #120-head-checks zijn groen
(waaronder native NAS, run 41511329639), maar gaan vooraf aan deze fix en
bewijzen haar niet.

**Lokale verificatie:**
- `python3 -B -m unittest discover -s tests -p 'test_nas_runner.py' -v` —
  exit 0, 19 tests; log `/tmp/nieuw-os-nas-runner-tests.log`.
- `python3 -B -m unittest discover -s tests -p 'test_volume_runner.py' -v` —
  exit 0, 3 tests; log `/tmp/nieuw-os-volume-runner-tests.log`.
- `cargo +1.90.0 test --manifest-path sdk/examples/materiaal/Cargo.toml --locked --target-dir "$PWD/target"` —
  exit 127, niet gestart omdat `cargo` ontbreekt.

`cargo fmt` en de native NAS-gate zijn op deze nieuwe code niet uitgevoerd;
de detailregistratie met exacte commando's, exits en logpaden staat in
[nas-token-archive-fix.json](verification/nas-token-archive-fix.json).
PR #125 exacte-head CI moet de Rust- en native gates uitvoeren.

**Remote status:** alle geconfigureerde checks op exacte PR #125-head
`8736e06` zijn groen, inclusief `nas-native-normal-cold`, `native-system`,
host quality, formattering/linting en pakketverificatie. NAS-run
[41514371211](https://github.com/simonvanieperen/nieuw-os/actions/runs/41514371211)
is uitgevoerd op deze head. CodeRabbit is overgeslagen zolang de PR concept
blijft; er is nog geen review van PR #125.

**Issues:** #109 en #110 blijven open totdat PR #120 aantoonbaar met behoud
van inhoud is geïntegreerd. #39 mist de volledige native netwerk- en
negatievenmatrix; #40 de volledige geïsoleerde sessie-/compositoracceptatie;
#41 productie-sleutelrotatie/-ankers en een afzonderlijk geautoriseerde
ondertekenhandeling; #42 USB/HID/hotplug plus fysieke acceptatie; #43 de
uitgevoerde bewijsroute naar 50/100. Plannen, hosttests, componentproeven en
AI-review sluiten die acceptatiegaten niet.

**Voortgang:** 34/100 → 34/100. Geen rubriccriterium is nieuw geaccepteerd.

**Eerstvolgend:** commit en push deze assessmentupdate, markeer daarna PR #125
klaar voor review om CodeRabbit te starten en handel eventuele bevindingen af.
Merge de vervangende integratie pas na
groene checks en inhoudsvergelijking; sluit PR #120 en #109/#110 alleen
nadat dezelfde wijzigingen aantoonbaar in `main` staan.

## Vorige integratiehandoffs

Integratie 3 (#115–#119) is vastgelegd in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).
Integraties 1 en 2 staan in
[pr-integration-20261030.json](verification/pr-integration-20261030.json) en
[pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).
Oudere handoffs staan in het [archief van 30 oktober](history/integration-2026-10-30/README.md).
