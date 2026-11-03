# Native PNG/JPEG-decodecomponent

TASK-IMAGE-CODEC-NATIVE, basis `985ed4da0ead6865c8d1fd0d4deb34ef7f97f3eb`.
Dit is een gedeeltelijke bouwsteen voor **FR-A03/UC04**, geen afbeeldingsviewer
en geen aftekening van PNG/JPEG in hun geheel. Eigen kernel/capabilities blijven
leidend; parsercode ontvangt bytes en geeft pixels terug, zonder display-,
opslag-, netwerk-, sleutel- of entropycapability.

## Dependencybesluit vóór opname

De officiële cratearchieven zijn eerst buiten de repository opgehaald en op
crates.io-SHA256, bron, licenties, features, MSRV, allocaties, unsafe en
buildscripts gecontroleerd. Daarna zijn de vijf exacte ontwikkelpins toegevoegd.
[dependency-ledger.json](../../services/image-codec/dependency-ledger.json)
legt alle hashes, broncommits uit `.cargo_vcs_info.json`, data en versies vast;
[THIRD-PARTY.md](../../services/image-codec/THIRD-PARTY.md) bewaart de MIT-notices.

| Crate | Pin / MSRV | Actieve keuze |
|---|---|---|
| zune-png | 0.5.2 / 1.87 | no_std+alloc; defaults, SSE en portable-SIMD uit |
| zune-jpeg | **0.5.16-rc3** / 1.77.1 | no_std+alloc; std/x86/neon/arith/portable-SIMD uit; expliciete ontwikkel-RC |
| zune-core | 0.5.3 / 1.75 | Bytecursor/options; geen std/log/serde |
| zune-inflate | 0.2.54 / geen opgegeven MSRV | Alleen zlib, geen gzip/std; werkelijk gebouwd met Rust1.90 |
| simd-adler32 | 0.3.10 / geen opgegeven MSRV | Defaults/std/nightly uit; unknown-none heeft geen SSE2-targetfeature en kiest scalar |

De actieve closure bevat exact deze vijf enginecrates: vier registrycrates en
één lokaal geverifieerde JPEG-vendorcopy. De interne
`zune-core feature default`-boog is leeg en activeert geen std. Geen crate heeft
een buildscript, C-/hostdecoder of downloader. Zune-crates zijn MIT OF Apache2
OF Zlib; Adler is MIT. Het inflate-cratearchief laat de volledige licentietekst
weg; de gepinde repository biedt MIT expliciet aan. Die auteursnotice en de
volledige MIT-tekst van hetzelfde project zijn daarom bewaard. Alleen de hieronder
beschreven lokale JPEG-correctie wijkt van upstream af.

