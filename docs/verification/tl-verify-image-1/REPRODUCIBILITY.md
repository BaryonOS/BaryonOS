# TL-VERIFY-IMAGE: read-only reproduceerbaarheidstrace

## Resultaat en bewijsgrens

**De uitvoerbare bestanden verschillen niet uitsluitend in onbeladen
buildmetadata.** Vier app-ELFs hebben dezelfde PT_LOAD-payloads en
segmentbeschrijvingen. Materiaal, keyring, volume en kernel hebben ook
verschillende geladen code/data. Semantische gelijkheid is niet bewezen.
De volledige oorzaak van de Materiaal-codegeneratieafwijking blijft open;
dit rapport wijst geen compilerfout, cachefout of runtimefout aan.

Uitgevoerd op 30 oktober 2026 vanuit `simon/tl-verify-image-1`, bewijsbasis
`7a74eaa28b64b04e270fbae886350c300d3b5bf0`; ongewijzigde codebron
`74d15ac09ac61c8d47a882c3c982711af1a9d891`, checkoutbasis
`3d2e1aba218feb39e02a1a5d257c5eb63f50e26b`, v3
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.
Alleen eigen bewijsdocumenten/JSON zijn geschreven. Geen nieuwe build,
QEMU-run, dependency-installatie, bronwijziging of wijziging van builderbytes.

## Exacte artifacts en eerste verschillen

Eigen basis: `build/tl-verify-image-1/interactive`.
Originele basis:
`/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/build/native-test-version/review-interactive`.
De hashes uit `blocker.json` blijven ongewijzigd. `elf-differences.json`
bevat voor alle secties hashes, offsets, groottes en eerste relatieve
verschillen; ook alle PT_LOAD-segmenten zijn rechtstreeks uit de ELF64-bytes
vergeleken, zonder strippen, maskeren, remappen of bestanden te herschrijven.
Secties van type NOBITS hebben geen bestandsbytes; PT_LOAD-filesz en memsz
zijn afzonderlijk in de segmentvergelijking betrokken.

| Artifact | Eerste verschil in bestand, decimaal | Geladen bytes | Lokalisatie |
|---|---:|---|---|
| desktop-init | 33312 | Gelijk, inclusief segmentbeschrijvingen | Alleen `.symtab` en `.strtab` |
| input-policy | 40578 | Gelijk, inclusief segmentbeschrijvingen | Alleen `.strtab` |
| native-compositor | 105955 | Gelijk, inclusief segmentbeschrijvingen | Alleen `.strtab` |
| input-source | 30209 | Gelijk, inclusief segmentbeschrijvingen | Alleen `.strtab` |
| materiaal-app | 278229 | Verschillend | `.text` eerste relatief verschil 69333; ook symbolen/strings anders |
| keyring-service | 208 | Verschillend | Header/segmentgrootte; `.rodata` relatief 327, `.data` 1024, `.text` 2219 |
| volume-service | 40 | Verschillend | Header/sectiepositie; `.text` relatief 7492, `.data` 1024, `.rodata` 12616 |
| kernel | 152 | Verschillend | Header/segmentgrootte; `.text` relatief 965, `.rodata` 1203, `.data` 1257480 |
| loader PE32+ | 128 | `.text` gelijk; `.rdata` bevat afwijkende debugvelden | Alle afwijkingen hieronder geclassificeerd |

`first-loaded-instructions.json` bewaart de werkelijke `objdump`-uitvoer,
met instructiegrenzen. Bij Materiaal op virtueel adres `0x400c0ed5` staat
eigen `push %rbx`, oorspronkelijk `push %r13`. De daaropvolgende
`Iterator::try_fold`-code heeft ook andere register-/instructievolgorde;
de `.text`-grootte is in beide gevallen 85769 bytes.
Daarmee is "alleen andere symbolnamen" voor dit bestand uitgesloten.
Of functies/monomorfisaties anders geordend zijn, of ook andere
codegeneratie optreedt, is niet volledig onderzocht en geen acceptatiebewijs.

Keyring heeft op `0x400208a9` eigen `mov $0x40014830,%r8d`,
oorspronkelijk `mov $0x40014840,%r8d`. Volume heeft op `0x40001d43`
eigen `mov $0x40017678,%edi`, oorspronkelijk `mov $0x40017688,%edi`.
Beide `.rodata`-secties zijn oorspronkelijk 16 bytes groter.
`embedded-paths.json` toont werkelijk aanwezige verschillende absolute
worktreepaden in hun diagnostische strings. Dit onderbouwt pad-/layoutinvloed,
maar bewijst niet dat ieder code-/dataverschil uitsluitend daarvan komt.
De kernel bevat bovendien de volledige verschillende app-ELFs: ook
app-symbolmetadata is als embedded payload onderdeel van kerneldata.
Dat verklaart niet zonder verdere analyse elk kernelverschil.

