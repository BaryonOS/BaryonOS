# Koppeling met bindende releasecriteria

Deze tabel koppelt alle 62 genummerde rijen uit
[`FULL-TEST-RELEASE.md`](../../FULL-TEST-RELEASE.md) aan de relevante groepen
uit [`matrix.json`](matrix.json). Titels volgen de bindende tabel. De koppeling
is auditnavigatie: zij wijzigt geen acceptatie-eis, bewijsgrens of eigenaar.
Alle rijen blijven **OPEN** totdat de volledige eis op de finale image is
afgetekend. Een gerelateerde matrixgroep met deelbewijs sluit de release-eis
niet.

| ID | Bindende gebruikstaak | Relevante auditgroepen | Releasepoort |
|---|---|---|---|
| FR-K01 | start het eigen OS | `BOOT-UEFI` | OPEN |
| FR-K02 | gebruik meerdere apps zonder elkaar te beschadigen | `K-MEMORY`, `K-PROCESS` | OPEN |
| FR-K03 | verleen, weiger en trek taakrechten in | `K-CAPABILITY` | OPEN |
| FR-H01 | bedien met toetsenbord, muis en ondersteunde USB-apparaten | `K-DRIVERS` | OPEN |
| FR-H02 | zie beeld en hoor geluid | `K-DRIVERS`, `DESKTOP-COMPOSITOR`, `MEDIA` | OPEN |
| FR-H03 | gebruik afgesproken verbindingen en apparaten | `K-DRIVERS`, `NETWORK-INTERNET` | OPEN |
| FR-S01 | richt eerste gebruik en lokaal profiel in | `SESSION-AUTH`, `INSTALLER` | OPEN |
| FR-S02 | vergrendel en ontgrendel de sessie | `SESSION-AUTH` | OPEN |
| FR-S03 | wijzig instellingen en vind ze na herstart terug | `SETTINGS` | OPEN |
| FR-S04 | herstart of sluit af met open werk | `SESSION-AUTH`, `BACKUP-RESTORE`, `UPDATES-RECOVERY`, `POWER-ENERGY` | OPEN |
| FR-S05 | hervat en gebruik energiebeheer volgens profiel | `POWER-ENERGY` | OPEN |
| FR-U01 | navigeer Materiaal, Bewerken, Gereedschap, Toegang, Apparaten en Assistent | `DESKTOP-COMPOSITOR`, `WINDOW-MANAGEMENT`, `MATERIAL-EDITOR` | OPEN |
| FR-U02 | selecteer, kopieer/plak, gebruik contextmenu en sleep | `CLIPBOARD-SHARE` | OPEN |
| FR-U03 | begrijp voortgang, annuleren en fouten | `NOTIFICATIONS`, `APPS-UTILITIES`, `UTILITIES` | OPEN |
| FR-U04 | gebruik een afgewerkte consistente interface | `VISUAL-DESIGN`, `WINDOW-MANAGEMENT`, `DESKTOP-COMPOSITOR` | OPEN |
| FR-U05 | bedien toegankelijk | `ACCESSIBILITY`, `INTERNATIONALIZATION` | OPEN |
| FR-A01 | maak, zoek en orden materiaal/bestanden | `MATERIAL-EDITOR`, `STORAGE-VOLUME` | OPEN |
| FR-A02 | schrijf en bewerk tekst | `MATERIAL-EDITOR`, `UTILITIES` | OPEN |
| FR-A03 | bekijk afbeeldingen | `UTILITIES`, `MEDIA` | OPEN |
| FR-A04 | lees PDF | `UTILITIES` | OPEN |
| FR-A05 | browse en download binnen verleende rechten | `BROWSER`, `NETWORK-PERMISSIONS` | OPEN |
| FR-A06 | speel audio af | `MEDIA` | OPEN |
| FR-A07 | speel video af | `MEDIA` | OPEN |
| FR-A08 | reken met de rekenmachine | `UTILITIES` | OPEN |
| FR-A09 | maak en bewaar een screenshot | `UTILITIES`, `DESKTOP-COMPOSITOR` | OPEN |
| FR-A10 | bekijk systeeminformatie, diagnose en Nederlandse hulp | `UTILITIES`, `INTERNATIONALIZATION` | OPEN |
| FR-A11 | installeer een app uit afgesproken bronnen | `SDK-PACKAGES` | OPEN |
| FR-A12 | werk apps bij en verwijder ze | `SDK-PACKAGES` | OPEN |
| FR-A13 | kies standaardgereedschap en open/bewaar samenhangend | `SDK-PACKAGES`, `MATERIAL-EDITOR` | OPEN |
| FR-A14 | maak en bewerk tabelmateriaal | `APPS-UTILITIES`, `MATERIAL-EDITOR` | OPEN |
| FR-A15 | maak en bewerk presentatiemateriaal | `APPS-UTILITIES`, `MATERIAL-EDITOR` | OPEN |
| FR-O01 | bewaar en vind gegevens na koude herstart | `STORAGE-VOLUME` | OPEN |
| FR-O02 | werk met bruikbare volumes en documenten | `STORAGE-VOLUME`, `MATERIAL-EDITOR` | OPEN |
| FR-O03 | organiseer metadata en gebruik geschiedenis/herstel | `STORAGE-VOLUME`, `MATERIAL-EDITOR` | OPEN |
| FR-O04 | herstel bij opslagfouten | `STORAGE-VOLUME`, `UPDATES-RECOVERY` | OPEN |
| FR-O05 | bescherm inhoud, metadata en sleutels | `STORAGE-CRYPTO`, `PRIVACY-SECURITY` | OPEN |
| FR-O06 | importeer/exporteer en maak herstelkopieën | `BACKUP-RESTORE` | OPEN |
| FR-O07 | zoek lokale inhoud en metadata | `MATERIAL-EDITOR` | OPEN |
| FR-O08 | initialiseer nieuwe opslag veilig | `STORAGE-VOLUME`, `INSTALLER` | OPEN |
| FR-N01 | configureer netwerk en maak verbinding | `NETWORK-INTERNET`, `NETWORK-STACK` | OPEN |
| FR-N02 | gebruik naamresolutie | `NETWORK-INTERNET` | OPEN |
| FR-N03 | gebruik geauthenticeerd beveiligd transport | `TLS-PEER` | OPEN |
| FR-N04 | sta netwerk per app/bestemming toe en trek in | `NETWORK-PERMISSIONS` | OPEN |
| FR-P01 | koppel eigen of gedeelde NAS | `NAS-CONTINUITY` | OPEN |
| FR-P02 | bewaar versleuteld op NAS en werk offline | `NAS-CONTINUITY`, `STORAGE-CRYPTO` | OPEN |
| FR-P03 | synchroniseer, hervat en los conflicten op | `NAS-CONTINUITY` | OPEN |
| FR-P04 | nodig uit, deel, beheer quota en trek ruimte in | `NAS-CONTINUITY` | OPEN |
| FR-P05 | voeg apparaten toe, roteer/herstel gebruikerssleutels | `NAS-CONTINUITY`, `STORAGE-CRYPTO` | OPEN |
| FR-P06 | kies optionele externe ciphertextopslag | `NAS-CONTINUITY` | OPEN |
| FR-L01 | activeer/verwijder lokale AI en kies een model | `AI-LOCAL` | OPEN |
| FR-L02 | gebruik AI met expliciet geselecteerde context | `AI-LOCAL`, `MATERIAL-EDITOR` | OPEN |
| FR-L03 | annuleer AI en controleer toegestane acties | `AI-LOCAL`, `NETWORK-PERMISSIONS` | OPEN |
| FR-L04 | behoud native lokale quantumfunctie | `QUANTUM-LOCAL` | OPEN |
| FR-B01 | vertrouw entropy en sleutelbescherming | `K-SERVICES`, `STORAGE-CRYPTO` | OPEN |
| FR-B02 | boot/verifieer alleen toegestane images | `BOOT-UEFI`, `PRIVACY-SECURITY` | OPEN |
| FR-B03 | update het systeem en herstel na mislukking | `UPDATES-RECOVERY` | OPEN |
| FR-B04 | behoud privacy en Security Floor | `PRIVACY-SECURITY` | OPEN |
| FR-B05 | laat beveiligingsgrenzen onafhankelijk controleren | `PRIVACY-SECURITY`, `RELEASE-VALIDATION` | OPEN |
| FR-R01 | doorloop het dagelijkse product als geheel | `RELEASE-VALIDATION` | OPEN |
| FR-R02 | start herhaald en werk langdurig gecombineerd | `RELEASE-VALIDATION` | OPEN |
| FR-R03 | installeer/herstel op ondersteunde fysieke configuraties | `RELEASE-VALIDATION`, `K-DRIVERS`, `INSTALLER` | OPEN |
| FR-R04 | reproduceer en beoordeel het complete testpakket | `GOVERNANCE`, `RELEASE-VALIDATION` | OPEN |

De introductietekst en historische vermeldingen van `FR-*` elders in de
releaseopdracht zijn geen extra rijen. De 62 bovenstaande IDs zijn de
genummerde tabelrijen; zij staan allemaal open.
