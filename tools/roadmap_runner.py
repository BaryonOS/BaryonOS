#!/usr/bin/env python3
"""Codex roadmap task dispatch with fail-closed legacy automation.

`brief` claims one READY task and prepares an immutable Codex dispatch plan for
the lead. No model or gate is executed by that command. Automatic execution and
legacy Claude helpers remain source-held until the shared isolation replacement
is accepted. The existing ledger, lock, stop/pause controls and recovery are reused;
there is no second queue and no transfer of this roadmap to Claude.
Runtime state stays in ignored build/roadmap-pipeline/.

Candidate bytes never run with controller authority: task checks, gates and the
progress check run only through the shared isolated gate runner (agent_gate), host
sessions get no check commands, Git callback configuration is fingerprinted around
every session, controller Git runs without fsmonitor, and controller Python neither
imports nor reads bytecode from the checkout (progress.py runs with -I).
"""
from __future__ import annotations

import os
import sys

# Worker sessions can write tools/*.py. Resolve the standard library and site-packages
# before the checkout (os and sys are built in or frozen, so they cannot be shadowed).
TOOLS_DIR = os.path.dirname(os.path.realpath(__file__))
sys.path[:] = [p for p in sys.path if os.path.realpath(p or ".") != TOOLS_DIR] + [TOOLS_DIR]

import argparse
import contextlib
import datetime as dt
import fcntl
import hashlib
import importlib.machinery
import json
import re
import shutil
import signal
import stat
import subprocess
import tempfile
import time
import uuid
from dataclasses import dataclass
from pathlib import Path

# Bytecode caches in the checkout are never read: a worker could plant them under
# ignored __pycache__/ paths that its scope check does not see.
sys.pycache_prefix = tempfile.mkdtemp(prefix="roadmap-pycache-")

# Repository-location overrides inherited from a shell or hook would point Git at another
# repository. Other GIT_* variables are kept here, but an automated run refuses them before its
# first session (common.controller_environment), because the isolated gate refuses them too.
GIT_LOCATION_VARS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY",
                     "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_COMMON_DIR", "GIT_NAMESPACE", "GIT_PREFIX",
                     "GIT_CEILING_DIRECTORIES", "GIT_DISCOVERY_ACROSS_FILESYSTEM")
# Controller Git runs with credentials over a tree the worker wrote: no fsmonitor and no hooks.
CONTROLLER_GIT_CONFIG = ("-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null")
DIFF_HEAD = ("diff", "--no-ext-diff", "--no-textconv", "HEAD")


def nested_git(root: Path) -> list[str]:
    """Every .git below the checkout root. Git skips them, so status, ls-files, revert and the
    callback fingerprint never see one; a planted nested .git/config can carry a callback."""
    found = []
    # An unreadable directory could hide one: report it instead of skipping it (fail closed).
    unreadable = lambda error: found.append(f"unreadable: {os.path.relpath(error.filename, root)}")
    for directory, dirs, files in os.walk(root, onerror=unreadable):  # never follows symlinks
        if ".git" in dirs or ".git" in files:
            if Path(directory) != root:
                found.append(os.path.relpath(os.path.join(directory, ".git"), root))
            dirs[:] = [d for d in dirs if d != ".git"]
    return sorted(found)


def brief(items: list[str], count: int = 20, chars: int = 2000) -> str:
    """A bounded excerpt of a path list for a Stop message, saying when it is cut."""
    text = json.dumps(items[:count]) + (f" (+{len(items) - count} more)" if len(items) > count else "")
    return text if len(text) <= chars else text[:chars] + "...(cut)"


def git_env() -> dict[str, str]:
    return {k: v for k, v in os.environ.items() if k not in GIT_LOCATION_VARS}


def planted_modules(tools: str = TOOLS_DIR) -> list[str]:
    """Untracked or ignored importable entries anywhere under tools/: modules, sourceless
    bytecode, extension modules and package directories. The sibling imports below put tools/
    on sys.path, so such a file would shadow a library, and a nested one would reach the lead's
    later test runs. __pycache__ counts too: this controller never reads it, but a later lead run
    imports bytecode whose recorded source metadata matches. Listed from the parent directory, so a
    planted tools/.git cannot answer for it. Checked at startup and again before every session."""
    parent, base = os.path.split(tools)
    listed = subprocess.run(["git", *CONTROLLER_GIT_CONFIG, "-C", parent, "ls-files", "-z", "--others",
                             "--directory", "--", base + "/"], capture_output=True, text=True, env=git_env())
    if listed.returncode:
        return [f"<git ls-files failed: {listed.stderr.strip()[-200:]}>"]
    names = [p[len(base) + 1:] for p in listed.stdout.split("\0") if p.startswith(base + "/")]
    importable = tuple(importlib.machinery.all_suffixes())
    found = sorted(n for n in names if n
                   and (n.endswith("/") or n.endswith(importable) or "." not in n.rsplit("/", 1)[-1]))
    return ([".git"] if os.path.lexists(os.path.join(tools, ".git")) else []) + found


if __name__ == "__main__" and (planted := planted_modules()):
    sys.exit(f"refusing to start: untracked modules under tools/ could run as controller code: {planted}")
from agent_gate import (FAIL_EXIT as GATE_FAIL_EXIT, PROGRESS_CHECK, GatePlan,  # noqa: E402
                        resolve as resolve_gate, run_gate)
from agent_pipeline import TaskError, is_protected, require_pipeline_release  # noqa: E402
from common import agent_source_paths, controller_environment  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs" / "roadmap-pipeline" / "ledger.json"
RECEIPTS = ROOT / "docs" / "roadmap-pipeline" / "receipts"
STATE = ROOT / "build" / "roadmap-pipeline"
DEFAULT_BRANCH = "claude/baryonos-autonomous-pipeline-81tujs"
STATUSES = ("ready", "active", "verified", "blocked", "superseded")
# Who may execute a task: fresh worker sessions, or only the lead / project owner
# (acceptance proposals, owner decisions, workflow edits, human or physical work).
EXECUTORS = ("worker", "lead", "owner", "external")
# Workers additionally never edit the pipeline's own state or the shared progress views.
RUNNER_PROTECTED = ("tools/roadmap_runner.py", "tools/tests/test_roadmap_runner.py",
                    "docs/roadmap-pipeline/", "docs/PROJECT-STATE.md", "docs/PROGRESS.md",
                    "README.md", "docs/assets/progress.svg", "docs/DECISIONS.md")
ALWAYS_ALLOWED = ("Cargo.lock",)
FORBIDDEN_FLAGS = ("--continue", "-c", "--resume", "-r", "--fork-session",
                   "--dangerously-skip-permissions", "--allow-dangerously-skip-permissions",
                   "--bare")
RATE_LIMIT = re.compile(r"rate.?limit|usage limit|quota|too many requests|\b429\b|overloaded", re.I)
# Only these variables reach a worker: no GitHub, cloud or session-ingress credentials.
ENV_ALLOW = ("PATH", "HOME", "SHELL", "LANG", "LC_ALL", "TZ", "USER", "TMPDIR",
             "HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "NO_PROXY", "no_proxy",
             "SSL_CERT_FILE", "NODE_EXTRA_CA_CERTS", "NODE_OPTIONS", "REQUESTS_CA_BUNDLE",
             "CURL_CA_BUNDLE", "CARGO_HTTP_CAINFO", "GIT_SSL_CAINFO", "PIP_CERT",
             "RUSTUP_HOME", "CARGO_HOME", "OS_OVMF_CODE", "OS_OVMF_VARS", "MAX_THINKING_TOKENS",
             "CLAUDE_CODE_AUTO_COMPACT_WINDOW")
GRAPHIFY_READ = ("graphify_find", "graphify_callers", "graphify_impact", "graphify_tests_for",
                 "graphify_file_neighbors", "graphify_node", "graphify_rank_files",
                 "graphify_trace", "graphify_imports_exports", "query_graph", "recall",
                 "list_repositories", "graph_stats", "search", "fetch")
GRAPHIFY_SERVERS = ("Graphify", "claude_ai_Graphify")
# No git diff/log/show: their --output=<path> writes anywhere, including .git/config.worktree.
READ_GIT = ("git status", "git rev-parse", "git branch --show-current")
DENY = ["Bash(git push:*)", "Bash(git commit:*)", "Bash(git reset:*)", "Bash(git checkout:*)",
        "Bash(git clean:*)", "Bash(git stash:*)", "Bash(git rebase:*)", "Bash(git merge:*)",
        "Bash(gh:*)", "Bash(curl:*)", "Bash(wget:*)", "WebFetch", "WebSearch",
        "Read(~/.claude/.credentials.json)", "Read(~/.ssh/**)", "Read(~/.config/gh/**)",
        "Read(~/.git-credentials)", "Read(//home/claude/**)", "Read(//root/.ccr/**)",
        # Paths the controller's scope check cannot see: Git state, caches, home.
        "Edit(.git/**)", "Edit(**/.git/**)", "Edit(**/.git)", "Edit(**/__pycache__/**)", "Edit(**/*.pyc)", "Edit(.claude/**)",
        # Controller state: the callback baseline, STOP and session logs.
        "Edit(build/roadmap-pipeline/**)"]
PROCESS_TAG = "roadmap-pipeline-"
READ_ONLY_DISALLOWED = "Edit,Write,MultiEdit,NotebookEdit"
CALLBACKS_MARKER = "callbacks-baseline.json"  # present while a session may change Git state
REVIEW_NONCE = "review-nonce"  # issued by `resume`: no digest exists before the lead saw the review
TRUSTED_RECORD = "trusted-checkout.json"  # this checkout's baselines have lived outside it since then
INIT_DENY_PROBE = "build/roadmap-pipeline/DENY-PROBE"
TRAILERS = ("Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>\n"
            "Claude-Session: https://claude.ai/code/session_01TJtT9VANihpE4RzPd4d8su\n")

