# Aanvullende M6b-review en adapterproeven

De servicecore, wirecodec en native adapter zijn afzonderlijk door een andere
AI-agent beoordeeld. Dit is geen formeel bewijs of menselijke audit.

`commands.json` bevat de uitgevoerde Rust 1.90-commando's en ruwe output voor
13 core- en 9 wiretests. Deze 22 tests staan ook in de gewone Cargo-gate en
worden daarom niet dubbel geteld. `source-hashes.json` en `review-summary.json`
binden de controles aan de onderzochte bronbytes.

Daarnaast is de echte `user_service.rs` tegen een tijdelijke host-ABI-shim
gecompileerd. Zeven scenario's: normale WRITE/FLUSH, WRITE-submitfout,
WRITE-pollfout, FLUSH-submitfout, FLUSH-pollfout, malformed completion en timeout.
Bij de foutvarianten bleef de sequence 0; volgende PUT/GET gaf Unknown zonder
nieuwe WRITE/FLUSH. Een voorafgaande request naar het andere domein gaf Access
zonder device-I/O. `adapter-probe-result.json` en `adapter_probe.rs` bewaren de
resultaten en exact gebruikte probe, inclusief de oorspronkelijke lokale paden.

Deze hostproeven bewijzen geen native devicefoutinjectie. De afzonderlijke
QEMU-gate in `../m6b/` test echte privileges/IPC, WRITE/FLUSH, domain denial,
uitputting, corruptie en twee externe proceskills met koude recovery.

Andere AI-reviews controleerden de kernelmanifest-/ACK-koppeling en het
onafhankelijke Python-schijforakel. Review van capture vond twee tekortkomingen:
het verwisselen van geslaagde hostlogs werd niet expliciet geblokkeerd en de
volledige artifact-keyset werd niet geëist. Beide zijn vóór het eindbewijs
hersteld en met negatieve substitutie-/weglatingstests afgedekt. Die review
vond geen daadwerkelijk ontbrekende run in de eerste volledige native gate.

De laatste guardreview vond geen blocker in die capturewijzigingen, de strikte
M1-fasecontrole of de binding van gekopieerde kernel/loader aan de verwachte
buildhashes vóór QEMU-start. Vijf bindingtests, zeven guestparsertests en negen
capturetests zijn daarvoor afzonderlijk opnieuw uitgevoerd (exit 0).
`guards/m6b-final-existing-capture.json` is de historische hostrun op `20f7ed2`,
voor het eindbewijs vervangen door de brongebonden v2-run in
`guards-final/m6b-final-existing-capture-v2.json` met `guards-final/manifest.json`.

`native-audit-20f7ed2.json` is de historische R9-audit van de complete gate
voor checkpoint `20f7ed282e579b96d964ae84509d895a7a826f4d` en bronfingerprint
`09fef202b74c362b2ead29c7cac12fee772bee3a3d382a6dbf7d576798c3e8f5`:
18 groepen, 14 echte VM-runs, 121 artifacthashes, zeven volumes, zes koude
herstelparen, twee vastgelegde SIGKILL-cuts, 22 Rust- en 26 harnastests.
De aparte parser reconstrueerde de bronfingerprint uit `git archive` en de
record-, revisie-, verwijder- en canarytoestand uit de werkelijke schijfbytes.
Het R9-script is niet in de repository gearchiveerd; `audit_script_runtime_path`
verwijst naar het oorspronkelijke tijdelijke pad `/tmp/m6b-final-audit-r9.py`.
De latere wijziging naar private M1-buildsnapshots heeft een eigen eindgate:
`final-native-audit.json` controleert opnieuw alle 18 groepen/14 VM-runs en 121
artifacthashes op `1e61a20`. Het exacte afzonderlijke stdlib-auditscript staat in
`independent_native_audit.py`; oorspronkelijke runtimepaden zijn behouden.
De daadwerkelijke gate/hostlogs staan in `../m6b/`. `guards/` bevat historische
guard-output voor `20f7ed2`. `guards-final/` bevat de actuele guardset voor
`1e61a20`: 8 snapshotbinding-, 7 parser- en 9 bestaande capturetests.

De eerste capture-wrapperconsole bleek leeg ondanks de geregistreerde exit 0.
De audit gebruikt daarvoor geen consolemarkerclaim: gastrecords, schijfbeelden
en capturehashes zijn afzonderlijk gecontroleerd. `capture-recheck/` bevat een
latere geslaagde capture-hercontrole van M1 en M6b, met volledige in-memory
opgevangen stdout, exitstatus en hashes; daarbij zijn geen nieuwe VM's gestart.
