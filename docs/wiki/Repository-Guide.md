# Repository Guide

The [repository introduction](https://github.com/simonvanieperen/BaryonOS/blob/main/README.md) and [architecture](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ARCHITECTURE.md) are the starting points for the source layout.

| Location | Reading purpose |
|---|---|
| [boot/](https://github.com/simonvanieperen/BaryonOS/tree/main/boot) | Loader and boot handoff |
| [kernel/](https://github.com/simonvanieperen/BaryonOS/tree/main/kernel) | Privileged kernel implementation |
| [crates/](https://github.com/simonvanieperen/BaryonOS/tree/main/crates) | Shared contracts and reusable logic |
| [drivers/](https://github.com/simonvanieperen/BaryonOS/tree/main/drivers) | Device implementations and bounded probes |
| [services/](https://github.com/simonvanieperen/BaryonOS/tree/main/services) | System services and separately labeled host prototypes |
| [sdk/](https://github.com/simonvanieperen/BaryonOS/tree/main/sdk) | App-facing interfaces, runtime and example apps |
| [ui/](https://github.com/simonvanieperen/BaryonOS/tree/main/ui) | UI components and separately labeled prototypes |
| [tests/](https://github.com/simonvanieperen/BaryonOS/tree/main/tests) and [tools/](https://github.com/simonvanieperen/BaryonOS/tree/main/tools) | Tests, build tools and execution harnesses |
| [experiments/](https://github.com/simonvanieperen/BaryonOS/tree/main/experiments) | Research implementations |
| [docs/](https://github.com/simonvanieperen/BaryonOS/tree/main/docs) | Contracts, research, status and verification records |

Folder presence alone does not establish native integration. Follow component READMEs and [project state](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md) for evidence boundaries. Before contributing, read [CONTRIBUTING](https://github.com/simonvanieperen/BaryonOS/blob/main/CONTRIBUTING.md) and applicable [project instructions](https://github.com/simonvanieperen/BaryonOS/blob/main/AGENTS.md). Licensing follows [LICENSE-PLAN](https://github.com/simonvanieperen/BaryonOS/blob/main/LICENSE-PLAN.md); security reports follow [SECURITY](https://github.com/simonvanieperen/BaryonOS/blob/main/SECURITY.md).

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
