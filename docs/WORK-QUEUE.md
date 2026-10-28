# Werkqueue — M6a observerherstel

28 oktober 2026, geïntegreerde basis `803bf43`, runtimeherstel `f60445b`, branch `codex/m6a-write-recovery`. Contract: implementation/M6A-WRITE-RECOVERY.md. Eerdere M6a-gate blijft als historisch bewijs onder verification/m6a/history/6fce5c12/.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K07 | builder | kernel/src/m2/m3/m6.rs | drie observergaten hersteld; crossbuild en M6a-gate exit0 |
| G07 | root | tests/docs/Git | 43 hostcontroles groen; acht echte QEMU-tegenproeven exit0; checkpoints gepusht |
| R07 | securityreviewer | alleen lezen | concrete bronreview zonder blokkerende bevindingen; geen menselijke audit |
| V09/V10 | verifier | alleen eigen rapport onder build/logs/ | audits exit0,nul fouten:122 VM/677artifacthashes;normale finalebinaries gelijk |
| I07 | root | gedeelde buildslot | M6a54groepen/49VM;M1–M5 77groepen/64VM;alle captures en normale M6a-start exit0 |

Eén schrijver per bestand, maximaal drie agents en één gedeeld buildslot in de hoofdwerkboom. De tegenproeven hebben een eigen externe wegwerpworktree, target en images. Geen nieuwe externe dependency. Bron/resultaten gaan per stap naar de privébranch; noreplyafzender alleen per Git-commando. QEMU-procescrash bewijst geen fysieke stroomuitval. Vervolg na afronding: observerherstel met de afzonderlijke vervolgbranch integreren en daar de gecombineerde poorten draaien; andere worktrees blijven behouden.

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

Volgende stap: de M6a-bronpoort en regressie hervatten vanaf deze checkpoint. De M6a-specificatie, huidige teststatus en open punten staan in PROJECT-STATE.md. Geen echte hardware/disks/NAS/keys, betaalde infrastructuur, publieke release of merge zonder passende opdracht. Licentie vóór publicatie blijft open.
