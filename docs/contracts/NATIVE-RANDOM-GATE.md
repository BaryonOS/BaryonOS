# Native RANDOM-integratiegate

Deze gate controleert de eigen ring3-RND1-client, geïsoleerde Key Service,
werkelijke virtio-rng en onderhouden ChaCha20Rng samen met de bestaande
init-/document-/volume-/cryptoketen. Het is een deelcriterium van
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md), geen gereedverklaring.
Het native contract staat in
[NATIVE-RANDOM-PROBE](../implementation/NATIVE-RANDOM-PROBE.md).

## Profielen en bediening

| `--case` | Kernelprofiel | Verwachte native case |
|---|---|---|
| `raw` | `native-system-random` | 0: variabele uitvoer, validatie, quota/reseed, rebind, Closed, revoke |
| `sdk` | `test-system-random-sdk` | 1: echte SDK-uitvoer en annulering na werkelijk enqueue |
| `unknown` | `test-system-random-unknown` | 2: expliciete onzekere caller-return ná werkelijk enqueue |
| `wrong-caller` | `test-system-random-wrong-caller` | 3: werkelijke ingress-consume en PIDweigering zonder RNG-uitvoer |

Bouw elk profiel op dezelfde bevroren bron met de gewone builder. Bijvoorbeeld:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/build.py --kernel-feature native-system-random
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/run_random.py --case raw
```

Gebruik een eigen `NOS_BUILD_DIR` en `NOS_TARGET_DIR` binnen de checkout,
het expliciet gekozen QEMU-machineprofiel en de daarbij vastgelegde OVMF-
bestanden. De gate accepteert alleen TCG, één qemu64-CPU,256MiB,geen netwerk,
geen hostshares of snapshots, één eigen wegwerpschijf en virtio-rng. De
QEMU-entropiebackend is de geïsoleerde virtuele tegenpartij; de probe kan
geen host-RNG of CPU-RNG aanroepen en krijgt geen EntropySource-capability.

Iedere case start een nieuwe normale schrijfproef op een lege1MiB-image
onder `BUILD/run`. Er is geen `--volume` om bestaande of persoonlijke inhoud
te kiezen. Maximumtimeout is180seconden. Er wordt geen vijfde readonly-RNG-
profiel verondersteld: de bestaande FR-I01-normal→koude readonly-recoverketen
blijft een afzonderlijk verplicht regressiebewijs. Deze gate bewijst geen
persistente sleutels of randomstate na koude herstart.

## Aftekening

`run_random` hergebruikt `run_system` en daarmee de echte native uitvoering,
rawQEMU-exit33, oorspronkelijke2seals/2unseals/9authweigeringen, recordproeven,
proceslifecycle/cleanup en onafhankelijke4096byte-schijforacle. Daarna worden
alle RANDOM-gegevens opnieuw uit de daadwerkelijk opgeslagen serial gelezen.
De expliciete volgorde, vaste20cijferige unsignedu64-tellers en precies één
RANDOM-PASS zijn verplicht. Onbekende, dubbele, ontbrekende, verkeerd
geordende of overlopende RANDOM-velden falen.

Afzonderlijke proefcijfers staan in de implementatiespecificatie. De gate eist
ook werkelijke, verschillende probe-/Key-/app-PID's en de daarbij passende
verwachte caller. Wrong-caller vereist ingressSeen1, badCaller1, nulcorecalls/
bytes/seeds/replies en minstens10ticks zonder antwoord ná de nieuwe STATUS-
roundtrip. Alleen stilte, een timeout of een succesvlag is geen weigering.
Cancel en uncertain vereisen de afzonderlijke juiste aflever-, sluit-,
output- en geenretrycijfers; reeds geënqueuede replies worden bij intrekking
niet voorgesteld als ongedaan gemaakte uitvoer.

Na de geobserveerde terminale RNG-handeling moet de gewone app met haar
bestaande enige KEY-sessie een **nieuwe echte STATUS-RPC** voltooien en Ready
ontvangen. De fase-ACK naar de probe volgt daarop. De gate controleert:

`probeStart <= RNGterminal <= STATUSbegin <= STATUSeinde <= probeEinde`

Een oude Ready-status, een heartbeat of alleen de oorspronkelijke cryptoproef
vervangt dit bewijs niet. Minstens twee echte legacypoll-returns na de RNG-
afhandeling zijn aanvullend vereist. De echte broker moet exact
`1024 + 256 × seeds` entropybytes leveren; STATUS vraagt geen extra entropy.

## Bron-, image- en bestandbinding

Voordat `random_gate=True` mogelijk is, controleert de gate:

- Bronvingerafdrukversie3 en de actuele geselecteerde manifestfeature;
  generieke normal-, net- of TCP-manifests zijn ongeldig.
- Precies zes geïnspecteerde systeem-ELFs in `manifest.system`, inclusief
  `random_probe`, met dezelfde aanvullende `manifest.random_probe`-entry.
  Hashes en nieuwe ELF-inspectie moeten overeenkomen met de
  werkelijk gebouwde bestanden; alle volledige ELFbytes moeten in de geladen
  kernel aanwezig zijn.
- De exacte kernel/loader in zowel build-ESP als de werkelijke QEMU-ESP,
  zonder onbekende bestanden, symlinks, hardlinked publicaties of specialfiles.
  Alleen read-only consumptie van echte Cargo-uitvoer laat Cargo's interne
  hardlinks toe.
- Het exacte QEMU-commandoprofiel, de geselecteerde executablehash, OVMF-
  codehash en preboot-VARStemplatehash. WritableUEFI-vars worden als actuele
  beheerde bytes gelezen, niet onjuist aan hun oude inhoud gelijkgesteld.
- Een werkelijk lege nieuwe image vóór de proef en exact dezelfde afterbytes
  in de livewegwerpimage en de aparte postrun-snapshot. De onafhankelijke
  on-diskparser moet het oorspronkelijke document met4096exacte fixturebytes,
  revisie1, naam `systeemproef` en echte SAVE-requestID3 bevestigen.
- Gelijkheid van de opgeslagen oracle/tellers met opnieuw gelezen bytes,
  eindcontrole op ongewijzigde bron, firmware en QEMU-executable.

Het resultaat behoudt `case=normal`, krijgt `random_case` en een aanvankelijk
onware `random_gate`. Iedere extra validatiefout maakt ook `passed=False`.
De actuele `random-report.json` wordt al vóór manifestcontrole en delegatie
atomair als `status=starting`, `passed=False`, `random_gate=False` gepubliceerd.
Een preflight- of launch-exceptie bewaart `status=failed` en de onware flags en
wordt doorgegeven aan de caller. Zo blijft een vroeger groen actueel indexrapport
niet staan na een nieuwe mislukte poging; historische per-runresultaten blijven behouden.
Een onveilig gelinkt rapportdoel wordt vóór publicatie geweigerd om externe
inhoud te beschermen. Ook de bron-ESP wordt vóór iedere shared copy/VM-delegatie
op exact de twee gehashte files en alle componenten/links gecontroleerd; de
controle van de gekopieerde ESP na uitvoering blijft aanvullend verplicht.
Resultaten worden atomair onder de eigen build-/runpaden opgeslagen als
`random-report.json`, `result.json` en `last-system-stage.json`. Deze laatste
kan door zijn andere feature geen normale FR-I01-coldrecovery autoriseren.
Het JSON-object is bewijsregistratie, geen ondertekende attestatie tegen een
kwaadwillende host die bron, binaries én alle bewijsbestanden vervalst.

De builderhook is rootintegratie: build de probe met
`CARGO_TARGET_DIR=TARGET/random-probe-app`, lever `NOS_RANDOM_PROBE_ELF` en
voeg de geïnspecteerde `random_probe` toe aan het bestaande systeemmanifest.
`run_system` accepteert deze vier features uitsluitend als normal; alle oude
profielen en hun eigen thresholds blijven behouden.

## Hostverificatie en open bewijzen

```sh
python3 -B -m unittest discover -s tools/tests -p test_random.py
```

De hosttests gebruiken expliciet synthetische logs/ELFs/wegwerpimages om
tegenvoorbeelden te bewijzen: timeout versus weigering, PIDmix, oude STATUS,
verkeerde profiel-/bronclaim, ontbrekende/dubbele cijfers, te veel output of
retry na annulering, gewijzigd ELF/ESP/serial/disk, snapshot-/netwerkcommando's
en externe symlink/hardlinkdoelen. Deze modelbestanden zijn geen VM-log of
bewijs dat randomuitvoer heeft gewerkt. Er wordt geen PASS-artifact voor een
niet-uitgevoerde VM ingevuld.

Vier echte VM-cases op de definitieve bevroren integratiebron en de bestaande
koude systeemregressies blijven noodzakelijk. Een volle replyqueue met
EAGAIN/deadline, defecte klok/entropiebron, echte account-/brokertoestemming,
TLS-identiteit/certificaattijd/PQC en fysieke entropiekwaliteit hebben hun eigen
open acceptatiecriteria. Deze gate tekent ze niet af en claimt geen volledige
geheugenerasure of volledige testrelease.
