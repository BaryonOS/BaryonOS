#!/usr/bin/env python3
"""Mutation check for the S06 volume rules: every listed mutant must be caught.

Each mutant removes or weakens exactly one rule in a scratch copy of the
source tree (the working tree is never edited). The script first checks that
the unmutated copy passes, then per mutant: the copy must still compile (a
mutant that does not build proves nothing and fails the run), and at least one
test must fail. The report lists the command, exit status and failing tests
of every run; only `passed: true` means every mutant was caught.
"""
from __future__ import annotations
import argparse
import hashlib
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from common import BUILD, ROOT, save_json, source_hash

RUST = ["cargo", "+1.90.0", "test", "--locked", "--offline", "-p", "volume-core",
        "--lib", "--test", "service", "--test", "history"]
PYTHON = [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_volume_*.py"]
TABLE = "services/volume/src/table.rs"
CLIENT = "services/volume/src/client.rs"
WIRE = "services/volume/src/wire.rs"
SERVICE = "services/volume/src/service.rs"
ORACLE = "tools/volume_disk.py"
STAT_REQUEST = ("        COLLECTION_STAT => {\n            base && h.revision == 0\n                && length == 0\n"
                "                && h.name_length == 0\n                && h.count == 0\n"
                "                && (1..=COLLECTIONS as u64).contains(&h.aux)\n        }\n")

# (id, file, exact text, replacement, suite). Each text occurs exactly once.
MUTANTS = [
    ("choose-unrelated-generations", TABLE, "if a.max(b) == a.min(b) + 1 {", "if true {", "rust"),
    ("collection-clock-bound", TABLE,
     ".any(|(index, collection)| collection.revision > self.clocks[index / COLLECTIONS])",
     ".any(|_| false)", "rust"),
    ("revision-clock-bound", TABLE, "!entry.validate() || entry.revision > clock || entry.modified > clock",
     "!entry.validate() || entry.modified > clock", "rust"),
    ("modified-clock-bound", TABLE, "!entry.validate() || entry.revision > clock || entry.modified > clock",
     "!entry.validate() || entry.revision > clock", "rust"),
    ("restore-keeps-history", SERVICE, "let restored = entry.take_history(slot);",
     "let restored = entry.history[slot];", "rust"),
    ("flush-after-recovery", SERVICE, "            disk.flush()?;\n            service.counters.flushes += 1;",
     "            service.counters.flushes += 1;", "rust"),
    ("stat-version-nonzero", CLIENT, "                && version != 0\n", "", "rust"),
    ("stat-history-nonzero", CLIENT, "revision != 0 && revision < newer", "revision < newer", "rust"),
    ("list-cursor-gate", CLIENT, "let entries = if ok { count } else { 0 };",
     "let entries = count.min(wire::LIST_MAX);", "rust"),
    ("collection-stat-name-length", CLIENT,
     "            h.name_length as usize == length\n                && h.count <= 1",
     "            h.count <= 1", "rust"),
    ("collection-stat-count", CLIENT, "                && h.count <= 1\n", "", "rust"),
    ("collection-stat-request-id", CLIENT, "                && (h.revision == 0) == (h.aux == 0)\n", "", "rust"),
    ("collection-stat-present-revision", CLIENT, "h.revision != 0 && length != 0 && named(body, length)",
     "length != 0 && named(body, length)", "rust"),
    ("collection-stat-present-name", CLIENT, "h.revision != 0 && length != 0 && named(body, length)",
     "h.revision != 0 && length != 0", "rust"),
    ("collection-stat-absent-empty", CLIENT, "                } else {\n                    length == 0\n",
     "                } else {\n                    true\n", "rust"),
    ("collection-stat-request-base", WIRE, "        COLLECTION_STAT => {\n            base && h.revision == 0",
     "        COLLECTION_STAT => {\n            h.revision == 0", "rust"),
    ("collection-stat-request-revision", WIRE,
     "        COLLECTION_STAT => {\n            base && h.revision == 0", "        COLLECTION_STAT => {\n            base",
     "rust"),
    *[(f"collection-stat-request-{field}", WIRE, STAT_REQUEST, STAT_REQUEST.replace(f"\n                && {line}", ""),
       "rust") for field, line in (("length", "length == 0"), ("name-length", "h.name_length == 0"),
                                   ("count", "h.count == 0"))],
    ("oracle-unrelated-generations", ORACLE, 'if newer["generation"] != older["generation"] + 1:', "if False:",
     "python"),
    ("oracle-modified-clock-bound", ORACLE, ' or e["modified"] > clocks[o // PER_DOMAIN]', "", "python"),
    ("oracle-revision-clock-bound", ORACLE, 'e["revision"] > clocks[o // PER_DOMAIN] or ', "", "python"),
    ("oracle-collection-clock-bound", ORACLE, 'any(c["revision"] > clocks[i // COLLECTIONS]',
     "any(False", "python"),
]


def test_inputs() -> dict[str, str]:
    """The Python tests lie outside the source fingerprint; bind them by hash."""
    return {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted((ROOT / "tests").rglob("test_volume_*.py"))}


def run(command: list[str], cwd: Path) -> dict:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    output = result.stdout + result.stderr
    failed = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED", output, re.M))
                    | set(re.findall(r"^(?:FAIL|ERROR): (\S+)", output, re.M)))
    return {"command": command, "exit": result.returncode, "failed_tests": failed,
            "compile_error": "error[E" in output or "could not compile" in output}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=BUILD / "volume-mutants.json")
    args = parser.parse_args()
    frozen, tests = source_hash(), test_inputs()
    report = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
              "test_inputs": tests,
              "status": "running", "passed": False, "baseline": [], "mutants": []}
    with tempfile.TemporaryDirectory(prefix="volume-mutants-") as scratch:
        copy = Path(scratch) / "tree"
        shutil.copytree(ROOT, copy, ignore=shutil.ignore_patterns(".git", "target", "build", "__pycache__"))
        suites = {"rust": RUST, "python": PYTHON}
        for name, command in suites.items():
            baseline = run(command, copy)
            report["baseline"].append({"suite": name, **baseline})
            if baseline["exit"] != 0:
                report["status"] = "failed"
                save_json(args.output, report)
                print(f"baseline {name} failed")
                return 1
        for ident, relative, text, replacement, suite in MUTANTS:
            path = copy / relative
            original = path.read_text(encoding="utf-8")
            if original.count(text) != 1:
                raise SystemExit(f"{ident}: text occurs {original.count(text)} times in {relative}")
            path.write_text(original.replace(text, replacement), encoding="utf-8")
            try:
                outcome = run(suites[suite], copy)
            finally:
                path.write_text(original, encoding="utf-8")
            caught = outcome["exit"] != 0 and not outcome["compile_error"] and bool(outcome["failed_tests"])
            report["mutants"].append({"id": ident, "file": relative, "suite": suite, "caught": caught, **outcome})
            print(f"{ident}: {'caught' if caught else 'SURVIVED'} {outcome['failed_tests']}")
    caught = all(m["caught"] for m in report["mutants"])
    report.update(status="completed", passed=caught and source_hash() == frozen and test_inputs() == tests)
    save_json(args.output, report)
    print(f"mutants={len(report['mutants'])} caught={sum(m['caught'] for m in report['mutants'])} "
          f"passed={report['passed']}")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
