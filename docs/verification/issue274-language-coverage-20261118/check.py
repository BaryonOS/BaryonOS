"""Read-only, source-bound documentation language inventory check."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile


WORDS = r"\b(?:het|een|zijn|wordt|worden|voor|geen|alleen|ook|deze|dit|moet|moeten|zonder|niet|gegevens|gebruiker|bestanden|veiligheid|mogelijk|beschikbaar|toestemming|verplicht|ondersteund|huidige|volgende|bewijs|onderzoek|ontwerp)\b"
GIT_ENV = {**os.environ, "GIT_NO_LAZY_FETCH": "1", "GIT_TERMINAL_PROMPT": "0"}


def candidates(text):
    result = []
    fence = None
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            kind = marker.group(1)[0]
            if fence is None:
                fence = kind
            elif fence == kind:
                fence = None
            continue
        if fence:
            continue
        visible = re.sub(r"`+[^`]*`+", "", line)
        visible = re.sub(r"\]\([^)]*\)", "]", visible)
        visible = re.sub(r"https?://\S+", "", visible)
        visible = re.sub(r"<[^>]*>", "", visible)
        if re.search(WORDS, visible, re.IGNORECASE):
            result.append(number)
    return result


def protected(relative):
    return (
        "/verification/" in relative
        or relative.startswith(("docs/history/", ".agents/skills/os-dev/references/corpus/", "docs/third-party/licenses/", "experiments/quantum/third-party/"))
        or ("/vendor/" in relative and relative != "services/keyring/vendor/README.md")
    )


def path_controls(root, scratch):
    if os.name != "posix":
        raise RuntimeError("POSIX special-path controls: NOT RUN on this platform")
    relative = Path("docs/verification/issue274-language-coverage-20261118")
    old_metadata = Path("docs/verification/english-translation-20261109")
    with tempfile.TemporaryDirectory(prefix="path-controls-", dir=scratch) as location:
        outer = Path(location).resolve()
        temporary = outer / "repository"
        temporary.mkdir()
        (temporary / relative).mkdir(parents=True)
        for name in ("check.py", "collect.cjs"):
            shutil.copyfile(root / relative / name, temporary / relative / name)
        (temporary / old_metadata).mkdir(parents=True)
        (temporary / old_metadata / "postmerge-link-audit.json").write_text(json.dumps({"snapshot": {"changed_active_paths": []}, "head_at_start": "HEAD", "preexisting_active_findings": []}))
        (temporary / old_metadata / "english-final-readonly-audit.json").write_text(json.dumps({"head": "HEAD"}))
        initial = b"# English fixture\n"
        readme = temporary / "README.md"
        readme.write_bytes(initial)
        digest = hashlib.sha256(initial).hexdigest()
        minimal = {"documents": [{"path": "README.md", "sha256": digest, "candidate_lines": []}], "community_forms": [], "compatibility_anchors": [], "translated_original": {"path": "README.md", "sha256": digest, "english_equivalent": "README.md"}, "counts": {"normal_navigation_reachable": 1}, "historical_dutch_residuals": []}
        (temporary / relative / "coverage.json").write_text(json.dumps(minimal))
        def git(*arguments):
            return subprocess.check_output(["git", "-C", str(temporary), *arguments], env=GIT_ENV)
        def commit(message):
            git("-c", "user.name=Language fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", message)
            return git("rev-parse", "HEAD").decode().strip()
        git("init", "-q")
        git("config", "core.quotePath", "true")
        git("add", "--all")
        base = commit("Initial documentation fixture")
        command = ["python3", "-B", str(temporary / relative / "check.py")]
        if subprocess.run(command, capture_output=True).returncode != 0:
            raise ValueError("Minimal path fixture failed its positive control")
        names = ["docs/café.md", "docs/cafe\u0301.md", "docs/tab\tname.md", "docs/line\nbreak.md", "docs/carriage\rreturn.md", 'docs/quote"name.md', "docs/back\\slash.md"]
        for name in names:
            target = temporary / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(initial)
            git("add", "--", name)
        special = commit("Unicode and special documentation paths")
        for arguments in ([], ["--ref", special]):
            result = subprocess.run(command + arguments, text=True, capture_output=True)
            if result.returncode != 1 or set(json.loads(result.stdout)["unclassified_paths"]) != set(names):
                raise ValueError("Index/tree path enumeration omitted an exact special filename")
        wiki = outer / "wiki"
        wiki.mkdir()
        collect = ["node", str(temporary / relative / "collect.cjs"), "--wiki", str(wiki)]
        result = subprocess.run(collect, text=True, capture_output=True)
        if result.returncode != 0:
            raise ValueError("Regular special-path collector control failed: " + result.stderr)
        report_path = temporary / "build/english-coverage-20261118/coverage.json"
        found = {item["path"] for item in json.loads(report_path.read_text())["inventory"]}
        if not set(names).issubset(found):
            raise ValueError("Collector omitted a Unicode or special tracked path")
        outside = outer / "outside.md"
        outside.write_text("Dit is een buiten repository gegevensbestand.\n")
        readme.unlink()
        readme.symlink_to(outside)
        git("add", "--", "README.md")
        commit("Escaping tracked source symlink")
        report_path.unlink()
        result = subprocess.run(collect, text=True, capture_output=True)
        if result.returncode == 0 or "Non-regular tracked source: README.md" not in result.stderr or report_path.exists():
            raise ValueError("Collector did not reject the escaping source before reporting")
        if subprocess.run(command + ["--ref", base], capture_output=True).returncode != 0:
            raise ValueError("Immutable regular snapshot incorrectly followed a working-tree symlink")
    return {"special_names": len(names), "index_unknown_paths_raw": 1, "tree_unknown_paths_raw": 1, "regular_git_blob_collector_raw": 0, "escaping_symlink_rejected": True}


def self_test(root, record):
    relative = Path("docs/verification/issue274-language-coverage-20261118")
    scratch = root / "build" / "english-coverage-20261118"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="controls-", dir=scratch) as location:
        temporary = Path(location).resolve()
        paths = {item["path"] for item in record["documents"] + record["community_forms"]}
        paths.add(str(relative / "check.py"))
        if record.get("historical_entrypoints_manifest"):
            paths.add(record["historical_entrypoints_manifest"])
        for name in paths:
            source = (root / name).resolve()
            target = (temporary / name).resolve()
            if not source.is_relative_to(root) or not target.is_relative_to(temporary):
                raise ValueError("Self-test path outside its source/scratch root")
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        (temporary / relative / "coverage.json").write_text(json.dumps(record) + "\n")
        subprocess.run(["git", "-C", str(temporary), "init", "-q"], check=True, capture_output=True)
        subprocess.run(["git", "-C", str(temporary), "add", "--all"], check=True, capture_output=True)
        command = ["python3", "-B", str(temporary / relative / "check.py")]
        baseline = subprocess.run(command, text=True, capture_output=True)
        if baseline.returncode != 0:
            raise ValueError("Copied-source positive control failed")
        target = temporary / "docs/ARCHITECTURE.md"
        target.write_text(target.read_text() + "\nDit is een nieuwe Nederlandse zin.\n")
        changed = subprocess.run(command, text=True, capture_output=True)
        result = json.loads(changed.stdout)
        if changed.returncode != 1 or "docs/ARCHITECTURE.md" not in result["source_changes"] or "docs/ARCHITECTURE.md" not in result["lexical_changes"]:
            raise ValueError("Changed-prose negative control was not detected")
        target = temporary / "docs/UNCLASSIFIED.md"
        target.write_text("# Nieuwe tekst\n\nDit is een nieuw document.\n")
        subprocess.run(["git", "-C", str(temporary), "add", "--", "docs/UNCLASSIFIED.md"], check=True, capture_output=True)
        added = subprocess.run(command, text=True, capture_output=True)
        if added.returncode != 1 or json.loads(added.stdout)["unclassified_paths"] != ["docs/UNCLASSIFIED.md"]:
            raise ValueError("New-document negative control was not detected")
    special = path_controls(root, scratch)
    print(json.dumps({"self_test": "PASS", "raw_exits": {"unchanged_source": 0, "changed_prose": 1, "new_tracked_document": 1}, "expected_negative_exits": 1, "path_controls": special}))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--ref", help="Optional full immutable Git commit SHA; default: working tree")
    parser.add_argument("--inventory", type=Path, help="Optional repository-relative path to a later versioned inventory")
    parser.add_argument("--self-test", action="store_true", help="Exercise isolated copied-document mutation controls under ignored build/")
    args = parser.parse_args()
    if args.ref and args.self_test:
        parser.error("--ref and --self-test are separate checks")
    if args.ref and not re.fullmatch(r"[0-9a-fA-F]{40}", args.ref):
        parser.error("--ref requires a complete immutable commit SHA")
    root = Path(__file__).resolve().parents[3]
    inventory = (root / args.inventory).resolve() if args.inventory else Path(__file__).with_name("coverage.json")
    if not inventory.is_relative_to(root):
        parser.error("--inventory must remain inside the repository")
    record = json.loads(inventory.read_text())
    if args.self_test:
        self_test(root, record)
        return
    command = ["git", "-C", str(root)]
    if args.ref:
        command += ["ls-tree", "-r", "--name-only", "-z", args.ref]
    else:
        command += ["ls-files", "--cached", "-z", "--", "*.md"]
    actual_paths = {
        relative for relative in (
            value.decode("utf-8")
            for value in subprocess.check_output(command, env=GIT_ENV).split(b"\0")
            if value
        )
        if relative.endswith(".md") and not protected(relative)
    }
    expected_paths = {item["path"] for item in record["documents"]}
    if actual_paths != expected_paths:
        print(json.dumps({"inventory_result": "SCOPE REVIEW REQUIRED", "unclassified_paths": sorted(actual_paths - expected_paths), "missing_paths": sorted(expected_paths - actual_paths)}))
        raise SystemExit(1)

    def read(relative):
        if Path(relative).is_absolute() or ".." in Path(relative).parts:
            raise ValueError("Path outside repository")
        if args.ref:
            return subprocess.check_output(["git", "-C", str(root), "show", args.ref + ":" + relative], env=GIT_ENV)
        target = (root / relative).resolve()
        if not target.is_relative_to(root):
            raise ValueError("Path outside repository")
        if not stat.S_ISREG((root / relative).lstat().st_mode):
            raise ValueError("Non-regular source")
        return target.read_bytes()

    changed = []
    lexical_changes = []
    for item in record["documents"]:
        raw = read(item["path"])
        if hashlib.sha256(raw).hexdigest() != item["sha256"]:
            changed.append(item["path"])
        if candidates(raw.decode()) != item["candidate_lines"]:
            lexical_changes.append(item["path"])
    for item in record["community_forms"]:
        raw = read(item["path"])
        if hashlib.sha256(raw).hexdigest() != item["sha256"]:
            changed.append(item["path"])
        if candidates(raw.decode()) != item["candidate_lines"]:
            lexical_changes.append(item["path"])
    for item in record["compatibility_anchors"]:
        text = read(item["path"]).decode()
        if 'id="' + item["legacy_anchor"] + '"' not in text:
            raise ValueError("Missing compatibility anchor: " + item["legacy_anchor"])
    original = read(record["translated_original"]["path"])
    if hashlib.sha256(original).hexdigest() != record["translated_original"]["sha256"]:
        raise ValueError("Historical Dutch original changed")
    equivalent = read(record["translated_original"]["english_equivalent"]).decode()
    source = original.decode()
    if not all(token in equivalent for token in re.findall(r"`([^`]+)`", source)):
        raise ValueError("Historical identifiers or commands changed")
    if set(re.findall(r"\b[0-9a-f]{40}\b", source)) != set(re.findall(r"\b[0-9a-f]{40}\b", equivalent)):
        raise ValueError("Historical commit identifiers changed")
    entrypoints = []
    if record.get("historical_entrypoints_manifest"):
        entrypoints = json.loads(read(record["historical_entrypoints_manifest"]))["cases"]
        for item in entrypoints:
            raw = read(item["original"])
            if hashlib.sha256(raw).hexdigest() != item["original_sha256"]:
                raise ValueError("Historical assessment original changed")
            target, _, fragment = item["english_entrypoint"].partition("#")
            english_raw = read(target)
            if hashlib.sha256(english_raw).hexdigest() != item["english_document_sha256"]:
                raise ValueError("English assessment entrypoint changed")
            text = english_raw.decode()
            if fragment and 'id="' + fragment + '"' not in text:
                raise ValueError("Missing English assessment entrypoint anchor")
            if item["coverage_kind"] == "Full historical English equivalent":
                original_text = raw.decode()
                if not all(token in text for token in re.findall(r"`([^`]+)`", original_text)):
                    raise ValueError("Session research identifier changed")
                if set(re.findall(r"\b[0-9a-f]{40}\b", original_text)) != set(re.findall(r"\b[0-9a-f]{40}\b", text)):
                    raise ValueError("Session research source commit changed")
                if not all(href in re.findall(r"\]\(([^)]+)\)", text) for href in re.findall(r"\]\(([^)]+)\)", original_text)):
                    raise ValueError("Session research reference target changed")
    if changed or lexical_changes:
        print(json.dumps({"source_changes": changed, "lexical_changes": lexical_changes, "inventory_result": "REVIEW REQUIRED"}))
        raise SystemExit(1)
    print(json.dumps({
        "inventory_result": "RECORDED SOURCE AND CANDIDATE DISPOSITIONS REPRODUCED",
        "first_party_documents": len(record["documents"]),
        "normal_navigation_documents": record["counts"]["normal_navigation_reachable"],
        "compatibility_anchors": len(record["compatibility_anchors"]),
        "historical_dutch_original_records_retained": len(record["historical_dutch_residuals"]),
        "historical_records_with_english_entrypoints": len(entrypoints),
        "language_quality_proof": False,
        "full_issue274_acceptance": "NOT COMPLETE",
        "ui_localisation": "OPEN; fixed Dutch native menu/status strings remain",
        "exit_code": 0,
    }))


if __name__ == "__main__":
    main()
