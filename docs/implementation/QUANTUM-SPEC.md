# Q0 afgerond: bouwspecificatie voor lokale quantumuitvoering
Onderzoeksdatum: 26 oktober 2026. Status: **gekozen specificatie voor Q1**, gereed voor implementatie. Dit document sluit de Q0-keuzevragen voor de eerste simulator af. Heropen deze keuzes alleen bij een aangetoond probleem, niet met een nieuwe algemene onderzoeksronde.

## 1. Vastgelegde keuzes
| Onderdeel | Besluit |
|---|---|
| Eerste resultaat | Een lokale commandoregel-simulator op de ontwikkelhost, plus een herbruikbare rekenbibliotheek |
| Implementatie | Safe Rust; eigen kleine statevector-engine; geen `unsafe`, Python-runtime, GPU-framework of externe quantumdienst |
| Bibliotheek | `qsim-core`: `#![no_std]` met `alloc`; geen bestanden, netwerk, klokken of omgevingstoegang |
| Hostprogramma | `qsim-cli`: `std`, één JSON-request via stdin, één JSON-response via stdout |
| Formaat | Eigen klein **jobprotocol qsim-v0**; geen nieuwe algemene programmeertaal en geen OpenQASM/QIR-compatibiliteitsclaim |
| Gates | H, X, Z, S, CX; initialisatie altijd `|0…0⟩`; uitsluitend volledige eindmeting |
| Niet in Q1 | Mid-circuit measurement, reset, klassieke conditionals, vrije matrixinvoer, pulsen, noise-model, hardwarebesturing |
| Rekenen | Complexe amplitudes met twee `f64`-velden; geen fast-math of hardwareafhankelijke versnellingspaden |
| Einddoel | Dezelfde engine achter een userspace-dienst op het eigen OS, later verwisselbare lokale QPU-backend |
| Privacy | Geen netwerkcode, cloudcredentials, telemetrie, modeldownload of NAS-toegang in simulator/runtime |

OpenQASM omvat veel meer taal- en timingfuncties en definieert niet zelf de volledige uitvoeromgeving. QIR richt zich op interoperabele hybride programma's. Beide zijn toekomstige adaptermogelijkheden, maar zouden de kleine eerste proef onnodig verbreden (OpenQASM Contributors, z.d.; QIR Alliance, z.d.). Het gekozen JSON-jobprotocol is een controleerbare interne grens, geen concurrent van die standaarden.

Q1 draait eerst als hostproces. Dat mag op de ontwikkelcomputer gebeuren; het nieuwe OS erft daardoor geen hostkernel. Een echte eigen-OS-port en een echte QPU-run blijven afzonderlijke resultaten.

## 2. Grenzen vóór allocatie en uitvoering
Alle onderstaande maxima zijn gekozen projectbudgetten, geen hardwareclaims.

| Veld/middel | Exacte grens |
|---|---|
| UTF-8-request | Maximaal 262.144 bytes; stop lezen na limiet + 1 |
| Schema | Exact `qsim-v0` |
| `job_id` | 1–64 ASCII-tekens uit `A-Z a-z 0-9 _ -` |
| `qubits` | Integer 1–12 |
| `gates` | Array 0–4.096 elementen |
| `shots` | Integer 1–65.536 |
| `seed` | Verplicht: precies 16 kleine hexadecimale tekens; unsigned 64-bit simulatorseed |
| Werkbudget | `(gate_count + 1) * (1 << qubits) + shots * (qubits + 1) <= 20_000_000`; alle berekeningen checked |
| Gelijktijdigheid Q1 | Eén job per CLI-proces; geen interne wachtrij of workerpool |
| Wachtlimiet testharness | 30 seconden voor één testproces; echte benchmarkresultaten bepalen later of aanpassing nodig is |

JSON wordt sterk getypeerd gedeserialiseerd. Onbekende velden, ontbrekende velden, dubbele sleutels, onbekende gatevarianten, negatieve/fractionele integers, onbekende schema's en trailing data worden afgewezen. Gebruik `deny_unknown_fields`, geen `flatten`, geen ongetypeerde tussen-`Value` die dubbele sleutels kan verbergen. Begrens de gates-array tijdens deserialisatie tot 4.096; niet eerst een onbeperkte `Vec` vullen. Behoud de parserrecursielimiet en voeg een diepteproef toe. Geen bestandsnamen, shellcommando's of URL's in het request.

