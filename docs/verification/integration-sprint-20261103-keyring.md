# Kleine keyring-herstelautorisatieslice

Basis: `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`.
Productcommit: `da6e89a0`, geporteerd uit bestaande General-code op
`707a768c`. De [volledige wijzigingsscope van base `ab85d633` naar
reviewhead `5caf7ee2`](https://github.com/simonvanieperen/nieuw-os/compare/ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff...5caf7ee23d16a9c9fd6d4ccca70bc290b99c132c)
bevat geen nieuwe auth/session-UI of concurrerende driverimplementatie.
Deze uitspraak geldt uitsluitend voor deze gepinde diff, niet voor latere branches.

Primaire herkomstbronnen, organisatie Nieuw OS-project, gepinde versies:

- [Herkenbare native sessiepreview en snellere QEMU-schermopbouw (#121)](https://github.com/simonvanieperen/nieuw-os/commit/a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e):
  commit `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`, basis van de extractie.
- [coordination: activeer SMP na ontvangen OOM-overdracht](https://github.com/simonvanieperen/nieuw-os/commit/707a768c1df81e674a15c4aa013f44b85b90c700):
  commit `707a768c1df81e674a15c4aa013f44b85b90c700`, historische General-boom
  waaruit de bestaande keyringwijziging is geporteerd, niet de geteste
  integratierevisie.

Gewone app-ingress weigert EXPORT_RECOVERY en RESTORE met een leeg,
gezeroiseerd antwoord. Alleen afzonderlijke manifest-gebonden beheer-ingress
mag herstelmateriaal behandelen. De native adapter controleert de afzender;
de gedeelde app-dispatcher krijgt geen beheerautoriteit.

Primaire implementatiebronnen (organisatie: Nieuw OS-project; versie
`da6e89a0`, 3 november 2026):

- [App- en beheerdispatchers, `services/keyring/src/wire.rs`](https://github.com/simonvanieperen/nieuw-os/blob/da6e89a0/services/keyring/src/wire.rs#L334):
  `dispatch_domain` sluit herstel uit; `dispatch_management` behandelt de
  afzonderlijke beheeroperaties. De dispatcher zelf authenticeert geen caller.
- [Native beheeradapter, `sdk/examples/keyring/src/service.rs`](https://github.com/simonvanieperen/nieuw-os/blob/da6e89a0/sdk/examples/keyring/src/service.rs#L232):
  `admin_ingress` controleert de manifestbinding. `poll_admin` weigert
  berichten met capabilities en controleert daarna afzender, tag en header
  voordat beheerdispatch wordt aangeroepen.
- [Native resetfixture, `kernel/src/m2/m3/keyring.rs`](https://github.com/simonvanieperen/nieuw-os/blob/da6e89a0/kernel/src/m2/m3/keyring.rs#L529):
  `restart_fixture` reset en herinitialiseert het entropydevice, behoudt
  gepinde resources en controleert een nieuwe 32-byte transactie. Geen
  proces wordt daarvoor afgesloten of opnieuw gestart.

## Uitgevoerd

Rust 1.90.0, bestaande C-toolchain, QEMU/OVMF en wegwerpimages.
Commando's vanuit de geisoleerde worktree `files/sprint-keyring`:

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path services/keyring/Cargo.toml` | Exit 0: 70 tests, waaronder positieve beheerexport/-restore en geweigerde app-/persistence-opcodes. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path services/keyring/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 fmt --manifest-path services/keyring/Cargo.toml -- --check` | Exit 0. |
| `python3 -B -u tools/test_keyring.py` | Eerste run exit 1: SDK-README gewijzigd tijdens de run; bronfreezecontrole wees het resultaat terecht af. Geen groen bewijs. |
| `python3 -B -u tools/test_keyring.py` | Volledige herhaling op bevroren bron: exit 0, 15,10 s; alle hostsubgates, vijf profielbuilds, vijf QEMU-cases en afsluitende normale build groen. |
| `git diff --check` | Exit 0. |

QEMU-cases: normal, no-device, stuck, timeout, restart; alle vijf
`passed=true`, ruwe QEMU-exit 33. Native bewijs betreft onder andere de
weigering van app-herstelverzoeken, entropyfailures en deviceherinitialisatie
na bevestigde reset. Dit is geen bewijs van een procesherstart.
Positieve managementrecovery is uitsluitend hostgetest; geen native
supervisorherstel-, hardware-, gebruikers- of releaseclaim. Deze wijziging
sluit B01/B02 niet op basis van hostbewijs.

Bevroren bronvingerafdruk v3:
`7cfa0356b3b9c4d3ea5cd00452146c2623eb30883558b110ef41863aa803afa0`.
Volledig lokaal gateverslag: `build/keyring-gate.json`.

| Log onder `/tmp/` | SHA-256 |
|---|---|
| `nieuw-os-sprint-keyring-host.log` | `d3b60d5296927dc3306568de6408a56e40824a8bf921c024d32ad449d5099044` |
| `nieuw-os-sprint-keyring-clippy.log` | `11f5568061b8438dfa19e4efe453866959a817c751098ff94e3f6bb615c620c6` |
| `nieuw-os-sprint-keyring-native.log` (afgewezen run) | `816e404ae07e6d42ff42a5c72d7c8ca91e40f31a2366b4d5c11fed05062f4461` |
| `nieuw-os-sprint-keyring-native-frozen.log` | `b3aa2b0dd961aab2c01f3264c4eaf23d908c0b5ed5e7ab6fca088a8ab5898872` |

## Testbewijs op gezamenlijke main78ff7c03

Nieuw OS-project, 3 november 2026:
[integratiebron 3639ecaa](https://github.com/simonvanieperen/nieuw-os/tree/3639ecaac8ce10bcf4f29581a969b8db7e7cedb9),
met de inmiddels werkelijk gemergde imageslice en B03 TEST-policy.
De tests draaiden op deze mergecommit `3639ecaa`, niet rechtstreeks op
[mainouder 78ff7c03 — fix(updates): vereis expliciete TEST-ondertekenpolicy (#158)](https://github.com/simonvanieperen/nieuw-os/commit/78ff7c03b015ad16fe665ae7df9fe239d078e08d).
Keyringcode bytegelijk de eerder geteste productbron; geen nieuwe rechten.
Vanuit de General-rootworktree werkelijk opnieuw uitgevoerd:

- `cargo +1.90.0 test --offline --locked --manifest-path services/keyring/Cargo.toml`:
  exit 0, 70 tests.
- `python3 -B -m unittest discover -s tests -p test_update_harness.py`:
  exit 0, 14 tests.
- `python3 -B -u tools/test_keyring.py`: exit 0, 24,83 s totale wandtijd,
  alle hostsubgates, vijf profielbuilds/QEMU-cases en normale afsluitbuild groen.

Omgeving: `PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`,
`CC=/tmp/nieuw-os-c-toolchain/bin/cc`,
`LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu`,
`OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd`,
`OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd`,
`NOS_QEMU_MACHINE=pc-q35-8.2`,
`CARGO_TARGET_DIR="$PWD/target/sprint-keyring-integration"`.
De twee gerichte hostcommando's gebruikten alleen het Rust-/C-pad en dezelfde targetmap.

Vingerafdruk v3:
`f31d0b9fdb7555846fe49a4fd6088a5795dbb792030f6d7d3c852e285850b932`.
`normal`, `no-device`, `stuck`, `timeout`, `restart` allemaal
`passed=true`, ruwe QEMU-exit 33; gate `status=completed`, `passed=true`.
In normal/restart beide app-hersteloperaties elk eenmaal geweigerd.
Dit vervangt geen native positieve manifest-bevoegde beheerrecovery:
B01 blijft open en de score blijft 39, niet 41.

| Lokaal artifact | SHA-256, gecontroleerd |
|---|---|
| `/tmp/nieuw-os-keyring-main78-targeted.log` | `354eaf1c2c9b8ecf1a7cedf906418dd516cf59a415831a05fe09f8c242744e5c` |
| `/tmp/nieuw-os-keyring-main78-native.log` | `ca025292e3ea04e3b8e9d47012eb627ac07287b8ab6983f421b4b2d40e65d82b` |
| `build/keyring-gate.json` | `f3e60b93df7384b9c71147710f9e77a5d24d2745a4102b825f2501da18f7be85` |

Ruwe artifacts blijven lokaal; hashes alleen geven geen publieke
inhoudscontrole. Geen hardware-, menselijke audit- of releaseacceptatie.

Rollback: de afzonderlijke acht code-/test-/specificatiebestanden samen
terugnemen. Geen bron-WIP van andere owners overgenomen; geen mainmerge,
keys, nieuwe dependency of scorepunten.
