# K06 F5/F7: STUCK ook in release en SERIAL-lock per regel (aanvulling ADR-074)

**Datum:** 6 november 2026. **Branch:** `fix/k06-f5-f7-diagnostics` (PR #238), gestapeld op
`feat/k06-5b2-kernel-lock` (`59476e17`, met F3/F4). **Bron:** `6307e3f8`, vingerafdruk (v3) `6161dde6…`.
**Taakspec:** `docs/implementation/k06-steps/03-F5-F7-stuck-serial.md`.
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
26/26 gevallen over 21 profielen, alle op dezelfde bron. Per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`
(alle builds zijn `--release`).
- **DG-2, positief.** In elk normaal en negatief profiel schrijft elke CPU die de synctest draaide
  na de stresstest 200 regels `SMP:SER:<cpu>:<seq>`, in vier fragmenten per regel, en daarna volgt
  `SMP:SERIAL:PASS`. De oracle eist elke regel heel, per CPU de volgorde 0..199, en geen regel
  buiten die fase. In `four` wisselde de schrijvende CPU na elke regel (800 regels, 800 runs van
  lengte 1): de lock stond echt onder contentie.
- **DG-1, `test-smp-lock-stuck` (2 vCPU's, releasebuild).** Na de IPI-rondes houdt de BSP een lock
  op KERNEL-niveau vast. De AP wacht en meldt `SMP:LOCK:STUCK:10:CPU=1`, exit 0x10 (raw 33). De BSP
  wacht zelf maximaal 1000 ms (PIT) en zou daarna `SMP_LOCK_STUCK_MISSED` geven. De melding kwam dus
  binnen 1 s, zonder host-timeout (gast totaal 2,4 s, inclusief firmware).
- **Controle `test-smp-serial-unlocked` (4 vCPU's).** Dezelfde fixture zonder `SERIAL`. De oracle
  eist minstens één vermengde regel en daarbuiten de exacte reeks. Vooraf experiment op dezelfde
  kernelcode, nog met de oude oracle: **10 van 10 runs** hadden vermengde regels (bijv.
  `SMSMP:SERP:SSER:S:MP:SER:2:0`). TCG multi-thread laat de race dus betrouwbaar zien, en daarom
  is het een native case en niet alleen een hosttest.
- De 18 bestaande profielen geven ongewijzigd hun marker. De normale reeks heeft de fixtureregels
  en `SMP:SERIAL:PASS` vóór `SMP:SYNC:PASS`.

## Eén-CPU-regressie (DG-3), nieuw `6307e3f8` tegen basis `59476e17`
Dezelfde commando's in twee worktrees: `tools/test_milestones.py --through m3` (M2 en M3),
`tools/build.py --kernel-feature native-rtc && tools/run_rtc.py --case all`,
`tools/test_supervisor.py` (K05) en `tools/test_resource_pressure.py` (K07). Alles exit 0 op
beide. De seriële uitvoer is per runlabel vergeleken (37 labels):
- **36/37 byte-gelijk.** Uitzonderingen in de normalisatie: alleen de meetwaarden
  `M3:MAX_SYSCALL_CYCLES`, `M3:MAX_TIMER_GAP_CYCLES` (TSC-metingen) en
  `SUPERVISOR:STATUS_CALLS` (een pollteller). Die verschilden ook tussen twee runs van de
  ongewijzigde basis (bijv. STATUS_CALLS 52 tegen 55).
- **1/37 niet byte-gelijk: `supervisor-quota`, regel `SUPERVISOR:TICKS`.** Dit is een telling van
  timerticks tijdens de workload, dus wandkloktijd. Basis: 30, 30, 29. Nieuw: 31, 32, 31 (drie
  runs elk, achter elkaar op dezelfde host). Het verschil is klein maar wel consistent. Een oorzaak
  in de code is niet aangetoond: het M3-pad krijgt alleen een niet-genomen `cfg!`-tak en een
  kleinere `IrqLock` (zonder het `name`-veld). **Open punt;** byte-gelijkheid is voor deze regel
  niet bewezen.

## Host en statisch
- `tools/tests/test_smp_serial.py` 16/16 (nieuw: vermengde, gesplitste, verloren, herhaalde en
  verplaatste fixtureregels worden afgewezen; `!` alleen op de terminale marker; de
  controle eist vermenging; STUCK alleen in zijn eigen profiel).
- `cargo fmt --all -- --check` schoon. Kernel-clippy (`-p os-kernel --features native-smp`) geeft
  dezelfde 8 al bestaande meldingen als de basis, geen nieuwe. `smp-lock` is niet gewijzigd.
- `tools/gate.sh` exit 0 (34 s): cargo check, cargo test 511 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is **niet uitgevoerd**.

## Afwijkingen van de spec en grenzen
- **Tijdsbasis (DG-1).** De spec zegt "gemeten met de PIT-klok". De PIT is van de BSP (ADR-070).
  Daarom ijkt de BSP vóór de eerste IPI 10 ms PIT-kanaal 2 tegen de TSC, en leest een wachter
  alleen de TSC. Aanname: de TSC loopt met een vaste snelheid (QEMU/TCG, KVM). Shortcut tot stap 06.
- **Lockdep en niveau 70.** `smp_lock::Lockdep` kent alleen niveaus 0..=63. `SERIAL` wordt niet in
  het lockdepwoord vastgelegd (shortcut in `lock.rs::tracked`). Verbreden kan in `crates/smp-lock`,
  buiten de schrijfbestanden van deze stap.
- Niet uitgevoerd: CI op de gestapelde PR (een gestapelde base start `native-smp.yml` niet),
  `tools/test_milestones.py --through m4` (pad-bug, #235), hardware. TCG modelleert geen echte
  geheugenordening.
