# Hosttest-preflight: POSIX- en Windows-linker

Dit rapport registreert de Linux-hosttesttooling voor de taakbasis
`5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`. De preflightwijziging is eerst
getest op broncommit `80393b67950c393dfa37336e66df5346ae6a37d7`; de geteste
codekandidaat is `4fb9a2320dd4d5383846507748ea8fdaf9e167cb`.
Dit is geen OS-, kernel-, boot- of QEMU-acceptatie.

`tools/preflight.py --strict` blijft bootvereisten controleren. De aparte
`--strict-host-tests`-modus meldt ontbrekende POSIX- en Windows-hostlinkers. Na het
beschikbaar maken van een lokale compiler onthulde dezelfde volledige
workspace-test een tweede concrete fout: de README-doctests van de vendored
HKDF-crate misten `sha2` en `hex-literal` in de testdependencies. Die
test-only dependencies zijn nu in beide vendormanifesten hersteld op basis
van de meegeleverde `Cargo.toml.orig`-bestanden in de vendorkopie; zie
`services/keyring/vendor/hkdf-0.13.0/Cargo.toml.orig` en
`services/keyring/vendor/hmac-0.13.0/Cargo.toml.orig`. Ze wijzigen de
runtime-dependencies niet.

Voor Windows selecteert de preflight `link.exe` en gebruikt de gedocumenteerde
MSVC-helpoptie `/?` in plaats van GNU `--version`. Bron: Microsoft Learn,
“MSVC Linker options”, MSVC 17.0-documentatie (gecontroleerd 30 oktober 2026),
<https://learn.microsoft.com/en-us/cpp/build/reference/linker-options?view=msvc-170>.
De Windows-uitvoering zelf is in deze controle niet uitgevoerd; de
regressietests controleren de gekozen toolnaam, de probe-argumenten en de
fail-closed melding wanneer `link.exe` ontbreekt.

