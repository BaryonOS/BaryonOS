# Native Materiaal — app-lokale semantische snapshot

Status: beperkte implementatiestap, geen aftekening van native toegankelijkheid.

## Bron en wijziging

- Basiscommit: `f52dab50af281dab2c94ab2c9e31aaabd299246a`
- Branch: `simonvanieperen-didactic-spork`
- Bronfingerprint v3 vóór documentatie/checkpointupdates:
  `d9e3c57244904e20bf52f95b2b7d80e3d2d93e22a303b86f179e583f53e80185`
- `App::semantics` geeft uitsluitend de huidige semantische view van de eigen
  app terug. Het deelt niets met een ander proces en verleent geen rechten.
- De bestaande `complete_chain`-integratietest controleert kop, focus,
  documentwaarde, bewerkbaarheid, opgeslagen status, heropenen en atomische
  weigering van een te kleine buffer.

## Uitgevoerde controles

| Exact commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --tests` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `python3 -B -m unittest tools.tests.test_materiaal_probe tools.tests.test_materiaal_host_scenarios tools.tests.test_materiaal_host_count` | exit 0; 27 tests |
| `LOCALAPPDATA="$PWD/.tools" python3 -B sdk/examples/materiaal/build.py` | exit 0; echte `x86_64-unknown-none`-ELF gebouwd |
| `sha256sum build/sdk-materiaal/x86_64-unknown-none/release/materiaal-app` | `cb1f73a07e18d35d77db0b0056d4cf4702ef64a9d59b61484a1e9c2ff1d3090c` |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` | exit 101; hostlinker `cc` niet gevonden; tests zijn niet uitgevoerd |
| `python3 -B tools/preflight.py --strict` | exit 2; QEMU en OVMF ontbreken; inventory bewijst geen build of boot |

## Bewijsgrens en open werk

De ELF-build, Rust-typecheck, lint en Python-harnas vervangen geen uitvoering
van de app of de Rust-integratietests. Er is geen native QMP-bediening,
schermlezer-/spraakuitvoer, toetsenbord-only gebruikersproef, instellingen
voor schaal/contrast/reduced motion, menselijke gebruikerstest of fysieke
hardwaretest uitgevoerd. Deze wijziging sluit geen U01–U06-criterium en
verhoogt de vaste score niet. Zie `docs/PROJECT-STATE.md` voor de eerstvolgende
hervatstap.
