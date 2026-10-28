# R08 — reproduceerbare hertoets na GitHub-review

28 oktober 2026. De eerste CodeRabbit-review van PR #10 beoordeelde het
broncheckpoint `bc4cd104`. De native eindresultaten waren inmiddels opgeslagen
in `1827302a3ed7d44b0edc372ec6c8d6d4ae17825f`. Dit verslag behandelt de vijf
concrete opmerkingen tegen de actuele bestanden. Het vervangt geen goedkeuring
door de reviewbot; die heeft een eigen GitHub-status.

## Afhandeling van de concrete opmerkingen

| Reviewopmerking | Beoordeling en verwerking |
|---|---|
| [ADR-030: uitgevoerde en geplande proeven scheiden](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395538095) | De checkpointzin kon gepland bewijs suggereren. De actuele ADR in `1827302` noemt de inmiddels werkelijk geslaagde gates met aantallen en verwijst naar het acceptatiedossier. Terugzetten naar “niet uitgevoerd” zou nu onjuist zijn. |
| [Projectstatus: native diagnostic traceerbaar maken](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395538117) | De vroege diagnostic is uit de actuele bewijsclaim vervangen door de definitieve bronacceptatie. [Gastrecords](../m6c/guest-runs.json) en [native audit](m6c-final-native-audit.json) bevatten raw exits, sequence, mutaties en schijfhashes van de eindruns. |
| [Adapterbuild: relatieve argv en commitbinding](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395538128) | [adapter-build.json](adapter-build.json) registreert een nieuwe echte build vanaf de repositoryroot, inclusief mkdir, relatieve bron-/uitvoerpaden, toolchainomgeving, runtimebroncommit, execution HEAD en bronhashes. De oorspronkelijke argv is bewaard. |
| [Core: geen formatteringsmutatie in verificatie, wel bronbinding](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395538133) | Alle vier controles opnieuw uitgevoerd; uitsluitend `fmt -- --check`. [core-recheck/results.json](core-recheck/results.json) registreert HEAD, werkmapstatus, exacte argv, exitcodes, logs en alle bronhashes vóór/na. De oorspronkelijke voorbereidende formattering blijft eerlijk in `core-final/` staan. |
| [Adapterprobe: absolute modulepaden](https://github.com/simonvanieperen/nieuw-os/pull/10#discussion_r4395538144) | Alleen drie `#[path]`-waarden zijn naar repositoryrelatieve paden veranderd. De probe is opnieuw gecompileerd en alle 15 scenario's zijn opnieuw uitgevoerd. Nieuwe hashes staan in de nieuwe build-/resultaatrecords; de vier originele bestanden blijven bytegetrouw onder `adapter-original/`. |

## Werkelijke hertoets

Execution HEAD was `1827302a3ed7d44b0edc372ec6c8d6d4ae17825f`.
De runtimebron bleef `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`, fingerprint v3
`4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345`.

De vier corecommando's (fmt --check, 30 Rust-tests, Clippy -D warnings en native
no_std-check) eindigden met exit 0. Alle 141 broninputs zijn vóór en na de
opdrachten bytegelijk aan de implementatiecommit. Cargo-uitvoer stond in een
afzonderlijke tijdelijke targetmap. De gemeten werkboom bevatte toen al de
afzonderlijke documentatie- en adapterwijzigingen; het rapport legt die status
letterlijk vast en claimt dus geen schone werkboom op dat meetmoment.

De portable adapterbuild en daaropvolgende uitvoering eindigden eveneens met
exit 0: 15 PASS-scenario's, stdout bytegelijk aan de oorspronkelijke uitvoering,
stderr leeg. De 11 vastgelegde bronbestanden en de toolchainhelper zijn aan de
runtimecommit én execution HEAD gebonden en bleven ongewijzigd. Alleen de
modulepaden van de hostprobe verschillen; de scenario's zijn ongewijzigd.
De eerste toolchain-identificatie vond geen `rustc` op het standaard PATH en
startte geen compiler. Die voorbereidingsfout staat in de metadata; de werkelijke
build gebruikt vervolgens de bestaande omgeving van `tools/common.py`.

Reproduceer vanuit de repositoryroot met de beschreven lokale toolchain:

```sh
mkdir -p build/m6c-review-portable
python3 -B tools/dev.py rustc +1.90.0 --edition=2021 docs/verification/m6c-review/m6c_adapter_probe.rs -o build/m6c-review-portable/m6c_adapter_probe
build/m6c-review-portable/m6c_adapter_probe
```

De exacte daadwerkelijk gebruikte argv/omgeving/tijden staan in
[adapter-build.json](adapter-build.json) en [adapter-result.json](adapter-result.json).
De bovenstaande `tools/dev.py`-route levert dezelfde toolchainomgeving aan een
handmatige herhaling. Dit zijn host-ABI-tests, geen native devicefoutinjectie.
Er zijn in R08 geen nieuwe VM's gestart; de native aantallen blijven 19/15 voor
M6c en 18/14 voor M6b-regressie. Herhaalde hostproeven worden niet dubbel geteld.

## Overige automatische signalen, gericht beoordeeld

Een afzonderlijke read-only triage onderzocht de aanvullende generieke
lintmeldingen op de bevroren bron; hiervoor waren geen bronwijzigingen of extra
tests nodig:

- `Popen(args)` en `check_output` in `tools/run_m6c.py` gebruiken argumentenlijsten,
  geen shell. QEMU-argumenten worden lokaal uit vaste opties en een gecontroleerd
  synthetisch volume opgebouwd. Gastuitvoer wordt geen opdracht. Lokale
  toolchain/PATH-betrouwbaarheid blijft een expliciete hostaanname.
- `random.Random(seed)` in `tools/m6c_disk.py` maakt uitsluitend reproduceerbare
  canarybytes voor een synthetische 64KiB-image; geen cryptografische sleutels,
  nonces of bevoegdheden.
- De `specs[key]`-regex komt uit een vaste tabel met begrensde tekenklassen;
  de gast bepaalt de waarde die wordt gecontroleerd, niet het patroon.
- `json.dumps` in de tests schrijft lokale fixtures. Er is geen HTTP-route of
  webframework waarvoor `jsonify` nodig zou zijn.

Dit oordeel betreft die concrete signalen en aanroeppaden, geen algemene
veiligheidsverklaring. Native bron, bestaande core-evidence en M6b-archieven zijn
behouden. De hertoets en triage zijn AI-werk, geen menselijke audit of formeel bewijs.
