# Issues #43, #39, #41 and #42 — implementation checkpoint

Date: 14 November 2026. Baseline: `b8a7e83658bdbcd4970c26f753dda1b342fce0d4`.
Branch: `fix/issue43-network-usb-followup`. Score: **41/100 → 41/100**.

## Changes and evidence boundaries

- #39: existing `--case revoke` and `--scenario loss-backpressure` now run in
  routine native CI. Separate reports are mandatory and their raw evidence is
  retained. This does not establish the missing unexpected-reset/exhaustion cases.
- #42: low-speed `bInterval` 1–9 is rejected under USB 2.0 §5.7.4; full-speed
  retains 1–255. The existing exhaustive interval test was corrected, and fails
  on the old production implementation. Component commands and source/log hashes
  are in [the USB receipt](../../../drivers/usb/verification/boot-interface-binding.md).
- #41: [production policy](../../implementation/B03-PRODUCTION-POLICY-PROPOSAL.md)
  is an unapproved decision proposal; no keys, production profile or authority
  are created. Existing TEST native acceptance on the baseline passed in
  [run 43594560820](https://github.com/BaryonOS/BaryonOS/actions/runs/43594560820).
- #43 remains open: the fixed criterion register totals 41, below its target of 50.

## Executed checks

| Command | Result | Scope |
|---|---|---|
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | Exit 0; 24 tests | Integrated workflow contracts |
| `python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'` | Exit 0; 31 tests | Integrated evidence tooling |
| `python3 -B tools/preflight.py --strict` | Exit 0 | Tool inventory only |
| USB tests, fmt, Clippy and freestanding build | Exit 0; 29 tests | Component source hashes in USB receipt |
| Corrected USB regression against original source | Expected exit 101 | Demonstrates the old acceptance defect |

Local integration logs: `build/issue43-checks/`; USB logs are recorded in the
component receipt. Raw logs are not copied into permanent instructions.

## Native checkpoints and remaining requirements

The native build/smoke passed on local commit
`426ed9bca89bb8d490c6f20a9dfb2b0d9aa6a477`, whose complete tree
`00d2a2021522a38b47413b1ec5d4cfe50ec2d2a8` equals public checkpoint
`860e3dbcf7417d3231b055ff2ecb284edd96474d`. The GitHub connector changes
commit metadata; content identity was verified before publication.

Normal/client-exit/revoke all passed on public checkpoint
`01ec2320d5255e261851fac154665313505d87bf`, source-v3
`fd03fa148b800552194d500837903b2cfca800c56066a74c5b27d208f7a7c1e6`.
The TCP loss/backpressure normal/cold pair passed on the identical tree
`3855efceb00d525022f7306aee4f183be4dc326d` at local merge `acd4d74`.
Both native TCP stages exited 33; cold recovery retained the normal image hash.
Two earlier attempts stopped before QEMU because a required Rust 1.91.1
toolchain and then an uncached SDK dependency were absent. Both setup failures
remain in `build/issue43-checks/`; no source fix or relaxed test was used.

The new reset profile and normal/client-exit/revoke then passed on the component
source recorded in [network-reset.md](network-reset.md), with independent review.
The integrated reset host checks pass: 18 driver tests, 19 peer/oracle tests,
25 workflow contracts and the build-path checks. Final combined-source native
gates are still required; this receipt does not relabel old passes as new ones.

QEMU 10.0.13 and OVMF 2025.02-8+deb13u1 run from disposable local tooling.
Rust 1.90.0 supplies the standard targets; the existing TCP harness additionally
requires Rust 1.91.1. Tool binaries and firmware hashes are recorded locally in
`build/issue43-checks/environment.json`. No physical device is attached.

Native owner-port exhaustion remains under implementation. Production policy
approval, native USB/hotplug, actual reference hardware and the 50/100 goal remain
open. The updated project merge policy and SHA-bound acceptance hold are retained.
