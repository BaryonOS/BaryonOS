# Licentie en reikwijdte

De eigen softwarebestanden van Nieuw OS die in
[MPL-2.0-FILES.txt](MPL-2.0-FILES.txt) zijn opgesomd, worden beschikbaar
gesteld onder de **Mozilla Public License, versie 2.0 (MPL-2.0)**. De
volledige, ongewijzigde licentietekst staat in [LICENSE](LICENSE).

## Bestanden waarop MPL-2.0 wordt toegepast

De afzonderlijke paden in `MPL-2.0-FILES.txt` vormen de exacte afbakening
voor deze bronpublicatie. Het gaat om eigen softwarebroncode, bouwscripts,
linkerscripts, testcode en geselecteerde bouw- en CI-configuratie in:

| Groep | Locaties |
|---|---|
| Loader, kernel en gedeelde code | `boot/`, `kernel/`, `crates/` |
| Drivers, services, SDK en gebruikersprocessen | `drivers/`, `services/`, `sdk/`, `userspace/` |
| UI-code en softwareproeven | `ui/`, `experiments/` |
| Bouw-, ontwikkel- en testcode | `tools/`, `tests/` |
| Bouw- en CI-configuratie | De in de lijst opgenomen Cargo-/toolchainbestanden en bestanden onder `.cargo/`, `.github/workflows/` en `.github/actions/` |

De tabel is een wegwijzer; zij verklaart niet ieder bestand onder deze
directories MPL-gelicentieerd. De exacte lijst en de uitzonderingen
hieronder zijn bepalend voor deze toepassing.

De volgende MPL-notice is aan ieder opgesomd bronbestand verbonden:

> This Source Code Form is subject to the terms of the Mozilla Public
> License, v. 2.0. If a copy of the MPL was not distributed with this
> file, You can obtain one at https://mozilla.org/MPL/2.0/.

Bij verspreiding van afzonderlijke bronbestanden moet de bijbehorende
licentievermelding worden meegeleverd. Voor nieuwe bronbestanden kan de
SPDX-vermelding `SPDX-License-Identifier: MPL-2.0` in de passende
commentaarvorm worden gebruikt.

De afbakening verandert niets aan de werking van de MPL zelf. Wijzigingen
aan gedekte bestanden en nieuwe bestanden die gedekte code bevatten,
blijven onder de relevante MPL-bepalingen vallen. Een nieuw, zelfstandig
bestand zonder MPL-code krijgt niet uitsluitend door zijn locatie een
licentie; voeg daarvoor een toepasselijke notice toe en werk de lijst bij.

## Externe code en assets

Externe onderdelen behouden hun eigen voorwaarden en notices. Zij worden
niet door deze projectlicentietoepassing opnieuw gelicentieerd. Raadpleeg
[THIRD-PARTY.md](THIRD-PARTY.md) en de bijbehorende componentregisters.

| Uitzondering | Behouden voorwaarden |
|---|---|
| `services/image-codec/vendor/zune-jpeg-0.5.16-rc3/` | De aanwezige upstreamlicenties: MIT, Apache-2.0 of Zlib; ook de gedocumenteerde wijzigingen behouden hun bestaande behandeling |
| `services/keyring/vendor/hkdf-0.13.0/` en `hmac-0.13.0/` | De aanwezige MIT-/Apache-2.0-voorwaarden en patchvermeldingen |
| Externe dependencybroncode en oorspronkelijke bronarchieven | Hun eigen pakket- en bestandslicenties; een lockfile herlicentieert een dependency niet |
| Open Sans-atlas `ui/graphics/assets/open-sans-atlas.rle` en de bijbehorende gegenereerde metadata in `ui/graphics/src/atlas.rs` | Apache-2.0, met de aanwezige herkomstregistratie en licentietekst; deze externe fontgegevens zijn uit de MPL-bestandslijst uitgesloten |
| Noto Sans Mono-fontgegevens | OFL-1.1; de betrokken Rust-cratecode heeft afzonderlijk MIT-voorwaarden |
| Bestanden in `third-party/`- en `licenses/`-directories | De daar bewaarde oorspronkelijke notices en licentieteksten |

