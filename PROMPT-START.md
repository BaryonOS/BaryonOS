# Copy from the next line
Act as the lead developer of an independent open-source operating system. Initial research and design decisions have already been completed. **Start implementing now; do not restart a general research or concept-selection phase.**

Read AGENTS.md, docs/PROJECT-STATE.md and docs/implementation/START-CODING.md. Then read docs/implementation/FIRST-BATCH.md and only the specifications relevant to the task. The implementation decisions are the current build baseline; older research documents provide background. Explicit user requirements remain authoritative.

Fixed product boundaries:

- An independently developed kernel and high-quality product experience; no Linux distribution or existing kernel as the foundation.
- AI inference exclusively on the device, without network permission or cloud fallback. External apps use separate permissions enforced per app; see docs/LOCAL-PRIVACY-CONTRACT.md for storage and NAS requirements.
- Quantum: post-quantum resilience, local quantum execution and a future independently developed OS control layer for hybrid quantum hardware.
- Open source, accessible, configurable and independent of mandatory big-tech services. Use the selected product direction; no new concept selection is required.

Execute the first batch now:

1. Check the actual project directory, Git status and applicable instructions. Preserve existing changes. Run tools/preflight.py and make the selected development environment usable within the available permissions. Inspect what actually exists; do not invent test results.
2. Assign bounded work to no more than three builders: B01 for the project's own boot path, loader and kernel; P02 for the selected interactive product prototype; and Q01 for the local quantum simulator. Follow FIRST-BATCH.md for exclusive write areas. Manage shared root configuration and tools centrally; coordinate interface versions. B01 owns the boot-protocol and ELF-parser crates.
3. Write source code and appropriate tests. Use the concrete contracts, schemas, limits, screens and test cases in the specifications. Consult primary documentation only when a specific API, error or incompatibility requires it.
4. Actually execute the build, emulator test and prototype and simulator tests. If a tool or permission is missing, record the precise blocker and complete independent work. Ask at most one focused question if a necessary decision genuinely cannot be resolved independently.
5. Integrate, arrange review, fix errors and check the combined version. Update PROJECT-STATE.md with the code, actual results and remaining concrete blockers.

Perform local implementation and testing autonomously. No public push or release, paid infrastructure, real disks, firmware flashing, NAS modifications or production keys without a separate instruction. Use synthetic data. The first batch does not yet deliver a production OS or hardware-QPU support.

Deliver code and a short report covering working components, startup instructions, tests actually executed, remaining limitations and the next implementation step. Do not stop after a plan or a new set of research assignments.
