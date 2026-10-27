# Start hier
**Aanbevolen: één lokale Git-projectmap in Codex, eerst testen in QEMU en GitHub gebruiken als praktische remote.** Ontkoppel het OS-ontwerp van die hosting, zodat later een Europese forge de hoofdlocatie kan worden.

## 1. Wat je aanmaakt
- Een **projectmap/repository** is de broncode plus versiegeschiedenis. Dit is het echte project.
- Een **Codex-project** opent die map en geeft de agent toegang tot bestanden en ontwikkelgereedschappen.
- Een **ChatGPT-project** kan ideeën bundelen, maar is optioneel en vervangt de repository en buildomgeving niet.
- Een **GitHub Project** is een optioneel takenbord; niet nodig voor de eerste boot.

Gebruik één canonieke map. Kopieer het project niet voor iedere chat opnieuw. Begin met een schone, nieuwe map zodat bestaande bestanden niet worden overschreven.

## 2. Eerste inrichting zonder terminalervaring
1. Pak het zipbestand uit op je computer. Open de map die rechtstreeks `AGENTS.md` en `README.md` bevat.
2. Gebruik GitHub Desktop of Git om van deze map een lokale repository te maken. Commit het startpakket.
3. Publiceer de repository via GitHub Desktop wanneer je de inhoud en licentie hebt gecontroleerd. Privé beginnen is mogelijk; een privéproject is nog geen openbaar open-sourceproject. Voor publieke publicatie moet een echte licentietekst aanwezig zijn; zie LICENSE-PLAN.md.
4. Open dezelfde lokale map in Codex. Kies de lokale computer/werkruimte, niet per ongeluk een andere cloudomgeving.
5. Begin in de hoofdmap. Laat Codex in zijn eerste antwoord bevestigen welke projectmap en instructiebestanden het gebruikt.
6. Kies het model hieronder en plak PROMPT-START.md.

GitHub is hosting; Git is versiebeheer. Lokaal werken met Codex vereist geen GitHub-hosting. De lokale agent gebruikt je checkout; je hoeft daarvoor geen GitHub-connector te installeren. Voor push/pull moet Git/GitHub Desktop wel via de normale aanmeldprocedure geauthenticeerd zijn. Zet geen toegangstoken in een prompt, bronbestand of remote-URL.

## 3. Optioneel: de Git-opdrachten
Voer dit alleen uit **in de nieuwe uitgepakte projectmap**, als deze nog geen Git-repository is:

```bash
git init -b main
git status --short
git add .
git diff --cached --stat
git commit -m "Add independent OS research and Codex starter"
```

Als Git om identiteit vraagt: stel je eigen naam en een passend e-mailadres in. Gebruik desgewenst het door GitHub opgegeven noreply-adres; verzin dat adres niet.

Maak daarna op GitHub een **lege** repository; voeg daar niet ook een README toe. Vervang de twee placeholders in de URL voordat je onderstaande opdrachten gebruikt:

```bash
git remote add origin https://github.com/JOUW-GEBRUIKERSNAAM/JOUW-REPOSITORY.git
git remote -v
git push -u origin main
```

Gebruik de door Git aangeboden browser-/credential-manager-login. Bestaat `origin` al, inspecteer die eerst en voer niet blind opnieuw `remote add` uit. Een push naar een publieke repository maakt bestanden openbaar: controleer inhoud en licentie vooraf.

## 4. GitHub of een Europees alternatief?
| Route | Mijn oordeel |
|---|---|
| GitHub + lokaal Codex | Beste eerste route voor een beginner vanwege de gedocumenteerde koppelingen en praktische samenwerking. Het is een Amerikaanse dienst. |
| Codeberg + lokaal Codex | Passender als Europese hosting vanaf het begin een harde eis is; controleer gebruiksvoorwaarden en beschikbare CI voor het project. |
| Zelf gehoste Forgejo + lokaal Codex | Meer controle over hosting en beheer, maar je onderhoudt dan ook de forge, back-ups, updates en beveiliging. Bewaar dit voor later. |

Dezelfde Git-workflow kan een andere remote gebruiken. Een mirror kopieert Git-geschiedenis, maar niet automatisch alle issues, CI-geheimen, instellingen en releaseprocessen. Bewaar daarom bouwscripts, besluiten en releasespecificaties in de repository.

Codex is een clouddienst/modelworkflow, ook als de bestanden lokaal staan. Lokaal werken betekent niet dat alle verwerking lokaal blijft. Het toekomstige OS heeft een striktere, afzonderlijke producteis: AI uitsluitend op het apparaat en OS-data alleen op het apparaat of eigen NAS binnen eigen LAN. Codex/GitHub dienen hier uitsluitend de ontwikkeling en mogen geen runtimevereiste of kanaal voor persoonlijke OS-data worden. Controleer de actuele privacy-/gegevensinstellingen van je gebruikte account; dit pakket stelt geen specifieke bewaartermijn of vertrouwelijkheid vast.