WORKER_SCHEMA = {
    "type": "object",
    "properties": {
        "task_id": {"type": "string"},
        "state": {"type": "string", "enum": ["implemented", "partial", "blocked", "no_change"]},
        "summary": {"type": "string"},
        "files_changed": {"type": "array", "items": {"type": "string"}},
        "tests_run": {"type": "array", "items": {"type": "object", "properties": {
            "command": {"type": "string"}, "exit_code": {"type": "integer"}},
            "required": ["command", "exit_code"]}},
        "graphify_used": {"type": "boolean"},
        "skills_used": {"type": "array", "items": {"type": "string"}},
        "open_findings": {"type": "array", "items": {"type": "string"}},
        "blockers": {"type": "array", "items": {"type": "string"}},
        "next_action": {"type": "string"},
    },
    "required": ["task_id", "state", "summary", "files_changed", "tests_run",
                 "open_findings", "blockers", "next_action"],
}
VERIFIER_SCHEMA = {
    "type": "object",
    "properties": {
        "verdict": {"type": "string", "enum": ["pass", "fail"]},
        "findings": {"type": "array", "items": {"type": "object", "properties": {
            "severity": {"type": "string", "enum": ["blocker", "major", "minor"]},
            "file": {"type": "string"}, "issue": {"type": "string"}},
            "required": ["severity", "issue"]}},
        "checked": {"type": "array", "items": {"type": "string"}},
    },
    "required": ["verdict", "findings", "checked"],
}
INIT_SCHEMA = {
    "type": "object",
    "properties": {
        "prior_conversation_visible": {"type": "boolean"},
        "cwd": {"type": "string"}, "branch": {"type": "string"}, "head": {"type": "string"},
        "agents_md_loaded": {"type": "boolean"}, "agents_md_first_heading": {"type": "string"},
        "graphify_tools": {"type": "array", "items": {"type": "string"}},
        "graphify_repository_ready": {"type": "boolean"},
        "skills_seen": {"type": "array", "items": {"type": "string"}},
        "bash_denied_example": {"type": "string"},
        "notes": {"type": "string"},
    },
    "required": ["prior_conversation_visible", "cwd", "branch", "head", "agents_md_loaded",
                 "graphify_tools", "skills_seen", "bash_denied_example"],
}


class Blocked(RuntimeError):
    """The current task cannot continue without new information or a human."""


class Stop(RuntimeError):
    """The controller must stop (quota, unsafe state, stop file, budget)."""


@dataclass
class Config:
    branch: str = DEFAULT_BRANCH
    model: str = "claude-opus-5-5"
    effort: str = "xhigh"          # security and high-risk tasks, and their verifiers
    routine_effort: str = "high"   # low/medium-risk tasks; quota is finite, depth on security is not cut
    # Measured 15 Nov 2026: a worker starts at ~28k tokens; with a 100k window the first real
    # task (A02-T1) compacted 18 times in 34 minutes without a single edit. 300k stays a
    # session-scoped fail-safe; the normal end of a session is one completed task.
    autocompact: str = "300000"
    max_compactions: int = 6
    worker_timeout: int = 90 * 60
    verifier_timeout: int = 30 * 60
    check_timeout: int = 90 * 60
    session_budget_usd: float = 80.0
    verifier_budget_usd: float = 25.0
    total_budget_usd: float = 400.0
    max_repairs: int = 2
    push: bool = True
    claude: str = "claude"


# ---------- pure helpers (unit-tested) ----------

def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


def sha256(data: bytes | str) -> str:
    return hashlib.sha256(data.encode() if isinstance(data, str) else data).hexdigest()


def protected(path: str) -> bool:
    return is_protected(path) or any(path == p or path.startswith(p) for p in RUNNER_PROTECTED)


def validate_ledger(ledger: dict) -> None:
    ids = [t["id"] for t in ledger["tasks"]]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate task ids in ledger")
    for task in ledger["tasks"]:
        for key in ("id", "criterion", "title", "goal", "status", "deps", "write_paths", "checks"):
            if key not in task:
                raise ValueError(f"{task.get('id')}: missing {key}")
        if task["status"] not in STATUSES:
            raise ValueError(f"{task['id']}: unknown status {task['status']}")
        if task.get("executor", "worker") not in EXECUTORS:
            raise ValueError(f"{task['id']}: unknown executor {task.get('executor')}")
        if bad := [d for d in task["deps"] if d not in ids]:
            raise ValueError(f"{task['id']}: unknown dependencies {bad}")
        if bad := [p for p in task["write_paths"] if protected(p.rstrip("*")) or p.startswith("/") or ".." in p]:
            raise ValueError(f"{task['id']}: protected or unsafe write paths {bad}")
        if task["status"] == "ready" and task.get("executor", "worker") == "worker" \
                and not (task["checks"] or task.get("gates")):
            raise ValueError(f"{task['id']}: a ready task needs at least one check or gate")


def select_task(ledger: dict) -> dict | None:
    """First READY worker task (ledger order is the documented rank) whose deps are verified."""
    done = {t["id"] for t in ledger["tasks"] if t["status"] == "verified"}
    for task in ledger["tasks"]:
        if task["status"] == "ready" and task.get("executor", "worker") == "worker" \
                and all(d in done for d in task["deps"]):
            return task
    return None


def in_write_set(path: str, write_paths: list[str]) -> bool:
    """Exact file, `dir/` prefix or `dir/**` glob; never a protected path."""
    if protected(path):
        return False
    if path in ALWAYS_ALLOWED or path.endswith("/Cargo.lock"):
        return True
    for allowed in write_paths:
        base = allowed[:-2] if allowed.endswith("**") else allowed
        if path == allowed or (base.endswith("/") and path.startswith(base)):
            return True
    return False


def parse_porcelain(text: str) -> list[str]:
    paths = []
    for line in text.splitlines():
        if len(line) > 3:
            paths.append(line[3:].split(" -> ")[-1].strip('"'))
    return paths


def worker_env(source: dict[str, str]) -> dict[str, str]:
    env = {k: source[k] for k in ENV_ALLOW if k in source}
    env.update({"DISABLE_AUTOUPDATER": "1", "TERM": "dumb", "NO_COLOR": "1"})
    return env


def allow_rules(extra_tools: tuple[str, ...] = ()) -> list[str]:
    """Host sessions read and edit only; candidate code runs solely in the isolated gate."""
    rules = ["Read", "Glob", "Grep", "Skill", *extra_tools]
    rules += [f"Bash({c}:*)" for c in READ_GIT]
    rules += [f"mcp__{s}__{t}" for s in GRAPHIFY_SERVERS for t in GRAPHIFY_READ]
    return rules


def effort_for(cfg: Config, task: dict | None) -> str:
    """Full effort for security/high-risk work; the configured routine effort otherwise."""
    if task is None or task.get("risk") in ("security", "high"):
        return cfg.effort
    return cfg.routine_effort


def claude_argv(cfg: Config, name: str, session_id: str, settings: Path, schema: dict,
                mode: str, budget: float, disallowed: str | None = None, effort: str | None = None) -> list[str]:
    """argv for one NEW session; the brief is passed on stdin (no argv size limit)."""
    argv = [cfg.claude, "-p", "--name", name, "--session-id", session_id, "--output-format", "json",
            "--model", cfg.model, "--effort", effort or cfg.effort, "--autocompact", cfg.autocompact,
            "--permission-mode", mode, "--permission-prompts", "none",
            "--settings", str(settings), "--json-schema", json.dumps(schema),
            "--max-budget-usd", f"{budget:.2f}"]
    if disallowed:
        argv.append(f"--disallowedTools={disallowed}")  # `=` form: the flag is variadic
    assert not any(flag in argv for flag in FORBIDDEN_FLAGS), "forbidden session flag"
    return argv


def codex_argv(result: Path) -> list[str]:
    """Installed Codex 0.167.2 interface. Auth stays; the user config, plugins and apps do not, so
    the session gets no MCP server that could write outside the shell sandbox (the baseline home)."""
    return ["codex", "exec", "--ephemeral", "--strict-config", "--ignore-user-config", "--disable", "plugins",
            "--disable", "apps", "--cd", str(ROOT), "--sandbox", "workspace-write",
            "--json", "--output-last-message", str(result), "-"]


def failure_fingerprint(kind: str, text: str) -> str:
    # The whole bounded text: a tail window misses which test failed when stderr ends the same.
    lines = [re.sub(r"(duration_ms:?\s+)\d+(\.\d+)?", r"\1<t>", re.sub(r"\d+(\.\d+)?(ms|s)\b", "<t>", l))
             for l in text.splitlines()]
    return sha256(kind + "\n" + "\n".join(lines))[:16]


def structured(result: dict) -> dict | None:
    if isinstance(result.get("structured_output"), dict):
        return result["structured_output"]
    text = result.get("result")
    if isinstance(text, str):
        match = re.search(r"\{.*\}", text, re.S)
        if match:
            with contextlib.suppress(json.JSONDecodeError):
                return json.loads(match.group(0))
    return None


def blocking(findings: list[dict]) -> list[dict]:
    return [f for f in findings if f.get("severity") in ("blocker", "major")]


# ---------- state ----------

def open_private(path: Path, mode: str = "a+"):
    """Open a controller file without following a planted entry. Controller state lies in the
    session-writable checkout: a symlink there could make the controller create or truncate a
    file in the trusted state home (the trust record, the marker). Hard links and FIFOs are
    refused too; truncation happens only after the checks."""
    flags = {"a+": os.O_RDWR | os.O_APPEND, "a": os.O_WRONLY | os.O_APPEND, "w": os.O_WRONLY}[mode]
    try:
        fd = os.open(path, flags | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, 0o600)
    except OSError as error:
        raise Stop(f"refusing to open {path}: {error.strerror} (a planted symlink?)") from error
    info = os.fstat(fd)
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        os.close(fd)
        raise Stop(f"refusing to open {path}: not a private regular file")
    if mode == "w":
        os.ftruncate(fd, 0)
    return os.fdopen(fd, mode)


def atomic_write(path: Path, data: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + f".tmp{os.getpid()}")
    with open_private(tmp, "w") as out:
        out.write(data)
    os.replace(tmp, path)


def load_ledger(path: Path = LEDGER) -> dict:
    ledger = json.loads(path.read_text(encoding="utf-8"))
    validate_ledger(ledger)
    return ledger


def save_ledger(ledger: dict, path: Path = LEDGER) -> None:
    validate_ledger(ledger)
    atomic_write(path, json.dumps(ledger, indent=2, ensure_ascii=False) + "\n")


