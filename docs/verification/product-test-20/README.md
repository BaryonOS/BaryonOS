# Rol 20 — beperkte native producttest DEV1

**Status: testontwerp gereed; uitvoering geblokkeerd op artifact- en
runnervrijgave.** Dit document specificeert een kleine, observeerbare
ontwikkeltest. Het is geen volledige releaseacceptatie en kent geen
voortgangspunten toe.

## Taakgrens en basis

- Taak: `TASK-QEMU-PRODUCT-DEV1`, rol 20.
- Testontwerp-basis: `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`
  (`simonvanieperen-probable-doodle`).
- Toegestane schrijfmap: uitsluitend `docs/verification/product-test-20/**`.
- Afnemer: agent 4 / kwaliteitsafdeling; uitkomsten via intake/release.
- Beoogde dekking: delen van `FR-U03`, `FR-U05`, `FR-O01`, `FR-O04`,
  `FR-O05` en `FR-R04`, en de bestaande rubricverwijzingen
  `U03`, `U04`, `R01`, `R03`.
- Productcode, kernel, runner, buildconfiguratie en gedeelde voortgangsbestanden
  vallen buiten deze rol.

De release-agent meldde dat nog geen geïntegreerde VM-FR-kandidaat was
vrijgegeven. De manager verduidelijkte daarna dat een exact bevroren,
uitdrukkelijk beperkte DEV-build wel vóór die eindkandidaat kan worden getest.
De genoemde UI-build op
`0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d` is specifiek
`native-system-materiaal-probe` met `run_materiaal_probe.py`, niet de gewone
`native-system-materiaal`-preview. De bestaande runner stuurt voor de normale
case begrensde QMP-muis-/toetsenbordinvoer en captures; de guest-build bevat
tegelijk expliciet een `desktop-controller`-probe, geen SDK runtime en geen
document-session. UI13 bevestigde dat QMP de create/edit/Save/list/reopen-
handelingen werkelijk als toetsen-/muisevents naar de UI stuurt en dat de
controller alleen bij de afsluitende `InitStopRPC`-barrier betrokken is.
Dat maakt het een instrumented development workflow, geen
gewone release-image of volledige gebruikerservaring. PT20-01 mag alleen
worden gerapporteerd als beperkte native UI-inputwaarneming nadat de artifact
en controllerrollen zijn gecontroleerd; geen probecontroller mag tekst
invoeren, opslaan of heropenen namens de gebruiker. De historische
interactieve rapporten op `eabc27b` zijn geen bewijs voor deze nieuwe bron.
De runnerbron op basis `ca45a7d` is alleen gelezen; er is geen runner- of
gastuitvoering uitgevoerd in deze taak.

## Startpoort: één onveranderlijk artifact

Geen run begint voordat release/agent 4 én runner-eigenaar 12 dezelfde
artifactspecificatie en runroute hebben bevestigd. De producttester ontvangt
een read-only kopie van dat artifact. Vereist in het kandidaatmanifest:

1. volledige Git-broncommit en bronfingerprint;
2. hashes van image, loader, kernel en alle meegeleverde userspace-apps/diensten;
3. build-feature/manifest en profielgrenzen; expliciete aanduiding DEV versus
   VM-FR;
4. QEMU-versie en binaryhash, machine/CPU/acceleratie/vCPU/RAM/display/input/
   deviceconfiguratie, en OVMF-versie plus CODE/VARS-templatehashes;
5. identificatie en beginhash van de wegwerpdisk/state; bevestiging dat geen
   host- of persoonlijke disk is gekoppeld;
6. exact goedgekeurd start- en stopcommando, werkdirectory en afgesproken
   observatie-/capturemethode;
7. fixture-instructies voor `Full` en `Unknown`, inclusief bevestiging dat
   fault injection alleen de test-I/O-tegenpartij beïnvloedt en geen product-
   handeling voor de gast uitvoert;
8. toegewezen runner-slot, eigenaar, exclusiviteit en de bijbehorende
   toolroute.

De lokale strikte preflight op de testworktree `ca45a7d` eindigde met exit 2:
Rust 1.90.0/rustup/cargo, `x86_64-unknown-uefi`, `x86_64-unknown-none`,
QEMU, OVMF-codefirmware en host-`cc` ontbreken. Dit is een runnerblokkade,
geen productbevinding. Rol 12 heeft daarna wel een bestaande lokale toolroute
doorgegeven, maar meldde dat er geen kandidaat/DEV1-runner-slot is: de enige
vrijgekomen VM-slot is gereserveerd voor rol 19 `CAP-DIGEST`. Er wordt geen
alternatieve toolchain geïnstalleerd of eigen artifact gebouwd; rol 12 levert
later de goedgekeurde omgeving/slot.

