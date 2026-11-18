# Repository Guide

The [repository introduction](https://github.com/BaryonOS/BaryonOS/blob/main/README.md) and [architecture](https://github.com/BaryonOS/BaryonOS/blob/main/docs/ARCHITECTURE.md) are the starting points for the source layout.

| Location | Reading purpose |
|---|---|
| [boot/](https://github.com/BaryonOS/BaryonOS/tree/main/boot) | Loader and boot handoff |
| [kernel/](https://github.com/BaryonOS/BaryonOS/tree/main/kernel) | Privileged kernel implementation |
| [crates/](https://github.com/BaryonOS/BaryonOS/tree/main/crates) | Shared contracts and reusable logic |
| [drivers/](https://github.com/BaryonOS/BaryonOS/tree/main/drivers) | Device implementations and bounded probes |
| [services/](https://github.com/BaryonOS/BaryonOS/tree/main/services) | System services and separately labeled host prototypes |
| [sdk/](https://github.com/BaryonOS/BaryonOS/tree/main/sdk) | App-facing interfaces, runtime and example apps |
| [ui/](https://github.com/BaryonOS/BaryonOS/tree/main/ui) | UI components and separately labeled prototypes |
| [tests/](https://github.com/BaryonOS/BaryonOS/tree/main/tests) and [tools/](https://github.com/BaryonOS/BaryonOS/tree/main/tools) | Tests, build tools and execution harnesses |
| [experiments/](https://github.com/BaryonOS/BaryonOS/tree/main/experiments) | Research implementations |
| [docs/](https://github.com/BaryonOS/BaryonOS/tree/main/docs) | Contracts, research, status and verification records |

Folder presence alone does not establish native integration. Follow component READMEs and [project state](https://github.com/BaryonOS/BaryonOS/blob/main/docs/PROJECT-STATE.md) for evidence boundaries. Before contributing, read [CONTRIBUTING](https://github.com/BaryonOS/BaryonOS/blob/main/CONTRIBUTING.md) and applicable [project instructions](https://github.com/BaryonOS/BaryonOS/blob/main/AGENTS.md). Licensing follows [LICENSING](https://github.com/BaryonOS/BaryonOS/blob/main/LICENSING.md); security reports follow [SECURITY](https://github.com/BaryonOS/BaryonOS/blob/main/SECURITY.md).

---

[Home](https://github.com/BaryonOS/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/BaryonOS/BaryonOS/blob/main/docs/PROJECT-STATE.md)
