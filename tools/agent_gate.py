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
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from agent_sandbox import Sandbox, SandboxError, resource_evidence
from common import ROOT, agent_source_paths, agent_source_state, isolated_environment, stage_agent_source

# Steps that install the runner environment; locally tools/common.py provides it.
SETUP_MARKERS = ("rustup ", "apt-get", "sudo ")
PROGRESS_CHECK = ("progress.py --check", "python3 tools/progress.py --check", {})
FAIL_EXIT = 2


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


def resolve(refs: list[str], workflows: Path) -> list[tuple[str, str, dict[str, str]]]:
    """Turn `ci.yml#host-quality` refs into steps; workflows come from the trusted checkout."""
    commands = []
    for ref in refs:
        name, sep, job = ref.partition("#")
        path = (workflows / name).resolve()
        if not sep or not job or path.parent != workflows.resolve() or not path.is_file():
            raise ValueError(f"unknown gate reference: {ref!r} (expected <workflow>.yml#<job>)")
        commands += job_steps(path, job)
    return commands + [PROGRESS_CHECK]


def run_gate(commands: list[tuple[str, str, dict[str, str]]], cwd: Path, log_path: Path,
             base_env: dict[str, str] | None = None, *, sandbox: Sandbox | None = None,
             source_paths: list[str] | None = None) -> int:
    """Run steps in order, stop at the first failure. 0 = pass, 2 = fail."""
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
            stage = Path(tempfile.mkdtemp(prefix="gate-source-", dir=log_path.parent))
            stage_agent_source(cwd, stage, paths)
            fingerprint = hashlib.sha256(json.dumps(agent_source_state(stage), sort_keys=True).encode()).hexdigest()
            log.write(f"## source SHA256: {fingerprint}\n## retained isolated outputs: {stage.name}\n")
            for index, (name, script, step_env) in enumerate(commands):
                log.write(f"## step: {name}\n")
                log.flush()
                values = {k: v.replace("${GITHUB_WORKSPACE}", "/workspace") for k, v in step_env.items()}
                env = isolated_environment({**values, **host})
                done = sandbox.run(["/bin/bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
                                   stage, profile="gate", env=env, read_only=True)
                for stream in ("stdout", "stderr"):
                    raw = getattr(done, stream + "_bytes", getattr(done, stream).encode())
                    log_path.with_name(f"{log_path.name}.step-{index}.{stream}.log").write_bytes(raw)
                log.write(f"## resource: {json.dumps(resource_evidence(done), sort_keys=True)}\n")
                log.write(done.stdout + done.stderr)
                log.write(f"\n## exit: {done.returncode}\n")
                if done.returncode:
                    log.write(f"## FAIL: {name} (exit {done.returncode})\n")
                    return FAIL_EXIT
            log.write("## PASS\n")
        except subprocess.TimeoutExpired as error:
            for stream in ("stdout", "stderr"):
                raw = getattr(error, stream + "_bytes", (getattr(error, stream) or "").encode())
                log_path.with_name(f"{log_path.name}.timeout.{stream}.log").write_bytes(raw)
            log.write(f"## resource: {json.dumps(resource_evidence(error), sort_keys=True)}\n"
                      f"{error.stdout or ''}\n{error.stderr or ''}\n## TIMEOUT: {error.timeout}s\n")
            return FAIL_EXIT
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
    commands = resolve(args.job, ROOT / ".github" / "workflows")
    if args.list:
        for name, script, _ in commands:
            print(f"## {name}\n{script.rstrip()}")
        return 0
    log = args.log or args.root / "build" / "agent-pipeline" / "gate.log"
    code = run_gate(commands, args.root.resolve(), log)
    print(f"gate: {'PASS' if code == 0 else 'FAIL'} ({len(commands)} steps); log: {log}")
    if code:
        print(condensed(log))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
