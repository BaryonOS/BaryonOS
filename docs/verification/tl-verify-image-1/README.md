# TL-VERIFY-IMAGE-1: gestopt op executable-bytebinding

**Geen geslaagde onafhankelijke native acceptatie.** Op 30 oktober 2026
is de controle volgens de opdracht bij de eerste concrete afwijking gestopt.
De bron is correct, maar de onafhankelijk gebouwde executablebytes zijn
niet gelijk aan het eerder geaccepteerde image. De oorzaak is niet vastgesteld;
dit rapport noemt geen bewezen compiler-, cache-, bron- of runtimefout.

De daarna afzonderlijk opgedragen **read-only reproduceerbaarheidstrace** staat
in [REPRODUCIBILITY.md](REPRODUCIBILITY.md). De afwijkingen zijn niet uitsluitend
onbeladen metadata: drie apps en de kernel hebben verschillende geladen
code/data. Vier andere apps hebben wel gelijke PT_LOAD-bytes; bij de loader
zijn alle afwijkingen tot tijdstempel- en CodeView-debugvelden gelokaliseerd.
Dit verandert geen oude hash, acceptatie-eis of runtimeuitslag.

Eigen branch `simon/tl-verify-image-1`, checkout/bewijsbasis
`3d2e1aba218feb39e02a1a5d257c5eb63f50e26b`, codebron
`74d15ac09ac61c8d47a882c3c982711af1a9d891`, bronvingerafdruk v3:

```text
24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd
```

Deze waarde is gelijk aan beide eigen buildmanifesten en de originele
builderrapporten. De builder bevestigde dat zijn oorspronkelijke runs
stabiel/alleen-lezen beschikbaar zijn. De SHA-256 van beide oorspronkelijke
rapporten is gecontroleerd tegen de gecommitte summaries; alle opgenomen
referentie-artifacthashes zijn opnieuw uit bytes berekend, inclusief de
volledige bestandenset. Ook de gewone seriële bytes, zes framehashes en
nadiskhash zijn vergeleken. **Deze statische binding is consistent**;
`builder-binding.json` bewaart de originele paden en rapporthashes.
Dit vervangt geen volledige replay of tweede gastuitvoering.

De bestaande replay vereist zowel eigen private ROOT als de oorspronkelijke
cwd/QMP-paden. Om geen padbinding te herschrijven of private_path te
versoepelen, is gekozen voor een eigen nieuwe referentie en gewone build,
met eigen targets, ESP's, VARS en gegenereerde 1-MiB-disks.
Er zijn geen builderbestanden of gedeelde code/interfaces gewijzigd.

## Concrete stopreden

Beide nieuwe builds slagen met exit 0. De bestaande
`package_windows_qemu.validate_build` valideert de gewone eigen build.
Een daarna expliciete vergelijking tegen het geaccepteerde image faalt:

| Bestand | Eigen build | Geaccepteerde builderbytes |
|---|---|---|
| kernel | `c91f39aafb8bcecd9858bcc2c082f2d22ff3f9179eff15d858f2880a3db12d68` | `f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3` |
| loader | `e7aa259a20b14553a734810c5c254727525756177d4b79d44f6aca10e1b264e5` | `7e55ab0fe6c232380f52dc5be1334b41388b70d80e0ebe2b0072de7e12373a09` |

Alle zeven app-ELF-hashes/records verschillen eveneens. De volledige
vergelijking staat in `blocker.json`; het eigen manifest staat in
`own-interactive-manifest.json`. Dit is geen bewijs dat verschillende
bytes hetzelfde gedrag hebben of dat de bron verschilt.
Buildpad-/omgevings-/cacheinvloed is hoogstens een onderzoeksvraag.

De eigen achtprocessenreferentie-VM was parallel aan de gewone build al
werkelijk gestart. QMP/readiness en echte snapshots verschenen; de gate
was nog niet afgerond. Na de hashafwijking is commandosessie 870 expliciet
gestopt. Er is daarna gecontroleerd dat geen proces met deze eigen
verificatieroute meer draaide. De gewone zevenprocessen-VM is **niet gestart**.
`interrupted-reference-initial-report.json` is het oorspronkelijke
onvoltooide startrecord, niet een eindrapport of groen testresultaat.
Er wordt geen exitstatus 0/33 of schijfinhoudacceptatie voor die onderbroken
run geclaimd. Geen assertie, hashvereiste of timeout is versoepeld.

