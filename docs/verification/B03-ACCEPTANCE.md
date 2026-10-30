# B03 — acceptatie ondertekende updates en rollback

Datum: 30 oktober 2026 (Europe/Amsterdam). Contract:
[B03-SIGNED-UPDATES](../implementation/B03-SIGNED-UPDATES.md); besluit
ADR-044. Bewijsmap: [`b03/`](b03/).

## Geaccepteerde eigenschap

Criterium B03: *authentieke updates, onderbreking, downgrade-aanvallen en
herstel naar een geldige vorige versie zijn end-to-end getest.* Op het eigen
OS in het QEMU-referentieprofiel start de eigen UEFI-loader alleen een
kernelimage uit een slot waarvan het manifest met beide vastgepinde TEST-
ankers (Ed25519 én ML-DSA-65) verifieert en waarvan het image lengte en
SHA-256 uit dat manifest heeft; een firmwaregebonden versievloer weigert
oudere slots; een falend slot valt terug op het andere slot en twee falende
slots stoppen de boot. De ring-3 update-dienst verifieert een inboxbundel
onafhankelijk vóór één schrijfactie, installeert in het niet-draaiende slot
(image, FLUSH, manifest, FLUSH, header, FLUSH), overleeft een echte kill
midden in de installatie zonder het actieve slot of de header te raken en
maakt de installatie bij de volgende boot af. Een onafhankelijk Python-
orakel controleert na iedere stage de bytes van de slotopslag.

## Bronbinding

- Branch `ccr-43832ef7-gi2qlz`, basis `main` `f63a0c1` (squash van PR #52,
  B01). De gate draaide op de werkboom die als PR-head is gecommit;
  bronvingerafdruk v3 `07994d47461a1dd5…` (`b03/update-gate.json`). De hosttestinputs
  `tests/test_update_*.py` zijn apart vastgelegd en vóór het verdict opnieuw
  gecontroleerd.
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket
  `1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF 4M,
  `NOS_QEMU_MACHINE=pc-q35-8.2`. Slotopslag: `virtio-blk-pci` op 00:05.0,
  6 MiB raw image (12 310 sectoren), `cache=writeback`; de OVMF_VARS-kopie
  (versievloer) deelt het leven van de store binnen een reeks.
- Drie kernelbuilds achter dezelfde `signed-boot`-loader: v1 en v2
  (`native-update`, `NOS_IMAGE_VERSION` 1/2) en v1 met de gelabelde
  crashfixture (`test-update-cut`). SHA-256 van loader, kernels, manifesten
  en de ring-3 dienst staan in `b03/update-gate.json` (`images`).
- Dependencies (nieuw, vastgepind in `Cargo.lock` en `sdk/Cargo.lock`): zie
  ADR-044 besluit 7 (`ed25519-dalek` 2.2.0, `curve25519-dalek` 4.1.3,
  `ml-dsa` 0.1.1, `module-lattice` 0.2.3, `hybrid-array` 0.4.15, `sha2`
  0.10.9 `force-soft`, `signature`, `subtle`); 0 xmm/ymm/x87-instructies in
  loader en dienst (`objdump`). TEST-ankers in `tools/update-keys/`.

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2
python3 -B -u tools/test_update.py                           # exit 0
python3 -B tools/capture_update.py                            # exit 0 -> docs/verification/b03/
```

## Hostcontroles (exit 0)

| Controle | Resultaat |
|---|---|
| `cargo test --locked -p update-core -p update-tool` | 7 + 1 tests: RFC 8032 Ed25519-testvector 1 via dezelfde API, ML-DSA-65 sleutel-/handtekeninggrootten (1952/3309) en deterministisch tekenen, manifestrondgang met manipulatiematrix (canonieke bytes, Ed25519-byte, ML-DSA-byte, staart) waarbij één geldige handtekening onvoldoende is, structurele weigering vóór iedere handtekeningcontrole, slotselectie met terugval en vloer, installatiebeslissingen die nooit het actieve slot kiezen of teruggaan, headertabel; `update-tool` keygen/sign/verify/pack/header-rondgang als subprocess (SIGNATURE, DIGEST, UNKNOWN_KEY, lengtecontrole) |
| `python3 -m unittest discover -s tests -p 'test_update_*.py'` | 10 harnastests: Python-tabel gelijk aan `update-core` en de kernelscope, headerhint/tabel, structurele manifestparse, orakel op synthetische store, iedere stage heeft een plan, volledige transcripten slagen, exit/kill/volgorde-weigeringen, elk verwacht getal dragend, mutaties raken alleen de genoemde bytes |

## QEMU-stages

Alle stages op één bevroren bron (vingerafdruk `07994d47461a1dd5…`, loader `299acb684e45d60c…`), één
`signed-boot`-loader, kernels v1/v2/v1-cut en één ondertekend niet-ELF-image.
Reeksen delen slotopslag en OVMF_VARS: main (0–2), cut (6–8); de overige stages
starten elk een verse store en verse variabelen.

