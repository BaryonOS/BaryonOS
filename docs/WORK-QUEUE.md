# Werkqueue — M5 blokdienst en virtio

Gestart 27 oktober 2026 vanaf `c630c59`, branch `codex/m5-block-service`. M1–M4-basis en bewijs behouden; vorige queue in verification/m2-m4/WORK-QUEUE.md.

Volledige eindcriteria vóór implementatie: implementation/M5-BLOCK-SERVICE.md. READ-only echte virtio-blk-I/O, geïsoleerde userdienst, typedREADcaps, nonblocking bounce-DMA, gecontroleerd reset/herstart/quarantaine,17negatieve cases+twee normaleimages, regressie/review/hash/privatepush. Geen claim van IOMMU-/maliciousdevice-containment.

| ID | Eigenaar | Exclusieve writes | Status | Afhankelijk/bewijs |
|---|---|---|---|---|
| G05 | root | rootconfig/tools/docs | actief | preflight strict exit0,basis schoon behalve bestaande .codex/environments/ |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | reviewherstel actief | eerste normale kernelbuild/QEMU-read raw33; volledige poort nog open |
| C05 | quantum_contract | virtio-core/,capability-core/ | hostcontroles geslaagd | 26 capabilitytests, 12 virtio-tests; fmt/Clippy/no_std exit0 |
| U05 | product_acceptance | services/block/,M5userspace | hostcontroles geslaagd | 9 protocoltests +11 ELF-artifacttests exit0; eerste echte dienst-read geslaagd |
| R05 | quantum_contract/product_acceptance | alleen lezen | actief | transport-/runtime-review, concrete tegenvoorbeelden en herstel |
| V05 | root | tools/docs/verification/m5/ | wacht | volledige host/QEMU/regressie/eindmatrix en remotehead |

Root beheert interfaces en buildslot; maximaal3 actieve agents/Sol. Geen gelijktijdige schrijvers. Tussentijdsbron/bewijs/openpunten committen en privépushen, ook vóór lange tests. Geen publieke release, hostdisks, hardware, NAS, productiegegevens of configwijziging.

Laatste checkpoint: eerste geïntegreerde bron op GitHub in `2bb856b`, concept-PR#3. Eerste normale QEMU-read geslaagd; vroege evidence in verification/m5/checkpoints/. Driver-TCB bestaat uit kerneltransport en protocolconform QEMU-device; userdienst heeft geen MMIO-/DMA-rechten. Review vond vijf concrete herstelpunten; volledige foutpoort/regressie nog open. Volgende actie: gericht herstel, volledige M5-poort en samengestelde regressie.
