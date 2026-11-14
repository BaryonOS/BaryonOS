# K06 stap 5b-2: KERNEL-lock om de M3-entry en scratchslot per CPU (ADR-075)

**Datum:** 6 november 2026. **Branch:** `feat/k06-5b2-kernel-lock`, gestapeld op
`fix/k06-f8-swapgs` (`7ffdacfa`). **Bron:** `48dcae6a`, bronvingerafdruk `7ede30a1…` (v3).
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
22/22 gevallen over 17 profielen, schone werkboom op `48dcae6a`, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6: `SMP:SCRATCH:PASS:CPUS=1/2/4/8/8`. Daarbij nult, vult en controleert elke
  CPU 1024 keer zijn eigen frame via zijn eigen slot, tegelijk met de anderen, en neemt hij
  1000 keer `KERNEL` voor een teller met losse load en store.
- Controle `test-smp-shared-scratch` (alle CPU's in slot 0, 4 vCPU's):
  `SMP:SCRATCH:CROSS:CPU=n`, exit 0x10. Lokaal 12/12 opeenvolgende runs rood zoals bedoeld.
  Op `f6912a63` gaf één run nog een afgekapte foutregel van een tweede CPU (#PF op het gedeelde
  slot). Sinds `48dcae6a` telt die #PF alleen in de controle ook als crossing.
- Alle bestaande negatieve en controleprofielen geven ongewijzigd hun marker.
- Extra: `four` en `eight` elk 5 keer opnieuw, 10/10 geslaagd.

## Eén-CPU-regressie (K-3)
Op de werkboom van `f6912a63` vóór `cargo fmt`. Daarna zijn alleen de opmaak en een
controle-only-cfg in `smp.rs` veranderd.
- `tools/test_milestones.py --through m4`: M2 8/8, M3 22/22 (incl. de rondes via
  `complete_round`/`resume()`), M4 12/12.
- `tools/test_supervisor.py`: K05 gate, 6 stappen. `tools/test_resource_pressure.py`: K07 en qsim
  geaccepteerd.
- `tools/run_rtc.py --case all` 5/5, `tools/run_ipc_wait.py`, `tools/run_fpu.py` (`native-fpu`
  en `test-fpu-wait`).
- Op de schone commit `48dcae6a`: `tools/test_memory_system.py`, 5 stappen.
- Alle 198 `KERNEL_FEATURES` uit `tools/build.py` bouwen (`native-update`/`test-update-cut`
  met `--image-version 2`).

## Statisch en host
`tools/gate.sh` exit 0 (cargo check; cargo test 507 geslaagd, 1 ignored; QEMU-smoke ontbreekt in
de gate). `cargo +1.90.0 fmt --all -- --check` en in `kernel/` schoon. `cargo +1.90.0 clippy
--locked -p smp-lock --all-targets -- -D warnings` schoon. `tools/tests/test_smp_serial.py` 11/11.

## Grenzen
- De 500-ms-deadline begint na het BSP-deel van de fixture.
- Idle met `sti; hlt` staat nog binnen de KERNEL-sectie, M3 draait niet op AP's (5c-2/5d-2).
- TCG modelleert geen speculatie. Een speculatief gecachete vreemde slot-entry is buiten dit bewijs.
- `tools/test_milestones.py:160` gebruikt een hardgecodeerd `build/users`-pad (M4 faalt met een
  eigen `NOS_BUILD_DIR`); losse taak.
