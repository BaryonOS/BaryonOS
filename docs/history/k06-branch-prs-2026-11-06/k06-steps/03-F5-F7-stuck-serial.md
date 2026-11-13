# L0-specificatie — K06 F5/F7: diagnose bij een vastgelopen lock en een serieel orakel zonder vermenging

**Datum:** 6 november 2026 · **Basis:** de head van stap 01 (5b-2) [nog niet gebouwd]; los van 02
**Branch/worktree:** `fix/k06-f5-f7-diagnostics` in `/home/simon/projects/nieuw-os-k06-f5f7`
**Spec:** `K06-SMP.md` §4.1 (begrensde spin), §4.2 (niveau 70 SERIAL), §4.3 (NMI/#DF met `try_lock`);
auditbevindingen F5, F7 en F12 (deel deadlines).

## 1. Probleem (waarom nu)
- **F5:** de begrensde spin met `SMP:LOCK:STUCK` staat achter `cfg!(debug_assertions)`, maar elke
  native gate bouwt met `--release`. Een deadlock in de gate wordt dus een hang tot de host-timeout,
  zonder diagnose. De grens van 2^24 is bovendien niet afgeleid van de shootdown-deadline.
- **F7:** `kernel/src/serial.rs` heeft geen lock. Een AP en de BSP kunnen tegelijk markers schrijven.
  De host-parser kan dan een terminale marker missen of een regel uit twee halve regels lezen. Vanaf
  stap 08 schrijven meerdere CPU's echt tegelijk.

## 2. Invarianten (normatief)
- **DG-1 Begrensd in de gate.** In elk `native-smp`- en `test-smp-*`-image is de spin begrensd,
  ongeacht `debug_assertions`. De grens is een tijd en geen iteratietelling:
  `LOCK_STUCK_MS = 4 × SHOOTDOWN_DEADLINE_MS` (400 ms), gemeten met de PIT-klok. Overschrijding →
  `SMP:LOCK:STUCK:<niveau>:CPU=<n>` en stop.
  *shortcut: PIT-polling loopt onder TCG mee met de hostklok; vervang dit door een gekalibreerde TSC zodra 06 de verhouding meet.*
- **DG-2 Eén regel is ondeelbaar.** Elke marker wordt in zijn geheel geschreven onder `SERIAL`
  (niveau 70, irqsave). Terminale paden (NMI, #DF, panic) gebruiken `try_lock`; lukt dat niet binnen
  een korte begrensde poging, dan schrijven ze toch, met het voorvoegsel `!` (zodat de oracle een
  mogelijk vermengde regel herkent).
- **DG-3 Geen verandering op één CPU.** Op één CPU is er nooit een wachter; de seriële uitvoer is
  byte-gelijk.

## 3. Interface
- `lock.rs`: de begrensde variant wordt gekozen met `cfg(feature = "native-smp")` of een testfeature,
  niet met `debug_assertions`. Release-builds van andere profielen blijven onbegrensd (daar is geen
  wachter).
- `serial.rs`: `static SERIAL: IrqLock<()>` op niveau SERIAL; `write_line(&[u8])` neemt de lock om de
  hele regel; `write_terminal(&[u8])` met `try_lock`.
- `tools/tests/test_smp_serial.py`: een regel met `!` mag alleen een terminale marker zijn.

## 4. Native fixture en negatieve tests
- Positief: elke CPU schrijft 200 regels `SMP:SER:<cpu>:<seq>` door elkaar. De host controleert
  dat elke regel heel is en dat de seq per CPU oploopt. Marker `SMP:SERIAL:PASS`.
- `test-smp-lock-stuck` (2 vCPU's): de BSP houdt een testlock vast en laat hem nooit los; de AP
  wacht. Verwacht: `SMP:LOCK:STUCK:10:CPU=1` binnen 1 s en exit 0x10, geen host-timeout.
- `test-smp-serial-unlocked` (controleprofiel, 4 vCPU's): zonder lock. De host-oracle moet een
  vermengde regel vinden in minstens één van tien runs [te verifiëren: of TCG multi-thread dat
  betrouwbaar laat zien; zo niet, dan alleen een hosttest van de parser met een vermengde invoer].

## 5. Schrijfbestanden
`kernel/src/m2/lock.rs`, `kernel/src/serial.rs`, `kernel/src/m2/smp.rs` (fixture),
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (aanvulling ADR-074), `docs/PROJECT-STATE.md`,
voortgangsbestanden.

## 6. Acceptatie
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py` met de nieuwe gevallen.
- Native: `native-smp` (alle gevallen), `test-smp-lock-stuck`, en de regressie van alle
  één-CPU-profielen met een byte-vergelijking van de seriële uitvoer.
- K06 blijft open (41/100).

## 7. Buiten scope
Een lockvrije regelbuffer per CPU (alternatief voor DG-2; alleen kiezen als de lock meetbaar
contentie geeft), kalibratie van de LAPIC-timer (06), het herhalingsharnas (09).
