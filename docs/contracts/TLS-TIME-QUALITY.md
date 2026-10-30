# TLS-TIME-QUALITY-1: expliciete tijdweigering

## Doel, eigenaar en scope

TL-CORE, bronbasis `3f3f99efd18ef70d6ca5737a5fd668a520f537e6`
(bevat main `0ade801`), branch `simon/tls-time-quality-1`.
Afgestemd met TL-NET en TL-SEC: uitsluitend
`sdk/runtime/src/tls_time.rs`, directe `sdk/runtime/tests/tls_time.rs`
en dit contract. Release heeft bovendien uitsluitend de export
`pub mod tls_time;` in `sdk/runtime/src/lib.rs` vrijgegeven; NET en apps
zijn hierover direct geinformeerd. Geen andere clock-, kernel-, syscall-, TLS-stack-,
certificaat-, sleutel- of ankerwijzigingen.

Dit is een directe afhankelijkheid van FR-N03/FR-S03, geen volledige
TLS-, tijdsync-, product- of rubricacceptatie. De RTC-interface en de
ABI 1.9-syscall bestaan al en worden niet opnieuw geimplementeerd.
Het [RTC-contract](../implementation/RTC-WALLTIME.md) blijft leidend.

## Invariant en interface

Een leesbare UTC-waarde is geen bevoegdheid om certificaatgeldigheid
goed te keuren. De bestaande SDK kent alleen
`WallClockQuality::UnauthenticatedRtc`. Een stabiele, plausibele,
oplopende of als minimum opgeslagen RTC-waarde verandert die kwaliteit niet.
Boot-monotone ticks zijn geen kalenderdatum. KEY_EXPIRED in het
update-/sleutelbeleid is een releaseversievenster, geen kalenderklok.

De pure allocatorvrije `no_std`-module bevat:

```rust,ignore
pub fn certificate_time(
    reading: Option<WallClockReading>,
) -> Result<CertificateTime, TimeUnavailable>;
```

`None` geeft `MissingSource`; iedere huidige RTC-lezing geeft
`UnauthenticatedRtc`. **Geen huidige publieke invoer kan `Ok` opleveren.**
De uitputtende match op klokkwaliteit dwingt een expliciete codekeuze af
als later een nieuwe kwaliteit wordt toegevoegd.

`CertificateTime` heeft een prive `u64`-secondeveld en een
`unix_seconds()`-getter voor toekomstige verifierintegratie. Geen publieke
constructor, `From`, `Default`, deserializer, trustboolean of toegankelijke
numerieke conversie. Veilige callers kunnen dus geen timestamp of
`WallClockReading` promoveren tot dit type. De module verbiedt eigen unsafe.
Een type/module begrenst veilige API-gebruikers, niet een kwaadaardig
proces met onveilige code of kerneltoegang; echte autoriteit moet later
bij de geautoriseerde provider-/procesgrens worden afgedwongen.

De toekomstige TLS-consument moet deze fout zichtbaar teruggeven voordat
hij datumverificatie probeert. Hij mag geen fallback naar hosttijd,
buildtijd, een vast testtijdstip, netwerkdatum of monotone ticks uitvoeren.
Geen certificaatdatum overslaan, accept-all verifier, TOFU of automatisch
andersoortige transportverbinding als gevolg van deze fout.
Deze stap implementeert geen TLS-consument en bewijst dus niet dat een
daadwerkelijke handshake al deze grens gebruikt.

## Voorwaarden voor een toekomstige succesvolle bron

De volgende voorwaarden blijven **OPEN**, niet impliciet ingevuld:

- Wie mag kalendertijd uitgeven en hoe wordt die bevoegdheid per proces
  afgedwongen? Een publieke `trusted: true` of caller-provided enum voldoet niet.
- Welke onderhouden standaard en echte cryptografische verificatie bindt
  bronidentiteit en tijdinhoud? Geen eigen tijdondertekenprotocol.
- Hoe worden versheid, onzekerheid, maximumleeftijd, voorwaartse sprongen,
  tijd tussen ontvangst en verificatie en suspend/herstart behandeld?
- Hoe worden terugloop, replay en stale samples over herstarts herkend,
  met welke geauthenticeerde persistente ankerbron en fouttransacties?
  Een versiefloor voor updates of de huidige RTC-floor is daarvoor geen
  kalenderbewijs.
- Hoe bindt de TLS-verifier de geldigheidsgrenzen van de volledige
  certificaatketen aan de toegestane onzekerheidsmarge? De bestaande TLS-port
  wordt hergebruikt; geen tweede TLS-stack.

