# Projectstatus — review en integratie van PR68 tot en met PR74

## Checkpoint van 30 oktober 2026

Dit checkpoint hoort bij branch `chatgpt/pr68-74-review-20261030`. De exacte
implementatiebron staat in de gegenereerde [voortgangsbeoordeling](PROGRESS.md).
De vaste productroadmap blijft **34/100**; criteria, gewichten en bestaande
acceptatiebewijzen zijn ongewijzigd.

| PR | Verwerking in deze opdracht | Bewijsgrens |
|---|---|---|
| #68 | Lokale Materiaal-semantiek, met herstelde venstergrenzen bij 200% schaal en regressietest | Hostproeven; geen nieuwe native toegankelijkheidsacceptatie |
| #69 | Gesloten negatieve rulesettest blijft uitgesloten | Opzettelijk ongeldige bron wordt niet geïntegreerd |
| #70 | Was al gemerged als `ca45a7da` | Bestaande DHCPv4-code blijft behouden |
| #71 | Is gemerged als `484ac33d`, inclusief drie gerichte reviewcorrecties | NAS-core, volume en resumeclient; exacte vorige-boot replay wacht op private export en startupbinding |
| #72 | IPC-enveloppen wissen via `zeroize`; opruiming en dependencybeschrijving gecorrigeerd | Bestaande securitygrenzen blijven gelden |
| #73 | Apppakketkern en beheerprotocol met beide categorieën antwoordvalidatie | Gereserveerde bytes blijven afgewezen; succesvolle antwoorden vereisen de juiste variant en een geldige revisie |
| #74 | Gedeeld update-sleutelbeleid met rotatie en intrekking | De 18 native B03-stappen zijn groen op bron `26f45321`; de samengestelde kandidaat krijgt eigen CI |

De integratiekandidaat bewaart de oorspronkelijke wijzigingen en historische
bewijzen. De verouderde centrale status uit #73 wordt niet teruggezet over
nieuwere voortgang: de volledige originele versies staan onder
[`history/integration-2026-10-30/pr73-dce3b05a/`](history/integration-2026-10-30/pr73-dce3b05a/).
De twee apppakketbesluiten krijgen ADR-055 en ADR-056, met verwijzing naar hun
oorspronkelijke nummers; bestaande TPM- en IPC-besluiten blijven behouden.

De apppakketwijziging omvat `app-package-core` en `sdk/app-management`, met
het wireprotocol, de transactiewachter en `app-management-quality.yml`.
De appbeheertests gebruiken fake backends; er is nog geen duurzaam journaal
of aangetoonde native installatie-/launchketen. De aanvullende kandidaat-
testresultaten worden brongebonden in de integratie-PR vastgelegd. De
GitHub-controles op de gepubliceerde head zijn op dit documentcheckpoint
nog een vereiste, geen reeds geleverd bewijs.

De lokale regressielogs horen bij de expliciete bron en bestanden in hun
manifest. Ze vervangen geen GitHub-controles op de uiteindelijke kandidaat.
Een merge vereist de actuele branchregels, de passende CI en afgehandelde
reviewdiscussies. Deze administratieve afhandeling sluit geen aanvullend
productcriterium en claimt geen native NAS-, E2EE- of toegankelijkheidsacceptatie.

## Historische projectstatus

De eerdere volledige main-status is ongewijzigd bewaard in
[`history/integration-2026-10-30/main-9a6703d3/docs/PROJECT-STATE.md`](history/integration-2026-10-30/main-9a6703d3/docs/PROJECT-STATE.md).
Dit huidige checkpoint verwijst naar dat archief in plaats van de oude
checkpointreeksen als actuele projectstatus te presenteren.
