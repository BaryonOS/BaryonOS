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
native uitvoering. Dit was de vooraf vastgelegde acceptatievoorwaarde; zie de afgeronde uitvoering hieronder.
Lokale logs/artifacten: `C:/Users/simon/Documents/Codex/2026-11-02/volume-format-admission/`.
Voortgang: historisch bleef 34/100 gelijk bij bron `94f12da`; deze reparatie creëert geen nieuw criterium. De huidige projectscore is 41/100.

## Afgeronde native uitvoering — 2026-11-02

Geteste bron `94f12da0b9740a1a0ced1f08b4a30787f82ae79f`: 10/10 toelatingsruns en de bestaande 9/9
volume-/crashgate geslaagd. De fix, hostcheckpointgeschiedenis, exacte
bronbinding, traces, hashes, herstelgrens en compatibiliteit staan in
[README.md](README.md). De historische productscore bij `94f12da` was 34/100; de huidige projectscore is 41/100.
