# Qsim — uitsluitend objectvormige requests en gates

Datum: 28 oktober 2026. Dit is werkelijk uitgevoerde **hostverificatie** van
de qsim-parserreparatie. Geen native OS-, QEMU-, eigen-OS-port- of fysieke
QPU-uitvoering is hiermee aangetoond.

## Bron en wijziging

De gematerialiseerde uitvoeringsbasis is
`1068e89c5160bd84849304360b6494ac77ece7b3`. Op die basis zijn twee afzonderlijke
werkbomen gemaakt:

- **Baseline:** de oude parser met uitsluitend de twee nieuwe regressietests.
- **Kandidaat:** dezelfde regressies met map-only deserialisatie voor `Request`
  en `GateRequest`. De actuele README-uitleg noemt geen verouderde vaste
  testtelling meer; de historische 25-testregistratie blijft ongewijzigd.

De reparatie gebruikt expliciete `visit_map`-visitors en `deserialize_map`.
De afgeleide, getypeerde veldcontroles blijven behouden, inclusief het
afwijzen van onbekende en dubbele velden. Positionele requests en H-/CX-gates
als arrays worden daardoor niet meer als een onbedoeld aliasformaat aanvaard.

| Bestand | Oude Git-blob | Gerepareerde Git-blob |
|---|---|---|
| `experiments/quantum/qsim-cli/src/main.rs` | `970c26833e76ad1774850493d39b9cb7d2dd7b6e` | `1ae6f40b955ac12b61be911ffb7af6c13e409dba` |
| `experiments/quantum/qsim-cli/tests/cli.rs` | `17f45e9583031a8724ac0716e6fdca3a5260c2cd` | `24ea4eba34bee04a5b7e161b893ae43fd812b3c8` |

Een latere vergelijking met integratiecommit
`c8945c609d64adaa5e23b57937c5886384e863ea` bevestigt dat alle 49 quantumfiles
en beide build-/toolchainconfiguraties nog dezelfde oorspronkelijke blobs
hebben. Alleen root-`AGENTS.md` is in die latere bron aangepast; die versie is
afzonderlijk gelezen en in de bundel bewaard. De uitvoering wordt daarom
niet achteraf aan een andere checkout toegeschreven. Details staan in
`later-source-equivalence.json` binnen de bundel.

## Werkelijke resultaten

De controles gebruikten de directe Linux-hosttoolchain Rust/Cargo 1.90.0.
De bestaande lockfile is ongewijzigd; `cargo fetch --locked` haalde uitsluitend
de daarin vastgelegde dependencies op. De test-, opmaak- en Clippy-controles
liepen daarna offline.

| Controle | Werkelijk resultaat |
|---|---|
| Nieuwe arrayregressies op oude parser | Cargo exit 101; beide tests falen doordat ongeldige invoer werkelijk CLI-exit 0 geeft waar exit 2 is vereist |
| Volledige gerepareerde quantumworkspace | Exit 0; 27 tests geslaagd, 0 mislukt: 10 CLI-tests en 17 core-/integratietests |
| `cargo fmt --all -- --check` | Exit 0; geen bronwijzigingen |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Exit 0 |

De twee nieuwe regressies zitten **binnen** de 27 geslaagde tests. De twee
voorafgaande verwachte failures worden daar niet bij opgeteld. Deze uitvoering
wordt evenmin opgeteld bij historische testgroepen.

Iedere commandregistratie bevat de 53 gematerialiseerde bron-, configuratie-
en instructiebestanden met SHA-256 vóór en na uitvoering: alle bleven binnen
hun eigen uitvoering bytegelijk. De kandidaatcode is na de geslaagde controles
niet aangepast.

## Reproduceren en ruwe uitvoer

Vanuit `experiments/quantum/`, met de vastgelegde Rust 1.90.0-toolchain en de
bestaande lockfile, zijn de relevante opdrachten:

```bash
# Baseline: oude parser met uitsluitend de twee nieuwe regressies.
cargo test --locked --offline -p qsim-cli --test cli array -- --nocapture

# Gerepareerde kandidaat.
cargo fmt --all -- --check
cargo test --workspace --locked --offline -- --nocapture
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

De exacte werkelijk gebruikte absolute executablepaden, werkmappen,
omgevingsinstellingen, begin-/eindtijden en exitcodes staan in
[review-bundle.json](review-bundle.json). De bundel bevat 28 verliesloos
opgenomen tekstbestanden: commandmetadata, originele stdout/stderr,
bronbinding, oorspronkelijke CLI-tests en beide gebruikte recorder-versies.
Er zijn geen logs samengevoegd of opnieuw als een andere uitvoering gelabeld.

Elke `raw_artifacts`-entry bewaart `encoding`, `content`, bytegrootte en
SHA-256. UTF-8-content levert na UTF-8-encoding de exacte oorspronkelijke
bytes; eventuele base64-content wordt als base64 gedecodeerd. Controleer
grootte en hash na reconstructie. Alle 28 entries zijn vóór oplevering
byte-voor-byte met de losse lokale bestanden vergeleken. De bundel bevat
uitsluitend tekst en hashes; buildbinaries en caches zijn niet ingecheckt.

## Grens en oorspronkelijke review

Dit herstelt de objectvormgrens van de bestaande host-CLI. Er worden geen
nieuwe dependencies, native kernelwijzigingen, fysieke quantumclaims of
nieuwe voortgangspunten op basis van alleen testaantallen geïntroduceerd.

Oorspronkelijke bevinding:
[CodeRabbit-comment 4390803356 op PR #2](https://github.com/simonvanieperen/nieuw-os/pull/2#discussion_r4390803356).
De oude resolved-status van die discussie betekende niet dat de reparatie
daadwerkelijk in de onderzochte actuele bron was opgenomen.