| # | Stage | Loader (serieel) | Kernel en dienst | Hostorakel |
|---|---|---|---|---|
| 0 | install | `FLOOR:0`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:1`, `SIGNED:OK` | v1; `INSTALLED:2`; 24 484 reads, 3 056 writes, 3 flushes, 3 048 sectoren gekopieerd, 1 headerwrite, 475 ticks | slot B = exact manifest v2 + image v2, header active=B gen 2, slot A en inbox byte-identiek |
| 1 | boot-v2 | `FLOOR:1`, `SLOT:B:VERSION:2`, `SIGNED:OK` | v2; `UP_TO_DATE`; 0 writes | store ongewijzigd |
| 2 | downgrade (host zet header op A) | `FLOOR:2`, `SLOT:A:REJECTED:DOWNGRADE`, `SLOT:B:VERSION:2`, `SIGNED:OK` | v2; `UP_TO_DATE`; 0 writes | store ongewijzigd |
| 3 | bad-signature (ML-DSA-byte slot B geflipt, header B) | `FLOOR:0`, `SLOT:B:REJECTED:SIGNATURE`, `SLOT:A:VERSION:1` | v1; draaiend slot A ondanks header B (`ACTIVE_REJECT` 3); `INSTALLED:2` | slot B hersteld naar v2, header active=B gen 2 |
| 4 | bad-digest (imagebyte slot B geflipt, header B) | `FLOOR:0`, `SLOT:B:REJECTED:DIGEST`, `SLOT:A:VERSION:1` | v1; manifest B geldig (`ACTIVE_VERSION` 2) maar niet draaiend; `INSTALLED:2` | slot B hersteld |
| 5 | bad-elf (ondertekend niet-ELF-image in B, header B) | `FLOOR:0`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:1` | v1; `INSTALLED:2` | slot B hersteld; vloer niet verhoogd (volgende reeksen starten vers) |
| 6 | cut (`test-update-cut`) | `FLOOR:0`, `SLOT:A:VERSION:1` | v1-cut; `UPDATE:CUT:IMAGE` na 64 niet-headerwrites, echte hostkill (raw −9) | header en slot A ongewijzigd, slot B zonder manifest met een prefix van ≥64 sectoren van image v2 en niets daarbuiten |
| 7 | cut-resume | `FLOOR:1`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:1` | v1-cut; `RESUMED` 1, geen cut; `INSTALLED:2`; 3 048 sectoren, 1 headerwrite | slot B = v2, header active=B gen 2 |
| 8 | cut-final | `FLOOR:1`, `SLOT:B:VERSION:2` | v2; `UP_TO_DATE`; 0 writes | store ongewijzigd |
| 9 | no-valid-slot (Ed25519-byte A en ML-DSA-byte B geflipt) | `FLOOR:0`, `SLOT:A:REJECTED:SIGNATURE`, `SLOT:B:REJECTED:SIGNATURE`, `BOOT:FAIL:NO_VALID_SLOT` | geen `KERNEL:ENTRY`; raw exit 35 (0x11) | store ongewijzigd |
| 10 | inbox-signature | `FLOOR:0`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:1` | v1; `UPDATE:REJECTED:SIGNATURE` (`INBOX_REJECT` 3); 0 writes, 0 flushes | store ongewijzigd |
| 11 | inbox-downgrade (A = v2, inbox = v1) | `FLOOR:0`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:2` | v2; `UPDATE:REJECTED:DOWNGRADE` (`INBOX_REJECT` 5); 0 writes | store ongewijzigd |
| 12 | inbox-key (inbox met vreemd sleutelpaar) | `FLOOR:0`, `SLOT:B:REJECTED:FORMAT`, `SLOT:A:VERSION:1` | v1; `UPDATE:REJECTED:UNKNOWN_KEY` (`INBOX_REJECT` 2); 0 writes | store ongewijzigd |

Elke kernelrun eindigt met `UPDATE:RESOURCES:PASS` (lege capability-engine,
framebaseline, kernel-eigen DMA-pool) en `M1:PASS` (raw exit 33); de
hostrunner controleert markervolgorde, exitcode, `FAIL`/`PANIC`, de getallen
en de bytes van de slotopslag onafhankelijk (`tools/run_update.py`,
`tools/update_disk.py`). De loader-, kernel- en dienstbinaries zijn per stage in
`b03/<n>-<stage>/result.json` gehasht.

## Negatieve tests

- Loader: `REJECTED:SIGNATURE` bij één geflipte ML-DSA-byte (bad-signature,
  no-valid-slot B) en bij één geflipte Ed25519-byte (no-valid-slot A);
  `REJECTED:DIGEST` bij één geflipte imagebyte (bad-digest);
  `REJECTED:FORMAT` voor een correct ondertekend image dat geen laadbare ELF
  is (bad-elf), waarna het andere slot start en de vloer niet stijgt;
  `REJECTED:DOWNGRADE` voor een geldig maar ouder slot onder de vloer
  (downgrade); `REJECTED:FORMAT` voor een leeg slot; `BOOT:FAIL:NO_VALID_SLOT`
  met exit 0x11 en zonder `KERNEL:ENTRY` als beide slots falen.
- Dienst: `UPDATE:REJECTED:SIGNATURE` (geflipte ML-DSA-byte in de inbox),
  `UPDATE:REJECTED:DOWNGRADE` (inbox v1 onder draaiende v2),
  `UPDATE:REJECTED:UNKNOWN_KEY` (inbox ondertekend met een ad-hoc vreemd
  sleutelpaar), telkens met 0 writes en 0 flushes (kernelbroker-telling);
  `UPDATE:UP_TO_DATE` zonder writes als de inbox niet nieuwer is.
- Hosttests: één geldige handtekening is onvoldoende; manipulatie van
  canonieke bytes, beide handtekeningen of de nulstaart weigert; een
  verkeerd-lange image weigert `pack`.

## Bewijsgrens

Niet bewezen: firmwareverificatie van de loader zelf (Secure Boot, B04),
sleutelrotatie/intrekking en releaseankers (de ankers zijn TEST-ankers met
seeds in de repository), netwerkdistributie (N-spoor), pakket-/app-updates
(A02), fysieke schijven of stroomuitval (een emulator-kill is iets anders),
een aanvaller die OVMF_VARS kan schrijven, SMP. De GitHub Actions-workflow
`B03 signed updates acceptance` moet op iedere PR-head groen zijn; zie
PROJECT-STATE.md voor de run op de eindhead.
