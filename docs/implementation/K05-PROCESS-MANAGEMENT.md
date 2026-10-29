# K05 — algemeen proces- en dienstbeheer (profiel `native-supervisor`)

Datum: 29 oktober 2026. Status: **ontwerpcontract, daarna geïmplementeerd
in dezelfde opdracht**; de uitgevoerde controles en hun uitkomst staan
uitsluitend in [K05-ACCEPTANCE](../verification/K05-ACCEPTANCE.md). Dit
document beschrijft het contract; afwijkingen die tijdens het bouwen nodig
bleken, staan in §8. Criterium (docs/progress.json): *niet-fixture-apps kunnen
via begrensde API's starten, afsluiten en diensten herstarten; afhankelijkheden
en resources worden correct vrijgegeven.* Besluit: ADR-042.

## 1. Probleem

Tot en met M6e/S06 maakt uitsluitend het ingebouwde kernelmanifest processen
en endpoints aan, vóór de eerste gebruikersinstructie. Een dienst die uitvalt,
komt niet terug; een app kan geen hulpdienst starten; er is geen
gebruikersruimte-autoriteit over levenscyclus. K05 voegt die autoriteit toe
als capabilities, niet als ambient recht.

## 2. Beveiligingsinvarianten (eerst)

1. **Starten is een capability.** Een proces kan alleen een image starten
   waarvoor het een `Launch`-capability bezit. Zonder die capability geeft
   `proc_spawn` `EACCESS`, `ETYPE` of `EBADHANDLE` en ontstaat geen proces,
   frame, endpoint of node.
2. **Geen erfenis.** Een kind krijgt uitsluitend de maximaal twee
   capabilities die de starter expliciet meegeeft, met verminderde rechten
   (dezelfde regels als IPC-overdracht: `TRANSFER` vereist, nooit `RECV`,
   nooit meer rechten dan de bron). De rest van de startpagina is nul.
3. **Kind-autoriteit is niet overdraagbaar.** Een `Process`-capability
   (observeren en beëindigen) kan niet via IPC worden overgedragen of
   afgeleid: `allowed_rights` kent voor dit objecttype geen `TRANSFER`,
   `DERIVE` of `REVOKE`. Alleen de starter beheert zijn kinderen.
4. **Dood ruimt alles op.** Exit, fault of `proc_kill` van een proces geeft
   al zijn frames en tabellen vrij, trekt alle door hem uitgegeven nodes in,
   sluit zijn endpoints (`ECLOSED` voor peers) en laat geen gequeued bericht
   met zijn capabilities achter. Dit is het bestaande `cleanup`-pad; K05
   voegt alleen de uitlokkers toe.
5. **Afhankelijkheid is expliciet.** Een kind hoort bij zijn starter. Sterft
   de starter, dan beëindigt de kernel in dezelfde gebeurtenis alle kinderen
   (en hun kinderen) iteratief, zonder recursie op de kernelstack. Er blijft
   geen proces over waarover niemand autoriteit heeft.
6. **Geen gedeeltelijke staat.** Alle fallibele controles van `proc_spawn`
   (handle, recht, vrij processlot, PID, engine-registratie, slots voor de
   meegegeven capabilities) gaan vooraf aan frame-allocatie; een fout na die
   controles doorloopt hetzelfde `cleanup`-pad als een procesdood, zodat de
   vrije framestand gelijk blijft.
7. **Quota blijven kernelquota.** Maximaal 16 processen, 4 eigen endpoints
   per proces, 64 endpoints, 1024 nodes, 256 slots. `EQUOTA` verandert niets.
8. **Een handle is geen geheim.** Een `Process`- of `Launch`-handle van
   proces B is in A hoogstens een eigen slot van A (bestaande KS-12-regel).

## 3. Interface (syscall-ABI 1.3, alleen in dit profiel)

