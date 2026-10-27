# Gerichte M3-regressie: groter dan 64 usermappings

`large_linker.ld` linkt dezelfde ongewijzigde `probe.rs` met 33 extra NOLOAD BSS-pagina's. Entry/text en de vier datawoorden blijven gelijk. De bestaande `linker.ld`, M3-probe en M3-tests blijven ongewijzigd.

```powershell
python tools/build_users.py
python tools/build_users.py --kind large
python -B -X utf8 userspace/tests/test_large_probe.py --elf build/users/probe_large.elf
```

Root beheert de buildtool en de echte `test-m3-large-image`-QEMU-run. Output: `build/users/probe_large.elf` en `build/users/manifest-large.json`. De geselecteerde variant wordt als `NOS_USER_ELF` ingebed. Er is geen nieuwe Rust-configuratie, library of usercode nodig.

Exacte rekening, afgestemd met kernelagent:

- RX op `0x40000000`, één pagina; de filebytes zijn gelijk aan normale `probe.elf`.
- RW op `0x40010000`, 32 filebytes en 139264 memorybytes: 34 pagina's.
- `.bss` begint op `0x40011000`, is SHT_NOBITS/NOLOAD en omvat 33×4096 bytes. De bytes tussen de vier datawoorden en de volgende paginagrens behoren eveneens tot het te nulmaken segmentgeheugen.
- Image: 35 pagina's. Met 16 heap- en 16 stackpagina's: **67 usermappings**, boven de oude64grens en onder de contractgrens2048.

Zes hosttests bevestigen deze artifacteigenschappen, gelijke tekstbytes, source/linker/binaryhashes en afwijzing van een gemuteerd onveilig segment. Dit bewijst geen werkelijk aanwezige/zeroed gast-RAM of geslaagde processtart. De echte kernelrun moet alle67 mappings aanmaken en auditen, het grote BSS nulmaken en de bestaande userprobe daadwerkelijk laten uitvoeren.

Bewijs: `verification/U03-LARGE-HOST-2026-10-27.md`, `verification/large-host-evidence.json` en het werkelijke testlog. De binary staat uitsluitend in het genegeerde buildpad. Herstel: verwijder deze optionele linker/hosttest en laat root de variantkoppeling terugnemen; normale M3 en IPC-fixture blijven bruikbaar.
