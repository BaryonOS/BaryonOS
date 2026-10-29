# Native netwerkproef binnen het gezamenlijke systeemprofiel

Taakcontract, 29 oktober 2026. App/testpeer/runner: `network_nas_ai`;
kernel/capabilities/SDK-netwerk-ABI: `kernel_storage`; build-/manifestintegratie
en gezamenlijke acceptatie: hoofdagent. Schrijfgebieden zijn gescheiden.

## Werkelijk pad en profiel

Een vijfde ring-3-proces gebruikt ABI 1.5 en een manifest-only `NetworkSource`
om via de eigen kernelbroker en virtio-net-driver echte Ethernetframes te
versturen en ontvangen. ARP, UDP en DNS worden in de app door de bestaande
`no_std`-codec verwerkt. De normale init-, sleutel-, volume-, app- en worker-
scenario's blijven tegelijk actief. Dit is geen volledige netwerkstack/N01:
TCP, TLS, DHCP, algemene configuratie en de rechtenportal blijven open.

QEMU q35/TCG/één CPU/256 MiB, NIC `00:07.0`, MAC `02:00:00:00:00:07`.
Alleen een QEMU socketnetdev naar een tijdelijke `127.0.0.1`-testpeer; geen
TAP, bridge, WAN, echte NAS, persoonlijke inhoud of externe accounts.
Client-IP `192.0.2.2`; expliciete fixture-next-hop `192.0.2.1` met MAC
`02:00:00:00:00:01`; DNS-fixture `192.0.2.53` via die next-hop.
Geen MAC/IP-paar wordt als geauthenticeerde NAS-/TLS-identiteit voorgesteld.

## Interfaces en scenario's

Startup: netwerkcapability op offset 48, aparte endpoint-handle op 56 voor
verkeerde-type-weigering, eigen PID op 176. Geen init-IPC of nagebootst DONE:
de kernel wacht op de werkelijke exit en controleert eigen brokercounters.

- `net_send(source, frame, timeout_ticks)` publiceert en geeft één ticket;
  `net_poll_tx(source, ticket)` bevestigt alleen devicecompletion. Na publicatie
  wordt hetzelfde frame niet automatisch herhaald.
- `net_receive(source, output)` geeft één volledig gevalideerd frame of
  `EAGAIN`. Geen ruimte- of pointerfout mag een frame consumeren.
- Drie echte ARP-aanvragen: de peer antwoordt met bronheaderafwijking,
  verkeerd target en daarna geldig. De native app moet twee afwijzingen en
  ongewijzigde cache bewijzen vóór hij de juiste next-hop gebruikt.
- Eén UDP-echo naar poort 9000. De peer echoot het werkelijk ontvangen payload
  inclusief de door de app toegevoegde eigen PID; de app vergelijkt alle bytes.
- Drie DNS-aanvragen voor `native.nieuw.test`: eerst verkeerd transaction-ID,
  dan een compressioncyclus, daarna een geldig A-record `198.51.100.42`.
  Alleen de laatste mag een resolutieresultaat geven.
- Eén query voor `timeout.nieuw.test` krijgt bewust geen antwoord. De app
  annuleert na een vast receive-pollbudget; dit bewijst begrensd wachten,
  geen nog ontbrekende algemene systeemtijd-/resolver-timeout-API.
- Zeven exact gematchte raw-ABI-fouten: verkeerde captype `-6`, slechte
  TX-pointer `-3`, TX-lengte 1515 `-11`, slechte RX-pointer `-3`, RX-capaciteit
  1515 `-11`, ticket nul `-1`, na sluiten opnieuw gebruiken `-4`.
  SDK-, kernel- en driverrechten worden hiervoor niet versoepeld.

## Bewijs en acceptatie

De kernel observeert de app-private evidencewoorden 400 t/m 480 en vergelijkt
de eigen TX/RX/denial-/cleanupwaarnemingen. Verplicht verwacht:

| Counter onder `SYSTEM:` | Waarde |
|---|---:|
| `NET_ARP_REPLIES`, `NET_UDP_ECHOES`, `NET_DNS_ANSWERS` | ieder 1 |
| `NET_DENIED` | 7 |
| `NET_TX_COMPLETIONS` | 8 |
| `NET_RX_FRAMES` | 7 |
| `NET_ARP_REJECTS`, `NET_DNS_REJECTS` | ieder 2 |
| `NET_RX_TIMEOUTS`, `NET_CACHE_INTACT`, `NET_STALE_CAP` | ieder 1 |
| `NET_APP_DENIED` | 7 |
| `NET_TX_SUBMITTED`, `NET_TX_COMPLETED` | ieder 8 |
| `NET_RX_COMPLETED`, `NET_RX_DELIVERED` | ieder 7 |
| `NET_TX_TIMEOUTS` | 0 |
| `NET_RESETS`, `NET_RESET_ACKS` | ieder 1 |

De hostpeer rapporteert onafhankelijk de acht werkelijk ontvangen aanvragen
en zeven verstuurde antwoorden met packet-hashes en geparste vragen. Zijn
protocolrollen zijn uitsluitend tegenpartij en foutinjector: hij voert geen
client-ARP/DNS uit en schrijft nooit kernel-/appsuccesmarkers.

`run_net_system.py` hergebruikt de bewaakte `run_system`-image-, bron-, ELF-,
exit-, opslagorakel- en cold-recoverycontroles. Hij vereist daarnaast exact
netwerkbewijs en peertranscript. Een mislukte netwerkcontrole zet ook de
opslag-stageoverdracht op `passed=false`; zo kan recovery geen gedeeltelijk
geslaagde normale run als volledige voorganger gebruiken.

Hosttests toetsen de peer als parser/framer/testharnas, met onafhankelijke
wirefixtures, gefragmenteerde framing, checksums, lengtegrenzen en vijandige
invoer. Een targetbuild bewijst alleen compilatie. Akkoord voor deze slice
vereist echte native uitvoering van zowel normal als koude readonly recovery
op dezelfde wegwerpimage, met exacte commit/configuratie/imagehashes.

## Eerste daadwerkelijk uitgevoerde native pair

De kernelagent heeft op bronfingerprint
`355c111fdaeed5dd7e7f487cbd702fba362d7fcec1c1376db6b60b5f3bcd17d8`
beide echte QEMU-runs uitgevoerd: normal 54,47 s en koude readonly recovery
60,02 s, beide raw exit 33 en `network_gate=true`. De diskhash na normal en
vóór/na recovery is telkens
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.
Recovery doet nul writes en nul flushes. In beide runs zijn de echte acht
transmissies, zeven ontvangen frames, foutpaden en onafhankelijke
serverwaarnemingen gecontroleerd, naast de volledige bestaande systeemproef.

De werkelijk geteste probe-ELF heeft hash
`36a57a9163777e2b0db9c7fda3426f6c50dc80153fe23290e5b46bf48f470273`.
Een afzonderlijke build in een andere worktree heeft dezelfde appbron en
geladen code/data, maar andere rodata door absolute buildpaden. Dat is een
zichtbare reproduceerbaarheidsgrens; de andere ELF krijgt geen nativebewijs
van deze run. Path-remapping blijft een aparte bouwverbetering.

Deze pair gebruikte de toen bevroren gezamenlijke ontwikkelbron. De definitief
samengevoegde rootbron en het aangescherpte harnas moeten opnieuw native worden
uitgevoerd; oude logs of een hostherbeoordeling sluiten die integratiepoort niet.
