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
| Update-core met TEST-policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-host /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path crates/update-core/Cargo.toml --features test-signing-anchors` | 7 unit- en 6 policytests, exit 0 |
| Update-tool | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-tool /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path tools/update-tool/Cargo.toml` | 2 integratietests, exit 0 |
| Update-core zonder TEST-feature | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-default /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path crates/update-core/Cargo.toml` | 7 tests, exit 0; TEST-policytests zijn niet inbegrepen |
| UEFI-testloader | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-loader /home/simon/.cargo/bin/cargo +1.90.0 build --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot-test --target x86_64-unknown-uefi` | exit 0 |
| Update-service testbuild | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-service python3 -B sdk/examples/update/build.py` | exit 0 |
| Formatter | `cargo +1.90.0 fmt --manifest-path boot/loader/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path crates/update-core/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path tools/update-tool/Cargo.toml -- --check`; `cargo +1.90.0 fmt --manifest-path sdk/examples/update/Cargo.toml -- --check` | alle exit 0 |
| Diffcontrole | `git diff --check` | exit 0 |
| Loader zonder policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-negative-loader /home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot --target x86_64-unknown-uefi` | exit 101 verwacht; expliciete policyfout |
| Update-service zonder policy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-negative-service /home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/update/Cargo.toml --features native --bin update-service --target x86_64-unknown-none` | exit 101 verwacht; alleen expliciete policyfout |
| Update-core Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-core-clippy /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path crates/update-core/Cargo.toml --features test-signing-anchors -- -D warnings` | exit 0 |
| Update-tool Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-tool-clippy /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path tools/update-tool/Cargo.toml -- -D warnings` | exit 0 |
| Update-service Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-service-clippy /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/update/Cargo.toml --features native,test-signing-anchors --bin update-service --target x86_64-unknown-none -- -D warnings` | exit 0 |
| Loader Clippy | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/nieuw-os-b03-rebased-loader-clippy /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path boot/loader/Cargo.toml --features signed-boot-test --target x86_64-unknown-uefi -- -D warnings -A clippy::not_unsafe_ptr_arg_deref` | exit 0; alleen genoemde bestaande lint toegestaan |
| Preflight | `PATH=/tmp/nieuw-os-c-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH python3 -B tools/preflight.py --strict` | exit 2: QEMU en OVMF ontbreken |
| Progressmeter | `python3 tools/progress.py --check` | exit 0; 41/100 consistent |

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
| `/tmp/nieuw-os-b03-rebased-python.log` | `2aea3551f7f5a30be0e89a1e11375679c2950b94432acc9b771eeaff6de7b865` |
| `/tmp/nieuw-os-b03-rebased-update-core.log` | `cd9e6e418d003d0bcc5a0784f84cdf2c344b7b4dd22a481c5ba9e4670ac8b9c8` |
| `/tmp/nieuw-os-b03-rebased-update-tool.log` | `bafdd9d557305db9557bddb22ceb00b926516be654da8a840cd472c78017a4a8` |
| `/tmp/nieuw-os-b03-rebased-default.log` | `040b980ea64b95692ab813b853b1756c48d05d50f422bbac1ec95ede91ef2311` |
| `/tmp/nieuw-os-b03-rebased-loader-build.log` | `5ea4fae2017dd10734ba9a299c2dfe1d88443c747089ad99131fb2c8aa52f7ca` |
| `/tmp/nieuw-os-b03-rebased-service-build.log` | `f2eb3f3ca00de6e3a1a2b2084ce8bae2ea85d12f860490444fbea7152e097ec0` |
| `/tmp/nieuw-os-b03-rebased-negative-service.log` | `f76441bd34510a5c0930a2e8630bf7700268a2ca2eaf09b7c612f381f90be511` |
| `/tmp/nieuw-os-b03-rebased-negative-loader.log` | `b385b6135b1aab5e0af993a5ff9f1290d69488ac9f89a24aa7925e6513a168af` |
| `/tmp/nieuw-os-b03-rebased-preflight.log` | `3a9624dee063ddbfeb1d3324fc5ee8b6afed1a4e85e7849bd9e3d8810b875c4a` |
| `/tmp/nieuw-os-b03-rebased-core-clippy.log` | `ef6ae8b62546ad0982642aa11dd338a89fc81040825de9e6cf841209179adb9f` |
| `/tmp/nieuw-os-b03-rebased-tool-clippy.log` | `fca740e0a976574b95718c225964a48744b5ab623df15915728fc0c8226bb055` |
| `/tmp/nieuw-os-b03-rebased-service-clippy.log` | `3e866b519c6c3c3ee797e18aa9c830368bdc1e4b9c02849901d8e30eaeeb819f` |
| `/tmp/nieuw-os-b03-rebased-loader-clippy.log` | `e7b5a33da99eebaaaaacb727c6d1116ada5bbbf628e879c73991fe24a0385b47` |
| `/tmp/nieuw-os-b03-rebased-loader-clippy-strict.log` | `6625d72c200a52597aea33b24838f56e8aa26f97ae05ee6b639450a63ef99969` |
| `/tmp/nieuw-os-b03-rebased-progress.log` | `1963f64b94a52a78464f272cac286fcafea13fea2314c5ea8cddbd65dd78be33` |
| UEFI testloader artifact | `068c91c32a6895096abf5079cf98d5f2049288f7c56ef10094a23aa7115edeb8` |
| Update-service artifact | `a45f777872edb82d5937659273bc8b18b97f180bee9c7412cf0856735fb05e52` |

## Open bewijsgrenzen

De volledige native 18-staps B03-QEMU-gate is **niet uitgevoerd**: lokale
preflight mist `qemu-system-x86_64` en OVMF. Een hosttest of cross-build
vervangt die gate niet. Ook ontbreken productieondertekenankers,
productiesleutelrotatie/-intrekking, een releaseceremonie en vertrouwde
kalendertijd. Issue #41 blijft daarom open; geen acceptatiepunt wordt met
deze wijziging gesloten.
