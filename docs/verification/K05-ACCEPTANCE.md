# K05 — acceptatie algemeen proces- en dienstbeheer

Datum: 29 oktober 2026 (Europe/Amsterdam). Contract:
[K05-PROCESS-MANAGEMENT](../implementation/K05-PROCESS-MANAGEMENT.md);
besluit ADR-042. Bewijsmap: [`k05/`](k05/).

## Geaccepteerde eigenschap

Criterium K05: *niet-fixture-apps kunnen via begrensde API's starten,
afsluiten en diensten herstarten; afhankelijkheden en resources worden
correct vrijgegeven.* Op het eigen OS, in het QEMU-referentieprofiel, start een
ring-3-supervisor met uitsluitend `Launch`-capabilities een worker-dienst en
een client-app, brokert hun endpoints, herstart de dienst na een kill en na
een echte `#UD`-fault, en beëindigt alles; de kernel bewijst dat frames,
nodes, endpoints en objecten daarna op de baseline staan en dat kinderen van
een gestorven starter in dezelfde gebeurtenis worden beëindigd.

## Bronbinding

- Branch `ccr-43832ef7-gi2qlz`, basis `main` `8b60354`. De gate draaide op de
  werkboom die als PR-head is gecommit (broninputs identiek: de workflow
  herberekent dezelfde vingerafdruk); bronvingerafdruk v3 `01735e0379a2e646…` (SHA-256
  over de broninputs, geen Git-tree). De hosttestinputs `tests/test_supervisor_*.py`
  vallen buiten die vingerafdruk en zijn apart in `k05/supervisor-gate.json`
  vastgelegd en vóór het verdict opnieuw gecontroleerd.
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket
  `1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF 4M uit het
  Ubuntu-pakket, `NOS_QEMU_MACHINE=pc-q35-8.2`.
- SHA-256 van kernel-ELF en de drie gebruikersbinaries staan per stap in
  `k05/<n>-<case>/result.json` (`kernel_sha256`, `supervisor.*_sha256`).

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2
python3 -B -u tools/test_supervisor.py                       # exit 0
python3 -B tools/capture_supervisor.py                        # exit 0 -> docs/verification/k05/
```

De gate voert eerst vier hostcontroles uit en daarna vijf echte QEMU-runs, elk
met een eigen kernelfeature en een vers gebouwd image; het verdict `passed`
wordt pas na de afsluitende normale build geschreven en is gebonden aan de
bevroren bronvingerafdruk.

## Hostcontroles (exit 0)

| Controle | Resultaat |
|---|---|
| `cargo test -p capability-core` | 39 tests, waaronder 9 K05-tests: typering, geen derive/transfer/revoke van `Launch`/`Process`, levenscyclusrapportage alleen aan de starter, wezen van gestorven starters, precheck van delegatie, quota |
| `cargo test --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 17 tests: ABI 1.3, rechtenmasker 1023, canonieke statuswoorden, typed handles |
| `cargo test … -p nieuw-os-supervisor-example --lib` | 1 test: protocolframes |
| `python3 -m unittest discover -s tests -p 'test_supervisor_*.py'` | 7 tests: markervolgorde, failure-markers, exitcode/timeout, elk verwacht getal is dragend |

## QEMU-runs (alle raw exit 33, `SUPERVISOR:PASS`, `M1:PASS`)

| # | Geval | Feature | Waargenomen (kernelbewijs) |
|---|---|---|---|
| 0 | normal | `native-supervisor` | 3 spawns, 2 kills, 1 herstart, 2 worker-instanties, client 24 antwoorden met ≥1 `ECLOSED`, 1 kindexit, 0 faults, 0 wezen |
| 1 | crash | `test-supervisor-crash` | worker `#UD` (vector 6) na 8 antwoorden, `proc_status` = faulted, herstart, client 24 antwoorden, 1 kill, 0 wezen |
| 2 | denied | `test-supervisor-denied` | 17 van 17 negatieve controles met de vastgelegde foutcodes (9 supervisor, 6 client, 2 worker), ≥7 geweigerde spawns en ≥3 geweigerde kills aan kernelzijde, precies 2 gestarte processen |
| 3 | parent-exit | `test-supervisor-parent-exit` | supervisor eindigt met levende worker en client; kernel beëindigt beide als wezen in dezelfde gebeurtenis (2), 0 kills, 0 kindexits |
| 4 | quota | `test-supervisor-quota` | 15 geslaagde spawns, 16e `EQUOTA` zonder frameverandering, 16 gelijktijdige processen, 15 kills bevestigd als `killed` |

Elke run eindigt met een lege capability-engine (0 processen, handles, nodes,
endpoints, launch- en procesobjecten, gequeued berichten) en een vrije
framestand gelijk aan de baseline plus de permanente kernelpaginatabellen die
tijdens de run zijn aangemaakt (`SUPERVISOR:KERNEL_TABLES_GROWN`, 0 of 1).

De hostrunner controleert onafhankelijk van de kernel: exacte markervolgorde,
raw exit 33, afwezigheid van `FAIL`/`PANIC`, en per geval de verwachte
getallen (`tools/run_supervisor.py`, `EXPECT`/`AT_LEAST`).

## Negatieve tests

- Zonder `Launch`: `proc_spawn` met endpoint-handle (`ETYPE`), met leeg slot
  (`EBADHANDLE`); `proc_kill`/`proc_status` met endpoint- of launch-handle
  (`ETYPE`), met leeg slot (`EBADHANDLE`).
- Delegatie bij spawn: recht dat het endpointtype niet kent (`EINVAL`),
  bron zonder `TRANSFER` (`EACCESS`).
- Tweede kill van een dood kind (`ECLOSED`); status na sluiten (`EBADHANDLE`).
- `endpoint_create` voorbij 4 eigen endpoints (`EQUOTA`).
- Hosttests: `Process`-/`Launch`-handles weigeren `derive`, `revoke`,
  `bootstrap_delegate` en IPC-overdracht met `EINVAL`; een getal van proces B
  levert in A hoogstens A's eigen object op.

## Bewijsgrens

Niet bewezen: SMP, fysieke hardware, laden van gebruikersgeleverde images
(A02), persistente dienstconfiguratie, prioriteiten, bescherming tegen een
kwaadwillende supervisor binnen zijn eigen rechten, en gedrag buiten het
QEMU-referentieprofiel. Emulatortijd is geen hostlatentie. De GitHub
Actions-workflow `K05 process management acceptance` moet op iedere PR-head
groen zijn; zie PROJECT-STATE.md voor de run op de eindhead.
