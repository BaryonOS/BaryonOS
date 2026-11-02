# Btrfs host-tool herstelbaarheid — 2 november 2026

Alleen offline gebruikersruimtetools zijn getest; geen Btrfs-kerneldriver, mounts, loopdevices, fysieke disks, snapshots, encryptie, hardwarestroomuitval of Nieuw-OS-native support.

Ubuntu 26.04.1 WSL2, Linux 6.18.54.1-microsoft-standard-WSL2. Officieel Ubuntu-pakket btrfs-progs 6.17.1-1build1 via apt download opgehaald en met dpkg-deb in een eigen tijdelijke map uitgepakt. Geen systeemwijde installatie. Pakketmetadata, SHA256 en gebruikte bestaande shared-libraryversies zijn vastgelegd.

Baseline: regulier disposable bestand van 256 MiB, data single, metadata/system DUP, crc32c, 4 KiB sectors, 16 KiB nodes. mkfs --rootdir bevat 132 bestanden en 1 symlink: 128 kleine bestanden, lege file, UTF-8-bestandsnaam, 128 KiB nullen en 1 MiB deterministische payload.

| Proef | Check-exit | Restore-exit | Onafhankelijke vergelijking |
|---|---:|---:|---|
| Gezonde baseline | 0 | 0 | Alle 133 items exact |
| Primaire superblock 4 KiB nul op 64 KiB | 1; met --super 1: 0 | 0, automatische backup-superkeuze; expliciet --super 1 ook 0 | Alle 133 items exact |
| Eén payloadbit gewijzigd | 1, csum mismatch | 0 zonder waarschuwing | Precies 1 corrupte file; precies byte 12345, XOR 1 |
| Image afgekapt van 256 naar 32 MiB | 1 | 1 | Geen volledig herstel; root/backup buiten bereik |

De eerste baseline-restore gaf exit 1 doordat de doelmap nog niet bestond en eerst een symlink werd verwerkt. Deze mislukte harnessrun blijft zichtbaar. Daarna is uitsluitend een nieuwe lege doelmap vooraf aangemaakt; de herhaalde restore slaagde. Dit wordt niet als filesystemfout gepresenteerd.

Belangrijk: een geslaagde btrfs restore-exitcode bewijst geen integere bestandsinhoud. Onafhankelijke corpus-/back-uphashes zijn vereist voordat herstel wordt bevestigd. Single-data bevat geen tweede payloadkopie; deze proef bewijst foutdetectie, geen self-healing. Readonly check/restore heeft geen bronimage gewijzigd (pre/post SHA256 gelijk).

`evidence.json` bevat resultaten, scope en mutatie-offsets; `commands.json` alle subprocessargumenten en exitcodes; `logs/` volledige stdout/stderr; `expected-corpus.json` het corpusmanifest; `payload-mapping.json` de via dump-tree afgeleide en bytegewijs geverifieerde logische/fysieke mapping; `sha256-manifest.json` controleert de bewijsbestanden. `baseline.img` is een kopie voor uitsluitend read-only native afwijzings-/integratieproeven. De overige disposable images blijven in /tmp/nieuw-os-storage-btrfs-20261102.

De testprogramma's zijn achtereenvolgens probe.py, experiment.py, faults.py en finalize.py. Ze weigeren bestaande seed/corruptiebeelden te overschrijven; een herhaling vraagt een apart gekozen eigen testdirectory. `probe.py` onderzoekt lokale help voordat experimentflags worden gebruikt.

Primaire documentatie:
- https://btrfs.readthedocs.io/en/latest/mkfs.btrfs.html
- https://btrfs.readthedocs.io/en/latest/btrfs-check.html
- https://btrfs.readthedocs.io/en/latest/btrfs-restore.html
