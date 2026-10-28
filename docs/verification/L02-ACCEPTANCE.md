# L02 — native lokale quantumuitvoering

**Status:** geaccepteerd voor roadmapcriterium L02 op uitvoeringsbron `433bab9feab74473517eb8d44f2fb4475848e0cf`; geïntegreerd in `main` als `7fb8c722fc853692251e26fcdcea0ed86954ef15`.

## Geaccepteerd gedrag

De eigen OS-runtime start twee geïsoleerde ring-3-processen: een caller en een qsim-jobdienst. De caller verstuurt het bestaande canonieke `QJ01`-request via de bestaande capability-IPC. De dienst voert de begrensde qsim-port uit, retourneert vaste resultaatpagina's en blijft binnen de eigen procesheap.

De native acceptatie controleert Bell met 4.096 shots, een zes-qubitreferentiecircuit, een Cancel die tijdens uitvoering via IPC wordt opgehaald, afdwinging van het aangevraagde work-budget, resultaatpaging, minimale SEND/RECV-rechten en volledige cleanup van processen, capabilities, endpoints, queues en frames.

Dit bewijs geldt uitsluitend voor lokale software-simulatie op het eigen OS. Het bewijst geen fysieke QPU, quantumversnelling, clouduitvoering of algemene quantumhardwareondersteuning.

## Bron- en CI-binding

- PR: #30, bronhead `433bab9feab74473517eb8d44f2fb4475848e0cf`.
- GitHub Actions: run `40743008938`, conclusie `success`.
- Job `host-quality`: `123009309122`, volledig groen.
- Job `qemu-smoke`: `123009970078`, volledig groen.
- QEMU-stap `Native quantum job interface`: groen naast baseline M6c en DocumentSession normal/koude recovery.

## Reproduceren

```sh
rm -rf build/repro-qsim-job target/repro-qsim-job
NOS_BUILD_DIR="$PWD/build/repro-qsim-job" \
NOS_TARGET_DIR="$PWD/target/repro-qsim-job" \
CARGO_TARGET_DIR="$PWD/target/repro-qsim-job" \
python3 -B tools/build.py --kernel-feature native-qsim-job

NOS_BUILD_DIR="$PWD/build/repro-qsim-job" \
NOS_TARGET_DIR="$PWD/target/repro-qsim-job" \
CARGO_TARGET_DIR="$PWD/target/repro-qsim-job" \
python3 -B tools/run_qsim_job.py
```

De runner vereist de brongebonden kernel/app-artifacts en accepteert uitsluitend de verwachte QEMU-exit, markerreeks, procesresultaten, resourcegrenzen en cleanup.
