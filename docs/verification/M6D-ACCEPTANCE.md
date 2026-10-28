# M6d — native onzekere opslag-I/O zonder observerpanic

Datum: 28 oktober 2026. Eigen kernel, branch `codex/m6d-io-recovery-roadmap`,
[PR #11](https://github.com/simonvanieperen/nieuw-os/pull/11).

**Native uitvoeringsbron:** `88e6c04be218b8b09ea4f0a226f6b9719da706a6`.
Fingerprint v3: `8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93`.
De broncheckpoint is vóór de volledige gates op GitHub bewaard; de ouders
`b739183c48a06cb7798d0be8cc683430152533ac` en
`5ba238cc113c5a18924adab6f6a0908fdf193de0` bewaren zowel onze M6d-implementatie
als de actuele M6c-reviewreparaties. Alle 45 eindrunrecords noemen deze ene
uitvoeringsbron. Latere wijzigingen zijn documentatie en opgeslagen bewijs.

## Gebouwd gedrag

De eigen ring-3-catalogusdienst kon al een onzekere mutatie met `Unknown`
beantwoorden. De kernelobserver verwachtte echter steeds de succesvolle reply
van de voorbereide mutatie en kon daardoor panikeren voordat de app het
antwoord kreeg. M6d bindt de eerste Unknown aan een werkelijk teruggegeven
terminale muterende syscallfout; `Pending` geldt niet als fout.

De onzekere RENAME levert geen succesbevestiging of commit in de levende toestand
op; de levende revisie blijft ongewijzigd.
Beide appdomeinen ontvangen daarna Unknown voor geldige GET/LIST/PUT/DELETE/
RENAME-aanvragen, zonder nieuwe blockcall, publicatie of completion. Ongeldige
of ongeautoriseerde aanvragen houden hun eigen foutantwoord. Gewoon succes
blijft WRITE én FLUSH vereisen. Capabilities, IPC v4, journal v2 en de
ring-3-service-implementatie blijven gelijk.

Het vooraf vastgelegde contract staat in
[M6D-IO-UNCERTAINTY.md](../implementation/M6D-IO-UNCERTAINTY.md); ontwerpbesluiten
ADR-031 en ADR-032 staan in [DECISIONS.md](../DECISIONS.md).

## Werkelijk uitgevoerde controles

Alle onderstaande geslaagde commando's eindigden met **exit 0**.

| Controle | Resultaat | Bewijs |
|---|---|---|
| Volledige uitgebreide M6c/M6d-gate | 35 groepen, 31 afzonderlijke VM-invocaties | [suite](m6d/suite.json), [ruwe gastresultaten](m6d/guest-runs.json) |
| Capture van M6d | bron, vaste dekking, raw exits, imagebytes, artifacthashes en hostlogbinding opnieuw gecontroleerd | [capture](m6d/capture.json) |
| Afzonderlijke volledige M6b-regressie | 18 groepen, 14 afzonderlijke VM-invocaties | [suite](m6d-m6b-regression/suite.json), [gastresultaten](m6d-m6b-regression/guest-runs.json) |
| Capture van M6b-regressie | eigen archief; historische M6b-archieven behouden | [capture](m6d-m6b-regression/capture.json) |
| Catalog-core en M6c-harnas | 30 Rust-tests en 48 Python-tests | [core](m6d/m6c-host-tests.stdout.txt), [harnas](m6d/m6c-harness-tests.stderr.txt) |
| Object-core en M6b-harnas | 22 Rust-tests en 30 Python-tests | [core](m6d-m6b-regression/m6b-host-tests.stdout.txt), [harnas](m6d-m6b-regression/m6b-harness-tests.stderr.txt) |
| Gerichte Rust-formatcontrole | gewijzigde kernelobserver en native probe | [commando](m6d-ready/m6d-final-rustfmt.json) |
| Normale afsluitende M6c-build | kernel en loader bytegelijk aan beide normale M6d-gateruns | [vergelijking](m6d-ready/verified.json), [buildcommando](m6d-ready/m6d-ready-build.json) |
| Afzonderlijke M6b-bewijsaudit | 121 artifacthashes, zeven volumes, zes koude herstelkoppelingen en exacte bronbinding | [audit](m6d-m6b-review/a04_m6b_audit.json) |
| Afzonderlijke M6d-bewijsaudit | 144 bronbestanden en 4 hosttestbestanden tegen de broncommit; 274 artifacthashes, 368 runfiles, 16 volumes | [audit](m6d-review/m6d-final-native-audit.json) |

De 31 VM-invocaties zijn 30 M6c-runs plus één gewone M1-boot. Er zijn 14 koude
recoveries, acht nieuwe foutcases en twee echte externe QEMU-kills. De 14 M6b-
invocaties zijn 13 M6b-runs plus een eigen gewone M1-boot, met zes recoveries en
twee externe kills. Samen **45 verschillende VM-invocaties**; hostfixtures en
audits tellen niet als extra VM's. De normale eindbuild is geen extra VM-run.
Dit zijn geen nieuwe volledige M1–M6a-regressiesuites.

### De acht nieuwe foutgevallen

Iedere proef begint met een bevestigde PUT (request 10, object 0, revisie 1).
De volgende RENAME (request 11) wordt onzeker. Elke foutproef bewijst één
terechte eerdere ACK, elf Unknown-antwoorden, 22 afwijzingen zonder I/O en
geen succes-ACK voor de RENAME. De levende sequence blijft 1; de daadwerkelijke
schijf wordt daarvan onafhankelijk gedecodeerd.

| Case | Foutgrens | Werkelijk gevonden journalprefix | Koude recovery |
|---|---|---|---|
| `write-error` | lokale statusfout na echte WRITE-completion | PUT + RENAME, sequence 2 | geslaagd, geen gewijzigde bytes |
| `flush-error` | lokale statusfout na echte FLUSH-completion | PUT + RENAME, sequence 2 | geslaagd, geen gewijzigde bytes |
| `write-timeout` | completion verborgen tot brokerdeadline | uitsluitend PUT, sequence 1 | geslaagd, geen gewijzigde bytes |
| `flush-timeout` | FLUSH-completion verborgen tot deadline | PUT + RENAME, sequence 2 | geslaagd, geen gewijzigde bytes |
| `reset-timeout` | timeout en niet waargenomen reset-ACK | uitsluitend PUT, sequence 1 | geslaagd, geen gewijzigde bytes |
| `bad-descriptor` | controle weigert vóór descriptorpublicatie | uitsluitend PUT, sequence 1 | geslaagd, geen gewijzigde bytes |
| `used-write` | ongeldige lokale used-length na WRITE | PUT + RENAME, sequence 2 | geslaagd, geen gewijzigde bytes |
| `used-flush` | ongeldige lokale used-length na FLUSH | PUT + RENAME, sequence 2 | geslaagd, geen gewijzigde bytes |

Voor WRITE-/reset-timeout liet het contract zowel sequence 1 als 2 toe: het
verbergen van completion bewijst niet dat een device niets heeft geschreven.
Het harnas heeft de images niet aangepast om de bovenstaande uitkomsten te
verkrijgen. De aparte decoder controleert ook exacte request-ID's, namen,
128 inhoudsbytes, revisies en alle bytes buiten de journal.

## Bewijsbinding en reproduceerroute

Het testprofiel is `pc-q35-10.0/qemu64/TCG/1CPU/256MiB`, QEMU 10.0.2 en
Rust 1.90.0, met een passend OVMF CODE/VARS-paar. De echte gebruikte QEMU-argv,
run-ID's, ruwe stdout/stderr, firmware- en binaire hashes staan per run in de
archieven. Normale beëindiging heeft de verwachte QEMU-debugexit **33**; de
vier bewuste externe proceskills hebben raw **-9** en zijn geen time-outs.
Een geslaagde runner of gate heeft daarbovenop exit **0**.

Uitgevoerd vanuit de repositoryroot:

```sh
python3 -B -u tools/test_m6c.py
python3 -B tools/capture_m6c.py --destination docs/verification/m6d
python3 -B -u tools/test_m6b.py
python3 -B tools/capture_m6b.py --destination docs/verification/m6d-m6b-regression
python3 -B tools/build.py --kernel-feature milestone-m6c
```

De laatste buildopdracht is via `common.run_logged` uitgevoerd; het exacte
argv, exitresultaat en stdout/stderr staan in `m6d-ready/`. De benodigde
firmware en targets moeten aanwezig zijn; deze proeven gebruiken uitsluitend
synthetische wegwerpimages en geen netwerk of echte schijven.

R09 controleerde de werkelijke runbestanden zonder imports van de project-
oracle of harnashulpfuncties. Alle 16 M6d-volumes zijn uit seeds en niet-lege
journal-sectoren als tekst opnieuw opgebouwd: 78 geldige v2-records plus de
corrupt-/legacyfixtures. Dezelfde begin- en eindhashes kwamen terug. Het
[zelfstandige auditprogramma](m6d-review/audit_m6d_native.py) en de
[sectorgegevens](m6d-review/m6d-actual-journal-records.json) blijven bewaard.
De controle die alleen die tekst nodig heeft is opnieuw uitgevoerd vanuit
`/tmp`, zonder de oorspronkelijke raw images:

```sh
python3 -B docs/verification/m6d-review/audit_m6d_native.py \
  --reconstruct docs/verification/m6d-review/m6d-actual-journal-records.json
```

Volledige audit op een checkout met de oorspronkelijke `build/run`-bestanden:

```sh
python3 -B docs/verification/m6d-review/audit_m6d_native.py \
  --repo . --out /tmp/nieuw-os-m6d-audit
```

De gearchiveerde oorspronkelijke commandorecords gebruiken de tijdelijke
scriptlocatie van de uitvoering. Beide paden hierboven gebruiken dezelfde
bewaarde scriptbytes. Een tekstreconstructie verifieert imagehashes; zij voert
geen nieuwe native gast uit. De AI-review vervangt geen menselijke audit.

De aparte [M6b-audit](m6d-m6b-review/a04_m6b_summary.md) controleert nog eens
121 artifacthashes, 164 run-/logbestanden en zeven volumes; 58 niet-lege sectoren
zijn als tekst bewaard. M6b bewaart nog geen hostloghashes op het gatetijdstip.
Die audit vergelijkt de oorspronkelijke huidige hostlogs met de capturebytes
en legt hun hashes vast; de strengere gate-time binding hierboven betreft M6c.
Dit verschil blijft zichtbaar en wordt niet als dezelfde garantie voorgesteld.

## Bewaarde mislukte start en herstelde reviewbevindingen

De eerste volledige gate stopte bij de toolinventaris: de aanwezige passende
OVMF-files stonden buiten het automatisch doorzochte pad. De gate gaf exit 1
nadat de preflight exit 2 gaf; **geen VM was gestart**. De oorspronkelijke
failed suite en logs staan onder [initial-preflight](m6d-review/initial-preflight/).
Beide firmwarefiles zijn vervolgens bytegelijk naar het al ondersteunde
`build/firmware` gekopieerd. De volledige gate is daarna opnieuw uitgevoerd,
zonder bron- of criteriumwijziging. Een eerdere sandboxprobe van rustup had
exit -6 door de signaalpipebeperking; de finale strikte inventaris buiten die
sandbox heeft alle vereiste toolprobes met exit 0 vastgelegd.

Twee concrete R08-bevindingen zijn vóór de eindbron hersteld: capture mag niet
de gehele verificatiemap vervangen en de schijfcontrole moet de precieze PUT10/
RENAME11-identiteit afdwingen. Negatieve tests dekken beide. De upstreamreview
van 5ba238cc blijft behouden, waaronder gate-time hostloghashes vóór én na
kopiëren. Geen historische mislukking is als een geslaagde native run geteld.

## Acceptatiegrens en voortgang

De vastgestelde gates, captures en brongebonden audit dragen het beperkte
criterium **S07 — Onzekere I/O zonder kernelpanic**. **13% → 14% (+1 procentpunt)** is samen met de afsluitende projectstatus in
`docs/progress.json` vastgelegd. De gewichten en alle andere criteria blijven
gelijk; de generator controleert rekensom, verwijzingen en weergave.

De dienst herstart nog niet automatisch in dezelfde VM. Na Unknown blijft zij
gesloten tot een nieuwe instantie de werkelijke journal heeft gelezen.
`reset-timeout` houdt DMA-buffers gepind en het device in quarantaine tot VM-stop.
Status-/used-lengthinjecties wijzigen lokale completionsnapshots; het zijn
native foutpadproeven, geen fysieke schijffouten of stroomuitval. QEMU en het
kerneltransport blijven vertrouwd; geen IOMMU-, SMP- of fysieke-hardwareclaim.

De historische M6b-observer is niet aangepast en krijgt geen nieuwe claim voor
zijn WRITE-/FLUSH-foutpad. Vier objecten, maximaal 128 inhoudsbytes, 32 naambytes
en 32 append-only records blijven proefbudgetten. Er zijn geen nieuwe desktop,
algemene apps, ruimtehergebruik, encryptie, entropybron, internet/NAS of lokale
AI aangetoond. Er is geen bestaande OS-kernel of nieuwe externe dependency
geïntroduceerd. K05 en S08 blijven open; 100% blijft een volledig dagelijks OS.
