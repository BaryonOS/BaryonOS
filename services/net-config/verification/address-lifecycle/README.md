# Eerste werkende adres-/routestap

Bron `2c0e37af9299f8ab6ef9561f5a2f54574f39dc70`, basisca45a7d,
branch `simonvanieperen-native-net-config`.
v3 `8cdc2d1cd9f3dd9d9370cef5a01de2188c5440da8c13e0ee55f11eec38d65443`.
Geen native configuratieservice of complete DHCP-lifecycle.

Zeven commandogates, alle exit0, op de exacte bron:

```text
cargo +1.91.1 test --manifest-path services/net-stack/Cargo.toml --locked --offline --target-dir target/net-config-host
cargo +1.91.1 clippy --manifest-path services/net-stack/Cargo.toml --locked --offline --target-dir target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-stack/Cargo.toml --all -- --check
cargo +1.91.1 build --locked --offline --release --target x86_64-unknown-none --manifest-path services/net-stack/Cargo.toml --target-dir target/net-config-none
cargo +1.91.1 test --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir target/net-config-host
cargo +1.91.1 clippy --manifest-path services/net-protocol/Cargo.toml --locked --offline --target-dir target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check
```

Resultaat: zeven nieuwe lifecyclecases + vijftien bestaande Stacktests,
58 bestaande protocoltests; Clippy/fmt en baremetal-librarybuild groen.
Geen ELF/runtime/handshake. `net-*.json` zijn de ongewijzigde records van
bestaande `tools/common.py::run_logged`: exacte argv/cwd/tijd/exitstatus.
stdout/stderr afzonderlijk bewaard. `gates.json` bevat bron-/artifacthashes.

Testbare interfaces: unconfigured start, valide adres/default-route toepassen,
coherent withdraw/reapply, pending-TX behouden tot echte completion, geen
oude RST/RX/neighborstate, invalid gateway/adres, active-socket Busy en
failclosed withdrawal ook bij klokrollback/overflow. De offlinkcase ziet een
echte Stack-ARP voor de gateway; geen hostsocket of native netwerkbewijs.

## Rode gevallen en gemaakte correcties

De eerste gate op basisca45 compileerde niet: interface ontbrak, geen runtime
gedraaid. Dat eerdere bewijs staat een map hoger en blijft zo gelabeld.

Daarna een echte runtimefailure op ongecommit werkstadium na45b1e19:
`reapply-rst-red.patch` bewaart de volledige exacte Stack/test-delta tegenover
die commit, `reapply-rst-red.txt` is exit101. Abort alleen behield een oude
RST-tuple; reapply van hetzelfde adres gaf oud egresswerk vrij.
Withdrawal verwijdert nu eerst bronadres/routes, reset upstream
neighbor-backoffmetadata via SocketSet remove/add met behoud van buffers en
handles, en laat gepinde smoltcp-dispatch oude tuples zonder lokaal adres
resetten VOOR emissie. Geen fake TX-completion, geen geleende kloktoekomst,
geen tijdelijke listener of tweede stack. De regression is daarna groen.

Een tweede testfixture faalde6/7 omdat zij op tick3 direct ARP verwachtte;
de upstream interface houdt echte ARP-rate-limiting ook na cacheflush.
De test laat nu begrensd de synthetische tijd voortgaan en controleert het
nieuwe id en ARP-frame. `configuration-fixed-host.txt` bewaart die fout,
maar de volledige bronfingerprint van dat tussenstadium is niet bewaard:
diagnose, GEEN brongebonden acceptatieclaim. De groene gates zijn wel
volledig aan bron2c0e37a gebonden.

## Open vervolg, geen eindacceptatie

DHCP-clientbron16adfe5 moet nog binnen het toegewezen configgebied worden
hergebruikt: gecontroleerde seconden/100Hz-conversie, T1/T2/expiry, DNS,
NAK/conflict/linkdown/reconnect en echte RFC5227-probes. Geen public trustboolean.
Alle controleframes en TCP moeten hetzelfde PacketDevicecompletionpad houden.
Een ongepubliceerd/Unknown-TX-slot wordt niet automatisch vrijgegeven;
echte driverreset/quiescence/epoch/issuer blijft bij bestaande eigenaren.

Niet uitgevoerd: native app/DHCP/configuratie, nieuwe CI op deze bron,
fysieke hardware, menselijke audit, formeel bewijs of releaseacceptatie.
TCP-CR3-blokkade blijft apart. Scorevoorstel netwerk0/10, totaal34 blijft gelijk;
centrale voortgangsbestanden zijn niet door NET gewijzigd.
Rollback van deze nog ongeintegreerde leaf laat historische statische Stack
en andere branches intact.
