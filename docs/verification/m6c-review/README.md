# M6c — afzonderlijke core-, adapter- en bewijscontroles

De finale bron is `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`.
Deze map bewaart afzonderlijke AI-controles; geen menselijke audit of formeel bewijs.
De volledige acceptatie staat in [M6C-ACCEPTANCE.md](../M6C-ACCEPTANCE.md).

| Bestand/map | Scope |
|---|---|
| `core-final/` | 30 Rust-tests, fmt, Clippy en native no_std-check met exacte opdrachten en logs |
| `core-diagnostic/` | Eerdere Clippy-melding; daarna hersteld, geen finale groene controle |
| `r07-summary.json` | Review van contract, core, echte adapter, native probe en brokergrens |
| `adapter-build.json`, `adapter-result.json`, `m6c_adapter_probe.rs` | 15 echte serviceadapterproeven met een host-ABI-shim; geen native devicefoutinjectie |
| `m6c-final-native-audit.json` | Onafhankelijke controle van de 15 finale gastrecords, bronbinding, echte kills en schijfuitkomsten |
| `m6c-actual-journal-records.json` | Seeds en alle 67 niet-lege sectoren van acht echte eindvolumes; imagehashes daaruit opnieuw opgebouwd |
| `m6c-all-artifact-hashes.json` | Hashinventaris van 176 werkelijke bestanden uit de finale runmappen |
| `m6c-audited-source-hashes.json` | 141 bron-/bouwinputs, ieder vergeleken met de finale broncommit |
| `audit_m6c_native.py` | Uitgevoerd onafhankelijk auditscript, zonder imports van projectparsers; bewaart oorspronkelijke sessiepaden |
| `m6c-integration-audit.json` | Statische vergelijking met de M6b-basis, ELF-budgetten en instructie-/symboolcontrole van de geteste binaries |

De native audit draait read-only op bestaande ruwe evidence en telt geen nieuwe
VM-starts. De hostshim en het auditscript bewaren de exacte lokale bron-/runpaden;
die moeten bij een andere checkout bewust worden aangepast. Niet alle genegeerde
buildbestanden worden in Git opgeslagen. De JSON-sector-export bewaart voldoende
gegevens om de gecontroleerde begin- en eindimages zonder binaries te reconstrueren.

De M6b-regressie staat afzonderlijk onder `../regression-m6c/`; de afsluitende
buildvergelijking onder `../m6c-ready/`. De oorspronkelijke M6b-evidence blijft intact.
