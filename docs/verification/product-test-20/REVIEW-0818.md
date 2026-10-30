# Onafhankelijke read-only productbewijscontrole — 0818f5e

**Uitkomst: aangeleverde normale native probe is consistent; geen productfout
vastgesteld binnen de geteste vaste handeling.** Rol 20 startte geen VM en
voerde geen eigen bedieningsproef uit. Deze controle herberekent het bestaande
bewijs van rol 13 en is geen gewone DEV1-, VM-FR-, koude-herstart- of menselijke
acceptatie.

## Exacte kandidaat en bewijs

| Onderdeel | Vastgestelde identiteit |
|---|---|
| Productbron | `0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d` |
| v3 bronfingerprint | `26b41e5586434ce51202a4e234ccaa99bad25c5a76ec3efdb3af9d7020c14f5f` |
| Feature | `native-system-materiaal-probe`, acht processen inclusief testcontroller |
| Profiel | `pc-q35-10.0/qemu64/TCG/1CPU/256MiB/VGA/PS2`, 800×600/100%, geen netwerk |
| Kernel | `48acb20ca8e12e0418f21c3790361c2a3ac8c935cc76de0ad6ddb977bfd10896` |
| Loader | `9686ab3b85eaeedc334053cc9f583f6459c99e2779673416eacac319c68aa387` |
| QEMU | 10.2.1, binary `fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb` |
| OVMF CODE | `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7` |
| OVMF VARS-template | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |
| VGA ROM | `c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde` |
| Bundel | `native-materiaal-probe-0818f5e-ad23a2ee91.tar.gz`, 2.682.413 bytes |
| Bundel-SHA-256 | `3c476387f7aa7657207e2a0ee96b7d1b159695de1716305cdbefa795b26447b2` |
| Producer-run | `materiaal-normal-ad23a2ee91`, gerapporteerd 132,461 s, raw QEMU-exit 33 |

De volledige oorspronkelijke QEMU-argumenten, userspace-/firmwarehashes,
40 memberhashes, zes snapshotbindings en gebruikte verifierbronhashes staan in
[`replay-0818.json`](replay-0818.json). De bronfingerprint van de bestaande
kandidaatcheckout is vóór en na replay opnieuw berekend en komt overeen.
Alle top-level Python-tools van die checkout zijn vóór import bytegelijk
vergeleken met Git-bron `0818f5e`; de bestaande `replay()`-assertions en
padcontroles zijn niet veranderd. Bundelleden zijn in geheugen gelezen,
niet uitgepakt. Alle 30 `run/`-leden zijn bytegelijk aan de canonieke run;
de canonieke bestanden zijn na replay opnieuw gehasht.

## Werkelijk teruggevonden handelingen

Beginsituatie: canonical empty synthetische wegwerpdisk, Materiaal-lijst leeg,
gepresenteerd Ready-frame, geen save-ACK.

1. De QMP-trace verplaatst de aanwijzer en klikt op **Nieuw**; daarna is het
   lege editorframe vastgelegd.
2. Veertien QMP-toetsaanslagen voeren `nieuw os werkt` in. Het dirty-frame
   toont die tekst en **Wijzigingen nog niet opgeslagen.**
3. `Ctrl+S` leidt tot één save-ACK, revisie 1, 14 bytes en een frame met
   **Op dit apparaat opgeslagen.**
4. `Escape` toont de lijst met **Nieuw document**.
5. `Tab`, `Tab`, `Enter` openen dat object opnieuw; het frame toont dezelfde
   tekst en **Document geopend.**
6. Pas na de laatste heropeningscapture volgt `F12`: de testcontroller voert
   de `InitStopRPC`-barrier/cleanup uit. Dit is geen normale productafsluiting
   of bewijs voor ACPI-poweroff.