## 5. Modelkeuze
Stand 26 oktober 2026, op basis van OpenAI's modeldocumentatie:
- **GPT-6 Astra, High of Extra High indien beschikbaar:** mijn keuze voor architectuur, geheugenisolatie, concurrerende kernelcode en moeilijke securityreviews.
- **GPT-6.1 Sol:** praktisch werkmodel voor afgebakende implementatie en herhaalde ontwikkeling; verhoog de redeneerinspanning bij complexe taken.
- Kleinere modellen alleen voor eenvoudige documentatie of triage, niet als enige beoordelaar van beveiligingskritieke code.

Dit is een taakgerichte aanbeveling, geen onafhankelijke benchmark die een model tot beste OS-bouwer uitroept. Welke opties jouw account vandaag toont: **Dit kan ik niet verifiëren.** Neem het sterkste beschikbare model voor risicovolle besluiten; forceer geen model-ID dat jouw client niet accepteert. Meer redeneertijd of meer agents levert geen bewijs van juistheid.

## 6. Bouwomgeving
De onderzoekskeuzes staan vast in docs/implementation/START-CODING.md en BOOT-AND-TOOLCHAIN.md. Laat Codex beginnen met `python3 tools/preflight.py` en de gekozen toolchain beschikbaar maken. Gebruik de gekozen versies; een aantoonbaar noodzakelijke afwijking wordt onderbouwd vastgelegd. Herstart geen algemeen toolchainonderzoek. Het script installeert niets en controleert aanwezigheid, niet de volledige bruikbaarheid.

Op Windows is een Linux-ontwikkelomgeving via WSL2 een redelijke optie, maar geen verplichte voorwaarde. Houd checkout, compiler en buildcommando's binnen één gekozen omgeving. Dat de compiler op Windows of Linux draait, betekent niet dat het nieuwe OS daarvan is afgeleid.

QEMU kan via software-emulatie werken wanneer hardwareversnelling ontbreekt. De eerste build moet dat ondersteunen. Geen echte schijven, USB-installers, BIOS-wijzigingen of telefoon-flashes voor de eerste mijlpalen.

## 7. Codex Cloud uitsluitend als ontwikkelalternatief
Deze sectie gaat over het schrijven van openbare/geschikte projectcode, niet over cloudfuncties of AI in het toekomstige OS. Het OS-contract staat in docs/LOCAL-PRIVACY-CONTRACT.md.

De actuele cloudroute gebruikt een omgeving met geselecteerde repositories. In web/desktop: kies de cloudomgeving, verbind GitHub wanneer gevraagd, selecteer alleen deze repository en laat de buildomgeving voorbereiden. Controleer de setupresultaten voordat je de omgeving publiceert. 'Omgeving publiceren' is niet hetzelfde als de broncode openbaar maken.

Laat expliciet testen of QEMU met software-emulatie, benodigde packages en time-outs er werken. Hardwareversnelling of toegang tot echte apparaten is niet aangenomen. Lukt booten in de cloud niet, laat daar review/hosttests doen en voer de bootcheck lokaal uit. Rapporteer beide apart.

## 8. Hoe je de gesprekken voert
Begin met één hoofdchat en maximaal drie gelijktijdige subagents. Geef per vervolgchat één technische of productmijlpaal, duidelijke toegestane bestanden en een acceptatietest. De productkeuze is al onderzocht: Materiaal. De eerste uitvoerbatch bouwt de bootproef, het interactieprototype en de lokale quantumsimulator parallel, volgens docs/implementation/FIRST-BATCH.md. Gebruik aparte worktrees voor onafhankelijke implementatiechats. Laat code pas samenvoegen nadat tests en review op de gecombineerde versie slagen.

AGENTS.md-bestanden zijn instructies, geen automatisch opgestart team of beveiligingsgrens. De optionele TOML-rollen moeten door de gebruikte client worden ondersteund. Laat de agent controleren wat geladen is en gebruik anders dezelfde rollen via expliciete taakprompts. De hoofdchat blijft verantwoordelijk.

## Bronnen
OpenAI. (z.d.). *Models*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/models
OpenAI. (z.d.). *Run coding tasks in the cloud*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/cloud
OpenAI. (z.d.). *Worktrees*. Geraadpleegd op 26 oktober 2026, van https://learn.chatgpt.com/docs/environments/git-worktrees
GitHub. (z.d.). *Creating a new repository*. https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-new-repository
GitHub. (z.d.). *Cloning a repository*. https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository
Voor de primaire bronnen over Codeberg/Forgejo en open source: zie docs/PRODUCT-AND-SOVEREIGNTY.md.
