# TASK-N04-NET-BROKER - bewijs op bron 1a14890

Dit archief is hostmodel- en librarycompilebewijs, geen native service,
kernel-/deviceafdwinging, geauthenticeerde peer of productacceptatie.

- Bron: `1a14890506ad62a71853119a8504b0a184df8a9b`.
- Basis/ouder: `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`, concept-TLS #81.
- Branch: `simonvanieperen-net-broker`.
- v3-bronfingerprint:
  `4799a328ef1f5d4b19b7cce47d3c9eac5825394cf83b88b3dc409559adff332e`.
- Baremetal-rlib SHA256:
  `322aa3e6fd8850e2327322b272415e73036c6ab02ab3c370ec9846f616edd541`.
- Datum: 30 oktober 2026, Europe/Amsterdam. Toolchain Rust 1.91.1.

`gates.json` bevat de exacte argv's, exitstatussen, gemeten `time -p`-duur,
acht onveranderde registryversies/checksums en tien bronblob-SHA256's.
De registrygraaf is programmatisch exact vergeleken met de bestaande
`services/net-stack/Cargo.lock`; geen nieuwe registryversie of feature in
de productieboom. De extra directe smoltcp-devdependency is alleen de
synthetische testtegenpartij. De echte client blijft de bestaande Stack.

## Uitgevoerde commando's en resultaten

Werkmap: `build/net-broker-worktree` binnen de NET-checkout. Elk command
uitgevoerd op bovenstaande bron, met eigen host- en baremetal-targetmap.

```text
cargo +1.91.1 test --locked --offline --manifest-path services/net-broker/Cargo.toml --target-dir target/net-broker-host
cargo +1.91.1 clippy --locked --offline --manifest-path services/net-broker/Cargo.toml --target-dir target/net-broker-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-broker/Cargo.toml --all -- --check
cargo +1.91.1 build --locked --offline --release --target x86_64-unknown-none --manifest-path services/net-broker/Cargo.toml --target-dir target/net-broker-none
cargo +1.91.1 test --locked --offline --manifest-path services/net-stack/Cargo.toml --target-dir target/net-broker-host
cargo +1.91.1 clippy --locked --offline --manifest-path services/net-stack/Cargo.toml --target-dir target/net-broker-host --all-targets -- -D warnings
cargo +1.91.1 fmt --manifest-path services/net-stack/Cargo.toml --all -- --check
```

| Log | Resultaat | Exit | Duur (s) |
|---|---|---|---|
| broker-host.txt | 28 unit + vier compile-fail tests groen | 0 | 0.08 |
| broker-clippy.txt | all-targets, warnings geweigerd | 0 | 1.08 |
| broker-fmt.txt | formattercheck groen | 0 | 0.05 |
| broker-none.txt | echte baremetal-release-librarybuild, geen ELF/runtime | 0 | 1.55 |
| stack-host.txt | negen adapter + zes conversatietests groen | 0 | 0.38 |
| stack-clippy.txt | all-targets, warnings geweigerd | 0 | 0.20 |
| stack-fmt.txt | formattercheck groen | 0 | 0.03 |

De korte hostduur gebruikt bestaande buildintermediates; de testbinaries en
doctests zijn werkelijk uitgevoerd. Geen cachehit wordt als testbewijs geteld.
Alle geciteerde logs bevatten de runneroutput en gemeten duur.
De workflow voert dezelfde gates uit zonder lokale `--offline`/targetmapkeuze.
De nieuwe GitHub Actions-run is hier **niet geobserveerd**, dus niet groen
verklaard. Andere verplichte CI-/releasegates blijven afzonderlijk nodig.

De bestaande lokale Cargo/Rustup-cache onder `.tools/nieuw-os` is hergebruikt.
Hostlinkeromgeving voor deze Linux-installatie:

```sh
export CARGO_HOME="<primaire NET-checkout>/.tools/nieuw-os/cargo"
export RUSTUP_HOME="<primaire NET-checkout>/.tools/nieuw-os/rustup"
export PATH="$CARGO_HOME/bin:$PATH"
export LD_LIBRARY_PATH=/tmp/nieuw-os-dhcp-hostcc/root/usr/lib/x86_64-linux-gnu
export GCC_EXEC_PREFIX=/tmp/nieuw-os-dhcp-hostcc/root/usr/lib/gcc/
export COMPILER_PATH=/tmp/nieuw-os-dhcp-hostcc/root/usr/libexec/gcc/x86_64-linux-gnu/13
export LIBRARY_PATH=/tmp/nieuw-os-dhcp-hostcc/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-dhcp-hostcc/root/usr/lib/gcc/x86_64-linux-gnu/13
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuw-os-qemu-102/host-cc
```