De bestaande replay correleert alle 31 QMP-commando's en antwoorden exact
met deze vaste input/capturevolgorde. Iedere fase is aan haar actuele complete
seriële snapshot en screenshot gekoppeld. De zes frames zijn 800×600, veranderen
bij de bedoelde acties en behouden de systeemrand. Rol 20 heeft de zes
aangeleverde PNG's met AI-beeldwaarneming bekeken; dat is geen menselijke
vormgevings-, keyboard-only- of schermlezertest.

Het onafhankelijke bestaande volume-orakel leest uit de werkelijke actieve
disk en bytegelijke afterimage object 0, revisie 1, naam `Nieuw document`,
inhoud `nieuw os werkt`, generation 1 en clocks `[1, 0]`. Inhoud-SHA-256:
`9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`.
Andere objecten/domeinen zijn leeg en canaries buiten het volume blijven intact.
Begindiskhash:
`49c40c316234e2e4e4c8dd681a057b1b36611648896dcacba1794263b711c266`;
einddiskhash:
`246c8f9773533796718a729389c26d6103f45f04e7a83b4d06503e17c6d4e07e`.
De finale records bevatten 276 reads, 35 writes, drie flushes, nul levende
processen/handles en één herstelde framepool. Dit zijn opnieuw gecontroleerde
producerrecords, geen nieuwe gastmeting van rol 20.

## Compacte matrix voor agent 4

| Controle / acceptatieverband | Uitkomst | Bewijsgrens |
|---|---|---|
| Bundel-/bron-/artifactbinding, FR-R04 deel | PASS, read-only bewijscontrole | 40 members, source fingerprint, kernel/loader/embedded ELF's en bestaande QEMU-binary gecontroleerd |
| Create/edit/Save/list/reopen, PT20-01 / FR-U03 deel | PASS voor aangeleverde geïnstrumenteerde native probe | Echte QMP-invoer en opgeslagen bytes opnieuw gecontroleerd; geen eigen VM of gewone DEV1-aftekening |
| Opslagbytes na stop, FR-O01 deel | PASS voor afterimage-inspectie | Exacte inhoud/revisie en canaries; geen koude herstart uitgevoerd |
| PT20-02 cold reopen | GEBLOKKEERD / NIET UITGEVOERD | Alleen normale probe ontvangen; geen vrijgegeven gewone koude kandidaat/run |
| PT20-03 deny/revoke | GEBLOKKEERD / NIET UITGEVOERD | Geen vrijgegeven echte grant-/deny-/revoke-UX/testcase |
| PT20-04 Full | GEBLOKKEERD / NIET UITGEVOERD | Geen goedgekeurde Full-fixture op dit artifact |
| PT20-05 Unknown | GEBLOKKEERD / NIET UITGEVOERD | Geen goedgekeurde Unknown-fixture op dit artifact |
| FR-U05 / menselijke U04-beoordeling | OPEN | Geen screenreader, 200%, keyboard-only-VM of menselijke toets |
| Gewone DEV1 / volledige FR-R04 | OPEN | Probecontrollerprofiel, geen gewone interactieve productimage of complete aflevering |

Geen concreet productdefect aangetroffen in deze beperkte bewijscontrole;
daarom geen reparatie-intake aangemaakt. Niet-uitgevoerde negatieve gevallen
zijn geen software-FAIL. De oude blokkaderegistratie en de oorspronkelijke
mislukte verifierpoging blijven behouden.

## Uitgevoerde controle

Vanuit de eigen worktree, op basis `1d1b42c` plus de nieuwe reviewbestanden:

```sh
/usr/bin/time -f 'elapsed_seconds=%e\nexit_status=%x' \
  -o docs/verification/product-test-20/replay-0818-execution.txt \
  python3 -B docs/verification/product-test-20/verify_0818_bundle.py \
  --archive /home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818/native-materiaal-probe-0818f5e-ad23a2ee91.tar.gz \
  --handoff /home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818/handoff.json \
  --source-root /tmp/nieuw-os-render-review-e382 \
  > docs/verification/product-test-20/replay-0818.json \
  2> docs/verification/product-test-20/replay-0818.stderr.txt
```

