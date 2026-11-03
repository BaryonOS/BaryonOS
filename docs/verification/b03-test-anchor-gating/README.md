# B03 TEST-ankergrens — verificatiebewijs

**Broncodecommit:** `07e7f9a0471798c49e1844bf885f94491bb481d8`

**Diagnostiekcommit:** `ceb957557af821bb95786aa505b5df41077f7403`

**Basis:** `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`
**Branch:** `fix/b03-test-anchors-guard`  
**Beoordeling:** gerichte hosttests en cross-builds; geen volledige B03-acceptatie.

De wijziging maakt de openbare TEST-ondertekenpolicy opt-in via
`test-signing-anchors`, voegt `signed-boot-test` toe en laat builds van de
loader of update-service zonder expliciet policyprofiel tijdens compilatie
falen. Dit beperkt onbedoeld gebruik van publieke TEST-fixtures; het levert
geen productieankers, sleutelbeheer, releaseceremonie of kalendergebaseerde
vervalsemantiek.

## Geslaagde controles

Alle onderstaande commando's zijn uitgevoerd vanuit de repositoryroot.

| Controle | Exact commando | Resultaat |
|---|---|---|
| B03 Python-harnas | `python3 -B -m unittest discover -s tests -p 'test_update_harness.py'` | 14 tests, exit 0 |
| Update-core met TEST-policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-final-host-target /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path crates/update-core/Cargo.toml --features test-signing-anchors` | 7 unit- en 6 policytests, exit 0 |
| Update-tool | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-final-tool-target /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path tools/update-tool/Cargo.toml` | 2 integratietests, exit 0 |
| Update-core zonder TEST-feature | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-final-default-target /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path crates/update-core/Cargo.toml` | 7 tests, exit 0; TEST-policytests zijn niet inbegrepen |
| UEFI-testloader | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-loader-target /home/simon/.cargo/bin/cargo +1.90.0 build --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot-test --target x86_64-unknown-uefi` | exit 0 |
| Update-service testbuild | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-sdk-update-final python3 -B sdk/examples/update/build.py` | exit 0 |
| Formatter | `cargo +1.90.0 fmt --manifest-path boot/loader/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path crates/update-core/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path tools/update-tool/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path sdk/examples/update/Cargo.toml -- --check` | alle exit 0 |
| Diffcontrole | `git diff --check` | exit 0 |
| Loader zonder policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-negative-loader-final /home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot --target x86_64-unknown-uefi` | exit 101 verwacht; expliciete policyfout |
| Update-service zonder policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-negative-service-clean /home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/update/Cargo.toml --features native --bin update-service --target x86_64-unknown-none` | exit 101 verwacht; alleen expliciete policyfout |
| Update-core Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-clippy-core /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path crates/update-core/Cargo.toml --features test-signing-anchors -- -D warnings` | exit 0 |
| Update-tool Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-clippy-tool /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path tools/update-tool/Cargo.toml -- -D warnings` | exit 0 |
| Update-service Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-clippy-service-final /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/update/Cargo.toml --features native,test-signing-anchors --bin update-service --target x86_64-unknown-none -- -D warnings` | exit 0 |
| Loader Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-clippy-loader /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot-test --target x86_64-unknown-uefi -- -D warnings -A clippy::not_unsafe_ptr_arg_deref` | exit 0; alleen genoemde bestaande lint toegestaan |
| Preflight | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH python3 -B tools/preflight.py --strict` | exit 2: QEMU en OVMF ontbreken |

De strikte loader-Clippy-run zonder lintuitzondering eindigde met exit 101
op de bestaande `clippy::not_unsafe_ptr_arg_deref` in
`boot/loader/src/main.rs:672`. Dit bestand en deze entrypoint zijn niet
gewijzigd.

