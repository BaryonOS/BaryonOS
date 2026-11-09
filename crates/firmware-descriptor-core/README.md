# Firmware descriptor core

This standalone `no_std` primitive validates bounded firmware metadata:
firmware identity, exact device match, version tuple, claimed SHA-256 digest,
provenance reference, license reference, and device-revision/platform
compatibility requirements.

Validation is structural only. Provenance and digest are caller-supplied
claims; this crate does not authenticate them, approve firmware, load bytes,
download packages, or grant a driver access. A driver request never changes
the descriptor's `Unverified` status.
