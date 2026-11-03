# AUDIT-04 — defensieve securityreview

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder verwijzen naar die snapshot.
Externe normen en hardwaregedrag zijn niet onafhankelijk geverifieerd;
behandel zulke uitspraken als `ONBEKEND` of als toetsvraag.

Datum: 3 november 2026. **Uitkomst: één concrete bevinding met ernst HOOG;
geen algemene veiligheidsgoedkeuring.** Alleen dit rapport is geschreven.
Geen reparaties, builds, native tests, commits, pushes of externe diensten.

## Opdracht, bron en werkwijze

Letterlijke deelopdracht: “Voer een defensieve securityreview uit van
kernelgrenzen, capabilities, IPC, geheugen, cryptografie, boot, updates en
foutpaden.”

- Repository: `simonvanieperen/nieuw-os`.
- Onveranderlijke auditbron: `beafdedfc188154fe7285848fbd411e651013f54`.
- Opgegeven bronbranch: `copilot/add-os-specialist-agents`; de lokale en
  remote-trackingref wezen bij controle naar deze commit.
- Werkbranch bij binnenkomst: `simonvanieperen-symmetrical-couscous`,
  HEAD `69809c2cb53dfb3bd77abe5352892810fba6e82d`, schone werkboom.
  Dit was dus niet letterlijk branch `main`.
- Na de toegestane `git switch --detach`: detached HEAD op de auditbron.
  Alle inhoudelijke conclusies hieronder betreffen uitsluitend die commit.
  De werkbranch is niet hernoemd of gewijzigd.
- Werkmap: eigen worktree; het lokale pad is niet gepubliceerd.

Het profiel `.github/agents/os-security-reviewer.agent.md` is op deze bron
gelezen als werkinstructie, niet technisch als OS-customagent geladen.
**Werkwijzeafwijking:** er is daarnaast wel een algemene ingebouwde
`security-review`-hulpreview via agentselectie gebruikt voor de concrete
sleutelbevinding. Dit was dus niet uitsluitend een gewone chatsessie zonder
agentselector. De integrator moet die beperking zichtbaar houden; dit rapport
is geen bewijs dat de gevraagde selectorvrije werkwijze volledig is gevolgd.
De hoofdsessie heeft de overige hieronder afgebakende broncontroles en
rapportintegratie uitgevoerd. Dit is AI-review, geen onafhankelijke menselijke
audit.

Gelezen instructies: root-`AGENTS.md`, `docs/AGENTS.md`,
`kernel/AGENTS.md`, `boot/AGENTS.md`, `services/AGENTS.md`,
`drivers/AGENTS.md`, `tests/AGENTS.md`. Context: README, actueel checkpoint
bovenaan PROJECT-STATE, FULL-TEST-RELEASE, LOCAL-PRIVACY-CONTRACT revisie 4,
ARCHITECTURE, SECURITY-AND-QUANTUM en relevante contractpassages. Niet alle
historische statuspagina's zijn volledig gelezen.

De OS-dev-skill is gebruikt. De lokale workflow
`04-isolation-and-concurrency.md` en thema `21-beveiliging.md` dienen alleen
als checklist; conclusies steunen op projectcode en projectcontracten.
Geen externe technische bron is opgehaald en geen cryptografische primitief
of hardwarestandaard is hier opnieuw gecertificeerd.

## Bewijsniveaus

E0 = ontbrekend/ambitie; E1 = ontwerp; E2 = hostprototype/simulatie;
E3 = native code; E4 = native positieve test; E5 = native negatieve test;
E6 = herhaalde CI/releasegate; E7 = gedocumenteerde hardwaretest;
E8 = onafhankelijke menselijke audit/formele evaluatie.

Bronlezing in deze sessie levert maximaal E3 voor de onderzochte native
paden, geen nieuwe E4–E8. De aanwezigheid van testbron is niet hetzelfde als
een uitgevoerde host- of native test. Historisch bewijs blijft aan zijn eigen
broncommit gebonden.

## Concrete kwetsbaarheid

| ID | Ernst | Locatie | Samenvatting | Confidence |
|---|---|---|---|---|
| AUDIT-04-01 | HOOG | `sdk/examples/keyring/src/service.rs:520–536` | Gewoon domeinendpoint retourneert herstelgeheim én versleutelde master via legacy-herstelexport | 9/10 |

