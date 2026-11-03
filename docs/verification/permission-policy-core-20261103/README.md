# M1/M2-profiel- en policykern — hostbewijs

**Status:** gerichte hosttests, typecheck, Clippy, formatter en diffcontrole
geslaagd. Geen kernel-/dienstintegratie, geauthenticeerde beleidsopslag,
native rechtenuitgifte, QEMU- of hardwareacceptatie.

- **Bron:** `7e9bddcfc92b6d5f788d7b2ccafb98205af7e0ce`
- **M1:** vier tests voor ConventionalMemory-som, GOP-modus, onbekende
  eigenschappen en ongeldige framebuffer.
- **M2:** vijf tests voor exacte scope, actieve taak/foreground, deny/no-rule,
  canonieke round-trip, malformed/duplicate bytes en ongeldige scope.
- **Policygrens:** codec authenticiteit en rollback niet gecontroleerd;
  policyuitkomst is geen runtime-capability.

## Uitgevoerde controles

```sh
cargo +1.90.0 test --offline --locked -p platform-profile-core -p permission-policy-core
```

Result: exit 0, 9 tests.
De testlog staat op
`/tmp/nieuwos-profiler-toolchain/platform-policy-tests-final.log`,
SHA-256 `1dc913df14c6d0aad519ae36b0ea898afa222ca96086eb5553c9328e116526fe`.

Voor deze native hosttest is GCC 15.2 met `libc6-dev` uit Ubuntu-pakketten
alleen uitgepakt in `/tmp/nieuwos-profiler-toolchain`; er is niets
systeemwijd geïnstalleerd. Exacte omgevingsvariabelen en het commando staan in
de M1-rapportage; de uiteindelijke test gebruikte dezelfde tijdelijke linker
en Rust-flags.

De volgende commando's zijn uitgevoerd met Rust 1.90.0 en elk exit 0:

```sh
cargo +1.90.0 fmt --package platform-profile-core --package permission-policy-core -- --check
cargo +1.90.0 check --offline --locked -p platform-profile-core -p permission-policy-core --tests
cargo +1.90.0 clippy --offline --locked -p platform-profile-core -p permission-policy-core --all-targets -- -D warnings
git diff --check
```

Dit is hostbewijs voor gegevensmodellering en parser-/evaluatielogica, geen
bewijs van persistente opslag, vertrouwde contextinvoer of kernelafdwinging.
