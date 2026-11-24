# Specialist N — resume pointer

Compact resume pointer for the network specialist (N01 guard, N02–N04).
This file is not a backlog: tasks come from `docs/progress.json`, issues
and pull requests. Newer ADRs and live evidence take precedence over it.

## Scope, mandate and working agreements (recorded once)

- Scope: everyday internet configuration (N02), native authenticated
  transport (N03), enforceable network permissions (N04); N01 is guarded
  against regression. Protocol and service behaviour belong to N; NIC, Wi-Fi,
  Bluetooth and driver recovery to H; key and trust preconditions to B;
  configuration and permission UI to UI-U.
- Mandate (project owner, 24 November 2026): N commits, pushes, opens, reviews,
  repairs and squash-merges its own pull requests under
  ADR-MERGE-DELEGATION-20261123; N is the temporary scoped lead for necessary
  protected or shared files and keeps the review outside the workstream.
- At most one primary open pull request per cycle. A host handshake is not a
  native handshake; a staged fixture is not internet acceptance.
- Owner-gated, unchanged: the N03 native TLS provider, test peer, certificate
  time and random route (`docs/implementation/N03-PROVIDER-DECISION-PROPOSAL.md`,
  PROPOSED) and all N04 issuer, subgrant, regrant and durable-denial code
  (`docs/implementation/N04-ISSUER-ADR-PROPOSAL.md`, PROPOSED).
- Owner decision, 24 November 2026: the N02 libslirp case may use
  `-netdev user,restrict=off` (guest reaches a host-loopback oracle and the
  runner's real DNS resolver) in a separate, non-required CI job.

## Current cycle

- cycle_id: `N-20261124-slirp-nx`
- task: N02-T6 negative slice — NXDOMAIN from the real resolver in the
  libslirp profile (the reserved name `missing.invalid`)
- branch: `claude/n02-slirp-negative-dns-20261124`
- pull request, tested source SHA, hosted runs: recorded in the pull request
  description, not in this file
- evidence: `tools/run_network_dhcp.py --case slirp`; CI job
  `net-broker-quality.yml#native-network-slirp` (unchanged job, extended case)
- result: app A gets `NotFound` in-guest; the kernel pins two UDP queries and
  one more rejected request; the runner requires exactly one rcode-3 response
  without addresses, bound to the query's ID, port and name, on the wire. The
  answer comes from the host-configured resolver (systemd-resolved stub on
  the runner) reached through libslirp, not necessarily from upstream.
- previous cycle `N-20261124-slirp` merged as `b5cb839f` (#485).

## Blockers

- N02 stays open: libslirp is still QEMU user networking on the same host; no
  physical NIC, operator router or internet HTTP endpoint. Timeout,
  truncation and stale-response cases against a real resolver are not run;
  a silent real resolver needs a new harness boundary (a re-pointed host
  resolver or a network namespace), which is an owner decision.
- N03 and N04: the owner decisions above.

## Lessons

1. Run a new native profile locally against the real counterpart before
   writing exact kernel counters: the first libslirp run found a parser
   interoperability defect that host tests with self-built frames hid.
2. libslirp answers ARP only for its host and DNS aliases, rejects `dns`
   equal to `host`, and with `restrict=on` drops guest UDP and omits DHCP
   option 6. A destination grant must name the DNS alias separately.
3. `NOS_BUILD_DIR` must stay inside the worktree; build separate profiles in
   separate build directories so reports do not overwrite each other.
4. The local systemd-resolved stub answers `missing.invalid` with NXDOMAIN
   and no authority SOA (33 bytes, NSCOUNT 0), so the resolver derives no
   negative-cache lifetime; the case therefore asserts one query only and
   does not claim negative caching against the real resolver.

## Next executable task

After the merge: once the owner decides the N03 or N04 ADR, the first slice
of that decision; otherwise a host-side N02 negative-cache regression for an
NXDOMAIN without SOA (lesson 4), or the owner question on a
harness boundary for a real-resolver timeout case.
