"""Static contracts for the freestanding Ed25519 dependency boundary."""

from __future__ import annotations

import ast
from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
MANIFESTS = (
    ROOT / "crates/update-core/Cargo.toml",
    ROOT / "crates/app-package-core/Cargo.toml",
)
LOCKFILES = (
    ROOT / "Cargo.lock",
    ROOT / "sdk/Cargo.lock",
    ROOT / "sdk/app-management/Cargo.lock",
)
CURVE_CFG = 'curve25519_dalek_backend="serial"'
SHA2_CFG = 'sha2_backend="soft"'


class Ed25519DependencyContractTests(unittest.TestCase):
    def test_manifests_and_independent_locks_move_atomically(self):
        for path in MANIFESTS:
            with self.subTest(path=path.relative_to(ROOT)):
                manifest = tomllib.loads(path.read_text())
                dependency = manifest["dependencies"]["ed25519-dalek"]
                self.assertEqual(dependency["version"].lstrip("="), "3.0.0")
                self.assertFalse(dependency["default-features"])

        for path in LOCKFILES:
            with self.subTest(path=path.relative_to(ROOT)):
                lock = tomllib.loads(path.read_text())
                packages = lock["package"]
                dalek = [item for item in packages if item["name"] == "ed25519-dalek"]
                self.assertEqual(len(dalek), 1)
                self.assertEqual(dalek[0]["version"], "3.0.0")
                self.assertIn("sha2 0.11.1", dalek[0]["dependencies"])
                self.assertTrue(
                    any(item["name"] == "sha2" and item["version"] == "0.11.1"
                        for item in packages)
                )

    def test_every_freestanding_consumer_pins_scalar_backends(self):
        cargo_config = tomllib.loads((ROOT / ".cargo/config.toml").read_text())
        uefi_flags = cargo_config["target"]["x86_64-unknown-uefi"]["rustflags"]
        self.assertIn(CURVE_CFG, uefi_flags)
        self.assertIn(SHA2_CFG, uefi_flags)

        update_build = ast.parse((ROOT / "sdk/examples/update/build.py").read_text())
        string_literals = {
            node.value for node in ast.walk(update_build)
            if isinstance(node, ast.Constant) and isinstance(node.value, str)
        }
        self.assertIn(CURVE_CFG, string_literals)
        self.assertIn(SHA2_CFG, string_literals)

        workflow = (ROOT / ".github/workflows/app-management-quality.yml").read_text()
        encoded_flags = (
            r'--cfg\x1fcurve25519_dalek_backend="serial"'
            r'\x1f--cfg\x1fsha2_backend="soft"'
        )
        self.assertEqual(workflow.count(encoded_flags), 2)


if __name__ == "__main__":
    unittest.main()
