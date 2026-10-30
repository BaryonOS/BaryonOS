import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest

from replay_handoff import HandoffError, create_cold_input, load_handoff, verify_cold_input

REPOSITORY = Path(__file__).resolve().parents[4]
# Structural placeholder bytes only: never decoded as tokens, sent to the NAS
# service or logged. The repository-boundary test removes its private fixture.
STRUCTURAL_PLACEHOLDER = bytes(range(80))


class ReplayHandoffTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        os.chmod(self.root, 0o700)
        self.run = self.root / "normal-run"
        self.run.mkdir(mode=0o700)
        self.manifest_path = self.run / "manifest.json"
        self.payload_path = self.run / "opaque-pair.bin"
        self.manifest = {
            "schema_version": 1,
            "source_commit": "a" * 40,
            "origin_run_id": "normal-guest-run-001",
            "origin_role": "guest",
            "origin_stage": "normal",
            "capture_stage": "normal-after-stage",
            "completed": True,
            "payload_length": 80,
            "payload_sha256": hashlib.sha256(STRUCTURAL_PLACEHOLDER).hexdigest(),
        }
        self.write_files()

    def tearDown(self):
        self.temporary.cleanup()

    def write_files(self):
        self.manifest_path.write_text(json.dumps(self.manifest), encoding="utf-8")
        self.payload_path.write_bytes(STRUCTURAL_PLACEHOLDER)
        self.manifest_path.chmod(0o600)
        self.payload_path.chmod(0o600)

    def load(self):
        return load_handoff(
            self.manifest_path,
            self.payload_path,
            REPOSITORY,
            self.manifest["source_commit"],
            self.manifest["origin_run_id"],
        )

    def test_exact_private_normal_guest_stage_artifact_is_bound(self):
        manifest, payload = self.load()
        self.assertEqual(manifest["origin_role"], "guest")
        self.assertEqual(manifest["origin_stage"], "normal")
        self.assertEqual(manifest["capture_stage"], "normal-after-stage")
        self.assertTrue(manifest["completed"])
        self.assertEqual(len(payload), 80)

    def test_cold_input_is_private_read_only_and_unchanged(self):
        _, payload = self.load()
        cold = self.root / "cold-run"
        cold.mkdir(mode=0o700)
        destination = cold / "startup-input.bin"
        digest = create_cold_input(payload, destination, REPOSITORY)
        self.assertEqual(destination.stat().st_mode & 0o777, 0o400)
        self.assertEqual(verify_cold_input(destination, digest, REPOSITORY), payload)
        destination.chmod(0o600)
        destination.write_bytes(b"x" * 80)
        destination.chmod(0o400)
        with self.assertRaises(HandoffError):
            verify_cold_input(destination, digest, REPOSITORY)

    def test_wrong_role_stage_incomplete_or_wrong_digest_fail_closed(self):
        for field, value in (
            ("origin_role", "administrator"),
            ("origin_stage", "cold"),
            ("capture_stage", "before-stage"),
            ("completed", False),
            ("payload_length", 79),
            ("payload_sha256", "0" * 64),
        ):
            with self.subTest(field=field):
                self.manifest[field] = value
                self.write_files()
                with self.assertRaises(HandoffError):
                    self.load()
                self.manifest[field] = {
                    "origin_role": "guest",
                    "origin_stage": "normal",
                    "capture_stage": "normal-after-stage",
                    "completed": True,
                    "payload_length": 80,
                    "payload_sha256": hashlib.sha256(STRUCTURAL_PLACEHOLDER).hexdigest(),
                }[field]

    def test_missing_extra_or_secret_bearing_manifest_fields_fail_closed(self):
        for changed in (
            {key: value for key, value in self.manifest.items() if key != "source_commit"},
            {**self.manifest, "session_token": "must-not-appear"},
        ):
            self.manifest_path.write_text(json.dumps(changed), encoding="utf-8")
            self.manifest_path.chmod(0o600)
            with self.assertRaises(HandoffError):
                self.load()
        self.write_files()

    def test_truncated_oversized_and_tampered_payload_fail_closed(self):
        for payload in (b"x" * 79, b"x" * 81, b"y" * 80):
            self.payload_path.write_bytes(payload)
            self.payload_path.chmod(0o600)
            with self.assertRaises(HandoffError):
                self.load()

    def test_artifacts_inside_repository_or_with_broad_permissions_fail_closed(self):
        with tempfile.TemporaryDirectory(prefix="nas-replay-negative-", dir=REPOSITORY) as inside:
            os.chmod(inside, 0o700)
            repository_artifact = Path(inside) / "replay-token-negative-fixture.bin"
            repository_artifact.write_bytes(STRUCTURAL_PLACEHOLDER)
            repository_artifact.chmod(0o600)
            with self.assertRaisesRegex(
                HandoffError, "handoff material must remain outside the repository"
            ):
                load_handoff(
                    self.manifest_path,
                    repository_artifact,
                    REPOSITORY,
                    self.manifest["source_commit"],
                    self.manifest["origin_run_id"],
                )
        self.payload_path.chmod(0o640)
        with self.assertRaises(HandoffError):
            self.load()

    def test_source_and_origin_are_bound_to_the_run_selected_by_root(self):
        for expected_source, expected_run in (
            ("b" * 40, "normal-guest-run-001"),
            ("a" * 40, "different-run"),
            ("not-a-commit", "normal-guest-run-001"),
            ("a" * 40, ""),
        ):
            with self.subTest(expected_source=expected_source, expected_run=expected_run):
                with self.assertRaises(HandoffError):
                    load_handoff(
                        self.manifest_path,
                        self.payload_path,
                        REPOSITORY,
                        expected_source,
                        expected_run,
                    )

    def test_boolean_schema_version_is_not_integer_one(self):
        self.manifest["schema_version"] = True
        self.write_files()
        with self.assertRaises(HandoffError):
            self.load()

    def test_symbolic_link_input_is_never_followed(self):
        link = self.run / "linked-payload.bin"
        link.symlink_to(self.payload_path)
        with self.assertRaises(HandoffError):
            load_handoff(
                self.manifest_path,
                link,
                REPOSITORY,
                self.manifest["source_commit"],
                self.manifest["origin_run_id"],
            )


if __name__ == "__main__":
    unittest.main()