Bij 12 qubits heeft de amplitudevector 4.096 elementen × 16 bytes = 65.536 bytes. CDF en tellerarray vragen respectievelijk 32.768 en 16.384 bytes: samen 114.688 bytes, exclusief gatebuffer, input, allocator en hostruntime. Streef naar maximaal 512 KiB voor door de engine bezeten jobbuffers. Meet dit; presenteer het niet als totaal process-RAM. Reserveer begrensde buffers met fallible allocatie en geef `RESOURCE_EXHAUSTED` bij falen; geen paniekpad voor onbetrouwbare invoer.

## 3. Rekenkundig contract
Amplitudes staan in oplopende basisindex. Qubit `q` is bit `q` van de index; qubit 0 is het minst significante bit. Een resultaatstring toont `q[n-1] … q[0]`. Deze keuze sluit aan bij de beschreven Qiskit-indexconventie, maar vereist geen Qiskit-dependency (IBM, z.d.).

`Complex { re: f64, im: f64 }`; beginvector: amplitude 0 = (1, 0), alle andere = (0, 0). Geen door gebruiker aangeleverde beginamplitudes.

Voor elke gate op `q`, neem paren `i` met bit `q = 0` en `j = i | (1 << q)`:
- X: wissel `a[i]` en `a[j]`.
- H: nieuwe waarden zijn `(a[i]+a[j])/√2` en `(a[i]-a[j])/√2`. Gebruik `core::f64::consts::FRAC_1_SQRT_2`; bereken beide uit de oude waarden.
- Z: vermenigvuldig alleen `a[j]` met −1.
- S: vermenigvuldig alleen `a[j]` met `i`: `(re, im) → (-im, re)`.
- CX: control en target moeten verschillend en kleiner dan `qubits` zijn. Wissel alleen paren met controlbit 1 en targetbit 0; nooit dezelfde twee amplitudes tweemaal verwisselen.

Na alle gates: bereken `p[k] = re² + im²` in oplopende indexvolgorde. Niet-eindige of negatieve waarden en `|sum(p)-1| > 1e-9` geven `NUMERICAL_ERROR`. Voor sampling deel je de kansen door die som; dat herstelt alleen afrondingsruis binnen de toegestane marge. Bouw een oplopende cumulatieve verdeling en zet het laatste element op precies 1.0. Samples kiezen de eerste index met `CDF[k] > u`; een CDF met nulmassa wordt zo nooit gekozen.

Dit is herhaald bemonsteren van dezelfde eindtoestand, equivalent aan opnieuw voorbereiden en aan het einde meten voor deze beperkte unitair-circuitset. Het is geen implementatie van tussentijdse meting of van een fysieke quantumtoevalsbron.

### Deterministische sampler
Gebruik SplitMix64 uitsluitend voor reproduceerbare simulatie (Vigna, 2015):
- Toestand begint met de hexadecimale `seed`.
- Bij elke trekking eerst optellen met `0x9e3779b97f4a7c15`, daarna de gepubliceerde xor-shift/multiply-mix met `0xbf58476d1ce4e5b9`, `0x94d049bb133111eb` en shifts 30, 27, 31.
- Alle integeroptellingen en vermenigvuldigingen wrappen modulo 2^64.
- Zet de bovenste 53 bits van het resultaat om naar `u = (word >> 11) * 2^-53`, dus `0 <= u < 1`.
- Precies één trekking per shot. Geen verborgen extra trekkingen.

Dit is **geen cryptografische RNG**, geen quantum-RNG en geen bron voor OS-/NAS-/updatesleutels. Bewaar de bronvermelding en toepasselijke toestemming voor de gebruikte algoritme-implementatie. De kernel-crypto mag deze module niet importeren.

## 4. Exact job- en resultaatformaat
Alle velden in dit voorbeeld zijn verplicht. Gates hebben alleen de getoonde velden. Gatevolgorde is uitvoervolgorde. Een eindmeting is impliciet.

```json
{
  "schema": "qsim-v0",
  "job_id": "bell-8",
  "qubits": 2,
  "gates": [
    {"op": "h", "target": 0},
    {"op": "cx", "control": 0, "target": 1}
  ],
  "shots": 8,
  "seed": "0000000000000000"
}
```

Gekozen succesformaat, met histogram in lexicografische bitstringvolgorde en zonder nulwaarden:

```json
{
  "schema": "qsim-v0",
  "job_id": "bell-8",
  "backend": "local-statevector-v0",
  "status": "completed",
  "qubits": 2,
  "shots_completed": 8,
  "counts": {"00": 5, "11": 3},
  "seed": "0000000000000000"
}
```

