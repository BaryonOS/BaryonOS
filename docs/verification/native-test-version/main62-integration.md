# Actuele main na PR #62: inhoudsbehoudende integratie

Integratie vanaf `3d2e1aba218feb39e02a1a5d257c5eb63f50e26b` met
`main` `748e29b7455ca4213a6bf3ba3808c3097458263d` (squash van PR #62).
Nieuwe runtimevingerafdruk v3:
`c20a33000ad604ff91733e73f452aa49613ac0a5db2f8573cc7a55839d22f70b`.
Koude runner-WIP is geïsoleerd op `simon/native-cold-acceptance` a31fd54,
geen onderdeel van deze integratie.

Preflightproductiecode en `tests/test_preflight.py` zijn bytegelijk aan
actuele main. Vendorinhoud is behouden. Root-lock verschilt alleen door
de reeds geïntegreerde eigen `nieuw-os-tpm-crb`-package en kerneldependency;
geen main-dependency verwijderd of vervangen. Conflicten waren uitsluitend
historische status/assessment/documentatie; beide uitvoeringshistories
blijven behouden. De bestaande assessment blijft historisch staan; centrale
queue/status/assessment zijn nu van de teamleider.

Eerste gerichte testselector gebruikte ten onrechte
`tools.tests.test_preflight`: exit1, `ModuleNotFoundError`; de echte
repositorytest staat onder `tests/test_preflight.py`. Geen productfout,
geen skip. Herhaling gebruikt de werkelijke module.

Uitgevoerde commando's, allemaal exit0:

```sh
python3 -B -m unittest tests.test_preflight tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
python3 -B -m unittest discover -s tools/tests
LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" CARGO_TARGET_DIR="$PWD/target/native-test-version/main62-host" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace
python3 tools/progress.py --write
python3 tools/progress.py --check
git diff --check
```

79 gerichte en138 tools-hosttests groen; rootworkspace inclusief doctests
exit0. Logs `/tmp/nieuw-os-main62-integrated-{targeted,tools,workspace}.log`.
Historische nieuwe reviewgates op74d15ac/v324d5f4c1 blijven in
`../full-release-interactive-review/` op hun exacte bron staan.
Na deze gewijzigde v3 moeten normale referentie en gewone zevenprocessen-
bediening opnieuw worden uitgevoerd; geen oud rapport omgelabeld.

Lifecyclecoördinator controleerde inhoudsbehoud tegen PR #65 e07cded:
workflow/desktop.rs en zeven historische bewijsblobs (behalve gecorrigeerde
README) identiek; display/ps2 alleen cfg-formatting; overige toolsverschillen
zijn de zes expliciete reviewfixes en bijbehorende testuitbreidingen.
De afzonderlijke Windowsbewijsmap is blob-identiek aan peercommit3f3f99e.
Dit is een inhoudsbehoudcontrole, nog geen besluit tot PR-sluiting; inhoud
moet werkelijk main bereiken en concrete reviewpunten moeten opgelost zijn.

Geen nieuwe Windowsrun/ZIP op deze bron, geen volledige product-/cold-
recovery-/menselijke-/hardwareacceptatie. Voortgangvoorstel **34% → 34%**;
minstens50% blijft open.
