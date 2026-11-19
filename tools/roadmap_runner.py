#!/usr/bin/env python3
"""Clean-session roadmap runner: one fresh headless Claude Code session per ledger task.

The long-lived part is this small deterministic controller, not a chat transcript.
It owns the task ledger, an exclusive controller lock, a write lease with heartbeat,
stop/pause files, logs and receipts. For each READY task it

1. verifies controller Git callbacks and a protected, pinned offline sandbox policy;
2. stages reviewed regular files into a disposable Git-free source copy;
3. starts a NEW `claude -p` process inside that sandbox with a fresh `--session-id`
   (never --continue, --resume or --fork-session), a private home/network namespace,
   `--permission-prompts none`, scoped file edits and `--autocompact`;
4. runs every trusted check against the disposable copy with source mounted read-only;
5. exports only validated regular-file changes in the frozen write set after checks
   pass, then asks a fresh sandboxed read-only session to verify the candidate;
6. commits verified bytes without executing them, writes a compact receipt, pushes and
   releases the lease before the next task starts in another fresh session.

It reuses `agent_gate.resolve` for CI-job modelling, the shared `agent_sandbox`
boundary and `agent_pipeline.is_protected` for the rule that agents never edit the
files that judge them. It never merges, never enables auto-merge and never pushes
to any branch but `--branch`.
Runtime state lives in ignored build/roadmap-pipeline/; the task ledger and
receipts are versioned under docs/roadmap-pipeline/ so a new container can resume.
"""
from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fcntl
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from agent_gate import FAIL_EXIT as GATE_FAIL_EXIT, resolve as resolve_gate  # noqa: E402
from agent_pipeline import TaskError, is_protected, verify_controller  # noqa: E402
from agent_sandbox import (Sandbox, SandboxError, SandboxResourceExceeded,  # noqa: E402
                           resource_evidence)
from common import (agent_source_paths, agent_source_selection_state, agent_source_state,  # noqa: E402
                    import_agent_changes, stage_agent_source)

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs" / "roadmap-pipeline" / "ledger.json"
RECEIPTS = ROOT / "docs" / "roadmap-pipeline" / "receipts"
STATE = ROOT / "build" / "roadmap-pipeline"
SANDBOX_POLICY = ROOT / ".github" / "agent-pipeline.json"
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
GRAPHIFY_READ = ("graphify_find", "graphify_callers", "graphify_impact", "graphify_tests_for",
                 "graphify_file_neighbors", "graphify_node", "graphify_rank_files",
                 "graphify_trace", "graphify_imports_exports", "query_graph", "recall",
                 "list_repositories", "graph_stats", "search", "fetch")
GRAPHIFY_SERVERS = ("Graphify", "claude_ai_Graphify")
READ_GIT = ("git status", "git diff", "git log", "git show", "git rev-parse", "git branch --show-current")
DENY = ["Bash(git push:*)", "Bash(git commit:*)", "Bash(git reset:*)", "Bash(git checkout:*)",
        "Bash(git clean:*)", "Bash(git stash:*)", "Bash(git rebase:*)", "Bash(git merge:*)",
        "Bash(gh:*)", "Bash(curl:*)", "Bash(wget:*)", "WebFetch", "WebSearch",
        "Read(~/.claude/.credentials.json)", "Read(~/.ssh/**)", "Read(~/.config/gh/**)",
        "Read(~/.git-credentials)", "Read(//home/claude/**)", "Read(//root/.ccr/**)"]
PROCESS_TAG = "roadmap-pipeline-"
READ_ONLY_DISALLOWED = "Edit,Write,MultiEdit,NotebookEdit"
INIT_ALLOWED_PROBE = "python3 -B tools/progress.py --check"
INIT_DENY_PROBE = "build/roadmap-pipeline/DENY-PROBE"
TRAILERS = ("Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>\n"
            "Claude-Session: https://claude.ai/code/session_01TJtT9VANihpE4RzPd4d8su\n")
SANDBOX_OUTPUT_DIRS = ("build", "target")
SANDBOX_HOST_KEYS = ("OS_OVMF_CODE", "OS_OVMF_VARS")

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


