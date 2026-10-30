# B03-KEY-POLICY-1 — native signing-key lifecycle

## Actuele herhaling na mainca45-integratie

30 oktober 2026: main `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`
(#70, DHCP-codec) inhoudsbehoudend opgenomen op echte nieuwe testbron
`69d5c3acce8a97e38895f284f13d3de3dbf4a5d9`. De merge was conflictvrij.
Nieuwe v3-fingerprint
`df5a6f39dc081340d8d40dd6c1ba1eaa7bc40ecf372e728ba1df67120b3a73b9`;
geen oud main94-bewijs herlabeld. Updatecode, policy, loader/dienst en
updateharnas zijn per diff ongewijzigd tegenover798ef70. DHCP-codec en
reviewfixes, rootlock, vendor/preflight, SDK-WAIT/admin/RANDOM, centrale
status/score/queue zijn per diff exact mainca45 behouden.

Op deze bron opnieuw uitgevoerd in dezelfde hieronder beschreven omgeving:
`python3 -B -u tools/test_update.py` exit0:18/18 echte QEMU-stages,
15 Rusttests en12 Python-harnastests. Ook
`cargo +1.90.0 clippy --offline --locked -p update-core -p update-tool --all-targets -- -D warnings`
exit0 (eigen target/security-update en lokale hostlinker),
`cargo +1.90.0 fmt --check -p update-core -p update-tool -p os-loader`
exit0,
`cargo +1.90.0 fmt --check --manifest-path sdk/Cargo.toml -p nieuw-os-update-example`
exit0,
`python3 -B -m unittest discover -s tests -p test_preflight.py`
exit0/24tests, en `python3 -B tools/progress.py --check` exit0/34.
Geen centrale assessmentwrite of nieuwe punten.

`python3 -B tools/capture_update.py --destination docs/verification/b03-key-policy-mainca45`
exit0; nieuwe capture `b03-key-policy-mainca45/`. Gate-JSON SHA256
`f1c711e78d450c828d0b854857409460fadf7b1131ba74f033025f2635318475`.
Volledig commandolog `/tmp/nieuw-os-security-update-mainca45.log`, tevens
ongewijzigd als `native-gate.log`; gerichte
`/tmp/nieuw-os-security-{update-clippy,preflight}-mainca45.log`.
Alle56 capturehashes zijn opnieuw tegen de gecommitte Git-blobs gecontroleerd.
SHA256SUMS SHA256:
`50e58f7da1a9f9584f63f89ade2e665cf09fdab1ca00d502f916a8415b47e022`.
De eerste whitespacecontrole signaleerde uitsluitend echte firmware-CRLF
in de ruwe serialcaptures; de bestaande bronbehoudende `cr-at-eol`-regel
is ook voor deze nieuwe capturedirectory toegepast, zonder bytes te wijzigen.

Actuele-head-CI en lifecyclebesluit blijven afzonderlijk nodig; oude
798-CI is geen nieuwe headacceptatie. De eerdere codegerichte AI-peerreview
betrof ongewijzigde updatecode, geen menselijke audit of nieuwe integratie.
Niet uitgevoerd: volledige overige lokale native matrix, fysieke hardware
en externe menselijke deskundigenreview. #41 en alle eerder beschreven
beleidsdistributie-/antirollback-/productieceremoniegrenzen blijven open.
Score34→34/security4of12, centrale eigenaar blijft TL.

## Historische herhaling na main94-integratie

30 oktober 2026: main `94c8908ee363d06a5be51417c5cae635549d566f`
(squashintegratie #66) inhoudsbehoudend opgenomen op geteste Gitbron
`33cc4a7639e5bd4d0cb9a5bb65e40618a130bd00`. Nieuwe v3-fingerprint
`c6b90b631f139a4dd307f65bd3ad49e0376e0b1fee63367869dc92c3f50071fa`.
Alle main94 SDK-WAIT-/admin-/RANDOM-/TPM-bestanden, rootlock, vendor-/
preflightwijzigingen en centrale status/score zijn ongewijzigd behouden.
Het enige conflict was de toevoeging bovenin DECISIONS: zowel ADR-B03K1
als main-ADR-053 zijn volledig behouden, zonder runtimekeuze.

De volledige eerder beschreven `python3 -B -u tools/test_update.py` is
**opnieuw echt uitgevoerd**, exit0, alle18 QEMU-stages, 15 Rusttests en
12 Python-harnastests. De24 preflighttests, update-core/update-tool-Clippy
met `-D warnings` en beide gerichte fmt-commands hieronder slagen eveneens.
`python3 -B tools/progress.py --check` exit0/34; geen centrale assessmentwrite.

Nieuwe brongebonden capture `b03-key-policy-main94/`, via
`python3 -B tools/capture_update.py --destination docs/verification/b03-key-policy-main94`
exit0; gate-JSON SHA256
`b38755f69fcc9420a001d629abaad11dd59d91931af88d315e2c2485b7d1d715`,
SHA256SUMS SHA256
`6011c40ee515e646ad77d3e0b01f7a4be196cca1ac5a94ef31f0c1d07d8662c1`.
Alle55 capturehashes worden opnieuw tegen de Git-blobs gecontroleerd.
Log `/tmp/nieuw-os-security-update-main94.log` ook als `native-gate.log`
gearchiveerd; Clippylog `/tmp/nieuw-os-security-update-clippy-main94.log`.
De onveranderde main94 full-release raw-hostlogs bevatten bestaande lege
EOF-regels: de merge-whitespacecontrole sloot die brongebonden captures uit
en controleerde hun inhoud afzonderlijk exact gelijk aan main94.

De oorspronkelijke codegerichte peerreview vond geen high-confidence fout;
de lifecyclecoördinator controleerde inhoudsbehoud/capturehashes op c5.
Dat is geen menselijke audit of actuele-head-CI-resultaat. Nieuwe-head-CI
blijft een aparte integratiegate; alle open grenzen, #41 en34/security4of12
blijven onveranderd.

## Historische herhaling na main748-integratie

30 oktober 2026: op geteste Gitbron
`933591d4f9aadca357a3e3a4f1c1fd40c8734712` is main
`748e29b7455ca4213a6bf3ba3808c3097458263d` inhoudsbehoudend geïntegreerd.
De main-rootlock, vendor-testdependencies, platformpreflight en centrale
status-/scorebestanden zijn ongewijzigd behouden; de drie conflicten betroffen
uitsluitend README, PROJECT-STATE en progressassessment. Geen blind
runtimebestand gekozen en geen nieuw rubricpunt.

De volledige eerder hieronder beschreven commandoreeks is **opnieuw uitgevoerd**:
`python3 -B -u tools/test_update.py` exit 0, 18/18 echte QEMU-stages, 15 Rust-
en 12 Python-harnastests. Nieuwe v3-bronfingerprint:
`1e38e27ba67c7c6793931cfd59380c1c66210ad25b0a2bd02828bf69252901c1`.
Zelfde emulator/firmware/profiel; totale guesttijd 77,692 seconden.
Clippy en beide gerichte fmt-commands hieronder: exit 0.
`python3 -B -m unittest discover -s tests -p test_preflight.py`: exit 0,
24 behouden preflighttests. `python3 -B tools/progress.py --check`: exit 0,
34/100 consistent; geen assessmentwijziging door de securitydeelagent.

Nieuwe capture: `b03-key-policy-main748/`, via
`python3 -B tools/capture_update.py --destination docs/verification/b03-key-policy-main748`
(exit 0). Gate-JSON SHA-256
`105ad0a7ff6aef156fe55610a62314d8956af496ad17ddf00a5fc0d3d8ef3232`;
SHA256SUMS SHA-256
`73a0d193a367d2208f83ad680373cffec0ed8baa650fa32f23eca059317b5fa4`.
Commandolog `/tmp/nieuw-os-security-update-main748.log`, tevens compact
gearchiveerd als `native-gate.log`; gerichte overige logs
`/tmp/nieuw-os-security-{preflight,update-clippy}-main748.log`.
De 55 capturehashes worden opnieuw tegen de Git-blobs gecontroleerd.

De eerdere fecc4b9/cad3b55-resultaten hieronder blijven **historisch eigen
bronbewijs**, niet naar 933591d omgelabeld. CI en review moeten de actuele
PR-head beoordelen; native herhaling alleen is geen mergebesluit, externe
menselijke review of volledige releaseacceptatie. De open grenzen en
34→34/security4→4 blijven ongewijzigd.

## Historische uitvoering op fecc4b9

30 oktober 2026. **Afgebakende native proef geslaagd; geen nieuwe rubricpunten
en geen sluiting van #41.** Dit vult het historische 13-stage B03-bewijs aan,
zonder dat bewijs achteraf te herlabelen. De beleidsdistributie, kalender-
geldigheid en productiekeyceremonie zijn nog niet gerealiseerd.

## Bron en schrijfgrens

Basis `0ade8014c8179241dcfc68ea0ba8083664691240`, implementatie
`e357f5f8ea88f49a20887205a76152c5b7a23a1e`, definitieve testbron
`fecc4b9202cd247324e3d1f4de173a7bbab1eb55`, branch
`codex/update-key-policy-20261030`.

V3-bronfingerprint:
`4970be1dc386af232ecd40d2b39c9db3744f945951480b809adb572878ce5aa1`.
Het harnas bindt bovendien `tests/test_update_harness.py` apart aan
`9cbfcb0efd8a5c2887e2c9e3406722780615042bf6567b9ed931b9b01d2d9a3f`.
Docs en de CI-stapnaam zitten niet in deze runtimefingerprint. De native code
is sinds e357f5f ongewijzigd; fecc4b9 voegt twee gerichte harnastests en een
CLI-test toe. De volledige native poort is op beide bronversies echt gedraaid;
alleen de tweede uitvoering wordt hier als definitief vastgelegd.

Gewijzigd: `crates/update-core/{src/{lib,policy,test_policy}.rs,tests/key_policy.rs}`,
`boot/loader/src/signed.rs`, `sdk/examples/update/src/service.rs`,
`kernel/src/m2/m3/update.rs` (alleen rejectnamen), `tools/update-tool`,
`tools/{test_update,run_update}.py`, `tests/test_update_harness.py`, openbare
`tools/update-keys`-fixtures en bijbehorende docs/CI-stapnaam. Geen nieuwe
dependency, unsafe, capability, IPC-wire of TPM/NV-/NAS-interface. Het aparte
IPC-WIP op concept-PR #72 is niet onderdeel van deze branch.

Contract: `../implementation/B03-SIGNED-UPDATES.md`, B03-KEY-POLICY-1;
besluit ADR-B03K1. Loader en dienst gebruiken dezelfde `test_policy`, niet
zelfstandig gekopieerde trustregels.

## Exacte uitvoering

Onderstaande lokale omgeving hergebruikt de reeds aanwezige QEMU/OVMF-fixtures
van de buildpeer uitsluitend read-only; alle disks, variabelen, Cargo-targets
en ESP-kopieen worden in deze eigen worktree geschreven. Er is geen fysieke
schijf, productiekey of fysieke TPM gebruikt.

```sh
. .tools/bootstrap/security-native/env.sh
export LOCALAPPDATA="$PWD/.tools"
export CARGO_HOME="$PWD/.tools/nieuw-os/cargo"
export RUSTUP_HOME="$PWD/.tools/nieuw-os/rustup"
export PATH="$PWD/.tools/nieuw-os/cargo/bin:$PWD/.tools/bootstrap/host-linker/bin:$PWD/.tools/bootstrap/host-linker/usr/bin:$PATH"
export GCC_EXEC_PREFIX="$PWD/.tools/bootstrap/host-linker/usr/lib/gcc/"
export LIBRARY_PATH="$PWD/.tools/bootstrap/host-linker/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/bootstrap/host-linker/bin/cc"
python3 -B -u tools/test_update.py > /tmp/nieuw-os-security-update-native-final.log 2>&1
```

Exit **0**, 18/18 echte QEMU-stages; totale gemeten guesttijd 76,323 seconden.
Het bestaande script heeft zelf de hosttests, drie native kernels, signed
UEFI-loader, ring3-update-service en fixturemanifests gebouwd, alle stages
uitgevoerd en daarna de normale ESP-build hersteld. Die laatste build is
geen aanvullende normale bootproef.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --locked -p update-core -p update-tool` (door de gate) | exit 0; 7 core + 6 key-policy + 2 CLI-tests |
| `python3 -B -m unittest discover -s tests -p 'test_update_*.py'` | exit 0; 12 harnastests |
| `cargo +1.90.0 clippy --offline --locked -p update-core -p update-tool --all-targets -- -D warnings` | exit 0; apart `target/security-update` |
| `cargo +1.90.0 fmt --check -p update-core -p update-tool -p os-loader` | exit 0 |
| `cargo +1.90.0 fmt --check --manifest-path sdk/Cargo.toml -p nieuw-os-update-example` | exit 0 |
| `python3 -B tools/capture_update.py --destination docs/verification/b03-key-policy-1` | exit 0; 18 captures |
| `cd docs/verification/b03-key-policy-1 && sha256sum --check SHA256SUMS` | exit 0; alle 55 capturebestanden |
| `python3 -B tools/progress.py --write` en `--check` | exit 0 bij implementatiecheckpoint e357f5f; 34/100 onveranderd |
| `git diff --check` | exit 0 |

De Clippy-uitvoering gebruikt dezelfde lokale hostlinker, met
`RUSTFLAGS="-C linker=$PWD/.tools/bootstrap/host-linker/bin/cc"` en
`CARGO_TARGET_DIR="$PWD/target/security-update"`. Deze hostflags zijn bewust
**niet** aan de native targetbuilds doorgegeven. Formatteringscommando's lopen
via `LOCALAPPDATA="$PWD/.tools" python3 -B tools/dev.py`.

## Native uitkomsten

| Stages | Gemeten uitkomst |
|---|---|
| Oorspronkelijke 13 stages | Alle behouden en groen: install/boot/downgrade, signatures/digest/ELF, echte kill tijdens kopieren en twee herstarts, geen geldig slot, inbox-signature/downgrade/foreign key |
| `rotate-install` | v1 van TEST installeert v2 van vooraf vastgepind TEST-next; exacte opvolgermanifestbytes + image in B, A/inbox behouden, headergeneratie +1, minstens drie flushes |
| `rotate-boot` | Koude nieuwe QEMU-process op dezelfde disk/OVMF-state start v2 uit B; nul writes en flushes |
| `key-revoked` | Correct getekend TEST-revoked-paar geweigerd door loader en dienst; fallback A; nul writes/flushes; gehele slotdisk byte-identiek |
| `key-expired` | Correct getekend v2 buiten TEST-expired-venster 1..=1; zelfde dubbele weigering en ongewijzigde slotdisk |
| `key-not-yet-valid` | Correct getekend v1 voor TEST-next dat vanaf v2 mag tekenen; zelfde dubbele weigering en ongewijzigde slotdisk |

Alle normale guestexits zijn 33; `no-valid-slot` eindigt 35 voor kernelentry;
de cut-stage eindigt -9 nadat het harnas een nog levend QEMU-proces echt heeft
gedood. Een vermelde verwachte weigering is geen genegeerde testfout.

Hostnegatieven testen daarnaast beide verplichte signatures, gewijzigde
versiebytes, ongeldige/dubbele beleidsentries in beide volgordes, gemengde
keyparen, maximale versies en de scheiding tussen cryptografische geldigheid
en autoriteit. De synthetische Python-orakeltest weigert expliciet het oude
v2-manifest op de rotatiedisk; die test is geen native bewijs.

## Bewijs en identiteit

`b03-key-policy-1/update-gate.json` en de 18 genummerde submappen bevatten
brongebonden result/serial/stderr-captures, exacte QEMU-commandoregels,
disk-/vars-/artifacthashes, getallen en onafhankelijke slotdiskorakels.
`native-gate.log` en `clippy.log` bewaren de volledige commandologs.
`SHA256SUMS` bindt de oorspronkelijke 55 capturebestanden; de twee extra logs
zijn geen invoer voor het native verdict.
De lokale `.gitattributes` bewaart de ruwe firmware-CRLF-bytes; de hashes
moeten ook tegen de Git-blobs slagen, niet alleen tegen de lokale werkboom.

| Invoer of rapport | SHA-256 |
|---|---|
| definitief `update-gate.json` | `22cfcc241d6ae0580d2214d3679c491fc4b3522b0334b8b80c0351bbe997f743` |
| capture `SHA256SUMS` | `5c267a337c4d530597f6931ab9bcf12eeaffa285d4ea78a920158cf96af19f5f` |
| OVMF2025.11 code 4M | `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7` |
| OVMF2025.11 schone vars 4M | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |
| signed loader | `07fd2931c58ffa9d43b57a167a6c66ebd8664d5374f0bc6351fc104b2c23b6c0` |
| update-service | `ef985fdb6c67f4381f88bc28f1187a92d3825383b996c724119b1f5e800df12d` |

QEMU10.2.1 (Ubuntu `1:10.2.1+ds-1ubuntu3.3`), TCG single-thread,
`pc-q35-8.2`, qemu64, 1 CPU, 256 MiB, geen netwerkdevice. Dit is bewijs op
deze emulator/firmware, geen fysieke hardwareacceptatie.

## Open grenzen en overdracht

**Niet uitgevoerd:** volledige overige native CI-matrix, CI op de uiteindelijke
integratiecommit, fysieke tests en onafhankelijke menselijke deskundigenreview.
De integrator moet de eigen samengevoegde bron beoordelen en testen; dit
rapport maakt main niet automatisch groen.

Geldigheid is uitsluitend op basis van ondertekende releaseversies; geen
vertrouwde kalenderexpiry. Het oorspronkelijke TEST-paar blijft geldig voor
bestaande profielen. Intrekking is statisch en de opvolger is vooraf vastgepind;
de proef toont geen veilige distributie, monotone opslag/antirollback van
nieuwe beleidsversies, uitschakeling van een gecompromitteerde oude releasekey
of productiekeyceremonie. Het UEFI-versieanker en firmwarevertrouwen behouden
hun bestaande beperkingen; B02/NV en B04 blijven open.

Voorstel aan centrale score-eigenaar: **34 → 34 totaal, 4/12 → 4/12
beveiliging**. B03 was reeds geaccepteerd; deze stap sluit geen nieuw criterium.
B05 vereist externe menselijke deskundigen, niet een tweede AI.

Rollback: gerichte revert van deze updatepolicycommits, zonder manifest-,
slotstore- of IPC-formaatmigratie. Dat verwijdert lifecycleafdwinging en mag
niet als behoud van dezelfde beveiligingsgarantie worden gepresenteerd.
