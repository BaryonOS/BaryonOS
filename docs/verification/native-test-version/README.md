# Native testversie — lopende integratie, nog geen releaseacceptatie

Basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`, branch
`simon/native-test-version`; eerste componentintegratie `8becc511a08be128ce0db417f15efa180bfd2eb3`.
De bestaande IPC-WAIT-1- en inputconsumentenbron is overgenomen en aangepast
aan de inmiddels gereserveerde RTC-ABI 1.9: wachten gebruikt ABI 1.10.
Doel is de werkelijk rode native Materiaal-gebruikersketen te herstellen,
niet de 20.000-PIT-tickgrens of acceptatievoorwaarden te versoepelen.

## Eerste hostcontrole

Voor elk Cargo-commando:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_DIR="$PWD/target/native-wait-checks"
```

| Exact commando | Resultaat |
| --- | --- |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p capability-core` | Exit 0: 57 coretests, 7 tests van echte kernel-wachtdecoder/evaluator, 1 bestaande benchmark ignored |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | Exit 0: 23 tests, inclusief ABI 1.10 en weigering van onbekende minors |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml` | Exit 0: 23 tests |
| `python3 -B -m unittest tools.tests.test_ipc_wait` | Exit 0: 2 rapportverifier-tests |
| `python3 -B tools/preflight.py --strict` | Exit 0: Rust 1.90.0, beide targets, QEMU 10.2.1 en OVMF aanwezig; geen build- of bootbewijs |

Logs staan lokaal in `/tmp/nieuw-os-native-wait-{core,sdk,input,verifier}.log`.
De eerste formattingpoging faalde doordat rustfmt ontbrak. Na expliciete
toolinstallatie slaagden `cargo +1.90.0 fmt --all`,
`cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all` en
`cargo +1.90.0 fmt --manifest-path sdk/examples/input-service/Cargo.toml --all`,
ieder via `python3 -B tools/dev.py`, exit 0.

De lokale QEMU/OVMF-pakketten komen uit officiële Ubuntu-pakketten,
zonder sudo of systeeminstallatie. Genegeerd herkomstrapport:
`.tools/nieuw-os/env-native-provenance.json`. QEMU 10.2.1
(`1:10.2.1+ds-1ubuntu3.3`), OVMF `2025.11-3ubuntu7.3`:

| Input | SHA-256 |
| --- | --- |
| QEMU implementatie | `fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb` |
| OVMF CODE | `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7` |
| OVMF VARS-template | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |

## Open bewijsgrens

De twee daadwerkelijke GUI-pogingen blijven rood, met ongewijzigd
20.000-PIT-tickbudget: zonder wachten 202,396 s, met alleen inputwachten
202,363 s; beide drie echte frames, 14 getypte bytes, nog geen save.
Zie `gui-attempts.json` voor bron-/loghashes. De volgende versie voegt
alleen quiescente servicewachten toe; 100 keyring/volume-hosttests en
21 desktop-init-tests slagen, plus native Clippy en de samengestelde
ELF/kernelbuild. Die runtime is nog niet uitgevoerd.

De native wachtbuild en runtime zijn inmiddels **geslaagd**:
het exacte commando en compacte hashbinding staan in `wait-evidence.json`.
Raw 33, exit 0; rolmaskers 63/60, een kernel-gecontroleerde volle queue
na intrekking, 14 blokkeringen, 7 ready-wakes, 6 deadline-wakes,
1 intrekkingswake en 199 werkelijk getelde PIT-idle-interrupts.
De eerste echte run faalde met raw 35 door audit van mappings van een reeds
overleden eerder processlot. Die audit slaat nu uitsluitend Dead slots over.
Een tweede raw-33-run bleef host-rood omdat de generieke runner de nieuwe
poort zonder M2-marker als M1-only behandelde; M2 is nu expliciet vereist.
De derde volledige build/run slaagt. Dit is geen testversoepeling.
Een daaropvolgende gerichte AI-review vond een mogelijke extra PIT-tick
vóór foutwake als de selectiescan een ongeldig userreturnframe opruimt.
De selectielus herstart nu bij haar waitrefresh na die cleanup; hertoetsing
van de laatste correctie volgt. Deze review is geen menselijke audit.

**Niet uitgevoerd op deze integratie:** native kill/hergebruik en wachten
met FPU-context, GUI-gebruikersketen, bredere native regressies, fysieke
hardware en menselijke gebruiks-/securityacceptatie. De CPL3-probe is inmiddels als echt scalair ELF gebouwd; de gezamenlijke
kernelbuild en runtime moeten nog volgen. Probe-hosttests: 3/3, exit 0.
Gerichte Clippy voor capability-core, SDK-runtime en inputconsumenten:
ieder exit 0 met `--all-targets -- -D warnings`, via dezelfde lokale
Cargo-omgeving. Die controle miste aanvankelijk de Clippy-component;
na `python3 -B tools/dev.py rustup component add clippy --toolchain 1.90.0`
zijn de drie controles daadwerkelijk herhaald.

`python3 -B -m unittest discover -s tests` slaagt met 414 tests (4,719 s);
`python3 -B -m unittest discover -s tools/tests` met 100 tests (3,496 s).
Deze zijn hostharnastests; hun gesimuleerde transcriptuitvoer is geen nieuwe
native VM-uitvoering. Logs: `/tmp/nieuw-os-native-test-{python,tools}.log`.
De score blijft **34% → 34%**; het gevraagde doel van minstens 50% is
niet bereikt. Een hosttest, geïntegreerde component of extra PASS-teller
sluit geen volledig rubriccriterium.
