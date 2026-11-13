# L0-specificatie — K06 F3/F4: lockdep met handlercontext en lockinstanties

**Datum:** 6 november 2026 · **Basis:** de head van stap 01 (5b-2) [nog niet gebouwd]
**Branch/worktree:** `fix/k06-f3-f4-lockdep` in `/home/simon/projects/nieuw-os-k06-f3f4`
**Spec:** `K06-SMP.md` §4.2–4.4; ADR-074 (aanvullingen 5b, F2); auditbevindingen F3, F4 en F-T1.

## 1. Probleem (waarom nu)
- **F3:** dat handlers nooit `KERNEL`/`ASPACE` nemen, staat alleen in commentaar. Neemt een handler
  ze toch op een CPU die niets vasthoudt, dan slaagt lockdep. Het IF-venster doet dan `sti` **binnen
  de handler**, met geneste interrupts op dezelfde stack als gevolg.
- **F4:** lockdep heeft één bit per niveau. Een IPC-kopie tussen twee adresruimten (twee `ASPACE`)
  of een migratie (twee `RUNQ`, spec §4.2 "oplopende CPU-index") geeft nu een valse
  `SMP:LOCKDEP:ORDER`. Daarmee is de geplande IPC-route op SMP onmogelijk.

## 2. Invarianten (normatief)
- **LD-1 Context.** Per CPU telt `irq_depth` de geneste IDT-entries; elke stub verhoogt hem na de
  `swapgs`-proloog (ADR-076) en verlaagt hem vóór de epiloog. `lock_irqsave` op een niveau
  ≤ `ASPACE` met `irq_depth > 0` → `SMP:LOCKDEP:IN_HANDLER:<niveau>` en stop.
  NMI/#DF/#MC gebruiken geen GS (GS-6) en tellen niet mee; zij nemen geen locks.
- **LD-2 Venster alleen buiten handlers.** Het `sti; pause; cli`-venster opent alleen bij
  `irq_depth == 0` (aanvulling op S1/S3 uit 5b).
- **LD-3 Instanties.** Twee locks van hetzelfde niveau mogen alleen samen via `lock_pair(a, b)`. Die
  neemt ze in vaste volgorde: `ASPACE` op oplopende pid, `RUNQ` op oplopende CPU-index. Lockdep
  houdt per niveau een teller (max 2) en de laatst genomen sleutel bij. Een tweede acquire op
  hetzelfde niveau buiten `lock_pair` of met een dalende sleutel → `SMP:LOCKDEP:ORDER`.
- **LD-4 LIFO blijft** (F2): de release controleert `level == highest()`, ook bij paren (eerst de
  tweede sleutel).

## 3. Interface
- `smp_lock::Lockdep`: `acquire(level, key)`, `release(level, key)`; een teller per niveau.
  `held_mask` blijft beschikbaar voor de switchcontrole.
- `lock.rs`: `IrqLock::lock_pair(&self, other: &Self) -> (Guard, Guard)` met de sleutel uit de lock
  (pid of CPU-index, vastgelegd bij de constructie).
- `CpuLocal.irq_depth: u32`, alleen door deze CPU geschreven. De stubs krijgen `inc`/`dec` op
  `gs:[irq_depth]` na de proloog.

## 4. Hosttests (F-T1, goedkoop, vóór native)
- Een propertytest: willekeurige reeksen van acquire en release (met paren) tegen een eenvoudig
  referentiemodel. Lockdep en het model moeten het over elke stap eens zijn.
- Een loom-model van `TicketLock` (exclusie en FIFO) [te verifiëren: loom als dev-dependency;
  controleer herkomst, licentie en pin de versie volgens AGENTS.md].
- Gevallen: een paar in de verkeerde volgorde, een derde lock op hetzelfde niveau, release van de
  eerste van een paar, `IN_HANDLER`.

## 5. Native fixture en negatieve tests
- Positief in `native-smp`: twee AP's nemen `lock_pair` op twee testlocks van niveau ASPACE, met
  tegengestelde argumentvolgorde, 1000×. Er mag geen deadlock en geen marker optreden. Marker
  `SMP:LOCKPAIR:PASS`.
- `test-smp-lock-in-handler` (2 vCPU's): de 0xe8-handler neemt `KERNEL` →
  `SMP:LOCKDEP:IN_HANDLER:10`, exit 0x10.
- `test-smp-lockpair-order` (2 vCPU's): twee ASPACE-locks los genomen in dalende volgorde →
  `SMP:LOCKDEP:ORDER:20->20`, exit 0x10.

## 6. Schrijfbestanden
`crates/smp-lock/src/lib.rs` (+ dev-dependency als loom wordt gekozen, met een uitzonderingsnotitie),
`kernel/src/m2/lock.rs`, `kernel/src/m2/smp.rs` (stubs en fixture), `kernel/src/m2/arch.rs` (stubs
buiten `smp.rs`, als die bestaan), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (nieuwe ADR
of aanvulling ADR-074), `docs/PROJECT-STATE.md`, voortgangsbestanden.

## 7. Acceptatie
- `tools/gate.sh` exit 0; `cargo test -p smp-lock` (inclusief de property- en loomtests); clippy; fmt.
- Native: `native-smp` (alle gevallen, inclusief `SMP:LOCKPAIR:PASS`), de twee negatieve profielen en
  regressie `milestone-m2`.
- K06 blijft open (41/100).

## 8. Buiten scope
Meer dan twee locks van één niveau, een runtime-eigenaar per lock (05), een SERIAL-lock (03).
