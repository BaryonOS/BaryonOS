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

RUST = ["cargo", "+1.100.0", "test", "--locked", "--offline", "-p", "volume-core",
        "--lib", "--test", "service", "--test", "history", "--test", "table_v3", "--test", "install",
        "--test", "l2_restore", "--test", "archive_v2"]
PYTHON = [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_volume_*.py"]
TABLE = "services/volume/src/table.rs"
CLIENT = "services/volume/src/client.rs"
WIRE = "services/volume/src/wire.rs"
SERVICE = "services/volume/src/service.rs"
UPLOAD = "services/volume/src/upload.rs"
ARCHIVE = "services/volume/src/archive.rs"
ORACLE = "tools/volume_disk.py"
STAT_REQUEST = ("        COLLECTION_STAT => {\n            base && h.revision == 0\n                && length == 0\n"
                "                && h.name_length == 0\n                && h.count == 0\n"
                "                && (1..=COLLECTIONS as u64).contains(&h.aux)\n        }\n")

# The L2 restore refusals before its first write (distinct from L1 by the next line).
L2_GUARD = ("    if plan.tombstones {\n        return Err((Status::Invalid, report));\n    }\n"
            "    match never_used(client, domain) {\n        Ok(true) => {}\n"
            "        Ok(false) => return Err((Status::Stale, report)),\n"
            "        Err(status) => return Err((status, report)),\n    }\n"
            "    let objects = &plan.objects[..plan.object_count];\n")

# (id, file, exact text, replacement, suite). Each text occurs exactly once.
MUTANTS = [
    ("disk-source-v2-version-records", ARCHIVE,
     "OBJECT_RECORD_V2 as u64\n                            + u64::from(get16(header, 14)) * VERSION_RECORD_V2 as u64,",
     "OBJECT_RECORD_V2 as u64,", "rust"),
    ("disk-source-v2-as-unknown", ARCHIVE, "                    VERSION_V2 => (", "                    0xFFFF => (",
     "rust"),
    ("disk-source-v2-bound", ARCHIVE, "                    .filter(|length| *length <= limit)\n", "", "rust"),
    ("choose-unrelated-generations", TABLE, "if a.min(b).checked_add(1) == Some(a.max(b)) {", "if true {", "rust"),
    ("collection-clock-bound", TABLE,
     ".any(|(index, collection)| collection.revision > self.clocks[index / COLLECTIONS])",
     ".any(|_| false)", "rust"),
    ("revision-clock-bound", TABLE, "                || entry.revision > clock.saturating_add(entry.excess())\n",
     "", "rust"),
    ("modified-clock-bound", TABLE,
     "                || entry.revision > clock.saturating_add(entry.excess())\n                || entry.modified > clock\n",
     "                || entry.revision > clock.saturating_add(entry.excess())\n", "rust"),
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
    # S09 own-domain reads (docs/implementation/S09-OWN-DOMAIN-READS.md section 6).
    ("domain-stat-request-base", WIRE, "        DOMAIN_STAT => {\n            base && h.revision == 0",
     "        DOMAIN_STAT => {\n            h.revision == 0", "rust"),
    ("domain-stat-global-generation", SERVICE, "self.table.clocks[domain],", "self.table.generation,", "rust"),
    ("version-stat-prefix", SERVICE, "selector > Self::prefix(entry)", "selector > entry.history_count", "rust"),
    ("version-stat-reply-revision", CLIENT,
     "        wire::VERSION_STAT if request.name_length == 0 => {\n"
     "            let (created, modified) = (get64(body, 64), get64(body, 72));\n"
     "            h.revision == request.revision\n",
     "        wire::VERSION_STAT if request.name_length == 0 => {\n"
     "            let (created, modified) = (get64(body, 64), get64(body, 72));\n"
     "            true\n", "rust"),
    ("version-stat-name-reply-revision", CLIENT,
     "        wire::VERSION_STAT => {\n            h.revision == request.revision\n",
     "        wire::VERSION_STAT => {\n            true\n", "rust"),
    ("oracle-unrelated-generations", ORACLE, 'if newer["generation"] != older["generation"] + 1:', "if False:",
     "python"),
    ("oracle-modified-clock-bound", ORACLE, ' or e["modified"] > clocks[o // PER_DOMAIN]', "", "python"),
    ("oracle-revision-clock-bound", ORACLE,
     'e["revision"] > clocks[o // PER_DOMAIN] + excess(e.get("installed", 0))\n           or ', "", "python"),
    ("oracle-collection-clock-bound", ORACLE, 'any(c["revision"] > clocks[i // COLLECTIONS]',
     "any(False", "python"),
    # S09 L2 provenance (docs/implementation/S09-L2-PROVENANCE.md section 8).
    ("excess-n", TABLE, "u64::from(self.installed.saturating_sub(1))", "u64::from(self.installed)", "rust"),
    ("excess-without-install", TABLE, "u64::from(self.installed.saturating_sub(1))",
     "u64::from(self.installed.saturating_sub(1).max(1))", "rust"),
    ("install-begin-stale", SERVICE, "        if self.table.entries[object].revision != 0 {\n            return Err((Status::Stale",
     "        if false {\n            return Err((Status::Stale", "rust"),
    # Unreachable through the wire (no operation can use the slot while an
    # install is staged); killed by the `install_commit_tests` unit test.
    ("install-commit-stale", SERVICE, "        if self.table.entries[session.object].revision != 0 {",
     "        if false {", "rust"),
    ("install-begin-collection", SERVICE, "        if !self.collection_ok(domain, collection) {",
     "        if false {", "rust"),
    ("install-commit-collection", SERVICE, "        if !self.collection_ok(domain, session.collection) {",
     "        if false {", "rust"),
    ("install-collection-stale", SERVICE, "        if self.table.collections[slot].revision != 0 {",
     "        if false {", "rust"),
    ("install-begin-busy", SERVICE,
     "        if self.sessions[domain].is_some() {\n            return Err((Status::Busy, 0));\n        }\n"
     "        let manifest", "        let manifest", "rust"),
    ("begin-keeps-install", SERVICE, "        if self.installing(domain) {\n            return Err((Status::Busy, 0));\n",
     "        if false {\n            return Err((Status::Busy, 0));\n", "rust"),
    ("legacy-guard-install", SERVICE,
     "                    | wire::INSTALL_BEGIN\n                    | wire::INSTALL_APPEND\n"
     "                    | wire::INSTALL_COMMIT\n", "", "rust"),
    ("v7-install-binding", SERVICE,
     "            wire::APPEND | wire::COMMIT | wire::ABORT | wire::INSTALL_APPEND | wire::INSTALL_COMMIT\n"
     "        ) && !self.sessions[domain]",
     "            wire::APPEND | wire::COMMIT | wire::ABORT\n        ) && !self.sessions[domain]", "rust"),
    ("v7-install-token-required", UPLOAD,
     "    let required = matches!(\n        h.operation,\n"
     "        wire::APPEND | wire::COMMIT | wire::ABORT | wire::INSTALL_APPEND | wire::INSTALL_COMMIT\n",
     "    let required = matches!(\n        h.operation,\n        wire::APPEND | wire::COMMIT | wire::ABORT\n",
     "rust"),
    ("v7-install-begin-token", UPLOAD, "    if matches!(request.operation, wire::BEGIN | wire::INSTALL_BEGIN)",
     "    if matches!(request.operation, wire::BEGIN)", "rust"),
    ("manifest-consecutive", WIRE, "                && first.checked_sub(index as u64) == Some(v.origin)\n", "",
     "rust"),
    # `v.origin != 0` is implied by the consecutive rule and `first >= n`
    # (an equivalent mutant), so it has no mutant of its own.
    ("manifest-first-below-origin", WIRE, "            && first <= self.origin_revision\n", "", "rust"),
    ("collection-origin-nonzero", WIRE, "                && get64(body, 64 + k) != 0", "", "rust"),
    ("provenance-prefix", SERVICE, "versions = 1 + Self::prefix(entry);", "versions = 1 + entry.history_count;",
     "rust"),
    ("provenance-absent", SERVICE, "            if entry.present {\n                versions = 1",
     "            if true {\n                versions = 1", "rust"),
    ("provenance-reply-length", CLIENT, "                        && length == 24 + 8 * versions\n", "", "rust"),
    ("provenance-reply-count", CLIENT, "&& h.count <= 1 && (h.count == 1) == (h.aux != 0)", "&& h.count <= 1",
     "rust"),
    ("provenance-object-stale", SERVICE,
     "        if entry.revision != request.revision {\n            return Err(stale(entry.revision));",
     "        if false {\n            return Err(stale(entry.revision));", "rust"),
    ("provenance-reply-versions", CLIENT, "                    versions <= INSTALL_MAX\n"
     "                        && length", "                    length", "rust"),
    ("provenance-reply-origin-shape", CLIENT, "                        && (revision == 0) == (created == 0)\n", "",
     "rust"),
    ("install-append-received", CLIENT, "                && h.count == request.count\n"
     "                && h.aux == request.aux + u64::from(request.length)\n",
     "                && h.count == request.count\n", "rust"),
    ("recreate-keeps-origin", SERVICE,
     "            // A re-created object never inherits the old incarnation's origin.\n"
     "            entry.end_incarnation();\n",
     "            // A re-created object never inherits the old incarnation's origin.\n", "rust"),
    ("purge-keeps-origin", SERVICE,
     "            // Nothing of the installed incarnation is left (2.2).\n            entry.end_incarnation();\n",
     "            // Nothing of the installed incarnation is left (2.2).\n", "rust"),
    ("collection-delete-keeps-origin", SERVICE, "            next.origin_revision = 0;\n", "", "rust"),
    ("install-commit-v2", SERVICE,
     "        let mut patch = Patch::one(session.object, entry);\n        patch.format = table::VERSION_V3;\n",
     "        let mut patch = Patch::one(session.object, entry);\n", "rust"),
    # Table v3 decode and validate checks (section 2.1). The v2 history
    # reserved range and the `decode_head` excess are equivalent (section 11).
    *[(f"table-unknown-version-as-{name}", TABLE,
       "                    VERSION_V3 => COLLECTION_ORIGINS_END,\n                    _ => return Err(Status::Corrupt),\n",
       f"                    VERSION_V3 => COLLECTION_ORIGINS_END,\n                    _ => {end},\n", "rust")
      for name, end in (("v2", "COLLECTION_ORIGINS"), ("v3", "COLLECTION_ORIGINS_END"))],
    ("table-v2-header-reserved", TABLE, "                    VERSION => COLLECTION_ORIGINS,\n",
     "                    VERSION => COLLECTION_ORIGINS_END,\n", "rust"),
    ("table-v3-header-reserved", TABLE, "VERSION_V3 => COLLECTION_ORIGINS_END,", "VERSION_V3 => TRAILER,", "rust"),
    ("table-v2-head-reserved", TABLE, "let reserved = if v3 { HEAD_END } else { INSTALLED };",
     "let reserved = HEAD_END;", "rust"),
    ("table-v3-head-reserved", TABLE, "let reserved = if v3 { HEAD_END } else { INSTALLED };",
     "let reserved = if v3 { TRAILER } else { INSTALLED };", "rust"),
    ("table-head-installed-padding", TABLE,
     "            || bytes[INSTALLED + 1..OBJECT_ORIGINS]\n                .iter()\n                .any(|byte| *byte != 0)\n",
     "", "rust"),
    ("table-v3-history-reserved", TABLE, "let reserved = if v3 { HISTORY_END } else { HISTORY_ORIGINS };",
     "let reserved = if v3 { TRAILER } else { HISTORY_ORIGINS };", "rust"),
    ("choose-format-sticky", TABLE, "if newer.format < older.format {", "if false {", "rust"),
    ("provenance-installed-zero", TABLE,
     "            return object.iter().chain(&origins).all(|origin| *origin == 0);",
     "            return true;", "rust"),
    ("provenance-installed-max", TABLE, "        let mut ok = usize::from(self.installed) <= HISTORY + 1\n",
     "        let mut ok = true\n", "rust"),
    ("provenance-revision-installed", TABLE, "            && self.revision >= u64::from(self.installed)\n", "",
     "rust"),
    ("provenance-origin-pair", TABLE,
     "            && (self.origin_revision == 0) == (self.origin_created == 0)\n", "", "rust"),
    ("provenance-origin-modified-zero", TABLE, "                self.origin_modified == 0\n",
     "                true\n", "rust"),
    ("provenance-created-le-modified", TABLE, "                self.origin_created <= self.origin_modified\n",
     "                true\n", "rust"),
    ("provenance-origins-distinct", TABLE,
     "ok &= origins[..index].iter().all(|other| other != origin)\n                    && ", "ok &= ", "rust"),
    ("provenance-origin-bound", TABLE,
     "\n                    && (self.origin_revision == 0 || *origin <= self.origin_revision);", ";", "rust"),
    ("collection-origin-present", TABLE,
     ".any(|collection| collection.origin_revision != 0 && !collection.present)", ".any(|_| false)", "rust"),
    ("collection-origin-carry", TABLE, "                    collection.origin_revision = origin;\n", "", "rust"),
    # Rule R bound semantics near u64::MAX (spec 2.1: computed without overflow).
    ("rule-r-bound-fails-closed", TABLE, "|| entry.revision > clock.saturating_add(entry.excess())",
     "|| clock.checked_add(entry.excess()).is_none_or(|bound| entry.revision > bound)", "rust"),
    ("rule-r-bound-wraps", TABLE, "clock.saturating_add(entry.excess())", "(clock + entry.excess())", "rust"),
    ("head-bound-fails-closed", TABLE, "self.revision > generation.saturating_add(self.excess())",
     "generation.checked_add(self.excess()).is_none_or(|bound| self.revision > bound)", "rust"),
    ("head-bound-wraps", TABLE, "generation.saturating_add(self.excess())", "(generation + self.excess())", "rust"),
    # Revision u64::MAX on a crafted image: refused as Full, never a panic.
    ("commit-revision-overflow", SERVICE,
     "revision: current.revision.checked_add(1).ok_or((Status::Full, 0))?,\n            length: session.length,",
     "revision: current.revision + 1,\n            length: session.length,", "rust"),
    ("restore-revision-overflow", SERVICE,
     "revision: current.revision.checked_add(1).ok_or((Status::Full, 0))?,\n                    ..restored",
     "revision: current.revision + 1,\n                    ..restored", "rust"),
    ("collection-revision-overflow", SERVICE,
     "next.revision = next.revision.checked_add(1).ok_or((Status::Full, 0))?;", "next.revision += 1;", "rust"),
    # The service-side v2 to v3 upgrade (Patch.format, sticky in memory).
    ("format-not-adopted", SERVICE, "        self.table.format = format;\n", "", "rust"),
    ("install-collection-v2", SERVICE,
     "        patch.collections = Some(collections);\n        patch.format = table::VERSION_V3;\n",
     "        patch.collections = Some(collections);\n", "rust"),
    ("export-window-range", ARCHIVE,
     "                return Err(Status::Range);\n            }\n            return Err(Status::Stale);",
     "                return Err(Status::Stale);\n            }\n            return Err(Status::Stale);", "rust"),
    ("l2-restore-tombstones", ARCHIVE, L2_GUARD, L2_GUARD.replace(
        "    if plan.tombstones {\n        return Err((Status::Invalid, report));\n    }\n", ""), "rust"),
    ("l2-restore-never-used", ARCHIVE, L2_GUARD, L2_GUARD.replace(
        "Ok(false) => return Err((Status::Stale, report)),", "Ok(false) => {}"), "rust"),
    # The content checksum repeats walk_v2's own check of each version against
    # its record (`checksum_update(..) != object.versions[index].content_sum`
    # in walk_v2) and the plan comparison binds the record, so removing the
    # checksum alone is an equivalent mutant; one mutant removes both.
    ("l2-restore-second-read", ARCHIVE,
     "        if !same {\n            return Err(Status::Corrupt);\n        }\n"
     "        if index >= keep {\n            return Ok(());\n        }\n"
     "        if checksum_update(FNV_OFFSET, content) != planned.versions[index].content_sum {\n"
     "            return Err(Status::Corrupt);\n        }\n",
     "        if index >= keep {\n            return Ok(());\n        }\n", "rust"),
    ("l2-restore-guaranteed-free", ARCHIVE,
     "    if needed > GUARANTEED_FREE {\n        return Err((Status::Full, report));\n    }\n"
     "    // A mid-restore Invalid",
     "    // A mid-restore Invalid", "rust"),
    ("oracle-format-sticky", ORACLE, 'if newer["format"] < older["format"]:', "if False:", "python"),
    ("oracle-frozen-v2-reader", ORACLE, "fmt not in ((2, 3) if allow_v3 else (2,))", "fmt not in (2, 3)", "python"),
    ("oracle-v2-provenance-bytes", ORACLE,
     "elif any(head[HEAD + VERSION_BYTES:TRAILER]) or any(history[HISTORY * VERSION_BYTES:TRAILER]):",
     "elif False:", "python"),
    ("oracle-installed-max", ORACLE, "if installed > HISTORY + 1:", "if False:", "python"),
    ("oracle-excess-n", ORACLE, "    return max(installed - 1, 0)", "    return installed", "python"),
    ("oracle-origin-repeat", ORACLE, "if len(set(nonzero)) != len(nonzero):", "if False:", "python"),
    ("oracle-collection-origin", ORACLE,
     'if origin and not (collection["revision"] and collection["present"]):', "if False:", "python"),
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