### AUDIT-04-01 — legacy-herstelexport passeert de administratieve grens

**Invariant.** Een gewone app met een domeingebonden sleutelendpoint krijgt
geen export-/herstelautoriteit over de volledige sleutelnamespace.
`docs/DECISIONS.md:373–383` reserveert deze autoriteit voor de afzonderlijke
supervisor-/herstelprincipal.

**Aanvalspad.** Een app die uitsluitend haar reguliere sleutelendpoint bezit,
stuurt KEY1-opcode `EXPORT_RECOVERY` (6) met lege body. De domeindispatcher
weigert alleen `EXPORT_PERSISTENT` en `RESTORE_PERSISTENT` en laat de oudere
herstelexport door. De succesvolle reply bevat het herstelgeheim en de
daarmee versleutelde masterblob samen. Geen admin-capability, vervalste
afzender of cryptografische doorbraak is nodig.

**Primaire bronketen op de auditcommit, aangeleverd door de hulpreview:**

- `kernel/src/m2/m3/system.rs:211–219,242–254,291–306`: gewone app krijgt
  domeinendpoint/replyrechten; init krijgt het aparte admin-endpoint.
- `sdk/examples/keyring/src/service.rs:629–638`: manifestgebonden app
  passeert de afzendercontrole en bereikt de domeindispatcher.
- `sdk/examples/keyring/src/service.rs:520–536`: onvolledige weigering;
  legacy-export kopieert eerst `secret`, daarna `blob` naar één reply.
- `services/keyring/src/lib.rs:462–489`: herstelblob bevat `self.master`
  en intrekkingsstatus; `:564–569` bevat de wrapping-key-afleiding;
  `:158–167` leidt domein-/indexsleutels uit die master af.
- `sdk/examples/keyring/src/app.rs:297–337`: bestaande positieve
  herstelproef vraagt deze operatie via een domeinendpoint aan.
- `docs/DECISIONS.md:373–383`: exportautoriteit is expliciet gelijkwaardig
  aan de volledige sleutelnamespace en hoort niet bij gewone clients.

**Impact.** De app kan buiten de dienst de master herstellen en vervolgens
sleutels van andere domeinen afleiden. Intrekkings- en domeincontroles van de
dienst beschermen dan niet meer tegen offline verwerking van ciphertext die
de app kan verkrijgen. Deze bevinding bewijst niet dat de app automatisch
ook andermans ciphertext kan lezen of kernelrechten krijgt.

**Bewijs.** E3: concrete native integratie en rechtstreeks traceerbaar
autorisatiepad. De hulpreview rapporteerde zes geslaagde statische
bronassertions; de exacte Pythoncode daarvan is niet meegeleverd en geldt
daarom niet als reproduceerbare regressietest in dit rapport.
Geen eigen exploit of runtimeherhaling uitgevoerd. Historische B01-herstelruns
zijn geen E5-bewijs dat het nieuwe administratieve integratiepad deze
legacy-aanvraag weigert. Het probleem zit in de geïntegreerde code, niet
noodzakelijk in de laatste commit die agentprofielen toevoegt.

**Minimale reparatie.** Maak domeinoperaties expliciet toegestaan en weiger
daar ook legacy `EXPORT_RECOVERY` en `RESTORE`. Scheid de oude herstelproef
zo nodig met een expliciet fixtureprofiel. Herstel als productfunctie blijft
nodig, maar vereist een apart geautoriseerd pad; alleen de opcode weghalen
zonder zo'n migratie is geen volledige productoplossing.

**Vereiste regressietest.** Geef een adversariële native app alleen
domein-0-rechten, zonder admin-capability. Stuur beide legacy-herstelopcodes;
vereis weigering zonder herstelmateriaal en zonder namespacewijziging.
Controleer een domein-1-canary en een ingetrokken sleutel. Laat afzonderlijk
de bevoegde admin-export/-restore slagen. Geen herstel uitgevoerd in deze
audit.

## Onderzochte paden zonder aanvullende concrete bevinding

