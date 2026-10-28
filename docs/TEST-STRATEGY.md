# Teststrategie en bewijsstatus
Dit is een eigen testplan voor het project, geen verslag van al uitgevoerde OS-tests.

## N02 — onderscheid echte TLS, beleid en native bewijs

De uitgevoerde N02-integratie onder `verification/pqc-transport/` bevat 124 tests: 85 service inclusief 30 echte loopback-TLS-proeven, plus 39 UI-statustests. Handshakebewijs registreert de onderhandelde groep/protocol/cipher, wederzijdse identiteit, negatieve downgrade-/servergevallen en het ontbreken van sessiehergebruik. Policytests zijn geen TLS-bewijs; een configureerbare groepsnaam zonder handshake evenmin. Harnasaannames LAN/internet blijven onderscheiden van daadwerkelijk gemeten loopbacktransport. Test geen productiekeys of externe apparaten. App-/toestemmingsgeneraties voorkomen herleving van oude geauthenticeerde verbindingen na intrekken en opnieuw toestaan. Geen native/QEMU-, formeel-, menselijke-audit- of PQ-authenticatieclaim aan deze resultaten verbinden.

## Vanaf M1
Een geslaagde build, een geslaagde emulatorboot en functionele isolatie zijn aparte resultaten. Geef elke test een naam, requirement, exacte versie/configuratie, verwachte observatie, time-out en foutcriterium. Archiveer de relevante seriële uitvoer buiten de bronboom; leg in de repository vast hoe die te reproduceren is.

De bootrooktest moet slagen op een marker die alleen door de eigen kernel na overdracht wordt geschreven. Een loaderbanner is onvoldoende. De harness moet falen bij ontbrekende marker, onverwachte exit en time-out. Test ook dat de harness werkelijk een fout detecteert. Controleer latere stille hangs via voortgang/exit in plaats van een eindeloze QEMU-run.

## Testlagen
| Laag | Voorbeelden | Bewijsgrens |
|---|---|---|
| Host-unit/propertytests | Bounds, allocatiemodel, capability-transities, parsers | Geen bewijs voor bare-metal timing/MMIO |
| Cross-build/lint | Target, ABI, linker-layout, warnings, unsafe-inventaris | Geen bootbewijs |
| Emulator-integratie | Boot, exceptions, processen, syscall/IPC, crashherstel | Virtuele hardware is geen fysieke hardware |
| Fuzzing/fault injection | IPC, onbetrouwbare parsers, updates, disk full, uitval | Beperkt tot onderzochte inputs en uitvoertijd |
| Formeel model | Rechten kunnen niet spontaan groeien; revocation/lifetimes | Model moet met implementatie en aannames overeenkomen |
| Fysieke referentietests | DMA, firmware, slaapstand, energie en deviceherstel | Alleen exacte hardware-/firmwarecombinatie |
| Externe review | Invarianten, ontwerp en implementatie | Reviewscope/rest-risico's expliciet |

Gebruik tools als Miri/sanitizers alleen voor onderdelen/targets die zij ondersteunen; presenteer een hosttest niet als volledige kernelcontrole. Houd test-debugvoorzieningen buiten productieprofielen.

## Integratie-eisen per mijlpaal
- Geen gerelateerde regressie; bekende falende tests blijven zichtbaar.
- Negatieve test voor iedere nieuwe bevoegdheidsgrens.
- Test run en output horen bij de samengevoegde versie.
- Gemarkeerde mock/stub kan een interface verkennen, maar mag geen requirement afvinken.
- Geen test aanpassen zodat een bug groen wordt zonder requirementbesluit.
- Beperkte, reproduceerbare inputs; time-outs; geen live persoonlijke data.

## Bindende privacychecks
Na invoering van netwerk, NAS of AI gelden alle relevante LP-tests uit LOCAL-PRIVACY-CONTRACT.md. Test peer-identiteit, eenmalige uitnodigingen, expliciete remote toegang, encryptie van inhoud én namen, adminverwijdering zonder gebruikerssleutel, stale uploads na revoke, quota/herstart, netwerkovergangen, geen leveranciersfallback, netwerkloze inferentie, gescheiden modeldownloader, combinaties van apprechten en onafhankelijk herstel van NAS-backups. Dit zijn toekomstige acceptatiecriteria; nog geen uitgevoerde resultaten.

## Prestaties
Vaste machineconfiguratie, versies, workload en meetgrenzen. Rapporteer sampleaantal, mediaan en spreiding/p95 waar zinvol. Vergelijk hetzelfde werk met dezelfde beveiligingsgaranties. TCG-resultaten niet vergelijken met native Windows/macOS-cijfers. Stel realistische budgetten pas na een gemeten baseline vast.

## Supply chain en releases
Later: dependencies en checksums vastleggen, SBOM, repeatable builds in schone omgevingen, bronreview, ondertekende artifacts en hersteltest. Geen secrets in CI voor onbeoordeelde externe pull requests. Releasekeys buiten gewone agents/CI-buildjobs; aparte beperkte ondertekeningsstap. Zelfhosted runners voor onbetrouwbare PR-code moeten wegwerpbaar en geïsoleerd zijn.

Een CI-badge bewijst uitsluitend dat de geconfigureerde controles voor de genoemde versie zijn geslaagd. Geen OS-release publiceerbaar noemen voordat criteria, licentie, meldpunt, herstelroute en beperkingen zijn gecontroleerd.

