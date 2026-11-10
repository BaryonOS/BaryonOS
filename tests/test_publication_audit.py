"""Contract tests for the bounded local publication-audit state machine.

The API and scanner fixtures below are local simulations.  They prove failure
semantics and binding rules, not a live GitHub inventory or a Gitleaks audit.
"""

import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile
import unittest
from unittest import mock
import zipfile


ROOT = Path(__file__).resolve().parents[1]
TOOL_PATH = ROOT / "tools/publication_audit.py"
SPEC = importlib.util.spec_from_file_location("publication_audit", TOOL_PATH)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class AuditTestCase(unittest.TestCase):
    """Isolate inherited Actions state while preserving explicit Actions tests."""

    def setUp(self):
        """Remove inherited GITHUB_ACTIONS and restore it after each test."""
        inherited = os.environ.pop("GITHUB_ACTIONS", None)

        def restore():
            """Restore the runner marker exactly as inherited by the test process."""
            os.environ.pop("GITHUB_ACTIONS", None)
            if inherited is not None:
                os.environ["GITHUB_ACTIONS"] = inherited

        self.addCleanup(restore)


class FakeReservation:
    """Model a completed or cancelled state reservation for local fixtures."""

    def __init__(self):
        """Start one active no-op fixture reservation."""
        self.active = True

    def commit(self, unused_size):
        """Mark the fixture reservation committed."""
        self.active = False

    def cancel_clean(self):
        """Mark the fixture reservation cleanly cancelled."""
        self.active = False


def limits(**changes):
    value = dict(AUDIT.DEFAULT_LIMITS)
    value.update(changes)
    return value


def source():
    refs = [{"name": "refs/heads/main", "oid": "1" * 40}]
    return {
        "head": "1" * 40,
        "origin": {
            "configured_url": "https://github.com/simonvanieperen/nieuw-os.git",
            "configured_repository": "simonvanieperen/nieuw-os",
            "canonical_url": "https://github.com/BaryonOS/BaryonOS.git",
            "repository_id": AUDIT.REPOSITORY_ID,
        },
        "refs": refs,
        "refs_sha256": AUDIT.object_hash(refs),
    }


def binding():
    return {
        "name": "gitleaks",
        "version": AUDIT.GITLEAKS_VERSION,
        "binary_sha256": "2" * 64,
        "config_sha256": AUDIT.sha256_bytes(AUDIT.CONFIG_BYTES),
        "ignore_sha256": AUDIT.sha256_bytes(AUDIT.IGNORE_BYTES),
        "auditor_sha256": AUDIT.file_hash(TOOL_PATH),
        "max_archive_depth": 0,
        "max_decode_depth": 5,
    }


def repository(visibility="public-local-explicit"):
    return {
        "id": AUDIT.REPOSITORY_ID,
        "full_name": AUDIT.REPOSITORY,
        "canonical_clone_url": "https://github.com/BaryonOS/BaryonOS.git",
        "visibility": visibility,
    }


def run(identifier=7, **changes):
    value = {
        "id": identifier,
        "run_attempt": 1,
        "status": "completed",
        "conclusion": "success",
        "head_sha": "3" * 40,
        "created_at": "2026-11-10T01:00:00Z",
        "updated_at": "2026-11-10T01:02:00Z",
    }
    value.update(changes)
    return value


def artifact(identifier=9, **changes):
    value = {
        "id": identifier,
        "digest": "sha256:" + "4" * 64,
        "expired": False,
        "size_in_bytes": 12,
        "created_at": "2026-11-10T01:00:00Z",
        "updated_at": "2026-11-10T01:02:00Z",
        "expires_at": "2026-11-13T01:00:00Z",
        "workflow_run_id": 7,
    }
    value.update(changes)
    return value


def manifest(inventory=None, blockers_expected=None):
    value = AUDIT.build_manifest(
        repository=repository(),
        source=source(),
        cutoff=AUDIT.parse_time("2026-11-10T02:00:00Z"),
        binding=binding(),
        limits=limits(),
        inventory=inventory or {"runs": [], "artifacts": []},
    )
    if blockers_expected is not None:
        assert value["blockers"] == blockers_expected
    return value


class FakeBudget:
    """Provide deterministic budget behavior without filesystem accounting."""

    def __init__(self, state, *, remaining=10_000, state_remaining=1024**3):
        """Initialize fixed time/state allowances and aggregate counters."""
        self.state = Path(state)
        self.seconds = remaining
        self.bytes = state_remaining
        self.consumed = {"requests": 0, "download_bytes": 0, "expanded_bytes": 0, "entries": 0}

    def remaining(self):
        """Return the fixed command-time allowance."""
        return self.seconds

    def state_remaining(self):
        """Return the fixed retained-state allowance."""
        return self.bytes

    def consume(self, field, amount):
        """Accumulate one synthetic resource charge."""
        self.consumed[field] += amount

    def reserve_file(self, unused_path, unused_maximum):
        """Return a no-op fixture reservation for one file."""
        return FakeReservation()

    def reserve_bulk(self, unused_maximum):
        """Return a no-op fixture reservation for multi-file output."""
        return FakeReservation()

    def remove(self, path):
        """Remove a fixture path after validating it is not a symlink."""
        path = Path(path)
        if path.is_symlink():
            raise AUDIT.AuditError("state_symlink_present")
        if path.is_dir():
            shutil.rmtree(path)
        elif path.exists():
            path.unlink()

    def assert_state_budget(self):
        """Treat fixture state as inside its configured bound."""

    def write_bytes(self, path, value):
        """Write fixture bytes through the production private writer."""
        AUDIT.safe_write_bytes(path, value)

    def write_json(self, path, value):
        """Write fixture JSON through the production private writer."""
        AUDIT.safe_write_json(path, value)


class FakeHeaders(dict):
    pass


class InventoryApi:
    def __init__(self, inventories):
        self.inventories = list(inventories)

    def inventory(self, suffix, field):
        current = self.inventories[0]
        result = current[field]
        if suffix == "/actions/artifacts":
            self.inventories.pop(0)
        return result


class StableFinalApi:
    """Return repository metadata and the fixture's empty stable inventory."""

    def __init__(self, unused_token, unused_budget):
        """Accept the production API constructor arguments."""

    def json(self, unused_suffix):
        """Return metadata bound by the default test manifest."""
        return (
            {
                "id": AUDIT.REPOSITORY_ID,
                "full_name": AUDIT.REPOSITORY,
                "clone_url": "https://github.com/BaryonOS/BaryonOS.git",
                "private": False,
            },
            FakeHeaders(),
        )

    def inventory(self, unused_suffix, unused_field):
        """Return one side of the repeated stable empty inventory."""
        return []


class Response(io.BytesIO):
    def __init__(self, body=b"", *, status=200, headers=None):
        super().__init__(body)
        self.status = status
        self.headers = FakeHeaders(headers or {})

    def __enter__(self):
        return self

    def __exit__(self, *unused):
        self.close()


class RequestSequence:
    def __init__(self, responses):
        self.responses = list(responses)
        self.requests = []

    def open(self, request, timeout=None):
        self.requests.append(request)
        return self.responses.pop(0)


def private_temp(testcase):
    temporary = tempfile.TemporaryDirectory(prefix="publication-audit-test-")
    testcase.addCleanup(temporary.cleanup)
    path = Path(temporary.name)
    os.chmod(path, 0o700)
    return path