Laatste uitvoering: **exit 0, 0,74 seconden**; meetlog
[`replay-0818-execution.txt`](replay-0818-execution.txt), inclusief vier negatieve read-only
reportmutaties: verkeerde exitstatus, bronhash, serialmemberhash en
screenshot-QMP-index worden alle geweigerd door de bestaande replay.
Deze mutaties schrijven niets naar de oorspronkelijke run.

Eerste verifierpoging: exit 1 ná geslaagde bestaande replay, door vergelijking
van Python-tuples met JSON-lijsten in het eigen handoff-vergelijkingsscript.
Alleen die representatievergelijking is JSON-genormaliseerd; geen product,
runner-assertion of artifact is gewijzigd. Foutlog:
[`replay-0818-attempt1.stderr.txt`](replay-0818-attempt1.stderr.txt).
De herhaalde uitvoering en de laatste uitvoering met vier negatieve checks
slagen; `replay-0818.stderr.txt` is leeg.

**Hervatten:** agent 4/8 levert een vaste gewone interactieve DEV-kandidaat
en rol 12 een slot als eigen bediening vereist is; cold/deny/Full/Unknown
alleen met passende vrijgegeven kandidaatcases/fixtures. Geen VM gestart,
geen tools geïnstalleerd en geen producerbestanden gewijzigd. Centrale
score blijft manager-owned; voorstel **34% → 34%**, geen nieuw volledig
geaccepteerd rubriccriterium.

De bestaande taak en dit schrijfgebied blijven geldig zonder nieuwe manager-ACK.
De vervolgstap is een benodigde kandidaat-/runnerbeslissing, geen verzoek om
hernieuwde toestemming voor deze read-only taak. Centrale status-/scorebestanden
zijn buiten dit schrijfgebied en blijven ongewijzigd.

## NEXT AGENT HANDOFF — 30 oktober 2026, 19:35 Europe/Amsterdam

Eigen bewijsbasis: `97b73e496d57c577978671d90aa392fa2c86fa7e`.
De hierboven gecontroleerde 0818-producerprobe blijft ongewijzigd geldig
binnen haar beperkte scope; gewone DEV1 blijft niet uitgevoerd.

Release meldde om 19:35 een nieuwe freeze `c969de2`, maar expliciet
**geen artifactvrijgave, slot of testopdracht**. Volgens die melding was
een eerdere eigen same-source-probe rood (raw exit 0, 3,18 s, geen ready).
Release schrijft dit toe aan tweemaal geerfde Cargo-linkerscriptconfiguratie
in een geneste worktree, met alle kernelgrenzen op `0x02000000`.
Dezelfde bron buiten die ancestorconfiguratie zou tweemaal met exit 0 en
correcte grenzen zijn gebouwd, met zeven overeenkomstige apps; 45 bestaande
probe-/previewhosttests zouden slagen. Dit zijn uitsluitend ontvangen
releasebevindingen: rol 20 heeft de nieuwe bron, builds, logs of oorzaak
**niet onafhankelijk gecontroleerd**. De verkorte bron-/fingerprintnotatie
in de melding is geen vervanging voor een volledig kandidaatmanifest.

Release heeft bij rol 12 een vers slot voor twee seriele gates aangevraagd;
de gewone VM is volgens release nog niet gestart. De eerdere rode run mag
niet door een latere groene build of test worden overschreven.

**Precieze hervatpoort:** ontvang na de ordinary-gate het immutable artifact,
volledige broncommit/fingerprint, hashes, profiel, bedienings-/stoproute en
diskbewijs; controleer eerst hun binding aan dezelfde kandidaat. Een eigen
VM start uitsluitend na expliciete slottoewijzing. Tot die tijd geen build,
alternatieve kandidaat of VM starten. Er is nu geen andere vrijgegeven
uitvoerbare productcase binnen deze taak; geen herhaalde toestemmingslus.
Scorevoorstel blijft 34% naar 34%; centrale beoordeling blijft bij de manager.
