# Kernel rules

Read the root `AGENTS.md`, `docs/ARCHITECTURE.md`, and `docs/SECURITY-AND-QUANTUM.md`.

Use Rust `no_std`. Each `unsafe` block requires a local SAFETY justification addressing validity, aliasing, alignment, and lifetime. A comment is not proof; it must withstand counterexamples and review.

Describe ownership of frames and page tables, exception and syscall ABIs, interrupt state, and lock order. Validate user pointers and lengths. Do not retain references to unmanaged guest/user-space memory across rescheduling without a valid ownership model.

Capabilities are process-scoped, kernel-validated handles with rights and validity. Raw integers/pointers are not unrestricted authority. Test privilege escalation, stale handles, exhaustion, and revocation.

Keep the privileged kernel small. Do not add a GUI, network parser, cryptographic protocol, or storage policy to privileged code without an explicit ADR. Do not expand SMP support before single-core invariants have been demonstrated.
