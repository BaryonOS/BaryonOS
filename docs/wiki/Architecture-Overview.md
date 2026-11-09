# Architecture Overview

The [repository introduction](https://github.com/simonvanieperen/BaryonOS/blob/main/README.md) describes this simplified execution path:

```text
UEFI / OVMF → project loader → boot handoff → kernel
                                              ↓
                                   isolated processes / services
                                              ↓
                                         SDK / apps
```

The loader prepares the handoff to the kernel; the kernel manages privileged resources and process boundaries. Services perform bounded jobs, while apps use defined interfaces. This drawing is a reading aid, not a full dependency graph or a claim that every planned service is integrated.

The authoritative details are the [architecture](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ARCHITECTURE.md), [boot contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/BOOT-AND-TOOLCHAIN.md), [kernel and security contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) and [SDK runtime guide](https://github.com/simonvanieperen/BaryonOS/blob/main/sdk/runtime/README.md). Read them with [current implementation status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md): a dated design document alone is not implementation evidence.

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