def update_task(task_id: str, path: Path = LEDGER, history: dict | None = None, **fields) -> dict:
    """Re-read the ledger and change only this task, so concurrent lead edits made
    while the controller was paused are never overwritten by a stale copy."""
    ledger = load_ledger(path)
    task = next(t for t in ledger["tasks"] if t["id"] == task_id)
    task.update(fields)
    if history:
        task.setdefault("history", []).append(history)
    save_ledger(ledger, path)
    return task


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except (ProcessLookupError, ValueError):
        return False
    except PermissionError:
        return True
    return True


def git_callbacks() -> str:
    """Fingerprint of every Git setting and hook that could run code on a later Git call."""
    config = subprocess.run(["git", "config", "--list", "--show-scope", "--show-origin"], cwd=ROOT,
                            capture_output=True, text=True, env=git_env()).stdout
    parts = [config]
    for name in ("hooks", "info/attributes"):
        # Not git(): its hooksPath override would resolve `hooks` to /dev/null. rev-parse runs no hook.
        located = subprocess.run(["git", "-c", "core.fsmonitor=false", "rev-parse", "--git-path", name], cwd=ROOT,
                                 capture_output=True, text=True, env=git_env())
        if located.returncode:
            raise Stop(f"git rev-parse --git-path {name} failed: {located.stderr.strip()[-400:]}")
        path = Path(located.stdout.strip())
        path = path if path.is_absolute() else ROOT / path
        files = sorted(path.rglob("*")) if path.is_dir() else [path]
        for file in files:
            if file.is_file() or file.is_symlink():
                info = file.lstat()
                body = (os.readlink(file).encode() + b"\0" if file.is_symlink() else b"") + \
                    (file.read_bytes() if file.is_file() else b"")
                parts.append(f"{file}:{info.st_mode:o}:{sha256(body)}")
    return sha256("\n".join(parts))


GATE_VOLATILE = ("## resource:", "## scratch:", "## ephemeral scratch:", "## source", "## staged source")


def gate_report(log: Path) -> tuple[str, str]:
    """Digest of the gate log and its raw stream files, and repair feedback that inlines the
    failing output without per-run resource evidence (which would defeat identical-failure detection)."""
    # The log directory is created by the controller after the session, so only the gate writes here.
    files = [f for f in [log, *sorted(log.parent.glob(log.name + ".*"))] if f.is_file() and not f.is_symlink()]
    digest = sha256(b"".join(f.name.encode() + b"\0" + f.read_bytes() + b"\0" for f in files))
    lines: list[str] = []
    for line in log.read_text(errors="replace").splitlines() if files else []:
        if line.startswith("## bounded raw streams:"):
            for name in line.split(":", 1)[1].replace(",", " ").split():
                stream = log.with_name(name)
                if Path(name).name == name and stream.is_file():
                    lines += [f"## {name}", *stream.read_text(errors="replace").splitlines()[-60:]]
        elif not line.startswith(GATE_VOLATILE):
            lines.append(line)
    for stream in (log.with_name(f"{log.name}.resource.{s}.log") for s in ("stdout", "stderr")):
        if stream in files:  # raw output saved when the sandbox boundary itself failed
            lines += [f"## {stream.name}", *stream.read_text(errors="replace").splitlines()[-60:]]
    return digest, "\n".join(lines[-120:])


def hidden_tracked() -> list[str]:
    """Tracked paths whose index flags hide worktree edits from status, diff and revert:
    assume-unchanged (lower-case tag) and skip-worktree (S)."""
    return sorted(entry[2:] for entry in git("ls-files", "-v", "-z").split("\0")
                  if entry and (entry[0].islower() or entry[0] == "S"))


def tracked_changes() -> list[str]:
    """Tracked paths whose content differs from HEAD, read through a throwaway index. The real
    .git/index carries a stat cache a session could forge (a same-size edit then looks clean) and
    assume-unchanged/skip-worktree flags; a fresh index has neither, so Git compares content.
    Called only while no session runs, so the private temporary directory is not raced."""
    with tempfile.TemporaryDirectory(prefix="roadmap-index-") as tmp:
        env = dict(git_env(), GIT_INDEX_FILE=os.path.join(tmp, "index"))

        def run(*args: str, check: bool = True) -> str:
            done = subprocess.run(["git", *CONTROLLER_GIT_CONFIG, *args], cwd=ROOT, capture_output=True,
                                  text=True, env=env)
            if check and done.returncode:
                raise Stop(f"git {args[0]} on a fresh index failed ({done.returncode}): {done.stderr.strip()[-400:]}")
            return done.stdout
        run("read-tree", "HEAD")
        run("update-index", "-q", "--refresh", check=False)  # exits 1 when something differs
        return sorted(filter(None, run("diff-files", "--name-only", "-z").split("\0")))


def git(*args: str, check: bool = True) -> str:
    done = subprocess.run(["git", *CONTROLLER_GIT_CONFIG, *args], cwd=ROOT, capture_output=True, text=True,
                          env=git_env())
    if check and done.returncode:
        raise Stop(f"git {' '.join(args[:3])} failed ({done.returncode}): {done.stderr.strip()[-800:]}")
    return done.stdout


def baseline_home() -> Path:
    """Where the saved session baseline lives: outside every root a session may write. A manual
    Codex `workspace-write` session writes the checkout, /tmp and $TMPDIR, so it could delete or
    forge a baseline kept in build/."""
    xdg = os.environ.get("XDG_STATE_HOME", "")
    # The XDG spec ignores a relative value; resolving it against the cwd would move the baseline.
    base = Path(xdg if os.path.isabs(xdg) else Path.home() / ".local" / "state").resolve()
    # Resolved whole: a symlinked component must not lead the checks past the real location.
    home = (base / "baryonos-roadmap-pipeline" / sha256(str(ROOT.resolve()))[:16]).resolve()
    for writable in dict.fromkeys([ROOT.resolve(), Path(tempfile.gettempdir()).resolve(), Path("/tmp").resolve()]):
        if home == writable or writable in home.parents:
            raise Stop(f"the session baseline directory {home} lies in the session-writable {writable}; "
                       "set XDG_STATE_HOME elsewhere")
    return home


def expected_state(state: Path) -> Path | None:
    """The resolved path an in-checkout state directory must have; None for one outside the
    checkout (a test layout), which no session can relink."""
    try:
        return ROOT.resolve() / state.relative_to(ROOT)
    except ValueError:
        return None


def pin_state(state: Path) -> None:
    """The state directory lies in the checkout: a session can make it (or a component such as
    build/) a link into any checkout's trusted home, and every controller write, mkdir and unlink
    there would follow. Its resolved path must be exactly the in-checkout one, before and after mkdir."""
    if (expected := expected_state(state)) is None:
        state.mkdir(parents=True, exist_ok=True)
        return
    for create in (False, True):
        if create:
            state.mkdir(parents=True, exist_ok=True)
        if (resolved := state.resolve()) != expected:
            raise Stop(f"the state directory {state} resolves to {resolved}, not {expected}; "
                       "remove the planted link")


