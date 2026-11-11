# What knowledge Codex and the developer need

Status: learning and research plan, 26 October 2026. A language model has no demonstrable competence as a kernel developer. Good outcomes require current specifications, concrete interfaces, executable tests and checking of assumptions. This file is a roadmap, not a substitute for the manuals it names.

## 1. Knowledge per work area

Use the matrix to prepare a task. The architecture choices and the build order are in `ARCHITECTURE.md` and `ROADMAP.md`; this file states which knowledge must demonstrably be available before those tasks.

| Work area | What the executing agent must understand | Evidence or knowledge file to deliver |
| --- | --- | --- |
| Git and collaboration | Commits, branches, worktrees, conflicts, review, rollback and file ownership. | Small reviewable change, diff and recovery instruction. |
| Rust | Ownership, borrowing, lifetimes, generics, error handling, `no_std`, layout, FFI and the preconditions of `unsafe`. | A contract and targeted test for every unsafe boundary; reference to the applicable language rules. |
| CPU and memory | Registers, privilege, virtual/physical memory, page tables, exceptions, atomics and memory ordering. | Specific chapter from the chosen ISA manual; invariant and test on the chosen target. |
| Building and booting | Cross-compilation, object format, linker, boot protocol, firmware handoff and debug symbols. | Recorded toolchain, build commands, artifact identity and repeatable boot log. |
| Kernel mechanisms | Allocation, tasks, context switching, scheduling, IPC, capability rights and object lifetime. | Interface description plus permitted and rejected scenarios. |
| Concurrency | Interrupt context, races, deadlocks, lock ordering, cancellation and multicore behaviour. | Recorded lock and lifetime rules; stress and targeted regression tests. |
| Drivers | Device discovery, MMIO, interrupts, DMA, descriptor bounds, timeouts and recovery after faults. | Precise device/protocol specification and invalid-input tests. |
| Storage | Cache behaviour, persistent metadata, crash consistency, corrupt media, recovery and access. | Documented on-disk format, fault injection and recovery test with disposable images. |
| Networking | Framing, parsing, addressing, routing, name resolution, time, transport security and resource limits. | Protocol versions and limits; tests for malformed packets and exhaustion. |
| Security | Trust boundaries, authentication versus authorisation, keys, updates, provenance and attacker models. | Current threat model; every claim linked to a test, review or evidence. |
| Apps and SDK | ABI/API, versioning, packages, permission manifest, install/uninstall and developer tooling. | A small external example app that uses only the documented SDK. |
| Graphical environment | Composition, rendering, input, fonts, Unicode, accessibility, windows and user expectations. | Interaction prototype and later an integration test in the real OS; clearly separated. |
| Portability and energy | Architecture-dependent code, board support, firmware, sleep/wake and batteries. | Hardware matrix with measured supported functions; no generic device promise. |
| Verification | Unit/integration tests, property tests, fuzzing, fault injection, model checking and possibly formal proofs. | Test coverage and limitations; reproducible results, not merely a generated success message. |
| Product and management | UX, accessibility, localisation, privacy, release maintenance and community contributions. | Task list, user tests, privacy behaviour and maintenance policy. |

The matrix is deliberately broad: a usable OS is also an app platform, a driver project and a maintenance organisation. Codex can surface knowledge and make proposals. The missing empirical evidence must come from builds, tests and independent assessment.

### Product research and consumer adoption

