# AUDIT-15 - Netwerk en verdeelde systemen

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder zijn repository-observaties uit die
snapshot. Externe TLS-, protocol- en securityclaims zijn niet onafhankelijk
geverifieerd en blijven `ONBEKEND`; de OSDev-verwijzing is alleen een
aanvullende leerindex.

Datum: 3 november 2026, Europe/Amsterdam. **Audit-only; geen productacceptatie.**

## Resultaat en bronafbakening

Nieuw OS heeft native netwerkbron en historisch positief en negatief VM-bewijs voor ARP/UDP/DNS/TCP, procesgescheiden netwerkgebruik en lokale NAS-opaqueopslag met koude herstart en tokenreplayweigering. Dit is meer dan een hostprototype. De volledige native productketen voor TLS, cryptografische peeridentiteit, E2EE, duurzame achtergrondsynchronisatie en optionele provideropslag is daarmee niet aangetoond. De bindende FR-N01-N04 en FR-P01-P06 blijven afzonderlijke acceptatie-eisen: `docs/FULL-TEST-RELEASE.md:206-215`.

- Opdracht: "Beoordeel native networking, TLS, DNS, firewall, appnetwerkrechten, peeridentiteit, NAS, synchronisatie en foutscenario's."
- Enige onderzochte bron: `simonvanieperen/nieuw-os`, `beafdedfc188154fe7285848fbd411e651013f54`; lokale ref `refs/heads/copilot/add-os-specialist-agents` wees exact naar deze commit.
- Eigen werkmap: eigen worktree; lokaal pad niet gepubliceerd. Geen bestanden
  in de hoofdcheckout gelezen of geschreven. Alleen worktree-metadata kwam
  voor in Git-worktreemetadata.
- Eerlijke beginafwijking: branch `simonvanieperen-vigilant-parakeet`, HEAD `69809c2cb53dfb3bd77abe5352892810fba6e82d`, schone beginboom. Daarna expliciet `git switch --detach beafdedfc188154fe7285848fbd411e651013f54`; audit uitgevoerd met detached HEAD, niet op de genoemde bronbranch zelf.
- Normen gelezen: root `AGENTS.md`, `docs/AGENTS.md`, `kernel/AGENTS.md`, `drivers/AGENTS.md`, `services/AGENTS.md`, `tests/AGENTS.md`; README, actueel checkpoint, release-/privacycontract, relevante architectuur- en beveiligingsregels/besluiten. `git ls-files '*AGENTS.md'` leverde geen dieper instructiebestand onder het rapportpad of de onderzochte netwerkmodules.
- Profiel `.github/agents/os-network-distributed-reviewer.agent.md` expliciet gelezen als gewoon bestand. **Geen technisch geladen customagent**, geen subagents. OS-dev- en reviewskill geraadpleegd; CodeRabbit niet uitgevoerd wegens repository-only/externe-dienstenverbod.
- Alleen dit rapport geschreven. Geen productcode, roadmap, score, projectstatus, commits, pushes, PR's of andere rapporten gewijzigd. Geen live GitHub-/PR-/remotecontrole: uitsluitend lokale Git-objecten/refs.

## Bewijslegenda en methode

E0 = ontbrekend/ambitie; E1 = ontwerp; E2 = hostprototype/simulatie/hosttest; E3 = native bron; E4 = native positief; E5 = native negatief; E6 = herhaalde CI/releasegate; E7 = fysieke hardware; E8 = onafhankelijke menselijke audit/formele evaluatie. De niveaus zijn scopegebonden, geen optelbare productscore. Een hosttest van een `no_std`-crate is E2; native bron daarvan E3; crosscompilatie is geen E4.

**Deze audit:** statisch, begrensd en repository-only. Geen nieuwe host-, VM-, hardware-, gebruiks-, TLS- of CI-test uitgevoerd. E4/E5 hieronder verwijst uitsluitend naar opgeslagen historische artefacts met hun eigen bronbinding. E6 wordt alleen als historische gatevermelding aangeduid; geen nieuwe externe verificatie. AI-review is nooit E8. Hashcontrole toont bytegelijkheid met een genoemde hash, niet de waarheid van een testclaim of een complete heruitvoering.

### Vastgestelde implementatie en afzonderlijke bewijslagen

