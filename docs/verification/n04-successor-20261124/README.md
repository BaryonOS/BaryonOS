# N04 successor native evidence — 24 November 2026

This directory retains exact compact reports for the separate full-port sweep
successor in PR #478. It is a bounded native-QEMU engineering checkpoint, not
acceptance of the global N04 product criterion. The score remains 41/100 and the
unattended controller remains held.

## Source and observation

- Published head: `863541014d92de0108b2c29063b04a012acbe8db`.
- Tested merge commit: `4faa2b40c18257fb8bf5ba85bfef78d49d240a53`.
- Identical reviewed/tested tree: `d7d113f8970ffe0f968591efa0ea0b5a77835d85`.
- V3 source fingerprint: `ea1d1128137f01e8dec9cf4557325fa07588dc515815352bfbb0cf1c9feb4e15`.
- Run: [45308625592](https://github.com/BaryonOS/BaryonOS/actions/runs/45308625592).
- [Native acceptance and independent review](https://github.com/BaryonOS/BaryonOS/pull/478#issuecomment-7119750537).
- Verified main landing: [`c3afc04ac2bc92228f2201af02ab69148c47e596`](https://github.com/BaryonOS/BaryonOS/commit/c3afc04ac2bc92228f2201af02ab69148c47e596), with the identical tree above. `main-git-proof.json` records the actual ref/object check.

All six baseline cases and DHCP/link/route/full-sweep reports passed with raw
QEMU exit 33 and no timeout. The sweep completed in 178.683958385 seconds. Its
pre/post phases each recorded 65,535 SYNs and exact RSTs plus 16 UDP probes:
131,070 SYNs/RSTs and 32 UDP probes in total, with zero probe SYN-ACK or UDP replies.
Phase/revoke ticks were 8522 / 8606 / 9020 / 17627; application exits were 17656
and 17670, below the unchanged 32,000-tick guest deadline. The host bound remains
360 seconds. The published static service repair sends a newly produced reply
after the existing bounded poll; completion, revocation fences and limits remain.

The hosted peer suite passed 37 tests. A separately recorded existing test on the
same source exercised 14 refusal subcases, including synthetic SYN-ACK and UDP
replies, and its exact-RST positive control. The hosted excerpt is an exact
contiguous part of the original job log; its provenance JSON records byte/line
offsets and the full-log hash. The targeted log and command receipt retain their
original bytes.

## Retention and limits

The ten native report JSON files and both compact archive receipts are exact
artifact bytes. The metadata file preserves the actual GitHub source/run/job/step
and artifact records. Original hosted paths in those raw records are retained as
provenance; the source-bound task receipt maps them to the files here.

The 39,890,212-byte full sweep capture was omitted under the compact-by-default
contract. Its recorded SHA-256 is
`4c516784a039199f53d15ab46a58ef0f57efaa7c6fe00c386929398a0b6f73db`.
The exhaustion raw binary is also omitted. The hosted harnesses replayed their
causal captures before reporting success; no independent local full-capture
replay or retained raw capture is claimed. The seven service serial reports were
independently re-evaluated against the exact-source pure serial parser.

The historical `160153...` report is retained as an explicit **failure**: raw35
at the guest deadline with an incomplete post-sweep. It is not successor evidence.
The original #380 failed prototype receipt remains unchanged at its
[immutable source](https://github.com/BaryonOS/BaryonOS/blob/70140ffcfb142e7728725458aa205869c448cc16/docs/roadmap-pipeline/receipts/N04-T1.json).

Production issuer/delegation, durable revocation, physical NICs, hardware and
human audit remain outside this bounded task. A later checkpoint carrying these
files does not acquire the same source fingerprint or an unrun native result.