De eigen generator `ui/graphics/scripts/generate_font.py` staat wel in de
MPL-bestandslijst. Die licentie op het programma verandert de licentie
van zijn externe fontinvoer en de daarvan afgeleide uitvoer niet. Eigen
code die Noto Sans Mono via de crate-API gebruikt, bevat daarmee geen
MPL-herlicentieverlening voor de afzonderlijke fontgegevens.

Bronvermeldingen bij vrijgegeven algoritmen, zoals SplitMix64 en
`days_from_civil`, blijven behouden. De MPL-toepassing op eigen integratie
of wijzigingen maakt het oorspronkelijk publiek-domeinmateriaal niet
exclusief eigendom van Nieuw OS en trekt bestaande rechten daarop niet in.

Een specifieke, bestaande externe licentievermelding wordt niet
vervangen door de algemene verwijzing naar `LICENSE`. Breng een eventuele
inconsistentie tussen het padregister en een specifieke bronnotice eerst
in kaart; verwijder de bronnotice niet.

## Documentatie, onderzoeksbronnen en overige bestanden

Dit document past MPL-2.0 **niet automatisch** toe op documentatie,
rapporten, gespreks- of bouwlogs, screenshots, ontwerpen, afbeeldingen,
lettertypen, datasets, testgegevens of andere bestanden die niet in
`MPL-2.0-FILES.txt` staan. Ook historische bewijsbijlagen onder `docs/`
vallen buiten deze toepassing.

Een eigen of externe afzonderlijke licentie blijft gelden waar die is
vermeld. Waar geen hergebruiklicentie is gegeven, verleent deze
MPL-afbakening geen aanvullende rechten. Zij beperkt geen rechten die
ontvangers al hebben, bijvoorbeeld op niet-auteursrechtelijk beschermde
feiten of publiek-domeinmateriaal.

## Verspreiding van software

De MPL staat ook commercieel gebruik toe. Haar voorwaarden gelden voor
de gedekte bestanden, met ruimte voor grotere werken waarin afzonderlijke
onderdelen andere licenties hebben. De naam, logo's en andere merken
krijgen hierdoor geen zelfstandige merklicentie.

Wie een binary met gedekte software verspreidt, moet de bijbehorende
gedekte broncode onder MPL beschikbaar stellen en ontvangers vertellen
hoe zij die redelijkerwijs kunnen verkrijgen. Houd daarnaast de
toepasselijke notices van werkelijk meegeleverde dependencies, fonts en
firmware bij. De publicatie van deze bronrepository is geen verklaring
dat ieder denkbaar releasepakket al aan zijn distributievoorwaarden voldoet.

Er is geen afzonderlijke keuze gemaakt voor de melding *Incompatible With
Secondary Licenses*. De aanwezigheid van Exhibit B in de ongewijzigde
volledige MPL-tekst betekent niet dat die melding op de software is
toegepast.

## Bijdragen

Lever wijzigingen aan MPL-gedekte software onder MPL-2.0 aan en vermeld
de herkomst van overgenomen materiaal. Behoud bestaande externe notices.
Wie een bijdrage indient, moet de rechten hebben om haar onder de
aangegeven voorwaarden te leveren. Voor bijdragen aan materiaal buiten
de hierboven afgebakende software moet de toepasselijke licentie apart
duidelijk zijn. Het project neemt geen auteursrecht van bijdragers over
uitsluitend doordat een bijdrage wordt ingediend of samengevoegd.

## Bronnen

Mozilla. (z.d.). *Mozilla Public License, version 2.0*, in het bijzonder
§§ 1.4, 1.10, 2.1–2.5, 3.1–3.4 en Exhibit A.
https://www.mozilla.org/en-US/MPL/2.0/

Mozilla. (2024, 30 januari). *MPL 2.0 FAQ*, vragen 4, 11, 13, 18, 22,
25 en 27. https://www.mozilla.org/en-US/MPL/2.0/FAQ/
