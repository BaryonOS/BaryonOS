# Native venstersessie — U01 bouwcontract

Bronbasis: `0fe648b`, werkbranch `codex/mature-native-session-20261102`.
Dit contract beschrijft lopend werk; het sluit U01 niet en wijzigt geen punten.

## Resultaat en grenzen

Een native compositor toont Materiaal en Rekenmachine als twee afzonderlijke
CPL3-processen. Alleen de compositor bezit DisplaySource; alleen de inputbron
bezit InputSource. Applicaties krijgen uitsluitend hun eigen SUR1-/INP1-kanalen.
De bestaande document-, sleutel- en volumerechten blijven ongewijzigd. De nieuwe
grafische bibliotheek gebruikt begrensde vooraf gegenereerde letterbeelden en
eigen vectorpictogrammen, zonder runtime-fontparser of nieuwe privileged code.

De compositor beheert vensterposities, stapelvolgorde en invoerfocus. Een app kan
geen systeemrand overschrijven of invoer uit een ander venster opeisen. De
kernel levert de echte afzender van ieder bericht; een client-ID of coördinaat
verleent nooit rechten. Per venster bestaan een begrensde backing store, één
SUR1-ontvanger en een begrensde INP1-aanbod/ACK-wachtrij. Totaal maximaal
1.400.000 pixels backing store, twee vensters, geen heap of gedeeld appgeheugen.

## Herstel

De gebruikersruimtesupervisor beslist over herstel. Een compositorfout mag een
documentdraft of berekening niet vernietigen. De apps pauzeren presentatie en
sluiten de oude presentatiekanalen, waarna de supervisor de vooraf vastgelegde
compositor/inputdienstgroep kan herstarten met verse procesidentiteiten,
capabilities en epochs. Apps binden die nieuwe kanalen expliciet en tekenen
opnieuw. Opslag- en documentkanalen worden daarbij niet vervangen. De oude
kanalen leveren geen succes meer. Herstel verruimt geen apparaatrechten.

De kernel bevat alleen de begrensde proces-/manifestmechaniek, geen rendering,
invoerfocuspolicy of fontparser. Partial initialization, onjuiste aanroeper,
levende oude dienst, stale procescapability, endpoint-/frametekort en foute
generaties moeten gesloten falen. Bestaande diagnostische profielen blijven
apart bruikbaar.

## Acceptatie

Eerst hosttests voor geometrie, bounds, focus, aanbod/ACK, malformed berichten
en herstelbindingscontrole; daarna no_std-build/ELF-inspectie. Native acceptatie
vereist werkelijk twee apps, toetsenbord/muisfocus, vensterbeweging, negatieve
surface-/callerproeven en een echte compositorfout met behoud van draft en
berekening. Vervolgens echt bewaren/lijsten/heropenen, onafhankelijke diskcontrole
en resourcecleanup. De fysieke GPU, echte accountauthenticatie, schermlezer,
volledige tekstshaping en menselijke gebruikerstests vallen buiten deze gate.

N01 wordt daarnaast in een afzonderlijk native profiel geïntegreerd; een
netwerkdienstcompile is geen native netwerkacceptatie. U01/N01 krijgen alleen
hun vaste punten als de volledige bestaande criteriumgrens aantoonbaar slaagt.
