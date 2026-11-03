Uitvoeringsspoor N3

Alle .sh-bestanden zijn gestart met:

    wsl -d Ubuntu -- bash /mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs/<script>

De scripts bevatten de volledige flags, outputpaden, grenzen en environment. Code 134 is SIGABRT in de directe zdb-aanroep; ztest zelf retourneert 3 na zijn aborterende child. Dat verschil staat expliciet in evidence.json.

| Script | Status / bewijs |
|---|---|
| 01-acquire.sh | Download/extract geslaagd; zes pinned pakketten; logs/package-metadata.txt, download.txt, package-sha256.txt. Geen installatie. |
| 02-preflight.sh | ldd en help opgenomen. |
| 03-create-initial-help-mismatch.sh | Mislukt; logs/create*. |
| 03-create-invalid-dedup.sh | Mislukt; logs/create-retry*. |
| 03-create-unsupported-tunable.sh | Mislukt; logs/create-run*. |
| 03-create-low-as.sh | Mislukt; logs/create-final*. |
| 03-create.sh | Geslaagd host-create; logs/create-bounded*. |
| 04-baseline-freshcheck.sh | Baselinekopieën, hashcontrole en zdb fresh check geslaagd; logs/fresh-zdb*. |
| 05-reuse-kill.sh | ztest -E exit 3, geen killrun bereikt; logs/reuse-kill*. |
| 06-labels.sh | Labelinspectie baseline geslaagd; logs/baseline-labels.txt. |
| 07-crash-create.sh | 3 echte kills, exit 0; inhoudelijke no-leak gate niet geaccepteerd; logs/crash-create*. |
| 08-backup-stream.sh | Beide interne zdb-aanroepen exit 134; scriptstatus zelf is niet de oracle; logs/backup-stream*, ds0-objects*. |
| 09-crash-freshcheck-blocks.sh | Beide interne opdrachten exit 0; crashlog heeft 9216-byteverschil, baseline no leaks; afzonderlijke *-exit.txt is oracle. |
| 10-blockpointers.sh | Blockpointerinventaris exit 0; logs/baseline-blockpointers*. Geen corruptie uitgevoerd. |
| 11-package-evidence.sh | Ruwe logs/pakketten/bronnen, crasharchive en definitieve hashes gebundeld. |
| 12-write-evidence.py | JSON opgesteld uit bewaarde exitcodes/logs; zes pakkethashes vergeleken. |
| 13-hash-artifacts.py | Artefactmanifest gegenereerd en opnieuw gelezen; hashes daarin opnieuw gecontroleerd. |

Aanvullende read-only broninspecties gebruikten curl naar de vijf in evidence.json opgenomen officiële upstream URLs en lokale sed/grep/nm. Een niet-bestaande upstream lib/libzpool/import.c gaf HTTP 404; de correcte relevante functies zijn vervolgens in lib/libzpool/util.c gevonden. Deze inspecties veranderden geen enginecode. De eerste inline PowerShell/bash-quotingpoging voor acquisitie gaf een mkdir-fout vóór acquisitie; de vastgelegde scripts gebruiken daarna letterlijke here-strings.