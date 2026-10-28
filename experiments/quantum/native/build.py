#!/usr/bin/env python3
"""Build and inspect the scalar ring-3 qsim probe; this does not boot a VM."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from build_users import inspect_user  # noqa: E402


# GNU objdump can print several legacy/REX prefixes before the real opcode.
# A prefix is never an opcode verdict: e.g. `data16 fninit` still uses x87.
PREFIXES = frozenset({
    "addr16", "addr32", "data16", "data32", "lock", "rep", "repe", "repz",
    "repne", "repnz", "cs", "ds", "es", "ss", "fs", "gs", "bnd", "notrack",
})
CONDITIONS = "a ae b be c e g ge l le na nae nb nbe nc ne ng nge nl nle no np ns nz o p pe po s z".split()
SCALAR_OPCODES = frozenset({
    "adc", "add", "and", "bsf", "bsr", "bswap", "bt", "btc", "btr", "bts",
    "call", "cbw", "cdq", "cdqe", "clc", "cld", "cmc", "cmp", "cmps",
    "cmpxchg", "cmpxchg8b", "cmpxchg16b", "cqo", "cwd", "cwde", "dec", "div",
    "idiv", "imul", "inc", "int", "int3", "jmp", "jcxz", "jecxz", "jrcxz",
    "lahf", "lea", "leave", "lfence", "lods", "loop", "loope", "loopne",
    "mfence", "mov", "movabs", "movbe", "movs", "movsx", "movsxd", "movzx",
    "mul", "neg", "nop", "not", "or", "pause", "pop", "popf", "popfq",
    "push", "pushf", "pushfq", "rcl", "rcr", "ret", "rol", "ror", "sahf",
    "sal", "sar", "sbb", "scas", "sfence", "shl", "shld", "shr", "shrd",
    "stc", "std", "stos", "sub", "test", "ud2", "xadd", "xchg", "xlat",
    "xor",
} | {stem + condition for stem in ("j", "cmov", "set") for condition in CONDITIONS})


def decode_instruction(text: str, address: str) -> tuple[str, str]:
    """Peel known objdump prefixes, then require an explicit scalar opcode."""
    fields = text.split(None, 1)
    prefixes = 0
    while fields and (fields[0] in PREFIXES or re.fullmatch(r"rex(?:\.[WRXB]+)?", fields[0])):
        prefixes += 1
        # An x86 instruction is at most 15 bytes, including its real opcode.
        if prefixes > 14 or len(fields) != 2:
            raise ValueError(f"unparsed instruction at {address}: {text}")
        fields = fields[1].split(None, 1)
    if not fields or fields[0] not in SCALAR_OPCODES:
        raise ValueError(f"unsupported instruction at {address}: {text}")
    return fields[0], fields[1] if len(fields) == 2 else ""


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect_instructions(disassembly: str) -> dict:
    """Reject decoded extended-register instructions, not byte-pattern guesses.

    This is a bounded x86 scalar profile check, not a formal instruction proof.
    The emitted GNU objdump text is retained and tied to the ELF SHA256.
    """
    count = 0
    syscalls = 0
    for line in disassembly.splitlines():
        match = re.match(r"^\s*([0-9a-f]+):\s*(.*)$", line)
        if not match:
            if re.match(r"^\s*[0-9a-f]+:", line):
                raise ValueError(f"unparsed instruction: {line.strip()}")
            continue
        address, instruction = match.groups()
        mnemonic, operands = decode_instruction(instruction, address)
        count += 1
        forbidden_register = re.search(r"\b(?:[xyz]mm\d+|mm[0-7]|st\(?[0-7]?\)?|k[0-7]|bnd[0-3]|tmm[0-7]|cr\d+|dr\d+)\b", operands)
        if forbidden_register:
            raise ValueError(f"non-scalar instruction at {address}: {mnemonic} {operands}")
        if mnemonic == "int":
            if operands.strip() != "0x80":
                raise ValueError(f"unexpected interrupt at {address}")
            syscalls += 1
    if count == 0 or syscalls == 0:
        raise ValueError("empty code or missing native int80 boundary")
    return {"decoded_instructions": count, "int80_sites": syscalls, "extended_registers_found": False}


def sources() -> dict[str, str]:
    paths = [HERE / "Cargo.toml", HERE / "Cargo.lock", HERE / "linker.ld", HERE / "build.py"]
    paths.extend(sorted((HERE / "src").glob("*.rs")))
    paths.extend(sorted((HERE / "tests").glob("*.rs")))
    paths.extend(sorted((HERE / "tests").glob("*.py")))
    paths.extend(sorted((HERE.parent / "qsim-core/src").glob("*.rs")))
    paths.extend([HERE.parent / "qsim-core/Cargo.toml", HERE.parent / "Cargo.toml", ROOT / "tools/build_users.py"])
    return {str(path.relative_to(ROOT)): digest(path) for path in paths}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=HERE / "build")
    parser.add_argument("--target-dir", type=Path)
    args = parser.parse_args()
    output = args.output_dir.resolve()
    target = (args.target_dir or output / "target").resolve()
    output.mkdir(parents=True, exist_ok=True)
    report = {
        "schema_version": 1,
        "kind": "I01 native target compilation and scalar inspection",
        "native_runtime_performed": False,
        "passed": False,
        "commands": [],
        "source_sha256": sources(),
        "limitations": ["No VM run is performed by this script.", "Disassembly inspection is not formal verification."],
    }
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join([
        "-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes",
        "-C", "linker=rust-lld", "-C", f"link-arg=-T{HERE / 'linker.ld'}",
        "-C", "link-arg=--no-pie", "-C", "link-arg=--build-id=none",
        "-C", "link-arg=-zmax-page-size=0x1000",
    ])
    report["rustflags"] = env["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")

    def run(command: list[str], name: str) -> str:
        completed = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=180)
        (output / f"{name}.stdout.txt").write_text(completed.stdout)
        (output / f"{name}.stderr.txt").write_text(completed.stderr)
        report["commands"].append({"argv": command, "returncode": completed.returncode, "log": name})
        if completed.returncode:
            raise RuntimeError(f"{name} failed with exit {completed.returncode}: {completed.stderr}")
        return completed.stdout

    try:
        report["rustc"] = run(["rustc", "--version"], "rustc").strip()
        if not report["rustc"].startswith("rustc 1.90.0 "):
            raise RuntimeError("I01 requires the project's pinned Rust 1.90.0")
        run(["cargo", "build", "--manifest-path", str(HERE / "Cargo.toml"), "--locked", "--offline",
             "--release", "--features", "baremetal", "--target", "x86_64-unknown-none",
             "--bin", "qsim-native-probe"], "build")
        binary = output / "qsim-native-probe.elf"
        shutil.copyfile(target / "x86_64-unknown-none/release/qsim-native-probe", binary)
        report["binary_sha256"] = digest(binary)
        report["elf"] = inspect_user(binary)
        disassembly = run(["objdump", "-d", "--no-show-raw-insn", "-M", "intel", str(binary)], "disassembly")
        report["scalar_gate"] = inspect_instructions(disassembly)
        report["disassembly_sha256"] = digest(output / "disassembly.stdout.txt")
        symbols = run(["nm", str(binary)], "symbols")
        report["software_float_symbols"] = [name for name in ("__adddf3", "__muldf3", "__divdf3", "__floatdidf") if re.search(r"\b" + name + r"$", symbols, re.MULTILINE)]
        if len(report["software_float_symbols"]) != 4:
            raise RuntimeError("expected compiler-builtins scalar floating point helpers are absent")
        report["passed"] = True
        print(f"I01 target build and scalar inspection passed: {binary}")
    except Exception as error:
        report["error"] = str(error)
        raise
    finally:
        (output / "build-report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
