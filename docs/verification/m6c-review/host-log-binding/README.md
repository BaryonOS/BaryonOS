# M6c: hostlogs aan het gate-moment binden

Datum: 28 oktober 2026. Reviewbasis:
`b3d9d682f3f043ef7926f112b659ad77a5f00e1a`.
Aanleiding: [CodeRabbit-review van de hostlogbinding](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395750096).

## Probleem en herstel

De oorspronkelijke M6c-gate bewaarde voor zijn drie hostgroepen wel het
logpad, maar geen hashes van de toenmalige logbytes. Capture controleerde de
vaste paden, opdracht, werkmap en exitstatus. Een later vervangen log kon
daardoor passeren; de archiefhash werd pas na het kopiëren gemaakt.

`tools/test_m6c.py` legt nu direct na iedere hostgroep de SHA-256 van exact
stdout, commandometadata en stderr vast in `log_sha256`.
`tools/capture_m6c.py` vereist de volledige map met drie geldige hashes en
vergelijkt die met de bronbestanden. Het vergelijkt ook de gekopieerde bytes
in het tijdelijke archief vóór vervanging van het bestaande archief.
Ontbrekende of gewijzigde binding wordt afgewezen.

De drie gewijzigde bronbestanden zijn de twee genoemde tools en
`tests/test_m6c_capture.py`. De gastvalidatoren, native runtimebron en
oorspronkelijke native archieven zijn ongewijzigd.

## Werkelijk uitgevoerde hostcontroles

| Uitvoering | Testmethoden | Resultaat |
|---|---:|---|
| Oorspronkelijke b3-tools met de nieuwe regressies | 15 | Exit 1; 16 geregistreerde test- en subtestfouten |
| Gerepareerde capture en gate-producer | 15 | Exit 0; alle geslaagd |
| Bestaande M6c-harnasregressie | 6 | Exit 0; alle geslaagd |

Er slagen na dit herstel **21 gerichte hosttestmethoden**.
De producercontrole
`GateTimeHostLogBinding.test_each_completed_host_command_is_hashed_before_next_gate_step`
zit in de 15 capturetests. Vier nieuwe regressies controleren de echte capture-route met synthetische
gastrecords; de vijfde controleert de gate-producer met commandodoubles.
De native gastcontrole wordt in deze unitproeven vervangen door testdoubles. De gerichte harnasregressie
gebruikt bestaande host-harnasprocessen. Geen Rust-build of QEMU/native gate
is voor dit herstel uitgevoerd.

De demonstratie vóór de fix faalde inhoudelijk: gewijzigde stdout-, JSON-
en stderrbytes werden door de oude capture geaccepteerd, ook bij vervanging
tijdens het kopiëren. Verwachte `ValueError`-afwijzingen bleven uit.
Er waren geen signature- of `TypeError`-failures. De producer legde evenmin de
vereiste map vast. Na de fix slagen dezelfde nieuwe regressies; bestaande
directe callers zijn aangepast aan de nieuwe validatorparameter.

De exacte opdrachten, werkmap, UTC-tijd, exitstatus en 42 Pythonbronhashes
vóór/na staan in de drie commandorecords onder `records/`, samen met de
ongewijzigde ruwe uitvoer. De bronhashes bleven tijdens elke uitvoering
gelijk. Beide geslaagde runs horen bij de gepubliceerde eindversie van de
drie gewijzigde bestanden. `manifest.json` bindt bronnen en bewijsbestanden
aan SHA-256, byteaantallen en Git-blobhashes.

Deze 21 controles worden niet bij de eerdere 123 unieke hosttests opgeteld:
de testgroepen overlappen en de oudere runs horen bij hun eigen eerdere
bronversies. Zie [merge-followup](../merge-followup/README.md).

## Herhalen vanuit een afzonderlijke checkout

Voer vanuit de repositoryroot van de gerepareerde bron uit:

```sh
python3 -B -m unittest discover -s tests -p test_m6c_capture.py -v
python3 -B -m unittest discover -s tests -p test_m6c_harness.py -v
```

Voor de demonstratie van het oude gedrag gebruikt men een afzonderlijke
checkout van de bovenstaande b3-basis en vervangt men daar
`tests/test_m6c_capture.py` door
`records/before-fix-test_m6c_capture.py` uit dit archief. De eerste opdracht
demonstreert vervolgens de ontbrekende afwijzingen. Dit is een
reproductie-instructie, geen registratie van een extra uitvoering.

De bewaarde vóór-fix-testbron is bytegelijk gereconstrueerd door uitsluitend
de latere aanpassingen van bestaande API-callers en hun fixture terug te
zetten. De SHA-256
`fa89d0360acaa84c0db73fb5c46bcc8f54422c2d8caacdcc72363626682cd5cb`
komt exact overeen met het daadwerkelijk uitgevoerde testbestand in
`before-fix.json`; de vijf nieuwe regressies zijn inhoudelijk onveranderd.

`run_host_check.py` bewaart de gebruikte opnamewrapper en
`host-log-binding.patch` de bronwijziging. De wrapper verwacht zijn
oorspronkelijke indeling met een aparte `candidate/`, `tmp/` en
`records/`; hij is geen commando om het bewijsarchief ter plaatse opnieuw
te vullen. Herhalingen krijgen een eigen werkmap en logs. Absolute paden in
de commandorecords beschrijven de werkelijke historische uitvoering.

## Grenzen van het bestaande bewijs

De oorspronkelijke M6c-suite op `bc4cd104` heeft geen gate-time
`log_sha256`. Zij bewaart de bestaande uitvoering en capturehashes, maar
bewijst niet dat de hostlogs tussen gate en capture ongewijzigd bleven.
Die suite en logs blijven behouden. Er worden geen gatehashes achteraf
verzonnen; de aangescherpte capture weigert oude suites zonder deze binding.

Deze hostreparatie verandert de native bron niet en bewijst geen nieuwe
native uitvoering. De eerder vastgelegde Unknown-observerpanic blijft een
afzonderlijke, ongerepareerde onderzoeksbeperking. De volledige afbakening
staat in [M6C-ACCEPTANCE.md](../../M6C-ACCEPTANCE.md).

