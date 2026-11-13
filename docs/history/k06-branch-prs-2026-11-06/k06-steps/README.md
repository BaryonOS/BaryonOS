# K06 — resterende stappen na F8 (L0-specificaties)

**Datum:** 6 november 2026 (Europe/Amsterdam) · **Status:** ontwerp, niets hiervan is gebouwd of getest.
**Basis:** `7ffdacfa` (`fix/k06-f8-swapgs`, PR #231), de top van de gestapelde K06-keten
#227 (5b) → #228 (5c) → #229 (5d-1) → #230 (F1/F2) → #231 (F8). `main` bevat alleen 5a (#225).
**Gelezen:** `docs/implementation/K06-SMP.md`, `docs/progress.json`, `docs/PROGRESS.md`,
`docs/verification/K07-ACCEPTANCE.md`, `docs/PROJECT-STATE.md` (handoff 5b), ADR-074 en ADR-076,
de lokale taakspecs 5b/5c/5d/F8 en het auditrapport `/tmp/k06_audit_report.md` (lokaal, niet in Git).

## Waarom er geen "K07 na K06" is

K01–K07 zijn **acceptatiecriteria** in het puntenregister (`docs/progress.json`), geen opeenvolgende
mijlpalen. **K07 (Resourcebeheer onder druk) is al behaald** op het single-CPU-QEMU-profiel
([bewijs](../../verification/K07-ACCEPTANCE.md)). De grens van dat bewijs zegt letterlijk: *"geen
SMP-bewijs"*. K06 is het laatste open criterium van spoor K (13/15). Wat er "na K06" komt, is dus:

1. de resterende K06-stappen hieronder, en
2. binnen 5f het herbewijs van de K07-quota onder SMP (spec §9.1 eist dat al).

Na acceptatie van K06 (+2 punten) is spoor K vol (15/15). Het volgende werk valt dan in andere
sporen (S09, H02–H06 enzovoort). Dat valt buiten deze map.

## Volgorde (auditrapport §4, aangevuld)

| # | Bestand | Stap | Dekt | Blokkeert |
|---|---|---|---|---|
| 1 | [01-5b-2-kernel-lock-scratch.md](01-5b-2-kernel-lock-scratch.md) | 5b-2 | spec §4.2/§5, F20, ADR-075 | alles met M3 op SMP |
| 2 | [02-F3-F4-lockdep-context.md](02-F3-F4-lockdep-context.md) | F3/F4 | handlercontext, lock per instantie | 04, 07, 08 |
| 3 | [03-F5-F7-stuck-serial.md](03-F5-F7-stuck-serial.md) | F5/F7 | begrensde spin in release, SERIAL-lock | betrouwbaar orakel voor 09 |
| 4 | [04-F16-F17-block-wake-kill.md](04-F16-F17-block-wake-kill.md) | F16/F17 | blokkeren, wekken en doden zonder lost-wakeup | 08 |
| 5 | [05-F19-F15-icr-quarantine.md](05-F19-F15-icr-quarantine.md) | F19/F15 | ICR-lock, quarantainebeleid met gehouden locks | 08 |
| 6 | [06-5c-2-bsp-timer-idle.md](06-5c-2-bsp-timer-idle.md) | 5c-2 | BSP-timer, idle, BSP-venster (F6, F9, F11) | 07, 08 |
| 7 | [07-5e-process-shootdown.md](07-5e-process-shootdown.md) | 5e | procesbrede shootdown op `active_cpus` (F18) | 08 |
| 8 | [08-5d-2-ring3-on-aps.md](08-5d-2-ring3-on-aps.md) | 5d-2 | M3-processen op AP's | 09 |
| 9 | [09-5f-acceptance.md](09-5f-acceptance.md) | 5f | migratie, stress, §9-matrix, K07 onder SMP | K06 → accepted |

Stap 2, 3 en 5 raken elkaars bestanden nauwelijks (`crates/smp-lock` en `lock.rs` versus `smp.rs`
en `serial.rs`). Ze kunnen in aparte worktrees parallel lopen, maar wel gestapeld op 01. Er mogen geen
twee schrijvers tegelijk op `kernel/src/m2/smp.rs` werken.

## Vaste regels voor elke stap (AGENTS.md)

- Eén kleine PR per stap, als concept, met auto-merge. Niet op CI wachten; een rode check herstel je
  vóór nieuw werk op dezelfde branch.
- Volgorde: eerst het invariant, dan de interface, dan de implementatie, dan de negatieve test.
- De één-CPU-regressie (`milestone-m2`, M3, K05, K07, `native-rtc`) blijft ongewijzigd groen.
- ADR-nummers: 075 is gereserveerd voor 5b-2, 076 is F8. Neem daarna het **eerstvolgende vrije
  nummer bij de start**: er werkt een parallelle sessie in dezelfde repo.
- Na elke stap: `PROJECT-STATE.md` (handoff), `tools/progress.py --write` en `--check`. De score
  blijft 41/100 tot stap 9 native groen is.
- "Niet uitgevoerd" is geen groen resultaat. Lokaal is QEMU niet zeker aanwezig (WSL). Controleer
  het met `command -v qemu-system-x86_64`; ontbreekt het, dan geldt `native-smp.yml` op de PR.
