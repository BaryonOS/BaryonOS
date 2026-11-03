# Kleine keyring-herstelautorisatieslice

Basis: `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`.
Productcommit: `da6e89a0`, geporteerd uit bestaande General-code op
`707a768c`; geen nieuwe auth/session-UI of concurrerende driverimplementatie.

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
  `admin_ingress`/`poll_admin` controleren manifestbinding, afzender en
  capabilitydelegatie voordat beheerdispatch wordt aangeroepen.
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

Rollback: de afzonderlijke acht code-/test-/specificatiebestanden samen
terugnemen. Geen bron-WIP van andere owners overgenomen; geen mainmerge,
keys, nieuwe dependency of scorepunten.