| Nr. | Naam / argumenten | Succes (RDX) | Fouten |
|---|---|---|---|
| `0x30` | `endpoint_create()` | roothandle met `ALL_RIGHTS` | `EQUOTA` (4 eigen / 64 globaal / slots / nodes) |
| `0x50` | `proc_spawn(launch, arg, cap0, rights0, cap1, rights1)` | `Process`-handle met `OBSERVE\|KILL` | `EBADHANDLE`, `ETYPE`, `EACCESS` (geen `LAUNCH`, geen `TRANSFER` op cap, `RECV` gevraagd), `EINVAL` (onbekende rechten, `cap0 == 0` maar `rights0 != 0`), `EQUOTA` |
| `0x51` | `proc_kill(process)` | 0 | `EBADHANDLE`, `ETYPE`, `EACCESS` (geen `KILL`), `ECLOSED` (kind al dood) |
| `0x52` | `proc_status(process)` | `(state << 56) \| (code & (2^56 − 1))` | `EBADHANDLE`, `ETYPE`, `EACCESS` (geen `OBSERVE`) |

`state`: 1 = running, 2 = exited (code = exitcode mod 2^56), 3 = faulted
(code = exceptievector), 4 = killed. Een `cap0`/`cap1` van 0 betekent "geen
capability"; dan moet het bijbehorende rechtenveld 0 zijn. Ongebruikte
argumenten blijven volgens ABI v1 nul.

Nieuwe rechtenbits: `LAUNCH = 128` (alleen op `Launch`-objecten),
`OBSERVE = 256`, `KILL = 512` (alleen op `Process`-objecten). `KNOWN_RIGHTS`
wordt 1023. `derive`, `revoke` en IPC-overdracht van deze objecten falen met
`EINVAL` omdat het objecttype die operatierechten niet kent.

### Startpagina van een kind (`0x40010000`, 1024 bytes, rest nul)

| Offset | Inhoud |
|---|---|
| 0 | `arg` van de starter (vrij woord, geen autoriteit) |
| 8 | 0; het kind gebruikt dit als voortgangsteller (bestaande conventie) |
| 16 | kernelpatroon (bestaande audit) |
| 32 | 1 (startepoch) |
| 56 | handle van `cap0` in de tabel van het kind, of 0 |
| 64 | handle van `cap1`, of 0 |

Lege posities worden niet samengepakt: bij cap0=0 en geldige cap1 blijft56
nul en staat de gedelegeerde cap1 op64. Het aantal niet-lege capabilities
bepaalt uitsluitend de resourcepreflight. Dit bestaande contract is na de
P2-review van #51 met een echte native sparse-capregressie afgedwongen.
| 176 | eigen PID |

## 4. Kernelobjecten

- `Launch { owner, image }`: per manifest aangemaakt, `image` wijst naar een
  ingebouwde ELF (0 = worker, 1 = client). Niet door apps aan te maken.
- `Process { owner, child_pid, state, code }`: aangemaakt door `proc_spawn`,
  eigenaar is de starter. Bij dood van het kind zet de kernel `state`/`code`;
  het object blijft bestaan tot de eigenaar zijn laatste handle sluit. Bij
  exit van de eigenaar sluit de kernel het object en wordt het kind, als het
  nog leeft, beëindigd (invariant 5).

Het `Engine`-type in `capability-core` krijgt beide objectsoorten achter de
bestaande endpoint- en blokobjecten (`index ≥ 68`). Alle bestaande regels
(generaties, tombstones, `maintain`, `check_invariants`, `state_digest`)
gelden ongewijzigd; hosttests dekken de nieuwe overgangen.

## 5. Gebruikersprogramma's (SDK-voorbeeld `sdk/examples/supervisor`)

- **supervisor**: ontvangt bij het manifest twee `Launch`-capabilities en
  maakt zelf een registratie-endpoint. Start de worker en de client met
  een `SEND`-capability op dat endpoint. Worker en client registreren zich
  door een `SEND`-capability op hun eigen endpoint terug te sturen. De
  supervisor geeft de client de worker-capability, bewaakt de worker met
  `proc_status`, herstart hem (na `proc_kill` in het normale geval, na een
  waargenomen fault in het crashgeval) en publiceert de nieuwe capability
  opnieuw. Aan het einde beëindigt hij de kinderen en controleert `killed`.
