"""H05-T5d: the init LOCK caller (feature lock-caller) exists only in the S3
TEST init. A build with lock-caller but without test-system-s3 must stop at
the compile_error; the test-system-s3 build must succeed. Native build only;
it never boots anything."""
from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUARD = "lock-caller (LOCK 0x92) exists only in the S3 TEST init; enable feature test-system-s3"


def system_build():
    spec = importlib.util.spec_from_file_location("system_example_build", ROOT / "sdk/examples/system/build.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@unittest.skipUnless(shutil.which("cargo"), "cargo is unavailable on this host")
class LockCallerGuardTests(unittest.TestCase):
    def build(self, features: str) -> subprocess.CompletedProcess:
        command, environment = system_build().cargo_command(features)
        # The pinned toolchain, the init binary only, a target dir in this checkout.
        command = ["cargo", "+1.100.0", *command[1:command.index("--bins")], "--bin", "system-init",
                   *command[command.index("--bins") + 1:]]
        environment["CARGO_TARGET_DIR"] = os.environ.get(
            "NOS_LOCK_GUARD_TARGET", str(ROOT / "target/s3-lock-caller-guard"))
        return subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, text=True,
                              check=False, timeout=1800)

    def test_lock_caller_without_the_s3_feature_does_not_compile(self):
        result = self.build("native,lock-caller")
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(GUARD, result.stderr)

    def test_the_s3_test_init_builds(self):
        result = self.build("test-system-s3")
        self.assertEqual(result.returncode, 0, result.stderr[-4000:])
        self.assertNotIn(GUARD, result.stderr)


if __name__ == "__main__":
    unittest.main()
