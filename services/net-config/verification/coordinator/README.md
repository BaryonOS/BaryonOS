# DHCP-/probe-/DNS-coordinator: brongebonden gates en regressies

Bron `f16062b920a84d76599c4a089f8b79a6b4cfb925`,
branch `simonvanieperen-native-net-config`.
v3 `331af359595218719246889265e129e5554876ae0a5648855aa37922c6a4d04a`.
Dit zijn de pre-reviewgates; geen actuele eindacceptatie. De aparte codecheck
vond daarna een echte late-renewalfout. Zie hieronder en de herstelgates.
Geen native proces of productacceptatie.

Tien exacte gates, alle exit0; volledige argv/cwd/tijden/status van bestaande
`tools/common.py::run_logged` staan in `frozen-*.json`. De stdout/stderr zijn
ongewijzigd. `gates.json` bindt bronbestanden en rlib; SHA256SUMS de artifacts.

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
```

Resultaat:26configtests (dertien coordinator, elf client, twee clock),
24Stacktests en58bestaande codecproeven. Baremetal-library, geen native ELF.
De coordinator gebruikt werkelijk gecodeerde DHCP-/ARPframes en dezelfde
bestaande Stack/PacketDevice, met expliciet gemodelleerde backendcompletions.
Een hostfunctieaanroep is GEEN bewijs van een echte NICcompletion.

## Twee gereproduceerde runtimefouten, niet weggepoetst

`peer-probe-red.patch` is de volledige exacte WIPdelta tegenover
`4c335d10f8c1a46dd68e5eda46fda147dc2d26f6`. De gerichte peerprobeproef
`peer-probe-red.*` eindigde exit101: geen antwoord op zero-source ARPprobe
na configuratie. De gepinde upstreamsmoltcpneighborafhandeling weigert die
bron; RFC5227sectie2.5 vereist het antwoord wel. Nu alleen voor gevalideerde
vreemde probe voor het werkelijk eigen toegepaste IP een reply via hetzelfde
TXslot, zonder0 alsneighbor te cachen. Geen upstreamfork/nieuwe parser.

`selecting-secs-red.patch` is een test-onlydelta tegenover geteste bron
`678081c9bae9436c0b393635e2f8096e6187eeaf`. `selecting-secs-red.*`
eindigde exit101:REQUESTsecs5 versusDISCOVERsecs1. De eigen verplaatste
DHCPclient bewaart nu de laatst gecodeerde DISCOVERsecs voor selectingREQUEST
en laat de conservatieve eersteREQUESTtijd/expiry ongemoeid.
Die correctie is afzonderlijk boven de bytegelijke donorimport vastgelegd;
claim daarom niet dat de huidige client nog geheel ongewijzigd16ad is.

De groene13coordinatorcases omvatten de volledige probe-/announcementvolgorde
en bytes; geen premature Bound; backpressure/stale completion/equal ticks;
vreemde probe/claim, eigen echo, proberesponse naar targetIP0 en late conflict;
ARPpeerprobeantwoord tijdensBound; T1/unicast,T2/broadcast,NAK/expiry;
gewijzigde gateway/DNS wachten op oudecompletion; linkdown/reconnect en
Unknownterminal; tienconflictenrate-limit; klokrollback/overflow en
onrepresenteerbare leasedeadlines; selectingsecs EN werkelijke expiry.

## Bronnen en grenzen

Primair daadwerkelijk gelezen:
- https://www.rfc-editor.org/rfc/rfc5227.txt secties2.1.1,2.3,2.4,2.5.
- https://www.rfc-editor.org/rfc/rfc2131.txt sectie3.1stappen3en5.

ProbeTiming valideert0..100/100..200ticks; een echte native caller moet
uniform geautoriseerde entropy leveren. Geen verborgen RNG/fallback.
Conflictbehandeling kiest onmiddellijk intrekken, niet permanente verdediging.
RFC2131DECLINE ontbreekt nog: bestaande codec heeft geen encoder; een
begrensde extra writeclaim/handoff is gevraagd, NIET overgenomen.
De aanbevolen10swachttijd wordt al afgedwongen, na10conflicten60s.
Retry/XIDplanning en autoreconnect moeten nog aan een echte serviceloop.

Niet uitgevoerd: native caller/app/device/DMAfence/reset/epoch/CR3gate,
brokergrant-/egressintegratie, native RNG/kalendertijd/TLS, resolver-/apprechten,
nieuwe dedicatedCI, fysieke hardware, menselijke audit of formeel bewijs.
Queued/onzekere frames blijven behouden; reeds gepubliceerde hardwareframes
zijn niet terugroepbaar. Clockfault/Unknown zijn terminaal zonder herstelAPI.
Route/prefixwijziging kan TCPverbreken; ongewijzigde leasevernieuwing niet.
Geen native Internetclaim. Netwerk0/10/totaal34 blijft het scorevoorstel;
centrale status/progress zijn eigendom van manager, niet door NET gewijzigd.
De afgebakende aparte AI-codecheck is uitgevoerd op f16062b, read-only,
geen tests of writes door die agent en geen menselijke audit.
Zij vond een vertraagde ACK met reeds verstreken nieuwe T1: de client gaf
RenewalDue terug, maar de coordinator verwerkte alleen Bound. Daardoor
bleven oude gateway/DNS zichtbaar. `late-renewal-red.patch` is de exacte
test-onlydelta tegenoverf16062b, `late-renewal-red.*` eindigt exit101.
De productiecorrectie verwerkt beide geaccepteerde renewaluitkomsten.
De onderstaande herstelgates worden apart aan de gecorrigeerde bron gebonden;
de eerdere tien groene gates sluiten deze nieuw ontdekte regressie NIET uit.

## Herstelde bron

`03277a74412e8cb70f3fed86a162bfe34192484b`,
v3 `d0282b2b4974186d3fb199aeeab3892385290905ca8b3f86df163302ed2fd0b9`.
Dezelfde tien exacte commando's opnieuw uitgevoerd: alle exit0.
`review-fixed-*.json`/stdout/stderr zijn die ongewijzigde commandorecords;
`review-fixed-gates.json` bevat bron, rlib en de review-/reproductiebinding.
27configtests (veertien coordinator+elf client+tweeclock),24Stack,58codec.
De nieuwe case vereist nieuwe route/DNS, T1=41 en expiry=120, geen verlenging.
Geen nieuwe review door dezelfde AI aangevraagd om eigen herstel groen
te verklaren; de hoofdagent heeft de concrete regressie gereproduceerd en
de exacte bron opnieuw getest. Geen menselijke/nativeacceptatie.

Vervolg nu expliciet vrijgegeven: TASK-N01-DHCP-DECLINE-SEAM,
queuecommit `dea97f9b57c6fda9549a48072119663fe155a563`.
Alleen `services/net-protocol/src/dhcp.rs` en `tests/dhcp.rs` extra, eigen
bewijs blijft hier. Deze herstelbron bevat NOG GEEN DECLINEimplementatie.
