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

## Pending checks and remaining requirements

At this intermediate checkpoint `tools/build.py`, `tools/run.py --smoke` and
the integrated native network scenarios have not yet run. QEMU 10.0.13 and
OVMF 2025.02-8+deb13u1 have been extracted to disposable local tooling;
Rust 1.90.0 with both native targets is installed. No physical device is attached.
The final source checkpoint must record real outcomes and preserve failures.

The three subordinate issues remain open. Production policy approval and its
physical trust assumptions, the complete native USB/hotplug path and actual
reference hardware evidence are not supplied by these component checks.
