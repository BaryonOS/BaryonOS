# Dependency choice for the active DOCX import slice

New scoped DOCX text import with real ZIP/OPC binding,
raw DEFLATE and host CLI. The XML parser is maintained upstream code,
not a new XML engine. Exact executed acceptance is in the slice's own receipt;
no native app/Office client claim.

Selected on 3 November 2026:

| Dependency | Pin and provenance | Licence and build | Use/limit |
|---|---|---|---|
| roxmltree | `=0.21.2`, <https://github.com/RazrFalcon/roxmltree/tree/v0.21.2> | MIT OR Apache-2.0; actual registry metadata has no build script; no_std+alloc, unsafe forbidden upstream | Namespace-aware XML, DTD off, no entity resolver, node budget 32768, XML at most 1 MiB |
| memchr | `=2.7.6`, existing upstream closure of roxmltree, <https://github.com/BurntSushi/memchr/tree/2.7.6> | Unlicense OR MIT; build.rs absent (404), default features off | Only no_std substring search code; std/logging/runtime CPU detection off. Upstream may use target-configured SIMD; the native target build still has to confirm this |
| zune-inflate | `=0.2.54`, default features off | Existing ledger `services/image-codec/dependency-ledger.json`; MIT OR Apache-2.0 OR Zlib, no build script | ZIP raw DEFLATE; limit/size hint set, final length and ZIP CRC re-checked; best-effort heap limit, no new compression engine |

The roxmltree Cargo.toml, src/lib.rs, LICENSE-MIT, CHANGELOG and the complete
tag root tree of the initial 0.21.1 candidate have actually been read.
After that the registry reported patch 0.21.2: its Cargo.toml and changelog
have been read; the release of 24 October 2026 fixes a stack overflow on deeply
nested XML by parsing iteratively. That is why 0.21.1 is **not** the selected
product version. Changelog 0.21 also covers entity resolver and
parser corrections. The resolver function is **not** enabled.
The memchr Cargo.toml/README/LICENSE-MIT have been read: no_std explicitly
supported, MSRV 1.61, optional log/core deps not selected.
This is a provenance/design check, not an independent security audit.

Upstream XML uses infallible allocations despite the input/node budget.
Native use requires a separate bounded parser process; this slice
will not claim native OOM recovery, a kernel capability or a sandbox.
No third-party source files or licence texts copied.
Final registry checksums are in Cargo.lock. Actual
`cargo metadata` targets contain no `custom-build` for any of the three upstream crates;
`cargo tree -e features` contains exclusively these three
crates without default features. All three also compile for
`x86_64-unknown-none`. The updated roxmltree registry source contains
`#![no_std]`; the original MIT licence artifacts of roxmltree and
memchr are present. Lockfile checksums, manifest/licence hashes and
executed commands are in the receipt.
The first offline lock attempt failed with exit 101 because of a missing roxmltree cache;
after that only pinned public registry crates were fetched.