For UX, onboarding, migration and positioning tasks,
[MARKET-AND-ADOPTION-2026-10-28.md](research/MARKET-AND-ADOPTION-2026-10-28.md)
and [CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
are the current research entry points. Keep **attention, trying, switching and continued
use** as distinct outcomes. Privacy, local AI, design and
European origin are not adoption guarantees on their own. Research
task performance, learning burden, required apps/files/devices, recovery and
voluntary reuse; design proposals remain hypotheses until suitable
user trials support them.

## 2. What belongs in the context before every development task

Do not give an agent all documents at once. Have it first read the relevant repository instructions and then only the source documents needed for the task.

1. **Goal and boundary:** one observable outcome, permitted files and explicit non-goals.
2. **Current state:** commit, target platform, versions, existing APIs and known defects.
3. **Normative source:** document name, version, chapter and URL; mark whether something is a standard, a vendor rule or a project choice.
4. **Invariant:** what must always remain true, for example “a revoked right grants no new access”.
5. **Acceptance:** concrete positive and negative cases, exact commands and the meaning of exit codes.
6. **Evidence:** keep the commands executed, relevant logs, outcome and limitations.
7. **Handoff:** what changed, what is still uncertain and what the next task can safely assume.

A reference to a document does not mean its entire content is present in the model's memory. For OS development tasks, `.agents/skills/os-dev/SKILL.md` is the local route to the bundled OSDev Wiki snapshot and *The little book about OS development*; have the agent retrieve only the relevant passage with `scripts/zoeken.py`. For other sources: have the agent retrieve the relevant passage or read a verified local copy. Record the version and checksum of downloaded specifications when the build or review depends on them. Do not store complete protected books in the repository without appropriate rights.

## 3. Personal learning route

This route runs alongside the OS project and has no fixed calendar duration. Move on once the exercise can be explained and repeated independently.

| Step | Exercise | Done when |
| --- | --- | --- |
| A. Controlling changes | Make a small text/Rust project, change it on a branch and recover from an error with Git. | A commit, diff and recovery action are understandable without copying the whole project folder again. |
| B. Rust basics | Build a small local program that reads input, validates it and handles errors. | Ownership, borrowing and `Result` can be explained using your own code. |
| C. Thinking in boundaries | Design a message format with a maximum length and validate it in an ordinary host test. | Malformed input causes a controlled error and no unbounded allocation. |
| D. From source to machine | Have Codex compile one function and explain the object file, symbols and relevant assembly. | The difference between source code, machine code, linker and runtime is clear. |
| E. Boot trial | Follow the first milestone from the roadmap in QEMU. | The exact build and boot are repeatable; an error log can be found again. |
| F. Understanding isolation | Use a diagram and tests to compare two processes with different rights. | A successful legitimate access and a rejected unauthorised access are demonstrable. |
| G. Reviewing | Have another agent review a small change and assess the test log yourself. | “The agent says it is correct” can be distinguished from reproducible evidence. |

For an unfamiliar concept, ask for three things: an explanation of at most ten sentences, a minimal example and what goes wrong if the assumption is incorrect. Let explanations support the current build step; do not try to read all OS literature first.

## 4. Maintaining active knowledge

Alongside the source code, keep track of:

- **Decisions:** ADRs with problem, options, chosen route, consequences and reconsideration criterion.
- **Interfaces:** syscall/IPC contracts, data structures, versioning policy and ownership.
- **Assumptions:** hardware capabilities, firmware, compiler, emulator and external services.
- **Evidence register:** claim, test ID, target, commit, result and remaining limitation.
- **Status:** last working commit, current blocker and next small task.
- **Dependencies:** name, version, provenance, licence, maintenance and reason for inclusion.

Update these documents when reality changes. An old chat is not an authoritative source for the current system. An executing agent may record a missing result as “not executed”; it may not fill in a passed test to complete a milestone.

## 5. First primary study sources

Read selectively. For Rust, the *Book* is a learning source; the *Reference* describes language rules and is not meant as a first introduction. Intel's system manual covers, among other things, memory management, privilege, interrupts and multiprocessor support. These sources do not determine product design; they bound what can be technically correct (Klabnik et al., n.d.; The Rust Project, n.d.; Intel, 2026).

| Topic | Starting point | Use |
| --- | --- | --- |
| Learning Rust | https://doc.rust-lang.org/book/ | First ownership, borrowing, enums, errors and tests. |
| Rust rules | https://doc.rust-lang.org/reference/ | Look up per task; tie to the pinned compiler. |
| x86_64 systems programming | https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html | Volume 3 for system mechanisms; record the relevant versions. |
| Open source | https://opensource.org/osd | Understand what the term does and does not mean. |
| Accessible web interface | https://www.w3.org/TR/WCAG22/ | Testable web criteria; do not equate with full OS accessibility. |

The sources consulted are not a complete technical library. Boot, driver, network, cryptography and hardware standards are selected per milestone in the other knowledge files. For a source that was not read or cannot be opened, the agent writes: “I cannot verify this.”

## Sources

Intel. (2026, October 9). *Intel 64 and IA-32 architectures software developer manuals*. https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html

Klabnik, S., Nichols, C., & Krycho, C. (n.d.). *The Rust programming language*. Retrieved October 26, 2026, from https://doc.rust-lang.org/book/

Open Source Initiative. (n.d.). *The Open Source Definition*. Retrieved October 26, 2026, from https://opensource.org/osd

The Rust Project. (n.d.). *The Rust Reference*. Retrieved October 26, 2026, from https://doc.rust-lang.org/reference/

World Wide Web Consortium. (2024, December 12). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/


## Addendum after the privacy correction
Read LOCAL-PRIVACY-CONTRACT.md for the binding scope. Additional knowledge: local NAS identity pairing and limited share rights, encrypted transport and client-side backup vault, key recovery, network transitions/IPv6/DNS rebinding, isolation of the AI runtime and model downloader, model file validation, combined file/network rights and restore points outside the authority of the sync account. Every implementation task names the relevant LP acceptance tests.
