# Q01 — lokale quantumsimulator

Deze hostproef voert `qsim-v0`-circuits lokaal uit met een eigen statevector-engine. De engine is `no_std + alloc`, bevat geen externe dependencies en verbiedt `unsafe`. Dit is geen OS-port, fysieke quantumprocessor, quantumversnelling of cryptografische implementatie.

## Starten

Vanuit de projectroot, met de project-Rust-toolchain in PATH:

```powershell
cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli
```

Het programma leest één volledig JSON-document tot EOF via stdin en schrijft één JSON-response naar stdout. Voor de Bellfixture in PowerShell:

```powershell
Get-Content -Raw experiments/quantum/fixtures/bell8.json | cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli
```

Voor de projectlokale Windows-toolchain kan deze procesomgeving worden gebruikt:

```powershell
$env:CARGO_HOME = "$env:LOCALAPPDATA\nieuw-os\cargo"
$env:RUSTUP_HOME = "$env:LOCALAPPDATA\nieuw-os\rustup"
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
```

Dit verandert alleen de huidige shell. Een build na de eerste dependency-fetch werkt met `--offline`. Het gebouwde programma staat onder `experiments/quantum/target/debug/qsim-cli.exe` op Windows. Het opent geen bestanden, netwerkpoorten, NAS of cloudendpoint; stdin/stdout vormen de enige gebruikersinterface.

## Protocol en grenzen

Verplichte velden: `schema`, `job_id`, `qubits`, `gates`, `shots`, `seed`. Schema is exact `qsim-v0`; job-ID bevat 1–64 ASCII-letters/cijfers/`_`/`-`; seed bestaat uit precies 16 kleine hextekens. Gates zijn uitsluitend `h`, `x`, `z`, `s` met `target`, of `cx` met `control` én `target`. Onbekende/ontbrekende/dubbele velden, control op enkelvoudige gates, negatieve/fractionele integers, trailing JSON en te diepe/onjuist gevormde data worden afgewezen.

| Middel | Grens |
|---|---:|
| Request | 262.144 bytes; reader stopt na limiet + 1 |
| Qubits | 1–12 |
| Gates | 0–4.096; begrensd tijdens deserialisatie |
| Shots | 1–65.536 |
| Werkbudget | `(gates + 1) × 2^qubits + shots × (qubits + 1) ≤ 20.000.000` |
| Uitvoeringsdeadline CLI | 10 seconden, monotone klok na validatie |
| Testproces incl. I/O | 30 seconden |

Qubit 0 is het minst significante bit; strings tonen `q[n-1]…q[0]`. De output bevat alleen niet-nulle tellingen in lexicografische volgorde. Alle metingen gebeuren aan het einde van het circuit. Er zijn geen reset-, puls-, noise- of tussentijdse metingen.

SplitMix64 verzorgt reproduceerbare sampling: **geen cryptografische RNG en nooit gebruiken voor sleutels of systeementropy**. Bron en toestemming staan in [THIRD-PARTY.md](THIRD-PARTY.md).

Exitstatus: 0 succes; 2 invoer/schema; 3 quota/geheugen; 4 cancellation/deadline; 5 numerieke/interne fout. Foutresponses bevatten vaste veldnamen, nooit ruwe input. Een ongeldig of nog niet betrouwbaar gelezen job-ID wordt `null`.

## Controles en werkelijk bewijs

Voer vanuit de projectroot uit:

```powershell
python experiments/quantum/verify.py
```

Dit script vindt ook de projectlokale Windows-toolchain, installeert niets, gebruikt Cargo offline, controleert formatting, compileert en voert de Rust-tests uit, controleert Clippy zonder toegestane waarschuwingen en compileert de core met target `x86_64-unknown-none`. Daarna voert het de vijf losse fixtures uit, met 30 seconden per proces. Ruwe logs, uitvoer, bron-/binaryhashes en exacte opdrachten komen in `verification/`; die map is een lokaal gegenereerd artifact.

Op 27 oktober 2026 zijn deze controles werkelijk geslaagd met Rust 1.90.0 en de Windows MSVC-hosttoolchain:

- 25 Rust-tests geslaagd; de tests dekken Q1-01 t/m Q1-17 uit de specificatie, deels gegroepeerd. Voor Q1-16 is uitvoering met technisch geblokkeerd netwerk niet getest; de beperktere offline-/dependencycontrole staat hieronder.
- Bell8, seed 0: exact `00:5, 11:3`; Bell4096: exact `00:2082, 11:2014`.
- Identiteit: uitsluitend `000`; X op qubit 1: uitsluitend `10`.
- Maximale job: 12 qubits, 4.096 gates en 65.536 shots uitgevoerd; gemeten totale capaciteit van enginebuffers inclusief gatebuffer **212.992 bytes**. Dit is exclusief input, JSON-parser, allocatoroverhead, stack en hostruntime; geen meting van totaal procesgeheugen.
- Vooraf, tijdens gates en tijdens sampling annuleren geeft een fout zonder gedeeltelijk voltooid resultaat. Deadline wordt deterministisch via `Control` getest.
- Fase, norm, omkeerbare gates en foutieve/NaN/infinite interne amplitudes zijn rechtstreeks getoetst; histogramtests alleen zouden fasefouten missen.
- De contractaudit Q01V herstelde een afrondingsrandgeval: het CDF-eindpunt wordt vanaf de laatste positieve kans op 1 gezet, zodat een trailing nulmassa geen afrondingsrest krijgt. Een vaste unit-test faalt met de vorige implementatie en slaagt na herstel. Een aanvullende test controleert maximaal 256 basisindices en maximaal 64 shots tussen controlepunten afzonderlijk.
- Standalone fixture-uitvoering en offline builds slagen. De dependencyboom bevat geen netwerkclient. De host is hiervoor niet fysiek van het netwerk losgekoppeld en er is geen OS-netwerksandbox ingesteld.

De engine reserveert jobbuffers fallible en geeft `ResourceExhausted` bij een mislukte reservering. De hostadapter en Serde gebruiken daarnaast begrensde reguliere Rust-/parserallocaties; wereldwijd uitgeput hostgeheugen is niet als herstelbaar procesfoutscenario getest. De 10-secondenkern-deadline onderbreekt geen geblokkeerde stdin/stdout. Er is nog geen IPC-cancelobject of eigen-OS floating-point/contextsave-ABI. Deze grenzen horen bij de volgende userspace-integratie, niet bij een kernelport van de simulator.

## Ontwerp en herstel

`ValidatedJob` heeft private velden; targets, shifts, allocatieomvang en werkberekeningen worden voor uitvoering gecontroleerd. De core kent geen klok, omgeving, bestanden of netwerk. De caller levert `Control`; controlepunten volgen vóór allocatie, iedere gate, maximaal 256 basisindices en maximaal 64 shots. Een geannuleerde job levert geen histogram op.

De quantumworkspace is zelfstandig en staat buiten de root-kernelworkspace. Verwijderen van deze experimentmap of terugdraaien van alleen haar toekomstige Git-wijziging verwijdert de proef zonder OS-/NAS-/hostinstellingen te wijzigen. Buildartifacts en checks worden opnieuw gemaakt met bovenstaande opdracht.

## Volgende stap

Laat de code onafhankelijk beoordelen en verbind vervolgens een afgebakende userspace-dienst met IPC-cancellation, een correcte floating-point-context en concrete scheduler-/geheugenquota zodra het eigen OS die ondersteunt. Fysieke QPU-drivers en postkwantumcryptografie blijven aparte taken.
