# M6b — begrensde native objectopslag

## Doel en beveiligingsinvarianten

Deze stap maakt van de enkele M6a-testwaarde een kleine lokale documentdienst
in echte ring 3. Hij gebruikt de eigen kernel, capability-IPC en virtio-I/O.
De eerste limieten maken uitputting en herstel volledig toetsbaar; het zijn
geen definitieve productlimieten.

- Vier object-ID's (0–3), twee per domein, maximaal 128 waarde-bytes per object.
- Het ontvangende endpoint bepaalt het domein. Een request, sendergetal of
  fysieke sector uit appdata verleent geen rechten. De dienst controleert ook
  de door de kernel ingevulde sender-PID.
- Alleen de dienst krijgt READ/WRITE/FLUSH voor sectoren 4–35 van een synthetisch
  volume. Iedere app heeft uitsluitend SEND naar zijn eigen domeinendpoint en
  RECV voor zijn eigen antwoorden; tijdelijke brede bootstraprechten sluiten.
- Een PUT of DELETE vergelijkt de actuele objectrevisie. DELETE bewaart een
  tombstone met verhoogde revisie. Een oud verzoek kan een verwijderd object
  daardoor niet herstellen. Een lege waarde is een bestaand object.
- Eerst valideren en voorbereiden; dan WRITE-completion, FLUSH-completion,
  levende toestand wijzigen en pas daarna ACK. Bij onzeker resultaat: Unknown,
  geen automatische replay en geen verder gebruik van mogelijk achtergebleven
  in-memory data.
- 32 append-only records. Vol betekent Full vóór muterende I/O, zonder wrap of
  overschrijven. Deze stap heeft nog geen ruimtehergebruik.
- Recovery scant een volledige geldige prefix. Elk corrupt bezet record, gat,
  volgordefout of ongeldige revisieovergang sluit de dienst af. De dienst kiest
  niet stilzwijgend een oudere versie. Alleen een geheel lege journal is leeg.

## Interfaces

`services/objects` is een dependencyvrije `no_std`-crate met `forbid(unsafe_code)`.
`user_service.rs` is de afzonderlijke native adapter die de bestaande scalar
syscallruntime gebruikt. `userspace/m6b_probe.rs` bevat de twee testapps.
`kernel/src/m2/m3/m6b.rs` bevat de begrensde blockbroker en een expliciete
acceptatieobserver; het objectbeleid wordt door de ring-3-service uitgevoerd.

IPC-versie 3 gebruikt 224 payloadbytes binnen Message320. De velden zijn:
versie op 0, operatie op 2, request-ID op 8, service-epoch op 16, object-ID op 24,
verwachte/antwoordrevisie op 32, lengte op 40, status op 44 en waarde op 64.
Overige bytes zijn canoniek nul. GET=1, PUT=2, DELETE=3. NotFound=-19 kan voor
een geautoriseerd object de tombstonerevisie teruggeven; Access onthult die niet.

Een journalrecord van 512 bytes heeft magic `NOSM6B01`, versie 1, headersize 64,
globale sequence op 16, request-ID op 24, object-ID op 32, objectrevisie op 40,
operatie op 48, lengte op 50, data op 64 en FNV64 op 504. De checksum wordt over
het volledige record met nul checksumveld berekend. Dit is geen cryptografie.

## Native acceptatie

Build: `python3 -B tools/build.py --kernel-feature milestone-m6b`.
Smoke: `python3 -B tools/run_m6b.py --case normal --smoke`.
Volledige gate: `python3 -B -u tools/test_m6b.py`; daarna
`python3 -B tools/capture_m6b.py` voor hercontrole en tekstarchief.

Normaal maakt iedere app twee objecten, werkt één bij, verwijdert één en leest
beide terug. De eindtoestand heeft sequence 8: objecten 0/2 live op revisie 2,
objecten 1/3 als tombstone op revisie 2. Een koude tweede VM mag geen diskbytes
wijzigen en moet exact die toestand teruglezen.

Negatieve proeven controleren alle CRUD-operaties over de domeingrens,
malformed/padding/epoch/revisie, onjuiste handles, rechten en gesloten handles.
De observer controleert dat afgewezen serviceverzoeken geen device-I/O geven.
De full-proef doet 32 echte writes; extra writes geven Full zonder overschrijven.
De host controleert onafhankelijk alle journalbytes en canaries buiten scope.

Twee externe QEMU-kills: vóór de eerste WRITE-publicatie en na de eerste echte
mutatie-FLUSH maar vóór ACK. Een tweede koude VM moet respectievelijk de oude
lege toestand of de geschreven eerste record teruglezen. De host moet een nog
levend QEMU-child werkelijk beëindigen; een marker gevolgd door vrijwillige
exit is onvoldoende bewijs. Een beschadigd journal moet vóór READY weigeren.

## Bewijsgrenzen

Bekende beperking van de native acceptatieobserver: bij een WRITE- of FLUSH-fout kan `kernel/src/m2/m3/m6b.rs` een kernel panic veroorzaken, doordat `sent()` de reply nog met de verwachte `Ok`-reply vergelijkt voordat `Unknown` correct wordt verwerkt. Deze fout is niet gerepareerd. De opgeslagen native gates dekken dit foutpad niet en tonen de `Unknown`-invariant niet end-to-end aan. De zeven host-ABI-proeven controleren uitsluitend de serviceadapter onder gesimuleerde I/O-uitkomsten. Binnen de begrensde, vertrouwde QEMU/device-fixture blijft dit een expliciete onderzoeksbeperking; een latere native reparatie vereist gerichte devicefoutinjectie, nieuwe native verificatie en bijbehorende bronbinding.

De bovenstaande `Unknown`-regel blijft het servicecontract en de vereiste voor
latere native foutafhandeling. Zij is met het huidige observerpad niet voor
native devicefouten gerealiseerd. Dit geldt ook voor `milestone-m6b`.

Host-unittests, native uitvoering en onafhankelijke schijfcontrole worden apart
geregistreerd. Een QEMU-kill is geen fysieke stroomuitval. Het QEMU-device en
kerneltransport zijn vertrouwd, zonder IOMMU- of vijandige-DMA-claim. Er zijn nog
geen bestandnamen, directories, native desktop, encryptie, NAS-sync, algemene
service-supervisor of bescherming tegen vijandige schijfrollback. Historische
payloadbytes blijven in het append-only journal staan na een tombstone.
