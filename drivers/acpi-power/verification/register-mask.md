# DRV-ACPI-S5-REGISTER-MASK

Basis `ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff`, branch
`copilot/driver-acpi-register-mask`, 3 november 2026.
Alleen nieuwe `tests/register_mask.rs` en dit bewijsbestand.
Geen productieparser, poweroff-I/O, kerneladapter of bevoegdheid gewijzigd.

De tests voeren echte publieke `parse_s5` uit op checksummed FADT/DSDT-fixtures.
Daarna wordt voor elk van de 64 A/B-sleeptypeparen elk van de 65536 mogelijke
huidige PM1-registerwaarden gecontroleerd voor beide banken: SLP_TYP exact de
eigen bankwaarde, SLP_EN aan, alle andere bits ongewijzigd en voorbereiding
idempotent. De tweede test controleert dat een afwezige PM1b-bank de primaire
instelling niet verandert. Geen register wordt werkelijk gelezen/geschreven.

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/acpi-mask"
cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/acpi-power/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/acpi-power/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/acpi-power/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Alle exit 0. 22 bestaande unittests en 2 nieuwe integratietests geslaagd;
1 bestaande externe-QEMU-tablefixturetest standaard genegeerd. Die genegeerde
test is niet als geslaagd geteld bij de standaardrun; de afzonderlijke
expliciete uitvoering hieronder slaagde wel.
0 doctests. Geen nieuwe dependency/toolinstallatie/rootworkspacewijziging.

Logs onder `/tmp/nieuw-os-acpi-mask-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `f93597dad404083775d116fd377a587ae1ad91b22b96f754372a1bb1e8e24103` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `cba9652d19c5917e24049dec94f4bc8ca65692f486b608fb2813aea982d24d16` |
| `none.log` | `e636de838a1141513f6d0779b1b39a61f03b8bb496b6ad4f40f49947e5c39b5a` |

## Expliciete externe firmwarefixture

De vier gedocumenteerde invoervariabelen uit `verification/README.md` zijn
gebruikt. De bestanden waren aanwezig en hun hashes kwamen exact overeen
met het bewaarde captureverslag:

```sh
export NOS_ACPI_FADT=/home/simon/.copilot/session-state/b6c80d1bf5744c2e81c568572bf7c83d/files/h05-fw3/fadt.bin
export NOS_ACPI_DSDT=/home/simon/.copilot/session-state/b6c80d1bf5744c2e81c568572bf7c83d/files/h05-fw3/dsdt.bin
export NOS_ACPI_FADT_ADDRESS=2142736384 NOS_ACPI_DSDT_ADDRESS=2142740480
cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml -- --ignored --exact tests::validates_external_qemu_tables_and_rejects_mutations
```

Exit 0: 1 externe-fixturetest geslaagd, 22 unittests en 2 nieuwe
integratietests uitgefilterd, niet opnieuw uitgevoerd door dit commando.
FADT (244 bytes) SHA-256:
`410716dfb169eaba296ed3c336028843b3cf9fce6ca7c179bb242199cfec9d1a`;
DSDT (8344 bytes):
`95bb7cd45de3e16a85206f9f50cb6597b3b8dd97ec4afef3e3004ef3170430e5`.
Log `/tmp/nieuw-os-acpi-mask-external.log`:
`ba2ff30eae9cbc7a054a0a9f4f97674cfd76f0ee0f17167b7718ff240836c87c`.
Dit hergebruikt eerder opgeslagen firmwarebytes op de host; geen nieuwe
guestcapture of eigen-OS-poweroffproef.

Bewijsgrens: pure hostfixtures/registerberekening, geen shutdown, firmware-
discovery, ACPI-method interpreter of autorisatie bewezen. Geen nieuwe VM-,
fysieke of menselijke tests; librarybuild is geen uitvoering. Geen nieuw
criterium/scorepunten; centraal assessment bij integrator.
Revert van tests/bewijs raakt productiegedrag niet.
