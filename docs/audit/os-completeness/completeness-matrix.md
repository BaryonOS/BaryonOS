# Completenessmatrix

De machineleesbare bron is [`matrix.json`](matrix.json), gevalideerd tegen
[`matrix.schema.json`](matrix.schema.json). Eén rij is een samenhangende
voorziening/acceptatiecluster; niet elk subonderdeel uit de uitgebreide
productmatrix is een aparte auditrij. De gedetailleerde en bindende
release-eisen blijven de 62 `FR-*`-rijen in
[`FULL-TEST-RELEASE.md`](../../FULL-TEST-RELEASE.md).
De één-op-één bronrijlijst met relevante auditgroepen staat in
[`release-criteria-map.md`](release-criteria-map.md); alle 62 releasepoorten
blijven open.

## Dashboard

Gebruik de aantallen in [`verification.json`](verification.json); zij zijn uit
de definitieve matrix geteld. Status telt rijen, niet implementatiepercentages.
Bewijsniveaus worden per rij cumulatief genoemd. Meerdere niveaus in één rij
betekenen niet dat alle stappen op dezelfde bron of in één productimage zijn
gevalideerd.

## Domeindekking

| Gebied | Matrixrijen | Kernconclusie |
|---|---|---|
| Boot, kernel, geheugen, processen, IPC en diensten | `BOOT-UEFI`, `K-MEMORY`, `K-PROCESS`, `K-CAPABILITY`, `K-SERVICES` | Sterke eigen technische basis in begrensde QEMU-profielen; geen complete productkernel/release. |
| Drivers, opslag en herstel | `K-DRIVERS`, `STORAGE-VOLUME`, `STORAGE-CRYPTO`, `BACKUP-RESTORE`, `POWER-ENERGY` | Native deelproeven bestaan; fysieke ondersteuning, versleutelde gebruikersdata en onafhankelijke backup blijven open. |
| Desktop, sessie, Materiaal en gegevensuitwisseling | `DESKTOP-COMPOSITOR`, `WINDOW-MANAGEMENT`, `SESSION-AUTH`, `MATERIAL-EDITOR`, `CLIPBOARD-SHARE` | Native preview/bouwstenen bestaan; echte login en complete save/reopen/desktoproute zijn niet afgetekend. |
| Installer, instellingen, apps en platform | `INSTALLER`, `SETTINGS`, `APPS-UTILITIES`, `SDK-PACKAGES`, `UTILITIES` | SDK en beperkte apps bestaan; installer, pakketlevenscyclus en dagelijkse utilityset ontbreken. |
| Browser, media, notificaties en toegankelijkheid | `BROWSER`, `BROWSER-ENGINE`, `MEDIA`, `NOTIFICATIONS`, `ACCESSIBILITY`, `INTERNATIONALIZATION`, `VISUAL-DESIGN` | Prototype-/designwerk aanwezig; native web, media, a11y en brede i18n niet bewezen. |
| Netwerk, privacy, NAS, AI en quantum | `NETWORK-STACK`, `NETWORK-INTERNET`, `TLS-PEER`, `NETWORK-PERMISSIONS`, `NAS-CONTINUITY`, `PRIVACY-SECURITY`, `AI-LOCAL`, `QUANTUM-LOCAL` | Native L02 QSIM-jobservice is beperkt QEMU-geaccepteerd; de geïntegreerde FR-L04-regressie en overige native productdiensten blijven open. |
| Compatibiliteit en release | `COMPATIBILITY`, `UPDATES-RECOVERY`, `RELEASE-VALIDATION`, `GOVERNANCE` | Specificaties en losse gates; geen finale image, hardware- of releasevalidatie. |

## Bewijsclusters

- **Aantoonbaar sterkste native fundamenten:** eigen loader/kernel, paging,
  ring-3-processen, capability/IPC-primitives, virtio-blokpad en native
  update-/randomness-gates. Elk bewijs heeft een afgebakend profiel.
- **Code maar onvoldoende productbewijs:** compositor/sessie, Materiaal,
  appverificatie, netwerkdiensten en NAS-adapters. Source/testcode alleen is
  geen releasefeature.
- **Sterke specificatie, beperkte implementatie:** encrypted object storage,
  lokale AI, internet-/NASrechten, installatie, recovery, accessibility en
  compatibiliteit.
- **Onderscheidend productidee:** Materiaal als blijvend object met
  verwisselbare gereedschappen, herstelbare geschiedenis en expliciete
  capabilitygrenzen. Productonderzoek/design is geen gebruikerstest.
- **Disproportionele onderhoudslast:** eigen browserengine, codecdekking,
  brede hardwaredrivers, Windows-/gamecompatibiliteit en volledige
  postkwantum-/QPU-route. Hergebruikte, onderhouden componenten moeten vóór
  zelfbouw worden onderzocht.

Geen telling in dit document is een schatting van resterende ontwikkeltijd.
