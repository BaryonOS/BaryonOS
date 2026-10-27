# N02 — echte hybride TLS-proef op loopback

`transport.mjs` voegt een uitvoerbare, begrensde netwerkproef toe naast de N01-opslagmodules. De eigen OS-kernel heeft hiermee nog geen netwerkstack. Het doel is de overgang van synthetische transportfeiten naar een werkelijk onderhandelde, geauthenticeerde TLS-verbinding te testen met het bestaande rechtenmodel.

## Uitvoeren

Vanaf de projectroot, met de geteste Node 24.22.1/OpenSSL 3.5.9-runtime en de `openssl`-CLI voor tijdelijke klassieke testcertificaten:

```sh
node services/nas/transport-demo.mjs
node --test --test-isolation=none services/nas/tests/transport.test.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs
```

De demo en transporttests openen alleen `127.0.0.1` op automatisch toegewezen tijdelijke poorten. Een sandbox kan daarvoor een expliciete loopback-uitvoermachtiging vereisen. Een EPERM-run is geen uitgevoerde transporttest. De oorspronkelijke N01-demo blijft zonder sockets werken.

Testidentiteiten worden per run in een eigen tempmap met directorymodus 0700 gegenereerd. De fixture verwijdert deze eigen map na inlezen; de CA-private key wordt niet naar de adapter doorgegeven. `close()` overschrijft de beheerde Buffer-kopieën. Dit is geen secure-erasegarantie of bewijs dat alle kopieën in OpenSSL/geheugen verdwijnen. Geen certificaten, private keys, sessietickets of documentinhoud in Git/logs.

## Vast cryptografisch proefprofiel

| Eigenschap | Afgedwongen waarde |
|---|---|
| Luister-/verbindingsadres | uitsluitend `127.0.0.1`; geen extern hostargument |
| Protocol | TLS 1.3 als minimum én maximum |
| Sleuteluitwisseling | uitsluitend `X25519MLKEM768`, RFC 10024 |
| TLS-recordcipher | `TLS_AES_256_GCM_SHA384` |
| Applicatieprotocol | ALPN `nieuw-os-nas-host/2` |
| Authenticatie | wederzijdse CA-validatie, exacte SHA-256-certificaatpin; serverhostnamecontrole |
| Certificaatsleutels | klassieke Ed25519; geen PQ-authenticatieclaim |
| Hervatting | geen sessiecache/resumption; volledige handshake, geen 0-RTT |
| Applicatie-inhoud | begrensde N01-ciphertext-envelope; alleen client ontsleutelt |

De client controleert na de handshake de werkelijk gerapporteerde groep, TLS-versie, cipher, ALPN, peer en `isSessionReused`. Als de runtime geen overeenkomende groep kan rapporteren, faalt de verbinding. De server-API rapporteert deze groep niet via `getEphemeralKeyInfo`; serverbewijs maakt daarom onderscheid tussen geconfigureerde groep en de door de client waargenomen onderhandeling.

Geen zelfgemaakte hybride KDF, extra PSK-combiner of eigen TLS-recordencryptie. Node/OpenSSL voert het gestandaardiseerde protocol uit. Een omhulsel rond TLS regelt uitsluitend projectbeleid, framing, grenzen en foutafhandeling. De actuele standaard en de motivatie staan in [de beveiligingsblauwdruk](../../docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md).

## API en bevoegdheden

`createLoopbackNasServer({credentials, allowedClientPin, namespace})` retourneert een poortnummer, minimale `evidence()` en `close()`. Het endpoint controleert mTLS, de clientpin en de envelope-namespace. Het echoot ciphertext vanuit RAM. Dit is geen NAS-opslagservice, geen duurzaam ACK en geen nieuwe beheerinterface.

`connectLoopbackNas({port, credentials, expectedPeerPin, serverName, engine, app, networkScope, localNetworkId})` retourneert alleen `sendCiphertext(bytes)`, `evidence()` en `close()`. De socket en opaque transportfact blijven intern. De vertrouwde hostadapter houdt `engine.authority`; een onbetrouwbare app mag die niet ontvangen.

De adapter controleert vóór socketaanmaak `authorizeNasConnection`. Deze preflight bevestigt alleen toestemming om te proberen, geen peeridentiteit. Pas na echte TLS-controle wordt een transportfact gemaakt, gebonden aan de app, de huidige peer-koppeling en bij internet de intrekkingsgeneratie van dat recht. Vóór iedere ciphertextverzending en vóór aflevering van het antwoord wordt `engine.check` opnieuw uitgevoerd. Een fout of gesloten verbinding trekt de fact in. Na intrekken en opnieuw toestaan blijft een eerder geauthenticeerde fact ongeldig; er is een nieuwe verbinding nodig. Internetintrekking laat de toegestane LAN-route bestaan. Reeds verstuurde bytes zijn niet terug te halen. Een idle socket wordt niet door een beleidsnotificatie actief gesloten en leeft maximaal de vaste verbindingstermijn. Dit is geen native proces- of socketcapabilitygrens.

`networkScope` en `localNetworkId` zijn expliciete harnasaannames. Ook een test met het label `internet` loopt fysiek via loopback. De tests bewijzen het peergebonden toestemmingspad, geen WAN-bereikbaarheid, interface-identificatie, firewall of NAT-traversal. Vertrouwde netwerkwaarneming blijft een native integratietaak.

Pins zijn hier hashes van het volledige certificaat. Certificaatvernieuwing verandert dus de pin; deze fixturekeuze is geen afgerond consumentenontwerp voor identiteit/rotatie. Die productflow moet veilig én automatisch binnen vooraf verleende bevoegdheden kunnen verlopen.

## Grenzen en herstel

Maximaal vier gelijktijdige TCP-verbindingen, ook vóór TLS-authenticatie; acht frames per verbinding; maximaal 140.000 bytes per frame. Een frame heeft een vierbytes-lengteheader. De parser reserveert pas na lengtecontrole één begrensde payloadbuffer. De frame-deadline begint bij de eerste byte en wordt niet door langzaam druppelende invoer verlengd.

De proefgrenzen zijn 1.500 ms voor de handshake, 750 ms voor een frame/antwoord en 5.000 ms voor de volledige verbinding. Dit zijn deterministische loopbackproefwaarden, geen internetprestatie-eisen of consumentenlimieten. Het endpoint stopt bij foute framing, ongeldige namespace, quotumoverschrijding of deadline. Geen retry met een zwakker profiel, andere peer of leveranciersrelay.

## Bewijs en open werk

De tests gebruiken echte TLS-clients, waaronder clients die de normale adapter omzeilen om de servergrens te controleren. Zij controleren onder meer echte downgradefouten, verkeerde identiteiten/certificaten, ontbrekende toestemming vóór socketaanmaak, intrekking, oude facts, framing, quota en trage invoer. Een aangeboden TLS 1.3-sessieticket moet opnieuw tot een volledige hybride handshake leiden. Testaantallen, bronhashes, commando's en review staan in [verification/pqc-transport](../../docs/verification/pqc-transport/).

Nog open: native netwerk-/cryptoport, productie-identiteits-/certificaatbeheer, postkwantumauthenticatie, echte multi-devicepairing, bruikbare blijvende clientkeys, achtergrondsync, store-transportintegratie, OS-isolatie en onafhankelijke menselijke audit. Het bestaande admin-revoke/evict-gedrag wordt door N01 getest; de ciphertext-echo is daaraan niet als server gekoppeld. M1–M6a, qsim, de bootketen en fysieke apparatuur zijn door deze proef niet gewijzigd.
