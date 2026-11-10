# Kernel rules
Read the root AGENTS.md and ARCHITECTURE.md and SECURITY-AND-QUANTUM.md under docs/.
Use Rust no_std; every unsafe block requires a local SAFETY justification covering validity, aliasing, alignment and lifetime. A comment alone proves nothing: subject it to counterexamples and review.
Describe ownership of frames/page tables, exception and syscall ABIs, interrupt state and lock ordering. Validate user pointers and lengths. Never retain references to unmanaged guest/userspace memory across rescheduling without a valid model.
Capabilities are process-bound, kernel-validated handles with rights and validity. Integers/pointers must not confer unrestricted authority. Test privilege escalation, stale handles, exhaustion and revocation.
Keep the kernel small. No GUI, network parser, cryptographic protocol or storage policy in privileged code without an explicit ADR. Do not extend SMP before single-core invariants have been demonstrated.
