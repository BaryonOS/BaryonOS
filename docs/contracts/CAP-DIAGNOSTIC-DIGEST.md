# Capabilitydiagnostiek — bitexacte varianten en native tegenmeting

## Actuele uitkomst: hostwinst voorspelde de TCG-uitkomst niet

De eerste nulreeksvariant staat in db04c385ee8a19c45594b267f5dfa654a9b8919e.
De hostratio van circa2,48 hieronder is werkelijk gemeten, maar de daarop
geïntegreerde GUI-proef werd trager en faalde. Dit is geen native
performanceverbetering of afgetekende GUI-acceptatie. Broncommit
1fd879dfdde88890a68f0a23901c69acd464b582, bronfingerprintv3
6f9410a38d6cc1323344489c19c57f4e59c5a858e41f8e0710d7792065081fd3:

- De integrator meldde170,0707855s tot de eerste Ready-capture met1608 tegels,
  tegenover circa106s bij de eerdere variant. De170,07s komt uit het verschil
  tussen capture- en inputmanifest-mtime en omvat meer dan alleen digestwerk.
- Het echte resultaat is `passed:false`, rawQEMUexit35, guest_timeouttrue en
  duration203,591759599s. De gast bereikte bij zijn deadline1932 tegels en één
  volledig frame; geen Saved-/STOP-/cleanup-acceptatie.
- Het resultaat is zelf gelezen op
  `/workspace/nieuw-os-worktrees/root-materiaal/build/materiaal-digest/run/materiaal-normal-230683c431/result.json`,
  SHA25668c5cf5dd67d9b63c8b1c03a29bce9ab9a488d33fe1380eac52c9fa9c4504b28.
  De timeout200s is een gastgrens; de geregistreerde hostwatchdog is240s.
  Dat rapport bevat de exacte kernel-, ELF-, firmware-, QEMU- en diskhashes.

PERF-NATIVE-BENCH-1 voegt daarom uitsluitend een expliciet meetprofiel toe.
De gewone productie-`state_digest`, `stats`, rechten en alle voor-/na-checks
van iedere syscall blijven bytegelijk aan db04. Er wordt in deze stap geen
nieuwe productievariant gekozen en de GUI-timeout wordt niet verhoogd.

## Opt-in native meetcontract

Cratefeature `capability-core/diagnostic-bench` bouwt twee extra read-only
methoden: `Engine::diagnostic_reference_digest()` en
`Engine::diagnostic_direct_digest()`. De eerste gebruikt de letterlijk
behouden oude body uit3f20; de tweede verschilt daarvan alleen doordat de
40 Message-wirewoorden direct worden gehasht in plaats van via encode().
Ieder woord krijgt onvoorwaardelijk dezelfde XOR/multiply; er is geen
nuldetectie, zerorun of selectie op live/len. Alle bestaande differentiële
corpora toetsen nu ook deze directe variant. Zonder de feature of cfg(test)
worden beide extra paden niet gebouwd. Geen defaultfeature of dependency.

De nieuwe module `kernel/src/m2/m3/cap_bench.rs` levert
`pub(super) fn run(engine: &capability_core::Engine) -> !`.
Root bezit de moduledeclaratie, featurealias, hook, builder en runner:

```text
native-cap-digest-bench = [native-system, capability-core/diagnostic-bench]
State::setup();
cap_bench::run(m4::engine()); // vóór timer_init, IF=0, één CPU
```

Dit gebruikt de werkelijk opgebouwde vier-processenmanifestgraaf en de
bestaande kernelhelpers `clock()->u64`, `print_number(&[u8],u64)` en
`serial::{write,exit}`. Er wordt geen tweede Engine gemaakt en geen proces
uitgevoerd. De manifestassertie eist4 processen,21 handles,29 nodes,
9 open endpoints,0 queued messages,1 block-,1 launch- en1 entropyobject,
0 process-/network-/display-/inputobjecten en geldige structurele invarianten.
De8 gesloten parentnodes blijven aanwezig wegens de afgeleide nodes.
De normale manifestdigest is0x31b7de1c7a15245b,
decimaal3582576242485175387; het programma berekent de werkelijke waarde.

