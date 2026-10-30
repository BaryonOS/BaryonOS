# TL-VERIFY-SOURCE-RUNTIME: onafhankelijke functionele uitvoering

**Geslaagd binnen deze afgebakende bron-runtimeproef. Geen geslaagde
byte-reproduceerbaarheid.** Op 30 oktober 2026 zijn de bestaande eigen
artifacts opnieuw met een verse diagnostische referentie en de gewone
zevenprocessenvariant uitgevoerd. De ongewijzigde runners accepteerden
dezelfde bron, eigen artifactidentiteiten, echte invoer, zes volledige
frames en het schijforakel na afsluiten.

| Gate | Commandostatus | Werkelijke QEMU-exit | Duur |
|---|---:|---:|---:|
| Verse achtprocessenreferentie | 0 | 33 | 135.625256873 s |
| Gewone zevenprocessenrun | 0 | 0, na QMP-quit | 93.752015893 s |
| Opnieuw replay/frame/disk/bronbinding controleren | 0 | Geen nieuwe VM | Hosthercontrole |

De afzonderlijke [TL-VERIFY-IMAGE-blokkade](../tl-verify-image-1/REPRODUCIBILITY.md)
blijft ongewijzigd open. De nieuwe resultaten vervangen geen oude hashes of
verdicts. Verschillende instructiebytes zijn niet hierdoor bytegelijk of
algemeen semantisch equivalent verklaard.

## Bron, platform en eigen artifacts

Vaste codebron `74d15ac09ac61c8d47a882c3c982711af1a9d891`, checkoutbasis
`3d2e1aba218feb39e02a1a5d257c5eb63f50e26b`; runtime-HEAD
`79b956e79da7a861da3cb1fc9b42d45138be73c2` bevat uitsluitend aanvullende
bewijsdocs. V3 voor/na:
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.
Branch `simon/tl-verify-image-1`. Geen rebuild of productiecodewijziging.

Eigen gewone kernel:
`c91f39aafb8bcecd9858bcc2c082f2d22ff3f9179eff15d858f2880a3db12d68`.
Eigen referentiekernel:
`1ca3b074eeec8a4ca42799008948a4d6f49d17f1f3e8aec106df324139e39b9c`.
Gedeelde eigen loader:
`e7aa259a20b14553a734810c5c254727525756177d4b79d44f6aca10e1b264e5`.
Alle zeven apprecords zijn tussen deze twee eigen builds gelijk,
niet gelijk verondersteld aan de oorspronkelijke builderartifacts.
De achtste referentie-app is uitsluitend de diagnostische controller.

Linuxhost, QEMU 10.2.1, `pc-q35-10.0`, TCG single-thread, qemu64,
1 CPU, 256 MiB, headless/PS2, rng-random. Wrapperhash:
`09b7d8ab4a583935d449882fcc5fbd73ca76398c4be3a8c0d46108424d5c4b5d`;
implementatiehash:
`fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.
OVMF/SeaBIOS/VGA-hashes en volledige commands staan in de summaries.
Elke VM gebruikte eigen ESP, verse OVMF-VARS en eigen synthetische
1-MiB-disk; geen fysieke disk of persoonlijke data.

## Exacte uitgevoerde commands

Vanuit de eigen repositoryroot:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export LOCALAPPDATA=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools
export RUSTUP_HOME="$P/rustup" CARGO_HOME="$P/cargo"
export PATH="$P/cargo/bin:$P/qemu:$PATH"
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
export OS_OVMF_CODE="$P/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$P/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"

python3 -B -u tools/run_materiaal_probe.py \
  --build-dir "$PWD/build/tl-verify-image-1/reference" \
  --machine pc-q35-10.0 --timeout 240 \
  --vga-rom "$P/qemu/usr/share/seabios/vgabios-stdvga.bin" \
  --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde

python3 -B -u tools/run_materiaal_preview.py \
  --build-dir "$PWD/build/tl-verify-image-1/interactive" \
  --interactive-reference-run "$PWD/build/tl-verify-image-1/reference/run/materiaal-normal-a69441f209" \
  --timeout 240 --machine pc-q35-10.0 \
  --bios-rom "$P/qemu/usr/share/seabios/bios-256k.bin" \
  --vga-rom "$P/qemu/usr/share/seabios/vgabios-stdvga.bin"

python3 -B docs/verification/tl-verify-source-runtime/recheck.py
git diff --check
```

Deze commands eindigden met exit 0. De uitvoerlogs staan onder:

