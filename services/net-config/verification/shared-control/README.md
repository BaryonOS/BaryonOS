# Bestaande DHCP-client en gedeeld control-TX-pad

Exacte bron `f6d9f1f75557d1ee60cfe8d07c3b671f85c5e043`,
branch `simonvanieperen-native-net-config`.
v3 `68736b9d46a242d47db2f179689d52f1de0741277e5ede2ea8b39e6edd71ea75`.
Dit is nog geen DHCP-/probe-/DNS-coordinator of native runtime.

Tien gates, alle exit0, door bestaande `tools/common.py::run_logged`.
De ongewijzigde `final-*.json` bevatten exacte argv/cwd/tijden/status,
`*.stdout.txt`/`*.stderr.txt` de volledige output.

Voor elk van net-config, net-stack, net-protocol:

```text
cargo +1.91.1 test --manifest-path services/<crate>/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host
cargo +1.91.1 clippy --manifest-path services/<crate>/Cargo.toml --locked --offline --target-dir build/native-net-config-worktree/target/net-config-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/<crate>/Cargo.toml --all -- --check
```

Plus:

```text
cargo +1.91.1 build --locked --offline --release --target x86_64-unknown-none --manifest-path services/net-config/Cargo.toml --target-dir build/native-net-config-worktree/target/net-config-none
```

13 configtests: elf bestaande DHCP-pakketproeven en twee checked-clockproeven.
24 Stacktests: vijftien bestaande, zeven adreslifecycle, een shared-control
slot/id/completioncase en een private id-exhaustioncase.
58 bestaande codecproeven. Baremetal-library, geen native ELF/handshake.
Alle controleframes en TCP gebruiken hetzelfde PacketDevice TX-slot en dezelfde
id-/completionregels; geen tweede NIC-producer.

Clientbron exact overgenomen van16adfe5. `client-import.patch` is het complete
verschil: alleen een importregel (private helpergrens en fmt). Automatisch
bytegelijk gecontroleerd na die ene vervanging, niet slechts testaantallen.
Eigen lokale Stack-/codecdependencies; de registrybron-/versieset is
programmatisch identiek aan de bestaande Stack en de lockfile gepind.
Geen nieuwe protocol-/SDK-/driver/rootwrites of trustboolean.

De eerste gates op0d73169 zijn eerlijk rood bewaard:
`checkpoint-net-config-clippy.*` exit101: test assertte een constante; f6d9f1f
controleert de daadwerkelijke conversiewaarde, geen lintuitzondering.
`checkpoint-net-config-none.*` exit101: invocation vergat bekende bestaande
host-cc-env voor Cargo-buildscripts. Geen dependencyinstallatie; correcte
bestaande env hergebruikt bij de geslaagde baremetalgate.
Geen van die rode pogingen is groen geherlabeld.

`gates.json` bevat bronbestanden, rlib en v3; SHA256SUMS bindt de artifactbytes.
Niet uitgevoerd: echte configuratiedienst, RFC5227-probes, DNS-toepassing,
native caller/device/fence/CR3-acceptatie, nieuwe dedicatedCI, fysieke hardware
of menselijke audit. Netwerk0/10/totaal34 ongewijzigd voorstel aan centrale
voortgangseigenaar. Volgende stap blijft coordinator op ditzelfde devicepad.
