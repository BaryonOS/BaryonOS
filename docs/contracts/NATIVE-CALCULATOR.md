# Native rekenmachine: componentcontract

NATIVE-CALCULATOR-1, basis `5e126b74bba2452bfc68b42caa400c61d471378c`.
Deze component levert een echte no_std-CPL3-app, algemene decimale rekenkern,
getypeerde INP1/SUR1-clients en een eigen native bediening in de bestaande
Materiaal-vormgeving. Zij is onderdeel van FR-A08 uit
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md); componenttests en een ELF-bouw
tekenen de volledige native app-/releaseacceptatie niet af.

## Rekencontract, vooraf vastgelegd

Er was op de gecontroleerde appsbranch/ecfeab4 geen bestaande rekenengine.
De nieuwe begrensde parser is gewone Rust-rekenlogica, geen cryptografie,
float-/C-/hostbackend of tabel met antwoorden. Dezelfde library wordt door
`calculator-app` en de afzonderlijke hostreferentiedriver gebruikt.

- Maximaal256ASCII-tekens. Spaties/tabs mogen tussen tokens. Maximaal32
  gecombineerde niveaus van haakjes en unaire `+`/`-`; binaire operatorlussen
  groeien de stack niet per teken.
- `+`, `-`, `*`, `/`, haakjes en unaire tekens. Vermenigvuldigen/delen gaan vóór
  optellen/aftrekken; gelijke prioriteit is linksassociatief.
- Komma en punt zijn beide decimale tekens, geen duizendtalseparator. Geen
  exponent-, procent- of wetenschappelijke-precisieclaim.
- Elke literal en ieder tussenresultaat moet liggen tussen
  −9.999.999.999,999999999 en +9.999.999.999,999999999. Een buitenbereik-literal of
  tussenresultaat wordt geweigerd, ook als een latere operatie zou kunnen
  compenseren. Er wordt niet stil gesatureerd, gewrapt of met Infinity gewerkt.
- De vaste schaal is10^9. Optellen/aftrekken zijn exact op dit raster.
  Literalcijfers na de negende decimaal en producten/quotiënten worden afgerond
  naar het dichtstbijzijnde rasterpunt; precies half gaat naar het **even**
  laatste cijfer, voor beide tekens. Werkelijk weggegooide niet-nulwaarde maakt
  de afrondingsindicator waar; eerdere afronding blijft door de berekening heen zichtbaar.
- Delen door nul geeft een concrete Nederlandse fout. Nul na onderloop heeft
  geen minteken. De weergave gebruikt een komma en laat overbodige nullen weg.

`|raw| ≤ 10^19−1` garandeert `|a×b| < 10^38 < i128::MAX`; bij delen is de
vermenigvuldigde teller kleiner dan10^28. Alle operatoren gebruiken bovendien
checkedbewerkingen en controleren hun uitkomst. Positieve/negatieve maxima,
beide half-evenrichtingen, roundup-overflow en kleine geldige uitkomsten na
grote ruwe producten worden daadwerkelijk getest. Dit is een begrensde
rekenmachine, geen universele exacte-rationaal- of arbitraire-precisie-engine.

## Invoer, fouten en widgets

De expressie is lokaal opgeslagen met caret en selectie; zij blijft volledig
behouden bij syntax-, bereik-, lengte- en deelfouten. Ook de vorige geldige
uitkomst blijft behouden, met zichtbaar label `Vorig`. Die uitkomst wordt niet
als antwoord op foutieve of gewijzigde invoer gepresenteerd.

Toetsenbord: cijfers, normale US-fysieke operatorcombinaties, haakjes, punt/
komma, numpad met NumLock, Enter of `=` evalueren, Backspace/Delete,
Left/Right/Home/End, Shiftselectie, Ctrl+A, Tab/ShiftTab en Enter/Space op een
gefocuste knop. Escape annuleert de huidige invoer en behoudt de eerdere
uitkomst; `C` en de C-knop resetten invoer/uitkomst. Enter-/Tab-/Escape-/C-repeat
wordt niet als een reeks nieuwe bewuste acties behandeld. Releases rekenen niet.

