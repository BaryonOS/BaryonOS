# First execution batch — research done, now implement
Revision 5, 26 October 2026. These assignments use the chosen implementation specifications under docs/implementation/. They do not repeat a broad architecture or concept exploration.

## Division of work
The main agent reads START-CODING.md, checks existing user changes and manages shared configuration. After that, at most three builders can work independently:
- B01: boot/ and kernel/ plus crates/boot-protocol/ and crates/elf-loader-core/; changes to root build files and tools/ go via the main agent.
- P02: exclusively ui/prototype/.
- Q01: exclusively experiments/quantum/ with its own Cargo workspace; the root workspace excludes this directory.

New source code is still missing; these paths are to be created. The main agent owns root Cargo.toml, rust-toolchain.toml, root Cargo.lock and tools/. B01 owns the boot-protocol/ELF parser crates according to the agreed ABI. Q01 owns only its nested Cargo.lock. Each shared file has one owner. Do not use concurrent writers for those files. Review and test after integration, not only per agent.

## B00 — prepare the actual development environment
**Input:** BOOT-AND-TOOLCHAIN.md and tools/preflight.py.
**Action:** run the read-only inventory; compare versions, add missing tools within the available permissions and verify the chosen targets/firmware. Record actual versions and firmware hash. Do not replace existing user config.

**Done:** build tooling works or there is an exact concrete blocker with evidence. A blocker for QEMU does not block P02 and need not block Q01 if a Rust host toolchain is available.

## B01 — own loader hands over to own kernel
**Input:** BOOT-AND-TOOLCHAIN.md, kernel contract and applicable AGENTS.md.
**Action:** implement the agreed ELF loader, BootInfo, handoff after firmware boot services, kernel entry and serial smoke test. Follow the fixed baseline; only open targeted documentation for a concrete implementation error.

**Done:** kernel marker and correct emulator status on a real run; corrupt/missing kernel and timeout are reported as failure. A firmware banner or cross-compilation alone does not count as a successful kernel boot. Logs state commit/base, commands, versions and result.

## P02 — experienceable prototype of the chosen product direction
**Input:** PRODUCT-SPEC.md.
**Action:** code the screens, state transitions, tokens and keyboard operation chosen there. No new round of concept selection. Use local fictitious fixtures. No cloud, tracking, external fonts or real NAS files.

**Done:** the agreed core tasks have been walked through interactively; error/empty/loading states and accessibility checks are present. Test desktop and the smaller viewport from the specification. Label this as a product prototype; it does not yet run on the own kernel.

## Q01 — local quantum trial
**Input:** QUANTUM-SPEC.md.
**Action:** implement the fixed small simulator profile, the job interface and limits. No new language/backend selection and no cryptographic implementation in this task. Keep the simulator core separate from host JSON/CLI.

**Done:** reference circuits, invalid requests, quotas and cancellation pass the specified tests. Report host simulation separately from the future OS port/QPU. No network or quantum cloud.

## Integration and closing
1. Main agent checks the actual changed files and shared interface versions.
2. Have a reviewer look for concrete errors and missing evidence; review does not turn unexecuted tests green.
3. Run relevant checks on the combined state, fix errors and repeat only the affected tests.
4. Update PROJECT-STATE.md with what was implemented/run/not run.
5. Close this batch with the next bounded implementation task; do not generate a new general research assignment.

No publication, real disks, firmware flash, NAS configuration changes or release keys in this batch. This is about local source code, prototypes and isolated tests.

## Concrete design change during building
If a chosen contract demonstrably turns out to be wrong: record the error, source/test and smallest corrective choice; coordinate dependent files and continue. A good implementation does not blindly follow a wrong document. 'I want to compare all options again first' is not a blocking reason.
