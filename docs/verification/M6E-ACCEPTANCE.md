# M6e — acceptatie S04 en S05 (schaalbare documentopslag)

Datum: 28 oktober 2026 (Europe/Amsterdam). Contract:
[M6E-SCALABLE-VOLUME](../implementation/M6E-SCALABLE-VOLUME.md); besluit ADR-040.

## Bronbinding

- Branch `ccr-d3fe1610-gl4r8s`, commit `e30e10d` (twee CodeRabbit-reviewrondes:
  pogingdigest in `bac9cbb`, gateverdict na de afsluitende build in `e30e10d`;
  bevat `main` `78a96ae`).
- Bronvingerafdruk v3: `63c56bc0bb4c41c3…` (volledig in `m6e/volume-gate.json`).
  Eerder bewijs van bron `535d672e…` (vóór de reviewcorrecties) en `24dbc374…`
  (op `bac9cbb`) is vervangen; het staat in de Git-geschiedenis (`8ae9194`,
  `1037ee3`). De runs op `bac9cbb` en `e30e10d` gaven byte-identieke images en
  ELF-hashes; tussen die twee bronnen verschilde alleen `tools/test_volume.py`.
- Kernel-ELF (normal) SHA-256 `5ff76ae42ba7bd51…`; `volume-service` `d799ff90f05669a0…`;
  `volume-app` `1924a6b533bfd5ba…` (volledig per stap in `result.json`).
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket),
  `pc-q35-8.2`, TCG, `qemu64`, 1 CPU, 256 MiB, OVMF 4M uit het Ubuntu-pakket.
  Dit is een ander hostprofiel dan de historische Windows- en Debian-routes.

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2
python3 -B -u tools/test_volume.py     # exit 0
python3 -B tools/capture_volume.py     # exit 0 -> docs/verification/m6e/
```

De gate voert eerst drie hostcontroles uit (alle exit 0):
`cargo +1.90.0 test --locked -p volume-core` (17 tests: 14 dienst, 3 crash),
`cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime`
en `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` (8 orakeltests).
Daarna volgen zeven echte QEMU-invocaties:

| # | Stap | Image vóór → na | Uitkomst | Raw exit | Duur |
|---|---|---|---|---|---|
| 0 | normal (schrijvend) | nieuw `b8f331f4` → `b3c109a1` | geslaagd | 33 | 86,6 s |
| 1 | recover (read-only, koud) | `b3c109a1` → `b3c109a1` | geslaagd, image byte-identiek | 33 | 48,6 s |
| 2 | cut-data: kill na 40 inhoudswrites van de 2e upload | nieuw `b8f331f4` → `0eb98c00` | echte hostkill van live guest | −9 | 4,1 s |
| 3 | recover-cut (read-only, koud) | `0eb98c00` → `0eb98c00` | revisie 1 intact | 33 | 4,3 s |
| 4 | cut-table: kill na 4 van 9 tabelsectoren | nieuw `b8f331f4` → `a59951f7` | echte hostkill van live guest | −9 | 4,9 s |
| 5 | recover-cut (read-only, koud) | `a59951f7` → `a59951f7` | revisie 1 intact | 33 | 4,2 s |
| 6 | corrupt (verzegelde, niet-canonieke tabel) | fixture `c8f750b8` → `c8f750b8` | geweigerd vóór READY | 33 | 3,7 s |

Iedere herstelstap is gekoppeld aan de vorige: before-hash = after-hash van de
voorgaande geslaagde stap, dezelfde volume-identiteit en dezelfde bron.

## Waarnemingen (normal, guest- en brokertellers)

- 71 commits (de oude M6c-limiet was 32 records), generatie 71.
- 296 chunk-allocaties op een volume van 64 chunks: ruimte is ruim viermaal hergebruikt.
  Het orakel ziet object 12 ("Groot 2") op chunks 0–15. Die kwamen vrij door het
  verwijderen van object 0, waarna domein B ze hergebruikte.
- Documenten tot 65 536 bytes, gestreamd in stukken van 160 bytes. Alle inhoud is na
  iedere write volledig teruggelezen en byte-voor-byte gecontroleerd (A: 50 documentreads,
  817 639 bytes; B: 28 reads, 356 962 bytes).
- Quota: domein A kreeg `Quota` bij chunk 41 zonder I/O. Volume vol: domein B kreeg
  `Full` bij 62 van 64 bezette chunks, terwijl het onder zijn eigen quota bleef. Na de
  delete van A slaagde dezelfde upload.
- Deelwrites: een afgebroken upload, een verlaten sessie die door een nieuwe BEGIN is
  vervangen, en een verouderde APPEND (`Invalid`) laten geen objecten achter.
- Grenzen: `Access` voor objecten en LIST van het andere domein, `Range` voor
  object 16, `Invalid` voor een v4-body en voor READ-count 161; 27 afgewezen verzoeken
  in totaal, 0 `Unknown`.
- Broker: 2 746 writes (639 tabelsectoren = 71 × 9, plus 2 107 inhoudssectoren),
  140 flushes, publicaties = completions. Apps hebben 6 geweigerde blokcalls
  (eigen endpointhandle en vervalste handle).
- Cleanup: framebaseline exact terug, capability-engine leeg, DMA-pagina's
  kernel-owned, en een reset-ACK.

## Onafhankelijke orakelcontrole

`tools/volume_disk.py` is een aparte Python-implementatie van het formaat. Per
stap:
- de sectoren buiten de scope (canaries) zijn onaangeroerd;
- de keuze tussen de tabelkopieën volgt dezelfde regels;
- iedere chunkchecksum en padding klopt;
- de inhoud is gelijk aan een geschreven fixture-revisie;
- de domeindigests zijn gelijk aan de door de guest gerapporteerde digests.

Na cut-table bevatte kopie 0 vier sectoren van generatie 2 en vijf nulsectoren.
Zowel het orakel als de dienst kozen generatie 1. De corrupte fixture wordt ook door
het orakel geweigerd.

Na de reviewcorrectie (pogingdigest) weigeren dienst én orakel sectoren van twee
verschillende schrijfpogingen voor dezelfde generatie als één geldige kopie
(`mixed_attempts_for_one_generation_are_never_combined` en
`test_sectors_of_two_attempts_never_form_one_copy`; de hosttest faalt zonder de
correctie). Native is die menging niet als aparte VM-stap opgewekt.

## Beoordeling volgens rubric v1

- **S04 (3 punten), geaccepteerd.**
  - Verwijderde en overschreven ruimte wordt veilig hergebruikt en het volume werkt
    voorbij 32 records (71 commits).
  - De crashconsistente vrijgave (ADR-040: copy-on-write vervult de rol van compactie)
    is getest: uitputtend op de host en met twee echte VM-kills plus koude recovery.
  - Quota is native getest.
  - Grens: het is een proefgeometrie van 256 KiB inhoud, geen grote schijf.
- **S05 (3 punten), geaccepteerd.**
  - Documenten van 64 KiB overschrijden de 128-bytegrens ruim en worden gestreamd.
  - Buffers zijn begrensd: 160 B IPC, 512 B sector, 4 KiB cache, geen heap.
  - Volle volumes zijn getest (`Full`/`Quota`), en deelwrites ook: abort, verlaten
    sessie, kill midden in een upload, en zekere I/O-fouten in de hosttests.

Score: **20 → 26**.

## Niet bewezen / open

Niet bewezen: fysieke schijven of stroomuitval (een emulator-kill is iets anders),
authenticatie of encryptie (B02), back-up/export (S09), collecties en historie (S06),
grotere volumes en migratie van M6c-volumes. Native I/O-foutinjectie voor dit profiel
is niet uitgevoerd; zekere en onzekere I/O-fouten zijn alleen hostgetest. S07/S08
blijven gekoppeld aan de M6c-keten.

GitHub Actions op de PR-head moeten deze uitslag nog onafhankelijk herhalen. Zie
PR #47. `tools/capture_volume.py` weigert een bestaande doelmap; voor een nieuwe
vastlegging wordt de oude map eerst bewust verwijderd (`git rm -r
docs/verification/m6e`) of wordt `--destination` met een nieuwe map gebruikt.