Voor X/Z/S/H: `{"op":"x","target":0}`, analoog voor de andere letters. CX heet exact `cx`, niet `cnot`; geen aliases in v0. Bewaar schema en backendversie in resultaten. Geen amplitudes, inputdump, tijdstip, hostnaam of machine-ID standaard in uitvoer.

Foutformaat:

```json
{
  "schema": "qsim-v0",
  "job_id": null,
  "backend": "local-statevector-v0",
  "status": "error",
  "error": {"code": "INVALID_REQUEST", "field": "qubits"}
}
```

Gebruik `job_id: null` zolang het ID niet betrouwbaar gevalideerd is. Na validatie mag het geldige ID terug. `field` is een vaste veldnaam, geen ruwe input. Foutcodes: `INVALID_REQUEST`, `UNSUPPORTED_SCHEMA`, `RESOURCE_EXHAUSTED`, `NUMERICAL_ERROR`, `CANCELLED`, `DEADLINE_EXCEEDED`, `INTERNAL_ERROR`. Input-/versiefouten: exit 2; budget/geheugen: exit 3; cancel/deadline: exit 4; numerieke/interne fout: exit 5. Succes: exit 0.

Een vroeg afgebroken stdin zonder volledige JSON is `INVALID_REQUEST`. Logs met technische details gaan naar stderr zonder circuit-/gebruikersinhoud. Er is geen serverpoort.

## 5. Annulering, deadlines en backendgrens
`qsim-core` biedt één synchrone operatie op gevalideerde invoer, met een door de aanroeper geleverd controlemiddel:

```rust
pub trait Control {
    fn checkpoint(&mut self, work_done: u64) -> Result<(), Abort>;
}
pub enum Abort { Cancelled, DeadlineExceeded }
pub fn execute(job: &ValidatedJob, control: &mut impl Control)
    -> Result<JobResult, ExecuteError>;
```

De core heeft geen wallclock of globale signal handler. Een checkpoint gebeurt vóór allocatie, vóór iedere gate, tijdens loops na maximaal 256 basisindices, vóór sampling en na maximaal 64 shots. De aanroeper vergelijkt monotone tijd met zijn deadline of signaleert annulering. Controlepunten maken coöperatieve afbreking toetsbaar; zij zijn geen realtimegarantie. Test vooraf geannuleerd en midden in uitvoering geannuleerd. Geef na annulering geen gedeeltelijk histogram terug als voltooid resultaat.

De CLI gebruikt een vaste monotone uitvoeringsdeadline van 10 seconden na ontvangst en validatie van het begrensde request; de tests kunnen een directe `Control`-stub gebruiken voor deterministische deadlinegevallen. Deze deadline onderbreekt geen geblokkeerde stdin-read of stdout-write. Het omvattende testharnas begrenst het gehele proces, inclusief invoer/uitvoer, tot 30 seconden. In Q1 hoeft geen cross-platform Ctrl+C-library te worden toegevoegd: process termination blijft mogelijk; coöperatieve cancellation wordt via de core-API bewezen. De toekomstige userspace-dienst verbindt het controlemiddel met haar IPC-cancelobject, I/O-deadlines en schedulerquota.

De later te implementeren backendgrens biedt `capabilities`, `execute` en `cancel`, met schema-/backendversie en resourcegrenzen. Q1 implementeert alleen de lokale simulator. Geen backendnaam of endpoint uit gebruikers-JSON laden. Q2/Q3 introduceren expliciet geregistreerde, lokale hardwareadapters met eigen controlemiddelen, driverrechten en reset-/isolatiebewijs.

## 6. Bestanden en dependencies die Codex nu moet maken
Onder `experiments/quantum/`:
- `Cargo.toml`: aparte workspace, geen dependency van kernel/boot.
- `qsim-core/src/{lib,complex,gate,validate,statevector,sampler}.rs`.
- `qsim-core/tests/{gates,validation,cancellation,fixtures}.rs`.
- `qsim-cli/src/main.rs`: begrensde stdin/typed JSON, core-aanroep, result encoding.
- `qsim-cli/tests/cli.rs`: exitstatus, limieten, stdout-schema.
- `fixtures/{identity,x1,bell8,bell4096,invalid-target}.json`.
- `README.md`: commando's, exacte gates, limieten, seedwaarschuwing en bewijsstatus.

