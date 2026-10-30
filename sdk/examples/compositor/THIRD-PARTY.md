# Derdenmateriaal voor de native compositor

De compositor gebruikt exact `noto-sans-mono-bitmap =0.3.2` met dezelfde statische rasterfeatures als `ui/native`: regular, size_16, size_32, unicode-basic-latin, unicode-latin-1-supplement en unicode-specials. De crate is beschikbaar uit de vastgelegde offline Cargo-cache, heeft geen buildscript of fontparser en wordt zonder standaardfeatures gebouwd. Dit is de bestaande gecontroleerde asset; er wordt geen runtime-fontbestand geopend.

De Rust-bitmapverpakking is MIT; de afgeleide Noto Sans Mono-fontdata is SIL OFL 1.1. Beide notices staan ongewijzigd onder `licenses/`. De rechtstreeks opgenomen pure `ui/native/src/design.rs` deelt de kleurentokens met `ui/prototype/tokens.css`; er is geen afhankelijkheid van de documentcontroller of opslagclient. Alle pad-afhankelijkheden staan in de eigen Cargo.lock.

Dit is geen bewijs van native weergave. Het daadwerkelijke compositor-ELF moet via de eigen DisplaySource-broker worden uitgevoerd.
