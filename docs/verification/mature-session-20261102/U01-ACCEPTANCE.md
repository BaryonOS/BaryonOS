# U01 — native compositor en sessie: geaccepteerd

Functionele bron `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`, source-v3
`3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`.
Rubric U01 krijgt zijn vaste4 punten: **37→41/100**. De veel bredere
FR-/hardware-/menselijke releasecriteria en U02 blijven afzonderlijk open.

## Uitgevoerd

De eigen UEFI-loader/kernel draait Materiaal en Rekenmachine als afzonderlijke
CPL3-processen. Alleen compositor heeft Display; alleen inputbron InputSource.
Toetsenbord- en muisfocus, twee versleepbare vensters, brongebonden complete
appbeelden en negatieve SUR/INP/capabilityproeven zijn werkelijk uitgevoerd.
Een echte compositor-ud2 (slot4/vector6) leidt tot uitsluitend retirement van
oude presentatiekanalen, beide pauseACKs, een nieuwe dienstgroep en gevalideerde
rebind. Oude kanalen blijven ongeldig. MateriaalPID4 en CalculatorPID8 blijven;
compositorPID5 wordt9, epochs/generatie worden vers. Probeaantallen3→10 komen
van werkelijke afwijzingen. Draft en berekening blijven behouden. Daarna
volgen echt bewaren, lijst en heropenen, onafhankelijke schijfcontrole en nul
resterende processen/handles/queued messages plus volledige frameterugwinning.

| Uitvoering | Resultaat en grens |
|---|---|
| Root QEMU10.0.13/q35-10.0/TCG/1CPU/256MiB | Volledige13fasen, raw33,753,648s; eigen tooluitvoering |
| CI QEMU/Ubuntu-profiel | Volledige13fasen, raw33,853,340s; artifact onafhankelijk herlezen, alle13framehashes/18hostoracles, bytes/cleanup opnieuw gecontroleerd |
| Lokale Windows/WHPX | Geautoriseerde lokale chat rapporteert alle13fasen/raw33,36,946s; eerste volledige beeld6,971s; exactbron/adapterguards blijven. Ruwe Windowsbestanden zijn niet opnieuw in cloud gelezen |
| Korte herstelhandshake | Actualcomp5→9/Rec1,15,387s/QMPexit0; expliciet smoke_only/native_gate=false, geen afzonderlijke acceptatiepunten |

Dit zijn afzonderlijke ontwikkelruns op verschillende hosts/configuraties,
geen algemene of statistische snelheidsvergelijking. De rootnative opdracht:

```sh
NOS_BUILD_DIR=$PWD/build/window-final NOS_TARGET_DIR=$PWD/target/window-final python3 tools/build.py --kernel-feature native-window-session-probe
NOS_BUILD_DIR=$PWD/build/window-final NOS_TARGET_DIR=$PWD/target/window-final python3 tools/run_window_session.py --timeout 1800 --bios-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/bios-256k.bin --vga-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/vgabios-stdvga.bin
```

De werkelijke QEMU-commandoregel, firmware/ROM/kernel/loader/8ELFhashes,
procesimage-identiteit, immutable-input-/bronhercontroles, QMP-trace en alle
fasestaten staan in [het volledige resultaat](window-337-complete.json).
De samenvatting [U01-ACCEPTANCE.json](U01-ACCEPTANCE.json) bevat alle framehashes,
voor/na-modellen en de definitieve1MiB-schijfhash. De schijforacle vindt object0,
revisie1, `Nieuw document`,14bytes `nieuw os werkt`, intacte canaries. Rawserial
is exact gecomprimeerd bewaard; SHA's staan in die samenvatting.

[CI-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41487814892),
[artifact13164815420](https://github.com/simonvanieperen/nieuw-os/actions/runs/41487814892/artifacts/13164815420),
[CI-readback](window-337-ci-verification.json),
[gerapporteerd Windowsbewijs](window-337-windows-reported.json).
CI bouwde merge663bb8c met exactdezelfde source-v3; dit wordt niet als direct337
herlabeld. Alle18workflows op337 zijn daadwerkelijk groen; zie ci-337-workflows.json.

Root UI-host/Clippy/fmt/scalarbuild en elf objectarchieven slagen opdeze bron;
179Python-tooltests en8runnernegatieven slagen. Recoveryhelper heeft6 werkelijke
faseregressies plus scene/rastertests; SDK-/app-/SUR-/input-/authoritytests en
nativeM4/IPCwait/N01 zijn aan hun eigen eerdere bron en onveranderde scope gebonden.
Geen nieuw bewijs wordt stil naar een andere binary/sourceversie overgezet.

## Reviewbaar pakket en open grenzen

De normale `native-window-session`-bundel bevat geen crashinjectie. De Debian-
firmwarekandidaat337 heeft ZIP-SHA
`bac273f2771180573de93790891941ad07bfdce1ed4399d9773fd4b9ba87fb89`;
[19payload/8ELF/font/firmware/launcherbewijs](window-337-package-verification.json).
De afzonderlijke CI-bundel heeft andere Ubuntu-firmware en dus een andere ZIP-SHA;
zie CI-readback. Geen ondertekende productie-release of Windows-SDL-starterclaim.

Echte authenticatie/FIDO2/biometrie, klembord, volledige tekstshaping,
toegankelijkheid, algemene appinstallatie en gemeten dagelijkse interactiebudgetten
blijven open. Fysieke devices en algemene veiligheidssuperioriteit zijn niet
bewezen. PR123/idlecode en CPUproeven zijn afzonderlijk van deze337-acceptatie.
