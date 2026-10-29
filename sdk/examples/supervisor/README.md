# K05-voorbeeld: supervisor, worker en client

Drie gewone `no_std`-programma's voor de Nieuw OS-ABI 1.3 (profiel
`native-supervisor`, ADR-042). Samen bewijzen zij dat een ring-3-proces met
alleen capabilities diensten kan starten, bewaken, herstarten en beëindigen.

| Programma | Autoriteit bij start | Gedrag |
|---|---|---|
| `k05-supervisor` | twee `Launch`-capabilities uit het bootmanifest | maakt een registratie-endpoint, start worker en client met een `SEND`-capability daarop, brokert de worker-ingress naar de client, bewaakt de worker met `proc_status`, herstart hem na een `proc_kill` (normaal) of een waargenomen `#UD` (crash) en beëindigt alle kinderen vóór zijn exit |
| `k05-worker` | één `SEND`-capability naar de registratie | maakt zijn eigen ingress-endpoint, registreert een `SEND\|TRANSFER`-capability daarop, beantwoordt genummerde verzoeken van de ene client die de supervisor noemt; in crashmodus `ud2` na acht antwoorden |
| `k05-client` | één `SEND`-capability naar de registratie | maakt een antwoordendpoint, stuurt 24 genummerde verzoeken en gaat na `ECLOSED` door met hetzelfde volgnummer zodra een nieuwe worker is gepubliceerd |

Het protocol (`src/protocol.rs`, lib `k05_protocol`) is een appafspraak in
gewone Message320-payloads; de kernel kent het niet en leest alleen de
bewijswoorden op de startpagina van de supervisor plus zijn eigen tellers.

Modi via startwoord 0 (laag byte) en de PID van de starter (vanaf bit 8):
worker `0` dienen, `1` crash na 8, `2` idle (quotaproef), `3` negatieve
controles; client `0` 24 verzoeken, `3` negatieve controles. De supervisor
leest zijn case uit woord 0/112: normal, crash, denied, parent-exit, quota.

```sh
python3 -B sdk/examples/supervisor/build.py
```

Standaarduitvoer: `build/sdk-supervisor/x86_64-unknown-none/release/k05-*`.
De kernelgate `tools/test_supervisor.py` bouwt deze binaries zelf via
`tools/build.py --kernel-feature native-supervisor` en varianten.

## Grenzen

Geen blok-, apparaat- of netwerkrecht; geen persistente dienstconfiguratie;
geen beleid voor wie supervisor mag zijn. De programma's zijn
acceptatiefixtures voor K05, geen productdienstbeheerder.
