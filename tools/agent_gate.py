#!/usr/bin/env python3
"""Agent-pipeline gate: refuse unsupported CI steps, then run the modelled commands.

Pass/fail is only the raw exit code of each step (bash -eo pipefail). `rtk`, when
installed, only condenses the failure log that goes back to an agent. This is a local
pre-filter: GitHub CI and review on the PR remain the real boundary.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import time
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from agent_sandbox import Sandbox, SandboxError, SandboxResourceExceeded, resource_evidence
from common import (ROOT, SOURCE_RECEIPT_NAME, agent_source_paths,
                    agent_source_selection_state, agent_source_state,
                    controller_source_commit, isolated_environment, source_hash,
                    stage_agent_source)

# Steps that install the runner environment; locally tools/common.py provides it.
SETUP_MARKERS = ("rustup ", "apt-get", "sudo ")
PROGRESS_CHECK = ("progress.py --check", "python3 tools/progress.py --check", {})
FAIL_EXIT = 2

GateCommand = tuple[str, str, dict[str, str]]


@dataclass(frozen=True)
class GatePlan:
    """Commands sharing one deadline, plus bounded cleanup that always follows execution."""

    commands: tuple[GateCommand, ...]
    finalizers: tuple[GateCommand, ...] = ()
    timeout_seconds: int | None = None
    finalizer_reserve_seconds: int = 0

    def __post_init__(self) -> None:
        object.__setattr__(self, "commands", tuple(self.commands))
        object.__setattr__(self, "finalizers", tuple(self.finalizers))
        if (self.timeout_seconds is not None
                and (type(self.timeout_seconds) is not int or self.timeout_seconds < 1)):
            raise ValueError("gate timeout must be a positive integer")
        if (type(self.finalizer_reserve_seconds) is not int
                or self.finalizer_reserve_seconds < 0
                or len(self.finalizers) > 1
                or (self.finalizer_reserve_seconds and not self.finalizers)
                or (self.finalizers and (self.timeout_seconds is None
                                         or not self.finalizer_reserve_seconds))
                or (self.timeout_seconds is not None
                    and self.finalizer_reserve_seconds >= self.timeout_seconds)):
            raise ValueError("gate finalizer reserve is invalid")


WORKSPACE = "${{ github.workspace }}"
# Host firmware paths differ per machine; an exported local value wins over the runner path.
HOST_KEYS = ("OS_OVMF_CODE", "OS_OVMF_VARS")
# Build dirs belong to the checkout under test (as on CI), never to the caller's shell:
# parallel worktrees must not share a Cargo target dir.
CHECKOUT_KEYS = ("NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR")


def _expand(text: str, where: str) -> str:
    """Only `github.workspace` has a local meaning (the checkout); anything else is refused."""
    text = text.replace(WORKSPACE, "${GITHUB_WORKSPACE}")
    if "${{" in text:
        raise ValueError(f"{where}: needs GitHub Actions context")
    return text


def _env(mapping: object, where: str) -> dict[str, str]:
    return {str(k): _expand(str(v), where) for k, v in (mapping or {}).items()}


def job_steps(workflow: Path, job: str) -> list[tuple[str, str, dict[str, str]]]:
    """Return (name, script, env), refusing workflow semantics the gate cannot preserve."""
    import yaml  # PyYAML: on the ubuntu runners and the dev host; only the gate needs it

    spec = yaml.safe_load(workflow.read_text(encoding="utf-8"))["jobs"][job]
    job_env = _env(spec.get("env"), f"{workflow.name}#{job}")
    steps = []
    for step in spec.get("steps", []):
        name = str(step.get("name") or step.get("run") or step.get("uses") or "<unnamed>")
        where = f"{workflow.name}#{job}/{name}"
        if "uses" in step:
            raise ValueError(f"{where}: uses step needs an explicit local adapter")
        if "if" in step:
            raise ValueError(f"{where}: conditional step needs an explicit local adapter")
        run = step.get("run")
        if not isinstance(run, str) or not run.strip():
            raise ValueError(f"{where}: step has no runnable script")
        if any(marker in run for marker in SETUP_MARKERS):
            raise ValueError(f"{where}: setup step needs an explicit local adapter")
        steps.append((name, _expand(run, where),
                      {**job_env, **_env(step.get("env"), name)}))
    if not steps:
        raise ValueError(f"{workflow.name}#{job}: no runnable check steps")
    return steps


def resolve(refs: list[str], workflows: Path) -> GatePlan:
    """Turn `ci.yml#host-quality` refs into steps; workflows come from the trusted checkout."""
    commands = []
    for ref in refs:
        name, sep, job = ref.partition("#")
        path = (workflows / name).resolve()
        if not sep or not job or path.parent != workflows.resolve() or not path.is_file():
            raise ValueError(f"unknown gate reference: {ref!r} (expected <workflow>.yml#<job>)")
        commands += job_steps(path, job)
    return GatePlan(tuple(commands + [PROGRESS_CHECK]))


def run_gate(commands: GatePlan | list[GateCommand], cwd: Path, log_path: Path,
             base_env: dict[str, str] | None = None, *, sandbox: Sandbox | None = None,
             source_paths: list[str] | None = None) -> int:
    """Run one bounded job; always run its reserved finalizers. 0 = pass, 2 = fail."""
    plan = commands if isinstance(commands, GatePlan) else GatePlan(tuple(commands))
    started = time.monotonic()
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w", encoding="utf-8") as log:
        try:
            if sandbox is None:
                policy = json.loads((ROOT / ".github" / "agent-pipeline.json").read_text())
                if policy.get("schema") != 1 or policy.get("enabled") is not True:
                    raise SandboxError("isolated gate execution is not authorized")
                sandbox = Sandbox(policy.get("sandbox", {}))
            sandbox.verify()
            # Only firmware overrides have a meaning in the provisioned offline runtime.
            host = {k: v for k, v in (base_env or {}).items() if k in HOST_KEYS}
            paths = agent_source_paths() if source_paths is None else source_paths
            if any(Path(path).parts[0] in {"build", "target"} for path in paths):
                raise SandboxError("gate output mounts must not mask source inputs")
            commit = controller_source_commit(cwd)
            before = agent_source_selection_state(cwd, paths)
            stage = Path(tempfile.mkdtemp(prefix="gate-source-", dir=log_path.parent))
            stage_agent_source(cwd, stage, paths)
            if (agent_source_selection_state(cwd, paths) != before
                    or agent_source_selection_state(stage, paths) != before):
                raise SandboxError("gate source changed while staging")
            frozen = source_hash(stage)
            if controller_source_commit(cwd) != commit:
                raise SandboxError("source commit changed while staging the gate")
            receipt = stage / SOURCE_RECEIPT_NAME
            receipt.write_text(json.dumps({"schema": 1, "source_commit": commit,
                                           "source_sha256": frozen}, sort_keys=True) + "\n",
                               encoding="utf-8")
            receipt.chmod(0o444)
            fingerprint = hashlib.sha256(json.dumps(agent_source_state(stage), sort_keys=True).encode()).hexdigest()
            log.write(f"## source commit: {commit}\n## source inputs SHA256: {frozen}\n"
                      f"## staged source SHA256: {fingerprint}\n## retained isolated outputs: {stage.name}\n")

            def execute(index: int, command: GateCommand, *, finalizer: bool,
                        status: str) -> bool:
                name, script, step_env = command
                kind = "finalizer" if finalizer else "step"
                label = f"{kind}-{index}"
                log.write(f"## {kind}: {name}\n")
                log.flush()
                try:
                    values = {k: v.replace("${GITHUB_WORKSPACE}", "/workspace")
                              for k, v in step_env.items()}
                    values["NOS_SOURCE_RECEIPT"] = f"/workspace/{SOURCE_RECEIPT_NAME}"
                    env = isolated_environment({**values, **host})
                    if finalizer:
                        script = f"export CI_TEST_STATUS={status}\n{script}"
                    timeout = None
                    if plan.timeout_seconds is not None:
                        reserve = plan.finalizer_reserve_seconds if not finalizer else 0
                        remaining = plan.timeout_seconds - (time.monotonic() - started) - reserve
                        if remaining < 1:
                            log.write(f"## TIMEOUT: total gate deadline exhausted before {name}\n")
                            return False
                        timeout = min(int(remaining), plan.finalizer_reserve_seconds) \
                            if finalizer else int(remaining)
                        sandbox_limit = getattr(sandbox, "timeout", None)
                        if type(sandbox_limit) is int:
                            timeout = min(timeout, sandbox_limit)
                    kwargs = {"profile": "gate", "env": env, "read_only": True}
                    if timeout is not None:
                        kwargs["timeout"] = timeout
                    done = sandbox.run(
                        ["/bin/bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
                        stage, **kwargs)
                except subprocess.TimeoutExpired as error:
                    for stream in ("stdout", "stderr"):
                        raw = getattr(error, stream + "_bytes", (getattr(error, stream) or "").encode())
                        log_path.with_name(
                            f"{log_path.name}.{label}.timeout.{stream}.log").write_bytes(raw)
                    log.write(f"## resource: {json.dumps(resource_evidence(error), sort_keys=True)}\n"
                              f"{error.stdout or ''}\n{error.stderr or ''}\n"
                              f"## TIMEOUT: {error.timeout}s\n")
                    return False
                except SandboxResourceExceeded as error:
                    for stream in ("stdout", "stderr"):
                        raw = getattr(error, stream + "_bytes", None)
                        if isinstance(raw, bytes):
                            log_path.with_name(
                                f"{log_path.name}.{label}.resource.{stream}.log").write_bytes(raw)
                    log.write(f"## resource: {json.dumps(resource_evidence(error), sort_keys=True)}\n"
                              f"## BLOCKED: {error}\n")
                    return False
                except SandboxError:
                    raise
                except (ValueError, OSError) as error:
                    log.write(f"## BLOCKED: {error}\n")
                    return False
                for stream in ("stdout", "stderr"):
                    raw = getattr(done, stream + "_bytes", getattr(done, stream).encode())
                    log_path.with_name(f"{log_path.name}.{label}.{stream}.log").write_bytes(raw)
                evidence = resource_evidence(done)
                log.write(f"## resource: {json.dumps(evidence, sort_keys=True)}\n")
                log.write(done.stdout + done.stderr)
                log.write(f"\n## exit: {done.returncode}\n")
                resource_result = evidence.get("Result")
                if done.returncode or resource_result not in (None, "success"):
                    log.write(f"## FAIL: {name} (exit {done.returncode})\n")
                    return False
                return True

            failed = False
            for index, command in enumerate(plan.commands):
                if not execute(index, command, finalizer=False, status="success"):
                    failed = True
                    break
            primary_failed = failed
            status = "failure" if primary_failed else "success"
            for offset, command in enumerate(plan.finalizers, len(plan.commands)):
                if not execute(offset, command, finalizer=True, status=status):
                    if primary_failed:
                        log.write("## FINALIZER FAIL: primary gate failure preserved\n")
                    failed = True
            if failed:
                return FAIL_EXIT
            log.write("## PASS\n")
        except (SandboxError, ValueError, OSError) as error:
            if isinstance(error, SandboxError):
                for stream in ("stdout", "stderr"):
                    raw = getattr(error, stream + "_bytes", None)
                    if isinstance(raw, bytes):
                        log_path.with_name(f"{log_path.name}.resource.{stream}.log").write_bytes(raw)
                log.write(f"## resource: {json.dumps(resource_evidence(error), sort_keys=True)}\n")
            log.write(f"## BLOCKED: {error}\n")
            return FAIL_EXIT
    return 0


def condensed(log_path: Path, lines: int = 200) -> str:
    """Bounded presentation only; no ambient helper executes in the controller."""
    with log_path.open("rb") as raw:
        raw.seek(max(0, log_path.stat().st_size - 65536))
        text = raw.read(65536).decode("utf-8", "replace")
    return "\n".join(text.splitlines()[-lines:])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--job", action="append", default=[], metavar="WORKFLOW#JOB",
                        help="CI job whose run-steps form the gate, e.g. ci.yml#host-quality")
    parser.add_argument("--root", type=Path, default=ROOT, help="checkout to test (cwd of the steps)")
    parser.add_argument("--log", type=Path, help="full log (default: <root>/build/agent-pipeline/gate.log)")
    parser.add_argument("--list", action="store_true", help="print the resolved steps and exit")
    args = parser.parse_args()
    # Workflows are always read from this (trusted) checkout, never from --root.
    plan = resolve(args.job, ROOT / ".github" / "workflows")
    if args.list:
        for name, script, _ in plan.commands:
            print(f"## {name}\n{script.rstrip()}")
        return 0
    log = args.log or args.root / "build" / "agent-pipeline" / "gate.log"
    code = run_gate(plan, args.root.resolve(), log)
    count = len(plan.commands) + len(plan.finalizers)
    print(f"gate: {'PASS' if code == 0 else 'FAIL'} ({count} steps); log: {log}")
    if code:
        print(condensed(log))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