Dezelfde commando's worden gebruikt door de5×4-knoppen. `Wis` verwijdert de
selectie of het vorige teken, C reset, `=` rekent en de overige knoppen voegen
het werkelijk getoonde teken in. Een knop werkt pas na echte linkermuis-down
én up op dezelfde knop. Buiten assignedRect, verloren focus, inputcancel of
overflow maakt de capture ongeldig; geen clamping naar een toevallige knop.
Klik/drag in de expressie plaatst/selecteert echte invoer. Lange invoer scrollt
horizontaal met de caret. De berekening zelf is een begrensde synchrone parser;
er is geen voorgestelde achtergrondtaak met een fictieve cancelstatus.

Layout/hittest/tegelrender delen dezelfde geometrie. 100/200% vanaf320×240
logisch, binnen het bestaande maximale1920×1080-fysieke profiel. Compacte
vensters gebruiken dezelfde fontfamilie en tokens met passende controls;
geen bediening verdwijnt om acceptatie te vereenvoudigen. De ene bestaande
`ui/native/src/design.rs`/`ui/prototype/tokens.css` blijft de tokenbron.
De bestaande goedgekeurde Noto Sans Mono bitmap0.3.2 wordt rechtstreeks met
hetzelfde featureprofiel gebruikt. Licenties/notices blijven onder
`ui/native/licenses`; herkomst en SHA staan in
[NATIVE-MATERIAAL](../implementation/NATIVE-MATERIAAL.md).

De app schildert alleen begrensde32×8-pixeltiles in eigen RAM. Typografie,
radii, kleuren, hover, selectie en focus volgen Materiaal. Bitmapfontdekking
is de bestaande Basic Latin/Latin1/Specials-dekking, geen algemene shaping-/
schermlezerclaim. Er zijn geen productanimaties, lorem ipsum of nepknoppen.
Hostvoorbeelden uit de previewdriver zijn expliciet RAM-renderproeven;
echte QMP-/native visuele en bedieningscontrole blijft verplicht.

## Diensten, grants en levensduur

De rekenmachine heeft geen volume-, document-, Key-, entropy-, netwerk-,
rawdisplay-, rawinput-, launch- of bestandsgrants nodig. De enige zes handles:
registry-SEND, control-RECV, surface-SEND/reply-RECV, INP1-SEND/reply-RECV.
Getal/PID/epoch in de startup verleent geen autoriteit; de echte grant en
dienstingress bepalen de rechten. Handle-aliases en bekende oude document-
metadata worden geweigerd. De root moet werkelijk uitsluitend deze zes
endpointleaves installeren en eigen reply-endpoints zonder RECV-alias gebruiken.

| Startupwoord | Betekenis |
|---|---|
|0|bootmodus; de werkelijke SDK/kernel-ABI wordt afzonderlijk via `native::check_abi()` opgevraagd |
|88/96|Initregistry-SEND / eigen control-RECV |
|176/184|eigen PID / InitPID |
|544/552/560/568|SUR1-SEND / eigen reply-RECV / compositorPID / epoch |
|576/584/592/600/608|INP1-SEND / eigen reply-RECV / policyPID / epoch / clientID |
|616|schaalfactor1of2 |

Bekende documentvelden48/56/64/72/80/168 zijn0. De constructor leest echte,
geauthenticeerde SUR1-INFO-geometrie; startup geeft geen framebufferadres of
vooraf ingevulde pixels. INP1 PEEK/ACK gebruikt de bestaande exclusieve typed
Session. Eén step verwerkt maximaal4offers en8SUR1-tegeltransacties. Een half
of verouderd frame maakt `ready` niet waar. ACK-supersession cancelt captures;
transportverlies maakt input/oppervlaktefalen zichtbaar en geeft geen nieuwe
fictieve ACK of hergrant.

