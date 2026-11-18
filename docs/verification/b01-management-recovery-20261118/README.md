# B01-T1 native management recovery

The normal keyring fixture reuses the existing app ELF for a separate management
principal named in the kernel manifest. Service A exports recovery through its
existing admin ingress; management rejects a wrong restore secret, restores into B
and is denied GENERATE. The ordinary app then unseals A's ciphertext through B.
Ordinary domain recovery remains denied. A test-only SEND alias cannot forge IPC
sender identity; it receives no ordinary-app reply. No service crypto or KEY1 change.

[Receipt](checks.json) binds the source and actual native result files. All five
existing QEMU cases pass with raw guest exit 33. Normal adds the six required
management/bad-caller counters; non-normal cases retain the three-process manifest.
Host tests from the existing gate and all 16 Python harness/crypto-contract tests pass.

The isolated one-line negative control admits the ordinary caller at poll_admin.
The same normal probe then times out at the 20-second diagnostic bound, produces no
case-pass markers and returns host exit 1 (raw QEMU exit is null in this harness).
The unchanged positive normal run completes in 1.5 seconds. This is a detected
negative control, not a passing run or a production attack claim. The original
candidate remains unchanged; no test expectation was weakened.

One bounded read-only AI review found no blocking defect. Physical entropy quality,
authenticated production recovery, human audit and full B01 acceptance remain open.
The unsafe issue/roadmap execution hold remains active. Full raw logs stay local.
