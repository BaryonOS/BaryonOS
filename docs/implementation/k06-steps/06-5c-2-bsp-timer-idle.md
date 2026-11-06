# L0-specificatie — K06 stap 5c-2: BSP-timer, idle met `sti; hlt` en het spinvenster op de BSP

**Datum:** 6 november 2026 · **Basis:** de head van stap 05 (F19/F15) [nog niet gebouwd]
**Branch/worktree:** `feat/k06-5c2-bsp-timer` in `/home/simon/projects/nieuw-os-k06-5c2`
**Spec:** `K06-SMP.md` §3.1 stap 3–6, §3.3, §4.3, §6.5, §9.6; ADR-045 (Sys3), -053, -073, -074;
auditbevindingen F6, F9, F11 en F13.

## 0. Vastgestelde feiten (op `7ffdacfa`)
- De PIT-IRQ0 loopt via de 8259 PIC (`m3.rs:1904-1908` programmeert de PIC). [te verifiëren: of de
  PIC in `native-smp` geïnitialiseerd wordt, of IRQ0 daar gemaskeerd is, en of de PIC naar een AP
  kan leveren. In virtual-wire-modus levert hij alleen aan de BSP-LINT0.]
- AP-timer: vector 0xd0, `TIMER_INITIAL = 20000`, deler 16, ongekalibreerd (F9).
- 0xe8 springt naar het timerpad en telt daardoor ook als tick (F11).

## 1. Probleem (waarom nu)
De BSP heeft geen preëmptie, geen idle met interrupts en geen spinvenster. Daardoor:
- kan de BSP geen ring-3-werk eerlijk delen (stap 08);
- krijgt de BSP 0xe0 niet binnen als hij op `KERNEL` spint terwijl een AP een ronde start, of bij
  een wijziging in de kernelhelft. Het gevolg is een **valse quarantaine van alle AP's** (F6);
- gaat een uitgestelde 0xe8 verloren als de timer gemaskeerd is (F11).

## 2. Invarianten (normatief)
- **BT-1 Eén klok.** De PIT-IRQ0 blijft de enige bron van de monotone klok en levert alleen aan de BSP.
  De LAPIC-timer levert alleen preëmptieticks. *Weerlegd als* Sys3 sneller of trager loopt dan de PIT,
  of als een AP een IRQ0 ontvangt.
- **BT-2 BSP gelijk aan een AP.** De BSP krijgt vector 0xd0 met dezelfde deler als de AP's, plus het
  INV-5c-uitstel en het spinvenster (ADR-074 S1/S3). S3 is pas toegestaan als **elke** ongemaskeerde
  bron op de BSP een geïnstalleerde IDT-gate heeft (IRQ0, 0xd0, 0xe0, 0xe8, 0xff).
- **BT-3 Idle.** `sti; hlt; cli` alleen in de idle-thread op de kernel-CR3, met lege lockdep.
- **BT-4 Reschedule-vlag in plaats van tick.** 0xe8 krijgt een eigen teller en zet `need_resched`. Een
  0xe8 die door INV-5c wordt uitgesteld, blijft als `need_resched` staan. De release van de laatste
  lock (lockdep wordt leeg) controleert de vlag en wisselt dan bij de eerstvolgende veilige plek
  (terugkeer naar user mode of idle). Ticks en resched worden apart geteld (F11).
- **BT-5 Gekalibreerde band.** Bij de boot telt de BSP LAPIC-ticks per PIT-tick over 100 ms en logt
  `SMP:TIMER:RATIO:<r>`. De gate eist een band [min, max] die vooraf in de ADR staat en niet achteraf
  wordt aangepast.
- **BT-6 Contract kernelthreads** (F13): leg in de ADR vast dat fixture-kernelthreads met IF=1 draaien
  als "pseudo-user". Echte kernelcode blijft IF=0 (spec §4.3).

## 3. Interface
- `smp.rs`: `bsp_timer_init()` na de M2-init en vóór het vrijgeven van `SMP_SCHED_GO`;
  `CpuLocal.need_resched: AtomicBool`, `resched_count` en `tick_count` apart.
- `lock.rs`: het venster-predicaat wordt `level <= ASPACE && irq_depth == 0 && this_cpu().irq_ready`
  (in plaats van `on_ap()`); `irq_ready` wordt per CPU gezet zodra BT-2 waar is.
- Idle-thread per CPU (uit 04), nu ook op de BSP.

## 4. Native fixture en negatieve tests
- **Klok:** tijdens de bestaande SMP-belasting blijft Sys3 monotoon en stijgt hij binnen 1% van de
  PIT-verwachting. Marker `SMP:CLOCK:PASS`.
- **BSP-preëmptie:** twee kernelthreads op de BSP met registercanaries, zoals stap 4 op de AP's.
  Marker `SMP:BSP_PREEMPT:PASS`.
- **BSP als shootdowndoel:** een AP houdt `KERNEL` vast en start een ronde met alleen de BSP als doel;
  de BSP spint op `KERNEL`. Eis: ack binnen 100 ms, geen quarantaine. Marker `SMP:BSP_SPIN:PASS`.
- **Idle-wek:** de BSP is idle; een AP wekt hem met 0xe8; de wake-latentie wordt gelogd. Marker
  `SMP:IDLE:PASS`.
- **Uitgestelde resched:** 0xe8 tijdens een lockwacht met gemaskeerde timer → de thread wisselt na
  de release. Marker opgevouwen in `SMP:IDLE:PASS`.
- `test-smp-bsp-spin-irq-off` (controleprofiel): het BSP-venster staat uit → de ronde naar de BSP moet
  een timeout geven (`SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0`).
- Regressie: `native-rtc` 5/5 (de CMOS-eigendom verandert niet) en de monotone-klokcheck.

## 5. Schrijfbestanden
`kernel/src/m2/smp.rs`, `kernel/src/m2/lock.rs`, `kernel/src/m2/arch.rs` (IDT-gates op de BSP, als die
daar staan), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (nieuwe ADR, of een aanvulling op ADR-073:
BSP-timer, kalibratieband, need_resched), `docs/implementation/K06-SMP.md` (§3.1 en §6.5),
`docs/PROJECT-STATE.md`, voortgangsbestanden.

## 6. Acceptatie
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`.
- Native: `native-smp` (alle gevallen, 1/2/4/8 vCPU's), `test-smp-bsp-spin-irq-off`, `native-rtc` 5/5,
  `milestone-m2`, en de K05-regressie op 1 CPU.
- K06 blijft open (41/100).

## 7. Buiten scope
TSC-deadline, tickless idle, IOAPIC-routering van device-IRQ's naar AP's, energiebeheer (H05).
