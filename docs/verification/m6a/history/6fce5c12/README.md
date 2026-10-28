# Historisch M6a-bewijs vóór aanscherping van de observer

Bron: `8c53ea4`, fingerprint v3
`6fce5c12e87cf6b1a1353743357593de3d8b6843012b12be6deaf7fe17ce3b66`.
Werkelijk uitgevoerd op 27 oktober 2026: `python -u tools/test_m6.py`, exit 0,
54 groepen / 49 VM-runs; daarna `python tools/capture_gate.py m6a`, exit 0.
Deze oorspronkelijke capture stond lokaal ongecommit en is op 28 oktober
zonder inhoudelijke wijziging in deze historische map bewaard.

Dit is geen eindacceptatie van de huidige bron. De review vond drie gaten:
FULL vereiste geen waargenomen ENOSPC-probe, service-errors telde geen unieke
request-id-dekking/clientontvangst, en Unknown vereiste geen terminale
replyontvangst of exact begrensde mutatorpublicaties. Deze observercontroles
worden aangescherpt en de volledige poort wordt opnieuw uitgevoerd.

De aansluitende M1–M5-regressie was niet afgerond: de M3-build
`test-m3-b-exit` eindigde met exit 101 door Windows bestandsvergrendeling
(`os error 32`). M1 werd door de conditionele opdracht daarna niet gestart.
Dat is een geblokkeerde/afgebroken controle, geen groen eindresultaat.
Het lokale oorspronkelijke consolelog staat onder
`build/logs/m6a-regression-m2-m5-console.txt`.

De reportpaden en raw hashes verwijzen naar de oorspronkelijke lokale
wegwerpimages en logs. Er zijn geen binaries of images in Git opgenomen.
