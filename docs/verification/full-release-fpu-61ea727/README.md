Echte native FPU/klok-proef op commit `61ea7278390d5235a8bc024cbf084222a7b0c658`, bronvingerafdruk `2eb46ac69aec5c7d60ed06278705c97f4fd7d67347afc3213b77eac3624cc013`.

`build(None, "native-fpu", "native-fpu")` en `run_fpu()` via de bestaande tools slagen (exit 0, QEMU raw 33). Drie schone proceslevens, 48 registercontroles, 51 kloklezingen, 18 geweigerde aanvragen, 49 timerinterrupts en 172 contextopslagen/herstellingen; één echte unsupported-AVX-fout, twee exits, één hergebruikt slot en 8192 gewiste contextbytes. De ruwe uitvoer, exacte QEMU-opdracht, bron-, loader-, kernel-, firmware- en apphashes staan in de bijgevoegde rapporten.

Artifacts: `build/full-release-61ea-fpu/esp/` en het runpad in `result.json`. Alleen pc-q35-8.2/qemu64/TCG/1 CPU/256 MiB. Geen bewijs van fysieke CPU-correctheid, AVX-ondersteuning, gereserveerde MXCSR-bits, SMP of volledige releaseacceptatie.
