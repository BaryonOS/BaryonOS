# NAS-VOLUME-1 — adapter op de bestaande volumedienst

`VolumeStorage` implementeert de NAS-`Storage`-interface via de bestaande
`volume_core::client::Client`: geen tweede filesystem, cryptolaag, raw-diskpad
of gewijzigde kernel-/SDK-/netwerkinterface. Dependencies zijn uitsluitend
de exacte lokale `0.1.0`-crates NAS-core en volume-core (inclusief catalog-core).
Deze zijn repositorycode, `no_std`, zonder externe packages of build scripts.

Het vertrouwde startproces levert een **exclusief** NAS-volumeobject en een
kernelgebonden volume-endpoint. Een objectnummer is geen capability en dit
Rust-type maakt geen endpointbeveiliging. De bestaande volumedienst blijft
de domeintoegang, streaming, quota, contentchecks en flush/commitvolgorde
afdwingen. Initialisatie mag alleen bij `NotFound` met revisie nul; een
verwijderd, vreemd, onleesbaar of beschadigd object wordt niet opnieuw leeg
aangemaakt.

Elke vervanging streamt de volledige canonieke snapshot onder verwachte
revisie, commit vóór ACK en ruimt daarna de volumeversiegeschiedenis op.
Dat laatste is essentieel: gewoon WRITE zou oudere actieve autoriteit en
verwijderde ciphertext als herstelbare geschiedenis bewaren. De cleanup
behoudt de huidige snapshot en verwijdert uitsluitend zijn eerdere versies.
Een fout, ontbrekend antwoord of onzekere uitkomst geeft geen succes en
maakt de adapter onbruikbaar voor de volgende mutatie.

Een crash tussen WRITE en PURGE kan een nieuwe huidige snapshot met
achtergebleven geschiedenis nalaten. Normaal `load` schrijft niets en
weigert die toestand met `ReopenRequired`. De vertrouwde herstelcontroller
kan **expliciet** `finish_cleanup` uitvoeren: huidige versie, checksum en
NAS-schema valideren, dan uitsluitend geschiedenis purgen. Vervolgens moet
de NAS-core opnieuw openen. Geen terugval op oude autoriteit, geen stil
opnieuw uitvoeren van de oorspronkelijke gast-/beheeroperatie.

Dit is logische cleanup van de actieve volumetabel, **geen fysieke secure
erase of bescherming tegen schijfreplay**. De oude COW-sectoren, alternatieve
tabellen en externe kopieën zijn niet fysiek gewist. De bestaande
volumechecksum is geen MAC. Kwaadwillige diskrollback blijft een afzonderlijke
client-/sleutel-/ankeropgave; geen NAS-inhoudssleutel wordt hier ingevoerd.

De code kan `no_std` worden gebouwd, maar is nog geen draaiende native
NAS-service. De integratietests gebruiken de werkelijke volumeclient en
volumedienst met diens bestaande gesimuleerde blockdevice. Hostuitvoering
en foutinjectie vervangen geen QEMU-, onafhankelijke disk- of P04-acceptatie.

Gerichte controles, met Rust 1.90.0, hostlinker en een eigen targetmap:

```sh
cargo test --offline --locked --manifest-path services/nas/native-volume/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-volume/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-volume/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-volume/Cargo.toml --all -- --check
cargo build --offline --locked --manifest-path services/nas/native-volume/Cargo.toml --release --target x86_64-unknown-none
```

De release-tests omvatten een NAS-core plus volumeclient op een echte
64-KiB-hostthreadstack. De host-volumedienst draait in een aparte thread,
zodat haar stack niet ten onrechte bij die van de client wordt opgeteld.
Dit test geheugenbudget en besturingslogica, **geen kernel-IPC-authenticatie**.
De bestaande volume-testhelper wordt read-only hergebruikt; er is geen
tweede gesimuleerd blockdevice toegevoegd.
