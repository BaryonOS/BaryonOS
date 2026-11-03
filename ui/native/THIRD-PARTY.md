# Native tekstbronnen

De proportionele systeemtekst, vectorpictogrammen en geometrie komen uit de
lokale, dependencyvrije `nieuw-os-graphics`-crate. De gehashte Open Sans 1.10-
bron, offline rastergeneratie en Apache 2.0-notices staan in
[graphics/THIRD-PARTY.md](../graphics/THIRD-PARTY.md). De app parseert geen TTF.
De bestaande Noto Sans Mono-subset hieronder blijft uitsluitend voor de
bewerkbare documenttekst, zodat caret-/selectiemetrics bij de tekst passen.

# Noto Sans Mono bitmap backend

- Crate `noto-sans-mono-bitmap = 0.3.2`, defaults uit; regular, size16/32,
  unicode-basic-latin, unicode-latin-1-supplement, unicode-specials.
- Sparse-indexversie gepubliceerd 2026-01-07; actuele niet-yanked versie bij
  broncontrole op 2026-10-30. MSRV 1.56.1, editie 2021, no_std, geen runtime-
  dependencies/buildscript/alloc/floats. De minifb-hostexample is uitsluitend
  upstream dev-dependency en is niet in deze graph opgenomen.
- Tarball https://static.crates.io/crates/noto-sans-mono-bitmap/noto-sans-mono-bitmap-0.3.2.crate
  SHA-256 `f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
- Upstreamgit `636fb222507237549c20e9b23dae54fc9d639ae3` uit de daadwerkelijke
  `.cargo_vcs_info.json`. Codegenfont `codegen/src/res/NotoSansMono-Regular.ttf`
  op dat commit: 405892 bytes, versie 2.014, SHA-256
  `56134342e3d5137fbfcb25361039d4a98def7df75db7dd821ef47c3ae03127b6`.
- TTF name-table0: Copyright 2022 The Noto Project Authors
  (https://github.com/notofonts/latin-greek-cyrillic); name-table13/14 verklaren
  expliciet SIL Open Font License 1.1. Het oorspronkelijke TTF-bestand is niet
  gewijzigd, meegedistribueerd of als gewone asset in Git opgenomen.
- Cratelibrary MIT-notice: [licenses/noto-sans-mono-bitmap-MIT.txt](licenses/noto-sans-mono-bitmap-MIT.txt),
  upstream-SHA `d084589aca58e74853c2d03f8a1f7b6f3a13f1fcc4b0c451f30455da09c624b7`.
- Fontnotice met volledige OFL1.1-tekst: [licenses/NotoSansMono-OFL.txt](licenses/NotoSansMono-OFL.txt),
  primaire bron https://raw.githubusercontent.com/notofonts/latin-greek-cyrillic/main/OFL.txt,
  oorspronkelijke SHA `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a`.
  Die copyrightnotice komt overeen met de gepinde fontmetadata. Geen merk- of
  productnaam afgeleid van fontauteurs; de fontbackend staat los van OS-identiteit.

Deze gerichte compatibiliteits-/herkomstcontrole is geen juridische audit of
garantie van onderhoud. De upstream API/codegenfeatures worden niet gewijzigd;
de gekozen subset heeft een expliciete glyphfallback. Later shaping/fontwissel
kan achter dezelfde native render-/componentgrens worden geïntegreerd.

De OFL-notice is inhoudelijk volledig behouden; uitsluitend één upstream
trailing spatie is verwijderd voor de repository-whitespacecontrole. Het
bovenstaande oorspronkelijke bestandshash blijft als herkomst bewaard.
