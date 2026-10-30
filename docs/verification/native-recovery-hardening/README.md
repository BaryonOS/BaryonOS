# Native herstelhardening — taakcontract en bewijs

29 oktober 2026. Basis `00e6af5db4d315d2536471ccbce1cb1f4cf14958`,
branch `codex/native-recovery-hardening`. Eigen Rust no_std-kernel,
x86_64, één CPU, UEFI/OVMF, QEMU/TCG; geen hardware of echte schijf.

Doel: herstel de concrete B01-P1-fouten uit #52 en voltooi de bruikbare
S08-timingwijzigingen uit #48. Invarianten: ingetrokken sleutels herleven niet
in een levende instantie; entropy-gezondheidstoestand blijft over buffers
bestaan; timeout/eigenaarsexit laat geen herbruikbare onbevestigde DMA achter;
opslagfoutcompletion telt mee in diagnostiek zonder write-replay/rechtenwijziging.
Geen nieuwe crypto, afhankelijkheid, hardwareclaim of nieuw roadmappunt.

Interfaces: B01 IPC blijft v1. Herstel van een andere master is alleen in een
ongebruikte dienst toegestaan; herstel van dezelfde master behoudt tombstones.
Kernel is generieke broker; crypto blijft ring 3. Root bezit gedeelde feature-
lijsten en bewijs. Builders hebben afzonderlijke worktrees (WORK-QUEUE.md).

Acceptatie: gerichte keyring-/driver-/harnastests en clippy/fmt; echte B01-gate
met niet-antwoordend apparaat en herstart; S08-gate en native-volumegate op
gecombineerde bevroren bron. Negatief: oude blob na revoke, masterwissel,
RCT/APT over buffergrenzen, ontbrekende completion, cap-/DMA-cleanup en
malformed/foutcompletions. Nog NIET UITGEVOERD bij dit eerste checkpoint.

Bronnen: projectcontract B01-ENTROPY-AND-KEYS / ADR-043, NIST SP 800-90B
§4.4 (doorlopende RCT/APT), OASIS virtio 1.2 (reset en queue-eigendom).
OSDev: Virtio W003311, revisie 30307, 17 maart 2026, is aanvullende leerbron;
Random Number Generator W002935 onderscheidt bronentropy en CSPRNG.
Geen corpuscode overgenomen. Rollback via gewone revert van herstelcommits;
B01 blijft open als het noodzakelijke bewijs niet slaagt.

## Uitvoering op geïntegreerde bron

Bevroren commit `53e0d57af99c9673c97bc900fae15b98e08c8ca3`, fingerprint
`60edda7943e979a8e2c16caf6aa873a8b7d7535cd633e2a10f094d4b0bb6c00f`.
Alle native gates gebruiken expliciet `NOS_QEMU_MACHINE=pc-q35-8.2`,
QEMU 10.0.13, OVMF 2025.02-8+deb13u1, qemu64, TCG, één CPU, 256 MiB.
QEMU/OVMF staan alleen lokaal onder `.tools/`; officiële pakketnamen en
SHA-256 staan in `emulator-packages.json`, inventaris in `preflight.json`.
Sibling-worktrees gebruiken `LOCALAPPDATA=/workspace/nieuw-os/.tools`;
PATH/CARGO_HOME/RUSTUP_HOME verwijzen naar de bijbehorende cargo/rustup-
mappen. Build-/target-/Cargo-targetmappen zijn per gate gescheiden.

- `NOS_BUILD_DIR=$PWD/build/verify-keyring NOS_TARGET_DIR=$PWD/target/verify-keyring CARGO_TARGET_DIR=$PWD/target/verify-keyring python3 -B -u tools/test_keyring.py`: exit 0; 5 VM-runs, alle raw 33; `b01/keyring-gate.json`.
- In `/workspace/nieuw-os-verify-s08`: `NOS_BUILD_DIR=$PWD/build/verify-s08 NOS_TARGET_DIR=$PWD/target/verify-s08 CARGO_TARGET_DIR=$PWD/target/verify-s08 python3 -B -u tools/test_storage_resilience.py`: exit 0; 13 VM-runs, 7 gevallen en 6 koude read-only recoveries; `s08/s08-storage-resilience.json`.
- Hostverificatie: 11 commando's exit 0; 325 Python-, 6 skill-, 238 root-Rust- en 46 SDK-tests; clippy/fmt/syntax/voortgang groen. Exacte commands, totals en loghashes: `host-checks.json`.

Gerichte `used-write`/`used-flush` zijn eerder op agentcommit
`7fcf6bb1595f4d9ba49cbde5718b63d834dd3dcd` uitgevoerd (beide raw33,
Unknown/no-replay, resources groen, max1tick, nul timeouts). Deze twee
rapporten blijven apart aan die bron gebonden; ze worden niet voorgesteld als
de nieuwe gezamenlijke gate. Corefouten, verkeerde directe librarybuild en
de mislukte eerste firmwarefixture staan expliciet in de hostsummary en
`development-failures.json`. Correcte native builds gebruiken de bestaande
portable-crypto-configuratie via de SDK-builder.

Nog niet native getest: requestercrash en ontbrekende reset-ACK. Geen fysieke
RNG-/min-entropie- of hardwareclaim; geen persistente antirollback/B02.

- In `/workspace/nieuw-os-verify-volume`: `NOS_BUILD_DIR=$PWD/build/verify-volume NOS_TARGET_DIR=$PWD/target/verify-volume CARGO_TARGET_DIR=$PWD/target/verify-volume python3 -B -u tools/test_volume.py`: exit 0; 9/9 VM-runs, normal, read-only koude recovery, drie echte kill/recovery-paren en corruptieweigering; 28 volume-, 17 SDK- en 21 Python-tests groen. `volume/volume-gate.json`.

De drie geïntegreerde gates omvatten **27 echte VM-runs**; de drie opzettelijke
volume-kills eindigen met raw -9, de andere 24 met raw 33. Alle gates hebben
dezelfde bronvingerafdruk; de S08- en volumewerkbomen bleven schoon en op
exact dezelfde HEAD. `integration.json` vat alleen dit lokale bewijs samen.
GitHub controleert de actuele PR-head apart; afronding mag uitsluitend met
groene toepasselijke checks en zonder onopgeloste concrete blockers.
