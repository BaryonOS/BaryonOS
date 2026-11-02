# Volume-admissie — taakcontract en checkpoint

Basis: `74a34d05ef5591b782a0da9da34e4669910d0722`; 2 november 2026.

Doel: onbekende/nulgevulde volumes vóór applicatietoegang weigeren, zonder
runtimeformatteerpad, kernelvervanging of nieuwe bestandssysteemdriver.
Alle native opslagmedia zijn eigen wegwerpbestanden; geen fysieke disks.

Contract: recovery selecteert uitsluitend volledig geldige eigen metadata.
Lege fixtures worden expliciet vooraf gecodeerd als canonieke sealed generatie
0 in slot 0. Bestaande gecommitte generaties en crashherstel blijven geldig.
Scope: `volume-core` parser/chooser, onafhankelijke Python-orakel/fixturemaker,
afhankelijke hostfixturevalidators, SDK-testinitialisatie, contract en bewijs.
Kernel, IPC, capabilities en blokschrijfrechten worden niet uitgebreid.

Hostcheckpoint: de nieuwe rawzero-regressietest faalt op de oude code en
slaagt na de fix. `volume-core` tests en Clippy, document-session en
Materiaal-apptests zijn groen. De onafhankelijke Python-suites volume,
system, images, random en materiaalprobe zijn groen; platformafhankelijke
Python-suites draaien ongewijzigd onder WSL. De eerdere platformfouten zijn
in de lokale logs behouden. Dit checkpoint is **nog geen native acceptatie**.

Native acceptatie: expliciete eigen lege fixture → normale writes → cold
recovery; Btrfs, OpenZFS-mirrorlid, rawzero en nulheaders met payload elk in
read-only en schrijfbaar profiel. Weigering vraagt expliciet startup-Corrupt,
geen appprogress, geen timeout, trace reads >0 en writes=0, source/proef-SHA256
gelijk. Daarnaast de volledige bestaande negen-VM-volume/crashgate.

Bevries broncommit, bronfingerprint, Python-testinputs en runner vóór de
native uitvoering. Native bewijs blijft open totdat de echte runs slagen.
Lokale logs/artifacten: `C:/Users/simon/Documents/Codex/2026-11-02/volume-format-admission/`.
Voortgang: 34/100 blijft gelijk; deze reparatie creëert geen nieuw criterium.