Primaire bronnen: [zune-image](https://github.com/etemesi254/zune-image),
[zune-png0.5.2](https://crates.io/crates/zune-png/0.5.2),
[zune-jpeg0.5.16-rc3](https://crates.io/crates/zune-jpeg/0.5.16-rc3),
[simd-adler32](https://github.com/mcountryman/simd-adler32).
Het upstreamproject had op 24 oktober2026 recente JPEG-commits. Gerichte
RustSec-bronzoekopdrachten voor zune-png/jpeg/inflate leverden geen vermeldingen
op; dit is geen verklaring dat er geen kwetsbaarheden bestaan of volledige audit.

Stable JPEG0.5.15 is bewust **niet** gekozen: upstream
[#409](https://github.com/etemesi254/zune-image/issues/409) en
[#382](https://github.com/etemesi254/zune-image/issues/382) tonen scalar/progressive
panics en [#496](https://github.com/etemesi254/zune-image/issues/496) verkeerde
kleuren, met fixes in nieuwere bron. De gekozen RC is geen productiegoedkeuring.
Open upstreamvarianten blijven zichtbaar: APP14/grijs
[#444](https://github.com/etemesi254/zune-image/issues/444), DRI-interval
[#509](https://github.com/etemesi254/zune-image/issues/509), YCCK
[#442](https://github.com/etemesi254/zune-image/issues/442). Hun impact moet per
ondersteund profiel met echte fixtures worden opgelost vóór formaatacceptatie.

## API, formaten en resourcegrens

`services/image-codec` is een zelfstandige no_std-workspace. `decode` krijgt
onveranderlijke geselecteerde objectbytes, expliciete `Limits` en annulering.
Een geslaagde uitkomst bevat owned `Rgba8` of **owned `Rgba16`** en behoudt een
borrow naar alle originele bytes inclusief metadata; er vindt geen re-encode
of wijziging van het object plaats. `rgba8_into` maakt een gecontroleerde
weergaveconversie met afronding naar acht bits; volle samples blijven behouden.

PNG gebruikt de onderhouden `decode_into(&mut[u8])`-API met big-endian samples.
De wrapper bouwt u16-waarden veilig iteratief op. Daardoor wordt de bekende
`convert_u16_to_u8_slice`-aliascast uit upstreams andere `decode()`-pad niet
uitgevoerd; de engine wordt niet onterecht als volledig unsafe-vrij beschreven.
Filters, palette/tRNS, DEFLATE, CRC en Adam7 blijven in de onderhouden engine.

De PNG-resource-envelop controleert alleen chunkgrenzen/aantal, afmetingen,
metadatahoeveelheid en verboden animatiepad. iCCP en zTXt worden vooraf door
de onderhouden inflater met beperkte output gecontroleerd: upstreams eigen
metadata-inflater gebruikt anders een 1GiB-default. CRC+Adler en strict mode
staan aan. De inflaterlimiet is best effort en kan tijdelijk overshoot/padding
vragen; zij vervangt **niet** de harde procesarena. JPEG begrenst afmetingen,
pixels, scans en werktoelating vóór de pixeldecode. Inputlengte begrenst metadata.

De wrapper reserveert eigen output fallible, maar upstream heeft infallible
allocaties. Daarom moet de caller een apart parserproces met begrensde private
arena gebruiken. OOM/panic daarin is terminaal; geen hostallocator of gedeeltelijk
beeld wordt als succes gepubliceerd. Een echte native OOM-/herstart-/peerisolatie-
proef blijft open. De huidige 8MiB-procesquota en 256MiB-allocatorpool worden
niet tot productbeleid verheven; grote foto's blijven een expliciete producteis.

| Profiel | Werkelijk getoetst / huidige grens |
|---|---|
| PNG | RGB8, RGBA8/transparantie, palette4+tRNS, grayscale1, gray-alpha8, Adam7RGB8 en RGBA16 met exact behoud van samples. Geen claim dat alle bitdiepte-/kleuren-/metadata-combinaties afgetekend zijn. |
| JPEG | Huffman baselineRGB4:4:4, progressiveYCbCr4:2:0 en grayscale8; pixels worden werkelijk gedecodeerd. Geldige EXIF-orientatie 1–8 stuurt de displaypixels van `rgba8_into`; zie de metadataregel. De oorspronkelijke progressive-kwaliteitsfout is hieronder rood→groen bewezen, zonder drempelwijziging. |
| Nog open | PNG2/andere combinaties, APNG-compositie/animatie, CMYK/YCCK, arithmetic/lossless/high-precisionJPEG, DRI/APP14 en verdere corpusdekking. `UnsupportedAnimation/Color/Precision` is een expliciete implementatiegrens, geen verborgen hardwaretoestand. |
| Metadata | Originele bytes inclusief EXIF/ICC en ruwe decoderpixels blijven onveranderd; bronafmetingen en `pixel_hash` blijven in decoderorde. Voor JPEG past `rgba8_into` geldige EXIF-orientatie 1–8 toe op de displaypixels en -afmetingen. Een EXIF-APP1-payload groter dan `Limits::metadata_bytes` weigert het beeld met `Error::MetadataLimit`, zonder terugval naar ontbrekende orientatie. Ongeldige EXIF-structuur of -waarden worden genegeerd (geen orientatie). `metadata_transforms_applied=false` beschrijft de ruwe samples; ICC/gamma-kleurbeheer en PNG eXIf blijven buiten scope. De EXIF-tag volgt CIPA DC-008-2012, *Exchangeable image file format for digital still cameras: Exif Version 2.3* ([primaire specificatie](https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf)). Zoom/pan/export/editor moeten dit later werkelijk integreren. |
| Annulering | Vooraf/PNG-chunkgrenzen/na decode; JPEG heeft daarnaast de echte upstream MCU-checkhook. Native proef toetst vooraf annuleren, geen gemeten mid-decode-annuleringslatentie. PNG-mid-decode vereist nog procesannulering of onderhouden incremental pad. |

## Stackgrens en eigen arena

**Implementatie:** Nieuw OS, *image-viewer: render_tile*, versie
`d269f9b4` (3 november 2026), tekent nog de ruwe decoderpixels
([primaire rendererbron](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/sdk/examples/image-viewer/src/lib.rs)).
EXIF-correctie in `rgba8_into` bewijst daarom geen georienteerde viewer of
native gebruikersflow.

**Testbewijs:** Nieuw OS, *viewer-fixture en kleine image-integratieslice*,
versie `d269f9b4` (3 november 2026): de fixture is voor de nieuwe
`Image`-velden bijgewerkt en afzonderlijk meegetest
([primaire testbron](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/sdk/examples/image-viewer/tests/viewer.rs),
[commando's en resultaten](https://github.com/simonvanieperen/nieuw-os/blob/d269f9b45a54576b39629b36de94f9cdbb9f6f71/docs/verification/integration-sprint-20261103-image.md)).

Een eerste instructiescreen zag alleen losse stackaanpassingen tot4KiB. Dat
was onvoldoende: het werkelijke decoderframe reserveerde cumulatief30760bytes.
Samen met een statisch bereikbare scan-/DHT-keten was77232bytes zichtbaar,
meer dan de huidige64KiB-stack. Dit was een codegeneratiebevinding, geen
uitgevoerde stackfault en geen volledige branchtrace.

De eigen wrapper construeert de grote JPEG-decoder nu in een afzonderlijke
`#[inline(never)]`-helper en verplaatst hem naar een owned Box-slice in de
private arena. De allocatie gebruikt eerst het stabiele
`Vec::try_reserve_exact(1)`; capaciteit1 wordt gecontroleerd zodat de daaropvolgende
Box-conversie geen shrinkallocatie nodig heeft. Een constructieallocatiefout
is fallible; latere infallible upstreamallocaties blijven terminaal bij OOM.
PNG/JPEG-stackframes blijven apart. Kernelstack en procesquota zijn niet verhoogd.

[stack-codegen.json](../../services/image-codec/verification/stack-codegen.json)
legt werkelijk none-ELF en LLVM-IR vast: dispatchframe8bytes, JPEGcaller616,
constructiehelper30216. Het30056-byte-objectslot staat uitsluitend in die helper,
die terugkeert vóór het parserwerk. Dezelfde onderzochte DHT-callchain reserveert
nu47104bytes, circa18KiB onder64KiB. De gastentry reserveert daarnaast248bytes.
Een afzonderlijke statische progressive→marker→Huffman-keten staat ook in het
rapport:17664bytes vóór guestentry/overige niet gemodelleerde frames; de echte
progressivefixture bevat DHT-markers na de eerste SOS. Dit blijft statische
bereikbaarheid, geen uitgevoerde branchtrace of volledige maximale callchain.
Dit is geen maximale-stackbewijs voor alle branches of native OOM-/guardproef;
de echte VM-uitvoering blijft verplicht. Alle beeldhashes blijven ongewijzigd.

De volgende echte VM op `9aa74870de55387094414c3ec1a909fbeec53c81` vond een
andere, eerder niet meegenomen keten: PID5 kreeg een pagefault in
`capture_scan_header_state`, RIP `0x40118aaa`, adres `0x7ffee9e8` onder de
stackguard `0x7ffef000`. De grote inline snapshot plus clone reserveerde daar
ongeveer60KiB vóór de callerframes. De oorspronkelijke fault blijft rood bewijs;
de eerdere geselecteerde DHT-ketens waren geen volledige maximale-stackanalyse.

De tweede lokale vendorpatch bewaart nu exact vier DC- en vier AC-tabelslots,
inclusief `None`, in fallible gereserveerde Vec-buffers. Vastgelegde quantisatie,
arith-conditioning/statistiekvelden, restartinterval, kleurenruimte, Adobe en
mjpegstatus blijven behouden. Capture en beide replaykopieën geven bij een
mislukte reservering `AllocationFailed`; de wrapper geeft `OutOfMemory`.
Alle nieuwe reserveringen slagen vóór hun lokale parser-/checkpointmutatie.
`headers_decoded` wordt pas na succesvolle snapshot gepubliceerd. Herstel
kopieert afzonderlijke tabellen; EOF-/annuleringsrollback en SOS-checkpoints
behouden hun bestaande gedrag. Er is geen nieuwe unsafe of codec-algoritme.
De bestaande kleine `Box::new`-owners en overige upstreamallocaties zijn hiermee
niet alle fallible gemaakt; native OOM-isolatie blijft apart verplicht.

[scan-snapshot-codegen.json](../../services/image-codec/verification/scan-snapshot-codegen.json)
bindt de nieuwe ELF en geselecteerde cumulatieve prologen/callpaden. Dit blijft
statisch codegenbewijs, geen uitgevoerde stack- of maximale-callchainproef.
De arena blijft4MiB, de gebruikersstack64KiB en het native fixture-/hashcontract
ongewijzigd. De volgende geïntegreerde VM moet het echte decoderen nog aftekenen.

## Uitgevoerd bewijs en echte rode kwaliteitsvraag

Fixtures zijn eigen patronen met uiteenlopende maten, geen vooraf ingevuld
succesbeeld. [generate.py](../../services/image-codec/fixtures/generate.py) is
uitsluitend een hostencoder/orakel; Pillow12.3/libjpeg en bekende16bitsamples
leveren onafhankelijke pixels. De OS-code bevat geen Pillow/std/hostdecodepad.
Het manifest legt input-/referentiehashes vast. De PNG16-preview rondt full16
samples; Pillow's achtbits PNG-view kapt af. De full16-referentie komt daarom
uit de bekende bronwaarden van de fixtureencoder en wordt sample voor sample
gecontroleerd; Pillow leest daarnaast de echte PNG-file.

Het oorspronkelijke [rode rapport](../../services/image-codec/verification/host-tests-before-h2v2.json)
blijft bewaard op checkpoint5910204: 10 groepen,8 groen/2 rood,exit101. Beide
rode groepen bevatten dezelfde progressive4:2:0-vergelijking: maximaal4 in plaats
van de vooraf vastgelegde3, twee van87236 kanalen, beide blauw op(176,17)/(175,18).
Totale absolute afwijking27763; kwadratische afwijking43153. De grens blijft3.

De causale hostdiagnose leest echte componentplanes uit de ongewijzigde decoder.
Pillow/libjpeg-turbo**3.1.4.1** levert via JPEGdraft(YCbCr, oorspronkelijke grootte)
een onafhankelijke referentie, zonder RGB-roundtrip. Het oude twee-stapsmodel
reproduceert alle65427 oorspronkelijke YCbCr-bytes exact. De afwijkende pixels
hebben hetzelfde Y en een Cb dat2 hoger is; dat verklaart de blauwe afwijking4.
De standaard9:3:3:1-interpolatie met één afronding reduceert deze chromafout.
[Componentbewijs](../../services/image-codec/verification/jpeg-components.json)
behoudt de meetwaarden; de hostdiagnostic is uitsluitend hostgereedschap.

De lokale correctie verandert **alleen scalar `upsample_hv`**. Zij drukt de
wiskundige trianglegewichten en afwisselende8/7-tiebias uit in eigen veilige
Rust-code; er is geen C-bron overgenomen. Primaire referentie:
[libjpeg-turbo jdsample.c, commit9217719d…](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/9217719d3a58633923b096af4c1d50d304768a64/src/jdsample.c#L362).
Er wordt geen upstreamacceptatie geclaimd. Het originele cratearchief, SHA,
[exacte kleine diff](../../services/image-codec/vendor/upstream/upsample-hv.patch)
en [alle bronhashes](../../services/image-codec/vendor/upstream/source-verification.json)
blijven bewaard. Deze eerste patch wijzigt alleen `upsample_hv`. De afzonderlijke
[snapshotpatch](../../services/image-codec/vendor/upstream/scan-header-heap.patch)
wijzigt `decoder.rs`, `mcu.rs` en de expliciete allocatiefout in `errors.rs`;
de overige37 upstreambestanden en alle licentieteksten zijn byte-identiek.
Alleen deze letterlijke upstreambron en de unified-diff-contextregels hebben
een lokale whitespace-stijluitzondering; zij worden niet herschreven om
opmaakwaarschuwingen weg te poetsen. Wrapper/probe/tests blijven normaal getoetst.
`verify_vendor.py` controleert die volledige bestandsset, archivehash, patchhash
en ongewijzigde upsamplerprefix/-suffix vóór iedere native build; de tweede
patch moet de exacte unified diff van uitsluitend haar drie bronbestanden zijn.
Vier hostproeven weigeren gewijzigde bron/archive, een extra buildscript en
een gewijzigde of niet met de bron overeenkomende snapshotpatch.

Vijf nieuwe hostproeven injecteren echte allocatiefouten in de productie-engine,
controleren beide publieke OOM-foutpaden en hergebruiken dezelfde decoder na
snapshot-/checkpointfouten. De synthetische
[drie-SOS-fixture](../../services/image-codec/fixtures/scan-rollback/oracle.json)
met DQT/DHT/DRI/COM/APP-wijzigingen is uitsluitend testdata; Pillow/libjpeg dient
als onafhankelijke hostreferentie. EOF en annulering tussen scans moeten herstel
met exact dezelfde pixels toelaten. Deze extra hostfixture verandert de tien
native invoerbeelden, de kwaliteitstolerantie of de verplichte formaatomvang niet.

Het [oorspronkelijke rapport](../../services/image-codec/verification/host-tests.json)
bevat **14 geslaagde testgroepen,exit0**:11 decodegroepen plus3 groepen op de
werkelijk gepatchte upsampler. PNG-pixels en volle16bitsamples matchen exact.
BaselineJPEG maximaal3,grayscale1,progressive nu3; progressive totaalabs1375 en
kwadratisch1903. Dezelfde eerder rode tests blijven actief. Twaalf aanvullende
echte baseline/progressiveJPEGs toetsen1×1,1×17,17×1,3×5,5×3,17×19; directe
upsamplerproeven toetsen minimum-/maximumsamples, oneven breedtes, tiebias,
capaciteitstaarten en ongeldige geometrie vóór enige outputwrite.

Andere proeven toetsen echte decompressiebom, truncatie, CRC, afmetings-/input-/
werkgrenzen, cancel, onveranderde bron/output bij verkeerde capaciteit en
hergebruik na fouten; 512 deterministische malformed mutaties veroorzaken
geen panic. Dit is geen onbeperkte fuzz-/securityaudit. Clippy met `-D warnings`
is uitgevoerd en slaagt. None-check en een echt statisch native ELF zijn gebouwd.
Miri is niet uitgevoerd; deze proeven vervangen geen volledige memory-safety-audit.
**Bij componentcheckpoint `9ae1c73` was QEMU-uitvoering nog niet uitgevoerd.**
De eerste geïntegreerde imageproef op bron `2b5d69c69eaf4196eb00a8488c88273defe14770`
stopte met raw exit35 en errorstep1 vóór decode: startupword0 is de bootmodus,
niet de ABI-versie.
De probe vraagt de ABI nu via `native::check_abi()` aan de lopende kernel;
de oude mislukte proef blijft bewaard. Een nieuwe native decodeproef is nog open.

## Native procescontract voor integratie

`sdk/examples/image-probe` bouwt met dezelfde Rust1.90 en eigen targetmap:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-core-target \
  python3 -B tools/dev.py cargo +1.90.0 test --locked --offline \
  --manifest-path services/image-codec/Cargo.toml -- --nocapture
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-image-probe-target \
  python3 -B sdk/examples/image-probe/build.py
```

De eerste opdracht slaagt na de lokale afrondingscorrectie; het oorspronkelijke
exit101-rapport en de ongewijzigde kwaliteitsdrempel blijven bewaard.
De tweede geeft `extra_env.NOS_IMAGE_PROBE_ELF`, ELF-inspectie en SHA256; dit
betekent bouwen, niet native succes. De root koppelt later opt-in
`native-system-images=[native-system,process-memory,fpu-context]`, ABI1.7,
Sys3/Sys4 en uitsluitend een bootmanifest-slot4 zonder extra caps. `_start`
doet CLD+JMP en behoudt de loader-SysV-stackalignment. De guest vraagt zelf
1024 verse heap-pagina's en draagt die uitsluitend over aan UserAllocator.
Kernel/harnas leveren geen heap, gedecodeerde pixels of antwoordsucces.

| Startupoffset | Native feit / verwachte pariteit |
|---|---|
| 40/240 | done1 / errorstep0 alleen na alle assertions |
| 400/408 | PNG8 (zevenvarianten+herdecode na fouten), JPEG3 |
| 416/424 |149647 werkelijk verwerkte pixels /10 gecontroleerde negatieve uitkomsten |
| 432/440 | Werkelijke FNV-1a64 pixelaggregatie `079d5118b890ae14` /623808 decoded samplebytes |
| 448/456/464 |1024 heapgrowthpagina's /werkelijke arena-highwater>0≤4MiB /nul resterende allocaties |
| 472/480/488 |1 cancel /werkelijke monotone start-/eindticks |
| 496 |**QUALITY_OPEN=0** voor de opgeloste specifieke JPEG-orakelvraag; algemene formaat-/vieweracceptatie blijft open |

FNV is uitsluitend een paritychecksum, geen cryptografische authenticiteitsclaim.
PNG-checksums corresponderen met onafhankelijke exacte samples. JPEG-checksums
vergelijken dezelfde engineconfiguratie tussen host en guest; hun onafhankelijke
beeldkwaliteit blijft apart getoetst. De rootobserver moet de werkelijke boot-PID
bewaren, zodat latere workerslotreuse geen verkeerde identiteit accepteert.
Daarna volgen exacte bron/imagebinding en cold jointregressie, echte OOM en
procesannulering, objectRPC en compositor-/Materiaal-integratie. Een fullscreen
preview, ELF of losse decode-proef tekent FR-A03 niet af.