Stop vóór starten bij een bron-, artifact-, firmware- of profielhashmismatch,
ontbrekende vrijgave, gedeelde/gebruikersdisk, niet-exclusief runner-slot,
onbekende buildfeature, ontbrekende stoproute of een testfixture die zelf
uitvoer namens de gast zou verrichten. Bewaar zo'n poging als `BLOCKED`, niet
als product-`FAIL`.

## Observeerbare cases

Alle invoer voor documenthandelingen gebeurt via de werkelijk zichtbare
gebruikersinterface met normale toetsenbord-/muisbediening. QMP-toetsen mogen
alleen diezelfde invoer leveren; geen serial/syscall/testcommando of
guestcontroller mag een ontbrekende UI- of opslaghandeling namens de gebruiker
uitvoeren. De probecontroller mag uitsluitend de vooraf verklaarde
observatie-/barrier-/cleanupfunctie vervullen. Controleer en bewaar de exacte
QMP-trace, runnerbronhash en controllerrol. Leg echte schermbeelden en
inputtrace vast. Automatische pixels/assertions ondersteunen de waarneming,
maar zijn geen menselijke visuele of toegankelijkheidsreview.

| Case | Handeling en controle | Verwacht resultaat | Uitvoeringsstatus | Bewijs / status bij ontbrekende functie |
|---|---|---|---|---|
| `PT20-01` | Start vers artifact; navigeer via de gewone UI naar een nieuw document; voer via UI twee regels in met UTF-8 (`Rol20: café` en `tweede regel`); sla op; keer terug naar de lijst; open hetzelfde document. | Werkelijke UI toont navigatie, bewerkte tekst, begrijpelijke opgeslagen-status, document in lijst en heropende identieke tekst. Bewaaractie resulteert in duurzame opslag, niet alleen een succesframe/log. | `BLOCKED` — artifact en VM-slot ontbreken. | Bron- en imagebinding, inputtrace, relevante schermen/status, service-/seriële log, ruwe diskhash vóór/na en onafhankelijk inhoud-/revisie-orakel. Ontbrekende UI/bediening is `BLOCKED` voor deze kandidaat; onjuist gedrag is `FAIL`. |
| `PT20-02` | Na `PT20-01` de VM volledig stoppen volgens de vrijgegeven route; nieuwe QEMU-instantie starten met exact dezelfde diskbytes en opnieuw via de UI hetzelfde document openen. Geen snapshot-restore of gastgeheugensnapshot. | Dezelfde object/revisie/UTF-8-bytes openen; geen valse cold-reopenmelding. De onafhankelijke opslagcontrole bevestigt dezelfde bevestigde gegevens. | `BLOCKED` — geen vrijgegeven cold-run-image; normale probe en recoveryfeature verschillen. | Parent-/child-run-ID, stopbevestiging, diskhashes tussen runs, verse VM-/VARS-identificatie, screenshots/logs en onafhankelijk orakel. Alleen uitvoerbaar na geslaagde `PT20-01`; anders `BLOCKED`. Geen “zelfde image”-coldclaim tenzij release/agent 4 die expliciete imagegrens goedkeurt. |
| `PT20-03` | Alleen als de kandidaat een echte, zichtbare en afgebakende taakgrant implementeert: weiger de gevraagde documenttoegang via de normale prompt; probeer de taak; herstel door expliciet te verlenen. Indien intrekken zichtbaar ondersteund is, trek daarna in en probeer opnieuw. | Weigering/intrekking voorkomt toekomstige taaktoegang; UI verklaart de toestand zonder stil te slagen. Verlenen herstelt uitsluitend de bedoelde taak. Geen andere bestaande data of bevoegdheden veranderen. | `BLOCKED` — zichtbare grant-/deny-/revoke-UX niet in vrijgegeven artifact aangetoond. | Prompt-/focus-/inputbewijs, voor/na rechten-/service-observatie en opslagcontrole. Als deze echte UX/grant ontbreekt, `BLOCKED` wegens ontbrekende implementatie; geen verborgen capabilitymanipulatie om een UI te simuleren. |
| `PT20-04` | Met een afzonderlijk, uitsluitend synthetisch wegwerpvolume dat vóór de gaststart volgens de goedgekeurde fixture aantoonbaar `Full` is, probeer via de echte UI een nieuw document te bewaren. Heropen daarna een eerder bevestigd document indien aanwezig. | Zichtbare fout en bruikbare herstelmogelijkheid; geen onterechte savebevestiging; eerder bevestigd document blijft intact en heropenbaar. Geen stille overschrijving of herinitialisatie. | `BLOCKED` — rol 12 heeft geen Full-fixture vrijgegeven. | Fixturemanifest/beginhash, inputtrace, schermen, I/O-resultaat en onafhankelijk orakel na stop. `Full` moet uit werkelijk gastgedrag volgen; hostfixture mag alleen de testomgeving voorbereiden. Ontbrekende geautoriseerde Full-fixture is `BLOCKED`. |
| `PT20-05` | Alleen met een vooraf goedgekeurde fault-fixture: start een save via de echte UI en laat de externe virtuele I/O-tegenpartij een gecontroleerde fout/timeout opleveren op het overeengekomen punt, zodat de service de contractstatus `Unknown` kan rapporteren. Stop daarna volgens protocol en controleer disk en eventuele koude recovery. | Geen succes-ACK of opgeslagen-status voor een onzekere mutatie, geen automatische replay; fout blijft begrijpelijk en eerder bevestigde gegevens blijven volgens contract intact. De onafhankelijke cold oracle bepaalt wat werkelijk duurzaam is. | `BLOCKED` — rol 12 heeft geen Unknown-fixture vrijgegeven. | Exacte fault-injectieconfiguratie/trigger, inputtrace, gaststatus/logs, raw outcome, stop- en recoverybindings, onafhankelijke disk-/volume-orakel. Geen fault injection door een gast-hidden command. Zonder goedgekeurde fixture of passend UI-pad is dit `BLOCKED`, niet `FAIL`. |