def write_clean_receipt(state, value, item, *, findings=None, gaps=None, unavailable=None):
    findings = [] if findings is None else findings
    work = AUDIT.resource_work(state, item["key"])
    receipt = {
        **AUDIT.receipt_base(value, item),
        "content_scanned": unavailable is None,
        "finding_count": len(findings) if unavailable is None else 0,
        "rules": AUDIT.compact_findings(findings)["rules"] if unavailable is None else {},
        "coverage_gaps": list(
            gaps
            or (
                []
                if unavailable is None
                else ["expired_artifact_at_cutoff" if unavailable == "expired_at_snapshot" else "resource_unavailable"]
            )
        ),
    }
    if unavailable is None:
        AUDIT.safe_write_json(work / "findings.json", findings)
        receipt.update(
            {
                "scanner_exit_code": 0 if not findings else 1,
                "report_relpath": "findings.json",
                "report_sha256": AUDIT.file_hash(work / "findings.json"),
            }
        )
        if item["key"].startswith(("run-", "artifact-")):
            AUDIT.safe_write_bytes(work / "payload.zip", b"payload")
            receipt.update(
                {
                    "payload_relpath": "payload.zip",
                    "payload_sha256": AUDIT.file_hash(work / "payload.zip"),
                    "download_bytes": 7,
                    "archive": {
                        "entries": 1,
                        "expanded_bytes": 7,
                        "non_utf8_or_nul_members": 0,
                        "nested_archive_members": 0,
                        "encrypted_members": 0,
                        "unsupported_members": 0,
                    },
                }
            )
        elif item["key"] == "git-objects":
            AUDIT.safe_write_bytes(work / "reachable-objects.batch", b"objects")
            receipt.update(
                {
                    "payload_relpath": "reachable-objects.batch",
                    "payload_sha256": AUDIT.file_hash(work / "reachable-objects.batch"),
                    "object_counts": {
                        "objects": 1,
                        "object_bytes": 7,
                        "non_utf8_or_nul_blobs": 0,
                        "non_utf8_or_nul_commits": 0,
                        "non_utf8_or_nul_tags": 0,
                    },
                }
            )
        elif item["key"] == "git-metadata":
            receipt["metadata_counts"] = {
                "historical_paths": 0,
                "historical_archive_paths": 0,
                "lfs_pointer_blobs": 0,
                "submodules_seen": False,
                "non_utf8_paths": 0,
            }
        retained_paths = AUDIT.RETAINED_PATHS.get(item["key"])
        if retained_paths is not None:
            for relative in retained_paths:
                if relative != "findings.json":
                    AUDIT.safe_write_bytes(work / relative, (relative + "\n").encode())
            receipt["retained_files"] = AUDIT.retained_files_binding(work, retained_paths)
    else:
        receipt["unavailable_reason"] = unavailable
    AUDIT.write_receipt(state, item["key"], receipt)
    return receipt


