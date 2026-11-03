# Browser-parserhostproef

Datum: 3 november 2026 (Europe/Amsterdam). Branch:
`simonvanieperen-vigilant-robot`. Basiscommit:
`17cd0ace7ca8730e3fd6fa8ebc76a226cbc2d1be`. De gewijzigde bronbestanden en
Cargo-lockfile:

| Bestand | SHA-256 |
|---|---|
| `experiments/browser-parser-host/Cargo.toml` | `06588b0d5396c4d877549e86030d0963930a3be5b259f88bf918fe79d060ef7c` |
| `experiments/browser-parser-host/Cargo.lock` | `0219a1ccb541a6d35eace66d4597216439afc47a1e12d003267e1d45f59489b4` |
| `experiments/browser-parser-host/src/lib.rs` | `22dfb750df0d0aaa94ad578f314a79c328cb7ef9320cde4c978bbeee5ef95aef` |
| `experiments/browser-parser-host/fixtures/page.html` | `584e0374d6c51fe304b88a7ca12752b7ba9b8318f1dd4c472546445795201855` |

## Resultaat

Een fixture met UTF-8-HTML wordt via upstream `scraper 0.29.0` omgezet naar
een HTML5-DOM. Een CSS-selector levert een begrensde titel-/linkinventaris.
De adapter accepteert uitsluitend door de caller gegeven bytes; zij maakt
geen sockets, opent geen bestanden en heeft geen Nieuw-OS-interface.

Vijf hosttests controleren de fixture, entity-decoding/selectorresultaten,
ongeldige UTF-8, de limiet van 64 KiB, maximaal 64 links en het behouden van
een `javascript:`-URL als onbetrouwbare data. De grenzen beperken de caller-
invoer en retourvorm, niet interne heap-/tijd-/diepteconsumptie van
`scraper`.

## Dependency- en platformgrens

De versie is exact gepind in
[`experiments/browser-parser-host/Cargo.toml`](../../../experiments/browser-parser-host/Cargo.toml)
en de geïsoleerde lockfile. Herkomst, licenties, dependencyboom en actieve
buildscripts staan in
[`THIRD-PARTY.md`](../../../experiments/browser-parser-host/THIRD-PARTY.md).
Er is geen code van derden gekopieerd.

Een echte `x86_64-unknown-none`-check eindigde met exit 101: `scraper`/
`html5ever`/`tendril` gebruiken `std`. Dit is dus uitsluitend hostbewijs en
sluit geen OS-portgeschiktheid aan. De enginekeuze blijft open. Er zijn geen
CSS-styling/layout, JavaScript, netwerk/TLS/DNS, downloads, opslag, fonts,
input, procesisolatie, compositor, native rendering of capability-integratie
gebouwd of getest. Er is geen browser gestart en geen website opgehaald.

## Uitgevoerde controles

Rust `1.90.0`. De standaard `cc` ontbrak; de repository-preflight
`python3 -B tools/preflight.py --strict-host-tests` eindigde met exit 2.
Voor de hostchecks is dezelfde bestaande GCC 15.2/libc-extractie uit
`/tmp/nieuwos-terminal-toolchain` gebruikt; niets is systeembreed
geïnstalleerd. Cargo targetoutputs staan in
`/tmp/nieuwos-browser-parser-host/target`.

