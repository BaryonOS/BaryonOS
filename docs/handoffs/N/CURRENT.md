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

Checkpoint **2026-11-27T21:32:54Z**. [HANDOFF N on
#529](https://github.com/BaryonOS/BaryonOS/issues/529#issuecomment-7228304319)
carries the branch, worktree and log locations. Check #529 for later updates. N01 remains accepted. N02, N03
and N04 remain open at 43/100.

- **N02.** The clean published branch
  [`codex/n02-slirp-timeout-20261127`](https://github.com/BaryonOS/BaryonOS/commit/d9ec9616bfbae9033a9ebf9ff5bc15d8ae4f9c3d)
  implements ring-3 mode 8 and a loopback-only network namespace containing libslirp,
  DNS and HTTP controls. Host oracle/DHCP suites, formatting, target Clippy and
  namespace checks passed. Existing libslirp and N01 controls passed at raw QEMU exit
  33. The [N02
  checkpoint](https://github.com/BaryonOS/BaryonOS/issues/529#issuecomment-7227579418)
  records the evidence. N has added the [build selector](https://github.com/BaryonOS/BaryonOS/issues/529#issuecomment-7228526798)
  under CI's scoped assignment. Native mode 8 still requires the [K kernel
  fixture](https://github.com/BaryonOS/BaryonOS/issues/614#issuecomment-7228025405).
- **N03.** [B1
  preflight](https://github.com/BaryonOS/BaryonOS/issues/529#issuecomment-7227862853)
  records ML-KEM validation against NIST vectors, strict Ed25519 and portable target
  compilation. A scratch host TLS oracle exercised the strict verifier for both the
  certificate and CertificateVerify. Native provider integration, native handshake,
  trusted calendar and independent security review remain open.
- **N04.** The extend-chain host model and two-slot torn-write recovery landed in
  #575 and #584. B's open [#649](https://github.com/BaryonOS/BaryonOS/pull/649)
  carries the S7 host foundation. The independent [network regression
  review](https://github.com/BaryonOS/BaryonOS/pull/649#issuecomment-7227967069)
  covers head `c680505e` and eleven native QEMU network cases. Native S7 still needs
  the authorized TEST grant, authenticated loader binding, TPM compare/read-back and
  source-bound swtpm evidence. The [U13c
  decision](https://github.com/BaryonOS/BaryonOS/issues/529#issuecomment-7227825311)
  authorizes staging the signed TEST record at a second path and promotion after both
  accepted-serial copies advance. Decision 5A stays in force pending joint native 5B
  acceptance. The store model assumes a single writer.

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

1. Start N04 S5 when B publishes the native S7 interface and TEST/QEMU/swtpm proof.
  Verify the domain-0 grant, instance and extend budget, and B02 protected-slot route
  before binding the separate policy process. Preserve broker authority and exercise
  the condition 9 cold, torn-write, replay and lost-reply cases.
2. If K releases the N02 fixture first, integrate the exact K and N commits, verify
   mode 8 and the unchanged controls, then open one N PR with source-bound evidence.
   Physical NIC, router and authorized internet acceptance remain open.
3. Review H04's in-flight TX amendment when H releases it. Preserve the distinct
  completion mapping to `Completed` and its counter. H owns the driver and kernel
  implementation.