Dit is gerichte brondekking, geen volledige review van elk genoemd bestand.
“Geen bevinding” betekent uitsluitend dat de hieronder genoemde controle
geen tweede concrete kwetsbaarheid opleverde.

| Oppervlak | Gelezen bron en controle | Resultaat en grens |
|---|---|---|
| Capabilities | `crates/capability-core/src/lib.rs:1312–1456,1467–1565`: derive/delegatie, revoke, send, receive, exit, lokale handle/generatie | Rechtenvermindering en TRANSFER/DERIVE worden gecontroleerd; queues/resources vóór commit; afzender wordt door engine gezet. Geen aanvullende concrete autorisatieomweg gevonden. Niet alle objecttypen of generatie-uitputtingspaden integraal onderzocht. |
| IPC/usercopy | `crates/kernel-core/src/user.rs:188–240`; `kernel/src/m2/m3/m4.rs:180–317,317–411` | Overflow, maximaal 320 bytes, userbereik, paginabezit en outputschrijfrecht worden vóór kopie/receivecommit gecontroleerd; fouten vergelijken autoriteit en output. Geen nieuwe pointerfout gevonden binnen één CPU/IF=0/geen gedeeld usergeheugen. Niet geëxtrapoleerd naar SMP of gedeelde mappings. |
| Geheugenrechten | `kernel/src/m2/paging.rs:297–355` | Usermappingbereik, paginalijning, USER op tabelniveaus, frame-aliascontrole voor W^X en NX-leaves zijn zichtbaar. Geen aanvullende concrete fout in dit fragment; volledige allocatorreclamation en alle unsafe-aliases niet opnieuw bewezen. |
| Kernelterugkeer | `kernel/src/m2/m3.rs:1050–1115`, symbolen in `kernel/src/m2/arch.rs` en `crates/kernel-core/src/user.rs` | Return-layout en uitvoerbare RIP/schrijfbare NX-stack gecontroleerd; ongeldige context markeert proces Dying en ruimt op. Assembly alleen gericht geïnventariseerd, niet instructie voor instructie geaudit. |
| Cryptografie | `services/keyring/src/records.rs:64–173` | Record-AAD bindt domein, volume, object, revisie, chunk, lengte en schema; authenticatie gaat vóór plaintextcopy; tijdelijke buffers gebruiken Zeroizing. Geen tweede concrete fout in deze functies. Geen bibliotheekaudit, nonce-collisionmeting of side-channelanalyse. |
| Entropy/replay/quota | `services/keyring/src/random.rs:95–210`; symbolen in `health.rs` | Monotone request-ID/tijd, byte/callquota, reseed en Degraded bij bron-/healthfout zichtbaar. Geen stille succesfallback gevonden. Healthtests bewijzen geen fysieke min-entropie. |
| Boot | `boot/loader/src/main.rs:405–506`, `boot/loader/src/signed.rs:1–220` | Begrensde EBS-retry, mapvalidatie en stop bij fout; signed profiel documenteert geen unsigned fallback; manifest/imageverificatie vóór ELF-gebruik. Niet de volledige loader, firmware of volledige image-selectielus geaudit. |
| Updatebeleid | `crates/update-core/src/policy.rs:1–64`; `docs/implementation/B03-SIGNED-UPDATES.md:183–220` | Gepinde signingparen, intrekking, dubbel-ID/tabelvalidatie en geauthenticeerd versievenster zichtbaar. Publieke TEST-ankers expliciet fixture, geen gevonden verborgen productieankers. Installatiedienst en alle crashvensters niet integraal opnieuw onderzocht. |
| DMA/foutpaden | `kernel/src/m2/m3/update.rs` gerichte reset/quarantaine-inventaris; `docs/DECISIONS.md:636–644` | Reset-ACK/quarantaine en vertrouwde QEMU-backend zijn expliciet. Geen conclusie over kwaadaardige fysieke DMA of IOMMU-containment; driverdescriptorcode niet volledig geaudit. |

Bestaande negatieve testbronnen zijn gelokaliseerd in
`crates/capability-core/src/tests.rs` (onder andere stale handles,
rechtenvergroting, queueoverdracht en atomische fouten) en
`crates/kernel-core/src/user.rs` (cross-page-eigendom en volledige
bereikvalidatie). Deze tests zijn hier **niet uitgevoerd**.

