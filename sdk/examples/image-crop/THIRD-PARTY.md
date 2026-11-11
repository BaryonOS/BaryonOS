# Dependency check for image crop

The decoder remains the existing `services/image-codec` with its pinned
ledger/vendor source. No decoder foundation changed.

The optional **host-only** PNG encoder is `png = "=0.18.1"`,
default features off, from <https://github.com/image-rs/image-png>.
Cargo.toml.orig, CHANGES.md and LICENSE-MIT of the registry source on docs.rs
were read before the dependency was added.
Licence MIT OR Apache-2.0, MSRV 1.73. The package include contains source code,
licences and examples, no build.rs. Maintained by image-rs;
the 0.18.1 changelog contains current encoder/decoder improvements.
No nightly/unstable or zlib-rs feature enabled.

This is a std encoder, **not** a native no_std encoder port. Only the
host-cli feature selects this dependency; the crop core uses existing
no_std codec interfaces. Registry versions/checksums of the complete
closure are in Cargo.lock. The actual closure was checked before the first
compilation with cargo metadata, registry Cargo.toml.orig, the licences present
and all custom-build sources:

| Crate | Version | Licence | Build script |
|---|---|---|---|
| png | 0.18.1 | MIT OR Apache-2.0 | none |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | none |
| crc32fast | 1.5.4 | MIT OR Apache-2.0 | checked exception below |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | none |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | none |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | none |
| miniz_oxide | 0.8.9 and 0.9.1 | MIT OR Zlib OR Apache-2.0 | none |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | none |
| simd-adler32 | existing 0.3.10 | existing codec ledger | none |

The repositories are those in the registry manifests: image-rs/image-png,
bitflags/bitflags, srijs/rust-crc32fast, rust-lang/cfg-if, image-rs/fdeflate,
rust-lang/flate2-rs, Frommi/miniz_oxide and oyvindln/adler2.
Maintenance is evident from registry releases/manifests and the png changelog, not an
independent code/security audit. Checksums in Cargo.lock pin all
transitives without adding separate unnecessary direct dependencies.

**Build script exception:** crc32fast/build.rs has been read in full. It
only runs the `RUSTC --version` chosen by Cargo and emits
compile-time cfg/check-cfg for stabilised CPU intrinsics.
No downloads, C toolchain, disk/network operation or extra script chain.
This applies only to the host encoder closure; not in the default no_std core.
Runtime CPU detection in std upstream code is exclusively host functionality.
The actual feature tree selects flate2/miniz_oxide, **not**
zlib-rs or a C backend. Cargo fetch downloads an unused optional zlib-rs
tarball for lock/resolution; that is not a selected build target.