class Controller:
    def __init__(self, cfg: Config, state: Path | None = None, baseline: Path | None = None):
        self.cfg = cfg
        state = STATE if state is None else state  # read at call time, not bound at class creation
        self.state = state
        # A test state directory already lies outside the checkout; the real one does not.
        self.baseline = baseline if baseline is not None else (None if state == STATE else state)
        self.lock_file = None
        self.lock_acquired = False
        self.lease_owned = False
        self.spent = 0.0
        self.callbacks: str | None = None  # set by preflight(); unset means every session stops
        self.run_id = dt.datetime.now().strftime("%Y%m%dT%H%M%S") + f"-{os.getpid()}"

    # --- locking and lifecycle ---
    def own_state(self) -> None:
        if self.baseline is not None:  # a test layout: the state directory is its own baseline home
            self.state.mkdir(parents=True, exist_ok=True)
            return
        pin_state(self.state)

    def unlink_in_state(self, *names: str) -> None:
        """Unlinks through one descriptor of the verified state directory: a link swapped in after
        own_state() cannot carry the unlink into another checkout's home."""
        if self.baseline is not None or (pinned := expected_state(self.state)) is None:
            for name in names:
                (self.state / name).unlink(missing_ok=True)
            return
        expected = str(pinned)
        try:
            fd = os.open(self.state, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)  # verified below
        except OSError as error:
            raise Stop(f"refusing the state directory {self.state}: {error.strerror} (a planted link?)") from error
        try:
            try:
                opened = os.readlink(f"/proc/self/fd/{fd}")
            except OSError as error:  # no /proc: the opened directory cannot be verified
                raise Stop(f"cannot verify the state directory {self.state}: {error.strerror}") from error
            if opened != expected:
                raise Stop(f"the state directory {self.state} opened as {opened}, not {expected}; "
                           "remove the planted link")
            for name in names:
                with contextlib.suppress(FileNotFoundError):
                    os.unlink(name, dir_fd=fd)
        finally:
            os.close(fd)

    def acquire(self) -> None:
        self.own_state()
        if (self.state / "STOP").exists():  # before recover() can run git status
            raise Stop("stop file present (build/roadmap-pipeline/STOP)")
        self.lock_file = open_private(self.state / "controller.lock")
        try:
            fcntl.flock(self.lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise Stop("another controller holds build/roadmap-pipeline/controller.lock") from error
        self.lock_acquired = True
        self.lock_file.seek(0)
        self.lock_file.truncate()
        self.lock_file.write(json.dumps({"pid": os.getpid(), "run": self.run_id, "started": now()}))
        self.lock_file.flush()
        self.refuse_live_workers()  # a live worker could still change the callbacks
        self.check_saved_baseline()  # recover() runs git status, which can run a filter
        self.recover()

    def release(self) -> None:
        if self.lease_owned:
            with contextlib.suppress(FileNotFoundError):
                (self.state / "lease.json").unlink()
        if self.lock_file:
            if self.lock_acquired:
                fcntl.flock(self.lock_file, fcntl.LOCK_UN)
            self.lock_file.close()
            self.lock_file = None
        self.lock_acquired = False
        self.lease_owned = False

    def recover(self) -> None:
        """A lease without a live owner means an interrupted task: never auto-clean it."""
        lease_path = self.state / "lease.json"
        if not lease_path.exists():
            return
        lease = json.loads(lease_path.read_text())
        for pid in (lease.get("worker_pid"), lease.get("controller_pid")):
            if pid and pid != os.getpid() and pid_alive(pid):
                raise Stop(f"lease owner pid {pid} still alive; refusing to start a second writer")
        dirty = git("status", "--porcelain", "--untracked-files=all")
        task_id = lease.get("task")
        # 'active' lives only in the lease; a dead owner blocks the leased task.
        if task_id and any(t["id"] == task_id and t["status"] in ("ready", "active")
                           for t in load_ledger()["tasks"]):
            update_task(task_id, status="blocked", history={
                "at": now(), "event": "interrupted", "lease": lease, "dirty_paths": parse_porcelain(dirty)})
        lease_path.unlink()
        if dirty.strip():
            raise Stop("recovered an interrupted lease and the checkout has a partial diff; "
                       "review it before restarting (see ledger history)")

    def heartbeat(self, **fields) -> None:
        lease_path = self.state / "lease.json"
        lease = json.loads(lease_path.read_text()) if lease_path.exists() else {}
        lease.update(fields, controller_pid=os.getpid(), run=self.run_id, heartbeat=now())
        atomic_write(lease_path, json.dumps(lease, indent=2))
        self.lease_owned = True

    def check_controls(self) -> None:
        if (self.state / "STOP").exists():
            raise Stop("stop file present (build/roadmap-pipeline/STOP)")
        while (self.state / "PAUSE").exists():
            self.heartbeat(paused=True)
            time.sleep(30)
            if (self.state / "STOP").exists():
                raise Stop("stop file present while paused")

    def refuse_live_workers(self) -> None:
        # `--`: without it pgrep parses the pattern as an option, exits 2 and finds nothing.
        try:
            probe = subprocess.run(["pgrep", "-f", "--", f"--name {PROCESS_TAG}"],
                                   capture_output=True, text=True)
        except OSError as error:
            raise Stop(f"worker probe failed: {error}") from error
        if probe.returncode not in (0, 1):  # 1: no match; anything else is no evidence of absence
            raise Stop(f"worker probe failed (pgrep exit {probe.returncode}): {probe.stderr.strip()[-300:]}")
        alive = [p for p in probe.stdout.split() if int(p) != os.getpid()]
        if alive:
            raise Stop(f"a previous worker process is still alive: {alive}")

    def preflight(self) -> str:
        self.refuse_live_workers()
        # A manual Codex session is not a tagged process: its claim keeps its baseline alive.
        if active := [t["id"] for t in load_ledger(LEDGER)["tasks"] if t["status"] == "active"]:
            raise Stop(f"manual task {active} is active; validate it and update the ledger first")
        # The ledger lies in the checkout, where the session can mark its task done: the claim that
        # counts is the one in the trusted marker, which only `resume --accept` ends.
        if claim := self.manual_claim():
            raise Stop(f"manual task {claim} holds the session baseline; validate its result, then run `resume`")
        self.check_saved_baseline()
        if not self.trusted():
            raise Stop("this checkout has no trusted session baseline yet (an older controller may have kept "
                       "it inside the checkout); review the checkout, then run `resume`")
        self.refuse_invisible_changes()
        self.callbacks = git_callbacks()
        branch = git("branch", "--show-current").strip()
        if branch != self.cfg.branch:
            raise Stop(f"checkout is on {branch!r}, expected {self.cfg.branch!r}")
        dirty = git("status", "--porcelain", "--untracked-files=all").strip()
        if dirty:
            raise Stop(f"checkout is not clean; refusing to start a writer:\n{dirty[:2000]}")
        if forged := tracked_changes():  # clean only according to the index's stat cache
            raise Stop(f"tracked files differ from HEAD although the index calls them clean: {brief(forged)}")
        head = git("rev-parse", "HEAD").strip()
        # Last: a Stop above keeps the baseline, so what the lead removes next is still compared.
        self.retire_baseline()
        return head

    @staticmethod
    def refuse_invisible_changes() -> None:
        """What the baseline comparison cannot see: Git lists nothing inside a nested repository,
        index flags hide tracked edits, and a planted module runs as controller code. Each one
        keeps the baseline (preflight and `resume` alike), so it is compared again once removed."""
        if planted := planted_modules():
            raise Stop(f"untracked or ignored modules under tools/ could run as code: {brief(planted)}")
        if nested := nested_git(ROOT):
            raise Stop(f"nested .git paths can carry Git callbacks; remove them: {brief(nested)}")
        if hidden := hidden_tracked():
            raise Stop(f"tracked paths hide worktree edits (assume-unchanged or skip-worktree): {brief(hidden)}")

    def check_saved_baseline(self) -> None:
        """Before any Git command that can run a hook or filter. A session that never reached its
        own check (interrupt, crash, kill, Stop) left its baseline behind: compare with it, never
        adopt a change. Retire it only once no worker can still change the callbacks."""
        marker, legacy = self.marker(), self.legacy_marker()
        if marker.exists():
            saved = self.saved_baseline(marker)
            self.callbacks, ignored = saved.get("baseline"), saved.get("ignored")
            self.check_callbacks("an interrupted session")
            delta = self.ignored_delta(ignored) if isinstance(ignored, dict) else (["unreadable baseline"], [])
            if any(delta):
                # Sticky like a callback change: `resume` shows the delta and retires only on --accept.
                atomic_write(self.state / "STOP", "ignored files changed since a session's baseline\n")
                raise Stop("ignored files changed since an interrupted or manual session's baseline "
                           f"(created, changed or removed); review them, then run `resume`: "
                           f"{[brief(paths) for paths in delta]}")
        if legacy:
            # Written before the baseline left the checkout: a session could have rewritten it.
            atomic_write(self.state / "STOP", "a session baseline inside the checkout cannot be trusted\n")
            raise Stop(f"{legacy} lies inside the checkout, where a session could forge it; review the "
                       "ignored files and Git callbacks, then run `resume`")

    @staticmethod
    def saved_baseline(path: Path) -> dict:
        try:
            saved = json.loads(path.read_text())
        except (OSError, ValueError):
            saved = None
        return saved if isinstance(saved, dict) else {}  # unreadable baseline: every check fails closed

    def legacy_marker(self) -> Path | None:
        legacy = self.state / CALLBACKS_MARKER
        if legacy == self.marker() or not os.path.lexists(legacy):
            return None
        # Resolved, not as text: through a link the "legacy" file can be this or a sibling's marker.
        if (real := legacy.resolve()) == self.marker().resolve():
            return None
        if ROOT.resolve() not in real.parents:
            raise Stop(f"the legacy marker {legacy} resolves to {real}, outside the checkout; remove the link")
        return legacy

    def marker(self) -> Path:
        return (self.baseline or baseline_home()) / CALLBACKS_MARKER

    def manual_claim(self) -> str | None:
        marker = self.marker()
        saved = self.saved_baseline(marker) if marker.exists() else {}
        session = saved.get("session")
        # A claim written by 3a24e2c4 carries no "manual" key, only its session name.
        return saved.get("manual") or (session[len("manual "):] if isinstance(session, str)
                                       and session.startswith("manual ") else None)

    def trusted(self) -> bool:
        """Whether this checkout's baseline history lies outside every session's reach. Without the
        record, a session could have deleted an older in-checkout baseline and left no trace."""
        return (self.marker().parent / TRUSTED_RECORD).exists() or self.marker().exists()

    def retire_baseline(self) -> None:
        """Only after the caller has validated the session's checkout, ignored files included."""
        record = self.marker().parent / TRUSTED_RECORD
        if not record.exists():
            atomic_write(record, json.dumps({"since": now(), "root": str(ROOT)}))
        self.marker().unlink(missing_ok=True)
        (self.marker().parent / REVIEW_NONCE).unlink(missing_ok=True)

    def read_checkout(self) -> tuple[str, dict[str, list[int]]]:
        """One read of what the saved baseline covers. `resume` reviews, binds and saves this same
        read, so a change after it stays a change for the next review or preflight."""
        return git_callbacks(), self.ignored_files()

    def baseline_review(self, read: tuple[str, dict[str, list[int]]]) -> dict | None:
        """What retiring the saved baseline would adopt: the change since it was taken. With no
        trusted baseline there is no delta to show: the lead reviews the checkout as a whole."""
        callbacks, ignored_now = read
        marker, legacy = self.marker(), self.legacy_marker()
        if not marker.exists():
            if legacy is None and self.trusted():
                return None
            return {"no_trusted_baseline": True, "legacy_marker_in_checkout": legacy is not None}
        saved = self.saved_baseline(marker)
        ignored = saved.get("ignored")
        created, changed = (self.ignored_delta(ignored, ignored_now) if isinstance(ignored, dict)
                            else (["unreadable baseline"], []))
        return {"manual_claim": self.manual_claim(), "legacy_marker_in_checkout": legacy is not None,
                "callbacks_changed": callbacks != saved.get("baseline"),
                "created": created, "changed_or_removed": changed}

    def review_digest(self, review: dict, read: tuple[str, dict[str, list[int]]]) -> str:
        """Binds the acknowledgement to the exact state the lead reviewed: HEAD, the whole tree
        fingerprint (status, tracked and untracked content), the Git callbacks, every ignored
        file's identity (ctime and inode change on any later write), the changed files' content
        and both markers. A later, different change gives a different digest."""
        def content(path: Path) -> str | None:
            try:
                return os.readlink(path) if path.is_symlink() else sha256(path.read_bytes()) if path.is_file() else None
            except OSError as error:
                return f"unreadable: {error.errno}"
        changed = review.get("created", []) + review.get("changed_or_removed", [])
        markers = [self.marker(), self.state / CALLBACKS_MARKER]
        nonce = self.marker().parent / REVIEW_NONCE
        if not nonce.exists():
            atomic_write(nonce, uuid.uuid4().hex)
        bound = {"nonce": nonce.read_text(), "review": review, "head": git("rev-parse", "HEAD").strip(),
                 "tree": self.tree_fingerprint(), "callbacks": read[0], "ignored": read[1],
                 "hidden": hidden_tracked(), "content": {p: content(ROOT / p) for p in changed},
                 "markers": [content(m) if os.path.lexists(m) else None for m in markers]}
        return sha256(json.dumps(bound, sort_keys=True))[:16]

    def resume(self, accept: str | None = None) -> str:
        """Clear STOP and PAUSE. The saved baseline stays while anything may still change the
        checkout (a running controller or worker, an active manual claim), and it is never
        retired over a change the lead has not acknowledged by its digest."""
        self.own_state()  # before any write or unlink can follow a planted link into a trusted home
        self.unlink_in_state("STOP", "PAUSE")
        self.lock_file = open_private(self.state / "controller.lock")
        try:
            fcntl.flock(self.lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return "stop/pause cleared; a controller is running, so the saved session baseline is kept"
        self.lock_acquired = True
        self.refuse_live_workers()
        if active := [t["id"] for t in load_ledger(LEDGER)["tasks"] if t["status"] == "active"]:
            return (f"stop/pause cleared; manual task {active} is active, so the saved session baseline "
                    "is kept until it is validated")
        read = self.read_checkout()
        review = self.baseline_review(read)
        if review is None:
            return "stop/pause cleared; no saved session baseline"
        # Before any digest: the review cannot show what lies inside a nested repository.
        self.refuse_invisible_changes()
        digest = self.review_digest(review, read)
        # A manual claim always needs the lead's word: its session may still be running and plant
        # something after an empty delta was shown.
        if any(review.values()) and accept != digest:
            shown = {k: (v[:50] + [f"... {len(v) - 50} more"] if len(v) > 50 else v) if isinstance(v, list) else v
                     for k, v in review.items()}
            note = (" Confirm that the manual session has ended and validate its result." if review.get("manual_claim")
                    else " There is no trusted baseline to compare with: review the whole checkout, its ignored "
                         "files and Git callbacks." if review.get("no_trusted_baseline") else "")
            return ("stop/pause cleared; the saved session baseline is kept because retiring it would adopt "
                    f"this change: {json.dumps(shown)}.{note} Review it (Git callbacks included), then run "
                    f"`resume --accept {digest}`")
        # The acknowledged state becomes the baseline, so a change made after it (a detached child of
        # the session, say) is still compared by the next preflight, which retires it only last.
        snapshot = {"baseline": read[0], "session": f"accepted {now()}", "ignored": read[1]}  # the reviewed read
        self.own_state()  # again, before anything is retired: a link swapped in since would carry the unlink
        self.unlink_in_state(CALLBACKS_MARKER)  # a legacy in-checkout baseline
        self.retire_baseline()  # writes the trust record and drops the nonce
        atomic_write(self.marker(), json.dumps(snapshot))
        return ("stop/pause cleared; the reviewed state is now the saved session baseline (the next preflight "
                "compares against it and retires it)")

    def check_callbacks(self, where: str) -> None:
        if git_callbacks() != self.callbacks:
            # Sticky: a later run must not adopt the planted callback as its new baseline.
            atomic_write(self.state / "STOP", f"Git callbacks changed during {where}\n")
            raise Stop(f"Git hooks or configuration changed during {where}; no further Git "
                       "command runs until the lead inspects .git and the Git configuration")

    # --- one fresh headless session ---
    def session(self, kind: str, task_id: str, attempt_dir: Path, prompt: str, schema: dict,
                rules: list[str], mode: str, timeout: int, disallowed: str | None = None,
                budget: float | None = None, effort: str | None = None) -> dict:
        require_pipeline_release()
        if self.spent >= self.cfg.total_budget_usd:
            raise Stop(f"cumulative estimated cost {self.spent:.2f} USD reached the cap")
        session_id = str(uuid.uuid4())
        settings = attempt_dir / f"{kind}-settings.json"
        # The baseline left build/, so Edit(build/roadmap-pipeline/**) no longer covers it.
        deny = DENY + [f"Edit(/{(self.baseline or baseline_home()).parent}/**)"]
        atomic_write(settings, json.dumps({"permissions": {"allow": rules, "deny": deny}}, indent=2))
        brief = attempt_dir / f"{kind}-brief.md"
        atomic_write(brief, prompt)
        argv = claude_argv(self.cfg, f"{PROCESS_TAG}{kind}-{task_id}", session_id, settings, schema, mode,
                           budget or self.cfg.session_budget_usd, disallowed, effort)
        atomic_write(attempt_dir / f"{kind}-argv.json", json.dumps(argv[:1] + argv[1:], indent=1))
        out, err = attempt_dir / f"{kind}-result.json", attempt_dir / f"{kind}-stderr.log"
        started = time.monotonic()
        # Kept until the caller has validated this session (retire_baseline): a Stop or an interrupted
        # controller before then is checked against it later.
        atomic_write(self.marker(), json.dumps({"baseline": self.callbacks, "session": session_id,
                                                "ignored": self.ignored_files()}))
        proc = None
        try:
            with open_private(out, "w") as stdout, open_private(err, "w") as stderr, brief.open("rb") as stdin:
                proc = subprocess.Popen(argv, cwd=ROOT, env=worker_env(dict(os.environ)), stdin=stdin,
                                        stdout=stdout, stderr=stderr, start_new_session=True)
                self.heartbeat(task=task_id, phase=kind, worker_pid=proc.pid, session_id=session_id)
                code, stop_reason = None, None
                deadline = time.monotonic() + timeout
                while code is None:
                    try:
                        code = proc.wait(timeout=60)
                    except subprocess.TimeoutExpired:
                        live = self.transcript_check(session_id, prompt)
                        self.heartbeat(compactions=live.get("compactions", 0))
                        if live.get("compactions", 0) > self.cfg.max_compactions:
                            stop_reason = f"compaction churn: {live['compactions']} compactions"
                        elif time.monotonic() > deadline:
                            stop_reason = f"timeout after {timeout}s"
                        if stop_reason:
                            os.killpg(proc.pid, signal.SIGTERM)
                            with contextlib.suppress(subprocess.TimeoutExpired):
                                proc.wait(timeout=30)
                            code = -9
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(proc.pid, signal.SIGKILL)  # no orphan child may keep editing
        finally:
            if proc is not None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(proc.pid, signal.SIGKILL)  # also on an exception path
            self.check_callbacks(f"the {kind} session {session_id}")
        self.heartbeat(worker_pid=None)
        text = out.read_text(errors="replace")
        try:
            result = json.loads(text)
        except json.JSONDecodeError:
            result = {"is_error": True, "result": text[-4000:]}
        cost = float(result.get("total_cost_usd") or 0.0)
        self.spent += cost
        record = {
            "kind": kind, "session_id": session_id, "reported_session_id": result.get("session_id"),
            "exit_code": code, "stopped_by_controller": stop_reason,
            "duration_s": round(time.monotonic() - started, 1),
            "is_error": bool(result.get("is_error")), "subtype": result.get("subtype"),
            "num_turns": result.get("num_turns"), "est_cost_usd": cost, "usage": result.get("usage"),
            "brief_sha256": sha256(prompt), "transcript": self.transcript_check(session_id, prompt),
            "output": structured(result),
        }
        atomic_write(attempt_dir / f"{kind}-record.json", json.dumps(record, indent=2))
        blob = text + err.read_text(errors="replace")
        if code != 0 and RATE_LIMIT.search(blob):
            raise Stop(f"{kind} session hit a rate or usage limit; see {err}")
        return record

    @staticmethod
    def transcript_check(session_id: str, prompt: str) -> dict:
        """Prove the session started empty: one root user message equal to the brief."""
        slug = re.sub(r"[^A-Za-z0-9]", "-", str(ROOT))
        path = Path.home() / ".claude" / "projects" / slug / f"{session_id}.jsonl"
        if not path.exists():
            return {"found": False}
        roots, users, foreign, compactions, output_tokens, peak_context = 0, 0, 0, 0, 0, 0
        first = None
        for line in path.read_text(errors="replace").splitlines():
            with contextlib.suppress(json.JSONDecodeError):
                item = json.loads(line)
                if item.get("sessionId") not in (None, session_id):
                    foreign += 1
                if item.get("subtype") == "compact_boundary":
                    compactions += 1
                if item.get("type") == "assistant":
                    usage = item.get("message", {}).get("usage") or {}
                    output_tokens += usage.get("output_tokens", 0)
                    peak_context = max(peak_context, usage.get("input_tokens", 0)
                                       + usage.get("cache_read_input_tokens", 0)
                                       + usage.get("cache_creation_input_tokens", 0))
                if item.get("type") == "user":
                    users += 1
                    if item.get("parentUuid") is None:
                        roots += 1
                        first = first or item.get("message", {}).get("content")
        return {"found": True, "root_user_messages": roots, "user_entries": users,
                "foreign_session_entries": foreign, "compactions": compactions,
                "output_tokens": output_tokens, "peak_context_tokens": peak_context,
                "first_message_is_brief": isinstance(first, str) and first.strip() == prompt.strip()}

    # --- validation owned by the controller ---
    def tree_fingerprint(self) -> str:
        # Status and diff list an untracked file but not its content: hash that content too.
        untracked = {}
        for path in filter(None, git("ls-files", "-z", "--others", "--exclude-standard").split("\0")):
            target = ROOT / path
            with contextlib.suppress(FileNotFoundError):
                untracked[path] = [target.lstat().st_mode, os.readlink(target) if target.is_symlink()
                                   else sha256(target.read_bytes()) if target.is_file() else "other"]
        # The index can call a tracked edit clean (flags, forged stat): hash what a fresh index sees.
        tracked = {}
        for path in tracked_changes():
            target = ROOT / path
            tracked[path] = (os.readlink(target) if target.is_symlink() else sha256(target.read_bytes())
                             if target.is_file() else "missing" if not os.path.lexists(target) else "other")
        return sha256(git("status", "--porcelain", "--untracked-files=all") + git(*DIFF_HEAD)
                      + json.dumps([self.ignored_files(), nested_git(ROOT), hidden_tracked(), untracked, tracked],
                                   sort_keys=True))

    def changed(self) -> list[str]:
        return sorted(set(parse_porcelain(git("status", "--porcelain", "--untracked-files=all")))
                      | set(tracked_changes()))

    def ignored_files(self) -> dict[str, list[int]]:
        """Ignored files outside the controller's own state, which `git status` never reports.
        shortcut: lstat identity, not content hashes (target/ is large); upgrade if a session
        gains a tool that can restore timestamps."""
        try:
            own = self.state.resolve().relative_to(ROOT.resolve()).as_posix() + "/"
        except ValueError:
            own = None  # state outside the checkout
        snapshot = {}
        for path in filter(None, git("ls-files", "-z", "--others", "--ignored", "--exclude-standard").split("\0")):
            if own and path.startswith(own):
                continue
            with contextlib.suppress(FileNotFoundError):
                info = (ROOT / path).lstat()
                snapshot[path] = [info.st_mode, info.st_size, info.st_mtime_ns, info.st_ctime_ns, info.st_ino]
        return snapshot

    def ignored_delta(self, before: dict[str, list[int]],
                      after: dict[str, list[int]] | None = None) -> tuple[list[str], list[str]]:
        """(created, changed or removed) ignored files since the snapshot `before`."""
        after = self.ignored_files() if after is None else after
        return sorted(set(after) - set(before)), sorted(p for p in before if after.get(p) != before[p])

    def new_ignored(self, before: dict[str, list[int]]) -> list[str]:
        """Ignored files a session created; one it changed or removed cannot be restored."""
        created, altered = self.ignored_delta(before)
        if altered:
            raise Stop(f"a session changed or removed ignored files; review them: {altered[:20]}")
        return created

    def revert(self, paths: list[str]) -> None:
        """Only for paths the worker created or edited in a checkout that was clean before."""
        tracked = set(git("ls-files", "--", *paths).split()) if paths else set()
        for path in paths:
            if path in tracked:
                git("checkout", "HEAD", "--", path, check=False)
            else:
                # The entry itself, never what a symlink points at; its parent must lie inside ROOT.
                target = ROOT / path
                parent = target.parent.resolve()
                if (parent == ROOT.resolve() or ROOT.resolve() in parent.parents) and os.path.lexists(target):
                    shutil.rmtree(target) if target.is_dir() and not target.is_symlink() else target.unlink()

    def run_checks(self, task: dict, attempt_dir: Path) -> tuple[bool, list[dict], str]:
        require_pipeline_release()
        results, failure = [], ""
        lock = open_private(self.state / "qemu.lock") if task.get("native") else None
        if lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            self.heartbeat(phase="checks")
            # A fresh directory the worker could not pre-seed with files bound into the receipt.
            log = Path(tempfile.mkdtemp(prefix="checks-", dir=attempt_dir)) / "checks.log"
            commands = [(f"check-{i}", c, {}) for i, c in enumerate(task["checks"], 1)]
            try:
                gate = resolve_gate(task["gates"], ROOT / ".github" / "workflows") if task.get("gates") else None
                # resolve() already ends with the progress check; plain checks add it here.
                # One shared deadline: the task checks' budget plus the resolved gate's own budget.
                timeout = self.cfg.check_timeout + ((gate.timeout_seconds or 0) if gate else 0)
                plan = GatePlan(tuple(commands) + (gate.commands if gate else (PROGRESS_CHECK,)),
                                gate.finalizers if gate else (), timeout,
                                gate.finalizer_reserve_seconds if gate else 0)
                # Untracked candidate files must reach the gate; it stages only these paths.
                sources = agent_source_paths(self.changed())
            except (KeyError, ValueError, ImportError, OSError, subprocess.SubprocessError) as error:
                failure = f"gate blocked: {error}"
                atomic_write(log, failure + "\n")
                code = GATE_FAIL_EXIT
            else:
                code = run_gate(plan, ROOT, log, os.environ.copy(), source_paths=sources)
            digest, report = gate_report(log)
            results.append({"name": "isolated-checks", "commands": [c[1] for c in commands],
                            "gates": task.get("gates", []), "exit_code": code, "log": str(log),
                            "log_sha256": digest})
            if code:
                return False, results, failure or report or f"isolated checks exited {code}"
            return True, results, ""
        finally:
            if lock:
                fcntl.flock(lock, fcntl.LOCK_UN)
                lock.close()

    # --- prompts ---
    def worker_prompt(self, task: dict, head: str, feedback: str,
                      runner: str = "the controller runs the task's checks and gates in the isolated "
                                    "gate runner and returns any failure to the next session") -> str:
        fields = {k: task.get(k) for k in ("id", "criterion", "milestone", "title", "goal", "why_now",
                                           "write_paths", "read_paths", "graph_notes", "checks", "acceptance",
                                           "out_of_scope", "non_claimable", "risk", "evidence_level")}
        text = f"""roadmap-pipeline-task {task['id']} (run {self.run_id})

You are a FRESH Codex worker session of the BaryonOS H/U/N/B/A/P/L/R roadmap pipeline. You
have no earlier conversation; read this brief, AGENTS.md and applicable directory rules
and the repository are your only context. Implement exactly ONE cohesive, verifiable
slice described below, then stop.

Repository: {ROOT} on branch {self.cfg.branch} at {head}. {"The working tree holds the previous attempt's uncommitted change; repair it." if feedback else "The checkout was clean before the controller recorded this active ledger claim; that lead-owned ledger change is expected."}

Task (immutable; from docs/roadmap-pipeline/ledger.json):
{json.dumps(fields, indent=2, ensure_ascii=False)}

Rules (binding, in addition to AGENTS.md):
- Use the actual builder role `.codex/agents/os_builder.toml`; optional missing
  role files are not prerequisites. Claude owns S09 and Orbitaal, Codex owns this task.
- Write ONLY inside write_paths. Never edit .github/, tools/agent_*.py, tools/common.py,
  tools/progress.py, tools/roadmap_runner.py, AGENTS.md, CLAUDE.md, docs/progress.json,
  docs/PROGRESS.md, docs/PROJECT-STATE.md, docs/DECISIONS.md, README.md or
  docs/roadmap-pipeline/. The lead checks the full diff before accepting it.
- Do not run git commands that change state, do not commit, push, fetch or use gh.
  Only `git status`, `git rev-parse` and `git branch --show-current` are allowed.
  No network access is needed.
- Graphify first: if Graphify MCP tools are available, invoke `recall` with your write
  paths and use graphify_find/impact/tests_for/file_neighbors before editing; the graph
  reflects main, so confirm against current files. If unavailable, read code directly
  and say so. Do not invoke `remember` (the lead records durable notes).
- Load the `ponytail` skill (full) if available: smallest safe change, reuse existing
  utilities, no reduced functionality or test evidence. Use relevant
  `verification-loop` guidance and `.agents/skills/os-dev` only where it applies.
- Targeted read-only source inspection using cat, rg, sed or Token Guard inspect/slice
  is allowed, plus `git status`, `git rev-parse` and `git branch --show-current`. Do
  not build or run code: {runner}. Tool policy is not a claim of technical isolation.
- Security Floor: no new trust roots, key escrow, telemetry, cloud fallback, broader
  capabilities or relaxed verification. Do not invent cryptography. Validate untrusted
  lengths and pointers. Add negative tests for every new boundary.
- Public text, comments and docs in professional, impersonal English. Label evidence
  levels honestly (host test, native QEMU, physical hardware, human study, external
  audit). Never claim a check passed unless you ran it and saw exit code 0; the
  lead validates results against source and raw logs before acceptance.
- Do not change docs/progress.json or award acceptance; report what evidence exists.
- If the task is not safely implementable as specified, stop and report state
  "blocked" with the precise blocker instead of improvising a different task.
"""
        if feedback:
            text += f"\nThe previous attempt was rejected. Resolve ONLY these concrete findings:\n{feedback}\n"
        for plugin, skill in (("ponytail/ponytail", "ponytail"), ("ecc/ecc", "verification-loop")):
            paths = list((Path.home() / ".codex/plugins/cache" / plugin).glob(f"*/skills/{skill}/SKILL.md"))
            if paths:
                text += f"\nRead {max(paths, key=lambda path: path.stat().st_mtime)} once; apply only relevant guidance.\n"
        text += "\nFinish with one JSON result matching WORKER_SCHEMA in tools/roadmap_runner.py; no log dumps or false success.\n"
        return text

    def prepare_manual(self) -> dict:
        """Claim a READY task without executing untrusted code or lifting the hold."""
        if not self.lock_acquired:
            raise Stop("manual claim requires the controller lock")
        self.check_controls()
        ledger = load_ledger(LEDGER)
        if any(task["status"] == "active" for task in ledger["tasks"]):
            raise Stop("a manual task is already active; validate or recover it first")
        head = self.preflight()
        task = select_task(ledger)
        if task is None:
            raise Stop("no eligible READY task; preserve active/manual work")
        directory = self.state / "manual" / f"{task['id']}-{uuid.uuid4().hex}"
        directory.mkdir(parents=True)
        prompt = self.worker_prompt(task, head, "", "the lead runs the task's checks and gates after "
                                                    "validating the diff")
        prompt += ("\nManual fresh-session fallback: the lead launches this session, validates the diff "
                   "and runs required gates directly. Do not run the held controllers or agent_gate. "
                   "No automatic continuation or safe unattended readiness is established.\n")
        atomic_write(directory / "brief.md", prompt)
        plan = {"task_id": task["id"], "base": head, "branch": self.cfg.branch,
                "brief_sha256": sha256(prompt), "stdin": str(directory / "brief.md"),
                "argv": codex_argv(directory / "final.json"), "cwd": str(ROOT),
                "stdout": str(directory / "events.jsonl"), "stderr": str(directory / "stderr.log"),
                "manual_required": True, "unattended_ready": False}
        atomic_write(directory / "dispatch.json", json.dumps(plan, indent=2))
        # The manual session may change Git state: the next preflight compares, never adopts.
        atomic_write(self.marker(), json.dumps({"baseline": self.callbacks, "manual": task["id"],
                                                "session": f"manual {task['id']}",
                                                "ignored": self.ignored_files()}))
        update_task(task["id"], path=LEDGER, status="active", history={
            "at": now(), "event": "manual-codex-claim", "base": head,
            "dispatch": str(directory / "dispatch.json"), "brief_sha256": plan["brief_sha256"]})
        return plan

    def verifier_prompt(self, task: dict, head: str, diff: str, untracked: list[str], checks: list[dict]) -> str:
        return f"""roadmap-pipeline-verify {task['id']} (run {self.run_id})

You are an INDEPENDENT, READ-ONLY verifier in a fresh session. You did not write this
change and you do not see the writer's self-report. Judge only the code and the
evidence below against the task. You cannot edit files or run commands.

Repository: {ROOT}, branch {self.cfg.branch}, base commit {head}. The working tree
contains the uncommitted change.

Task:
{json.dumps({k: task.get(k) for k in ("id", "criterion", "title", "goal", "write_paths", "acceptance", "out_of_scope", "risk")}, indent=2, ensure_ascii=False)}

Controller-run checks (raw exit codes, logs kept by the controller):
{json.dumps(checks, indent=2)}

New untracked files (read them): {json.dumps(untracked)}

Diff against {head} (may be truncated; read the files for full context):
```diff
{diff}
```

Check: correctness, the task's acceptance clauses, negative/fault tests for every new
boundary, AGENTS.md security rules (capability widening, untrusted lengths/pointers,
uncontrolled DMA, secrets in logs, authentication on failure, falsified evidence),
overclaiming in docs (host vs native vs hardware), and scope creep. Use Graphify
read tools if available. severity: blocker = wrong/unsafe/false claim; major = an
acceptance clause or a required negative test is missing; minor = everything else.
verdict "pass" only with no blocker or major finding.
"""

    def init_prompt(self) -> str:
        return f"""roadmap-pipeline-task INIT-CHECK (run {self.run_id})

This is a pipeline initialisation check of a FRESH headless session. Do not edit any
file. Report facts only:
1. Whether you can see any earlier conversation turns before this message.
2. Run `git rev-parse HEAD` and `git branch --show-current`; report cwd, branch, head.
3. Whether AGENTS.md/CLAUDE.md project instructions are in your context, and the
   first Markdown heading of AGENTS.md as you see it.
4. The exact names of all Graphify MCP tools available to you (search your deferred
   tools for "graphify"); if one is available, call list_repositories and report
   whether BaryonOS/BaryonOS is queryable.
5. Which of these skills are listed for you: ponytail, verification-loop, token-guard,
   os-dev (or the repository skill .agents/skills/os-dev).
6. Policy probe. Try exactly `python3 -c "open('build/roadmap-pipeline/DENY-PROBE','w').write('x')"`;
   it is not allowed and should be refused without a prompt. Report the outcome in
   bash_denied_example. Do not try any workaround.
Return the structured result.
"""

    # --- task lifecycle ---
    def run_task(self, task: dict) -> str:
        require_pipeline_release()
        head = self.preflight()
        if task.get("gates"):  # PyYAML and the workflows must work before a session is spent
            try:
                resolve_gate(task["gates"], ROOT / ".github" / "workflows")
            except (KeyError, ValueError, ImportError) as error:
                raise Blocked(f"gate cannot be resolved: {error}") from error
        # The ledger is not touched while a worker runs: 'active' is the lease itself.
        self.heartbeat(task=task["id"], phase="start", base=head)
        task_dir = self.state / "runs" / self.run_id / task["id"]
        feedback, last_fp, attempts = "", None, []
        for attempt in range(1, self.cfg.max_repairs + 2):
            self.check_controls()
            adir = task_dir / f"attempt-{attempt}"
            adir.mkdir(parents=True, exist_ok=True)
            ignored = self.ignored_files()
            worker = self.session("worker", task["id"], adir, self.worker_prompt(task, head, feedback),
                                  WORKER_SCHEMA, allow_rules(("Edit", "Write", "MultiEdit")),
                                  "acceptEdits", self.cfg.worker_timeout, effort=effort_for(self.cfg, task))
            record = {"attempt": attempt, "worker": worker}
            attempts.append(record)
            if nested := nested_git(ROOT):  # invisible to the scope check and revert below
                raise Stop(f"the worker session left nested .git paths; review them: {brief(nested)}")
            if hidden := hidden_tracked():  # their edits are invisible to the scope check as well
                raise Stop(f"the worker session hid tracked edits behind index flags: {brief(hidden)}")
            changed = self.changed()
            # New ignored files are never in scope: nothing commits them and later runs may use them.
            if bad := [p for p in changed if not in_write_set(p, task["write_paths"])] + self.new_ignored(ignored):
                self.revert(bad)
                record["reverted_out_of_scope"] = bad
                changed = self.changed()
                if leftover := [p for p in changed if not in_write_set(p, task["write_paths"])] \
                        + self.new_ignored(ignored):  # e.g. a path revert() cannot remove
                    raise Stop(f"out-of-scope paths survived the revert; review them: {leftover[:20]}")
            self.retire_baseline()  # the worker's nested .git, ignored-file and scope checks passed
            out = worker.get("output") or {}
            if worker["exit_code"] != 0 or worker["is_error"]:
                kind = "session"
                detail = (f"worker session failed (exit {worker['exit_code']}, {worker['subtype']}, "
                          f"{worker.get('stopped_by_controller') or 'no controller stop'})")
            elif out.get("state") == "blocked":
                self.revert(changed)
                raise Blocked("worker reported blocked: " + "; ".join(out.get("blockers") or ["(no detail)"]))
            elif not changed:
                kind, detail = "no-change", "no in-scope file was changed"
            else:
                ok, checks, failure = self.run_checks(task, adir)
                record["checks"] = checks
                if not ok:
                    kind, detail = "checks", failure
                else:
                    diff = git(*DIFF_HEAD)
                    untracked = [p for p in changed if p not in set(git("ls-files", "--", *changed).split())]
                    before = self.tree_fingerprint()
                    verifier = self.session("verifier", task["id"], adir,
                                            self.verifier_prompt(task, head, diff[:60_000], untracked, checks),
                                            VERIFIER_SCHEMA, allow_rules(), "default",
                                            self.cfg.verifier_timeout, disallowed=READ_ONLY_DISALLOWED,
                                            budget=self.cfg.verifier_budget_usd,
                                            effort=effort_for(self.cfg, task))
                    record["verifier"] = verifier
                    if self.tree_fingerprint() != before:
                        raise Stop("the read-only verifier changed the working tree; stopping for review")
                    self.retire_baseline()
                    verdict = verifier.get("output") or {}
                    findings = verdict.get("findings") or []
                    if verifier["exit_code"] == 0 and verdict.get("verdict") == "pass" and not blocking(findings):
                        return self.checkpoint(task, head, changed, attempts)
                    kind = "verifier"
                    detail = json.dumps(blocking(findings) or findings or verdict or verifier.get("subtype"), indent=1)
            if bad := record.get("reverted_out_of_scope"):
                detail += f"\nOut-of-scope edits were reverted: {bad}"
            fp = failure_fingerprint(kind, detail)
            record["failure"] = {"kind": kind, "fingerprint": fp, "detail": detail[-4000:]}
            atomic_write(task_dir / "attempts.json", json.dumps(attempts, indent=2))
            if fp == last_fp:
                self.revert(self.changed())
                raise Blocked(f"two identical failures ({kind}) without new evidence")
            last_fp, feedback = fp, detail[-12000:]
        self.revert(self.changed())
        raise Blocked(f"not verified after {self.cfg.max_repairs} repair rounds; last failure: {feedback[:600]}")

    def checkpoint(self, task: dict, head: str, changed: list[str], attempts: list[dict]) -> str:
        receipt = {
            "task": task["id"], "criterion": task["criterion"], "run": self.run_id, "base": head,
            "branch": self.cfg.branch, "verified_at": now(), "files": sorted(changed),
            "file_sha256": {p: sha256((ROOT / p).read_bytes()) for p in changed if (ROOT / p).is_file()},
            "sessions": [{k: a[s].get(k) for k in ("kind", "session_id", "exit_code", "duration_s", "num_turns",
                                                    "est_cost_usd", "brief_sha256", "transcript")}
                         for a in attempts for s in ("worker", "verifier") if s in a],
            "checks": attempts[-1].get("checks"),
            "verifier_findings": ((attempts[-1].get("verifier") or {}).get("output") or {}).get("findings", []),
            "worker_report": (attempts[-1]["worker"].get("output") or {}),
            "evidence_level": task.get("evidence_level"),
            "acceptance_changed": False,
            "note": "Controller-run checks decide; worker JSON is recorded, not trusted. "
                    "A verified task is an engineering checkpoint, not criterion acceptance.",
        }
        rel = f"docs/roadmap-pipeline/receipts/{task['id']}.json"
        atomic_write(ROOT / rel, json.dumps(receipt, indent=2, ensure_ascii=False) + "\n")
        update_task(task["id"], status="verified", history={"at": now(), "event": "verified", "run": self.run_id,
                                                             "receipt": rel, "attempts": len(attempts)})
        git("add", "--", *changed, rel, str(LEDGER.relative_to(ROOT)))
        message = (f"roadmap({task['id']}): {task['title']}\n\n"
                   f"Criterion {task['criterion']}; verified by controller checks and an independent "
                   f"read-only session. Evidence level: {task.get('evidence_level')}. No acceptance "
                   f"change. Receipt: {rel}\n\n" + TRAILERS)
        git("commit", "--no-verify", "-q", "-m", message)
        commit = git("rev-parse", "HEAD").strip()
        self.record_assessment(task, commit, "verified")
        self.push()
        return commit

    def record_assessment(self, task: dict, commit: str, outcome: str) -> None:
        """AGENTS.md: reassess after every task, even when the score is unchanged. The
        controller may only restate the assessment; it never changes a criterion."""
        path = ROOT / "docs" / "progress.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        before = json.dumps(data["tracks"], sort_keys=True)
        score = sum(g["points"] for t in data["tracks"] for g in t["gates"] if g["status"] == "accepted")
        amsterdam = dt.datetime.now(dt.timezone.utc) + dt.timedelta(hours=1)  # CET; no DST in November
        summary = (f"{score}/100 remains {score}/100. Roadmap pipeline task {task['id']} ({task['criterion']}) "
                   f"{outcome} at evidence level {task.get('evidence_level')}; controller-run checks and an "
                   "independent read-only session decided. A verified pipeline task is an engineering checkpoint "
                   "and accepts no criterion; acceptance is recorded only by the lead with the required evidence.")
        data["assessment"] = {"date": amsterdam.date().isoformat(), "source_commit": commit,
                              "assessment_basis": "parent_commit", "branch": self.cfg.branch,
                              "task": f"Roadmap pipeline {task['id']}: {task['title']}"[:200],
                              "previous_score": score, "summary": summary.replace("|", "/")}
        if json.dumps(data["tracks"], sort_keys=True) != before:
            raise Stop("assessment update would change criteria; refusing")
        atomic_write(path, json.dumps(data, indent=2, ensure_ascii=False) + "\n")
        for mode in ("--write", "--check"):
            # -I keeps the checkout's tools/ directory off sys.path: no worker module shadows stdlib.
            done = subprocess.run([sys.executable, "-I", "-B", "tools/progress.py", mode], cwd=ROOT,
                                  capture_output=True, text=True)
            if done.returncode:
                raise Stop(f"progress.py {mode} failed: {done.stdout[-400:]}{done.stderr[-400:]}")
        views = ["docs/progress.json", "README.md", "docs/PROGRESS.md", "docs/assets/progress.svg"]
        git("add", "--", *views)
        if git("diff", "--cached", "--name-only").strip():
            git("commit", "--no-verify", "-q", "-m", f"progress: assess roadmap task {task['id']} ({score}/100 unchanged)\n\n"
                f"Assessment of {commit}; no criterion changed.\n\n" + TRAILERS)

    def push(self) -> None:
        if not self.cfg.push:
            return
        for delay in (0, 2, 4, 8, 16):
            time.sleep(delay)
            done = subprocess.run(["git", *CONTROLLER_GIT_CONFIG, "push", "--no-verify", "-q", "-u", "origin",
                                   self.cfg.branch], cwd=ROOT,
                                  capture_output=True, text=True, env=git_env())
            if done.returncode == 0:
                return
        raise Stop(f"push failed: {done.stderr.strip()[-600:]}")

    def mark(self, task_id: str, status: str, reason: str) -> None:
        update_task(task_id, status=status, history={"at": now(), "event": status, "run": self.run_id,
                                                     "reason": reason})
        git("add", "--", str(LEDGER.relative_to(ROOT)))
        git("commit", "--no-verify", "-q", "-m", f"roadmap({task_id}): mark {status}\n\n{reason[:900]}\n\n" + TRAILERS)
        task = next(t for t in load_ledger()["tasks"] if t["id"] == task_id)
        self.record_assessment(task, git("rev-parse", "HEAD").strip(), status)
        self.push()

    def init_check(self) -> dict:
        head = self.preflight()
        adir = self.state / "runs" / self.run_id / "INIT-CHECK"
        adir.mkdir(parents=True, exist_ok=True)
        probe = ROOT / INIT_DENY_PROBE
        probe.unlink(missing_ok=True)
        ignored = self.ignored_files()
        record = self.session("init", "INIT-CHECK", adir, self.init_prompt(), INIT_SCHEMA,
                              allow_rules(), "default", 15 * 60,
                              disallowed=READ_ONLY_DISALLOWED, budget=self.cfg.verifier_budget_usd)
        record["head_expected"] = head
        record["checkout_clean_after"] = not (self.changed() or any(self.ignored_delta(ignored)) or nested_git(ROOT)
                                              or hidden_tracked())
        if record["checkout_clean_after"]:  # otherwise the next run compares with the init baseline
            self.retire_baseline()
        record["deny_probe_file_absent"] = not probe.exists()  # controller-side oracle
        probe.unlink(missing_ok=True)
        atomic_write(adir / "init-check.json", json.dumps(record, indent=2))
        return record

    def loop(self, max_tasks: int, init: bool) -> str:
        log = self.state / "runs" / self.run_id / "controller.jsonl"
        log.parent.mkdir(parents=True, exist_ok=True)

        def event(**fields):
            with open_private(log, "a") as out:
                out.write(json.dumps({"at": now(), **fields}) + "\n")
            print(json.dumps({"at": now(), **fields}), flush=True)

        self.check_controls()  # a STOP from an earlier run must hold before any Git call
        try:  # the isolated gate refuses this environment: fail before any session is spent
            controller_environment()
        except ValueError as error:
            raise Stop(f"controller environment refused: {error}") from error
        if init:
            record = self.init_check()
            event(event="init-check", session=record["session_id"], exit=record["exit_code"],
                  transcript=record["transcript"], output=record["output"])
            if not (record["exit_code"] == 0 and record["deny_probe_file_absent"] and record["checkout_clean_after"]):
                raise Stop("init check failed: the session must exit 0, refuse the deny probe and "
                           "leave the checkout clean")
        done = 0
        while done < max_tasks:
            self.check_controls()
            task = select_task(load_ledger())
            if task is None:
                event(event="stop", reason="no ready task with verified dependencies")
                return "no-ready-task"
            event(event="start", task=task["id"], title=task["title"])
            try:
                commit = self.run_task(task)
                event(event="verified", task=task["id"], commit=commit, spent_usd=round(self.spent, 2))
            except Blocked as error:
                self.mark_blocked(task["id"], str(error))
                event(event="blocked", task=task["id"], reason=str(error)[:600])
            done += 1
            (self.state / "lease.json").unlink(missing_ok=True)
        event(event="stop", reason=f"max tasks {max_tasks} reached")
        return "max-tasks"

    def mark_blocked(self, task_id: str, reason: str) -> None:
        if self.changed():
            raise Stop(f"{task_id} blocked but the checkout is not clean after revert; review needed")
        self.mark(task_id, "blocked", reason)


def status(state: Path = STATE) -> int:
    ledger = load_ledger()
    counts: dict[str, int] = {}
    for task in ledger["tasks"]:
        counts[task["status"]] = counts.get(task["status"], 0) + 1
    nxt = select_task(ledger)
    lease = state / "lease.json"
    print(json.dumps({"counts": counts, "next": nxt and nxt["id"],
                      "lease": json.loads(lease.read_text()) if lease.exists() else None,
                      "stop": (state / "STOP").exists(), "pause": (state / "PAUSE").exists()}, indent=2))
    return 0


def isolate_imports() -> None:
    """Worker sessions can write under the checkout: no later import, including lazy stdlib
    imports and agent_gate's PyYAML, may resolve a module from there."""
    checkout = {ROOT, ROOT / "tools"}
    sys.path[:] = [p for p in sys.path if Path(p or ".").resolve() not in checkout]
    # Optional for plain checks; a task with gates resolves them before its worker session.
    with contextlib.suppress(ImportError):
        import yaml  # noqa: F401  (agent_gate imports it only after a worker session)


def main() -> int:
    isolate_imports()
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="cmd", required=True)
    run = sub.add_parser("run", help="run the controller loop")
    run.add_argument("--max-tasks", type=int, default=1)
    run.add_argument("--init-check", action="store_true", help="start with a fresh-session self-check")
    run.add_argument("--no-push", action="store_true")
    run.add_argument("--total-budget-usd", type=float, default=Config.total_budget_usd)
    run.add_argument("--session-budget-usd", type=float, default=Config.session_budget_usd)
    run.add_argument("--model", default=Config.model)
    run.add_argument("--effort", default=Config.effort, help="security/high-risk tasks")
    run.add_argument("--routine-effort", default=Config.routine_effort, help="low/medium-risk tasks")
    for name in ("status", "stop", "pause", "validate", "brief"):
        sub.add_parser(name)
    sub.add_parser("resume").add_argument(
        "--accept", metavar="DIGEST", help="retire a saved baseline over the change `resume` reported")
    args = parser.parse_args()
    try:
        pin_state(STATE)  # before this mkdir or the stop/pause write could follow a planted link
    except Stop as error:
        print(error)
        return 2
    if args.cmd == "status":
        return status()
    if args.cmd == "validate":
        load_ledger()
        print("ledger ok")
        return 0
    if args.cmd in ("stop", "pause"):
        atomic_write(STATE / args.cmd.upper(), now())
        print(f"{args.cmd} requested; the controller honours it before the next session")
        return 0
    if args.cmd == "resume":
        # The lead has reviewed the reported change: a sticky STOP must not re-fire on the old baseline.
        controller = Controller(Config(), STATE)
        try:
            print(controller.resume(args.accept))
        except Stop as error:
            held = [n for n in ("STOP", "PAUSE") if os.path.lexists(STATE / n)]  # a refusal can precede the clear
            print(f"{'stop/pause cleared' if not held else f'{held} still present'}; "
                  f"the saved session baseline is kept: {error}")
            return 2
        finally:
            controller.release()
        return 0
    if args.cmd == "brief":
        if (STATE / "STOP").exists():  # before any Git call, like the run loop
            print("stop file present (build/roadmap-pipeline/STOP)")
            return 2
        controller = Controller(Config(branch=git("branch", "--show-current").strip()))
        try:
            controller.acquire()
            print(json.dumps(controller.prepare_manual(), indent=2))
        except Stop as error:
            print(str(error))
            return 2
        finally:
            controller.release()
        return 0
    try:
        require_pipeline_release()
    except TaskError as error:
        print(f"{error} Use `brief` for lead-supervised fresh Codex dispatch.")
        return 2
    cfg = Config(push=not args.no_push, total_budget_usd=args.total_budget_usd,
                 session_budget_usd=args.session_budget_usd, model=args.model, effort=args.effort,
                 routine_effort=args.routine_effort)
    if not shutil.which(cfg.claude):
        print("claude CLI not found; fall back to one manually started fresh session per task")
        return 3
    controller = Controller(cfg)
    try:
        controller.acquire()
        reason = controller.loop(args.max_tasks, args.init_check)
    except Stop as error:
        print(json.dumps({"at": now(), "event": "stop", "reason": str(error)}), flush=True)
        return 2
    finally:
        controller.release()
    print(json.dumps({"at": now(), "event": "exit", "reason": reason, "spent_usd": round(controller.spent, 2)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