def load_sandbox(path: Path = SANDBOX_POLICY) -> Sandbox:
    """Load only the protected, complete sandbox policy and verify its runtime."""
    try:
        policy = json.loads(path.read_text(encoding="utf-8"))
        if policy.get("schema") != 2 or not isinstance(policy.get("sandbox"), dict):
            raise ValueError("sandbox policy schema is not supported")
        sandbox = Sandbox(policy["sandbox"])
        sandbox.verify()
        return sandbox
    except (OSError, ValueError, json.JSONDecodeError, SandboxError) as error:
        raise Stop("roadmap execution requires the reviewed isolated runtime: " + str(error)) from error


def clear_sandbox_outputs(stage: Path) -> None:
    """Remove disposable command output before validating the candidate source delta."""
    for name in SANDBOX_OUTPUT_DIRS:
        path = stage / name
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.exists():
            shutil.rmtree(path)


def staged_changes(before: dict[str, tuple[str, int]], stage: Path) -> list[str]:
    after = agent_source_state(stage)
    return sorted(path for path in before.keys() | after.keys() if before.get(path) != after.get(path))


def allow_rules(task: dict | None, extra_tools: tuple[str, ...] = ()) -> list[str]:
    rules = ["Read", "Glob", "Grep", "Skill", *extra_tools]
    rules += [f"Bash({c}:*)" for c in READ_GIT]
    rules += [f"mcp__{s}__{t}" for s in GRAPHIFY_SERVERS for t in GRAPHIFY_READ]
    for command in (task or {}).get("checks", []) + (task or {}).get("worker_commands", []):
        rules.append(f"Bash({command}:*)")
        rules.append(f"Bash(python3 -B tools/tokenguard.py run -- {command}:*)")
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


def failure_fingerprint(kind: str, text: str) -> str:
    lines = [re.sub(r"\d+(\.\d+)?(ms|s)\b", "<t>", l) for l in text.splitlines()[-20:]]
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

def atomic_write(path: Path, data: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + f".tmp{os.getpid()}")
    tmp.write_text(data, encoding="utf-8")
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


def git(*args: str, check: bool = True) -> str:
    done = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if check and done.returncode:
        raise Stop(f"git {' '.join(args[:3])} failed ({done.returncode}): {done.stderr.strip()[-800:]}")
    return done.stdout


