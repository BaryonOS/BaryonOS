# Veld-02 — gereedschappen op hetzelfde document

Baseline `264ad5b0`, geteste productbron
`3e79b6a8a6bba75c42efe5e307813eefc066fa71`, fingerprint v3
`7da356474e591a062456f395e70ac6bba053acc29a080d077df86ec488557e71`.

**Eigen-kernel-/VM-bewijs:** volledige venstergate exit 0, 31 fasen,
254,22 s, ruwe QEMU-exit 33. Native-, klembord-, workspace- en
document-toolflags zijn true. F4 opent Lezen op dezelfde echte draft;
X/Delete/Backspace veranderen geen byte; F4 herstelt Schrijven met exact
het oorspronkelijke volledige appbeeld. Alle pixels buiten de toegewezen
body blijven ongewijzigd. Bestaande volledige appbeelden, klembord,
verplaatsing, echte compositorfault/rebind, behouden draft/berekening,
bewaren/heropenen, onafhankelijke diskcontrole en cleanup blijven vereist
en slagen. Geen extra pixelmaskers.

Na de bestaande geïsoleerde `veld-env/env.sh`:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-tools-20261103/probe"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103"
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Beide Python-opdrachten exit 0. Build 3,34 s; runner inclusief
hostoraclevoorbereiding 260,90 s. QEMU 10.2.1, `pc-q35-10.0`,
TCG single-thread, eigen x86_64-kernel/UEFI, synthetische wegwerpdisk,
geen gastnetwerk of hostschijf. [Compacte uitslag](native-report.json)
bindt fasen, bron, firmware, apporakels, schijf en volledig lokaal rapport.
[Loghashes](logs.sha256) binden de volledige lokale commandologs.

**Hostbewijs**, alle exit 0:

- `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml`
- `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml`
- `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings`
- `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings`
- `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check`
- `python3 -B tools/test_window_session.py` — 13 tests
- `git diff --check`

Een eerste hostrun gaf exit 101: een oude hersteltest beperkte Tab-navigatie
tot twee stappen. De focuscyclus bevat nu ook de toolknop; de test navigeert
begrensd tot Document en blijft dezelfde verse-invoerepoch en bewerkte bytes
controleren. Geen rode test als groen omgelabeld.

**Grenzen:** dit zijn twee interne gereedschappen in hetzelfde geïsoleerde
proces, geen DV-01 met twee afzonderlijke appidentiteiten. Read-only opslag
blijft read-only na terugwisselen. Wisselen doet geen opslag-RPC en behoudt
object, selectie, scroll en status; hosttegenvoorbeelden controleren ook
plakken/semantische mutatie, weigering tijdens save/modal en 100%/200% layout.
200% is in deze run uitsluitend hostbewijs, geen verse native schaalproef.
Geen nieuwe GitHub-CI-run, fysieke/human-audit-/schermlezer- of releaseacceptatie.
Score **41 -> 41**. De volledige gebruikersomgeving blijft in uitvoering.
Geen merge/release/PR-sluiting. Terugbouw kan vanaf baseline in een aparte
worktree; geen destructieve reset nodig.
