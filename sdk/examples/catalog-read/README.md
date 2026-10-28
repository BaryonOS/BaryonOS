# Native voorbeeldapp: catalogus lezen

Dit voorbeeld is een echt `no_std`-gebruikersprogramma voor de bestaande Nieuw
OS-int80-ABI. Het vraagt één LIST op en leest daarna ieder beschreven document
met GET via de gedelegeerde catalogusendpoint. Het geeft geen schrijf-, blok-,
apparaat- of netwerkrechten door en gebruikt geen fictieve documentinhoud.

De app gebruikt de herbruikbare [typed catalogusclient](../../catalog-client/README.md):
de bestaande B01-client plus de native SDK-transportadapter. Zij implementeert
geen tweede wireclient. GET levert een gecontroleerd `Document` of expliciet
`NotFound`; een onverwachte verwijdering sluit de voorbeeldtaak af.

De app valideert de kernelafzender, tag, canonical envelope, request-ID, epoch,
replyformaat en de revisie/naam/lengte uit de catalogussnapshot. Een onverwachte
ontvangen capability wordt gesloten. Een concurrent gewijzigde catalogus,
afgewezen aanvraag of ontbrekend antwoord beëindigt de proef met een foutcode.
Alleen een pre-publication `EAGAIN` wordt binnen een vast budget opnieuw geprobeerd;
een eenmaal gequeued RPC wordt nooit automatisch herhaald.

```sh
python3 -B sdk/examples/catalog-read/build.py
```

Standaarduitvoer is `build/sdk-catalog-read/x86_64-unknown-none/release/catalog-read`.
Een eigen `CARGO_TARGET_DIR` verandert de bestemming. De builder gebruikt het
eigen app-linkerscript, bewaart W^X-segmenten en maakt geen QEMU-proces.

## Startvoorwaarde en bewijsgrens

Het voorbeeld gebruikt alleen de al bestaande M6c-startupvelden van de private
pagina op `0x40010000`: rol 1/2 op offset 0, dienst-epoch op 32,
ontvangendpoint op 56, sendendpoint op 64 en dienst-PID op 176. Die gegevens
moeten door een geautoriseerde bootstrap worden ingevuld. De app kan daarmee
geen capabilities aanmaken. Het aparte supportbestand houdt deze tijdelijke
fixture-adapter zichtbaar buiten de herbruikbare SDK.

De bestaande M6c-acceptatiemanifest verwacht zijn eigen probe. Het vervangen
van die ELF door dit programma is daarom **geen ondersteund startcommando**.
Een gewone appstarter of expliciete aanvullende testmanifest blijft nodig
om dit zelfstandige voorbeeld als nieuw proces te starten. De SDK kan apart
in de bestaande M6c-probe worden gekoppeld om de echte IPC-route te testen.

Exit 0 betekent dat deze app zijn aanvragen heeft gevalideerd; offsets 240/248
bewaren lokale diagnostiek (aantal documenten/totaal inhoudsbytes). Deze
velden zijn geen kerneluitgegeven PASS-marker en geen duurzaamheidsbewijs.
Een geslaagde build is alleen compilatiebewijs; geen nieuwe native Materiaal-
gebruikershandeling of fysieke-hardwareclaim.
