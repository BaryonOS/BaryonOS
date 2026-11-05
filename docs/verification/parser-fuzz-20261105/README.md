# Parser-fuzzing van onbetrouwbare invoer — 5 november 2026

**Basis:** main `d777df69`, branch `verify/parser-hardening-20261105`.
**Bewijsniveau:** uitsluitend hostmodel (`cargo test`): dev-profiel met
overflow-checks, behalve update-core (release met overflow-checks) en
image-codec (release plus een kleinere debug-run). Geen native, QEMU- of
echt-netwerkbewijs; geen menselijke audit. Geen criterium in
`docs/progress.json` wijzigt: **41 → 41/100**.

Methode: per gebied een tijdelijke host-harness met eigen xorshift-PRNG (geen
nieuwe dependencies), willekeurige bytes plus mutaties van door de crate zelf
gemaakte geldige encodings, vaste seeds. De harnassen zijn niet gecommit;
alleen geminimaliseerde regressietests.

## Bevindingen en status

| ID | Plaats | Klasse | Status |
|---|---|---|---|
| NET-1 | smoltcp 0.14.0 `wire/tcp.rs:73` via `socket/tcp.rs:1783` | expliciete `panic!` op netwerk-sequencenummer (ook release; net-stack aborteert) | **open**, canary-test |
| NET-2 | `services/net-stack` `Stack::push_frame` | RST/ICMP-reflectie vanaf broadcast- of vreemd IPv4-adres | **hersteld** (029718c0) |
| STOR-1 | `services/volume/src/table.rs:302` `Entry::validate` | `revision + 1` overflow op schijfbytes | **hersteld** (87e32ea8) |
| LOAD-1 | `crates/update-core` `choose`/`decide` via `other()` | `1 - slot` bij handgebouwde `Header { active > 1 }`: panic, of in release installatiedoel 255 | **hersteld**, fail-closed `Format` (laag: niet bereikbaar vanuit schijfbytes, `decode` weigert al) |

**NET-1, oorzaak:** bij SYN-ACK zet smoltcp `remote_last_ack = Some(seq)` terwijl
`remote_last_win` 0 is. Tot het eerste ACK is verstuurd geldt dus
`window_end = window_start − 1`. Een segment ~2^31 verder slaagt door de
wrap-around van de `i32`-vergelijking voor de venstertoets en de aftrekking
panickt. Dit staat ook op upstream main (regel 2055, gecontroleerd op
5 november); 0.14.0 is de nieuwste release.

In deze stack weigert `PacketDevice::receive` ingress zolang er TX openstaat.
De toestand ontstaat daarom alleen als het ACK niet weg kan doordat de buur
niet ARP-opgelost is. De aanvaller moet het ISN kennen, dus on-path zijn (bijv.
een kwaadwillende gateway). Of dit op een echt netwerk bereikbaar is, is **niet**
vastgesteld.

Voor de oplossing is een dependencybesluit nodig van de netwerkeigenaar. Opties:
(a) gepatchte smoltcp vendoren met `Some(seq + 1)`, of (b) wachten op upstream.
Een upstreammelding is een publicatie en is niet verstuurd. Test:
`known_defect_half_space_segment_before_first_ack_panics_smoltcp` in
`services/net-stack/tests/hostile_ingress.rs`; die faalt zodra het defect weg is.

**STOR-1:** in release wrapte de waarde naar 0. Met `history_count == 0` werd
het image dan geaccepteerd en gaf elke commit `Full`; geen geheugen- of
rechtenprobleem. Het Python-model `tools/volume_disk.py` spiegelt nu dezelfde
weigering.

## Zonder bevinding gefuzzt (iteraties)

- **Volume** `table::recover` + service-operaties op geaccepteerde images: 200k
  bytemutaties en 200k veldmutaties. Daarnaast 20k willekeurige tabelbytes en
  300k `wire`/`upload`/`Service::handle`-verzoeken.
- **Block-journal:** 400k. **Objects-journal:** 200k. Beide met een
  canonieke-hercoderingscontrole.
- **net-protocol** UDP/IPv4-, ARP-, DHCP- en DNS-parsers: elk 3M, inclusief
  DNS-pointerketens (geen lus, < 200 ms).
- **net-stack** willekeurige TCP/ICMP/ARP-injectie: 300k. De TCP-replayfuzz
  stopte op NET-1 bij iteratie 63.395.

- **elf-loader-core** `parse`: 400k mutaties en 200k willekeurige invoer.
  **kernel-core** `user::parse`: 800k invoer en 303k opnieuw ondertekende
  ELF-mutaties. Gecontroleerd: geen W+X, geen overlap, alles binnen het venster
  en de entry ligt in een X-segment.
- **app-package-core**: manifest/app-ID 600k, `verify_package` 300k
  (geen gewijzigde byte geaccepteerd) en een `SourceRegistry`-model van 300k.
- **update-core**: manifest/header elk 300k, `verify_manifest` 2×300k (geen
  gewijzigde byte geaccepteerd), `choose`/`decide` 300k en de image-digest
  300k. Getest in release met overflow-checks; ML-DSA was in debug te traag.
- **artifact-store-core**: 400k gevallen tegen een vijandige mock-store.
- **image-codec** `decode`: 2M (release) plus 200k (debug), inclusief
  IHDR tot u32::MAX.

Niet gefuzzt:
- volume `client::validate_reply`, `block/src/wire.rs`, `m6_wire.rs` en
  `objects/src/wire.rs`
- TLS-records (in rustls)
- het RX-pad van virtio-net (alleen handmatig gelezen)

## Uitgevoerde commando's (lokaal, exit)

Host zonder `cc`; de tests linkten via een tijdelijke GCC-extract in `/tmp`
(geen systeeminstallatie). De logs staan lokaal in `/tmp` en zijn niet
gecommit. De sha256 hieronder dient alleen ter identificatie.

| Commando | Exit | Log-sha256 (prefix) |
|---|---|---|
| net-stack `cargo +1.91.1 test --locked --test hostile_ingress` vóór fix | 101 (2 verwachte fails) | `537f45ef` |
| net-stack `cargo +1.91.1 test --locked` na fix (31 ok) | 0 | `00921665` |
| net-stack clippy `-D warnings`, `fmt --check` | 0, 0 | — |
| net-broker, net-config, network-service `cargo test --locked` | 0, 0, 0 | `18b57dad`, `b869cf80`, `990fce56` |
| `cargo +1.90.0 test --locked -p volume-core --test service deleted_object` vóór fix | 101 (panic `table.rs:302`) | `ce8aba56` |
| `cargo +1.90.0 test --locked -p volume-core` na fix (50 ok) | 0 | `d9ba718c` |
| volume-core clippy `-D warnings`, `fmt --check` | 0, 0 | — |
| `python3 -B -m unittest tests.test_volume_disk tests.test_volume_runner` (34 ok) | 0 | `bcd5de86` |
| `cargo +1.90.0 test --locked -p update-core --lib out_of_range` vóór fix | 101 (panic `lib.rs:377`) | `426a5400` |
| `cargo +1.90.0 test --locked -p update-core -p update-tool` na fix (16 ok) | 0 | `bf1b9957` |
| update-core clippy `-D warnings`, `fmt --check` | 0, 0 | — |

Hosted CI op de PR-head moet nog worden beoordeeld.
