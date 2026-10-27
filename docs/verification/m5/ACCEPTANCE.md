# M5-acceptatie — definitief bewijs

27 oktober 2026. Bron `7af544aa1b8d731ac27f384f0ba95aa44586a57d`, basis `c630c59`. Fingerprint v3: `c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307`. Eigen kernel, één CPU, qemu64/TCG, 256 MiB, geen gastnetwerk. Geen fysieke hardwareclaim.

## Werkelijk geslaagd

| Laag | Groepen | Echte VM-runs | Bewijs |
|---|---:|---:|---|
| M1 volledige bootfoutreeks | 10 | 8 | regression/m1/ |
| M2 geheugen/exception | 8 | 7 | regression/m2/ |
| M3 processen/preëmptie | 22 | 19 | regression/m3/ |
| M4 capabilities/IPC | 12 | 10 | regression/m4/ |
| M5 systeemdienst/virtio | 25 | 20 | suite.json, guest-runs.json |
| **Definitieve suites** | **77** | **64** | validation.json |
| Extra normale M5-CLI-start | — | 1 | manual-m5.json |

Alle genoemde opdrachten eindigden met exit0; exacte opdrachten en gewijzigde bronbestanden staan in validation.json. De eerste gecombineerde gate eindigde met exit1 door een verkeerde scopefoutcode en blijft als failed bewaard onder diagnostics/. Alleen de finale hertests staan in bovenstaande aantallen. Verwachte M1-panic/timeout/fouten behouden hun negatieve guestuitkomst; de buitenste suite slaagt op de juiste foutafhandeling.

## M5-contract en cases

Alle onderstaande cases hebben echte raw-exit33, geen timeout, exacte casemarkers, overlevende peers en resourcecontrole. Foutvarianten zijn expliciet test-only. Volledige commands, serial, stderr en hashes staan per run in guest-runs.json en de daar genoemde lokale build/run-mappen.

| Eis / case | Uitgevoerd bewijs |
|---|---|
| Normale READ / extern orakel | Dezelfde kernel/loader op twee verschillende werkelijke 64KiB-images. Ontvangen FNV-checksums van sector4/6 gelijk aan hostberekening; image-SHA256 ongewijzigd. Geen seed/antwoord in guestconfig. m5-two-image-check.json |
| service-errors | 13 echte malformed requests en exacte lege foutreplies. Kernel controleert nul blockcall-/submission-/completiondelta, masks13/13 en twee daadwerkelijke capcloses zonder lek |
| device-rights | Verkeerd objecttype, ontbrekende rechten, buitenlandse/ongeldige handle, sectorscope, grenslengtes, verkeerd/hergebruikt ticket en ongeldig output. Geen verbruik bij EFAULT; latere goede poll slaagt |
| async | Vertraagde notify, echte EAGAIN/PIT/peerprogress, daarna devicecompletion |
| bad-descriptor | Ongeldige draft vóór publicatie/notify geweigerd; geen onveilige DMA gepubliceerd; herstel |
| used-id/index/short/long/status | Lokale snapshots na echte devicecompletion geïnjecteerd; ongeldige metadata afgewezen vóór usercopy; reset/herstel |
| io-error | Bekende IOERR-status geeft EIO zonder payload; herstel |
| timeout | Notify onderdrukt, echte tickdeadline, bevestigde reset en latere succesvolle reads |
| reset-timeout | Resetack testmatig niet waargenomen: offline, gepinde pool in quarantaine, peerprogress en veilige cleanup; geen datadigests geclaimd |
| restart | Drie echte pending-read-dienstcrashes, nieuwe PID/epoch, resetack vóór hergebruik, echte IPC-rebind, oude autoriteit ingetrokken, client/supervisor PID+CR3 behouden; epoch4 leest succesvol |
| cpu-mmio/dma/pci | Echte CPL3-read van supervisor-MMIO/pool of PCI-out veroorzaakt alleen attackerfault; dienst/client blijven werken |
| no-device | Testdevice daadwerkelijk niet gekoppeld, boot-ESP blijft ander device. ENODEV, geen submission, peerprogress, geen datadigests |

Hostlaag: 26 capability-, 15 kernel-core-, 12 virtio- en 9 protocoltests; 11 daadwerkelijke ELF-artifacttests, 4 harnasparsertests en 5 negatieve schijf-oracletests. Cores: fmt/Clippy/no_std exit0, exact afgebakend in core/report.json. De algemene type/refcount/revoke/quota/generation- en pure PCI/DMAplan/ticket/epochgrenzen zijn hostbewijs, geen kwaadaardige-hardwarebewijs.

## Reviews en audit

Transportreview en service/user-review beschrijven concrete bevindingen en herstel. De afzonderlijke verifier controleerde alle65 definitieve runs en279 artifacthashes, de daadwerkelijke schijfbytes, sourcebinding en negatieve M1-semantiek. Rapporten: m5-verifier-guest.json en m5-verifier-final.json. Finale kernel/loader zijn bytegelijk aan beide normale gate-runs en de extra CLI-start.

UI9/9 en quantum15/15 bron-/binaryhashes zijn ongewijzigd: m5-verifier-prototypes.json. Tests daarvoor zijn niet herhaald. AI-reviews vervangen geen menselijke audit.

## Grenzen en volgende stap

De dienst is CPU-geïsoleerd; kerneltransport en protocolconform QEMU-device/backend blijven vertrouwd. Geen IOMMU of containment van kwaadwillige DMA. De vijf poolpagina's blijven gepind; geen hergebruik zonder completion of bevestigde reset, geen vrijgave bij quarantaine. Een used-entry bevat geen epoch/nonce; softwaretickets bieden geen hardware-replaybewijs.

DRIVER_OK-readbackfout, device1-scope en exit na resetack vóór terminale foutpoll zijn bronmatig hersteld/gereviewd, zonder afzonderlijke gastinjectie. Geen SMP, fysieke hardware, schrijvende opslag, bestandssysteem, Secure Boot/PQC of productiegeschiktheid bewezen.

Volgende stap M6a: WRITE/FLUSH op een synthetisch wegwerpvolume en crashherstelproeven. Nog niet gestart. Herstel via een aparte consistente checkout van `c630c59`; bestaande baseline en gebruikerswerk behouden.
