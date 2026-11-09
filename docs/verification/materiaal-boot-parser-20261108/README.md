# Materiaal-bootparser na runnerherstel — 8 november 2026

## Probleem en afgebakende oplossing

Bronbasis: `8c8b528306f9d0e474c6b5d338cead904564c2df` (main na #249/#261).
De nieuwe [Materiaal-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/42526590198/job/128871457051)
op eerdere main `1f3e7b6a` startte op `ubuntu-24.04`, maar de hostparser
wees de twee verwachte q35-bootdiagnoses af. Na 3,21 seconden volgde
`Wrong boot/manifest sequence`; exception-cleanup beëindigde QEMU met -9.
Het rapport meldde geen guest- of hosttimeout. De artifactupload slaagde.

De parser gebruikt nu `with_acpi_markers(BOOT)` in dezelfde strikte
incrementele grammatica. De ACPI/DEVICE-regels zijn verplicht, exact,
aaneengesloten en op hun vaste plaats na KS-02. Pre-handoff-markers worden
geweigerd. De kernel, QEMU-configuratie, watchdogs, QMP-bediening, pixel- en
schijforakels veranderen niet. Dit herstelt de parser; het claimt geen
voltooide native Materiaal-keten.

## Werkelijk gecontroleerd

| Controle | Resultaat | Bewijs |
|---|---|---|
| Nieuwe bootprefixregressie op oude parser | Verwachte fout, exit 1; identieke ValueError | [before-fix.txt](before-fix.txt) |
| Volledige Materiaal-probe hosttests | 30 geslaagd, exit 0 | [after-fix.txt](after-fix.txt) |
| Bestaande Materiaal-cold hosttests | 17 geslaagd, exit 0 | [cold.txt](cold.txt) |
| `progress.py --write`, `--check` | Beide exit 0; 41/100 consistent | [verification.json](verification.json) |
| `git diff --check` | Exit 0 | [verification.json](verification.json) |
| Afzonderlijke AI-review en 3 gerichte hosttests | Geen concrete blokkerende regressie; exit 0 | [verification.json](verification.json) |

Python 3.12.14. De [bronhashes](source-hashes.json) binden de tests en review
exact aan de gewijzigde parser en regressietests. De [oorspronkelijke
foutregels](original-failure.txt) zijn een gelabeld logfragment, geen complete
native bewijsbundel. De eerste losse aanroep van de cold-test startte niet
wegens Python-importpad; de bovenstaande bestaande unittest-discoveryroute
voerde daarna werkelijk alle 17 tests uit.

## Integratie en bewijsgrens

De verplichte [main-CI op 8c8b5283](https://github.com/simonvanieperen/nieuw-os/actions/runs/42526645196)
slaagde vóór deze parserfix. Dat oudere resultaat wordt niet als test van
deze nieuwe bron gebruikt. Nieuwe PR-CI en de handmatige native
`materiaal-probe.yml`-run moeten de gepubliceerde branch toetsen. Lokale
Rust-/QEMU-uitvoering is niet uitgevoerd omdat die tools hier ontbreken.

Vier verouderde main-runs en een wachtende run van de gesloten #251 zijn
geannuleerd; storage- en volumevervangers op `1f3e7b6a` slagen. Oudere
featurebranches moeten actuele workflows opnemen; hun reruns nemen de oude
runnerconfiguratie opnieuw over. Er zijn geen branchregels aangepast.

Voortgang: **41 → 41/100**. Geen criterium gewijzigd. De oorspronkelijke
Materiaal-proef is een afzonderlijke beperkte diagnostische route; een
parserreparatie levert geen productacceptatie op. Rollback: deze beperkte
commit via PR terugdraaien; dat herintroduceert de bekende parserfout.