De vaste drie varianten zijn REF, DIRECT en ZERO. Voor iedere variant:
één prehashcontrole,4 warmupcalls,144 gemeten calls enéén posthashcontrole.
Gemeten wordt9 rondes×16 calls; de volgorde roteert ABC/BCA/CAB enzovoort.
Iedere aanroep gebruikt `black_box` voor de functiepointer, Engine-ref en
het resultaat. Iedere uitkomst telt mee in een verschilaccumulator en wordt
na de batch met de verwachte hash vergeleken. Er staat geen Stats- of serial-
bewerking in het gemeten interval. Alle Statsvelden vóór en na zijn exact
gelijk vereist; de volledige structurele invarianten worden opnieuw getoetst.
Geen actor-/devicebewerking, allocatie, Engine-clone of unsafe in de meetmodule.

De bestaande clockhelper gebruikt **ongeserialiseerde RDTSC**. De resultaten
zijn grove QEMU/TCG-timestampverschillen per batch, inclusief lus-, indirecte
call-, black_box- en pariteitsrekenwerk. Het zijn geen fysieke CPU-cycli,
exacte per-calllatenties, hostwallclock- of realtimegaranties. De runner
registreert QEMU-versie/profiel en de werkelijke totale duur. Doel voor deze
kleine proef is minder dan5s gastmeetwerk; dat is nog geen gemeten feit.
Het aantal wordt niet achteraf aangepast om een gewenste winnaar te krijgen.

De exacte outputvolgorde is:

1. `CAPBENCH:MANIFEST:PASS`.
2. Numerieke regels `CAPBENCH:{NAAM}:` met20 decimale cijfers:
   DIGEST,PROCESSES,HANDLES,NODES,ENDPOINTS,WARMUP,ROUNDS,BATCH.
3. REF_COUNT,REF_TSC_SUM,REF_TSC_MIN,REF_TSC_MAX; dezelfde vier met DIRECT_
   en daarna ZERO_. COUNT is144 gemeten aanroepen, SUM is de som van9
   batchintervallen, MIN/MAX zijn de uiterste **16-call-batchintervallen**.
4. Per ronde drie ROUND_VARIANT-regels met de werkelijk uitgevoerde indices
   (0=REF, 1=DIRECT, 2=ZERO), gevolgd door ROUND_REF_TSC, ROUND_DIRECT_TSC
   en ROUND_ZERO_TSC in vaste variantvolgorde, voor rondes0..8. De host
   controleert de rotatie, som, minimum en maximum en bewaart de volgorde.
5. `CAPBENCH:PARITY:PASS`, `CAPBENCH:STATE:PASS`, `CAPBENCH:PASS`, `M1:PASS`;
   exit0x10 geeft rawQEMU33. Iedere afwijking geeft CAPBENCH:FAIL en exit0x11
   (raw35), inclusief hash-, staat-, manifest-, tijdprogressie- of somoverflow.

Het profiel krijgt een echte lege wegwerpdisk en virtio-blk/rng zoals de
systemmanifestbouw vereist. Tijdens de meting draaien geen gebruikersapps,
disk-/entropy-syscalls of andere actoren. Een groene CAPBENCH is daarom
uitdrukkelijk **geen** functionele app-, GUI-, Saved-, entropy-, disk-,
cleanup-, beveiligings- of releasetest. De integrator selecteert pas na echte
meetresultaten een volgende productievariant en herhaalt dan de volledige
GUI-proef met ongewijzigde syscallcontroles.

### Componentcontroles van het meetpad

Zonder feature:52 tests geslaagd,1 benchmark ignored. Met diagnostic-bench:
53 tests geslaagd,1 ignored; de extra test bouwt dezelfde manifestgraaf,
controleert alle drie hashes en vergelijkt de volledige Engine én Stats vóór
en na. Beide sets behouden alle47 oorspronkelijke coretests. De body van de
oude referentie is bytegelijk aan3f20 gebleven, SHA256
29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed.

