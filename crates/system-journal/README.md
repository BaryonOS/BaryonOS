# System journal core

This standalone `no_std` crate provides a fixed-capacity, in-memory diagnostic
journal. It is not yet wired into the kernel or a native diagnostic service;
the root/kernel and service runtime paths remain separately owned.

Records contain only typed identities, event codes, timestamps, correlation
IDs, and a small allowlist of typed metadata. The API accepts no strings, byte
buffers, pointers, passwords, tokens, keys, recovery secrets, document content,
or memory dumps. Producers must select metadata whose declared meaning is
non-sensitive; the type boundary prevents accidental free-form payload logging,
not deliberate encoding of secrets into numeric fields.

The crash extension records a process ID, optional service role, fixed 32-byte
build identity, fault class, typed bounded context, and restart outcome. It
does not validate the provenance of the build identity or persist kernel panic
state.

The ring overwrites the oldest record when full and reports that in saturating
counters. Debug and Info events are rate-limited per producer; Warning and
higher are not rate-limited. Journal access requires a reader permit issued by
the trusted initializer and scoped to an explicit subsystem allowlist. Journal
IDs must be unique within the owning system. This is an in-memory primitive,
not durable storage or cloud telemetry.

The integration test is an external crate consumer. Its host execution tests
the public API, not native OS enforcement.
