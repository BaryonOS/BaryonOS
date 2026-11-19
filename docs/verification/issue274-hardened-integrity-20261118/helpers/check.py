#!/usr/bin/env python3
"""Reproduce one immutable documentation inventory without reading its worktree."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import tempfile


WORDS = re.compile(r"\b(?:het|een|zijn|wordt|worden|voor|geen|alleen|ook|deze|dit|moet|moeten|zonder|niet|gegevens|gebruiker|bestanden|veiligheid|mogelijk|beschikbaar|toestemming|verplicht|ondersteund|huidige|volgende|bewijs|onderzoek|ontwerp)\b", re.IGNORECASE)
REDIRECT_ENV = {
    "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_COMMON_DIR", "GIT_CEILING_DIRECTORIES",
    "GIT_PREFIX", "GIT_CONFIG", "GIT_CONFIG_GLOBAL", "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_PARAMETERS", "GIT_NAMESPACE", "GIT_REPLACE_REF_BASE",
    "GIT_SHALLOW_FILE", "GIT_GRAFT_FILE", "GIT_EXEC_PATH",
}


def git_environment() -> dict[str, str]:
    environment = {
        name: value for name, value in os.environ.items()
        if name not in REDIRECT_ENV
        and not re.fullmatch(r"GIT_CONFIG_(?:COUNT|KEY_\d+|VALUE_\d+)", name)
    }
    environment.update({
        "GIT_NO_LAZY_FETCH": "1",
        "GIT_NO_REPLACE_OBJECTS": "1",
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": "/dev/null",
    })
    return environment


GIT_ENV = git_environment()


def git(repo: Path, *arguments: str) -> bytes:
    return subprocess.check_output(
        ["git", "--no-optional-locks", "-C", str(repo), *arguments],
        env=GIT_ENV,
    )


def canonical_commit(repo: Path, reference: str) -> str:
    if not re.fullmatch(r"[0-9a-f]{40}", reference):
        raise ValueError("A complete immutable commit SHA is required")
    value = git(repo, "rev-parse", "--verify", reference + "^{commit}").decode().strip()
    if value != reference:
        raise ValueError("Commit identity mismatch")
    return value


def git_tree(repo: Path, reference: str) -> dict[str, tuple[str, str, str]]:
    result: dict[str, tuple[str, str, str]] = {}
    for row in git(repo, "ls-tree", "-r", "-z", "--full-tree", reference).split(b"\0"):
        if not row:
            continue
        metadata, separator, raw_path = row.partition(b"\t")
        if not separator:
            raise ValueError("Invalid Git tree record")
        mode, kind, object_id = metadata.decode().split(" ")
        relative = raw_path.decode("utf-8", "strict")
        result[relative] = (mode, kind, object_id)
    return result


def regular(entry: tuple[str, str, str] | None) -> bool:
    return bool(entry and entry[0] in {"100644", "100755"} and entry[1] == "blob")


def read_blob(repo: Path, tree: dict[str, tuple[str, str, str]], relative: str) -> bytes:
    entry = tree.get(relative)
    if not regular(entry):
        raise ValueError("Inventory path is not a regular Git blob: " + relative)
    return git(repo, "cat-file", "blob", entry[2])


def protected(relative: str) -> bool:
    return (
        "/verification/" in relative
        or relative.startswith((
            "docs/verification/", "docs/history/", ".agents/skills/os-dev/references/corpus/",
            "docs/third-party/licenses/", "experiments/quantum/third-party/",
        ))
        or ("/vendor/" in relative and relative != "services/keyring/vendor/README.md")
    )


def candidate_lines(text: str) -> list[int]:
    result: list[int] = []
    fence: tuple[str, int] | None = None
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^ {0,3}((?:\x60){3,}|~{3,})(.*)$", line)
        if marker:
            current = (marker.group(1)[0], len(marker.group(1)))
            if fence is None:
                if current[0] != "\x60" or "\x60" not in marker.group(2):
                    fence = current
                    continue
            elif current[0] == fence[0] and current[1] >= fence[1] and not marker.group(2).strip():
                fence = None
                continue
        if fence:
            continue
        visible = re.sub(r"(?:\x60)+[^\x60]*(?:\x60)+", "", line)
        visible = re.sub(r"\]\([^)]*\)", "]", visible)
        visible = re.sub(r"https?://\S+", "", visible)
        visible = re.sub(r"<[^>]*>", "", visible)
        if WORDS.search(visible):
            result.append(number)
    return result


def safe_local_file(root: Path, value: str) -> Path:
    pure = PurePosixPath(value)
    if pure.is_absolute() or ".." in pure.parts:
        raise ValueError("Path escapes repository")
    source = root / Path(*pure.parts)
    if not stat.S_ISREG(source.lstat().st_mode):
        raise ValueError("Path is not a canonical regular file inside repository")
    target = source.resolve(strict=True)
    if source != target or not target.is_relative_to(root) or not stat.S_ISREG(target.lstat().st_mode):
        raise ValueError("Path is not a regular file inside repository")
    return target


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_manifest(root: Path, manifest_path: Path) -> dict[str, object]:
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("schema_version") != 1:
        raise ValueError("Unsupported evidence manifest schema")
    files = manifest.get("files")
    if not isinstance(files, dict) or not files:
        raise ValueError("Evidence manifest has no bound files")
    roles = {
        "inventory", "collector_output", "review", "collector_helper",
        "publication_record", "published_receipt",
    }
    if not roles.issubset(manifest):
        raise ValueError("Evidence manifest omits a required role")
    helpers = manifest.get("helpers")
    if not isinstance(helpers, list) or {PurePosixPath(value).name for value in helpers} != {"collect.cjs", "check.py", "verify.py"}:
        raise ValueError("Evidence manifest must bind the complete helper set")
    required = {manifest[role] for role in roles} | set(helpers)
    if not required.issubset(files):
        raise ValueError("Evidence manifest omits a required file digest")
    for relative, expected in files.items():
        if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
            raise ValueError("Invalid evidence manifest digest: " + relative)
        target = safe_local_file(root, relative)
        if digest(target) != expected:
            raise ValueError("Manifest digest mismatch: " + relative)
    return manifest


def normalized_output(value: dict[str, object]) -> dict[str, object]:
    result = dict(value)
    result.pop("generated_at_utc", None)
    return result


def reproduce_collector(
    root: Path,
    commit: str,
    wiki: Path,
    wiki_commit: str,
    review_path: Path,
    helper_path: Path,
    marked_node_path: Path,
) -> tuple[dict[str, object], dict[str, object]]:
    marked_package = marked_node_path / "marked" / "package.json"
    if not marked_package.is_file():
        raise ValueError("The supplied Node module path does not contain marked")
    environment = dict(GIT_ENV)
    environment["NODE_PATH"] = str(marked_node_path.resolve(strict=True))
    with tempfile.TemporaryDirectory(prefix="issue274-reproduce-") as location:
        command = [
            "node", str(helper_path), "--root", str(root), "--ref", commit,
            "--wiki", str(wiki), "--wiki-ref", wiki_commit,
            "--review", str(review_path), "--out", location,
        ]
        result = subprocess.run(command, env=environment, capture_output=True)
        if result.returncode != 0:
            raise ValueError(
                "Collector reproduction failed: "
                + result.stderr.decode("utf-8", "backslashreplace")
            )
        directory = Path(location)
        return (
            json.loads((directory / "coverage.json").read_text()),
            json.loads((directory / "collector.json").read_text()),
        )


def validate(
    root: Path,
    reference: str,
    inventory_path: Path,
    review_path: Path,
    manifest_path: Path,
    wiki: Path,
    marked_node_path: Path,
) -> dict[str, object]:
    commit = canonical_commit(root, reference)
    tree = git_tree(root, commit)
    manifest = validate_manifest(root, manifest_path)
    relative_inventory = inventory_path.relative_to(root).as_posix()
    relative_review = review_path.relative_to(root).as_posix()
    if manifest["inventory"] != relative_inventory or manifest["review"] != relative_review:
        raise ValueError("Evidence role path mismatch")
    inventory = json.loads(inventory_path.read_text())
    if inventory.get("source_commit") != commit:
        raise ValueError("Inventory source commit mismatch")
    if inventory.get("source_tree") != git(root, "rev-parse", commit + "^{tree}").decode().strip():
        raise ValueError("Inventory source tree mismatch")
    review = inventory.get("combined_review_manifest", {})
    if review.get("path") != review_path.relative_to(root).as_posix():
        raise ValueError("Review manifest path mismatch")
    if digest(review_path) != review.get("sha256"):
        raise ValueError("Review manifest digest mismatch")
    if manifest.get("source_commit") != commit or manifest.get("source_tree") != inventory.get("source_tree"):
        raise ValueError("Evidence manifest repository identity mismatch")
    if manifest.get("wiki_commit") != inventory.get("wiki_commit") or manifest.get("wiki_tree") != inventory.get("wiki_tree"):
        raise ValueError("Evidence manifest Wiki identity mismatch")
    actual_documents = {
        relative for relative, entry in tree.items()
        if relative.endswith(".md") and regular(entry) and not protected(relative)
    }
    expected_documents = {item["path"] for item in inventory["documents"]}
    if actual_documents != expected_documents:
        raise ValueError(json.dumps({
            "unclassified_documents": sorted(actual_documents - expected_documents),
            "missing_documents": sorted(expected_documents - actual_documents),
        }))
    form_pattern = re.compile(r"^\.github/(?:ISSUE_TEMPLATE|DISCUSSION_TEMPLATE)/.*\.ya?ml$", re.IGNORECASE)
    actual_forms = {relative for relative, entry in tree.items() if form_pattern.match(relative) and regular(entry)}
    expected_forms = {item["path"] for item in inventory["community_forms"]}
    if actual_forms != expected_forms:
        raise ValueError(json.dumps({
            "unclassified_forms": sorted(actual_forms - expected_forms),
            "missing_forms": sorted(expected_forms - actual_forms),
        }))
    for item in inventory["documents"] + inventory["community_forms"]:
        raw = read_blob(root, tree, item["path"])
        entry = tree[item["path"]]
        if entry[2] != item["blob"] or hashlib.sha256(raw).hexdigest() != item["sha256"]:
            raise ValueError("Source identity changed: " + item["path"])
        if candidate_lines(raw.decode("utf-8", "strict")) != item["candidate_lines"]:
            raise ValueError("Candidate lines changed: " + item["path"])
    for item in inventory["compatibility_anchors"]:
        text = read_blob(root, tree, item["path"]).decode("utf-8", "strict")
        patterns = (
            'id="' + item["legacy_anchor"] + '"',
            "id='" + item["legacy_anchor"] + "'",
            'name="' + item["legacy_anchor"] + '"',
            "name='" + item["legacy_anchor"] + "'",
        )
        if not any(pattern in text for pattern in patterns):
            raise ValueError("Compatibility anchor changed: " + item["legacy_anchor"])
    collector_output_path = safe_local_file(root, manifest["collector_output"])
    collector_helper_path = safe_local_file(root, manifest["collector_helper"])
    reproduced_inventory, reproduced_collector = reproduce_collector(
        root, commit, wiki.resolve(strict=True), inventory["wiki_commit"],
        review_path, collector_helper_path, marked_node_path.resolve(strict=True),
    )
    recorded_collector = json.loads(collector_output_path.read_text())
    if normalized_output(reproduced_inventory) != normalized_output(inventory):
        raise ValueError("Coverage output does not reproduce from immutable inputs")
    if normalized_output(reproduced_collector) != normalized_output(recorded_collector):
        raise ValueError("Collector output does not reproduce from immutable inputs")
    return {
        "source_commit": commit,
        "source_tree": inventory["source_tree"],
        "documents": len(inventory["documents"]),
        "community_forms": len(inventory["community_forms"]),
        "compatibility_anchors": len(inventory["compatibility_anchors"]),
        "manifest_files": len(manifest["files"]),
        "full_issue274_acceptance": False,
        "exit_code": 0,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--inventory", required=True)
    parser.add_argument("--review", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--wiki", type=Path, required=True)
    parser.add_argument("--marked-node-path", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    if not (root / ".git").exists() and not (root / ".git").is_file():
        raise ValueError("Repository root has no Git metadata")
    inventory = safe_local_file(root, args.inventory)
    review = safe_local_file(root, args.review)
    manifest = safe_local_file(root, args.manifest)
    print(json.dumps(validate(
        root, args.ref, inventory, review, manifest,
        args.wiki, args.marked_node_path,
    ), sort_keys=True))


if __name__ == "__main__":
    main()
