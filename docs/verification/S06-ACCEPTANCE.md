# S06 — acceptatie collecties, metadata, revisiegeschiedenis en transacties

Datum: 29 oktober 2026 (Europe/Amsterdam). Contract:
[S06-COLLECTIONS-HISTORY](../implementation/S06-COLLECTIONS-HISTORY.md);
besluit ADR-041. Bewijsmap: [`s06/`](s06/).

## Bronbinding

- Branch `ccr-d3fe1610-gl4r8s`. S06 staat als `71a0f9f` direct op `main`
  `092ee52` (de squash van #47). De gate en de mutatiecontrole draaiden op
  `ce181bf`, in een schone kloon. Die commit bevat de correcties uit de tweede
  review: tests en docs, plus een afgebroken docregel in `wire.rs`.
- Bronvingerafdruk v3: `528dfe4c0535a0f6…` (een SHA-256 over de broninputs,
  geen Git-tree). De hosttestinputs onder `tests/` vallen buiten die
  vingerafdruk. Hun SHA-256 is daarom apart vastgelegd en vóór het verdict
  opnieuw gecontroleerd:
  - `tests/test_volume_disk.py`: `c612c63f…`;
  - `tests/test_volume_runner.py`: `adcc2676…`.

  Alles staat in `s06/volume-gate.json`.
- SHA-256 van de binaries (normal):
  - kernel-ELF: `c494c13ae9104535…`;
  - `volume-service`: `8b3c33ee4e155bde…`;
  - `volume-app`: `b2bbc890b7bcf3e5…`.

  De volledige hashes staan per stap in `result.json`. Ze verschillen van de
  vorige run op `cb20c27`, maar de guest-uitvoer (`serial.txt`) en alle
  image-hashes zijn byte-gelijk.
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket
  `1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF 4M uit het
  Ubuntu-pakket.

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2 CARGO_BUILD_JOBS=2
python3 -B -u tools/test_volume.py                                # exit 0
python3 -B -u tools/mutate_volume.py                              # exit 0
python3 -B tools/capture_volume.py --destination docs/verification/s06-ce181bf   # exit 0
```

De capture is daarna als `s06/` vastgelegd. Het mutatierapport
(`build/volume-mutants.json`) is ongewijzigd gekopieerd naar
`s06/volume-mutants.json`, en de SHA-256 ervan is aan `SHA256SUMS`
toegevoegd. `sha256sum -c SHA256SUMS` geeft exit 0.

Eerst draaien drie hostcontroles, alle met exit 0. De testtotalen staan
letterlijk in `volume-gate.json`.

| Hostcontrole | Tests |
|---|---|
| `cargo +1.90.0 test --locked -p volume-core` | 28: 4 crash, 8 historie, 16 dienst |
| `cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 14 |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 18: 15 orakel, 3 runner |

Daarna volgen negen echte QEMU-invocaties op dezelfde bevroren bron:

| # | Stap | Image vóór → na | Uitkomst | Raw exit | Duur |
|---|---|---|---|---|---|
| 0 | normal (schrijvend) | nieuw `49c40c31` → `19e2fdb3` | geslaagd | 33 | 73,7 s |
| 1 | recover (read-only, koud) | `19e2fdb3` → `19e2fdb3` | geslaagd; image byte-identiek | 33 | 41,1 s |
| 2 | cut-data: kill na 40 inhoudswrites | nieuw `49c40c31` → `48dcfcdd` | echte hostkill van de live guest | −9 | 3,7 s |
| 3 | recover-cut (read-only, koud) | `48dcfcdd` → `48dcfcdd` | revisie 1 intact; andere kopie onaangeroerd | 33 | 4,0 s |
| 4 | cut-table: kill in de tabelwrite | nieuw `49c40c31` → `2d37c7c1` | echte hostkill; 4 van 34 sectoren van g+1 | −9 | 4,2 s |
| 5 | recover-cut (read-only, koud) | `2d37c7c1` → `2d37c7c1` | revisie 1 intact | 33 | 4,3 s |
| 6 | cut-txn: kill in de tabelwrite van een transactie over twee objecten | nieuw `49c40c31` → `ede3a54c` | echte hostkill; 5 van 34 sectoren van g+1 | −9 | 3,6 s |
| 7 | recover-cut (read-only, koud) | `ede3a54c` → `ede3a54c` | generatie 3 volledig vóór de transactie | 33 | 3,8 s |
| 8 | corrupt (verzegelde, niet-canonieke tabel) | fixture `a19398f1` → `a19398f1` | geweigerd vóór READY | 33 | 3,6 s |

Elke herstelstap is aan de vorige gekoppeld: de before-hash is de after-hash
van de voorgaande geslaagde stap, met dezelfde volume-identiteit en dezelfde
bron.

Bij de drie cut-stappen bevat de andere tabelkopie wat de runner eist. Na
cut-table en cut-txn is dat een echte gescheurde poging: een niet-lege,
onvolledige prefix van sectoren met generatie g+1, gevolgd door de vorige
kopie. Na cut-data is de andere kopie onaangeroerd. Het orakel leest die
sectorstatus rechtstreeks uit de image (`oracle.slots`, `oracle.torn_sectors`).

## Waarnemingen (normal, guest-, broker- en orakeltellers)

- 84 commits (generatie 84). De domeinklokken zijn A = 57 en B = 27; hun som
  is gelijk aan de generatie. 296 chunk-allocaties op 64 chunks; na afloop
  zijn 62 chunks in gebruik, behouden historie meegeteld.
- Collecties: A heeft 'Werk' en 'Privé', B heeft 'Fotos'. De guest leest ze
  ook terug met COLLECTION_STAT (naam, aanwezigheid, revisie, request-ID), en
  dat gebeurt ook in de koude read-only recovery.
- Eén transactie verplaatst twee documenten in één commit. Een transactie met
  één verouderde voorwaarde verandert niets (`Stale`). Een niet-lege collectie
  verwijderen geeft `Busy`; META naar een ontbrekende collectie geeft `Range`.
- Historie: twee eerdere versies zijn leesbaar. RESTORE van selector 2 maakt
  een nieuwe revisie met inhoud 38. Een verwijderd document wordt teruggezet;
  de teruggezette versie verlaat daarbij de historie, zodat niets dubbel staat.
  `Quota` treedt op door behouden historie; PURGE geeft die ruimte vrij.
  Er zijn 2 duurzame restores en 1 transactie.
- De guest-digests van beide domeinen, inclusief collectielidmaatschap, zijn
  gelijk aan die van het onafhankelijke Python-orakel. Ook de eindtoestand per
  object (revisie, inhoudsrevisie, lengte, naam, collectie, historie) komt
  overeen.
- De koude recovery verifieert de volledige eindtoestand en weigert iedere
  mutatie met `Access`, ook RESTORE en TRANSACT. De image blijft byte-identiek.
- Broker: 47 672 reads, 4 963 writes, 154 flushes. Dat aantal flushes is gelijk
  aan de servicetelling, inclusief de flush na schrijfbare recovery. De apps
  deden 6 blokcalls zonder blokrecht; alle zes zijn geweigerd.

## Reviewcorrecties vóór acceptatie

Een onafhankelijke review (drie AI-reviewers) leverde vijf inhoudelijke punten
op. Alle vijf zijn opgelost in `590e27e`, elk met een regressietest. Dat die
tests de correctie echt bewaken, is pas sinds `ce181bf` vastgelegd bewijs: zie
*Mutatiecontrole* hieronder.

| # | Bevinding | Correctie | Test |
|---|---|---|---|
| 1 | STAT gaf de globale generatie; een domein kon de commits van het andere tellen | klok per domein in de header; stempels uit de eigen klok | `stamps_do_not_reveal_other_domain_commits`, `table_sector_layout_is_canonical`, orakeltest klokken |
| 2 | collecties waren via IPC alleen schrijfbaar | COLLECTION_STAT (op 15), ook read-only | `collection_state_is_readable_and_resolves_unknown` |
| 3 | RESTORE van de nieuwste versie bij volle historie wiste de oudste | teruggezette versie verlaat de historie | `restoring_the_newest_history_version_keeps_the_oldest` |
| 4 | een nieuwe instantie kon schrijven terwijl de gekozen kopie alleen in de cache stond | flush na schrijfbare recovery, anders niet starten | `recovery_makes_the_chosen_copy_durable_before_writing` |
| 5 | de gate kon een kill vóór de tabelwrite niet onderscheiden van een gescheurde tabelwrite | `torn_sectors` in de runner | `tests/test_volume_runner.py` |

Kleinere punten:
- strengere clientcontrole van STAT/LIST (`client_rejects_inconsistent_stat_and_list_replies`);
- de read-only recovery controleert ook RESTORE en TRANSACT;
- een misleidend stackcommentaar is gecorrigeerd.

### Tweede review (van de correcties)

Vijf AI-reviewers keken naar `590e27e`/`cb20c27`, elk op één dimensie. Twee
konden hun werk afmaken (klokken; COLLECTION_STAT en clientvalidatie); de
andere drie stopten door een gebruikslimiet, net als de verificatiestap. De
twee vonden geen fout in de productiecode, maar wel zwakke plekken in tests en
docs. Alles is gecorrigeerd in `ce181bf`:

| Bevinding | Correctie |
|---|---|
| de test op ongerelateerde generaties werd vacuüm: de vervalste kopie faalde al op de kloksom | geldige klokken, en de kopie decodeert op zichzelf |
| de LIST-cursortest bereikte de overloop niet; twee STAT-gevallen werden door de ordeningsregel gevangen | ieder geval breekt precies één regel |
| COLLECTION_STAT had geen negatieve tests | `client_rejects_forged_collection_stat_replies` en vijf misvormde verzoeken in `malformed_s06_requests_are_rejected` |
| de revisie-, modified- en collectieklokgrens waren niet los getest | losse gevallen in Rust en in het orakel |
| een read-only antwoord op een `Unknown` commit werd als definitief gepresenteerd | de test toont dat het na stroomverlies kan verdwijnen; een schrijfbare instantie maakt het duurzaam; invariant 6 aangepast |
| invariant 7 claimde volledige non-interferentie | `Full` (gedeelde, overboekte ruimte) en de instantiebrede `Unknown` staan nu als restrisico in invariant 7 en ADR-041 |

De resterende reviewdimensies (RESTORE/flush, gate-bewijs, merge/docs) en een
controle van `ce181bf` lopen in een derde ronde. De uitkomst staat in
PROJECT-STATE.md.

### Mutatiecontrole

`tools/mutate_volume.py` kopieert de bron naar een tijdelijke map. Per mutant
schakelt het daar precies één regel uit, en eist dan twee dingen: de kopie
bouwt nog, en ten minste één test faalt. Een mutant die niet bouwt, telt als
mislukte controle. De Rust-suite is `--lib`, `service` en `history`; de
crashtests vallen erbuiten, omdat ze 34 s per run kosten.

Op `ce181bf` (bronvingerafdruk `528dfe4c…`, dezelfde als die van de gate)
slaagden eerst beide basislijnen: Rust exit 0, Python exit 0. Daarna zijn alle
24 mutanten gevangen; het rapport geeft per mutant de falende tests.

| Groep | Mutanten | Gevangen door |
|---|---|---|
| tabel: ongerelateerde generaties, klokgrenzen (collectie, revisie, modified) | 4 | `corrupted_content_and_tables_fail_closed`, `table_sector_layout_is_canonical` |
| RESTORE houdt historie | 1 | `restoring_the_newest_history_version_keeps_the_oldest`, `history_restore_undelete_and_purge` |
| flush na schrijfbare recovery | 1 | `collection_state_is_readable_and_resolves_unknown` |
| client STAT/LIST | 3 | `client_rejects_inconsistent_stat_and_list_replies` |
| client COLLECTION_STAT | 6 | `client_rejects_forged_collection_stat_replies` |
| verzoek COLLECTION_STAT | 5 | `malformed_s06_requests_are_rejected` |
| orakel: ongerelateerde generaties, drie klokgrenzen | 4 | `test_unrelated_generations_are_refused`, `test_domain_clocks_add_up_and_bound_every_stamp` |

De eerste run op de werkkopie vond twee overlevers: de nieuwe Rust-gevallen
voor de revisie- en de modified-grens werden al door een andere regel
geweigerd (waarde boven de generatie). Die gevallen zijn gecorrigeerd vóór
`ce181bf`. De run hierboven is de herhaling op de commit.

Grens: de mutanten dekken de regels uit beide reviews, niet iedere regel van
volume v2. De crashtests vallen buiten de Rust-suite van deze controle; ze
zijn wel onderdeel van de gate.

## Niet bewezen en grenzen

- Dit is de opslaglaag. De Materiaal-interface draait nog niet native (G-/U-criteria).
- Geen fysieke schijven of stroomuitval; een emulator-kill is iets anders.
- Geen authenticatie of encryptie (FNV is geen MAC; B02 open).
- Proefbudget: 2 historische versies, 4 collecties per domein, maximaal
  4 objecten per transactie, 16 objecten, 256 KiB inhoud.
- Geen native I/O-foutinjectie voor dit profiel. De flushfout na recovery,
  het `Unknown`-pad en de gemengde schrijfpogingen zijn alleen hostgetest.
- Bekend restrisico (vastgepind in een test): één onleesbare sector in de
  nieuwste kopie geeft terugval naar de vorige generatie.
