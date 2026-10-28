# I04 — prefixbewuste scalarinstructiecontrole

28 oktober 2026. Gerichte reparatie van de concrete G04-reviewbevinding.
Alleen `experiments/quantum/native/build.py`, de bestaande bijbehorende
Python-tests en deze bewijsmap veranderen. De simulator, allocator, ABI,
kernel en gedeelde bouwconfiguratie blijven buiten deze wijziging.

## Fout en herstel

GNU objdump 2.42 decodeert de echte bytes `cd80 66dbe3` als `int 0x80`
gevolgd door `data16 fninit`, en `cd80 f3dbe3` als `int 0x80` gevolgd door
`repz fninit`. De oude controle behandelde `data16` respectievelijk `repz`
als opcode, waardoor de x87-instructie ten onrechte groen werd. Ook andere
prefixen kunnen dezelfde grens raken; de reproductie bewaart werkelijke
objdump-uitvoer en de oude én nieuwe uitkomst.

De controle verwijdert nu de volledige bekende legacy-/REX-prefixketen
voordat zij de eigenlijke opcode beoordeelt. Een expliciete lijst staat
alleen de ondersteunde scalarinstructies toe. Een onbekende opcode,
onbekende prefix, onafgemaakte prefix of onmogelijke prefixlengte wordt
geweigerd. De controle op uitgebreide, control- en debugregisters en de
uitsluitende `int 0x80`-grens volgt daarna. Dit is een begrensd profiel voor
de huidige artifact; toekomstige compilerinstructies moeten eerst worden
beoordeeld voordat zij aan de lijst worden toegevoegd.

## Uitgevoerd bewijs

- **8 gate-tests slagen.** Zij bevatten 14 echte bytefixtures met prefixed
  x87/MMX/SSE/AVX-instructies, drie prefixed hardware-/andere ABI-gevallen,
  vier incomplete of onbekende decodergevallen en een positieve scalar-
  bytefixture. De onbetrouwbare fixtures worden uitsluitend gedisassembleerd,
  nooit uitgevoerd. Extra tekstcases dekken alle herkende prefixnamen.
- **15 bestaande I03-harnastests slagen.** De bron-/artifact- en native
  markercontroles blijven behouden.
- De volledige native targetbuild slaagt met Rust 1.90.0. De nieuwe gate
  accepteert de werkelijke **4.096 instructies en vier int80-sites** van de
  bestaande scalar-port. De ELF-SHA256 is exact gelijk aan de I01-artifact:
  `ee9ec71e1c37719c51ffd92021b62aaa8c9d6303bc92482547d717ac1d3a0481`.
- Pythoncompilatie en diff-whitespacecontrole slagen. `checks.json` bindt
  commando's, logs en bronhashes; `build-report.json` bindt de volledige
  native ELF-/disassembly-/broncontrole. `prefix-reproduction.json` en
  `prefix-reproduction.txt` leggen de oude fout en het herstel vast.

## Reproduceren

```bash
python3 -B experiments/quantum/native/tests/test_build_gate.py
python3 -B tests/test_qsim_harness.py
python3 experiments/quantum/native/build.py \
  --output-dir /tmp/nieuw-os-i04-build \
  --target-dir /tmp/nieuw-os-i04-target
```

Rust 1.90.0 met `x86_64-unknown-none`, rust-lld en GNU objdump/nm zijn
vereist. De native gate bouwt offline met het bestaande lockbestand.

## Grenzen en integratie

Dit repareert een artifactgate; het voegt geen OS-functie toe. Geen nieuwe
VM-run is vanuit deze deelbranch uitgevoerd. De hoofdagent herbouwt het
native-qsim-profiel op de geïntegreerde bron, zodat het bouwrapport aan de
nieuwe checkerbron is gebonden, en voert de strikte I03-runner daarop uit.
De oude native runtime-uitkomst behoort bij haar eigen bronversie.

De inspectie van GNU-disassembly is geen formeel bewijs van alle mogelijke
controlflow of machinecode. De uitvoer en hashes blijven nodig om de
afgebakende claim te beoordelen. Deze fix opent geen FPU-context-ABI en
maakt geen algemene app-, simulatorservice- of QPU-claim. Voorstel voor
de vaste roadmap: L02 blijft open, **+0 punten**.
