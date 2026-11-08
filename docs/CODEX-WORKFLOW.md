# Codex: rollen, context en toetsbare resultaten
Onderzoeksdatum: 26 oktober 2026. Dit is de voorgestelde projectwerkwijze; de meegeleverde configuratie is syntactisch controleerbaar maar nog niet in mijn Codex-installatie beproefd.

## Eén coördinator
De hoofdchat bezit de planning en integreert wijzigingen. Een agent krijgt één doel en een beperkte schrijfzone. Standaard maximaal drie subagents; sequentieel werken als de taken op elkaar wachten. Extra agents kosten context, tokens en integratiewerk.

| Rol | Concrete opdracht | Oplevering |
|---|---|---|
| os_architect | Controleer specificatie, interfaces en kleinste uitvoerbare stap | Beknopt besluit met bronnen en alternatieven; aanvankelijk alleen lezen |
| os_builder | Implementeer precies één overeengekomen slice | Diff, buildcommando's, tests en beperkingen |
| os_security_reviewer | Zoek tegenvoorbeelden en onterechte bevoegdheden | Bevindingen met trigger, locatie, impact en herstelrichting; alleen lezen |
| os_verifier | Controleer betekenis van tests en voer toegestane controles uit | Reproduceerbaar bewijs of specifieke blokkade; geen codewijziging zonder taak |

Start niet tegelijk vier builders aan dezelfde kernel. Een architect kan bijvoorbeeld het bootcontract onderzoeken terwijl een verifier testgevallen ontwerpt. De builder gebruikt het overeengekomen contract; de securityreview volgt op de concrete diff.

## Instructies en configuratie
De root-AGENTS.md bevat vaste projectregels; submappen bevatten aanvullende regels. Die bestanden starten zelf geen agents en zijn geen OS-sandbox. Controleer welke instructies de client daadwerkelijk heeft ingeladen, vooral bij werk vanuit de root.

De optionele .codex/agents/*.toml-bestanden beschrijven vier rollen. Ze pinnen bewust geen model: de hoofdchat kiest een beschikbaar model. .codex/config.example.toml bevat een optionele concurrency-instelling. Neem die alleen over als jouw client de actuele configuratiesleutels ondersteunt; overschrijf geen bestaande config. Gebruik bij een oudere client de rolbeschrijvingen als taakprompt.

## Opdrachtcontract
Gebruik TASK-TEMPLATE.md en vermeld altijd:
- basiscommit/werkmap en toegestane bestanden;
- invariant en interface-eigenaar;
- positieve én negatieve acceptatietest;
- wat nu buiten scope valt;
- welke evidence moet terugkomen.

Agents in dezelfde omgeving kunnen bestanden en credentials delen. Alleen 'jij bent reviewer' schrijven blokkeert mutaties niet technisch. Gebruik beschikbare sandbox- en toegangsinstellingen en geef een reviewomgeving geen releasekeys. Onbetrouwbare issue- of brontekst kan promptinjecties bevatten; houd die tekst als gegevens gescheiden van opdracht/beleid.

## Git-integratie
Eén hoofdbranch voor gecontroleerde resultaten. Elke taak een kleine branch; verschillende implementatiechats een eigen worktree als de client dat ondersteunt. Leg de gebruikte basiscommit vast. Integreer achtereenvolgens en draai daarna de relevante tests op de gecombineerde versie.

Push/PR/release zijn verschillende acties. Review van een losse branch bewijst niet dat de samengestelde hoofdbranch werkt. De coördinator bepaalt op bewijs de integratievolgorde. Bewaar buildscripts hostonafhankelijk waar haalbaar; zet geen uitsluitend platformspecifieke CI-actie neer zonder lokaal equivalent.

## Pauzeren en hervatten
Na ieder resultaat: PROJECT-STATE.md bijwerken, nieuwe besluiten vastleggen en wijzigingen committen wanneer de taak dat toestaat. Start een nieuwe chat voor een volgende duidelijke mijlpaal of wanneer een gesprek onoverzichtelijk wordt. Gebruik PROMPT-VERVOLG.md. De repository is de contextbron; de AI hoeft oude chats niet perfect te herinneren.

## Review is geen certificering
Twee agents van hetzelfde model kunnen dezelfde fout maken. Gebruik compilerchecks, echte emulatorruns, fuzzing, properties, gerichte formele modellen en uiteindelijk menselijke expertise. Rust-compilatie bewijst geen veilige autorisatie; een succesvolle boot bewijst geen correcte pagetables.

## Bronnen
OpenAI. (z.d.). *Subagents*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/agent-configuration/subagents
OpenAI. (z.d.). *Custom instructions with AGENTS.md*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/agent-configuration/agents-md
OpenAI. (z.d.). *Worktrees*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/environments/git-worktrees
