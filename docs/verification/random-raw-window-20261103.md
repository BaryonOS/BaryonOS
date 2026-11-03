# RANDOM raw: begrensde fixturecorrectie

## Scope en bron

- Basis: `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e` (`origin/main` bij start).
- Werkbranch: `fix/random-raw-window-20261103`, eigen geisoleerde worktree.
- Alleen `sdk/examples/random-probe/src/{main,contract}.rs`,
  `sdk/examples/random-probe/tests/quota.rs` en dit ontvangstbewijs.
- Geen wijziging aan productiequota, entropy, rechten, keyring-service, kernel,
  runnerassertions, centrale status/score of de bevroren #156.
- Score blijft ongewijzigd; centrale voortgangsbestanden zijn eigendom van de
  integratiesessie en vallen expliciet buiten deze opdracht.

## Rood bewijs en oorzaak

Run `41671957624`, job `126042385200`, CI-head
`16eefda390cce7d71a35ddf35c6e58dcb44a3938`: de gast faalt op stap 15,
request 1064, expected count 0, status 0, bodylengte 17. Burst start 131,
eind 245, failure tick 247. De 64 sequentiele volledige RPC's duren 114 ticks,
terwijl het productiequotavenster 100 ticks duurt. Een success na vernieuwing
van het venster is toegestaan; dit is geen aangetoonde productquotafout.

`git diff 16eefda3 a32eadb1 -- sdk/examples/random-probe
services/keyring/src/random.rs tools/run_random.py
sdk/examples/keyring/src/service.rs` is leeg. Deze betrokken bronnen zijn dus
gelijk, zonder overige resultaten van beide branches te combineren.

De ongewijzigde lokale native raw-run slaagde: exit 0, QEMU-exit 33,
`gate=True`. Dat is nadrukkelijk **geen lokale rode reproductie**.
Het rode VM-bewijs komt uit de bestaande CI-run; de 114-tick-schedule wordt
daarnaast deterministisch als hostregressie nagebootst.

## Correctie en invarianten

Dezelfde requests 1000..1064, elk een byte, gaan in vier batches van 16.
Beide bestaande IPC-wachtrijen hebben capaciteit 16. Iedere batch wordt
volledig ontvangen voordat de volgende wordt verstuurd. De service behoudt
haar bestaande verwerking, scheduler en pending-replygedrag.

Iedere reply blijft strikt gecontroleerd op afzender, tag, capabilities,
request-ID, status, lengte en canonicalisatie; buffers behouden zeroizing.
Request 1064 moet nog steeds echt `Quota` opleveren met een lege body.
Daarnaast moet de volledige burst inclusief deze denial volgens de gastklok
strikt binnen de ongewijzigde 100 ticks vallen. Geen skip, retry, kunsttijd
of groter venster. Een te trage uitvoering blijft rood.

De hosttests controleren beide volledig gevulde echte capability-wachtrijen,
FIFO, ongewijzigde 64 succesvolle calls/64 bytes/een seed en de echte
65e-call-denial. De CI-schedule-test toont dat success na venstervernieuwing
nooit als quota-denial geaccepteerd wordt. Synthetische hosttijd en testentropy
zijn geen bewijs van native timing of entropykwaliteit.

## Eerste controles voor checkpoint

Werkdirectory:
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/random-raw-window`.
Omgevingsinstellingen staan buiten Git in `../random-environment.sh`.
Rust 1.90.0, bestaande geisoleerde GCC-sysroot, QEMU 10.2.1, OVMF 4M,
TCG, alleen wegwerpimages; geen installatie of fysieke schijf.

| Commando | Resultaat |
|---|---|
| `python3 -B tools/build.py --kernel-feature native-system-random` (ongewijzigd) | exit 0 |
| `python3 -B tools/run_random.py --case raw` (ongewijzigd) | exit 0, raw 33, gate True, 6,8 s |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/random-probe/Cargo.toml` | exit 0, zes tests |
| `python3 -B tools/build.py --kernel-feature native-system-random` (eerste correctie) | exit 0 |
| `python3 -B tools/run_random.py --case raw` (eerste correctie) | exit 0, raw 33, gate True, 6,6 s |

Eerste buildpogingen mislukten door ontbrekende hostlinker, daarna een
ontbrekende sysroot in een buildscript. Hersteld door de al aanwezige
`/tmp/nieuw-os-c-toolchain/cc-wrapper` te gebruiken; geen dependencywijziging.
Eerste nieuwe hosttestcompile gaf een usize/u64-verschil; gecorrigeerd.
Eerste Clippy-run wees een gewone assertion op constanten af; vervangen door
een compile-time assertion. Finale controles volgen hieronder.

Logs buiten Git: `build/random-window-evidence/`. Baseline native run:
`build/random-window-before/run/system-normal-61f0a4c9f0204216baa9a57eb08ed505/`.
Eerste gecorrigeerde run:
`build/random-window-after/run/system-normal-31a91166d513451f9753fa94c53e6efe/`.

## Open bij dit checkpoint

Finale format/Clippy, herhaalde brongebonden native gates en de andere drie
RANDOM-profielen worden nog gecontroleerd. De oorspronkelijke CI-run is niet
opnieuw uitgevoerd. Geen hardwaretest, onafhankelijke audit, merge of
releasevalidatie. Rollback: alleen deze fixturecommit terugdraaien; er is
geen productpolicy- of opslagformaatwijziging.
