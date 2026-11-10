# Test rules
Read the root instructions and docs/TEST-STRATEGY.md. Link every test to a requirement/invariant.
Boot logs must come from an actual emulator test; scripts must not fabricate expected success logs. The harness fails on timeout, a missing marker or an unexpected exit status.
Mocks demonstrate only the mocked layer. Explicitly identify checks that were not run.
Isolate images, seeds and temporary files. Do not use internet destinations, personal data or real disks. Compare performance only under recorded conditions.
Report failures without weakening tests to obtain a passing result.
