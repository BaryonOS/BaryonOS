# K05 — vaste posities van startcapabilities

Taakcontract, 29 oktober 2026. Basis main3836361 (#54), eigen branch
codex/supervisor-startup-slots. Eigen Rust no_std-kernel/x86_64/UEFI;
Het bestaande startcontract (ingevoerd met ABI1.3), rechten en quota blijven gelijk. Alleen de raw-spawn-
argumentindex wordt behouden; geen nieuwe feature of SDK-interface.

Invarianten: cap0↔offset56 en cap1↔64, nul betekent afwezig; beide worden
vóór allocatie op rechten/type/TRANSFER gecontroleerd. count telt alleen
benodigde nodes. Geen gedeeltelijke allocatie bij ongeldige slotrechten.
Bron: K05-PROCESS-MANAGEMENT §3 en P2-thread PRRT_kwDOVZC91s6xvSPj (#51).

Acceptatie: echte sparse cap1 in native NORMAL, legecap0 en geldige SENDcap1,
RECV/DERIVE geweigerd,24IPC-antwoorden en cleanup. Vier extra negatieve
raw-slot-/rechtengevallen in DENIED. Oude code faalt aantoonbaar; volledige
K05poort moet vijf native gevallen én hostcontroles bevestigen. Root draait
precies één volledige integratiegate op gecommit bevroren bron. Bij het eerste
checkpoint nog niet uitgevoerd; de volledige gate is hieronder vastgelegd.
Score blijft32; geen nieuw criterium.

Rollback: normale revert van de K05-slotfix; geen resets/nieuwe rechten.
Bewijs geldt voor QEMU, niet voor SMP/fysiekehardware of gebruikersimages.

## Geïntegreerde acceptatie

Exact geteste bron `56b576b16da1e4503bc5f9167a476474a58c33cf`, fingerprint
`cccf0e3e7fcec439428208c4458a0194ec23c6120e1f03d96abe65b0711631b7`.
Geen bron-/HEADwijziging tijdens de gate. Rust1.90.0, DebianQEMU10.0.13,
expliciet pc-q35-8.2/qemu64/TCG/1CPU/256MiB, OVMF2025.02-8+deb13u1/4M.

`NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR=$PWD/build/verify-supervisor-slots NOS_TARGET_DIR=$PWD/target/verify-supervisor-slots CARGO_TARGET_DIR=$PWD/target/verify-supervisor-slots python3 -B -u tools/test_supervisor.py`: exit0;5/5 echte VM-runs, alle raw33. Vier hostcontroles:40capability-,17SDK-,1voorbeeldlib- en7harnastests groen. Plan normal/crash/denied/parent-exit/quota is behouden.

NORMAL: één echte sparse spawn, cap0nul/cap1SEND plus RECV/DERIVEweigering
(3controles),24antwoorden, herstart en volledige frame/enginecleanup.
DENIED:21/21 gematchte negatieve controles,11 geweigerde spawns, geen
partialallocation. Andere drie gevallen behouden echte fault, orphan- en
quotacontroles. Nieuwe native scenario faalde vóór de productiefix met de
oude kernel na15s (before/result.json); herstelde gerichte versie groen
(focused-after), apart van de volledige integratiegate.

Verder exit0: `cargo +1.90.0 clippy --locked --manifest-path sdk/Cargo.toml -p nieuw-os-supervisor-example --all-targets -- -D warnings`, root-/SDKfmtchecks en diffcontrole. Afzonderlijke AI-review op2665f68: geen blocker,7harnastests onafhankelijk herhaald. Ruwe gate-resultaten/serial/stderr en hostuitvoer staan onder gate/; SHA256SUMS bindt de exacte bytes, inclusief firmwareCRLF. Geen fysiekehardware-/SMPclaim. Geen nieuwe afhankelijkheden, rechten of ABI.
