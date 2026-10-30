# Hosttest-preflight: POSIX C-linker

Dit rapport registreert de Linux-hosttesttooling voor de taakbasis
`5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`. De preflightwijziging is eerst
getest op broncommit `80393b67950c393dfa37336e66df5346ae6a37d7`; de huidige
kandidaat is `dfc64cdad130b5d0aa9deff4362844829d8bf514`.
Dit is geen OS-, kernel-, boot- of QEMU-acceptatie.

`tools/preflight.py --strict` blijft bootvereisten controleren. De aparte
`--strict-host-tests`-modus meldt ontbrekende POSIX-hostlinkers. Na het
beschikbaar maken van een lokale compiler onthulde dezelfde volledige
workspace-test een tweede concrete fout: de README-doctests van de vendored
HKDF-crate misten `sha2` en `hex-literal` in de testdependencies. Die
test-only dependencies zijn nu hersteld op basis van de meegeleverde upstream
`Cargo.toml.orig`; ze wijzigen de runtime-dependencies niet.

| Exacte opdracht | Exit | Uitkomst |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p 'test_preflight.py' -v` | 0 | 22 tests geslaagd; log `/tmp/nieuw-os-infra-preflight-tests.log`, SHA-256 `0d7560f39d2c8324034ba8ec19f61a33afaed1c1f794af039e729fc405aaab8e` |
| `python3 -B tools/preflight.py --strict-host-tests` | 2 | Verwachte melding `missing_host_test_tools: ["cc"]`; JSON `/tmp/nieuw-os-infra-preflight.json`, SHA-256 `a69b5f95f05983848f34fecf92a6a840dda685ef095e1eab3fb65c129c81aa3d` |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` (voor lokale `cc`) | 101 | Stopte bij `linker cc not found`; historisch log `/tmp/nieuw-os-infra-root-final.log`, SHA-256 `95e4d7518ea42c507b3d6bb92242cfa2c95ac58ded3890323baf750516cef1b8` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` (vóór HKDF-testdeps) | 101 | Compiler werkte; drie README-doctests faalden omdat `sha2`/`hex_literal` ontbraken; log `/tmp/1793368389455-copilot-tool-output-7287-5b317826-ac72-468d-beb7-f558d9d8462e.txt`, SHA-256 `59469300748959f1a3a664fd4c232ceb4f7684ddf09206748fb90332936c309c` |
| `LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cargo +1.90.0 test -p hkdf --doc` | 0 | Alle drie HKDF-README-doctests geslaagd; log `/tmp/nieuw-os-infra-hkdf-docs.log`, SHA-256 `efd0780e571efccc60f94a272c9fdcfa7e538cc3462596ec3e91b36b209aa8b9` |
| Lokale C-compileer-/linkproef en uitvoering (exacte opdracht hieronder) | 0 | Geslaagd; geen systeembrede installatie |
| `python3 -m py_compile tools/preflight.py tests/test_preflight.py` | 0 | Python-syntaxis geldig |
| `python3 tools/progress.py --write` en `python3 tools/progress.py --check` | 0 | Voortgangsbestanden consistent; 34% blijft ongewijzigd |
| `gh pr checks 62 --repo simonvanieperen/nieuw-os --watch --interval 15` | 0 | Alle 10 workflows (11 jobchecks) groen op oudere PR-head `00f91e7`; nieuwe kandidaat moet opnieuw door CI |

De lokale GCC-driver, libc-devbestanden en runtimebibliotheken zijn alleen
uitgepakt onder de door `.gitignore` uitgesloten `.tools/`-map. De wrapper
gebruikt die bestanden via `tools/dev.py`; de C-smoke hierboven slaagt.
`sudo -n true` eindigde met exit 1 doordat interactieve authenticatie vereist
is; er zijn geen systeempakketten geïnstalleerd.

Exacte succesvolle C-smokeopdracht:

```sh
printf 'int main(void) { return 0; }\n' | LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" python3 -B tools/dev.py cc -x c -o /tmp/nieuw-os-cc-test - && LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" /tmp/nieuw-os-cc-test
```

**Nog open:** de volledige `cargo +1.90.0 test --offline --locked --workspace`
is na toevoeging van de HKDF-doctestdependencies nog niet opnieuw uitgevoerd.
Dit is de eerstvolgende controle. Rust fmt/Clippy zijn niet uitgevoerd. De
eerder groene QEMU-/native CI-checks op `00f91e7` vervangen geen CI op de nieuwe
kandidaat en geen fysieke of menselijke acceptatiegate. Geen nieuw
productcriterium of native QEMU-bewijs is toegevoegd: **34% → 34%**.