| Exacte opdracht | Exit | Uitkomst |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p 'test_preflight.py' -v` | 0 | 22 tests geslaagd; log `/tmp/nieuw-os-infra-preflight-tests.log`, SHA-256 `0d7560f39d2c8324034ba8ec19f61a33afaed1c1f794af039e729fc405aaab8e` |
| `python3 -B tools/preflight.py --strict-host-tests` | 2 | Verwachte melding `missing_host_test_tools: ["cc"]`; JSON `/tmp/nieuw-os-infra-preflight.json`, SHA-256 `a69b5f95f05983848f34fecf92a6a840dda685ef095e1eab3fb65c129c81aa3d` |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` (voor lokale `cc`) | 101 | Stopte bij `linker cc not found`; historisch log `/tmp/nieuw-os-infra-root-final.log`, SHA-256 `95e4d7518ea42c507b3d6bb92242cfa2c95ac58ded3890323baf750516cef1b8` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` (vóór HKDF-testdeps) | 101 | Compiler werkte; drie README-doctests faalden omdat `sha2`/`hex_literal` ontbraken; log `/tmp/1793368389455-copilot-tool-output-7287-5b317826-ac72-468d-beb7-f558d9d8462e.txt`, SHA-256 `59469300748959f1a3a664fd4c232ceb4f7684ddf09206748fb90332936c309c` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test -p hkdf --doc` | 0 | Alle drie HKDF-README-doctests geslaagd; log `/tmp/nieuw-os-infra-hkdf-docs.log`, SHA-256 `efd0780e571efccc60f94a272c9fdcfa7e538cc3462596ec3e91b36b209aa8b9` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` (na HKDF-fix, vóór HMAC-testdeps) | 101 | HKDF-doctests slagen; de twee HMAC-README-doctests faalden door dezelfde ontbrekende testdependencies; log `/tmp/nieuw-os-infra-root-final.log`, SHA-256 `8721ee2059a3a371641b9338630bb6ccd37a9e92a72dd5f4e64ff5a998712521` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test --offline -p hmac --doc` | 0 | Beide HMAC-README-doctests geslaagd; log `/tmp/nieuw-os-infra-hmac-docs.log`, SHA-256 `22202d7d5fb13699e0d503e617eec02976210f245af5b8766692d89d01859954` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` | 0 | Volledige workspace: 360 passed, 0 failed, 1 ignored over 51 resultaatgroepen; log `/tmp/nieuw-os-infra-root-post-vendor-testdeps.log`, SHA-256 `192ddcaed335916cd18020e4eda41d66c930b5ced2b66b3a50cc808ddcbcee4c` |
| Lokale C-compileer-/linkproef en uitvoering (exacte opdracht hieronder) | 0 | Geslaagd; geen systeembrede installatie |
| `python3 -m py_compile tools/preflight.py tests/test_preflight.py` | 0 | Python-syntaxis geldig |
| `python3 tools/progress.py --write` en `python3 tools/progress.py --check` | 0 | Voortgangsbestanden consistent; 34% blijft ongewijzigd |
| `gh pr checks 62 --repo simonvanieperen/nieuw-os --watch --interval 15` | 0 | Alle 10 workflows (11 jobchecks) groen op PR-head `4fb9a23`; CodeRabbit-status onbekend |

De lokale GCC-driver, libc-devbestanden en runtimebibliotheken zijn alleen
uitgepakt onder de door `.gitignore` uitgesloten `.tools/`-map. De wrapper
gebruikt die bestanden via `tools/dev.py`; de C-smoke hierboven slaagt.
`sudo -n true` eindigde met exit 1 doordat interactieve authenticatie vereist
is; er zijn geen systeempakketten geïnstalleerd.

Exacte succesvolle C-smokeopdracht:

```sh
printf 'int main(void) { return 0; }\n' | LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cc -x c -o /tmp/nieuw-os-cc-test - && LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" /tmp/nieuw-os-cc-test
```

De volledige workspace-test is lokaal geslaagd na herstel van beide
vendormanifesten. Alle GitHub Actions-workflows op `4fb9a23` slagen;
CodeRabbit-status is onbekend. Rust fmt/Clippy zijn niet uitgevoerd. Geen fysieke of
menselijke acceptatiegate of nieuw native QEMU-bewijs is toegevoegd:
**34% → 34%**. De daaropvolgende voortgangs-/statuscheckpoint is alleen
documentatie.

## Conflictintegratie met main f52dab5

30 oktober 2026: PR-head `c87fd769223e95fbf3a9423247468a0230ca9de1` met
actuele `main` `f52dab50af281dab2c94ab2c9e31aaabd299246a`. Alleen
README, PROJECT-STATE en progress.json conflicteerden. De historische
preflight-, B02- en RTC-checkpoints zijn behouden; de actuele assessment
beschrijft de integratie met **34% → 34%**. Criteria en gewichten zijn gelijk.
De lockfile bevat zowel RTC-core als de oorspronkelijke PR-doctestdependencies.
Geen runtimebron handmatig aangepast, geen nieuwe dependency toegevoegd.

De onderstaande controles zijn uitgevoerd vanuit
`/home/runner/work/nieuw-os/nieuw-os` op de samengevoegde werkboom vóór
de twee-oudermergecommit. Waargenomen eindtijd: 16:21 UTC (17:21 Amsterdam).
Tests en buildoutput staan lokaal onder `/tmp/` en genegeerde target/buildmappen;
dit rapport archiveert compacte resultaten en hashes, geen nieuwe VM-acceptatie.

| Exact commando | Exit | Resultaat / lokaal log |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p test_preflight.py -v` | 0 | 22 tests, 0,210 s; `/tmp/pr62-v2-preflight.log` |
| `python3 -B -m unittest tools.tests.test_rtc tools.tests.test_system_clock -v` | 0 | 12 tests, 0,026 s; `/tmp/pr62-v2-clock.log` |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 0 | 46 tests, 9,235 s; `/tmp/pr62-v2-system.log` |
| `python3 -B -m unittest discover -s tools/tests` | 0 | 98 tests, 32,204 s; `/tmp/pr62-v2-tools.log` |
| `cargo +1.90.0 fetch --locked` | 0 | Bestaande locked dependencies lokaal opgehaald; `/tmp/pr62-v2-fetch.log` |
| `cargo +1.90.0 test --offline --locked -p keyring-core` | 0 | 68 tests, inclusief 5 B02-persistent-wiretests; `/tmp/pr62-v2-keyring.log` |
| `cargo +1.90.0 test --offline --locked --workspace` | 0 | 365 passed, 0 failed, 1 ignored; inclusief HKDF/HMAC-doctests; `/tmp/pr62-v2-workspace.log` |
| `cargo +1.90.0 fetch --locked --manifest-path sdk/Cargo.toml` | 0 | SDK-cache hersteld; `/tmp/pr62-v2-sdk-fetch.log` |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 0 | 20 tests; `/tmp/pr62-v2-sdk.log` |
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/rtc/Cargo.toml` | 0 | 11 tests; `/tmp/pr62-v2-rtc.log` |
| `cargo +1.90.0 fmt --all -- --check` | 0 | `/tmp/pr62-v2-root-fmt-final.log` |
| `cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | 0 | `/tmp/pr62-v2-sdk-fmt-final.log` |
| `cargo +1.90.0 clippy --offline --locked -p keyring-core --all-targets -- -D warnings` | 0 | `/tmp/pr62-v2-keyring-clippy.log` |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/rtc/Cargo.toml --all-targets -- -D warnings` | 0 | `/tmp/pr62-v2-rtc-clippy.log` |
| `python3 -B tools/build.py --kernel-feature native-system-clock` | 0 | Loader, kernel en RTC/B02-systeemimages gebouwd/geïnspecteerd; `/tmp/pr62-v2-build-final.log` |
| `python3 tools/progress.py --write` en `python3 tools/progress.py --check` | 0 | Beide geslaagd; 34/100 consistent |
| `git diff --check origin/main` | 0 | Eigen PR-delta zonder witruimtefouten |
| `python3 -B tools/preflight.py --strict` | 2 | Native vereisten ontbreken; `/tmp/pr62-v2-native-preflight.json` |

Eerste formatteringspogingen eindigden met exit 1 doordat Rust 1.90.0 nog
geen rustfmt had (`/tmp/pr62-v2-root-fmt.log`, `/tmp/pr62-v2-sdk-fmt.log`);
de tests zelf waren al geslaagd. De eerste crossbuild eindigde met exit 1
(onderliggende Cargo exit 101) omdat `x86_64-unknown-none` nog ontbrak
(`/tmp/pr62-v2-build.log`). De bewaarde logs leggen de installatie- of
herstelopdracht voor de Rust-toolchain niet vast. `rustup toolchain list`
toont alleen geïnstalleerde toolchains; `rustup component list --toolchain
1.90.0 --installed` bevestigt alleen de aanwezige componenten en targets.
De bovenstaande herhalingen slagen zonder bronwijziging. Geen QEMU/OVMF geïnstalleerd:
alle native runtimegates zijn **niet uitgevoerd**.

SHA-256 van de geslaagde lokale controlelogs:

```text
0b4160ed900c438e622f75b0c8a02e1792c69f3c8b9bb2167f557a0e5001cb00  /tmp/pr62-v2-preflight.log
3930dfdc67982387dadad07f146a56e18d28d450cc0e0ebb67ee0bbbdf219d07  /tmp/pr62-v2-clock.log
7ddd1e59858aa83e31650e64b94c65b33a18188e8c6e99b3eea5eab8c4631c1c  /tmp/pr62-v2-system.log
657a0c217fe0fbab90b096af8c04430d9cd581743d83e6c3f1c03e35797920b6  /tmp/pr62-v2-tools.log
32605d7aefcfb92dabf4e6a640a3b047ae24914ca92bf3f885ddb8fde917b08d  /tmp/pr62-v2-workspace.log
760b1405cddf882872a2c1919d30454b28fa9efbbcb5cac8841e5b53ab11cba0  /tmp/pr62-v2-sdk.log
5e09555bff8a935eed6ad1ae24f67675ce02715f7abb439dd97676204a3e0cf7  /tmp/pr62-v2-keyring.log
e41a8e173bbbd468d1923769ccccdc0570d37f17523000c6ca123f597338ea41  /tmp/pr62-v2-rtc.log
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/pr62-v2-root-fmt-final.log
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/pr62-v2-sdk-fmt-final.log
e8c4426a52f1322a834c926b379934fff325526b2ebcfefd2a4628bbdcc5dbef  /tmp/pr62-v2-keyring-clippy.log
1454bfaefd325bc11249ec98cbba417f8bac01575eec45caf434e045d623a7d3  /tmp/pr62-v2-rtc-clippy.log
95b04ddf0d095db3f94645118766b4141240320cfc61676ce08c8aa25e573049  /tmp/pr62-v2-build-final.log
```

Inhoudsbehoud gecontroleerd: kernel, SDK, keyringruntime en native runners zijn
bytegelijk aan main; preflight, regressietest en beide vendormanifesten zijn
bytegelijk aan de PR-head. Hosted CI van de oude head is via GitHub gecontroleerd:
de laatste CI-run `41069511112` slaagt, zonder gefaalde jobs. Dit is geen
nieuwe-head-CI of volledige productacceptatie. B02's rollbackanker/write-ahead,
RTC-authenticiteit, fysieke hardware en menselijke audit blijven open.

### Historisch publicatiecheckpoint

Publicatie bevestigd: `git ls-remote origin
refs/heads/simonvanieperen-studious-sniffle refs/heads/main` (exit 0) geeft
mergecommit `cc136a3d4a84dc045089ad86f20925ce39e11a72` en basis
`f52dab50af281dab2c94ab2c9e31aaabd299246a`. De merge heeft exact twee ouders:
`c87fd769` en `f52dab5`. Finale `python3 tools/progress.py --check` en
`git diff --check origin/main HEAD` slagen (exit 0); werkboom schoon.
De PR-delta is beperkt tot de oorspronkelijke preflight-/doctestwijzigingen,
assessment/status en dit bewijsrapport; geen gegenereerde binaries toegevoegd.
Secretscan vindt niets. CodeQL Actions/Python vindt nul alerts, maar de
totale analyse krijgt een timeout vóór alle talen gereed zijn; deze gedeeltelijke
uitkomst is geen volledige securitygoedkeuring. Nieuwe-head-CI en menselijke
review blijven afzonderlijk. Dit afsluitende bewijscheckpoint wijzigt alleen
documentatie; PR #62 is niet gemergd.

### PR #62 reviewherstel — 30 oktober 2026

De preflight inventariseert nu `link.exe` op Windows, gebruikt voor de MSVC
linker de gedocumenteerde helpoptie `/?`, en rapporteert ontbrekende linkers
als `missing_host_test_tools`. Een synthetische regressietest controleert
zowel die argumentkeuze als de Windows-foutstatus; de bestaande POSIX-`cc`
negatieve test blijft behouden. Microsoft Learn, “MSVC Linker options”,
MSVC 17.0 (geraadpleegd 30 oktober 2026):
<https://learn.microsoft.com/en-us/cpp/build/reference/linker-options?view=msvc-170>.
De native Windows-uitvoering is **niet uitgevoerd**.

| Exact commando | Exit | Resultaat |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p 'test_preflight.py' -v` | 0 | 24 tests geslaagd; `/tmp/pr62-review-preflight-final.log`, SHA-256 `e3092a33106096d778c6154365052c0b5df9a93c8d7cf06c9fbd7ed4b9af71b6` |
| `python3 -B -m py_compile tools/preflight.py tests/test_preflight.py` | 0 | Python-syntaxis geldig |
| `python3 -B tools/preflight.py --strict-host-tests` | 2 | Verwacht op Linux: `cc` ontbreekt; `/tmp/pr62-review-host-preflight-final.json`, SHA-256 `312f1a1c262a05ea3c175d0a64f648ff9308f8912a0c08d20137a11175c0dc9a` |
| `python3 -B tools/progress.py --write` en `python3 -B tools/progress.py --check` | 0 | Beide geslaagd; score blijft 34/100 |
| `git diff --check` | 0 | Geen whitespacefouten |

