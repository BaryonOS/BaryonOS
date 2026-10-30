# Native TCP-client — TASK-TCP

Toestand: implementatie in uitvoering; native TCP en FR-N01 blijven open.
Eigenaar: netwerkagent. Schrijfgebied: `services/net-stack/`,
`sdk/examples/tcp-probe/` en dit document. Kernel, ABI, centrale buildprofielen,
generieke runner en voortgang blijven van de integrator. Basis `309ff53`,
clockprerequisite `c4667c9` lokaal als `8151023`; bewijs van die oude bron is
geen bewijs voor de hier toegevoegde code.

## Besluit: onderhouden stack met afzonderlijke appcompiler

De integrator keurde exact `smoltcp = 0.14.0` met uitsluitend de appcompiler
Rust **1.91.1** goed. Kernel en bestaande apps behouden Rust 1.90.0. De nieuwe
zelfstandige Cargo-workspaces bouwen alle pathafhankelijkheden opnieuw uit
bron met 1.91.1, in een aparte targetmap. Er worden geen Rust-rlibs tussen
compilerreleases uitgewisseld; alleen de vastgelegde syscall-ABI kruist de
kernelgrens. De bijbehorende 1.91.1 rust-lld wordt expliciet geselecteerd.
Dit vervangt geen kernel en verplaatst de TCP-stack niet naar de vertrouwde kern.

smoltcp 0.14.0 is gepubliceerd op 17 augustus 2026, vereist Rust 1.91 en heeft
de 0BSD-licentie. Het is de actuele onderhouden uitgave bij controle op
29/30 oktober 2026. Stock 0.12 is verworpen wegens inmiddels herstelde
TCP-windowonderloop, ongeldige IPv4-IHL en SYN/SACK-panicpaden. De gerichte
RustSec-/upstreamcontrole vond geen smoltcp-advisory, maar is geen audit of
garantie dat er geen kwetsbaarheden zijn.

Features zijn uitsluitend Ethernet, IPv4, TCP en integer Reno, met één
interfaceadres, vier buren, één route en vier assemblersegmenten. Standaard-
features staan uit: geen std, alloc, libc, hostsocket, TUN/TAP, CUBIC, floating
point, log, async, IPv6, fragmentatie of heimelijke hostfallback. DNS blijft
de bestaande afzonderlijke native parser; TLS is nog open.

De daadwerkelijke Cargo.lock heeft acht registrycrates. `stable_deref_trait`
is een x86_64-targetafhankelijkheid van heapless; de eerdere voorinspectie
zonder die targettabel is gecorrigeerd. Alle crates zijn op herkomst,
licentie, no_std-code, buildscript en checksum gecontroleerd:

| Crate | Versie | Licentie | SHA-256 van .crate |
|---|---|---|---|
| smoltcp | 0.14.0 | 0BSD | b6f8b28ad56c6e35524a37dd492af5d1a47e31e1a4d175cd12f89c075f01980f |
| heapless | 0.9.3 | MIT OR Apache-2.0 | 25ba4bd83f9415b58b4ed8dc5714c76e626a105be4646c02630ad730ad3b5aa4 |
| managed | 0.8.0 | 0BSD | 0ca88d725a0a943b096803bd34e73a4437208b6077654cc4ecb2947a5f91618d |
| byteorder | 1.5.0 | Unlicense OR MIT | 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |
| bitflags | 1.3.2 | MIT OR Apache-2.0 | bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | 4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600 |
| hash32 | 0.3.1 | MIT OR Apache-2.0 | 47d60b12902ba28e2730cd37e95b8c9223af2808df9e902d4df49588d1470606 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |

De gekozen MIT-/0BSD-licentieteksten staan in
[`THIRD-PARTY.md`](../../services/net-stack/THIRD-PARTY.md) en horen bij een
latere binaire distributie.

Graaf: smoltcp → managed, byteorder, bitflags, cfg-if, heapless;
heapless → hash32, stable_deref_trait; hash32 → byteorder. Geen proc-macro's
of buildafhankelijkheden. smoltcp/build.rs genereert configuratie in OUT_DIR
(SHA `77331ea0f4f21b60da452aec44028978deedb06a3577cf2aaa95c837886c9ac0`).
heapless/build.rs doet een begrensde RUSTC-arm_llsc-compileprobe
(SHA `8bb1b56df2c231f086314c0e61fa995594b221a4c8b67c1bab2d743bc8e2957c`).
Geen netwerk/downloader in beide scripts. Overige crates hebben geen buildscript.
Omgevingsvariabelen `SMOLTCP_*` mogen het profiel niet wijzigen; vaste
configuratieconstanten worden ook tijdens compilatie gecontroleerd.

