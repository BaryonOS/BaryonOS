# Bijdragen aan Nieuw OS

Nieuw OS is een zelfstandig OS-onderzoeksproject met een eigen Rust-kernel. Een bijdrage mag de kern niet stilzwijgend vervangen door Linux, BSD, XNU, Windows, Android of een andere bestaande OS-kernel.

## Voor iedere wijziging

1. Lees `AGENTS.md` en de lokale `AGENTS.md` van de mappen die je wijzigt.
2. Lees de relevante architectuur-, security- en implementatiecontracten.
3. Houd kernel, userspace, capabilities, IPC en driverrechten gescheiden.
4. Voeg geen dependency toe zonder herkomst-, licentie-, onderhouds- en no_std/buildscriptcontrole waar relevant.
5. Label hostprototype, native uitvoering, emulatorbewijs en fysieke-hardwareclaims afzonderlijk.

## Verificatie

Gebruik de commando's uit `docs/DEVELOPMENT.md` en `docs/TEST-STRATEGY.md`. Minimaal moeten de bij de wijziging horende tests, formatting en linting slagen. Een niet-uitgevoerde test is geen groen resultaat. De CI voert een brede hostcontrole, native cross-build en een geïsoleerde QEMU-smoke uit; mijlpaalacceptatie kan aanvullende brongebonden gates vereisen.

## Pull requests

Houd wijzigingen klein en beoordeelbaar. Beschrijf het beveiligingsinvariant, de relevante negatieve tests, exacte uitgevoerde commando's, bewijsgrenzen en rollback. Verruim geen capability- of trustgrens alleen om een test te laten slagen.

Beveiligingsproblemen volgen `SECURITY.md`; zet geen secrets, echte gebruikersdata, productiekeys of hostschijven in tests of bewijsbundels.
