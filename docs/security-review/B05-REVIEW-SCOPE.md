# Proposed external security review scope

**PROPOSED — no external review is commissioned or performed, and no new trust
root, escrow, capability or security acceptance is adopted.** Baseline main
`fac5a864` (includes the separately integrated Rust1.100 transition). Previous
branch-bound evidence retains its original compiler, source and limitations.

The machine-readable [scope](scope.json) names seven trust boundaries, their
source files, invariants, executable test/runner starting points and existing
evidence/contract references. Run `python3 -B tools/b05_scope_check.py` to reject
missing/escaping paths, omitted boundaries and empty invariants. A successful
exit validates inventory structure only. It does not run the referenced tests,
prove that coverage is complete, or accept any referenced result.

Evidence links deliberately include contracts, host records and candidate runs.
Their original source, failures, scope and acceptance status must be read before
reliance. A reference to the M5 candidate does not establish DMA confinement;
network codec references do not establish native TLS; the kernel contract does
not replace executed usercopy/ELF evidence. No historical artifact is rewritten
or relabelled, and no broad audit is started by this packet.

## Proposed review questions

- Syscalls/usercopy and IPC: which serialization, memory-map and concurrency
  assumptions remain valid for each actual profile? Do invalid lengths/pointers
  fail before output, resource or queue mutation?
- Capabilities: can revoked descendants, closed objects, queued transfers or
  recycled process/slot generations regain or amplify authority?
- VirtIO/DMA: what remains in the trusted kernel transport, and which real
  quiescence/completion assumptions protect retained buffers after failure?
- DHCP/DNS/ARP/ELF: are every nested length, compression/range/alignment and
  destination/permission boundary checked without attacker-controlled allocation?
- Key recovery: are authentication, allocation/revocation merge, ownership,
  uncertainty and cold rollback claims separated from mere checksum validity?

## Release boundary

The project owner selects the external human reviewer, scope, budget and access.
Any broader trust boundary or evaluator change requires a separate scoped decision.
A second AI review is not a human audit. Remaining Daybreak/runtime findings stay
in their existing records; this packet neither reopens nor resolves them.
Unattended execution stays held, production keys and hardware remain excluded,
and mandatory hosted checks remain required for integration. B05 and the roadmap
score remain open/unchanged at41/100.
