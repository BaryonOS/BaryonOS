#!/usr/bin/env python3
"""Deterministic security regressions for the hardened issue #274 helpers."""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(command: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None,
        input_bytes: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(command, cwd=cwd, env=env, input=input_bytes, capture_output=True)


def need(result: subprocess.CompletedProcess[bytes], expected: int, label: str) -> None:
    if result.returncode != expected:
        raise ValueError(
            label + " returned " + str(result.returncode) + ", expected " + str(expected)
            + "\nstdout=" + result.stdout.decode("utf-8", "backslashreplace")
            + "\nstderr=" + result.stderr.decode("utf-8", "backslashreplace")
        )


def git(repo: Path, *arguments: str, input_bytes: bytes | None = None) -> str:
    result = run(
        ["git", "--no-optional-locks", "-C", str(repo), *arguments],
        input_bytes=input_bytes,
        env={**os.environ, "GIT_NO_REPLACE_OBJECTS": "1", "GIT_TERMINAL_PROMPT": "0"},
    )
    need(result, 0, "git " + " ".join(arguments))
    return result.stdout.decode().strip()


def initialise(repo: Path) -> None:
    repo.mkdir()
    git(repo, "init", "-q")
    git(repo, "config", "user.name", "Hardened fixture")
    git(repo, "config", "user.email", "fixture@example.invalid")


def commit(repo: Path, message: str) -> str:
    git(repo, "commit", "-qm", message)
    value = git(repo, "rev-parse", "HEAD")
    if len(value) != 40:
        raise ValueError("fixture commit is not canonical")
    return value


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def load(name: str, path: Path):
    specification = importlib.util.spec_from_file_location(name, path)
    if not specification or not specification.loader:
        raise ValueError("cannot load helper " + str(path))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def command_environment(marked_node_path: Path, trap: Path) -> dict[str, str]:
    return {
        **os.environ,
        "NODE_PATH": str(marked_node_path),
        "GIT_DIR": str(trap / ".git"),
        "GIT_WORK_TREE": str(trap),
        "GIT_INDEX_FILE": str(trap / ".git" / "index"),
        "GIT_CONFIG_COUNT": "1",
        "GIT_CONFIG_KEY_0": "core.replaceRefs",
        "GIT_CONFIG_VALUE_0": "true",
        "GIT_NAMESPACE": "hostile-namespace",
        "GIT_SHALLOW_FILE": str(trap / "hostile-shallow-file"),
        "GIT_REPLACE_REF_BASE": "refs/hostile-replace/",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--marked-node-path", type=Path, required=True)
    parser.add_argument("--live-root", type=Path, required=True)
    parser.add_argument("--live-wiki", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--published", type=Path, required=True)
    args = parser.parse_args()
    helpers = Path(__file__).resolve().parent
    draft = helpers.parent
    collector = helpers / "collect.cjs"
    checker = helpers / "check.py"
    verifier_path = helpers / "verify.py"
    checker_module = load("hardened_checker", checker)
    verifier = load("hardened_verifier", verifier_path)
    cases: dict[str, object] = {}
    raw_exits: dict[str, int] = {}
    fixture_text = "\n".join([
        "# Fixture",
        "",
        "[repository](docs/plain.md#heading)",
        "[Wiki duplicate](https://github.com/BaryonOS/BaryonOS/wiki/Target#a-1-1)",
        "[Wiki invalid fragment](https://github.com/BaryonOS/BaryonOS/wiki/Target#missing)",
        "[Wiki fenced anchor](https://github.com/BaryonOS/BaryonOS/wiki/Target#fenced-hidden)",
        "[Wiki inline anchor](https://github.com/BaryonOS/BaryonOS/wiki/Target#inline-hidden)",
        "[Wiki overlapping comment anchor](https://github.com/BaryonOS/BaryonOS/wiki/Target#overlap-hidden)",
        "[Wiki symlink](https://github.com/BaryonOS/BaryonOS/wiki/Alias)",
        "[Malformed](docs/bad%ZZ.md)",
        "",
    ])
    fence_text = "\n".join([
        "\x60\x60\x60\x60text",
        "Dit is een verborgen tekst.",
        "\x60\x60\x60",
        "Dit is ook verborgen.",
        "\x60\x60\x60\x60",
        "Dit is een zichtbaar document.",
        "",
    ])
    if checker_module.candidate_lines(fence_text) != [6]:
        raise ValueError("Python fence character/length regression")
    cases["python_fence_character_and_length"] = "PASS"
    try:
        verifier.strict_unquote("%ZZ")
    except ValueError:
        cases["strict_percent_decoder"] = "PASS"
    else:
        raise ValueError("malformed percent control unexpectedly succeeded")
    indented = "    \x60\x60\x60\nDit is zichtbaar.\n    \x60\x60\x60\n"
    illegal_info = "\x60\x60\x60bad\x60info\nDit is zichtbaar.\n\x60\x60\x60\n"
    if checker_module.candidate_lines(indented) != [2] or checker_module.candidate_lines(illegal_info) != [2]:
        raise ValueError("illegal CommonMark fence regression")
    cases["illegal_fence_openers"] = "PASS"
    anchor_fixture = "\x60\x60\x60\n# Hidden\n<a id=\"fenced-hidden\"></a>\n\x60\x60\x60\nInline \x60<a id=\"inline-hidden\"></a>\x60\n<div>\n<!<!--hidden-->--><a id=\"overlap-hidden\"></a>-->\n</div>\n\n# A\n# A\n# A-1\n"
    anchor_set = verifier.anchors(anchor_fixture)
    if {"hidden", "fenced-hidden", "inline-hidden"} & anchor_set or not {"a", "a-1", "a-1-1"}.issubset(anchor_set):
        raise ValueError("fence-aware duplicate-anchor regression")
    cases["fence_aware_duplicate_anchors"] = "PASS"

    with tempfile.TemporaryDirectory(prefix="self-test-", dir=draft) as location:
        temporary = Path(location)
        source, wiki, trap = temporary / "source", temporary / "wiki", temporary / "trap"
        initialise(source)
        initialise(wiki)
        initialise(trap)
        (trap / "TRAP.md").write_text("# Trap\n")
        git(trap, "add", "TRAP.md")
        commit(trap, "trap")
        (source / "docs" / "wiki").mkdir(parents=True)
        (source / ".github" / "ISSUE_TEMPLATE").mkdir(parents=True)
        (source / "README.md").write_text(fixture_text)
        (source / "docs" / "plain.md").write_text("# Heading\n")
        (source / "docs" / "fence.md").write_text(fence_text)
        (source / "docs" / "indented-fence.md").write_text(indented)
        (source / "docs" / "illegal-info-fence.md").write_text(illegal_info)
        (source / "docs" / "wiki" / "Page.md").write_text("[Relative](Target#a-1-1)\n")
        (source / ".github" / "ISSUE_TEMPLATE" / "fixture.yml").write_text("name: Fixture\ndescription: A safe fixture\n")
        (source / "review.json").write_text('{"review":"bound"}\n')
        git(source, "add", "--all")
        base = commit(source, "fixture base")
        unicode_blob = git(source, "hash-object", "-w", "--stdin", input_bytes=b"# Unicode\n")
        nfc = "docs/caf\u00e9.md"
        nfd = "docs/cafe\u0301.md"
        git(source, "update-index", "--add", "--cacheinfo", "100644," + unicode_blob + "," + nfc)
        git(source, "update-index", "--add", "--cacheinfo", "100644," + unicode_blob + "," + nfd)
        source_commit = commit(source, "exact Unicode paths in tree")
        (wiki / "Target.md").write_text(anchor_fixture)
        git(wiki, "add", "Target.md")
        alias_blob = git(wiki, "hash-object", "-w", "--stdin", input_bytes=b"Target.md")
        git(wiki, "update-index", "--add", "--cacheinfo", "120000," + alias_blob + ",Alias.md")
        wiki_commit = commit(wiki, "Wiki regular and symlink fixture")
        malicious_blob = git(source, "hash-object", "-w", "--stdin", input_bytes=b"# Replacement\n")
        malicious_tree = git(source, "mktree", input_bytes=("100644 blob " + malicious_blob + "\tREADME.md\n").encode())
        malicious_commit = git(source, "commit-tree", malicious_tree, "-m", "replacement")
        git(source, "replace", source_commit, malicious_commit)
        replaced = run(["git", "-C", str(source), "ls-tree", "-r", "--name-only", source_commit])
        need(replaced, 0, "replacement positive control")
        if b"docs/plain.md" in replaced.stdout:
            raise ValueError("replacement-ref fixture did not activate")
        environment = command_environment(args.marked_node_path, trap)
        evidence_helpers = source / "evidence" / "helpers"
        evidence_helpers.mkdir(parents=True)
        for helper in (collector, checker, verifier_path):
            shutil.copyfile(helper, evidence_helpers / helper.name)
        fixture_collector = evidence_helpers / "collect.cjs"
        out = source / "output"
        collect_command = [
            "node", str(fixture_collector), "--root", str(source), "--ref", source_commit,
            "--wiki", str(wiki), "--wiki-ref", wiki_commit,
            "--review", str(source / "review.json"), "--out", str(out),
        ]
        collected = run(collect_command, cwd=source, env=environment)
        need(collected, 0, "fixture collector")
        raw_exits["fixture_collector"] = collected.returncode
        coverage = json.loads((out / "coverage.json").read_text())
        collection = json.loads((out / "collector.json").read_text())
        paths = {item["path"] for item in coverage["documents"]}
        if not {nfc, nfd}.issubset(paths):
            raise ValueError("exact NFC/NFD tree paths omitted")
        fence_item = next(item for item in coverage["documents"] if item["path"] == "docs/fence.md")
        if fence_item["candidate_lines"] != [6]:
            raise ValueError("collector fence character/length regression")
        for relative in ("docs/indented-fence.md", "docs/illegal-info-fence.md"):
            item = next(value for value in coverage["documents"] if value["path"] == relative)
            if item["candidate_lines"] != [2]:
                raise ValueError("collector accepted an illegal fence opener: " + relative)
        if coverage["combined_review_manifest"]["path"] != "review.json":
            raise ValueError("review manifest path is not repository-relative")
        if coverage["counts"]["extracted_links"] <= coverage["counts"]["classified_local_links"]:
            raise ValueError("extracted and classified counts were not separated")
        unresolved = collection["unresolved_local_targets"]
        if not any(item.get("fragment") == "missing" and item.get("reason") == "unresolved-fragment" for item in unresolved):
            raise ValueError("invalid Wiki fragment not detected")
        for hidden in ("fenced-hidden", "inline-hidden", "overlap-hidden"):
            if not any(item.get("fragment") == hidden and item.get("reason") == "unresolved-fragment" for item in unresolved):
                raise ValueError("hidden explicit anchor was accepted: " + hidden)
        if any(item.get("fragment") == "a-1-1" and item.get("reason") == "unresolved-fragment" for item in unresolved):
            raise ValueError("duplicate GitHub-style anchor not resolved")
        if not any(item.get("target") == "Alias.md" and item.get("reason") == "missing-wiki-target" for item in unresolved):
            raise ValueError("non-regular Wiki target was accepted")
        if not any(item.get("reason") == "Invalid percent encoding" for item in collection["unsupported_local_targets"]):
            raise ValueError("malformed percent encoding was not bounded")
        if coverage["source_commit"] != source_commit or coverage["wiki_commit"] != wiki_commit:
            raise ValueError("exact source commits not recorded")
        cases.update({
            "immutable_git_tree_and_replacement_refs": "PASS",
            "inherited_git_redirection": "PASS",
            "unicode_nfc_nfd_tree_entries": "PASS",
            "wiki_fragment_duplicate_anchor": "PASS",
            "wiki_invalid_fragment": "PASS",
            "wiki_nonregular_blob": "PASS",
            "malformed_percent_encoding": "PASS",
            "yaml_form_inventory": "PASS",
            "split_link_counts": "PASS",
            "fixed_point_html_comment_elision": "PASS",
        })
        marker = source / "marker.txt"
        marker.write_text("bound\n")
        write_json(source / "publication.json", {"fixture": "publication"})
        write_json(source / "published.json", {"fixture": "receipt"})
        manifest = source / "manifest.json"
        helper_paths = ["evidence/helpers/collect.cjs", "evidence/helpers/check.py", "evidence/helpers/verify.py"]
        manifest_files = {
            "output/coverage.json": digest(out / "coverage.json"),
            "output/collector.json": digest(out / "collector.json"),
            "review.json": digest(source / "review.json"),
            "marker.txt": digest(marker),
            "publication.json": digest(source / "publication.json"),
            "published.json": digest(source / "published.json"),
        }
        manifest_files.update({relative: digest(source / relative) for relative in helper_paths})
        write_json(manifest, {
            "schema_version": 1,
            "source_commit": source_commit,
            "source_tree": coverage["source_tree"],
            "wiki_commit": wiki_commit,
            "wiki_tree": coverage["wiki_tree"],
            "inventory": "output/coverage.json",
            "collector_output": "output/collector.json",
            "review": "review.json",
            "collector_helper": "evidence/helpers/collect.cjs",
            "publication_record": "publication.json",
            "published_receipt": "published.json",
            "helpers": helper_paths,
            "files": manifest_files,
        })
        check_command = [
            "python3", "-B", str(checker), "--root", str(source), "--ref", source_commit,
            "--inventory", "output/coverage.json", "--review", "review.json", "--manifest", "manifest.json",
            "--wiki", str(wiki), "--marked-node-path", str(args.marked_node_path),
        ]
        checked = run(check_command, cwd=source, env=environment)
        need(checked, 0, "positive immutable checker")
        raw_exits["positive_immutable_checker"] = checked.returncode
        (source / "README.md").write_text("# Mutable worktree attack\n")
        (source / "docs" / "plain.md").unlink()
        checked_again = run(check_command, cwd=source, env=environment)
        need(checked_again, 0, "checker after worktree mutation")
        raw_exits["checker_after_worktree_mutation"] = checked_again.returncode
        cases["positive_immutable_inventory"] = "PASS"
        missing_roles = json.loads(manifest.read_text())
        missing_roles.pop("publication_record")
        write_json(source / "manifest-missing-role.json", missing_roles)
        missing_role_command = check_command.copy()
        missing_role_command[missing_role_command.index("manifest.json")] = "manifest-missing-role.json"
        missing_role_result = run(missing_role_command, cwd=source, env=environment)
        if missing_role_result.returncode == 0 or b"omits a required role" not in missing_role_result.stderr:
            raise ValueError("manifest omission was accepted")
        raw_exits["manifest_missing_wiki_role_negative"] = missing_role_result.returncode
        cases["manifest_required_wiki_roles"] = "PASS"
        outside = temporary / "outside.json"
        shutil.copyfile(out / "coverage.json", outside)
        link = source / "inventory-link.json"
        link.symlink_to(outside)
        symlink_command = check_command.copy()
        symlink_command[symlink_command.index("output/coverage.json")] = "inventory-link.json"
        symlink_check = run(symlink_command, cwd=source, env=environment)
        if symlink_check.returncode == 0:
            raise ValueError("symlinked inventory escaped canonical check")
        raw_exits["symlinked_inventory_negative"] = symlink_check.returncode
        cases["canonical_inventory_path"] = "PASS"
        marker.write_text("tampered\n")
        tampered_manifest = run(check_command, cwd=source, env=environment)
        if tampered_manifest.returncode == 0 or b"Manifest digest mismatch" not in tampered_manifest.stderr:
            raise ValueError("manifest SHA binding did not reject mutation")
        raw_exits["manifest_tamper_negative"] = tampered_manifest.returncode
        marker.write_text("bound\n")
        cases["manifest_sha_binding"] = "PASS"
        count_coverage = json.loads((out / "coverage.json").read_text())
        count_coverage["counts"]["extracted_links"] = 0
        write_json(source / "coverage-count-tamper.json", count_coverage)
        count_manifest = json.loads(manifest.read_text())
        count_manifest["inventory"] = "coverage-count-tamper.json"
        count_manifest["files"]["coverage-count-tamper.json"] = digest(source / "coverage-count-tamper.json")
        write_json(source / "manifest-count-tamper.json", count_manifest)
        count_command = check_command.copy()
        count_command[count_command.index("output/coverage.json")] = "coverage-count-tamper.json"
        count_command[count_command.index("manifest.json")] = "manifest-count-tamper.json"
        count_result = run(count_command, cwd=source, env=environment)
        if count_result.returncode == 0 or b"Coverage output does not reproduce" not in count_result.stderr:
            raise ValueError("tampered coverage count was accepted")
        raw_exits["coverage_count_tamper_negative"] = count_result.returncode
        cases["collector_semantic_count_reproduction"] = "PASS"
        drift = source / ".github" / "ISSUE_TEMPLATE" / "drift.yaml"
        drift.write_text("name: Drift\ndescription: Must be discovered\n")
        git(source, "add", ".github/ISSUE_TEMPLATE/drift.yaml")
        drift_commit = commit(source, "new YAML form")
        drift_coverage = json.loads((out / "coverage.json").read_text())
        drift_coverage["source_commit"] = drift_commit
        drift_coverage["source_tree"] = git(source, "rev-parse", drift_commit + "^{tree}")
        write_json(source / "coverage-drift.json", drift_coverage)
        drift_files = {
            "coverage-drift.json": digest(source / "coverage-drift.json"),
            "output/collector.json": digest(out / "collector.json"),
            "review.json": digest(source / "review.json"),
            "marker.txt": digest(marker),
            "publication.json": digest(source / "publication.json"),
            "published.json": digest(source / "published.json"),
        }
        drift_files.update({relative: digest(source / relative) for relative in helper_paths})
        write_json(source / "manifest-drift.json", {
            "schema_version": 1,
            "source_commit": drift_commit,
            "source_tree": drift_coverage["source_tree"],
            "wiki_commit": wiki_commit,
            "wiki_tree": coverage["wiki_tree"],
            "inventory": "coverage-drift.json",
            "collector_output": "output/collector.json",
            "review": "review.json",
            "collector_helper": "evidence/helpers/collect.cjs",
            "publication_record": "publication.json",
            "published_receipt": "published.json",
            "helpers": helper_paths,
            "files": drift_files,
        })
        drift_command = [
            "python3", "-B", str(checker), "--root", str(source), "--ref", drift_commit,
            "--inventory", "coverage-drift.json", "--review", "review.json", "--manifest", "manifest-drift.json",
            "--wiki", str(wiki), "--marked-node-path", str(args.marked_node_path),
        ]
        drift_result = run(drift_command, cwd=source, env=environment)
        if drift_result.returncode == 0 or b"unclassified_forms" not in drift_result.stderr:
            raise ValueError("new YAML form drift was not rejected")
        raw_exits["yaml_form_drift_negative"] = drift_result.returncode
        cases["yaml_form_drift_negative"] = "PASS"
        short_ref = collect_command.copy()
        short_ref[short_ref.index(source_commit)] = "HEAD"
        noncanonical = run(short_ref, cwd=source, env=environment)
        if noncanonical.returncode == 0 or b"complete immutable commit SHA" not in noncanonical.stderr:
            raise ValueError("mutable source ref was accepted")
        raw_exits["mutable_ref_negative"] = noncanonical.returncode
        cases["full_sha_required"] = "PASS"
        wiki_index = verifier.tree(wiki, wiki_commit)
        try:
            verifier.blob(wiki, wiki_index, "Alias.md")
        except ValueError:
            cases["verifier_wiki_regular_blob"] = "PASS"
        else:
            raise ValueError("verifier accepted Wiki symlink blob")
        regular_entry = ("100644", "blob", "0" * 40)
        artificial_before = {"A.md": regular_entry, "B.md": regular_entry}
        artificial_after = {**artificial_before, "New.md": regular_entry}
        try:
            verifier.validate_wiki_domain(
                artificial_before, artificial_after,
                [{"path": "A.md"}], [{"path": "docs/wiki/New.md"}],
            )
        except ValueError:
            cases["wiki_baseline_inventory_completeness_negative"] = "PASS"
        else:
            raise ValueError("incomplete Wiki baseline inventory was accepted")
        verifier.validate_wiki_domain(
            artificial_before, artificial_after,
            [{"path": "A.md"}, {"path": "B.md"}], [{"path": "docs/wiki/New.md"}],
        )
        cases["wiki_baseline_inventory_complete_positive"] = "PASS"
        (wiki / "Target.md").write_text("# Changed\n")
        git(wiki, "add", "Target.md")
        changed_wiki_commit = commit(wiki, "mutated preserved page")
        changed_wiki_index = verifier.tree(wiki, changed_wiki_commit)
        before_target = verifier.blob(wiki, wiki_index, "Target.md")
        preserved_record = [{
            "path": "Target.md",
            "sha256_before": hashlib.sha256(before_target).hexdigest(),
            "preserved_byte_identical": True,
        }]
        try:
            verifier.verify_existing_pages(wiki, wiki_index, changed_wiki_index, preserved_record)
        except ValueError:
            cases["wiki_before_after_preservation_negative"] = "PASS"
        else:
            raise ValueError("changed preserved Wiki page was accepted")
        verifier.require_ancestor(wiki, wiki_commit, changed_wiki_commit)
        unrelated_wiki_commit = git(
            wiki, "commit-tree", git(wiki, "rev-parse", changed_wiki_commit + "^{tree}"),
            "-m", "unrelated Wiki history",
        )
        try:
            verifier.require_ancestor(wiki, wiki_commit, unrelated_wiki_commit)
        except ValueError:
            cases["wiki_baseline_ancestry_negative"] = "PASS"
        else:
            raise ValueError("unrelated Wiki publication history was accepted")

        tampered = json.loads(args.published.read_text())
        tampered["published_files"][0]["sha256"] = "0" * 64
        tampered_path = draft / "tampered-published-self-test.json"
        write_json(tampered_path, tampered)
        optimise_environment = {**os.environ, "PYTHONOPTIMIZE": "1"}
        verify_command = [
            "python3", "-O", str(verifier_path), "--root", str(args.live_root),
            "--wiki", str(args.live_wiki), "--record", str(args.record), "--published", str(tampered_path),
        ]
        optimised = run(verify_command, cwd=args.live_root, env=optimise_environment)
        tampered_path.unlink()
        if optimised.returncode == 0 or b"Published digest mismatch" not in optimised.stderr:
            raise ValueError("optimised verifier suppressed an integrity check")
        raw_exits["python_optimize_tamper_negative"] = optimised.returncode
        for field in ("source_candidate", "wiki_before", "wiki_after"):
            symbolic = json.loads(args.published.read_text())
            symbolic[field] = "HEAD"
            write_json(tampered_path, symbolic)
            symbolic_command = [
                "python3", "-B", str(verifier_path), "--root", str(args.live_root),
                "--wiki", str(args.live_wiki), "--record", str(args.record), "--published", str(tampered_path),
            ]
            symbolic_result = run(symbolic_command, cwd=args.live_root)
            if symbolic_result.returncode == 0 or b"complete immutable commit SHA" not in symbolic_result.stderr:
                raise ValueError("verifier accepted symbolic ref: " + field)
            raw_exits["verifier_symbolic_" + field + "_negative"] = symbolic_result.returncode
        symbolic_record = json.loads(args.record.read_text())
        symbolic_record["repository_source"] = "HEAD"
        symbolic_record_path = draft / "tampered-record-self-test.json"
        write_json(symbolic_record_path, symbolic_record)
        record_command = [
            "python3", "-B", str(verifier_path), "--root", str(args.live_root),
            "--wiki", str(args.live_wiki), "--record", str(symbolic_record_path), "--published", str(args.published),
        ]
        record_result = run(record_command, cwd=args.live_root)
        symbolic_record_path.unlink()
        tampered_path.unlink()
        if record_result.returncode == 0 or b"complete immutable commit SHA" not in record_result.stderr:
            raise ValueError("verifier accepted symbolic repository source")
        raw_exits["verifier_symbolic_repository_source_negative"] = record_result.returncode
        cases["verifier_all_source_refs_require_full_sha"] = "PASS"
        parsed = ast.parse(verifier_path.read_text())
        if any(isinstance(node, ast.Assert) for node in ast.walk(parsed)):
            raise ValueError("verifier still contains an assert statement")
        cases["python_optimize_integrity_negative"] = "PASS"
        cases["assert_free_verifier_ast"] = "PASS"
        cases["base_fixture_commit"] = base
        cases["source_fixture_commit"] = source_commit
        cases["wiki_fixture_commit"] = wiki_commit
    print(json.dumps({
        "self_test": "PASS",
        "cases": cases,
        "raw_exits": raw_exits,
        "expected_negative_exit": 1,
        "exit_code": 0,
    }, sort_keys=True))


if __name__ == "__main__":
    main()
