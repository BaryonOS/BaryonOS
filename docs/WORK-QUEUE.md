# Werkqueue — M5 blokdienst en virtio

Gestart27 oktober2026 vanaf c630c59, branch codex/m5-block-service. M1–M4-basis en bewijs behouden; vorige queue in verification/m2-m4/WORK-QUEUE.md.

Volledige eindcriteria vóór implementatie: implementation/M5-BLOCK-SERVICE.md. READ-only echte virtio-blk-I/O, geïsoleerde userdienst, typedREADcaps, nonblocking bounce-DMA, gecontroleerd reset/herstart/quarantaine,17negatieve cases+twee normaleimages, regressie/review/hash/privatepush. Geen claim van IOMMU-/maliciousdevice-containment.

| ID | Eigenaar | Exclusieve writes | Status | Afhankelijk/bewijs |
|---|---|---|---|---|
| G05 | root | rootconfig/tools/docs | actief | preflight strict exit0,basis schoon behalve bestaande .codex/environments/ |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | bouw actief | contractv1,geen nieuwe run geclaimd |
| C05 | quantum_contract | virtio-core/,capability-core/ | bouw actief | getypeerde caps/driverpolicy,tests nog niet uitgevoerd |
| U05 | product_acceptance | services/block/,M5userspace | bouw actief | echte3rollen/protocol,tests nog niet uitgevoerd |
| R05 | latere onafhankelijke reviewer | eerst alleen lezen | wacht | concrete negatieve tegenvoorbeelden en herstel |
| V05 | root | tools/docs/verification/m5/ | wacht | volledige host/QEMU/regressie/eindmatrix en remotehead |

Root beheert interfaces en buildslot; maximaal3 actieve agents/Sol. Geen gelijktijdige schrijvers. Tussentijdsbron/bewijs/openpunten committen en privépushen, ook vóór lange tests. Geen publieke release, hostdisks, hardware, NAS, productiegegevens of configwijziging.

Laatste checkpoint: M5-invarianten en ownership vastgesteld, nieuwe branch aangemaakt; builders begonnen. Driver-TCB bestaat uit klein kerneltransport en protocolconformQEMU-device; userdienst heeft geen MMIO/DMArechten. Geen fictief groen bewijs. Volgende rootactie: workspace/lock/ELFbuild/diskharnas integreren en eerste sourcecheckpoint bewaren.