class ManifestContractTests(AuditTestCase):
    """PA-SNAPSHOT: schema v2 snapshot bindings must be canonical and immutable.

    Configuration: mocked API/Git with the pinned Gitleaks binding and no live
    network. Expected: exact repository/cutoff/HEAD/ref/inventory/limit binding.
    Failure: a stable AuditError code within the host-unit-test timeout.
    """
    def metadata(self, *, private=False, identifier=AUDIT.REPOSITORY_ID):
        return {
            "id": identifier,
            "full_name": AUDIT.REPOSITORY,
            "clone_url": "https://github.com/BaryonOS/BaryonOS.git",
            "private": private,
        }

    def test_canonical_hash_is_key_order_independent(self):
        self.assertEqual(AUDIT.object_hash({"b": 2, "a": 1}), AUDIT.object_hash({"a": 1, "b": 2}))

    def test_compact_findings_does_not_expose_sensitive_fields(self):
        compact = AUDIT.compact_findings(
            [{"RuleID": "token", "File": "secret.env", "Secret": "do-not-print", "Fingerprint": "private"}]
        )
        self.assertEqual(compact, {"finding_count": 1, "rules": {"token": 1}})
        self.assertNotIn("secret.env", json.dumps(compact))
        self.assertNotIn("do-not-print", json.dumps(compact))

    def test_public_repository_requires_explicit_local_route(self):
        metadata = self.metadata()
        with self.assertRaisesRegex(AUDIT.AuditError, "repository_visibility_rejected"):
            AUDIT.verify_repository(metadata, allow_public_local=False)
        self.assertEqual(
            AUDIT.verify_repository(metadata, allow_public_local=True)["visibility"],
            "public-local-explicit",
        )

    def test_public_repository_is_rejected_inside_actions(self):
        metadata = self.metadata()
        with mock.patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}):
            with self.assertRaisesRegex(AUDIT.AuditError, "public_actions_execution_rejected"):
                AUDIT.verify_repository(metadata, allow_public_local=True)

    def test_private_repository_does_not_need_public_override(self):
        metadata = self.metadata(private=True)
        self.assertEqual(AUDIT.verify_repository(metadata, allow_public_local=False)["visibility"], "private")

    def test_repository_id_and_name_are_both_bound(self):
        metadata = self.metadata(private=True, identifier=AUDIT.REPOSITORY_ID + 1)
        with self.assertRaisesRegex(AUDIT.AuditError, "repository_identity_mismatch"):
            AUDIT.verify_repository(metadata, allow_public_local=False)

    def test_repository_transfer_requires_current_canonical_name(self):
        metadata = self.metadata(private=True)
        metadata["full_name"] = "simonvanieperen/BaryonOS"
        metadata["clone_url"] = "https://github.com/simonvanieperen/BaryonOS.git"
        with self.assertRaisesRegex(AUDIT.AuditError, "repository_identity_mismatch"):
            AUDIT.verify_repository(metadata, allow_public_local=False)

    def test_cutoff_excludes_only_strictly_later_resources(self):
        cutoff = AUDIT.parse_time("2026-11-10T02:00:00Z")
        inventory = AUDIT.normalized_inventory(
            [run(1), run(2, created_at="2026-11-10T02:00:01Z")],
            [artifact(3), artifact(4, created_at="2026-11-10T03:00:00Z")],
            cutoff,
        )
        self.assertEqual([item["id"] for item in inventory["runs"]], [1])
        self.assertEqual([item["id"] for item in inventory["artifacts"]], [3])

    def test_active_run_is_blocking_but_prior_attempt_is_enumerated(self):
        inventory = {
            "runs": [run(7, run_attempt=3, status="in_progress", conclusion=None)],
            "artifacts": [],
        }
        resources, blockers = AUDIT.inventory_resources(inventory)
        self.assertEqual([item["key"] for item in resources], ["run-7-attempt-1", "run-7-attempt-2"])
        self.assertEqual(blockers, ["active_run_at_cutoff"])

    def test_expired_artifact_remains_in_manifest_and_blocks(self):
        inventory = {"runs": [], "artifacts": [artifact(expired=True)]}
        resources, blockers = AUDIT.inventory_resources(inventory)
        self.assertEqual(resources[0]["key"], "artifact-9")
        self.assertTrue(resources[0]["identity"]["expired"])
        self.assertEqual(blockers, ["expired_artifact_at_cutoff"])

    def test_manifest_resource_order_is_deterministic(self):
        inventory = {
            "runs": [run(20), run(3)],
            "artifacts": [artifact(30), artifact(2)],
        }
        keys = [item["key"] for item in manifest(inventory)["resources"]]
        self.assertEqual(
            keys,
            ["git-history", "git-objects", "git-metadata", "run-20-attempt-1", "run-3-attempt-1",
             "artifact-2", "artifact-30"],
        )

    def test_manifest_round_trip_validates(self):
        value = manifest({"runs": [run()], "artifacts": [artifact()]})
        self.assertIs(AUDIT.validate_manifest(value), value)

    def test_manifest_resource_tampering_is_rejected(self):
        value = manifest()
        value["resources"][0]["identity"]["head"] = "f" * 40
        with self.assertRaisesRegex(AUDIT.AuditError, "manifest_semantics_invalid"):
            AUDIT.validate_manifest(value)

    def test_manifest_scope_cannot_be_upgraded_to_clearance(self):
        value = manifest()
        value["scope"]["publication_clearance"] = True
        with self.assertRaisesRegex(AUDIT.AuditError, "manifest_semantics_invalid"):
            AUDIT.validate_manifest(value)

    def test_invalid_run_attempt_is_rejected(self):
        with self.assertRaisesRegex(AUDIT.AuditError, "run_attempt_invalid"):
            AUDIT.run_identity(run(run_attempt=0))

    def test_server_cutoff_requires_a_date_header(self):
        with self.assertRaisesRegex(AUDIT.AuditError, "api_date_header_invalid"):
            AUDIT.server_cutoff(FakeHeaders())

    def test_two_inventory_passes_must_match(self):
        first = {"workflow_runs": [run()], "artifacts": []}
        second = {"workflow_runs": [run(updated_at="2026-11-10T01:03:00Z")], "artifacts": []}
        api = InventoryApi([first, second])
        with self.assertRaisesRegex(AUDIT.AuditError, "actions_snapshot_changed"):
            AUDIT.stable_inventory(api, AUDIT.parse_time("2026-11-10T02:00:00Z"))

    def test_post_cutoff_addition_does_not_change_snapshot_scope(self):
        first = {"workflow_runs": [run()], "artifacts": []}
        second = {
            "workflow_runs": [run(), run(8, created_at="2026-11-10T02:00:01Z")],
            "artifacts": [],
        }
        api = InventoryApi([first, second])
        value = AUDIT.stable_inventory(api, AUDIT.parse_time("2026-11-10T02:00:00Z"))
        self.assertEqual([item["id"] for item in value["runs"]], [7])

    def test_origin_url_accepts_renamed_repository_alias_but_not_other_hosts(self):
        self.assertEqual(
            AUDIT.origin_repository_name("https://github.com/simonvanieperen/nieuw-os.git"),
            "simonvanieperen/nieuw-os",
        )
        self.assertEqual(
            AUDIT.origin_repository_name("https://github.com/simonvanieperen/BaryonOS.git"),
            "simonvanieperen/BaryonOS",
        )
        self.assertEqual(
            AUDIT.origin_repository_name("git@github.com:BaryonOS/BaryonOS.git"),
            AUDIT.REPOSITORY,
        )
        with self.assertRaisesRegex(AUDIT.AuditError, "origin_url_invalid"):
            AUDIT.origin_repository_name("https://example.invalid/BaryonOS/BaryonOS.git")

    def test_advertised_ref_inventory_rejects_duplicates(self):
        raw = (b"1" * 40 + b"\trefs/heads/main\n") * 2
        with self.assertRaisesRegex(AUDIT.AuditError, "remote_ref_inventory_invalid"):
            AUDIT.parse_advertised_refs(raw)

    def test_remote_ref_inventory_must_be_stable(self):
        first = [{"name": "refs/heads/main", "oid": "1" * 40}]
        second = [{"name": "refs/heads/main", "oid": "2" * 40}]
        with mock.patch.object(AUDIT, "advertised_refs", side_effect=[first, second]):
            with self.assertRaisesRegex(AUDIT.AuditError, "remote_refs_changed_during_snapshot"):
                AUDIT.stable_advertised_refs(
                    repository()["canonical_clone_url"], mock.Mock(), private=False
                )

    def test_private_advertised_refs_use_scoped_askpass(self):
        """Invariant: private ls-remote exposes the token only to transient askpass."""
        secret = "private-sentinel-token"
        captured = {}

        def command(arguments, unused_budget, **options):
            environment = options["environment"]
            helper = Path(environment["GIT_ASKPASS"])
            captured.update(
                arguments=list(arguments),
                helper=helper,
                helper_bytes=helper.read_bytes(),
                helper_mode=stat.S_IMODE(helper.stat().st_mode),
                home=Path(environment["HOME"]),
                environment=dict(environment),
                username=subprocess.check_output(
                    [str(helper), "Username for https://github.com"], env=environment, text=True
                ).strip(),
                password=subprocess.check_output(
                    [str(helper), "Password for https://github.com"], env=environment, text=True
                ).strip(),
            )
            return b"1" * 40 + b"\trefs/heads/main\n"

        with (
            mock.patch.dict(
                os.environ,
                {"GH_TOKEN": secret, "GITHUB_TOKEN": "must-not-pass", "GIT_ASKPASS": "/unsafe"},
            ),
            mock.patch.object(AUDIT, "run_command", side_effect=command),
        ):
            refs = AUDIT.advertised_refs(repository("private")["canonical_clone_url"], mock.Mock(), private=True)
        self.assertEqual(refs[0]["name"], "refs/heads/main")
        self.assertEqual(captured["username"], "x-access-token")
        self.assertEqual(captured["password"], secret)
        self.assertEqual(captured["helper_mode"], 0o700)
        self.assertNotIn(secret.encode(), captured["helper_bytes"])
        self.assertNotIn(secret, " ".join(captured["arguments"]))
        self.assertNotIn("GH_TOKEN", captured["environment"])
        self.assertNotIn("GITHUB_TOKEN", captured["environment"])
        self.assertEqual(captured["environment"][AUDIT.ASKPASS_TOKEN_ENV], secret)
        self.assertFalse(captured["helper"].exists())
        self.assertFalse(captured["home"].exists())

    def test_private_advertised_refs_require_credential(self):
        """Invariant: the private route fails before Git when GH_TOKEN is absent."""
        with (
            mock.patch.dict(os.environ, {"GH_TOKEN": ""}),
            mock.patch.object(AUDIT, "run_command") as command,
            self.assertRaisesRegex(AUDIT.AuditError, "missing_git_credential_environment"),
        ):
            AUDIT.advertised_refs(repository("private")["canonical_clone_url"], mock.Mock(), private=True)
        command.assert_not_called()

    def test_public_advertised_refs_scrub_inherited_credentials(self):
        """Invariant: public ls-remote receives no inherited token or askpass helper."""
        captured = {}
        temporary = {}

        def command(unused_arguments, unused_budget, **options):
            captured.update(options["environment"])
            temporary["home"] = Path(captured["HOME"])
            temporary["existed"] = temporary["home"].is_dir()
            return b"1" * 40 + b"\trefs/heads/main\n"

        with (
            mock.patch.dict(
                os.environ,
                {"GH_TOKEN": "secret-a", "GITHUB_TOKEN": "secret-b", "GIT_ASKPASS": "/unsafe"},
            ),
            mock.patch.object(AUDIT, "run_command", side_effect=command),
        ):
            AUDIT.advertised_refs(repository()["canonical_clone_url"], mock.Mock(), private=False)
        self.assertNotIn("GH_TOKEN", captured)
        self.assertNotIn("GITHUB_TOKEN", captured)
        self.assertNotIn(AUDIT.ASKPASS_TOKEN_ENV, captured)
        self.assertNotEqual(captured["GIT_ASKPASS"], "/unsafe")
        self.assertEqual(captured["GIT_NO_REPLACE_OBJECTS"], "1")
        self.assertTrue(temporary["existed"])
        self.assertFalse(temporary["home"].exists())

    def test_git_snapshot_binds_alias_redirect_and_advertised_tips(self):
        commands = {
            ("git", "rev-parse", "--is-shallow-repository"): b"false\n",
            ("git", "rev-parse", "HEAD"): ("1" * 40 + "\n").encode(),
            ("git", "remote", "get-url", "origin"): b"https://github.com/simonvanieperen/nieuw-os.git\n",
        }

        def command(arguments, unused_budget, **unused):
            if arguments[:2] == ["git", "cat-file"]:
                return ("1" * 40 + " commit\n").encode()
            return commands[tuple(arguments)]

        api = mock.Mock()
        api.repository_alias.return_value = {
            "id": AUDIT.REPOSITORY_ID,
            "full_name": AUDIT.REPOSITORY,
            "clone_url": repository()["canonical_clone_url"],
        }
        refs = [{"name": "refs/heads/main", "oid": "1" * 40}]
        with (
            mock.patch.object(AUDIT, "run_command", side_effect=command),
            mock.patch.object(AUDIT, "stable_advertised_refs", return_value=refs),
        ):
            result = AUDIT.git_snapshot(ROOT, mock.Mock(), repository(), api, "1" * 40)
        self.assertEqual(result["refs"], refs)
        self.assertEqual(result["origin"]["configured_repository"], "simonvanieperen/nieuw-os")
        api.repository_alias.assert_called_once_with("simonvanieperen/nieuw-os")

    def test_git_snapshot_rejects_unfetched_remote_tip(self):
        outputs = iter(
            [
                b"false\n",
                ("1" * 40 + "\n").encode(),
                b"https://github.com/BaryonOS/BaryonOS.git\n",
                ("1" * 40 + " missing\n").encode(),
            ]
        )
        api = mock.Mock()
        api.repository_alias.return_value = {
            "id": AUDIT.REPOSITORY_ID,
            "full_name": AUDIT.REPOSITORY,
            "clone_url": repository()["canonical_clone_url"],
        }
        refs = [{"name": "refs/heads/main", "oid": "1" * 40}]
        with (
            mock.patch.object(AUDIT, "run_command", side_effect=lambda *args, **kwargs: next(outputs)),
            mock.patch.object(AUDIT, "stable_advertised_refs", return_value=refs),
        ):
            with self.assertRaisesRegex(AUDIT.AuditError, "remote_ref_tip_missing_locally"):
                AUDIT.git_snapshot(ROOT, mock.Mock(), repository(), api)

    def test_git_snapshot_requires_head_to_be_an_advertised_tip(self):
        """Invariant: a local descendant or detached commit is outside the snapshot."""
        head = "1" * 40
        advertised = "2" * 40
        outputs = iter(
            [
                b"false\n",
                (head + "\n").encode(),
                b"https://github.com/BaryonOS/BaryonOS.git\n",
                (advertised + " commit\n").encode(),
            ]
        )
        api = mock.Mock()
        api.repository_alias.return_value = {
            "id": AUDIT.REPOSITORY_ID,
            "full_name": AUDIT.REPOSITORY,
            "clone_url": repository()["canonical_clone_url"],
        }
        refs = [{"name": "refs/heads/main", "oid": advertised}]
        with (
            mock.patch.object(AUDIT, "run_command", side_effect=lambda *args, **kwargs: next(outputs)),
            mock.patch.object(AUDIT, "stable_advertised_refs", return_value=refs),
            self.assertRaisesRegex(AUDIT.AuditError, "source_head_not_advertised"),
        ):
            AUDIT.git_snapshot(ROOT, mock.Mock(), repository(), api)

    def test_git_replacement_refs_cannot_change_audited_object_views(self):
        """Invariant: every local Git execution ignores attacker-controlled replacements."""
        state = private_temp(self)
        repository_path = state / "repo"
        repository_path.mkdir(mode=0o700)
        fixture_environment = {
            **os.environ,
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_TERMINAL_PROMPT": "0",
        }
        fixture_environment.pop("GIT_NO_REPLACE_OBJECTS", None)
        subprocess.run(["git", "init", "--quiet"], cwd=repository_path, env=fixture_environment, check=True)
        for key, value in (
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
        ):
            subprocess.run(
                ["git", "config", key, value],
                cwd=repository_path,
                env=fixture_environment,
                check=True,
            )
        tracked = repository_path / "tracked.txt"
        tracked.write_text("original\n")
        subprocess.run(["git", "add", "tracked.txt"], cwd=repository_path, env=fixture_environment, check=True)
        subprocess.run(
            ["git", "commit", "--quiet", "-m", "original"],
            cwd=repository_path,
            env=fixture_environment,
            check=True,
        )
        original = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=repository_path, env=fixture_environment, text=True
        ).strip()
        tracked.write_text("replacement\n")
        subprocess.run(["git", "add", "tracked.txt"], cwd=repository_path, env=fixture_environment, check=True)
        subprocess.run(
            ["git", "commit", "--quiet", "-m", "replacement"],
            cwd=repository_path,
            env=fixture_environment,
            check=True,
        )
        replacement = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=repository_path, env=fixture_environment, text=True
        ).strip()
        subprocess.run(
            ["git", "replace", original, replacement],
            cwd=repository_path,
            env=fixture_environment,
            check=True,
        )
        visible = subprocess.check_output(
            ["git", "log", "-1", "--format=%s", original],
            cwd=repository_path,
            env=fixture_environment,
            text=True,
        ).strip()
        self.assertEqual(visible, "replacement")
        budget = FakeBudget(state)
        audited = AUDIT.run_command(
            ["git", "log", "-1", "--format=%s", original],
            budget,
            cwd=repository_path,
        ).decode().strip()
        self.assertEqual(audited, "original")
        output = state / "git-output.txt"
        AUDIT.run_command_to_file(
            ["git", "log", "-1", "--format=%s", original],
            budget,
            output,
            cwd=repository_path,
            input_bytes=b"",
            maximum=4096,
        )
        self.assertEqual(output.read_text().strip(), "original")

    def test_inventory_rejects_duplicate_and_truncated_rows(self):
        api = object.__new__(AUDIT.GitHubApi)
        api.json = mock.Mock(return_value=({"total_count": 2, "artifacts": [{"id": 1}, {"id": 1}]}, None))
        with self.assertRaisesRegex(AUDIT.AuditError, "inventory_id_invalid"):
            api.inventory("/actions/artifacts", "artifacts")
        api.json = mock.Mock(return_value=({"total_count": 2, "artifacts": [{"id": 1}]}, None))
        with self.assertRaisesRegex(AUDIT.AuditError, "inventory_count_mismatch"):
            api.inventory("/actions/artifacts", "artifacts")


