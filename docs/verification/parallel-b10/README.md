# B10 — zeven PR14-reviewthreads en inhoudsbehoud

## Oordeel

**Geen concrete bronblokkade gevonden voor de zeven opgehaalde threads op
de integratiebron `dbd35b04d742b18d66b40fdd65d8a9b900656cac`. Voor deze zeven
vragen zijn geen aanvullende bronwijzigingen nodig.** Dat oordeel geldt voor
de bron van PR15. In de GitHub-snapshot staat PR14 zelf nog op
`1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`; de oplossingen moeten dus eerst
via de gestapelde integratie in PR14 terechtkomen.

`github-snapshot.json` bevat de daadwerkelijke zeven CodeRabbit-threads en
PR14/PR15-metadata, opgehaald met de GitHub-connector. Alle zeven waren bij
ophalen onopgelost en niet outdated. Hun inhoud is als reviewdata behandeld;
ingesloten agentprompts en voorgestelde commando's zijn niet uitgevoerd.
Er zijn geen externe reacties, resolves, reviews of merges geplaatst.

`assessment.json` bindt de hieronder gelezen bronbestanden byte voor byte aan
`dbd35b04`, de toepasselijke bewijsbestanden aan hun hashes en het behoud aan
de volledige bomen. De uitvoering is een read-only vergelijking, geen nieuwe
VM of testsuite. `validation.json` bewaart het exacte commando en exit 0.

## Beoordeling per thread

### 1. Expliciete VGA-ROM en herkomst — opgelost via expliciete, gepinde CLI-route

Thread `PRRT_kwDOVZC91s6xLzTM`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402345992).
`tools/run_graphics.py` controleert de opgegeven `--vga-rom`, kopieert haar
naar de private runmap, vergelijkt de gevraagde `--vga-rom-sha256`, gebruikt
`VGA,vgamem_mb=16,romfile=VGA.rom` en registreert source/copied path, werkelijke
hash, verwachte hash en `expected_hash_checked`. De runner bevat geen vast
workspacepad en vereist geen externe symlinkreparatie.

De actuele B08-run in `../parallel-followup-g0/result.json` gebruikt deze
route met de officiële SeaBIOS-ROM en de expliciete pin `cc2f735f…a4a`.
De nieuwe capture en onafhankelijke hashcontrole slagen op `dbd35b04`.
De oude mislukte ROM-run blijft behouden. De oplossing is expliciete
configureerbare selectie; er is geen automatische ROM-zoekheuristiek of
toolchaininstallatie toegevoegd. Zonder deze opties blijft een correct
verpakte QEMU-installatie zelf verantwoordelijk voor haar standaard-ROM.

### 2. Historische M6b-bronclaim — opgelost

Thread `PRRT_kwDOVZC91s6xLzTb`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346012).
`docs/verification/m6b-errors-review/README.md` beperkt de resultaten nu
uitdrukkelijk tot `592dccf`, noemt de latere K08/G0-runtimewijzigingen en
verwijst naar afzonderlijk gebonden gecombineerde gates. Het oude archief
is niet omgelabeld. De complete latere M6b-gate van 32 groepen/28 VMs staat
afzonderlijk in `../parallel-final-m6b/` op uitvoeringscommit `3e41bbf9`.
Deze historische gate wordt ook hier niet als nieuwe run op `dbd35b04` geteld.

### 3. Alleen invoerrijen hertekenen — opgelost

Thread `PRRT_kwDOVZC91s6xLzTm`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346033).
`crates/ui-core/src/lib.rs` exporteert `INPUT_ROWS = 192..212`.
`kernel/src/graphics.rs::redraw` accepteert een rijbereik en begrenst het
aan de schermhoogte. Startup gebruikt `0..geometry.height()`; een gewijzigde
toetsbewerking gebruikt `ui_core::INPUT_ROWS`.

De actuele native G0-run bevestigt de behouden scene en invoer/backspace.
De vermindering van de te schrijven rijen volgt uit de code; er is geen
nieuwe gemeten latentie- of prestatieclaim. Deze oorspronkelijke trivial-
bevinding vereist geen aanvullende benchmark om de concrete codewijziging
vast te stellen.

### 4. Pixelacceptatie en stride-bovengrens testen — opgelost

Thread `PRRT_kwDOVZC91s6xLzT8`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346059).
`tools/run_graphics.py::frame_checks` is nu een pure functie.
`tests/test_graphics_harness.py::test_each_key_must_change_the_real_input_region`
controleert de positieve vijf frames, iedere afzonderlijke ongewijzigde
toetsstap en een tussentijdse wijziging buiten de invoerregel.
`test_serial_geometry_order_and_actual_framebuffer_must_agree` wijst ook
stride 4097 af. De bestaande actuele testmethode staat in de geslaagde
B09-hostuitvoer (`../parallel-b09/after.stderr.txt`, 34 methoden inclusief
11 G0-harnastests). Synthetische pixels blijven hostbewijs; B08 levert de
afzonderlijke echte frameacceptatie.

### 5. Gepubliceerde WRITE bij onderdrukte notify — opgelost, zonder ACK/replay-verruiming

