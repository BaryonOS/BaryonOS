# L02 — native quantum job interface

This profile adds a real user process route on top of the existing
bounded qsim port. The kernel executes no simulator logic: it creates two
processes, two endpoint objects and only the minimal SEND/RECV grants.

The **caller** sends canonical 256-byte `QJ01` requests. The **service**
validates sender, tag, length and request codec, runs the existing
`qsim-native` adapter and sends back a bounded reply. During a job
the `Control` callback checks the same request endpoint for a Cancel. The
requested `work_budget` is enforced at every checkpoint.

The acceptance runs four job paths: Bell with 4,096 shots; a six-qubit
X circuit; a Cancel that is picked up via IPC during execution; and a
job that is terminated by its work budget acting as a deadline. Results are
returned in fixed pages of at most sixteen histogram values.

The service uses a private 64 KiB arena. After each terminated `execute`
all temporary Vecs have been destroyed; only then is the monotonic arena
reset for the next job. The later kernel gate must demonstrate that both
processes, capabilities/endpoints, queues and frames are completely cleaned up.

This is a local software simulator/job interface on the own OS. It is not a
physical QPU, not quantum speedup and not a cloud route.
