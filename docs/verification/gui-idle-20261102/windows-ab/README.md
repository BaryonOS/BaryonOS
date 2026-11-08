# Volledige Windows-native A/B — 2 november 2026

**Geslaagd: 6/6 volledige native gates, 3 per arm.** Het mediane CPU-gebruik van het QEMU-proces daalt met 88,260028%; dat van de vastgehouden, op eigenaarschap gecontroleerde vCPU-thread met 88,526432%. Dit is één afgebakende headless WHPX-proef op mijn Windows-pc. Bij deze percentages staat één logische CPU voor 100%; ze beschrijven niet de capaciteit van de gehele pc.

| Meting | Basis A, mediaan | Idle B, mediaan | Daling medianen |
|---|---:|---:|---:|
| QEMU-proces | 99.372367% | 11.666288% | 88.260028% |
| Gemapte vCPU-thread | 98.955756% | 11.353756% | 88.526432% |

A is `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`, source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`. B is `1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b`, source-v3 `8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a`. Geen productcodewijziging na de B-freeze. De geïsoleerde code staat in concept-PR123; main/merges en eventuele productpunten worden door de centrale integrator behandeld.

## Methode en grenzen

Volgorde A,B,B,A,A,B, ieder met een nieuw wegwerpvolume en15s meetinterval. Zelfde expliciete QEMU11.1.0-binary (SHA256 in ieder rapport), WHPX, pc-q35-8.2, qemu64, één vCPU en256MiB, firmware/ROMs,1280x720, compiler/ELF-controles en synthetische data. Geen netwerk, hostmap, fysieke disk of USB/PCI/GPU-doorgifte. Er is geen Windows-/BIOS-/driver-/beveiligingsinstelling gewijzigd.

Alle18 endpointbeelden (initial-reference/before/after per run) zijn bytegelijk aan het oorspronkelijke volle scherm `6ed1645b503ba3957005cfd984c2bb5031b2f6788e9ee0b9040ae21ecef07fd7`, inclusief de oorspronkelijke Material-oracle. De bestaande muisbeweging naar1260,50 gaat aan de meting vooraf; er wordt vóór de meting geen toets verstuurd. Tijdens de15s zijn er geen QMP-opdrachten, geïnjecteerde invoer of stop/cont. De absolute oorspronkelijke deadline blijft staan. Win32-proces- en vCPU-counters worden afzonderlijk op hun eigen monotone meetduur berekend uit100ns kernel+user-tijden, met controle van vaste creatietijd, proces/threadeigenaar en liveness.

Na iedere meting loopt de oorspronkelijke bronrunner volledig door: boot, initial, editor, dirty, calculated, moved, recovered, rebound-editor, calculator-restored, restored-editor, saved, list, reopened. Werkelijke compositorfault slot4/vector6, herstart/rebind en behouden inhoud/berekening worden gecontroleerd. Daarna verplichte opslagoracle en echte cleanup/rawexit33. Alle zes oracle-uitkomsten zijn object0/revision1/14bytes, plaintext-SHA256 `9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`, canaries intact. Alle Windows-QMP-workers en proceshandles zijn schoon afgerond; geen watchdogkill of achterblijvende QEMU.

Dit bewijst de gezamenlijke verbetering van de vier GUI-loops. Geen afzonderlijke procesattributie binnen de gast, energie-/temperatuurmeting of native TCGlokale herhaling van deze idlebron. Gelijke beelden en conditionele FRAMES/COMMITS-logs bewijzen niet dat er tussendoor nooit getekend of geACKt is. Totale proefduur bevat de diagnostische15s; die is geen opstartmeting of productlatentiegarantie. Echte accountauthenticatie en fysieke hardwareacceptatie volgen niet uit deze test.

## Bewijs en reproductie

`aggregate-original.json.gz` bewaart de ongewijzigde formele aggregatoruitvoer; de gedecomprimeerde SHA256 is `26fc841be9ff0d1414db0eb23c245cb0a812504993cbd44ff432d213930f3637`. De leesbare `result.json` wijzigt uitsluitend CRLF naar LF; de eigen SHA staat in de index. De uitvoer bevat ieder interval, ruwe counters, uitkomsten en native-rapporthash. `index.json` bindt de ruwe native- en serialkopieën; decompressie van de deterministische .gz-bestanden levert exact de oorspronkelijke gerapporteerde hashes. Consolekopieën normaliseren CRLF naar LF en verwijderen alleen lege eindregels en bewaren de oorspronkelijke SHA apart. De losse ongeïnstrumenteerde baseline slaagde in36,946360s en hoort niet bij de CPUstatistiek. De eerste A/B-pilot met de eerdere cleanuphelper is uitgesloten.

De exact gebruikte lokale adapter en73 gerichte tests (72pass,1 expliciet overgeslagen pausestest) staan in `../windows-harness/` met bestandshashes en instructies. De helperfout rond CloseHandle is eerst met twee rode regressies aangetoond en daarna minimaal gerepareerd; de zes formele runs gebruiken allemaal de gecorrigeerde helper en identieke adapterhashes. De oorspronkelijke bronverifier, pixels, ACKs, deadlines, schijforacle en exitvoorwaarden zijn niet vervangen of afgezwakt.

Voor de baseline bestaat aanvullend geslaagd TCG-CI-bewijs op source-v3 `391940...1995`: [run41487814892/job125422649431](https://github.com/simonvanieperen/nieuw-os/actions/runs/41487814892/job/125422649431),13fasen/native_gate=true/raw33. Dit wordt niet als TCG-acceptatie van de gewijzigde B-bron gelabeld.

Een afzonderlijke AI-bewijsreview heeft de ruwe100ns-tellers onafhankelijk met Decimal herberekend en alle24 volledige bron-/referentie-/meetbeelden,78 fasebeeldhashes, materiaaloracles en zes werkelijke volumes gecontroleerd. Geen afwijking gevonden; dit is geen onafhankelijke menselijke audit. Exacte verifieerde grenzen staan in index.json.