Rustdistributie: officiële `channel-rust-1.91.1.toml`, 10 november 2025,
SHA `483bb8b271168ee654e3be4f111beefa23b9d0c8b7f4a563a8278978dbe982e5`.
rustc is `1.91.1 (ed61e7d7e 2025-11-07)`; de x86_64-none std-distributie heeft
SHA `069edc1d99b45efd97f1d9746289667dc4dc38d8532da547bcc3d0d735420d16`.
Installatie gebruikte de bestaande project-rustupmap en veranderde geen
defaulttoolchain of globale shellinstellingen.

Herbeoordeling is nodig vóór luisteren/servergebruik of grotere buffers:
upstream issue 1227 meldt window-scale-panic boven 64 KiB, issue 1215
verkeerde SYN-toewijzing bij meerdere luisteraars op dezelfde poort. Deze
actieve-client-API exposeert geen listen, gebruikt maximaal 8 KiB per buffer
en kent geen socketverwijdering/hergebruik. Dit is geen algemene serverreview.

## Interface en invarianten

`services/net-stack` is een no_std, allocatorvrije gebruikersbibliotheek.
`Stack::new` leent vaste RX-/TX-pakketbuffers van elk 1514 bytes en één of
twee caller-owned socketopslagplaatsen. `add_client` leent per socket
1..8192 RX- en TX-bytes; de proef gebruikt 4096 per richting. Geen stille
capaciteitsgroei. `connect`, `send`, `receive`, `state` en `close` gebruiken
alleen proceslokale indices, geen IPC- of kernelcapabilities.

Eén ontvangen frame wordt pas gekopieerd wanneer de RX-slot vrij is; een
volle slot blijft intact. De Device biedt geen nieuwe TX-token zolang een
vorige transmissie lokaal staat. `transmission()` geeft onveranderlijke bytes
met een niet-hergebruikt nummer. De native brug publiceert die bytes met
`NetworkSource`; pas een overeenkomende echte brokercompletion mag
`complete_transmission(id)` uitvoeren. Een lokale TxToken is geen NIC- of
remote-ACK. Onzekere TX-uitkomst veroorzaakt een fout, geen automatische
herpublicatie. TCP-hertransmissie wordt door de echte stack gepland.

Iedere `poll` verwerkt hoogstens één ingresspacket en één begrensde
egresspass over maximaal twee sockets. De caller yieldt tussen stappen.
Tijd komt native uitsluitend uit ABI 1.6 `monotonic_ticks` (syscall 3,
100 Hz, alle zes argumenten nul), met gecontroleerde omzetting naar
microseconden. Achteruitgaande/overlopende tijd wordt geweigerd vóór
pakket-/timerverwerking. Herhaald dezelfde tick aanbieden laat timers niet
verlopen. Hosttests mogen expliciet geïnjecteerde modeltijd gebruiken;
dat is geen native klokbewijs.

De conversie reserveert bovendien de bovenste helft van het i64-bereik voor
timeroptellingen. Twee actieve sockets met dezelfde vier-tuple (lokaal/remote
adres en poort) worden geweigerd. Annuleren sluit de gekozen socket en vereist
vervolgens TX-drain of echte transportreset.

De synthetische proef mag een zichtbare vaste testseed gebruiken. Daarmee
wordt geen beveiligde Internet-ISN of entropy-integratie geclaimd. Een echte
netwerkdienst moet een afzonderlijk toegestane entropybron gebruiken. De
bibliotheek verleent geen netwerkbestemmingsrechten; de bestaande proefcap
en de nog open productiepoort mogen niet met elkaar worden verward.

## Native profielcontract in afstemming