Core: nul externe dependencies. JSON alleen in de hostadapter. Voorstel voor expliciete parserpins op basis van geraadpleegde maintainerdocumentatie: `serde = "=1.0.229"` met `derive`, `serde_json = "=1.0.154"`. De geopende latest-documentatie rapporteert die versies; de exacte serde-pagina was bereikbaar, de directe serde_json-versiepagina niet. Registry-fetch en compatibiliteit zijn **nog niet uitgevoerd**. De implementatieagent probeert deze pins éénmaal, legt Cargo.lock vast en rapporteert een echte resolutiefout. Alleen bij aantoonbare niet-beschikbaarheid mag een geverifieerde beschikbare versie worden vastgelegd in DEPENDENCIES.md; geen ongepinde stille upgrade. Deze dependencycontrole blokkeert de dependencyvrije core niet.

Gebruik de project-Rust-toolchain uit BOOT-AND-TOOLCHAIN.md zodra die is vastgelegd. Het quantumexperiment compileert voor de hosttarget; de latere eigen-OS-userspace-port heeft bovendien een correcte floating-point/contextsave-ABI nodig. Plaats de simulator niet in de kernel om dat te omzeilen.

Na het toevoegen van bestanden:
```bash
cargo test --manifest-path experiments/quantum/Cargo.toml --workspace --locked
cargo clippy --manifest-path experiments/quantum/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli < experiments/quantum/fixtures/bell8.json
```
Dit zijn uitvoeropdrachten voor de toekomstige implementatie, **geen reeds geslaagde projecttests**.

## 7. Concrete acceptatietests
| ID | Test | Vereist resultaat |
|---|---|---|
| Q1-01 | Geen gates, n=3 | Alleen `000`, count = shots |
| Q1-02 | X op q1, n=2 | Alleen `10`; detecteert omgekeerde bitvolgorde |
| Q1-03 | H gevolgd door H | Beginvector terug, complexverschil per amplitude <= 1e-12 |
| Q1-04 | X/X, Z/Z, S/S/S/S, CX/CX | Beginvector terug op meerdere vaste basis-/superpositietoestanden |
| Q1-05 | H(q0), S(q0), n=1 | Amplitudes (1/√2, i/√2); test fase, niet alleen histogram |
| Q1-06 | H(q0), CX(q0,q1), n=2 | Alleen indices 0 en 3 met kans 0,5 ± 1e-12; norm 1 ± 1e-12 |
| Q1-07 | Bell8, seed 0 | Exact `00:5,11:3`; resultaat hierboven |
| Q1-08 | Bell4096, seed 0 | Exact `00:2082,11:2014`; andere strings afwezig |
| Q1-09 | Gevalideerde job herhalen | Zelfde histogram en schema; geen timestampvelden |
| Q1-10 | q=0/13, target>=n, CX control=target, onbekende gate | Afwijzen vóór statevectorallocatie |
| Q1-11 | 4097 gates, 65537 shots, >256KiB, overflow-input | Begrensde fout, geen panic/hang of afgekapt 'succes' |
| Q1-12 | Dubbele sleutel, extra veld, fout seed, fractional qubits, trailing JSON | Specifieke fout/exit 2 |
| Q1-13 | Vooraf of tijdens gate/sampling geannuleerd | `CANCELLED`, geen voltooid/deelresultaat |
| Q1-14 | Deadline bereikt via test-Control | `DEADLINE_EXCEEDED`, deterministic test zonder slapen |
| Q1-15 | Foutieve/niet-eindige interne amplitudes geïnjecteerd in unit-test | `NUMERICAL_ERROR`; niet normaliseren alsof alles klopt |
| Q1-16 | Offline uitvoering na build | Bellfixture slaagt zonder netwerk; dependencyboom bevat geen netwerkclient |
| Q1-17 | Geldige maximale invoer | Eindigt binnen hosttestbudget of geeft expliciete deadlinefout; geen onbegrensde allocatie |

Statevector-tests gebruiken analytische toestanden; samplertests gebruiken vaste fixtures, zodat CI niet willekeurig faalt. Een extra steekproefexperiment met variërende seeds mag statistiek onderzoeken, maar vervangt de vaste regressietests niet. Prestaties zijn pas bekend na echte implementatie en meting.

**Uitgevoerde specificatiecontrole:** een zelfstandige Python-referentieberekening van de gepubliceerde SplitMix64-mix en Bellverdeling leverde 26 oktober 2026 voor seed 0 de genoemde 8-/4.096-shot-histogrammen. Ook de vectorgeheugenberekening (114.688 bytes) en maximale werkbegroting (17.633.280 eenheden) zijn rekenkundig gecontroleerd. Er is geen Rust-simulator gebouwd of getest; de Python-berekening is fixturecontrole, geen bewijs van de toekomstige implementatie.

