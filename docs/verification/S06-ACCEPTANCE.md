# S06 — acceptatie collecties, metadata, revisiegeschiedenis en transacties

Datum: 29 oktober 2026 (Europe/Amsterdam). Contract:
[S06-COLLECTIONS-HISTORY](../implementation/S06-COLLECTIONS-HISTORY.md);
besluit ADR-041. Bewijsmap: [`s06/`](s06/).

## Bronbinding

- Branch `ccr-d3fe1610-gl4r8s`, broncommit `71a0f9f` direct op `main`
  `092ee52` (de squash van #47). De gate draaide op `cb20c27`, dat een identieke
  bronboom heeft. `cb20c27` was de lokale ontwikkelhead: de onafhankelijke
  S06-review verwerkt (`590e27e`) en M6e-inhoud van `main` hersteld die een
  eerdere lokale merge had teruggezet, waaronder de subpackage-binding van
  `d9e4b6b` in `tools/test_volume.py`.
- Bronvingerafdruk v3: `056445accfb8f26e…`. De SHA-256 van iedere testinput
  die `unittest discover` kan laden is daarnaast vastgelegd
  (`tests/test_volume_disk.py` `d2da21e4…` en `tests/test_volume_runner.py`
  `adcc2676…`). De gate controleert die vóór het verdict opnieuw; alles staat in
  `s06/volume-gate.json`.
- Kernel-ELF (normal): SHA-256 `6966269f364cea36…`; `volume-service`
  `e367a1d2d95ddb7c…`; `volume-app` `82387e69548366fa…`. De volledige hashes
  staan per stap in `result.json`.
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket
  `1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF 4M uit het
  Ubuntu-pakket.

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2 CARGO_BUILD_JOBS=2
python3 -B -u tools/test_volume.py                                # exit 0
python3 -B tools/capture_volume.py --destination docs/verification/s06   # exit 0
```

Eerst draaien drie hostcontroles, alle met exit 0. De testtotalen staan
letterlijk in `volume-gate.json`.

| Hostcontrole | Tests |
|---|---|
| `cargo +1.90.0 test --locked -p volume-core` | 27: 4 crash, 8 historie, 15 dienst |
| `cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 14 |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 18: 15 orakel, 3 runner |

Daarna volgen negen echte QEMU-invocaties op dezelfde bevroren bron:

| # | Stap | Image vóór → na | Uitkomst | Raw exit | Duur |
|---|---|---|---|---|---|
| 0 | normal (schrijvend) | nieuw `49c40c31` → `19e2fdb3` | geslaagd | 33 | 74,1 s |
| 1 | recover (read-only, koud) | `19e2fdb3` → `19e2fdb3` | geslaagd; image byte-identiek | 33 | 39,7 s |
| 2 | cut-data: kill na 40 inhoudswrites | nieuw `49c40c31` → `48dcfcdd` | echte hostkill van de live guest | −9 | 3,8 s |
| 3 | recover-cut (read-only, koud) | `48dcfcdd` → `48dcfcdd` | revisie 1 intact; andere kopie onaangeroerd | 33 | 4,3 s |
| 4 | cut-table: kill in de tabelwrite | nieuw `49c40c31` → `2d37c7c1` | echte hostkill; 4 van 34 sectoren van g+1 | −9 | 4,4 s |
| 5 | recover-cut (read-only, koud) | `2d37c7c1` → `2d37c7c1` | revisie 1 intact | 33 | 4,3 s |
| 6 | cut-txn: kill in de tabelwrite van een transactie over twee objecten | nieuw `49c40c31` → `ede3a54c` | echte hostkill; 5 van 34 sectoren van g+1 | −9 | 3,4 s |
| 7 | recover-cut (read-only, koud) | `ede3a54c` → `ede3a54c` | generatie 3 volledig vóór de transactie | 33 | 3,9 s |
| 8 | corrupt (verzegelde, niet-canonieke tabel) | fixture `a19398f1` → `a19398f1` | geweigerd vóór READY | 33 | 3,4 s |

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
op. Alle vijf zijn opgelost in `590e27e`; elk heeft een regressietest die
zonder de correctie faalt (mutatiecontrole uitgevoerd):

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

Een tweede, onafhankelijke review van deze correcties loopt. De uitkomst staat
in PROJECT-STATE.md.

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
