# Beveiligingsbeleid

Baryon OS is experimenteel en nog niet geschikt voor dagelijks gebruik,
gevoelige gegevens of productie.

## Ondersteunde versies

Er bestaan nog geen ondersteunde releases. Meldingen over de actuele
ontwikkelstand op `main` zijn welkom voor onderzoek; dit houdt geen
ondersteuningsgarantie voor die ontwikkelstand of oudere commits in.

## Een kwetsbaarheid privé melden

GitHub Private Vulnerability Reporting is ingeschakeld voor deze repository.
Gebruik [Report a vulnerability](https://github.com/simonvanieperen/BaryonOS/security/advisories/new)
om kwetsbaarheidsdetails privé aan de repositorybeheerders te melden.
Deze knop staat ook op de openbare
[Security-pagina](https://github.com/simonvanieperen/BaryonOS/security).
Voor het indienen is aanmelden bij GitHub nodig.

Neem voor zover bekend het volgende op:

- de getroffen commit of versie en het betrokken onderdeel;
- een beschrijving van het probleem, de mogelijke impact en de voorwaarden
  waaronder het optreedt;
- reproduceerstappen en eventueel een minimaal demonstratievoorbeeld;
- de testomgeving, gebruikte tools en relevante, opgeschoonde logfragmenten;
- eventuele voorstellen voor een oplossing of tijdelijke beperking.

Deel geen exploiteerbare details in gewone publieke issues, discussies of
pull requests. Neem geen wachtwoorden, sleutels of echte gebruikersgegevens
op in de melding. Test alleen lokale emulatorimages of expliciet
geautoriseerde testapparaten. Als de meldroute niet beschikbaar is, publiceer
kwetsbaarheidsdetails dan niet alsnog via een publieke melding.

## Opvolging en openbaarmaking

Vervolgvragen en afstemming over een oplossing en openbaarmaking verlopen
via de privémelding. Er is geen gegarandeerde termijn voor ontvangstbevestiging,
beoordeling of herstel. Stem openbaarmaking van exploiteerbare details eerst
via de privémelding af met de repositorybeheerders.

## Grenzen en verdere toetsing

Een privémeldkanaal is geen bewijs van systeemveiligheid of een voltooide
releaseprocedure. Voor ontwerp en toetscriteria:
[SECURITY-AND-QUANTUM.md](docs/SECURITY-AND-QUANTUM.md).
Vóór een openbare testrelease blijven onder meer ondersteunde versies,
triage- en patch-/advisoryprocedures, productieondertekeningssleutels,
keybeheer, herstel bij keycompromis en externe review vast te leggen en te toetsen.
