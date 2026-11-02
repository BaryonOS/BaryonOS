# Afzonderlijke normale Windowsbundel337

De bundelhashes, alle19 payloadbestanden en acht ingebedde ELF-apps zijn gecontroleerd tegen het geslaagde CI-artifact13164815420. Dit is de centrale337-bron, niet de idlevariant1f2. De launcher is werkelijk onder WindowsPowerShell5.1 gestart, kiestWhpx en schrijft de juiste source-v3 in launch.json. Er verschijnt één QEMUvenster; alle acht diensten bereiken INITMASK31.

De aanvullende visuele/inputcontrole is NIET voltooid: twee verse vensteropnamepogingen geven exact `executable signature is not trusted (0x800b0101)`. De normaleSDL-pixels en invoer zijn daarmee niet geverifieerd, en er is geen betrouwbare schermlatentie gemeten. Twee QEMUCLI-waarschuwingen staan in normal-launch-result.json; hun effect is onbekend. Dit is geen vastgestelde product-UI-fout en wordt ook niet als volledige runtimeacceptatie aangemerkt.

Het eigen QEMUproces is na PID/image/titlecontrole via CloseMainWindow gesloten; de oorspronkelijke starter eindigt0, zonder geforceerde beëindiging. Dit bewijst geen geordende guestshutdown of opslagduurzaamheid. Wegwerpmedia blijven onder de nieuwe lokale map; geen systeem-, driver-, firmware- of beveiligingswijziging.

De zes formele volledige Windows/WHPX native probe-runs en hun onafhankelijke exacte beeld-/invoer-/herstel-/schijfcontroles staan apart onder ../windows-ab/ en behouden hun geldigheid. Deze onvolledige extraSDL-check is niet als zevende nativepass toegevoegd.
