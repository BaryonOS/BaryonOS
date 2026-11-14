"""A04 evidence-validator regressions; no native guest or binary execution.

Requirements: exact M1/M6b QEMU profile and executable binding, and a complete
immutable 164-file raw inventory. Archived records are fixtures, not new boots.
"""
import ast
import copy
import importlib.util
import json
from pathlib import Path, PurePosixPath
import tempfile
import unittest
import unittest.mock as mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "docs/verification/m6d-m6b-review/a04_m6b_audit.py"
SPEC = importlib.util.spec_from_file_location("a04_m6b_audit", SCRIPT)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
RUNS = json.loads((ROOT / "docs/verification/m6d-m6b-regression/guest-runs.json").read_text(encoding="utf-8"))
RECORDED_ROOT = PurePosixPath(json.loads(
    (ROOT / "docs/verification/m6d-m6b-regression/m6b-preflight.json").read_text(encoding="utf-8"))['cwd'])
M1 = next(record for record in RUNS if record.get("case", "m1") == "m1")
M6 = next(record for record in RUNS if record.get("case") == "normal")
PROGRAM = M6["command"][0]
QEMU_HASH = M6["qemu_sha256"]


def assert_validation_before_acceptance(tree):
    """Guard the two fail-closed orchestration boundaries, not exact source text.

    Pure validator tests cannot detect a correct helper that main never calls.
    Here an unconditional validation must dominate VM acceptance and construction
    of a successful audit report. This is a bounded source-flow regression check,
    not execution of the historical audit or a general Python proof checker.
    """
    def calls(node, name):
        return (isinstance(node, ast.Expr) and isinstance(node.value, ast.Call)
                and isinstance(node.value.func, ast.Name) and node.value.func.id == name)

    def appends_vm(node):
        return any(isinstance(child, ast.Call) and isinstance(child.func, ast.Attribute)
                   and child.func.attr == "append" and isinstance(child.func.value, ast.Name)
                   and child.func.value.id == "vm" for child in ast.walk(node))

    def creates_success(node):
        return any(isinstance(child, ast.Call) and isinstance(child.func, ast.Name)
                   and child.func.id == "dict"
                   and any(keyword.arg == "status" and isinstance(keyword.value, ast.Constant)
                           and keyword.value.value == "passed_actual_evidence_audit"
                           for keyword in child.keywords)
                   for child in ast.walk(node))

    main = next(node for node in tree.body
                if isinstance(node, ast.FunctionDef) and node.name == "main")
    loops = [node for node in main.body if isinstance(node, ast.For)
             and any(appends_vm(statement) for statement in node.body)]
    if len(loops) != 1:
        raise AssertionError("Expected one VM acceptance loop for the audit contract")
    qemu_checked = False
    for statement in loops[0].body:
        if calls(statement, "validate_qemu"):
            qemu_checked = True
        if appends_vm(statement) and not qemu_checked:
            raise AssertionError("VM accepted before unconditional QEMU validation")
    inventory_checked = False
    found_success = False
    for statement in main.body:
        if calls(statement, "validate_raw_inventory"):
            inventory_checked = True
        if creates_success(statement):
            found_success = True
            if not inventory_checked:
                raise AssertionError("Successful report precedes unconditional raw inventory validation")
    if not found_success:
        raise AssertionError("Expected an explicit successful audit report")


class AuditOrchestrationTests(unittest.TestCase):
    """Mutants prove detection of the skipped-M1 and optional-inventory regressions."""

    def setUp(self):
        self.tree = ast.parse(SCRIPT.read_text(encoding="utf-8"))
        self.main = next(node for node in self.tree.body
                         if isinstance(node, ast.FunctionDef) and node.name == "main")
        self.vm_loop = next(node for node in self.main.body if isinstance(node, ast.For)
                            and isinstance(node.target, ast.Name) and node.target.id == "group")

    def remove_validation(self, statements, name):
        statement = next(node for node in statements if isinstance(node, ast.Expr)
                         and isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name)
                         and node.value.func.id == name)
        statements.remove(statement)
        return statement

    def test_main_requires_both_validators_before_acceptance(self):
        assert_validation_before_acceptance(self.tree)

    def test_missing_qemu_call_is_detected(self):
        self.remove_validation(self.vm_loop.body, "validate_qemu")
        with self.assertRaisesRegex(AssertionError, "VM accepted before"):
            assert_validation_before_acceptance(self.tree)

    def test_qemu_call_after_m1_continue_is_detected(self):
        call = self.remove_validation(self.vm_loop.body, "validate_qemu")
        m1_branch = next(node for node in self.vm_loop.body if isinstance(node, ast.If)
                         and any(isinstance(child, ast.Continue) for child in ast.walk(node))
                         and any(isinstance(child, ast.Constant) and child.value == "m1"
                                 for child in ast.walk(node.test)))
        self.vm_loop.body.insert(self.vm_loop.body.index(m1_branch) + 1, call)
        with self.assertRaisesRegex(AssertionError, "VM accepted before"):
            assert_validation_before_acceptance(self.tree)

    def test_missing_raw_inventory_call_is_detected(self):
        self.remove_validation(self.main.body, "validate_raw_inventory")
        with self.assertRaisesRegex(AssertionError, "Successful report precedes"):
            assert_validation_before_acceptance(self.tree)

    def test_inventory_call_after_success_report_is_detected(self):
        call = self.remove_validation(self.main.body, "validate_raw_inventory")
        success = next(node for node in self.main.body if isinstance(node, ast.Assign)
                       and any(isinstance(target, ast.Name) and target.id == "report"
                               for target in node.targets))
        self.main.body.insert(self.main.body.index(success) + 1, call)
        with self.assertRaisesRegex(AssertionError, "Successful report precedes"):
            assert_validation_before_acceptance(self.tree)