class StateAndBudgetTests(AuditTestCase):
    """PA-STATE: private single-writer state and monotonic bounds fail closed.

    Configuration: temporary host filesystems under schema v2 limits. Expected:
    modes 0700/0600, bounded linear accounting and persisted consumption.
    Failure: mutation, overlap, exhaustion or clock ambiguity raises AuditError.
    """
    def test_state_directory_symlink_is_rejected(self):
        parent = private_temp(self)
        target = parent / "target"
        target.mkdir(mode=0o700)
        link = parent / "link"
        link.symlink_to(target, target_is_directory=True)
        with self.assertRaisesRegex(AUDIT.AuditError, "state_directory_symlink"):
            with AUDIT.StateLock(link):
                pass

    def test_broad_state_permissions_are_rejected(self):
        state = private_temp(self) / "state"
        state.mkdir(mode=0o700)
        broad = mock.Mock(st_mode=stat.S_IFDIR | 0o710)
        with mock.patch.object(Path, "lstat", return_value=broad):
            with self.assertRaisesRegex(AUDIT.AuditError, "state_directory_permissions"):
                with AUDIT.StateLock(state):
                    pass

    def test_second_writer_is_rejected(self):
        state = private_temp(self) / "state"
        with AUDIT.StateLock(state):
            with self.assertRaisesRegex(AUDIT.AuditError, "state_locked"):
                with AUDIT.StateLock(state):
                    pass

    def test_request_exhaustion_is_persistently_fail_closed(self):
        state = private_temp(self)
        configured = limits(max_total_requests=1)
        ledger = AUDIT.UsageLedger(state, configured)
        ledger.consume("requests", 1)
        with self.assertRaisesRegex(AUDIT.AuditError, "max_total_requests_exhausted"):
            ledger.consume("requests", 1)
        reloaded = AUDIT.UsageLedger(state, configured)
        with self.assertRaisesRegex(AUDIT.AuditError, "max_total_requests_exhausted"):
            reloaded.start()

    def test_usage_is_bound_to_the_manifest_limits(self):
        state = private_temp(self)
        AUDIT.UsageLedger(state, limits())
        with self.assertRaisesRegex(AUDIT.AuditError, "usage_limit_binding_mismatch"):
            AUDIT.UsageLedger(state, limits(max_total_requests=99_999))

    def test_invalid_limit_relationship_is_rejected(self):
        configured = limits(max_command_seconds=100, max_total_seconds=10)
        with self.assertRaisesRegex(AUDIT.AuditError, "command_limit_exceeds_total"):
            AUDIT.validate_limits(configured)

    def test_crashed_active_interval_is_charged_on_resume(self):
        state = private_temp(self)
        ledger = AUDIT.UsageLedger(state, limits())
        ledger.value["active_started_at"] = 100.0
        ledger._save()
        with mock.patch.object(AUDIT.time, "time", return_value=105.0):
            reloaded = AUDIT.UsageLedger(state, limits())
            reloaded.start()
        self.assertGreaterEqual(reloaded.value["elapsed_seconds"], 5.0)

    def test_failure_record_never_claims_success_or_clearance(self):
        result = AUDIT.failure("finalize", "receipt_missing")
        self.assertFalse(result["phase_complete"])
        self.assertFalse(result["audit_complete"])
        self.assertFalse(result["snapshot_audit_passed"])
        self.assertFalse(result["publication_clearance"])

    def test_negative_usage_counter_is_rejected(self):
        state = private_temp(self)
        ledger = AUDIT.UsageLedger(state, limits())
        ledger.value["download_bytes"] = -1
        AUDIT.safe_write_json(ledger.path, ledger.value)
        with self.assertRaisesRegex(AUDIT.AuditError, "usage_counter_invalid"):
            AUDIT.UsageLedger(state, limits())

    def test_nonfinite_usage_time_is_rejected(self):
        state = private_temp(self)
        ledger = AUDIT.UsageLedger(state, limits())
        raw = ledger.path.read_text().replace('"elapsed_seconds": 0.0', '"elapsed_seconds": NaN')
        ledger.path.write_text(raw)
        with self.assertRaisesRegex(AUDIT.AuditError, "json_nonfinite_number"):
            AUDIT.UsageLedger(state, limits())

    def test_duplicate_state_json_key_is_rejected(self):
        with self.assertRaisesRegex(AUDIT.AuditError, "json_duplicate_key"):
            AUDIT.strict_json(b'{"value":1,"value":2}')

    def test_atomic_writer_refuses_bytes_beyond_state_cap_before_write(self):
        state = private_temp(self)
        destination = state / "too-large.bin"
        accountant = AUDIT.StateAccountant(state, 3)
        with self.assertRaisesRegex(AUDIT.AuditError, "max_total_state_bytes_exhausted"):
            AUDIT.safe_write_bytes(destination, b"1234", accountant=accountant)
        self.assertFalse(destination.exists())

    def test_replacement_reserves_old_and_new_bytes_then_commits_exact_size(self):
        """Invariant: replacement transient bytes fit before retained usage changes."""
        state = private_temp(self)
        destination = state / "replace.bin"
        destination.write_bytes(b"old!")
        accountant = AUDIT.StateAccountant(state, 7)
        AUDIT.safe_write_bytes(destination, b"new", accountant=accountant)
        self.assertEqual(destination.read_bytes(), b"new")
        self.assertEqual((accountant.used, accountant.reserved), (3, 0))

    def test_replacement_over_transient_limit_preserves_original(self):
        """Invariant: old plus temporary replacement bytes are bounded atomically."""
        state = private_temp(self)
        destination = state / "replace.bin"
        destination.write_bytes(b"old!")
        accountant = AUDIT.StateAccountant(state, 6)
        with self.assertRaisesRegex(AUDIT.AuditError, "max_total_state_bytes_exhausted"):
            AUDIT.safe_write_bytes(destination, b"new", accountant=accountant)
        self.assertEqual(destination.read_bytes(), b"old!")
        self.assertEqual((accountant.used, accountant.reserved), (4, 0))

    def test_failed_atomic_replace_cancels_reservation_after_cleanup(self):
        """Invariant: a failed rename retains old bytes and releases no phantom state."""
        state = private_temp(self)
        destination = state / "replace.bin"
        destination.write_bytes(b"old")
        accountant = AUDIT.StateAccountant(state, 32)
        with (
            mock.patch.object(AUDIT.os, "replace", side_effect=OSError("fixture rename failure")),
            self.assertRaisesRegex(OSError, "fixture rename failure"),
        ):
            AUDIT.safe_write_bytes(destination, b"replacement", accountant=accountant)
        self.assertEqual(destination.read_bytes(), b"old")
        self.assertFalse(destination.with_name("replace.bin.tmp").exists())
        self.assertEqual((accountant.used, accountant.reserved), (3, 0))

    def test_bulk_reservation_commits_exact_bytes_and_can_cancel_cleanly(self):
        """Invariant: multi-file work retains its exact size or releases all capacity."""
        state = private_temp(self)
        accountant = AUDIT.StateAccountant(state, 16)
        committed = accountant.reserve_bulk(8)
        (state / "a.bin").write_bytes(b"abc")
        (state / "b.bin").write_bytes(b"de")
        committed.commit(5)
        cancelled = accountant.reserve_bulk(4)
        cancelled.cancel_clean()
        self.assertEqual((accountant.used, accountant.reserved), (5, 0))
        accountant.reconcile()
        self.assertEqual(accountant.used, 5)

    def test_many_accounted_writes_do_not_rescan_the_state_tree(self):
        """Invariant: retained-state accounting remains O(1) after one measurement."""
        state = private_temp(self)
        with mock.patch.object(AUDIT, "directory_size", wraps=AUDIT.directory_size) as measure:
            accountant = AUDIT.StateAccountant(state, 1024 * 1024)
            ledger = AUDIT.UsageLedger(
                state,
                limits(max_total_state_bytes=1024 * 1024),
                accountant=accountant,
            )
            for unused in range(32):
                ledger.consume("requests", 0)
            for index in range(128):
                AUDIT.safe_write_bytes(
                    state / f"entry-{index}.bin",
                    bytes([index % 251]) * (index % 17 + 1),
                    accountant=accountant,
                )
            for index in range(128):
                AUDIT.safe_write_bytes(
                    state / f"entry-{index}.bin",
                    b"xy",
                    accountant=accountant,
                )
            self.assertEqual(measure.call_count, 1)
        self.assertEqual(accountant.used, AUDIT.directory_size(state))
        self.assertEqual(accountant.reserved, 0)

    def test_audit_state_must_not_overlap_repository(self):
        with self.assertRaisesRegex(AUDIT.AuditError, "state_repository_overlap"):
            AUDIT.reject_repository_state_overlap(ROOT, ROOT / "build/private-audit")
        with self.assertRaisesRegex(AUDIT.AuditError, "state_repository_overlap"):
            AUDIT.reject_repository_state_overlap(ROOT, ROOT.parent)

    def test_command_deadline_overrun_cannot_return_success(self):
        state = private_temp(self)
        ledger = AUDIT.UsageLedger(state, limits())
        with self.assertRaisesRegex(AUDIT.AuditError, "command_time_budget_exhausted"):
            with AUDIT.Budget(ledger) as budget:
                budget.deadline = AUDIT.time.monotonic() - 1

    def test_total_deadline_overrun_is_persisted_and_raised(self):
        state = private_temp(self)
        configured = limits(max_total_seconds=1, max_command_seconds=1)
        ledger = AUDIT.UsageLedger(state, configured)
        ledger.value["active_started_at"] = 100.0
        ledger._save()
        with mock.patch.object(AUDIT.time, "time", return_value=102.0):
            with self.assertRaisesRegex(AUDIT.AuditError, "total_time_budget_exhausted"):
                ledger.finish()
        reloaded = AUDIT.UsageLedger(state, configured)
        self.assertEqual(reloaded.value["exhausted"], "total_time_budget_exhausted")


