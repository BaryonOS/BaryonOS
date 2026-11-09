# K06 auditfixes F1 en F2: on_cpu-overdracht en LIFO-lockdep

**Datum:** 6 november 2026. **Branch:** `fix/k06-f1-f2-lifo-oncpu`, gestapeld op
`feat/k06-5d-runqueues` (`76c67f22`). **Bronfingerprint:** `b082ee5f…` (v3).
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Wat is gerepareerd
- **F2:** `Lockdep::release` eist LIFO (`level == highest()`). Bij een release buiten
  volgorde geeft `IrqGuard::drop` `SMP:LOCKDEP:LIFO:<held>-><released>` en stopt, vóórdat IF
  wordt hersteld. Hosttest: `lockdep_rejects_out_of_order_release`.
- **F1:** `THREAD_ON_CPU` per thread. De timerstub wist de vlag van de vorige thread pas na
  `mov rsp` (adres in RDX via `Resume`). `sched_switch` claimt de vlag van de volgende thread
  met een CAS en wacht nooit. Lukt de claim niet, dan volgt READY, de thread komt achter in de
  queue en deze tick draait thuis. Hosttest: `stack_is_never_shared_after_early_ready`
  (200 rondes, een model met een thread die op de stack werkt).

## Native gevallen (18/18 geslaagd, geen time-out)
Alle 13 SMP-profielen: `native-smp` 6/6 en elk negatief of controleprofiel 1/1, waaronder
`double-run` (`SMP:SCHED:DOUBLE_RUN`), `held-across-switch` en `lockdep-order`. Herhaald:
`double-run` 10/10, `four`, `eight` en `two-single` elk 5/5. Per geval staan de SMP-regels,
exitcode en manifesthash in [native-report.json](native-report.json). Alle 18 gevallen
hebben `kernel_sha256: null`: kernelbinding is dus niet via dit rapport te controleren.
Daarvoor zijn de oorspronkelijke manifesten en bijbehorende kernelartifacts nodig,
voor zover die elders bewaard zijn; hun opslaglocatie is hier niet vastgelegd.

## Commando's
```
export NOS_TARGET_DIR="$PWD/target/k06"
export NOS_BUILD_DIR="$PWD/build/k06-<profiel>"
python3 -B tools/build.py --kernel-feature <profiel>
python3 -B -u tools/run_smp.py --profile <profiel>
```
Daarnaast: `tools/gate.sh` exit 0 (507 geslaagd, 1 ignored, 28 s). De QEMU-smoke ontbreekt
in de repo en is dus niet uitgevoerd. De gate compileert de `native-smp`-kernel niet; dat
gebeurde via `tools/build.py` hierboven. Verder: `cargo test -p smp-lock` (14),
`clippy -D warnings -p smp-lock`, `cargo fmt --check` (workspace en kernel) en
`tools/tests/test_smp_serial.py` OK.

## Grenzen
- Geen negatief native profiel voor `SMP:LOCKDEP:LIFO`: de kernel heeft nu geen pad dat buiten
  volgorde vrijgeeft. Het afwijzen is alleen op de host getest.
- F1 is native niet deterministisch geïnjecteerd. De race (claim mislukt) treedt alleen op
  bij een dubbele queue of migratie. Die zijn er nu niet, behalve in het double-run-profiel.
- De double-run-detectie blijft timingafhankelijk (F17 open). F3–F21 uit de audit zijn
  niet behandeld.
