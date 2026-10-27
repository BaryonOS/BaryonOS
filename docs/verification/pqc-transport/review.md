# N02 — afzonderlijke AI-reviews en herstel

27 oktober 2026. Twee reviewers hebben de nieuwe transport-/policycode en de bewijsclaims los van de builder beoordeeld. Dit is geen onafhankelijke menselijke audit of formeel bewijs.

## Oorspronkelijke review

De reviewers controleerden TLS-profiel, CA-/hostname-/pincontroles, wederzijdse clientauthenticatie, huidige toestemming vóór socketaanmaak en per operatie, ontbreken van downgrade/resumption, framing, grenzen, deadlines, cleanup en scheiding tussen clientinhoudssleutel en transport. Eén reviewer reproduceerde 41/41 toenmalige policy-/transporttests met echte loopback-TLS. Die 41 zijn een oudere deelverzameling van de definitieve suite en worden niet extra opgeteld.

Drie zelfstandige negatieve probes zijn vervolgens als permanente tests opgenomen: intrekken tussen preflight en voltooide handshake, geauthenticeerde server met te grote response, en een server met een syntactisch correct maar ander ciphertextantwoord. Bij de eerste onafhankelijke probe was de verwachte foutcode te algemeen; de werkelijke `UNPINNED_PEER` was correct. Alleen de testverwachting is hersteld. De geïntegreerde tests bevatten deze gevallen.

## Bevinding en oplossing

**Bevinding:** de eerste transportfacts representeerden peer-/routevalidatie zonder app-/toestemmingsgeneratie. Intrekken blokkeerde een volgend gebruik, maar expliciet opnieuw pinnen van dezelfde peer kon een oude idlefact weer geldig maken wanneer er tussendoor geen operatie plaatsvond. Dit was geen ongeautoriseerde toegang onder de toen actuele nieuwe grant, maar voldeed niet aan de strengere wens dat oude geauthenticeerde sessies ingetrokken blijven.

**Oplossing:** `attestVerifiedTransport` vereist een apphandle en legt de generatie van de peerpin vast, plus de internetintrekkingsgeneratie bij internet. Generaties zijn interne objectidentiteiten, geen overlopende numerieke teller. `check` weigert een andere app of verouderde generatie. Opnieuw koppelen of toestaan kan het oude fact niet herstellen. Intrekken van internet verbreekt niet het LAN-recht; wijzigen van een andere peer maakt deze verbinding niet ongeldig.

**Onafhankelijke hercontrole:** drie policyregressies en twee echte TLS-regressies, 5/5 exit 0. De oorspronkelijke losse regrant-probe geeft nu `allowed:false`, `reason:STALE_TRANSPORT_AUTHORITY`. De definitieve rootrun bevat al deze tests: 124/124, geen skips. Niet optellen bij deelruns.

Op de delta gereviewde SHA-256:

```text
ff8a7422e8958e1d57fd98ab515d27832d623bb266d4241dd73b5adbf324fdb0  services/nas/policy.mjs
26742c44b6239f64edafed8cfd1796e0797059642a820e7190abd20fe83990b4  services/nas/transport.mjs
```

De volledige definitieve sourcehashes staan in `report.json`.

## Niet als opgelost claimen

- App-/authorityisolatie binnen een gedeeld Node-proces is geen native capabilitygrens. De vertrouwde adapter houdt `engine.authority`.
- Pins identificeren volledige certificaten; bruikbare automatische vernieuwing/rotatie is niet geïmplementeerd.
- De server controleert envelopeformaat en namespace, niet de inhoudelijke GCM-tag: alleen de client heeft daarvoor de sleutel.
- De TLS-echo is geen opslag-ACK en heeft geen N01-store-/adminrevocationadapter.
- Een reeds geattesteerde fact blijft na intrekking ongeldig; de generatie wordt pas bij attestation vastgelegd. Geen permanente blokkering van iedere handshake die ooit vóór intrekking begon geclaimd.
- Geen pushsluiting van idle sockets bij intrekking; de volgende operatie/response weigert. Er is daarnaast een absolute vijfsecondenlevensduur voor de proef.
- Klassieke Ed25519-authenticatie, harnasaannames over LAN/internet, host-CSPRNG en ontwikkelhost-TLS blijven expliciet buiten een native/volledige PQ-garantie.
- Verwijderde tijdelijke keybestanden en overschreven eigen buffers bewijzen geen fysieke secure erase van alle kopieën.

Binnen deze afgebakende scope bleef na herstel geen concrete blokker open.
