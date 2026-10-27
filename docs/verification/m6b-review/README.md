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
buildhashes vóór QEMU-start. Vijf bindingtests, zeven guestparsertests en tien
capturetests zijn daarvoor afzonderlijk opnieuw uitgevoerd (exit 0). De finale
native gate op de aangescherpte verifierbron volgt na het broncheckpoint.
