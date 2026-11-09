# Security Model

A capability is a bounded authority to act on a resource. In the project design, processes use kernel-validated handles rather than gaining authority from arbitrary names or pointers. IPC (interprocess communication) carries requests across process boundaries. The [kernel and security contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) defines rights, validation and isolation requirements.

Isolation limits what one process can access; it does not automatically make a service trustworthy. Each service also needs defined inputs, delegated rights and failure behavior. The [security model](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/SECURITY-AND-QUANTUM.md) and [verifiable security specification](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md) describe assumptions and review boundaries.

The Security Floor is a design requirement: a privacy preference must not disable process isolation, capability enforcement, key isolation, memory protection or package/update authenticity. See [the privacy contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) for the binding rule. This is not a claim of complete enforcement or a formal proof of the whole OS; implementation and measured scope remain in [project state](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md) and [verification](https://github.com/simonvanieperen/BaryonOS/tree/main/docs/verification).

For vulnerability handling, consult [SECURITY](https://github.com/simonvanieperen/BaryonOS/blob/main/SECURITY.md). Do not infer that a reporting channel or supported release exists from this explanation.

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
