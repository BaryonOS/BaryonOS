# DocumentSession — herstel expliciete native dependencyketen

**Geaccepteerd op bron `0d70af000f58d2140bc71ad02055dc7486310a8d`:**
twee echte QEMU-runs, normal en een afzonderlijke koude read-only recovery,
beide raw exit 33. Bronvingerafdruk v3:
`c7f635961e64600c9eebb785fb42025e40efeb8e8c7324a46a8644d9b48d3118`.
Latere wijzigingen in deze branch archiveren alleen dit bewijs.

Basis `309ff53cd861f26aa37b2615c0276695d9b62f8f`, PR #56.
De CI-job `qemu-smoke` op deze bron faalde vóór de eerste DocumentSession-VM:
de handmatige rustc-route bouwde vier libraries en gaf de nieuwe
`volume_core`-dependency niet door. De lokale oorspronkelijke build bevestigt
dit met exit 1 en E0432/E0433; de exacte fout en compileropdracht staan in
`before-document-library.*`.

De builder compileert nu vijf werkelijke rlibs in afhankelijkheidsvolgorde.
`volume_core` krijgt expliciet `catalog_core`; de documentlibrary krijgt haar
vier directe dependencies. De gebruikers-ELF linkt dezelfde vijf geselecteerde
libraries. Het harnas controleert de exacte compiler-externsets, de werkelijke
librarybytes en alle volume-Cargo-/Rust-bronnen, inclusief `upload.rs`.
Geen functie, ABI, recht of bestaande inhoudelijke acceptatie is uitgezet.

De 32 gerichte Pythonproeven slagen, inclusief weigering van ontbrekende
volumebronnen, ontbrekende volume-rlib en een ontbrekende volume-extern.
Ook de echte freestanding M6c-gebruikers-ELF bouwt nu. De native poort
bevestigt vijf werkelijke rlibs en zes afzonderlijke compilercommando's per
profiel. Normal en recover gebruiken bytegelijke user-/library-artifacts;
het harnas controleert de in de kernel ingebedde user-ELF en alle inputhashes.

De koude run leest exact de opgeslagen image terug, zonder writes of ACKs.
Sequence 10, 10 normale mutatie-ACKs en 0 recovery-ACKs. Schijfhash:
`9f8e37040d7f3d79fabe2ae28782c538c5e656328b0537002f66738de1df6999`.
Het onafhankelijke bestaande schijforakel, raw exits, markers en de complete
freeze-/artifactcontroles slagen. Zie `native-gate.json`, `normal/`, `recover/`
en de bewaarde compiler-/faselogs. SHA256SUMS bindt alle archiefbytes.

Omgeving: Rust 1.90.0, Debian QEMU 10.0.13, pc-q35-8.2, qemu64, TCG,
één CPU, 256 MiB, matching OVMF 4M. Er zijn geen binaries, images, caches of
persoonlijke gegevens in dit bewijsarchief opgenomen. De daadwerkelijke
images blijven in de aangewezen buildmap van de geteste worktree.

Uitgevoerde opdracht op de genoemde bron, met de bestaande lokale toolchain:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools \
PATH=/workspace/.nieuw-os-env/bin:$PATH \
OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
NOS_QEMU_MACHINE=pc-q35-8.2 NOS_DOCUMENT_SESSION=1 NOS_SDK_RUNTIME=1 \
NOS_BUILD_DIR=$PWD/build/ci-document-deps \
NOS_TARGET_DIR=$PWD/target/ci-document-deps \
CARGO_TARGET_DIR=$PWD/target/ci-document-deps \
python3 -B tools/test_document_session.py \
  --expected-head 0d70af000f58d2140bc71ad02055dc7486310a8d \
  --expected-source c7f635961e64600c9eebb785fb42025e40efeb8e8c7324a46a8644d9b48d3118
```

Dit is een gerichte reparatie van de bestaande headless M6c-poort. Zij bewijst
geen grafische Materiaal-ervaring, native v7-workflow of volledige testrelease.