De hostcommando's gebruikten deze tijdelijke linkeromgeving:

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuwos-terminal-toolchain/root/usr/bin/x86_64-linux-gnu-gcc-15
export COMPILER_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuwos-terminal-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu
export RUSTFLAGS='-C link-arg=--sysroot=/tmp/nieuwos-terminal-toolchain/root'
export CARGO_TARGET_DIR=/tmp/nieuwos-browser-parser-host/target
```

| Exact commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fetch --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml` | exit 0 |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml` | exit 0; 5 tests |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fmt --manifest-path experiments/browser-parser-host/Cargo.toml --all -- --check` | exit 0 |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path experiments/browser-parser-host/Cargo.toml --target x86_64-unknown-none` | exit 101; upstream dependency uses `std`; host-only incompatibility bevestigd |
| `python3 -B tools/preflight.py --strict-host-tests` | exit 2; `cc` ontbreekt in de standaard ontwikkelomgeving |
| `git diff --check` | exit 0 op de definitieve wijzigingen |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 metadata --locked --format-version 1 --manifest-path experiments/browser-parser-host/Cargo.toml` | exit 0; dependency-inventaris opgeslagen onder `/tmp/nieuwos-browser-parser-host/metadata.json` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 tree --locked --manifest-path experiments/browser-parser-host/Cargo.toml --target x86_64-unknown-linux-gnu -e normal,build` | exit 0; actieve hostboom onder `/tmp/nieuwos-browser-parser-host/active-tree.txt` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 info scraper@0.29.0 --offline` | exit 0; `/tmp/nieuwos-browser-parser-host/info-scraper.log` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 info html5ever@0.40.1 --offline` | exit 0; `/tmp/nieuwos-browser-parser-host/info-html5ever.log` |

Een hosttestpoging zonder `cc` eindigde met exit 101. De herhaling met
bovenstaande tijdelijke, eerder gebruikte toolchain slaagde; de mislukte
run is afzonderlijk bewaard en is geen groen resultaat. Logs
en cross-targetfout staan buiten Git onder
`/tmp/nieuwos-browser-parser-host/`: `test-missing-linker.log` (exit 101),
`test.log`, `clippy.log`, `fmt.log`, `fetch-locked.log`,
`none-target.log` (Cargo exit 101), `preflight.log` (exit 2),
`metadata.json`, `active-tree.txt`, `info-scraper.log` en
`info-html5ever.log`. De dependency werd voor het eerst
opgehaald met `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fetch --manifest-path experiments/browser-parser-host/Cargo.toml`
(exit 0); de twee offline `cargo info`-commando's hierboven controleren de
opgeslagen releasemetadata. Er is geen QEMU/native-/CI-run uitgevoerd.

De volledige ruwe logs zijn alleen op de oorspronkelijke ontwikkelhost
beschikbaar en maken geen deel uit van deze commit; hun `/tmp`-paden en hashes
zijn geen zelfstandig reproduceerbaar bewijs. De vastgelegde uitvoersamenvatting
is:

```text
test.log: running 5 tests; 5 passed, 0 failed, 0 ignored
  rejects_invalid_utf8 ... ok
  rejects_documents_above_the_input_limit ... ok
  returns_script_like_links_as_untrusted_data ... ok
  parses_fixture_title_and_selector_matches ... ok
  rejects_link_output_above_the_limit ... ok
clippy.log: Finished `dev` profile; exit 0, no warnings
fmt: exit 0, no output
none-target.log: error[E0463]: can't find crate for `std`
  std is required by stable_deref_trait and precomputed_hash
preflight: exit 2; default host has no `cc`
first test attempt: linker `cc` not found; exit 101
```

SHA-256 van relevante uitvoer:

| Log | SHA-256 |
|---|---|
| `test-missing-linker.log` | `1e89bec691da50a060992622f0b78876deb99c8ddbf6a81cd3154a72189536c6` |
| `test.log` | `a1d481b4c50f5618ca4a81bb2ef19ed6ea72a5573fb131bef8c309191f89f8d4` |
| `clippy.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `none-target.log` | `a1f257e58b32e8ebacd37d83633e6397d444391230c606d2f2bb852200e06956` |
| `preflight.log` | `25fd87c7c77ac6e0dee4b08bf48dbae69ce872725b46afce3d825c1b81d23564` |
| `metadata.json` | `1bcd4cac1b293f5d0fb376833a8888b974ac7613d4651baa8bc23c3086dc8a18` |
| `active-tree.txt` | `47c0f46c76d5a0a42cab8d1c216c6c0fec392f7dcac0eea28d1c9a7250dcfc54` |
| `info-scraper.log` | `5d18dd90d69cd6faee05e82d5244a171d9dd77c20b73d0d4fcc3ef1d0d7f0749` |
| `info-html5ever.log` | `f9942e2cd871af8ab3842f3b7675818e966f2d25d525c71a89279582073e7e08` |

**Roadmap:** 41 → 41/100; geen vast criterium is met een hostparserproef
afgetekend. Er is geen engine- of OS-architectuurbesluit genomen.
