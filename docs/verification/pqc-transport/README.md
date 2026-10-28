# N02-verificatie: hybride TLS en bestaande NAS-regressies

**Uitkomst:** 124 tests geslaagd, 0 mislukt, 0 overgeslagen; twee demo's exit 0; 20 syntaxiscontroles en `git diff --check` exit 0. Uitgevoerd op 27 oktober 2026 om 22:06 UTC (23:06 Europe/Amsterdam), Node 24.22.1/OpenSSL 3.5.9/Linux x64. Basis `ece94a5cfebecaec763eb2855aa9f84e55dcdcbc`; tussencheckpoint `3d05f9f8dccb4f515d8522578d20bbf00d8b8ed7`. De sourcehashes binden het bewijs aan de geteste bestanden, zonder een commit-ID circulair in zichzelf op te nemen.

| Bestand | Inhoud |
|---|---|
| `report.json` | Exacte command-argv, exitcodes, runtime, tellingen, scope en SHA-256 van alle 20 JS/mjs-bestanden |
| `tests.txt` | Ruwe geïntegreerde testuitvoer: 13 crypto, 19 policy, 23 store, 30 TLS-transport en 39 UI |
| `storage-demo.json` | N01-opslag-/intrekkingsdemo, uitsluitend eigen synthetische data |
| `transport-demo.json` | Werkelijk onderhandelde TLS 1.3/X25519MLKEM768/AES-256-GCM-SHA384; clientdecryptie en ingetrokken send gecontroleerd |
| `syntax.json` | Exacte 20 `node --check`-opdrachten en exitcodes |
| `diff-check.txt` | Ruwe uitvoer van `git diff --check`; leeg betekent exit 0 zonder bevindingen |
| `review.md` | Twee afzonderlijke AI-reviews, één hersteld randgeval en precieze bewijsgrenzen |
| `checkpoint-policy.txt` | Historisch eerste checkpoint met 16 policytests; niet opgeteld bij de definitieve 124 |

## Reproduceren

```sh
node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs
node services/nas/demo.mjs
node services/nas/transport-demo.mjs
```

Vereist lokale loopbacksockets en `openssl` voor tijdelijke testcertificaten. De CLI is OpenSSL 3.0.13; de hybride TLS-handshake draait via Node's afzonderlijke OpenSSL 3.5.9. De standaard sandbox gaf eerst `listen EPERM 127.0.0.1`; expliciete loopback-escalatie is automatisch goedgekeurd. De definitieve opgeslagen run is echt uitgevoerd, niet overgeslagen of met mock-sockets vervangen.

## Grenzen

Dit is hostbewijs: echte loopback-TLS, lokaal opslag-API-gedrag en UI-state. De TLS-testserver echoot ciphertext in RAM en is niet aan de N01-store gekoppeld. Geen native netwerkdienst, geobserveerde WAN/LAN-route, NAS-installatie, formeel kernelbewijs, nieuwe QEMU-run, visuele browsertest of fysieke-hardwaregarantie. Ed25519-testauthenticatie blijft klassiek; alleen de sleuteluitwisseling is hybride postkwantum. Rust/Cargo en QEMU ontbreken in deze sessie; geen native PQC-code, entropy-syscall, boot-/blokwijziging of Grover-uitbreiding uitgevoerd.

Een fact bindt aan app en de toestemmingsgeneratie op het moment van attestation. Eerdere geauthenticeerde facts worden niet door hernieuwde toestemming hersteld. Intrekken sluit een idle socket niet proactief op dat exacte moment; de volgende gecontroleerde handeling weigert en sluit. Al verstuurde bytes kunnen niet worden teruggehaald. De tijdelijke grenzen van vier sockets, acht frames en vijf seconden zijn proefwaarden, geen productbeperkingen.
