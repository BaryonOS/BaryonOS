# Notification core

This standalone `no_std` primitive stores bounded notification references,
applies expiry and per-producer flood limits, removes a producer's entries on
exit, and hides non-public references while the session is locked. Action
references are identifiers only; the core executes no callbacks, and priority
does not grant authority.

The fixed producer table and notification capacity are explicit bounds. This
crate is not integrated with an authenticated producer broker, persistent
history, per-app permission policy, session service, or desktop UI. Its host
tests establish only the local state-machine behavior.
