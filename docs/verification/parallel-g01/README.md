# G01 — SDK-bron, hostcontroles en native compilatie

28 oktober 2026. Uitvoeringsbron
`89b352f8cf246794d18a783373f1dcedc8de8bd3`, basis
`1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`, branch
`codex/parallel-native-sdk-20261028`.

## Geleverde functie

- Eigen allocatorvrije app-runtime voor de bestaande int80-ABI 1.0–1.2:
  getypeerde handle-/endpointwrappers, niet-blokkerende IPC, capsluiten,
  rechtenvermindering, intrekking, yield en exit.
- Canonieke Message320, gecontroleerde lengtes en gereserveerde bytes,
  afzonderlijke kernelretourvalidatie en behoud van de oorspronkelijke
  antwoordbytes voor diensten. Geen automatisch opnieuw verzenden van RPCs.
- Een native gecompileerde catalogusleesapp gebruikt echte LIST/GET-berichten
  en vereist bestaande, expliciet gedelegeerde M6c-endpoints. Zij voegt geen
  algemene bootstrap of appstarter toe.

De bron wijzigt alleen `sdk/`. Bootprotocol, syscallnummers, capabilities,
cataloguswire, journal, bestaande probes en gedeelde buildconfig blijven gelijk.

## Uitgevoerd op deze bron

De [opname](report.json) bevat per opdracht echte argv, cwd, exitstatus,
stdout-/stderrbestanden en SHA-256. Alle zeven geregistreerde controles
slagen met exit 0; gecontroleerde broninputs blijven gedurende de run gelijk.

| Controle | Resultaat |
|---|---|
| Hosttests | 13/13; alle 257 payloadlengtes tegen de bestaande kernelparser, beschadigde envelopes, fouten, daadwerkelijke pure Engine-IPC/intrekking |
| Host Clippy | Alle SDK-targets, `-D warnings`, exit 0 |
| Rustfmt | Alleen eigen Rust-bestanden; exit 0 |
| Native runtime Clippy | `x86_64-unknown-none`, `-D warnings`, exit 0 |
| Native app Clippy | `x86_64-unknown-none`, `-D warnings`, exit 0 |
| Native app build | Release-ELF, 26.776 bytes; eigen app-linkerscript |
| Diffcontrole | `git diff --check`, exit 0 |

De bestaande `tools/build_users.py::inspect_user` accepteert het ELF:
entry `0x40000000`, afzonderlijke RX/RW/RO-segmenten, geen pagina-overlap,
passende useradresgrenzen en de bestaande startupdatapagina. Binary SHA-256:
`bbd131bed07519b72fb9465d28998868c2b7e17cb41d0df359f9ae0c44a159ea`.
De binary is een reproduceerbaar buildresultaat en wordt niet in Git opgeslagen.

Een vroege ontwikkelrun had al 13 geslaagde hosttests maar miste `rustdoc` in
PATH; de volledige opdracht was toen exit 101. De finale opname gebruikt de
werkende toolomgeving. De vroege native Clippy wees twee oude modulo-
uitdrukkingen in de app-bootstrap af; deze zijn vóór bovenstaande broncommit
vervangen door `is_multiple_of`. Deze diagnostiek wordt niet als extra
geslaagde eindtest geteld.

## Reproduceren

Met Rust 1.90.0, rustfmt/Clippy, de `x86_64-unknown-none`-target en Python:

```sh
export CARGO_TARGET_DIR=/tmp/nieuw-os-g01-reproduction
python3 -B docs/verification/parallel-g01/verify.py
```

De opname overschrijft uitsluitend haar eigen report/logbestanden; voor een
nieuwe bewijsopname eerst een eigen checkout gebruiken. De SDK en app hebben
afzonderlijke [API-uitleg](../../../sdk/runtime/README.md) en
[bouw-/startgrenzen](../../../sdk/examples/catalog-read/README.md).

## Open grenzen en roadmap

Dit bewijs voert geen VM uit. Het bewijst nog geen native SDK-syscall,
zelfstandige voorbeeldappstart, processupervisie of nieuwe Materiaal-handeling.
De rootintegratie kan de SDK in de bestaande M6c-probe opnemen en de echte
catalogus-/virtio-route afzonderlijk controleren. Eventueel nieuw native bewijs
krijgt zijn eigen bronbinding en wordt niet aan deze host-/buildrun toegeschreven.

De voor A01 vereiste gewone appstarter, complete SDK-contractacceptatie en
native sandbox-/foutproeven blijven open. U03 blijft eveneens open. Voorgestelde
roadmapwijziging: **14% → 14%**, zonder aangepaste criteria of gewichten.