Nieuw opt-in TCP-profiel vereist ABI 1.6/monotonic-clock en dezelfde echte
virtio-net/NetworkSource-broker. Het bestaande ARP/UDP/DNS-profiel behoudt zijn
eigen vaste counters. De TCP-proef heeft variabele pakketvolumes en krijgt
een afzonderlijke observer die app-TX/RX met werkelijke brokercounters
vergelijkt. Geen succes zonder volledige procescleanup en NIC-reset-ACK.

Voorgestelde tegenpartij: uitsluitend localhost QEMU-socketnetdev met
synthetisch Ethernet, MAC `02:00:00:00:00:01`, IPv4 `192.0.2.1`; client
`02:00:00:00:00:07`, `192.0.2.2/24`. Geen TAP/bridge/internet, echte accounts
of persoonlijke data. De host implementeert alleen de protocoltegenpartij.

| Port | Native gebruikershandeling en vereist bewijs |
|---|---|
| 8080 | TCP-handshake, echte HTTP GET, exact fixtureantwoord, FIN/ACK-close |
| 8081 | SYN krijgt geldige RST; client observeert geweigerde verbinding |
| 8082 | Handshake/GET gevolgd door RST; geen geaccepteerd HTTP-antwoord |
| 8083 | Eerste SYN blijft onbeantwoord; transcript bewijst echte retransmissie, daarna HTTP/close |
| 8084 | Tegenpartij antwoordt niet; echte stack-time-out met gemeten Sys3-ticks |

Daarbij horen ongeldige checksums/segmenten, volle buffers/backpressure,
capweigering/intrekking en koude herhaling op het uiteindelijk gecombineerde
image. Counterparty-transcript en kernel/app-evidence worden aan exact bron,
ELF, profiel en imagehashes gebonden. Hosttest, doelbuild en native uitvoering
blijven afzonderlijke bewijsniveaus. Native uitvoering is nog niet gedaan.

Het overeengekomen startupcontract gebruikt cap48, wrong-type endpoint56,
PID176, succes40 en foutstap240. Native counters: 400=HTTP-compleet2,
408=refused1, 416=reset1, 424=retransmitgeval1, 432=werkelijk voltooide TX,
440=werkelijk afgeleverde RX, 448=time-out1, 456=gecontroleerde bodybytes40,
464=startticks, 472=eindticks, 480=cap gesloten/stale weigering1,
488=clockreads, 496=blackhole elapsed ticks minimaal300. De broker verwacht
voor dit aparte profiel precies één stale-capweigering; het oude profiel
behoudt zijn zeven negatieve ABI-proeven. TX/RX worden tegen de echte broker
en het causale peertranscript gecontroleerd. HTTP vereist geobserveerde EOF
en gesloten verbinding vóór de eigen timeout; refused/reset moeten eveneens
vóór die timeout optreden. Dit voorkomt dat een LastAck-timeout bij een
verloren laatste ACK als geslaagde FIN/ACK-afsluiting telt.
Hertransmissie vereist minimaal 100 verstreken PIT-ticks én een echte herhaalde
SYN in het transcript.

De HTTP-body is twintig bytes `Nieuw OS native TCP\n`, SHA-256
`239cdc415a27dd042c4b1388a29bcbb1aa6cd8fa724359706373263092640692`.
Het volledige antwoord is 78 bytes, SHA-256
`d128fa1924be70f9e1f42e2a466e1e94123182b23b11110d273ced90cefda90b`.
Twee gecontroleerde responses leveren samen 40 bodybytes. De 8084-timeout is
300 guestticks; de guestdeadline is 500 ticks voor dit geval en 900 voor de
overige gevallen. Hostwandtijd vervangt dit klokbewijs niet.

FR-N01, TLS/certificaatklok, DHCP/configuratie, bestemmingsrechten, NAS,
browserintegratie, veilige productie-entropy en volledige releaseacceptatie
blijven zichtbaar open. Deze stap verlaagt geen producteis.

## Reproduceren en actuele bewijsgrens