De tests verifiëren de Windows-keuze met synthetische probes, niet een
daadwerkelijke Windows-host of volledige OS-acceptatie. Actuele PR-CI,
reviewherhaling en merge blijven afzonderlijke gates.

### Vervolgreview: platformbeleid van de synthetische test

Basis `f6d3a108c8b53debd5a8d5415a46ce0772794020`. De bestaande
POSIX-`cc`-test gebruikte impliciet `HOST_TEST_TOOLS` van de echte host.
Met het Windowsbeleid `('link.exe',)` faalden daardoor twee assertions
(één testcase, exit 1), terwijl de productiepreflight correct alleen de
Windowslinker controleerde.

De testcase kiest nu expliciet `host_test_tools('posix')`, met behoud van
alle assertions en beide strict-scenario's. Geen skip: de synthetische
POSIX-proef blijft ook op een Windows-host uitvoerbaar. De afzonderlijke
Windowsproef blijft haar eigen Windowsbeleid kiezen.

| Exact commando vanuit de PR-worktree | Exit | Resultaat |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p test_preflight.py -v` | 0 | 24 tests geslaagd onder normaal Linux/POSIX-beleid |
| Onderstaand Pythoncommando | 0 | Dezelfde 24 tests geslaagd met synthetisch Windows-hostbeleid |
| `python3 -B tools/progress.py --write` en `python3 -B tools/progress.py --check` | 0 | Score blijft 34/100 |
| `git diff --check` | 0 | Geen whitespacefouten |

```sh
python3 -B -c 'import sys,unittest; sys.path[:0]=["tests","tools"]; import test_preflight; from unittest.mock import patch; suite=unittest.defaultTestLoader.loadTestsFromModule(test_preflight); context=patch.object(test_preflight.preflight,"HOST_TEST_TOOLS",("link.exe",)); context.start(); result=unittest.TextTestRunner(verbosity=2).run(suite); context.stop(); sys.exit(not result.wasSuccessful())'
```

Logs onder
`/home/simon/.copilot/session-state/d7a153299dbd45d7ab72b2596483b441/files/`:
`pr62-windows-policy-before.log` (reproductie vóór herstel, exit 1),
`pr62-platform-policy-normal.log` en `pr62-platform-policy-windows.log`.
De voorafgaande reproductie gebruikte dezelfde policy-patch, maar draaide
uitsluitend `RequiredToolProbes("test_missing_host_linker_is_reported_separately_and_can_be_strict")`.
Geen productiecodewijziging, nieuwe toolinstallatie of Windows-hostuitvoering
voor deze testcase; dit is uitsluitend host-/mockbewijs.
Alle CI-checks op f6d3a108 waren groen, maar nieuwe-head-CI en review blijven
vereist. De vorige bronclaim in PROJECT-STATE is expliciet als historische
integratiebron gelabeld. Geen native-/hardware- of nieuw rubricpunt.
