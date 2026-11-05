# DRV-TPM-IO-FAILURE

Basis `ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff`, branch
`copilot/driver-tpm-io-failure`, 3 november 2026.
Alleen `tests/core.rs` en dit bewijsbestand wijzigen. Geen productiecode,
NV-command, kerneladapter, autoriteit, dependency of hardware-I/O gewijzigd.

Een wrapper rond de bestaande model-Io telt iedere register-/bufferactie in
de geslaagde Startup(Clear)-flow: acquire, ready, command, completion,
bufferwissen, idle en locality release. Voor iedere actie wordt afzonderlijk
een fout voor de modelaccess en een fout na de volledig uitgevoerde access
geinjecteerd. Daardoor worden ook writes die het model al hebben bereikt
maar toch IoError teruggeven getest. Dit is geen native Startup-opdracht.

Iedere route moet de injectie bereiken, expliciet Fault::Io teruggeven en
in Quarantined eindigen. Herhaalde acquire/execute/release/detach doen daarna
geen verdere I/O, veranderen geen buffer en herhalen START niet. Drop doet
evenmin nieuwe I/O en de quarantainehook wordt exact eenmaal aangeroepen.
De positieve referentieflow moet alle ACKs ontvangen en detach toestaan.

Vanuit de eigen worktree, bestaande tools, geen installatie:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/tpm-io"
cargo +1.90.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/tpm-crb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Alle exit 0. Suite: 24 integratietests geslaagd, 0 doctests.
Logs onder `/tmp/nieuw-os-tpm-io-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `f51c00149e26efca24e03b14cd8c0fa2835c107aad5ac051534bc5a6ef2ff9b4` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `d9ec2dfea04751722ed43cf8a80b472e18901316c25c9954ef5254094e95a6cf` |
| `none.log` | `83cb35757bdec2f2d41336e31d6140c8cb729cb59f5e39b6682f6cd3b74befa6` |

Bewijsgrens: geserialiseerd hostregister-/buffermodel. Geen fysieke TPM,
native MMIO, NV-autorisatie, cryptografie of rollbackanker bewezen. Deze
test verandert geen productie- of security-authority. Nieuwe VM-, fysieke
en menselijke controles niet uitgevoerd; freestanding build is geen runtime.
Geen criterium gesloten of scorepunten; centraal assessment bij integrator.
Revert van tests/bewijs raakt productiegedrag niet.
