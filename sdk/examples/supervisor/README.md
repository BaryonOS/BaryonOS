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

De afzonderlijke zesde case `wait` (`test-supervisor-wait`, ABI1.10)
combineert ditzelfde procesmechanisme met IPC-wachten. Workermodi4/5
registreren een echte lege-ingresswait met een deadline40/500 ticks.
De supervisor laat beide werkelijk parkeren, controleert dat een
Process-handle zonder DERIVE niet mag worden afgeleid naar OBSERVE en dat
een endpoint geen Process-killrecht is, beëindigt de eerste met KILL,
sluit diens handles en start een verse PID in hetzelfde slot. Oude status/
killhandles moeten dan `EBADHANDLE` geven. Na80 ticks, voorbij de oude
deadline maar vóór de nieuwe, moet ook de opvolger nog echt Waiting zijn
bij zijn geautoriseerde kill. Iedere onverwachte terugkeer uit de worker-
wait faalt expliciet; er wordt geen readiness of kill gesimuleerd.

Alleen dit expliciete proefprofiel leest bij de bestaande kill/spawn-
grens de werkelijke Waiting-status en gewiste wachtmetadata, zonder
capability-afdwinging, scheduler of cleanup te wijzigen. Het eindorakel
eist twee Waiting-kills, één schoon slothergebruik, vier echte blokkeringen,
twee ouder-deadlinewakes en volledig Engine-/frameherstel. De hostpoort
controleert die getallen onafhankelijk, inclusief werkelijk getelde idle-
IRQ's binnen de monotone tickteller. De vijf legacycases blijven verplicht;
`tools/test_supervisor.py` voert nu zes afzonderlijke QEMU-VM's uit.
Uitvoeringsstatus en brongebonden bewijs staan in
`docs/verification/native-test-version/`; deze beschrijving is geen native
acceptatie op zichzelf.

De eerste native wait-case faalde terecht op een onjuiste proefaanname:
K05 geeft Process uitsluitend `OBSERVE|KILL`, zonder `DERIVE`. De proef maakt
daarom geen OBSERVE-only-handle en verruimt geen rechten om dat mogelijk te
maken. Zij verwacht juist `EINVAL` voor die afleidingspoging en `ETYPE` voor
de endpoint-kill, naast beide stale-handleweigeringen.
`Engine::require` weigert DERIVE eerst als ongeldig voor het Process-
objecttype; het is niet slechts een ontbrekend toegestaan recht (`EACCESS`).
De tweede rode step104-run hield die foutcodes terecht uit elkaar.

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