class ContentBoundaryTests(AuditTestCase):
    """PA-CONTENT: only completely bounded and classified bytes can scan clean.

    Configuration: synthetic ZIP/HTTP/Git/scanner inputs with pinned limits.
    Expected: exact byte classes and counts; failure is a gap or AuditError.
    Evidence is host-only and bounded by the unittest process timeout.
    """

    def setUp(self):
        """Create isolated content state and a deterministic fake budget."""
        super().setUp()
        self.state = private_temp(self)
        self.budget = FakeBudget(self.state)

    def make_zip(self, name, body=b"content"):
        path = self.state / "payload.zip"
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr(name, body)
        return path

    def test_commit_and_path_text_classifier_rejects_invalid_utf8_and_nul(self):
        """Invariant: commit/path bytes outside scanner text scope remain gaps."""
        self.assertTrue(AUDIT.utf8_without_nul("valid/π.txt".encode()))
        self.assertFalse(AUDIT.utf8_without_nul(b"invalid-\xff-name"))
        self.assertFalse(AUDIT.utf8_without_nul(b"commit\0message"))

    def test_archive_paths_are_flattened_and_never_followed(self):
        source_path = self.make_zip("../../outside.txt")
        target = self.state / "expanded"
        result = AUDIT.extract_zip(source_path, target, self.budget)
        self.assertEqual(result["entries"], 1)
        self.assertEqual((target / "0.data").read_bytes(), b"content")
        self.assertFalse((self.state.parent / "outside.txt").exists())

    def test_nested_archive_is_a_terminal_coverage_gap(self):
        source_path = self.make_zip("inside.zip", b"not-even-a-valid-nested-archive")
        result = AUDIT.extract_zip(source_path, self.state / "expanded", self.budget)
        self.assertEqual(result["nested_archive_members"], 1)

    def test_declared_expansion_limit_is_enforced_before_extraction(self):
        source_path = self.make_zip("large.txt", b"ab")
        with mock.patch.object(AUDIT, "MAX_RESOURCE_EXPANDED", 1):
            with self.assertRaisesRegex(AUDIT.AuditError, "resource_expansion_limit"):
                AUDIT.extract_zip(source_path, self.state / "expanded", self.budget)

    def test_aggregate_entry_budget_is_charged(self):
        source_path = self.make_zip("one.txt")
        AUDIT.extract_zip(source_path, self.state / "expanded", self.budget)
        self.assertEqual(self.budget.consumed["entries"], 1)

    def test_stale_scanner_report_is_deleted_before_execution(self):
        """Invariant: scanner execution is clean, bounded and replacement-disabled."""
        scanner = self.state / "fake-gitleaks"
        scanner.write_text(
            "#!/usr/bin/env python3\n"
            "import os, pathlib, sys\n"
            "if os.environ.get('GIT_NO_REPLACE_OBJECTS') != '1': sys.exit(7)\n"
            "path = pathlib.Path(sys.argv[sys.argv.index('--report-path') + 1])\n"
            "path.write_text('[]')\n"
        )
        scanner.chmod(0o700)
        work = self.state / "scan"
        work.mkdir(mode=0o700)
        (work / "findings.json").write_text('[{"RuleID":"stale"}]')
        result = AUDIT.run_scanner(scanner, "dir", self.state, work / "findings.json", self.budget)
        self.assertEqual(result["finding_count"], 0)
        self.assertEqual(json.loads((work / "findings.json").read_text()), [])

    def test_api_authorization_is_not_forwarded_to_storage(self):
        api = object.__new__(AUDIT.GitHubApi)
        api.token = "private-token"
        api.budget = self.budget
        sequence = RequestSequence(
            [
                Response(status=302, headers={"Location": "https://safe.blob.core.windows.net/archive.zip"}),
                Response(b"zip-bytes", status=200, headers={"Content-Length": "9"}),
            ]
        )
        api.opener = sequence
        api.remaining = None
        status, size = api.download("/actions/artifacts/1/zip", self.state / "download.zip")
        self.assertEqual((status, size), (200, 9))
        api_headers = dict(sequence.requests[0].header_items())
        storage_headers = dict(sequence.requests[1].header_items())
        self.assertIn("Authorization", api_headers)
        self.assertNotIn("Authorization", storage_headers)

    def test_unapproved_storage_origin_is_rejected(self):
        api = object.__new__(AUDIT.GitHubApi)
        api.token = "private-token"
        api.budget = self.budget
        api.opener = RequestSequence(
            [Response(status=302, headers={"Location": "https://blob.core.windows.net.evil.invalid/a"})]
        )
        api.remaining = None
        with self.assertRaisesRegex(AUDIT.AuditError, "download_origin_unapproved"):
            api.download("/actions/artifacts/1/zip", self.state / "download.zip")

    def test_http_404_is_unavailable_not_a_clean_scan(self):
        item = {
            "key": "artifact-1",
            "endpoint": "/actions/artifacts/1/zip",
            "identity": {"kind": "artifact", "id": 1, "expired": False, "digest": None},
        }

        class MissingApi:
            """Return one deterministic unavailable artifact response."""

            def download(self, suffix, target):
                """Model an HTTP 404 without creating retained bytes."""
                return 404, 0

        result = AUDIT.scan_action_resource(
            item,
            self.state / "work",
            self.state / "unused",
            MissingApi(),
            self.budget,
        )
        self.assertFalse(result["content_scanned"])
        self.assertEqual(result["unavailable_reason"], "http_404")
        self.assertIn("resource_unavailable", result["coverage_gaps"])

    def test_nul_archive_member_is_a_terminal_coverage_gap(self):
        prepared = self.make_zip("binary.dat", b"prefix\0token-inside")
        payload = prepared.read_bytes()
        item = {
            "key": "artifact-1",
            "endpoint": "/actions/artifacts/1/zip",
            "identity": {"kind": "artifact", "id": 1, "expired": False, "digest": None},
        }

        class PayloadApi:
            def download(self, suffix, target):
                target.write_bytes(payload)
                return 200, len(payload)

        work = self.state / "action-work"
        work.mkdir(mode=0o700)
        scanner_result = {
            "content_scanned": True,
            "scanner_exit_code": 0,
            "finding_count": 0,
            "rules": {},
            "report_sha256": "0" * 64,
        }
        with mock.patch.object(AUDIT, "run_scanner", return_value=scanner_result):
            result = AUDIT.scan_action_resource(item, work, self.state / "unused", PayloadApi(), self.budget)
        self.assertIn("non_utf8_or_nul_archive_members", result["coverage_gaps"])
        self.assertEqual(result["archive"]["non_utf8_or_nul_members"], 1)

    def test_binary_reachable_git_blob_is_a_terminal_coverage_gap(self):
        """Invariant: mocked fixture Git config cannot alter binary-blob coverage."""
        repository_path = self.state / "repo"
        repository_path.mkdir(mode=0o700)
        fixture_env = {
            **os.environ,
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_TERMINAL_PROMPT": "0",
        }
        subprocess.run(["git", "init", "--quiet"], cwd=repository_path, env=fixture_env, check=True)
        subprocess.run(
            ["git", "config", "user.name", "Fixture"], cwd=repository_path, env=fixture_env, check=True
        )
        subprocess.run(
            ["git", "config", "user.email", "fixture@example.invalid"],
            cwd=repository_path,
            env=fixture_env,
            check=True,
        )
        subprocess.run(
            ["git", "config", "commit.gpgsign", "false"], cwd=repository_path, env=fixture_env, check=True
        )
        (repository_path / "text.txt").write_text("ordinary text\n")
        (repository_path / "binary.dat").write_bytes(b"prefix\0token-inside")
        subprocess.run(
            ["git", "add", "text.txt", "binary.dat"], cwd=repository_path, env=fixture_env, check=True
        )
        subprocess.run(
            ["git", "commit", "--quiet", "-m", "fixture"],
            cwd=repository_path,
            env=fixture_env,
            check=True,
        )
        head = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=repository_path, env=fixture_env, text=True
        ).strip()
        item = {
            "key": "git-objects",
            "endpoint": "local-git",
            "identity": {
                "kind": "git-objects",
                "head": head,
                "refs_sha256": "unused",
                "refs": [{"name": "refs/heads/main", "oid": head}],
                "tips": [head],
            },
        }
        work = self.state / "git-work"
        work.mkdir(mode=0o700)
        scanner_result = {
            "content_scanned": True,
            "scanner_exit_code": 0,
            "finding_count": 0,
            "rules": {},
            "report_sha256": "0" * 64,
        }
        with mock.patch.object(AUDIT, "run_scanner", return_value=scanner_result):
            result = AUDIT.scan_git_objects(
                item,
                repository_path,
                work,
                self.state / "unused",
                self.budget,
            )
        self.assertIn("non_utf8_or_nul_git_blobs", result["coverage_gaps"])
        self.assertEqual(result["object_counts"]["non_utf8_or_nul_blobs"], 1)

    def test_mocked_git_batch_parser_rejects_truncation(self):
        """Invariant: mocked process truncation cannot produce an LFS/clean count."""
        identifier = b"1" * 40
        truncated = identifier + b" blob 3\nabc"
        with mock.patch.object(AUDIT, "run_command", return_value=truncated):
            with self.assertRaisesRegex(AUDIT.AuditError, "git_blob_inventory_truncated"):
                AUDIT.parse_small_blobs(self.state, [identifier], self.budget)

    def test_mocked_git_batch_parser_rejects_misordered_object(self):
        """Invariant: mocked process reordering cannot bind bytes to another OID."""
        expected = b"1" * 40
        other = b"2" * 40
        misordered = other + b" blob 3\nabc\n"
        with mock.patch.object(AUDIT, "run_command", return_value=misordered):
            with self.assertRaisesRegex(AUDIT.AuditError, "git_blob_inventory_invalid"):
                AUDIT.parse_small_blobs(self.state, [expected], self.budget)

    def test_mocked_git_process_failure_is_not_a_clean_result(self):
        """Invariant: a non-zero mocked Git process becomes a bounded hard failure."""
        completed = mock.Mock(returncode=1, stdout=b"")
        with mock.patch.object(AUDIT.subprocess, "run", return_value=completed):
            with self.assertRaisesRegex(AUDIT.AuditError, "local_command_failed"):
                AUDIT.run_command(["git", "status"], self.budget)


