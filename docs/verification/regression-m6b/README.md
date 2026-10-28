# Native regressies tijdens M6b

M2–M5 zijn volledig uitgevoerd op `44892e7ddd311021c3ae262f8b8e07f321da87a3`: **67 groepen en 56 echte VM-invocaties**. De mappen `m2/` tot en met `m5/` bevatten gecapturede suites, ongewijzigde gastrecords en hostlogs. `audit/capture-commands.json` bewaart de echte captureopdrachten en exitstatussen.

Op `20f7ed282e579b96d964ae84509d895a7a826f4d` zijn deze 56 gastrecords opnieuw met de aangescherpte parser gevalideerd. Alle 243 geregistreerde artifacthashes en seriële/stderr-bytes zijn opnieuw gecontroleerd. De 68 native bron-/bouwinputs in `audit/prior-regressions-new-verifier.json` zijn tussen beide commits bytegelijk. Dit is een hosthercontrole van bestaande echte VM-runs; er zijn geen 56 nieuwe runs uitgevoerd en hun oorspronkelijke bronfingerprint is behouden.

De afzonderlijke M6a-normal/herstelproef op `44892e7` bevat twee echte VM-invocaties: schrijven en daarna een nieuwe, alleen lezende herstel-VM. Dit is geen tweede volledige M6a-gate. De volledige M6a-baseline is elders onder `m6a-linux/` bewaard.

## Afgewezen M1-pogingen

Alle drie diagnostische pogingen blijven zichtbaar en tellen niet mee als geslaagde M1-gate:

- De eerste gate op `44892e7` eindigde met scriptstatus 0, maar coldboot 3 bevatte exact de oude M5-no-device-kernel. De oude M1-parser accepteerde deze ten onrechte. De gehele poging van acht VM-invocaties is afgewezen; `audit/rejected-m1-first-attempt/` bevat het bewijs.
- De nieuwe gate op `20f7ed2` startte één correcte coldboot. Voor coldboot 2 ontdekte de nieuwe hashcontrole een kopie van exact de oude M6a-recover-kernel en stopte vóór een tweede QEMU-start (scriptstatus 1). `audit/rejected-m1-second-attempt/` bewaart de suite, foutmelding, eerste gastrecord en bestandshashes/metadata.
- De derde gate op `20f7ed2` startte twee correcte coldboots. Voor coldboot 3 stopte de hashcontrole vóór QEMU wegens een afwijkende kernelhash (scriptstatus 1). `audit/rejected-m1-third-attempt/` bewaart beide gastrecords en de foutuitvoer.

De oorzaak van het terugkeren van een ouder gegenereerd artifact is niet vastgesteld. Paden waren zelfstandig, geen symlinks of gedeelde hardlinks naar de andere worktrees. Na afloop is geen concurrerend proces met deze werkmap waargenomen. Dit bewijst niet welke component de vervanging veroorzaakte. De nieuwe controle voorkomt wel dat zo'n verkeerde snapshot wordt gestart of als geslaagd wordt geteld.

De definitieve M1-gate op `1e61a20` is geaccepteerd: 10 controlegroepen en acht echte VM-invocaties. [FINAL-ACCEPTANCE.md](FINAL-ACCEPTANCE.md) beschrijft de uitkomst en bewijsgrenzen; `summary.json` onderscheidt de scopes en broncommits.

Het historische `copied-v3-proof-manifest.json` legt de oorspronkelijke kopie vast; latere documentatiecorrecties en aangevulde provenance veranderen die oorspronkelijke hashes niet. De checkoutcommit van de hosttest in `m3/m3-large-artifact-tests.json` is niet afzonderlijk vastgelegd en blijft expliciet onbekend. De M5-twee-imagecontrole is achteraf gekoppeld aan de bestaande `m5/guest-runs.json`-records; die hercontrole voegt geen VM-runs toe.
