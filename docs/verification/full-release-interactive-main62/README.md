# Actuele main748e29b: native referentie en gewone bediening

Vaste geteste code `d9f8c216fd80aee3ef38fa52286f1c3c570b1f2b`,
branch `simon/native-interactive-cold`, met main
`748e29b7455ca4213a6bf3ba3808c3097458263d` inhoudsbehoudend geïntegreerd.
V3 `c20a33000ad604ff91733e73f452aa49613ac0a5db2f8573cc7a55839d22f70b`.
Preflight/test/vendor-behoud en rootlockgrens:
`../native-test-version/main62-integration.md`.

Uitgevoerde commando's, elk exit0:

```sh
python3 -B -m unittest tests.test_preflight tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
python3 -B -m unittest discover -s tools/tests
LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" CARGO_TARGET_DIR="$PWD/target/native-test-version/main62-host" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
NOS_BUILD_DIR="$PWD/build/native-test-version/main62-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-test-version/main62-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-test-version/main62-interactive" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-test-version/main62-interactive" --interactive-reference-run "$PWD/build/native-test-version/main62-reference/run/materiaal-normal-ce4fa5bbc0" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
```

79 gerichte/138 tools-tests en rootworkspace inclusief doctests groen.
Referentie: 135.971s/raw33; gewone zevenprocessenvariant:
97.227s/QMP-quit/raw0. Alle zes volledige native frames bytegelijk,
geen maskers/fallback. Zeven app/ELF-records, bron, loader, machine,
firmware en QEMU-implementatie zijn gebonden. Na exit zelfstandig
diskorakel: object0/rev1, `Nieuw document`, `nieuw os werkt`, generation1,
clocks[1,0], andere domeinen leeg en canaries intact.

Logs `/tmp/nieuw-os-main62-integrated-{targeted,tools,workspace}.log` en
`/tmp/nieuw-os-main62-{reference,interactive}-{build,run}.log`.
Summaries zijn gegenereerde selecties van originele rapporten met hun
SHA256 en originele lokale pad. Volledige QMP-traces, PPM's en disks staan
in de genoemde gegenereerde runmappen. Ruwe seriële bytes zijn exact bewaard.

Gewone kernel:
`f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3`.
Loader:
`7e55ab0fe6c232380f52dc5be1334b41388b70d80e0ebe2b0072de7e12373a09`.
Dit zijn geverifieerde artifacts uit deze werkboom, geen belofte van
bitreproduceerbaarheid in een andere werkboom. De afzonderlijke verificatiesessie
vond op historische v324d5f4c1 andere payloadhashes bij eigen builds en heeft
die grens terecht open gelaten; hashes/orakels zijn niet versoepeld.

**Niet uitgevoerd op deze v3:** Windowsgast/nieuwe ZIP, geïntegreerde gewone
koude recovery, guest clean shutdown, volledige gebruikerssessie/duurmatrix,
menselijke UX/securityaudit of fysieke hardware. Historisch Windowsbewijs
blijft aan eabc27b/c532 gebonden; koude runner-WIP is een aparte branch en
geen onderdeel van deze integratie. Geen volledige-release- of cleanupclaim.
Scorevoorstel **34% → 34%**; minstens50% open. Centrale beoordeling/status
zijn van de teamleider, lifecycle/merge van de toegewezen coördinator.
