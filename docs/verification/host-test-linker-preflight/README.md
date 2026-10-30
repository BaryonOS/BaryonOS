# Hosttest-preflight: ontbrekende C-linker

Dit rapport registreert alleen de Linux-hosttesttooling op broncommit
`80393b67950c393dfa37336e66df5346ae6a37d7` (basis `5c334f9`). Het is geen
OS-, kernel-, boot- of QEMU-acceptatie. De bronvingerafdruk uit
`tools.common.source_hash()` is
`882df4d24a551795464ba80142b76495dbbc81e4d62a3d0f03a39de92eb4d731`.

De oorzaak is een ontbrekende hostlinker: de root-workspace Rust-tests stoppen
bij `linker cc not found`. `tools/preflight.py --strict` blijft bootvereisten
controleren; de nieuwe `--strict-host-tests`-modus controleert afzonderlijk de
POSIX-hostlinker. De melding benoemt de blokkade, maar installeert geen tools.

| Exacte opdracht | Exit | Uitkomst |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p 'test_preflight.py' -v` | 0 | 22 tests geslaagd; log `/tmp/nieuw-os-infra-preflight-tests.log`, SHA-256 `0d7560f39d2c8324034ba8ec19f61a33afaed1c1f794af039e729fc405aaab8e` |
| `python3 -B tools/preflight.py --strict-host-tests` | 2 | Verwachte blokkademelding `missing_host_test_tools: ["cc"]`; JSON `/tmp/nieuw-os-infra-preflight.json`, SHA-256 `a69b5f95f05983848f34fecf92a6a840dda685ef095e1eab3fb65c129c81aa3d` |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace` | 101 | Niet geslaagd: linker `cc` ontbreekt; log `/tmp/nieuw-os-infra-root-final.log`, SHA-256 `95e4d7518ea42c507b3d6bb92242cfa2c95ac58ded3890323baf750516cef1b8` |
| `git diff --check` | 0 | Geen diff-whitespacefouten |
| `python3 -m py_compile tools/preflight.py tests/test_preflight.py` | 0 | Python-syntaxis geldig |

Rustup en Rust 1.90.0 zijn voor deze controle geïnstalleerd in de door
`.gitignore` uitgesloten `.tools/`-toolmap; `cargo fetch --locked` eindigde
ook met exit 0. De lokale `cc`-installatie is niet hersteld: `sudo -n true`
eindigde met exit 1 doordat interactieve authenticatie vereist is. Na installatie
van een C-toolchain is de exacte root-workspace-test opnieuw uit te voeren.
Clippy, rustfmt, de volledige CI en QEMU zijn niet uitgevoerd. Een oudere
HKDF-doctestmelding is op deze bron niet bevestigd, omdat de huidige run al
bij de ontbrekende linker stopte.
