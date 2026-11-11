# Executed checks and evidence boundaries
Research date: 26 October 2026. This overview concerns revision 5 of the starter package, not an implemented OS.

## Research

Five bounded investigations carried out: boot/toolchain, kernel isolation and rights, product concept, local quantum execution/PQC, European requirements. Primary sources and precise consultation limitations are at the bottom of each corresponding file. OSDev is the learning route; mechanisms/standards have where possible been checked against documentation from their makers.

A concrete product comparison was made, not a user trial. The chosen kernel and simulator are designed, not an adopted implementation. Unreachable exact version pages and packages not yet fetched are recognisably named in the specifications; known version data is not presented as a successful build.

## These checks were actually executed

| Check | Result | What this proves |
|---|---|---|
| `python3 tools/preflight.py` | Exit 0, valid JSON | Inventory works in the research environment |
| `python3 tools/preflight.py --strict` | Exit 2 | Missing Rust/QEMU/firmware basis is visibly reported as not ready; this is the expected result |
| Python `ast.parse` on preflight | Passed | Syntactically valid Python; not all future host variants tested |
| Python `tomllib` on five TOML files | Passed | Syntax of agent configuration; no evidence that every Codex client loads these roles |
| Python `ctypes` BootInfo | 128 bytes, alignment 8; all 19 offsets equal to spec | Arithmetically prescribed C layout; Rust compile-time asserts still to be run |
| Python `ctypes` IPC | 320 bytes; caps@32, data@64 | Arithmetic message format; syscall/copy safety still to be tested |
| Standalone SplitMix64/Bell reference calculation | seed0: 8 shots `00:5,11:3`; 4096 shots `00:2082,11:2014` | Deterministic fixture expectation; no working Rust engine/QPU |
| JSON parse of three quantum examples | Passed | Examples are syntactically JSON; future typed validator still to be tested |
| sRGB contrast calculation for six colour pairs | 13.56; 5.15; 7.65; 6.75; 5.81; 3.77 | The fixed colour pairs are correct; no rendered WCAG audit |

The Python calculations mentioned ended with exit 0. The main check independently recalculated the ABI/IPC layout, Bell fixtures and colour ratios. An extra AI review is a document check, not an independent human security audit.

## Handover and package check

44 project files, 30 local Markdown references checked; no missing target and all code fences closed. The separate document review found a duplicate README paragraph and unclear ownership of build tools; both have been corrected. P02 uses ui/prototype/ everywhere. The main agent manages root configuration and tools; B01 the loader/kernel/protocol/parser crates; Q01 its own nested workspace.

## Environment of this research

Linux x86_64, Python 3.12.14, Git 2.51.1, host C compiler GCC 13.3.0. `rustc`, `cargo`, `rustup`, `qemu-system-x86_64`, `clang` and `ld.lld` are missing from PATH; the examined standard locations contain no OVMF. No binaries, toolchain components or firmware were installed during this research assignment. Re-inventory the execution environment in a next development session. This research describes the Linux research environment; the local development computer has not been inspected.

The preflight script only looks for presence and versions. It installs nothing, runs no VM, does not search for NAS devices and changes no firmware. `RUSTUP_AUTO_INSTALL=0` prevents implicit toolchain download via rustup proxies. Source: https://rust-lang.github.io/rustup/environment-variables.html .

## Not executed

Rust dependency resolution, compiler/linker tests, OVMF/QEMU boot, EBS retry trial, real process isolation, capability tests, UI browser trials, screen reader trials, usability research, simulator Rust tests, crypto integration, physical QPU control, hardware ports and legal conformity assessment.

These are execution assignments with their own acceptance criteria, not reasons to restart the general research round. The next results are code and real test logs according to FIRST-BATCH.md.
