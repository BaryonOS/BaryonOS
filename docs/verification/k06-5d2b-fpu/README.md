# K06 stap 5d-2b: FPU-context per CPU en FPU-canary (R3-3, aanvulling ADR-081)

**Datum:** 6–7 november 2026. **Branch:** `feat/k06-5d2-ring3-smp` (PR #247). Taakspec:
`docs/implementation/k06-steps/08-5d-2-ring3-on-aps.md` §2 R3-3 en §4 FPU-canary.
**Broncommit:** `ca0b7e5f`. **Bron:** vingerafdruk (v3) `21b4bf64…` (volledig in
`native-report.json`). **Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI,
geen hardware, review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Wat is aangetoond
Elk SMP-proces is een FPU-canary (`userspace/probe.rs`, modus 19). Het zet `st0`, `xmm0` en
`xmm15` op waarden uit zijn eigen patroon en controleert ze bij elke lusronde. Per CPU draaien 2
tot 8 canaries met verschillende waarden. De run stopt pas als **elke CPU 1000 proceswissels**
heeft gedaan. Daarna leest de kernel in elk opgeslagen FXSAVE-beeld ST0, XMM0 en XMM15 terug en
vergelijkt ze met het patroon van dat proces:
```
SMP:M3:CPU=0:USER_TICKS=994:SWITCHES=1000 ... CPU=7:USER_TICKS=993:SWITCHES=1000
SMP:SCHED:CPUS=8
SMP:M3:FPU:PASS:PROCESSES=16
```
**Controle `test-smp-m3-fpu-skip`:** dezelfde kernel zonder de FXRSTOR in het SMP-exitpad. De
canary slaat direct na de start alarm (`SMP:M3:FPU:CORRUPT:CONTROL`). De test meet dus een echt
gevaar.

## Uitgevoerd op `ca0b7e5f` (na elkaar, één QEMU tegelijk)
| Controle | Resultaat |
|---|---|
| alle 30 SMP-profielen, 38 gevallen (nieuw: `m3-fpu-skip-control`) | 38/38 |
| `native-smp-m3` (2, 4 single-thread TCG, 4 en 8 vCPU's), `test-smp-m3-fault` | 4/4, 1/1 (AP2-fault: 15 canaries intact) |
| oracle `test_smp_serial.py` (nieuw: <1000 wissels, geen FPU-bewijs, corrupte canary) | 22/22 |
| `test_gs_discipline.py`, `cargo test -p smp-lock`, `cargo fmt --check` | groen |
| `test_milestones.py --through m3` (één CPU, met de gewijzigde probe) | M2 8 en M3 22 groepen |
| K05 (`test_supervisor.py`), K07 (`test_resource_pressure.py`), `run_rtc.py`, `run_ipc_wait.py`, `run_fpu.py` (+ `test-fpu-wait`) | groen |
| `tools/gate.sh` | exit 0 (520 geslaagd, 1 ignored; `qemu-smoke-test.sh` ontbreekt) |
| kernel-clippy `native-smp`, `milestone-m3`, `native-smp-m3` | geen nieuwe soorten meldingen |

## Niet uitgevoerd en open
- CI, hardware, menselijke audit, M4 (#235).
- Spec 08 §4 rest: IPC-ping-pong over CPU's (R3-5; vraagt M4 en `ipc-wait` op SMP), usercopy
  onder `ASPACE` (R3-6), K05 op SMP (R3-7), Ring-3-GS N1-N3 met M3-processen. Zie ADR-081.
- De `SMP:CLOCK`-zelftest (5c-2) blijft onder TCG timinggevoelig; in deze run geen afwijking.
