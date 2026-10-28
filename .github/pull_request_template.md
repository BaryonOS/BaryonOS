## Doel

Beschrijf het concrete probleem en de kleinste wijziging die het oplost.

## Architectuur- en securitygrens

- [ ] Relevante `AGENTS.md`-bestanden gelezen.
- [ ] Geen bestaande OS-kernel als basis toegevoegd.
- [ ] Capability-/privilege-/IPC-/drivergrenzen niet ongemerkt verruimd.
- [ ] Nieuwe `unsafe` of dependency is expliciet gemotiveerd en gecontroleerd.

## Verificatie

Vermeld exacte commando's en resultaten.

- [ ] Relevante unit/integratietests
- [ ] Formatting/linting
- [ ] Native build indien van toepassing
- [ ] Negatieve test voor nieuwe trust- of bevoegdheidsgrens
- [ ] Niet-uitgevoerde checks expliciet als zodanig genoemd

## Bewijsgrens en rollback

Wat bewijst deze wijziging niet, en hoe kan ze veilig worden teruggedraaid?

## Voortgang

- [ ] `docs/progress.json`/README alleen aangepast wanneer de vaste acceptatiecriteria dit rechtvaardigen.
