# N04 — native intrekking van een per-app netwerkrecht — 5 november 2026

**Bron:** `7bedac0bf0939220a13445f0c7067b2a033e6b70` op
`net/n04-native-revoke-20261105` (main `cb3d169c` ingemerged), schone werkboom.
**Status:** N04 blijft **open**. Dit is een bewijsvoorstel voor één
afgebakende stap, geen acceptatie en geen scorewijziging (41/100 → 41/100).

## Wat nu native werkt

In het opt-in profiel `test-net-service-revoke` draaien vier echte, geïsoleerde
ring-3-processen: de netwerkdienst (enige NIC-eigenaar), app A, app B en een
**beleidsproces**. Het beleidsproces heeft precies één capability: SEND op het
privé controle-endpoint van de dienst. Het heeft geen NIC, entropy of
app-endpoint; de kernel controleert dat bij de bootstrap, en het proces bewijst
zelf met een geweigerde NIC-syscall dat het geen netwerktoegang heeft.

1. App A opent een TCP-verbinding binnen zijn eigen grant (8080), stuurt een
   HTTP-verzoek en houdt de verbinding live. De peer antwoordt bewust niet.
2. De kerneltestdriver ziet dat app A live is en geeft het beleidsproces het
   startsein. Het beleidsproces stuurt `NETRVK01 ‖ epoch ‖ slot`.
3. De dienst accepteert dat alleen van de kernel-geauthenticeerde beleids-PID,
   met de juiste tag, lengte en epoch. Daarna roept hij `Service::revoke` aan:
   de verbinding wordt afgebroken (RST op de lijn), gebufferde en nieuwe UDP
   voor app A wordt weggegooid, en elke latere open-/data-operatie van app A
   krijgt `Access`. Alleen State, CloseTcp en Retire blijven beschikbaar voor
   opruimen.
4. App A ziet `Closed` binnen 500 ticks. De verbinding heeft een timeout van
   3000 ticks, dus alleen de intrekking kan haar hebben beëindigd. Daarna
   worden vijf vervolgacties geweigerd: send, receive, een nieuwe open, UDP
   verzenden en UDP ontvangen.
5. **Helperroute:** app A probeert de bestemming van app B (8090) en krijgt
   `Access`. Een grant is gebonden aan de kernel-PID; een ander proces kan hem
   niet gebruiken.
6. App B doorloopt intussen zijn volledige N01-scenario (HTTP, weigering,
   reset, retransmissie, timeout, UDP) en wordt niet geraakt.

## Waargenomen via verkeer (host-peer, causaal transcript)

- Precies één abort-RST|ACK van app A op `client_next`/`server_start`. Komt de
  ACK van de peer pas ná de abort binnen, dan antwoordt de gesloten tuple
  RFC-conform met één kale RST (`SEQ=SEG.ACK`). Elke vorm is maximaal één keer
  toegestaan. Vóór de wachttijd van 10 ticks (zie Reviews) kwamen beide vormen
  in elke run voor. Met die wachttijd zag elke run van `7bedac0b` alleen de
  abort-RST|ACK.
- Na de intrekking komt er geen enkele SYN, data, FIN of UDP meer van app A.
  Elk zo'n frame laat de peer gesloten falen.
- **Geen onbedoelde luisterpoorten:** een inkomende SYN naar gastpoort 22 en
  443 (vóór de intrekking) en naar app A's afgebroken lokale poort 40000 (ná
  de intrekking) krijgt telkens precies één RST, nooit een SYN-ACK.

## Causale volgorde op één klok

Elke rol schrijft op offset 552 een absolute boot-monotone tick. Userspace
`monotonic_ticks` is exact `state.ticks` van de kernel. Kernel en runner eisen
beide: `verzoek ≤ beleid verzonden ≤ dienst ingetrokken ≤ app A ziet Closed`.
Het beleidsproces meet zijn tick vóór de send. Daardoor kan preëmptie na de
send de volgorde niet vervalsen. Gemeten waarden staan per run in
[`gates.json`](gates.json). Een voorbeeld is 17 ≤ 17 ≤ 17 ≤ 19; van 'ready'
tot `Closed` duurde het telkens 2 ticks. Het intrektijdstip van de dienst wordt
ná de ontvangst van het bericht gemeten, niet aan het begin van de lus.

