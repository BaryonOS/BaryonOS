# Native M6b-foutafhandeling — bewijs en grenzen

28 oktober 2026. Basis `1f24f99`, runtime-/harnascommit `592dccf`, branch
`codex/m6b-storage-errors`. Latere commits bevatten uitsluitend documentatie en
bewijs. Exacte commando's/exits en bronvingerafdruk: `validation.json`.

- `../m6b-storage-errors/`: volledige gate, 22 groepen / 18 echte VM-runs en
  opnieuw gecontroleerde capture. Hostlaag: 22 Rust- en 31 Python-tests.
- `verification.json`: onafhankelijke AI-audit van alle 18 VM-runs, 157
  artifacthashes, ruwe uitvoer, journalbytes en acht koude herstelkoppelingen.
- `review-report.json`: read-only bronreview en afzonderlijke hostproeven;
  de genoemde 30 tests zijn een eerdere scope vóór de extra archiefpadtest.
- `directed-runs.json`: twee voorafgaande foutVMs, niet bij de gate opgeteld.
- `final-smoke.json` en `final-build-manifest.json`: extra normale M6b-start;
  beide binaries gelijk aan de normale gatebinaries. Lokale build is M6b.
- `m6b-errors-final-*.json` en bijbehorende uitvoer: exacte capture-, format-,
  diff-, build- en smoke-opdrachten; allemaal exit 0.

De nieuw geteste fouten vervangen één lokale IOERR-statussnapshot na echte
completion. De write kan dus al uitgevoerd zijn terwijl de levende dienst
Unknown meldt, niets bevestigt/herhaalt en geen oude inhoud toont. Het externe
orakel en de aparte audit vinden precies record 1 op object 0; beide koude
herstarts lezen dat record zonder te schrijven.

Geen nieuwe echte backendfout, gedeeltelijke write, timeout/reset-timeout of
fysieke stroomuitvalproef. De eigen poll-LIMIT van de serviceadapter zonder
terminale brokerfout blijft een afzonderlijk observerrandgeval. Geen nieuwe
volledige M1–M6a-regressies; de M6b-gate bevat één gewone M1-regressiestart.
Geen nieuwe rechten/dependencies/unsafe/schijfformaat. Dit is geen menselijke
audit, productieclassificatie of IOMMU-/fysieke-hardwarebewijs.

Historisch bewijs onder `../m6b/` is behouden. De voorlopige dubbele
M6a-coreplanbranch uit de oude lokale checkout is niet in deze wijziging
geïntegreerd. De uiteindelijke wijziging bouwt op de actuele M6b-mainbasis.
