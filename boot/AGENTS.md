# Bootregels
Lees root-AGENTS.md en docs/ARCHITECTURE.md. Alleen een UEFI-loader is nog geen eigen werkende OS-kernel.
Definieer het boot-ABI: geheugenkaart/eigenaarschap, toegankelijke mappings, entrypoint, stack, CPU-status en foutpad. Controleer UEFI-specificatie en correcte ExitBootServices-sequentie; geen firmwarebootservices na overdracht. Het UEFI-calling convention en de kernel-ABI zijn niet stilzwijgend uitwisselbaar.
Valideer ELF-/imagegrenzen en rekenkundige overflows. Stel geen beveiligde bootketen vast op basis van een gewone unsigned emulatorboot. Geen echte firmware of apparaten wijzigen.
Test een geslaagde overdracht én corrupte/ontbrekende kernel. Marker moet door kernelcode na overdracht worden geproduceerd. Documenteer firmware- en emulatorversies.
