# TASK-APP-SOURCE-SNAPSHOT-CORE-1

30 oktober 2026, Europe/Amsterdam. Branch `codex/app-source-snapshot-core`;
exacte basis `dce3b05aa358f4dbde6f2349b426217779b40052`.
Exclusieve codevrijgave queue
`e6e382cb7a02e0f36951842df04993277ee3a6cd`, blob
`60ff361c5c4553d37b112fc14fdb19894a9122a3`.
Ontwerp: [APP-SOURCE-SNAPSHOT op 083af5e][design].
SEC-herstelcontract: [05784243][recovery].

## Resultaat en beveiligingsgrens

Alleen `crates/app-package-core/src/lib.rs` en deze nieuwe bewijsnote wijzigen.
Nieuwe `SourceRecord`, `SourceStatus`, `SnapshotError`, `export_snapshot` en
`from_snapshot` leveren een structurele kopie/kandidaat, **geen actorbewijs,
geauthenticeerde bronlijst, actualiteit, duurzame ACK of actieve publicatie**.
De toekomstige vertrouwde adapter moet het resultaat buiten haar actieve
verificatiepad houden tot haar afzonderlijke controles zijn geslaagd.
Er is geen runtimeadapter, issuer, decoder, backend, anker of launchgrant
toegevoegd. Frozen #73, #80 en #84 worden niet bijgewerkt.

Export telt eerst de bezette slots; bij te kleine buffer wordt niets
geschreven. Daarna kopieert hij active en revoked in slotvolgorde en laat de
uitvoerstaart ongemoeid. Reconstructie controleert de gehele invoer voordat
zij een nieuwe registry vult: capaciteit, niet-nul SourceId, bestaande
`valid_anchors`, minimumgeneraties en unieke IDs/volledige keyparen.
Geen oude mutaties naspelen, private layout kopieren, sleutelbeleid aanscherpen,
generaties resetten of tombstones hergebruiken.

`SourceStatus` is een Rust-enum met alleen Active/Revoked; een ongeldige
discriminant kan veilige Rust hier niet leveren. Daarom is er geen fictieve
`InvalidState`-runtimefout of byteparser. Een toekomstige decoder moet
onbekende wirestates afwijzen voor deze API wordt aangeroepen.
`SnapshotError` beschrijft alleen structurele fouten; `Stale`, `Unknown`,
actor/domein/rollback en publicatie blijven adapterfeiten. De bestaande
`SourceRegistry::new` en verifiersemantiek zijn niet veranderd.

## Uitgevoerde gevallen

Zes nieuwe tests, naast alle negen bestaande coretests:

- Leeg, volle acht slots en gemengd active/revoked: export/import/export
  behoudt records en volgorde; vervolgmutaties geven dezelfde uitkomsten.
- Nul/te kleine uitvoerbuffer blijft ongewijzigd; grotere buffer behoudt
  haar sentinelstaart. Leeg exporteren naar lege buffer slaagt met nul.
- Ongeldige laatste entry na valide prefix: nul-ID, bestaande keyparsefout,
  dubbele IDs/keyparen inclusief tombstone en ongeldige minimumgeneratie;
  expliciete fout, bestaande registry blijft gelijk. Te veel records weigert.
- Beide statusvarianten behouden `u32::MAX`. Rotate/revoke op actieve max
  weigert overflow; revoked blijft revoked, stale generatie blijft stale.
- Gedeeld afzonderlijk keycomponent maar verschillend volledig paar blijft
  geldig, evenals terugrotatie naar een eerder paar. Geen historische keyban.
- Werkelijk hybride ondertekend ELF wordt na reconstructie geverifieerd;
  stale generatie en latere intrekking blijven geweigerd. Een eerder
  teruggegeven executable is geen actuele autoriteitsverklaring.

Tests-first-aanroep voor de ontbrekende API faalde met exit 101
(compilefouten: types/methoden bestonden nog niet). Dat is **geen
gereproduceerde runtimebug**. Na implementatie slagen alle 15 coretests
en alle 12 bestaande app-managementtests, inclusief de 4.080 BEGIN-reserved
en 16.384 app-ID-pariteitsgevallen. Geen oude tests verwijderd of afgezwakt.

## Exacte commando's en logs

Omgeving voor onderstaande commando's:
```
CC="$PWD/.tools/nieuw-os/dev/cc-wrapper"
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC"
CARGO_TARGET_DIR=/tmp/nieuw-os-source-snapshot-target
```
Rust via bestaande launcher `python3 -B tools/dev.py cargo +1.90.0`;
geen nieuwe dependencies of installatie. Beide freestanding builds krijgen
alleen voor die commandoregel
`CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"'`,
zoals in de bestaande app-package-qualityworkflow.