```text
/tmp/nieuw-os-tl-verify-source-runtime-reference-canonical.log
/tmp/nieuw-os-tl-verify-source-runtime-interactive.log
```

De eerste aanroep probeerde bovendien
`--run-root "$PWD/build/tl-verify-source-runtime/reference"` en stopte
voor een VM-start met exit 1:
`ValueError: Runs must use the dedicated BUILD/run directory`.
Dat was een ongeldige locatiekeuze, geen artifactweigering. Daarna is die
optionele parameter weggelaten en de verplichte canonieke `BUILD/run`
gebruikt. Geen runner/private_path/reportpad herschreven.
Het foutlog blijft bewaard als
`/tmp/nieuw-os-tl-verify-source-runtime-reference.log`.
Een eerste inspectie zocht ten onrechte `report.json` (FileNotFoundError,
exit 1); de runners schrijven `result.json`. Geen gate is daarvoor herhaald.

## Werkelijk runtimebewijs

Nieuwe runmappen, allebei in de eigen worktree:

```text
build/tl-verify-image-1/reference/run/materiaal-normal-a69441f209
build/tl-verify-image-1/interactive/preview-run/materiaal-3d940fcfdd
```

Referentierapport-SHA256:
`59482914b7ac50c19ae19a4c66b20b7f73d3ed57ed0e00d587bc566a30c391a3`.
Gewoon rapport-SHA256:
`5608ef37440cf33e5972c50eae2ee03f174c10fb335589b04b7159faaacab130`.
Originele paden zijn behouden, niet aangepast om replay toe te staan.

De gewone run heeft 1064 QMP-traceregels en echte muis/toetseninvoer voor
`ready`, `editor`, `dirty`, `saved`, `list`, `reopened`. Alle zes volledige
800x600-PPM-bestanden zijn bytegelijk aan de nieuwe eigen referentie:
geen cursor-/caretmaskers, geen F12/controller in de gewone variant.
De ongewijzigde native replay is daarna nogmaals uitgevoerd, exit 0.

Het onafhankelijke schijforakel is na QMP-quit en daadwerkelijke raw0
opnieuw op `volume.after.img` uitgevoerd: object0/rev1, naam
`Nieuw document`, tekst `nieuw os werkt`, 14 bytes, generation1,
clocks `[1,0]`, canaries intact en andere domeinen leeg volgens het orakel.
De bytes van `volume.img` en `volume.after.img` zijn gelijk.
Nadisk-SHA256:
`246c8f9773533796718a729389c26d6103f45f04e7a83b4d06503e17c6d4e07e`.

`reference-summary.json` en `ordinary-summary.json` zijn gegenereerde
selecties van de echte rapporten met hun originele paden/hashes. Alleen
de omvangrijke gewone `qmp_trace` is vervangen door aantal en hash van de
gesorteerde JSON-weergave; de ongewijzigde volledige trace blijft in het
originele `result.json`. Ruwe serialbytes zijn apart opgenomen.
`recheck.json` bevat nieuwe frame-, disk- en loghashes; `recheck.py` is
uitsluitend een bewijshercontrole/generator, geen alternatief runorakel.
De bestaande runners/orakels zijn niet gewijzigd.

Git normaliseerde aanvankelijk twee CRLF-regeleinden in elke serialkopie.
De eerste committed-bytecontrole gaf daarom exit 1. De lokale
`.gitattributes` en een expliciete `git add --renormalize` herstellen dit;
de daarna uitgevoerde vergelijking van `git show HEAD:<serialpad>` met
de werkelijke runbytes en opgenomen SHA-256 slaagt, exit 0.
De gastuitvoering en originele runbestanden zijn daarvoor niet veranderd.

## Open grenzen en overdracht

Deze proef toont begrensde functionele uitvoering van dezelfde bron met
eigen binaries. Geen byte-reproduceerbaarheid, menselijke UX/securityaudit,
algemene correctheid, controller-/procescleanup voor de gewone variant,
koude herstart, nieuwe Windowsrun/ZIP, fysieke hardware, volledige
productmatrix of NAS/internet/privacyacceptatie.
De synthetische disk is geen bewijs van versleutelde persoonlijke opslag.

FR-U01/FR-A02/FR-O01-deelbewijs is onafhankelijk aangevuld; geen volledig
nieuw rubriccriterium toegekend. Scorevoorstel **34% -> 34%**.
Centrale status/assessment zijn van de manager en hier niet gewijzigd.
Reproduceerbaarheidsrootcause blijft expliciet open in het eerdere rapport;
deze functionele gate sluit die niet.
