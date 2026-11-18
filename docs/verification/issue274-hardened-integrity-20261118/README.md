# Issue #274 hardened integrity evidence — 18 November 2026

This child supersedes the integrity claims of the earlier issue #274 inventory and
Wiki-publication receipts without rewriting them. It binds immutable repository and
Wiki trees, corrected helpers, fixed inputs, fresh semantic output, raw exit records
and a complete SHA-256 register.

The checked repository source is
`4ae9a1c4f19bcf84ed06f0a8afcfdfc40fb237a8` (tree
`a6eb48f68f30cf443b96c7cfc9da636a2a179532`). The checked Wiki publication is
`4a01b8b2ad2695da509a512ea688007c964f56da` (tree
`f7bee30ee39abb60da294d48357af5b6c85f2fae`).

[DESIGN.md](DESIGN.md) records the trust boundaries, all 22 finding dispositions,
reproduction commands and evidence limits. `manifest.json` binds the required
evidence roles; `hashes.json` covers every retained file except itself.

The host checks establish immutable-source inventory and publication integrity. They
do not establish professional language quality, runtime UI localisation, native QEMU
behaviour, hardware acceptance or completion of issue #274. The issue remains open
and the roadmap score remains 41/100.