- **worker**: dienst met teller; beantwoordt verzoeken met volgnummer en
  instantienummer. In het crashgeval voert hij na acht antwoorden `ud2` uit.
- **client**: stuurt 24 verzoeken; bij `ECLOSED` op de oude capability wacht
  hij op de nieuwe en gaat verder met hetzelfde volgnummer. Telt antwoorden,
  `ECLOSED`-waarnemingen en instanties.

Alle drie zijn gewone SDK-programma's zonder blok-, apparaat- of
netwerkrecht. De kernel kent hun protocol niet.

## 6. Acceptatiegevallen (echte QEMU-runs, elk een eigen kernelfeature)

| Geval | Feature | Vereiste observatie |
|---|---|---|
| normal | `native-supervisor` | start worker+client; gecontroleerde herstart na 8 antwoorden; client 24 antwoorden, ≥1 `ECLOSED`, 2 worker-instanties; alle kinderen `killed`/`exited`; frames en engine op baseline |
| crash | `test-supervisor-crash` | worker fault (`#UD`) na 8 antwoorden, `proc_status` = faulted(6); herstart; client 24 antwoorden; baseline |
| denied | `test-supervisor-denied` | client/worker proberen `proc_spawn`, `proc_kill` en `proc_status` met endpoint-, nul- en vervalste handles en `endpoint_create` voorbij het quotum; exact de vastgelegde foutcodes; de kernel telt geweigerde spawns en ziet precies 2 gestarte processen |
| parent-exit | `test-supervisor-parent-exit` | supervisor eindigt terwijl worker en client leven; beide worden in dezelfde gebeurtenis beëindigd; baseline |
| quota | `test-supervisor-quota` | supervisor start workers tot `EQUOTA` (15 geslaagd), vrije framestand ongewijzigd door de mislukte poging; alle 15 beëindigd en `killed`; baseline |

Kernelmarkers: `SUPERVISOR:MANIFEST:PASS`, `SUPERVISOR:CASE:<GEVAL>:PASS`,
`SUPERVISOR:RESOURCES:PASS`, `SUPERVISOR:PASS`, `M1:PASS`, exitcode 0x10
(raw 33). Elke `SUPERVISOR:FAIL:*` of `PANIC` faalt. Getalsbewijs als
`SUPERVISOR:<NAAM>:<20 cijfers>`. De gate `tools/test_supervisor.py` voert
eerst de hosttests uit (`capability-core`, SDK-runtime, Python-harness) en
daarna de vijf runs op één bevroren bronvingerafdruk.

## 7. Wat K05 niet bewijst

Geen SMP, geen echte hardware, geen laden van gebruikersgeleverde images
(A02), geen persistente dienstconfiguratie, geen prioriteiten of deadlines,
geen bescherming tegen een kwaadwillende supervisor die zijn eigen kinderen
misbruikt binnen de hem verleende rechten. De supervisor is één gewoon
proces; een systeembrede dienstbeheerder met beleid volgt later.

## 8. Afwijkingen tijdens implementatie

- **Frame-baseline en kernelpaginatabellen.** De identiteitsmap van nieuw
  aangeraakt RAM vraagt permanente `PageTable(0)`-frames (M2-contract). Bij
  16 gelijktijdige processen (quotaproef) groeide de kerneltabel met één
  frame. De eindcontrole eist daarom `vrij + gegroeide kerneltabellen ==
  baseline` en drukt de groei af als `SUPERVISOR:KERNEL_TABLES_GROWN`; een
  echt gebruikersframelek faalt nog steeds.
- **Denied-geval.** De client in controlemodus eindigt zelf na zijn rapport;
  de supervisor observeert `Exited(0)`, sluit de handle en controleert dat het
  nummer daarna `EBADHANDLE` geeft. De kernel verwacht dus één kill (worker)
  en één kindexit (client) in plaats van twee kills.
- **Startwoord 0** draagt naast de modus ook de PID van de starter (vanaf
  bit 8), zodat worker en client besturingsberichten alleen van hun
  supervisor accepteren. Dit is appprotocol, geen kernelautoriteit.
