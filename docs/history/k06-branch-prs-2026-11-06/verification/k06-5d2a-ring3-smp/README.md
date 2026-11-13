# K06 stap 5d-2a: M3-ring-3-processen op elke online CPU (ADR-081)

**Datum:** 6 november 2026. **Branch:** `feat/k06-5d2-ring3-smp`, gestapeld op
`feat/k06-5e-proc-shootdown` (PR #244) met F5/F7 (PR #238) erin opgenomen
(`docs/verification/k06-5d2-f5f7-integration/`). Taakspec: `docs/implementation/k06-steps/08-5d-2-ring3-on-aps.md`.
**Broncommit:** `efe75124`. **Bron:** vingerafdruk (v3) `76264ef7…` (volledig in `native-report.json`).
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Wat is aangetoond
Het basis-M3-manifest (16 ring-3-processen, elk met eigen adresruimte) draait verdeeld over alle
online CPU's. Elke CPU doet ring-3-werk (M3-ticks vanuit CPL3) en wisselt zelf van proces.
Voorbeeld `m3-eight` (8 vCPU's):
```
SMP:M3:GO:CPUS=8
SMP:M3:CPU=0:USER_TICKS=64:SWITCHES=70   ... CPU=7:USER_TICKS=62:SWITCHES=69
SMP:SCHED:CPUS=8
SMP:M3:CLEANUP:PASS
SMP:M3:PASS:CPUS=8
```
De oracle (`run_smp.py`, `m3_end`) eist per online CPU `USER_TICKS > 0` en `SWITCHES >= 4`,
`SMP:SCHED:CPUS=n`, cleanupbalans (elk frame dat een proces bezat is terug) en alle bestaande
SMP-zelftests vooraf. Elke run eindigt met een stop-the-world: de andere CPU's parkeren met de
kernel-CR3, daarna volgt de cleanup.

**Negatief (K03 onder SMP), `test-smp-m3-fault` op 4 vCPU's:** het proces in slot 2 (thuis AP2)
voert `ud2` uit. Alleen dat proces stopt (`SMP:M3:FAULT:UD:CPU=2`). Bij de cleanup gaat er een
kernelronde naar alle CPU's terwijl die doordraaien. De overige 15 processen lopen op alle 4
CPU's door tot de eindvoorwaarde (`SMP:M3:K03:PASS`).

## Uitgevoerd op `efe75124` (na elkaar, één QEMU tegelijk)
| Controle | Resultaat |
|---|---|
| alle 29 SMP-profielen (`build.py` + `run_smp.py --profile`), 37 gevallen | 37/37 |
| waarvan `native-smp-m3` | 4/4: 2 vCPU's, 4 single-thread TCG, 4, 8 |
| waarvan `test-smp-m3-fault` | 1/1 (4 vCPU's) |
| `test_smp_serial.py` (oracle, nieuw: M3-fase met 9 weerleggingen) | 22/22 |
| `test_gs_discipline.py` (`m3_ap_timer` als beoordeelde stub) | 8/8 |
| `cargo test -p smp-lock`, `cargo fmt --all -- --check` | exit 0 |
| `test_milestones.py --through m3` (één CPU) | M2 8 en M3 22 groepen |
| `test_supervisor.py` (K05) | 6/6 stages |
| `test_resource_pressure.py` (K07) | accepted (2e poging in een verse `NOS_BUILD_DIR`; de 1e weigerde een bestaande uitvoermap) |
| `run_rtc.py --case all`, `run_ipc_wait.py`, `run_fpu.py` (+ `test-fpu-wait`) | exit 0 |
| `tools/gate.sh` | exit 0 (520 geslaagd, 1 ignored; `qemu-smoke-test.sh` ontbreekt) |
| kernel-clippy `native-smp` en `milestone-m3` | gelijk aan de basis `4444eb52` |

## Stabiliteit (eerlijk gemeld)
- **Race gevonden en hersteld** (`efe75124`): twee CPU's konden tegelijk stoppen en op elkaar
  wachten (`SMP:M3:STOP:TIMEOUT` op `m3-two`). Nu gaat de stopvlag op onder `KERNEL`.
- **Klokzelftest (5c-2) is timinggevoelig:** `SMP:CLOCK` eist 1% op 500 PIT-ticks. In herhaalde
  runs faalde die soms met 1–9% afwijking, ook op de basis zonder 5d-2: basis `eight` 1 van 10,
  nieuw `eight` 1 van 6 in een afwisselende A/B-reeks. In een eerdere reeks faalde nieuw 4 van 5
  en ook M3-gevallen (`m3-two`, `m3-eight`, `m3-fault-ap2`) in dezelfde zelftest. Een M3-fase is
  daarbij nooit mislukt. De band is niet verruimd (ADR-080). Dit is een open punt voor 09.

## Niet uitgevoerd en open
- CI op de gestapelde PR, fysieke hardware, menselijke audit, M4 (#235).
- Spec 08 §4 nog niet: FPU-canary (R3-3), IPC-ping-pong over CPU's (R3-5), usercopy onder
  `ASPACE` (R3-6), K05 op SMP (R3-7), Ring-3-GS N1-N3 met M3-processen. Zie ADR-081 "Open".
- Shortcuts: geen spawn na `M3_GO`; een kernelronde zonder ack stopt de VM (geen quarantaine plus
  migratie); vaste LAPIC-telling voor de AP-tick.
