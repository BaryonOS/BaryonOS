# Eerste virtio-blk-transport

Deze driver hoort bij de M5-ontwikkelproef: QEMU q35/TCG, één CPU, exact PCI `00:05.0`, vendor/device `1af4:1042`, modern virtio 1.2 en een afzonderlijk readonly64KiB-testimage. De virtio-boot-ESP wordt niet geselecteerd. Er zijn uitsluitend reads, één splitqueue met acht descriptors en maximaal één uitstaande sectorread.

De driver en het protocolconforme QEMU-device/backend behoren tot de trusted computing base. Een echte ring-3-dienst gebruikt alleen een getypeerde READ-capability en begrensde submit/poll-syscalls. Userspace krijgt geen PCI-config, BAR/MMIO, busmaster-, descriptor- of DMA-adresmacht. Er is geen IOMMU-initialisatie of bescherming tegen een kwaadaardig device; CPU-isolatie en gecontroleerde aanvragen zijn daarvan afzonderlijke eigenschappen.

De kernel behoudt vijf afzonderlijke, gepinde supervisor-RW/NX-pagina's tot VM-stop: descriptors, available ring, used ring, request/status en bounce. Header16B is device-readable; bounce512B en status1B zijn device-writable. De gevalideerde keten is `0→1→2`. Alleen gecontroleerde BAR-subregio's krijgen supervisor-NX/UC-mappings, na controle van BAR-grenzen, PAT-UC en CPU-MAXPHYADDR. De paginapool is geen usergeheugen en gaat nooit terug naar de frameallocator.

Publicatie volgt na volledige descriptorvalidatie en memorybarrières. Completion gebruikt volatile snapshots en barrières; head/index/length/status worden gevalideerd vóór data wordt gekopieerd. Ongeldige metadata geeft lokaal een protocolfout zonder payload. Faultinjectie wijzigt uitsluitend een afgewezen draft of een lokale completion-snapshot; er wordt geen onveilig DMA-adres gepubliceerd.

Reset vraagt eerst virtio-status0 en wist PCI-busmasterenable. Busmaster-clear alleen bewijst geen beëindigde oude DMA. Queue/bounce worden pas na werkelijk waargenomen resetstatus0 opnieuw geïnitialiseerd. Ontbreekt bevestiging, dan blijft de pool in quarantaine en het device offline tot VM-stop. Een software-epoch/ticket staat niet in de gebruikte virtqueue en bewijst geen bescherming tegen kwaadwillige hardware-replay.

Vanaf de repositoryroot:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

De volledige M5-reeks, inclusief variantgebonden markers en externe image-oracle:

```powershell
python -u tools/test_m5.py
```

De normale smoke-CLI verwacht de READ-case. Negatieve features moeten via de variantbewuste gate worden gestart; alleen een raw33 is geen volledige testuitslag. Alle runs gebruiken eigen gegenereerde images onder `build/run/`, zonder netwerk of hostdevices. FNV-sectorchecksums zijn diagnostische testmetadata; SHA256 legt de ongewijzigde hostimage vast. Zie `docs/implementation/M5-BLOCK-SERVICE.md` voor ABI1.1, de 17 foutcases, scope en bewijsgrenzen.
