# Btrfs metadataherstel — aparte aanvullende hostproef

Resultaat: inhoudherstel slaagt; deze mkfs --rootdir + btrfs restore-route bewaart niet alle metadata. Alle vier toolcommando's hebben exitcode 0 en de bronimage blijft bytegelijk. Baseline en N1-fixture zijn niet gewijzigd.

| Eigenschap | Na mkfs --rootdir, gecontroleerd met dump-tree | Na restore -m -x -S |
|---|---|---|
| Bestandsbytes, grootte, symlinktarget | Contentchecksums geverifieerd; volledige bytesvergelijking na restore | Alle inhoud gelijk |
| Gecontroleerde modes en user.* file-xattr | Behouden | Normale files behouden |
| mtime inclusief 123456789 nanoseconden | Fractie verloren; alle mtimes hele seconden | Hele seconden behouden behalve lege map |
| Lege map mode0700, oude mtime, user.* xattr | Alle drie aanwezig, tijd tot hele seconden | Mode0755, actuele mtime, xattr ontbreekt |
| Hardlinkpaar | Eén inode, linkcount2, twee namen | Twee losse inodes, linkcount1 |
| Sparse file8MiB, slechts8KiB oorspronkelijk gealloceerd | Alle8MiB gealloceerd in acht extents | Alle8MiB gealloceerd; inhoud gelijk |
| uid/gid | Gelijk aan huidige WSL-gebruiker1000 | Gelijk; remapping naar andere gebruikers niet getest |

Er zijn8 padvermeldingen (inclusief2 directories en1 symlink). Mode, mtime-nanoseconden, uid/gid, user.xattrs, filetype, reguliere grootte/SHA256 en symlinktarget zijn vergeleken. Hardlinktopologie is apart vergeleken op gelijke inode+linkcount, zonder inodegetallen tussen verschillende filesystems gelijk te eisen. Sparse-allocatie is geobserveerd met st_blocks op dezelfde WSL-/tmp-backingstore en Btrfs nbytes/extents; geen performanceclaim.

Deze verliezen zijn aan import of restore toegeschreven door de tussenliggende Btrfs-tree te inspecteren. Dit bewijst beperkingen van deze toolroute, niet dat het Btrfs-diskformaat de eigenschappen niet kan opslaan.

Niet getest: ACLs, security.xattrs, creation/ctime/atime, rootdirectory-metadata, uid/gid-remapping, Windows ADS, resource forks, mounted driver, native Nieuw OS, encryptie, snapshots of fysieke stroomuitval.

metadata-evidence.json bevat de3 stadia en verschillen; logs/ bevat volledige stdout/stderr. De image blijft in /tmp/nieuw-os-storage-btrfs-20261102/metadata/metadata.img. De bovenliggende metadata_probe.py en metadata_review.py beschrijven corpus, commando's en vergelijking. Geen herstel-fallback of herhaalde succesvolle substituutrun gebruikt.
