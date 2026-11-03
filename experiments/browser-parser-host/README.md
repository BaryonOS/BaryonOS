# Browser parser-hostproef

Dit geïsoleerde hostexperiment verbindt expliciete HTML-bytes met de upstream
`scraper`-component voor HTML5-DOM-opbouw en CSS-selectorqueries. Het levert
alleen een begrensde titel-/linkinventaris op. De proef gebruikt geen socket,
bestandstoegang, Nieuw-OS-interface, renderer of browserrechten.

`scraper` is geen browserengine: CSS-styling/layout, JavaScript, netwerk/TLS,
downloadportaal, sandbox en native UI ontbreken. De crate gebruikt de Rust
hostbibliotheek en is niet als `no_std`-component gekozen; dit experiment bewijst
dus geen Nieuw-OS-port of runtime-geschiktheid. De enginekeuze blijft open.

## Invarianten

- De caller levert UTF-8-bytes; meer dan 64 KiB wordt vóór parsing geweigerd.
- Maximaal 64 links komen in de uitvoer. Meer links veroorzaken een fout, geen
  stil afgekapt resultaat.
- Deze invoer-/uitvoerlimieten stellen geen interne heap-, rekentijd- of
  parserdieptelimiet in bij de upstreambibliotheek.
- Een `href` is onbetrouwbare documentdata. Het wordt niet opgelost, opgehaald,
  genormaliseerd of als bestemming/recht beschouwd.
- Markup wordt alleen geparseerd. Er is geen JavaScript-uitvoering.

## Uitvoeren

```sh
python3 -B tools/dev.py cargo +1.90.0 test --locked --manifest-path experiments/browser-parser-host/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 clippy --locked --manifest-path experiments/browser-parser-host/Cargo.toml --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path experiments/browser-parser-host/Cargo.toml --all -- --check
```

Dependencyherkomst, licenties, lockfile en buildscriptinspectie staan in
[`THIRD-PARTY.md`](THIRD-PARTY.md). De uitgevoerde proef en bewijsgrenzen staan
in [`docs/verification/browser-parser-host/`](../../docs/verification/browser-parser-host/README.md).
