# Licentiepakket voor de schone publicatiekandidaat

**Uitkomst:** het MPL-2.0-pakket is gereed voor opname in de private, controleerbare publicatiekandidaat: 861 expliciet genoemde eigen software-, bouw- en testbestanden, met behoud van de afzonderlijke voorwaarden voor externe code en fontgegevens. Dit rapport verandert geen live repository, zichtbaarheid of Git-geschiedenis.

## Controlebasis

De kandidaat is gebaseerd op `simonvanieperen/nieuw-os` op commit **363b486b06023d3d578f8eca2b23ff49045a1757**, aangevuld met de vastgelegde CI-herstelwijzigingen. De veranderingen ten opzichte van die bron staan in `docs/verification/publication-candidate-20261108.json`. De volledige `.agents/skills/os-dev/`-component is uit deze kandidaat weggelaten; zij wordt niet door de nieuwe projectlicentie gelegitimeerd. Het afzonderlijke private auditdossier (PR #259, `RIGHTS-REVIEW.md`) beschrijft het concrete oude wiki-herkomstgat en waarom de oorspronkelijke repositorygeschiedenis een andere publicatiescope heeft.

Deze controle richt zich op licentietoepassing en herkomst. De overige code-, privacy-, log-, geschiedenis- en workflowcontroles blijven bij de hoofdaudit. De inhoudelijke kandidaat is door de workflow-auditor samengesteld; de licentie-auditor heeft uitsluitend de hier beschreven aparte licentiebestanden en bewijsstukken geschreven.

## Vier bestanden voor de kandidaat

| Bestand | Functie |
|---|---|
| `LICENSE` | Volledige, ongewijzigde officiële Mozilla Public License 2.0 |
| `LICENSING.md` | Verbindt de MPL-notice aan de expliciete bestandslijst; begrenst software, derdenmateriaal en documentatie |
| `MPL-2.0-FILES.txt` | Exacte paden van de 861 bestanden waarop de projectlicentie wordt toegepast |
| `THIRD-PARTY.md` | Verwijst naar aanwezige componentnotices en benoemt de uitzonderingen, waaronder beide fontfamilies |

De licentietekst is rechtstreeks opgehaald van Mozilla op 8 november 2026:

`https://www.mozilla.org/media/MPL/2.0/index.txt`

- Omvang: **16.726 bytes**.
- SHA-256: **3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04**.
- Er is niets toegevoegd of verwijderd in de officiële tekst.
- De aanwezigheid van Exhibit B in die volledige tekst activeert niet de afzonderlijke melding *Incompatible With Secondary Licenses*; die melding is niet toegepast.

`LICENSING.md` en de precieze padenlijst houden de toepassingsscope zichtbaar zonder een nieuwe copyrightregel met onbewezen eigendom toe te voegen. De MPL staat een notice op een waarschijnlijke vindplaats toe als een bronheader niet wenselijk is; Mozilla beschrijft bronheaders wel als de voorkeursroute. Voor nieuwe zelfstandige softwarebestanden beschrijft het document daarom het toevoegen van een passende SPDX-notice en het bijwerken van de lijst (Mozilla, 2024, vragen 4 en 22; Mozilla, z.d., Exhibit A).

## Exacte softwareafbakening

Het register bevat **861 unieke, bestaande bestanden**. SHA-256 van `MPL-2.0-FILES.txt`:

**e28809974cb9cd12b4d408045b9820176c358c189e7cc2391603c63fa4f31675**

| Locatie | Bestanden in de MPL-lijst |
|---|---:|
| `boot/` | 7 |
| `crates/` | 44 |
| `drivers/` | 39 |
| `experiments/` | 34 |
| `kernel/` | 50 |
| `sdk/` | 283 |
| `services/` | 144 |
| `tests/` | 43 |
| `tools/` | 125 |
| `ui/` | 44 |
| `userspace/` | 22 |
| `.github/` | 23 |
| `.cargo/` | 1 |
| Root-`Cargo.toml` en `rust-toolchain.toml` | 2 |
| **Totaal** | **861** |

De tabel geeft aantallen, geen blanketlicentie voor de directories. De afzonderlijke paden zijn leidend. `public-candidate-scope-evidence.json` legt voor ieder opgenomen bestand de omvang en SHA-256 vast. Die hashes leggen de onderzochte versie vast; zij trekken de MPL niet in wanneer gedekte code later wordt gewijzigd.

De nieuwe uitvoerbare bestanden `tools/ci_evidence.py`, `tests/test_ci_evidence.py` en `.github/actions/preserve-evidence/action.yml` zijn opgenomen. De historische verifier onder `docs/verification/ci-artifact-storage-20261108/` is niet opgenomen, net als de rest van `docs/`.

Uitgesloten zijn alle vendor-, `third-party/`-, `licenses/`- en `verification/`-subtrees, de volledige documentatie- en agentinstructiebomen, lockfiles, datafixtures, afbeeldingen en overige binaire assets. Daarnaast is één belangrijk softwareachtig bestand expliciet uitgezonderd: de uit een externe fontbron gegenereerde `ui/graphics/src/atlas.rs`. De lijst is dus niet enkel op bestandsextensie toegepast.

## Fonts en ingebedde gegevens

| Materiaal | Behandeling en bewijs |
|---|---|
| `ui/graphics/src/atlas.rs` | **Buiten MPL.** De header en generator tonen dat glyphmetrics en rijoffsets uit vastgelegde Open Sans 1.10-bronnen worden gegenereerd. Het bestand verwijst naar de afgeleide atlas via `include_bytes!`. De bestaande Apache-2.0-fontvoorwaarden blijven gelden. |
| `ui/graphics/assets/open-sans-atlas.rle` | **Buiten MPL; Apache-2.0.** De SHA-256 van de 193.017 bytes komt exact overeen met `font-provenance.json`: `a4e3efa8027d5eba26bea742f7350bda34380cab9d77030d21481cb06c5f2e07`. |
| `ui/graphics/scripts/generate_font.py` | **In MPL.** Eigen generatiesoftware; de licentie op het programma verandert de externe fontlicentie van invoer en afgeleide uitvoer niet. De generator controleert beide bronfont-hashes. |
| Noto Sans Mono | **OFL-1.1 voor fontdata; MIT voor cratecode.** Eigen render-/testcode gebruikt de gepinde externe crate-API. De aangetroffen eigen aanroepcode bevat geen zelfstandig gekopieerde Noto-dataset die door de bestandslijst opnieuw wordt gelicentieerd. Beide notices staan onder `ui/native/licenses/` en `sdk/examples/compositor/licenses/`. |
| `crates/ui-core/src/font.rs` | **In MPL.** De bron verklaart expliciet dat de compacte 5×7-diagnostische glyphpatronen voor dit project zijn geschreven. Dit is positief herkomstbewijs uit de bron, geen onafhankelijke garantie dat iedere vorm uniek of auteursrechtelijk beschermd is. Er is geen externe fontbron voor deze tabel aangetroffen. |

De Open Sans-keuze is gebaseerd op de concrete gepinde versie 1.10 en haar bestaande metadata, niet op een veronderstelling dat iedere huidige distributie met die fontnaam dezelfde licentie heeft. `ui/graphics/THIRD-PARTY.md`, de volledige Apache-tekst en beide oorspronkelijke font-hashes blijven behouden.

De gerichte controle heeft ook `include_bytes!`- en `include_str!`-gebruik meegenomen. Eigen assembly en UI-tokens behouden hun projectscope. Ingebedde, tijdens de bouw geproduceerde ELF-bestanden behouden de voorwaarden van hun werkelijke componenten; de MPL-header van de omringende loader herlicentieert geen dependency of font. Externe assets en testgegevens krijgen geen nieuwe licentie door een verwijzing vanuit een MPL-bestand.

De exacte fontbestanden, hashes, Git-blob-ID's en controle-uitkomsten staan in `public-candidate-fonts-and-validation.redacted.json`.

## Externe software

De drie meegeleverde vendorpakketten zijn volledig uit de MPL-lijst gehouden: zune-jpeg 0.5.16-rc3, hkdf 0.13.0 en hmac 0.13.0. Bestaande volledige licentieteksten, upstreamherkomst en patchnotices blijven behouden.

De eerder uitgevoerde dependencycontrole omvatte 112 unieke registryversies in 62 lockfiles en 361 checksumvermeldingen. Alle checksumvermeldingen kwamen overeen met de officiële metadata voor de exacte pakketversie. De kandidaat bevat geen wijziging aan die dependencykeuze. Bijzondere gecombineerde voorwaarden voor `ring`, `unicode-ident` en Noto zijn benoemd in `THIRD-PARTY.md`; de volledige inventaris staat in `dependency-license-inventory.redacted.json`.

De bestaande bronverwijzingen bij Vigna's SplitMix64 en Hinnants `days_from_civil` blijven behouden. Hun oorspronkelijke publiek-domein-/vrijgebruiksverklaringen worden niet ingetrokken door de MPL-toepassing op de eigen integratiebestanden.

De bronpublicatie is geen aftekening van ieder toekomstig binarypakket. Bij verspreiding van een echte runner of release horen de notices van de daadwerkelijk opgenomen software, fonts en firmware. Dat is een concrete distributiestap en geen reden om een bronrepository zonder meegeleverde externe binaries automatisch tegen te houden.

## Auteurschapsbewijs en zijn grenzen

De GitHub-commit-API is bevraagd vanaf de exacte broncommit. De eerste pagina bevatte **98 bereikbare commits**; pagina's 2, 3 en 4 waren leeg. De oudste commit, `04bb3a388efd6b873177b6b3bc650f9ebfd19ba6`, is afzonderlijk gecontroleerd en heeft geen ouders. De telling is daarmee volledig voor de vanaf dit `main`-snapshot bereikbare commitmetadata.

| Waarneming | Uitkomst |
|---|---:|
| Auteur gekoppeld aan GitHub-account `simonvanieperen` | 96 commits |
| Auteur gekoppeld aan `coderabbitai[bot]` | 2 commits |
| Committer `web-flow` | 97 commits |
| Committer `simonvanieperen` | 1 commit |
| Afzonderlijke andere menselijke auteuridentiteit in de onderzochte auteurvelden of coauthortrailers | Geen aangetroffen |

De twee bot-auteurcommits zijn `8ad1bcf1851db75ca28bf1b7e511a59722d80e73` en `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`. De waargenomen coauthortrailers verwijzen naar het eigenaaraccount of benoemde AI-/automatiseringshulpmiddelen. Persoonlijke namen, e-mailadressen en volledige commitberichten zijn niet in het geredigeerde bewijs opgenomen.

Dit ondersteunt de bekende projectworkflow: ontwikkeling onder het eigenaaraccount met AI-ondersteuning. **Git-auteursvelden, squash-merges, botlabels en coauthortrailers zijn geen auteursrechtoverdracht.** Zij bewijzen niet dat alle eerder ingevoerde regels oorspronkelijk zijn. Er is geen retrospectieve CLA/DCO of overdrachtsakte voor iedere regel geverifieerd. Accountspecifieke AI-providercontracten en de onderliggende prompts zijn niet onderzocht. Dat AI betrokken was, levert op zichzelf geen garantie van originaliteit, afwezigheid van rechten van derden of exclusief auteursrecht voor Simon. Dit kan ik niet verifiëren.

De opdracht van de repository-eigenaar en het bevestigde MPL-uitgangspunt vormen de basis voor het voorbereiden van deze afgebakende toepassing. De toepassing kan uitsluitend rechten verlenen waarover de betrokken bijdrager beschikt; zij kan geen ontbrekende rechten van derden scheppen (Mozilla, z.d., §§ 1.9 en 2.1–2.5). In de onderzochte eigen software is naast de al benoemde externe componenten geen aanvullende concrete aanspraak van een onafhankelijke menselijke bijdrager gevonden die een apart herstelpunt oplevert. De beperkte identificerende reviewverwijzing in `sdk/heap-allocator/tests/arena.rs` noemt geen overdracht of externe bron; daaruit wordt geen onbekende externe licentie verzonnen.

Andere branches en historische PR-refs vallen buiten dit auteurschapsregister. Een schone export erft die niet. Het geverifieerde register met alle 98 commit-ID's staat in `public-candidate-authorship.redacted.json`.

## Eindcontrole en gebruik

De controle bevestigt: 861 unieke bestaande paden; alle bekende vendortrees, notices en documentatie buiten de lijst; Open Sans-metadata expliciet uitgesloten; eigen generator en de drie nieuwe CI-codebestanden opgenomen; atlas-hash passend bij de fontprovenance; officiële MPL-tekst ongewijzigd.

De vier bestanden kunnen nu gezamenlijk in de private publicatiekandidaat worden opgenomen. De verwijzingen in README, licentieplan en projectstatus moeten aansluiten bij dezelfde scope. Deze klaarstelling verleent geen toestemming voor het herschrijven van de bestaande geschiedenis en maakt de oude geïmporteerde wiki-teksten niet publiceerbaar. De concrete keuze tussen een schone publicatie en wijziging van bestaande repositoryscope blijft bij de hoofdtask.

## Bronnen

Mozilla. (z.d.). *Mozilla Public License, version 2.0*. https://www.mozilla.org/en-US/MPL/2.0/ — officiële tekstversie: https://www.mozilla.org/media/MPL/2.0/index.txt

Mozilla. (2024, 30 januari). *MPL 2.0 FAQ*. https://www.mozilla.org/en-US/MPL/2.0/FAQ/

Nieuw OS-bijdragers. (2026). *Repositorybron en componentnotices* (commit 363b486b06023d3d578f8eca2b23ff49045a1757) [Private repository, intern gecontroleerd]. https://github.com/simonvanieperen/nieuw-os/tree/363b486b06023d3d578f8eca2b23ff49045a1757

GitHub. (2026). *Commitmetadata voor simonvanieperen/nieuw-os* [API-dataset, geraadpleegd 8 november 2026]. https://api.github.com/repos/simonvanieperen/nieuw-os/commits?sha=363b486b06023d3d578f8eca2b23ff49045a1757&per_page=100&page=1

Crates.io. (2026). *Versiemetadata van 112 packages* [API-dataset]. De exacte versie-URL's en checksums staan in `dependency-license-inventory.redacted.json`. https://crates.io/api/v1/crates/