Gebruik de bestaande projectomgeving met de afzonderlijk geïnstalleerde
Rust 1.91.1 en een eigen targetmap. Bijvoorbeeld vanuit de repository:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-tcp-core python3 -B tools/dev.py cargo +1.91.1 test --offline --locked --manifest-path services/net-stack/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-tcp-probe-rust1911 python3 -B sdk/examples/tcp-probe/build.py
```

De helper bouwt uitsluitend `tcp-probe` voor x86_64-unknown-none en meldt
`extra_env.NOS_TCP_PROBE_ELF` plus `net_probe` ELF-/hashmetadata. Hij voert
geen hostclient of kernel uit. Absolute worktreepaden in appcode worden naar
`/nieuw-os` omgezet; native bewijs bindt de werkelijk geteste ELF-hash.
De centrale integrator sluit `native-system-tcp`/`test-system-tcp-recover`
en de runner aan.

Vijftien hostproeven omvatten packetbuffergrenzen/stale completion,
achteruit/overlopende/modeltijd, vier-tuple-isolatie/annuleren, 1501 kapotte
IPv4-IHL-frames, echte ARP→SYN en zes smoltcp-gespreksmodellen (HTTP/EOF,
refusal, reset, SYN-hertransmissie, blackhole-timeout en verloren laatste ACK).
De laatste regressie draait de werkelijk door native gebruikte HTTP-predicate:
eerst rood met de oude predicate, daarna groen met de tijdcontrole. In het
negatieve gesprek zijn volledige bytes en EOF ontvangen, maar alle ACKs
na LastAck worden onthouden; een timerafsluiting blijft geweigerd. De rode
exit101 en bron-/loghashes staan in `verification/lost-finack-red.json`.
Deze modellen gebruiken
geïnjecteerde tijd en geen hostsocket. Het compacte host-/doelbouwrapport
staat onder `services/net-stack/verification/`; native integratieacceptatie
is nog open. Tests en builds zijn geen N01-punt.

## Primaire bronnen

- smoltcp-rs, [smoltcp 0.14.0](https://crates.io/crates/smoltcp/0.14.0),
  bron/manifest/CHANGELOG en 0BSD, gecontroleerd 29/30 oktober 2026.
- smoltcp-rs, [windowcorrectie PR1079](https://github.com/smoltcp-rs/smoltcp/pull/1079),
  [issue1227](https://github.com/smoltcp-rs/smoltcp/issues/1227),
  [issue1215](https://github.com/smoltcp-rs/smoltcp/issues/1215).
- Rust Project, [officiële 1.91.1-distributie](https://static.rust-lang.org/dist/channel-rust-1.91.1.toml),
  10 november 2025.
- IETF, [RFC9293 TCP](https://www.rfc-editor.org/rfc/rfc9293), augustus 2022.
- IETF, [RFC9112 HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112), juni 2022.

Rootintegratie: native-system-tcp/test-system-tcp-recover combineren de echteTCPclient met alle bestaande diensten, optionele100Hzklok en per-proces fpu-context. De aparte native-fpu-proef blijft vereist voor daadwerkelijke x87/SSE-registerpatronen; de integerTCPprobe vervangt dat bewijs niet. Rootgate/capture gebruikt tools/test_tcp_system.py en capture_system.py --tcp; nativeacceptatie blijft open tot deze daadwerkelijk slagen.

## Eerste complete nativepair op1b6a308

Exact1b6a3086176a6c0a4e6514bf4d2f7105bef3d480/v3source25a523bbb8b56ee38b0ec18ab223a855e61e54c8cf454578643298c0660971b1:
normal7.61s en koude readonlyrecovery7.49s, beideraw33. PerVM21werkelijke
TX/13RX gelijk aan app,broker en protocolpeer; HTTP2/refused1/reset1/retransmit1/
timeout1/body40/stalecap1,resetACK1. Sys3loopt438ticks; blackhole300ticks.
Schijfhashad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea
is normal-after/recover-before/recover-after gelijk. Finale geteste normale kernel
bc6c5931777819829ed57de6a2014deece3229f337ddf84d4c61ae0b89583103;
werkelijkeTCP-ELFfea2154f68682f6403a060a013a39c656b9e4c302b0a41061d2d0a31e8f6bb91.
[capture --tcp](../verification/full-release-tcp-1b6a308/capture-validation.json)
hercontroleert echte ESPs/disks/rawserial/peersequenties/finale artifacten.
De eerste mislukteARPtestserverrun blijft apart bewaard; geen onbekende frames genegeerd.
Dit is TCP-integratiebewijs met echte andere diensten, CPUcontext en klok;
TLS/configuratie/destinationbroker/NAS en volledigeFR-N01/N02 blijven open.