| Onderdeel | Huidige bron / ontwerp | Historisch bewijs en grens |
|---|---|---|
| Virtio-net / eigen kernel | E3. Alleen dienst krijgt raw NIC; apps krijgen IPC. Manifest controleert dat apphandles geen NIC/entropy/launch/blockrecht geven: `kernel/src/m2/m3/network_service.rs:59-124`. Reset/foutpad houdt DMA-geheugen gepind en quarantaineert zonder reset-ACK: `kernel/src/m2/m3/network.rs:313-355`. | E4/E5 op vaste QEMU-configuratie, geen E7/IOMMU-garantie. Privileged driver/broker blijft TCB: `docs/implementation/NATIVE-NET-INTEGRATION.md:24-43`. |
| ARP, UDP, DNS | E3 native probe gebruikt echte protocolbytes, gecontroleerde endpoints en responses: `sdk/examples/net-probe/src/main.rs:220-298`. | E4/E5 normal/cold op `985ed4da0ead6865c8d1fd0d4deb34ef7f97f3eb`; source-v3 `32a4168e9ffe0ccfebb0cae692856ad27f12687a78beb63c63c3e8560f22a3ff`. Beide resultaatbestanden noemen bron/hash: `docs/verification/full-release-net-985ed4d/0-normal/result.json:1-6`, `docs/verification/full-release-net-985ed4d/1-recover/result.json:1-6`. Positieve en negatieve counters: `docs/verification/full-release-net-985ed4d/0-normal/result.json:446-453`. Geen volledige resolveracceptatie. |
| TCP/UDP via netwerkdienst | E3. Ring-3-dienst met twee afzenders, vaste bestemmingen, tokens, replayweigering en begrensde buffers: `services/network-service/src/lib.rs:14-35`, `services/network-service/src/lib.rs:328-389`. Werkelijke kernelafzender wordt doorgegeven: `sdk/examples/network-service/src/service.rs:248-267`. | E4/E5 op `f152aec0615285065689f0a7e2855e5858fdb350`, source-v3 `56ed10aade50703e08c8422251e323b4400bf37196320d24befe0b46883bbce8`: 10 TCP-openingen, 4 HTTP-antwoorden, resets/retransmissies/timeouts, UDP-malformed/backpressure, raw-NIC-deny, cleanup. Afzonderlijke client-exitrun behoudt B: `docs/verification/n01-native-service-20261102/README.md:3-58`, `docs/verification/n01-native-service-20261102/README.md:61-85`. Synthetische Ethernetpeer, geen internet/TLS. |
| Latere TCP-reparaties | E3: connectdeadline geldt alleen voor handshake; terminale ontvangst herkent ook `TimedOut`: `services/network-service/src/lib.rs:181-208`, `services/network-service/src/lib.rs:461-480`. | Historisch rood/groen hostbewijs voor connectdeadline/SendTcp: `docs/verification/pr127-review-integration.json:61-78`. Latere ReceiveTcp-fix heeft hosttests en opnieuw beide native gates op `5925c915f6b8d02842fffd89a94f21c0e018689a`, v3 `9a8b521f4b2e5a24b279142d8554538c5cf618e3e55a9cac3e7070e12ad4fc39`: `docs/verification/issue-43-receive-timeout.md:24-86`. Het specifieke timeoutassert is hostbewijs, niet een nieuwe native scenarioassertie. Niet opnieuw als open bug gerapporteerd. |
| DHCP/adres/routes | E3 servicecore, gedeelde stack, lease-/linklifecycle, retry met expliciete entropybron: `services/net-config/src/automatic.rs:1-75`. | E2 hostmodellen en codec-/coordinatortests; geen aangetoonde native DHCP-consument/RNG-/resetfence: `services/net-config/README.md:254-271`, `services/net-config/README.md:273-320`. Niet reduceren tot alleen ontwerp; ook niet native accepted noemen. |
| TLS/PQC/tijd | E3 randvoorwaardelijke Rust-adapters; tijdpoort weigert elke huidige RTC-bron: `sdk/runtime/src/tls_time.rs:46-66`, `services/net-tls/src/time.rs:9-47`. Random-adapter begrenst deadline/polls, zeroizet uitvoer, annuleert bij fout: `services/net-tls/src/random.rs:75-147`. | E2 echte host-mTLS/TLS1.3/X25519MLKEM768, vaste ALPN/cipher, klassieke Ed25519-certificaten: `services/nas/TRANSPORT.md:19-35`. Historische hosttests: `docs/verification/pqc-transport/README.md:3-28`. Geen native handshake/provider/signer/trust-storeketen: `services/net-tls/README.md:72-103`. |
| Lokale native NAS-autoriteit/opslag | E3 core + volumeadapter + IPC-probe. Commit controleert revisie opnieuw en bevestigt pas na persist; iedere vervangingsfout wordt Unknown/poisoned: `services/nas/native-core/src/lib.rs:369-383`, `services/nas/native-core/src/lib.rs:617-641`. Native adapter gebruikt bestaande volume-write/purge: `services/nas/native-volume/src/lib.rs:98-138`. | E4/E5 lokale opaqueopslag, revoke/evict, cold en exacte boot-1-session/uploadreplay. Historische runbasis `e08fb0e48c1bb0781e98357f588c2f73b6ae8500` bevatte WIP: doorslaggevend v3 `7255534f462eceba32f6d5059f2fc3b2219a8ea5e4ca18b1bca7626940ea648b`, niet commitnaam alleen. Normal/cold raw QEMU-exit 33; cold geen writes/flushes: `docs/verification/nas-replay-110/README.md:1-31`. Geen netwerk-NAS of E2EE. |
| NAS-syncclient | E3 `no_std` library, E2 hostmodel. Checkpoint bevat edits maar geen live tickets; restore maakt onafgeronde edits NeedsReconcile: `services/nas/native-client/src/lib.rs:129-146`. | Hostscenario's voor verloren ACK, conflict, quota, revoke en herstel aanwezig: `services/nas/native-client/tests/resume.rs:135-174`, `services/nas/native-client/tests/resume.rs:256-307`, `services/nas/native-client/tests/resume.rs:420-446`. Geen native duurzame syncdienst: `services/nas/native-client/README.md:49-67`. |
| Gateherhaling | Native netwerkworkflow definieert normal/client-exit: `.github/workflows/net-broker-quality.yml:85-97`; NAS normal/cold-job: `.github/workflows/nas-core.yml:74-82`, `.github/workflows/nas-core.yml:110-124`. | Historisch E6 vermeld voor NAS op fa97c692/562be4d: `docs/verification/nas-replay-110/README.md:33-47`; netwerkacceptatiemetadata bewaart hosted normal/client-exit en exacte geteste boom: `docs/verification/pr127-main-acceptance.json:5-13`, `docs/verification/pr127-main-acceptance.json:47-54`. Workflowbron op zichzelf bewijst geen uitvoering op beafdedf. Geen E7/E8 gevonden binnen deze audit. |

