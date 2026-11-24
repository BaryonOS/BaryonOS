#!/usr/bin/env python3
"""CI impact selector for BaryonOS (phase 2a: shadow only, no CI wiring).

The selector maps the cumulative diff of a change onto the CI executions
registered in `.github/ci-impact.toml` and writes a deterministic plan. It
never runs tests and never narrows on doubt: every runtime error, missing
history, unknown path or central input yields `mode = "full"` with a
`fallback` code and exit status 0. Only usage errors exit 2.

The file is deliberately self-contained (standard library only, no imports
from `tools/`) so a later phase can run the trusted copy taken from the base
commit. `coverage` additionally needs PyYAML and reports drift between the
catalogue and the checked-in workflows; it is a report, not an enforcement.
See docs/CI-CONTRACT.md.
"""

from __future__ import annotations

import argparse
import functools
import glob as globmod
import hashlib
import itertools
import json
import os
import posixpath
import re
import subprocess
import sys
import tomllib

PLAN_SCHEMA = "baryonos.ci-impact.plan"
COVERAGE_SCHEMA = "baryonos.ci-impact.coverage"
SCHEMA_VERSION = 1
CATALOGUE_PATH = ".github/ci-impact.toml"
SELECTOR_PATH = "tools/ci_impact.py"

CLASSES = ("core", "rwa", "advisory", "manual", "operational", "publication")
SELECTABLE = ("core", "rwa")
EVENTS = ("pull_request", "merge_group", "push", "schedule", "workflow_dispatch")
FILTERED_EVENTS = ("pull_request", "push", "merge_group")
GRANULARITIES = ("variant", "job", "workflow")
POLICY_MODES = ("shadow", "enforce", "select", "full")
EVENT_POLICIES = ("select", "full", "manual")
ESCAPE_WHEN = ("always", "root")
# Tool configuration cargo, rustfmt and clippy read from any ancestor directory.
REQUIRED_ALWAYS_FULL = (
    "rustfmt.toml", "**/rustfmt.toml", ".rustfmt.toml", "**/.rustfmt.toml",
    "clippy.toml", "**/clippy.toml", ".clippy.toml", "**/.clippy.toml",
    "rust-toolchain*", "**/rust-toolchain*", ".cargo/**", "**/.cargo/**")
# Ids reach Markdown summaries and check lookups; keep them to inert characters.
WORKFLOW_PATH_RE = re.compile(r"\.github/workflows/[A-Za-z0-9_][A-Za-z0-9_-]*\.ya?ml")
JOB_ID_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_-]*")
VARIANT_ID_RE = re.compile(r"[A-Za-z0-9_.*-]+")
EVENT_FULL_CODES = {"merge_group": "merge-group", "push": "push-main", "schedule": "schedule"}
METADATA = "@metadata"
ZERO_SHA = "0" * 40
SHA_RE = re.compile(r"[0-9a-f]{40}")
CITATION_RE = re.compile(r"[^:\s]+:[1-9][0-9]*(-[1-9][0-9]*)?")
ACTIVATION_RE = re.compile(r"automatic|dispatch|schedule|label:[^\s]+|push-branch:[^\s]+")
DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
ROOT_ONLY_TARGETS = ("bin", "test", "example", "bench")

MAX_CHANGED = 5000
MAX_LISTED_PATHS = 1000
MAX_DETAIL = 20
MAX_REASONS = 3
MAX_MANIFESTS = 500
FILTER_FILE_LIMIT = 300
DEEPEN_STEPS = (64, 256, 1024, 4096)
GIT_TIMEOUT = 120

TOP_KEYS = {"schema_version", "revision", "policy", "input_sets", "crate_escape", "workflow"}
POLICY_KEYS = {"mode", "always_full", "inert", "core_only", "events"}
WORKFLOW_KEYS = {"path", "owner", "granularity", "events", "obligation", "trigger", "job"}
TRIGGER_KEYS = {"types", "branches", "paths", "paths_ignore"}
JOB_KEYS = {"id", "name", "class", "activation", "condition", "needs", "timeout_minutes",
            "max_parallel", "cargo", "input_sets", "inputs", "evidence", "ruleset_required", "variant"}
VARIANT_KEYS = {"id", "check_name", "matrix", "dynamic", "cargo", "input_sets", "inputs", "evidence"}
ESCAPE_KEYS = {"crate", "when", "inputs", "evidence"}


class Full(Exception):
    """A reason that forces mode=full; `detail` lists at most MAX_DETAIL items."""

    def __init__(self, code, detail=()):
        super().__init__(code)
        self.code = code
        self.detail = list(detail)[:MAX_DETAIL]


# ---------------------------------------------------------------- globs

@functools.lru_cache(maxsize=None)
def _glob_regex(pattern):
    regex, i = "", 0
    while i < len(pattern):
        if pattern.startswith("**", i):
            regex += ".*"
            i += 2
        elif pattern[i] == "*":
            regex += "[^/]*"
            i += 1
        else:
            regex += re.escape(pattern[i])
            i += 1
    # DOTALL: a control character in a path must not escape a '**' match.
    return re.compile(regex, re.DOTALL)


def github_path_filter_matches(pattern, path):
    """Match GitHub path-filter globs: '*' stays within a segment, '**' spans."""
    return _glob_regex(pattern).fullmatch(path) is not None


def first_match(patterns, path):
    for pattern in patterns:
        if github_path_filter_matches(pattern, path):
            return pattern
    return None


def glob_problem(pattern):
    if not isinstance(pattern, str) or not pattern:
        return "empty or non-string glob"
    if pattern.startswith(("/", "!")) or "\\" in pattern:
        return "leading '/', '!' or a backslash"
    if any(part in ("..", ".") for part in pattern.split("/")):
        return "'.' or '..' segment"
    if any(char in pattern for char in "?[]+"):
        # GitHub gives these meaning; the matcher here models only '*' and '**'.
        return "unsupported glob syntax ('?', '[', ']' or '+')"
    return None


# ---------------------------------------------------------------- git

def _git_env():
    env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    env.update(GIT_TERMINAL_PROMPT="0", LC_ALL="C")
    return env