### Loader: volledige byteclassificatie

`pe-differences.json` bewaart hashes van iedere ruwe PE-sectie en alle
verschillende bestandsbytebereiken, halfopen `[begin,einde)`:

```text
[128,130)     COFF TimeDateStamp, veld op offset 128
[30440,30442) IMAGE_DEBUG_DIRECTORY.TimeDateStamp, veld op offset 30440
[30468,30476) eerste 8 bytes van CodeView RSDS GUID
```

De debugdirectory staat op RVA `0x86e4`, grootte 28. De twee tijdstempels
zijn respectievelijk eigen `1793380151` en oorspronkelijk `1793372139`.
De `.text`, `.eh_fram`, `.reloc` en lege ruwe `.data` zijn bytegelijk.
De overige bytes, ook buiten de secties, zijn vergeleken; alleen bovenstaande
velden verschillen. Het RSDS-record staat in `.rdata`: geladen sectiebytes,
maar hier gelokaliseerde debugmetadata, niet verschillende instructies.
Deze diagnose vervangt de oorspronkelijke volledige loaderhash niet.

## Feitelijke buildinputs

`build-commands.json` bewaart beide volledige commandorecords, cwd,
returncode en de eerste volledige JSON-regel van de app-buildstdout
(geneste Cargo-command, target_dir en waar aanwezig toolchain/linker).
Alle oorspronkelijke en eigen opgenomen buildcommando's rapporteerden exit 0.
Dit is een archief van die builds, geen nieuwe build.

Beide routes gebruiken dezelfde fysieke Rust-installatie onder de
buildertoolroot. Nieuw rechtstreeks opgevraagd en met SHA-256 vastgelegd in
`toolchain-versions.json`:

```text
rustc 1.90.0, commit 1159e78c4747b02ef996e55082b704c09b970588
cargo 1.90.0, commit 840b83a10fb0e039a83f4d70ad032892c287570a
LLVM 20.1.8
rust-lld LLD 20.1.8, LLVM commit e8a2ffcf322f45b8dce82c65ab27a3e2430a6b51
host x86_64-unknown-linux-gnu, Ubuntu 26.4.0
```

Dit meet de huidige gedeelde toolbytes. Zonder historische toolhashes is
niet aanvullend bewezen dat die installatie sinds de originele build nooit
gewijzigd is. De beschikbare Cargo-fingerprintcompileridentiteit is wel gelijk.

De builder bevestigde als inputfeit: geen shelloverrides voor `RUSTFLAGS`,
`CARGO_ENCODED_RUSTFLAGS`, `RUSTC`, `RUSTUP_TOOLCHAIN`; geen globaal
`CARGO_TARGET_DIR`; wel `NOS_TARGET_DIR=target/native-test-version/gui-baseline`.
De eigen route zette beide targetvariabelen expliciet naar de eigen
`target/tl-verify-image-1`. De orchestration kiest afzonderlijke app-targets.
Deze historische shelltoestand is een verklaring van de builder, niet
achteraf onafhankelijk gereconstrueerde procesomgeving.

`cargo-fingerprints.json` bewaart de beschikbare fingerprintrecords
voor Materiaal, keyring, volume, loader en kernel. Sommige targetcaches
bevatten meerdere features/probes; de volledige selectie is zichtbaar,
niet voorgesteld als uitsluitend de gewone build. Caches zijn geen
immutable origineel buildlog en kunnen later veranderen.

Voor de Materiaal-bin zijn `rustc`, `features`, `target`, `profile`,
`path`, `config` en `compile_kind` gelijk. Dependency-fingerprintwaarden
en crate/symbolhashes verschillen. De rustflags zijn inhoudelijk dezelfde
reeks, maar absolute linkerscript- en remap-prefixpaden verschillen:

```text
-C code-model=small
-C relocation-model=static
-C no-redzone=yes
-C linker=rust-lld
-C link-arg=-T<worktree>/sdk/examples/materiaal/linker.ld
-C link-arg=--no-pie
-C link-arg=--build-id=none
-C link-arg=-zmax-page-size=0x1000
-C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2
--remap-path-prefix=<worktree>=/nieuw-os
```

