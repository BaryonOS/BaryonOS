# Eerste N01-configcheckpoint - verwacht rode interfacegate

Basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
`services/net-stack/tests/configuration.rs` specificeert de ontbrekende
unconfigured/apply/withdraw/reapply-lifecycle zonder bestaande productiecode
te wijzigen.

Uitgevoerd:

```text
cargo +1.91.1 test --locked --offline --manifest-path services/net-stack/Cargo.toml --test configuration --target-dir target/net-config-host
exit101; E0432/E0599: LinkConfig, AddressConfiguration,
Stack::new_unconfigured en Error::NotConfigured ontbreken op de basis.
```

Dit is een **rode compile-/interfacegate**, nog geen uitgevoerde runtimecase
of vastgestelde pakketrechtenregressie. De test kan op de ongewijzigde basis
niet starten. De vervolgstap implementeert de echte lifecycle en herhaalt de
test plus bestaande transportgates. Geen rood resultaat groen genoemd.
De ruwe log en test-/productiebronhashes staan naast dit verslag.
Rust1.91.1, bestaande lokale compiler/linkercache, eigen target/net-config-host.
Geen nieuwe dependency of installatie.

De concrete semantiek: withdrawal sluit nieuwe publicatie onmiddellijk, abort
oude sockets, verwijdert adres/route en geeft geen oud RX/socketstate door.
Een bestaand pending TX-slot wordt nooit synthetisch voltooid of hergebruikt;
reapply blijft geblokkeerd totdat de echte matching completion/quiescencegrens
dit veilig maakt. Echte device-reset/epoch en native issuer blijven open.
DNS/DHCP/RFC5227/renewal-integratie volgt apart binnen dezelfde opdracht.

Geen native config/TCP/DHCP-acceptatie of punten. Netwerk0/10, totaal34 blijft
voorstel; centrale voortgang bij manager/queue-eigenaar.
