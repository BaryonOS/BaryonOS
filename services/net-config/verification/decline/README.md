# TASK-N01-DHCP-DECLINE-SEAM

Exacte bron `d7e7e4342fd8d8f02e299f4942582a0f705d5e03`,
branch `simonvanieperen-native-net-config`.
v3 `2e0cce85c6eaaddf730b5bda0a1f66fc379661caaa6f1a4e99ad5b6d803c5b63`.
Queue `dea97f9b57c6fda9549a48072119663fe155a563`,
blob `53ac59d53fa7daece42b07925d6ecffe852012f6`.

Alleen toegewezen DHCPcodec/testfiles plus reeds toegewezen configfiles
veranderd; set programmatisch tegen1490b77 gecontroleerd en in gates.json.
Geen manifest/lock/dependency/root/SDK/nieuweentropybron of netwerkgrant.

## Resultaat en exacte commands

Veertien gates, alle exit0. Bestaande `tools/common.py::run_logged` registreert
volledige argv/cwd/tijd/exitstatus in elk json; stdout/stderr ongewijzigd.
`gates.json` bevat bronbestanden en rlib; SHA256SUMS bindt artifactbytes.

```text
cargo +1.91.1 test --manifest-path services/net-config/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host
cargo +1.91.1 clippy --manifest-path services/net-config/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-config/Cargo.toml --all -- --check
cargo +1.91.1 test --manifest-path services/net-stack/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host
cargo +1.91.1 clippy --manifest-path services/net-stack/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-stack/Cargo.toml --all -- --check
cargo +1.91.1 test --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host
cargo +1.91.1 clippy --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check
cargo +1.91.1 build --locked --offline --release --target x86_64-unknown-none --manifest-path services/net-config/Cargo.toml --target-dir build/native-net-config-worktree/target/net-config-none
cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host
cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check
cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-none --release --target x86_64-unknown-none
```

28configtests (vijftien coordinator+elf client+tweeclock),24Stack,61codec.
De61codecproeven ook opnieuw met Rust1.90.0 uit de bestaande hoofd-CI.
Baremetal-librarybuilds, GEEN native ELF/runtime of echte NICcompletion.

## Encoder en koppeling

`build_decline(out,xid,mac,address,server)` hergebruikt de bestaande private
requestencoder en validators. Exact BOOTREQUEST/message4,zero secs/flags/ciaddr,
requestedIP/server/clientidentifier en een statische message. Geen
REQUEST-onlyPRL55/maxsize57/lease51 of overige verboden opties.
Exact300bytes, zero pad en onaangeraakte tail buiten de retourlengte.
Alle300tekorte buffers, ongeldige adressen/MAC en all-or-nothing gecontroleerd.
Oude DISCOVER/REQUEST/renewalproeven behouden.

De coordinator fenceert eerst oude adres/route/DNS/publicatie bij conflict en
bewaart lease/XID alleen voor DECLINE. Nieuw broadcastframe via hetzelfde
PacketDevice pas na echte oude TXcompletion. Nieuwe acquisitie wacht op
cooldown EN DECLINEcompletion. Stale/EAGAIN/NotPublished is geen completion;
zelfde id/bytes blijven behouden. Unknown/clockfault blijft terminaal.
Geen conclusie over native DMAownership of exclusieve devicefence.

`decline-api-red.patch` is de exacte test-onlydelta tegenover
`1490b77ac4afe64b87a36a77c29ec50b8d8f301e`;
`decline-api-red.*` eindigde compile101/E0425 (ontbrekende functie).
Daarbij GEEN runtime gedraaid; geen misleidend runtime-roodlabel.
Eerdere echte runtime-regressies/review/fixes staan in ../coordinator.

Primair daadwerkelijk gelezen: RFC2131sectie3.1,Table5en4.4.4,
https://www.rfc-editor.org/rfc/rfc2131.txt . DECLINE MUST;10swachten SHOULD,
hier afgedwongen. Na10conflicten60s volgens RFC5227sectie2.1.1.
Geen overname van RFCcode of nieuwe parser.

## Open native gates

Niet uitgevoerd: native configuratieproces/caller/driver/device-exclusiviteit/
DMAfence/reset/epoch/TCP-CR3; native RNG/retries/autoreconnect; broker-
egress/grantintegratie; native resolver-/apprechten; nieuwe dedicatedCI;
fysieke hardware of menselijke audit. Geen nieuwe punten.
Netwerk0/10->0/10/totaal34->34 als voorstel aan centrale eigenaar.
Geen centrale progress-/README-/PROJECT-STATEwrite door deze NETleaf.
