# K06 stap 5d-1 — runqueues per AP, 0xe8-wekken en CR3-wissel

**Bron:** `7079f788` op `feat/k06-5d-runqueues`, de 5d-1-code `fe32e308` plus een merge van
`feat/k06-5c-preempt-lock` (`b593930c`, PR #228, met de fix voor het spin-irq-off-profiel).
**Datum:** 6 november 2026. **Besluit:** aanvulling stap 5d-1 in ADR-074. **Bewijsniveau:**
native QEMU/TCG op deze WSL-host, geen CI-run, geen fysieke hardware, review door AI.
**Score:** 41 → 41/100. K06 blijft open, want 5b-2, 5c-2, 5d-2, 5e en 5f ontbreken.

## Gerealiseerd
- `smp_lock::sched`: een FIFO-runqueue met vaste capaciteit en een statuswoord per thread.
  Dispatch is een CAS READY → RUNNING(cpu) (INV-3). Host-tests dekken FIFO, de grens, de
  overgangen en concurrente dispatch met precies één winnaar.
- In de timer van elke AP: na het 5c-uitstel kiest `sched_switch` onder RUNQ de volgende
  thread uit de eigen queue. De adresruimte wisselt via `ACTIVE |= c` → `mov cr3` →
  `ACTIVE &= !c` (INV-4). 0xe8 is de wekvector.
- Fixture `SMP:SCHED:PASS:CPUS=n`: twee kernelthreads per AP, één per adresruimte
  (PML4-kopieën van de kernel-root). De tweede thread wordt cross-CPU gewekt met 0xe8. Eisen:
  voortgang, `resched ≥ 1`, ten minste 4 CR3-wissels per AP, en elke thread ziet zijn eigen CR3
  en `ACTIVE`-bit. Na STOP staat iedere AP thuis onder de kernel-CR3, is `ACTIVE` leeg, is
  lockdep leeg en staat de voortgang 10 ms stil.
- Negatief: `test-smp-double-run` zet thread 0 in twee queues. Dat geeft
  `SMP:SCHED:DOUBLE_RUN`, exit 0x10.

## Native gevallen (18/18 geslaagd, geen time-out)
`native-smp` 6/6 (1, 2, 4, 8 en 9 vCPU's, dubbele MADT), `test-smp-double-run` 1/1, en de
eerdere profielen elk 1/1: held-across-switch, lockdep-order, spin-irq-off, ap-timer-dead,
shootdown-skipped, ap-shootdown-deaf, ap-ipi-deaf, ap-nmi, ap-double-fault, ap-stall en
ap-mismatch. De SMP-regels, hashes en exitcodes staan per geval in
[native-report.json](native-report.json). Alle profielen hebben bronfingerprint `f76bde09…`.

## Commando's
```
export NOS_TARGET_DIR="$PWD/target/k06"
export NOS_BUILD_DIR="$PWD/build/k06-<profiel>"
python3 -B tools/build.py --kernel-feature <profiel>
python3 -B -u tools/run_smp.py --profile <profiel>
```
Daarnaast: `tools/gate.sh` exit 0 (505 geslaagd, 1 ignored; de QEMU-smoke ontbreekt in de repo
en is dus niet uitgevoerd), `python3 -B tools/test_milestones.py --through m2` (8 groepen
geslaagd, één CPU), `python3 -B tools/tests/test_smp_serial.py` 9/9, `cargo test` en
`clippy -D warnings -p smp-lock`, en `cargo fmt --check` voor de workspace en de kernel.

## Grenzen
- Kernelthreads, geen ring-3. Userprocessen op AP's (5d-2) vereisen eerst 5b-2 (INV-2).
- De BSP is nog geen schedulerdoel (5c-2). Er is geen migratie, kill over CPU's heen of
  procesbrede shootdown (5e/5f).
- QEMU-machine `pc-q35-10.0` van de host; CI pint `NOS_QEMU_MACHINE` mogelijk anders.
