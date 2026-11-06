# K06 F16/F17: blokkeren, wekken en doden zonder lost-wakeup (ADR-078)

**Datum:** 6 november 2026. **Branch:** `fix/k06-f16-f17-sched-protocol`, gestapeld op
`feat/k06-5b2-kernel-lock` (`59476e17`, met F3/F4 uit #237). Taakspec:
`docs/implementation/k06-steps/04-F16-F17-block-wake-kill.md`. **Bron:** vingerafdruk (v3)
`01daa964…` (volledig in `native-report.json`). **Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG
op de WSL-host. Geen CI, geen fysieke hardware, review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
25/25 gevallen over 20 profielen, allemaal op dezelfde bron, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6: `SMP:WAKE:PASS` en `SMP:KILL:PASS` met `CPUS=2/4/8/8`; met 1 CPU
  `SMP:WAKE:SKIPPED:CPUS=1` en `SMP:KILL:SKIPPED:CPUS=1` (geen AP, dus niets te bewijzen; geen
  PASS). Ping-pong: threads A en B (AP1/AP2; bij één AP op dezelfde CPU) geven elkaar 10.000 keer
  de beurt onder `lock::KERNEL`. Ze blokkeren via `block_prepare` onder KERNEL en worden gewekt
  via `RUNQ[d]` + 0xe8. Eis: beide tellers precies 10.000 binnen 20 s, beide threads daarna BLOCKED
  en geen fout gemeld. Kill: thread K, alleen in ruimte 1, draait op AP3 (`tid % AP's`). De BSP
  doet `kill_request` + 0xe8 en wacht op DEAD binnen `SHOOTDOWN_DEADLINE_MS` (100 ms). Daarna
  `on_cpu` = 0, `queued` = 0, `ACTIVE[1]` = 0 en geen voortgang meer na 10 ms. Daarna de normale
  STOP: elke AP thuis, geen ACTIVE-bit en geen lock meer.
- `test-smp-double-run` (4 vCPU's): de tweede push van thread 0 (op een tweede AP) faalt bij de
  push zelf → `SMP:SCHED:DOUBLE_ENQUEUE`, exit 0x10. Dat gebeurt vóór `SCHED = RUN`, dus geen AP
  kan eerder dispatchen: de uitkomst hangt niet meer van timing af (F17).
- `test-smp-resched-deaf` (4 vCPU's): K zet op AP3 TPR op klasse 0xe (0xd0 en 0xe8 worden
  tegengehouden) → `SMP:WAKE:PASS:CPUS=4`, `SMP:KILL:TIMEOUT:APIC=3`, `SMP:QUARANTINED:APS=3`,
  `SMP:ONLINE:1/4`, waarna de boot op de BSP verdergaat (M2:PASS).
- De 17 overige profielen geven ongewijzigd hun marker. De normale reeks heeft twee regels meer
  (`SMP:WAKE:…`, `SMP:KILL:…`), na `SMP:SCHED:PASS`.

## Hostmodellen (`cargo test -p smp-lock`, 24 unit + 1 doctest)
- Uitputtende interleavingmodellen (geen loom, zie ADR-078). Elke stap is één atomaire operatie
  van de echte functies in `sched.rs`; alle volgordes worden doorlopen.
  - `model_block_wake_never_loses_a_wakeup`: twee blokkerende threads op CPU 0/1 en een waker op
    CPU 2. Na afloop is geen gewekte thread BLOCKED, en een READY thread staat precies één keer
    in een queue.
  - `model_finds_the_lost_wakeup_of_the_single_phase_protocol`: hetzelfde model met het
    5d-1-protocol vindt de verloren wake (mutant, moet rood zijn).
  - `model_kill_reaps_only_after_the_stack_is_left`: kill op elk moment tijdens vier wissels van
    *d* (leave, pop/dispatch, reap, F1-claim, stub). De cleanup begint nooit terwijl een CPU op de
    stack staat of de thread nog in een queue staat. Binnen twee volle wissels na het verzoek is hij DEAD.
  - `model_finds_dead_published_on_the_stack`: een DEAD die in de wissel zelf wordt gepubliceerd
    (vóór de stub) wordt gevonden (mutant).
- Een dubbele push faalt deterministisch, in elke queue, en bij vier racende pushers wint precies
  één (200 rondes). Levenscyclus: kill vanuit elke levende toestand, nooit twee keer, nooit terug;
  `kill_commit` weigert zolang `on_cpu` staat.

## Eén-CPU-regressie (op dezelfde bron)
- `tools/test_milestones.py --through m3`: M2 8/8 groepen, M3 22/22 groepen. M3 bouwt de
  nieuwe `sched`-code mee, maar draait er niet op.
- `tools/build.py --kernel-feature native-rtc` + `tools/run_rtc.py --case all` geslaagd,
  `tools/test_supervisor.py` (K05, 6 stappen), `tools/test_resource_pressure.py` (K07 en qsim
  geaccepteerd).

## Host en statisch
- `cargo clippy --locked -p smp-lock --all-targets -- -D warnings` en `cargo fmt --all -- --check`
  schoon. `tools/tests/test_smp_serial.py` 13/13 (nieuw: WAKE/KILL verplicht, SKIPPED nooit als
  PASS, de dove kill moet in quarantaine en DOUBLE_RUN telt niet meer voor de double-run-controle),
  `test_gs_discipline.py` 8/8, `test_acpi_serial.py` groen.
- `tools/gate.sh` exit 0: cargo check, cargo test 518 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is niet uitgevoerd.

## Niet uitgevoerd en grenzen
- **Loom** (spec §4): bewust niet toegevoegd. Het zijn 33 crates met build scripts, en
  `generator 0.8.12` vereist Rust 1.95 terwijl de toolchain op 1.90 staat. De modellen hierboven
  dekken alleen sequentieel consistente uitvoeringen; zwakkere geheugenordening is niet
  gemodelleerd. Open punt F-T1.
- Geen koppeling met de echte M3-IPC-wachtrijen (stap 08), geen migratie (09). De kill raakt een
  kernelthread; K05-cleanup en procesbrede shootdown na DEAD volgen in 07/08.
- Het INV-5-pad is tot stap 05 "alle AP's in quarantaine"; met vastgehouden locks niet getest.
- CI: een gestapelde base start `native-smp.yml` niet. Geen hardware. TCG modelleert geen
  speculatie of echte geheugenordening.
- `tools/test_milestones.py --through m4` niet gedraaid (hardgecodeerd `build/users`-pad, PR #235).
