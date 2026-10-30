# Capabilitydigest: productie kiest de oorspronkelijk gemeten berekening

CAP-DIGEST-TCG-2 herstelt `Engine::state_digest()` naar de volledige
oorspronkelijke berekening uit commit
`3f20f087a7f0ddf4757af8d3ea110dd0c044a72c`. Dit is de productie-keuze na de
gerichte native tegenmeting. Het historische onderzoek en meetprotocol staan
in [CAP-DIAGNOSTIC-DIGEST.md](CAP-DIAGNOSTIC-DIGEST.md); onderstaande keuze
vervangt de eerdere tijdelijke selectie van de nulreeksvariant.

De productiehash verwerkt opnieuw ieder getypeerd woord met dezelfde
`h = (h XOR woord) * 0x100000001b3 modulo 2^64`, vanaf
`0xcbf29ce484222325`. Ieder bericht wordt met de bestaande `Message::encode()`
naar alle 320 bytes geserialiseerd en in 40 little-endian woorden verwerkt.
Alle velden, generaties, ongeldige logische toestanden, ongebruikte queues,
ongebruikte capabilities en payloadbytes buiten `len` blijven deelnemen.
Compilerpadding was geen deel van deze getypeerde stroom en wordt ook nu
niet gelezen. De vaste stroom heeft **74.569 woorden**. Het historische
contract telde deze formule ten onrechte op tot 74.669; dit corrigeert die
documentatiefout, zonder een veld of operatie uit de stroom te verwijderen:

```text
1 + 16*(2 + 256*4) + 1024*12 + 64*(8 + 16*44)
  + 4*9 + 8*7 + 16*9 + 10*6 = 74569
```

Geen cache, dirtyflag, gedeeltelijke hash, selectie op live velden,
rechtenverruiming, foutvolgordewijziging, ABI-wijziging of verwijderde
syscallcontrole. `stats()` blijft dezelfde volledige digest gebruiken.
Alle kernelcontroles vóór en na iedere IPC-operatie blijven verplicht,
ook wanneer de operatie EAGAIN retourneert. De digest is diagnostiek,
geen cryptografische hash, authenticatie of bewijs van botsingsvrijheid.

## Werkelijke meting en beperkte gevolgtrekking

De integrator heeft `native-cap-digest-bench` daadwerkelijk uitgevoerd op
commit `48b51878c03b5b73449f563735c7854b24926fc4`, bronfingerprint v3
`d54262733b9e2aca9b6ec1385d2c4ba7f0087d91cabab96d3f2a722edffca60d`.
Profiel: pc-q35-8.2, qemu64, één CPU, single-thread TCG, 256 MiB RAM,
QEMU 10.0.13 (Debian 1:10.0.13+ds-0+deb13u1). De echte vier-processen-
manifestgraaf bevatte 21 handles, 29 nodes en 9 endpoints. De gebruikersapps
en PIT-timer draaiden tijdens de meting niet. Alle drie varianten kregen
dezelfde Engine en 128 gemeten aanroepen, verdeeld over acht geroteerde
rondes van zestien aanroepen, met vier warmupaanroepen per variant.

| Variant | Som van acht TSC-batches | Kleinste batch | Grootste batch |
| --- | ---: | ---: | ---: |
| Oorspronkelijke referentie | 36.998.312 | 4.203.914 | 5.101.902 |
| Direct getypeerde berichtwoorden | 43.931.758 | 5.070.624 | 6.485.388 |
| Nulreekscompositie | 77.735.944 | 8.745.048 | 10.404.420 |

Alle hashvergelijkingen en volledige voor-/na-Statsvergelijkingen slaagden.
De digest was telkens `0x31b7de1c7a15245b`. Raw QEMU-exit was 33, zonder
timeout; de totale runnerduur was 3,68513638 seconden. De wegwerpdisk had
dezelfde hash vóór en na. Dit rapport is zelf gelezen:

```text
/workspace/nieuw-os-worktrees/root-materiaal/build/cap-bench/cap-bench-report.json
SHA256 8b3619dd4c33c652eb59ef338b68bd64b4f80984acf05d36c82dcbf7d0ad6b3f
kernel SHA256 27c787f021e1e81050b80821907efc2b9fbe7bbf3545caa91961b59fba71eca2
serial SHA256 d6a61a5c557d1e78614d373fba4773a0fcf84d828edd4348cbdc9dfcd56e7145
```

Deze **ongeserialiseerde RDTSC**-verschillen zijn grove TCG-batchdiagnostiek,
inclusief lus-, functiepointer-, black_box- en pariteitsrekenwerk. Zij meten
geen fysieke CPU-cycli of exacte per-calllatentie. De oorspronkelijke
berekening was de snelste van deze drie varianten in deze ene afgebakende
proef. Er volgt geen universele rangorde voor fysieke hardware, KVM, andere
CPU's of andere Engine-toestanden uit. Evenmin is dit GUI-, app-, disk-,
entropy-, bootlatentie-, beveiligings- of releaseacceptatie.

