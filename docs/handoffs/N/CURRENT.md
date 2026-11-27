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
- Owner decisions, 25 November 2026 (do not re-ask):
  - N03: provider route B1 with a later B2 migration, H3, P3, T3, R1,
    ml-kem and a verified ed25519-dalek (`ADR-N03-PROVIDER-20261125`).
  - N04: issuer ADR approved; the cold rollback anchor is agreed with B02
    (`ADR-NET-ISSUER-20261125`). Decision 5B (TPM anchor use) is approved;
    5A stays in force until native 5B acceptance.
  - M1 (stale store after a revocation): the protected-slot route through B,
    superseding the earlier reseal approval, which host tests showed to be
    insufficient. B owns the protected slot and the extend-chain contract.
  - HAUTH: TEST/QEMU/swtpm only. Residual: code at PCR13=0 can read HAUTH
    until B04.
  - N02: the silent or timeout real-resolver case runs QEMU inside a network
    namespace.
- Owner decision, 24 November 2026: the N02 libslirp case may use
  `-netdev user,restrict=off` (guest reaches a host-loopback oracle and the
  runner's real DNS resolver) in a separate, non-required CI job.
- B contract for the N04 store (agreed P1–P3): `read() -> chain`,
  `extend(expected_chain, D(S))` with the domain-derived event `E` computed by
  the kernel, `instance()`, a single `Stale` error, `Foreign` on an instance
  mismatch, and no Pending or Exhausted states. Startup: `chain ==
  H(S.prev ‖ E(S))` accepts; `chain == S.prev` rolls forward a
  never-acknowledged store only; anything else is `Stale`. B's golden vectors
  are in `COLD-FRESHNESS-ANCHOR.md`. A torn A1 write is a binding requirement
  on the N04 store.
- Handoff agreement: after each merged part, update this file or the
  workstream handoff comment, then report CLEAR_READY to the PR guard. The
  owner runs `/clear`. Rotation follows `ADR-SPECIALIST-TEAM-20261125`.

## Current cycle

- cycle_id: `N-20261126-n04-extend-chain`
- task: port the host-only N04 durable-store freshness model to B's
  extend-chain anchor contract (`docs/implementation/COLD-FRESHNESS-ANCHOR.md`
  on #524, basis `5db011cc`); amendment to ADR-NET-ISSUER-20261125
- branch: `claude/n04-durable-extend-chain-20261125`; tracking issue #529
- pull request, tested source SHA, hosted runs: recorded in the pull request
  description, not in this file
- evidence: host tests only. `durable.rs` computes the real SHA-256 extend
  chain in a fake anchor and reproduces B's golden vectors; the M1
  regression tests now pass against a hostile extend. No kernel anchor
  service, swtpm run or torn-A1 store recovery (C143) exists; N04 stays open
  and 5A remains the product behaviour.
- previous cycle `N-20261125-n03-n04-adr` (N03/N04 ADRs, N03-T2 host
  handshake) merged as `657f522f` (#543).

## Blockers

- N02 stays open: libslirp is still QEMU user networking on the same host; no
  physical NIC, operator router or internet HTTP endpoint. Timeout,
  truncation and stale-response cases against a real resolver are not run;
  a silent real resolver runs in a network namespace (owner decision above;
  not built yet).
- N04 durable store: waits for B's protected slot and extend-chain kernel
  side (#524) for native evidence.

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
5. A service must sample its tick after receiving a request when that tick
   starts a deadline the client measures from its own earlier reading; the
   loop-top tick can predate the client's send. The native Timeout check has
   a zero-tick margin (observed - start = 300 exactly).

## Next executable task

After this merge: torn-A1 store recovery (C143, #529), stacked on this
port on `claude/n04-store-slots-20261125`. Native S5 verification on swtpm
waits for B's kernel anchor service.
