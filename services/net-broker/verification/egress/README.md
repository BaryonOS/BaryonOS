# Brongebonden herstel van afgeleide egress

Dit vervolg vervangt geen eerdere bronbindings. Code:
`f9b59884d7acb6b56537838969a04cd5b5e52b77`, na historisch `0962d02`.
v3 `06e06c0a62d0a960cc4f9b23f9aa80d375c869fe3e0f20d363cd22a84935c667`.
Baremetal-library SHA256:
`1688597af5d5ada17892c7c4eccf602f54f8d9c45854c93a4c14d0c51722c9e8`.

Een aparte AI-codereviewer vond een concrete ontbrekende controle:
ongevraagde ingress kan de bestaande Stack een TCP-RST of ARP-antwoord naar
een endpoint buiten de actieve grants laten genereren. Oorspronkelijk gaf
`offer` die frames zonder extra bestemmingscontrole door. Dit is geen
menselijke audit of native exploitclaim.

## Rode regressie en reparatie

Ongewijzigde productiebron `0962d025fb3638d41b600dfe44ecab00ef8dde72`,
met uitsluitend extra regressietest:
`egress-regression-red.patch`, volledige exacte testfile
`egress-regression-test-before-fix.rs.txt`, SHA256
`42e14bc042602d441d9669312126423f2b1cbcc96b06120b71d8472b6c20e354`.
De patch behoudt originele contextbytes via lokale attributes.

```text
cargo +1.91.1 test --locked --offline --manifest-path services/net-broker/Cargo.toml --target-dir build/net-broker-worktree/target/net-broker-host unsolicited_arp_cannot_publish_reply_without_destination_authority
exit101: regressie terecht rood, bestaande offer gaf het antwoord vrij
```

Daarna controleert `offer` alle geproduceerde framecoordinaten met bestaande
smoltcp checked wire-views: TCP moet de actuele grantpeer/poort en lokale
socket matchen, ARP de eigen IPv4 en een actieve peer. Geen wildcard,
extra transportstack of ongecontroleerde gatewayuitzondering.
Weigering zet de lokale fence voordat cleanup wordt geprobeerd, biedt niets
aan en behoudt de gequeue-de framebytes/id. Geen nepcompletion of discard.
Gedeelde-backendquarantaine is daarbij een expliciete beschikbaarheidsgrens.

Drie nieuwe tests: ARP-reply zonder autoriteit, ongevraagde TCP met ongeoorloofde
reply/neighborlookup, en exactheid van TCP-egressbron/dest/poort/lokaal slot.
De normale conversatieproeven blijven groen, inclusief retransmissie en ARP.

## Groene gates op de reparatiebron

31 unit + vier productiecompile-fail brokerproeven, bestaande Stack15,
beide Clippy/all-targets/-Dwarnings, beide fmt-checks en baremetal-release-
librarybuild: zeven commandogates, alle exit0. Rust1.91.1. Geen native ELF.
De bestaande `tools/common.py::run_logged` is gebruikt, met een eigen
NOS_BUILD_DIR en behouden host-/baremetal-targetcaches.

`gates.json` bevat de exacte argv's, bronblobs en artifactbinding.
`net-*.json` zijn de ongewijzigde repository-commandologgers met cwd, start,
duur en returncode; bijbehorende stdout/stderr zijn afzonderlijk opgeslagen.
De registryversies/checksums zijn exact vergeleken met de bestaande Stack.
smoltcp is nu ook een directe productiedependency voor de policy-wireviews;
geen extra geselecteerde feature of registryversie.

De eerdere CI op `0962d02` is historisch groen (inclusief broker-run
41104644311); dit bewijst NIET deze reparatiebron. Geen nieuwe CI-groenclaim
zonder exacte nieuwe headbinding.

Alle eerdere native grenzen blijven open: productieissuer/SDK/devicebrede
fence/reset/epoch/crash-Dropownership/TLS en TCP-CR3-bootgate. Deze fix verleent
geen native rechten of punten. Netwerk0/10, totaal34 blijft het scorevoorstel.
Na dit bewijscheckpoint is broker #87 bevroren voor integratie; volgende
primaire NET-taak is apart toegewezen N01-config op codecmainca45.
