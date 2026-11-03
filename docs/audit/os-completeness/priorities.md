# Prioriteiten

De labels volgen de gevraagde prioriteitsdefinities. P0/P1 zijn gates en geen
inschatting van benodigde tijd. Bij gelijkwaardige labels gaat data-integriteit,
Security Floor en afhankelijke fundering vóór zichtbare polish.

## Top P0/P1

| Rang | Prioriteit | Werkgebied | Waarom eerst | Afhankelijkheden / observable exit |
|---:|---|---|---|---|
| 1 | P0 | Geïntegreerde finale VM-FR-image en FR-R01/R02 | Zonder één productimage bewijzen losse suites geen coherent OS. | Alle geïntegreerde services; cold start/use/recovery op dezelfde hashes. |
| 2 | P0 | Native data safety: format admission, encrypted storage, keyring, backup/restore | Persoonlijke gegevens kunnen anders niet veilig worden opgeslagen/hersteld. | O08/O05/O06; cold recovery, tamper, failure en onafhankelijke bytes. |
| 3 | P0 | Auth/session/lock/revoke | Een previewgast is geen veilig profiel. | Account/keyring; echte success/failure/race/revoke. |
| 4 | P0 | Native netwerkbestemmingsrechten en TLS peer-authenticatie | Internet zonder native authority creëert onbeschermde trust boundary. | Driver/config/DNS/TLS; geobserveerde deny/revoke en negatieve peers. |
| 5 | P0 | Native NAS pairing/encrypted objects/offline recovery | Hostprototype bewijst geen privacycontract op het OS. | #2 en #4; twee native clients, ciphertext-only observatie, revoke/recovery. |
| 6 | P0 | Update + herstel als één transactie | Accepted B03 componenttest is niet de volledige gebruikersherstelroute. | Signed image, durable data, recovery; onderbreking in elke kritieke fase. |
| 7 | P0 | Drivers en één fysiek supportprofiel | QEMU is geen fysiek hardwareondersteuningsbewijs. | Exacte hardware, inputs/display/network/storage/power tests. |
| 8 | P0 | Native accessibility en echte menselijke taakproef | Huidige browser/semantiekchecks sluiten screenreader- of usabilitybehoefte niet. | UI tree/events, tester, keyboard-only/200%/reduced motion. |
| 9 | P0/P1 | Native Materiaal/editor (P0), settings en dagelijkse utilities (P1) | Dit vormt de dagelijkse gebruikswaarde van het product. | Storage/UI/SDK; saved/reopen, real content, error handling. |
| 10 | P1 | Browser, formats, app package lifecycle en media | Overstappen vraagt internet, documenten en verwisselbare tools. | Maintained engines, formats/licenses, grants and update ownership. |

### P2/P3/Research

- **P2 adoption:** betrouwbare NAS automatisch hervatten binnen bestaande
  grants; onboarding begrijpelijk maken; table/presentation/media family;
  notifications/focus; import/export van prioritaire documentformaten; lokale
  AI met geselecteerde context.
- **P3 differentiator:** Materiaalcontinuïteit tussen tools/apparaten en
  quiet-futurism design, pas na native fundering en menselijke evaluatie.
- **P4 later:** uitbreidingen die geen dagelijkse beta blocker zijn, na
  resource-/supportonderzoek.
- **RESEARCH:** regressie van de eerder geaccepteerde native L02 QSIM-interface
  op de geïntegreerde release-image (FR-L04 blijft open), fysieke QPU, CHERI,
  extra ISA's en formele verkenning. Deze nemen geen P0/P1-capaciteit weg
  zonder expliciet besluit.

## Uitstelregels

Geen P0/P1-rij verplaatsen naar “later” om een releasepercentage te verhogen.
Nieuwe feature work volgt de bestaande queue en matrix-ID's; deze audit
introduceert geen concurrerende implementatieroadmap.
