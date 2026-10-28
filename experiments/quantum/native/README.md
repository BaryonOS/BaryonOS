# I01 — begrensde native qsim-port

Deze adapter bouwt de bestaande `qsim-core` als afzonderlijk Rust-programma
voor `x86_64-unknown-none`. De simulatoralgoritmen blijven ongewijzigd. Het
programma heeft geen blok-, document-, netwerk- of beheerrechten. Het voert
de berekeningen in ring 3 uit zodra het opt-in kerneltestmanifest deze ELF
laadt. Een geslaagde `build.py` is uitsluitend compilatie-/artifactbewijs;
native uitvoering krijgt afzonderlijk QEMU-bewijs van de integrator.

## Afbakening

- Maximaal zes qubits, 64 gates, 4.096 shots per job.
- De bestaande simulatorseed blijft deterministisch en is geen entropy of
  cryptografische toevalsbron.
- De bestaande `Control`-interface geeft annulering en een begrensd
  werkbudget door. De proef test deterministische deadline-injectie; er is
  geen nieuwe native klok- of IPC-cancelcapability.
- De allocator gebruikt uitsluitend de bestaande private 64-KiB-procesheap.
  Hij geeft geheugen in deze eindige proef pas bij procesexit terug. Dit is
  nog geen allocator voor een permanent herbruikbare quantumdienst.
- Alle heapaanvragen gebruiken gecontroleerde optellingen en uitlijning.
  De proef controleert zowel een geweigerde allocatoraanvraag als de echte
  `ResourceExhausted`-uitkomst van de bestaande simulatorcore.
- `f64` wordt door Rusts bestaande target/compiler-builtins met scalar
  instructies uitgevoerd. De kernel hoeft geen x87/SIMD-context toe te voegen.
  `build.py` controleert de gedecodeerde volledige `.text` en weigert x87,
  MMX, SSE, AVX, opmaskregisters en onbekende instructieregels. Dit is een
  artifactcontrole, geen formeel bewijs of fysieke-hardwarekwalificatie.

## Reproduceren

Benodigd: Rust 1.90.0 met `x86_64-unknown-none`, `rust-lld`, Python 3,
GNU `objdump` en `nm`. Er zijn geen nieuwe externe Rust-dependencies. De
eigen `Cargo.lock` bevat alleen deze adapter en de bestaande lokale core.
Gebruik per gelijktijdige bouwer een eigen target- en uitvoermap.

```bash
CARGO_TARGET_DIR=/tmp/nieuw-os-i01-target cargo test --manifest-path experiments/quantum/native/Cargo.toml --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-i01-target cargo clippy --manifest-path experiments/quantum/native/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 experiments/quantum/native/tests/test_build_gate.py
python3 experiments/quantum/native/build.py --output-dir /tmp/nieuw-os-i01-build --target-dir /tmp/nieuw-os-i01-target
```

`build-report.json` bindt bronhashes, opdrachten, Rust-versie, ELF-segmenten,
binaryhash en disassemblyhash. `native_runtime_performed` blijft daarin
expliciet `false`: deze opdracht start geen emulator. De ELF heet
`qsim-native-probe.elf`. Bestaande kernel-/boot-/capability-interfaces worden
niet door deze map gewijzigd.

## Native proefinterface voor de integrator

Entry `0x40000000`, private RW-rapportpagina `0x40010000`, aparte NX-rodata
vanaf `0x40011000`. De bestaande 64-KiB-heap begint op `0x60000000`.
Het kernelmanifest initialiseert case op nul en een eigen patroon. Alleen
`abi_info` en `yield` van de bestaande `int 0x80`-ABI worden aangeroepen.
Het programma blijft na afronding yielden, zodat de kernel de uitkomst en
voortgang van andere processen kan beoordelen.

| Offset | Betekenis |
|---:|---|
| +0 | Kernelcase; uitsluitend 0 ondersteund |
| +8 | Appcheckpointteller |
| +16 | Kernelpatroon; door app ongewijzigd gelaten |
| +24 | Gereserveerd; door app ongewijzigd gelaten |
| +32 | 0 bezig, 1 afgerond, `0x100 + code` bij fout |
| +40 | Geslaagde scenario's als bitmasker; vereist 127 |
| +48 / +56 | Bell4096, seed 0: count0=2082, count3=2014 |
| +64 | Bell-shots: 4096 |
| +72 | Gebruikte arenabytes vóór de uitputtingsproef; 1..65536 |
| +80 | Grootste werkafstand tussen checkpoints; maximaal 448 |
| +88 | Zes-qubit X-circuit: count63=4096 |
| +96 | Arenareservering na gecontroleerd uitputten: 65536 |
| +104 | Echte core geeft `ResourceExhausted`: 1 |

De zeven scenario's zijn Bell, zes qubits, vooraf annuleren, tijdens uitvoering
annuleren, geïnjecteerde deadline, ongeldige invoerlimieten en heapuitputting.
De kernel moet hun waarden onafhankelijk vergelijken en gebruikersfouten,
time-out en onverwachte termination als fout behandelen. Alleen een
gekopieerd appveld is geen zelfstandige kernelacceptatieclaim.

## Vervolg

Een echte quantumdienst vraagt een herbruikbare allocator of een expliciete
joblevensduur, begrensde IPC-aanvoer, een intrekbaar cancelobject en
procesgebonden quota. Geen fysieke QPU, quantumversnelling, lokale AI,
NAS-uitvoering of cloudfallback is in deze stap toegevoegd. De vaste
roadmap krijgt geen punt uitsluitend voor compilatie of hosttests.

Terugname: laat de opt-in quantumfeature uit het kerneltestmanifest weg;
de bestaande QEMU-profielen en hostsimulator blijven beschikbaar.