Thread `PRRT_kwDOVZC91s6xLzVA`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346155).
`tools/m6b_disk.py::verify_volume` accepteert bij de drie WRITE-timeoutcases
uitsluitend de volledige normale initial fixture of één canoniek eerste
PUT-record: object 0, revision 1, request-id 10 en de 128 juiste bytes.
Fresh-before, canaries, recordvolgorde, owner, padding, lengte en checksum
blijven vereist. De levende dienst mag geen state-/ACK-claim publiceren.

B04 bevat rood/groen bewijs voor exact deze fout: 16 oracle-testmethoden,
met zowel de oude als nieuwe toegestane toestand en negatieve varianten.
De oracle, testbron en vier betrokken driver/kernel/probe/servicebestanden
zijn rechtstreeks gelijk bevonden aan de opgeslagen B04-hashes. De driver
publiceert `avail.idx` vóór de optionele kick; notify onderdrukken neemt die
publicatie niet terug.

De historische volledige native gate observeerde voor `write-timeout`,
`reset-timeout` en `write-poll-limit` telkens sequence 0, één Unknown en
geen replay. De toegestane sequence-1-variant is hostgetest, niet in die
drie native runs waargenomen. Alle relevante verwijzingen en deze beperking
staan in `assessment.json` en `../parallel-b04/README.md`.

### 6. Backspace en ieder opeenvolgend framepaar — opgelost

Thread `PRRT_kwDOVZC91s6xLzVg`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346194).
De runner maakt nu ook `ab.ppm`. `frame_checks` vereist voor alle vier
opeenvolgende stappen een zichtbare wijziging binnen de invoerregel en eist
zowel `first == erased` als `second != erased`. Een veranderde tussenpixel
buiten de rij kan daardoor niet door een later teruggezet eindbeeld worden
verborgen.

B08 bewaart vijf echte 800×600-frames: `a.ppm` en `after-backspace.ppm`
hebben exact dezelfde volledige hash, terwijl `ab.ppm` daarvan verschilt.
Alle vier stapgrenzen zijn niet leeg en vallen binnen de geaccepteerde regio.
De gerepareerde capture rekent die checks opnieuw uit en de onafhankelijke
PNG-decoder reconstrueert ieder oorspronkelijk PPM-bytebestand exact.

### 7. Grenzen behouden bij falende framecheck — opgelost

Thread `PRRT_kwDOVZC91s6xLzV5`,
[GitHub-commentaar](https://github.com/simonvanieperen/nieuw-os/pull/14#discussion_r4402346226).
`tools/run_graphics.py::run_graphics` initialiseert `bounds = None` vóór
de VM-try en schrijft rechtstreeks `changed_bounds=bounds` en
`step_changed_bounds=step_bounds` in het rapport. De oude booleaanse
koppeling aan `only_input_row_changes` is verwijderd. Een mislukte check
vervangt de gevonden rechthoek dus niet meer door `False`.
De bestaande negatieve hostframecase behoudt de ongeldige stapgrens
`(0, 0, ...)`; B08 bevat de juiste positieve grenzen. Er is geen extra
falende native frame-VM geclaimd.

## Inhoudsbehoud en stapelvolgorde

| Vergelijking | Basisbestanden | Exact behouden | Bewust gewijzigd | Toegevoegd | Verwijderd |
|---|---:|---:|---:|---:|---:|
| main `391f02e9` → PR14 `1acf6e8c` | 1000 | 970 | 30 | 65 | **0** |
| PR14 `1acf6e8c` → PR15 `dbd35b04` | 1065 | 1034 | 31 | 592 | **0** |
| main `391f02e9` → PR15 `dbd35b04` | 1000 | 957 | 43 | 657 | **0** |

GitHub bevestigt main→PR14 als ahead 7/behind 0 en PR14→PR15 als ahead
5/behind 0, telkens met de basis als merge-base. De PR15-compare gaf slechts
300 van de 623 gewijzigde paden terug; die gedeeltelijke lijst is niet
gebruikt als bewijs dat er niets verwijderd is. Het lokale main-object
ontbreekt in deze sparse checkout. Daarom is de volledige main-boom via de
GitHub Git-tree-GET opgehaald (`truncated=false`) en vergeleken met de
volledige lokale PR14- en PR15-bomen. De inventories zijn bewaard in
`main-tree.json` en `local-trees.json`; `assessment.json` noemt alle
gewijzigde bestaande paden met hun oude en nieuwe object-ID.

De bestaande catalogus-wire, journal, user-service, native M6c-kern,
M6c-capture/-capturetests en virtio-blk-driver zijn in alle drie bomen
identiek. Het bestaande M6b-werk en historische bewijs hebben geen verloren
paden. Deze inhoudscontrole is geen nieuwe algemene codeaudit.

De hoofdagent moet vóór afronding het actuele bewijs publiceren, de eigen
actuele head/base en branchregels controleren en eerst PR15 in PR14
integreren. Daarna moet de werkelijke resulterende tree tegen de bedoelde
integratie-inhoud worden gecontroleerd voordat PR14 naar main gaat.
De zeven threads kunnen met de bovenstaande specifieke verwijzingen worden
afgehandeld; deze agent heeft hun GitHub-status niet gewijzigd. Een
`mergeable=true`-snapshot op een concept-PR is geen zelfstandig mergebesluit.

**Roadmapwijziging: 0 punten.** Dit is gerichte review- en behoudcontrole.
