# M6c final-read barrier

PR #361 integrates main #355/#359 while preserving containment. Case 2 previously
released the terminal retry probe at the tenth mutation ACK, before both clients'
final GET15/LIST16 could finish. Receipt of the expected successful LIST16 from each
domain now gates release; each client synchronously completes GET15 before LIST16.
A delayed second client provides the adversarial scheduling regression.

[Receipt](checks.json) binds the two changed files, pre-fix kernel, common delayed
probe and actual native result files. The old kernel fails with guest exit 35 and
user code 36; fixed errors and normal runs pass with guest exit 33 and actual disk
oracles. Both fixed sources remain unchanged through execution. Raw artifacts stay
local in `build/run/`; command logs stay in `build/tokenguard/`.

The initial attempt to run errors with the normal kernel was correctly refused
(exit 1, `run-vett_u4l.log`); this is a profile-selection error, not a native negative
control. The corrected `test-m6c-errors` profile produced the recorded negative.

49 M6c host tests and Rust formatting pass, exit 0. The host harness uses mocks;
its printed full-suite example is not a new execution of 35 native groups. One
focused read-only security review found no blocker in the barrier patch; no human
audit is claimed. No full matrix is repeated for this fixture-only correction.
Exact-head hosted checks remain required. Daybreak findings remain open.
