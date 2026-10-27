# Definitieve regressieacceptatie bij M6b

Deze latere eindstatus vult het oorspronkelijke V3-verslag in `README.md` en
`summary.json` aan; die bestanden en hun kopieermanifest blijven historisch
behouden. Geen eerder afgewezen poging is alsnog als geslaagd aangemerkt.

## M1 op de uiteindelijke verifierbron

De volledige `python3 -B -u tools/test.py` en aansluitende capture eindigden beide
met exit 0 op `1e61a20beac67d685cf4bff5f0f9f588bf2cb56e`:
**10 controlegroepen en acht echte VM-invocaties**. Het bronfingerprint is
`43e6c06e43e3ba602b49c304328f0c1700bc26dae0a1256f0d5d420421f55ae2`.

De drie coldboots gebruiken dezelfde gevalideerde kernel en loader. Ontbrekende
en corrupte kernels worden geweigerd (exit 35), de UEFI-retry slaagt (exit 33),
de panic geeft exit 35 en de opzettelijke hang eindigt na 60,0298 seconden als
time-out zonder PASS. De vier negatieve gastrecords blijven terecht `passed:false`;
de suite beoordeelt de verwachte foutafhandeling.

`m1/` bevat de volledige suite, acht gastrecords en hostlogs. `root-final/`
bevat beide commando's, de onafhankelijke audit en kopieën van de vier
gecontroleerde buildmanifests. De audit controleerde alle 31 artifacthashes,
de echte seriële/stderr-bytes, zes kernel-/loaderbindingen en de tijdsvolgorde.
De afzonderlijke runmappen zijn de bewijsbasis voor het aantal van acht VM's.

Het wrapper-stdoutbestand bevat slechts vijf QEMU-samenvattingen en wordt daarom
niet als volledige console geclaimd. De overige drie hebben wel hun eigen,
afzonderlijk gecontroleerde runrecords en ruwe logs. De latere capture-hercontrole
onder `../m6b-review/capture-recheck/` is opnieuw exit 0, met volledige opgevangen
stdout; dit voegt geen VM-runs toe.

## M2–M5 en M6a

M2–M5 zijn volledig uitgevoerd op `44892e7`: 67 groepen en 56 echte VM-runs.
`root-final/independent-audit.json` controleert die 56 bestaande records en
243 artifacthashes opnieuw met de uiteindelijke verifier. Alle 68 native bron-
en bouwinputs zijn bytegelijk tussen `44892e7` en `1e61a20`. De oude broncommit
en fingerprint blijven intact; er zijn geen 56 nieuwe runs geclaimd.

`m6a-pair/` bewaart een normale mutatie- en afzonderlijke herstel-VM op `44892e7`.
De volledige M6a-baseline op `10b32e3` staat apart onder `../m6a-linux/`:
54 groepen/49 VM, zes externe kills en 17 koude recoveries.

## Afgewezen diagnostiek en harnasreparatie

Alle drie eerdere M1-pogingen staan onder `audit/rejected-m1-*/` en zijn
uitgesloten van de geaccepteerde aantallen. In de eerste poging accepteerde de
oude parser een M5-testvariant. De twee volgende pogingen werden door de nieuwe
hashcontrole vóór de verkeerde QEMU-start geblokkeerd. Er waren achtereenvolgens
acht, één en twee daadwerkelijke VM-invocaties in die afgewezen pogingen.

De oorzaak van het terugkeren van oude gegenereerde artifacts is onbekend.
De harnaswijziging neemt de afhankelijkheid van het veranderlijke `build/esp`
tijdens de gate weg: iedere build krijgt direct een eigen UUID-kopie, met
kernel-/loaderhashcontrole. Coldboots en negatieve fixtures gebruiken dezelfde
normale kopie. De runner controleert iedere uiteindelijke kopie opnieuw vóór
QEMU-start en weigert hogere milestone-uitvoer als M1-bewijs.

Geen gevolgtrekking naar fysieke hardware, native internet/NAS, versleuteling
of formele systeemveiligheid. De controles zijn door AI-agents uitgevoerd.