Belangrijk onderscheid: geen shelloverride betekent niet dat geen remapping
plaatsvindt. App-buildscripts zetten zelf `CARGO_ENCODED_RUSTFLAGS`.
Materiaal/desktop remappen hun bronroot; de keyring-build doet dat niet.
Geen `target-cpu`-override aangetroffen in deze rustflagrecords/config/scripts;
QEMU `qemu64` is een afzonderlijke runtimekeuze, geen gemeten rustc-flag.
De rootreleaseconfig is `panic=abort`, `codegen-units=1`, `lto=false`.
Kernelflags: small/static/no-redzone, `-Tkernel/linker.ld`, no-pie,
build-id none, max-page-size 0x1000. UEFI heeft expliciet de
`curve25519_dalek_backend="serial"`-cfg. Geen van deze configs is gewijzigd.

## Herhaalbare controles en open input

De read-only Python-stdlibinspecties (ELF64/PE-structuren, SHA-256 en JSON)
en toolversionqueries eindigden met exit 0; hun compacte uitvoer staat in
de genoemde JSON-bestanden. `readelf`, `objdump` en `strings` zijn bestaande
hosttools, geen toegevoegde dependencies. Direct herhaalbare kerncontroles:

```sh
A=build/tl-verify-image-1/interactive
B=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/build/native-test-version/review-interactive
readelf -W -l -S "$A/users/materiaal/materiaal-app.elf"
readelf -W -l -S "$B/users/materiaal/materiaal-app.elf"
objdump -d --no-show-raw-insn --start-address=0x400c0ed0 --stop-address=0x400c0f01 "$A/users/materiaal/materiaal-app.elf"
objdump -d --no-show-raw-insn --start-address=0x400c0ed0 --stop-address=0x400c0f01 "$B/users/materiaal/materiaal-app.elf"
strings "$A/users/materiaal/keyring-service.elf"
strings "$B/users/materiaal/keyring-service.elf"
```

De hier gebruikte instructieselecties zijn werkelijk uitgevoerd, exit 0.
De `readelf`-/`strings`-inspecties zijn eveneens uitgevoerd; deze verkorte
herhaalroute selecteert specifiek de beslissende artifactparen.
Een tussentijdse probe naar `.stdout.log` faalde (bestand bestaat niet);
de werkelijke `.stdout.txt`-bestanden zijn daarna gebruikt en gearchiveerd.
Dit was geen build-/runtimegate. Er is geen uitgevoerde analyse als
geslaagde native gate geboekt.

Afrondingscontrole, exit 0: alle negen eigen en negen oorspronkelijke
executablehashes opnieuw vergeleken met het ongewijzigde `blocker.json`;
alle bewijs-JSON geparsed; exact vier gelijke PT_LOAD-appsets gecontroleerd.
`git diff --check` gaf exit 0. De bestaande `common.source_hash()` opnieuw
aangeroepen met `python3 -B` geeft exact bovenstaande v3, exit 0.
De readelf-uitvoer staat lokaal onder
`build/tl-verify-image-1/readelf-{own,builder}.log`; compacte sectiegegevens
en overige relevante uitvoer zijn in de gecommitte JSON opgenomen.

**Exact ontbrekend voor een volledige causale verklaring:** complete
historische rustc-invocations inclusief door Cargo gegenereerde `-C metadata`,
crate-disambiguators en procesomgeving zijn niet in de buildreports opgeslagen.
De huidige fingerprints bewaren niet de volledige rustc-commandline.
Een gecontroleerd herbouwexperiment dat worktreepad/targetpad/metadata
afzonderlijk varieert is niet toegestaan binnen deze read-only opdracht en
is dus niet uitgevoerd. De aanwezige paden/metadata verschillen aantoonbaar;
hun exclusieve causaliteit voor de Materiaal-instructievolgorde is niet bewezen.

Vervolgkeuze blijft bij manager/release: exact oorspronkelijk image testen,
of afzonderlijk een nieuwe eigen referentie/native gate autoriseren.
Geen oude hashes vervangen, geen private_path/hashoracle versoepeld,
geen cold/Windows/privacy/NAS- of menselijke-auditclaim. Rubricscore blijft
ongewijzigd; centrale status/assessment zijn expliciet van de manager.
De eerder gestopte onafhankelijke native acceptatie blijft geblokkeerd.

Formaatreferenties voor de classificatie: System V ABI, ELF Header,
Program Header en Sections, https://gabi.xinuos.com/elf/ ;
Microsoft, PE Format, COFF File Header en Debug Directory,
https://learn.microsoft.com/en-us/windows/win32/debug/pe-format .
De concrete gelijkheidsclaims hierboven komen uit de artifactbytes,
niet uit een veronderstelde eigenschap van deze formaten.