## Uitgevoerde routes en commands

Alle commands vanuit de eigen worktree. Bestaande door de builder
opgegeven toolroute is alleen gebruikt, niet gewijzigd:

```sh
git fetch origin simon/native-interactive-cold
git switch -c simon/tl-verify-image-1 3d2e1aba218feb39e02a1a5d257c5eb63f50e26b

P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export LOCALAPPDATA=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools
export RUSTUP_HOME="$P/rustup" CARGO_HOME="$P/cargo"
export PATH="$P/cargo/bin:$P/qemu:$PATH"
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
export OS_OVMF_CODE="$P/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$P/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
export NOS_TARGET_DIR="$PWD/target/tl-verify-image-1"
export CARGO_TARGET_DIR="$PWD/target/tl-verify-image-1"
NOS_BUILD_DIR="$PWD/build/tl-verify-image-1/reference" \
  python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
NOS_BUILD_DIR="$PWD/build/tl-verify-image-1/reference" \
  python3 -B -u tools/run_materiaal_probe.py \
  --build-dir "$PWD/build/tl-verify-image-1/reference" \
  --machine pc-q35-10.0 \
  --vga-rom "$P/qemu/usr/share/seabios/vgabios-stdvga.bin" \
  --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/tl-verify-image-1/interactive" \
  python3 -B -u tools/build.py --kernel-feature native-system-materiaal
```

Fetch/switch/beide builds exit 0. De probe is **onderbroken**, geen geslaagde
gate. De expliciete eigen-versus-builderbytevergelijking gaf
`AssertionError`, exit 1; vervolgdiagnostiek bevestigde bovenstaande hashes.
Geen gewone `run_materiaal_preview.py` uitgevoerd op deze eigen build.

Platformroute: QEMU 10.2.1, `pc-q35-10.0`, TCG single-thread, qemu64,
1 CPU, 256 MiB, headless/PS2, rng-random. De opgegeven QEMU-wrapperhash
is `09b7d8ab4a583935d449882fcc5fbd73ca76398c4be3a8c0d46108424d5c4b5d`;
de builder noemde implementatiehash
`fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.
Die toolidentiteiten zijn geen nieuw groen runtimeresultaat.

Volledige lokale logs, met hashes in `blocker.json`:

```text
/tmp/nieuw-os-tl-verify-reference-build.log
/tmp/nieuw-os-tl-verify-reference-run.log
/tmp/nieuw-os-tl-verify-interactive-build.log
```

Een eerste hostpadcontrole gebruikte ten onrechte het relatieve
`original_report_path` vanuit de eigen worktree: FileNotFoundError,
Python exit 1. De omvattende shell was niet fail-fast en ging door met
de referentiebouw (shell exit 0); dat is niet als geslaagde binding gebruikt.
De controle is daarna afzonderlijk met expliciete builderroot uitgevoerd
en slaagt. Er is geen rapport of orakel aangepast.

## Handoff en echte open grenzen

Manager, build en release zijn direct over de eerste afwijking ingelicht.
Gevraagd aan de builder: feitelijke oorspronkelijke RUSTFLAGS/remapping/
toolchain-/werkpadvoorwaarden; nog geen oorzaak vastgesteld.
Een vervolg vereist expliciete keuze: onafhankelijk het exacte stabiele
image kopieren en testen, of de reproduceerbaarheid van eigen builds
onderzoeken. Geen verdere runtime/herbouw na het stopbesluit.

Geen nieuwe gewone/cold/Windows-acceptatie, geen ZIP, geen
procescleanup- of menselijke auditclaim, geen fysieke tests. De
testdisks zijn synthetisch en niet versleutelde persoonlijke opslag;
dit verifieert geen internet/NAS/privacy- of Security-Floor-productmatrix.
Oude Windowsresultaten blijven aan hun eigen bron gebonden.
FR-U01/FR-A02/FR-O01-deelverificatie blijft hier geblokkeerd.
Geen rubricpunten toegekend; centrale status/assessment zijn van de manager.
