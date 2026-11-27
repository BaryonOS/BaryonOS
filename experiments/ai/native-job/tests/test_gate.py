import importlib.util
import os
from pathlib import Path
import stat
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("ai_job_build", Path(__file__).resolve().parents[1] / "build.py")
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)

OK = ("0000000040020000 <job>:\n  40020000:\tmulss  xmm0,DWORD PTR [rdi]\n  40020004:\tint    0x80\n"
      "  40020006:\tdata16 cs nop WORD PTR [rax+rax*1+0x0]\n  40020010:\tret\n  40020011:\tint3\n"
      "  40020012:\tint3\n\n0000000040020020 <next>:\n  40020020:\tcmovae rax,rcx\n")


class Gate(unittest.TestCase):
    def test_accepts_sse2_profile(self):
        self.assertEqual(build.inspect_instructions(OK)["sse_sse2_instructions"], 1)

    def test_rejects_outside_profile(self):
        for bad in ("vmulss xmm0,xmm1,xmm2", "addsubps xmm0,xmm1", "pshufb xmm0,xmm1", "fld DWORD PTR [rax]",
                    "vaddps ymm0,ymm1,ymm2", "paddb mm0,mm1", "syscall", "int    0x3", "mulss xmm16,xmm1",
                    "fxsave64 [rax]", "mov rax,cr3", "(bad)",
                    # Warden counterexamples: prefixed system/x87 forms, VEX without
                    # vector registers, SSE3+/BMI, system instructions and traps.
                    "rex.W syscall", "addr32 syscall", "rex syscall", "rex fld DWORD PTR [rax]",
                    "vzeroupper", "vldmxcsr DWORD PTR [rax]", "vstmxcsr DWORD PTR [rax]",
                    "shlx rax,rbx,rcx", "andn rax,rbx,rcx", "pdep rax,rbx,rcx",
                    "pextrw WORD PTR [rax],xmm0,0x1", "crc32 eax,ecx", "monitor", "mwait",
                    "popcnt rax,rbx", "lzcnt rax,rbx", "tzcnt rax,rbx", "sysretq", "clac", "wrpkru",
                    "rdfsbase rax", "xsavec [rax]", "xsaves [rax]", "xsaveopt64 [rax]", "sldt eax",
                    "str eax", "smsw eax", "int1", "ud1 eax,DWORD PTR [rax]", "ud2", "cpuid",
                    "mov ds,eax", "mov rax,QWORD PTR fs:0x28", "hlt", "rdtsc"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                build.inspect_instructions(OK + f"  40020010:\t{bad}\n")

    def test_int3_only_as_padding(self):
        for bad in ("  40020030:\tint3\n",  # after a non-terminator
                    "  40020030:\tret\n  40020031:\tint3\n  40020032:\tnop\n"):  # code after padding
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                build.inspect_instructions(OK + bad)

    def test_int3_after_call_only_for_noreturn_targets(self):
        panic = "_RNvNtCs8uuN1nrGffB_4core9panicking18panic_bounds_check"
        unwind = "_RNvCs5WqGp7zLRnA_7___rustc17rust_begin_unwind"
        slot = "QWORD PTR [rip+0xfffffffffffe97f7]        # 40010220 <DW.ref.rust_eh_personality+0x20>"
        slots = {0x40010220: unwind}
        for good in (f"400262f7 <{panic}>", "40026a10 <_RNvNtNtCs8uuN1nrGffB_4core9panicking11panic_const"
                     "23panic_const_div_by_zero>", slot):
            with self.subTest(good=good):
                build.inspect_instructions(OK + f"  40020030:\tcall   {good}\n  40020035:\tint3\n", slots)
        for bad, table in (("40020000 <job>", slots),  # returning callee
                           (f"400262f7 <{panic}+0x4>", slots),  # not the exact symbol
                           (f"400262f3 <{panic}-0x4>", slots),  # negative offset, not the entry
                           ("40020000 <_RNvNtCs8uuN1nrGffB_4core3fmt5write>", slots),  # core, not panicking
                           (slot, {}),  # unresolved slot
                           (slot, {0x40010220: "memmove"}),  # returning slot target
                           ("rax", slots)):  # register call
            with self.subTest(bad=bad), self.assertRaisesRegex(ValueError, "after call to"):
                build.inspect_instructions(OK + f"  40020030:\tcall   {bad}\n  40020035:\tint3\n", table)

    def test_publish_is_owner_only_even_with_an_open_umask(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "build-report.json"
            path.write_bytes(b"old")
            os.chmod(path, 0o644)
            previous = os.umask(0)
            try:
                build.publish(path, b"new")
            finally:
                os.umask(previous)
            self.assertEqual(path.read_bytes(), b"new")
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)

    def test_source_tree_rejects_directory_symlink(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "tree/real").mkdir(parents=True)
            (root / "tree/real/a.rs").write_text("fn a() {}\n")
            with mock.patch.object(build, "ROOT", root), mock.patch.object(build, "SOURCE_TREES", (root / "tree",)):
                build.source_tree_sha256()
                (root / "tree/link").symlink_to(root / "tree/real", target_is_directory=True)
                with self.assertRaisesRegex(ValueError, "unexpected source entry"):
                    build.source_tree_sha256()


if __name__ == "__main__":
    unittest.main()
