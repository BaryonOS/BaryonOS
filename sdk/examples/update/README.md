# B03-voorbeeld: update-dienst

Eén gewoon `no_std`-programma voor de Nieuw OS-ABI (testprofiel
`native-update`, ADR-049). De fixturepolicy is expliciet opt-in via Cargo-
feature `test-signing-anchors`; zonder die feature compileert de dienst niet.
Alle formaat- en beleidskennis zit in
[`update-core`](../../../crates/update-core/); dit voorbeeld verplaatst
alleen sectoren tussen de kernelbroker en die core.

| Programma | Autoriteit bij start | Gedrag |
|---|---|---|
| `update-service` | één blokcapability over de hele slotopslag (lezen, schrijven, flush) | leest header en actief manifest, verifieert de inboxbundel met beide TEST-ankers en de image-digest **vóór** één schrijfactie, beslist met `update_core::decide` tegen de door de kernel gemelde huidige versie, installeert in het niet-actieve slot (image, FLUSH, manifest, FLUSH, header, FLUSH) en meldt alles als bewijswoorden |

De build (`build.py`) activeert deze testfeature expliciet en pint de portable
curve25519-backend; `update-core` pint sha2 op `force-soft`. Het eigen
linkerscript geeft de code 256 KiB vanaf
`0x40030000`; de startpagina blijft op `0x40010000`.

```sh
python3 -B sdk/examples/update/build.py
```

Standaarduitvoer: `build/sdk-update/x86_64-unknown-none/release/update-service`.
De kernelgate `tools/test_update.py` bouwt deze binary zelf via
`tools/build.py --kernel-feature native-update`.

## Grenzen

Geen netwerk, geen pakket- of app-updates (A02), geen releasesleutels (de
ankers zijn TEST-ankers), geen vloerbeheer buiten de loader: de dienst kent
alleen de huidige kernelversie en weigert alles daaronder of gelijk.