De configuratieloze servicecheck geeft alleen de bedoelde policyfout. Een
eerste directe `cargo build --release` buiten de projectwrapper eindigde
tijdens LLVM-codegeneratie voor `curve25519-dalek` met SIGILL; de voorgeschreven
wrapperbuild slaagde daarna. Het directe-buildlog is door de wrapperrun
overschreven en heeft geen bewaarde hash. De strikte loader-Clippy-run
rapporteerde daarnaast de bestaande lint op de ongewijzigde UEFI-entrypoint;
de scoped run slaagde. Geen van deze controles is runtime- of
geheugenveiligheidsbewijs.

## Bewijsbestanden

De tests, buildlogs en artefacten staan buiten Git onder `/tmp`:

| Bestand | SHA-256 |
|---|---|
| `/tmp/nieuw-os-b03-final-python-harness.log` | `bb4a180f52c27a5838dac787915ef086a807cd4af5d4460eed59e3fa6fc9ead8` |
| `/tmp/nieuw-os-b03-final-update-core-test.log` | `dea985e058d4f0ac27bde1834270520084e13886fa13c3628bb9725bce4e8359` |
| `/tmp/nieuw-os-b03-final-update-tool-test.log` | `3ac75deb02bb4af75969aa5d0717160e56bc0a4970d32522ab4a6a8d0d4f1063` |
| `/tmp/nieuw-os-b03-final-default-test.log` | `ba827db7d378ac631a83a3b7cf16359afe0a81391dee45261d8eaf7f6393b9b5` |
| `/tmp/nieuw-os-b03-loader-build.log` | `bb13409b61402e049244b5e2fe9fe0093857fe0be949a90b554f3c42159d162f` |
| `/tmp/nieuw-os-b03-final-service-build.log` | `73b218d2540f4f89f7b1d559d47096171d450f0fd77dd2d67f70bda858318d7d` |
| `/tmp/nieuw-os-b03-final-service-negative-clean.log` | `9d8a274d3f8acea671c839d57857f332b809a2031fff8844742b2340e13371ad` |
| `/tmp/nieuw-os-b03-final-loader-negative.log` | `175b8176d24dd612c18278a84bfb32d4d2e57c313683f26158fe7ac05a814320` |
| `/tmp/nieuw-os-b03-preflight.log` | `3a9624dee063ddbfeb1d3324fc5ee8b6afed1a4e85e7849bd9e3d8810b875c4a` |
| `/tmp/nieuw-os-b03-final-core-clippy.log` | `bc86e5bce683f1c1fa0792039fdb29865441c35656e6545a6b81d275dcc6dddf` |
| `/tmp/nieuw-os-b03-final-tool-clippy.log` | `349cb9bbbbd71f138c2cabb06f3c8e497c6cea5764744edfc56568bf3cf2d13c` |
| `/tmp/nieuw-os-b03-final-service-clippy-clean.log` | `4bf396d7ee4b07b767a1d21fcdd857932128512cc327ad687f3e06ac92132643` |
| `/tmp/nieuw-os-b03-final-loader-clippy-strict.log` | `ac6637337b7805864b58c0924fda35c046c8177781ed53fe19dc674ffc2eb101` |
| `/tmp/nieuw-os-b03-final-loader-clippy-scoped.log` | `70b85aec2b1bbac5cf3451ed4010500a61519d2ecfebcbea1d8f83a0c13e88a2` |
| UEFI testloader artifact | `e79151c6d2ff72c60e099c4c9603d3f9926ed81a41e194ba8929a5c313f88097` |
| Update-service artifact | `a45f777872edb82d5937659273bc8b18b97f180bee9c7412cf0856735fb05e52` |

## Open bewijsgrenzen

De volledige native 18-staps B03-QEMU-gate is **niet uitgevoerd**: lokale
preflight mist `qemu-system-x86_64` en OVMF. Een hosttest of cross-build
vervangt die gate niet. Ook ontbreken productieondertekenankers,
productiesleutelrotatie/-intrekking, een releaseceremonie en vertrouwde
kalendertijd. Issue #41 blijft daarom open; geen acceptatiepunt wordt met
deze wijziging gesloten.
