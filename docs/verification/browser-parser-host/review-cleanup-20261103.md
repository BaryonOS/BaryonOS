# Browser-parserhostproef — reviewcorrecties

Datum: 3 november 2026. Branch: `simonvanieperen-vigilant-robot`.
Basiskop: `ee46451bca4540268905b39dbe2cdde7ca962ca2`.
Bronhash na de correcties:
`experiments/browser-parser-host/src/lib.rs`,
SHA-256 `86e614d7bbb966b5dfc6b79af11447b137ed01f98e7d81616940f38cd094935b`.

## Reproductie

De twee regressietests zijn eerst zonder de codecorrectie uitgevoerd. De
test-only run eindigde met exit 101: vijf tests slaagden, twee faalden. Een
SVG-`title` werd als documenttitel gekozen en 65 links in `<template>` leidden
tot `TooManyLinks`.

Na de correctie gebruikte iedere controle Rust 1.90.0, de bestaande tijdelijke
GCC 15-toolchain onder `/tmp/nieuwos-terminal-toolchain` en de geïsoleerde
Cargo-targetmap `/tmp/nieuwos-browser-parser-host/target`.

| Exact commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml` | exit 0; 7 tests geslaagd |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fmt --manifest-path experiments/browser-parser-host/Cargo.toml --all -- --check` | exit 0 |

De gerapporteerde uitvoer is in deze map bewaard:

| Bestand | Betekenis | SHA-256 |
|---|---|---|
| `review-red.log` | Twee regressietests vóór de codecorrectie; exit 101 | `ac1d0b2bd76c60034720bce6a5f4e0eb605d6325b7ae5b6d060501fa8bec7c80` |
| `review-green-test.log` | Tests na de correctie; exit 0; extra lege eindregels genormaliseerd | `8aae12e611e6f3c1122155bed04e0338dd35c8bcb1c90de236d443649e930824` |
| `review-green-clippy.log` | Clippy na de correctie; exit 0 | `47e741d09aa9bc982f6b3cdfc5a199b7f9e991b2abf48f80d9d88d580d986c93` |
| `review-green-fmt.log` | rustfmt-controle; exit 0, geen uitvoer | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Dit is een hosttest van de bestaande parserproef. Het bewijst geen
browserengineport, OS-integratie, layout/renderpipeline of native browser.