Zolang die bron niet bestaat, geven ook onbekende, onbeschikbare, stale,
gereplayde of teruggelopen tijdpaden geen certificatentijd: er is geen
publieke succesconstructor. Een toekomstige provider moet zulke toestanden
eerst expliciet representeren en negatief testen; deze module beweert niet
dat ze cryptografisch zijn gedetecteerd.

## Acceptatie en bewijsgrens

Gerichte tests toetsen ontbrekende bron, RTC bij negatieve/epoch/plausibele/
extreme `i64`-waarden, stilstaande/oplopende/teruglopende reeksen en een
RTC-fallbackpoging. Compile-faildoctests toetsen onmogelijke veilige
conversie vanuit RTC, publieke structconstructie en monotone getallen.
Verdere gates: bestaande runtime-hosttests/doctests, `no_std`-targetcheck,
rustfmt, clippy en diffcontrole volgens de opgeslagen bewijsuitvoering.

Geen nieuwe dependency, I/O, klokinstelling, sleutelpersistentie of
netwerkrecht. Geen native TLS-, QEMU-, Windows-, fysieke of vertrouwde-
tijdacceptatie door alleen deze pure coretests. De manager onderhoudt
gedeelde queue/status/rubric; voorstel **34% -> 34%** tenzij diens actuele
rubric inmiddels een andere, brongebonden uitgangsscore heeft.

## Uitgevoerd bewijs op 30 oktober 2026

Exacte v3-bronvingerafdruk van de gewijzigde code:

```text
15eac332d8044bcfb0053ebeb7dfc89d70c90d44d96201ab04e6f2d70cb4844c
```

De onderstaande commands zijn werkelijk uitgevoerd op deze code, alle
**exit 0**. De runtime-export is de enige gewijzigde regel in een bestaand
bronbestand; de andere drie bestanden zijn nieuw. Geen dependencies,
manifests, lockfiles, buildtools of gedeelde voortgangsbestanden gewijzigd.

```sh
. .tools/nieuw-os/rtc-env.sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export CARGO_TARGET_DIR="$PWD/target/tls-time-quality-1/host"
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/tls-time-quality-1/native" \
  python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --lib --target x86_64-unknown-none
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime -- --check
git diff --check
```

Rust/Cargo 1.90.0; runtime **27 hosttests + 3 compile-faildoctests** groen,
waarvan 4 nieuwe semantische tijdtests en 3 nieuwe compile-failtests.
Volledige SDK **81 hosttests + dezelfde 3 doctests** groen; die telling
omvat de runtime en wordt niet erbij opgeteld. Clippy waarschuwingsvrij;
`x86_64-unknown-none` targetcheck groen, geen host-libc/allocator nodig.
Dit is een target-typecheck, **geen uitgevoerde native TLS-handshake**.

Volledige lokale logs en SHA-256:

| Log | SHA-256 |
|---|---|
| `/tmp/nieuw-os-tls-time-runtime-tests.log` | `5057c93480b0eb7fa77234f5d2feaf3f73ac44001bc5095c7abd9340ef8c5182` |
| `/tmp/nieuw-os-tls-time-sdk-tests.log` | `66faba2e3df8576e29761d34adc5c3f76b412d75cafdf03ef8a25a79c1a1dfe2` |
| `/tmp/nieuw-os-tls-time-clippy.log` | `841330560dc8011a07ed720344531b084b1b8e7d558510511ab7903f4e5b0f3c` |
| `/tmp/nieuw-os-tls-time-target.log` | `bd2750ef00c4822aa45aa0b0aafda459a894973b40abae988b21d8e41c0b00c3` |
| `/tmp/nieuw-os-tls-time-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Twee ontwikkelcommando's waren eerst rood en zijn niet als acceptatie
gebruikt: Cargo weigerde `--test tls_time --doc` met exit 101 omdat die
selectors niet gecombineerd mogen worden; de volledige package-test
hierboven omvat beide en slaagt. Rustfmt wees de aanvankelijke plaats
van de nieuwe module-export af (exit 1); de ene export staat nu in
alfabetische volgorde en de hercontrole slaagt. Geen code- of testfallback.

De nieuwe codebron heeft een andere v3 dan de eerder geteste
Windows-testbundel. Die bundel blijft uitsluitend aan zijn oorspronkelijke
`eabc27b`/`a1998a12...`-bron gebonden; dit contract labelt die runtime
niet om als geteste TLS-code. Native TLS-consument, echte vertrouwde bron,
freshness, persistent rollbackanker en eindimage-integratie blijven open.