De eerdere hostratio van circa 2,48 voor de nulreeksvariant voorspelde dit
resultaat niet. De geïntegreerde GUI-run op
`1fd879dfdde88890a68f0a23901c69acd464b582` faalde met raw exit 35 bij de
gastdeadline: één volledig frame en 1.932 tegels. De eerste capture met
1.608 tegels verscheen volgens de integrator na circa 170,07 seconden,
tegenover circa 106 seconden eerder. De exacte rode run en haar grenzen
staan in het historische contract. Er wordt geen native optimalisatiewinst
uit die hostmeting geclaimd.

## Diagnostische alternatieven blijven controleerbaar

De opt-in cratefeature `diagnostic-bench` behoudt:

- `diagnostic_reference_digest()`: de letterlijk behouden oude referentie.
- `diagnostic_direct_digest()`: alle getypeerde woorden, zonder nuldetectie.
- `diagnostic_zero_digest()`: de volledige eerdere nulreeksberekening.

De nulreeksberekening is ongewijzigd verplaatst uit de productie-methode.
Alle vijf bestaande differentiële testgroepen vergelijken voortaan ook
deze expliciete nulreeksvariant met de referentie en productiehash. De
wirebittests controleren daarnaast de directe berichtberekening. De
private-corruptietests blijven ook inactive, dead, ongeldige indices,
niet-nul trailing bytes en ongebruikte capabilities onderzoeken.
De manifesttest vergelijkt de volledige Engine én Stats vóór en na.

Zonder de feature en buiten `cfg(test)` worden de referentie-, direct- en
nulreeksalternatieven niet gebouwd. De productie-rlib wordt hierop met
symbolinspectie gecontroleerd. De expliciet gestarte historische
hostmicrobenchmark vergelijkt nog steeds referentie met nulreeks, via
`diagnostic_zero_digest()`; hij wordt geen zinloze referentie-tegen-referentie-
vergelijking doordat de productie-keuze verandert.

De integrator moet de ZERO-functiepointer in het bestaande native meetharnas
laten wijzen naar `Engine::diagnostic_zero_digest()`. De naam ZERO mag na
deze productie-wijziging niet meer `Engine::state_digest` aanduiden.
Deze componentcommit wijzigt geen kernelhook, modulewiring, runner of
GUI-waarnemer; die bestanden blijven eigendom van de integrator.

## Componentcontrole en hervatting

De productie-`state_digest`-body wordt letterlijk vergeleken met de bron
uit 3f20; SHA256 van de gelijke body is
`29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed`.
De body van `stats()` blijft gelijk aan 952e9ace, evenals de verplaatste
nulreeksbody. Dit bewaakt de exacte stroom; tests bewijzen op zichzelf geen
botsingsvrijheid of volledige veiligheid.

Eigen logs staan onder `build/cap-reference-checks/`. Reproduceerbare controles
met Rust 1.90.0, offline/locked, eigen targetmap:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test --offline --locked -p capability-core
cargo +1.90.0 test --offline --locked -p capability-core --features diagnostic-bench
cargo +1.90.0 clippy --offline --locked -p capability-core --all-targets -- -D warnings
cargo +1.90.0 clippy --offline --locked -p capability-core --features diagnostic-bench --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --release -p capability-core --target x86_64-unknown-none
cargo +1.90.0 build --offline --locked --release -p capability-core --features diagnostic-bench --target x86_64-unknown-none
```

Beide scalar-none-configuraties krijgen ook Clippy met `-D warnings`.
De scalarflags zijn code-model=kernel, relocation-model=static, no-redzone,
SSE/SSE2/SSE3/SSSE3/SSE4.1/SSE4.2/AVX/AVX2 uit en expliciete padremap.
Alle 47 oorspronkelijke coretests blijven deel van beide testsets.
Uitgevoerd op de bevroren componentbron: **52 standaardtests en 53 tests met
diagnostic-bench geslaagd**, telkens nul mislukt en één expliciete
hostmicrobenchmark ignored. Alle negen bovengenoemde fmt-, test-, Clippy-
en buildcontroles hadden exitstatus 0. De productie-rlib bevatte geen
DiagnosticHasher, reference-, direct- of zero-alternatiefsymbolen.

```text
componentbron v3 ea2db2786e74508e84e42d7166194869ce550ee01243a1e4b564b92112e9801d
lib.rs SHA256 932f48e14bfb9cc545064a5d5d6dd23e7b87f500c58f90d65745dfe11f391da5
default none-rlib SHA256 cfdc9f5aa6db892f6726973f281f97e8e5413741c00f3e332d79daac7408104e
diagnostic none-rlib SHA256 2cbe91f0bee7ba2da82fab4bf1218d9c608369c4cf15b753ed3c4efa5f496bca
build/cap-reference-checks/report.json SHA256 e050cc6b3b2735bebb1745a5179c8f4058f80abf38258abc0e2e7e97f00b7015
```

De fingerprint betreft deze eigen componentworktree,
niet roots volgende geïntegreerde kandidaat. Een read-only bronreview door
een tweede agent bevestigde de letterlijke bodies en cfg-afscherming zonder
concrete blocker; dat is geen onafhankelijke menselijke audit of native proef.

De geïntegreerde native GUI en relevante regressies moeten op de nieuwe
exacte commit en image opnieuw worden uitgevoerd. Deze productie-keuze
tekent geen GUI-criterium af en verhoogt geen releasepercentage.
