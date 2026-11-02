# Native toets: formaatidentiteit vóór volumetoelating

Deze map bevat uitsluitend externe testtooling, wegwerpimages, builds en logs.
`native_admission.py` implementeert geen bestandssysteemdriver. De echte
Nieuw-OS-kernel wordt uit `../source` gebouwd; testmachines gebruiken QEMU/TCG,
UEFI, één CPU, 256 MiB en geen netwerk.

## Uitvoering

Voer pas uit nadat de bron als een concrete commit is bevroren. Vervang
`<COMMIT>` door de volledige SHA van dat checkpoint:

```powershell
python -B native_admission.py build --expected-head <COMMIT>
python -B native_admission.py matrix --expected-head <COMMIT>
python -B native_admission.py regression --expected-head <COMMIT>
```

`freeze.json` bindt alle acties aan HEAD, source-fingerprint v3, de Python
volumetestbestanden en deze runner. Bestaande runlabels worden nooit
overschreven. Er is één Cargo-writer tegelijk. De bestaande vergrendelde
dependencycache wordt offline hergebruikt; er worden geen dependencies of
toolchains geïnstalleerd.

## Acceptatie van tien matrixruns

1. Een expliciet geformatteerde eigen lege fixture doorloopt `native-volume`.
   De bestaande onafhankelijke host-oracle controleert de verwachte generatie,
   documenten, collecties en domeindigests. QEMU moet werkelijk writes traceren.
2. Een nieuwe VM voert `test-volume-recover` uit op exact de eindbytes van run1.
   De recovery blijft alleen-lezen; generatie, digests en volledige bytes blijven
   gelijk. Er worden geen virtio-blk-writes getraceerd.
3. Btrfs, een volledig nulimage, een nulheader met payload op sector80 én
   sector900, en één OpenZFS-mirrorlid worden elk tweemaal aangeboden:
   alleen-lezen/recovery en schrijfbaar/native-volume. Dit zijn acht negatieve
   runs. Het schrijfbare profiel heeft READ/WRITE/FLUSH-capabilities; alleen de
   QEMU-diskflag wijzigen zou dat niet aantonen.

Elke negatieve run vereist werkelijk bootbewijs, precies startupfout
   `Corrupt/-18`, raw QEMU-exit35, geen timeout, geen APP-foutdiagnostiek, geen READY/PASS,
getraceerde reads en nul getraceerde writes. Zowel de oorspronkelijke image
als de wegwerpkopie moeten hun volledige SHA-256 behouden. De normale en koude
positieve controles vereisen exit33 en de bestaande geordende succesmarkers.

QEMU-traces omvatten alle virtio-blk-devices, inclusief de boot-ESP. Nul writes
is daarom een controle over beide virtio-disks; het readtotaal wordt niet als
een exclusief datapadcount gepresenteerd. Raw traces staan in
`runs/*/stderr-trace.txt`; onbewerkte seriële uitvoer in `runs/*/serial.txt`.
Resultaten, commando's, binaryhashes en imagehashes staan per run in `result.json`.
Het eindoordeel staat uitsluitend in `matrix.json` wanneer `passed=true`.

## Bestaande regressiepoort

`regression` roept de bestaande `tools/test_volume.py` aan en verandert alleen
de werkmap voor QEMU-paden. Bronpaden, bronfingerprints, hosttests, acceptatie-
markers en oracles blijven uit de repository komen. Alle negen native stages
worden uitgevoerd: normal, koude recovery, drie echte kill/recoverparen en de
bestaande corruptieproef. `regression/build/volume-gate.json` is het bronharnas-
oordeel; `regression/invocation.json` bindt het aan deze bron en uitvoering.

## Bewijsgrens

De vreemde bronimages zijn de vooraf bestaande regular-file-fixtures onder
`../../native-storage-gate`; hun verwachte hashes worden eerst gecontroleerd.
Zij worden niet gemount of aan de VM zelf doorgegeven. Alleen lokale kopieën
worden als virtuele disk aangeboden. Een geslaagde weigering bewijst geen
Btrfs- of OpenZFS-import, native bestandsinhoudsherstel, fysieke stroomuitval-
bestendigheid of veiligheid van echte hardware. De OpenZFS-fixture bevat DMU-
testobjectsets; dit is geen test van een normale ZPL-bestandsboom.

## Uitgevoerde resultaten

Uitgevoerd op 2 november 2026, broncommit
`94f12da0b9740a1a0ced1f08b4a30787f82ae79f`, source-fingerprint v3
`2cffba892608c907b717158a83b8a1c66c7d79dc5843d2d03eabc4ff1831c028`.
Beide profielbuilds eindigden met exit0. `matrix.json` bevat tien geslaagde
native runs; `regression/invocation.json` en de bestaande `volume-gate.json`
bevatten een geslaagde volledige regressie met negen native stages.

| Matrixgroep | Uitkomst |
|---|---|
| Eigen expliciete lege fixture → normal | PASS, exit33, generatie84, 4.963 getraceerde writes, 52,42s |
| Koude recovery op exacte normal-eindbytes | PASS, exit33, generatie84 en beide digests gelijk, nul writes, 25,20s |
| Acht vreemde/ongeformatteerde proeven | Allemaal startup-Corrupt/-18, exit35, nul writes, bron- en kopiehash gelijk, 2,02–2,91s per run |

De normale en koude eindimage hebben beide SHA-256
`19e2fdb3081ed07d54e5f42d152919606aac51ac07cd9dd079769ec011146bb3`.
De domeindigests zijn `5586531168511146659` en `16427685867521082448`.
Alle acht negatieve runs hebben 328 getraceerde virtio-reads en nul writes.
Startup-Corrupt bewijst weigering vóór beschikbaarstelling/afhandeling van
opslagverzoeken. Afwezigheid van APP-foutmarkers bewijst niet dat geen appcode
heeft gedraaid.

De volledige regressie duurde 163,02s. Alle hostchecks eindigden met exit0:
45 volume-core-tests, 32 SDK-runtime-tests en 31 Python-volumetests. Alle negen
VM-stages slaagden: normal/recover; cut-data/recover-cut;
cut-table/recover-cut; cut-txn/recover-cut; corrupt. De drie cutruns beëindigden
een werkelijk draaiende VM met hostkill/raw1; de overige zes eindigden raw33.
Er waren geen time-outs. Het bestaande bronharnas behield zijn eigen markers,
oracles, checks en gecontroleerde eindbuild.

De eerste poging stopte vóór enige build of VM door Windows-executablelookup
van een bare `rustc`-naam in deze externe runner. Die mislukte preflight en
oude runner/freeze zijn bewaard onder `preflight-failure-01/`. Alleen de
executable-resolutie is hersteld. De actieve, volledig geteste runner heeft
SHA-256 `dc5707cc6972b6b666d2e9d6925a275850d5afa9b48fa9bd68ce69d2c26175a0`.
De broncode bleef gedurende beide succesvolle builds en alle negentien
native runs op hetzelfde checkpoint staan.