## Open bewijsgrenzen en releaseblokkers

Dit zijn geen extra bewezen kwetsbaarheden.

| ID | Ernst | Invariant / ontbrekend bewijs | Bron en niveau | Minimale vervolgstap / regressie |
|---|---|---|---|---|
| AUDIT-04-G01 | BLOKKEREND voor een productieclaim | Vertrouwde boot moet de loader en het beleid zelf omvatten, niet alleen kernelmanifesten | `docs/DECISIONS.md:1247–1264`; `boot/loader/src/signed.rs:15–17`; `crates/update-core/src/test_policy.rs:1`. Signed boot is opt-in; TEST-ankers, firmwareauthenticiteit, OVMF_VARS-aanvaller en productbeleid blijven open. E1/E3, geen nieuwe E4–E8. Confidence 10/10 over de expliciete beperking. | Producttrustankers en loader-/beleidsantirollback apart ontwerpen en native negatief testen; geen TEST-image als producttrustchain accepteren. |
| AUDIT-04-G02 | BLOKKEREND voor duurzame sleutelclaims | Crash/herstart mag sleutelintrekking en lineage niet ongemerkt terugdraaien | `docs/DECISIONS.md:357–391`: appvolgorde is coöperatief, koude vloer geen vertrouwd anker, write-ahead per GENERATE/REVOKE open. E1/E3, historisch beperkt E4/E5. Confidence 10/10 over de gedocumenteerde grens. | Duurzame wijziging in sleuteldienst afdwingen; crash vóór/na iedere stap, oude blobs en verkeerde lineage native testen. Los van AUDIT-04-01. |
| AUDIT-04-G03 | ONBEKEND buiten virtueel profiel | Een onbetrouwbaar apparaat mag geen geheugen buiten toegewezen buffers benaderen | `docs/DECISIONS.md:644`; `drivers/AGENTS.md`: geen M5-IOMMU, QEMU/backend onderdeel TCB. E1/E3, E7/E8 ontbreken in deze review. Confidence 10/10 over de scopegrens, geen bewezen hardware-exploit. | Vóór fysieke DMA aparte isolatie/negatieve test; nu geen devices of disks aansluiten. |
| AUDIT-04-G04 | ONBEKEND voor deze exacte commit | Native foutisolatie en releaseacceptatie moeten aan dezelfde complete bron zijn gebonden | `docs/FULL-TEST-RELEASE.md:1–100`; actuele status is integratiecheckpoint, geen verse complete securitygate op auditcommit. E0 voor zelf herhaalde gates. Confidence 10/10 dat hier niet getest is. | Integrator moet brongebonden native negatieve gates en actuele CI controleren; dit rapport sluit geen productcriterium. |

### Historische resultaten, niet opnieuw uitgevoerd

- `docs/verification/B01-ACCEPTANCE.md:3–42` rapporteert op
  `53e0d57af99c9673c97bc900fae15b98e08c8ca3` vijf QEMU-runs voor
  normal/no-device/stuck/timeout/restart en eerdere hostproeven. Dit is
  historisch E4/E5, geen run van de auditcommit; requestercrash en ontbrekende
  reset-ACK staan expliciet buiten native bewijs.
- `docs/verification/B02-IPC-ZEROIZATION.md:1–55` beschrijft onder meer twaalf
  native gasten op `df1ea1d10e50345bb410c985e5994c12f699350b`, hostchecks en
  de pointerprovenancecorrectie. Historisch E4/E5; Miri en menselijke audit
  expliciet niet uitgevoerd. Dit is geen bewijs dat compiler-/registerkopieën
  of alle callerbuffers worden gewist.
- `docs/verification/B03-KEY-POLICY-1.md:3–46` rapporteert op
  `69d5c3acce8a97e38895f284f13d3de3dbf4a5d9` achttien QEMU-stages,
  host- en harnaschecks. Historisch E4/E5. Actuele-head-CI en
  productbeleidsdistributie zijn afzonderlijk open; niet gepromoveerd naar E6
  voor deze bron.
- E7 en E8 zijn door deze audit niet vastgesteld. De aanwezige workflownamen
  zijn geïnventariseerd, maar geen externe CI-runs of artifacts opgevraagd.

