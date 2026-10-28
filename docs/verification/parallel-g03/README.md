# G03 — typed catalogusclient via SDK-transport

28 oktober 2026. Gecontroleerde bron
`e29c2b4ac51efecd1df9e36321f5c8ec3250cba0`, B01-afhankelijkheid exact van
`29559563eab51a708afe4ffb3d469280863e5792`. De catalogusdienst/clientbron en
de bestaande G02-probe zijn bytegelijk aan die respectieve checkpoints.

De nieuwe allocatorvrije `sdk/catalog-client`-crate koppelt B01 aan echte
SDK-send/receive/close/yield. De native voorbeeldapp gebruikt nu dezelfde
typed client voor LIST→GET; de eerdere afzonderlijke wireclient is verwijderd.
De adapter behandelt PUT/CAS, RENAME en DELETE via hetzelfde contract en
verandert geen kernel-, capability-, IPC- of opslagformaat.

## Werkelijk gecontroleerd

Alle zeven opdrachten in [report.json](report.json) slagen met exit 0.
Het report bewaart argv, cwd, toolchain, volledige stdout/stderr en
bron-/artifacthashes. Broninputs blijven tijdens de opname ongewijzigd.

- **14 adaptertests** door het werkelijke pure capabilitymodel en de bestaande
  cataloguscore: LIST, NotFound, PUT/CAS, GET, RENAME/DELETE; volle kernelqueue;
  verloren antwoord ná geheugencommit; Unknown; verkeerde kernel-PID/tag/ID;
  extra capabilities en afzonderlijke cleanupfout; echte intrekking; begrensde
  polling en lokale afwijzing zonder transport.
- **13 bestaande SDK-runtimechecks** opnieuw uitgevoerd op de combinatie.
  Dit is 27 hosttests; de eerdere afzonderlijke B01-45 worden niet bijgeteld.
- Host Clippy met beide SDK-crates en alle testtargets, native adapter Clippy,
  native voorbeeld-Clippy, gerichte rustfmt en diffcontrole, allemaal exit 0.
- Native releasebuild van de voorbeeldapp en bestaande ELF-structuurcontrole
  slagen. Exacte afmetingen, segmenten en binary SHA-256 staan in het report.

De lost-reply-test voert één request en één geheugencommit uit en houdt het
antwoord achter. De sessie raakt gesloten; er volgt geen tweede enqueue,
ook niet als dezelfde PUT nogmaals wordt gevraagd. Een malformed syscallretour
blijft `Delivery::Unknown`. De captest laat twee werkelijk gedelegeerde handles
bij B01 aankomen; B01 ziet de oorspronkelijke capvelden, weigert de reply en
de adapter probeert daarna beide ontvangen caps plus beide endpoints te sluiten.
Een mislukte close slaat de overige closes niet over en blijft zichtbaar.

## Reproduceren en grenzen

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client -p nieuw-os-sdk-runtime
cargo clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client -p nieuw-os-sdk-runtime --all-targets -- -D warnings
python3 -B sdk/examples/catalog-read/build.py
```

De overige exacte native/formatopdrachten staan in het report. Gebruik een
eigen checkout en `CARGO_TARGET_DIR` bij parallelle herhaling.

Het hostmodel gebruikt de echte `Engine`- en `State`-implementaties, maar zijn
commit is uitsluitend een geheugenwijziging. Dit bewijs bevat geen nieuwe
native WRITE/FLUSH, stroomuitval, QEMU-boot of zelfstandige voorbeeldappstart.
G02's native SDK-route heeft afzonderlijk rootbewijs. Gewone appstart,
grafische Materiaal-integratie en volledige criteria A01/U03 blijven open;
voorgestelde roadmapverandering is **14% → 14%**.
