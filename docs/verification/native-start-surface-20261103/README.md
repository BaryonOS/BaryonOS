# Native startoppervlak — 3 november 2026

## Bron en doel

Broncommit: `bd4134c1d565f58736a2c829587f337a415dd47b`, branch
`simonvanieperen-reimagined-train`; basis `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`.

Deze stap verfijnt alleen de opt-in native VM-preview. De Materiaalkaart
gebruikt de bestaande `View`-lijststatus en toont niet-geladen, lege, fout- en
gevulde toestanden. Een regressietest vergelijkt de gerenderde pixels en
controleert kaart-/Start-geometrie op vier viewport-/schaalcombinaties. Geen
kernel-, capability-, authenticatie- of opslaginterface is gewijzigd.

## Controles

| Exacte opdracht | Exit | Resultaat |
|---|---:|---|
| `/home/simon/.cargo/bin/cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml --all -- --check` | 0 | Formattering consistent |
| `/home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | Productie- en testcode compileert |
| `/home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | Geen Clippy-waarschuwingen |
| `CARGO_TARGET_DIR=/tmp/nieuw-os-native-ui-scalar-target RUSTFLAGS="-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2" /home/simon/.cargo/bin/cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none --manifest-path ui/native/Cargo.toml` | 0 | Scalar freestanding UI-library gebouwd; geen runtime uitgevoerd |
| `cd /tmp/nieuw-os-c-toolchain/debs && apt-get download gcc-15 gcc-15-x86-64-linux-gnu cpp-15 cpp-15-x86-64-linux-gnu gcc-15-base libgcc-15-dev libc6-dev libc-dev-bin linux-libc-dev rpcsvc-proto libcc1-0 libisl23 libmpc3` | 0 | Ubuntu 26.04-pakketten naar `/tmp` gedownload; niet systeem-breed geïnstalleerd |
| `git diff --check` | 0 | Geen whitespacefouten vóór commit |
| `python3 tools/progress.py --write` | 0 | README, rubrictabel en SVG bijgewerkt op 41/100 |
| `python3 tools/progress.py --check` | 0 | Score en gegenereerde weergaven consistent |
| `/home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test render_regressions desktop_home_card_follows_the_real_library_state_and_stays_above_the_bar -- --nocapture` | 101 | Eerste poging zonder tijdelijke linker stopte op `cc`; assertions niet bereikt |
| Volledige `ui/native`-hosttestsuite met tijdelijke linker (exacte opdracht hieronder) | 0 | 55 tests geslaagd: 37 algemene, 5 renderer-, 10 sessie- en 3 fixturetests |
| Preflight met tijdelijke linker (exacte opdracht hieronder) | 2 | Alleen QEMU en OVMF ontbreken; tijdelijke `cc` is gevonden |

De volledige hosttestsuite is exact zo uitgevoerd:

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-native-ui-test-target \
PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:$HOME/.cargo/bin:/usr/bin \
GCC_EXEC_PREFIX=/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/ \
CPATH=/tmp/nieuw-os-c-toolchain/root/usr/include \
LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15 \
LD_LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu \
/home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml
```

Preflight met dezelfde tijdelijke compileromgeving:

```sh
env CARGO_TARGET_DIR=/tmp/nieuw-os-native-ui-test-target \
PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:$HOME/.cargo/bin:/usr/bin \
GCC_EXEC_PREFIX=/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/ \
CPATH=/tmp/nieuw-os-c-toolchain/root/usr/include \
LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15 \
LD_LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu \
python3 tools/preflight.py --strict --strict-host-tests
```

Compiler: Ubuntu GCC `15.2.0-16ubuntu1`; `libc6-dev` `2.43-2ubuntu2.4`.
Pakketten zijn met `dpkg-deb --extract` onder `/tmp` uitgepakt en een tijdelijke
`cc`-wrapper gebruikt die naar dat sysroot linkt. Geen systeeminstallatie of
repositorydependency. Logbestanden staan buiten Git in
`/tmp/nieuw-os-native-ui-{fmt-check,check,clippy,test,test-isolated-cc,render-suite,full-host-tests,preflight-path,preflight-isolated-cc,scalar-build}.log`
en `/tmp/nieuw-os-progress-{write,check}.log`; pakketdownload:
`/tmp/nieuw-os-c-toolchain-download.log`.

Preflight is alleen inventarisatie, geen native runtimebewijs. De hostpixel-
asserties zijn uitgevoerd; er is geen QEMU-boot of native gast-screenshot
gemaakt. De startschermwijziging sluit geen voortgangscriterium; de score
blijft 41/100.
Het geproduceerde artifact
`/tmp/nieuw-os-native-ui-scalar-target/x86_64-unknown-none/release/libnieuw_os_materiaal.rlib`
heeft SHA-256
`42eda9397dc739d098b666d9f5a013911fc0b3d867469fde8a558b34b3888b5d`.

SHA-256 van de belangrijkste externe logs:

```text
bbd3c6aa26756aa3591de34f200cc2bd47ec2273e1dfd594eeaef01292334a33  /tmp/nieuw-os-native-ui-full-host-tests.log
b80baceac8b0705d897814deb8026e9db657c30409c0dc59099003de8c6451c8  /tmp/nieuw-os-native-ui-render-suite.log
5631981096abb568678a34fd8892e712e1bd500afd0cdefb6f7550f0aca325b1  /tmp/nieuw-os-native-ui-scalar-build.log
76a8fbb662582ae8ff6a09b211cb036ff10889d11dd36f7138b310c2191d7831  /tmp/nieuw-os-native-ui-preflight-isolated-cc.log
bca6ea1409f9893657884e605278e1cce24922a48a7ceef38f8ac08092824af6  /tmp/nieuw-os-c-toolchain-download.log
```

## Open verificatie

De freestanding stap komt uit `.github/workflows/native-display-input.yml` en
is geslaagd. De resterende gate is `tools/run_session_preview.py` met QEMU en
de expliciete BIOS/VGA-ROM's; die vergelijkt gastpixels na echte QMP-invoer.
QEMU en firmware ontbreken in deze omgeving. De hostcompile en pixeltests
vervangen die native runtimegate niet.
