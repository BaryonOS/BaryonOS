# Werkqueue — M6a gestart

28 oktober 2026. Basis `5a58a74`, branch `codex/m6a-write-flush-core`.
Contract: implementation/M6A-WRITE-FLUSH.md. Eerste stap bevat uitsluitend
coreplannen; volledige schrijvende opslag/crashherstel blijven open.

| ID | Eigenaar | Gebied | Status / bewijs |
|---|---|---|---|
| A06/B06 | m6a_architect_review | crates/virtio-core/ | coreplannen klaar; 19 hosttests/fmt/Clippy/no_std exit 0 |
| E06 | m6a_environment | projectlokale .tools/, build/ | QEMU/OVMF ingericht; strict preflight exit 0 |
| G06 | root | docs/tools/Git/integratie | contract en broncheckpoint; geïntegreerde eindcontroles volgen |
| R06 | m6a_environment | alleen lezen | onafhankelijke bronreview/externe hostproeven lopen |

## Bewaarde M5-queue

27 oktober 2026. Bron `7af544a`, basis `c630c59`, branch `codex/m5-block-service`, concept-PR#3. Definitieve fingerprint v3: c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307.

| ID | Eigenaar | Gebied | Status / bewijs |
|---|---|---|---|
| G05 | root | config/tools/docs/Git | geïntegreerd; preflight exit0, private checkpoints, einddocs |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | gereed; M5 25 groepen/20 VM exit0 |
| C05 | quantum_contract | virtio-core/,capability-core/ | gereed; 12/26 tests, fmt/Clippy/no_std exit0 |
| U05 | product_acceptance | services/block/,M5-userspace | gereed; 9 protocol- en 11 artifacttests exit0, echte gastdienst |
| R05 | twee reviewers | alleen lezen | herstelpunten gesloten; ongeïnjecteerde randen expliciet |
| V05 | root/verifier | ruwe evidence/hashbinding | 77 groepen/64 VM plus M5-eindstart; alle 65 VM en 279 artifacthashes geaudit |

Opdrachten, resultaten, grenzen en hervatroute: PROJECT-STATE.md en verification/m5/ACCEPTANCE.md. Eerste mislukte gate blijft failed bewaard; de finale bron is opnieuw getest. Bestaande UI/quantum en M1–M4-baseline behouden.

Volgende stap: M6a, veilige WRITE/FLUSH en crashherstel op een synthetische wegwerpimage. Nog niet gestart. Geen echte hardware/disks/NAS/keys, betaalde infrastructuur, publieke release of merge zonder passende opdracht. Licentie vóór publicatie blijft open.
