"""Static contract for the key service cryptography upgrade.

These checks inspect manifests, lockfiles, compile guards, and the three
freestanding build entry points. They do not claim runtime or object-code
evidence; those remain separate build and test gates.
"""

from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
LOCKS = (
    "Cargo.lock",
    "sdk/Cargo.lock",
    "sdk/examples/random-probe/Cargo.lock",
    "sdk/keyring-client/Cargo.lock",
)
BUILDERS = (
    ".github/workflows/native-foundation.yml",
    "sdk/examples/keyring/build.py",
    "sdk/examples/system/build.py",
)
AEAD_CHECKSUM = "9b89e1c441e926b9c82a8d023f6e1b7ae0adcfaa7d621814e4d60789bac751cb"


class KeyringCryptoContractTests(unittest.TestCase):
    """Keep every independently locked keyring consumer on one contract."""

    def test_manifest_pins_zeroizing_aead_and_direct_chacha(self):
        manifest = tomllib.loads(
            (ROOT / "services/keyring/Cargo.toml").read_text(encoding="utf-8")
        )
        dependencies = manifest["dependencies"]
        self.assertEqual(dependencies["chacha20poly1305"]["version"], "=0.11.0")
        self.assertEqual(dependencies["chacha20poly1305"]["features"], ["zeroize"])
        self.assertFalse(dependencies["chacha20poly1305"]["default-features"])
        self.assertEqual(dependencies["chacha20"]["version"], "=0.9.1")
        self.assertEqual(dependencies["chacha20"]["features"], ["zeroize"])
        self.assertFalse(dependencies["chacha20"]["default-features"])

    def test_all_four_locks_resolve_the_same_aead_stack(self):
        expected = {
            "aead": "0.6.1",
            "chacha20poly1305": "0.11.0",
            "poly1305": "0.9.1",
        }
        for relative in LOCKS:
            with self.subTest(lock=relative):
                lock = tomllib.loads((ROOT / relative).read_text(encoding="utf-8"))
                packages = lock["package"]
                for name, version in expected.items():
                    matches = [package for package in packages if package["name"] == name]
                    self.assertEqual([package["version"] for package in matches], [version])
                aead = next(
                    package for package in packages
                    if package["name"] == "chacha20poly1305"
                )
                self.assertEqual(aead["checksum"], AEAD_CHECKSUM)
                direct = [
                    package for package in packages
                    if package["name"] == "chacha20" and package["version"] == "0.9.1"
                ]
                self.assertEqual(len(direct), 1)

    def test_freestanding_builders_select_only_portable_crypto_backends(self):
        required = (
            "chacha20_force_soft",
            'chacha20_backend="soft"',
            'poly1305_backend="soft"',
        )
        for relative in BUILDERS:
            with self.subTest(builder=relative):
                text = (ROOT / relative).read_text(encoding="utf-8")
                for selector in required:
                    self.assertIn(selector, text)
                self.assertNotIn("poly1305_force_soft", text)

    def test_poly1305_selector_is_declared_and_required_for_none(self):
        manifest = (ROOT / "services/keyring/Cargo.toml").read_text(encoding="utf-8")
        source = (ROOT / "services/keyring/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn('cfg(poly1305_backend, values("soft"))', manifest)
        self.assertIn(
            'all(target_os = "none", not(poly1305_backend = "soft"))', source
        )
        self.assertIn("Key Service AEAD requires", source)


if __name__ == "__main__":
    unittest.main()
