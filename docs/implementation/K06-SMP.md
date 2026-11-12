# L0-specificatie K06 — Meerdere processorkernen (SMP)

**Status:** voorstel (ontwerp), niet geïmplementeerd, niet getest.
**Datum:** 6 november 2026 (Europe/Amsterdam).
**Basis:** branch `docs/niet-wachten-op-ci`, HEAD `0d968f3d` (main met #204 = K06 stap 4).
**Gelezen bronnen:** `docs/progress.json` (K06), `docs/PROJECT-STATE.md` (handoff stap 4),
`docs/DECISIONS.md` ADR-024, -045, -053, -060, -070 t/m -073,
`docs/verification/k06-smp-preempt/README.md`, `kernel/AGENTS.md`, kop van `kernel/src/m2/smp.rs`.
**Niet gelezen, dus gemarkeerd als [te verifiëren]:** de M3-scheduler, het IRQ0-pad
(8259 PIC of IOAPIC), de FXSAVE-implementatie, de IPC-wachtrijen, `tools/run_smp.py`.

K06-acceptatie (progress.json): *"SMP-scheduling, synchronisatie, interrupts en
adresruimtewisseling werken onder concurrente belasting op het ondersteunde profiel."*
Bewijsniveau: native QEMU q35/OVMF/TCG; fysieke hardware valt buiten K06.

---

## 0. Uitgangspunt: al gerealiseerd (stap 1–4)

| Stap | ADR | Wat staat er (alleen `native-smp`) |
|---|---|---|
| 1 | 070 | MADT als onbetrouwbare invoer; INIT-SIPI-SIPI per AP (unicast); CAS STARTING→CLAIMED/ABANDONED; AP-stacks met guards in het kernelimage; `MAX_CPUS = 8`; xAPIC-MMIO alleen via IA32_APIC_BASE; AP neemt CR0/CR3/CR4/EFER/PAT exact over. |
| 2 | 071 | Per AP een eigen GDT/TSS (RSP0, IST1 #DF, IST2 NMI), GS-anker; IPI-vector 0xf0, spurious 0xff; alleen unicast; een dove AP → INIT voor alle AP's. |
| 3 | 072 | TLB-shootdown (vector 0xe0) met generatie-ack vóór het vrijgeven van een frame; een ontbrekende ack → frame in quarantaine en INIT voor alle AP's. Eén bootstrapvenster. |
| 4 | 073 | LAPIC-timer (vector 0xd0) per AP; twee gepreëmpteerde kernelthreads per AP; GPR-context; bevriezen na STOP. |

**Wat ontbreekt** (handoff + ADR-073 "Open"): BSP-interrupts/-preëmptie, een SMP-veilige
scheduler met runqueues per CPU, een lockvolgorde voor kernelsubsystemen, gebruikersprocessen
op AP's, een CR3-wissel per thread, shootdown voor procesadresruimten en de herziening van ADR-024.

---

## 1. Scope

**Binnen K06:** stap 5a–5f hieronder, in het opt-in profiel `native-smp`, opgewaardeerd van
een bringupprofiel naar een profiel dat M3-processen draait.

**Buiten K06 (expliciet open):** x2APIC, IOAPIC-routering van device-IRQ's naar AP's,
PCID/INVPCID, TSC-deadline, NUMA, CPU-hotplug, fysieke hardware, IOMMU/DMA-containment, energiebeheer.
Andere profielen (`milestone-m2`, M3–M6, GUI) blijven één CPU en ongewijzigd.

**Kernelregel (`kernel/AGENTS.md`):** "Geen SMP uitbreiden vóór single-core-invarianten
aantoonbaar zijn." Die zijn geaccepteerd (K02–K05, K07). Elke stap hieronder moet de
één-CPU-regressie ongewijzigd laten slagen.

---

## 2. Beveiligingsinvarianten (eerst, normatief)

Elke invariant heeft een weerlegging; de weerlegging wordt een negatieve test (§9).

- **INV-1 Exclusieve frame-eigendom.** Een fysiek frame krijgt pas een nieuwe eigenaar
  nadat geen enkele CPU er nog een vertaling van kan gebruiken (ADR-072, uitgebreid
  naar alle adresruimten).
  *Weerlegd als* een CPU na `free` via een oude TLB-entry het frame leest of schrijft.
- **INV-2 Geen gedeelde veranderlijke kernelstaat zonder lock.** Engine, frame-allocator,
  paging, IPC-wachtrijen, procestabel en quota worden alleen onder de lock uit §4 gemuteerd.
  *Weerlegd als* lockdep (§4.4) een mutatie zonder de vereiste lock detecteert, of als
  de stress-run (§9) een audit-mismatch geeft.
- **INV-3 Eén CPU per thread.** Een thread draait op hoogstens één CPU tegelijk. Zijn
  kernelstack, FXSAVE-beeld en GPR-frame zijn alleen van die CPU zolang hij draait.
  *Weerlegd als* twee CPU's tegelijk `RUNNING` voor dezelfde TID publiceren.
- **INV-4 Adresruimte-isolatie blijft per proces.** Een CPU heeft alleen de CR3 geladen
  van het proces dat hij uitvoert, of de kernel-CR3 als hij idle is. De bitmap
  `active_cpus` van een proces is een superset van de CPU's die zijn vertalingen kunnen cachen.
  *Weerlegd als* een CPU vertalingen van proces P cachet terwijl zijn bit in `P.active_cpus` 0 is.
- **INV-5 Fail-closed bij een dove CPU.** Een CPU die een shootdown of reschedule-IPI niet
  binnen de deadline bevestigt, wordt nooit genegeerd: frames blijven in quarantaine en
  de CPU wordt gereset met INIT (ADR-071/072). Een mislukte INIT stopt de boot.
- **INV-6 Capability- en usercopy-grenzen blijven gelijk.** SMP voegt geen recht, syscall
  of ABI toe. Usercopy controleert de pointer opnieuw onder de lock van de doeladresruimte,
  zodat een gelijktijdige unmap door een thread van hetzelfde proces geen
  TOCTOU-venster opent.
- **INV-7 Geen lock over een contextwissel.** Bij een wissel houdt een CPU geen lock vast,
  behalve de runqueue-overdracht uit §6.3. Er is geen lock die een thread kan vasthouden
  terwijl hij geparkeerd is.

---

## 3. APIC- en SMP-initialisatie (precieze volgorde)

Stap 1–4 blijven ongewijzigd. Nieuw is alleen wat de BSP en een AP na de bestaande
bringup doen.

### 3.1 BSP

1. Bestaande M2-boot: IDT, paging, frames, BootInfo, MADT-inventaris (ADR-070).
2. Bestaande AP-bringup, IPI-zelftest, shootdownzelftest en preëmptiezelftest
   (stap 1–4), met IF=0. De zelftests blijven verplichte poorten: als er één faalt, worden
   alle AP's gequarantaineerd (INIT) en draait de boot verder als één CPU, zoals nu.
3. **Nieuw:** per-CPU-blok voor de BSP (CPU-index 0) op hetzelfde GS-anker-mechanisme
   als de AP's (ADR-071), zodat `this_cpu()` overal hetzelfde pad heeft.
4. **Nieuw:** LAPIC-timer op de BSP met dezelfde vector 0xd0 en dezelfde deler als de AP's.
   De PIT-IRQ0 blijft **alleen op de BSP** de bron van de monotone klok (ADR-045 Sys3,
   ADR-053). De LAPIC-timer levert alleen preëmptieticks en wordt niet als klok gebruikt.
   [te verifiëren: de huidige IRQ0-route (8259 of IOAPIC) en of die gemaskeerd naar AP's blijft]
5. Daarna pas de M3-init (procestabel, manifest, eerste processen) **onder de kernellock** (§4).
6. BSP zet IF=1 alleen in user mode en in de idle-lus (`sti; hlt; cli`, het bestaande
   ADR-053-patroon). Kernelcode draait verder met IF=0, behalve waar §4.3 dat toestaat.

### 3.2 AP

1. Stap 1–4 ongewijzigd t/m de preëmptiezelftest. De zelftestthreads stoppen.
2. **Nieuw:** de AP wacht in de idle-lus op `SMP_SCHED_GO` (één atomic, release door de BSP
   na stap 3.1.5). Pas daarna kiest hij werk uit zijn eigen runqueue (§6).
3. Op een AP blijft IF=0 in kernelcode en IF=1 in user mode en in de idle-lus.

### 3.3 Vectoren (vast)

| Vector | Doel | Ontvanger |
|---|---|---|
| 0x20 (of de bestaande IRQ0-vector) [te verifiëren] | PIT, monotone klok | alleen BSP |
| 0xd0 | LAPIC-timer, preëmptie | alle CPU's |
| 0xe0 | TLB-shootdown | doel-CPU's uit `active_cpus` of alle online |
| 0xe8 | **nieuw:** reschedule/kill-IPI | één doel-CPU |
| 0xf0 | test-IPI (bestaand) | alleen zelftest |
| 0xff | spurious | alle |

Alleen unicast-ICR, geen broadcast-shorthands (ADR-071). Prioriteitsklassen (vector >> 4):
shootdown 0xe staat boven timer 0xd, zodat een CPU met een openstaande timer een shootdown
toch accepteert. Elke ICR-schrijfactie gebeurt onder `ICR_LOCK` (§4.2) met een check op
delivery-status vóór de volgende.

---

## 4. Spinlocks en lockvolgorde

### 4.1 Lockprimitief

- **Ticketlock** (`next`/`owner` als `AtomicU32`, FIFO, geen starvation onder TCG).
  `lock`: `fetch_add(next, Relaxed)`, daarna `owner` lezen met `Acquire` en `pause` in de
  spin. `unlock`: `owner.fetch_add(1, Release)`.
- **IRQ-veilig:** `lock_irqsave` bewaart RFLAGS.IF, doet `cli` en neemt dan de lock;
  `unlock_irqrestore` herstelt IF pas na de release. Elke lock die ook vanuit een handler
  (0xd0/0xe0/0xe8) wordt genomen, mag **alleen** via `irqsave`.
- **Begrensde spin in debugbuilds:** na `2^24` iteraties volgt een terminale
  `SMP:LOCK:STUCK:<naam>:CPU=<n>` en stopt de VM. Release-builds spinnen onbegrensd, maar
  lockdep (§4.4) moet de volgorde al in de gate hebben bewezen.
- Een guardtype (RAII) zonder `Send`, zodat een guard niet over een contextwissel heen kan
  worden meegenomen (INV-7). Dat is een compile-time-grens; de runtimecontrole staat in §4.4.
- Geen `unsafe` buiten de `UnsafeCell`-toegang in de guard; elke SAFETY-notitie noemt
  aliasing (exclusief door de lock), alignment (repr) en levensduur (guard).

### 4.2 Locks en vaste volgorde (buiten → binnen)

Een CPU mag alleen een lock met een **hoger** niveau nemen dan de hoogste die hij al vasthoudt.

| Niveau | Lock | Beschermt | Vanuit handler? |
|---|---|---|---|
| 10 | `KERNEL` (grote kernellock) | capability-engine, procestabel, IPC-wachtrijen, quota, supervisor, M3-diensten, volume-/net-/keyringbrokers | nee |
| 20 | `ASPACE[pid]` | pagetables en mappingarena van één proces (ADR-024 2048 mappings) | nee |
| 30 | `FRAMES` | frame-allocator en eigenaarsmetadata | nee |
| 40 | `RUNQ[cpu]` | runqueue van één CPU; twee tegelijk alleen in oplopende CPU-index | ja (0xd0, 0xe8) |
| 50 | `SHOOTDOWN` | één lopende shootdownronde (adresbereik, doelset, generatie) | nee (handler leest alleen de gepubliceerde ronde) |
| 60 | `ICR_LOCK` | xAPIC-ICR-schrijfacties van de eigen CPU [per CPU, want de ICR is lokaal; alleen nodig tegen herintreding vanuit een handler] | ja |
| 70 | `SERIAL` | seriële poort | ja (alleen terminale paden) |

**Waarom een grote kernellock (niveau 10).** De bestaande subsystemen zijn gebouwd en
bewezen op het contract "één CPU, IF=0" (ADR-024, -053, -060). Fijnmazige locks per
subsysteem zouden al dat bewijs opnieuw openen. De grote kernellock behoudt dat contract
letterlijk: binnen `KERNEL` gedraagt de kernel zich als één CPU. Parallellisme komt in
deze stap uit user mode op meerdere CPU's, niet uit parallelle syscalls.
*shortcut: syscalls serialiseren op één lock; vervang dit per subsysteem door eigen locks zodra metingen contentie tonen (herzieningscriterium in ADR-074).*

**Gevolgen voor bestaande ADR's:**
- ADR-060 (vooraf gereserveerde buffer van 461.816 bytes met niet-herintredende lease):
  blijft geldig, omdat ze alleen onder `KERNEL` wordt gebruikt. De expliciete
  single-CPU/IF0-check wordt "houdt `KERNEL`".
- ADR-053 (IPC-wait): het parkeren en wekken van een wachtende gebeurt onder `KERNEL`;
  het wekken van een thread op een andere CPU stuurt een 0xe8-IPI (§6.4).
- ADR-045 (FXSAVE): zie §7.3.

### 4.3 Interruptstatus per context

| Context | IF | Locks toegestaan |
|---|---|---|
| user mode | 1 | geen |
| syscall-/exceptionpad in de kernel | 0 | alles volgens §4.2 |
| idle-lus | 1 tussen `sti; hlt`, anders 0 | geen tijdens `hlt` |
| handler 0xd0/0xe8 | 0 | alleen `RUNQ[eigen cpu]` (irqsave), `ICR_LOCK` |
| handler 0xe0 | 0 | geen lock: leest de gepubliceerde ronde met `Acquire`, doet `invlpg`/CR3-reload, ackt de generatie met `Release` |
| NMI/#DF (IST) | — | geen lock; alleen terminaal pad via `SERIAL` met `try_lock` |

**Deadlockvrijheid bij shootdown:** een CPU die op acks wacht, houdt `SHOOTDOWN` en
eventueel `ASPACE`/`FRAMES` vast, maar heeft IF=0. Doel-CPU's moeten de 0xe0-IPI dus kunnen
nemen terwijl zij zelf in de kernel spinnen op `KERNEL`. Daarom:
**spinnen op `KERNEL` en `ASPACE` gebeurt met IF=1** (de spinlus doet `sti; nop; pause; cli`),
en de 0xe0-handler neemt geen lock. Op die manier kan een CPU die op een lock wacht altijd
een shootdown afhandelen. De CPU die de lock vasthoudt, wacht nooit op een CPU die zelf
een handler-lock wil.

### 4.4 Lockdep (debug, verplicht in de gate)

Per CPU een `held_mask: u64` en `max_level` in het per-CPU-blok. `lock` controleert
`level > max_level`, anders `SMP:LOCKDEP:ORDER:<a>-><b>` en stop. Bij een contextwissel moet
`held_mask` leeg zijn, met de §6.3-uitzondering voor `RUNQ` die de wisselcode zelf overdraagt.
Anders `SMP:LOCKDEP:HELD_ACROSS_SWITCH`. Een negatief testprofiel neemt bewust de
omgekeerde volgorde en moet die marker geven (§9).

---

## 5. Herziening ADR-024 (voorgesteld als ADR-074)

ADR-024 eist zelf een herziening "bij SMP". Voorgestelde nieuwe tekst:

1. **Mappingarena:** ongewijzigd 2048 mappings per proces, statisch, buiten de kernelstack.
   Toegang alleen onder `ASPACE[pid]` (+ `KERNEL`).
2. **Scratchslot per CPU in plaats van één globaal slot.** CPU *i* gebruikt alleen VA
   `SCRATCH_BASE + i·PAGE` (supervisor, NX, buiten iedere user-range, in de gedeelde
   kernelhelft). Invalidatie vóór gebruik en na het wissen blijft lokaal (`invlpg`).
   **Voorwaarde:** geen enige CPU dereferenceert ooit het slot van een andere CPU. Een
   eventuele speculatief gecachete vreemde slot-entry wordt dan architecturaal nooit gebruikt.
   Die voorwaarde wordt afgedwongen doordat het adres alleen via `this_cpu().scratch_va()` bestaat.
   *Restrisico:* speculatieve side-channels op zo'n vertaling vallen buiten dit model
   (vastgelegd, niet opgelost).
3. **Executable-alias-weigering en RO/NX-helper:** ongewijzigd.
4. **Geen identity-map van vrij RAM/MMIO:** ongewijzigd.
5. **ASID:** blijft een interne teller; PCID blijft uit. Elke CR3-load flusht dus de
   niet-globale TLB-entries van die CPU.
6. **Kernelmappingwijziging:** een wijziging in de gedeelde kernelhelft (behalve de eigen
   scratchslot) vereist een shootdown naar **alle online CPU's** (ADR-072-protocol).
7. **Herzien bij:** PCID, fijnmazige subsysteemlocks, DMA/IOMMU of een nieuwe mapping-API.

---

## 6. Scheduler (SMP)

### 6.1 Datastructuren

- `PerCpu { index, apic_id, runq: RUNQ lock<FixedQueue<Tid, N>>, current: Option<Tid>,
  idle_tid, scratch_va, tss, lockdep, need_resched: AtomicBool, timer_ticks: AtomicU64 }`
  via het GS-anker.
- `Thread { tid, pid, state: AtomicU8 {READY, RUNNING(cpu), BLOCKED, DEAD}, home_cpu,
  kstack, fxsave: [u8; 512] align 16, gpr_frame }`.
  [te verifiëren: hoe de huidige M3-procesentry thread en proces combineert; dit wordt
  één thread per proces zolang M3 geen threads heeft]
- `Process.active_cpus: AtomicU64` (bit per CPU-index; `MAX_CPUS = 8` past ruim).
- Een runqueue heeft een vaste capaciteit (geen allocator), gelijk aan de bestaande
  procescapaciteit; overloop is onmogelijk omdat een thread maar in één queue staat.

### 6.2 Plaatsing en migratie

- Bij spawn kiest de kernel `home_cpu` = de online CPU met de kortste runqueue (bij gelijke
  lengte de laagste index), onder `KERNEL`.
- **Geen automatische load balancing in K06.** Migratie gebeurt alleen expliciet:
  (a) bij quarantaine van een CPU (alle threads naar de BSP), en (b) in een testfixture
  die migratie forceert om §7 te bewijzen.
  *shortcut: geen work-stealing; voeg het pas toe als R02-prestatiebudgetten het vragen.*

### 6.3 Contextwissel (exacte volgorde, op CPU *c*, IF=0)

1. Aanleiding: een 0xd0-tick zet `need_resched`, een syscall blokkeert, of een 0xe8-IPI
   arriveert. De wissel zelf gebeurt alleen bij terugkeer naar user mode of in de idle-lus,
   nooit midden in een `KERNEL`-sectie.
2. `held_mask` moet leeg zijn (lockdep).
3. FXSAVE van de huidige thread `T` naar `T.fxsave` (ADR-045: na de laatste Rust-code van de
   oude context, vóór de CR3-selectie).
4. `lock_irqsave(RUNQ[c])`; `T.state`: RUNNING(c) → READY (in de queue) of BLOCKED/DEAD
   (niet in de queue). Kies de volgende `U` (round-robin); `U.state` READY → RUNNING(c)
   met `compare_exchange` (INV-3; mislukt het, dan volgt `SMP:SCHED:DOUBLE_RUN` en stop).
5. **Adresruimtewissel:** als `U.pid != T.pid`:
   a. `U.process.active_cpus.fetch_or(1<<c, SeqCst)` **vóór** de CR3-load;
   b. `mov cr3, U.process.cr3`;
   c. `T.process.active_cpus.fetch_and(!(1<<c), SeqCst)` **ná** de CR3-load.
   Volgorde a→b→c houdt INV-4 waar: het bit staat al aan voordat de CPU de nieuwe vertalingen
   kan cachen, en gaat pas uit als de oude niet meer bereikbaar zijn (CR3-load zonder PCID
   flusht ze).
6. TSS.RSP0 van CPU *c* ← `U.kstack` top.
7. `unlock_irqrestore(RUNQ[c])` (IF blijft 0 tot IRETQ).
8. FXRSTOR uit `U.fxsave`, direct vóór de GPR-pops/IRETQ (ADR-045).
9. Idle: geen thread READY → CR3 ← kernel-CR3 (stap 5 met de kernel als "proces"), daarna
   `sti; hlt; cli` en opnieuw kijken.

### 6.4 Wekken en kill over CPU's heen

- Een thread op CPU *d* wekken vanaf CPU *c* (bijvoorbeeld IPC-antwoord, ADR-053):
  onder `KERNEL` → `lock_irqsave(RUNQ[d])` → BLOCKED→READY, enqueue → unlock → als *d* ≠ *c*,
  een 0xe8-IPI naar *d*. De IPI is een hint: *d* controleert zijn queue altijd zelf.
  Een readiness-wake (geen timeout/fout) vanaf *c* ≠ *d* komt vooraan in `RUNQ[d]`, eenmaal per
  BLOCKED→READY en zonder extra quantum (ADR-081-aanvulling R3-5, 11 november 2026).
- Kill van een thread die op *d* RUNNING is: markeer `DEAD_PENDING`, stuur 0xe8 en wacht
  begrensd (deadline gelijk aan de shootdown-deadline) tot *d* `state = DEAD` publiceert
  bij zijn eerstvolgende wissel. Pas daarna volgt de bestaande `cleanup` (K05) en de
  adresruimte-shootdown (§7.2). Geen ack binnen de deadline → INV-5.

### 6.5 Preëmptie

LAPIC-timer 0xd0 op elke CPU, periodiek, dezelfde deler als ADR-073. De handler doet alleen:
`timer_ticks += 1`; bij elk *k*-de tick `need_resched = true`; EOI. Kalibratie tegen de PIT is
**niet** nodig voor K06 (de klok komt uit de PIT), maar het aantal ticks per PIT-tick wordt
in de gate gelogd. [ongekalibreerd, zie ADR-073 "Open"]

---

## 7. Adresruimtewissel en TLB-shootdown per proces

### 7.1 Algemeen protocol (veralgemening van ADR-072)

Voor elke wijziging die een bestaande user-vertaling van proces P verwijdert of verzwakt
(unmap, rechtenverlaging, procesexit):

1. Neem `KERNEL` → `ASPACE[P]` → (indien vrijgeven) `FRAMES` → `SHOOTDOWN`.
2. Wis of wijzig de PTE('s); lokaal `invlpg` per pagina (≤ 32 pagina's), anders een
   CR3-reload als P lokaal actief is.
3. `fence(SeqCst)`; snapshot `targets = P.active_cpus.load(SeqCst) & !(1<<self)`.
   Door de volgorde in §6.3 (stap 5a vóór de CR3-load) ziet elke CPU die P daarna laadt
   al de nieuwe PTE; elke CPU die P eerder laadde, zit in `targets`.
4. Publiceer de ronde `{pid, range, generatie+1}` met `Release`; stuur unicast 0xe0 naar elke
   CPU in `targets`.
5. Handler op de doel-CPU: als zijn huidige CR3 die van P is → `invlpg` per pagina of een
   CR3-reload; anders niets (een CR3-load zonder PCID heeft al geflusht). Daarna altijd een
   ack met de generatie (`Release`).
6. Wacht (IF=0, `pause`) op alle acks tot de deadline (dezelfde als ADR-072).
   Ontbreekt er één → frames in quarantaine (eigenaar en mapping-metadata blijven), INIT
   voor alle AP's, alle threads naar de BSP (§6.2a), en de boot gaat verder als één CPU (INV-5).
7. Pas na alle acks: frames naar `FRAMES.free`, gevolgd door de bestaande paging-audit.

**Kernelhelft:** dezelfde stappen met `targets = alle online CPU's` (ADR-024-herziening punt 6).

### 7.2 Procesexit

Volgorde: alle threads DEAD (§6.4) → `P.active_cpus` moet 0 zijn, behalve de exiterende CPU
zelf, die eerst naar de kernel-CR3 wisselt → shootdown met `targets` = de snapshot
(meestal leeg, maar het protocol wordt altijd doorlopen) → `cleanup` (K05) → frames vrij.
`check_invariants` krijgt: *geen vrij frame wordt nog door een PTE in een levende adresruimte genoemd*
en *`active_cpus` van een dood proces is 0*.

### 7.3 FPU-context bij migratie (ADR-045)

`T.fxsave` is geldig zodra stap 6.3.3 op de oude CPU klaar is. De `RUNQ`-release (stap 6.3.7)
en de `RUNQ`-acquire op de nieuwe CPU geven een happens-before naar de FXRSTOR daar. Een thread
wordt dus nooit hersteld voordat zijn beeld volledig is weggeschreven. Een migratiefixture
bewijst dat met een FPU-canary (§9).

### 7.4 Usercopy tijdens gelijktijdige unmap (INV-6)

Usercopy valideert en kopieert onder `ASPACE[P]`. Een gelijktijdige unmap van dezelfde P
moet daardoor wachten. Pas na de kopie mag de PTE verdwijnen, en de shootdown zorgt dat de
kopiërende CPU geen oude vertaling meer gebruikt. Met één thread per proces is dit nu
onbereikbaar, maar het contract geldt vooruit.

---

## 8. Implementatiestappen (elk een eigen kleine PR, gestapeld)

| Stap | Resultaat | Schrijfbestanden (verwacht) | Poort |
|---|---|---|---|
| 5a | Ticketlock + irqsave + lockdep + per-CPU-blok voor de BSP | `kernel/src/m2/smp.rs` of een nieuw `kernel/src/m2/lock.rs` [te verifiëren: moduleconventie] | hosttests van de lock (loom/stress op de host), native lockdep-negatief |
| 5b | `KERNEL`-lock om alle M3-syscalls; scratchslot per CPU; ADR-074 | M3-entrypoint, paging-scratchcode, `docs/DECISIONS.md` | volle `milestone-m2`- en M3-regressie op 1 CPU, ongewijzigd |
| 5c | BSP LAPIC-timer + idle met `sti; hlt`; PIT blijft de klok | `smp.rs`, idle-lus | `native-rtc` 5/5, monotone-klokcheck, BSP-preëmptiecanary |
| 5d | Runqueues per CPU; gebruikersprocessen op AP's; CR3-wissel §6.3; 0xe8 | M3-scheduler, `smp.rs` | K05 6/6 op 4 vCPU's; `SMP:SCHED:CPUS=n` met user-werk op elke CPU |
| 5e | Procesbrede shootdown §7.1/7.2 | paging/unmap, cleanup | dove-AP-negatief, controle zonder shootdown (moet stale-read tonen) |
| 5f | Migratiefixture + concurrente stress + K06-acceptatie | testfixtures, `tools/run_smp.py`, `docs/verification/k06-smp-sched/` | §9 compleet; progress K06 → accepted |

Na elke stap: PROJECT-STATE bijwerken, committen, pushen, een concept-PR met auto-merge
(AGENTS.md, 5 november 2026) en daarna direct de volgende stap.

---

## 9. Acceptatie- en negatieve tests (native QEMU q35/OVMF/TCG)

Profielen: 1, 2, 4 en 8 vCPU's; zowel single- als multi-thread-TCG (`-accel tcg,thread=multi`).

**Positief (moet slagen):**
1. **Concurrente belasting:** ≥ 2× zoveel CPU-gebonden ring-3-processen als CPU's, plus een
   IPC-ping-pong tussen processen op **verschillende** CPU's, 30 s. Eis per CPU: user-ticks > 0,
   contextwissels ≥ 4, geen auditfout, en alle K07-quota blijven per app begrensd.
2. **Adresruimte-isolatie onder SMP:** twee processen op verschillende CPU's schrijven op
   hetzelfde user-VA verschillende patronen en controleren die herhaald (de K03-proef, gelijktijdig).
3. **Unmap-race:** proces P draait op CPU 1 en leest in een lus pagina X; vanaf CPU 0 wordt X
   ge-unmapt en het frame vrijgegeven en hergebruikt. Eis: P krijgt een #PF en geen oud of
   nieuw patroon.
4. **Migratie met canary:** de fixture migreert T tussen CPU 1→2→0 met unieke GPR- én
   x87/SSE-waarden (ADR-045); na elke migratie zijn alle waarden gelijk en is `active_cpus` correct.
5. **Kill op een remote CPU:** kill van een proces dat op CPU 3 draait; K05-cleanup-balans
   (vrije frames voor = na), `active_cpus == 0`.
6. **BSP-interrupts:** de monotone klok loopt tijdens de belasting (Sys3 stijgt) en de idle-BSP
   wordt aantoonbaar gewekt.

**Negatief (moet de juiste marker en fail-closed gedrag geven):**
7. `test-smp-lockdep-order`: omgekeerde lockvolgorde → `SMP:LOCKDEP:ORDER`.
8. `test-smp-held-across-switch` → `SMP:LOCKDEP:HELD_ACROSS_SWITCH`.
9. `test-smp-proc-shootdown-deaf`: één AP negeert 0xe0 → frame in quarantaine, INIT voor
   alle AP's, threads naar de BSP, de boot gaat verder als één CPU.
10. `test-smp-proc-no-shootdown` (controleprofiel): zonder shootdown leest P in test 3 het
    hergebruikte frame. Dat bewijst dat test 3 een echt gevaar meet (zelfde methode als ADR-072).
11. `test-smp-double-run`: forceer dezelfde TID op twee runqueues → `SMP:SCHED:DOUBLE_RUN`.
12. `test-smp-resched-deaf`: AP negeert 0xe8 tijdens een kill → INV-5-pad.
13. Canary-injectie (open punt uit ADR-073): corrumpeer bewust een bewaard register → de
    canary moet afgaan.

**Regressie (ongewijzigd groen):** `milestone-m2` (TCG/1CPU), K05 6/6, `native-rtc` 5/5,
alle stap-1–4-gevallen (14/14), host-tests `boot-protocol`, `tools/tests`, Clippy, fmt.

**Bewijs:** `docs/verification/k06-smp-sched/{README.md,native-report.json}` met broncommit,
exacte commando's, exitcodes, markers per geval en hashes. Daarna `progress.json` K06 →
`accepted` (+2 punten, 41 → 43) en `tools/progress.py --write` en `--check`.

**Commando's (uit de handoff; `run_smp.py` bouwt niet zelf):**
```
export NOS_BUILD_DIR="$PWD/build/k06-native-smp"
python3 -B tools/build.py --kernel-feature native-smp
python3 -B -u tools/run_smp.py --profile native-smp
```

---

## 10. Primaire bronnen

- Intel SDM Vol. 3A (actuele editie, https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html):
  MP-initialisatie (INIT-SIPI-SIPI); APIC (ICR, delivery status, LVT-timer, TPR/PPR,
  prioriteitsklassen per vector >> 4, EOI); "Invalidation of TLBs and Paging-Structure Caches"
  en "Propagation of Paging-Structure Changes to Multiple Processors" (§7.1, inclusief CR3-load
  zonder PCID); memory ordering (locked instructies, LFENCE/MFENCE) voor §4.1/§6.3;
  FXSAVE/FXRSTOR. Zoek op sectietitel; de nummers verschillen per editie.
- ACPI 6.5 §5.2.12 (MADT), https://uefi.org/specs/ACPI/6.5/ (ongewijzigd ten opzichte van ADR-070).
- Ticketlock: Mellor-Crummey & Scott, "Algorithms for Scalable Synchronization on
  Shared-Memory Multiprocessors", ACM TOCS 9(1), 1991. [nog niet opnieuw nagelezen voor deze spec]

---

## 11. Open aannames en risico's

- **[te verifiëren]** De IRQ0-route en of AP's die nooit ontvangen; de vorm van de
  M3-procesentry; waar de FXSAVE-code zit; de moduleconventie voor een nieuwe lock-module.
- De grote kernellock begrenst de parallelle syscall-doorvoer. Dat is acceptabel voor K06
  (het criterium vraagt correctheid onder concurrente belasting, geen schaalbaarheid), maar
  het is een R02-risico.
- QEMU TCG modelleert geen echte TLB-timing of speculatie. Test 10 bewijst dat stale
  vertalingen gemeten worden, niet dat fysieke CPU's zich hetzelfde gedragen.
- Device-IRQ's blijven gepolld/op de BSP; IOAPIC naar AP's valt buiten K06.
- Review door AI is geen menselijke audit (B05 blijft open).
