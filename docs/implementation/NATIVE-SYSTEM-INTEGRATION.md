# FR-I01 — gezamenlijke native systeemdiensten

29 oktober 2026. Basis `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`.
Dit is het taakcontract voor de eerste integratiestap van de volledige
testrelease. De volledige productacceptatie blijft afzonderlijk open in
[FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md).

## Invariant en resultaat

Eén eigen kernel draait gelijktijdig een ring-3-init, Key Service,
documentvolumedienst en gewone app. Een door init gestarte worker kan stoppen
en opnieuw starten. De app gebruikt echte capability-IPC, virtio-blok-I/O en
cryptografische verwerking door de Key Service. De kernel bewaart geen
inhoudssleutels en interpreteert geen documentformaat.

De bestaande profielen blijven regressieproeven. `native-system` is een
gezamenlijk integratieprofiel, nog geen volledige release, interactieve
desktop of algemene installatie van dynamische diensten. Het vastgelegde
256 MiB-regressieprofiel bewijst geen ondersteuning van het geplande
8 GiB-productprofiel.

## Eigenaarschap en interfaces

- Kernelworker: `kernel/**`, nieuwe `sdk/examples/system/**` en uitsluitend
  noodzakelijke generieke aanpassingen van bestaande serviceadapters.
- Integrator: build-/run-/testtools, workspaceleden en lockfiles, CI,
  acceptatiebewijs, projectstatus en voortgang. Eén schrijver per bestand.
- De bestaande Message320-/SDK-interface, volume-v6- en keyring-KEY1-wire
  blijven leidend. Blok- en entropy-capabilities behoren alleen toe aan de
  bijbehorende dienst; een handtekening of service-status verleent geen recht.
- Boot-ELF-invoer: `NOS_SYSTEM_INIT_ELF`, `NOS_SYSTEM_APP_ELF`,
  `NOS_SYSTEM_WORKER_ELF`, `NOS_VOLUME_SERVICE_ELF`,
  `NOS_KEYRING_SERVICE_ELF`. De build controleert ieder ELF afzonderlijk.

## Acceptatie vóór aftekenen van FR-I01

1. Start vanuit een nieuwe eigen bewaakte synthetische disk; beide echte
   diensten worden READY, de app slaat via de volumedienst op en leest terug.
2. De Key Service genereert met echte virtio-entropy, versleutelt en ontsleutelt
   een onbekende plaintext van de app. Verkeerd domein/manipulatie faalt.
3. De app krijgt geen directe blok-, entropy- of procesbeheerrechten;
   daadwerkelijke verboden syscalls worden geweigerd.
4. Init start en herstart een afzonderlijke worker; fout/exit wordt via het
   normale opruimpad afgehandeld. Alle proces-, capability- en frametellers
   keren na de eindige proef terug naar de vastgelegde baseline.
5. Het onafhankelijke Python-schijforakel controleert werkelijk opgeslagen
   bytes, revisies, geometrie en canaries. Serialmarkers alleen zijn onvoldoende.
6. De harnascontrole weigert time-out, foutmarker, ontbrekende/duplicaat- of
   ongeldige observatie, verkeerde raw exit, veranderde bron en foute diskstaat.
7. Herhaal de relevante supervisor-, keyring-, volume- en herstelregressies op
   de geïntegreerde bron. Nieuwe hosttests of compilatie vervangen deze niet.

## Open productgrenzen

Deze stap gebruikt nog de bestaande kleine proefgeometrie en RAM-sleutels.
Persistente beveiligde opslag, koude rollbackankering, algemene pakket-/
dienstenstart, volledige geheugen-/threadondersteuning, grafische sessie,
netwerk, apps en AI blijven verplicht open. Het profiel mag die functies niet
als hardware-afwezigheid of succesvolle simulatie voorstellen. De vaste score
stijgt niet door deze integratie alleen.

De nieuwe B02-contextrecords en persistente sleutelrecords krijgen eerst hun
eigen cryptografische en fouttests. Native wire-/opslagintegratie en een echte
vertrouwde rollbackankerbron blijven daarna noodzakelijk.

## Veiligheid en bewijs

Alleen nieuw gegenereerde files onder de aangewezen buildmap, private OVMF-VARS,
geen hostdisk, persoonlijke data, netwerkaansluiting of productiekeys. Bewijs
bevat broncommit en -vingerafdruk, tool-/firmwareversies en hashes, exacte
QEMU-opdracht, raw exit, serial, diskhash vóór/na en onafhankelijk orakel.
Deze tests leveren geen fysieke-hardware-, menselijke-audit- of productieclaim.
