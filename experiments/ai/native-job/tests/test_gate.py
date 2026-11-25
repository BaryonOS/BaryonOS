import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("ai_job_build", Path(__file__).resolve().parents[1] / "build.py")
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)

OK = "  40020000:\tmulss  xmm0,DWORD PTR [rdi]\n  40020004:\tint    0x80\n  40020006:\tdata16 cs nop WORD PTR [rax+rax*1+0x0]\n"


class Gate(unittest.TestCase):
    def test_accepts_sse2_profile(self):
        self.assertEqual(build.inspect_instructions(OK)["sse_sse2_instructions"], 1)

    def test_rejects_outside_profile(self):
        for bad in ("vmulss xmm0,xmm1,xmm2", "addsubps xmm0,xmm1", "pshufb xmm0,xmm1", "fld DWORD PTR [rax]",
                    "vaddps ymm0,ymm1,ymm2", "paddb mm0,mm1", "syscall", "int    0x3", "mulss xmm16,xmm1",
                    "fxsave64 [rax]", "mov rax,cr3", "(bad)"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                build.inspect_instructions(OK + f"  40020010:\t{bad}\n")


if __name__ == "__main__":
    unittest.main()