## 8. Besloten PQC-beleid voor latere systeemintegratie
PQC is een apart beveiligingsonderdeel en geen vereiste om Q1-simulatie te bouwen.

| Gebruik | Vastgelegd uitgangspunt | Implementatiegrens |
|---|---|---|
| Nieuwe gedeelde geheimen in een passend transportprotocol | ML-KEM-768 als beoogde PQ-component | Geen zelfbedacht handshake-/hybride protocol; definitieve gestandaardiseerde protocolbinding en onderhouden library bij de netwerkfase |
| Ondertekende OS-/pakketmetadata | ML-DSA-65 als beoogd eigen updateprofiel | Vertrouwensankers, canonieke metadata, rollbackbeleid, key rotation en herstel samen testen |
| Symmetrische gegevensbescherming | AES-256 als kandidaat binnen een bestaand geauthenticeerd opslag-/berichtformaat | Modus, nonce-/tweakbeleid, authenticatie en key lifecycle niet zelf improviseren |
| Algoritmemigratie | Expliciete algoritme-ID, formaatversie en beperkte allowlist | Geen onveilig onderhandelde terugval; oude versies alleen volgens expliciet migratiebeleid |
| Entropy/sleutels | Betrouwbare systeementropy en gecontroleerde key lifecycle | Nooit simulatorseed/SplitMix64 gebruiken |

De parameterkeuzes zijn projectbesluiten, geen certificering of claim van geschiktheid voor elk toekomstig apparaat. NIST FIPS 203/204 beschrijven respectievelijk een KEM en handtekeningen; SP 800-227 geeft toepassingsaanbevelingen voor KEMs. De publicatiepagina's van FIPS 203 en 204 noemen errata: die moeten bij crypto-integratie expliciet worden verwerkt (NIST, 2024a, 2024b; Alagic et al., 2025). Een ongewijzigde klassieke firmwarevertrouwensketen wordt niet postkwantumveilig door alleen een PQ-handtekening op een later pakket toe te voegen.

Q1 implementeert **geen** van deze cryptografische primitives. Een nieuwe crypto-implementatie schrijven voor originaliteit valt buiten de opdracht; onderhouden, beoordeelde libraries worden gekozen in de concrete integratietaak.

## 9. Wat Q0 nu afsluit
De eerste simulatorrepresentatie, gates, bitvolgorde, grenzen, output, sampling, foutcodes, cancellationmodel, bestandsindeling en testfixtures zijn besloten. Codex kan Q1 direct programmeren. Specifieke fysieke QPU-drivers, realtimecontrollerelektronica en het bewijs voor een hybride systeemport blijven taken van Q2/Q3; ze blokkeren Q1 of de eerste OS-boot niet.

## Bronnen (APA 7)
Alagic, G., Barker, E., Chen, L., Moody, D., Robinson, A., Silberg, H., & Waller, N. (2025). *Recommendations for key-encapsulation mechanisms* (NIST Special Publication 800-227). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-227

IBM. (z.d.). *Bit-ordering in the Qiskit SDK*. Geraadpleegd op 26 oktober 2026, van https://quantum.cloud.ibm.com/docs/en/guides/bit-ordering

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203 — actuele errataverwijzing: https://csrc.nist.gov/pubs/fips/203/final

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204 — actuele errataverwijzing: https://csrc.nist.gov/pubs/fips/204/final

OpenQASM Contributors. (z.d.). *Introduction: OpenQASM live specification*. Geraadpleegd op 26 oktober 2026, van https://openqasm.com/intro.html

QIR Alliance. (z.d.). *QIR Alliance*. Geraadpleegd op 26 oktober 2026, van https://www.qir-alliance.org/

Serde Contributors. (z.d.-a). *Container attributes*. Geraadpleegd op 26 oktober 2026, van https://serde.rs/container-attrs.html

Serde Contributors. (z.d.-b). *Serde 1.0.229*. https://docs.rs/serde/1.0.229/serde/

Serde JSON Contributors. (z.d.). *Serde JSON*. Geraadpleegd op 26 oktober 2026, van https://docs.rs/serde_json/latest/serde_json/

Vigna, S. (2015). *SplitMix* [Broncode]. https://prng.di.unimi.it/splitmix.c
