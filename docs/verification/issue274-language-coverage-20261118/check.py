"""Read-only, source-bound documentation language inventory check."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


WORDS = r"\b(?:het|een|zijn|wordt|worden|voor|geen|alleen|ook|deze|dit|moet|moeten|zonder|niet|gegevens|gebruiker|bestanden|veiligheid|mogelijk|beschikbaar|toestemming|verplicht|ondersteund|huidige|volgende|bewijs|onderzoek|ontwerp)\b"


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


def self_test(root, record):
    relative = Path("docs/verification/issue274-language-coverage-20261118")
    scratch = root / "build" / "english-coverage-20261118"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="controls-", dir=scratch) as location:
        temporary = Path(location).resolve()
        paths = {item["path"] for item in record["documents"] + record["community_forms"]}
        paths.add(str(relative / "check.py"))
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
    print(json.dumps({"self_test": "PASS", "raw_exits": {"unchanged_source": 0, "changed_prose": 1, "new_tracked_document": 1}, "expected_negative_exits": 1}))


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
        command += ["ls-tree", "-r", "--name-only", args.ref]
    else:
        command += ["ls-files", "--cached", "--", "*.md"]
    actual_paths = {
        relative for relative in subprocess.check_output(command, text=True).splitlines()
        if relative.endswith(".md") and not protected(relative)
    }
    expected_paths = {item["path"] for item in record["documents"]}
    if actual_paths != expected_paths:
        print(json.dumps({"inventory_result": "SCOPE REVIEW REQUIRED", "unclassified_paths": sorted(actual_paths - expected_paths), "missing_paths": sorted(expected_paths - actual_paths)}))
        raise SystemExit(1)

    def read(relative):
        target = (root / relative).resolve()
        if not target.is_relative_to(root):
            raise ValueError("Path outside repository")
        if args.ref:
            return subprocess.check_output(["git", "-C", str(root), "show", args.ref + ":" + relative])
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
    if changed or lexical_changes:
        print(json.dumps({"source_changes": changed, "lexical_changes": lexical_changes, "inventory_result": "REVIEW REQUIRED"}))
        raise SystemExit(1)
    print(json.dumps({
        "inventory_result": "RECORDED SOURCE AND CANDIDATE DISPOSITIONS REPRODUCED",
        "first_party_documents": len(record["documents"]),
        "normal_navigation_documents": record["counts"]["normal_navigation_reachable"],
        "compatibility_anchors": len(record["compatibility_anchors"]),
        "historical_dutch_records_without_equivalent": len(record["historical_dutch_residuals"]),
        "language_quality_proof": False,
        "full_issue274_acceptance": "NOT COMPLETE",
        "ui_localisation": "OPEN; fixed Dutch native menu/status strings remain",
        "exit_code": 0,
    }))


if __name__ == "__main__":
    main()
