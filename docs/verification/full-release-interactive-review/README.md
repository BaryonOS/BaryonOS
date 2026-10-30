# Gecombineerde reviewfixes: echte native herhaling

Vaste codecommit `74d15ac09ac61c8d47a882c3c982711af1a9d891` op
`simon/native-interactive-cold`. Alle zes reviewfixes inhoudsbehoudend
geïntegreerd; tools/kernel exact gelijk aan
`d104fd9a7f33b478c3e1b43f8d99c6d65af73fd6`.
Bronvingerafdruk v3:
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.
Geen bronwijziging tijdens beide echte VM's.

Uitgevoerde commando's, allemaal exit0:

```sh
python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
python3 -B -m unittest discover -s tools/tests
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
NOS_BUILD_DIR="$PWD/build/native-test-version/review-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-test-version/review-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-test-version/review-interactive" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-test-version/review-interactive" --interactive-reference-run "$PWD/build/native-test-version/review-reference/run/materiaal-normal-ea87d6fd9b" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
```

55 gerichte en 138 tools-hosttests groen. Normale achtprocessenreferentie:
139.078s/raw33. Gewone zevenprocessenvariant: 94.889s/QMP-quit/raw0.
Zes volledige 800x600-PPM's bytegelijk zonder maskers, alle zeven
app/ELF-records, loader, bron en platform gebonden. Onafhankelijke diskcontrole
na QEMU-exit: object0/rev1, `Nieuw document`, `nieuw os werkt`, generation1,
clocks[1,0], andere domeinen leeg en canaries intact.

Kernel gewone variant:
`f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3`.
Loader:
`7e55ab0fe6c232380f52dc5be1334b41388b70d80e0ebe2b0072de7e12373a09`.
Build/imagepaden staan in summaries; eigen ESP, verse OVMF-VARS en eigen
1MiB-volume per run, geen fysieke disk.
QEMU10.2.1/pc-q35-10.0/TCG/1CPU/256MiB/headless/rng-random.
Alle firmware-/QEMU-/framehashes staan in de summaries.

Logs `/tmp/nieuw-os-review-integrated-{targeted,tools}.log`,
`/tmp/nieuw-os-review-{reference,interactive}-{build,run}.log`.
Summaries zijn gegenereerde selecties van originele rapporten met hun
SHA256 en originele lokale pad; volledige QMP-traces, PPM's en disks
blijven in de gegenereerde runmappen. Ruwe seriële bytes zijn exact bewaard.
Historisch Windowsbewijs onder `../windows-qemu-interactive-1/` blijft
blob-identiek en aan eabc27b/c532 gebonden, niet omgelabeld naar deze bron.

**Niet uitgevoerd op deze nieuwe v3:** Windowsgast/nieuwe ZIP, koude gewone
herstart als geïntegreerde acceptatie, volledige product-/duurmatrix,
menselijke UX/securityaudit en fysieke hardware. Koude verkenning met
renderer-afgeleid volledig lijstframe staat buiten deze releasegate.
Geen cleanup-/controllerclaim voor de gewone variant. Geen extra volledig
rubriccriterium: voorgestelde score **34% → 34%**. Centrale status/queue/
assessment zijn vanaf nu van de teamleider; dit rapport vervangt ze niet.
Lifecyclecoördinator beslist over PR #66-opname en reviewthreads op de
actuele head/base, geen merge of branchregelomzeiling door dit bewijs.
