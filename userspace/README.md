# Synthetische eigen userprobe voor M3

Deze kleine statische ELF64-probe is bron voor de echte ring-3-proeven KS-04–08. De hostbouw en artifacttests hieronder bewijzen de bestandsinterface; alleen de kernel/QEMU-harness kan isolatie, preëmptie en foutafhandeling aantonen.

Geen dependencies, hostlibc, allocator, TLS, dynamische linker of port-I/O. De probe gebruikt uitsluitend `abi_info` (0), `yield` (1) en `exit` (2). Zij kan geen seriële succesmarker schrijven. `probe.rs` bevat bewust afzonderlijke verboden instructies voor de negatieve tests; die schakelen geen compiler-targetfeatures in.

## Bouwinterface

De hoofdagent beheert `tools/build_users.py` en bouwt met de bestaande lokale Rust 1.90.0-omgeving:

```powershell
python tools/build_users.py
python -B -X utf8 userspace/tests/test_probe.py --elf build/users/probe.elf
```

De eerste opdracht roept rechtstreeks rustc aan, zodat `.cargo/config.toml` niet per ongeluk het kernellinkerscript op de userprobe toepast. Exacte flags, naast absolute bron-/uitvoerpaden:

```text
rustc +1.90.0 --edition=2021 --crate-type=bin --crate-name=os_user_probe
--target=x86_64-unknown-none -C panic=abort -C opt-level=2
-C code-model=small -C relocation-model=static -C no-redzone=yes
-C linker=rust-lld -C link-arg=-T<absolute-pad>/userspace/linker.ld
-C link-arg=--no-pie -C link-arg=--build-id=none
-C link-arg=-zmax-page-size=0x1000 --emit=link
-o <absolute-pad>/build/users/probe.elf <absolute-pad>/userspace/probe.rs
```

De kernel krijgt dit artifact via `NOS_USER_ELF`; root beheert de embed-/buildvolgorde. Er is geen root-Cargo-build of wijziging aan rootconfig nodig vanuit deze map.

## Vaste interface met de kernel

ELF: `ET_EXEC`, AMD64 little-endian, entry/text op `0x40000000` met RO/X; data op `0x40010000` met RW/NX. Het linkerscript begrenst de tekst tot vóór data en houdt het datagebied exact 32 bytes. Kernelmapping verleent de daadwerkelijke paginarechten; ELF-flags op zichzelf doen dat niet.

Vier little-endian `u64`-velden, door de kernel vóór processtart ingevuld:

| Offset | Veld | Gebruik |
|---|---|---|
| +0 | mode | Selecteert één van de onderstaande takken |
| +8 | counter | Privévoortgang; wordt vóór iedere geselecteerde handeling verhoogd |
| +16 | pattern | Privépatroon; probe bewaart de initiële waarde in r13 en vergelijkt iedere lus |
| +24 | target | Alleen voor modes 3–5; testharness kiest het beschermde doeladres |

Iedere procesinstantie moet eigen fysieke data-/textframes krijgen. Hetzelfde virtuele adres of hetzelfde ELF-bestand betekent geen gedeelde frames. De kernel leest voor meetdoeleinden alleen gevalideerde eigen procesframes; er is geen gedeelde usercounter.

| Mode | Handeling na counter++ |
|---|---|
| 0 | Onbegrensde busy worker zonder yield |
| 1 | Yield en opnieuw tellen |
| 2 | Exit met code 0 zodra counter ≥ 8 |
| 3 | Lees `u64` op target |
| 4 | Schrijf `u64=1` op target; harness kiest B's supervisor-identityadres |
| 5 | Schrijf `u64=1` op target; harness kiest kerneltext |
| 6 | Spring naar stackbasis `0x7ffef000` |
| 7 | Spring naar heapbasis `0x60000000` |
| 8 | Schrijf op onderste stackguard `0x7ffee000` |
| 9 | Schrijf op bovenste stackguard `0x7ffff000` |
| 10 | `cli` |
| 11 | `fninit` |
| 12 | `pxor xmm0,xmm0` |
| 13 | `vxorps ymm0,ymm0,ymm0` |
| 14 | `ud2` |
| 15 | Direct exit met code 7 na de eerste counterverhoging; geen acht-iteratiedrempel |
| 16 | ABI-info met nulargumenten via `int 0x80` nadat ESP/RSP nul is gemaakt; geen normale terugkeer |

