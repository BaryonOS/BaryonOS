# Kleine image-integratieslice op actuele main

Basis: `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`.
Herkomst: PR #156, bevroren head
`8c837d5c285498352d617f71c804918c443a2b07`; formatrouting uit
`c2c9b64b`, EXIF-correctie uit `b2317dd7` en ruwe-pixelregressie uit
`e5cded7e`. Alleen codec, bijbehorende tests en NATIVE-IMAGES overgenomen;
geen policy-, terminal-, browser-, score- of centrale statusdelta.

Integratiecontract: ondersteunde magicbytes kiezen de bestaande decoder;
geldige JPEG EXIF-orientatie stuurt uitsluitend `rgba8_into` en zijn
displayafmetingen. Originele bytes, ruwe samples en `pixel_hash` blijven
gelijk. TIFF-offsets/lengtes worden begrensd; ongeldige metadata verleent
geen autoriteit. Geen nieuwe dependency, unsafe, bestandstoegang of IPC.

## Werkelijk uitgevoerd

Rust 1.90.0, offline/locked. Omgeving:
`PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`,
`CC=/tmp/nieuw-os-c-toolchain/bin/cc`. Commando's vanuit repositoryroot:

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path services/image-codec/Cargo.toml` | Exit 0: 15 decode-, 5 snapshot/OOM- en 3 upsampletests. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path services/image-codec/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 check --offline --locked --manifest-path services/image-codec/Cargo.toml --target x86_64-unknown-none` | Exit 0; freestanding typecheck, geen VM-test. |
| `cargo +1.90.0 fmt --manifest-path services/image-codec/Cargo.toml -- --check` | Exit 0. |
| `CARGO_TARGET_DIR="$PWD/target/sprint-viewer" cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml` | Eerst exit 101: bestaande `Image`-literal mist nieuwe velden. Na uitsluitend fixturecompatibiliteit exit 0: 1 unit- en 14 viewertests. |
| `CARGO_TARGET_DIR="$PWD/target/sprint-viewer" cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `CARGO_TARGET_DIR="$PWD/target/sprint-viewer" cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml --target x86_64-unknown-none` | Exit 0. |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/image-viewer/Cargo.toml -- --check` | Exit 0. |
| `git diff --check` | Exit 0. |

Volledige logs: `/tmp/nieuw-os-sprint-image-{host,clippy,none}.log`,
`/tmp/nieuw-os-sprint-viewer-{before,green,clippy,none}.log`.

## Grenzen

De viewer gebruikt nog ruwe samples; deze slice claimt geen georienteerde
viewer, native appflow, QEMU, hardwaretest, menselijke review of release.
Het oorspronkelijke volledige PR #156 is hiermee niet integraal beoordeeld
of vervangen. Geen punten toegekend; geen merge uitgevoerd.
Rollback: deze afzonderlijke code-/fixture-/specslice samen terugnemen.