class ReceiptAndFinalizationTests(AuditTestCase):
    """PA-FINALIZE: receipts bind reports/payloads/retained bytes before success.

    Configuration: schema v2 synthetic receipts and stable mocked inventories.
    Expected: every byte revalidates and publication_clearance remains false.
    Failure: any missing or mutated binding raises or keeps finalizable false.
    """

    def setUp(self):
        """Create one isolated manifest state for receipt/finalize contracts."""
        super().setUp()
        self.state = private_temp(self)
        self.value = manifest()
        AUDIT.safe_write_json(self.state / "manifest.json", self.value)

    def fill(self, **options):
        for item in self.value["resources"]:
            write_clean_receipt(self.state, self.value, item, **options)

    def test_missing_receipt_never_finalizes(self):
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["missing_receipts"], len(self.value["resources"]))
        self.assertFalse(current["finalizable"])

    def test_all_bound_clean_receipts_are_finalizable(self):
        self.fill()
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["receipts"], len(self.value["resources"]))
        self.assertTrue(current["finalizable"])

    def test_findings_keep_finalization_open(self):
        self.fill(findings=[{"RuleID": "generic-token", "Secret": "redacted"}])
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["finding_count"], len(self.value["resources"]))
        self.assertFalse(current["finalizable"])

    def test_coverage_gap_keeps_finalization_open(self):
        self.fill()
        item = next(item for item in self.value["resources"] if item["key"] == "git-metadata")
        receipt = AUDIT.load_json(AUDIT.receipt_path(self.state, item["key"]))
        receipt["metadata_counts"]["lfs_pointer_blobs"] = 1
        receipt["coverage_gaps"] = ["git_lfs_payloads"]
        AUDIT.write_receipt(self.state, item["key"], receipt)
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["coverage_gap_count"], 1)
        self.assertFalse(current["finalizable"])

    def test_unavailable_resource_keeps_finalization_open(self):
        self.value = manifest({"runs": [run()], "artifacts": []})
        AUDIT.safe_write_json(self.state / "manifest.json", self.value)
        for item in self.value["resources"]:
            if item["key"].startswith("run-"):
                write_clean_receipt(self.state, self.value, item, unavailable="http_410")
            else:
                write_clean_receipt(self.state, self.value, item)
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["unavailable_resources"], 1)
        self.assertFalse(current["finalizable"])

    def test_manifest_blocker_keeps_finalization_open(self):
        self.value = manifest({"runs": [], "artifacts": [artifact(expired=True)]})
        AUDIT.safe_write_json(self.state / "manifest.json", self.value)
        for item in self.value["resources"]:
            if item["key"].startswith("artifact-"):
                write_clean_receipt(self.state, self.value, item, unavailable="expired_at_snapshot")
            else:
                write_clean_receipt(self.state, self.value, item)
        current = AUDIT.coverage(self.value, self.state)
        self.assertEqual(current["manifest_blockers"], 1)
        self.assertFalse(current["finalizable"])

    def test_receipt_from_another_manifest_is_rejected(self):
        item = self.value["resources"][0]
        receipt = write_clean_receipt(self.state, self.value, item)
        receipt["manifest_sha256"] = "0" * 64
        AUDIT.write_receipt(self.state, item["key"], receipt)
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_binding_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_modified_report_is_rejected(self):
        item = self.value["resources"][0]
        write_clean_receipt(self.state, self.value, item)
        work = AUDIT.resource_work(self.state, item["key"])
        AUDIT.safe_write_json(work / "findings.json", [{"RuleID": "late-change"}])
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_report_binding_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_modified_payload_is_rejected(self):
        self.value = manifest({"runs": [run()], "artifacts": []})
        item = next(item for item in self.value["resources"] if item["key"].startswith("run-"))
        write_clean_receipt(self.state, self.value, item)
        work = AUDIT.resource_work(self.state, item["key"])
        AUDIT.safe_write_bytes(work / "payload.zip", b"changed")
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_payload_binding_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_modified_git_history_work_byte_is_rejected(self):
        """Invariant: history success binds scanner support files, not only findings."""
        item = next(item for item in self.value["resources"] if item["key"] == "git-history")
        write_clean_receipt(self.state, self.value, item)
        work = AUDIT.resource_work(self.state, item["key"])
        AUDIT.safe_write_bytes(work / "scanner.log", b"modified history log\n")
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_retained_bytes_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_modified_git_metadata_byte_is_rejected(self):
        """Invariant: commit metadata bytes are rehashed before finalization."""
        item = next(item for item in self.value["resources"] if item["key"] == "git-metadata")
        write_clean_receipt(self.state, self.value, item)
        work = AUDIT.resource_work(self.state, item["key"])
        AUDIT.safe_write_bytes(work / "metadata/commit-metadata.bin", b"modified metadata\n")
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_retained_bytes_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_extra_git_metadata_file_is_rejected(self):
        """Invariant: a retained Git manifest binds the complete expected path set."""
        item = next(item for item in self.value["resources"] if item["key"] == "git-metadata")
        write_clean_receipt(self.state, self.value, item)
        work = AUDIT.resource_work(self.state, item["key"])
        AUDIT.safe_write_bytes(work / "metadata/unbound.bin", b"unbound\n")
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_retained_bytes_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_binary_or_nested_count_cannot_be_hidden_by_empty_gap_list(self):
        self.value = manifest({"runs": [], "artifacts": [artifact()]})
        item = next(item for item in self.value["resources"] if item["key"].startswith("artifact-"))
        receipt = write_clean_receipt(self.state, self.value, item)
        receipt["archive"]["nested_archive_members"] = 1
        receipt["coverage_gaps"] = []
        AUDIT.write_receipt(self.state, item["key"], receipt)
        with self.assertRaisesRegex(AUDIT.AuditError, "receipt_coverage_mismatch"):
            AUDIT.validate_receipt(self.value, item, self.state)

    def test_scan_batch_cannot_report_audit_success_even_when_complete(self):
        self.fill()

        class MetadataApi:
            def __init__(self, token, budget):
                pass

            def json(self, suffix):
                return (
                    {"id": AUDIT.REPOSITORY_ID, "full_name": AUDIT.REPOSITORY,
                     "clone_url": "https://github.com/BaryonOS/BaryonOS.git", "private": False},
                    FakeHeaders(),
                )

        with (
            mock.patch.object(AUDIT, "scanner_binding", return_value=self.value["scanner"]),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "GitHubApi", MetadataApi),
        ):
            result = AUDIT.phase_scan_batch(
                self.state,
                ROOT,
                Path("unused"),
                FakeBudget(self.state),
                batch_size=20,
            )
        self.assertTrue(result["finalizable"])
        self.assertFalse(result["audit_complete"])
        self.assertFalse(result["snapshot_audit_passed"])
        self.assertFalse(result["publication_clearance"])

    def test_scan_batch_resumes_at_first_missing_resource(self):
        first = self.value["resources"][0]
        write_clean_receipt(self.state, self.value, first)

        class MetadataApi:
            def __init__(self, token, budget):
                pass

            def json(self, suffix):
                return (
                    {"id": AUDIT.REPOSITORY_ID, "full_name": AUDIT.REPOSITORY,
                     "clone_url": "https://github.com/BaryonOS/BaryonOS.git", "private": False},
                    FakeHeaders(),
                )

        def scan_objects(item, root, work, binary, budget):
            AUDIT.safe_write_json(work / "findings.json", [])
            AUDIT.safe_write_bytes(work / "reachable-objects.batch", b"objects")
            return {
                "content_scanned": True,
                "scanner_exit_code": 0,
                "finding_count": 0,
                "rules": {},
                "coverage_gaps": [],
                "report_relpath": "findings.json",
                "report_sha256": AUDIT.file_hash(work / "findings.json"),
                "payload_relpath": "reachable-objects.batch",
                "payload_sha256": AUDIT.file_hash(work / "reachable-objects.batch"),
                "object_counts": {
                    "objects": 1,
                    "object_bytes": 7,
                    "non_utf8_or_nul_blobs": 0,
                    "non_utf8_or_nul_commits": 0,
                    "non_utf8_or_nul_tags": 0,
                },
            }

        with (
            mock.patch.object(AUDIT, "scanner_binding", return_value=self.value["scanner"]),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "GitHubApi", MetadataApi),
            mock.patch.object(AUDIT, "scan_git_objects", side_effect=scan_objects) as called,
        ):
            result = AUDIT.phase_scan_batch(
                self.state,
                ROOT,
                Path("unused"),
                FakeBudget(self.state),
                batch_size=1,
            )
        self.assertEqual(result["processed_this_batch"], 1)
        self.assertEqual(result["receipts"], 2)
        self.assertEqual(called.call_args.args[0]["key"], "git-objects")

    def test_finalize_success_remains_bounded_and_not_clearance(self):
        self.fill()

        class FinalApi:
            def __init__(self, token, budget):
                pass

            def json(self, suffix):
                return (
                    {"id": AUDIT.REPOSITORY_ID, "full_name": AUDIT.REPOSITORY,
                     "clone_url": "https://github.com/BaryonOS/BaryonOS.git", "private": False},
                    FakeHeaders(),
                )

            def inventory(self, suffix, field):
                return []

        with (
            mock.patch.object(AUDIT, "GitHubApi", FinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
        ):
            result, passed = AUDIT.phase_finalize(self.state, ROOT, FakeBudget(self.state))
        self.assertTrue(passed)
        self.assertTrue(result["snapshot_audit_passed"])
        self.assertTrue(result["audit_complete"])
        self.assertFalse(result["publication_clearance"])
        self.assertIn("resources deleted before the snapshot", result["scope_limitations"])
        self.assertFalse((self.state / "final.json").exists())

    def test_finalize_commits_success_only_after_budget_closes(self):
        """Invariant: the durable success transition follows all budget checks."""
        self.fill()
        events = []
        real_budget = AUDIT.Budget
        real_write_json = AUDIT.safe_write_json

        class TrackingBudget(real_budget):
            def __exit__(self, *unused):
                result = super().__exit__(*unused)
                events.append("budget-closed")
                return result

        def tracking_write(path, value, *, accountant=None):
            if Path(path).name == "final.json":
                events.append("final-written")
            return real_write_json(path, value, accountant=accountant)

        output = io.StringIO()
        with (
            mock.patch.object(AUDIT, "GitHubApi", StableFinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "Budget", TrackingBudget),
            mock.patch.object(AUDIT, "safe_write_json", side_effect=tracking_write),
            mock.patch("sys.stdout", output),
        ):
            exit_code = AUDIT.main(
                ["--state-dir", str(self.state), "--repo", str(ROOT), "finalize"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(events, ["budget-closed", "final-written"])
        console = json.loads(output.getvalue())
        final = AUDIT.load_json(self.state / "final.json")
        self.assertTrue(console["snapshot_audit_passed"])
        self.assertTrue(final["snapshot_audit_passed"])

    def test_finalize_budget_exit_failure_leaves_no_success_record(self):
        """Invariant: a late command cannot persist its computed success candidate."""
        self.fill()
        real_budget = AUDIT.Budget

        class LateBudget(real_budget):
            def __exit__(self, *unused):
                self.deadline = AUDIT.time.monotonic() - 1
                return super().__exit__(*unused)

        output = io.StringIO()
        with (
            mock.patch.object(AUDIT, "GitHubApi", StableFinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "Budget", LateBudget),
            mock.patch("sys.stdout", output),
        ):
            exit_code = AUDIT.main(
                ["--state-dir", str(self.state), "--repo", str(ROOT), "finalize"]
            )

        self.assertEqual(exit_code, 1)
        self.assertEqual(json.loads(output.getvalue())["error"], "command_time_budget_exhausted")
        self.assertFalse((self.state / "final.json").exists())

    def test_finalize_quota_failure_leaves_no_success_record(self):
        """Invariant: final-record capacity is checked before the success commit."""
        self.fill()
        real_usage_ledger = AUDIT.UsageLedger

        class FinalQuotaAccountant(AUDIT.StateAccountant):
            def reserve_file(self, path, maximum):
                if Path(path).name == "final.json":
                    raise AUDIT.AuditError("max_total_state_bytes_exhausted")
                return super().reserve_file(path, maximum)

        def quota_ledger(state, configured_limits):
            accountant = FinalQuotaAccountant(
                state,
                configured_limits["max_total_state_bytes"],
            )
            return real_usage_ledger(state, configured_limits, accountant=accountant)

        output = io.StringIO()
        with (
            mock.patch.object(AUDIT, "GitHubApi", StableFinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "UsageLedger", side_effect=quota_ledger),
            mock.patch("sys.stdout", output),
        ):
            exit_code = AUDIT.main(
                ["--state-dir", str(self.state), "--repo", str(ROOT), "finalize"]
            )

        self.assertEqual(exit_code, 1)
        self.assertEqual(json.loads(output.getvalue())["error"], "max_total_state_bytes_exhausted")
        self.assertFalse((self.state / "final.json").exists())

    def test_finalize_usage_finish_failure_leaves_no_success_record(self):
        """Invariant: a ledger-save failure follows no durable success transition."""
        self.fill()
        real_usage_ledger = AUDIT.UsageLedger

        class FinishQuotaAccountant(AUDIT.StateAccountant):
            def __init__(self, state, maximum):
                super().__init__(state, maximum)
                self.usage_writes = 0

            def reserve_file(self, path, maximum):
                if Path(path).name == "usage.json":
                    self.usage_writes += 1
                    if self.usage_writes == 3:
                        raise AUDIT.AuditError("max_total_state_bytes_exhausted")
                return super().reserve_file(path, maximum)

        def quota_ledger(state, configured_limits):
            accountant = FinishQuotaAccountant(
                state,
                configured_limits["max_total_state_bytes"],
            )
            return real_usage_ledger(state, configured_limits, accountant=accountant)

        output = io.StringIO()
        with (
            mock.patch.object(AUDIT, "GitHubApi", StableFinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
            mock.patch.object(AUDIT, "UsageLedger", side_effect=quota_ledger),
            mock.patch("sys.stdout", output),
        ):
            exit_code = AUDIT.main(
                ["--state-dir", str(self.state), "--repo", str(ROOT), "finalize"]
            )

        self.assertEqual(exit_code, 1)
        self.assertEqual(json.loads(output.getvalue())["error"], "max_total_state_bytes_exhausted")
        self.assertFalse((self.state / "final.json").exists())

    def test_failed_refinalization_invalidates_earlier_success(self):
        """Invariant: a failed retry cannot leave an earlier positive record active."""
        self.fill()
        AUDIT.safe_write_json(
            self.state / "final.json",
            {"audit_complete": True, "snapshot_audit_passed": True},
        )
        changed = dict(self.value["source"])
        changed["refs_sha256"] = "f" * 64
        output = io.StringIO()

        with (
            mock.patch.object(AUDIT, "GitHubApi", StableFinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=changed),
            mock.patch("sys.stdout", output),
        ):
            exit_code = AUDIT.main(
                ["--state-dir", str(self.state), "--repo", str(ROOT), "finalize"]
            )

        self.assertEqual(exit_code, 1)
        self.assertEqual(json.loads(output.getvalue())["error"], "git_snapshot_changed")
        self.assertFalse((self.state / "final.json").exists())

    def test_finalize_rejects_inventory_deletion_or_mutation(self):
        self.value = manifest({"runs": [run()], "artifacts": []})
        AUDIT.safe_write_json(self.state / "manifest.json", self.value)
        self.fill()

        class EmptyApi:
            def __init__(self, token, budget):
                pass

            def json(self, suffix):
                return (
                    {"id": AUDIT.REPOSITORY_ID, "full_name": AUDIT.REPOSITORY,
                     "clone_url": "https://github.com/BaryonOS/BaryonOS.git", "private": False},
                    FakeHeaders(),
                )

            def inventory(self, suffix, field):
                return []

        with (
            mock.patch.object(AUDIT, "GitHubApi", EmptyApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=self.value["source"]),
        ):
            with self.assertRaisesRegex(AUDIT.AuditError, "actions_inventory_changed_since_snapshot"):
                AUDIT.phase_finalize(self.state, ROOT, FakeBudget(self.state))

    def test_finalize_rejects_changed_git_refs(self):
        self.fill()
        changed = dict(self.value["source"])
        changed["refs_sha256"] = "f" * 64

        class FinalApi:
            def __init__(self, token, budget):
                pass

            def json(self, suffix):
                return (
                    {"id": AUDIT.REPOSITORY_ID, "full_name": AUDIT.REPOSITORY,
                     "clone_url": "https://github.com/BaryonOS/BaryonOS.git", "private": False},
                    FakeHeaders(),
                )

            def inventory(self, suffix, field):
                return []

        with (
            mock.patch.object(AUDIT, "GitHubApi", FinalApi),
            mock.patch.object(AUDIT, "git_snapshot", return_value=changed),
        ):
            with self.assertRaisesRegex(AUDIT.AuditError, "git_snapshot_changed"):
                AUDIT.phase_finalize(self.state, ROOT, FakeBudget(self.state))