## Zelf uitgevoerde controles

Alle onderstaande shellcontroles zijn lokaal. `view`-lezingen en `rg`/`sed`-
inventarisaties zijn broninspectie, geen builds of geslaagde OS-tests.
De relevante volledige commando's en waargenomen exitstatus staan hieronder;
de lange zoekuitvoer is niet als nieuw verificatieartifact opgeslagen.

| Controle | Exact commando | Exit |
|---|---|---|
| Begintoestand | `git branch --show-current && git rev-parse HEAD && git status --short && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0 |
| Toegestane bronselectie | `git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && git ls-files '*AGENTS.md' '.github/agents/os-security-reviewer.agent.md'` | 0 |
| Detached bevestigen | `git symbolic-ref -q --short HEAD` | 1, verwacht voor detached HEAD |
| Bronrefs | `git --no-pager branch --all --points-at HEAD` | 0 |
| Werkboom tijdens lezen | `git status --short` | 0, leeg |
| Ongewijzigde tracked bron vóór rapport | `git diff --quiet && git diff --cached --quiet && git rev-parse HEAD` | 0, auditcommit ongewijzigd |
| Rapportwhitespace | `git diff --no-index --check /dev/null docs/audits/subagents/04-security.md` | 1 doordat het nieuwe bestand verschilt van `/dev/null`; geen whitespace-diagnostiek |
| Eindcontrole bron en schrijfgebied | `git diff --exit-code && git diff --cached --exit-code && git status --short --untracked-files=all && git rev-parse HEAD` | 0; uitsluitend het rapport untracked, HEAD blijft auditcommit |

Voorbeelden van daadwerkelijk uitgevoerde broninspectiecommando's, geen
runtimevalidatie:

```sh
# exit 0
nl -ba kernel/src/m2/paging.rs | sed -n '285,355p'
# exit 0
nl -ba crates/capability-core/src/lib.rs | sed -n '1450,1565p'
# exit 0
nl -ba docs/DECISIONS.md | sed -n '1237,1267p'
```

De gerichte `view`-lezingen met regelbereik zijn in de dekkingstabel en de
contextparagrafen verantwoord. Grote gecombineerde documentlezingen leverden
afgekorte tooluitvoer op; relevante passages zijn daarna afzonderlijk gelezen.
Een afgekorte leesuitvoer telt niet als volledige bestandsreview.

De hulpreview rapporteerde daarnaast een read-only diff tegenover de toen
lokale mainref. Die is niet gebruikt als auditbron of acceptatiebewijs;
dit rapport trekt geen conclusie uit main-/auditbronvermenging.
Een aangevraagde aanvullende rapportage aan dezelfde synchrone hulpreview
werd door het hulpmiddel geweigerd. Geen nieuwe agent voor dezelfde review
gestart; ontbrekende dekking is niet als voltooid ingevuld.

**Niet uitgevoerd:** Cargo/build/preflight, unit- en integratietests, QEMU,
fuzzing, Miri, sanitizer, fysieke hardware, destructieve foutinjectie,
cryptobibliotheek-/supplychainaudit, complete secretscan, netwerk-/NAS-/TLS-
penetratietest, volledige app-pakketparserreview, actuele CI/releasegate en
onafhankelijke menselijke audit. Geen externe credentials of echte data
gebruikt. Logreferenties uit historische rapporten blijven historisch; deze
sessie levert geen nieuwe runtimecommandologs.

## Overdracht

Rapport: `docs/audits/subagents/04-security.md`, ongecommit in eigen worktree.
De integrator moet dit bestand vanaf deze worktree overnemen; er is geen
gezamenlijke commit gemaakt. Belangrijkste technische vervolgpunt:
AUDIT-04-01 met bijbehorende native negatieve regressie.

Geen technische blocker voor het schrijven van dit rapport. Wel een expliciete
werkwijzeafwijking: de selectorvrije route is niet volledig gevolgd.
De review is begrensd en niet uitputtend; ontbrekende native tests,
producttrustchain en duurzame sleutelankering blijven open.
Voortgangsbestanden zijn op verzoek niet gewijzigd; geen nieuwe punten
voorgesteld en geen veiligheidsscore afgeleid uit de 41%-productmeter.
