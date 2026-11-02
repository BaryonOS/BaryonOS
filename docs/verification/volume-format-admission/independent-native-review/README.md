# Onafhankelijke AI-teamcontrole van native bewijs

Beide artifactcontroles zijn geslaagd: de matrix bevat tien gecontroleerde VM-runs en de volledige regressie negen gecontroleerde VM-runs. Dit is een onafhankelijke controle binnen het AI-team, geen menselijke audit en geen nieuwe uitvoering van builds of VM's.

- Broncommit: `94f12da0b9740a1a0ced1f08b4a30787f82ae79f`.
- Bronfingerprint: `2cffba892608c907b717158a83b8a1c66c7d79dc5843d2d03eabc4ff1831c028`.
- Runner: `dc5707cc6972b6b666d2e9d6925a275850d5afa9b48fa9bd68ce69d2c26175a0`.

De matrixcontrole heeft 71 werkelijke bestanden opnieuw gehasht, waaronder alle beschikbare images, kernel- en loaderbestanden, firmware, seriallogs en traces. De tien afzonderlijke resultaten komen overeen met het verzamelrapport. Alle acht negatieve gevallen, zowel readonly als writable aangeboden, eindigen met Corrupt(-18) tijdens service-startup, vóór beschikbaarstelling van de opslag. Hun imagehashes zijn gelijk gebleven; onafhankelijke hertelling van de virtio-traces vindt nul schrijfverzoeken. De positieve normal-run telt 4.963 schrijfverzoeken; de cold-recovery nul. De cold-recovery gebruikt exact de eindbytes van normal.

De regressiecontrole heeft 67 werkelijke bestanden opnieuw gehasht en negen resultaten, vier koude herstelparen, de bijbehorende builds en drie hosttestlogs gecontroleerd. De drie cut-runs registreren een gerichte host-kill met raw exit 1; de overige zes runs eindigen met raw exit 33. De hostlogs melden 45 volume-core-tests, 32 SDK-runtime-tests en 31 Python-tests, zonder fouten. De standaard regressie heeft geen read/write-traces; de onafhankelijke tracehertelling geldt alleen voor de tien matrixruns.

Receipts en herhaalbare read-only verifiers:

- `matrix-review.json` — SHA256 `b32e9287b99c6a453e0714ede09492e138eb8fe37783a36ebe2aabcecbccbef8`.
- `regression-review.json` — SHA256 `f6ce39fb80c69ddd02dd04b485f94c5e595792977d35cea75605d66b41cb89c1`.
- `verify_receipts.py` — voer uit met `python -B verify_receipts.py matrix`.
- `verify_regression.py` — voer uit met `python -B verify_regression.py`.

De scripts lezen alleen bron en bewijs; uitsluitend hun eigen receipts worden geschreven. Historische before-hashes zijn gebonden aan bewaarde bronimages en de oorspronkelijke runrapporten: de verifier heeft de oorspronkelijke VM-uitvoering niet zelf waargenomen. De bevindingen gelden voor deze QEMU/virtio-tests en bewijzen geen fysieke stroomuitvalbestendigheid, native Btrfs/OpenZFS-ondersteuning of bestandherstel uit die bestandssystemen.
