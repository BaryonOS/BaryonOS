#!/usr/bin/env python3
"""Build and inspect the native SSE2 AI job ELF and its host oracle; no VM is started.

The job is compiled for the hard-float x86_64-unknown-linux-gnu ABI as a
freestanding static ELF. The gate below checks the linked ELF, including the
precompiled core/compiler_builtins code against a positive allowlist of baseline
x86-64 integer and SSE/SSE2 mnemonics: no x87, MMX, SSE3+, BMI, VEX/EVEX, privileged
or Linux syscall instructions; int 0x80 is the only executable trap.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = "x86_64-unknown-linux-gnu"

SSE = set("""addps addss andnps andps cmpps cmpss comiss cvtpi2ps cvtps2pi cvtsi2ss cvtss2si
cvttps2pi cvttss2si divps divss ldmxcsr maxps maxss minps minss movaps movhlps movhps movlhps
movlps movmskps movss movups mulps mulss orps rcpps rcpss rsqrtps rsqrtss shufps sqrtps sqrtss
stmxcsr subps subss ucomiss unpckhps unpcklps xorps sfence""".split())
SSE2 = set("""addpd addsd andnpd andpd cmppd cmpsd comisd cvtdq2pd cvtdq2ps cvtpd2dq cvtpd2ps
cvtps2dq cvtps2pd cvtsd2si cvtsd2ss cvtsi2sd cvtss2sd cvttpd2dq cvttps2dq cvttsd2si divpd divsd
maxpd maxsd minpd minsd movapd movd movq movdqa movdqu movhpd movlpd movmskpd movsd movupd mulpd
mulsd orpd shufpd sqrtpd sqrtsd subpd subsd ucomisd unpckhpd unpcklpd xorpd packsswb packssdw
packuswb paddb paddw paddd paddq paddsb paddsw paddusb paddusw pand pandn pavgb pavgw pcmpeqb
pcmpeqw pcmpeqd pcmpgtb pcmpgtw pcmpgtd pextrw pinsrw pmaddwd pmaxsw pmaxub pminsw pminub
pmovmskb pmulhuw pmulhw pmullw pmuludq por psadbw pshufd pshufhw pshuflw pslldq psllw pslld
psllq psraw psrad psrldq psrlw psrld psrlq psubb psubw psubd psubq psubsb psubsw psubusb
psubusw punpckhbw punpckhwd punpckhdq punpckhqdq punpcklbw punpcklwd punpckldq punpcklqdq pxor
lfence mfence""".split())
CMP_PSEUDO = re.compile(r"^cmp(?:eq|lt|le|unord|neq|nlt|nle|ord)(?:ss|ps|sd|pd)$")
# Baseline x86-64 integer instructions (no BMI, POPCNT/LZCNT/TZCNT, CRC32, VEX or
# system instructions). Everything not listed here or in SSE/SSE2 is rejected.
GPR = set("""mov movabs movzx movsx movsxd lea add adc sub sbb and or xor not neg inc dec cmp
test imul mul idiv div shl shr sal sar rol ror rcl rcr shld shrd bt bts btr btc bsf bsr bswap
xchg cmpxchg xadd push pop call ret jmp nop cbw cwde cdqe cwd cdq cqo leave movs stos cmps scas
lods int""".split())
CC = "o|no|b|c|nae|ae|nb|nc|e|z|ne|nz|be|na|a|nbe|s|ns|p|pe|np|po|l|nge|ge|nl|le|ng|g|nle"
CONDITIONAL = re.compile(rf"^(?:j(?:{CC}|rcxz|ecxz)|set(?:{CC})|cmov(?:{CC}))$")
TERMINATORS = {"ret", "jmp", "call", "int3"}
PREFIXES = {"lock", "rep", "repz", "repe", "repnz", "repne", "data16", "addr32", "cs", "ds", "es", "ss",
            "fs", "gs", "notrack", "bnd", "xacquire", "xrelease"}
REX = re.compile(r"^rex(?:\.[WRXB]+)?$")
WIDE = re.compile(r"\b(?:[yz]mm\d+|mm[0-7]|st\(?\d?\)?|k[0-7]|tmm\d|cr\d+|dr\d+)\b")
SEGMENT = re.compile(r"(?:^|,)\s*[c-gs]s\s*(?:,|$)|\b[fg]s:")
XMM = re.compile(r"\bxmm(\d+)\b")
LABEL = re.compile(r"^[0-9a-f]+ <[^>]+>:$")


def inspect_instructions(disassembly: str) -> dict:
    """Positive mnemonic allowlist (baseline GPR + SSE/SSE2) on the linked ELF.

    `int 0x80` is the only executable trap; `int3` is accepted only as linker
    padding after an unconditional transfer and before the next symbol. A bounded
    decoded-mnemonic check of GNU objdump text, not a formal proof.
    """
    count = sse = traps = 0
    previous, padding = "", False
    for line in disassembly.splitlines():
        if LABEL.match(line.strip()):
            previous, padding = "", False
            continue
        match = re.match(r"^\s*([0-9a-f]+):\t(.*)$", line)
        if not match:
            if re.match(r"^\s*[0-9a-f]+:", line):
                raise ValueError(f"unparsed instruction: {line.strip()}")
            continue
        address, text = match.groups()
        words = text.split()
        while words and (words[0] in PREFIXES or REX.match(words[0])):
            words = words[1:]
        if not words:
            continue  # prefix-only padding line
        mnemonic, operands = words[0], " ".join(words[1:])
        count += 1
        if mnemonic == "int3":
            if not (padding or previous in TERMINATORS):
                raise ValueError(f"int3 outside inter-function padding at {address}")
            padding, previous = True, mnemonic
            continue
        if padding:
            raise ValueError(f"code follows int3 padding without a symbol at {address}")
        if WIDE.search(operands) or SEGMENT.search(operands):
            raise ValueError(f"register outside the SSE2 profile at {address}: {text}")
        if any(int(r) > 15 for r in XMM.findall(operands)):
            raise ValueError(f"EVEX register at {address}: {text}")
        simd = mnemonic in SSE or mnemonic in SSE2 or bool(CMP_PSEUDO.match(mnemonic))
        if mnemonic == "pextrw" and "PTR" in operands.split(",")[0]:
            simd = False  # the memory-destination form is SSE4.1
        if not simd and (XMM.search(operands) or not (mnemonic in GPR or CONDITIONAL.match(mnemonic))):
            raise ValueError(f"instruction outside the baseline GPR + SSE/SSE2 profile at {address}: {text}")
        if mnemonic == "int":
            if operands.strip() != "0x80":
                raise ValueError(f"unexpected interrupt at {address}")
            traps += 1
        sse += simd
        previous = mnemonic
    if count == 0 or sse == 0 or traps == 0:
        raise ValueError("empty code, no SSE2 use or missing int 0x80 boundary")
    return {"decoded_instructions": count, "sse_sse2_instructions": sse, "int80_sites": traps}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


SOURCE_TREES = (HERE, ROOT / "experiments/ai/reference-forward")
SKIPPED = {"target", "__pycache__", ".git"}


def source_tree_sha256() -> str:
    """Fingerprint of the job and forward-engine inputs (the kernel source hash omits them)."""
    digest = hashlib.sha256()
    for tree in SOURCE_TREES:
        for path in sorted(tree.rglob("*")):
            relative = path.relative_to(tree)
            if SKIPPED & set(relative.parts) or path.is_dir():
                continue
            if path.is_symlink() or not path.is_file():
                raise ValueError(f"unexpected source entry: {path}")
            digest.update(path.relative_to(ROOT).as_posix().encode() + b"\0" + sha(path).encode() + b"\n")
    return digest.hexdigest()


def publish(path: Path, data: bytes) -> None:
    """Write via an exclusive temporary file and rename; never through an existing symlink."""
    if path.is_symlink():
        raise ValueError(f"refusing symlinked output: {path}")
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o644)
    with os.fdopen(descriptor, "wb") as handle:
        handle.write(data)
    os.replace(temporary, path)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    args = parser.parse_args()
    if args.output_dir.is_symlink():
        raise SystemExit("refusing a symlinked output directory")
    output, target = args.output_dir.resolve(), args.target_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    report: dict = {"schema_version": 1, "target": TARGET, "passed": False, "commands": []}
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join([
        "-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes",
        "-C", "linker=rust-lld", "-C", f"link-arg=-T{HERE / 'linker.ld'}", "-C", "link-arg=--no-pie",
        "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
    ])
    host_env = dict(env)
    host_env.pop("CARGO_ENCODED_RUSTFLAGS")

    def run(command: list[str], name: str, environment: dict) -> str:
        done = subprocess.run(command, cwd=ROOT, env=environment, text=True, capture_output=True, timeout=600)
        publish(output / f"{name}.stdout.txt", done.stdout.encode())
        publish(output / f"{name}.stderr.txt", done.stderr.encode())
        report["commands"].append({"argv": command, "returncode": done.returncode})
        if done.returncode:
            raise RuntimeError(f"{name} failed: {done.stderr[-2000:]}")
        return done.stdout

    manifest = str(HERE / "Cargo.toml")
    try:
        report["source_tree_sha256"] = source_tree_sha256()
        report["rustc"] = run(["rustc", "+1.100.0", "--version"], "rustc", env).strip()
        run(["cargo", "+1.100.0", "build", "--manifest-path", manifest, "--locked", "--offline", "--release",
             "--target", TARGET, "--features", "native", "--bin", "ai-job-native"], "cargo-build", env)
        binary = output / "ai-job-native.elf"
        publish(binary, (target / TARGET / "release/ai-job-native").read_bytes())
        undefined = run(["nm", "-u", str(binary)], "undefined", env)
        if undefined.strip():
            raise RuntimeError(f"undefined symbols: {undefined}")
        disassembly = run(["objdump", "-d", "-M", "intel", "--no-show-raw-insn", str(binary)], "disassembly", env)
        report["instruction_gate"] = inspect_instructions(disassembly)
        report["disassembly_sha256"] = sha(output / "disassembly.stdout.txt")
        expected = run(["cargo", "+1.100.0", "run", "--manifest-path", manifest, "--locked", "--offline",
                        "--release", "--bin", "ai-job-expected"], "host-expected", host_env)
        report["expected"] = json.loads(expected)
        report["binary_sha256"] = sha(binary)
        if source_tree_sha256() != report["source_tree_sha256"]:
            raise RuntimeError("job or forward-engine source changed during the build")
        report["passed"] = True
        print(f"Native AI job artifact passed: {binary}")
    finally:
        publish(output / "build-report.json", (json.dumps(report, indent=2) + "\n").encode())
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
