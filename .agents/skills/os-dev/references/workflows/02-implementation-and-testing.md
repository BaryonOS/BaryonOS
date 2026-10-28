# Implementation and testing

Use for a requested OS feature or continued implementation. Combine with the
subsystem-specific checklist. This workflow does not authorize publishing or merging.

## Establish reality

Read applicable AGENTS.md files, the current branch and working-tree status, project
state/ADRs, build scripts, target descriptions and tests. Use real repository reads.
Do not infer current implementation from a historical project document alone. Preserve
unrelated modifications and use the repository's tool versions and build entrypoints.

Identify which environment can actually execute code. Host-side Python/Rust/C tests,
freestanding target compilation and an emulator run are different capabilities.
Without execution, write the patch or test plan but label every unrun test honestly.

## Define and implement one vertical slice

Write the [task contract](../../assets/task-brief.md): observable behavior, interfaces,
ownership, errors, resources, source IDs and affected files. Choose the narrowest slice
that advances the user's goal without hiding a broken dependency behind a stub.
Keep temporary scaffolding explicit. Avoid broad reorganizations unrelated to the goal.

Use source knowledge to justify the design rather than pasting a tutorial wholesale.
Keep unsafe/assembly boundaries small, document preconditions, and check representation,
alignment and ABI assumptions. Do not import a host runtime into a freestanding target
accidentally. Specify panic/failure behavior, cleanup and partial-initialization rollback.

## Verification ladder

- **Static/build checks:** formatting/lint where configured, compile for the real target,
  check warnings, inspect the linked artifact and relevant ABI/linker properties.
- **Host/unit checks:** arithmetic, parsers, allocators and state machines using isolated
  inputs; distinguish modeled hardware from actual privileged execution.
- **Integration/emulator checks:** reproducible boot/run with explicit machine/CPU,
  firmware, memory and device configuration. Capture serial output, a timeout and an
  observable completion condition; a process that hangs until timeout is not a pass.
- **Hardware/formal checks:** claim only what was actually executed or proved. Record
  machine and assumptions or theorem/tool versions. Do not infer this level from QEMU.

Select applicable levels; report unavailable levels. Add negative cases for boundaries,
wrong state, insufficient resources and interrupted/failed initialization. An expected
fault is a pass only if the harness confirms the expected fault and isolation behavior.

## Integrate and report

Read the final diff, remove accidental artifacts/secrets, update relevant docs and
preserve source notices. Report exact commands, results, remaining limitations and a
rollback using [the report template](../../assets/change-report.md). Re-run affected
checks after the final modification, not just before it.

Parallelize only with real available agents/tools and explicit file ownership. Freeze
shared interfaces, track dependencies, and have one integrator run cross-boundary tests.
Do not describe a suggested delegation as a launched agent.
