# TASK-USER-ALLOC: host- en compilebewijs

Basis `cda42f2`, 30 oktober 2026. Dit is geen native OS-acceptatie. De exacte
wrapper-/test-/manifest-/lock-/licentiebestanden staan met SHA-256 in
`checks.json`; `source_sha256` verwerkt achtereenvolgens de gesorteerde
repo-relatieve bestandsnaam + NUL + 8-byte little-endian byteomvang + bronbytes,
voorafgegaan door `nieuw-os heap-allocator source v1\n`.

Vanuit de repositoryroot, Rust 1.90.0 en bestaande toolchainomgeving:

```sh
CARGO_TARGET_DIR=build/user-allocator-target cargo +1.90.0 test --offline --locked --manifest-path sdk/heap-allocator/Cargo.toml
CARGO_TARGET_DIR=build/user-allocator-target cargo +1.90.0 clippy --offline --locked --manifest-path sdk/heap-allocator/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/heap-allocator/Cargo.toml -- --check
CARGO_TARGET_DIR=build/user-allocator-none-target RUSTFLAGS='-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2' cargo +1.90.0 build --offline --locked --manifest-path sdk/heap-allocator/Cargo.toml --release --lib --target x86_64-unknown-none --message-format=json-render-diagnostics
cargo +1.90.0 tree --offline --locked --manifest-path sdk/heap-allocator/Cargo.toml -e features
```

15 tests slagen: twee interne grenscontroles en dertien arena-/GlobalAlloc-tests,
inclusief 40.000 gemengde alloc/free-stappen met gecontroleerde live bytes,
alignment, niet-overlappende extents, canaries en accounting.
De aparte `harness=false` collections-executable gebruikt voor alle Rust
GlobalAlloc-aanvragen uitsluitend één 8-MiB statische BSS-arena: echte Vec,
String, Box, 4096-byte alignment, 1-MiB-inhoud, hergebruik en twee zichtbare
`try_reserve`-OOM's met behouden inhoud. De gerapporteerde highwater is 1.134.168
bytes; dit is een fixturemeting, geen productcapaciteit of prestatieclaim.
Er is geen System/malloc-terugval. Andere tests laten alleen hun eigen arena's
door de hostharness leveren; de library kent geen dergelijke backend.

De broncontrole en Cargo.lock binden de oorspronkelijke cratetarball aan SHA-256
`2b23ac50abb8261cb38c6e2a7192d3302e0836dac1628f6a93b82b4fad185897`.
De feitelijke Cargo-featuregraph bevat uitsluitend de engine, zonder defaults,
spinlockdep, nightly-API of buildscript. De behouden MIT-tekst is upstreamtekst.

Beide `.rlib`-paden zijn uit de daadwerkelijke Cargo `compiler-artifact`-records
genomen, geen aangenomen bestandsnamen. `objdump -d` disassembleerde de ELF64-
x86-64-objectleden: 429 wrapper- en 663 engine-instructies. Geen x/y/zmm-, mmx-
of x87-registerreferenties; ook geen f*/v*-mnemonics, emms, wait, ldmxcsr of
stmxcsr. `nm -u` vond geen malloc/calloc/realloc/free/mmap/sbrk/VirtualAlloc.
Artifacthashes en controleaantallen staan in `checks.json`. De archieven zelf
blijven buiten Git. Deze controle geldt voor deze gebouwde libraries, niet
automatisch voor later gekoppelde applicaties of Rust-runtime-objecten.

`host.txt`, `clippy.txt`, `fmt-check.txt`, `scalar-build.txt` en `features.txt`
behouden werkelijke exitstatus, commando en compacte uitvoer. De volledige
lokale disassembly staat onder `build/logs/user-allocator/`; daarvan zijn alleen
de uitkomsten en hashes gearchiveerd. Er is geen Miri-, QEMU- of fysieke test
uitgevoerd. Native entrypoint, alloc-runtime/linkage, arena-RW+NX/procesisolatie,
heapgroei en procescleanup moeten de hoofdagent nog geïntegreerd aantonen.

De andere agent `git_build_audit` heeft de bron read-only beoordeeld, de
oorspronkelijke 14 tests plus een eigen 40.000-stappen tegenproef uitgevoerd,
de collectionsexecutable herhaald en de gepinde tarball/artifactscans afzonderlijk
gecontroleerd. Geen concrete blocker gevonden. De tegenproef is daarna als
permanente vijftiende test opgenomen en uitgebreid. Dit is AI-peerreview;
Miri, een menselijke audit en native acceptatie worden daarmee niet vervangen.