Eigen controles onder `build/cap-native-checks/`: fmt, beide hosttestsets,
Clippy/all-targets met feature, scalar-none releasebuild/Clippy met én zonder
feature: alle exit0. Een afzonderlijk no_std compileharnas typecheckt de echte
cap_bench-module voor scalar-none met externe, nooit uitgevoerde
clock/serial-symbolen; build en Clippy exit0. Dat compileharnas is geen native
meetuitvoering. De gewone productie-rlib bevat geen reference/direct-symbolen.
De eigen componentbronfingerprintv3 is
8f59dbbc1169ef8b78af3c57f961c0bab518e7b129e918eeb2dddc78d823d49b.
Native bron-/imagehashes en meetuitkomst volgen uitsluitend uit roots runner;
deze componentcheck vult die niet in.

## Historische eerste optimalisatie en hostonderzoek

Dit wijzigt uitsluitend de berekening van de bestaande diagnostische
`Engine::state_digest` in `crates/capability-core/src/lib.rs`. Statsvelden,
autoriteit, foutvolgorde, PID-/slot-/nodegeneraties, queues, ABI en limieten
blijven gelijk. Er is geen digestcache, dirtyflag of selectie op live/len.
De kernel houdt zijn volledige voor-/na-digestcontrole bij iedere syscall,
ook op EAGAIN. Native GUI-doorlooptijd blijft afzonderlijk te meten; hosttijd
is daarvoor geen bewijs. Geen releasecriterium of score wordt hiermee afgetekend.

## Dezelfde volledige woordstroom

Basis is commit3f20f087a7f0ddf4757af8d3ea110dd0c044a72c. De bestaande mixer
start met0xcbf29ce484222325 en voert voor elk getypeerd woord uit:
`h = (h XOR woord) * 0x100000001b3 modulo2^64`.
De volgorde en het aantal woorden blijven exact gelijk, voor iedere geldige
Rust-veldwaarde, inclusief ongeldige logische toestanden. Compilerpadding was
geen deel van de oude getypeerde digest en wordt ook nu niet geïnterpreteerd.
Deze diagnostiek is geen cryptografische hash, authenticatie of bewijs van
botsingsvrijheid. Engine-equality en daadwerkelijke capabilitycontroles blijven
hun eigen rol houden.

Iedere Message levert dezelfde40 little-endian wirewoorden als encode():
senderPID, tag, `u64(len) | (u64(cap_count)<<32)`, reserved, beide volledige
cap-handle/rights-paren en alle32 payloadwoorden. Er wordt nergens op len of
cap_count gesliced en geen ongebruikte byte als impliciet0 behandeld.
Queued-cap-IDs worden daarna zoals voorheen apart gehasht. Hiermee verdwijnen
de1024 tijdelijke320B-encodes per digest zonder een veld of byte over te slaan.
QUEUE_CAPACITY is16, dus64×16=1024 messages en327680B tijdelijke encode-output
per oude digest; een dubbel gecontroleerde EAGAIN had655360B alleen hieraan.

Een40-woordenbericht mag alleen als nulreeks worden toegevoegd nadat ieder
header-/capveld én ieder payloadwoord werkelijk0 blijkt. Bij een afwijking
worden alle40 waarden normaal verwerkt, ook bij een inactive endpoint,
len0, cap_count0, niet-canonieke reserved/unused caps of niet-nul trailingdata.

## Exacte nulreekscompositie

Voor een nulwoord is `(h XOR0)*P = h*P`. Daarom is een reeks van k werkelijk
getoetste nulwoorden exact `h*P^k modulo2^64`. De private DiagnosticHasher telt
zulke woorden. Vóór ieder niet-nulwoord wordt de volledige factor toegepast,
pas daarna XOR en multiply van dat niet-nulwoord. Finish verwerkt de laatste
reeks. Er worden dus geen nulwoorden weggelaten uit het resultaat.

