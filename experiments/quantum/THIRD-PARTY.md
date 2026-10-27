# Q01 dependency- en bronregistratie

Controle op 27 oktober 2026. De core heeft nul externe dependencies. De JSON-adapter gebruikt exact `serde =1.0.229` met derive en `serde_json =1.0.154`; beide documentatiepins zijn ongewijzigd werkelijk door Cargo opgelost, opgehaald en met Rust 1.90.0 gebouwd. `Cargo.lock` bewaart alle registrychecksums en exacte transitieve versies. [dependency-inventory.json](dependency-inventory.json) bewaart pakketmetadata, upstreamrepository, minimum-Rustversie, buildscript-hash en noticepaden.

| Package | Versie | Rol | Licentie / gekozen route | Buildscript |
|---|---|---|---|---|
| serde | 1.0.229 | Hostserialisatie | MIT | Compiler-versie, private module in OUT_DIR |
| serde_core | 1.0.229 | Hostserialisatietraits | MIT | Compiler-/targetdetectie, private module in OUT_DIR |
| serde_derive | 1.0.229 | Compile-time proc macro | MIT | Geen |
| serde_json | 1.0.154 | Host-JSON-parser/encoder | MIT | Target-arithmeticselectie |
| proc-macro2 | 1.0.107 | Compile-time macro-infrastructuur | MIT | Compilerprobes; tijdelijke OUT_DIR/probe-bestanden |
| quote | 1.0.47 | Compile-time codegeneratie | MIT | Compiler-versie |
| syn | 3.0.9 | Compile-time Rust-parser | MIT | Geen |
| unicode-ident | 1.0.27 | Compile-time identifierdata | MIT én Unicode-3.0 | Geen |
| itoa | 1.0.18 | Host-integerencoding | MIT | Geen |
| memchr | 2.8.3 | Host-JSON-bytezoeken | MIT | Geen |
| zmij | 1.0.23 | Host-numberencoding | MIT | Compiler-/optimalisatieprofiel |

De verpakte manifests en zes buildscriptbestanden zijn gelezen. Hun aangetroffen acties zijn lokale compiler-/targetdetectie, het genereren van configuratie/modulecode en tijdelijke compilerprobes. Geen installatiescript, netwerkdownload of systeemwijziging is aangetroffen in die buildscripts. De scripts blijven onderdeel van de vertrouwde ontwikkelketen; dit is geen complete code- of supplychainaudit van alle transitive source. Upstream notices zijn ongewijzigd gekopieerd naar `third-party/`.

De primaire documentatie bevestigt de gekozen serde-versies en typed deserialisatie: [Serde 1.0.229](https://docs.rs/serde/1.0.229/serde/), [Serde JSON 1.0.154](https://docs.rs/serde_json/1.0.154/serde_json/) en [Serde container attributes](https://serde.rs/container-attrs.html). `deny_unknown_fields` en typed structs blijven actief; er is geen input via `serde_json::Value`, flatten of uitgeschakelde recursielimiet. `Value` wordt alleen in de testcode gebruikt om synthetische requests/resultaten te vergelijken.

Onderhoud: wijzigingen aan directe pins en `Cargo.lock` gebeuren bewust, met herkomst-/licentie-/buildscriptcontrole en herhaling van de volledige regressieproef. De opgehaalde releaseversies zijn beschikbaar en buildbaar; er is geen volledige actuele advisorydatabase-audit uitgevoerd. Er is geen claim dat deze versies vrij van kwetsbaarheden zijn. Verwijderplan: de hostadapter kan worden verwijderd/vervangen zonder de dependencyvrije core te wijzigen; bij eigen-OS-integratie wordt een eigen begrensde IPC-adapter apart gebouwd.

## SplitMix64

De deterministische sampler volgt Sebastiano Vigna's [SplitMix64-bron, 2015](https://prng.di.unimi.it/splitmix64.c), geraadpleegd op 27 oktober 2026. De upstream plaatst de implementatie in het publieke domein volgens CC0. De mixconstanten en volgorde zijn behouden en gekoppeld aan exacte regressionfixtures. Alleen voor simulatie; geen cryptografische generator of fysieke quantumtoevalsbron.
