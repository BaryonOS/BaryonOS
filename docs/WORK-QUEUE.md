# Werkqueue — M5 blokdienst en virtio

Gestart 27 oktober 2026 vanaf `c630c59`, branch `codex/m5-block-service`. M1–M4-basis en bewijs behouden; vorige queue in verification/m2-m4/WORK-QUEUE.md.

Volledige eindcriteria vóór implementatie: implementation/M5-BLOCK-SERVICE.md. READ-only echte virtio-blk-I/O, geïsoleerde userdienst, typedREADcaps, nonblocking bounce-DMA, gecontroleerd reset/herstart/quarantaine,17negatieve cases+twee normaleimages, regressie/review/hash/privatepush. Geen claim van IOMMU-/maliciousdevice-containment.

| ID | Eigenaar | Exclusieve writes | Status | Afhankelijk/bewijs |
|---|---|---|---|---|
| G05 | root | rootconfig/tools/docs | actief | preflight strict exit0,basis schoon behalve bestaande .codex/environments/ |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | eerste bronversie | nog geen kernelbuild of QEMU-resultaat |
| C05 | quantum_contract | virtio-core/,capability-core/ | hostcontroles geslaagd | 26 capabilitytests, 12 virtio-tests; fmt/Clippy/no_std exit0 |
| U05 | product_acceptance | services/block/,M5userspace | ELF gebouwd | 9 protocoltests en rustc-build exit0; guestgedrag nog niet getest |
| R05 | quantum_contract/product_acceptance | alleen lezen | actief | transport-/runtime-review, concrete tegenvoorbeelden en herstel |
| V05 | root | tools/docs/verification/m5/ | wacht | volledige host/QEMU/regressie/eindmatrix en remotehead |

Root beheert interfaces en buildslot; maximaal3 actieve agents/Sol. Geen gelijktijdige schrijvers. Tussentijdsbron/bewijs/openpunten committen en privépushen, ook vóór lange tests. Geen publieke release, hostdisks, hardware, NAS, productiegegevens of configwijziging.

Laatste checkpoint: eerste geïntegreerde bronversie met hosttests, user-ELF en extern schijf-orakel. Driver-TCB bestaat uit klein kerneltransport en protocolconform QEMU-device; userdienst heeft geen MMIO-/DMA-rechten. Nog geen geslaagde M5-kernelbuild of QEMU-run. Volgende actie: compilecorrecties en echte QEMU-proeven; daarna volledige regressie en review.
