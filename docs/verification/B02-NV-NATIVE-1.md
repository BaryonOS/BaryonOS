# B02-NV-NATIVE-1 — native counter over koude TPM-herstart

30 oktober 2026. **Vier native NV-guests en beide bestaande TPM-regressies
geslaagd.** Dit realiseert een ontbrekende bouwsteen, niet de volledige
rollbackbescherming van gebruikerssleutels. Geen nieuwe rubricpunten.

## Bron, contract en besluit

Basis `c532ba88fbf0f53b960e9dbf77421cee29d97096` van
`simon/native-test-version`; eigen implementatie/geteste Gitbron
`9ae314deedd1b0aa75a8c84d0d0af299cad50ae2`, branch
`codex/b02-nv-native-20261030`. V3-bronfingerprint
`36944a001df3a8476e1b36fb36c2b347be826702a5c855b6b295af25ece32761`.
Deze basis bevat de bestaande IPC-WAIT/TPM-MMIO-integratie; ze is niet
stilzwijgend als main of een geïntegreerde productrelease gelabeld.

Contract/invarianten:
[`TPM-NV-NATIVE-1`](../contracts/TPM-NV-NATIVE-1.md).
Ontwerpbesluit NV1: uitsluitend aparte opt-in bootstrap-testfeatures mogen
de bestaande TPM-codec/transportkern met openbare synthetic-password-auth
voor NV gebruiken. De gewone propertyquery, ontbrekend-deviceproef en
apprechten behouden hun bestaande gedrag. Geen productpolicy in de kernel
en geen nieuwe userspace-TPM-capability. Dit is geen productie-authontwerp;
de fixtureautoriteit blijft expliciet alleen in wegwerp-QEMU/swtpm.

Schrijfbestanden: `kernel/Cargo.toml`,
`kernel/src/m2/m3/tpm_fixture.rs`, `tools/build.py` (featureselectie),
`tools/run_tpm.py` (expliciete private-stateherstart),
`tools/test_tpm_nv.py`, `tools/tests/test_tpm_nv.py` en eigen contract/bewijs.
Geen nieuwe unsafe of dependency; de bestaande MMIO-SAFETY-onderbouwing,
buffers, supervisor-RW/NX/UC-mapping en PIT-deadlines zijn hergebruikt.
Geen KEY1-wire, keyring-, NAS-, publisher-, RTC- of gewone appinterface gewijzigd.
Writegebied is vooraf afgestemd met Build en aan release/teamleider gemeld.

## Exacte uitgevoerde commands

Werkdirectory: eigen worktree. De vooraf bestaande peerfixtures zijn
uitsluitend read-only gebruikt; nieuw aangemaakte TPM-state en alle
bouw-/runbestanden zijn eigendom van deze worktree.

```sh
. .tools/bootstrap/security-native/env.sh
export LOCALAPPDATA="/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools"
export CARGO_HOME="$PWD/.tools/nieuw-os/cargo"
export RUSTUP_HOME="$PWD/.tools/nieuw-os/rustup"
export PATH="$PWD/.tools/nieuw-os/cargo/bin:$PWD/.tools/bootstrap/host-linker/bin:$PWD/.tools/bootstrap/host-linker/usr/bin:$PATH"
export GCC_EXEC_PREFIX="$PWD/.tools/bootstrap/host-linker/usr/lib/gcc/"
export LIBRARY_PATH="$PWD/.tools/bootstrap/host-linker/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/bootstrap/host-linker/bin/cc"
python3 -B -u tools/test_tpm_nv.py > /tmp/nieuw-os-nv-native.log 2>&1
python3 -B -u tools/run_tpm.py --case normal > /tmp/nieuw-os-nv-tpm-normal.log 2>&1
python3 -B -u tools/run_tpm.py --case absent > /tmp/nieuw-os-nv-tpm-absent.log 2>&1
```

Alle drie commands exit **0**, zes echte QEMU-guests raw33, geen timeout.
De vier NV-kernels worden elk via bestaande `tools/build.py` gebouwd; dezelfde
gewone CPL3-wachtapps en hun volledige IPC-WAIT-assertions blijven actief.
Geen hosttool voert de NV-commands uit; de eigen kernel encodeert en bedient
de CRB-registers met echte PIT-tijd.

| Gerichte overige controle | Uitkomst |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml` | exit0; 23 bestaande transport-/codectests |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml --all-targets -- -D warnings` | exit0 |
| `python3 -B -m unittest tools.tests.test_tpm tools.tests.test_tpm_nv` | exit0; 6 tests |
| `rustfmt --check --edition 2021 kernel/src/m2/m3/tpm_fixture.rs` | exit0 |
| `git diff --check` | exit0 vóór codecheckpoint |

Rust/formattering via `LOCALAPPDATA="$PWD/.tools" python3 -B tools/dev.py`.
Hosttest/Clippy gebruikt de lokale GCC met dezelfde GCC_EXEC_PREFIX/
LIBRARY_PATH plus
`RUSTFLAGS="-C linker=$PWD/.tools/bootstrap/host-linker/bin/cc"`;
die hostflags zijn niet aan native targetbuilds doorgegeven. Volledige logs
zijn compact opgenomen in `b02-nv-native-1/`.

## Werkelijke waarnemingen