Een compile-time tabel met65 u64-factoren (520B, P^0..P^64) behandelt korte
reeksen. Langere reeksen krijgen dezelfde wrapping-factor via binaire
exponentiatie. Die onverkorte arbitrary-state-fallback staat afzonderlijk
cold/noinline, zodat zij niet in iedere hot-path-woordbewerking wordt gekopieerd.
Ongeldige volledig nul gemaakte tabellen blijven door de volledige fallback
onderzocht. De vaste maximale stroom bevat74569 woorden:1 lastPID,
16×(2+256×4) processen/slots,1024×12 nodes,64×(8+16×44) endpoints/queues,
4×9 block-,8×7 launch-,16×9 process- en10×6 console/entropy/networkwoorden.
De lokale usize-nulteller kan onder deze vaste grenzen niet overlopen.

Geen heap, unsafe, raw cast, wire-/structlayoutwijziging, rechtenverruiming of
nieuwe afhankelijkheid. Alleen de lokale hashberekening verandert; de engine
blijft read-only geobserveerd. De bestaande serialisatie van enginebewerkingen
blijft vereist. Er wordt geen nieuwe SMP-/constant-time-/formele-verificatie-
garantie geclaimd.

## Referentie en differentiële controle

De oude state_digest-body is letterlijk uit3f20 overgenomen als
`state_digest_reference`; alleen de functienaam is anders. De
body wordt bytegewijs vergeleken met de bronversie, met opgeslagen SHA hieronder.
Reguliere productiebuilds bevatten deze referentiefunctie niet. Het expliciete
diagnostic-bench-profiel hierboven schakelt haar wel in, naast cfg(test).

Naast alle47 bestaande coretests zijn vijf actieve testgroepen toegevoegd:

- Alle320 wirebytes×8 afzonderlijke bits×5 hashstartwaarden, plus256 volledig
  gemengde berichten; ook verboden sender/reserved, capvelden buiten count,
  grote len/count en iedere trailing-payloadbyte blijven meedoen.
- Exacte nulreeksfactoren rond0/1/40/64/65 en tot de74569 woorden van een
  volledig nul gemaakte engine, plus10000 gemengde nul-/niet-nulwoorden.
- Echte registraties, queuevol/wrap, EAGAIN-equality, captransfer/revoke,
  alle objectklassen, gerapporteerde processterfte, close en exit. Na de
  relevante stappen worden oude/nieuwe digest en Stats-digest vergeleken.
- Drie deterministische dichte corruptiecorpora over ieder veld van iedere
  tabel, inclusief inactive queues, ongeldige indices/counts/scopes/generaties
  en alle messagebytes. De digest derefereert zulke IDs niet.
- Een volledig logischerwijs ongeldige nultoestand plus afzonderlijke
  wijzigingen in eerste/middelste/laatste inactive queues en privatevelden,
  met vergelijking en waargenomen verandering van de diagnostische waarde.

De zesde nieuwe test is standaard ignored en draait alleen expliciet als
hostmicrobenchmark. Hij vergelijkt de ongewijzigde referentie en productie-
variant op dezelfde7-processen/21-endpoint-engine met std::Instant/black_box.
Dat is kosten-/ratioonderzoek op de host, geen native OS-performanceproef.

## Metingen en reproduceerbare controles

De eerste losse benchmark gebruikte500 calls×5 samples per toestand en
nam de mediaan. Canonieke hostmetingen gebruiken de normale x86_64-hosttarget,
Rust1.90.0 release, codegen-units1, LTOuit. Een eerdere verkennende hostrun met
SSE uit gaf een Rust-host-ABIwaarschuwing; die is geen canonieke vergelijking.
Alle genoemde benchmarks zijn integercode; de freestanding scalarbuild staat
hieronder apart. Gedeelde hostbelasting veroorzaakt zichtbare spreiding.

| Variant | GUI-achtige digest µs | GUI-achtige Stats µs | Volle1024-queue digest µs |
| --- | ---: | ---: | ---: |
|Oorspronkelijk|87,323|90,946|89,560|
|Alleen typed wirehash|89,492|104,281|86,323|
|Alleen40-nulwoordenbericht samenvoegen|45,282|49,890|85,577|
|Algemene nulreeksen, cold langepad|40,439|39,416|76,290|

