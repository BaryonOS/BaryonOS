# K06 stap 5d-2, voorwaarde 03: F5/F7 opgenomen in de 5e-stapel

**Datum:** 6 november 2026. **Branch:** `feat/k06-5d2-ring3-smp` (worktree `nieuw-os-k06-5d2`).
**Basis:** `65f3bc7e` (`feat/k06-5e-proc-shootdown`, PR #244) + merge van `bebfc1ed`
(`fix/k06-f5-f7-diagnostics`, PR #238). **Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de
WSL-host. Geen CI, geen hardware, review door AI. **Score:** 41 → 41/100.

**Correctie:** de mergecommit `fb7a23cd` miste de locknaam-correctie in de index (compileerde niet
voor `native-smp`); die staat in `4444eb52`. Het bewijs hieronder is op `fb7a23cd` + `4444eb52` gedraaid.

## Waarom
De L0-spec van 5d-2 (`08-5d-2-ring3-on-aps.md` §0) eist stap 01–07 en F8 in de stapel. Stap 03
(F5/F7) hing nog aan een oudere `feat/k06-5b2-kernel-lock` en zat niet onder 5e.

## Conflictoplossing
- `lock.rs`: F5/F7-spin (DG-1, ook release) + de 5c-2-releasehook (BT-4). Locknamen vervallen
  (F5/F7); ook de nieuwe locks uit F19/5c-2/5e (`ICR`, `SHOOTDOWN`, `IDLE_LOCK`, testruimten).
- `smp.rs`: één TSC-basis. De dubbele `TSC_PER_MS`/`rdtsc`/`SHOOTDOWN_DEADLINE_MS` van F5/F7
  vervallen; `calibrate_tsc` (10 ms, vóór de eerste IPI) blijft als begrenzing tijdens de bringup,
  `bsp_timer_init` verfijnt die over 100 ms vóór GO (BT-5). De shootdownronde volgt 5e (`round`).
- Oracle: volgorde `RATIO`, `SERIAL:PASS`, `SYNC`. In de ongelockte controle blijft de
  RATIO-regel staan (die valt vóór GO, vóór enige fixtureregel).
- `test_gs_discipline.py`: accepteert `pub(crate) mod smp;` (F5/F7), de cfg-poort blijft verplicht.

## Uitgevoerd (op deze bron)
| Controle | Resultaat |
|---|---|
| alle 27 SMP-profielen (`build.py` + `run_smp.py --profile`, 3 parallel) | 26/27; `native-smp` faalde `smp-one` met `SMP:CLOCK:DRIFT:US=439379` (1%-band) onder de last van 3 parallelle QEMU's |
| `native-smp` opnieuw, alleen | 6/6 (1, 2 single-thread TCG, 4, 8, 9 vCPU, dubbele MADT) |
| `test_smp_serial.py`, `test_gs_discipline.py` | 21/21, 8/8 |
| `cargo test -p smp-lock`, `cargo fmt --all -- --check` | exit 0 |
| `tools/test_milestones.py --through m3` | M2 + M3 22 groepen |
| `test_supervisor.py` (K05), `test_resource_pressure.py` (K07), `run_rtc.py --case all` | exit 0 |
| `tools/gate.sh` | exit 0 (520 geslaagd, 1 ignored; `qemu-smoke-test.sh` ontbreekt) |

## Niet uitgevoerd en open
- CI, hardware, menselijke audit, M4 (#235). K05 en K07 draaiden tegelijk met een gedeelde
  Cargo-targetmap (Cargo-lock serialiseert de builds).
- De klokdrift onder hostlast is niet verder onderzocht: de 1%-band van `SMP:CLOCK` is gevoelig
  voor parallelle TCG-runs. Native acceptatie draait de profielen daarom na elkaar.