Alle commando's hieronder hebben exit 0 na implementatie:
```
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p app-package-core
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/app-management/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/app-management/Cargo.toml --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path crates/app-package-core/Cargo.toml -- --check
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/app-management/Cargo.toml --all -- --check
python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release -p app-package-core --target x86_64-unknown-none
python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --manifest-path sdk/app-management/Cargo.toml --target x86_64-unknown-none
git diff --check
```
Vooraf formatter: `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path
crates/app-package-core/Cargo.toml`, exit 0. De volledige hosttests melden
0,20 s core en 0,01 s SDK testtijd; dit is geen native prestatiemeting.

Geteste `src/lib.rs` SHA-256:
`c12b2a1b9922946157aa51b4405bf4a7cd21f6c0617d3ab7d5d5f1b1e615715e`.
Raw logs zijn blijvend bewaard onder sessie
`838f3a4c84a0459f86a1d8927df8e808/files/source-snapshot-core-evidence/`.
De hieronder genoemde namen hebben prefix `nieuw-os-source-snapshot-`.

| Log | SHA-256 |
|---|---|
| `before.log` (compilefout, exit 101) | `8ce81f70dacb3e1beb9ca63747dc4910ca93c83ccb83da7708ec08a3b67c6239` |
| `core-test.log` | `878e6da80bf7236e2f18971328fef209a43edc5b7de42a692e6d3ef2b222175a` |
| `sdk-test.log` | `b87c6d42a5af91d534ca179e147c024a221021e9d3dd674a7637e6a111b26d42` |
| `core-clippy.log` | `ac24372f41a2af694b038e304366d4a0427316802c227385ee08a582f225beef` |
| `sdk-clippy.log` | `60288e0cb8b0fda8f8592bb2d57cba9ea697cbff28e4a5e1cadbc6aa5465679b` |
| `core-none.log` | `0939b6a163e0bc2dfa06c11cc1dbce19c540d77034409ea13fa6d59afd240480` |
| `sdk-none.log` | `a4fa04f74ed41c1aa3eb9a7ecdb262d5afea62729eb2affa26364fc0e5e3fb04` |

## Niet uitgevoerd en overdracht

Niet uitgevoerd op deze nieuwe code: VM/native consumer, duurzame opslag,
crash/cold-rollback, actorautorisatie, integrated-image-CI, hardware of
menselijke audit. Vier native SDK-proeven op basis dce3 blijven historisch
aan die basis gebonden; zij zijn niet herlabeld als uitvoering van deze API.
Het latere providerpad moet zowel correcte als kwaadaardige metadata
behandelen en de afzonderlijke herstelgrens werkelijk afdwingen.

Op codefreeze `0a4e4677764053b3953035a6313c745d89f55a55` zijn alle acht
host/lint/fmt/targetcommando's hierboven opnieuw uitgevoerd, exit 0:
15 coretests en 12 SDK-managementtests. Deze herhaling compileert geen
andere bron; de bovenstaande sourcehash is gelijk gebleven.

De afgesproken gerichte SEC-interfacelezing van die exacte delta en
bewijsnote is afgerond: geen concrete contract-, interface- of trustgrensfout
gevonden tegen ontwerp083 en herstelcontract057. De reviewer bevestigde
dezelfde sourcehash, all-or-nothing, keypairpolicy, eindgeneraties en de
kandidaat-/autoriteitsgrens. Dit is een scoped AI-bronlezing, geen eigen
testuitvoering door SEC, brede audit, menselijke acceptatie of automatische
backend-/publicatierechten. De vervolgcommit wijzigt alleen deze bewijsnote.

Loghashes van de herhaling op codefreeze; prefix
`nieuw-os-source-snapshot-0a4-`, in hetzelfde sessiearchief:

| Log | SHA-256 |
|---|---|
| `core-test.log` | `55bb25dddb67b75a1648222fc95e0bf1e9b4aaa7fe3f41cb720db387a524c9f9` |
| `sdk-test.log` | `5a71a499ccb966ea3327641cf73dff069f59a797c460168a317e6edabd94b470` |
| `core-clippy.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `sdk-clippy.log` | `6d77f64dfc8c5325b90fdbe05d8be367bdd578bcccec24f275c5ffee7d03dd6e` |
| `core-none.log` | `92434e198f27984c035c376ec77a7cf9e6ebdde782f8ccb1336ae3329b66ddb7` |
| `sdk-none.log` | `6bfa9e18ffdef821c711a542cfebd7db0982665615563e5a191285e489609c6f` |

Centrale
assessment/README/PROJECT-STATE hebben een andere eigenaar. Voorstel:
30 oktober 2026, Apps 2/8, totaal **34 -> 34**, geen nieuw geaccepteerd
criterium. De nieuwe API is hostgetest/crosscompiled maar geen duurzame
productinstallatie. Terugdraaien vereist geen datamigratie: deze leaf
introduceert geen opgeslagen encoding of actieve dienst.

[design]: https://github.com/simonvanieperen/nieuw-os/blob/083af5e567689639b3358e16d75463339b7524b4/docs/contracts/APP-SOURCE-SNAPSHOT.md
[recovery]: https://github.com/simonvanieperen/nieuw-os/blob/05784243d3d1ea273b588de82069596648fbde4d/docs/contracts/APP-SOURCE-RECOVERY.md
