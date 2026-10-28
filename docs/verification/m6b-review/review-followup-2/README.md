# M6b: vervolg op de volledige CodeRabbit-review

Datum: 28 oktober 2026. Basis:
`3f308b07e0bab445d1ea926561eab8d8e01c86a4`,
tree `0f3768a0afa55ce09bc80cba5b707a5fbc156e52`.
[Review met negen bevindingen](https://github.com/simonvanieperen/nieuw-os/pull/9#pullrequestreview-5676422635).

## Aangepaste controles

De portable [M5-archiefchecker](../merge-followup/verify_m5_archive.py) gebruikt
expliciete foutcontroles in plaats van `assert`. De controles van
gereconstrueerde imagehashes, sectordigests en negatieve vergelijkingen
blijven daardoor ook met `python -O` actief.

De M6a-gate verwijst voor `M6A-USER-ELF` voortaan naar stderr, waar de
unittest-resultaten staan. De bijbehorende capture herkent zowel stdout
als stderr als ingang, controleert de command-exitstatus en bewaart steeds
het volledige trio stdout/JSON/stderr. Vier nieuwe gerichte hosttests
controleren de daadwerkelijke producer- en consumerpaden met testdoubles
voor native build en gastuitvoering.

## Werkelijk uitgevoerd bewijs

| Controle | Resultaat |
|---|---|
| Nieuwe M6a-tests op de oude 3f-tools | Vier testmethoden; vier failures en één error; exit 1 |
| Dezelfde M6a-testbron op de gerepareerde tools | Vier van vier geslaagd; exit 0 |
| Nieuwe M5-checker op oorspronkelijke archiefinputs | Exit 0; twee images gereconstrueerd, acht negatieve vergelijkingen afgewezen |
| Dezelfde nieuwe checker met `-O` | Exit 0; bytegelijke stdout aan de normale uitvoering |
| Oude checker met `-O` op een afzonderlijk gewijzigde seedfixture | Exit 0; reproduceert onterechte acceptatie |
| Nieuwe checker met `-O` op dezelfde seedwijziging | Exit 1; verwachte afwijzing vóór een geslaagd rapport |

De negatieve fixture wijzigt uitsluitend de seed van `m5-read-a` van
`9220620458041727053` naar `9220620458041727054` in een aparte kopie.
De oorspronkelijke archiefinputs zijn niet aangepast. De nieuwe checker
constateert dat de gereconstrueerde image niet overeenkomt met de
gearchiveerde hashes. `prepare_seed_fixture.py` en de mutatierecords leggen
de reproduceerbare wijziging vast.

Alle zes commandorecords bevatten exacte argv, werkmap, timing, exitstatus,
invoerhashes vóór/na en hashes van de ruwe uitvoer. De invoer bleef tijdens
iedere run gelijk. `manifest.json` bindt de drie gewijzigde bronbestanden,
de nieuwe testmodule en de bewijsbestanden aan hun hashes. De basis werd
vooraf met 72 Git-blobs op 3f vergeleken.

Deze controles zijn hosttests en reconstructies uit bestaande gastrecords.
Er is geen nieuwe Rust-build, QEMU-run, native devicefoutinjectie of nieuwe
native gate uitgevoerd. De vier tests worden niet als een herhaling van de
eerdere veertig hosttests opgevoerd.

## Overige reviewpunten

| Punt | Afhandeling |
|---|---|
| ADR-029 en objectservicehandleiding | Het `Unknown`-servicecontract is expliciet alleen via host-ABI getest; bekende native observerpanic en vereiste latere native reparatie/foutinjectie staan vermeld. |
| M6a-bewijsverwijzing in het historische archief | [Afgeleide leesindex](../../m6a-linux/suite.corrected.json); oorspronkelijke suite, capturemanifest en rawlogs blijven bytegelijk. |
| M6a-summary en M6b-ready | Beschrijvend proza naar Nederlands vertaald; meetwaarden, hashes en bronverwijzingen behouden. |
| Historische guardrun | [Manifest](../guards/manifest.json) en README markeren de oude capture-run expliciet als historisch en verwijzen naar de brongebonden v2-eindset. |
| Native bouwinputlijst | [Historische hashselectie](../native-input-hashes.json) aangevuld met `services/objects/Cargo.toml`, bytegelijk op checkpoint 44892e7 en reviewbasis 3f. De oorspronkelijke veertien hashes blijven gelijk. |
| Historische capturetesttelling | Negen tests op de 20f7ed2-basis, in overeenstemming met de vastgelegde uitvoer. |
| M6b-samenvatting | De ongerepareerde observerpanic en het ontbreken van gecontroleerde native `Unknown`-afhandeling zijn expliciet opgenomen. |

Het historische M6a-stdout was leeg; het al gearchiveerde stderr bevat
twaalf geslaagde tests. De gecorrigeerde leesindex behoudt alle 54
resultaatgroepen, tijden, uitslagen en oorspronkelijke bronfingerprint.
Het nieuwe provenanceveld onderscheidt de indexcorrectie van uitvoering.
De oorspronkelijke hashes blijven geldig; er is geen oorspronkelijke
capture geactualiseerd alsof zij opnieuw was uitgevoerd.

## Historie en herhalen

`records/historical/merge-followup/` bewaart alle 22 bestanden van de
oorspronkelijke merge-followup-map bytegelijk, inclusief de oude checker en
zijn werkelijke uitvoer. Het bestaande manifest op de oorspronkelijke
locatie behoudt zijn oorspronkelijke bestandshashes en verwijst nu
expliciet naar commit 3f en deze historische kopie. De nieuwe checkerhash
en nieuwe uitvoeringen staan afzonderlijk in dit vervolgmanifest.
Oude commandorecords zijn niet herschreven.

De historische scriptkopie bewaart ook haar oorspronkelijke relatieve
padberekening. Voor reproductie hoort zij op haar oorspronkelijke pad in
een afzonderlijke checkout van de genoemde bron. Voer haar niet vanaf de
nieuwe archieflocatie uit en schrijf herhalingen naar een eigen werkmap.

Op de gerepareerde bron kunnen vanuit de repositoryroot deze positieve
controles worden herhaald:

```sh
python3 -B -m unittest discover -s tests -p test_m6_capture.py -v
python3 -B docs/verification/m6b-review/merge-followup/verify_m5_archive.py
python3 -B -O docs/verification/m6b-review/merge-followup/verify_m5_archive.py
```

Dit zijn reproductie-instructies; de werkelijk uitgevoerde volledige
opdrachten en absolute werkmappen staan in `records/`. `record_check.py`
bewaart de gebruikte opnamewrapper, geen opdracht om dit historische
bewijs ter plaatse te overschrijven.
