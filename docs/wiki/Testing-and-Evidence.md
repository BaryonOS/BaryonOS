# Testing and Evidence

Evidence answers a particular question for a particular source version and environment. The [test strategy](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/TEST-STRATEGY.md) defines the layers and their limits.

| Evidence | What it can establish | Boundary |
|---|---|---|
| Host tests | Behavior of tested libraries, tools or prototypes on the development OS | Does not establish execution under the project kernel |
| Build checks | That the selected sources compile under the selected configuration | Does not establish runtime behavior |
| QEMU tests | Native guest behavior in the recorded virtual configuration | Does not establish physical-device support |
| Physical hardware tests | Behavior on the exact tested hardware and configuration | Does not generalize to all PCs |
| Human review | Findings within a stated review scope | Is distinct from automated tests or AI review |

The [verification directory](https://github.com/simonvanieperen/BaryonOS/tree/main/docs/verification) holds version-bound results, logs and acceptance reports. The [release contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/FULL-TEST-RELEASE.md) specifies required evidence; [PROGRESS](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md) explains when a criterion counts. A plan, simulation, successful compilation or another AI's review must retain its own label. No test totals or pass/fail register are maintained in this wiki.

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
