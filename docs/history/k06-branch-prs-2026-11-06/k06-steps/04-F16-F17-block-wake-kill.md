# L0-specificatie — K06 F16/F17: blokkeren, wekken en doden over CPU's heen

**Datum:** 6 november 2026 · **Basis:** de head van stap 02 (F3/F4) [nog niet gebouwd]
**Branch/worktree:** `fix/k06-f16-f17-sched-protocol` in `/home/simon/projects/nieuw-os-k06-f16`
**Spec:** `K06-SMP.md` §2 (INV-3, INV-5, INV-7), §6.1, §6.3, §6.4; ADR-053 (IPC-wait), ADR-074;
auditbevindingen F16, F17 en F1 (het `on_cpu`-protocol uit #230 blijft de basis).

## 1. Probleem (waarom nu)
- **F16:** `crates/smp-lock/src/sched.rs` kent geen overgang RUNNING→BLOCKED, geen DEAD of
  DEAD_PENDING, geen kill en geen idle-thread. De IPC-route uit spec §6.4 heeft een
  **lost-wakeup-race**: een thread besluit te blokkeren, terwijl een waker op een andere CPU al
  wekt. `wake` faalt dan met `Err(RUNNING)` en het antwoord gaat verloren.
- **F17:** de runqueue herkent een dubbele enqueue niet. De `double-run`-test leunt daardoor op
  timing en niet op het invariant.

## 2. Invarianten (normatief)
- **BW-1 Geen verloren wake.** Blokkeren gaat in twee fasen: `RUNNING(c) → BLOCKING` (de
  wachtvoorwaarde is nog niet waar) en daarna, bij de wissel, `BLOCKING → BLOCKED`. Een `wake` die
  `BLOCKING` ziet, zet `BLOCKING → READY_PENDING`. De wissel ziet dat en zet de thread terug op
  READY in plaats van BLOCKED. Alle overgangen gaan met `compare_exchange`.
  *Weerlegd als* het loom-model een uitvoering vindt waarin een wake gezien is en de thread toch
  BLOCKED eindigt.
- **BW-2 Eén keer in de queue.** Per thread een `queued`-bit: `push` doet CAS 0→1, `pop` 1→0. Een
  tweede push → `SMP:SCHED:DOUBLE_ENQUEUE` en stop. De double-run-test faalt daardoor deterministisch
  bij de push.
- **BW-3 Kill is begrensd.** Kill van een thread op CPU *d*: zet `DEAD_PENDING`, stuur 0xe8 en wacht
  op `DEAD` binnen `SHOOTDOWN_DEADLINE_MS`. *d* publiceert `DEAD` bij zijn eerstvolgende wissel, ná
  het verlaten van de stack (`on_cpu` gewist, F1). Geen `DEAD` binnen de deadline → INV-5-pad
  (stap 05 bepaalt het beleid). Pas na `DEAD` mogen de K05-cleanup en de shootdown (07) lopen.
- **BW-4 Idle per CPU.** Elke CPU heeft een vaste idle-thread die nooit in een queue staat. Is de
  queue leeg, dan draait idle op de kernel-CR3 (spec §6.3 stap 9).
- **BW-5 Geen lock over de wissel** (INV-7, INV-5c): ongewijzigd. Alleen RUNQ wordt binnen de wissel
  overgedragen.

## 3. Interface
- `smp_lock::sched`: toestanden `READY, RUNNING(cpu), BLOCKING, READY_PENDING, BLOCKED,
  DEAD_PENDING, DEAD`; functies `block_prepare`, `block_commit`, `wake`, `kill_request`,
  `kill_commit`; `RunQueue::push` controleert `queued`.
- `smp.rs`: `sched_switch` handelt `BLOCKING`, `READY_PENDING` en `DEAD_PENDING` af. Een idle-thread
  per CPU in `CpuLocal`.
- Kernelfeature `test-smp-resched-deaf` (de doel-AP negeert 0xe8 tijdens een kill).

## 4. Hosttests (vóór native)
- Een loom-model van block/wake met twee threads en een waker op een andere "CPU": geen lost wakeup.
- Een loom-model van kill tijdens RUNNING: de cleanup start nooit vóór `DEAD`, en `DEAD` komt pas na
  het wissen van `on_cpu`.
- Een dubbele push faalt altijd, ongeacht de timing.

## 5. Native fixture en negatieve tests (`native-smp`, kernelthreads)
- Ping-pong: thread A op AP1 en thread B op AP2 blokkeren en wekken elkaar 10.000× via een gedeelde
  "brievenbus" onder `KERNEL`. Eis: beide tellers 10.000, geen timeout. Marker `SMP:WAKE:PASS`.
- Kill: de BSP doodt een draaiende thread op AP3; `DEAD` binnen de deadline, de stack is vrij en
  `active_cpus` van zijn ruimte is 0. Marker `SMP:KILL:PASS`.
- `test-smp-double-run` (bestaand): verwacht nu `SMP:SCHED:DOUBLE_ENQUEUE` bij de push, in elke run.
- `test-smp-resched-deaf`: kill naar een dove AP → het INV-5-pad volgens 05 (tot 05 af is:
  `SMP:KILL:TIMEOUT:APIC=<n>` gevolgd door quarantaine).

## 6. Schrijfbestanden
`crates/smp-lock/src/sched.rs`, `crates/smp-lock/src/lib.rs`, `kernel/src/m2/smp.rs`,
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (nieuwe ADR: schedulerprotocol),
`docs/implementation/K06-SMP.md` (§6.1 en §6.4 bijwerken), `docs/PROJECT-STATE.md`, voortgangsbestanden.

## 7. Acceptatie
- `tools/gate.sh` exit 0; `cargo test -p smp-lock` (inclusief loom); clippy; fmt; `test_smp_serial.py`.
- Native: `native-smp` (alle gevallen), `test-smp-double-run`, `test-smp-resched-deaf`, regressie
  `milestone-m2`.
- K06 blijft open (41/100).

## 8. Buiten scope
De koppeling met de echte IPC-wachtrijen van M3 (08), migratie en load balancing (09), ring 3.