class QemuContractTests(unittest.TestCase):
    def validate(self, record, actual_digest=QEMU_HASH):
        # Supplying the archived digest models the validator's hash input; these
        # host tests do not claim access to the original QEMU executable.
        image = record.get("volume_path")
        AUDIT.validate_qemu(
            record,
            PurePosixPath(record["run_directory"]).relative_to(RECORDED_ROOT).as_posix(),
            PROGRAM,
            QEMU_HASH,
            actual_digest,
            image_relative=None if image is None else PurePosixPath(image).relative_to(RECORDED_ROOT).as_posix(),
            has_share=True,
        )

    def test_archived_m1_profile_matches_shared_qemu_binding(self):
        self.assertNotIn("qemu_sha256", M1)
        self.validate(M1)

    def test_all_archived_m6b_profiles_match(self):
        records = [record for record in RUNS if record.get("case", "m1") != "m1"]
        self.assertEqual(len(records), 13)
        for record in records:
            with self.subTest(case=record["case"], run=record["run_directory"]):
                self.validate(record)

    def test_m1_rejects_changed_vm_profile(self):
        for flag, bad in (("-machine", "q35"), ("-cpu", "host"), ("-smp", "2"),
                          ("-m", "512M"), ("-nic", "user")):
            record = copy.deepcopy(M1)
            record["command"][record["command"].index(flag) + 1] = bad
            with self.subTest(flag=flag), self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
                self.validate(record)

    def test_m1_rejects_substituted_executable_path(self):
        record = copy.deepcopy(M1)
        record["command"][0] += "-other"
        with self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
            self.validate(record)

    def test_m1_rejects_changed_actual_executable_hash(self):
        with self.assertRaisesRegex(AssertionError, "actual QEMU executable hash"):
            self.validate(M1, actual_digest="0" * 64)

    def test_m1_rejects_contradictory_recorded_executable_hash(self):
        record = copy.deepcopy(M1)
        record["qemu_sha256"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "record QEMU executable hash"):
            self.validate(record)

    def test_m6b_rejects_changed_recorded_executable_hash(self):
        record = copy.deepcopy(M6)
        record["qemu_sha256"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "record QEMU executable hash"):
            self.validate(record)

    def test_m1_rejects_extra_disk(self):
        record = copy.deepcopy(M1)
        record["command"] += ["-drive", "file=/dev/sda"]
        with self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
            self.validate(record)


class RawInventoryTests(unittest.TestCase):
    def setUp(self):
        self.manifest_bytes = AUDIT.RAW_INVENTORY_PATH.read_bytes()
        self.expected = AUDIT.decode_raw_inventory(self.manifest_bytes)
        self.vars_path = "build/run/m1-after-m6b-421b383906/OVMF_VARS.fd"

    def test_complete_historical_inventory_matches(self):
        self.assertEqual(len(self.expected), 164)
        self.assertIn(self.vars_path, self.expected)
        AUDIT.validate_raw_inventory(dict(self.expected), self.expected)

    def test_missing_m1_firmware_state_is_rejected(self):
        actual = dict(self.expected)
        del actual[self.vars_path]
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_renamed_file_is_rejected_even_when_count_matches(self):
        actual = dict(self.expected)
        actual[self.vars_path + ".replacement"] = actual.pop(self.vars_path)
        self.assertEqual(len(actual), 164)
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_changed_raw_bytes_are_rejected(self):
        actual = dict(self.expected)
        actual[self.vars_path] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "exact raw-file SHA-256 values"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_extra_raw_file_is_rejected(self):
        actual = dict(self.expected)
        actual["build/run/unexpected.txt"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_modified_expected_manifest_is_rejected(self):
        modified = dict(self.expected)
        del modified[self.vars_path]
        with self.assertRaisesRegex(AssertionError, "pinned historical raw inventory hash"):
            AUDIT.decode_raw_inventory(json.dumps(modified).encode("utf-8"))

    def test_output_cannot_replace_expected_manifest(self):
        prefix = AUDIT.RAW_INVENTORY_PATH.with_name("a04_m6b")
        with self.assertRaisesRegex(AssertionError, "cannot overwrite historical raw inventory"):
            AUDIT.validate_output_prefix(prefix)

    def test_separate_output_prefix_is_allowed(self):
        with tempfile.TemporaryDirectory() as directory:
            AUDIT.validate_output_prefix(Path(directory) / "audit")

    def test_output_hardlink_cannot_replace_expected_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            protected = Path(directory) / "expected.json"
            protected.write_text("fixed fixture", encoding="utf-8")
            prefix = Path(directory) / "audit"
            Path(str(prefix) + "_raw_hashes.json").hardlink_to(protected)
            with mock.patch.object(AUDIT, "RAW_INVENTORY_PATH", protected):
                with self.assertRaisesRegex(AssertionError, "cannot overwrite historical raw inventory"):
                    AUDIT.validate_output_prefix(prefix)


if __name__ == "__main__":
    unittest.main()
