# Actuele projectstatus

## NEXT AGENT HANDOFF — K06 stap 5e, procesbrede TLB-shootdown op `active_cpus` (aanvulling ADR-072/-075) — 6 november 2026

**Bron:** `730ce80a` op `feat/k06-5e-proc-shootdown` (worktree `nieuw-os-k06-5e`), gestapeld op
`feat/k06-5c2-bsp-timer` (`0d197dc1`). Taakspec: `docs/implementation/k06-steps/07-5e-process-shootdown.md`
(L0, branch `docs/k06-step-specs`, PR #232). **41 → 41/100**; K06 blijft open.

**Gerealiseerd:** PS-1 `shootdown_process(space, start, pages)`: snapshot `ACTIVE[space] & !self`
na `fence(SeqCst)`. PS-2: één gepubliceerde ronde `{root, start, pages, gen}` onder `SHOOTDOWN`
(niveau 50). De 0xe0-handler staat nu in Rust: flush alleen bij de eigen CR3 of de kernelhelft
(≤ 32 pagina's `invlpg`, anders een CR3-reload), daarna altijd een ack en een teller per CPU.
De deadline loopt op de TSC, dus ook een AP kan een ronde starten (de ronde van AP1 naar de BSP uit
5c-2 gebruikt dezelfde functie). PS-3: `Paging::unmap_shot` (PTE wissen → ronde → metadata vrij).
PS-4: `shootdown_kernel` naar `ONLINE_CPUS`. De kernelhelft is PDPT[0], gedeeld via de pointer;
asserts houden dat zo. PS-5: `exit_space` met een ronde over de hele user-GiB, regels "levende ruimte
noemt geen vrij frame" en "`ACTIVE` van een dode ruimte is 0". Echte testruimten (`build_spaces`)
vervangen de PML4-kopieën. Fixture na ICR: `SMP:ISOLATION` (10 000×) en `SMP:UNMAP_RACE`
(gecontroleerde #PF via fixup; doelset per CPU geteld). Controles `test-smp-proc-shootdown-deaf`
en `test-smp-proc-no-shootdown`, ook in `native-smp.yml`. `smp-lock::sched::switch_space` krijgt
een hostmodel met nep-CR3. IQ-3 `held_by` kent nu ook ASPACE en SHOOTDOWN.

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG, bron `fdc7ab04…`):** zie `docs/verification/k06-5e-proc-shootdown/`. SMP 30/30 over 25 profielen; M2 8/8 en M3 22/22; `cargo test -p smp-lock` 26+1 (hostmodel, rood met de verkeerde volgorde); oracle + GS-lint 25/25; `tools/gate.sh` exit 0 (520 geslaagd, 1 ignored; QEMU-smoke-script ontbreekt).
**Niet uitgevoerd:** CI op de gestapelde PR, hardware, menselijke audit, M4 (pad-bug, #235).

**Open:** het M3-pad (`m3.rs`-cleanup, engine-`check_invariants`) is niet gewijzigd, omdat M3 niet
onder `native-smp` draait (`compile_error`); dat volgt met 08 (5d-2). Er is nog geen
rechtenverlagings-API (er is geen aanroeper). FRAMES is nog geen lock. `feat/k06-5c2-bsp-timer`
stond bij de start niet op de remote; deze branch draagt die commits mee.
Volgende stap: 08 (5d-2, ring-3-processen op AP's).
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-proc-shootdown-deaf"; python3 -B tools/build.py --kernel-feature test-smp-proc-shootdown-deaf && python3 -B -u tools/run_smp.py --profile test-smp-proc-shootdown-deaf`.

## Historisch checkpoint — K06 stap 5c-2, BSP-timer, PIT-klok, idle en reschedule-vlag (ADR-080) — 6 november 2026

**Bron:** `5020b904` op `feat/k06-5c2-bsp-timer` (worktree `nieuw-os-k06-5c2`), gestapeld op
`feat/k06-5b2-kernel-lock` (`068fd857`, met F3/F4, F16/F17 en F19/F15). Taakspec:
`docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md`; ontwerp ADR-080. **41 → 41/100**;
K06 blijft open.

**Vastgesteld:** `native-smp` sluit M3 uit, dus daar bestonden geen PIC-programmering en geen Sys3;
de BSP draaide volledig met IF=0. De PIT-IRQ0-klok is daarom in `native-smp` zelf gebouwd.

**Gerealiseerd:** BT-1: `bsp_timer_init` (vóór `GO`) remapt de 8259 (alleen IRQ0 → 0x20), PIT-kanaal
0 op ~1 kHz, IRQ0-stub telt `CLOCK_TICKS` en stopt bij een AP. BT-2: 0xd0 op de BSP (deler 16),
INV-5c-uitstel en `bsp_switch`; `CpuLocal.irq_ready`; vensterpredicaat in `lock.rs`
`level <= ASPACE && irq_depth == 0 && irq_ready`. BT-3: `idle_halt` (lege lockdep, kernel-CR3).
BT-4 (F11): 0xe8 telt `resched_count` en zet `need_resched`, geen tick meer; een uitgestelde 0xe8
wordt bij de release van de laatste lock een self-0xe8 (`resched_on_release`). BT-5 (F9):
LAPIC/PIT- en TSC-kalibratie, `SMP:TIMER:RATIO`, band [50 000, 75 000] vooraf in ADR-080. BT-6:
contract kernelthreads in ADR-080. Fixtures `SMP:CLOCK`, `SMP:BSP_PREEMPT`, `SMP:BSP_SPIN`,
`SMP:IDLE:WAKE_US` + `SMP:IDLE` (na SPIN, vóór SCHED); controle `test-smp-bsp-spin-irq-off`, ook in
`native-smp.yml`.

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG, bron `412e6740…`):** zie
`docs/verification/k06-5c2-bsp-timer/`. SMP 28/28 over 23 profielen (ratio 62 559–62 871,
wake 39–248 µs); rode mutant zonder release-hook → `SMP:IDLE:RESCHED_LOST`.
M2 8/8 en M3 22/22, rtc 5/5, K05 6/6; `cargo test -p smp-lock` 24+1, `-p boot-protocol` 28,
clippy en fmt schoon, oracle 16/16, GS-lint 8/8; `tools/gate.sh` exit 0 (518 geslaagd, 1 ignored;
QEMU-smoke-script ontbreekt).
**Niet uitgevoerd:** CI op de gestapelde PR, hardware, M4 (pad-bug, #235), kernel-clippy (faalt al
op de basis, geen nieuwe meldingen).

**Open:** `ADOPTED` (ADR-079) wacht nog steeds op een BSP-runqueue (5d-2 of 5f); TSC als constant
aangenomen (hardware: invariant-TSC-controle); de klokafwijking wordt bij PASS niet gelogd.
Volgende stap: 07 (5e, procesbrede shootdown op `active_cpus`).
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-bsp-spin-irq-off"; python3 -B tools/build.py --kernel-feature test-smp-bsp-spin-irq-off && python3 -B -u tools/run_smp.py --profile test-smp-bsp-spin-irq-off`.

## Historisch checkpoint — K06 F19/F15, ICR-lock per CPU en quarantaine stopt bij een gehouden lock (ADR-079) — 6 november 2026

**Bron:** branch `fix/k06-f19-f15-icr-quarantine` (worktree `nieuw-os-k06-f19`), gestapeld op
`feat/k06-5b2-kernel-lock` (`81b648ba`, met F16/F17 uit #239). Taakspec:
`docs/implementation/k06-steps/05-F19-F15-icr-quarantine.md`; ontwerp ADR-079. **41 → 41/100**;
K06 blijft open.

**Gerealiseerd:** IQ-1: `CpuLocal.icr` (niveau 60, irqsave) om idle-wacht, ICR-hoog, ICR-laag en
afsluitende wacht in `send_ipi`. IQ-2: `IrqLock.owner` (CPU-index of `FREE`) en `owner()`/`level()`.
IQ-3: `quarantine_all` controleert eerst alle AP's, daarna per AP `quarantine_cpu`: controle →
INIT → 1 ms → hercontrole → READY-threads naar `ADOPTED`, de lopende thread `crash`
(`kill_request`). Een gehouden KERNEL/RUNQ geeft `SMP:QUARANTINE:LOCK_HELD:<niveau>:CPU=<n>` en
een stop. IQ-4: `quarantine_cpu` op een AP stopt. Native fase `icr` na LOCKPAIR:
`SMP:ICR:PASS:ROUNDS=10000:CPUS=n` (onder twee AP's `SKIPPED`). Nieuwe profielen
`test-smp-icr-race` en `test-smp-quarantine-lock-held`, ook in `native-smp.yml`.

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG, bron `dc44d588…`):** zie
`docs/verification/k06-f19f15-icr/`. SMP 27/27 over 22 profielen, alle bestaande dove-AP-profielen
ongewijzigd. M2 8/8 en M3 22/22, rtc, K05, K07;
`cargo test -p smp-lock` 24+1, clippy en fmt schoon, oracle 15/15, GS-lint 8/8; `tools/gate.sh` exit 0
(518 geslaagd, 1 ignored; QEMU-smoke-script ontbreekt).
**Niet uitgevoerd:** CI op de gestapelde PR, hardware, M4 (pad-bug, #235).

**Open:** een AP die als *wachter* in een ticketlock sterft, wordt door IQ-3 niet gezien
(alleen eigenaren); `ADOPTED` wacht op de BSP-runqueue (5c-2); de K05-route bij een crash geldt
pas voor ring-3-threads (5d-2); ASPACE/FRAMES/SHOOTDOWN moeten in `held_by` zodra ze locks zijn;
of een AP in 08 zelf een shootdown start (IQ-4). Volgende stap: 06 (5c-2, BSP-timer en idle).
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-quarantine-lock-held"; python3 -B tools/build.py --kernel-feature test-smp-quarantine-lock-held && python3 -B -u tools/run_smp.py --profile test-smp-quarantine-lock-held`.

## Historisch checkpoint — K06 F16/F17, blokkeren, wekken en doden zonder lost-wakeup (ADR-078) — 6 november 2026

**Bron:** `880d6d74` op `fix/k06-f16-f17-sched-protocol` (worktree `nieuw-os-k06-f16`), gestapeld op
`feat/k06-5b2-kernel-lock` (#234, `59476e17`, met F3/F4 uit #237). Taakspec:
`docs/implementation/k06-steps/04-F16-F17-block-wake-kill.md` (branch `docs/k06-step-specs`);
ontwerp ADR-078. **41 → 41/100**; K06 blijft open.

**Gerealiseerd:** `smp_lock::sched` met READY/RUNNING(cpu)/BLOCKING/READY_PENDING/BLOCKED/
DEAD_PENDING/DEAD en `block_prepare`/`block_commit`/`wake`/`kill_request`/`kill_commit` (elk één
atomaire stap). `RunQueue::push`/`pop` beheren een gedeelde `queued`-bit; een tweede push geeft
`SMP:SCHED:DOUBLE_ENQUEUE`. `sched_switch` gebruikt `leave`/`pick`: READY_PENDING → requeue,
BLOCKING → BLOCKED, DEAD_PENDING → terug in de eigen queue en DEAD bij een latere wissel zodra
`on_cpu` gewist is. De thuislus is de idle-thread (BW-4). `SHOOTDOWN_DEADLINE_MS` (100 ms) wordt
gedeeld door shootdown en kill. Native fase `block_wake_kill` na SCHED: `SMP:WAKE:PASS`
(10.000 ping-pongrondes onder KERNEL) en `SMP:KILL:PASS`, of `…:SKIPPED:CPUS=1` zonder AP's.
`test-smp-double-run` stopt nu bij de push; nieuw `test-smp-resched-deaf`.

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG, bron `01daa964…`):** zie
`docs/verification/k06-f16f17-sched/`. SMP 25/25 over 20 profielen; M2 8/8 en M3 22/22, rtc, K05,
K07; `cargo test -p smp-lock` 24+1 (uitputtende interleavingmodellen met mutanten), clippy en fmt
schoon, oracle 13/13, GS-lint 8/8; `tools/gate.sh` exit 0 (518 geslaagd, 1 ignored;
QEMU-smoke-script ontbreekt).
**Afwijking van de spec:** geen loom (33 crates, `generator 0.8.12` vereist Rust 1.95 > 1.90). De
modellen dekken alleen SC-uitvoeringen. **Niet uitgevoerd:** loom, M4 (pad-bug, #235), CI op de
gestapelde PR, hardware.

**Open:** loom (F-T1), echte IPC-wachtrijen (08), migratie (09), INV-5-beleid met vastgehouden
locks (05). Volgende stap: 05 (F19/F15, ICR-lock en quarantainebeleid) of 06 (5c-2).
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-resched-deaf"; python3 -B tools/build.py --kernel-feature test-smp-resched-deaf && python3 -B -u tools/run_smp.py --profile test-smp-resched-deaf`.

## Historisch checkpoint — K06 F3/F4, lockdep met handlercontext en lockparen (ADR-077) — 6 november 2026

**Bron:** `394bba37` op `fix/k06-f3-f4-lockdep` (worktree `nieuw-os-k06-f3f4`), gestapeld op
`feat/k06-5b2-kernel-lock` (#234, `de7a07b5`). Taakspec: `docs/implementation/k06-steps/02-F3-F4-lockdep-context.md`
(branch `docs/k06-step-specs`); ontwerp ADR-077. **41 → 41/100**; K06 blijft open.

**Gerealiseerd:** LD-1: `CpuLocal.irq_depth`, opgehoogd na `gs_entry!` en verlaagd vóór
`gs_exit!` in de stubs 0xf0/0xe0 en het timerlichaam (0xd0/0xe8). KERNEL/ASPACE in een handler
geeft `SMP:LOCKDEP:IN_HANDLER:<niveau>`. LD-2: het venster alleen bij diepte 0. LD-3/LD-4:
`Lockdep` met per niveau een teller (0/1/2) en sleutels, `acquire`/`acquire_paired`/`release`
met sleutel en `IrqLock::keyed` + `lock_pair` (oplopende sleutel, `(low, high)`). `RUNQ` heeft
nu sleutel = CPU-index. Volledige staat per CPU in `lock::LockdepCell`; het `AtomicU64` blijft
als spiegel. Fixture `SMP:LOCKPAIR:PASS:CPUS=n` en controles `test-smp-lock-in-handler` en
`test-smp-lockpair-order`; GS-lint controleert de teller.

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG, bron `5ea3759b…`):** zie
`docs/verification/k06-f3f4-lockdep/`. SMP 24/24 over 19 profielen; M2 8/8 en M3 22/22, rtc, K05,
K07; `cargo test -p smp-lock` 18 (incl. propertytest), clippy en fmt schoon, oracle 13/13, GS-lint
8/8; `tools/gate.sh` exit 0 (511 geslaagd, 1 ignored; QEMU-smoke-script ontbreekt).
**Niet uitgevoerd:** loom-model van `TicketLock` (nieuwe dependency nog niet getoetst), M4
(pad-bug, #235), CI op de gestapelde PR, hardware.

**Open:** loom (F-T1), ASPACE-paren nog zonder echte IPC-gebruiker (komt met 5d-2/IPC op SMP).
Volgende stap: 03 (F5/F7, begrensde spin in release en SERIAL-lock) of 05 (F19/F15).
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-lock-in-handler"; python3 -B tools/build.py --kernel-feature test-smp-lock-in-handler && python3 -B -u tools/run_smp.py --profile test-smp-lock-in-handler`.

## Historisch checkpoint — K06 stap 5b-2, KERNEL-lock en scratchslot per CPU (ADR-075) — 6 november 2026

**Bron:** `48dcae6a` op `feat/k06-5b2-kernel-lock` (worktree `nieuw-os-k06-5b2`), gestapeld op
`fix/k06-f8-swapgs` (#231, `7ffdacfa`). Taakspec: `docs/implementation/k06-steps/01-5b-2-kernel-lock-scratch.md`
(branch `docs/k06-step-specs`); ontwerp ADR-075. **41 → 41/100**; K06 blijft open.

**Gerealiseerd:** `lock::KERNEL` met `kernel_held()`; `lock.rs` bouwt ook voor M3, met een
lockdep-woord voor één CPU. `m3_event` neemt KERNEL na de CS-check en geeft hem vrij vlak vóór
de stub. `m3::start`/`complete_round` hervatten via `resume()`, dat KERNEL eerst vrijgeeft: de
guard staat in een slot dat de lock zelf beschermt, want `complete_round` springt naar ring 3
zonder af te wikkelen. SAFETY-teksten in `m3.rs` en `m3/*.rs` noemen KERNEL als grond.
`SCRATCH_ENTRY` is per CPU (`SCRATCH + i·4096`, 8 slots, dezelfde PT). De audit slaat alleen
het actieve slot van de lezende CPU over. Native fixture `SMP:SCRATCH:PASS` en controle
`test-smp-shared-scratch` (`SMP:SCRATCH:CROSS:CPU=n`; een #PF op het gedeelde slot telt mee).

**Bewijs (lokaal, QEMU 10.2.1/OVMF/TCG):** zie `docs/verification/k06-5b2-kernel-lock/`. Volledige
SMP-matrix 17 profielen op `48dcae6a`; controle 12/12 rood zoals bedoeld. Eén-CPU-regressie
(op de werkboom van `f6912a63` vóór fmt; daarna zijn alleen opmaak en een
controle-only-cfg in `smp.rs` veranderd): `test_milestones.py --through m4` (M2 8/8, M3 22/22,
M4 12/12), `test_supervisor.py` (K05 6 stappen), `test_resource_pressure.py` (K07),
`run_rtc.py --case all` 5/5, `run_ipc_wait.py`, `run_fpu.py` en `test-fpu-wait`;
`test_memory_system.py` op de schone commit. Alle 198 kernelprofielen uit `build.py` bouwen
(de update-profielen met `--image-version 2`). `tools/gate.sh` exit 0 (507 geslaagd, 1 ignored).
`cargo fmt --check` (workspace en kernel) en clippy `smp-lock -D warnings` zijn schoon,
`test_smp_serial.py` 11/11. **Niet uitgevoerd:** `native-smp.yml`/CI op de PR (een gestapelde
base draait geen native-smp-CI), fysieke hardware. `tools/test_milestones.py:160` heeft een
hardgecodeerd `build/users`-pad, waardoor M4 faalt met een eigen `NOS_BUILD_DIR` (losse taak).

**Open:** idle (`sti; hlt`) staat nog binnen de KERNEL-sectie (5c-2) en M3 op AP's (5d-2). De
500-ms-deadline begint na het BSP-deel. Fijnmazige locks vallen buiten K06. Volgende stap:
5c-2 (BSP-timer en idle), daarna 5d-2.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-shared-scratch"; python3 -B tools/build.py --kernel-feature test-smp-shared-scratch && python3 -B -u tools/run_smp.py --profile test-smp-shared-scratch`.

## Historisch checkpoint — K06 auditbevinding F8 (`swapgs`-discipline, ADR-076) — 6 november 2026

**Bron:** `91e84b75` op `fix/k06-f8-swapgs` (worktree `nieuw-os-k06-f8`), gestapeld op
`fix/k06-f1-f2-lifo-oncpu` (#230, `5dccf290`). Ontwerp: ADR-076, K06-SMP.md INV-8 (GS-1..GS-10).
**41 → 41/100**; K06 blijft open.

**Gerealiseerd:** `arch::gs_entry!`/`gs_exit!` (één definitie) in de stubs 0xd0/0xe0/0xe8/0xf0 en
`m3_restore`. In `m2_exception_common` dezelfde proloog met de GS-7-fixup voor een faulting
`iretq`. NMI/#DF/#MC lopen via `m2_exception_paranoid` (GS-vrij, terminaal). GS-10-check op de BSP.
Ring-3-GS-fixture `test-smp-gs-{user,iret-fault,nmi-user}` en statische lint
`tools/tests/test_gs_discipline.py`.

**Bewijs (lokaal, native QEMU 10.2.1/OVMF/TCG):** eerst rood: `c2f1bed0` geeft `SMP:GS:HIJACK`.
Groen: 21/21 SMP-gevallen over 16 profielen, M2-gate, K05 6/6, native-fpu en fpu-wait. Een
mutant zonder GS-7 laat N2 falen. Zie `docs/verification/k06-f8-swapgs/`. `tools/gate.sh` exit 0
(507 geslaagd, 1 ignored; QEMU-smoke-script ontbreekt). Clippy schoon voor `smp-lock` en
`boot-protocol`.
**Niet uitgevoerd/open:** CI (een gestapelde base start `native-smp.yml` niet), hardware,
speculatie (alleen statisch). Kernel-clippy `-D warnings` faalt al vóór deze wijziging
(paging/main/serial). Afwijkingen: N2 via SS=0x13 (TCG controleert canonieke RIP bij `iretq`
niet); VA 0 niet user-gemapt.

**Open uit de audit:** 5b-2/F20, F3, F4, F16, F17, F19, F15, het selectorlek (vóór 5d-2) en
SMEP/SMAP; daarna 5c-2, 5e, 5d-2 en 5f.
**Eerstvolgend commando:**
`export NOS_TARGET_DIR="$PWD/target/smp-native" NOS_BUILD_DIR="$PWD/build/k06-test-smp-gs-user"; python3 -B tools/build.py --kernel-feature test-smp-gs-user && python3 -B -u tools/run_smp.py --profile test-smp-gs-user`.

## Historisch checkpoint — K06 auditfixes F1 (on_cpu) en F2 (LIFO-lockdep) — 6 november 2026

**Bron:** `600cee93` op `fix/k06-f1-f2-lifo-oncpu` (worktree `nieuw-os-k06-5d`), gestapeld op
`feat/k06-5d-runqueues` (`76c67f22`). Ontwerp: aanvulling "auditfixes F1/F2" in ADR-074.
**41 → 41/100**; K06 blijft open.

**Gerealiseerd:** F2: `Lockdep::release` eist LIFO; een release buiten volgorde geeft
`SMP:LOCKDEP:LIFO:<held>-><released>` en stopt, vóórdat IF wordt hersteld. F1: vlag
`THREAD_ON_CPU` per thread, gewist door de timerstub na `mov rsp` (`Resume` in RAX:RDX).
`sched_switch` claimt de vlag met een CAS en wacht nooit; bij een mislukte claim gaat de thread
terug in de queue en draait de CPU thuis.

**Bewijs (lokaal, native QEMU 10.2.1/OVMF/TCG):** 18/18 over 13 SMP-profielen; herhaald
`double-run` 10/10 en `four`/`eight`/`two-single` elk 5/5. Zie
`docs/verification/k06-audit-f1-f2/`. `tools/gate.sh` exit 0 (507 geslaagd, 1 ignored; de
gate bouwt de native-smp-kernel niet en de QEMU-smoke ontbreekt). Hosttests
`lockdep_rejects_out_of_order_release` en `stack_is_never_shared_after_early_ready`.
**Niet uitgevoerd:** CI op de PR (een gestapelde base start `native-smp.yml` niet), en een native
negatief profiel voor LIFO of de F1-race.

**Open uit de audit:** F8 (GS-anker of `swapgs`, blokkeert ring 3), 5b-2/F20, F3, F4, F16, F17,
F19 en F15; daarna 5c-2, 5e, 5d-2 en 5f.
**Eerstvolgend commando:**
`export NOS_TARGET_DIR="$PWD/target/k06" NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 5d-1, runqueues per AP en CR3-wissel — 6 november 2026

**Bron:** `7079f788` op `feat/k06-5d-runqueues` (worktree `nieuw-os-k06-5d`), gestapeld op
#228 (5c, `b593930c`) en #227 (5b). Taakspec: `/tmp/task_spec_5d.md` (lokaal); ontwerp:
aanvulling stap 5d-1 in ADR-074. **41 → 41/100**; K06 blijft open.

**Splitsing:** 5d uit spec §8 vraagt ook ring-3-processen op AP's. Dat kan pas na 5b-2
(KERNEL-lock om M3, scratchslot per CPU); anders muteren twee CPU's de M3-staat en het ene
scratchslot tegelijk (INV-2). Daarom: 5d-1 (dit) en 5d-2 (ring-3 op AP's, poort K05 6/6 op
4 vCPU's).

**Gerealiseerd:** `smp_lock::sched` (RunQueue en threadstatus-CAS, INV-3). `sched_switch`
draait in het timerpad ná het 5c-uitstel; CR3-wissel volgens §6.3 met `ACTIVE` (INV-4);
wekvector 0xe8. Fixture `SMP:SCHED:PASS:CPUS=n` met 2 kernelthreads per AP in twee
adresruimten, de tweede cross-CPU gewekt. Negatief profiel `test-smp-double-run` →
`SMP:SCHED:DOUBLE_RUN`.

**Bewijs (lokaal, native QEMU 10.2.1/OVMF/TCG):** 18/18 gevallen over 13 SMP-profielen,
zie `docs/verification/k06-smp-sched/`. `tools/gate.sh` exit 0 (505 geslaagd, 1 ignored;
QEMU-smoke ontbreekt in de repo), M2-gate geslaagd, `test_smp_serial.py` 9/9, fmt en clippy
schoon. **Niet uitgevoerd:** CI (`native-smp.yml`) op de PR, en K05 op meerdere vCPU's
(hoort bij 5d-2).

**Open:** 5b-2, 5c-2, daarna 5d-2 (ring-3 op AP's, FXSAVE in de wissel), 5e en 5f.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 5c, geen preëmptie met een actieve lock — 6 november 2026

**Bron:** `bdcced5e` op `feat/k06-5c-preempt-lock`, gestapeld op 5b (`b7cb7ce7`, PR #227).
Taakspec: `/tmp/task_spec_5c.md` (lokaal); ontwerp: aanvulling stap 5c in ADR-074.
**41 → 41/100**; K06 blijft open.

**Gerealiseerd:** `smp_preempt` stelt een tick uit (`CpuLocal.deferred`) zolang de
lockdepset van de CPU niet leeg is, dus zolang een KERNEL/ASPACE-lock wordt vastgehouden
of in het 5b-venster bewaakt. Dat geldt ook voor STOP. Een wissel met een niet-lege set
geeft `SMP:LOCKDEP:HELD_ACROSS_SWITCH` en stopt. Native fixture in `SMP:PREEMPT:PASS`:
de BSP houdt een lock op KERNEL-niveau terwijl de draaiende thread op elke AP erop wacht.
Eis: `deferred` stijgt, `switches` blijft staan en hervat na het vrijgeven. Het negatieve
profiel `test-smp-held-across-switch` moet de marker geven, met exit 0x10.

**Bewijs (lokaal):** `tools/gate.sh` exit 0 (502 geslaagd, 1 ignored, 34 s),
`test_smp_serial.py` 8/8, `cargo fmt --check` (workspace en kernel) schoon, clippy
`smp-lock -D warnings` schoon. Kernelbuilds `native-smp`, `test-smp-held-across-switch`
en `milestone-m2` slagen. **Niet uitgevoerd:** native QEMU (geen QEMU in deze
WSL-omgeving), dus `native-smp.yml` op de PR.
**Native (later, QEMU 10.2.1 + OVMF van de host, `tools/run_smp.py`):** `native-smp` 6/6
(one, two-single, four, eight, nine-capacity, duplicate-madt), `test-smp-held-across-switch`
1/1 (marker, raw 33), `test-smp-lockdep-order` 1/1 en `test-smp-spin-irq-off` 1/1
(`SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1` + quarantaine). Rapporten in `build/k06-*/smp-report.json`
(lokaal, niet in Git). Een fout die de 5d-sessie vond: zonder IF-venster kreeg de 5c-fixture in
`test-smp-spin-irq-off` nooit een tick (`SMP_PREEMPT_LOCK_NO_TICK`). Opgelost: dat controleprofiel
slaat de fixture over.

**Open:** 5b-2 (KERNEL-lock om M3-syscalls, scratchslot per CPU, ADR-075), 5c-2
(BSP-LAPIC-timer + idle `sti; hlt`, BSP-venster), en daarna 5d t/m 5f. Er is nog geen
runtimecheck dat handlers geen KERNEL/ASPACE nemen.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-held-across-switch"; python3 -B tools/build.py --kernel-feature test-smp-held-across-switch && python3 -B -u tools/run_smp.py --profile test-smp-held-across-switch`.

## Historisch checkpoint — K06 stap 5b, gecontroleerd spinnen met IF=1 — 6 november 2026

**Bron:** `5481d407` op `feat/k06-5b-spin-irq` (main `5ae0757d` met #225 = 5a, plus de
lokale `tools/gate.sh`). Taakspec: `/tmp/task_spec_5b.md` (lokaal); ontwerp:
aanvulling stap 5b in ADR-074. **41 → 41/100**; K06 blijft open.

**Gerealiseerd:** `TicketLock::lock_with` (onbegrensd met relax). `IrqLock` opent voor
KERNEL/ASPACE op AP's een venster `sti; pause; cli` tussen pollingen. Daarnaast een native
fixture na PREEMPT: de BSP houdt een KERNEL-lock vast terwijl elke AP met IF=0 wacht, en eist
een shootdown-ack (`SMP:SPIN:PASS`). Het dove controleprofiel `test-smp-spin-irq-off`
moet `SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1` geven, gevolgd door quarantaine.

**Bewijs (lokaal):** `tools/gate.sh` exit 0 (502 geslaagd, 1 ignored), `test_smp_serial.py`
7/7, `cargo fmt --check` (workspace en kernel) schoon, clippy `smp-lock -D warnings` schoon.
Kernelbuilds `native-smp`, `test-smp-spin-irq-off`, `test-smp-lockdep-order` en
`milestone-m2` slagen. **Niet uitgevoerd:** native QEMU (geen QEMU in deze WSL-omgeving),
dus `native-smp.yml` op de PR. De bredere `tools/tests`-discovery geeft 2 importfouten
(`test_tpm`, `test_materiaal_cold`); die staan er ook zonder deze wijziging.

**Open:** voordat een timer wordt gewapend op een CPU die KERNEL/ASPACE neemt, moet de
preëmptie worden uitgesteld zolang lockdep niet leeg is. Er is nog geen runtimecheck dat
handlers geen KERNEL/ASPACE nemen. Volgende stap: 5b-2 (KERNEL-lock om M3-syscalls,
scratchslot per CPU, ADR-075), daarna 5c t/m 5f.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-spin-irq-off"; python3 -B tools/build.py --kernel-feature test-smp-spin-irq-off && python3 -B -u tools/run_smp.py --profile test-smp-spin-irq-off`.

## Historisch checkpoint — K06 stap 5a, ticketlock, irqsave en lockdep — 6 november 2026

**Bron:** branch `feat/k06-5a-ticketlock-lockdep` op main `2c7edaf5`; spec
`docs/implementation/K06-SMP.md` (§4, stap 5a). **41 → 41/100**; K06 blijft open.
Ontwerp: ADR-074.

**Gerealiseerd:** `crates/smp-lock` (ticketlock en lockdep),
`kernel/src/m2/lock.rs` (`IrqLock`), een GS-anker voor de BSP, een BSP-zelftest
(`SMP:LOCKDEP:PASS`) en het negatieve profiel `test-smp-lockdep-order`. De
AP-stresstest draait nu op de ticketlock.

**Bewijs (lokaal):** `tools/gate.sh` exit 0, met 483 hosttests waarvan 8 nieuw.
`test_smp_serial.py` 6/6. Kernelbuilds `native-smp`, `test-smp-lockdep-order`
en `milestone-m2` slagen. `cargo fmt --check` en clippy op `smp-lock` zijn schoon.
**Niet uitgevoerd:** native QEMU (geen QEMU in deze WSL-omgeving). Die staat
open voor `native-smp.yml` op de PR.

**Open:** stap 5b (KERNEL-lock om M3-syscalls, scratchslot per CPU, ADR-075)
t/m 5f; zie de spec §8.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-lockdep-order"; python3 -B tools/build.py --kernel-feature test-smp-lockdep-order && python3 -B -u tools/run_smp.py --profile test-smp-lockdep-order`.

## Historisch checkpoint — K06 stap 4, gepreëmpteerde kernelthreads op AP's — 5 november 2026

**Bron:** `c308b1ee9d8deedcfacd923c407392222ff7a12f` op `k06/smp-step4-preempt`. De keten is #191
(`0cfcdd4a`) → #201 (`64f8abc6`) → #202 (`58e11fd0`) → #203 (`e49e7819`) →
stap 4. **41 → 41/100**; K06 blijft open. Ontwerp: ADR-073.

**Gerealiseerd:** op elke AP preëmpt de LAPIC-timer twee kernelthreads, met
een registercanary, voortgang tussen twee metingen en bevriezing na STOP.
Een AP met een dode timer leidt tot quarantaine. **Bewijs:** 14/14 native
gevallen; regressie K05 6/6, `milestone-m2` en `native-rtc` 5/5. Zie
[k06-smp-preempt](verification/k06-smp-preempt/README.md).

**Open:** merge van de keten (beslissing van Simon). Daarna volgt de
eigenlijke SMP-scheduler voor processen: per-CPU-runqueues, lockvolgorde,
interrupts op de BSP, een CR3-wissel per thread met shootdown voor de
hele procesadresruimte, en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 3, TLB-shootdown vóór frame-hergebruik — 5 november 2026

**Bron:** `9c64c1dbe85290501934331a28a9ec7f61f39399` op
`k06/smp-step3-shootdown`. De keten is #191 (`5263a5ef`) → #201
(`cd78ea74`) → #202 (`8c694ef0`) → stap 3; main is `c7a61d11`.
**41 → 41/100**; K06 blijft open. Ontwerp: ADR-072.

**Gerealiseerd:** een vervangen kernelframe wordt pas vrijgegeven nadat
elke AP beide vertalingen (het venster en de identity-alias) met `invlpg`
heeft ongeldig gemaakt en de generatie van de ronde heeft bevestigd.
Daarna volgen `free`, een volledige audit en een zichtbaarheidsprobe. Een
ontbrekende ack zet het frame in quarantaine en reset alle AP's.

**Bewijs:** 13/13 native QEMU/TCG-gevallen, waaronder een dove AP en een
controle zonder shootdown. Bij die controle lezen alle AP's
deterministisch het vrijgegeven frame. Regressie: K05 6/6, `milestone-m2`
en `native-rtc` 5/5. Zie
[k06-smp-shootdown](verification/k06-smp-shootdown/README.md).

**Open:** de beslissing over #191 ligt bij Simon. Volgens de integrator
loopt #191 achter op main, draaide de volledige CI daar niet, en staan er 6
CodeRabbit-threads open plus de `run_cap_bench`-regel; de integrator kan
niet op die branch schrijven. Daarna komen CI en integratie van
#201/#202/stap 3. Volgende K06-stap: interrupts op de BSP en een
SMP-veilige scheduler, met per-CPU-runqueues, lockvolgorde, procesbrede
shootdown per CR3 en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 2, eigen TSS/IST en IPI's per AP — 5 november 2026

**Bron:** `486371e5a3790c7c1df817820ff4cc373c5fc42c` (stap-2-code `1993ade8` + #201-stackfix `36703f50` + main `8bb15309`) op
`k06/smp-step2-percpu-ipi`. Dat is gestapeld op #201
(`k06/smp-ap-bringup`, `047acf39`), dat op #191 staat. Main is `8bb15309`.
**41 → 41/100**; K06 blijft open. Ontwerp: ADR-071.

**Gerealiseerd:** per AP een eigen GDT/TSS (RSP0, IST1 #DF, IST2 NMI) en
een GS-anker. Een IPI-vector 0xf0 met EOI, unicast-rondes met een exacte
telling, parkeren via IPI, en quarantaine van alle AP's als er één doof is.
Een AP met een afwijkende IA32_APIC_BASE wordt afgewezen.

**Bewijs:** 11/11 native QEMU/TCG-gevallen over zes profielen, waaronder
NMI en #DF op de eigen AP-IST-stack en een dove AP met quarantaine.
Daarnaast host-tests, Clippy en fmt, plus regressie van `milestone-m2` en
`native-rtc` (5/5). Zie [k06-smp-ipi](verification/k06-smp-ipi/README.md).
Review door AI; alle bevindingen zijn verwerkt.

**Open:** CI-beoordeling van #201 en de nieuwe stap-2-PR, plus integratie in
de volgorde #191 → #201 → stap 2. Volgende K06-stap: een
TLB-shootdownprotocol op deze IPI's, daarna BSP-interrupts, de
SMP-scheduler en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 1 en conflictintegratie — 5 november 2026

**Historische bron van deze conflictintegratie:** PR-head
`cd78ea74f14d8b8921aad51cc46a41c41274ab45` op `k06/smp-ap-bringup`,
geïntegreerd met base `main` op
`7dd91fad6d341d0c7b66b885bbe8713d2020e1df`. De PR-head bevat ook de
device-exec/ACPI-handoff-integratie. Drie statusconflicten zijn opgelost met
behoud van de K06-projectstatus. **41 → 41/100**: geen criteria of gewichten
gewijzigd; K06 blijft open. De actuele PR-head en -base staan in PR #201
en in `docs/verification/k06-smp-bringup/native-report.json`; deze sectie
wordt niet bij iedere update-merge herschreven.

**Gerealiseerd:** een opt-in M2-profiel `native-smp` (ADR-070) met
MADT-inventaris, INIT-SIPI-SIPI, een nooit gemapte SIPI-pagina, AP-stacks met
guards in het kernelimage, contextclaims via CAS met fail-closed time-out en
INIT-quarantaine, registergelijkheid met de BSP en een begrensde
atomic- en spinlockproef. Een native gevonden BSP-stack-overloop door een
by-value MADT is hersteld; die zou alle profielen hebben geraakt.

**Bewijs:** 8 native QEMU/TCG-gevallen geslaagd (1/2/4/8/9 vCPU's, single-
en multi-TCG, capaciteit, dubbele MADT via `-acpitable`, AP-stall,
registermismatch). Daarnaast 25 boot-protocol-tests (exit 0; 28 na de #191-tests), 4+2+10+8
Python-harnastests, Clippy/fmt en regressie van `milestone-m2` en
`native-rtc` (5/5). Zie
[k06-smp-bringup](verification/k06-smp-bringup/README.md). Twee
AI-reviews, geen menselijke audit: alle concrete bevindingen zijn verwerkt.

**Controles na integratie:** `cargo +1.90.0 test --locked -p kernel-core`
en de relevante USB-, PS/2- en Python-tests; formatting, Clippy, freestanding
driverbuilds en native-smp kernelbuild geslaagd. `tools/progress.py --check`
geslaagd. CodeQL op de eerste mergecommit `c014b17d` vond geen
Actions/Python-alerts maar liep af vóór alle talen klaar waren; de
geresynchroniseerde head is dus niet volledig gescand. Nieuwe-head hosted CI
moet nog worden beoordeeld.
**Open:** volledige CodeQL-dekking en native K06-QEMU-bewijs op de geïntegreerde
head.
Voor K06 nog nodig: per-CPU-staat, TSS/IST, IPI/TLB-shootdown, SMP-scheduler,
lockvolgorde en herziening van ADR-024. Er is geen fysiek hardwarebewijs.
**Eerstvolgend commando:**
`NOS_BUILD_DIR="$PWD/build/k06-native-smp" python3 -B tools/build.py --kernel-feature native-smp`
en daarna, met dezelfde `NOS_BUILD_DIR`,
`NOS_BUILD_DIR="$PWD/build/k06-native-smp" python3 -B -u tools/run_smp.py --profile native-smp`
(`run_smp.py` bouwt niet zelf).
## Historisch checkpoint — sessie-/bibliotheekintegratie (#206) — 5 november 2026

**Basis:** main `bce6cabc71184d90fc228fe2ae8215339ff7bf28`, branch `integrate/20261105-session-library`
(PR #206). **41 → 41/100.**

**Opgenomen uit #135/#147:** `ui/native`-bibliotheek, `window-session`-shell,
`document-session`-volumeuitbreiding, `ui/graphics`-reflow/damage, kernelfeature
`native-session-route` (alleen diagnostische tellers) en native bibliotheekgates in
`native-window-session.yml`. **Niet opgenomen:** het sealed-generatie-0-volumeontwerp
uit #133, dat is ingehaald door #148/ADR-064 en #205, samen met de dubbele ADR-064
en het bijbehorende bewijs. **Integratorfixes:** Escape annuleert een hangende
ontgrendeling, `Font::advance` schaalt mee met magnify, en de `DropRead`-test raakt nu
werkelijk een late read.

**U03:** het branchbewijs `docs/verification/materiaal-library-20261102/` claimt
41→44, maar is hashgebonden aan `64044318`. Op main blijft U03 open, totdat de native
bibliotheekgates op de main-merge groen zijn en de eigenaar dit bevestigt.
**Open vervolg (CodeRabbit op #206):** hoverdamage alleen bij een hoverwissel,
`hit()`-focus op uitgeschakelde editorknoppen, een positieve checkpointtest in
`test_window_library.py`, de provenance-controles in `verify-ci-artifact.py` en
documentatiebronnen.

## Historisch checkpoint — integratie PR-wachtrij — 5 november 2026

**Basis:** main `c7a61d1126caa346a128fba3004fe4add4ccff61` (na #198), branch `integrate/20261105-platform-batch`
(PR #199). **41 → 41/100**; geen criterium volledig geaccepteerd.

**Gemerged:** #198 = #179 #183 #187 #190 (USB-framing, HID boot reports en
-interfacebinding, PS/2-health; hostmodellen). **In #199:** #169 #173 #176 #182
#188 #189 #193 #194 (acht zelfstandige platformcrates, nu in de
`component-leaves`-matrix; voorheen draaide CI er geen enkele test van),
#165 (EXIF in tile-viewer), #150 (agentprofielen) en #195 (instructieopschoning).
Lokaal (Rust 1.90.0): test, clippy `-D warnings`, fmt en freestanding build per
crate exit 0; onafhankelijke AI-reviews MERGE (geen menselijke audit).

**Open:** #155 vraagt Simons besluit (score 41→39, B01-criterium); #181, #191
en #177 FIX (reviewcomments op de PR's); calculator #184/#186/#192 onderling
CONFLICT (26+ hunks, ook in `tools/run_window_session.py`); #133, #135, #146,
#147, #154, #157 nog niet beoordeeld; concepten #167 #185 #197 niet overgedragen;
#196 overbodig na het besluit om alleen de Graphify-MCP te gebruiken.
**Bewijsgrens:** geen native app-launch, fysieke USB/HID of menselijke test.
Eerstvolgend commando: `python3 tools/progress.py --check`.

## Historisch checkpoint — instructieonderhoud — 4 november 2026

**Basis:** `8bb153092e967aae314ecde35bcf963565a87ac7`, branch
`copilot/dream-curate-memories-20261104132442`. Vijf instructiebestanden
verliezen verouderde beperkingen op opslaglocaties, native UI-werk en
voortgangsrapportage. De actuele privacycontracten, lokale AI, apprechten,
eigen kernel en bewijsgrenzen blijven ongewijzigd. **41 → 41/100**:
geen runtimecode, criterium, gewicht of nieuwe productacceptatie.

**Controles (alle exit 0):** `python3 tools/progress.py --write`,
`python3 tools/progress.py --check`,
`python3 -B .agents/skills/os-dev/scripts/validate_skill.py`,
`python3 -B -m unittest discover -s .agents/skills/os-dev/tests -v`
(zes tests) en `git diff --check`. De onafhankelijke AI-tegenvoorbeeldreview
vond geen concrete weerleggingen; dit is geen menselijke audit.
**Bewijsgrens:** documentatieonderhoud; geen OS-build, QEMU-, hardware- of
menselijke acceptatie uitgevoerd. Deze controles bewijzen geen OS-functionaliteit.
De bestaande productblokkades en historische receipts hieronder blijven gelden.

**Vervolg:** beoordeel de concept-PR en haar actuele controles vóór integratie.
Eerstvolgend lokaal commando: `python3 tools/progress.py --check`.

## Historisch checkpoint — PR158 met main 95d79451 — 3 november 2026

**Bronnen:** PR-head `04d727c306d13e03a192d9d002a7d44348841c9f` en
main `95d7945110ce853a1b5827ec59f6115475691805`, branch
`fix/b03-test-anchors-guard`. De drie documentconflicten behouden de
B03- en PR138-checkpointgeschiedenissen en hun brongebonden bewijs.
De automatisch geïntegreerde Rust-wijzigingen uit main zijn uitsluitend
commentaar; de B03-ondertekenpolicy blijft ongewijzigd.
**41 → 41/100**; geen nieuw criterium, gewicht of native acceptatie.
De historische PR148-integratiereviewvelden blijven expliciet historisch.

**Validatie op de opgeloste boom:** 78 Rust-hosttests en 14 Python-harnastests
geslaagd; gerichte Clippy en beide formatcontroles exit 0. De eerste
parallelle Rust-commando's raakten een toolchain-downloadrace; na voltooide
installatie en één herstel van ontbrekende rustfmt/Clippy zijn alle controles
opnieuw geslaagd. Geen dependency of lockfile gewijzigd.
Vergelijkingen bevestigen exact behoud van de B03-code uit de PR en van de
drie gedocumenteerde Rust-bestanden uit main. Exacte commando's, logpaden,
exitstatussen en hashes staan in
[integratiebewijs](verification/pr158-main-integration-20261103.json).
`tools/progress.py --write` en `--check` slagen; de SVG blijft bytegelijk.
Native builds en QEMU niet herhaald voor deze documentintegratie.
Oud testbewijs geldt niet automatisch voor deze integratie.

**Vervolg:** na publicatie nieuwe-head CI laten beoordelen; geen oude
CI-run herstarten. Productiesleutelbeleid en issue #41 blijven open.
Eerstvolgende lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — B03 TEST-ankergrens — 3 november 2026

**Basis:** `main` `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`; codecommits
`07e7f9a0471798c49e1844bf885f94491bb481d8` en
`ceb957557af821bb95786aa505b5df41077f7403` op
`fix/b03-test-anchors-guard`. `test_policy` vereist nu de expliciete
`test-signing-anchors`-feature; UEFI-loader- en update-servicebuilds zonder
gekozen ondertekenbeleid falen tijdens compilatie. TEST-sleutels blijven
uitsluitend fixtures. Productieankers, sleutelrotatie/-intrekking en
releaseceremonie zijn niet ontworpen of bewezen; #41 blijft open.

**Gerichte controles:** update-core met TEST-feature 13 tests, update-tool
2 tests, update-core zonder TEST-feature 7 tests en B03 Python-harnas 14 tests
geslaagd. De testloader en update-service zijn crossgebouwd. Negatieve
compilatiechecks voor de ongemarkeerde loader en service falen verwacht met
alleen de bedoelde policyfout. Update-core/update-tool- en service-Clippy
slagen; loader-Clippy slaagt met alleen de bestaande
`clippy::not_unsafe_ptr_arg_deref`-lint toegestaan; de ongewijzigde pointer
entrypoint is geen onderdeel van deze wijziging. Zie de exacte commando's en
bewijshashes in
[B03-verificatierapport](verification/b03-test-anchor-gating/README.md).
Een directe release-`cargo build` buiten de projectwrapper raakte daarnaast
een LLVM SIGILL tijdens codegeneratie voor `curve25519-dalek`; de
voorgeschreven update-service-buildwrapper slaagde wel en is het gerapporteerde
buildresultaat.

**Native vervolgbewijs:** op productbron `26108a25` is
`python3 -B -u tools/test_update.py` uitgevoerd: exit 0, 106,91 s,
alle 18 QEMU-stappen en de afsluitende normale build geslaagd. Zie
[native-sprint.json](verification/b03-test-anchor-gating/native-sprint.json)
voor de exacte omgeving, hashes en bewijsgrens. Evidencehead `568741f3`
had 31 geslaagde checks en een overgeslagen check. Geen hardwaretest,
productiesleutelbeleid of nieuwe productcriteria geaccepteerd.

**Voortgang:** branchscore **41 → 41/100**; `tools/progress.py --write`
en `--check` bevestigen deze berekening. Er zijn geen criteria of gewichten
gewijzigd en deze wijziging levert geen productacceptatiepunten op.

**Eerstvolgend:** de gerichte sprintcorrectie na actuele-head-CI integreren;
#41 blijft open voor productiesleutelbeleid en bijbehorende acceptatie.

## Historisch checkpoint — PR138 met main d686bd54 — 3 november 2026

**Bronnen:** PR-head `fc3b5e2522e04f636bdb443299da6e8f99161afe` en
main `d686bd54e766aed542e9918d1a37dcce434f18e2`, branch
`simonvanieperen-laughing-fortnight`. Alleen PROJECT-STATE.md conflicteerde;
beide checkpointgeschiedenissen blijven behouden. Code is automatisch
samengevoegd zonder handmatige runtimewijzigingen. **41 → 41/100**;
geen criterium, gewicht of historisch testbewijs gewijzigd.

**Uitgevoerd op de opgeloste boom:** 87 Python-harnastests en 76
Rust-hosttests geslaagd; gerichte Clippy en workspace-formatcontrole exit 0.
De `native-volume`-cross-build van UEFI-loader, kernel en volume-apps slaagt
na één herstel van ontbrekende, reeds vergrendelde SDK-downloads. De eerste
offline build mislukte wegens ontbrekende `zeroize v1.9.1`; geen dependency
of lockfile gewijzigd. Codevergelijkingen bevestigen exact behoud van main
voor kernel/volume/harnassen en van de PR voor crates/window-session.
Exacte commando's, exitstatussen en loghashes:
[integratiebewijs](verification/pr138-main-integration-20261103.json).
Historische receipts bewijzen niet automatisch deze opgeloste boom.
QEMU niet uitgevoerd: `qemu-system-x86_64` ontbreekt. Geen nieuwe
native-runtime-, release-, fysieke-hardware- of menselijke acceptatie.

**Vervolg:** na publicatie de nieuwe-head-controles laten beoordelen.
De oude native-foundation-run wordt na deze integratie niet herstart.
Eerstvolgende lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — PR138 documentatieconflicten — 3 november 2026

**Bronnen:** PR-head `d263acdeb9f94905c52457cccfeafabdc3011571` en
main `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`. De vier conflicterende
statusbestanden behouden beide geschiedenissen; de laatst vastgelegde
beoordeling blijft **41/100**. Historische controles blijven aan hun eigen
bron gebonden en gelden niet automatisch voor de samengevoegde boom.

**Vervolg:** beoordeel de controles op de nieuwe mergecommit. Volledige
release-, fysieke-hardware- en menselijke acceptatie blijven open.
## Historisch checkpoint — PR148 met main 8e0c15d — 3 november 2026

**Basis:** PR148 `47ec535f15bb10912dbc6cbf1af3de9df0c6c226` met main
`8e0c15d8c71ec4bd4e946e14dbf58bcfe748baa2`, branch
`simonvanieperen-upgraded-disco`; schone werkboom bij aanvang.
Drie centrale documentconflicten behouden beide checkpointreeksen en de
brongebonden receipts. Gedeelde harnashulpfuncties komen exact uit main;
volume-admissie blijft uit PR148. Geen handmatige codewijziging.
**41 → 41/100**, geen criterium of gewicht gewijzigd.

**Uitgevoerd op de opgeloste boom (alle eindcommando's exit 0):**
- `python3 -B -m unittest discover -s tests -p 'test_common.py'`: 8 tests.
- `python3 -B -m unittest discover -s tools/tests -p 'test_random.py'`: 21 tests.
- `python3 -B -m unittest discover -s tools/tests -p 'test_images.py'`: 20 tests.
- `python3 -B tools/test_fpu.py`: 12 tests.
- `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'`: 26 tests.
- `python3 tools/progress.py --write` en `python3 tools/progress.py --check`.
- `git diff --check`; vergelijking met beide parents bevestigt exact behoud
  van de gewijzigde harnassen uit main en de volume-implementatie uit PR148.

De eerste discoverypogingen voor random/images/fpu onder `tests` vonden
geen tests (elk exit 5); daarna zijn de werkelijke bovenstaande entrypoints
uitgevoerd. Verwachte foutmeldingen in negatieve harnastests zijn geen
native uitvoering. Uitvoer staat in het sessietranscript, zonder afzonderlijke
logbestanden. Rust-builds en QEMU niet herhaald; geen native acceptatie.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:69f0ce3ef4e7240e6d8b1e44a29d90b2c36c7dcd2008b6a678a3f075e30514e8 -->

**PR148-integratiereview: open.** De assessmentbasis is PR148 `47ec535f`;
de tweede mergeouder is main `8e0c15d8` en de resulterende merge is `7412d0bb`.
De hierboven gemelde gerichte controles hebben alleen uitvoer in het
sessietranscript; een afzonderlijk brongebonden integratiereviewbewijs ontbreekt
in de checkout. De PR148-hostreceipts hieronder betreffen eerdere werkbomen en
bewijzen deze integratiereview niet. De `reviewed_commit`-verwijzing naar
`cc1dea7` in `docs/progress.json` is een historische review van de eerdere
verduidelijking van documentatiestatus en hostbewijs.

**Eerstvolgend:** nieuwe-head CI en review beoordelen en passend reviewbewijs
vastleggen. Native volume-acceptatie blijft afzonderlijk open.

<!-- /coderabbit-ai-generated:v1 -->

## Historisch checkpoint — PR142 main-integratie — 3 november 2026

**Basis vóór integratie:** PR-head `37ad1e98328e59b752c230310a4abeaca749b669`;
main `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`. De enige conflicthunk was
`docs/verification/retained-document-heads-20261103.json`. De opgeloste
snapshotlijst behoudt alle unieke PR131/132-, PR141- en PR142-ingangen;
PR149's code/tests en receipts zijn eveneens behouden uit main. Geen receipt
wordt als bewijs voor deze nieuwe mergeboom aangeduid.

**Uitgevoerd op de opgeloste boom:** `node --test ui/prototype/tests/*.test.mjs`
— 41 geslaagd, 0 mislukt; `node --check ui/prototype/app.js` en
`node --check ui/prototype/state.js` — exit 0. De README-voorgestelde
`--test-isolation=none`-optie wordt niet herkend door Node v22.23.4; de suite
is daarna met de ondersteunde standaardopties geslaagd. Progressmeter is
bijgewerkt: **41 → 41/100**, zonder nieuw criteriumbewijs. Native/QEMU,
fysieke en menselijke acceptatie zijn niet uitgevoerd.
**Open:** finaliseer de merge na JSON/progress- en diffcontroles en secret scan;
na publicatie moeten de nieuwe-head-checks worden geëvalueerd.

## Historisch checkpoint — PR149 derde main-integratie — 3 november 2026

**Basis:** Watch-merge `bb8877e3e1aaec304d7478510b65c8116aeb634e`, met
ouders `416886354dbb0c6135a0740c429a3fbd4530a16d` en main
`99d132768fd72276caa4a3b3e9a09f8f1909f0ed`, branch
`simonvanieperen-reimagined-train`. Vier centrale documentatieconflicten;
de compacte UI-fixes en hun bewijs blijven uit de PR behouden, de
PR139-receipts komen ongewijzigd uit main. Beide historische statuslijnen
blijven behouden en krijgen geen nieuwe bewijsbron toegewezen.
Rubric en score blijven **41 → 41/100**.

**Werkelijk opnieuw uitgevoerd op schone `bb8877e3`:**
`cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml`,
59 hosttests, exit0, 10,816s. Cwd, HEAD, schone status en Cargo.lock-/Rust-
bronhashes vóór én na, exact commando en volledig gehasht log:
[primaire receipt](verification/pr149-bb8877-host-receipt.json).
Dit is een nieuwe run, geen retrospectieve bronbinding van de door Watch
gemelde tooluitvoer/tijden. De integrator controleerde apart rubricbehoud,
ongewijzigde Rust-code ten opzichte van532, één handoff en geen markers.
Een historische PR140-verwijzing naar PR139-bewijs is bij deze integratie
hersteld naar haar eigen oorspronkelijke commando's en bron.
Dit is geen menselijke audit. Geen nieuwe formatter-/Clippy-run,
cross-build, volledige CI of native/QEMU-uitvoering voor deze
uitsluitend documentatie betreffende integratie.
**Open/eerstvolgend:** na publicatie nieuwe-head-controles laten evalueren.
Alleen bewijs-/statuscorrecties; geen oude CI-retry of nieuwe runtimecode.
Native/QEMU-, fysieke en menselijke acceptatie blijven afzonderlijk open.

## Historisch checkpoint — PR148 slot1-admissie opvolging — 3 november 2026

**Basis:** PR148-head `c6b25873c915a98f8c0b2bb7400395e01ba44741`, branch
`simonvanieperen-upgraded-disco`, met slot1-admissiefix, marker-crashgrens
en host-suitebewijs al aanwezig. Score en rubric blijven **41 → 41/100**.

**Status en classificatie (actueel):**
- Vereiste checks (`host-quality`, `qemu-smoke`) zijn groen.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:d8b787176e01e7132a22480ebc41537196905df76f55d8e674355a4700a5bc60 -->

- `native-foundation` faalde volgens de handoff in run `41640409938` met
  `system marker sequence mismatch`/`RANDOM guest report ... not bound`.
  De oorzaak is **onbekend**: de primaire workflowbron, de exacte gefaalde
  stap en het resultaat van de gestarte rerun (`gh run rerun --failed 41640409938`)
  ontbreken. De gate blijft open.

<!-- /coderabbit-ai-generated:v1 -->
- Open CodeRabbit-threads betreffen vooral documentatiestatus en bronlabels;
  productiegedrag voor slot1-admissie is al gecorrigeerd en hostmatig bewezen.

**Primaire bronnen (code + bewijs):**
- Recovery/admissie: `services/volume/src/table.rs` op commit `c6b25873`.
- Onafhankelijk orakel: `tools/volume_disk.py` op commit `c6b25873`.
- Bewijs: `docs/verification/pr148-slot1-review.json`,
  `docs/verification/pr148-marker-crash-review.json`,
  `docs/verification/pr148-volume-core-host-suite.json`.
- Repositorybron: https://github.com/simonvanieperen/nieuw-os/tree/c6b25873c915a98f8c0b2bb7400395e01ba44741

**Open/eerstvolgend:** rond PR148 de resterende documentatie-threads sluiten,
uitkomst van de `native-foundation` rerun verwerken en daarna opnieuw
merge-preflight op exact head/base/checks/reviewthreads draaien.
Native metadata-only fixture en native QEMU-volume-admissie blijven apart open.

## Historisch checkpoint — PR149 compacte kaartregressies hersteld — 3 november2026

**Bron:** `53256f82baf097ebaab2497018ef2c37196c8e5d`, boven
layoutfix `f67b1e85` en actuele Watch-parent `5492158d` met main `d115cc96`.
Smalle/hoge viewports gebruiken nu dezelfde compacte geometrie voor
hit-testing en tekenen. De Start-breedtetest volgt de werkelijke
werkbalkbreedte; nieuwe320/400/639/640-pixelgevallen en echte geschaalde
framebufferpixels controleren de grens. Compacte toestandstekst past
binnen de kaart en onderscheidt opslagfouten van volle objectruimte en
andere fouten. Geen autorisatie-/opslagcontract gewijzigd.

**Echt rood/groen:** oude kaart was200 in plaats van96 pixels hoog;
`Ready` veranderde pixel109,163 buiten de kaart en opslag-/niet-opslagfouten
gaven identieke pixels. Na correctie **59 hosttests**, Clippy, formattering
en scalar `x86_64-unknown-none`-librarybuild: exit0.
Volledige relevante logs, exacte commando's, SHA-256 en bronbinding:
[reviewbewijs](verification/pr149-compact-review.json).
De eerste extra pixeltest gebruikte onjuist logische coördinaten bij200%;
die testfout is hersteld, de mislukte poging blijft vermeld.

**Open/eerstvolgend:** `GH_PAGER="" gh pr checks 149`; verse volledige
CI/native-QEMU-/screenshotacceptatie en review op exact de nieuwe head.
Geen gast-run, fysieke/human-/releaseacceptatie of gebruiksmeting geclaimd.
Oude55/57-tests en Watch-meldingen hieronder zijn historisch, niet verse
bronacceptatie. **41 → 41**, geen criteriumwijziging of mandaatvrijgave.

## Historisch checkpoint — PR149 tweede main-integratie

**Bron:** PR-head `b75cfc3c7c5cc16efd618b2d6d179938abdabbdf` met main
`d115cc96c19ac4371baed77409a23d1adddffaf0`, branch
`simonvanieperen-reimagined-train`. Alleen vier centrale
documentatieconflicten; beide checkpointgeschiedenissen behouden.
De artifact-store-tests en bewijsrapporten komen ongewijzigd uit main;
de UI blijft ongewijzigd uit de PR-parent. Score **41 → 41/100**.

**Validatie op de opgeloste boom (exit 0):**
`cargo +1.90.0 test --locked -p artifact-store-core` (9 tests),
`cargo +1.90.0 test --manifest-path ui/native/Cargo.toml --locked`
(57 hosttests), `cargo +1.90.0 clippy --locked -p artifact-store-core --all-targets -- -D warnings`,
`cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml -- --check`,
`python3 tools/progress.py --write`, `python3 tools/progress.py --check`
en `git diff --check`. De eerste UI-poging stopte met exit 1 doordat de
gelijktijdige Rust-installatie een downloadbestand verplaatste; de herhaling
na installatie slaagde. Bestaande rustfmt/Clippy-componenten hersteld met
`rustup component add --toolchain 1.90.0 rustfmt clippy` (exit 0).
Uitvoer: toolsessies `artifact-tests`, `ui-retry`, `clippy`, `fmt`, `progress`.
Een directe vergelijking met Gitblobs bevestigt de ongewijzigde rubric
buiten assessment voor beide parents, behouden UI uit de PR en exacte
tests/bewijsrapporten uit main; geen conflictmarkeringen (exit 0).
Geen nieuwe cross-build of volledige CI-uitvoering.
**Open/eerstvolgend:** nieuwe-head-controles en open reviewpunten door de
bevoegde vervolgstap laten beoordelen. Deze tick is uitsluitend voor
conflictoplossing; geen reviewafhandeling of PR-merge.
Native/QEMU-acceptatie en menselijke audit zijn niet uitgevoerd.

## Historisch checkpoint — PR149-conflictoplossing met main

**Bron:** PR-head `92fdf2ff941284dfaeacffc9ad49295b2e9da21f` met main
`69809c2cb53dfb3bd77abe5352892810fba6e82d`, branch
`simonvanieperen-reimagined-train`. De vier conflicten betreffen alleen
README, PROJECT-STATE, PROGRESS en het assessment in progress.json.
Beide checkpointgeschiedenissen blijven behouden; de gegenereerde meter
wordt opnieuw opgebouwd. Geen handmatige runtime-, rechten- of testcodewijziging.

**Uitgevoerd op de opgeloste boom (alle onderstaande eindcontroles exit 0):**
`python3 tools/progress.py --write`, `python3 tools/progress.py --check`,
`cargo +1.90.0 test --manifest-path ui/native/Cargo.toml --locked`
(57 hosttests), `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml --all -- --check`,
`cargo +1.90.0 clippy --manifest-path ui/native/Cargo.toml --locked --all-targets -- -D warnings`
en `git diff --check`.
`umask 077 && python3 -m unittest discover -s tests -p 'test_storage_backend_replay.py'`
slaagt met 16 tests. De eerste opslagrun met de omgevings-umask faalde
(1 failure, 2 errors: schrijfbare fixturebestanden); de private umask
herstelt de fixturevoorwaarde zonder test-/productiecode te wijzigen.
Rustfmt startte eerst niet door een gelijktijdige toolchaininstallatie;
daarna ontbraken rustfmt/Clippy. Eén `rustup component add --toolchain 1.90.0 rustfmt clippy`
herstelde de bestaande gereedschappen, waarna beide controles slaagden.
De uitvoer staat in de toolsessies `ui-tests`, `storage-tests`,
`storage-private`, `fmt-final` en `clippy-final`; geen raw logs toegevoegd.
Secretscan op de opgeloste bestanden en geïmporteerde Python-code: geen
secrets gevonden. Een tweede AI controleerde alleen-lezen het inhoudsbehoud;
dat is geen menselijke audit.

**Bewijsgrens:** `git diff --quiet HEAD -- ui/native` bevestigt bytegelijkheid
met de PR-parent (exit 0); de vaste rubric buiten assessment is gelijk aan
beide parents. Historische netwerk-, opslag- en native resultaten hieronder
blijven aan hun eigen bron gebonden. QEMU/native gates, cross-build en de
volledige CI zijn deze tick niet uitgevoerd. Score **41 → 41/100**;
geen nieuw criterium, native runtimebewijs of volledige releaseacceptatie.

**Open/eerstvolgend:** na gevalideerde publicatie de nieuwe-head-CI door
GitHub laten beoordelen. Deze tick merge't de PR niet en herstart geen
oude-head-checks. De native preview- en productacceptatie blijven open.

## Historisch checkpoint — native startoppervlak verfijnen

**Bron:** `bd4134c1d565f58736a2c829587f337a415dd47b`, bovenop
`cbbc9c9941b08ea5edfbd3637d494635b4fb1df0` (op dat moment `origin/main`),
branch `simonvanieperen-reimagined-train`.
Het opt-in VM-startscherm heeft nu een duidelijke hiërarchie, een zichtbare
Start-actie en een Materiaalkaart die de echte bibliotheekstatus weergeeft.
De kaart onderscheidt niet-geladen, leeg, fout en bestaand document; de
compacte indeling voorkomt overlap met de navigatiebalk. Er zijn geen
capabilities, authenticatie, opslagrechten of sessieovergangen toegevoegd.

**Gewijzigd:** `ui/native/src/desktop.rs`, `ui/native/src/render.rs`,
`ui/native/tests/render_regressions.rs`, `ui/native/README.md` en
`docs/implementation/SESSION-AND-DESKTOP.md`.

**Uitgevoerd:** Rust 1.90.0 `fmt --check`, `cargo check --tests`,
Clippy (`--all-targets -- -D warnings`) en `git diff --check`: exit 0.
De bestaande scalar `x86_64-unknown-none`-releasebuild van de native UI slaagt
ook (exit 0); het `.rlib` heeft SHA-256
`42eda9397dc739d098b666d9f5a013911fc0b3d867469fde8a558b34b3888b5d`.
Dit is compilebewijs, geen OS-runtimebewijs. De volledige `ui/native`
hosttestsuite slaagt met 55 tests (37 algemene, 5 renderer-, 10 sessie- en
3 fixturetests). Omdat de host `cc` mist, is GCC 15.2/libc6-dev uit Ubuntu
26.04-pakketten alleen naar `/tmp` uitgepakt; er is niets op het systeem
geïnstalleerd. De eerste directe testpoging stopte daardoor op ontbrekende
`cc` (exit 101); de geïsoleerde rerun slaagde. Zonder tijdelijke linker
eindigde preflight met exit 2; met die linker op `PATH` was exit 2 alleen
wegens ontbrekende QEMU en OVMF.
Er is geen QEMU-runtime of native screenshot gemaakt. Commando's, logs en
bewijsgrenzen: `docs/verification/native-start-surface-20261103/README.md`.

**Voortgang:** 41 → 41/100. Geen rubriccriterium gesloten; UI-compilatie en
een toegevoegde test zijn geen native visuele acceptatie. U03/U04 en de
volledige release blijven open.

De vaste meter is opnieuw gegenereerd en gecontroleerd:
`python3 tools/progress.py --write` en `python3 tools/progress.py --check`,
beide exit 0. De assessmentbron is de codecommit hierboven; de huidige
status-/meterupdate is een documentatiecheckpoint daarop.

**Eerstvolgend:** voer bij beschikbaarheid van QEMU en BIOS/VGA-ROM's de
brongebonden sessiepreview uit via `python3 -B tools/run_session_preview.py`
met expliciete `--bios-rom` en `--vga-rom`. De hostrenderer is daarin alleen
oracle; alleen QMP-invoer en pixels uit de echte gast kunnen native bewijs
opleveren. De huidige omgeving heeft QEMU/firmware niet.

## Historisch checkpoint — PR140 bewijs en descriptorclaims — 3 november 2026

**Geteste bron:** `2bf9578290bef88d53c17f9913434da6f27441f2`, inclusief
main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
Op deze exacte bron zijn met Rust 1.90.0 de 9 artifact-store-contracttests,
`cargo check --tests`, Clippy en formattering opnieuw uitgevoerd: elk exit 0.
Exacte commando's, omgeving, source-tree, codehashes en volledige logs met
SHA-256 staan duurzaam in
[pr140-review-evidence.json](verification/pr140-review-evidence.json).
De tijdelijke logpaden zijn alleen oorspronkelijke uitvoerpaden.

De tests bewijzen descriptorvalidatie, binding en foutdoorgifte, niet
identiteit, herkomst of cryptografische authenticatie. Het historische
rapport behoudt bron `08710d55`, commando's, exits en hashes; alleen de
scopeclaim is gecorrigeerd en het verse bewijs toegevoegd.
Score **41 → 41**, geen nieuwe productacceptatie of productiecodewijziging.

**Open:** verse volledige CI en definitieve review vóór merge. Native/QEMU,
fysieke en menselijke acceptatie zijn niet opnieuw uitgevoerd. PR135's
volume-fixture- en RANDOM-gates blijven afzonderlijk open; bron-PRs niet
blind sluiten. **Eerstvolgend:** `GH_PAGER="" gh pr checks 140`; bij
Watch-headwijziging opnieuw head/base en inhoud controleren.

## Historisch checkpoint — PR139 exacte contracttestreceipt — 3 november 2026

**Werkelijk opnieuw geteste bron:** `0a49a3822e5d872f75573f12cd072e76e1249e1b`,
de geïntegreerde PR139-boom met main `d115cc96`.
Negen artifact-store-contracttests, Rust1.90.0 fmt en Clippy: elk exit0.
Volledige primaire logs, exacte commando's, tree en codehashes:
[contracttestreceipt](verification/pr139-artifact-contract-receipt.json).
Dit vervangt geen native acceptatie; eerdere Watch-testclaims zonder
primaire receipt zijn niet retrospectief als geverifieerd bestempeld.
Replay-/umaskbewijs blijft exact aan732 gebonden en OS-workflowhistorie
uitsluitend aan380. **41 → 41**, geen criteriumwijziging.

**Open/eerstvolgend:** `GH_PAGER="" gh pr checks 139`, daarna live
head/base/threads en inhoudsbehoud hercontroleren vóór guarded squash.
Geen testbewijs stilzwijgend naar een latere Watch-head promoveren.

## Historisch checkpoint — PR139 actuele main geïntegreerd — 3 november 2026

Basis vóór integratie: PR-head `341d1b428d436ac0e33c5bf3d7eb684f8543109c`
en actuele `main` `d115cc96c19ac4371baed77409a23d1adddffaf0`.
Conflicten in README, progress.json, PROGRESS.md en PROJECT-STATE.md zijn
opgelost met behoud van PR139-bronrollen en PR140-statushistorie. Main voegt
de `artifact-store-core`-contracttests toe. Eerder gerapporteerd, destijds
zonder primaire PR139-receipt (dus niet onafhankelijk geverifieerd):
`cargo test --locked
--manifest-path crates/artifact-store-core/Cargo.toml` slaagt (9 tests);
`cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml
-- --check` en `cargo +1.90.0 clippy --locked --manifest-path
crates/artifact-store-core/Cargo.toml --all-targets -- -D warnings` slagen.
Dit is contracttestbewijs, geen native acceptatie. Voortgang blijft **41/100**.
Er waren geen reviewtargets meegeleverd, dus geen reviewverzoek is afgehandeld.

De gerapporteerde `b03-native-update`-failure hoort bij oude head `341d1b42`.
Na publicatie van deze conflictintegratie is dat bewijs achterhaald; de nieuwe
head moet haar eigen checks krijgen. Reviews zijn volgens de app nog open.
Eerstvolgend: publiceer de gevalideerde merge en laat nieuwe checks evalueren.

## Historisch checkpoint — PR139 primaire receipts en bronrollen — 3 november 2026

**Geteste bron:** `732fa9cca04ced66b06bee4f0e4f57249c09e96a`, inclusief
main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
De 16 replay-hosttests slaagden met `umask 0077`; dezelfde suite met `umask
0002` faalde echt (1 failure, 2 errors) door groepsschrijfbare fixtures.
Exacte commando's, tree/codehashes en logs: `verification/pr139-review-receipts.json`.
Het historische prioriteitencheckpoint koppelt zijn 15 workflowruns uitsluitend
aan `380111b`; dat bewijs is niet opnieuw uitgevoerd.

## Historisch checkpoint — PR140 main-integratie — 3 november 2026

PR-head `75cda7f4d49c5afe895277ada1102d630fdb75ad` is inhoudsbehoudend
geïntegreerd met main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
De Nederlandse beschrijvingen in het PR140-hostrapport blijven behouden;
main's documentstapel, opslagherstelmateriaal, audit en replaytests zijn
eveneens behouden. De rubric is ongewijzigd: **41 → 41/100**.

**Destijds gerapporteerd:** `umask 077 && python3 -B -m unittest discover -s tests -p
'test_storage_backend_replay.py'` slaagt (16 tests). De eerste poging met de
omgeving-umask `0002` faalde op drie permission-gebonden fixtures; geen
productiecode gewijzigd. `python3 tools/progress.py --write` en `--check`,
JSON-syntaxiscontrole en `git diff --check` slagen.
Dit historische PR140-resultaat is niet het PR139-bewijs op `732fa9cc`
en is niet opnieuw uitgevoerd voor deze PR149-integratie.
**Toen open:** de nieuwe integratiehead moest opnieuw door hosted controles
worden beoordeeld. Geen nieuwe native/QEMU-acceptatie uitgevoerd.

## Historische context — eerdere main/statusintegratie op cbbc9c99

Main is inmiddels `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`: Simon
bevestigde zelf dat hij134 handmatig squash-mergede (3nov10:05:36Z).
Geen concurrerende agent of integratieoverdracht; geen rollback gevraagd.
Die mainbron is nu behoudend opgenomen. Drie assessment/statusconflicten
behouden de recentere documentcombinatie; de TCP-productiefix en haar
volledige regressie-/bewijsbestanden zijn ongewijzigd uit main overgenomen.
De mainrubric is buiten assessment exact gelijk aan de behouden41-puntenrubric.

Nieuwste119-head `5a8317f130cc388e661b876b5258c9978e391460` is nu
volledig behouden. Alleen vier centrale statusconflicten, geen nieuwe code.
Alle rubricvelden buiten assessment zijn identiek; twee latere NAS-
checkpointuitvoeringen blijven historisch, niet verse tests van deze boom.
Ook cloudmain-mergeheads136/143/144/145 zijn geconstateerd: hun exacte
centrale checkpoint-/assessmentbytes zijn met Gitblob en SHA-256 opgeslagen
in `verification/retained-document-heads-20261103.json`. Dit is archivering,
geen stil akkoord op nieuwe heads of sluiting van de bijbehorende PRs.

Bron `1552e5ae1a0e053a6df5ef802f6c0e4e78544cf7` bewaart volledige heads
143/d169796a,144/a974fb9f,145/da5918fb en136/f6921e49 als mergeparents
boven PR114-fix720cd431. Unieke README-codeleesintro, vaste14-commit-
changelogrange, beperkt AI-reviewrapport en volledigheidsaudit zijn behouden.
Vier centrale metadata-conflictklassen zijn inhoudelijk opgelost:
rubrics buiten assessment waren identiek; actuele score41 blijft leidend,
oudere unieke status is historisch opgenomen, niet als actieve opdracht.
De securitytekst noemt overal AI zonder bestandsdekking, geen menselijke audit.
Geen runtime-/tool-/testbron verschilt van720cd431 door deze vier merges.

PR114's vier vervolgthreads zijn beantwoord/resolved in720cd431; volledige
originele82+17 Btrfs-manifestbinding en16 hostregressies blijven behouden.
De audit blijft39 groepen/62open FR-rijen; geen nieuwe productpunten.
Nieuwe gecombineerde CI/review blijft vereist vóór guarded squashmerge.
136/143/144/145 nog niet sluiten: controleer opnieuw actuele heads en bewijs
na daadwerkelijke opname in main. Laatste119-head5a8317f1 is nu behouden.
121-b83 blijft apart
met eerder bewaarde historische metadata; geen blinde ancestryclaim.

Andere stack:135/eea1cdcc bevat echteN01-acceptatie op6f15/source1ca en
onafhankelijk gecontroleerde19VM-volumeproeven opce02/source2c410.
Hosted foreignfixtureartifact/run-ID blijft ontbreken; geen gate overgeslagen.
Main is cbbc9c99;39/41/42/43 blijven open. Integratormandaat is niet
overgedragen. Score **41 -> 41**. Eerstvolgend: actuele publicatie-CI/review
afhandelen, bronbehoud controleren en pas daarna geschikte PRs squashen.

## Historisch checkpoint PR119 — tweede en derde statusresolutie

De tweede119-resolutie op052812b4 combineerde ee5003e6 met main380:
overname van bestaande41 punten, geen nieuwe toekenning (branch34 ->41,
main41 ->41). Vier statusbestanden conflicteerden; buiten die bestanden
werd bytegelijkheid met main vastgelegd. De eigen run meldde39 NAS-
hosttests, exit0,0,806s; `/tmp/nieuw-os-pr119-second-nas.log`.
Progress write/check en diff-check exit0; geen nieuwe Rust/QEMU-uitvoering.

De derde119-resolutie5a8317f1 combineerde052812b4 met maincbbc:
drie statusconflicten, behoud van beide geschiedenissen en de TCP-fix.
Progress write/check en diff-check exit0; bytegelijkheid met main buiten
README/PROJECT-STATE/PROGRESS/progress/SVG exit0. De eerdere vergelijking
zonder SVG-uitzondering gaf1 door datum2 ->3nov, geen codeverschil.
Geen nieuwe Rust/native/QEMU-proef of productpunt. Historische volledige
opdrachten, bronversies en beperkingen staan exact in
`verification/retained-document-heads-20261103.json`; score41 ->41.

## Historisch checkpoint PR134 — terminale TCP-ontvangst

Maincbbc9c99 behoudt de oorspronkelijke134-head27cb3948 via squash.
ReceiveTcp geeft na connect-timeout Closed, terwijl State TimedOut behoudt;
buffer-/tokencontroles blijven intact en opnieuw openen vernieuwt het token.
De oorspronkelijke test was host-rood (Again versus Closed), daarna10
servicetests groen plus35 broker-,26 stack-,4 protocol-,6 client- en11
peerharnastests, format/Clippy/native-librarybuild.
Historische normal/client-exit op5925c915/source9a8b521f: raw33,
8,27/7,79s, peer compleet en cleanup. Dat was nog geen native bewijs
van de nieuwe timeout-ontvangstcase; die beperking blijft expliciet.
`verification/issue-43-receive-timeout.{json,md}` bewaart exacte opdrachten,
hashes en oorspronkelijke open hosted-CI/reviewgrenzen. AI-read-onlycheck
is geen menselijke audit. Geen nieuwe rechten/dependencies of punten.
De latere135-gastasserties en run41614590388 staan afzonderlijk beschreven.

## Historisch checkpoint — PR114-vervolgreview — 3 november 2026

Nieuwe reviewbasis `783a48aac610b42e4e1715ed9551f5012376d12b`:
vier vervolgpunten hersteld. Primaire bronmetadata staat gedeeld in
`verification/native-storage-backend-gate/PRIMARY-SOURCES.md`, met expliciet
verschil tussen handleidingsemantiek en lokaal gemeten metadataverlies.
Alle historische README-/archiefbytes blijven intact.
Lokale Btrfs-manifesten zijn nu aan twee onafhankelijk gepinde oorspronkelijke
receipts gebonden (82+17 entries). Beide volledige lijsten worden vóór enige
payloadhash gecontroleerd; gedeeltelijk/uitgebreid manifest faalt expliciet.
Een gepinde `../metadata_probe.py`-alias blijft na lexicale normalisatie binnen
de expliciete bewijsroot; geen absolute historische bronpaden gevolgd.
De echte nieuwe regressies waren rood (2 failures/2 errors); daarna16
hosttests groen en207 oorspronkelijke archiefhashes plus beide receipts
gecontroleerd. Geen full-local-payloads-, engine- of native replay op114.
Score **41 -> 41**. Eerst publiceren/reply+resolve, daarna verse CI/review;
technisch groene checks op783 vervangen geen review van deze followup.
Integrator blijft deze sessie;136-reconcile41034178 wacht lokale opname.

## Historisch checkpoint PR136 — volledigheidsaudit — 3 november 2026

Volledige bron `f6921e49d08a7c547ffda356f71bc17f1de15f31` behoudt d777,
reconcile `d7b4c378` en bewijs `41034178`. De historische native L02-bronnen
433bab9f/7fb8c722 zijn niet als ontbrekende port voorgesteld; geïntegreerde
FR-L04-regressie en fysiekeQPU blijven apart. A01 accepted2/R03open
verduidelijkt beperkte SDKacceptatie, geen releasegovernance of extra punten.
Parent valideerde de zeven geleverde bestandshashes, Draft2020-12,
39 unieke groepen,62 releasekoppelingen, VM-telling21 en0 gesloten
volledige releasecriteria. D777's exact-head-CI-entry is behouden.
Zie `verification/pr136-review-fixes.json` en
`verification/pr136-integrator-check.json` voor exacte commando's en logs.

De eerdere auditcontrole op4a52 rapporteerde29 groene checks, één overgeslagen
CodeRabbit-job en nul mislukt; vier toenmalige threads waren later opgelost.
Die branch-/reviewstatus blijft historisch, niet een verklaring dat de
nieuwe documentcombinatie is geaccepteerd. Geen nieuwe lokale productbuild,
native/QEMU-, hardware-, gebruikerstest, menselijke securityaudit of
releasevalidatie. Alle62 FR-rijen blijven open; score41 ->41.
De integrator is niet overgedragen. Eerstvolgend was nieuwe-head-CI/review.

## Historisch checkpoint PR145 — statische AI-review — 3 november 2026

Overgenomen van `da5918fbe6d8173c4cf44810d403f965c204d36e`, rapportbasis
`380111b68688c981eec6abb8fab29174a29f5601`, toenmalige branch
`simonvanieperen-effective-tribble`. De AI-review meldde geen bevestigde
kwetsbaarheid maar leverde geen lijst van gelezen bestanden/regels of
subsystemendekking. Dit is geen onafhankelijke menselijke audit, volledige
codebase-audit of bewezen branchdiff-dekking.

De toenmalige Dependabot-alert-API gaf geen open meldingen en de gerapporteerde
manifestdiff tegenover `origin/main` was leeg. Dat is geen volledige
dependency-advisoryscan. Secret Scanning stond uit (404); gitleaks,
trufflehog, detect-secrets, cargo-audit, osv-scanner en dependabot ontbraken.
Geen volledige scan, codefix, tests of builds uitgevoerd. Score41 ->41.
De toenmalige vervolgstap was volledige scans uitvoeren zodra tooling of
Secret Scanning beschikbaar is; geen afwezigheid van secrets/risico's
afleiden uit deze beperkte uitkomst. Volledige herkomst en beperkingen:
`verification/security-audit-20261103.md`.

## Historisch checkpoint PR144 — changelog — 3 november 2026

Overgenomen van `a974fb9f731847a8b4dc55f1e080ced945a58e63`, basis
`380111b68688c981eec6abb8fab29174a29f5601`, branch
`simonvanieperen-shiny-waddle`. `CHANGELOG.md` vat de14 commits in het vaste
bereik `ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601`
samen, niet een veranderlijk bereik "sinds main". Documentatie-only,
geen OS-tests of nieuwe productacceptatie; score41 ->41.

Het oorspronkelijke checkpoint signaleerde dat de HEAD-tekst30 CI-controles
meldde terwijl `pr127-review-integration.json` exacte-head/native bewijs nog
open liet. Dat is een historische provenancewaarschuwing, geen actuele
afwijzing van later brongebonden postmergebewijs. De changelogtaak heeft
die tests niet herhaald. Fysieke hardware, menselijke audit en volledige
testrelease bleven open; eerstvolgend was brongebonden CI-resultaten vastleggen.

## Historisch checkpoint — eerste PR114-reviewfixintegratie

Bron `d7d1fdc0356be47c225d7bd216075c4a62030ad4`: researchfix
`fcbf64e3`, veilige bewijsroute `305f1133` en historische metadatafix
`6d0852fa` zijn inhoudsbehoudend opgenomen. Vier researchbevindingen
zijn met echte Markdownrendering gecontroleerd;242 bronlinkgrenzen en
Ubuntu-desktop/Initial-Setup-volgorde hersteld.
Alle207 oorspronkelijke opslagblobhashes blijven geldig. Onveilige oude
scripts zijn niet-uitvoerbare historische snapshots, niet veilige huidige
entrypoints. Nieuwe private offlinevoorbereiding/bewijscontrole heeft7
groene negatieve hosttests. Volledige foreign-engine/native replay blijft
open; OpenZFS-create/crash is expliciet geblokkeerd, niet fictief gerepareerd.
Bewijs: `verification/pr114-doc4-fixes.json` en
`verification/pr114-storage-review-fixes.json`. Score **41 -> 41**.
Publicatiehead vereist verse CI/review en inhoudelijke threadafhandeling.
De eerdere guarded merge is door GitHub geweigerd; geen merge of bypass.
Main blijft380111b6.135 bevat de latere desktop-/volumestapel en134 wordt
afzonderlijk inhoudsbehoudend geïntegreerd; rode fixturelevering en B03-
flushcompletion moeten vóór merge opgelost/geverifieerd worden.
Eerstvolgend: publicatiehead controleren, threads beantwoorden en actuele
CI uitvoeren; geen fysieke writes, productiekeys of volledige releaseclaim.

## Historisch checkpoint — ontwerp, onderzoek en opslagbewijs combineren

Actuele metadatareviewbron: `52e1909cb2667a5862732c9d2dc74a6f81b4e016`.
De verwijzing naar het gebruikerswisselcontract noemt nu de echte
fixcommit68d427b in plaats van diens basis c1d62d83; score41 blijft41.

Basis `517117fbd8359084eca10e2e2d5f4cdf1015b52c` (PR114 en main74a34d05)
met PR131 `a03f7f0c355cedca72611893f07c4a766a7b7d4f`. Integrator `f090b913`.
Geen runtimecode uit onderzoek; ADR-058 blijft leidend voor de shell,
P2-bevriezing betreft het historische prototype. De onafhankelijke
read-only broncontrole bevestigt de Eurobarometer58%-dienstoverstapclaim
tegen de originele data-annex/PDF-hash; dit is geen OS-adoptiepercentage.
De onderzoeks- en ontwerpcheckpointteksten hieronder blijven historisch
behouden; hun toenmalige branchstatus is geen actuele integratie-instructie.
Ook PR132 `6afdf306f19072c19fd572701a6126220a4d56d2` wordt behouden:
de read-only AI-controle bevestigt207 echte Git-blobs en acht gekoppelde
bootmanifesten. Dit is archiefintegriteit, geen onafhankelijke native replay
of menselijke audit. Btrfs/OpenZFS-adoptie en S09 blijven open.
Nieuwste main is `380111b68688c981eec6abb8fab29174a29f5601` via PR127:
exact dezelfde tree als geteste head5343b0a,31 actuele checks geslaagd,
review zonder bevindingen en nul open threads. Native22fasen/raw33,
clipboard/herstel en onafhankelijke14-byte/rev1-schijfcontrole zijn groen;
beide verse N01-netwerkcases raw33. Main gaat34 -> 41 met U01/N01;
de documentatie zelf voegt0punten toe. Postmerge-CI loopt nog afzonderlijk.
PR126 en122 zijn na actuele head/ancestry/treecontrole als overgenomen gesloten.
Details: [mainacceptatie](verification/pr127-main-acceptance.json).
Bij documentintegratie zijn vijf botsende UI-ADR-nummers uniek gemaakt
(059-063), met oude titelankers en historische bewijsnummers behouden.
De laatste PR119-head `ee5003e6` is eveneens volledig behouden; unieke
PR121-statusinformatie `b83e4e05` is onderaan als historisch checkpoint
opgenomen. Controles/bewijsgrenzen: verification/pr114-doc-integration.json.
Vervolg: verse exacte-head CI/review en afstemming met de desktopintegratie.

## Onderzoekscheckpoint PR131 — volledige OS-journey — 2 november 2026

**Scope:** uitsluitend onderzoek en documentatiestatus, geen productcode.
Bronbasis main `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; branch
`codex/os-research-phases-2-4-20261102`. De apart gelezen ontwikkelbranch
`codex/mature-native-session-20261102` is vastgepind op `1f0dd9c93959e3b1335656da1e548525a288133e`
en niet in deze documentatiebranch geïntegreerd.

**Parallelle integratie:** laatste remotecontrole ziet main
`74a34d05ef5591b782a0da9da34e4669910d0722` (#125, nog 34/100).
Deze onderzoeksbranch behoudt de bovenstaande bronbasis; er is geen
productcode uit main ingemengd. Concept-PR #131 meldt integratieconflicten.
De aangewezen integrator moet bij opname de nieuwste main-status en
bestaande evidence behouden; onderzoeksvoorstellen leveren geen punten op.

**Geleverd:** [fase 2–4](research/2026-11-02/README.md) en de
[volledige installatie-/day-one-journey](research/2026-11-02/journey/README.md).
De uitbreiding bevat 31 installatieaspecten, 28 dagelijkse taken,
herkomstprofielen, mentale modellen, familiarity budget en afzonderlijk
gelabelde installer-/onboarding-/interactievoorstellen. Het bestaande
PRODUCT-SPEC/ROADMAP is pas na de vergelijking begrensd aangevuld;
P2 blijft bevroren. Bronregisters geven versie, datum, passage en beperkingen.
Eurobarometer 572/QB6.4 is gecontroleerd in originele vragenlijst, rapport en
data-annex: 21+37=58% instemming met dienstoverstap, geen OS-migratie.
Bronafwijkingen bij technische rapportkop en Cyprus-landlabel zijn zichtbaar bewaard.

**Bewijs en score:** mainbasis 34/100 → 34/100; geen nieuwe native, fysieke,
menselijke of releaseacceptatie. Ontwikkelbron `1f0dd9c` rapporteert afzonderlijk
41/100; deze onderzoekstaak kopieert die score niet naar main.
Geen installaties, fysieke schijven, productcode of echte geheimen gebruikt.
Document-/bronverwijzingen, matrixdekking, scope en progressgeneratie
worden vóór commit gecontroleerd; geen kernelbuild/QEMU-herhaling voor deze
documentatietaak. Definitieve controles:
[fase 2–4](research/2026-11-02/verification.json) en
[journey](research/2026-11-02/journey/verification.json).

**Open:** geen eigen gebruikers-/installatieproef, representatieve
EU-studie, praktijkmigratie of retentiemeting. Exacte macOS-paneelposities en
enkele KDE-staptellingen blijven onbekend; actuele KDE-ingangen zijn op
v6.8.0-code bevestigd. Native installer, persoonlijke authenticatie,
encryptie-/recoveryketen en OS-brede toegankelijkheid blijven afzonderlijke gates.
**Eerstvolgend:** lees journey-index, installer IN-/onboarding ON-/UX-criteria
naast de actuele ontwikkelbron. Hercontroleer code/services vóór een begrensde
niet-schrijvende proef. Productcode blijft buiten deze opdracht. Bestaande
integrator- en mergeafspraken blijven behouden; concept-PR wordt niet gemerged.

## Ontwerpcheckpoint PR114 — gebruikerswisselcontract — 2 november 2026

Fixbron `68d427b76fd4c5fc0a071433d567097038e71a65`, alleen documentatie.
De exclusieve integrator is `f090b913`; hieronder staat historisch mandaat,
geen nieuwe toewijzing. Main is inmiddels `74a34d05` via PR125; #109/#110
zijn aantoonbaar gesloten. Deze ontwerpbranch is nu inhoudsbehoudend met
main74a34d05 gecombineerd; latere desktopdocumentatie blijft apart.

PR114-review4428521664 is uitgewerkt: A's sessiehandles intrekken of via
generatie blokkeren voordat B actief wordt; A's sleutels blijven uitsluitend
in diens geisoleerde domein; B krijgt geen handles, objecten of mappings van A.
Terugkeer vereist authenticatie en nieuwe handles, geen stille herleving.
Afzonderlijk toegestane achtergrondsync blijft aan A en bestaande scope
gebonden. Bij onvoldoende scheiding wisselen weigeren tot expliciet
afmelden, geen stil verlies van werk. Native A/B/key-state/foutinjectie-
acceptatie is benoemd, **niet uitgevoerd**; geen implementatieclaim.

Controles: `python3 tools/progress.py --write`,
`python3 tools/progress.py --check`, `git diff --check`.
Score **34 -> 34**, nul nieuwe punten. Eerstvolgend: actuele main en
overige ontwerpvoorstellen inhoudsbehoudend combineren en verse CI/review;
PR127 heeft apart de gecombineerde desktop/native-netwerkgates lopen.

## Opslagcheckpoint PR132 — native backendtoets — 2 november 2026

**Bronnen:** eerste proef `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`;
bevestigingsproef `74a34d05ef5591b782a0da9da34e4669910d0722`; bewijsbranch
`codex/native-storage-backend-gate-20261102`. Eigen kernel en drivers ongewijzigd.
Alleen wegwerpbestanden; vreemde datadisks uitsluitend read-only. Vier nieuwe
VM-runs op iedere bron (acht totaal), met dezelfde uitkomsten; de eerdere
bewijzen zijn behouden en de upstreamintegratie is volledig overgenomen.

**Uitgevoerd:** opnieuw gebouwde native NOSVOL02-normal en cold-recover slagen
(QEMU raw exit 33). Btrfs en één OpenZFS-mirrorlid leveren geen native import
of bestandsherstel (exit 35). Btrfs-nulheaders worden als lege proefstaat
toegelaten, waarna de documentcontrole faalt; dit is geen veilige generieke
formaatafwijzing. OpenZFS wordt als Corrupt geweigerd. Alle vreemde bron- en
proefimagehashes zijn gelijk voor/na de boots.

**Hostbewijs:** Btrfs-reservesuperblockherstel slaagt; databitcorruptie wordt
gedetecteerd maar `restore` geeft corrupte inhoud met exit 0 terug. Aparte
metadata-import/salvage verliest onder meer hardlinktopologie en lege-mapdata.
OpenZFS-libzpool/zdb kan de hostpool controleren/importeren; `ztest -E` heeft
een `/dev/zfs`-discoverygrens, de crashproef meldt een onopgelost allocatieverschil
en gewone bestands-/backuprestore is niet aangetoond. Dit is geen native port.

**Besluit en score:** native adoptie van beide backends blijft niet geaccepteerd;
geen eigen driver of nieuw diskformaat gestart. **34/100 → 34/100**; S09 blijft
open. Bestaand positief NOSVOL02-bewijs wordt niet uitgebreid tot vreemde FS'en.
Zie [toets en reproduceerroute](verification/native-storage-backend-gate/README.md).
De oorspronkelijke hoofdcheckout en ongerelateerde `.github/github-app.yml`
zijn behouden; deze branch wijzigt uitsluitend bewijs/documentatie/assessment.

**Open:** positieve formaatidentiteit vóór writable toelating; gecontroleerde
bestaande-engineport naar de eigen ABI; onafhankelijk inhouds- én metadata-orakel;
verklaring van de OpenZFS-harnasbevindingen. Geen fysieke powercut-, volledige
snapshot-/encryptie-/backup- of prestatietoelating.

**Eerstvolgende controle:**
`python3 tools/progress.py --check`, daarna de beschreven native-enginepoort
beoordelen zonder een Linux-kernel of nieuw bestandssysteem als omweg in te voeren.

## NAS-checkpoint PR119 — conflictoplossing — 2 november 2026

**Bron:** PR-head `fa97c69221fd9b5635684c48b60e72bf67b2541a`, branch
`claude/issue-110-nas-replay`; geïntegreerde main:
`74a34d05ef5591b782a0da9da34e4669910d0722`. Alleen conflictoplossing is
geautoriseerd; de PR wordt niet gemerged en oude CI wordt niet herstart.

**Inhoudsbehoud:** main bevat de replay-implementatie en het bewijs uit deze
PR al via #125. De vijf conflicten zijn opgelost met behoud van mains latere
lekcontrole, negatieve tests, contractcorrecties en ruwe-bewijsattributen:
`.gitattributes`, `docs/verification/nas-replay-110/README.md`,
`services/nas/native-probe/CONTRACT.md`, `tests/test_nas_runner.py` en
`tools/test_nas.py`. Vóór dit status-/metercheckpoint was de volledige
opgeloste werkboom bytegelijk aan main (`git diff --exit-code origin/main -- .`,
exit 0). Er is geen nieuwe kernel-, interface- of rechtenwijziging ontworpen.

**Lokale verificatie op de opgeloste boom:**
- `python3 -B -m unittest discover -s tests -p 'test_nas_*.py' -v`:
  exit 0, 39 tests; `/tmp/nieuw-os-pr119-nas-host.log`.
- `python3 -B -m unittest discover -s services/nas/native-probe/tests -p 'test_*.py' -v`:
  exit 0, 17 tests; `/tmp/nieuw-os-pr119-replay-oracle.log`.
- `python3 -B -m unittest discover -s tests -p 'test_volume_runner.py' -v`:
  exit 0, 3 tests; `/tmp/nieuw-os-pr119-volume-host.log`.
- `sha256sum --check SHA256SUMS` vanuit `docs/verification/nas-replay-110`:
  exit 0; `/tmp/nieuw-os-pr119-evidence.log`.
- `python3 -B tools/preflight.py --strict`: exit 2;
  `/tmp/nieuw-os-pr119-preflight.json`. Rust 1.90.0, beide native targets,
  QEMU en OVMF ontbreken. Native build/QEMU en Rust-format/lint zijn daarom
  niet uitgevoerd; dit is geen nieuwe native acceptatie. De gerichte
  hosttests en exacte vergelijking met main verifiëren de conflictoplossing.

**Score en vervolg:** 34/100 → 34/100; geen criterium of bewijsniveau wijzigt.
Na publicatie moeten de checks op de nieuwe PR-head worden beoordeeld.
Historische bewijsbundels blijven aan hun eigen bron gebonden; alle open
product-, fysieke en menselijke acceptatiegrenzen blijven bestaan.
Eerstvolgende lokale controle bij hervatten: `git status --short`.

## Historisch checkpoint PR121 — conflictoplossing — 2 november 2026

Overgenomen van `b83e4e05b923dc2c0d8b4339fc9509b5752cb384`;
onderstaande resultaten zijn die historische uitvoering, niet hier herhaald.
De runtime-inhoud van ouder `0fe648b87d1f4959087d3f80a251a7d6fb64dd66`
zit in de PR127-ancestry; de nieuwe statuscommit zelf nog niet in main.

**Bron:** mergecommit `8686f977a7f1fbfba1eb1214d301935c1859e8ed`, met ouders
PR-head `0fe648b87d1f4959087d3f80a251a7d6fb64dd66` en actuele `main`
`74a34d05ef5591b782a0da9da34e4669910d0722`. De conflictintegratie behoudt de
native sessiepreview en de wijzigingen en bewijsbestanden van `main`. In
`ui/native/tests/interaction.rs` staan zowel de PR-desktopproeven als de
drie main-regressies voor focus en verouderde pointergebaren.

**Historische validatie:** `cargo test --manifest-path ui/native/Cargo.toml
--test interaction`: 34 tests geslaagd; `cargo clippy --manifest-path
ui/native/Cargo.toml --test interaction -- -D warnings` en
`rustfmt --edition 2021 --check ui/native/tests/interaction.rs` slagen.
De PR-delta `git diff --check origin/main` is schoon. Full-release-,
NAS-replay- en sparse-frame-`SHA256SUMS`-controles slagen. Een volledige
diff-check waarschuwde voor hashgebonden CRLF-/blank-at-EOF-bewijsbytes;
die bytes zijn niet aangepast.

**Historische scan/bewijsgrens:** CodeQL vond geen Python-alerts maar liep vast
voordat alle talen klaar waren; geen volledige CodeQL-scan. De app meldde vóór
deze integratie geen vereiste checks; nieuwe-headchecks bleven vereist.
Score34, geen productacceptatiebesluit. De toenmalige vraag
om publicatie via `report_progress` is historisch, geen nieuw mandaat.

## Historisch checkpoint — mainstand en documentatie — 3 november 2026

**Basis:** `380111b68688c981eec6abb8fab29174a29f5601` op
`simonvanieperen-laughing-fortnight`, destijds gelijk aan `origin/main`; parent
`74a34d05ef5591b782a0da9da34e4669910d0722`. PR126-inhoud en de zeven
PR127-reviewfixes zijn nu in deze main-commit geïntegreerd.

**Exact-head controles:** GitHub rapporteert 28/28 check-runs als `success`.
De [native venstersessie-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41535553113)
sluit de actuele QEMU-route met echte invoer, pixel-/isolatieasserties,
compositorherstel en schijfcontrole af; de
[netwerk-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41535553158)
sluit hostkwaliteit en de native netwerkdienst met gecontroleerde peer af.
Dit is geen Windows-runtime-, fysieke-hardware-, menselijke-audit- of
volledige-releasebewijs.

**Documentatie:** README, status, voortgangsassessment en verificatierecord
zijn bijgewerkt naar deze bron. Alle publieke functies in
`kernel-core`, `artifact-store-core` en de window-session-scene hebben nu
Rustdoc; comments
verduidelijken beschadigingstegels en toetsrelease-eigenaarschap. Deze
documentatiewijziging verandert geen OS-gedrag of acceptatiecriterium.

**Voortgang:** **41/100 → 41/100**, rubric v1; er zijn geen punten toegekend.
U02, authenticatie/vergrendeling, de resterende releasecriteria, fysieke
hardware en de volledige testrelease blijven open.

**Uitgevoerd:** `python3 tools/progress.py --write` exit 0;
`python3 tools/progress.py --check` exit 0; `git diff --check` en beide
`python3 -m json.tool`-controles exit 0. Publieke-functiedekking van de drie
gewijzigde Rust-bronbestanden is statisch gecontroleerd. Exact-head status is
gecontroleerd met `gh api repos/simonvanieperen/nieuw-os/commits/380111b68688c981eec6abb8fab29174a29f5601/check-runs --jq '.total_count'`
(28, alle success), plus `gh run view 41535553113 --json conclusion,headSha,jobs,workflowName,url`
en `gh run view 41535553158 --json conclusion,headSha,jobs,workflowName,url` (beide exit 0). Rustdocgeneratie
`RUSTDOCFLAGS='-D missing-docs' cargo doc --locked --no-deps -p capability-core -p kernel-core -p virtio-core -p ui-core -p update-core -p app-package-core -p artifact-store-core`
is **niet uitgevoerd**: exit 127, `cargo: command not found`.
`python3 -B tools/dev.py cargo +1.90.0 --version` exit 1 doordat de Cargo-
binary ontbreekt. CodeRabbit CLI-review is niet gestart; het uitvoerbare
bestand was niet vindbaar en beschikbaarheid/authenticatie zijn onbekend.

**Eerstvolgend commando:** `git status --short && git rev-parse HEAD &&
python3 -B tools/progress.py --check`; raadpleeg daarna `docs/WORK-QUEUE.md`
voor het volgende productwerk.

## Native checkpoint PR127 — zeven reviewfixes — 2 november 2026

**Basis:** PR127 `798706c2a53d0062ce0474e5f84f294b2c37b825`, drie lokale
fixleaves en PR126 `8f08db4c12573f91b57e55c30fa417e7abd358e6`, met main
`74a34d05ef5591b782a0da9da34e4669910d0722`. Integrator blijft `f090b913`.
Deze PR127-kandidaat behoudt de volledige main/sparse/retained/expiry/
clipboardinhoud en herstelt paletdispatch, proportionele pointerposities,
TCP-deadline/Closed, probegebruikersconfig, volledige dockhoverdamage en
write-only semantisch wachtwoord. Geen capabilities of dependencies verruimd.

**Nieuwe echte integratiebevindingen:** dubbele caretfunctie na automerge
opgelost met behoud van de proportionele implementatie. Bestaande
damage-regressie vond pixels buiten de oude 16px-/monospacevlakken;
renderer en model delen nu resultaat-/statusbounds met echte fonthoogten
en het formuleveld wordt correct begrensd hertekend. Alle bestaande
pixel- en >=90%-SUR-sendbudgetasserties blijven behouden. De nieuwe
wachtwoordtest gebruikte een ongeldige 1600x1200-viewport; beide schalen
worden nu op ondersteunde 800x600 fysieke pixels getest.

**Uitgevoerd:** 189 Rust-app/UI-tests en 31 Python-harnastests, exit0 in22s;
9 TCP-tests met echte gecontroleerde smoltcp-peer op Rust1.91.1, exit0.
Beide TCP-bugs zijn eerst op oude productiecode echt rood gereproduceerd
(7pass/2fail, exit101), niet alleen statisch aannemelijk gemaakt.
Clippy/fmt en echte native-targetchecks/releasebuild slagen.
Exacte commands/loghashes: verification/pr127-review-integration.json.

**Parentbewijs:** alle actuele PR126-jobs slagen. Native run41530207152,
artifact13182057101, testmergea1cb32b met exact main74a34d05/head8f08:
gelijke tree92c3200, source8b2238b2,22fasen/raw33/native_gate/clipboard_gate,
328,969s, herstel en schijf14bytes/rev1/canaries groen. Lokale volledige
PR126-probebuild eveneens exit0 in8s na hergebruik van Rust1.90 met private
shims. Dit bewijst NIET de nieuwe PR127-bron.

**Open:** verse gecombineerde CI/review en volledige native-/herstel-/
pixel-/klembord-/schijfgates. PR126 heeft een nieuwe TCP-Closed-review
die juist in deze kandidaat is hersteld: dus niet los ongefixte PR126 mergen.
De netwerk-CI herhaalt nu ook de bestaande normal/client-exit-QEMU-gates
op schone eigen-kernelimages: na TCP-productiewijzigingen is ouder N01-bewijs
alleen onvoldoende. Deze nieuwe job is nog niet als geslaagd aangemerkt.
Pas na main-inhoudsbehoud ingehaalde bron-PRs sluiten. #109/#110 zijn al
aantoonbaar via #125/main opgelost; #39-#43 blijven inhoudelijk open.
Branchscore **41 → 41**, main **34 → 34**; nul nieuwe punten.
Eerstvolgend: nieuwe `native-window-session` en netwerk-CI op deze head
controleren, reviewthreads inhoudelijk beantwoorden en nieuwe bevindingen
oplossen. Geen Windows/WHPX-, fysieke, menselijke of volledige releaseclaim.

## Vorig checkpoint — main geïntegreerd; gezamenlijke desktopgate volgt — 2 november 2026

**Basis:** PR126 `51580271eada508d9a1edcac5a1e3599f6d726f5` met main
`74a34d05ef5591b782a0da9da34e4669910d0722`; exclusieve integrator blijft
`f090b913`. PR125 is met 29 actuele groene jobs, actuele code-review zonder
bevindingen en head/base/treeguard gesquasht. Main-tree is bytegelijk aan
geteste kandidaat `b2761f1`. #109/#110 en aantoonbaar opgenomen bron-PRs
#115/#116/#117/#118/#120 zijn gesloten. #119 kreeg gelijktijdig een cloudmerge
`ee5003e`; alleen vier status-/meterdocumenten verschillen nog van main.
Die nieuwe bron is niet ten onrechte als oude ancestor gesloten.

**Deze integratie:** alle main-/PR120-followups en alle desktop/retained/
expiry/clipboardfuncties behouden. SUR1-sparsecontrole zit nu ook in de
retained-positionering; hersynchronisatie blijft volledig en doorlopend.
Materiaal behoudt beide APIs maar mengt geen impliciete digestgaten met
expliciete retain-coverage. Een gecombineerde cache+retained-regressie
controleert iedere tegel, exacte pixels en framecompletion. Bestaande
capability-/UI-tests van beide kanten zijn bytebehoudend samengevoegd.
App-hosttests voor Materiaal en Rekenmachine zijn aan de bestaande native
workflow toegevoegd; geen dependency of recht verruimd.

**Uitgevoerd:** volledige Rust-hostsuites voor window-session, Materiaal,
SUR1/client, ui/native en capability-core: exit 0. De vroegere ontbrekende
`cc` is hersteld met alleen in de sessiemap uitgepakte, gepinde en gehashte
Ubuntu GCC15/libgcc/libc-dev-pakketten; geen systeeminstallatie. Rust1.90,
host- en echte native-target type/lintchecks slagen. Exacte commando's,
exits, aantallen en loghashes: verification/pr126-main-integration.json.
De groene hostsuite is geen QEMU-, fysiek of Windows-bewijs.

**Nog open:** nieuwe exacte-head hosted CI met volledige native venstersessie,
herstel, pixels, klembord, schijf en package. #127-fixleaves zijn apart lokaal
gecombineerd; neem die later hunkgewijs op en behoud deze expiry/retained-route.
Main-meter **34 blijft 34**; deze branch behoudt zijn eerdere **41 → 41**
zonder nieuwe punten. U02, alle FR-rijen en volledige release blijven open.
Eerstvolgend: nieuwe `native-window-session`-run en alle concrete nieuwe
reviewbevindingen op de gezamenlijke head controleren, niet oude jobs hergebruiken.

## Vorig checkpoint — PR126: tweede fixturefout hersteld — 2 november 2026

Basis `bd981f79a234dc45fbf38ea1cae25c08ecd1e9c1`, branch
`codex/native-retained-frames-20261102`; main-integratie blijft uitsluitend
bij integrator `f090b913`. Die fixcommit activeert retained rendering alleen
voor de venstersessie en verwerkt verlaten SUR1-transacties via de bestaande
timeoutreply/backpressureroute. Geen ruimere capabilities of dependencies.

De klokcorrectie is uitgevoerd in CI-run `41526505812`, job `125555496660`.
Die run toont een tweede fixturefout op tests/idle.rs:689: de gedeeltelijke
Chunk gebruikte 1 byte, terwijl SUR1 voor iedere chunk exact
`min(resterend, CHUNK)` bytes vereist. De fixture stuurt nu de eerste geldige
volledige chunk van een nog onvolledige tegel. Een assert controleert dat
deze chunk kleiner is dan de tegel. Geen productie- of protocolwijziging;
de exacte 209/210-grens, nul bevestigde pixels, eenmaal timeout, tokenweigering,
monotone ticks en herstelasserties blijven behouden. De CI-hostsuite was
dus nog rood (exit 101), niet groen; de native fase werd niet bereikt.

`cargo fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check`
en `cargo clippy --offline --locked --manifest-path
sdk/examples/window-session/Cargo.toml --all-targets --target-dir
target/pr126-clock -- -D warnings`: beide exit 0 met bestaande Rust 1.90.0.
Log `/tmp/nieuw-os-pr126-chunk-clippy.log`. Lokale Rust-testuitvoering blijft
geblokkeerd door ontbrekende `cc`; geen QEMU/Windows uitgevoerd. Nieuwe
exacte-head CI moet het fixtureherstel en de native route nog bevestigen.
Rapport: `docs/verification/pr126-review-fixes-20261102.json`.

Geerfde ontwikkelbranchscore **41/100 → 41/100**; nul nieuwe punten voor
deze fixes. Dit is geen automatische overname van 41 punten op main
(integratieassessment #125 blijft afzonderlijk 34/100).
Eerstvolgend: na push `native-window-session` en alle relevante nieuwe
checks op de exacte head controleren; niet mergen zolang een gate rood is.

## Vorig checkpoint — PR126 CI-interfaceherstel — 2 november 2026

Basis `fb5810e45bd8d2ab4a73d886d6c23cf526b10cec`, branch
`codex/native-retained-frames-20261102`. Alleen de drie aangewezen falende
CI-controles onderzocht; geen merge of wijziging van capabilities/dependencies.
Ontbrekende damage-/presentatie-interfaces en scrolbewuste cursorplaatsing
hersteld; de native window-compositor behandelt nu ook gevalideerde Retain-replies.
Damage blijft staan over gedeeltelijk vervangen frames en wordt alleen na
actuele volledige presentatie gewist. Backspace op een automatische
regelovergang is afzonderlijk rood/groen gereproduceerd en afgedekt.

**Bewijs:** 169 gerichte hosttests groen (rekenmachine36, Materiaal40,
native UI52, window-session41), Clippy/fmt voor deze vier crates groen.
Alle acht native app-ELFs gebouwd/geïnspecteerd; ook de normale compositor
zonder fault-injection gebouwd. Dit is geen native runtimeacceptatie.
De integrale builder stopt daarna op ontbrekende offline dependency
`hex-literal`; één gerichte SDK dependency-fetch slaagde eerder.
Strict preflight meldt ontbrekende QEMU/OVMF. Volledige QEMU-route,
Windows-pakket en hosted CI op de herstelcommit zijn **niet uitgevoerd**.
Commandostatussen, bron-/ELF-hashes en lokale logpaden staan in
`verification/gui-idle-20261102/agent-merge-ci-repair.json`.
**41/100 → 41/100:** U02 en de volledige releaseacceptatie blijven open.
Eerstvolgende controle: de nieuwe CI-resultaten laten beoordelen; lokale
runtimeacceptatie pas met volledige dependencies, QEMU en expliciete firmware.

## Vorig checkpoint — native klembord geïntegreerd; U02 blijft open — 2 november 2026

Branch `codex/mature-native-session-20261102`, concept-PR122 op PR121.
PR124 is na18 groene CI-workflows en code-/artifactreview opgenomen als
`9863a52651504c4838d6826b3e1b2240ae0ed6b2`. Geen merge naar main;
de bestaande integrator behoudt die verantwoordelijkheid. **41/100 blijft41**.
U02, bredere clipboardcapaciteit, volledige interactiebudgetten, echte
accountauthenticatie en de volledige release blijven open.

**Nieuw:** echte CtrlC/CtrlV tussen twee geïsoleerde native processen. De
compositor controleert huidige focus, echte PID/inputepoch en het eigen nog
staande CtrlC/V-Press-aanbod; één aanbod geeft één gebruik. InputSession
blijft de enige RX-eigenaar. Pendingbackpressure bewaart alleen ticketmetadata,
geen plaintext; iedere flush valideert opnieuw. De app past ontvangen Text pas
na echte ACKAcknowledged toe; Superseded en transportfalen wissen de tijdelijke
kopie. Atomische rekenmachinepaste valideert de hele tekst vóór mutatie en
start geen berekening. Copy wist vooraf oude inhoud; CtrlShiftDelete wist de
vluchtige clipboard; compositorherstart begint leeg en behoudt de appmodellen.
Geen nieuwe kernel-ABI, proces, apparaat-/opslag-/netwerkrecht of dependencyversie.
Eerste capaciteit192 UTF-8-bytes met zichtbare weigering, zonder afkappen,
historie, hele-document- of oude-antwoordfallback. Geen credential-/authclaim.

**Bron en bewijs:** functionele d0ae570bd4666626096f92e57acef981c0f0d7b6,
source-v3 05307fef9684d97480a77ea39050d80f6affcf522a6c09c7eb53340c789b35aa.
Root-TCG/q35-8.2/qemu64/1CPU/256MiB/1280×720 slaagt in1196,887s,
alle22fasen/raw33/clipboard_gate=true. Oorspronkelijke13fasen behouden,
9clipboardfasen met werkelijk4Copy/5Paste/1Clear, roundtrip3→3+4,
atomisch letters weigeren, clear en post-faultEmpty. Source/frame/ACK/fault/
PID/epoch/werkbehoud en onafhankelijke diskoracle14bytes/rev1/canaries/cleanup0
zijn gecontroleerd. Rawserialhashb5bd0b0e…72961. Run:
build/clipboard-native/run/window-2efc331ed02d4f89bea34d7287840cde.
Screenshot clipboard-calculator-copied.png is verliesloos uit de echtePPM.

De finale e3182f0bba72f300596dd81417fbbd371a8d5d79 wijzigt daarna uitsluitend
beide app-chain-hosttests. Volledige source-v3:
038615b1b425d3f24ea2d9c52b36184ea666f81237c813303dc0fac93fb756ca.
Herbouw op dezelfde omgeving geeft alle10binaireSHA256s identiek. Oude rootrun
blijft op d0ae/source0530 gebonden. Alle18CI-workflows op e318 slagen. Echte
CI-merge5754763ffb257d6420f74126cae7252e36afab1e/source0386 doorloopt dezelfde
22fasen in1407,011s, native_gate/clipboard_gate/raw33,34hostorakels en17volledige
appbody-pixelvergelijkingen. Opgeslagen beelden/serial/disk/8ingebeddeELFs en
19normalepayloads/font/firmware zijn onafhankelijk teruggelezen. Geen masks.
Bewijs: verification/native-clipboard-20261102/README.md met raw.gz en hashes.

28client+40brokerhosttests/scalarbuilds; volledige UI/apphosttests/Clippy/fmt;
46gerichte appketens voor opt-in/default-uit, echte focuscancel en transportverlies
vóór én na serverACK zonder edit/eval/valseappACK.187Python-tooltests:184pass,
3expliciete skips.16gerichte runner/latencychecks groen, zes overlappen die187.
Geen menselijke audit. Fase-READY/GEN kan achterlopen op het juiste beeld:
claim appbodypixels+echte inputACK, geen causale laatste-SUR-generatiegarantie.
Tijdvelden omvatten host/QMP/observatie/capture, geen p95/whole-screen/scanout.
De latencysample-recorder is gebouwd, zijn volledige workload niet uitgevoerd;
TCG-schermopbouw blijft te traag. Geen performance- of veiligheidsvergelijking.

**Windowspakket:** normale CI-bundel zonder crashcontroller:
`/workspace/nieuw-os-windows-package/build/clipboard-e318/NieuwOS-klembord-e318-Windows-QEMU.zip`,
SHA256 e29a6ca5933a0958b10238ae2f660237bbf5295c170d5c92146eb9d4ddf2fc3d.
Alle19payloads/8ELFs/7fontinputs/firmware en fakeCheckOnly zonderstate/VM groen;
20gerichtePython/22launcher/33helperchecks eenmaal groen. Een apart lokaal
gebouwde ZIP heeft eigen hashes en wordt niet met CI vereenzelvigd. Nieuwe
Windows/WHPX/SDL-runtime nog niet uitgevoerd in deze chat; lokale testchat heeft
de exacte e318-bron en --clipboard opdracht gekregen. Geen hostinstellingen of
fysieke schijven/doorgifte gewijzigd. Beide Linux-QEMU-proeven eindigden netjes.

**Volgende stap:** U02-snelheid: meet invoer/focus/appwissel/plakken volgens het
vooraf vastgelegde plan, onderzoek volledige1792tile-Materiaalhertekeningen en
verminder onnodig werk/IPC zonder caller-/epoch-/bounds-/COMMIT-/deadlinechecks
te schrappen. Complete-frame-100/250ms-doelen op de vaste WHPX-referentie blijven
open; ruimere clipboardstreaming volgt apart. Eerstvolgende broninspectie:
`rg -n 'paint_step|paint_tile|invalidate' sdk/examples/materiaal/src/lib.rs sdk/examples/calculator/src/app.rs ui/native/src/controller.rs`.
Contracten: implementation/NATIVE-CLIPBOARD.md en GUI-IDLE-WAIT.md;
meethelper tools/window_latency.py. Auth/sessiebroker/keyring-unlock/FIDO2,
contextmenu/IME/toegankelijkheid en volledige releasecriteria blijven behouden.

## Vorig geïntegreerd checkpoint — desktop en GUI-idle

Branch `codex/mature-native-session-20261102`, concept-PR122 op PR121. PR123 is
na centrale review en **18/18 groene CI-workflows** geïntegreerd als
`8289d60cd5caa8a26b7823a694858a15371dc2af`. Geen merge naar main;
de bestaande integrator behoudt die verantwoordelijkheid. De bewijsupdate bevat alleen documentatie en rapporten; geen
productcodewijziging na de geteste freeze.

**Roadmap: 41/100**, tegenover 34 vóór deze opdracht: N01 +3 en U01 +4.
De CPU-optimalisatie krijgt geen extra punten. U02, echte authenticatie,
klembord, volledige interactiebudgetten en de volledige testrelease blijven open.

**Eigen native desktop:** 1280×720, proportionele antialiased letters, eigen
pictogrammen, twee geïsoleerde appvensters, focus, slepen, dock en aanwijzer.
Materiaal en Rekenmachine behouden hun proces en inhoud bij een echte
compositorcrash; nieuwe presentatie-/invoerprocessen krijgen verse epochs.
Alle dertien fasen inclusief herstel/save/list/reopen slagen op 337 in root-TCG
(753,648s), CI (853,340s) en lokale Windows/WHPX (36,946s).
Raw exit 33, exacte appbeelden,14 bytes/revisie 1, onafhankelijke diskoracle, canaries en
volledige cleanup. U01-bewijs: verification/mature-session-20261102/U01-ACCEPTANCE.md.
Rode 2bb/b1-resultaten blijven rood; geen release- of authenticatieclaim.
Screenshot: build/window-final/run/window-b42797a76f084c62a04223c8dbd9de1b/reopened.png.

**Idle:** vier GUI-lussen parkeren via bestaande IPC_WAIT uitsluitend als er
geen onvoltooid werk is; kernel, ABI, rechten en transactiedeadlines blijven
ongewijzigd. Zes formele Windows/WHPX-routes, drie per arm, hetzelfde volledige beeld
vóór/na 15 s zonder QMP/invoer/pause tijdens ieder interval. ProcesCPU mediaan
99,3724%→11,6663% van één logische CPU: 88,2600% lager. Gemapte vCPU-thread
98,9558%→11,3538%. Root heeft ruwe 100ns-tellers en monotone meetintervallen
onafhankelijk herberekend. Primaire en twee gerichte AI-bronreviews vonden
geen blocker. Geen energie-, geen-redraw-, harde 20ms- of algemene latencyclaim.

**Bronbinding:** formele A 337/source39194035…f1995 tegenover
B 1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b/source-v3
8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a.
Head 2a7b74426027093a4793d78d5ddca94e15ae018a voegt alleen bewijs,
CI-fetchvolgorde en een Python-F12-testfix toe. Volledige huidige source-v3:
084478cf4d2e137a3ba37bc6029716be3e38fecfdf76ca82fe55cd409b8536d0.
Loader/kernel/acht normale app-ELFs en alle19 pakketbytes zijn bytegelijk aan
de productbron 8faf; alleen pakketmanifest verschilt. Oude metingen worden
niet naar 2a omgelabeld. Alle 18 CI-workflows op2a slagen, inclusief de echte
TCG-venstergate 41495583952 en gezamenlijke native systeemgate 41495583552.
Bron-/artifact-/hashbewijs: verification/gui-idle-20261102/INTEGRATION.md.
181 Python-tooltests, gerichte Rust-hosttests/Clippy/fmt,22 launcher- en 33 helperchecks
slagen. De CI-fetchfix behoudt offline/locked; cachehit is geen cold-cachebewijs.

**Windowspakket:** normale native-window-session zonder crashcontroller:
`/workspace/nieuw-os-windows-package/build/window-package-2a7b744/NieuwOS-venstersessie-2a7b744-Windows-QEMU.zip`,
SHA256 b685eb421e279da680da2c84bd280cc1edbc8684471f2019b2879c129c3ba1e0.
19 payloads/8 ELFs/alle fontnotices teruggelezen, synthetische CheckOnly zonder state.
Het oude 337-CI-pakket startte lokaal werkelijk via PS5.1/Whpx/SDL en sloot met 0;
visuele/inputacceptatie van die normale bundel bleef open door capturefout
0x800b0101. De volledige headless probes hebben hun eigen geslaagde beeld-
en invoerbewijs. Geen hostinstellingen, fysieke disks of passthrough gewijzigd.

**Eerstvolgende implementatiestap:** U02 aanvullen met expliciet begrensd,
gebruikersgestuurd klembord tussen de afzonderlijke apps, intrekbare rechten,
negatieve cross-app-proeven en gemeten invoer-/focus-/appwisselbudgetten.
Bestaande documenten: implementation/NATIVE-WINDOW-SESSION.md,
implementation/GUI-IDLE-WAIT.md en implementation/SESSION-AND-DESKTOP.md.
Echte account-/sessiebroker, sleutelvrijgave, vergrendeling en FIDO2 blijven
afzonderlijk noodzakelijk; gastaanmelding is geen beveiligde gebruikersgrens.
Geen nieuwe roadmap of scopeverlaging; alle volledige releasecriteria behouden.

## Vorige afgeronde checkpoint — native sessiepreview


**Functionele implementatie / Windows-pakket:** `344dc0afde893784503217ac676e154b13662507`,
branch `codex/session-desktop-vm-20261102`, concept-PR #121. Bronbasis
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De latere opmaakcommit `cfed2db9a39396e48a7bc758eb2c533f3af3f22c`
wikkelt uitsluitend één kernelarray-element om volgens rustfmt. De volledig
lokaal geteste344-bron heeft source-v3
`05b047819f1c1cddce6868034ab2208d8a35a95d9f974860afb8bbc53eeb8658`;
de opgemaakte bron heeft
`0bfd8385bbe08d3ace384977848fb42f6a65f6261b28be19805eccd90fce14f0`
en een eigen geslaagde achtstappen-cloud-QEMU-herhaling. Geen verdere
functionele wijzigingen. Beide rootfmt- en diffchecks slagen.
De bestaande andere integrator blijft eigenaar van merges; deze PR is niet gemerged.

**Geleverd:** opt-in `native-system-session-preview` met gebruikerskeuze,
herkenbaar aanmeldformulier, expliciete openbare VM-gast, bureaublad/taakbalk/
programmasearch en echte Materiaal. Eigen no_std-toestanden binden authantwoorden
maar verlenen zelf geen rechten. `UnavailableBroker` weigert alle persoonlijke
credentials; geen gedeeld demowachtwoord of nepbiometrie. Owned secrets worden
met de reeds gepinde zeroize gewist. Geen nieuwe externe crate of capability.

Windows Auto/Whpx/Tcg gebruikt een echte sequentiële QMP-initprobe met één
10 s-deadline en binaire LF-invoer. De lokale chat reproduceerde en valideerde
de Windows-HMP-racefix. Source-v3 sorteert expliciet case-sensitive padcomponenten,
waardoor gelijke Windows/Linuxbronnen dezelfde hash leveren; controles blijven intact.
De compositor verwerkt max8 berichten per beurt; de interactieve app max4
tegeltransacties per input/savepass. Geen raw framebuffer of gedeelde userpointer.

**Eindbewijs cloud:** alle 8 native stappen op `344dc0a` slagen: accounts →
persoonlijke synthetische testinvoer → geweigerd/gewist → accountkeuze →
gastlogin → desktop → zoek 'mat' → Materiaal. Iedere app-pixel wordt vergeleken,
zonder interne masks; QEMU raw exit0. TCG/QEMU 10.0.13/q35-10.0/1CPU/256 MiB:
118,79s totaal; Materiaal16,08s versus21,63s op `51d883e` en33,94s op `7648a8f`.
Dit zijn losse metingen in deze cloudomgeving, geen productlatentiegarantie.
Ook de bestaande native Materiaal normal/save/list/reopen-regressie slaagt
met onafhankelijke diskoracle en cleanup (rawexit33, runnerexit0;163,82s).
46 frontendhosttests,27 SDK-apphosttests,15 compositorhosttests,72 Pythonchecks,
39 synthetische PowerShellchecks, Clippy/fmt en scalar-none/objectscan slagen.
De oude verifier telde9 archives terwijl de bestaande SDKzeroize10 oplevert;
hersteld naar de exacte tien namen, met behoud van alle objects-/broncontroles.

**Lokale hardware:** geautoriseerde chat `01a19c13-c83d-7781-8f61-de0541ac6d95`
rapporteert Windows 11 25H2, Ryzen 9 9950X3D,64 GiB. De bytegelijke QMPhelper
slaagt 6/6 op PS 5.1/PS 7.6. De 51d-nativebaseline slaagt 6/6 (3WHPX/3TCG).
De eindmeting op `344dc0a` slaagt 6/6 (3 WHPX/3 TCG), 48 appbeelden exact.
WHPX gebruikerskeuze 6,625 s gemiddeld (6,481–6,823), gast→desktop 4,070 s
(3,932–4,231). De echte PS 5.1-starter kiest Auto→WHPX, toont SDL en sluit
met launcherexit 0. Er blijft geen QEMU-proces achter. Ruwe Windowsrapporten
staan in de lokale testmap; aangeleverde hashes en grenzen zijn vastgelegd
in local-pc-reported.json. Geen hostinstellingen gewijzigd.
Geen netwerk, gedeelde hostmappen, fysieke disks of USB/PCI/GPU-doorvoer.

**Grenzen en hervatten:** de gast gebruikt hetzelfde synthetische appdomein met
vooraf verleende VM-rechten. Het accountformulier is geen beveiligde sessiegrens.
Echte profiel-/verifierpersistentie, auth-/sessiebroker, keyring-unlock, limitering,
compositor/input-lock en revocation blijven nodig; FIDO2/vingerafdruk zijn niet
gekoppeld. Ook dirty-regiontransport, vensterbeheer, algemene appstart en
OS-brede toegankelijkheid blijven open. Score **34 → 34**, geen FR-/fysieke/
menselijke aftekening. [Contract](implementation/SESSION-AND-DESKTOP.md),
[hardware](implementation/VM-HARDWARE-AND-PERFORMANCE.md),
[eindbewijs/reproductie](verification/session-desktop-20261102/README.md).

GitHub Actions op cfed2db meldt startup_failure zonder gestarte jobs; de
oorzaak is niet vastgesteld. Dit is geen groene remote-CI en de PR blijft concept.

Eerstvolgend: onderzoek de CI-startfout en bouw daarna een echte aparte
auth-/sessiedienst vóór beveiligd aanmelden.
Gebruik bestaande tools en behoud andere dirty checkouts/branches.

## Vorig checkpoint — PR #125-inhoudsbehoud — 2 november 2026

**Integratormandaat:** Simon heeft op 2 november expliciet het exclusieve
integratormandaat van sessie `7f895683` aan deze sessie (`f090b913`)
overgedragen en de #85/#76-mergepauze opgeheven, met behoud van review-, test-
en branchregels. De oude integrator bevestigde geen lopende/verzonden
transacties, actief WIP of conflictclaim en stopt met operationele integratie.
De wijziging staat ook in [WORK-QUEUE.md](WORK-QUEUE.md). Dit is de actuele
autorisatie; het bewijst niet achteraf dat de desktop→cloud-overdracht vóór
integratie 3 expliciet was.

**Bron:** PR #125, branch `simonvanieperen-studious-waddle`, basis
`c66b593dda8d651e3cc73f0fe65bbe72ff7c7de7`; actuele `main`:
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De inhoudscontrole toonde dat
PR #120 slechts tot `562be4d` was opgenomen. De latere followups tot
`fb1e1a5c25429a0495e861f01602d4c0691f4f03` worden nu met behoud van de
#125-reviewfixes samengevoegd. Opname en bestanden staan in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).

**Reviewfixes gecommit en gepusht in `4b044f5`:**
- `tools/test_nas.py` bewaart alleen vaste NAS-foutlabels en exact 20-cijferige
  diagnostics, naast bestaande allowlist-markers.
- `tests/test_nas_runner.py` test veilige foutdiagnostiek, te lange waarden en
  tokenbytes; `services/nas/native-probe/CONTRACT.md` beschrijft de precieze
  kernel- en runnercontroles.
- `docs/WORK-QUEUE.md` maakt expliciet dat historische afwezigheid van
  desktopactiviteit geen overdrachtsbewijs is en noteert alleen de nieuwe,
  rechtstreeks door Simon bevestigde overdracht van 2 november.

**Vervolgpunten uit #120, nu mede opgenomen:**
- #116: `APP_HANDLE_COUNT` dekt nu de volledige app-handletabel op het
  manifestmoment. Historisch native bewijs en een negatieve proef staan
  in het integratierapport; de huidige combinatie vereist nieuwe CI.
- #119: de lekcontrole draait op de ruwe QEMU-bytes vóórdat er iets wordt
  weggeschreven: elk 16-bytevenster per token, ruw en als hex. Een misvormde
  capture wordt in zijn geheel doorzocht; bij een lek komen er alleen
  placeholders. Lost ook de Codex-P1 en de CodeRabbit-securitybevinding op
  #120 op. De huidige #125-allowlist voor serialdiagnostiek blijft behouden.
- `.github/workflows/materiaal-probe.yml`: voorlopig alleen
  `workflow_dispatch`. Een tweede lokale run met de CI-commandoreeks liep na
  208 s vast op de gastwatchdog (~200 s), met dezelfde tegeltellingen als de
  geslaagde run. De timingmarge blijft open; geen watchdog/grens verruimd.

**Lokale verificatie:** `python3 -B -m unittest discover -s tests -p
'test_nas_*.py' -v` — exit 0, 39 tests (35 runner); volume-runner: 3 tests;
private replay/oracle: 17 tests; Materiaal-probeharnas: 23 tests, alle exit 0.
Logs: `/tmp/nieuw-os-pr125-preservation-{nas-all,volume,replay,materiaal}.log`.
Rustfmt root/Materiaal en capability-core-Clippy slagen (exit 0).
De gerichte capability-tests starten niet: exit 101, hostlinker `cc`
ontbreekt; log `/tmp/nieuw-os-pr125-preservation-capability.log`.
Er zijn geen systeempackages geinstalleerd. Dit is geen native acceptatie.

**CI vóór deze samenvoeging:** op `c66b593` slaagde onder meer de native
NAS normal/cold-gate, run `41524587617`. De artifact noemt checkout
`ef0796b5263958107d17e1a52b9fea83768e1797` (GitHub-testmerge). Ouders zijn
exact de hierboven genoemde main/head; boom `1174b01d9fbc0c245f751c5507acd7502696f25d`
is gelijk aan `c66b593`. De bronbinding is via de API bevestigd.
Deze run bewijst de nieuwe samenvoeging niet.

**Open issues:** #109/#110 blijven open tot aantoonbare main-integratie.
#39–#43 missen volledige native, fysieke, sleutel- of routeacceptatie.
Kernelbuilds moeten buiten een geneste checkout draaien om dubbele
`.cargo/config.toml`-linkerargumenten te voorkomen.

**Parallel:** de twee #126-reviewfixes van sessie `c44e6fed` zijn ontvangen
als `5dead467d491107f7244e00450435c41072d6dc7`, inhoudelijk bekeken en door
de integrator fast-forward naar de bestaande #126-branch gepusht; remote-SHA
bevestigd. Rust-hosttests en native CI blijven vereist; nog geen merge.
Die sessie werkt nu uitsluitend aan vier #127-UI-reviewfixes op een aparte
branch (palet, proportionele klik, hoverdamage, semantisch wachtwoord).
Agent `87ef37fe` herstelt alleen
de twee TCP-reviewfouten uit #127 in een afzonderlijke worktree. Geen claims
op centrale status/score of elkaars bestanden.

**Voortgang:** 34/100 → 34/100; geen rubriccriterium is nieuw geaccepteerd.

**Eerstvolgend:** valideer en push deze inhoudsbehoudende samenvoeging, volg
de nieuwe exacte-head CI en review. Integreer pas
wanneer alle vereiste gates groen zijn en de delta/overlap met actuele `main`
is gecontroleerd; sluit vervangen PR's/issues pas na aantoonbaar inhoudsbehoud.

## Vorige integratiehandoffs

Integratie 3 (#115–#119) is vastgelegd in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).
Integraties 1 en 2 staan in
[pr-integration-20261030.json](verification/pr-integration-20261030.json) en
[pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).
Oudere handoffs staan in het [archief van 30 oktober](history/integration-2026-10-30/README.md).