## Geprioriteerde bevindingen

Prioriteit duidt product-/integratierisico aan, **niet** een bewezen exploiteerbare kwetsbaarheid. P0 blokkeert de beveiligde NAS-/syncketen; P1 blokkeert brede dagelijkse netwerkfunctionaliteit; P2 betreft uitbreiding buiten het vaste VM-profiel.

### AUDIT-15-01 - P0: native TLS heeft nog geen werkende geauthenticeerde productverbinding

**Bron:** `services/net-tls/src/lib.rs:1-9`, `services/net-tls/README.md:72-103`, `sdk/runtime/src/tls_time.rs:58-66`. Het pakket bevat adapters, geen actieve TLS-dienst; iedere bestaande klokkwaliteit wordt terecht geweigerd. Geen aangetoonde native cryptoprovider, signer, volledige RNG-aansluiting, geautoriseerde kalenderbron, root-/certificaatbeheer of handshake naar NAS-identiteit. De adapters zijn dus niet defect omdat ze fail-closed weigeren.

**Impact:** TCP/HTTP-bereikbaarheid is geen beveiligd transport; veilige NAS-onboarding en FR-N03 kunnen niet op de host-TLS-proef worden afgetekend. **Afhankelijkheden:** Key Service/entropy en signerautoriteit, vertrouwde tijd met versheid/rollbackbeleid, begrensde allocator/OOM, bestaande rustls-port en netwerktransport. **Benodigd bewijs:** native handshake met exacte peer/hostname/ALPN/hybride groep; verkeerde/verlopen certificaten, klokfout, malformed handshake, RNG-fout en geweigerde fallback. Zekerheid: hoog voor dit integratiegat; geen runtime-exploitclaim.

### AUDIT-15-02 - P0: lokale procesidentiteit is geen NAS-peeridentiteit of multidevicecontinuiteit

