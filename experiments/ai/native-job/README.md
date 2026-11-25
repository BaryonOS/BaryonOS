# Native SSE2 AI job

This is the first ring-3 AI process using compiler-generated SSE2. It runs a tiny
two-layer Qwen2-style decoder fixture (the `reference-forward` engine on generated
BF16 weights) repeatedly under PIT preemption and publishes the logits' bit patterns
in its private report page. A host run of the same code (`ai-job-expected`) gives
the expected bits. Any difference indicates FPU/SSE state corruption or a
numeric-profile mismatch.

- Target: `x86_64-unknown-linux-gnu`, used only for its hard-float SSE2 ABI. The ELF
  is freestanding and static, with no libc and no Linux syscalls. The only trap is
  `int 0x80`, using yield (1), exit (2) and monotonic ticks (3).
- `build.py` gate: the linked ELF has no undefined symbols. Every instruction with an
  XMM operand is SSE/SSE2. It rejects x87, MMX, SSE3+, VEX/EVEX, privileged,
  `syscall` and FXSAVE/XSAVE instructions. This is a bounded decoded-mnemonic check,
  not a formal proof.
- `mem*` and `rust_eh_personality` are supplied in `src/main.rs`, because the
  target's precompiled `core` expects them from libc.
- Capabilities: none. The process has no IPC endpoint or network authority.

Evidence limits: no native run has been recorded yet. A single CPU only, with no
SMP or sleep/wake claim. Full Qwen does not fit `USER_QUOTA` (8 MiB per process).
