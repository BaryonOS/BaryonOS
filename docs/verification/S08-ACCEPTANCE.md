# S08 — native opslagweerbaarheid en expliciete koude alleen-lezen recovery

## Geaccepteerde eigenschap (28 oktober 2026)

De actuele M6c-keten blijft op het vastgelegde single-CPU QEMU-profiel reageren bij geïnjecteerde schrijf-/flushfouten, time-outs, resetquarantaine, volle opslag en corrupte opslag. Onzekere mutaties geven `Unknown` zonder latere write-replay; het afzonderlijke herstelpad opent **dezelfde bewaarde image daadwerkelijk read-only**.

- PR: [#34](https://github.com/simonvanieperen/nieuw-os/pull/34)
- Geteste PR-head: `db1b62e87d0a2b3be371074d1dc20d71980909c9`.
- Geïntegreerde squash-commit op `main`: `b9c7cf8f4e1804f0333e8fba8862941190224155`.
- [Storage resilience acceptance — run 40761232906](https://github.com/simonvanieperen/nieuw-os/actions/runs/40761232906): **13/13 echte QEMU-VM-invocaties geslaagd**, JSON-rapport als bewaard workflowartefact.

## Gecontroleerde gevallen

| Live-proef | Koude read-only recovery |
|---|---|
| WRITE-I/O-fout | Ja |
| FLUSH-I/O-fout | Ja |
| WRITE-timeout | Ja |
| FLUSH-timeout | Ja |
| Reset-timeout / quarantaine | Ja |
| Vol volume / resource-uitputting | Ja |
| Corrupte opslag | Niet automatisch gerepareerd; fout blijft zichtbaar |

Elke live-proef toetst survivor-progress, resourcecleanup en een onafhankelijk schijforakel. Onzekere schrijfuitkomsten vereisen `Unknown`, `NO_REPLAY` en nul blockcalls na `Unknown`. De zes koude recoveries verifiëren dezelfde volume-identiteit, directe koppeling aan de vorige live-run, exact de oorspronkelijke `after`-schijfbytes als nieuwe `before`, en ongewijzigde eindbytes.

De aanvankelijke strengere run [40760069127](https://github.com/simonvanieperen/nieuw-os/actions/runs/40760069127) **faalde terecht**: de echte read-only schijf maakte zichtbaar dat de oude startup nog FLUSH/schrijfrechten verwachtte. De reparatie is in native code gemaakt, niet door de read-only-eis terug te draaien:

- QEMU opent de recoverydrive met `readonly=on` en het testharnas toetst dat exacte commandobewijs.
- De virtio-driver onderhandelt het read-only-profiel; recovery krijgt uitsluitend `BLOCK_READ`, geen `BLOCK_WRITE` of `BLOCK_FLUSH`.
- De userspace-catalogusdienst slaat zijn muterende startup-`FLUSH` alleen over in de expliciete herstelmodus en weigert mutatieverzoeken daar.
- De kernelobserver verwacht in die modus nul FLUSH-publicaties en nul succesvolle mutaties, terwijl de normale schrijvende gevallen ongewijzigd blijven.

Aanvullend geslaagd op dezelfde head: [CI 40761232901](https://github.com/simonvanieperen/nieuw-os/actions/runs/40761232901) en [Resource pressure 40761232999](https://github.com/simonvanieperen/nieuw-os/actions/runs/40761232999).

## Grenzen

Dit is native bewijs op een gecontroleerde QEMU/virtio-image en softwarematige foutinjectie, **geen fysiek schijfuitval-, stroomuitval-, corruptieherstel- of algemene ransomwaregarantie**. Een corrupte image wordt niet stilzwijgend aangepast. Het testharnas bepaalt de herstelkeuze expliciet; een consumenten-UI hiervoor bestaat nog niet. Grotere documenten, schaalbare volumes en volledige back-up/export blijven S04/S05/S09-open. Beoordeling volgens vaste rubric v1: **S08 = 1 acceptatiepunt**, zonder de eerder toegekende S07-eigenschap dubbel te tellen.