**Bron:** `services/nas/native-probe/src/lib.rs:260-276` maakt `NOSPROC1 + PID`; dispatch gebruikt de werkelijk gecontroleerde afzender (`services/nas/native-probe/src/lib.rs:282-299`). Dat is goede lokale IPC-binding, geen cryptografische apparaatidentiteit. De core verlangt reeds vertrouwde ingress: `services/nas/native-core/README.md:9-27`. De hostfixture pint een volledig certificaat; vernieuwing verandert de pin en is nog geen productrotatieflow: `services/nas/TRANSPORT.md:37-47`.

**Impact:** uitnodigingen, LAN/internettoestemming en intrekking kunnen niet veilig aan een remote gebruikersapparaat worden verbonden door een PID, hostnaam, discoveryresultaat of caller-supplied fingerprint. **Afhankelijkheden:** 01, duurzame apparaat-/accountbinding en herroepingsgeneraties. **Benodigd bewijs:** QR/vertrouwd-kanaalbinding, verkeerde/verwisselde peer, verlopen/hergebruikte invite, sleutelrotatie, ingetrokken apparaat en oude TLS-sessie na reconnect. Native cold accept weigert bestaande invites conservatief zonder cross-boot tijdbron: `services/nas/native-probe/src/lib.rs:307-312`; dit is geen bruikbare complete onboarding. Zekerheid: hoog.

### AUDIT-15-03 - P0: syncqueue is niet duurzaam en nog niet aan een remote, versleutelde keten gekoppeld

**Bron:** `services/nas/native-client/src/lib.rs:129-146` maakt een RAMcheckpoint; directe calls naar `Service<S>` in commit/reconcile staan op `services/nas/native-client/src/lib.rs:255-280` en `services/nas/native-client/src/lib.rs:335-390`. De README noemt ontbreken van persistente queue, remote latest-discovery, peertransport en key-lifecycle expliciet: `services/nas/native-client/README.md:27-47`, `services/nas/native-client/README.md:62-78`.

**Wat wel klopt:** Unknown herhaalt geen commit automatisch; een waargenomen eigen ciphertext wordt `Observed`, niet een verzonnen oorspronkelijke ACK; conflicten behouden beide kanten. **Impact:** een hostcheckpoint is geen garantie dat offline edits na echte clientcrash/koude herstart blijven bestaan; geen automatische achtergrondresume over een netwerkpartition bewezen. **Afhankelijkheden:** 01/02/04/07, duurzame lokale edits/versieankers en object-/sleuteldienst. **Benodigd bewijs:** meerdere native clients, diskpersistente queue, disconnect rond iedere upload-/commitstap, ACK-verlies, gelijktijdige edits en herstart zonder dubbele publicatie of verlies. Zekerheid: hoog.

### AUDIT-15-04 - P1: fixed-destination appisolatie bestaat; algemene firewall/policy-issuer en dynamische revoke ontbreken als productketen

**Bron:** native `Configuration` heeft twee PIDs en statische bestemmingen (`services/network-service/src/lib.rs:14-35`); requestcontrole bindt sender, epoch, requestvolgorde, bestemming en connectiontoken (`services/network-service/src/lib.rs:328-389`). Bootstrap gebruikt vast `192.0.2.0/24` en portbereiken: `sdk/examples/network-service/src/service.rs:153-175`. Het afzonderlijke net-brokerpakket verklaart dat een native issuer/caller/devicefence ontbreekt: `services/net-broker/src/lib.rs:3-7`, `services/net-broker/src/lib.rs:36-53`.

**Impact:** niet beweren dat er geen appnetwerkafdwinging is: die is native aanwezig voor deze twee processen. Wel ontbreken afgetekende dynamische toestemming/intrekking, naam-naar-bestemmingsbinding, luisterpoortbeleid en alle helperroutes voor willekeurige derdenapps. Stop/retire is geen compleet consumentenbeleid met veilige regrant. **Afhankelijkheden:** vertrouwde policy-ingress, unieke procesincarnatie en epochs, deviceownership en lifecycle; koppeling met DNS/02. **Benodigd bewijs:** actieve en nieuwe verbinding na revoke, helper/delegatie, serviceherstart/PID-hergebruik, onverwachte listener en DNS-/redirectwijziging. FR-N04 blijft open zonder de afgebakende N01-acceptatie te ontkennen. Zekerheid: hoog.

### AUDIT-15-05 - P1: DHCP, routeherstel en DNS vormen nog geen algemene native resolver/configuratieketen

