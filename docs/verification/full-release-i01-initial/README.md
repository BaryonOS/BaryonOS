# Eerste geïntegreerde native uitvoering — tussenbewijs

Exact uitgevoerde bron: `1e0d6578105a355cc0288cb6dd39c3aeeafbf834`.
Vingerafdruk v3: `30ad329a60f2fcc0d94df263c776865a069dadb4f61ab25f5cc9eb5ed7a90d31`.
Dit archief beschrijft die bron; latere invoer-, record- en uploadreparaties
worden hiermee niet automatisch geaccepteerd. Alle volledige productrijen
blijven open en de score blijft 32%.

Werkelijk uitgevoerd: `tools/test_system.py`, exit 0, twee echte QEMU-runs
(normal + koude read-only recovery), ieder raw exit 33. De normale finale
kernel is bytegelijk aan de werkelijk geteste normale kernel:
`dd7f277f2616b16d6b7b1d10dc0e78e0ce4d3cd7af1cec90aee6ec51c1979e6c`.
De dienst bevestigt één document van 4096 bytes; het afzonderlijke Python-
orakel controleert die bytes, naam, revisie en onaangeroerde andere domeinen.
Dezelfde image wordt na volledige stop in een nieuwe VM read-only geopend:
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.

Uitvoering (geen globaal aangepaste toolinstellingen):

```sh
PATH=/workspace/.nieuw-os-env/bin:$PATH \
OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
NOS_QEMU_MACHINE=pc-q35-8.2 \
NOS_BUILD_DIR=/workspace/nieuw-os/build/full-release-i01 \
NOS_TARGET_DIR=/workspace/nieuw-os/target/full-release-i01 \
CARGO_TARGET_DIR=/workspace/nieuw-os/target/full-release-i01 \
python3 -B -u tools/test_system.py
```

`command.txt`, `system-gate.json`, beide resultaatbestanden en de raw serial
zijn onverkort bewaard. De runner controleert exit, markers, getallen, echte
opslag, cleanup en de werkelijk geladen ESP-bytes tegen het buildmanifest.
Alleen hashes en tekstbewijs staan in Git; lokale images onder de runpaden.

`root-host.txt`: 276 root-Rust-hosttests op deze geïntegreerde bron, exit 0.
`ui-reset-fix.txt`: daarna de gerichte 30 invoertests op reparatiebron
`9f8d4c8`, exit 0. Twee echte resetproblemen faalden vóór reparatie en werden
onafhankelijk herhaald. De latere native recordcaller is een afzonderlijke
implementatiestap; deze oorspronkelijke run had uitsluitend legacy SEAL/UNSEAL.

Geen interactieve desktop, volledige netwerkstack, persistente encryptie,
hardwareondersteuning of releaseoplevering. De diskfixture is plaintext en
de sleutels bestaan alleen in RAM.
