# R77: late serial failure en immutable parentfirmware

Reviewbasis `bef94615244f639a4df3934f54565551a6f0dc37`.
Uitsluitend de drie eigen cold-tool/testbestanden gewijzigd; geen baseupdate,
kernel, preview/reference-orakel, SDK, consumerlock of workflow gewijzigd.

## Rode reproductie

Uitgevoerd op de ongewijzigde productiecode, exit1:

```sh
python3 -B -m unittest tools.tests.test_materiaal_cold.ColdParentTests.test_complete_replay_and_refusal_of_source_disk_and_frame_tampering tools.tests.test_materiaal_cold.ColdLifecycleTests.test_late_guest_panic_after_final_frame_cannot_pass tools.tests.test_materiaal_cold.ColdLifecycleTests.test_late_guest_fail_after_final_frame_cannot_pass
```

Drie tests, zes failures: vier afzonderlijk gewijzigde parentfirmwarebestanden
werden ten onrechte geaccepteerd; zowel PANIC als MATERIAAL:FAIL na de
laatste screendump/preread bereikte ten onrechte passed=true.
Host-lifecyclemocks, geen claim dat de eerder opgeslagen native run faalde.
Log `/tmp/nieuw-os-cold-review-red.log`.

## Herstel en groene regressies

De runner valideert de definitieve seriële bytes na QMP-quit/raw0 en vóór
diskinspectie of passed=true. Late PANIC/FAIL schrijft daardoor een rood
rapport met raw0 en voert het diskorakel niet uit.

Parentreplay hasht nu de werkelijke OVMF_CODE.fd, OVMF_VARS.template.fd,
bios-256k.bin en VGA.rom tegen de gebonden firmwarehashes. Alle vier
tampergevallen worden geweigerd. OVMF_VARS.fd blijft mutable: expliciet
gewijzigde gaststate wordt in het positieve regressiegeval toegestaan.

Uitgevoerd na herstel, alle exit0:

```sh
python3 -B -m unittest tools.tests.test_materiaal_cold
python3 -B -m unittest discover -s tools/tests
git diff --check
PYTHONPATH=tools python3 -B -c 'from common import source_hash;print(source_hash())'
```

16 gerichte tests (3.469s) en 154 tools-tests (9.090s) groen.
Logs `/tmp/nieuw-os-cold-review-green.log` en
`/tmp/nieuw-os-cold-review-all-tools.log`.
Nieuwe v3 `949ce3abc8df4af3ed4cc29fd41e10ee0621e4f270ddf4ac2aafb3bacba9a901`.

## Expliciete bewijsgrens

Geen nieuwe native build/run of artifactreplay op deze gewijzigde v3
uitgevoerd. De geslaagde drie native runs blijven uitsluitend historisch
bewijs op code420fac5/v3f6d4; hun rapporten/hashes zijn niet aangepast.
Nieuwe same-source reference/parent/cold-herhaling is dus open voor deze
reviewhead. Lifecycle/teamleider bepalen de vervolggate; deze afgebakende
reviewfix stopt bij gerichte reparatie en controleerbaar checkpoint.
Clean shutdown, powerloss tijdens write, encryptie en volledige FR blijven
open. Scorevoorstel **34% -> 34%**, centrale beoordeling bij de teamleider.