def git(repo, *args, timeout=GIT_TIMEOUT):
    """Run git; return (returncode, stdout bytes). Never raises on a git failure."""
    try:
        result = subprocess.run(["git", "-C", repo, *args], capture_output=True,
                                env=_git_env(), timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        return 124, b""
    return result.returncode, result.stdout


def git_blob(repo, sha, path):
    """Blob bytes, or None when `path` is absent at `sha`. A git error is never absence."""
    code, out = git(repo, "cat-file", "blob", f"{sha}:{path}")
    if code == 0:
        return out
    code, listing = git(repo, "ls-tree", "-z", "--full-tree", sha, "--", path)
    if code == 0 and not listing:
        return None
    raise Full("git-read-failed", [f"{sha}:{path}"])


def has_commit(repo, sha):
    return git(repo, "cat-file", "-e", f"{sha}^{{commit}}")[0] == 0


def is_ancestor(repo, older, newer):
    return git(repo, "merge-base", "--is-ancestor", older, newer)[0] == 0


def fetch(repo, *args):
    return git(repo, "fetch", "--no-tags", "--no-recurse-submodules", "--quiet",
               "origin", *args)[0] == 0


def blob_id(data):
    """Git blob id of bytes, without calling git."""
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


# ---------------------------------------------------------------- catalogue

def load_catalogue_text(text):
    return tomllib.loads(text)


def _list_of_str(value):
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def stem(path):
    return posixpath.splitext(posixpath.basename(path))[0]


def default_check_name(job, combination=None):
    """Rendered check-run name for a job, optionally one matrix combination."""
    name = job.get("name")
    if combination is None:
        return name if name is not None else job["id"]
    if name is not None and "${{ matrix." in name:
        return re.sub(r"\$\{\{ matrix\.(\w+) \}\}", lambda m: str(combination[m.group(1)]), name)
    values = ", ".join(_scalar_text(value) for value in combination.values())
    return f"{name if name is not None else job['id']} ({values})"


def _scalar_text(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


def executions(catalogue):
    """Yield one record per registered execution (job or job variant)."""
    out = []
    for workflow in catalogue.get("workflow", []):
        for job in workflow.get("job", []):
            variants = job.get("variant") or [None]
            for variant in variants:
                xid = f"{stem(workflow['path'])}#{job['id']}"
                if variant is not None:
                    xid += f"/{variant['id']}"
                    check_name = variant.get("check_name", "")
                else:
                    check_name = default_check_name(job)
                out.append({"id": xid, "workflow": workflow, "job": job, "variant": variant,
                            "check_name": check_name, "class": job.get("class")})
    return out


def validate(catalogue):
    """Structural validation only: no filesystem or git access. Returns problems."""
    problems = []
    add = problems.append
    if not isinstance(catalogue, dict):
        return ["catalogue is not a table"]
    for key in sorted(set(catalogue) - TOP_KEYS):
        add(f"unknown top-level key: {key}")
    if catalogue.get("schema_version") != SCHEMA_VERSION:
        add(f"schema_version must be {SCHEMA_VERSION}")
    if not isinstance(catalogue.get("revision"), int) or isinstance(catalogue.get("revision"), bool):
        add("revision must be an integer")

    def globs(where, value, required=False):
        if value is None:
            if required:
                add(f"{where}: missing glob list")
            return
        if not isinstance(value, list):
            add(f"{where}: must be a list")
            return
        for pattern in value:
            problem = glob_problem(pattern)
            if problem:
                add(f"{where}: {pattern!r}: {problem}")

    policy = catalogue.get("policy")
    if not isinstance(policy, dict):
        add("policy table missing")
        policy = {}
    for key in sorted(set(policy) - POLICY_KEYS):
        add(f"policy: unknown key: {key}")
    if policy.get("mode") not in POLICY_MODES:
        add(f"policy.mode must be one of {', '.join(POLICY_MODES)}")
    for key in ("always_full", "inert", "core_only"):
        globs(f"policy.{key}", policy.get(key), required=True)
    if isinstance(policy.get("always_full"), list):
        for pattern in REQUIRED_ALWAYS_FULL:
            if pattern not in policy["always_full"]:
                add(f"policy.always_full must include {pattern!r}")
    events = policy.get("events")
    if not isinstance(events, dict):
        add("policy.events table missing")
        events = {}
    for event in EVENTS:
        if event in EVENT_FULL_CODES:
            if events.get(event) != "full":
                add(f"policy.events.{event} must be 'full'")
        elif event == "workflow_dispatch":
            if events.get(event) not in ("full", "manual"):
                add(f"policy.events.{event} must be 'full' or 'manual'")
        elif events.get(event) not in EVENT_POLICIES:
            add(f"policy.events.{event} must be one of {', '.join(EVENT_POLICIES)}")
    for event in sorted(set(events) - set(EVENTS)):
        add(f"policy.events: unknown event {event}")

    input_sets = catalogue.get("input_sets", {})
    if not isinstance(input_sets, dict):
        add("input_sets must be a table")
        input_sets = {}
    for name, value in input_sets.items():
        globs(f"input_sets.{name}", value, required=True)

    escapes = catalogue.get("crate_escape", [])
    if not isinstance(escapes, list):
        add("crate_escape must be an array of tables")
        escapes = []
    for index, escape in enumerate(escapes):
        where = f"crate_escape[{index}]"
        if not isinstance(escape, dict):
            add(f"{where}: not a table")
            continue
        for key in sorted(set(escape) - ESCAPE_KEYS):
            add(f"{where}: unknown key: {key}")
        if glob_problem(escape.get("crate")) or "*" in str(escape.get("crate")):
            add(f"{where}: crate must be a plain repository directory")
        if escape.get("when") not in ESCAPE_WHEN:
            add(f"{where}: when must be 'always' or 'root'")
        globs(f"{where}.inputs", escape.get("inputs"), required=True)
        if not escape.get("inputs"):
            add(f"{where}: inputs must not be empty")
        if not _list_of_str(escape.get("evidence", [])):
            add(f"{where}: evidence must be a list of strings")

    workflows = catalogue.get("workflow")
    if not isinstance(workflows, list) or not workflows:
        add("at least one [[workflow]] is required")
        workflows = []
    seen_paths, seen_ids, seen_checks = set(), set(), {}
    for windex, workflow in enumerate(workflows):
        if not isinstance(workflow, dict):
            add(f"workflow[{windex}]: not a table")
            continue
        path = workflow.get("path")
        where = f"workflow {path!r}"
        for key in sorted(set(workflow) - WORKFLOW_KEYS):
            add(f"{where}: unknown key: {key}")
        if not isinstance(path, str) or not WORKFLOW_PATH_RE.fullmatch(path):
            add(f"{where}: path must be .github/workflows/<id>.yml, id of [A-Za-z0-9_-]")
            continue
        if path in seen_paths:
            add(f"{where}: registered twice")
        seen_paths.add(path)
        if not isinstance(workflow.get("owner"), str) or not workflow.get("owner"):
            add(f"{where}: owner required")
        if workflow.get("granularity") not in GRANULARITIES:
            add(f"{where}: granularity must be one of {', '.join(GRANULARITIES)}")
        wf_events = workflow.get("events")
        if not _list_of_str(wf_events) or not wf_events or len(set(wf_events)) != len(wf_events):
            add(f"{where}: events must be a non-empty list of unique strings")
            wf_events = []
        obligation = workflow.get("obligation", [])
        if not isinstance(obligation, list) or not all(_citation_ok(c) for c in obligation):
            add(f"{where}: obligation must be a list of {{ cite = 'file:line', anchor = '<text on that line>' }}")
            obligation = []
        trigger = workflow.get("trigger", {})
        if not isinstance(trigger, dict):
            add(f"{where}: trigger must be a table")
            trigger = {}
        for event, spec in trigger.items():
            if event not in FILTERED_EVENTS:
                add(f"{where}: trigger.{event}: only {', '.join(FILTERED_EVENTS)} are recorded")
                continue
            if event not in wf_events:
                add(f"{where}: trigger.{event} without that event")
            if not isinstance(spec, dict):
                add(f"{where}: trigger.{event} must be a table")
                continue
            for key in sorted(set(spec) - TRIGGER_KEYS):
                add(f"{where}: trigger.{event}: unknown key: {key}")
            for key in ("types", "branches"):
                if key in spec and not _list_of_str(spec[key]):
                    add(f"{where}: trigger.{event}.{key} must be a list of strings")
            if "paths" in spec:
                globs(f"{where}: trigger.{event}.paths", spec["paths"])
            if "paths_ignore" in spec and spec["paths_ignore"] != METADATA:
                globs(f"{where}: trigger.{event}.paths_ignore", spec["paths_ignore"])
            if "paths" in spec and "paths_ignore" in spec:
                add(f"{where}: trigger.{event} has both paths and paths_ignore")
        jobs = workflow.get("job")
        if not isinstance(jobs, list) or not jobs:
            add(f"{where}: at least one [[workflow.job]] is required")
            continue
        job_ids = [job.get("id") for job in jobs if isinstance(job, dict)]
        if len(set(job_ids)) != len(job_ids):
            add(f"{where}: duplicate job ids")
        graph = {}
        for job in jobs:
            if not isinstance(job, dict):
                add(f"{where}: job is not a table")
                continue
            jwhere = f"{stem(path)}#{job.get('id')}"
            for key in sorted(set(job) - JOB_KEYS):
                add(f"{jwhere}: unknown key: {key}")
            if not isinstance(job.get("id"), str) or not JOB_ID_RE.fullmatch(job["id"]):
                add(f"{where}: invalid job id {job.get('id')!r}")
                continue
            klass, activation = job.get("class"), job.get("activation")
            if klass not in CLASSES:
                add(f"{jwhere}: class must be one of {', '.join(CLASSES)}")
            if not isinstance(activation, str) or not ACTIVATION_RE.fullmatch(activation):
                add(f"{jwhere}: invalid activation {activation!r}")
            if klass in SELECTABLE and activation != "automatic":
                add(f"{jwhere}: {klass} jobs must be automatic")
            if klass == "manual" and activation == "automatic":
                add(f"{jwhere}: a manual job cannot be automatic")
            if klass == "core" and "condition" in job:
                add(f"{jwhere}: a core job must not have a condition")
            if klass == "rwa" and not obligation:
                add(f"{jwhere}: rwa requires a workflow obligation citation")
            for key in ("name", "condition"):
                if key in job and not isinstance(job[key], str):
                    add(f"{jwhere}: {key} must be a string")
            for key in ("timeout_minutes", "max_parallel"):
                if key in job and not isinstance(job[key], (int, str)):
                    add(f"{jwhere}: {key} must be an integer or an expression string")
            if "ruleset_required" in job and not isinstance(job["ruleset_required"], bool):
                add(f"{jwhere}: ruleset_required must be a boolean")
            needs = job.get("needs", [])
            if not _list_of_str(needs):
                add(f"{jwhere}: needs must be a list of strings")
                needs = []
            for need in needs:
                if need == "merge-hold":
                    add(f"{jwhere}: no job may need merge-hold")
                elif need not in job_ids:
                    add(f"{jwhere}: needs unknown job {need}")
            graph[job["id"]] = needs
            _validate_inputs(add, jwhere, job, input_sets)
            if not _list_of_str(job.get("evidence", [])):
                add(f"{jwhere}: evidence must be a list of strings")
            variants = job.get("variant")
            if variants is not None:
                if not isinstance(variants, list) or not variants:
                    add(f"{jwhere}: variant must be a non-empty array of tables")
                    variants = []
                ids = [variant.get("id") for variant in variants if isinstance(variant, dict)]
                if len(set(ids)) != len(ids):
                    add(f"{jwhere}: duplicate variant ids")
                for variant in variants:
                    if not isinstance(variant, dict):
                        add(f"{jwhere}: variant is not a table")
                        continue
                    vwhere = f"{jwhere}/{variant.get('id')}"
                    for key in sorted(set(variant) - VARIANT_KEYS):
                        add(f"{vwhere}: unknown key: {key}")
                    if not isinstance(variant.get("id"), str) or not VARIANT_ID_RE.fullmatch(variant["id"]):
                        add(f"{jwhere}: invalid variant id {variant.get('id')!r}")
                    if not isinstance(variant.get("check_name"), str) or not variant.get("check_name"):
                        add(f"{vwhere}: check_name required")
                    if variant.get("dynamic", False) is not (variant.get("id") == "*"):
                        add(f"{vwhere}: id '*' if and only if dynamic = true")
                    if not isinstance(variant.get("matrix", {}), dict):
                        add(f"{vwhere}: matrix must be a table")
                    _validate_inputs(add, vwhere, variant, input_sets)
                    if not _list_of_str(variant.get("evidence", [])):
                        add(f"{vwhere}: evidence must be a list of strings")
            if klass == "rwa" and not _has_inputs(job) and not (
                    variants and all(_has_inputs(v) for v in variants if isinstance(v, dict))):
                add(f"{jwhere}: rwa requires inputs (cargo, inputs or input_sets)")
        for cycle_job in _cycle_members(graph):
            add(f"{stem(path)}#{cycle_job}: needs cycle")
    if not problems:
        for execution in executions(catalogue):
            if execution["id"] in seen_ids:
                add(f"duplicate execution id {execution['id']}")
            seen_ids.add(execution["id"])
            other = seen_checks.get(execution["check_name"])
            if other:
                add(f"check name {execution['check_name']!r} used by {other} and {execution['id']}")
            seen_checks[execution["check_name"]] = execution["id"]
    return problems


def _citation_ok(citation):
    return isinstance(citation, dict) and set(citation) == {"cite", "anchor"} \
        and isinstance(citation["cite"], str) and CITATION_RE.fullmatch(citation["cite"]) is not None \
        and isinstance(citation["anchor"], str) and citation["anchor"].strip() != ""


def _has_inputs(table):
    return bool(table.get("cargo") or table.get("inputs") or table.get("input_sets"))


def _validate_inputs(add, where, table, input_sets):
    for key in ("cargo", "inputs", "input_sets"):
        if key in table and not _list_of_str(table[key]):
            add(f"{where}: {key} must be a list of strings")
            return
    for manifest in table.get("cargo", []):
        if glob_problem(manifest) or "*" in manifest or not manifest.endswith("Cargo.toml"):
            add(f"{where}: cargo root {manifest!r} must be a plain Cargo.toml path")
    for pattern in table.get("inputs", []):
        problem = glob_problem(pattern)
        if problem:
            add(f"{where}: inputs {pattern!r}: {problem}")
    for name in table.get("input_sets", []):
        if name not in input_sets:
            add(f"{where}: unknown input set {name}")


def _cycle_members(graph):
    state, members = {}, set()

    def visit(node, stack):
        state[node] = 1
        for need in graph.get(node, []):
            if state.get(need) == 1:
                members.update(stack[stack.index(need):] if need in stack else [need])
            elif need in graph and state.get(need) is None:
                visit(need, stack + [need])
        state[node] = 2

    for node in graph:
        if state.get(node) is None:
            visit(node, [node])
    return sorted(members)


# ---------------------------------------------------------------- cargo closure

def repo_join(directory, relative, manifest):
    joined = posixpath.normpath(posixpath.join(directory, relative)) if directory else \
        posixpath.normpath(relative)
    if posixpath.isabs(joined) or joined == ".." or joined.startswith("../") or joined == ".":
        raise Full("cargo-path-escapes-repo", [manifest])
    return joined


def cargo_closure(roots, read, escapes):
    """Return (globs, crate dirs) for root manifests; `read(path)` gives bytes or None."""
    globs, dirs, seen = set(), set(), set()
    root_dirs = {posixpath.dirname(root) for root in roots}
    for ancestor in root_dirs:
        while ancestor:  # cargo also reads every ancestor manifest (a workspace root)
            ancestor = posixpath.dirname(ancestor)
            globs.add(posixpath.join(ancestor, "Cargo.toml"))
    queue = list(roots)
    while queue:
        manifest = queue.pop(0)
        if manifest in seen:
            continue
        seen.add(manifest)
        if len(seen) > MAX_MANIFESTS:
            raise Full("closure-too-large", [manifest])
        data = read(manifest)
        if data is None:
            continue
        try:
            doc = tomllib.loads(data.decode("utf-8"))
        except (UnicodeDecodeError, tomllib.TOMLDecodeError):
            raise Full("cargo-unreadable", [manifest]) from None
        if "patch" in doc or "replace" in doc:
            # shortcut: patched sources are not resolved; no tracked manifest uses them today.
            raise Full("cargo-patch-unresolved", [manifest])
        directory = posixpath.dirname(manifest)
        dirs.add(directory)
        globs.add(f"{directory}/**" if directory else "**")
        tables = [doc] + [table for table in (doc.get("target") or {}).values()
                          if isinstance(table, dict)]
        for table in tables:
            for name in DEP_TABLES:
                deps = table.get(name)
                if not isinstance(deps, dict):
                    continue
                for spec in deps.values():
                    if not isinstance(spec, dict):
                        continue
                    if spec.get("workspace") is True:
                        # shortcut: inherited paths are not resolved; no closure uses them today.
                        raise Full("cargo-workspace-inheritance", [manifest])
                    if isinstance(spec.get("path"), str):
                        dep = repo_join(directory, spec["path"], manifest)
                        globs.add(f"{dep}/**")  # declared, even where its manifest is absent
                        queue.append(f"{dep}/Cargo.toml")
        targets = []
        lib = doc.get("lib")
        if isinstance(lib, dict) and isinstance(lib.get("path"), str):
            targets.append(lib["path"])
        build = (doc.get("package") or {}).get("build")
        if isinstance(build, str):
            targets.append(build)
        if directory in root_dirs:
            for kind in ROOT_ONLY_TARGETS:
                for entry in doc.get(kind) or []:
                    if isinstance(entry, dict) and isinstance(entry.get("path"), str):
                        targets.append(entry["path"])
        for target in targets:
            path = repo_join(directory, target, manifest)
            if not path.startswith(f"{directory}/"):
                globs.add(path)
        for escape in escapes:
            if escape["crate"] == directory and (escape["when"] == "always" or directory in root_dirs):
                globs.update(escape["inputs"])
    return globs, dirs


def execution_roots(execution):
    job, variant = execution["job"], execution["variant"] or {}
    return list(job.get("cargo", [])) + list(variant.get("cargo", []))


def execution_static_globs(execution, catalogue):
    job, variant = execution["job"], execution["variant"] or {}
    result = set(job.get("inputs", [])) | set(variant.get("inputs", []))
    for name in list(job.get("input_sets", [])) + list(variant.get("input_sets", [])):
        result.update(catalogue.get("input_sets", {}).get(name, []))
    return result


def attach_globs(catalogue, execs, readers, require_roots=True):
    """Attach `globs` to every execution, union over all manifest readers.

    A cargo root absent at every reader raises Full: a typo must widen, never narrow.
    """
    escapes = catalogue.get("crate_escape", [])
    cache = {}
    for execution in execs:
        globs = execution_static_globs(execution, catalogue)
        roots = tuple(execution_roots(execution))
        if roots:
            if roots not in cache:
                combined, dirs = set(), set()
                for read in readers:
                    found, found_dirs = cargo_closure(roots, read, escapes)
                    combined |= found
                    dirs |= found_dirs
                missing = [root for root in roots if posixpath.dirname(root) not in dirs]
                if missing and require_roots:
                    raise Full("cargo-manifest-missing", missing)
                cache[roots] = (combined, dirs)
            globs |= cache[roots][0]
            execution["crate_dirs"] = sorted(cache[roots][1])
        execution["globs"] = sorted(globs)


def classify(paths, policy, execs):
    """Return (always_full hits, hits per execution id, unknown paths)."""
    by_glob = {}
    for execution in execs:
        for pattern in execution["globs"]:
            by_glob.setdefault(pattern, []).append(execution["id"])
    always, unknown, hits = [], [], {}
    for path in paths:
        central = first_match(policy["always_full"], path)
        if central:
            always.append((central, path))
            continue
        matched = False
        for pattern, ids in by_glob.items():
            if github_path_filter_matches(pattern, path):
                matched = True
                for xid in ids:
                    hits.setdefault(xid, []).append((pattern, path))
        if not matched and not first_match(policy["core_only"], path) \
                and not first_match(policy["inert"], path):
            unknown.append(path)
    return always, hits, unknown


# ---------------------------------------------------------------- plan

def changed_paths(repo, merge_base, head):
    code, raw = git(repo, "-c", "core.quotePath=false", "diff", "--no-ext-diff", "--no-textconv",
                    "--name-status", "-z", "--find-renames", merge_base, head)
    if code != 0:
        raise Full("diff-failed")
    fields = raw.split(b"\0")
    if fields and fields[-1] == b"":
        fields.pop()
    changed, index = set(), 0
    while index < len(fields):
        status = fields[index].decode("ascii", "replace")
        index += 1
        width = 2 if status[:1] in ("R", "C") else 1 if status in ("A", "M", "D", "T") else 0
        if width == 0:
            raise Full("unknown-diff-status", [status])
        if index + width > len(fields):
            raise Full("diff-parse")
        changed.update(os.fsdecode(field) for field in fields[index:index + width])
        index += width
    return sorted(changed)


def find_merge_base(repo, base, head, allow_fetch):
    for depth in (0,) + DEEPEN_STEPS:
        if depth and not (allow_fetch and fetch(repo, f"--deepen={depth}", base, head)):
            break
        code, out = git(repo, "merge-base", base, head)
        merge_base = out.decode("ascii", "replace").strip()
        if code == 0 and SHA_RE.fullmatch(merge_base) and is_ancestor(repo, merge_base, base) \
                and is_ancestor(repo, merge_base, head):
            return merge_base
    raise Full("missing-history")


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(",", ":"))


def paths_digest(paths):
    return "sha256:" + hashlib.sha256(b"\0".join(os.fsencode(p) for p in paths)).hexdigest()


class Planner:
    def __init__(self, repo, event, base, head, tested, catalogue_path, selector_bytes, allow_fetch):
        self.repo, self.event = repo, event
        self.base, self.head, self.tested = base or "", head or "", tested or head or ""
        self.catalogue_path, self.allow_fetch = catalogue_path, allow_fetch
        self.selector_blob = blob_id(selector_bytes) if selector_bytes is not None else None
        self.base_selector_blob = None
        self.catalogue, self.catalogue_blob, self.catalogue_source = None, None, None
        self.merge_base, self.changed = None, None
        self.reasons = []  # [(code, detail)], first one is the fallback
        self.hits, self.manual = {}, False
        self.policy_mode = None
        self.filter_uncertain = False

    def full(self, code, detail=()):
        self.reasons.append((code, list(detail)[:MAX_DETAIL]))

    def run(self):
        try:
            self._run()
        except Full as reason:
            self.full(reason.code, reason.detail)
        except Exception as error:  # the plan must widen, never fail
            self.full(f"selector-error:{type(error).__name__}")
        return self.finish()

    def _load_catalogue(self, sha, trusted):
        data = git_blob(self.repo, sha, self.catalogue_path)
        if data is None:
            return False
        self.catalogue_blob = blob_id(data)
        self.catalogue_source = sha
        try:
            catalogue = load_catalogue_text(data.decode("utf-8"))
        except (UnicodeDecodeError, tomllib.TOMLDecodeError):
            raise Full("invalid-trusted-catalogue" if trusted else "invalid-head-catalogue",
                       ["toml"]) from None
        problems = validate(catalogue)
        if problems:
            raise Full("invalid-trusted-catalogue" if trusted else "invalid-head-catalogue", problems)
        self.catalogue = catalogue
        return True

    def _run(self):
        if self.event not in EVENTS:
            raise Full("unknown-event", [self.event])
        for label, sha in (("head", self.head), ("tested", self.tested)):
            if not SHA_RE.fullmatch(sha):
                raise Full("invalid-sha", [label])
        if self.event in ("workflow_dispatch", "schedule") and not self.base:
            self._bootstrap_listing()  # no trusted catalogue exists without a base
            raise Full("no-trusted-base")
        if self.event == "push" and (not SHA_RE.fullmatch(self.base) or self.base == ZERO_SHA):
            self._bootstrap_listing()
            raise Full("push-unknown-before")
        if not SHA_RE.fullmatch(self.base):
            raise Full("invalid-sha", ["base"])
        for sha in dict.fromkeys((self.base, self.head, self.tested)):
            if not has_commit(self.repo, sha) and not (
                    self.allow_fetch and fetch(self.repo, "--depth=1", sha) and has_commit(self.repo, sha)):
                raise Full("fetch-failed", [sha])
        if self.event == "push" and not is_ancestor(self.repo, self.base, self.head):
            self._bootstrap_listing()
            raise Full("push-unknown-before")
        if self.event == "pull_request" and self.tested != self.head:
            code, out = git(self.repo, "rev-list", "--parents", "-n", "1", self.tested)
            if code != 0 or out.decode("ascii", "replace").split()[1:] != [self.base, self.head]:
                raise Full("merge-ref-mismatch")
        base_selector = git_blob(self.repo, self.base, SELECTOR_PATH)
        if base_selector is not None:
            self.base_selector_blob = blob_id(base_selector)
        if base_selector is None or not self._load_catalogue(self.base, trusted=True):
            self._bootstrap_listing()
            raise Full("bootstrap-no-base-catalogue")
        policy = self.catalogue["policy"]
        self.policy_mode = policy["mode"]
        if policy["mode"] == "full":
            self.full("policy-rollback")
        event_policy = policy["events"][self.event]
        if event_policy == "manual":
            self.manual = True
            return
        if event_policy == "full" or self.event == "merge_group":  # the queue is never narrowed
            self.full(EVENT_FULL_CODES.get(self.event, "event-policy-full"), [self.event])
        self.merge_base = find_merge_base(self.repo, self.base, self.head, self.allow_fetch)
        changed = changed_paths(self.repo, self.merge_base, self.head)
        self.changed = changed
        if len(changed) > MAX_CHANGED:
            raise Full("diff-too-large", [str(len(changed))])
        shas = list(dict.fromkeys((self.merge_base, self.base, self.head)))
        readers = [functools.partial(self._read_at, sha) for sha in shas]
        self.execs = executions(self.catalogue)
        attach_globs(self.catalogue, self.execs, readers)
        always, self.hits, unknown = classify(changed, policy, self.execs)
        if always:
            self.full("always-full", [f"{pattern}: {path}" for pattern, path in always])
        if unknown:
            self.full("unknown-path", unknown)

    @functools.lru_cache(maxsize=None)  # noqa: B019 - one planner per process
    def _read_at(self, sha, path):
        return git_blob(self.repo, sha, path)

    def _bootstrap_listing(self):
        """Best effort: list executions from the head catalogue for a full plan."""
        try:
            if SHA_RE.fullmatch(self.head) and has_commit(self.repo, self.head):
                self._load_catalogue(self.head, trusted=False)
        except Full:
            self.catalogue = None

    def applicable(self, execution):
        workflow = execution["workflow"]
        if self.event not in workflow["events"]:
            return False
        branches = workflow.get("trigger", {}).get(self.event, {}).get("branches")
        return not (self.event == "push" and branches is not None and "main" not in branches)

    def exclusion(self, execution):
        if execution["class"] not in SELECTABLE:
            return execution["class"]
        if execution["job"]["activation"] != "automatic":
            return "not-automatic"
        if not self.applicable(execution):
            return "event-not-applicable"
        return None

    def select(self, execs):
        eligible = {x["id"]: x for x in execs if self.exclusion(x) is None}
        selected = {xid for xid in self.hits if xid in eligible}
        selected |= {xid for xid, x in eligible.items() if x["class"] == "core"}
        while True:
            before = set(selected)
            for xid in list(selected):
                execution = eligible[xid]
                workflow, job = execution["workflow"], execution["job"]
                granularity = workflow["granularity"]
                needs = set(job.get("needs", []))
                for other in eligible.values():
                    if other["workflow"] is not workflow:
                        continue
                    if granularity == "workflow" or other["job"]["id"] in needs or (
                            granularity == "job" and other["job"] is job):
                        selected.add(other["id"])
            if selected == before:
                return selected

    def finish(self):
        catalogue = self.catalogue
        execs = getattr(self, "execs", None) or (executions(catalogue) if catalogue else [])
        if self.manual and not self.reasons:
            mode, selected = "manual", set()
        elif self.reasons:
            mode = "full"
            selected = {x["id"] for x in execs if self.exclusion(x) is None}
        else:
            mode = "select"
            selected = self.select(execs)
        changed = self.changed or []
        policy = (catalogue or {}).get("policy", {})
        selected_rows, excluded_rows = [], []
        for execution in sorted(execs, key=lambda x: x["id"]):
            row = {"id": execution["id"], "workflow": execution["workflow"]["path"],
                   "job": execution["job"]["id"],
                   "variant": (execution["variant"] or {}).get("id"),
                   "check_name": execution["check_name"], "class": execution["class"]}
            if execution["id"] in selected:
                hits = self.hits.get(execution["id"], [])
                row["reasons"] = [list(hit) for hit in hits[:MAX_REASONS]]
                row["reason_count"] = len(hits)
                row["blocked_by_filter"] = self.blocked(execution, changed, policy)
                selected_rows.append(row)
            else:
                reason = "manual-event" if mode == "manual" else self.exclusion(execution) or "no-input-match"
                excluded_rows.append({key: row[key] for key in ("id", "class")} | {"reason": reason})
        fallback = None
        if mode == "full":
            code, detail = self.reasons[0]
            reasons = []
            for reason_code, details in self.reasons:
                if reason_code == "unknown-path":
                    reasons += [f"unknown-path:{path}" for path in details]
                else:
                    reasons.append(reason_code)
            fallback = {"code": code, "detail": detail, "reasons": reasons[:MAX_DETAIL]}
        shas = {"base": self.base, "merge_base": self.merge_base, "head": self.head, "tested": self.tested}
        catalogue_info = {"path": self.catalogue_path, "source_sha": self.catalogue_source,
                          "blob": self.catalogue_blob,
                          "revision": (catalogue or {}).get("revision")}
        selector_info = {"blob": self.selector_blob, "base_blob": self.base_selector_blob,
                         "trusted": self.selector_blob is not None
                         and self.selector_blob == self.base_selector_blob}
        changes = {"count": len(changed), "digest": paths_digest(changed),
                   "paths": changed[:MAX_LISTED_PATHS]}
        identity = {"schema_version": SCHEMA_VERSION, "catalogue_blob": self.catalogue_blob,
                    "catalogue_revision": catalogue_info["revision"], "selector_blob": self.selector_blob,
                    "event": self.event, "mode": mode, "fallback": fallback and fallback["code"],
                    "shas": shas, "changes_digest": changes["digest"],
                    "selected": sorted(selected)}
        plan = {"schema": PLAN_SCHEMA, "schema_version": SCHEMA_VERSION,
                "catalogue": catalogue_info, "selector": selector_info,
                "event": self.event, "policy_mode": self.policy_mode, "mode": mode,
                "fallback": fallback, "shas": shas, "changes": changes,
                "filter_eval_uncertain": self.filter_uncertain,
                "selected": selected_rows, "excluded": excluded_rows,
                "plan_id": "sha256:" + hashlib.sha256(canonical(identity).encode("ascii")).hexdigest()}
        return plan

    def blocked(self, execution, changed, policy):
        """Would this workflow's own path filter keep it from starting? (shadow information)"""
        spec = execution["workflow"].get("trigger", {}).get(self.event, {})
        if "paths" not in spec and "paths_ignore" not in spec:
            return False
        if not changed or len(changed) > FILTER_FILE_LIMIT:
            self.filter_uncertain = True
        if "paths" in spec:
            return not any(first_match(spec["paths"], path) for path in changed)
        ignore = policy.get("inert", []) if spec["paths_ignore"] == METADATA else spec["paths_ignore"]
        return all(first_match(ignore, path) for path in changed)


def make_plan(repo, event, base, head, tested=None, catalogue_path=CATALOGUE_PATH,
              selector_bytes=None, allow_fetch=True):
    """Build a plan; never raises for runtime problems (they become mode=full)."""
    return Planner(repo, event, base, head, tested, catalogue_path, selector_bytes, allow_fetch).run()


def minimal_full_plan(event, code):
    plan = {"schema": PLAN_SCHEMA, "schema_version": SCHEMA_VERSION, "event": event, "mode": "full",
            "fallback": {"code": code, "detail": [], "reasons": [code]}, "selected": [], "excluded": []}
    plan["plan_id"] = "sha256:" + hashlib.sha256(canonical(plan).encode("ascii")).hexdigest()
    return plan


def md_cell(value):
    """Escape a value for a Markdown table cell: no links, HTML, code spans or extra cells."""
    text = str(value).replace("&", "&amp;")
    for char, entity in (("|", "&#124;"), ("<", "&lt;"), (">", "&gt;"), ("[", "&#91;"),
                         ("]", "&#93;"), ("`", "&#96;")):
        text = text.replace(char, entity)
    return " ".join(text.split())


def summary_markdown(plan):
    fallback = (plan.get("fallback") or {}).get("code")
    lines = ["## CI impact plan (shadow; does not change which jobs run)", "",
             f"- mode: `{plan['mode']}`" + (f" (fallback `{json.dumps(fallback)}`)" if fallback else ""),
             f"- plan_id: `{plan['plan_id']}`",
             f"- changed paths: {plan.get('changes', {}).get('count', 0)}"]
    counts = {}
    for row in plan.get("selected", []):
        counts[row["class"]] = counts.get(row["class"], 0) + 1
    lines.append("- selected per class: " + (", ".join(f"{k} {v}" for k, v in sorted(counts.items())) or "none"))
    blocked = [row["id"] for row in plan.get("selected", []) if row.get("blocked_by_filter")]
    lines.append(f"- selected but blocked by a workflow path filter: {len(blocked)}")
    lines += ["", "| execution | class | selected | blocked by filter |", "|---|---|---|---|"]
    for row in plan.get("selected", []):
        if row["class"] == "rwa":
            lines.append(f"| {md_cell(row['id'])} | rwa | yes | {md_cell(row['blocked_by_filter'])} |")
    for row in plan.get("excluded", []):
        if row["class"] == "rwa":
            lines.append(f"| {md_cell(row['id'])} | rwa | no ({md_cell(row['reason'])}) | |")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------- coverage

def _norm(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, list):
        return [_norm(item) for item in value]
    if isinstance(value, dict):
        return {str(key): _norm(item) for key, item in value.items()}
    return value


def yaml_matrix(job):
    """Return ('static', [combinations]) or ('dynamic', matrix) or ('unsupported', matrix)."""
    matrix = (job.get("strategy") or {}).get("matrix") if isinstance(job.get("strategy"), dict) else None
    if matrix is None:
        return "none", None
    if not isinstance(matrix, dict):
        return "dynamic", matrix
    if any(isinstance(value, str) for value in matrix.values()):
        return "dynamic", matrix
    if set(matrix) == {"include"}:
        return "static", [dict(entry) for entry in matrix["include"]]
    if "include" in matrix or "exclude" in matrix:
        return "unsupported", matrix
    keys = list(matrix)
    return "static", [dict(zip(keys, values)) for values in itertools.product(*(matrix[k] for k in keys))]


def _render(text, combination):
    if combination is None:
        return text
    return re.sub(r"\$\{\{ matrix\.(\w+) \}\}", lambda m: str(combination.get(m.group(1), m.group(0))), text)


def yaml_evidence(job, combination):
    names = []
    for step in job.get("steps") or []:
        if isinstance(step, dict) and step.get("uses") == "./.github/actions/preserve-evidence":
            name = (step.get("with") or {}).get("name")
            if isinstance(name, str):
                names.append(_render(name, combination))
    return sorted(set(names))


def _as_list(value):
    if value is None:
        return []
    return [value] if isinstance(value, str) else list(value)


AUTOMATIC_EVENTS = ("pull_request", "push", "merge_group")


def activation_problem(activation, on_events, condition):
    """Is the recorded activation consistent with the workflow's `on:` keys? (report only)"""
    automatic = [event for event in AUTOMATIC_EVENTS if event in on_events]
    if activation == "automatic":
        return None if automatic else "no pull_request, push or merge_group trigger"
    if activation.startswith("label:"):
        return None if "pull_request" in on_events else "label activation without pull_request"
    if activation.startswith("push-branch:"):
        return None if "push" in on_events else "push-branch activation without push"
    event = "schedule" if activation == "schedule" else "workflow_dispatch"
    if event not in on_events:
        return f"{activation} activation without {event}"
    # shortcut: only an explicit event_name equality counts as gating; others are reported.
    if automatic and f"github.event_name == '{event}'" not in (condition or ""):
        return f"{activation} activation, but the job also runs on {', '.join(automatic)}"
    return None


def workflow_problems(workflows, catalogue):
    """Compare parsed workflow mappings {path: BaseLoader doc} with the catalogue."""
    problems = []
    add = problems.append
    registered = {wf["path"]: wf for wf in catalogue.get("workflow", [])}
    inert = catalogue.get("policy", {}).get("inert", [])
    for path in sorted(set(workflows) - set(registered)):
        add(f"workflow-unregistered: {path}")
    for path in sorted(set(registered) - set(workflows)):
        add(f"workflow-removed: {path}")
    for path in sorted(set(workflows) & set(registered)):
        doc, entry = workflows[path], registered[path]
        on = doc.get("on", {}) if isinstance(doc, dict) else {}
        on = {key: {} for key in _as_list(on)} if not isinstance(on, dict) else on
        if sorted(on) != sorted(entry.get("events", [])):
            add(f"events-mismatch: {path}: catalogue={sorted(entry.get('events', []))} workflow={sorted(on)}")
        for event in FILTERED_EVENTS:
            spec = on.get(event) if isinstance(on.get(event), dict) else {}
            actual = {"types": spec.get("types"), "branches": spec.get("branches"),
                      "paths": spec.get("paths"), "paths_ignore": spec.get("paths-ignore")}
            recorded = dict(entry.get("trigger", {}).get(event, {}))
            if recorded.get("paths_ignore") == METADATA:
                recorded["paths_ignore"] = inert
            for key in ("types", "branches", "paths", "paths_ignore"):
                if _norm(actual[key]) != _norm(recorded.get(key)):
                    add(f"trigger-mismatch: {path}: {event}.{key}: catalogue={recorded.get(key)!r} "
                        f"workflow={actual[key]!r}")
        jobs = doc.get("jobs") or {}
        recorded_jobs = {job["id"]: job for job in entry.get("job", [])}
        for job_id in sorted(set(jobs) - set(recorded_jobs)):
            add(f"job-unregistered: {path}#{job_id}")
        for job_id in sorted(set(recorded_jobs) - set(jobs)):
            add(f"job-removed: {path}#{job_id}")
        for job_id in sorted(set(jobs) & set(recorded_jobs)):
            job, record = jobs[job_id], recorded_jobs[job_id]
            where = f"{path}#{job_id}"
            strategy = job.get("strategy") if isinstance(job.get("strategy"), dict) else {}
            pairs = (("name", job.get("name"), record.get("name")),
                     ("condition", job.get("if"), record.get("condition")),
                     ("needs", _as_list(job.get("needs")), record.get("needs", [])),
                     ("timeout_minutes", job.get("timeout-minutes"), record.get("timeout_minutes")),
                     ("max_parallel", strategy.get("max-parallel"), record.get("max_parallel")))
            for field, actual, wanted in pairs:
                if _norm(actual) != _norm(wanted):
                    add(f"field-mismatch: {where}: {field}: catalogue={wanted!r} workflow={actual!r}")
            mismatch = activation_problem(record.get("activation", ""), list(on), job.get("if"))
            if mismatch:
                add(f"activation-mismatch: {where}: {record.get('activation')}: {mismatch}")
            kind, matrix = yaml_matrix(job)
            variants = record.get("variant") or []
            record_named = {"id": job_id, "name": job.get("name")}
            if kind == "none":
                if variants:
                    add(f"variant-removed: {where}: the job has no matrix")
                actual_ev = yaml_evidence(job, None)
                if actual_ev != sorted(set(record.get("evidence", []))):
                    add(f"evidence-mismatch: {where}: catalogue={sorted(record.get('evidence', []))} "
                        f"workflow={actual_ev}")
                continue
            if kind == "unsupported":
                add(f"matrix-unsupported: {where}")
                continue
            if kind == "dynamic":
                dynamic = [v for v in variants if v.get("dynamic")]
                if not dynamic:
                    add(f"variant-unregistered: {where}: dynamic matrix needs a dynamic variant")
                for variant in dynamic:
                    if _norm(variant.get("matrix")) != _norm(matrix):
                        add(f"matrix-mismatch: {where}/*")
                continue
            actual_variants = {}
            for combination in matrix:
                check = default_check_name(record_named, combination)
                actual_variants[check] = combination
            recorded_variants = {v.get("check_name"): v for v in variants}
            for check in sorted(set(actual_variants) - set(recorded_variants)):
                add(f"variant-unregistered: {where}: {check}")
            for check in sorted(set(recorded_variants) - set(actual_variants)):
                add(f"variant-removed: {where}: {check}")
            for check in sorted(set(actual_variants) & set(recorded_variants)):
                combination, variant = actual_variants[check], recorded_variants[check]
                if _norm(variant.get("matrix")) != _norm(combination):
                    add(f"matrix-mismatch: {where}: {check}")
                actual_ev = yaml_evidence(job, combination)
                wanted_ev = sorted(set(record.get("evidence", [])) | set(variant.get("evidence", [])))
                if actual_ev != wanted_ev:
                    add(f"evidence-mismatch: {where}: {check}: catalogue={wanted_ev} workflow={actual_ev}")
    return problems


ESCAPE_RE = re.compile(
    r'#\[path\s*=\s*"(?P<attr>[^"]+)"\s*\]'
    r'|include(?:_str|_bytes)?!\s*\(\s*(?:"(?P<lit>[^"]*)"\s*\)|(?P<expr>[^)]*))')
FIRST_LITERAL_RE = re.compile(r'"([^"]*)"')


def scan_escapes(repo, crate_dirs):
    """Find literal and widened include/#[path] targets that leave their crate directory.

    shortcut: #[path] inside an inline `mod {}` block resolves differently; such
    uses are not modelled, upgrade if one appears in a closure crate.
    """
    found = []
    for crate in sorted(crate_dirs):
        queue = sorted(globmod.glob(os.path.join(repo, crate, "**", "*.rs"), recursive=True))
        queue = [posixpath.relpath(p.replace(os.sep, "/"), repo.replace(os.sep, "/")) for p in queue]
        seen = set()
        while queue:
            source = queue.pop(0)
            if source in seen:
                continue
            seen.add(source)
            try:
                with open(os.path.join(repo, source), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except OSError:
                continue
            for match in ESCAPE_RE.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                literal = match.group("attr") or match.group("lit")
                if literal is None:
                    first = FIRST_LITERAL_RE.search(match.group("expr") or "")
                    if first is None:
                        found.append((crate, None, f"{source}:{line}"))
                        continue
                    target = posixpath.dirname(posixpath.normpath(
                        posixpath.join(posixpath.dirname(source), first.group(1) + "x"))) + "/**"
                else:
                    target = posixpath.normpath(posixpath.join(posixpath.dirname(source), literal))
                if target.startswith(f"{crate}/"):
                    continue
                found.append((crate, target, f"{source}:{line}"))
                if target.endswith(".rs") and os.path.isfile(os.path.join(repo, target)):
                    queue.append(target)
    return found


def citation_problem(repo, citation, slack=5):
    """None when the anchor text is on the cited line(s); otherwise a report string."""
    cite, anchor = citation["cite"], citation["anchor"]
    file, _, lines = cite.rpartition(":")
    first, _, last = lines.partition("-")
    first, last = int(first), int(last or first)
    try:
        with open(os.path.join(repo, file), "rb") as handle:
            text = handle.read().decode("utf-8", "replace").split("\n")
    except OSError:
        return f"citation-missing: {cite}"
    if any(anchor in line for line in text[first - 1:last]):
        return None
    for number in range(max(1, first - slack), min(len(text), last + slack) + 1):
        if anchor in text[number - 1]:
            return f"citation-moved: {cite} -> {file}:{number}"
    return f"citation-anchor-missing: {cite}"


def coverage(repo, catalogue_file):
    report = {"schema": COVERAGE_SCHEMA, "schema_version": SCHEMA_VERSION}
    try:
        import yaml  # noqa: PLC0415 - optional; only coverage needs it
    except ImportError:
        return report | {"status": "coverage-unavailable", "problems": ["PyYAML is not importable"]}
    with open(os.path.join(repo, catalogue_file), "rb") as handle:
        catalogue = tomllib.load(handle)
    problems = [f"catalogue-invalid: {p}" for p in validate(catalogue)]
    if problems:
        return report | {"status": "drift", "problems": problems}
    workflows = {}
    for pattern in ("*.yml", "*.yaml"):
        for file in sorted(globmod.glob(os.path.join(repo, ".github", "workflows", pattern))):
            relative = ".github/workflows/" + os.path.basename(file)
            with open(file, encoding="utf-8") as handle:
                workflows[relative] = yaml.load(handle, Loader=yaml.BaseLoader)  # noqa: S506
    problems += workflow_problems(workflows, catalogue)
    for workflow in catalogue["workflow"]:
        for citation in workflow.get("obligation", []):
            problem = citation_problem(repo, citation)
            if problem:
                kind, _, detail = problem.partition(": ")
                problems.append(f"{kind}: {workflow['path']}: {detail}")

    def read_disk(path):
        try:
            with open(os.path.join(repo, path), "rb") as handle:
                return handle.read()
        except OSError:
            return None

    execs = executions(catalogue)
    for execution in execs:
        for root in execution_roots(execution):
            if read_disk(root) is None:
                problems.append(f"cargo-root-missing: {execution['id']}: {root}")
    try:
        attach_globs(catalogue, execs, [read_disk], require_roots=False)  # reported above
    except Full as reason:
        problems.append(f"closure-failed: {reason.code}: {reason.detail}")
        return report | {"status": "drift", "problems": problems}
    crate_dirs = set()
    for execution in execs:
        crate_dirs.update(execution.get("crate_dirs", []))
    escapes = catalogue.get("crate_escape", [])
    for crate, target, origin in scan_escapes(repo, crate_dirs):
        registered = [glob for escape in escapes if escape["crate"] == crate for glob in escape["inputs"]]
        if target is None or not first_match(registered, target):
            problems.append(f"escape-unregistered: {crate} -> {target} ({origin})")
    code, out = git(repo, "ls-files", "-z")
    if code != 0:
        problems.append("git-ls-files-failed")
        tracked = []
    else:
        tracked = [os.fsdecode(p) for p in out.split(b"\0") if p]
    always, _hits, unknown = classify(tracked, catalogue["policy"], execs)
    report |= {"status": "drift" if problems or unknown else "clean",
               "problems": sorted(set(problems)),
               "counts": {"workflows": len(workflows), "jobs": sum(len(w["job"]) for w in catalogue["workflow"]),
                          "executions": len(execs), "tracked_paths": len(tracked),
                          "always_full_paths": len(always), "unclassified_paths": len(unknown)},
               "unclassified": unknown[:100]}
    return report


# ---------------------------------------------------------------- CLI

def build_parser():
    parser = argparse.ArgumentParser(prog="ci_impact.py", description=__doc__.split("\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("validate", help="structurally validate a catalogue file")
    check.add_argument("catalogue")
    plan = sub.add_parser("plan", help="write a deterministic selection plan (never fails)")
    plan.add_argument("--event", required=True)
    plan.add_argument("--base-sha", default="", help="PR base tip, merge-group base or push 'before'")
    plan.add_argument("--head-sha", required=True)
    plan.add_argument("--tested-sha", default="", help="commit under test (PR merge ref); default head")
    plan.add_argument("--repo", default=".")
    plan.add_argument("--catalogue", default=CATALOGUE_PATH, help="repository-relative catalogue path")
    plan.add_argument("--out", default="-", help="plan JSON file, '-' for stdout")
    plan.add_argument("--summary", default="", help="append a Markdown summary to this file")
    plan.add_argument("--no-fetch", action="store_true", help="never run git fetch")
    report = sub.add_parser("coverage", help="report catalogue drift as JSON (always exits 0)")
    report.add_argument("--repo", default=".")
    report.add_argument("--catalogue", default=CATALOGUE_PATH)
    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    if args.command == "validate":
        try:
            with open(args.catalogue, "rb") as handle:
                problems = validate(tomllib.load(handle))
        except (OSError, tomllib.TOMLDecodeError) as error:
            problems = [f"unreadable catalogue: {type(error).__name__}: {error}"]
        for problem in problems:
            print(f"ci-impact: invalid: {json.dumps(problem)}")
        if not problems:
            print("ci-impact: catalogue valid")
        return 1 if problems else 0
    if args.command == "coverage":
        try:
            report = coverage(args.repo, args.catalogue)
        except Exception as error:  # a report, never a gate
            report = {"schema": COVERAGE_SCHEMA, "schema_version": SCHEMA_VERSION,
                      "status": "coverage-error", "problems": [f"{type(error).__name__}: {error}"]}
        print(json.dumps(report, indent=2, sort_keys=True, ensure_ascii=True))
        return 0
    try:
        with open(os.path.abspath(__file__), "rb") as handle:
            selector_bytes = handle.read()
        plan = make_plan(args.repo, args.event, args.base_sha, args.head_sha, args.tested_sha,
                         args.catalogue, selector_bytes, allow_fetch=not args.no_fetch)
    except Exception as error:
        plan = minimal_full_plan(args.event, f"selector-error:{type(error).__name__}")
    text = json.dumps(plan, indent=2, sort_keys=True, ensure_ascii=True) + "\n"
    try:
        if args.out == "-":
            sys.stdout.write(text)
        else:
            os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
            with open(args.out, "w", encoding="ascii") as handle:
                handle.write(text)
        if args.summary:
            with open(args.summary, "a", encoding="utf-8") as handle:
                handle.write(summary_markdown(plan))
    except OSError as error:
        sys.stdout.write(text)
        print(f"ci-impact: output-error: {type(error).__name__}", file=sys.stderr)
    print(f"ci-impact: mode={plan['mode']} plan_id={plan['plan_id']}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
