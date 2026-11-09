#!/usr/bin/env python3
"""Agent-pipeline gate: run the exact `run:` steps of chosen CI jobs, then progress.py --check.

Pass/fail is only the raw exit code of each step (bash -eo pipefail). `rtk`, when
installed, only condenses the failure log that goes back to an agent. This is a local
pre-filter: GitHub CI and review on the PR remain the real boundary.
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import ROOT, environment

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
    """Return (name, script, env) for every check step of one workflow job."""
    import yaml  # PyYAML: on the ubuntu runners and the dev host; only the gate needs it

    spec = yaml.safe_load(workflow.read_text(encoding="utf-8"))["jobs"][job]
    job_env = _env(spec.get("env"), f"{workflow.name}#{job}")
    steps = []
    for step in spec.get("steps", []):
        run = step.get("run")
        if not run or "uses" in step or "if" in step or any(m in run for m in SETUP_MARKERS):
            continue
        name = str(step.get("name") or run.splitlines()[0])
        steps.append((name, _expand(run, f"{workflow.name}#{job}/{name}"),
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
             base_env: dict[str, str] | None = None) -> int:
    """Run steps in order, stop at the first failure. 0 = pass, 2 = fail."""
    env = environment() if base_env is None else dict(base_env)
    host = {k: env[k] for k in HOST_KEYS if k in env}
    for key in CHECKOUT_KEYS:
        env.pop(key, None)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w", encoding="utf-8") as log:
        for name, script, step_env in commands:
            log.write(f"## step: {name}\n")
            log.flush()
            # Env values may reference ${GITHUB_WORKSPACE}; bash expands it in the script,
            # so values are expanded here the same way.
            step_env = {k: v.replace("${GITHUB_WORKSPACE}", str(cwd)) for k, v in step_env.items()}
            code = subprocess.call(["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
                                   cwd=cwd, env={**env, **step_env, **host, "GITHUB_WORKSPACE": str(cwd)},
                                   stdout=log, stderr=subprocess.STDOUT)
            if code:
                log.write(f"## FAIL: {name} (exit {code})\n")
                return FAIL_EXIT
        log.write("## PASS\n")
    return 0


def condensed(log_path: Path, lines: int = 200) -> str:
    """Failure log for the agent: rtk-filtered when available, else the last lines."""
    rtk = shutil.which("rtk")
    text = None
    if rtk:
        done = subprocess.run([rtk, "log", str(log_path)], capture_output=True, text=True)
        text = done.stdout if done.returncode == 0 and done.stdout.strip() else None
    if text is None:
        text = log_path.read_text(encoding="utf-8", errors="replace")
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
