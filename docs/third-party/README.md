# Third-party boot component inventory

Initially recorded on 27 October 2026 for r-efi 7.1.0. Re-reviewed on 17 November 2026 for the exact 7.2.0 registry package and root `Cargo.lock`; historical build evidence remains version-specific.

## Current loader dependency: r-efi 7.2.0

| Item | Verified value |
|---|---|
| Purpose | UEFI reference types, protocol definitions and constants; no bootloader or kernel implementation |
| Upstream | [r-efi/r-efi](https://github.com/r-efi/r-efi) |
| Direct pin | `r-efi = { version = "=7.2.0", default-features = false }` in `boot/loader/Cargo.toml` |
| Source | Cargo registry `https://github.com/rust-lang/crates.io-index` |
| Cargo-checksum | `985ede7b27e96936c369ba18787c9178af0ef38d130792fc88e936242a7071c4` |
| Packaged VCS reference | `c0371abfefa6e779867c378f62f750747e320ca9` from `.cargo_vcs_info.json`; package identity remains determined by version/checksum |
| Minimum Rust version in the manifest | 1.68 |
| `no_std` | `#![cfg_attr(not(test), no_std)]` in the actual `src/lib.rs` |
| Build script | `build = false` |
| Dependencies in this configuration | No external transitive dependency; optional `rustc-std-workspace-core` is disabled |
| Upstream licence options | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| Selected route for this project | MIT |

The source review covered the normalised packaged `Cargo.toml`, `README.md`, `src/lib.rs`, `.cargo_vcs_info.json`, `AUTHORS` and the root lockfile actually generated. The dependency is linked only to the UEFI loader; the project's own kernel does not use this crate. This does not demonstrate a complete code audit or absence of vulnerabilities.

[r-efi-MIT.txt](r-efi-MIT.txt) is an **unmodified copy of upstream `AUTHORS`** from package 7.1.0, rechecked byte-identical against package 7.2.0 (`SHA-256 cfb1806058da831762bbfd08d2f44da0f4cba49339e0782d8985ed84d42de75e`). That file contains the MIT permission, copyright notices and authors, alongside the two alternative upstream licensing routes. The complete original is retained to avoid truncating notices; this project selects the MIT route. No independently written replacement licence text has been added.

[The version-bound review and test receipt](../verification/phase0-pr344-20261117/README.md) records the 71,579-byte registry archive, matching checksum/VCS, unchanged feature/dependency set and byte-identical notice. Version 7.2.0 adds three protocol modules; existing ABI definitions are unchanged. This does not substitute for loader and native update tests.

Changing the version or checksum is an intentional dependency change: review provenance, manifest, licence, features and any build scripts again, and run the loader/boot tests. Preserve notices during distribution. Removing or replacing r-efi requires reassessment of the raw UEFI ABI bindings; it must not silently introduce an existing loader or kernel as the foundation.

## Other development components

The compiler, emulator and firmware are separate development components. Their actual local provenance and hashes are recorded in [DEVELOPMENT.md](../DEVELOPMENT.md) and `build/toolchain-provenance.json`. The EDK II firmware is the packaged development variant; its build commit has not been independently established. This boot-dependency notice is not a complete distribution SBOM or publication approval.

The quantum host experiment has its own lockfile, [dependency register](../../experiments/quantum/THIRD-PARTY.md) and preserved upstream notices. The quantum core has no external dependencies. The Materiaal browser prototype has no external runtime packages or fonts.
