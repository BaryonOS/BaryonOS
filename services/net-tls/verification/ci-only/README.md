# TASK-N81-TLS-CI - afzonderlijke workflowdelta

Toegewezen queuebron `b58925f9ce1a9a4d045ba0753f007e7d070be7c2`.
Basis frozen #81 `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`.
Alleen nieuw `.github/workflows/net-tls-quality.yml` en dit eigen
`services/net-tls/verification/ci-only/**`-bewijs.

De bestaande algemene CI testte de zelfstandige TLS-workspace niet. De nieuwe
job voert daarom echt `services/net-tls/Cargo.toml` locked hosttests, Clippy
all-targets/-Dwarnings, fmt en x86_64-unknown-none release-librarybuild uit.
Rust1.90.0 volgens de bevroren crate; checkoutactie hergebruikt de bestaande
gepinde repositoryversie, contentsread-only en geen opgeslagen credentials.
Geen pathfilter of voorwaardelijke testskip, ook concept-PRs worden getest.

Geen TLS-/CORE-/SDK-/dependencylock-/provider-/grantwijziging. De native
cryptoprovider, vertrouwde certificaattijd, proces/RANDOM-binding, mTLS-signer
en TCP-bootgate blijven open. Librarybuild is geen handshake of runtime.
De latere SEC72 SDKDropversie wordt hier niet stil geintegreerd/getest.

Broncheckpoint `1b3c279de491f2fc0000b5c225eb70c2a704ea46` heeft vier echt
uitgevoerde lokale gates, alle exit0: 13 hosttests, Clippy, fmt en none-library.
Exacte argv/cwd/tijd/returncode staan in `tls-*.json` (bestaande
`tools/common.py::run_logged`); stdout/stderr ongewijzigd apart opgeslagen.
Toolchain Rust1.90.0; verschillende targetmappen
`target/tls-ci-host` en `target/tls-ci-none`, gedeelde downloadcache hergebruikt.
Geen nieuwe toolchain/dependency geinstalleerd. Gitdiff tegen4a0 is
programmatisch beperkt tot nieuwe workflow en deze bewijsmap; adaptercode,
CORE, SDK, manifests en locks zijn identiek.

Lokale commands:

```text
cargo +1.90.0 test --manifest-path services/net-tls/Cargo.toml --locked --offline --target-dir target/tls-ci-host
cargo +1.90.0 clippy --manifest-path services/net-tls/Cargo.toml --locked --offline --target-dir target/tls-ci-host --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path services/net-tls/Cargo.toml --all -- --check
cargo +1.90.0 build --locked --offline --release --target x86_64-unknown-none --manifest-path services/net-tls/Cargo.toml --target-dir target/tls-ci-none
```

Echte dedicated Actions-CI moet op de gepushte head apart worden geobserveerd,
nog geen groenclaim daarvoor. De workflow gebruikt dezelfde commands zonder
lokale offline-/targetmapkeuze. Oudere algemene groene checks zijn geen
TLS-testbewijs.
Geen punten; netwerk0/10 en totaal34 blijft het voorstel.
Geen mainrebase/merge; lifecycle bewaakt de integratie na #74.