Geen toolchain/dependency geherinstalleerd, geen Cargo-clean en geen gelijktijdige
writers in dezelfde targetmap. Native library gebruikte eigen
`target/net-broker-none`, hosttests `target/net-broker-host`.

## Wat de tests doen

Caller-, policy- en servicecontexten komen uitsluitend uit private fixtures in
een `cfg(test)`-unitmodule. `autotests=false` voorkomt dat deze interne
rechtentest als publieke integratie-issuer wordt behandeld. Er is geen
fixture-Cargofeature of productieconstructiepad. Vier compile-fail doctests
weigeren externe contextconstructie/fixture-import; de productie-library
crosscompileert zonder testmodule en zonder unsafe.

Getest: normale connect, exacte bestemming/poort, individuele rechten,
cross-caller/startincarnatie/dienstinstantie en helperweigering, out-of-bounds
en stale tokens, slotuitputting, generatieoverflow, grant- en socketintrekking,
inclusieve timeout, rollback/overflow, malformed ingress, definitief
niet-gepubliceerde backpressure versus echte modelcompletion en terminale
Unknown/verkeerde completion zonder ACK/retry/release.

Vijf conversatieproeven voeren echte Ethernet/ARP/TCP-frames door dezelfde
productie-Stack en de brokercore op een synthetisch modelmedium:
normale 384-byte aanvraag/96-byte antwoord, gedeeltelijk verzenden en
backpressure, SYNverlies/retransmissie met dezelfde sequentie, totale
linkuitval/timeout, intrekking met gequeue-de ontvangst en gesloten-slotreuse
zonder oude RX naar een nieuwe caller. Modeltijd is expliciet synthetisch;
`Publication::Completed` is hier modelmediumcompletion, geen NIC-/peer-ACK.

## Historische rode WIP-proef

`conversation-host.txt`: exit101, 23 groen/twee rood. De fixture stopte lezen
bij Closed en verzamelde zo slechts 64 van de 96 antwoordbytes.
De testfixture is hersteld naar drain tot leeg/EOF inclusief Closed.
Daarna zijn de definitieve 28 plus vier compile-fail proeven op bovenstaande
bron uitgevoerd en groen. Geen productie- of native dataverlies gerepareerd
of geclaimd.

De rode proef was een ongecommit werkstadium na `9bef711`; de volledige
WIP-bronfingerprint is niet bewaard. Daarom is die log expliciet alleen
historische diagnose, geen brongebonden acceptatiebewijs. De definitieve
groene gates hebben wel hun echte bron-, dependency- en artifactbinding.

## Open interfaces en vervolgstap

Ontbrekend en **niet uitgevoerd**: native issuer, app-RPC-/SDK-dispatch,
manifest/startbinding, echte exclusieve devicelease, backendbrede fence,
quiescence/reset/epochmatching, DMA-/crash-/Dropownership en echte
proces-/bestemmingsintrekking met helpers. Geen nieuwe instantie/boolean/PID
mag die grenzen simuleren. De core heeft geen herstel-/reset-API.

Intrekking fenceert eerst lokaal ALLE nieuwe TX-publicatie en retireert ALLE
sessies op de gedeelde Stack voor abort/cleanup. Bestaande hardwarepublicatie
kan nog de wire op. Een tweede producer kan alleen door de toekomstige echte
backend-eigenaar worden uitgesloten; deze library bewijst dat niet.

Ook niet uitgevoerd: native TLS, onderhouden native cryptoprovider,
vertrouwde certificaatkalender, fysieke hardware, menselijke audit,
formeel bewijs of volledige releaseacceptatie. De bevroren TLS-leaf en
historische DHCP/client/listenbranches blijven intact.
REL bevestigde geen herstelbewijs voor de eerdere TCP-CR3-fetchfault op
`7af60df`; andere groene ELF/profielen zijn geen TCP-fix.

Concrete hervatting: intake/manager kiezen met REL/CORE de enige eigenaar van
authentieke callerstart/service-instantie en exclusieve devicelease/fence.
Daarna pas native issuer/SDK-koppeling met echte negatieve ring-3-proeven en
packetobservatie. Geen root/driver/SDK/Stack-writes zonder overdracht.

Scorevoorstel aan centrale manager: N04 0/2 -> 0/2, netwerk 0/10 -> 0/10,
totaal 34 -> 34. Hostcore/librarybuild sluiten N04/A03/B06 noch
FR-N01/FR-N04/FR-K03 af. Centrale voortgangsbestanden blijven bij de manager.
Rollback: deze nog niet gebruikte leaf/workflow verwijderen; bestaande
Stack/TLS/SDK zijn niet aangepast.