**Bron:** automatische configuratie heeft een expliciete RandomSource zonder native implementatie: `services/net-config/README.md:309-320`. De native dienst start statisch (`sdk/examples/network-service/src/service.rs:153-175`). DNS-code valideert ID/vraag, weigert truncatie en bindt beide endpoints/checksum/deadline: `services/net-protocol/src/dns.rs:369-395`, `services/net-protocol/src/dns.rs:478-545`. Retry/TCP-policy is callerverantwoordelijkheid, geen resolver in deze module.

**Belangrijk bewijsdetail:** native DNS voert een echte A-query uit en controleert TTL, maar `NET_CACHE_INTACT` controleert `neighbor.lookup(PEER_IP)` - dus **ARP-burencache**, niet DNS-cache: `sdk/examples/net-probe/src/main.rs:269-298`. AAAA-parsing is geen IPv6-stack; geselecteerde smoltcp-features zijn IPv4/TCP: `services/net-stack/Cargo.toml:8-13`.

**Impact:** native DNS-deelbewijs mag niet worden opgewaardeerd tot TTL-expiry/cache-isolatie, negatieve caching, algemene resolver of TCP-fallback. Host-DHCP-state-machines bewijzen geen native lease/reconnect. **Afhankelijkheden:** dezelfde NIC-completionfence, echte entropy, linkevents en app-/bestemmingsbeleid. **Benodigd bewijs:** native DHCP renew/rebind/NAK/expiry/conflict/DECLINE/linkverlies; resolver-NXDOMAIN/stale/TC/TCP-fallback, cache-expiry en geen rechtenverruiming door antwoordwijziging. Zekerheid: hoog voor de onderzochte native entrypoints; geen algemene claim dat nergens protocolbouwstenen bestaan.

### AUDIT-15-06 - P0: opaque NAS-opslag en schema-integriteit zijn geen E2EE/antirollback voor een kwaadwillende server

**Bron:** core accepteert syntactisch geldige snapshots bij recover (`services/nas/native-core/src/lib.rs:303-313`); de snapshot heeft geen eigen MAC/antirollbackanker (`services/nas/native-core/README.md:74-85`). De volumeadapter gebruikt schema- en FNV-checksumvalidatie (`services/nas/native-volume/src/lib.rs:60-75`), geen cryptografisch bewijs tegen een beheerder die een oude geldige image terugzet. Clientqueue-invoer is al opaque, niet door deze library versleuteld (`services/nas/native-client/README.md:1-6`).

**Wat wel bestaat:** host-AES-256-GCM bindt namespace/object/revisie en geeft geen plaintext vrij vóór tagcontrole: `services/nas/crypto.mjs:12-16`, `services/nas/crypto.mjs:46-62`. Native lokale tokenreplay wordt al geweigerd bij cold; dat bewijst niet dat oude inhoud of een oude autoriteitsdatabase cryptografisch als stale wordt herkend.

**Impact:** een beheerder kan beschikbaarheid beëindigen en geldige oude ciphertext/snapshots aanbieden. Zonder onafhankelijk vertrouwde clientversiegeschiedenis is actualiteit niet bewezen. **Afhankelijkheden:** native geautoriseerde object/key-lifecycle, persistente clientankers, herstelbeleid en 02/03. **Benodigd bewijs:** oude maar geldig geauthenticeerde objects/manifests, gewijzigde namespace/revisie/tag, serverrollback, vers apparaat zonder geschiedenis en sleutelverlies; product moet onbewijsbare versheid melden. Geen eis om NAS-beheer te vertrouwen of diens verwijderrecht uit te schakelen. Zekerheid: hoog; bekend ontwerp-/integratiegat, niet een nieuw gereproduceerde bypass.

### AUDIT-15-07 - P1: native quota/revoke/evict zijn substantieel; netwerkpartition en dienstherstel blijven onvolledig

**Bron:** reserveringen tellen mee in quota (`services/nas/native-core/src/lib.rs:543-564`); revoke verhoogt epoch duurzaam en verwijdert handles pas daarna; evict is een tweede transactie (`services/nas/native-core/src/lib.rs:742-778`). Iedere replace-fout blijft Unknown/poisoned (`services/nas/native-core/src/lib.rs:369-383`). Historische cold/replayproef is echte VM-uitvoering, niet alleen hostmodel (`docs/verification/nas-replay-110/README.md:16-31`).

