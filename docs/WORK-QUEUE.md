# Werkqueue — M6a actief

27 oktober 2026, basis `448974b`, branch `codex/m6a-write-recovery`. Gekozen invariant/ABI/cases vóór implementatie: implementation/M6A-WRITE-RECOVERY.md.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K06 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | bron bevroren; 16hosttests en gerichte QEMU/cut/recoveryproeven groen |
| C06 | quantum_contract | capability-core/,virtio-core/ | 30+21tests/fmt/Clippy/no_std exit0; bron gepauzeerd |
| U06 | product_acceptance | services/block/,nieuweM6-userspace | 20host+12artifacttests exit0; echte dienstbron gepauzeerd |
| G06 | root | config/tools/tests/docs/Git | guarded writableimage/crashharnas/capture,19hostcontroles groen; volledige poort gereed |
| R06 | C06/U06 | alleen lezen | gerichte transport- en zes harnas/capturebevindingen hersteld/gesloten |
| V06 | later verifier | alleen lezen | finale M1–M6-poorten, echte cuts/recovery en hashbinding |

Eén schrijver per bestand, maximaal drie agents en één gedeeld build/QEMUslot. Geen nieuwe externe dependency. Na elke afgebakende stap bron/bewijs/open punten privépushen; noreplyafzender alleen per Git-commando, geen globale configwijziging. Gerichte M6-proeven groen; finale 54groepen/49VM-poort en regressie nog uit te voeren. QEMU-procescrash is geen fysieke stroomuitvalclaim.

## M5 — behaalde basis (historisch)

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
