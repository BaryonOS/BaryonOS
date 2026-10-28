# Debugging, regressions and code review

Use [the testing/debugging route](../themas/22-testen.md) and the relevant subsystem
source. The objective is an evidence-based diagnosis or repair, not a speculative
rewrite.

## Establish the failure

Record the exact command, current revision and diff, target/toolchain, boot configuration,
expected outcome, actual outcome and last observable milestone. Obtain the smallest
reproduction available. Preserve logs before modifying code. Distinguish a failing
program from an unavailable build tool, bad test fixture or blocked environment.

For a boot fault, locate the failing phase: firmware/loader, entry ABI, linker/mapping,
descriptor/exception setup, allocation, user transition or device initialization.
Inspect real serial/debugger output and exception information. Do not diagnose a
triple fault merely because a window closed; preserve enough emulator evidence.

## Hypothesis loop

1. State a small number of hypotheses tied to observed evidence.
2. Choose the smallest diagnostic that distinguishes them.
3. Use bounded runs, targeted logging or the repository's debugger setup.
4. Make one coherent fix, reproduce the original failure condition, and run regression
   tests for the affected boundary.
5. Remove temporary workarounds or document why any remain. Re-run checks after cleanup.

Do not hide faults with long delays, disabled tests, ignored return values, suppressed
warnings or weaker permission checks just to make a boot message appear.

## Review format

Prioritize exploitable boundary violations/data loss, crashes or races, correctness,
then maintainability. For each substantive finding give the exact file/location,
condition, impact, supporting source or execution evidence, and a concrete correction
or test. State when a concern is conditional on an unverified assumption.

Check ABI/layout, overflows and ranges, lifetime, locking, cleanup, unsupported hardware,
source provenance and tests. A compile-only review does not establish runtime safety.
Do not invent CodeRabbit findings or external review status.

## Completion

Report the reproduction, established cause or remaining uncertainty, precise changes,
checks actually run, failed/unrun checks and rollback. When evidence is insufficient,
provide the smallest next diagnostic rather than claiming a root cause. No green CI,
merge or hardware success may be claimed without the corresponding observed result.