De geselecteerde variant helpt ook het dichte corpus; niets wordt verborgen om
alleen het lege geval gunstig te maken. Exacte digestwaarden bleven gelijk:
empty55dcd2dffcdca9c3,7-processen5ff47a6aeb481b3d,
volle queues0e68e4b80f5acd13. De tempencode-verwijdering gaf geen aangetoonde
native winst; de werkelijk rode GUI-proef staat bovenaan. Root moet op de
exact geïntegreerde commit/image de GUI200s-
gate, echte invoer/frames en de relevante native regressies opnieuw uitvoeren.

Gerichte commando's, eigen CARGO_TARGET_DIR en beheerde LOCALAPPDATA/PATH:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test --offline --locked -p capability-core
cargo +1.90.0 clippy --offline --locked -p capability-core --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked --release -p capability-core digest_differential::host_microbenchmark_reference_and_optimized -- --ignored --nocapture
cargo +1.90.0 build --offline --locked --release -p capability-core --target x86_64-unknown-none
cargo +1.90.0 clippy --offline --locked --release -p capability-core --target x86_64-unknown-none -- -D warnings
```

De none-build gebruikt dezelfde scalar flags als de kernel: code-model=kernel,
relocation-model=static,no-redzone en SSE/SSE2/SSE3/SSSE3/SSE4.1/SSE4.2/AVX/AVX2uit,
met expliciete rootpadremap. De geproduceerde digestfunctie wordt op FP/SIMD,
Message.encode-aanroepen en aanwezigheid van de testreferentie geïnspecteerd.
Logs en voorlopige benchmarkharnassen staan uitsluitend onder de eigen
build/target-paden; onderstaande compacte bron-/testbinding blijft in Git.

### Vastgelegde componentcontrole

Op de bevroren code vóór/na de controles bleef bronfingerprintv3 gelijk:
`cc6dc31a7365704210828457e97e2169ec0d4c838fc6f29f85ab0d0ff5ae7410`.
Het gewijzigde lib.rs heeft SHA256
`4c48675b247e4a9f6076c228fa610ade884f235de373b4a91132286c144f24ea`.
De bytegelijk behouden referentiebody uit3f20 heeft SHA256
`29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed`.
De scalar-none-rlib heeft SHA256
`8c60a3727f7cc960d3822404d402288d6303df94b6610482eb9d4cde7056555f`.

Alle bovenstaande gerichte commando's: exit0. De normale testloop meldt
52passed,0failed,1ignored: alle47 bestaande tests plus vijf nieuwe groepen.
De expliciet gestarte ignored benchmark meldt1passed. Zijn laatste7-samples-
vergelijking op dezelfde7-processen/21-endpoint-engine:

```text
reference=true median_ns=88906 digest=5ff47a6aeb481b3d samples=[87197,87438,88384,88906,89193,89213,94514]
reference=false median_ns=35894 digest=5ff47a6aeb481b3d samples=[34111,35271,35858,35894,35981,38187,38539]
```

Dat is een waargenomen hostratio van circa2,48; geen native GUI-/productbudget.
De daadwerkelijk gegenereerde none-digestfunctie bevat geen FP/SIMD of
Message.encode-aanroep. De referentiefunctie komt niet voor in het productie-
archief. De scalar state_digest-symbolgrootte is11929B; de afzonderlijke
cold langepadfactor64B. Dit is een gerichte reken-/codeomvangafweging, geen
claim van een kleinere volledige kernel of formeel bewezen implementatie.

Nog uit te voeren door de integrator: onafhankelijke patchreview, nieuwe
geïntegreerde kernel/image met eigen hashes, de volledige GUI200s-proef en
relevante native regressies. Bij onvoldoende verbetering wordt de echte
bottleneck verder onderzocht; controles of verplichte functies worden niet
uitgezet om een groene uitslag te produceren. De volledige testrelease blijft
open en deze component krijgt geen zelfstandige voortgangspunten.