Vóór de eerste lus voert iedere instantie drie echte syscallcontroles uit:

1. ABI-info: RAX=0 en RDX=`1<<32`. Direct vóór `int 0x80` zet `std` de direction flag (DF). Na terugkeer leest de probe RFLAGS via `pushfq` en controleert dat DF weer nul is; het eigen foutpad voert altijd `cld` uit.
2. Onbekend nummer `0xffff`: RAX=-2 (`ENOSYS`), RDX=0.
3. ABI-info met één expres ongeldig ongebruikt argument RSI=1: RAX=-1 (`EINVAL`), RDX=0.

Iedere preludecontrole vergelijkt RBX/RBP/R14/R15/RCX/R11 met sentinels, R12 met het vaste datagebied, R13 met het privépatroon, RSP met de afgesproken initiële stackpositie en alle niet-resultaatargumenten met hun verwachte waarde. Buiten het opzettelijke RSI=1-geval zijn ongebruikte argumenten nul. Het zijn geen hostcalls; de kernel/QEMU-proef moet deze instructies daadwerkelijk uitvoeren. Er is geen TF-probe omdat een debug-exception vóór een betrouwbare controle kan optreden; TF-sanitizing blijft apart te toetsen.

Foutcodes via gewone `exit`: `0xe0` ABI-afwijking, `0xe1` veranderd patroon, `0xe2` onbekende mode, `0xe3` verboden handeling liep onverwacht door. Een terugkerende exit voert `ud2` uit. Geen van deze exits telt als een geslaagde verwachte userfault.

De afzonderlijke kerneltest `test-m3-b-exit` gebruikt mode 15 voor B en mode 0 voor A. Gewenst bewijs: B eindigt met code 7 en counter 1, de kernel ruimt B gecontroleerd op en A maakt daarna voortgang. Exitcode 7 is hierbij gewone procesmetadata en geen reden voor een kernelpanic. De syscallimplementatie verwerkt de exit; de testobserver beslist afzonderlijk of de verwachte toestand is bereikt.

De kerneltest `test-m3-invalid-rsp` gebruikt mode 16 voor het testproces en mode 0 voor de andere worker. Counter is eerst eenmaal verhoogd. Daarna wist de probe alle syscallargumenten en RAX voor ABI-info, maakt ESP/RSP nul en voert `int 0x80` uit. Dit moet de softwarevalidatie van de opgeslagen returncontext bereiken: alleen het proces wordt beëindigd, de andere worker maakt voortgang. De `ud2` na `int 0x80` is een onbereikbare foutvangst; uitvoering daarvan is geen geslaagde proef. Dit is geen hardwarefaultclaim.

Modes 6/7 vereisen in de echte harness een page fault op exact het bedoelde doel met de instruction-fetch- en presentbits. Zonder NX kan uitvoering van nulbytes later alsnog op een ander adres faulten; alleen constateren dat A ergens fault, bewijst geen NX. Bij modes 3–5 zijn CR2 en de user/read/write-foutbits eveneens onderdeel van het bewijs. Bij alle foutmodi moet B verdergaan.

## Bewijs en beperkingen

Zie `verification/R03-HOST-2026-10-27.md` en `verification/host-evidence.json` voor de actuele uitgevoerde bouw en tests. De artifacttests omvatten echte ELF-segmenten, hashes, relocatie/TLS-afwezigheid, syscallargument-/DF-codering en negatieve gemuteerde ELF-bestanden. Opcode-aanwezigheid bewijst geen uitgevoerde instructie. De tests gebruiken de door root beheerde buildinspector; zij vervangen de kernelparser en zijn geen onafhankelijke implementatie daarvan.

Alle tijdelijke mutatiebestanden staan uitsluitend onder `userspace/tests/` en worden na de tests verwijderd. Artifacts blijven onder `build/` buiten Git. Deze map bevat geen claims dat KS-04–08 al geslaagd zijn.
