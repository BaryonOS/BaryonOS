# Reviewafhandeling en integratievoorbereiding M6b

Datum: 28 oktober 2026. Oorspronkelijke PR-bron:
`bb22fdbe15919edc23baa158a46236195ce314b2`. CodeRabbit-herstelcommit:
`56c43da0806f5f4de57cfb49dfd0418954e2ed9c`.
De aanvullende bronwijziging betreft uitsluitend `tools/preflight.py` en
`tests/test_preflight.py`: ook een versieprobe met `OSError` of
`TimeoutExpired` telt als onbruikbaar. Dit is geen nieuwe native mijlpaal.

## Werkelijk uitgevoerde hostcontroles

Een afzonderlijke verifier haalde de benodigde tools, tests en ondersteunende
bestanden uit GitHub op en vergeleek hun bytes met de Git-blobhashes. Op die
afgebakende snapshot, aangevuld met de twee genoemde wijzigingen, zijn
**40 Python-hosttests** uitgevoerd: 5 preflight, 17 capture-gate, 1 M6b-herstel,
10 M6b-capture en 7 gastparser. Alle opdrachten eindigden met exit 0.
`host-tests.json` bevat de exacte argv, werkmap, exitcodes en alle
hostbronhashes vóór en na; de bron bleef ongewijzigd. De volledige stdout en
stderr staan naast het record. Dit zijn controles van de hosttools met hun
synthetische testfixtures, geen Rust-builds of QEMU-runs.

De twee aanvullende preflighttests zijn ook vóór de reparatie uitgevoerd.
De vier falende subtests voor probe-excepties zijn als diagnostiek behouden in
`preflight-before-extra-fix.*`. Het geslaagde eindresultaat gebruikt:

- `tools/preflight.py`: SHA-256 `a983b43293d5f277b9d8b810ab14aeab34ab0f9868bbb6a78068a0a6baa6be8f`;
- `tests/test_preflight.py`: SHA-256 `f49847a9d2ec090e1ac3896abd28bccd9bb4a02c78af65fdfafe287cf5726fdc`.

## M5-vergelijking en historische metadata

De M5-vergelijking is opnieuw aan de twee bestaande gastrecords gebonden.
Beide 64 KiB-images zijn uit de opgeslagen volledige seeds gereconstrueerd;
imagehashes en sectordigests komen overeen. Acht negatieve varianten zijn
afgewezen. Dit is hercontrole van bestaand tekstbewijs, zonder nieuwe VM-runs
of uitvoering van de oorspronkelijke ELF-binaries. De oorspronkelijke
uitvoeringsbron blijft `44892e7ddd311021c3ae262f8b8e07f321da87a3`.
De portable checker en zijn concrete heruitvoering staan in deze map.

`metadata-hash-check.json` controleert de oorspronkelijke gate-metadatahash
en beide gelinkte loghashes. Vier gewijzigde bestanden uit het historische
`../../regression-m6b/copied-v3-proof-manifest.json` hebben bewust een nieuwe
inhoud: regressie-README, M3-hosttestmetadata, M5-vergelijking en samenvatting.
De manifesthashes blijven de oorspronkelijke kopie beschrijven en passen
exact bij de inhoud op `bb22fdbe15919edc23baa158a46236195ce314b2`.
Ze zijn geen actuele hashclaim over de gewijzigde HEAD.
Vertalingen van auditbeschrijvingen veranderen geen gemeten waarden,
bronhashes of oorspronkelijke gastuitvoer. De niet vastgelegde checkoutbinding
van de oude M3-hosttest blijft onbekend.

## Native grens en integratie

De remote trees van de oorspronkelijke PR en de CodeRabbit-herstelcommit
bevatten dezelfde native bestanden onder boot, kernel, crates, drivers,
services en userspace. Deze follow-up behoudt die bron. Oude native
testresultaten worden niet als nieuwe uitvoering op de gewijzigde hosttools
gepresenteerd.

De bekende `Unknown`-observerfout staat nu expliciet in het
[M6b-contract](../../../implementation/M6B-OBJECT-STORAGE.md) en de
[acceptatiegrenzen](../../m6b/README.md): ook de gewone milestonebuild kan bij
native WRITE-/FLUSH-fouten panikeren vóór een gecontroleerd clientantwoord.
De zeven bestaande host-ABI-proeven vervangen geen native foutinjectie.
Het servicecontract en de noodzaak van een latere native reparatie blijven
behouden. [Inhoudelijke beoordeling door CodeRabbit](https://github.com/simonvanieperen/nieuw-os/pull/9#discussion_r4395596783).

De oorspronkelijke basis `10b32e3` en de bestaande integratiecommit
`7c2cc5b` hebben dezelfde tree
`616a74a41186c8c028f88edaf129765c0a0eb5b2`.
De eerdere NAS/PQC-squash heeft alleen de geschiedenis uiteen laten lopen.
De integratievoorbereiding verbindt beide geschiedenissen zonder native
inhoud te vervangen. Goedkeuring en squash-merge hebben afzonderlijke
GitHub-statussen; dit bestand claimt die nog niet.
