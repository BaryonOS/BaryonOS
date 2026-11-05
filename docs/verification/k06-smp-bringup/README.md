# K06 stap 1 — native AP-opstart en begrensde SMP-synchronisatieproef

**Bron:** `36703f50450d2075117b5a5e75a9bb4ca28b49a0` (vervangt `c687cd13`) op branch
`k06/smp-ap-bringup`, gestapeld op PR #191 (`a859db98`, begrensde
ACPI-handoff). **Datum:** 5 november 2026. **Besluit:** ADR-070.
**Bewijsniveau:** native QEMU/TCG (VM). Dit is geen fysieke hardware. De
review is door AI gedaan; er is geen menselijke audit.
**Score:** 41 → 41/100. K06 blijft open.

## Wat is gerealiseerd

- `crates/boot-protocol/src/acpi.rs`: een begrensde MADT-inventaris met
  alleen ingeschakelde, unieke processors (maximaal 32). Een kapotte,
  dubbele, onleesbare of te grote MADT maakt alleen SMP ongeldig. MCFG/DMAR
  blijven ongewijzigd. De MADT wordt in place geparsed: een by-value-struct
  liet de BSP-stack van 16 KiB overlopen, wat native is gevonden en
  hersteld.
- `kernel/src/m2/smp.rs` (opt-in profiel `native-smp`):
  INIT-SIPI-SIPI-opstart met een 16-bit SIPI-pagina die nooit gemapt wordt.
  De 32/64-bit-stap, de GDT en de AP-stacks met guards staan in het
  kernelimage. AP's claimen hun context op hardware-APIC-ID via
  `lock cmpxchg`, en de BSP trekt een context bij time-out met dezelfde CAS
  in. Gefaalde AP's krijgen INIT; lukt dat niet, dan stopt de boot. AP's
  nemen CR0/CR3/CR4/EFER/PAT van de BSP over en controleren die. Daarna
  doen ze 20 000 atomic- en spinlock-incrementen per CPU, waarbij de
  BSP verloren updates en dubbele toegang tot de kritieke sectie
  controleert.
- `tools/run_smp.py`, `tools/tests/test_smp_serial.py` en
  `.github/workflows/native-smp.yml`: een brongebonden gate met een exacte
  serieel-oracle. Die eist ook de exacte ACPI-marker
  `ACPI:UNAVAILABLE:NO_DMAR` voor q35/OVMF, zodat een regressie in de
  ACPI-discovery deze gate laat falen.

## Uitgevoerde native gevallen (alle exit 33 = geslaagd, geen time-out)

| Profiel | Geval | vCPU's / TCG | Uitkomst |
|---|---|---|---|
| native-smp | one | 1 / single | `SMP:ONLINE:1/1`, sync PASS |
| native-smp | two-single | 2 / single | 2/2 online, sync PASS |
| native-smp | four | 4 / multi | 4/4 online, sync PASS |
| native-smp | eight | 8 / multi | 8/8 online, sync PASS |
| native-smp | nine-capacity | 9 / multi | 8/9, `SKIPPED:CAPACITY=1` |
| native-smp | duplicate-madt | 4 / multi + `-acpitable` | `SMP:UNAVAILABLE:INVALID_MADT`, ACPI-inventaris gelijk |
| test-smp-ap-stall | ap-stall | 4 / multi | AP 1 `TIMEOUT`, INIT, opstart gestopt, 1/4 |
| test-smp-ap-mismatch | ap-mismatch | 4 / multi | AP 1 `MISMATCH` (CR0.WP gewist), 1/4 |

Exacte hashes (kernel, loader, manifest, bron, serial), SMP-regels en
duur per geval staan in [native-report.json](native-report.json).
De omgeving was QEMU 10.2.1, `pc-q35-8.2`, `qemu64`, met een geïsoleerde
OVMF-kopie.

## Commando's (vanuit de worktree, met de geïsoleerde toolchain)

