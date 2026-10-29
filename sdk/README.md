# Nieuw OS appbouwstenen

Deze eigen workspace bevat begrensde libraries voor gebruikersprocessen.
Zij veranderen geen kernel-ABI, capabilityrechten of opslagformaat.

| Onderdeel | Functie |
|---|---|
| [runtime](runtime/README.md) | Allocatorvrije `no_std`-wrappers voor de bestaande int80-ABI, Message320 en getypeerde handles. |
| [catalog-client](catalog-client/README.md) | B01-catalogusclient over de runtime, met begrensd wachten, canonieke antwoorden en geen replay na publicatie. |
| [document-session](document-session/README.md) | Bestaand document openen, lokaal bewerken, CAS-bewaren en expliciet heropenen; conceptbehoud bij conflict of onzekerheid. |
| [catalog-read](examples/catalog-read/README.md) | Compileerbaar native voorbeeld voor LIST → GET via de getypeerde catalogusclient. |
| [supervisor](examples/supervisor/README.md) | K05: ring-3 supervisor, herstartbare worker en client over de spawn/kill/status-ABI 1.3; de acceptatieprogramma's van het profiel `native-supervisor`. |

G01/G03 bewijzen veilige hostvalidatie en native compilatie. De afzonderlijke
G05-suite bewijst de runtimebridge in de bestaande native M6c-keten. G07 is
een nieuwe appbouwsteen met hostmodeltests en native librarycompilatie; er
is daarmee geen extra GUI- of appstartbewijs geclaimd. Het bestaande
G1/Materiaal-werk blijft van zijn eigen eigenaar.

Gebruik een eigen `CARGO_TARGET_DIR` bij parallel werk. De `document-session`
README beschrijft de publieke API en de expliciete herstelkeuzes voor de app.
