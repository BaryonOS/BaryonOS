# Onafhankelijke AI-teamcontrole van native bewijs

Beide artifactcontroles zijn geslaagd: de matrix bevat tien gecontroleerde VM-runs en de volledige regressie negen gecontroleerde VM-runs. Dit is een onafhankelijke controle binnen het AI-team, geen menselijke audit en geen nieuwe uitvoering van builds of VM's.

- Broncommit: `94f12da0b9740a1a0ced1f08b4a30787f82ae79f`.
- Bronfingerprint: `2cffba892608c907b717158a83b8a1c66c7d79dc5843d2d03eabc4ff1831c028`.
- Runner: `dc5707cc6972b6b666d2e9d6925a275850d5afa9b48fa9bd68ce69d2c26175a0`.

De matrixcontrole heeft 71 werkelijke bestanden opnieuw gehasht, waaronder alle beschikbare images, kernel- en loaderbestanden, firmware, seriallogs en traces. De tien afzonderlijke resultaten komen overeen met het verzamelrapport. Alle acht negatieve gevallen, zowel readonly als writable aangeboden, eindigen met Corrupt(-18) tijdens service-startup, vóór beschikbaarstelling van de opslag. Hun imagehashes zijn gelijk gebleven; onafhankelijke hertelling van de virtio-traces vindt nul schrijfverzoeken. De positieve normal-run telt 4.963 schrijfverzoeken; de cold-recovery nul. De cold-recovery gebruikt exact de eindbytes van normal.

De regressiecontrole heeft 67 werkelijke bestanden opnieuw gehasht en negen resultaten, vier koude herstelparen, de bijbehorende builds en drie hosttestlogs gecontroleerd. De drie cut-runs registreren een gerichte host-kill met raw exit 1; de overige zes runs eindigen met raw exit 33. De hostlogs melden 45 volume-core-tests, 32 SDK-runtime-tests en 31 Python-tests, zonder fouten. De standaard regressie heeft geen read/write-traces; de onafhankelijke tracehertelling geldt alleen voor de tien matrixruns.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:b1eb2ad1f7512df4365cf58dce138ec67bb2240568399bb1caf722d231f9507a -->

Bewaarde receipts:

- `matrix-review.json` — SHA256 `b32e9287b99c6a453e0714ede09492e138eb8fe37783a36ebe2aabcecbccbef8`.
- `regression-review.json` — SHA256 `f6ce39fb80c69ddd02dd04b485f94c5e595792977d35cea75605d66b41cb89c1`.

Voor herhaling zijn het oorspronkelijke externe dossier
`C:/Users/simon/Documents/Codex/2026-11-02/volume-format-admission/` en
de daarin vastgelegde absolute paden nodig. Plaats de huidige verifiers in
de map `independent-native-review` van dat dossier en voer daar uit:

- `python -B verify_receipts.py matrix`
- `python -B verify_regression.py`

Beide scripts verwachten naast die map `../source` als Git-checkout van de
vastgelegde bron en `../native` met de oorspronkelijke bestanden. Als de
checkout `../source-checkout` heet, moet deze ook onder `../source` bereikbaar
zijn. De matrixcontrole vereist de oorspronkelijke bronimages op de in de
runrapporten opgenomen paden, de runimages, firmware, kernel en loader, en de
ongecomprimeerde `../native/runs/*/stderr-trace.txt`-bestanden. De regressie
vereist daarnaast de oorspronkelijke gate-, run-, image- en buildbestanden op
de vastgelegde paden. `archive_native.py` sluit images en binaries uit en
comprimeert de traces; de compacte Git-bundel volstaat daarom niet voor deze
volledige controles. De scripts lezen bron en bewijs; zij schrijven hun eigen
succes- of foutreceipts in hun scriptmap en kunnen bestaande succesreceipts
overschrijven. Bewaar de oorspronkelijke receipts vóór herhaling.

Een zelfstandige archiefcontrole is beschikbaar als `python3 verify_archive.py
manifest`, uitgevoerd vanuit de map `volume-format-admission`. Deze controleert
inventaris en opgeslagen en gedecomprimeerde hashes; zij herhaalt de originele
image-, bron- en VM-bewijscontrole niet.

<!-- /coderabbit-ai-generated:v1 -->

Historische before-hashes zijn gebonden aan bewaarde bronimages en de oorspronkelijke runrapporten: de verifier heeft de oorspronkelijke VM-uitvoering niet zelf waargenomen. De bevindingen gelden voor deze QEMU/virtio-tests en bewijzen geen fysieke stroomuitvalbestendigheid, native Btrfs/OpenZFS-ondersteuning of bestandherstel uit die bestandssystemen.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:df4edbd17696af8235a34101691e7ebd31a5679d4999dfe7e87422a2f092497f -->

De bewaarde receipts zijn door eerdere versies van de verifiers gemaakt; hun
`verifier_sha256` identificeert die versies. De aangescherpte controles zijn
niet opnieuw op het externe dossier uitgevoerd.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:74d10f154bccbf1380fcf1b73f404af5e951880a3ac9ff620b8840cb8303e7f3 -->

De huidige README en verifiers wijken af van de oorspronkelijke geïmporteerde
bestanden in `native-import.json`. De volledige rootcontrole staat daarom open;
het bestaande `root-verification.json` is een historisch receipt en bewijst
geen nieuwe uitvoering na deze wijzigingen.

<!-- /coderabbit-ai-generated:v1 -->

<!-- /coderabbit-ai-generated:v1 -->
