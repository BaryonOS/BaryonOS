# L0-specificatie — K06 F19/F15: ICR-lock en quarantainebeleid bij gehouden locks

**Datum:** 6 november 2026 · **Basis:** de head van stap 04 (F16/F17) [nog niet gebouwd]
**Branch/worktree:** `fix/k06-f19-f15-icr-quarantine` in `/home/simon/projects/nieuw-os-k06-f19`
**Spec:** `K06-SMP.md` §2 (INV-5), §3.3, §4.2 (niveau 60 ICR), §6.2a, §7.1 stap 6; ADR-071, -072;
auditbevindingen F19 en F15.

## 0. Vastgestelde feiten (op `7ffdacfa`)
- `kernel/src/m2/smp.rs:555` `send_ipi(apic_id, low)` schrijft ICR-hoog (0x310) en ICR-laag (0x300)
  zonder lock. Het gaat ervan uit dat alleen de BSP verstuurt.
- Quarantaine (`quarantine_all`) stuurt INIT naar alle AP's en draait daarna verder op één CPU.

## 1. Probleem (waarom nu)
- **F19:** vanaf 04 en 08 sturen AP's 0xe8 en 0xe0, zowel vanuit threadcontext als vanuit een
  handler. Een interrupt tussen het schrijven van ICR-hoog en ICR-laag geeft een verkeerde
  bestemming. Het gevolg is een verloren wake of ack, en daarmee een valse quarantaine of een
  kill-timeout.
- **F15:** quarantaine is nu alleen veilig omdat AP's niets echts vasthouden. Zodra ze werk doen, kan
  een dove AP `KERNEL`, `ASPACE` of `RUNQ` vasthouden. Na INIT blijft die lock voor altijd vast, en
  de BSP blokkeert bij de volgende acquire. De context van de thread die op die AP liep, is weg, dus
  "alle threads naar de BSP" (spec §6.2a) kan voor die thread niet.

## 2. Invarianten (normatief)
- **IQ-1 ICR atomair per CPU.** Het paar hoog/laag plus de wacht op delivery-status gebeurt binnen
  `ICR_LOCK[this_cpu]` (niveau 60, irqsave). *Weerlegd als* lockdep een `send_ipi` zonder die lock ziet.
- **IQ-2 Elke lock kent zijn eigenaar.** In `native-smp`- en testbuilds schrijft `IrqLock` bij een
  acquire de CPU-index van de eigenaar, en wist die bij de release (één `AtomicU8`; geen verandering
  in de ticketlogica).
- **IQ-3 Beleid bij een dode CPU.** Alleen de BSP voert quarantaine uit. Vóór INIT controleert de BSP
  of de dove CPU een lock vasthoudt (de eigenaarsvelden van `KERNEL`, alle `ASPACE`, `FRAMES`,
  `RUNQ`, `SHOOTDOWN`):
  - houdt hij **niets** vast → INIT, de READY-threads uit zijn queue naar de BSP, en de thread die
    RUNNING was geldt als gecrasht (het proces wordt beëindigd via de K05-faultroute, geen migratie);
  - houdt hij **wel** iets vast → `SMP:QUARANTINE:LOCK_HELD:<niveau>:CPU=<n>` en een
    **systeemstop** (fail-closed). Een lock "losbreken" mag niet: de toestand eronder is onbekend.
- **IQ-4 Een AP initieert nooit quarantaine.** Een AP die een deadline overschrijdt, meldt dat aan de
  BSP (een vlag plus 0xe8) en wacht. [te verifiëren: of een AP in 08 zelf een shootdown kan
  starten; zo ja, dan moet de BSP de ronde overnemen.]

## 3. Interface
- `smp.rs`: `ICR_LOCK` als veld in `CpuLocal`; `send_ipi` neemt die lock.
- `lock.rs`: `IrqLock::owner() -> Option<u8>` (debug- en SMP-builds).
- `smp.rs`: `quarantine_cpu(slot)` met het IQ-3-beleid; `quarantine_all` wordt een lus daarover.
- Kernelfeatures `test-smp-quarantine-lock-held` en `test-smp-icr-race`.

## 4. Native fixture en negatieve tests
- Positief: AP1 en AP2 sturen elkaar elk 10.000 0xe8-IPI's, terwijl hun eigen timer (0xd0) ook
  IPI's verstuurt vanuit de handler. Eis: ontvangen = verzonden per paar, geen timeout. Marker
  `SMP:ICR:PASS`.
- `test-smp-icr-race` (controleprofiel): zonder `ICR_LOCK`, met een opzettelijke 0xd0 tussen hoog
  en laag. De oracle moet een verkeerd geadresseerde of verloren IPI zien
  (`SMP:ICR:MISROUTED` of `SMP:ICR:LOST`).
- `test-smp-quarantine-lock-held` (4 vCPU's): AP2 neemt `KERNEL` en wordt doof. Verwacht:
  `SMP:QUARANTINE:LOCK_HELD:10:CPU=2` en een stop met exit 0x10, geen hang.
- Bestaande dove-AP-profielen (`ap-ipi-deaf`, `ap-shootdown-deaf`, `ap-timer-dead`) blijven het oude
  pad volgen, omdat die AP niets vasthoudt.

## 5. Schrijfbestanden
`kernel/src/m2/smp.rs`, `kernel/src/m2/lock.rs`, `kernel/Cargo.toml`, `tools/build.py`,
`tools/run_smp.py`, `tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`,
`docs/DECISIONS.md` (nieuwe ADR: quarantainebeleid; aanvulling ADR-071/072), `docs/PROJECT-STATE.md`,
voortgangsbestanden.

## 6. Acceptatie
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`.
- Native: `native-smp` (alle gevallen), de twee nieuwe negatieve profielen en alle bestaande
  dove-AP-profielen ongewijzigd.
- K06 blijft open (41/100).

## 7. Buiten scope
Herstel van een lock na een dode eigenaar, x2APIC (ICR in één MSR-schrijfactie), broadcast-IPI's.