**Open grens:** netwerk-ACK-verlies, partities, meervoudige remote schrijvers en automatisch native herstel van de gecombineerde dienst zijn daarmee nog niet getest. Netwerkdienst kan een verdwenen appslot isoleren, maar gedeelde devicefout eindigt fail-closed; algemene autorestart is geen claim (`docs/verification/n01-native-service-20261102/README.md:51-58`). **Afhankelijkheden:** 01-04, durable queue en ownership/resetprotocol. **Benodigd bewijs:** outage/revoke tijdens staging/commit, NAS vol, crash tussen revoke en purge, recovery zonder oud recht, behoud andere app en expliciete status bij Unknown. Zekerheid: hoog.

### AUDIT-15-08 - P1/P2: provider- en bereikbaarheidsfuncties blijven afzonderlijke open scope

**Bron:** providercontract is een product-/architectuureis, geen native acceptatie (`docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:3-5`, `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:41-60`). Onderzochte NAS-client bevat geen providerimplementatie (`services/nas/native-client/README.md:1-7`, `services/nas/native-client/README.md:62-67`). Geen functionele native provideradapter aangetroffen in de onderzochte netwerk-/NAS-entrypoints.

**Impact:** optionele externe ciphertextopslag (P1, FR-P06) kan niet worden afgetekend op een lokale NAS-opslagproef. Wi-Fi, VPN, captive portal, IPv6-routing en fysiek Ethernet (P2 voor profieluitbreiding) zijn niet bewezen door virtio/IPv4 in QEMU; de profielgrens staat expliciet op `docs/implementation/NATIVE-NET-INTEGRATION.md:66-73`. LAN/WAN-identificatie en NAT-bereikbaarheid zijn niet bewezen door de hostloopbackfixture (`services/nas/TRANSPORT.md:43-45`).

**Afhankelijkheden/bewijs:** expliciete providerkeuze, afzonderlijke credentials zonder inhoudssleutel, gescopeerde netwerkrechten, manipulatie/rollback/outage/delete/revoke en geen fallback; voor elk nieuw netwerkprofiel een eigen device-/route-/veiligheidscontract en native tests. Geen automatische relay/UPnP toevoegen om bereikbaarheid te maskeren. Zekerheid: hoog voor ontbrekende ketenacceptatie; repositorybrede afwezigheid van iedere toekomstige stub is niet gesteld.

## Verplichte foutscenario's: dekking versus ontbrekend eindbewijs

| Scenario | Aanwezig bewijs / bron | Nog niet aangetoond |
|---|---|---|
| Packet loss / retransmit / blackhole | Historisch native TCP-retransmissies/timeouts en HTTP-antwoorden: `docs/verification/n01-native-service-20261102/README.md:20-34`; host retrybron DHCP: `services/net-config/README.md:284-307`. | Volledige TLS/NAS-sync onder echte loss. |
| Reorder / duplication | DNS-hosttest voor uit-volgorde CNAME: `services/net-protocol/tests/dns.rs:245-259`; RPC-requestreplay wordt geweigerd: `services/network-service/src/lib.rs:357-365`. | Geen aangetoonde native loss/reorder/duplication-faultmatrix voor de volledige keten; CNAME-recordvolgorde is geen IP-packetreorderbewijs. |
| Partition / offline / reconnect | Queue restore/reconcile en hostcases behouden edits: `services/nas/native-client/src/lib.rs:135-146`, `services/nas/native-client/tests/resume.rs:256-307`. | Duurzame native queue, routewissel LAN/internet en multi-clientheal. |
| Klokafwijking / verlopen certificaat | Native tijdpoort weigert RTC; echte hostcertificaat-/CA-/pinnegatieven zijn opgeslagen: `sdk/runtime/src/tls_time.rs:58-66`, `services/nas/tests/transport.test.mjs:89-112`. | Geautoriseerde kalendertijd en native certificaat-expiry-/clockskewhandshake. |
| Ingetrokken peer / oude sessie | Hosttransport controleert rechten opnieuw en oude facts herleven niet: `services/nas/TRANSPORT.md:41-47`; native NAS-boot-1-replay: `docs/verification/nas-replay-110/README.md:16-31`. | Cryptografisch remote peer-revoke gekoppeld aan native bestaande TLS/NAS-sessies. |
| Volle NAS / quota / beheerder wist | Reserveringsquota en tweefasen-evict in native bron; hostnegatieven: `services/nas/native-core/tests/authority.rs:210-239`, `services/nas/native-core/tests/authority.rs:274-333`. | Productgedrag via remote netwerk en UI, fysieke capaciteit/powerloss. |
| Gelijktijdige edits | Core commit-CAS bewaart winnaar; queue bewaart conflict: `services/nas/native-core/src/lib.rs:617-641`, `services/nas/native-client/src/lib.rs:374-390`. | Meerdere native remote clients, persistente versiegeschiedenis en begrijpelijke conflictoplossing. |
| Onderbroken upload / verloren ACK | Unknown poison/no-replay in bron, queue-hostcases: `services/nas/native-core/src/lib.rs:369-383`, `services/nas/native-client/tests/resume.rs:174-256`. | Native netwerkuitval op iedere commitgrens, duurzaam herstel na client- en servercrash. |
| Serverrollback / kwaadwillende admin zonder sleutel | Host-AEAD bindt context; native snapshot is niet cryptografisch verankerd: `services/nas/crypto.mjs:46-62`, `services/nas/native-core/README.md:80-85`. | Native E2EE en onafhankelijk verankerde actualiteit; beschikbaarheid blijft buiten encryptiegarantie. |
| Backpressure / appcrash / reset | Historisch native UDP-vol-slotdrop, client-exit en reset-ACK/cleanup: `docs/verification/n01-native-service-20261102/README.md:20-40`, `docs/verification/n01-native-service-20261102/README.md:61-85`. | Algemene native dienstrestart met dynamische policy- en TLS-/NAS-sessiegeneraties. |

