# Q01 dependency and source registration

Checked on 27 October 2026. The core has zero external dependencies. The JSON adapter uses exactly `serde =1.0.229` with derive and `serde_json =1.0.154`; both documented pins were actually resolved, fetched by Cargo unchanged and built with Rust 1.90.0. `Cargo.lock` stores all registry checksums and exact transitive versions. [dependency-inventory.json](dependency-inventory.json) stores package metadata, upstream repository, minimum Rust version, build script hash and notice paths.

| Package | Version | Role | Licence / chosen route | Build script |
|---|---|---|---|---|
| serde | 1.0.229 | Host serialisation | MIT | Compiler version, private module in OUT_DIR |
| serde_core | 1.0.229 | Host serialisation traits | MIT | Compiler/target detection, private module in OUT_DIR |
| serde_derive | 1.0.229 | Compile-time proc macro | MIT | None |
| serde_json | 1.0.154 | Host JSON parser/encoder | MIT | Target arithmetic selection |
| proc-macro2 | 1.0.107 | Compile-time macro infrastructure | MIT | Compiler probes; temporary OUT_DIR/probe files |
| quote | 1.0.47 | Compile-time code generation | MIT | Compiler version |
| syn | 3.0.9 | Compile-time Rust parser | MIT | None |
| unicode-ident | 1.0.27 | Compile-time identifier data | MIT and Unicode-3.0 | None |
| itoa | 1.0.18 | Host integer encoding | MIT | None |
| memchr | 2.8.3 | Host JSON byte search | MIT | None |
| zmij | 1.0.23 | Host number encoding | MIT | Compiler/optimisation profile |

The packaged manifests and six build script files have been read. The actions found in them are local compiler/target detection, generation of configuration/module code and temporary compiler probes. No install script, network download or system modification was found in those build scripts. The scripts remain part of the trusted development chain; this is not a complete code or supply-chain audit of all transitive source. Upstream notices have been copied unchanged to `third-party/`.

The primary documentation confirms the chosen serde versions and typed deserialisation: [Serde 1.0.229](https://docs.rs/serde/1.0.229/serde/), [Serde JSON 1.0.154](https://docs.rs/serde_json/1.0.154/serde_json/) and [Serde container attributes](https://serde.rs/container-attrs.html). `deny_unknown_fields` and typed structs remain active; there is no input via `serde_json::Value`, flatten or a disabled recursion limit. `Value` is used only in the test code to compare synthetic requests/results.

Maintenance: changes to direct pins and `Cargo.lock` are made deliberately, with a provenance/licence/build script check and a repeat of the full regression test. The fetched release versions are available and buildable; no full audit against a current advisory database has been performed. There is no claim that these versions are free of vulnerabilities. Removal plan: the host adapter can be removed/replaced without changing the dependency-free core; for own-OS integration a separate bounded IPC adapter of our own will be built.

## SplitMix64

The deterministic sampler follows Sebastiano Vigna's [SplitMix64 source, 2015](https://prng.di.unimi.it/splitmix64.c), accessed on 27 October 2026. Upstream places the implementation in the public domain under CC0. The mix constants and order are preserved and tied to exact regression fixtures. For simulation only; not a cryptographic generator or physical quantum randomness source.
