# Background job core

This standalone `no_std` primitive models bounded jobs whose owner and
capability-scope identities are registered by a trusted runtime. Jobs retain
the exact scope they were given; state changes never mint or widen authority.
Revoking a scope or exiting its owner cancels that owner's nonterminal jobs.
Scope generations must increase before a revoked identity can be registered
again.

The core does not authenticate registrations, execute work, account real
resources, persist job state, or connect to a process/session service. It has
no download/sync consumer and is not native runtime enforcement.