Native entry gebruikt CLD, controleert de bestaande SysV-RSP%16=8 en springt
naar Rust. Geen heapallocator, geen stackvergroting: bestaande64KiB en de
gewone W^X-loader. Gereedmelding is SYSTEM-tag `0x53595354454d0001`, op1;
start is geauthenticeerde Init-op2. Stopop5 sluit beide typed eigenaren,
stuurt pas dan DONE3 en sluit registry/control. Ook bij een closefout worden
alle betrokken eigenaren geprobeerd. STOP schrijft of bewaart geen bestanden.
Ook STOP vóór GO sluit de vier nog niet overgedragen dienstleaves voordat
DONE/registry-/controlcleanup volgt; er wordt dan geen frame als klaar verklaard.
De usermodelcounter loopt niet rond; uitputting wordt een terminale fout.

## Diagnostiek zonder vooringevuld succes

Alle onderstaande woorden komen uit werkelijk model-/clientgedrag, zijn
private userdata en geen app→kernelprivilege. Ready wordt bij publicatie eerst
gewist en als laatste gezet. Root moet een volledige huidige generatie
snapshotten vóór eventuele procescleanup en de echte PID bewaren bij slotreuse.

| Woord | Werkelijk feit |
|---|---|
|40/240|cleanSTOPdone1 / eventuele failstep |
|400|geheel huidige frame klaar plus echte INP1-response; geen rekenantwoordclaim |
|408/416|volledige frames / succesvol gecommitte tiles |
|424/432/440|gevalideerde key-/mouseoffers / echte ACKs |
|448/456|dienstfouten / gepresenteerde generatie |
|464/472|geslaagde berekeningen / concrete rekenfouten |
|480/488/496|expressielengte / huidige uitkomst / afrondingsvlag |
|504/512|caret / lokale visuele focusstate; geen broker-FocusACK |
|520/528/536|ruwe i128-uitkomst low64/high64 two'scomplement / uitkomstaanwezig |
|632|schaalfactor |

De uitkomst is `raw/10^9`; er is geen statische verwachte antwoordwaarde in de
native entry. Persoonlijke berekeningen worden niet naar OS-telemetrie of een
hostdienst verzonden. Native testobservers mogen alleen de bekende testdata
voor hun afgebakende proeven registreren; private userdata is geen toestemming
voor algemene logging.

## Reproduceerbare componentcontrole

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 test --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 clippy --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml --all-targets --features host-oracle -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 build --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml --features host-oracle --bin calculator-oracle
python3 -B sdk/examples/calculator/tests/decimal_oracle.py --binary /tmp/nieuw-os-calculator-host-target/debug/calculator-oracle
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-none-target python3 -B tools/dev.py python3 -B sdk/examples/calculator/build.py
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-none-target python3 -B tools/dev.py python3 -B sdk/examples/calculator/build.py clippy
```

De Python-Decimalreferentie gebruikt100cijferige hostprecisie en half-even per
literal/operator, plus de vastgelegde raster-/bereikgrens. Zij leest de echte
Rust-hostdriveruitkomsten voor deterministische gevarieerde expressies; zij is
nooit een native backend. De hostketen gebruikt de echte capability-engine,
INP1-dienst, SUR1Receiver en typed clients. Alleen de opnemende RAM-display-
tegenpartij is expliciet hostmodel. Verloren/negatieve replies, buitenklik,
focusgeneratie, cleanup en bootstraptampering zijn afzonderlijke tegenproeven.

De eigen lock bevat alleen bestaande lokale libraries en de reeds goedgekeurde
Noto-pin, zonder nieuwe runtimebibliotheek/download/buildscript. Root-/SDKlocks,
gedeelde UI-, kernel- en buildscripts worden hier niet veranderd. De builder
levert `NOS_CALCULATOR_ELF` en inspectie/SHA, met `native_runtime_performed=false`.

Rootintegreert vervolgens de echte kernelgrants, launcher/Init, meerdere
programma's, compositor-/focusbeleid en native QMP-scenario's. Nodig blijven
werkelijke cijfers/haakjes/decimalen typen, correctie, overflow/÷0/herstel,
keyboard/muis,100/200%, focuswissel en schoon stoppen op het geïntegreerde
image met exacte bron-/ELF-/imagehashes. Gedeeld klembord en de andere volledige
releasepoorten blijven hun eigen verplichte open acceptatiecriteria. Een
hostcheck, preview of geslaagd native doelbuild vervangt die uitvoering niet.
