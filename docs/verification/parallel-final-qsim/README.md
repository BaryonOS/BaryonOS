# I05 — definitieve native quantumacceptatie

## Bron en uitgevoerde uitkomst

De gecombineerde broncommit is `3e41bbf93822f7044c6020259f4a752925e1966d`,
met source-fingerprint v3
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
HEAD en de bronhash zijn voor en na de build/run gecontroleerd en gelijk.
De I04-reparatie van prefixed FPU-instructies maakt deel uit van deze bron.

**Eén werkelijke native QEMU-run slaagt**, met rawstatus 33 na
`BOOT:EBS_OK` en alle 28 vereiste regels in de strikte volgorde.
De zeven controles in `acceptance.json` zijn waar. Dit is eigen-OS-uitvoering
van een klassieke simulator in twee afzonderlijke ring-3-gebruikersprocessen.

| Waarneming | Werkelijke waarde |
|---|---:|
| Processen | PID 1 en PID 2 |
| Checkpoints | 253 per proces |
| Nuttige arena | 7.216 bytes per proces |
| Geteste arenagrens | 65.536 bytes per proces |
| Grootste werkafstand | 448 |
| Timerinterrupts | 4 |
| Syscall-aanroepen | 505 |
| Vrijgemaakte procesframes | 96 |

De bestaande qsim-core leverde de exacte Bell4096- en zesqubitreferenties.
Vooraf/middenin annuleren, een deterministische werkdeadline, invoergrenzen
en werkelijke allocatoruitputting worden door het gebruikersprogramma
getest; de kernel controleert het begrensde rapport en de frame-opruiming.

## Artifactbinding

| Artifact | SHA256 |
|---|---|
| Gebruikers-ELF | `3a20a6e8566b899ffec4939a03a9a7e5f970ca2f5b0c7d84fd85f5dccc860a5a` |
| Kernel-ELF | `84c75a9504e923426db1f86ac6d1e24b62d70024e4bebfe52b571b67106ac72d` |
| UEFI-loader | `0c62a346c778526e5ab1c865e9a21da4f7ab3d32f2218eba4ec85ef9afa84da9` |

De kernel bevat exact deze gebruikers-ELF. Het scalarbouwrapport controleert
4.096 instructies, vier int80-sites en vier software-floating-pointhelpers.
Het ongewijzigde qsim-core rekent zonder FPU-/SIMD-contextuitbreiding.
Disassemblycontrole is geen formeel controlflowbewijs. De rootbuild heeft
zijn eigen artifacthashes; hashes uit de eerdere agentworktree worden niet
als vervanging voor dit bewijs gebruikt.

`result.json`, `serial.txt` en `stderr.txt` komen rechtstreeks uit de echte
VM-map. `acceptance.json` bewaart het strikte verdict. `build-manifest.json`,
`quantum-build-report.json`, de disassembly en de commandologs binden bron,
compiler en artifacts. `i05-run-checks.json` bevat de bron-/HEADcontrole en
exacte omgevingskeuze. Firmwarehashes en de volledige QEMU-commandoregel staan
in het VM-resultaat. Binaries blijven buiten Git.

## Reproduceren

Vereist: de gepinde Rust 1.90.0-targets, rust-lld, GNU binutils, QEMU 7.2 en
het expliciet gepaarde OVMF CODE/VARS-paar. De gebruikte toolversies en hun
herkomst staan in `rust-tools.json` en `qemu-tools.json`.

```bash
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export CARGO_BUILD_JOBS=2
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_SDK_RUNTIME=0
export NOS_BUILD_DIR="$PWD/build/qsim-reproduce"
export NOS_TARGET_DIR=/tmp/nieuw-os-qsim-reproduce-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
python3 -B tools/build.py --kernel-feature native-qsim
python3 -B tools/run_qsim.py
```

Het werkelijke profiel was `pc-q35-7.2/qemu64/TCG/1CPU/256MiB`, zonder
gastnetwerk, met eigen firmwarekopieën en alleen-lezen ESP. De VM duurde
ongeveer 6,56 seconden; de ingestelde bovengrens was 60 seconden.

## Grenzen en roadmap

Dit is een vaste native selftest via functie-aanroepen, geen algemeen
ingediende IPC-job of gebruikersinterface. De werkdeadline is een
deterministisch controlegeval, geen bewezen realtimegarantie. Er is geen
fysieke QPU-run of hardwarecompatibiliteitsclaim. Hosttests zijn afzonderlijk
bewijs en worden niet bij deze ene native VM-run opgeteld.

L02 vereist de volledige native simulator/jobinterface. Voorstel: L02 blijft
open, +0 punten; de aantoonbare native port wordt als nieuwe mogelijkheid
gerapporteerd. De hoofdagent beheert het vaste scoreblad.
