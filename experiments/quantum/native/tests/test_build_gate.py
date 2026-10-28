"""Negative controls for the freestanding artifact's decoded-instruction gate."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

path = Path(__file__).resolve().parents[1] / "build.py"
spec = importlib.util.spec_from_file_location("qsim_native_build", path)
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class ScalarGate(unittest.TestCase):
    def test_scalar_integer_and_string_instructions_pass(self):
        result = build.inspect_instructions("40000000: int 0x80\n40000002: rep movs BYTE PTR es:[rdi],BYTE PTR ds:[rsi]\n40000004: lfence\n40000007: add rax,rdx")
        self.assertEqual(result["decoded_instructions"], 4)
        self.assertEqual(result["int80_sites"], 1)

    def test_extended_context_and_direct_hardware_are_rejected(self):
        for instruction in ("pxor xmm0,xmm0", "vzeroupper", "fadd st,st(1)", "emms", "fxsave [rax]", "xrstor [rax]", "kmovw k1,eax", "out dx,eax", "syscall", "int 0x81"):
            with self.subTest(instruction=instruction), self.assertRaises(ValueError):
                build.inspect_instructions("40000000: int 0x80\n40000002: " + instruction)

    def test_missing_or_unparsed_instructions_fail_closed(self):
        for text in ("", "40000000: nop", "40000000: int 0x80\n40000002: (bad)",
                     "40000000: int 0x80\n40000002: data16",
                     "40000000: int 0x80\n40000002: rex.W",
                     "40000000: int 0x80\n40000002: future-prefix nop",
                     "40000000: int 0x80\n40000002: data16 future-opcode",
                     "40000000: int 0x80\n40000002: " + "data16 " * 15 + "nop"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                build.inspect_instructions(text)

    def test_prefix_tokens_are_not_scalar_opcode_verdicts(self):
        for prefix in sorted(build.PREFIXES) + ["rex", "rex.W", "rex.WRXB", "data16 cs repz"]:
            for opcode in ("fninit", "emms", "ldmxcsr DWORD PTR [rax]", "vzeroupper", "out dx,al", "syscall", "int 0x81"):
                with self.subTest(prefix=prefix, opcode=opcode), self.assertRaises(ValueError):
                    build.inspect_instructions("40000000: int 0x80\n40000002: " + prefix + " " + opcode)


class ActualObjdump(unittest.TestCase):
    """Disassemble bytes only. These untrusted fixtures are never executed."""

    def disassemble(self, hex_bytes):
        with tempfile.TemporaryDirectory(prefix="qsim-prefix-test-") as directory:
            path = Path(directory) / "fixture.bin"
            path.write_bytes(bytes.fromhex(hex_bytes))
            result = subprocess.run(
                ["objdump", "-D", "-b", "binary", "-m", "i386:x86-64",
                 "--no-show-raw-insn", "-M", "intel", str(path)],
                text=True, capture_output=True, timeout=10, check=True,
            )
        return result.stdout

    def test_real_prefixed_extended_state_instructions_are_rejected(self):
        cases = {
            "66dbe3": "data16 fninit",
            "f3dbe3": "repz fninit",
            "f2dbe3": "repnz fninit",
            "2edbe3": "cs fninit",
            "67dbe3": "addr32 fninit",
            "66662ef3dbe3": "data16 data16 cs repz fninit",
            "2e0f77": "cs emms",
            "670f77": "addr32 emms",
            "2e0fefc0": "cs pxor mm0,mm0",
            "f20fae10": "repnz ldmxcsr DWORD PTR [rax]",
            "2e0fae18": "cs stmxcsr DWORD PTR [rax]",
            "670f57c0": "addr32 xorps xmm0,xmm0",
            "67c5f877": "addr32 vzeroupper",
            "2ec5fdefc0": "cs vpxor ymm0,ymm0,ymm0",
        }
        for machine_bytes, decoded in cases.items():
            with self.subTest(decoded=decoded):
                output = self.disassemble("cd80" + machine_bytes)
                self.assertIn(decoded, output)
                with self.assertRaises(ValueError):
                    build.inspect_instructions(output)

    def test_real_prefixed_hardware_and_foreign_interrupts_are_rejected(self):
        for machine_bytes, decoded in (("f3ee", "repz out"), ("2e0f05", "cs syscall"), ("f3cd81", "repz int")):
            with self.subTest(decoded=decoded):
                output = self.disassemble("cd80" + machine_bytes)
                self.assertIn(decoded, output)
                with self.assertRaises(ValueError):
                    build.inspect_instructions(output)

    def test_real_scalar_prefix_chains_and_strings_remain_valid(self):
        output = self.disassemble("cd80f3a4f3aa66662e0f1f840000000000f0480fb10f4088c04801c8")
        for instruction in ("rep movs", "rep stos", "data16 cs nop", "lock cmpxchg", "rex mov"):
            self.assertIn(instruction, output)
        result = build.inspect_instructions(output)
        self.assertEqual(result, {"decoded_instructions": 7, "int80_sites": 1, "extended_registers_found": False})

    def test_real_incomplete_prefix_and_unknown_opcodes_fail_closed(self):
        for machine_bytes in ("66", "48", "0fff", "0fa2"):
            with self.subTest(machine_bytes=machine_bytes), self.assertRaises(ValueError):
                build.inspect_instructions(self.disassemble("cd80" + machine_bytes))


if __name__ == "__main__":
    unittest.main()
