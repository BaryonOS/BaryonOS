# M6d — native opslagfouten zonder kernelpanic

Datum: 28 oktober 2026. Ontwerp vóór uitvoering, basis `b58ae2a` (M6c).
Werkbranch: `codex/m6d-io-recovery-roadmap`. Testresultaten staan afzonderlijk in
`../verification/m6d/`; dit contract zelf is geen geslaagd testresultaat.

## Probleem en gekozen grens

De echte ring-3-catalogusdienst beantwoordt mislukte WRITE/FLUSH-operaties al
conservatief met `Unknown`. De kernelobserver van de M6c-proef verwachtte echter
steeds de vooraf voorspelde succesvolle mutatie. Daardoor kon juist die correcte
foutreply een kernelpanic veroorzaken. Deze stap laat de fout de app bereiken en
controleert de grens met echte QEMU-uitvoering van de eigen kernel.

De bestaande IPC-v4-, journal-v2- en capabilitycontracten blijven gelden.
M6d bouwt voort op `milestone-m6c`, zonder nieuwe syscall, afhankelijkheid,
onbeperkte driverrechten of gewijzigde schijfindeling. De oudere afzonderlijke
M6b-proef blijft selecteerbaar; de M6b-observer wordt in deze stap niet veranderd.

## Invarianten

1. Een eerste `Unknown` volgt uitsluitend op een werkelijk waargenomen mislukte
   muterende submit/poll in de broker. Pending is geen fout. Een willekeurige
   Unknown-reply zonder die oorzaak mag de succescontroles niet omzeilen.
2. Na onzekerheid blijft de dienst afgesloten totdat een nieuwe instantie de
   schijf herstelt. GET, LIST, PUT, DELETE en RENAME van beide appdomeinen geven
   `Unknown`, lege gegevens en nulrevisie. Ongeldige of ongeautoriseerde aanvragen
   blijven hun eigen validatiefout krijgen.
3. De onzekere mutatie verhoogt geen levende revisie, succes-ACK of catalogusteller.
   Er komt geen nieuwe blokaanroep, descriptorpublicatie of automatische replay.
   Een al gepubliceerde device-operatie kan op de schijf aanwezig zijn zonder ACK.
4. Normaal succes blijft een echte WRITE-completion én FLUSH-completion vereisen.
   Capabilities en endpointgebonden domeinen worden niet verruimd.
5. De dienst en beide apps blijven gepland; beide apps maken aantoonbaar voortgang. Onbevestigde
   reset laat DMA-buffers gepind en het device in quarantaine tot VM-stop.
6. Koude recovery start een nieuwe VM op exact dezelfde imagebytes, valideert de
   werkelijke journalprefix en verricht geen writes. Geen verwijdering of
   formattering om een test te laten slagen.

## Acht afgebakende native foutproeven

Iedere proef bevestigt eerst één PUT van object 0 (`Café A`, revisie 1), en
onderbreekt daarna de RENAME naar `Café hernoemd A`. Eén eerdere ACK is dus
toegestaan; voor de RENAME is geen succes-ACK toegestaan.

| Case | Injectiegrens | Mogelijke werkelijke eindprefix |
|---|---|---|
| `write-error` | lokale statusfout na echte WRITE used-ring-completion | PUT + RENAME |
| `flush-error` | lokale statusfout na echte FLUSH used-ring-completion | PUT + RENAME |
| `write-timeout` | doel-WRITE blijft voor broker zonder completion tot deadline | PUT, of PUT + RENAME |
| `flush-timeout` | doel-FLUSH blijft voor broker zonder completion tot deadline | PUT + RENAME |
| `reset-timeout` | timeout plus lokaal niet waargenomen resetbevestiging | PUT, of PUT + RENAME |
| `bad-descriptor` | descriptorcontrole weigert vóór publicatie | uitsluitend PUT |
| `used-write` | ongeldige lokale used-length na echte WRITE-completion | PUT + RENAME |
| `used-flush` | ongeldige lokale used-length na echte FLUSH-completion | PUT + RENAME |

De tests veranderen geen DMA-adres in het echte device. Status/used-length-
injecties veranderen uitsluitend de lokale completionsnapshot. Timeoutproeven
verbergen completion aan de broker; onderdrukte notify alleen bewijst niet dat
een device nooit heeft gelezen. Dit is native foutpadbewijs in een expliciet
testprofiel, geen bewijs van een fysieke schijffout of stroomuitval.

## Acceptatie en bewijs

- Een onafhankelijke Python-decoder controleert alle echte imagebytes, namen,
  inhoud, revisies en de onaangeroerde sectoren buiten het journal.
- De nog geldige levende toestand (sequence 1) en de mogelijke schijftoestand
  (sequence 1 of 2) worden afzonderlijk gecontroleerd. Het harnas mag de schijf
  niet naar de gewenste uitkomst ombuigen.
- Elke foutcase krijgt een eigen native run én eigen koude recovery; geen
  synthetische succeslogs of vervanging van native tests door hostmocks.
- Negatieve harnastests wijzen dubbele/ontbrekende foutmarkers, valse ACKs,
  extra journalrecords, verkeerde naam/inhoud en verwisselde images af.
- De volledige M6c-gate en de relevante M6b-regressie blijven vereist. Bestaand
  historisch bewijs wordt bewaard onder zijn oorspronkelijke versie.
- Gecontroleerde broncheckpoint vóór lange tests; ruwe exitcodes, commando's,
  fingerprints en artifacthashes in het bewijsarchief. Afzonderlijke AI-review
  is geen menselijke audit of formeel bewijs.

## Primaire technische bron

OASIS Open. (2022, 1 juli). *Virtual I/O Device (VIRTIO) Version 1.2*
(Committee Specification 01), §§2.4, 2.7.8 en 5.2.6.
https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html
(geraadpleegd 28 oktober 2026).

De projectkeuzes voor `Unknown`, journalprefix, testbudgetten en puntentoekenning
zijn eigen contracten, geen door VIRTIO bewezen eigenschappen.
