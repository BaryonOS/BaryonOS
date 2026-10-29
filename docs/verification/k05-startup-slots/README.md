# K05 — vaste posities van startcapabilities

Taakcontract, 29 oktober 2026. Basis main3836361 (#54), eigen branch
codex/supervisor-startup-slots. Eigen Rust no_std-kernel/x86_64/UEFI;
ABI1.3, capabilityrechten en quota blijven gelijk. Alleen de raw-spawn-
argumentindex wordt behouden; geen nieuwe feature of SDK-interface.

Invarianten: cap0↔offset56 en cap1↔64, nul betekent afwezig; beide worden
vóór allocatie op rechten/type/TRANSFER gecontroleerd. count telt alleen
benodigde nodes. Geen gedeeltelijke allocatie bij ongeldige slotrechten.
Bron: K05-PROCESS-MANAGEMENT §3 en P2-thread PRRT_kwDOVZC91s6xvSPj (#51).

Acceptatie: echte sparse cap1 in native NORMAL, legecap0 en geldige SENDcap1,
RECV/DERIVE geweigerd,24IPC-antwoorden en cleanup. Vier extra negatieve
raw-slot-/rechtengevallen in DENIED. Oude code faalt aantoonbaar; volledige
K05poort moet vijf native gevallen én hostcontroles bevestigen. Root draait
precies één volledige integratiegate op gecommit bevroren bron. Bij dit
checkpoint NIET UITGEVOERD. Score blijft32; geen nieuw criterium.

Rollback: normale revert van de K05-slotfix; geen resets/nieuwe rechten.
Bewijs geldt voor QEMU, niet voor SMP/fysiekehardware of gebruikersimages.