| Eigen nieuwe guest + volledig nieuwe swtpm-child | Waarneming | Tijd |
|---|---|---|
| `provision` op nieuwe private state | Indexpublicarea gecontroleerd, eerste increment/read=1, volgende increment/read=2 | 4,827s |
| `read` op dezelfde bewaarde state | Na beide processen volledig te stoppen: counter=2, geen Define/Increment-command | 4,885s |
| `auth` opnieuw koud | Correcte read=2; verkeerde auth geeft canoniek TPM_RC `0x98e` met Completed-effect; volgende correcte read=2 | 4,812s |
| `missing` opnieuw koud | Correcte read=2; ontbrekende andere index geeft canoniek TPM_RC `0x18b` met Completed-effect; volgende correcte read=2 | 4,886s |
| Bestaande `normal` op afzonderlijke nieuwe state | TPM2-familyquery, zeven boundsweigeringen, bevestigde release; geen NV-proef | geslaagd |
| Bestaande `absent` zonder TPM | Unsupported/NotStarted/quarantaine, nul MMIO-writes; volledige bestaande isolatie-/wachtassertions | geslaagd |

Alle NV-guests tonen dezelfde TPM2-family, nul quarantaine, zeven
adapterboundsweigeringen en bevestigde Idle/locality-release. De vier
backendchildren hebben elk exit0; er is geen gedeelde nog draaiende TPM
tussen de guests. Nieuw control-/logdirectory per child; exact dezelfde
expliciete eigen state over de koude herstarts.

De harnasrelatie gebruikt de eerste echte counterread als baseline en vereist
exact +1 na de volgende increment; het programma neemt geen vaste beginwaarde
aan. Alle drie koude vervolgreads moeten exact de laatste bevestigde waarde
behouden. Verkeerde transcriptgetallen, duplicaten, formaat, foutcodes,
stagevolgorde, andere state of hergebruikte backenddirectory worden geweigerd.
De backendtests weigeren lege/publieke/buiten-scope/symlink-state vóór childstart.
Die synthetische tests zijn geen aanvullend native bewijs.

Een geweigerde auth kan TPM-autorisatie-/dictionary-attackstate veranderen.
De proef beweert uitsluitend dat de NV-counter niet wijzigde, niet dat alle
TPM-state byte-identiek bleef. MMIO-WRITES tellen ook buffer-/controlwrites
en zijn geen teller van NV-mutaties.

## Tools en bewijsbinding

QEMU10.2.1 (Ubuntu `1:10.2.1+ds-1ubuntu3.3`), OVMF2025.11,
`pc-q35-8.2`, qemu64, TCG single-thread, 1 CPU/256MiB, geen netwerkdevice.
swtpm0.10.1 met libtpms0.10.2; read-only peerbinary-SHA256
`1ff9ce32a8c6a60846629fdd6fd0f51eb18a076918dac9574d252afa7a1d1844`.
De portable `--tpmstate dir=...` zonder `,lock` blijft behouden.
Elke nieuw gegenereerde state is mode0700, controlesocket mode0600.
Geen fysieke TPM, echte schijf, persoonlijke inhoud of productiekeys.

`b02-nv-native-1/tpm-nv-gate.json` bevat alle vier brongebonden builds/
guestobservaties, device-/firmware-/artifacthashes, exacte QEMU/swtpmcommands,
backendexitstatus en hashes van persistente statebestanden na childstop.
`normal`/`absent` hebben eigen rapporten. Zes submappen bewaren ongewijzigde
result/serial/stderr-bytes en toepasselijke backendlogs. De TPM-state zelf
en binaries worden **niet** gepubliceerd. Lokale `.gitattributes` behouden
ruwe firmware-CRLF; SHA256SUMS bindt de 32 oorspronkelijke rapport-/logbestanden
en wordt tegen de Git-blobs gecontroleerd.

Gate-JSON SHA256:
`602f214d849a6be608b45560e039a5b878338532302eb2f4c43820fea02dc332`;
SHA256SUMS SHA256:
`5c5e6fbf6d3d956aa306fd17e242e6e1191a87e8319baef50193250b79d2c8d0`.

## Open grenzen, rollback en hervatten

Geen disk/master-lineagebinding, sleutelwrite-ahead, native encrypted
metadatajournal, disk-only rollbackweigering of powercut-reconciliatie.
Een counter alleen is geen vertrouwde restorebeslissing. De TPM/MMIO-keten
en expliciete onafhankelijke state zijn nog aannames; terugrollen van zowel
disk als TPM-state valt buiten de beschermingsclaim. De publieke PW-fixture
is geen productieprovisioning, beveiligde TPM-sessie of herstelgeheim.
Normatieve TCG-documentcontrole en onafhankelijke menselijke audit blijven
open zoals in het bestaande TPM2-CRB-contract.

**Niet uitgevoerd:** volledige overige CI/native matrix op deze nieuwe branch,
kernelbrede Clippy, fysieke hardware, productie-auth-/restorepad en menselijke
deskundigenreview. Geen volle B02-/B04-/B05-/B06- of releaseacceptatie.
Voorstel centraal assessment **34→34 totaal, beveiliging4/12→4/12**.

Volgende inhoudelijke stap: geauthenticeerde lineage-/generatiebinding en
crashcontract voor de keyring-volumetransactie ontwerpen vóór de native
restoreadapter een counter als minimum_generation mag gebruiken. Geen
counterwaarde uit de gewone terugrolbare disk als onafhankelijk anker kiezen.

Rollback: gerichte terugname van de opt-in NV-features/harnastoevoeging, zonder
KEY1-, slot-, bootinfo- of productdataformaatmigratie; de oorspronkelijke
read-only TPM/absent-fixtures blijven dan bestaan.
