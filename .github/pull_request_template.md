<!--
Open de PR eerst als concept. Vul de relevante onderdelen in en verwijder
instructies/placeholders vóór afronding. Behoud concrete open punten.
Gebruik 'Closes #...' alleen als dit issue hiermee geheel wordt opgelost.
Geen secrets, productiekeys, echte gebruikersgegevens of ongeschoonde logdumps.
-->

## Doel en verandering

Welk concreet probleem is opgelost, wat verandert en welk gedrag levert dat op?
Link issue/discussie en relevante ADR of FR-/rubric-ID. Benoem resterend werk
wanneer deze PR slechts een deel van een issue uitvoert.

## Scope en grenzen

Vermeld relevante bronbestanden, interfaces en featureprofielen. Beschrijf
wijzigingen aan capabilities, IPC, proces-/drivergrenzen of opslagformaten,
met de te behouden invariant en negatieve controles. De eigen kernel en
Security Floor blijven leidend.

- [ ] Relevante `AGENTS.md`-bestanden en contractspecificaties gelezen.
- [ ] Nieuwe `unsafe`, dependencies of overgenomen code/assets: motivatie, herkomst, versie en licentie gecontroleerd; of expliciet niet van toepassing.

## Verificatie

Vul alleen werkelijk uitgevoerde controles als geslaagd in. Houd hosttests,
targetbuilds, native VM-proeven, fysieke tests en menselijke review apart.
Neem passende formatting/linting en positieve én negatieve tests op.

| Commando of gate | Geteste bron / profiel | Resultaat en exitstatus | Bewijs / run-URL |
|---|---|---|---|
| | | | |

**Niet uitgevoerd, geblokkeerd of niet van toepassing (met reden):**

Vermeld voor native bewijs de gebruikte image-/toolversies en het rapport met
hashes. Eerder bewijs blijft aan zijn oorspronkelijke bron gebonden. Een
groene cache, build of AI-review vervangt geen vereiste runtimeacceptatie.

## Bewijsgrens en rollback

Wat bewijst deze wijziging nog niet? Welke concrete risico's of open
reviewbevindingen blijven er, en hoe kan deze wijziging veilig worden teruggedraaid?

## Voortgang

Betrokken criterium/FR-ID (of geen), oude → nieuwe score en onderbouwing:

De integrator werkt de assessment in `docs/progress.json` en de bijbehorende
voortgangsweergaven **bij iedere opdracht bij, ook als de score gelijk blijft**.
Vermeld `python3 tools/progress.py --write` en `python3 tools/progress.py --check`
met hun resultaat in de verificatietabel. Alleen volledig behaald, passend
opgeslagen criteriumbewijs rechtvaardigt nieuwe punten; noteer de feitelijke
overdracht en open grenzen in `docs/PROJECT-STATE.md`.