## Uitgevoerde controles, integriteit en niet-uitgevoerd

Alle onderstaande commando's zijn in de eigen worktree uitgevoerd. Exit 0 waar vermeld betekent uitsluitend de beschreven controle. De gewone `nl`/`sed`/`head`, `rg`, `wc`, `ls` en `git ls-files`-leesrondes inspecteerden bovengenoemde bestanden; zij zijn geen tests. Geen externe instructies uit bronbestanden uitgevoerd.

| Commando / controle | Exit / uitkomst |
|---|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0. Eigen worktree, afwijkende beginbranch/HEAD hierboven, status leeg, bronobject `commit` aanwezig. Worktreelijst is metadata, geen lezen andere checkouts. |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git status --short && git rev-parse HEAD && git ls-files '*AGENTS.md' '.github/agents/os-network-distributed-reviewer.agent.md' && git show-ref --verify refs/heads/copilot/add-os-specialist-agents` | 0. Exacte HEAD/ref, schoon. |
| `git log -5 --format='%H %s' -- services/network-service/src/lib.rs` | 0. Bronhistorie toont reeds opgenomen TCP-fixes; geen andere branch gecombineerd. |
| `git show --stat --oneline cbbc9c9941b08ea5edfbd3637d494635b4fb1df0 -- services/network-service docs/verification` | 0. ReceiveTcp-timeoutfix en eigen bewijs aanwezig in bronhistorie. |
| `sha256sum docs/verification/n01-native-service-20261102/native-report.json docs/verification/n01-native-service-20261102/client-exit-report.json docs/verification/nas-replay-110/nas-system-gate.json docs/verification/full-release-net-985ed4d/net-system-gate.json` | 0. Vier hashes hieronder; N01-rapporthashes gelijk aan opgeslagen README-verwijzingen. |
| `git rev-parse HEAD && git status --short && git diff --check && git diff --exit-code beafdedfc188154fe7285848fbd411e651013f54 -- services kernel drivers sdk tools .github/workflows && git log -1 --format='%H %s'` | 0 voorafgaand aan rapportschrijven. Exacte auditbron, geen productdiff. |

Werkelijk herberekende SHA-256:

| Artefact | SHA-256 |
|---|---|
| `docs/verification/n01-native-service-20261102/native-report.json` | `689edbbb1cefe310b9250e35f20b83f31749ce60960c05d72ebdeec7411209cd` |
| `docs/verification/n01-native-service-20261102/client-exit-report.json` | `e7fad081a1a0c5c307852e6ef1dd111ff6a0c3c2bf3432c198ff3860eb569ec7` |
| `docs/verification/nas-replay-110/nas-system-gate.json` | `7b0ee56a7e1c033f8aeb0bdea4f1163305a2a7c200ecd6dd4a8a2f10ed8d5ff0` |
| `docs/verification/full-release-net-985ed4d/net-system-gate.json` | `a042ba5589fa67713cd97161d52fb57066d658593cc5f4dc209474c3a7770614` |

Zoekdiagnostiek eerlijk behouden: een vroege `find ... services/nas/src ...` meldde dat die map niet bestaat; een daaropvolgende `rg ... services/nas/src/lib.rs` gaf exit 2 voor dat ontbrekende pad. Daarna de werkelijke `services/nas/native-*`-structuur gevolgd. Dit zijn onjuiste zoekpaden, geen falende producttests. Eén `ls -ld docs/audits docs/audits/subagents` gaf geen bestaande mappen; de opdrachtgebonden rapportmap is bij het toevoegen van dit bestand gemaakt. Lege zoekresultaten zijn niet als volledige repository-afwezigheidsbewijzen gebruikt. Enkele brede leesoutputs werden door de tool afgekapt; beslissende code en bewijs is daarna met gerichte regelbereiken gelezen.

Rapportcontrole: `python3 -B -` met standaardbibliotheek controleerde alle **106** backtick-pad/regelcitaten op bestaande bestanden en geldige regelgrenzen, de exacte HEAD en dat `git status --porcelain --untracked-files=all` uitsluitend dit rapport meldde: exit **0**. Dit is een structuurcontrole, geen automatische inhoudelijke bevestiging van elke citatie. `git diff --check` en de productpaddiff hierboven opnieuw exit **0**; rapport-SHA-256 wordt buiten dit bestand geleverd om zelfreferentie te vermijden.

**Niet uitgevoerd:** Cargo/build/clippy/test, Python-producttests, QEMU, TLS-/loopbacksockets, downloads/installaties, externe CodeRabbit/GitHub/CI, echte NAS/provider/credentials/data/devices, fysieke tests, menselijke gebruikerstest/audit en formele evaluatie. Historische exitcodes in geciteerde artefacts zijn niet de exitcodes van een nieuwe auditrun. Oude logpaden buiten deze worktree zijn niet geopend.

## Aannames, bronnen en overdracht

- De audit vertrouwt alleen op vastgelegde bron en opgeslagen artefacts, niet op de actuele waarheid van externe joblinks. Alleen vier geselecteerde artefacthashes herberekend; geen complete image-/firmware-/ELFherbouw of alle manifesthashes opnieuw gevalideerd.
- Begrensde leesdekking: netdriver/kernelbroker, native netwerkmanifest/IPC-entrypoint, service/stack/config/DNS, TLS-adapters, NAS-core/volume/probe/queue, hostcrypto/-transport, relevante tests en bewijs. Geen exhaustieve parserfuzzing, cryptoaudit of bewijs van afwezigheid van alle overige bugs.
- Lokale kennisbank: OSDev **Network Stack**, OSDev-community, W002538,
  revisie 23577 (16 april 2019),
  `.agents/skills/os-dev/references/corpus/wiki-05.md:14922-14959`,
  oorspronkelijke bron
  `https://wiki.osdev.org/index.php?title=Network_Stack&oldid=23577`.
  Alleen als aanvullende laagindeling gebruikt, geen normatieve
  protocolautoriteit. Geen externe bron opgehaald; geen protocolcorrectheids-
  claim uitsluitend op deze communitytekst gebaseerd.