## Uitgevoerde controles (bron `7bedac0b`)

| Gate | Resultaat |
|---|---|
| `services/network-service` tests (1.91.1) | 12/12, waarvan 2 nieuw |
| Clippy `-D warnings`, fmt, baremetal `x86_64-unknown-none` | exit 0 |
| `network-service/protocol` tests / `sdk/net-client` tests | 6/6 / 4/4 |
| Peer- en serialtegenvoorbeelden (`test_network_service_peer.py`) | 17/17, waarvan 3 nieuw |
| Native `normal` (N01-regressie) | geslaagd, raw 33 |
| Native `client-exit` (N01-regressie) | geslaagd, raw 33 |
| Native `revoke` | 5/5 geslaagd, raw 33, telkens ~7 s |

De commando's staan in `.github/workflows/net-broker-quality.yml`; de revoke-case
is daar toegevoegd naast normal en client-exit. Lokaal draaiden ze op QEMU
10.2.1 (`pc-q35-8.2`, TCG, 1 vCPU) met OVMF. Opgeslagen bestanden: `gates.json`
(alle runs, bron- en stdout-hashes), het volledige rapport, de serial en het
peer-transcript van de laatste revoke-run, plus de testuitvoer. Hashes staan in
`SHA256SUMS`.

## Reviews

Er zijn twee AI-reviews gedaan. Dat is geen menselijke audit.

- De eerste, onafhankelijke beveiligingsreview vond niets blokkerend. Drie van
  haar opmerkingen zijn verwerkt: de causaliteit met tijdstempels, de probe op
  de afgebroken poort, en de doc-comment over een al uitgegeven frame.
- De tweede review was adversarieel, in drie lenzen: autoriteit,
  bewijssoliditeit en regressie. Elke serieuze bevinding is door twee sceptici
  getoetst.
  - Eén bevinding bleef staan (4/4 bevestigd): de dienst gebruikte het tick van
    het begin van de lus als intrektijdstip. Na een preëmptie kon
    `REVOKE_ORDER` dan onterecht falen. Dat gaf een valse fout, geen valse pass.
    Hersteld in `7bedac0b`.
  - Eén nit is ook verwerkt: app A meldde 'ready' al toen de bytes alleen in de
    sendbuffer stonden. Nu wacht app A eerst 10 ticks; dit is een tijdelijke
    oplossing, gemarkeerd in de code.
  - De autoriteitslens vond niets.

## Open grenzen (waarom N04 open blijft)

- **Beleidsbron:** het startsein komt van een kernel-testdriver. Er is geen
  gebruikersgerichte toestemmings- of instellingenroute, en geen
  productie-issuer voor grants. Intrekken werkt alleen in één richting per
  epoch; opnieuw toekennen met een generatie ontbreekt nog.
- **Reikwijdte:** er zijn twee vaste apps met statische bestemmingen en een
  synthetische localhost-Ethernet-peer. Er is geen DNS of naamgebaseerde grant
  (N02), geen TLS (N03), geen fysieke NIC en geen dagelijks internet.
- **Helperroutes:** alleen gedekt als "een ander proces kan de grant van app A
  niet gebruiken". Een echte gedelegeerde helper met een eigen, beperkte
  subgrant bestaat nog niet.
- **Een al uitgegeven frame:** een frame dat al aan het device was overgedragen,
  blijft tot zijn echte completion staan. Dezelfde semantiek geldt bij
  `retire_slot`; in deze proef komt het niet voor.
- **UDP-teller:** UDP voor een ingetrokken eigenaar telt mee in
  `malformed_frames`. Een aparte beleidsteller ontbreekt nog.
- Geen menselijke beveiligingsaudit (B05) en geen fysieke hardwaregate.