```text
cargo test --locked -p boot-protocol                     # 25 tests op 36703f50, exit 0
                                                         # (28 op 75009e51 en later, na de #191-tests)
cargo clippy --locked -p boot-protocol --all-targets -- -D warnings   # exit 0
cargo fmt --all -- --check                               # exit 0
python3 -B -m unittest discover -s tools/tests -p test_smp_serial.py   # 4 tests
python3 -B -m unittest discover -s tools/tests -p test_acpi_serial.py  # 2 tests
python3 -B -m unittest discover -s tools/tests -p test_rtc.py          # 10 tests
python3 -B -m unittest discover -s tests -p test_common.py             # 8 tests
NOS_BUILD_DIR=build/k06-<profiel> python3 -B tools/build.py --kernel-feature <profiel>
NOS_BUILD_DIR=build/k06-<profiel> python3 -B -u tools/run_smp.py --profile <profiel>
```

**Regressie:** `milestone-m2` slaagt via `run_gate_case('m2',
'own-cr3-frame-ownership-wx')`, met ongewijzigd `-smp 1`,
`tcg,thread=single` en profiel `TCG/1CPU`. `native-rtc` met
`tools/run_rtc.py --case all` slaagt in 5 van de 5 boots. `cargo check` voor
`milestone-m2`, `framebuffer` en `native-rtc` slaagt.
`native-smp`+`milestone-m3` faalt zoals bedoeld met de `compile_error`.

**Niet uitgevoerd:** M3-, systeem- en desktopprofielen native, want
native-smp combineert er niet mee. Kernel-Clippy maakt geen deel uit van
CI; lokaal geeft het alleen vier bestaande, niet-gewijzigde meldingen in
`paging.rs` en `main.rs`. Er is geen fysieke hardware en geen
nieuwe-head-CI op GitHub op het moment van schrijven.

## Review

Twee onafhankelijke AI-reviews (Rust/`unsafe` en security) vonden geen
kritieke of hoge punten. De volgende bevindingen zijn verwerkt in
`c687cd13`:
- Een MADT-leesfout brak de hele discovery af (medium, regressie in
  niet-SMP-profielen).
- Een mislukte INIT na een time-out werd genegeerd (medium/laag).
- Er ontbrak een fence vóór de ICR-MMIO.
- `wait_us` faalde open op een zwevende bus.
- Het GDT-einde onder 4 GiB werd niet gecontroleerd.
- De oracle controleerde de AP-ID's niet.
- Een CI-run op `main` kon worden geannuleerd.

## CI-regressie hersteld (36703f50)

De eerste CI op #201 faalde in 14 bestaande native gates. In M3-profielen
liep het ge-inlinede ACPI-discoveryframe over in de stackguard van de
BSP, direct na `KS-02`; het frame bestond uit twee tabelbuffers van 4 KiB
plus de grotere `Discovery`. Lokaal gereproduceerd met `milestone-m3`. De
kernel bewaart scratch en inventaris nu statisch (`discover_into`). Lokaal
slagen opnieuw: de M3-boot, K05 (6/6), de M6C-smoke, `milestone-m2`,
`native-rtc` (5/5) en de SMP-gate. Les: na iedere boot-protocolwijziging
ook een M3-profiel native booten.

## Open grenzen (waarom K06 niet geaccepteerd is)

- Er is geen SMP-scheduling, geen per-CPU-runqueue, geen IPI's naast
  INIT/SIPI en geen TLB-shootdown. AP's parkeren met `cli; hlt`.
- Er is geen per-CPU TSS/IST: een NMI of #DF op een AP stopt de VM. Een
  exception op een AP raakt de gedeelde handlers en stopt terminaal.
- Er is geen x2APIC-modus: firmware in x2APIC-modus valt terug naar één CPU.
- MTRR's, XCR0 en FPU-staat worden niet vergeleken. De INIT/SIPI-timing
  is alleen begrensd met PIT-kanaal 2.
- De SIPI-pagina vereist de identity-map van de firmware onder 1 MiB. Dat
  geldt voor OVMF; andere firmware is niet bewezen.
- Alleen QEMU/TCG: geen fysieke meerkernhardware en geen
  prestatiemetingen.