- Bindende engineeringnormen komen uit de repository: privacy-/identiteitsgrenzen `docs/LOCAL-PRIVACY-CONTRACT.md:35-65`, providergrens `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:62-85`, native brokerbesluit `docs/DECISIONS.md:1348-1365`. Primaire standaarden en versies die de repository noemt zijn in deze audit niet extern opnieuw geverifieerd.
- Integratievolgorde op afhankelijkheden: (a) geautoriseerde native TLS/crypto/tijd en caller/policy/devicefence, (b) remote peer-/account-/keycontinuiteit en native objectencryptie/versieankers, (c) duurzame queue + resolver/configuratie en sync, (d) gecombineerde foutmatrix en expliciete provideradapter. Kleine positieve VM-deelproeven blijven bruikbaar, maar vervangen deze acceptatie niet.
- Geen productscorevoorstel, geen nieuwe punten en geen statuswijziging. Hoofdagent blijft eigenaar van score/roadmap en eventuele vervolgimplementatie. Dit rapport vraagt geen fixes, commits of publicatie.

**Blockers voor de gevraagde audit zelf:** geen. **Blockers voor productacceptatie:** 01/02/03/06 en de gekoppelde P1-integratiegaten. Het rapport bevat geen goedkeuring van de volledige release of absolute veiligheid.
