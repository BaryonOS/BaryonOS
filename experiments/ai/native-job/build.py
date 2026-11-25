#!/usr/bin/env python3
"""Build and inspect the native SSE2 AI job ELF and its host oracle; no VM is started.

The job is compiled for the hard-float x86_64-unknown-linux-gnu ABI as a
freestanding static ELF. The gate below checks the linked ELF, including the
precompiled core/compiler_builtins code: SSE/SSE2 only, no x87, MMX, SSE3+,
VEX/EVEX, privileged or Linux syscall instructions, int 0x80 as the only trap.
"""
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
FORBIDDEN = set("""hlt cli sti in out ins insb insw insd outs outsb outsw outsd lgdt lidt lldt ltr
sgdt sidt wrmsr rdmsr syscall sysenter sysexit sysret iretq iret invlpg wbinvd invd clts lmsw swapgs
xsave xsave64 xsaveopt xrstor xrstor64 xsetbv xgetbv fxsave fxsave64 fxrstor fxrstor64 emms femms
rdrand rdseed cpuid ud2""".split())
PREFIXES = {"lock", "rep", "repz", "repe", "repnz", "repne", "data16", "cs", "ds", "es", "ss", "fs", "gs", "notrack", "bnd"}
WIDE = re.compile(r"\b(?:[yz]mm\d+|mm[0-7]|st\(?\d?\)?|k[0-7]|tmm\d|cr\d+|dr\d+)\b")
XMM = re.compile(r"\bxmm(\d+)\b")


def inspect_instructions(disassembly: str) -> dict:
    """Allowlist SSE/SSE2 on the linked ELF; reject everything outside that profile.

    A bounded decoded-mnemonic check of GNU objdump text, not a formal proof.
    """
    count = sse = traps = 0
    for line in disassembly.splitlines():
        match = re.match(r"^\s*([0-9a-f]+):\t(.*)$", line)
        if not match:
            if re.match(r"^\s*[0-9a-f]+:", line):
                raise ValueError(f"unparsed instruction: {line.strip()}")
            continue
        address, text = match.groups()
        words = text.split()
        while words and words[0] in PREFIXES:
            words = words[1:]
        if not words:
            continue  # prefix-only padding line
        mnemonic, operands = words[0], " ".join(words[1:])
        count += 1
        if mnemonic == "(bad)":
            raise ValueError(f"undecodable bytes at {address}")
        if WIDE.search(operands):
            raise ValueError(f"register outside the SSE2 profile at {address}: {text}")
        if mnemonic in FORBIDDEN or (mnemonic.startswith("f") and mnemonic not in {"fence"}):
            raise ValueError(f"forbidden instruction at {address}: {text}")
        simd = mnemonic in SSE or mnemonic in SSE2 or bool(CMP_PSEUDO.match(mnemonic))
        if XMM.search(operands) and not simd:
            raise ValueError(f"xmm instruction outside SSE/SSE2 at {address}: {text}")
        if any(int(r) > 15 for r in XMM.findall(operands)):
            raise ValueError(f"EVEX register at {address}: {text}")
        sse += simd
        if mnemonic == "int":
            if operands.strip() != "0x80":
                raise ValueError(f"unexpected interrupt at {address}")
            traps += 1
    if count == 0 or sse == 0 or traps == 0:
        raise ValueError("empty code, no SSE2 use or missing int 0x80 boundary")
    return {"decoded_instructions": count, "sse_sse2_instructions": sse, "int80_sites": traps}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    args = parser.parse_args()
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
        (output / f"{name}.stdout.txt").write_text(done.stdout)
        (output / f"{name}.stderr.txt").write_text(done.stderr)
        report["commands"].append({"argv": command, "returncode": done.returncode})
        if done.returncode:
            raise RuntimeError(f"{name} failed: {done.stderr[-2000:]}")
        return done.stdout

    manifest = str(HERE / "Cargo.toml")
    try:
        report["rustc"] = run(["rustc", "+1.100.0", "--version"], "rustc", env).strip()
        run(["cargo", "+1.100.0", "build", "--manifest-path", manifest, "--locked", "--offline", "--release",
             "--target", TARGET, "--features", "native", "--bin", "ai-job-native"], "cargo-build", env)
        binary = output / "ai-job-native.elf"
        shutil.copyfile(target / TARGET / "release/ai-job-native", binary)
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
        report["passed"] = True
        print(f"Native AI job artifact passed: {binary}")
    finally:
        (output / "build-report.json").write_text(json.dumps(report, indent=2) + "\n")
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
