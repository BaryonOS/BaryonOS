# SUR1 — app naar afzonderlijke compositor

Pure `no_std`, safe Rust, geen allocator/runtime/device-I/O. Root verleent alleen
de compositor een DISPLAY-cap; de app heeft uitsluitend zijn toegewezen
Surface-SEND en eigen reply-RECV. De adapter controleert daadwerkelijk ontvangen
kernel-PID, tag `0x5355524641430001` en nul caps voordat hij `Receiver` aanroept.
Een payload of host-geconstrueerde envelope bewijst geen afzender of autoriteit.

De 48-byte little-endian header bevat `SUR1`, versie1 u16@4, opcode u16@6,
request-ID u64@8, surface-epoch u64@16, Controller-framegeneratie u64@24,
tile-index u64@32 en twee u32-argumenten@40/44. IDs/epoch zijn niet nul. Geen
Rust-structcasts of los paddingmateriaal. Op1=INFO,2=BEGIN,3=CHUNK,4=COMMIT.

| Operatie | Argumenten | Body |
|---|---|---|
| INFO | frame/tile/args nul | leeg; totaal48B |
| BEGIN | lokale x/y | width u16, height u16, pixelbytes u32; totaal56B |
| CHUNK | exact volgende offset / datalengte | maximaal208 bytes; totaal≤256B |
| COMMIT | args nul | leeg; totaal48B |

Eén request-ID geldt voor de hele geordende tiletransactie; maximaal vijf chunks
voor1024 bytes. Ieder chunk gebruikt precies min(208, resterend), zonder overlap,
gat, herhaling of extra bytes. BEGIN/CHUNK sturen geen positieve ACK. INFO is
een afzonderlijke RPC met een eigen oplopend request-ID. Een gepubliceerde
transactie wordt niet automatisch opnieuw verzonden bij een onzekere reply.

Reply heeft dezelfde binding en opcode|0x8000. Status u32@40 is0 bij succes,
anders een `Error`-nummer1..14; pixels u32@44 alleen bij geslaagde COMMIT.
INFO-reply is80B: body32 met schermbreedte/hoogte u32@48/52, toegewezen globale
x/y u32@56/60, toegewezen width/height u32@64/68, tilemax32/8 u16@72/74,
format1 u32@76. COMMIT-reply is48B en rapporteert exact het geschreven aantal
pixels. Negatieve replies hebben pixels0 en geen niet-nul body.

`Geometry::rectangle(tile_index)` bepaalt canonieke lokale row-major32×8-tegels,
met exact kleinere randen. Toewijzing past geheel binnen maximaal1920×1080;
de compositor telt pas na volledige validatie de globale offset op. Geen app
kan hierdoor de verplichte systeemrand beschilderen. Pixelbytes zijn packed
LEu32 `0x00RRGGBB`; ieder vierde byte is nul. De hele tile is gevalideerd vóór I/O.

`Receiver::new(peer, geometry, timeout_ticks)` bewaart maar één1024B-tile.
`accept(kernel_sender, monotone_tick, payload)` geeft Staged, Info(Reply) of een
opaak CommitReady. Alleen `commit_with(token, callback)` kan een positieve
tile-reply maken en voert die callback verplicht uit met het gevalideerde
globale rect/de volledige pixels. Root gebruikt hier **de echte**
`DisplayClient::blit`; een hostrecorder/foutcallback is uitsluitend een test,
geen native bewijs. Er is geen publieke positieve responsebuilder.
De exacte signature is `commit_with(token, now_tick, callback)`: de vertrouwde
native adapter samplet direct bij commit de werkelijke monotone SDK-tick, geen
payload-/host-/wallclockwaarde. Expiry gebeurt vóór payloaddecode en nogmaals
bij commit-admission vóór de eerste blit. Dit belooft geen vsyncdeadline over
latere preëmptie en maakt geschreven pixels niet met terugwerkende kracht ongedaan.

Nieuwere framegeneratie begint alleen op tile0 en vervangt een onaf frame;
oudere/overlappende tiles zijn verboden. `completed_frame()` is alleen Some
nadat alle canonical tiles werkelijk via callbacks zijn bevestigd. App-side
`Controller::presented` blijft eveneens gebonden aan actuele generatie/epoch.
Dit is geen atomische pageflip, vsync of desktopacceptatie.

Root samplet bestaande native monotone tijd en roept `expire(now)` in zijn
serviceloop aan. Timeout/clockrollback/retire geven nooit succes. `reject`
maakt hoogstens één negatieve reply voor een canonieke huidige actor/epoch-
transactie en wist die staging; verdere fragmenten van dezelfde gefaalde ID
worden zonder replystorm genegeerd. De adapter behoudt een te verzenden reply
bij echte IPC-EAGAIN en ontvangt/verwerkt dan geen nieuwe appaanvraag. Hij
sluit onverwachte ontvangen caps. Source-grants worden niet gedeeld of
overgenomen uit app-payloads. Nieuwe service lifetime vereist nieuwe epoch en
grants; een gelijk hergebruikte epoch bewijst geen nieuwe autoriteit.

Vanuit repositoryroot met Rust1.90.0:

```sh
cargo +1.90.0 test --offline --locked --manifest-path ui/surface-protocol/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --release --lib --target x86_64-unknown-none
```

Hosttests controleren werkelijk deze state machine, malformed payloads,
actor/epoch/replay, chunkvolgorde, randen, callbackfalen en timeout. Zij bewijzen
geen kernelafzender, capabilityafdwinging, pixels op GOP of native input/appwerking.

## Optionele appclient

Feature `client` voegt alleen de bestaande SDK-runtime/catalog-Transport toe;
default blijft de pure dependencyvrije assembler. `client::Session::new` neemt
één Surface-SEND/reply-RECV-paar in eigendom na geldige constructie. `geometry`
verifieert de echte compositorreply; `blit(frame,tile,rect,pixels)` publiceert
BEGIN/vijf geordende chunks/COMMIT en accepteert uitsluitend een volledig
gebonden reply met exact pixelcount. Alleen nog-niet-gepubliceerde sends mogen
na kernel-EAGAIN opnieuw. Lostreply/unknownsend/protocolfout sluit beide eigen
channels zonder een tile opnieuw te publiceren. `close` is expliciet en
idempotent. Draft/dirty/presented blijven verantwoordelijkheid van de app;
een error mag nooit die toestand als bevestigd aftekenen.

`verification/freeze.json` bewaart de oorspronkelijke8test-freeze. Het actuele
timeout-/clientbewijs staat afzonderlijk in `verification/deadline-fixed.json`:
de nieuwe regressie faalde daadwerkelijk op de oude bron, de correctie heeft
15 hosttests en onafhankelijke gerichte herreview. Geen native compositorclaim.
