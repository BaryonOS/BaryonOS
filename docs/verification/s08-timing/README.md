# S08-timingonderzoek — virtio-completiedeadline (29 oktober 2026)

Compact, hashgebonden bewijs bij het onderzoek naar de eenmalige
`s08-native-resilience`-fouten op PR #47 en de daaruit volgende verhoging van
de fixturebrokerdeadline (64 → 500 ticks) in PR #48. Besluit: aanvulling van
29 oktober op ADR-029 in `docs/DECISIONS.md`. Verloop: `docs/PROJECT-STATE.md`.

## Wat hier staat

- `runs.json`: per QEMU-run van de S08-gate (`tools/test_storage_resilience.py`,
  profiel `m6c`) en de M6e-gate (`tools/test_volume.py`, profiel `m6e`) de case,
  het label, de bronvingerafdruk, de HEAD-commit, de duur, `passed`, de
  gerapporteerde maximale completielatentie in ticks, het aantal verstreken
  deadlines en de SHA-256 van de bijbehorende `result.json`. De `result.json`-
  bestanden zelf liggen onder `build/run/` en worden niet gecommit; de hashes
  binden dit rapport eraan.
- Dit bestand: methode, meetwaarden en grenzen.

## Samenvatting per gate-invocatie

Een tick is 10 ms (PIT-divisor 11932, 100 Hz). De kolom "runs" telt echte
QEMU-invocaties; "timeouts = 1" telt runs waarin de broker precies één
verstreken deadline meldt (verwacht in de drie timeout-fixtures, 0 elders).

| Profiel | HEAD bij run | Bron | Runs | Geslaagd | Max. completielatentie (ticks) | Runs met timeouts = 1 |
|---|---|---|---|---|---|---|
| M6c/S08 | `a9c38e3` | `14fa4d37` | 13 | 13 | n.v.t. (vóór diagnostiek) | n.v.t. |
| M6c/S08 | `8ef3a9d` | `c0ee971e` | 26 | 26 | 0–34 | n.v.t. |
| M6c/S08 | `c491d9e` | `1236752b` | 13 | 13 | 1–2 | 3 |
| M6c/S08 | `581cac6` | `f5737e78` | 13 | 13 | 0–2 | 3 |
| M6e | `` | `c0ee971e` | 7 | 7 | 0–2 | n.v.t. |
| M6e | `` | `1236752b` | 7 | 7 | 0–2 | 0 |
| M6e | `` | `f5737e78` | 7 | 7 | 1–2 | 0 |

- `a9c38e3`: gate met de oude deadline (64), vóór de fix, onbelast. Geen
  latentiediagnostiek; alle 13 runs geslaagd.
- `8ef3a9d`: 26 M6c-runs = 13 onbelast + 13 onder host-last (zie methode).
  Onbelast 1–2 ticks; onder last 15–34 ticks (150–340 ms), dus tot 53% van het
  oude budget en 7% van het nieuwe. Timeout-teller bestond nog niet.
- `c491d9e` en `581cac6`: diagnostiek ook op faalpaden plus timeout-teller;
  de drie timeout-fixtures melden exact één timeout, alle overige runs nul.
- M6e: alle stages geslaagd; de normal-stage doet 2746 writes en 140 flushes
  met maximaal 2 ticks latentie; nul timeouts.

## Methode

Omgeving: Linux-container, 4 cores, ext4 op `/dev/vda`, Rust 1.90.0, QEMU
8.2.2 (Ubuntu-pakket), `NOS_QEMU_MACHINE=pc-q35-8.2`, OVMF 4M. GitHub Actions:
`ubuntu-24.04`-runners; over hun schijf-I/O is niets gemeten.

1. **Host-`fdatasync` zonder QEMU** (momentopname, geen script in de repo):
   20× 4 KiB schrijven + `fdatasync` op dit bestandssysteem: 0,19–1,36 ms
   onbelast; onder de lastgenerator hieronder (4 schrijvers + 2 branders,
   83 metingen in 15 s): mediaan 136 ms, p90 203 ms, p99 430 ms.
2. **`fdatasync` binnen QEMU** via een wrapper buiten de repo die
   `strace -f -T -e trace=fdatasync,fsync` om `qemu-system-x86_64` zet, bij
   `python3 -B tools/run_m6c.py --case flush-error` (twee flushes per run):
   onbelast 5,8–6,2 ms; onder 6 schrijvers + 2 branders 162, 215, 229, 241 en
   311 ms (vijf runs, alle geslaagd). `strace` vertraagt QEMU zelf sterk, dus
   de rundusen uit die proef zijn niet vergelijkbaar met de gates.
3. **Lastgenerator** (gebruikt bij de belaste S08-gate op `8ef3a9d`, 6
   schrijvers + 2 branders, gestart 3 s vóór de gate):

```python
#!/usr/bin/env python3
"""Host I/O + CPU load generator for the S08 timing experiment (scratch only).

Spawns N writer processes that each write 32 MiB files and fsync them in a loop
on the given directory, plus M CPU burners. Runs until the stop file exists.
"""
import os, sys, time, multiprocessing as mp

def writer(directory, index, stop):
    path = os.path.join(directory, f"load-{index}.bin")
    block = os.urandom(1 << 20)
    while not os.path.exists(stop):
        fd = os.open(path, os.O_CREAT | os.O_WRONLY | os.O_TRUNC)
        for _ in range(32):
            os.write(fd, block)
        os.fsync(fd)
        os.close(fd)

def burner(stop):
    x = 1
    while not os.path.exists(stop):
        for _ in range(100000):
            x = (x * 1103515245 + 12345) & 0xffffffff

if __name__ == "__main__":
    directory, stop, writers, burners = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
    os.makedirs(directory, exist_ok=True)
    procs = [mp.Process(target=writer, args=(directory, i, stop)) for i in range(writers)]
    procs += [mp.Process(target=burner, args=(stop,)) for _ in range(burners)]
    for p in procs: p.start()
    for p in procs: p.join()
```

4. **Wachtcapaciteit van de probe** (tijdelijke diagnostiek, niet gecommit):
   `write-timeout` 5607 syscalls in 86 ticks tegenover `flush-error` 1832 in
   16 ticks ⇒ ~5,4k syscalls/s voor alle processen samen tijdens het wachten.
   `rt::LIMIT` (1 000 000 iteraties van één of twee syscalls) dekt daarmee
   minstens ~185 s.
5. **Onbruikbaar gebleken**: QEMU-throttling met `throttling.iops-total=1`
   raakt ook de 128 init-reads (32 sectoren × 4 segmenten) en liep tegen de
   hosttimeout van 60 s vóór de eerste PUT.

## Grenzen

- De CI-fouten zelf (`8ae9194` write-error, `c87f980` flush-error, beide
  `M6C:USER_CODE:33` vóór `M6C:FAULT_INJECTED`, beide geslaagd bij één
  herhaling) zijn lokaal niet gereproduceerd; de logs van die eerste pogingen
  zijn via de API niet meer beschikbaar. Dat een gemiste deadline de oorzaak
  was, is aannemelijk op grond van de gemeten latentieklasse, niet bewezen.
- Alle metingen zijn momentopnamen op één host; de lastgenerator is geen
  gekalibreerde nabootsing van een GitHub-runner.
- Keert de fout terug terwijl `M6C:BROKER:MAX_COMPLETION_TICKS` en
  `M6C:BROKER:TIMEOUTS` op het faalpad onder respectievelijk 500 en 1 blijven,
  dan ligt de oorzaak elders dan in completielatentie.
