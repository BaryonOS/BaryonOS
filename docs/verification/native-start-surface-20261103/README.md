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
| `git diff --check` | 0 | Geen whitespacefouten vóór commit |
| `python3 tools/progress.py --write` | 0 | README, rubrictabel en SVG bijgewerkt op 41/100 |
| `python3 tools/progress.py --check` | 0 | Score en gegenereerde weergaven consistent |
| `/home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test render_regressions desktop_home_card_follows_the_real_library_state_and_stays_above_the_bar -- --nocapture` | 101 | Niet uitgevoerd: linker `cc` ontbreekt; assertions zijn niet bereikt |
| `PATH="$HOME/.cargo/bin:$PATH" python3 tools/preflight.py --strict --strict-host-tests` | 2 | Rust 1.90.0 en beide OS-targets beschikbaar; `cc`, QEMU en OVMF ontbreken |

Logbestanden staan buiten Git in `/tmp/nieuw-os-native-ui-{fmt-check,check,clippy,test,preflight-path,scalar-build}.log` en `/tmp/nieuw-os-progress-{write,check}.log`.
Preflight is alleen inventarisatie; dit is geen native runtimebewijs. Er is
geen QEMU-boot, screenshotinspectie of pixelassertie uitgevoerd. De
startschermwijziging sluit geen voortgangscriterium; de score blijft 41/100.
Het geproduceerde artifact
`/tmp/nieuw-os-native-ui-scalar-target/x86_64-unknown-none/release/libnieuw_os_materiaal.rlib`
heeft SHA-256
`42eda9397dc739d098b666d9f5a013911fc0b3d867469fde8a558b34b3888b5d`.

## Open verificatie

De freestanding stap komt uit `.github/workflows/native-display-input.yml` en
is geslaagd. Voer de renderregressietest uit zodra `cc` beschikbaar is. De
native QEMU-/screenshotgate vereist ook QEMU en OVMF. Een hostcompile of
AI-/code-inspectie vervangt die gates niet.
