# Artifact store core verification

Status: local verification of a read-only contract crate; not native service,
installer, loader, or release acceptance.

- Source checkpoint: `0816a530bda9006dbb5c96a96061d8fb29e385fd`
- Branch: `simon/native-artifact-store-contract`
- Remote branch was verified to point to the same commit.
- Toolchain: pinned Rust `1.90.0`; Cargo was run through `python3 -B
  tools/dev.py`. Host tests used the existing read-only host-tools bundle from
  a sibling checkout; no packages were installed. This local bundle is not a
  CI dependency.

| Gate | Exact command | Result |
|---|---|---|
| Host tests | `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p artifact-store-core` | exit 0; 7 passed |
| Clippy | `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p artifact-store-core --all-targets -- -D warnings` | exit 0 |
| Formatting | `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml -- --check` | exit 0 |
| Freestanding library | `python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none -p artifact-store-core` | exit 0 |
| Diff whitespace | `git diff --check` | exit 0 |

The freestanding artifact was
`target/artifact-store-core-final-none/x86_64-unknown-none/release/libartifact_store_core.rlib`,
SHA-256 `18dca212ccccda6d34d8ec1400d4b5729607023c1d976792a32e0537e54d659a`.

Local command logs:

- `/tmp/nieuw-os-artifact-store-core-tests.log`
- `/tmp/nieuw-os-artifact-store-core-clippy.log`
- `/tmp/nieuw-os-artifact-store-core-fmt.log`
- `/tmp/nieuw-os-artifact-store-core-none.log`

The GitHub Actions workflow was added but has not yet run on GitHub. No QEMU
guest, kernel caller binding, native NAS/app integration, writable store,
durability, cryptographic trust, or full-release criterion was tested or
accepted by these gates.

## Herhaling na main-integratie

Main `ca45a7dabe6a040921de03bdc2fad3e5045a75ba` is zonder conflicten
inhoudsbehoudend gemerged naar
`53c2f147ad299b16d8dc0136afefe7f845cd0117`. De remote branch wees na push naar
dezelfde commit. Alle vier bovenstaande Cargo-commando's en `git diff --check`
zijn op deze mergebron opnieuw uitgevoerd: ieder exit 0, opnieuw 7 hosttests
geslaagd. Dezelfde bestaande host-toolsbundel en eigen targetmappen zijn
hergebruikt; geen installatie of `cargo clean`.

Logs van deze herhaling:
`/tmp/nieuw-os-artifact-store-53c2-{tests,clippy,fmt,none}.log`.
Dit voegt geen native runtimebewijs of rubricpunten toe; advies blijft
34% naar 34%, met A02, L01 en R01-R03 open. Centrale assessment en
PROJECT-STATE blijven bij de manager.