class Controller:
    def __init__(self, cfg: Config, state: Path = STATE, sandbox: Sandbox | None = None):
        self.cfg = cfg
        self.state = state
        self.sandbox = sandbox
        self.sandbox_verified = False
        self.lock_file = None
        self.spent = 0.0
        self.run_id = dt.datetime.now().strftime("%Y%m%dT%H%M%S") + f"-{os.getpid()}"

    def ensure_isolation(self) -> Sandbox:
        """Fail before repository inspection if controller callbacks or isolation are unsafe."""
        if self.sandbox_verified:
            assert self.sandbox is not None
            return self.sandbox
        try:
            verify_controller()
            if self.sandbox is None:
                self.sandbox = load_sandbox()
            else:
                self.sandbox.verify()
        except (TaskError, ValueError, OSError, subprocess.SubprocessError, SandboxError) as error:
            raise Stop("roadmap isolation preflight failed: " + str(error)) from error
        self.sandbox_verified = True
        return self.sandbox

    @contextlib.contextmanager
    def isolated_workspace(self):
        sandbox = self.ensure_isolation()
        try:
            with sandbox.lease() as lease:
                host_tree = self.tree_fingerprint()
                paths = agent_source_paths(self.changed())
                selected = agent_source_selection_state(ROOT, paths)
                stage_agent_source(
                    ROOT, lease.path, paths,
                    max_bytes=sandbox.scratch["source_max_bytes"],
                    max_files=sandbox.scratch["source_max_files"])
                if (agent_source_selection_state(ROOT, paths) != selected
                        or agent_source_selection_state(lease.path, paths) != selected
                        or self.tree_fingerprint() != host_tree):
                    raise Stop("controller source changed while staging an isolated workspace")
                yield lease, agent_source_state(lease.path), host_tree
        except Stop:
            raise
        except (OSError, ValueError, SandboxError) as error:
            raise Stop("isolated workspace failed: " + str(error)) from error

    # --- locking and lifecycle ---
    def acquire(self) -> None:
        self.ensure_isolation()
        self.state.mkdir(parents=True, exist_ok=True)
        self.lock_file = (self.state / "controller.lock").open("a+")
        try:
            fcntl.flock(self.lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise Stop("another controller holds build/roadmap-pipeline/controller.lock") from error
        self.lock_file.seek(0)
        self.lock_file.truncate()
        self.lock_file.write(json.dumps({"pid": os.getpid(), "run": self.run_id, "started": now()}))
        self.lock_file.flush()
        self.recover()

    def release(self) -> None:
        with contextlib.suppress(FileNotFoundError):
            (self.state / "lease.json").unlink()
        if self.lock_file:
            fcntl.flock(self.lock_file, fcntl.LOCK_UN)
            self.lock_file.close()
            self.lock_file = None

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

    def check_controls(self) -> None:
        if (self.state / "STOP").exists():
            raise Stop("stop file present (build/roadmap-pipeline/STOP)")
        while (self.state / "PAUSE").exists():
            self.heartbeat(paused=True)
            time.sleep(30)
            if (self.state / "STOP").exists():
                raise Stop("stop file present while paused")

    def preflight(self) -> str:
        self.ensure_isolation()
        branch = git("branch", "--show-current").strip()
        if branch != self.cfg.branch:
            raise Stop(f"checkout is on {branch!r}, expected {self.cfg.branch!r}")
        dirty = git("status", "--porcelain", "--untracked-files=all").strip()
        if dirty:
            raise Stop(f"checkout is not clean; refusing to start a writer:\n{dirty[:2000]}")
        alive = subprocess.run(["pgrep", "-f", f"--name {PROCESS_TAG}"],
                               capture_output=True, text=True).stdout.split()
        if [p for p in alive if int(p) != os.getpid()]:
            raise Stop(f"a previous worker process is still alive: {alive}")
        return git("rev-parse", "HEAD").strip()

    # --- one fresh headless session ---
    def session(self, kind: str, task_id: str, attempt_dir: Path, prompt: str, schema: dict,
                rules: list[str], mode: str, timeout: int, lease,
                disallowed: str | None = None, budget: float | None = None,
                effort: str | None = None, read_only: bool = False) -> dict:
        if self.spent >= self.cfg.total_budget_usd:
            raise Stop(f"cumulative estimated cost {self.spent:.2f} USD reached the cap")
        sandbox = self.ensure_isolation()
        session_id = str(uuid.uuid4())
        settings = attempt_dir / f"{kind}-settings.json"
        settings_text = json.dumps({"permissions": {"allow": rules, "deny": DENY}}, indent=2)
        atomic_write(settings, settings_text)
        brief = attempt_dir / f"{kind}-brief.md"
        atomic_write(brief, prompt)
        stage_settings = Path("build") / "roadmap-session" / f"{kind}-{session_id}-settings.json"
        staged = lease.path / stage_settings
        staged.parent.mkdir(parents=True, exist_ok=True)
        staged.write_text(settings_text, encoding="utf-8")
        sandbox_settings = Path("/workspace") / stage_settings
        argv = claude_argv(self.cfg, f"{PROCESS_TAG}{kind}-{task_id}", session_id, sandbox_settings,
                           schema, mode, budget or self.cfg.session_budget_usd, disallowed, effort)
        atomic_write(attempt_dir / f"{kind}-argv.json", json.dumps(argv[:1] + argv[1:], indent=1))
        out, err = attempt_dir / f"{kind}-result.json", attempt_dir / f"{kind}-stderr.log"
        started = time.monotonic()
        self.heartbeat(task=task_id, phase=kind, worker_pid=None, session_id=session_id,
                       isolation="sandbox")
        stdout_bytes = stderr_bytes = b""
        stop_reason = None
        resource = {}
        try:
            done = sandbox.run([*argv, prompt], lease, profile="agent",
                               timeout=min(timeout, sandbox.timeout), read_only=read_only)
            code = done.returncode
            stdout_bytes, stderr_bytes = done.stdout_bytes, done.stderr_bytes
            resource = resource_evidence(done)
        except subprocess.TimeoutExpired as error:
            code = 124
            stop_reason = f"timeout after {error.timeout}s"
            stdout_bytes = getattr(error, "stdout_bytes", b"")
            stderr_bytes = getattr(error, "stderr_bytes", b"")
            resource = resource_evidence(error)
        except SandboxResourceExceeded as error:
            code = GATE_FAIL_EXIT
            stop_reason = str(error)
            stdout_bytes = getattr(error, "stdout_bytes", b"")
            stderr_bytes = getattr(error, "stderr_bytes", b"")
            resource = resource_evidence(error)
        except SandboxError as error:
            raise Stop(f"isolated {kind} session failed: {error}") from error
        finally:
            self.heartbeat(worker_pid=None)
        out.write_bytes(stdout_bytes)
        err.write_bytes(stderr_bytes)
        text = stdout_bytes.decode("utf-8", "replace")
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
            "brief_sha256": sha256(prompt),
            "transcript": {"host_transcript_retained": False, "fresh_session_id": session_id,
                           "private_sandbox_home": True, "single_prompt_sha256": sha256(prompt)},
            "resource": resource,
            "output": structured(result),
        }
        atomic_write(attempt_dir / f"{kind}-record.json", json.dumps(record, indent=2))
        blob = text + stderr_bytes.decode("utf-8", "replace")
        if code != 0 and RATE_LIMIT.search(blob):
            raise Stop(f"{kind} session hit a rate or usage limit; see {err}")
        return record

    # --- validation owned by the controller ---
    def tree_fingerprint(self) -> str:
        return sha256(git("status", "--porcelain", "--untracked-files=all") + git("diff", "HEAD"))

    def changed(self) -> list[str]:
        return parse_porcelain(git("status", "--porcelain", "--untracked-files=all"))

    def revert(self, paths: list[str]) -> None:
        """Only for paths the worker created or edited in a checkout that was clean before."""
        tracked = set(git("ls-files", "--", *paths).split()) if paths else set()
        for path in paths:
            if path in tracked:
                git("checkout", "HEAD", "--", path, check=False)
            else:
                target = (ROOT / path).resolve()
                if ROOT in target.parents and target.exists():
                    shutil.rmtree(target) if target.is_dir() else target.unlink()

    def sandbox_command(self, lease, script: str, *, env: dict[str, str] | None = None,
                        timeout: int | None = None, output_limit: int | None = None,
                        read_only: bool = True):
        sandbox = self.ensure_isolation()
        deadline = min(timeout or self.cfg.check_timeout, sandbox.timeout)
        try:
            done = sandbox.run(
                ["/bin/bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
                lease, profile="gate", env=env, timeout=deadline, read_only=read_only,
                output_limit=output_limit)
            return (done.returncode, done.stdout_bytes, done.stderr_bytes,
                    resource_evidence(done), "")
        except subprocess.TimeoutExpired as error:
            return (124, getattr(error, "stdout_bytes", b""),
                    getattr(error, "stderr_bytes", b""), resource_evidence(error),
                    f"timeout after {error.timeout}s")
        except SandboxResourceExceeded as error:
            return (GATE_FAIL_EXIT, getattr(error, "stdout_bytes", b""),
                    getattr(error, "stderr_bytes", b""), resource_evidence(error), str(error))
        except (SandboxError, ValueError, OSError) as error:
            return GATE_FAIL_EXIT, b"", b"", resource_evidence(error), f"sandbox blocked: {error}"

    @staticmethod
    def write_check_log(path: Path, stdout: bytes, stderr: bytes, detail: str = "") -> None:
        payload = stdout
        if stderr:
            payload += (b"\n" if payload and not payload.endswith(b"\n") else b"") + stderr
        if detail:
            payload += (b"\n" if payload and not payload.endswith(b"\n") else b"")
            payload += detail.encode("utf-8", "replace") + b"\n"
        path.write_bytes(payload)

    def run_gate_plan(self, plan, lease, log: Path) -> tuple[int, str, list[dict]]:
        sandbox = self.ensure_isolation()
        remaining_output = sandbox.profiles["gate"]["output_max_bytes"]
        started = time.monotonic()
        records: list[dict] = []
        chunks: list[bytes] = []
        host = {key: os.environ[key] for key in SANDBOX_HOST_KEYS if key in os.environ}

        def execute(index: int, command, *, finalizer: bool, status: str) -> bool:
            nonlocal remaining_output
            name, script, step_env = command
            values = {key: value.replace("${GITHUB_WORKSPACE}", "/workspace")
                      for key, value in step_env.items()}
            values.update(host)
            if finalizer:
                script = f"export CI_TEST_STATUS={status}\n{script}"
            timeout = self.cfg.check_timeout
            if plan.timeout_seconds is not None:
                reserve = plan.finalizer_reserve_seconds if not finalizer else 0
                available = plan.timeout_seconds - (time.monotonic() - started) - reserve
                if available < 1:
                    detail = f"total gate deadline exhausted before {name}"
                    records.append({"name": name, "exit_code": 124, "detail": detail})
                    chunks.append((detail + "\n").encode())
                    return False
                timeout = min(int(available), plan.finalizer_reserve_seconds) \
                    if finalizer else int(available)
            code, stdout, stderr, resource, detail = self.sandbox_command(
                lease, script, env=values, timeout=timeout, output_limit=remaining_output)
            retained = stdout + stderr
            remaining_output = max(0, remaining_output - len(retained))
            label = "finalizer" if finalizer else "step"
            chunks.append(f"## {label}: {name}\n".encode() + retained)
            if detail:
                chunks.append(("\n## BLOCKED: " + detail + "\n").encode())
            records.append({"name": name, "exit_code": code, "resource": resource})
            return code == 0 and resource.get("Result") in (None, "success")

        failed = False
        for index, command in enumerate(plan.commands):
            if not execute(index, command, finalizer=False, status="success"):
                failed = True
                break
        primary_failed = failed
        status = "failure" if primary_failed else "success"
        for offset, command in enumerate(plan.finalizers, len(plan.commands)):
            if not execute(offset, command, finalizer=True, status=status):
                failed = True
        log.write_bytes(b"".join(chunks))
        if failed:
            tail = "\n".join(log.read_text(errors="replace").splitlines()[-120:])
            return GATE_FAIL_EXIT, tail, records
        return 0, "", records

    def run_checks(self, task: dict, attempt_dir: Path, lease) -> tuple[bool, list[dict], str]:
        results, failure = [], ""
        lock = (self.state / "qemu.lock").open("a+") if task.get("native") else None
        if lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            commands = [(f"check-{i}", c) for i, c in enumerate(task["checks"], 1)]
            commands.append(("progress", "python3 tools/progress.py --check"))
            for name, command in commands:
                self.heartbeat(phase=f"check:{name}")
                log = attempt_dir / f"{name}.log"
                started = time.monotonic()
                env = {key: os.environ[key] for key in SANDBOX_HOST_KEYS if key in os.environ}
                code, stdout, stderr, resource, detail = self.sandbox_command(
                    lease, command, env=env)
                self.write_check_log(log, stdout, stderr, detail)
                results.append({"name": name, "command": command, "exit_code": code,
                                "duration_s": round(time.monotonic() - started, 1),
                                "log_sha256": sha256(log.read_bytes()), "resource": resource})
                if code:
                    failure = f"{command} exited {code}:\n" + "\n".join(
                        log.read_text(errors="replace").splitlines()[-120:])
                    return False, results, failure
            if task.get("gates"):
                log = attempt_dir / "gate.log"
                try:
                    commands = resolve_gate(task["gates"], ROOT / ".github" / "workflows")
                except (KeyError, ValueError) as error:
                    failure = f"gate blocked: {error}"
                    log.write_text(failure + "\n")
                    code = GATE_FAIL_EXIT
                    gate_records = []
                else:
                    code, failure, gate_records = self.run_gate_plan(commands, lease, log)
                results.append({"name": "gates", "command": " ".join(task["gates"]), "exit_code": code,
                                "log_sha256": sha256(log.read_bytes()), "steps": gate_records})
                if code:
                    return False, results, failure or "\n".join(
                        log.read_text(errors="replace").splitlines()[-120:])
            return True, results, ""
        finally:
            if lock:
                fcntl.flock(lock, fcntl.LOCK_UN)
                lock.close()

    # --- prompts ---
    def worker_prompt(self, task: dict, head: str, feedback: str) -> str:
        fields = {k: task.get(k) for k in ("id", "criterion", "milestone", "title", "goal", "why_now",
                                           "write_paths", "read_paths", "graph_notes", "checks", "acceptance",
                                           "out_of_scope", "non_claimable", "risk", "evidence_level")}
        text = f"""roadmap-pipeline-task {task['id']} (run {self.run_id})

You are a FRESH worker session of the BaryonOS H/U/N/B/A/P/L/R roadmap pipeline. You
have no earlier conversation; this brief, AGENTS.md (already loaded through CLAUDE.md)
and the repository are your only context. Implement exactly ONE cohesive, verifiable
slice described below, then stop.

Repository: {ROOT} on branch {self.cfg.branch} at {head}. {"The working tree holds the previous attempt's uncommitted change; repair it." if feedback else "The checkout was clean when you started."}

Task (immutable; from docs/roadmap-pipeline/ledger.json):
{json.dumps(fields, indent=2, ensure_ascii=False)}

Rules (binding, in addition to AGENTS.md):
- Write ONLY inside write_paths. Never edit .github/, tools/agent_*.py, tools/common.py,
  tools/progress.py, tools/roadmap_runner.py, AGENTS.md, CLAUDE.md, docs/progress.json,
  docs/PROGRESS.md, docs/PROJECT-STATE.md, docs/DECISIONS.md, README.md or
  docs/roadmap-pipeline/. The controller reverts anything else and records it.
- Do not run git commands that change state, do not commit, push, fetch or use gh.
  Read-only git (status/diff/log/show) is allowed. No network access is needed.
- Graphify first: if Graphify MCP tools are available, call recall with your write
  paths and use graphify_find/impact/tests_for/file_neighbors before editing; the graph
  reflects main, so confirm against current files. If unavailable, read code directly
  and say so. Do not call remember (the lead records durable notes).
- Load the `ponytail` skill (full) if available: smallest safe change, reuse existing
  utilities, no reduced functionality or test evidence. Use relevant
  `verification-loop` guidance and `.agents/skills/os-dev` only where it applies.
- You may run exactly the task's check commands (optionally through
  `python3 -B tools/tokenguard.py run -- <command>`), plus read-only git. Other commands
  are denied automatically; that is expected, not an error to work around.
- Security Floor: no new trust roots, key escrow, telemetry, cloud fallback, broader
  capabilities or relaxed verification. Do not invent cryptography. Validate untrusted
  lengths and pointers. Add negative tests for every new boundary.
- Public text, comments and docs in professional, impersonal English. Label evidence
  levels honestly (host test, native QEMU, physical hardware, human study, external
  audit). Never claim a check passed unless you ran it and saw exit code 0; the
  controller re-runs every check independently anyway.
- Do not change docs/progress.json or award acceptance; report what evidence exists.
- If the task is not safely implementable as specified, stop and report state
  "blocked" with the precise blocker instead of improvising a different task.
"""
        if feedback:
            text += f"\nThe previous attempt was rejected. Resolve ONLY these concrete findings:\n{feedback}\n"
        text += "\nFinish with the structured result (JSON schema enforced): no log dumps, no false success.\n"
        return text

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
6. Policy probes. (a) Run `python3 -B tools/progress.py --check`; it is explicitly
   allowed. (b) Then try exactly `python3 -c "open('build/roadmap-pipeline/DENY-PROBE','w').write('x')"`;
   it is not allowed and should be refused without a prompt. Report both outcomes
   (put (b) in bash_denied_example and (a) in notes). Do not try any workaround.
Return the structured result.
"""

    # --- task lifecycle ---
    def run_task(self, task: dict) -> str:
        head = self.preflight()
        # The ledger is not touched while a worker runs: 'active' is the lease itself.
        self.heartbeat(task=task["id"], phase="start", base=head)
        task_dir = self.state / "runs" / self.run_id / task["id"]
        feedback, last_fp, attempts = "", None, []
        for attempt in range(1, self.cfg.max_repairs + 2):
            self.check_controls()
            adir = task_dir / f"attempt-{attempt}"
            adir.mkdir(parents=True, exist_ok=True)
            checkpoint_changed: list[str] | None = None
            with self.isolated_workspace() as (lease, staged_before, host_tree):
                worker = self.session(
                    "worker", task["id"], adir, self.worker_prompt(task, head, feedback),
                    WORKER_SCHEMA, allow_rules(task, ("Edit", "Write", "MultiEdit")),
                    "acceptEdits", self.cfg.worker_timeout, lease,
                    effort=effort_for(self.cfg, task))
                clear_sandbox_outputs(lease.path)
                worker_delta = staged_changes(staged_before, lease.path)
                if self.tree_fingerprint() != host_tree:
                    raise Stop("controller source changed while the isolated worker was active")
                record = {"attempt": attempt, "worker": worker, "worker_delta": worker_delta}
                attempts.append(record)
                existing = self.changed()
                bad = [path for path in worker_delta if not in_write_set(path, task["write_paths"])]
                out = worker.get("output") or {}
                if bad:
                    record["discarded_out_of_scope"] = bad
                    kind, detail = "scope", f"worker changed paths outside the frozen write set: {bad}"
                elif worker["exit_code"] != 0 or worker["is_error"]:
                    kind = "session"
                    detail = (f"worker session failed (exit {worker['exit_code']}, {worker['subtype']}, "
                              f"{worker.get('stopped_by_controller') or 'no controller stop'})")
                elif out.get("state") == "blocked":
                    self.revert(existing)
                    raise Blocked("worker reported blocked: "
                                  + "; ".join(out.get("blockers") or ["(no detail)"]))
                elif not worker_delta and not existing:
                    kind, detail = "no-change", "no in-scope file was changed"
                else:
                    ok, checks, failure = self.run_checks(task, adir, lease)
                    record["checks"] = checks
                    clear_sandbox_outputs(lease.path)
                    if staged_changes(staged_before, lease.path) != worker_delta:
                        raise Stop("an isolated read-only check changed candidate source")
                    if self.tree_fingerprint() != host_tree:
                        raise Stop("controller source changed while isolated checks were active")
                    if not ok:
                        kind, detail = "checks", failure
                    else:
                        if worker_delta:
                            imported = import_agent_changes(
                                lease.path, ROOT, staged_before, worker_delta,
                                max_bytes=self.sandbox.scratch["export_max_bytes"],
                                max_files=self.sandbox.scratch["export_max_files"])
                            if imported != worker_delta:
                                raise Stop("validated worker delta changed during import")
                        changed = self.changed()
                        diff = git("diff", "HEAD")
                        tracked = set(git("ls-files", "--", *changed).split())
                        untracked = [path for path in changed if path not in tracked]
                        before = self.tree_fingerprint()
                        verifier = self.session(
                            "verifier", task["id"], adir,
                            self.verifier_prompt(task, head, diff[:60_000], untracked, checks),
                            VERIFIER_SCHEMA, allow_rules(None), "default",
                            self.cfg.verifier_timeout, lease,
                            disallowed=READ_ONLY_DISALLOWED, budget=self.cfg.verifier_budget_usd,
                            effort=effort_for(self.cfg, task), read_only=True)
                        record["verifier"] = verifier
                        clear_sandbox_outputs(lease.path)
                        if staged_changes(staged_before, lease.path) != worker_delta:
                            raise Stop("the isolated read-only verifier changed candidate source")
                        if self.tree_fingerprint() != before:
                            raise Stop("the read-only verifier changed controller source")
                        verdict = verifier.get("output") or {}
                        findings = verdict.get("findings") or []
                        if (verifier["exit_code"] == 0 and verdict.get("verdict") == "pass"
                                and not blocking(findings)):
                            checkpoint_changed = changed
                        else:
                            kind = "verifier"
                            detail = json.dumps(
                                blocking(findings) or findings or verdict or verifier.get("subtype"), indent=1)
            if checkpoint_changed is not None:
                return self.checkpoint(task, head, checkpoint_changed, attempts)
            if bad := record.get("discarded_out_of_scope"):
                detail += f"\nOut-of-scope edits stayed inside the disposable workspace: {bad}"
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
        try:
            verify_controller()
        except (TaskError, ValueError, OSError, subprocess.SubprocessError) as error:
            raise Stop("controller publication preflight failed: " + str(error)) from error
        if sorted(self.changed()) != sorted(changed):
            raise Stop("controller source changed after isolated verification")
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
        git("commit", "-q", "-m", message)
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
        views = ["docs/progress.json", "README.md", "docs/PROGRESS.md", "docs/assets/progress.svg"]
        generated = set(views[1:])
        with self.isolated_workspace() as (lease, staged_before, host_tree):
            code, stdout, stderr, _resource, detail = self.sandbox_command(
                lease, "python3 -I -S -B tools/progress.py --write", read_only=False)
            clear_sandbox_outputs(lease.path)
            delta = staged_changes(staged_before, lease.path)
            if code:
                output = (stdout + stderr).decode("utf-8", "replace")
                raise Stop(f"isolated progress.py --write failed ({code}): "
                           + (detail or output[-800:]))
            if set(delta) - generated:
                raise Stop(f"progress renderer changed unexpected paths: {delta}")
            if self.tree_fingerprint() != host_tree:
                raise Stop("controller source changed while progress views were rendered")
            code, stdout, stderr, _resource, detail = self.sandbox_command(
                lease, "python3 -I -S -B tools/progress.py --check")
            clear_sandbox_outputs(lease.path)
            if staged_changes(staged_before, lease.path) != delta:
                raise Stop("read-only progress validation changed rendered source")
            if code:
                output = (stdout + stderr).decode("utf-8", "replace")
                raise Stop(f"isolated progress.py --check failed ({code}): "
                           + (detail or output[-800:]))
            if delta:
                imported = import_agent_changes(
                    lease.path, ROOT, staged_before, delta,
                    max_bytes=self.sandbox.scratch["export_max_bytes"],
                    max_files=self.sandbox.scratch["export_max_files"])
                if imported != delta:
                    raise Stop("rendered progress delta changed during import")
        git("add", "--", *views)
        if git("diff", "--cached", "--name-only").strip():
            git("commit", "-q", "-m", f"progress: assess roadmap task {task['id']} ({score}/100 unchanged)\n\n"
                f"Assessment of {commit}; no criterion changed.\n\n" + TRAILERS)

    def push(self) -> None:
        if not self.cfg.push:
            return
        for delay in (0, 2, 4, 8, 16):
            time.sleep(delay)
            done = subprocess.run(["git", "push", "-q", "-u", "origin", self.cfg.branch], cwd=ROOT,
                                  capture_output=True, text=True)
            if done.returncode == 0:
                return
        raise Stop(f"push failed: {done.stderr.strip()[-600:]}")

    def mark(self, task_id: str, status: str, reason: str) -> None:
        update_task(task_id, status=status, history={"at": now(), "event": status, "run": self.run_id,
                                                     "reason": reason})
        git("add", "--", str(LEDGER.relative_to(ROOT)))
        git("commit", "-q", "-m", f"roadmap({task_id}): mark {status}\n\n{reason[:900]}\n\n" + TRAILERS)
        task = next(t for t in load_ledger()["tasks"] if t["id"] == task_id)
        self.record_assessment(task, git("rev-parse", "HEAD").strip(), status)
        self.push()

    def init_check(self) -> dict:
        head = self.preflight()
        adir = self.state / "runs" / self.run_id / "INIT-CHECK"
        adir.mkdir(parents=True, exist_ok=True)
        probe = ROOT / INIT_DENY_PROBE
        probe.unlink(missing_ok=True)
        with self.isolated_workspace() as (lease, staged_before, host_tree):
            record = self.session(
                "init", "INIT-CHECK", adir, self.init_prompt(), INIT_SCHEMA,
                allow_rules(None) + [f"Bash({INIT_ALLOWED_PROBE}:*)"], "default", 15 * 60, lease,
                disallowed=READ_ONLY_DISALLOWED, budget=self.cfg.verifier_budget_usd,
                read_only=True)
            clear_sandbox_outputs(lease.path)
            if staged_changes(staged_before, lease.path):
                raise Stop("the isolated initialization check changed candidate source")
            if self.tree_fingerprint() != host_tree:
                raise Stop("controller source changed during the isolated initialization check")
        record["head_expected"] = head
        record["checkout_clean_after"] = not self.changed()
        record["deny_probe_file_absent"] = not probe.exists()  # controller-side oracle
        probe.unlink(missing_ok=True)
        atomic_write(adir / "init-check.json", json.dumps(record, indent=2))
        return record

    def loop(self, max_tasks: int, init: bool) -> str:
        log = self.state / "runs" / self.run_id / "controller.jsonl"
        log.parent.mkdir(parents=True, exist_ok=True)

        def event(**fields):
            with log.open("a") as out:
                out.write(json.dumps({"at": now(), **fields}) + "\n")
            print(json.dumps({"at": now(), **fields}), flush=True)

        if init:
            record = self.init_check()
            event(event="init-check", session=record["session_id"], exit=record["exit_code"],
                  transcript=record["transcript"], output=record["output"])
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


def main() -> int:
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
    for name in ("status", "stop", "pause", "resume", "validate"):
        sub.add_parser(name)
    args = parser.parse_args()
    STATE.mkdir(parents=True, exist_ok=True)
    if args.cmd == "status":
        return status()
    if args.cmd == "validate":
        load_ledger()
        print("ledger ok")
        return 0
    if args.cmd in ("stop", "pause"):
        (STATE / args.cmd.upper()).write_text(now())
        print(f"{args.cmd} requested; the controller honours it before the next session")
        return 0
    if args.cmd == "resume":
        for name in ("STOP", "PAUSE"):
            (STATE / name).unlink(missing_ok=True)
        print("stop/pause cleared")
        return 0
    cfg = Config(push=not args.no_push, total_budget_usd=args.total_budget_usd,
                 session_budget_usd=args.session_budget_usd, model=args.model, effort=args.effort,
                 routine_effort=args.routine_effort)
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
