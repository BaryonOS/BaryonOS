# Third-party components in the initial boot probe

Recorded on 27 October 2026, based on the Cargo package actually retrieved and the root `Cargo.lock`.

## r-efi 7.1.0

| Item | Verified value |
|---|---|
| Purpose | UEFI reference types, protocol definitions and constants; no bootloader or kernel implementation |
| Upstream | [r-efi/r-efi](https://github.com/r-efi/r-efi) |
| Direct pin | `r-efi = { version = "=7.1.0", default-features = false }` in `boot/loader/Cargo.toml` |
| Source | Cargo registry `https://github.com/rust-lang/crates.io-index` |
| Cargo-checksum | `b0ff379dd174eec5b2cab797063a2e7592dfcf0bf0323a50eeee724ee3537f13` |
| Packaged VCS reference | `a45901440b107b966273dcacc5eaff95c9a5bfbd` from `.cargo_vcs_info.json`; package identity remains determined by version/checksum |
| Minimum Rust version in the manifest | 1.68 |
| `no_std` | `#![cfg_attr(not(test), no_std)]` in the actual `src/lib.rs` |
| Build script | `build = false` |
| Dependencies in this configuration | No external transitive dependency; optional `rustc-std-workspace-core` is disabled |
| Upstream licence options | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| Selected route for this project | MIT |

The source review covered the normalised packaged `Cargo.toml`, `README.md`, `src/lib.rs`, `.cargo_vcs_info.json`, `AUTHORS` and the root lockfile actually generated. The dependency is linked only to the UEFI loader; the project's own kernel does not use this crate. This does not demonstrate a complete code audit or absence of vulnerabilities.

[r-efi-MIT.txt](r-efi-MIT.txt) is an **unmodified copy of upstream `AUTHORS`** from package 7.1.0. That file contains the MIT permission, copyright notices and authors, alongside the two alternative upstream licensing routes. The complete original is retained to avoid truncating notices; this project selects the MIT route. No independently written replacement licence text has been added.

Changing the version or checksum is an intentional dependency change: review provenance, manifest, licence, features and any build scripts again, and run the loader/boot tests. Preserve notices during distribution. Removing or replacing r-efi requires reassessment of the raw UEFI ABI bindings; it must not silently introduce an existing loader or kernel as the foundation.

## Other development components

The compiler, emulator and firmware are separate development components. Their actual local provenance and hashes are recorded in [DEVELOPMENT.md](../DEVELOPMENT.md) and `build/toolchain-provenance.json`. The EDK II firmware is the packaged development variant; its build commit has not been independently established. This boot-dependency notice is not a complete distribution SBOM or publication approval.

The quantum host experiment has its own lockfile, [dependency register](../../experiments/quantum/THIRD-PARTY.md) and preserved upstream notices. The quantum core has no external dependencies. The Materiaal browser prototype has no external runtime packages or fonts.
