# K05 — vaste posities van startcapabilities

29 oktober 2026. Basis `main` `3836361` (PR #54), branch
`codex/supervisor-startup-slots`. Eigen Rust no_std-kernel, x86_64 en UEFI.
De vaste posities uit het bestaande startcontract (ingevoerd met ABI 1.3)
blijven behouden. Geen nieuwe rechten, afhankelijkheden of SDK-interface.

Een lege cap0 mag cap1 niet van offset 64 naar offset 56 verplaatsen.
Beide posities worden vóór allocatie op type, rechten en TRANSFER gecontroleerd.
Het aantal niet-lege capabilities telt uitsluitend voor het nodebudget.
Bron: K05-PROCESS-MANAGEMENT §3 en P2-thread PRRT_kwDOVZC91s6xvSPj (#51).

De normale native client krijgt werkelijk alleen cap1 en controleert cap0=0,
SEND en weigering van RECV/DERIVE. Daarna volgen 24 echte IPC-antwoorden,
herstart en volledige cleanup. DENIED bevat vier extra negatieve controles
op lege posities met rechten en verboden delegatie via cap1.

## Bronbinding en uitvoering

Exact geteste bron `56b576b16da1e4503bc5f9167a476474a58c33cf`, fingerprint
`cccf0e3e7fcec439428208c4458a0194ec23c6120e1f03d96abe65b0711631b7`.
Geen bron- of HEADwijziging tijdens de gate. Rust 1.90.0, Debian QEMU 10.0.13,
expliciet pc-q35-8.2, qemu64, TCG, 1 CPU, 256 MiB, OVMF 2025.02-8+deb13u1 (4M).

```sh
NOS_QEMU_MACHINE=pc-q35-8.2 \
NOS_BUILD_DIR=$PWD/build/verify-supervisor-slots \
NOS_TARGET_DIR=$PWD/target/verify-supervisor-slots \
CARGO_TARGET_DIR=$PWD/target/verify-supervisor-slots \
python3 -B -u tools/test_supervisor.py
```

Exit 0: vijf echte VM-runs (normal, crash, denied, parent-exit, quota), alle
raw 33. Vier hostcontroles: 40 capability-, 17 SDK-, 1 voorbeeldlib- en
7 harnastests groen. DENIED matcht 21/21 negatieve controles en 11 geweigerde
spawns; fault-, orphan-, quota- en resourcecontroles blijven intact.

Oude kernel met nieuwe test: timeout na 15 seconden, geen READY/PASS
(`before/result.json`). De gerichte herstelde normalcase is groen
(`focused-after/`); die blijft apart van de volledige integratiegate (`gate/`).

Ook exit 0: `cargo +1.90.0 clippy --locked --manifest-path sdk/Cargo.toml -p nieuw-os-supervisor-example --all-targets -- -D warnings`, root-/SDK-formatteringschecks en diffcontrole. Een afzonderlijke AI-review op `2665f68` vond geen blocker en herhaalde 7 harnastests. Dit is geen menselijke audit.

SHA256SUMS bindt de exacte rapport- en logbytes, inclusief firmware-CRLF.
De score blijft 32%; geen nieuw criterium. Geen fysieke hardware- of SMPclaim.
Rollback: normale revert van de K05-slotfix; geen reset of rechtenverruiming.
