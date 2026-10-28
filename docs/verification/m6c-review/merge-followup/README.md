# M6c: geïntegreerd reviewherstel en hostverificatie

Datum: 28 oktober 2026. M6c-basis:
`b58ae2a56be6978eab2b5ac4bb5d12e3d615865f`.
Ingevoegde M6b-reviewlijn: `3f308b07e0bab445d1ea926561eab8d8e01c86a4`,
waaronder CodeRabbit-commit `56c43da0806f5f4de57cfb49dfd0418954e2ed9c`
en de aanvullende preflightreparatie.

## Afzonderlijke hostcontroles

Op een bytegetoetste snapshot van de M6c-basis zijn de acht gewijzigde
hosttools/tests uit de botcommit toegevoegd. Deze bestanden hadden geen
overlap met M6c-specifieke wijzigingen. **121 bestaande Python-hosttests**
zijn uitgevoerd met exit 0. De exacte opdracht, werkmap en uitvoer staan in
`combined-host-tests.json` en de bijbehorende stdout/stderr.
`combined-snapshot.json` legt basis, overlay en bronidentiteit vast.

Daarna zijn uitsluitend `tools/preflight.py` en `tests/test_preflight.py`
bijgewerkt naar de aanvullend gecontroleerde versies die ook probe-excepties
afwijzen. De gerichte hertoets van **vijf preflighttests** slaagt met exit 0;
`preflight-followup-tests.json` bevat de geteste bestandshashes en opdracht.
Drie van deze vijf tests zaten al in de 121-run. Twee tests zijn nieuw:
`test_probe_exceptions_fail_strict_inventory` en
`test_non_strict_inventory_reports_probe_exceptions_without_failing`.
In totaal zijn 123 unieke hosttests uitgevoerd, verdeeld over twee runs.
Overige geteste bronbytes bleven gelijk.

Er zijn in deze follow-up geen Rust-builds of native/QEMU-suites uitgevoerd.
De bestaande native uitvoeringsbron blijft `bc4cd104`. De nieuwe hosttoolbron
krijgt niet de oude volledige bronfingerprint. De M5-archiefhercontrole en
de 40 gerichte M6b-hosttests hebben een eigen afgebakend
[bewijsarchief](../../m6b-review/merge-followup/README.md).

## Bekende native observerfout

De nieuwe reviewbevinding over `Unknown` is geldig en niet gerepareerd.
Ook de normale `milestone-m6c` kan bij een native WRITE-/FLUSH-fout of
time-out panikeren in de acceptatieobserver voordat de client antwoord
krijgt. De vijftien bestaande host-ABI-scenario's toetsen de echte
serviceadapter zonder die kernelobserver. Ze bewijzen geen gecontroleerde
native voortzetting.

Het [contract](../../../implementation/M6C-NAMED-CATALOG.md) en de
[acceptatie](../../M6C-ACCEPTANCE.md) benoemen dit nu expliciet. De bedoelde
veiligheidsregels blijven behouden: geen commit zonder geslaagde WRITE en
FLUSH, geen automatische write-replay en blokkeren bij onzekerheid.
Native reparatie, devicefoutinjectie en nieuw brongebonden bewijs zijn
vereist voordat dit foutpad als geleverd wordt beschouwd.
[CodeRabbits gerichte scopebeoordeling](https://github.com/simonvanieperen/nieuw-os/pull/10#issuecomment-6457209539)
staat deze begrensde onderzoeksacceptatie toe onder die expliciete voorwaarden;
dit is geen algemene goedkeuring of bewering dat de fout is opgelost.

Dezelfde beoordeling bevestigt de inhoudelijke afhandeling van de oude
`core-final`-thread met `core-recheck`. De oorspronkelijke formatteringslogs
blijven behouden. GitHub-reviewstatus en merge worden afzonderlijk gecontroleerd.

## Latere aanscherping van de M6c-hostlogbinding

De bovenstaande runs horen bij de hier vermelde snapshots. Het volgende
[reviewherstel](../host-log-binding/README.md) wijzigt `test_m6c.py`,
`capture_m6c.py` en gerichte harnastests. Dat herstel heeft eigen bronhashes
en uitvoeringslogs; de eerdere 123 tests worden niet als een nieuwe run op
die latere bron voorgesteld.

