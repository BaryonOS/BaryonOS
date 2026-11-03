# General-hertest van de begrensde driverreviewfix

Uitgevoerd op 3 november 2026, bron
`c1508aa05b79bcd57bd08bc5d320f6142d82bc71`, worktree
`/home/simon/.copilot/session-state/0fc9ee51f1e94fc69b3031941f626af0/files/sprint-xhci`.
Dit is een hosthertest door een tweede AI-uitvoerder, geen menselijke audit.

Omgeving: `PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`,
`CC=/tmp/nieuw-os-c-toolchain/bin/cc`,
`CARGO_TARGET_DIR="$PWD/target/sprint-review"`. Rust/Cargo 1.90.0,
offline en locked; geen nieuwe dependency of installatie.

| Exact commando vanuit de worktree | Exit | Tests | Gemeten commandoduur |
|---|---|---|---|
| `cargo +1.90.0 test --offline --locked -p driver-core -p pci-core` | 0 | 11 + 8 | 0,45 s |
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/xhci/Cargo.toml` | 0 | 15 | 0,27 s |

| Lokaal logbestand | SHA-256, opnieuw gecontroleerd |
|---|---|
| `/tmp/nieuw-os-sprint-driver-workspace-green.log` | `300cdb8bdd4fddd032379a5654a9e31b6d9d845148c2aee5ce91e162b502595e` |
| `/tmp/nieuw-os-sprint-xhci-green.log` | `18c2a6ee6413e72774082bbfcc67e19f2f271b487810514e4e3e36ba4c0df42a` |

De ruwe logs zijn lokale artifacts, geen publiek geleverde bundle. De
commando's en hashes maken herhaling en artifactvergelijking mogelijk; een
hash zonder het log levert geen onafhankelijke inhoudscontrole op.

De twee oorspronkelijke positieve xECP-fixtures faalden werkelijk op
`5ea1c349`; de owner verplaatste alleen de fixtures buiten de operational
MMIO-regio. Productievalidatie is niet versoepeld. De rootworkspace bevat nu
beide lokale prerequisitecrates. Geen controlleruitvoering, native QEMU,
USB-enumeratie, fysieke hardware of afsluiting van issue #42 bewezen.
