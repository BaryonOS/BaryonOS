# Dependencykeuze voor de actieve DOCX-importslice

Nieuwe afgebakende DOCX-tekstimport met echte ZIP/OPC-binding,
raw-DEFLATE en host-CLI. De XML-parser is onderhouden upstreamcode,
geen nieuwe XML-engine. Exacte uitgevoerde acceptatie in de eigen receipt;
geen native app-/Office-clientclaim.

Geselecteerd op 3 november 2026:

| Dependency | Pin en herkomst | Licentie en build | Gebruik/grens |
|---|---|---|---|
| roxmltree | `=0.21.2`, <https://github.com/RazrFalcon/roxmltree/tree/v0.21.2> | MIT OR Apache-2.0; daadwerkelijke registrymetadata heeft geen buildscript; no_std+alloc, unsafe verboden in upstream | Namespace-aware XML, DTD uit, geen entityresolver, nodebudget 32768, XML maximaal 1 MiB |
| memchr | `=2.7.6`, bestaande upstreamclosure van roxmltree, <https://github.com/BurntSushi/memchr/tree/2.7.6> | Unlicense OR MIT; build.rs afwezig (404), defaultfeatures uit | Alleen no_std substringzoekcode; std/logging/runtime-CPUdetectie uit. Upstream kan target-geconfigureerde SIMD gebruiken; native targetbuild moet dit nog bevestigen |
| zune-inflate | `=0.2.54`, defaultfeatures uit | Bestaande ledger `services/image-codec/dependency-ledger.json`; MIT OR Apache-2.0 OR Zlib, geen buildscript | ZIP raw-DEFLATE; limiet/size hint ingesteld, finale lengte en ZIP-CRC opnieuw getoetst; best-effort heapgrens, geen nieuwe compressie-engine |

roxmltree-Cargo.toml, src/lib.rs, LICENSE-MIT, CHANGELOG en de volledige
tag-rootboom van de aanvankelijke 0.21.1-kandidaat zijn daadwerkelijk gelezen.
Daarna meldde de registry patch 0.21.2: Cargo.toml en changelog daarvan
zijn gelezen; release 24 oktober 2026 verhelpt stackoverflow bij diep
geneste XML door iteratief parsen. Daarom is 0.21.1 **niet** de geselecteerde
productversie. Changelog 0.21 behandelt ook entityresolver- en
parsercorrecties. De resolverfunctie wordt **niet** ingeschakeld.
memchr-Cargo.toml/README/LICENSE-MIT zijn gelezen: no_std expliciet
ondersteund, MSRV 1.61, optional log/coredeps niet geselecteerd.
Dit is herkomst-/ontwerpcontrole, geen onafhankelijke securityaudit.

Upstream XML gebruikt infallible allocaties ondanks input/nodebudget.
Native gebruik vereist een afzonderlijk begrensd parserproces; deze slice
zal geen native OOM-herstel, kernelcapability of sandbox claimen.
Geen bronbestanden of licentieteksten van derden gekopieerd.
Definitieve registrychecksums staan in Cargo.lock. Daadwerkelijke
`cargo metadata`-targets bevatten voor alle drie upstreamcrates geen
`custom-build`; `cargo tree -e features` bevat uitsluitend deze drie
defaultfeatureloze crates. Alle drie compileren mee voor
`x86_64-unknown-none`. De bijgewerkte roxmltree-registrybron bevat
`#![no_std]`; oorspronkelijke MIT-licentie-artifacts van roxmltree en
memchr zijn aanwezig. Lockfile-checksums, manifests/licentiehashes en
uitgevoerde commando's staan in de receipt.
De eerste offline lockpoging faalde exit 101 door ontbrekende roxmltree-cache;
daarna zijn uitsluitend gepinde publieke registrycrates opgehaald.
