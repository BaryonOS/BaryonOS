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

Dit is een broncheckpoint met nog uit te voeren nieuwe qualitygates en echte
dedicated CI. Oudere algemene groene checks zijn geen TLS-testbewijs.
Geen punten; netwerk0/10 en totaal34 blijft het voorstel.
Geen mainrebase/merge; lifecycle bewaakt de integratie na #74.