`Full` en `Unknown` zijn verschillende uitkomsten; een foutmelding alleen
bewijst geen behoud van bytes. Als een geautoriseerde testfixture een status
niet werkelijk in de gast kan veroorzaken of waarnemen, wordt de case niet
uitgevoerd en blijft zij geblokkeerd.

## Per run vast te leggen

Maak voor elke case een aparte run-ID en registreer:

- status `PASS`, `FAIL`, `BLOCKED` of `NOT RUN`, zonder ontbrekende uitvoering
  als pass/fail te classificeren;
- exact acceptatie-ID, kandidaatnaam, broncommit/fingerprint en alle hashes
  uit de startpoort;
- fixture-/disk-/VARS-identiteit en hashes vóór en na elke VM-run;
- exact gestart commando, toolversies, run-/stop-/recoveryexitstatus en
  tijdstippen; ruwe logs buiten de gesprekscontext, compacte relevante
  uitsnedes en SHA-256 onder deze bewijsmap;
- genummerde bedieningsstappen, echte inputtrace, relevante schermafbeeldingen
  en onafhankelijke bytes-/revisie-orakeluitkomst;
- afwijking tussen verwacht en werkelijk gedrag, impact op gebruiker, en
  eigenaar/intake-ID voor één concrete reparatie indien er een reproduceerbare
  fout is.

Houd iedere mislukte poging ongewijzigd bewaard. Een reparatie vereist een
nieuwe artifacthash en een nieuwe run-ID; overschrijf of herlabel de mislukte
registratie niet. Deel de uitkomst met agent 4 en routeer een echt defect via
de afgesproken intake/release-eigenaar. Dit bewijs sluit geen volledige
`FR-*`-rij, `UC01–UC09`, VM-FR, menselijke accessibility/visuele review,
100 koude starts, 24-uursessie, fysieke hardware of volledige release af.

## Huidige uitvoering

| Item | Status | Bewijs / eerstvolgende voorwaarde |
|---|---|---|
| Testmatrix en stopregels | Gereed als ontwerp | Dit document; geen native uitvoering |
| Kandidaatartifact | `BLOCKED` | Release/agent 4 moet dezelfde immutable artifact en exacte hashes vrijgeven |
| Goedgekeurde toolroute/runner-slot | `BLOCKED` | Rol 12 kent een bestaande toolroute toe; het enige vrijgegeven VM-slot is bij rol 19 gereserveerd |
| Normale create/edit/save/reopen | `NOT RUN` | Artifact + runner + normale interactieve route |
| Cold reopen | `NOT RUN` | Geslaagde normale run; nieuw proces/VM, zelfde wegwerp-diskbytes |
| Deny/revoke, Full, Unknown | `NOT RUN` | Werkelijke kandidaatbediening en afzonderlijk goedgekeurde fixtures |
| Volledige product-/menselijke acceptatie | `OPEN` | Buiten DEV1-scope; geen aftekening door deze taak |
