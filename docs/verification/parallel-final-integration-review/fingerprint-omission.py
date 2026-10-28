#!/usr/bin/env python3
"""G06 narrow negative control; writes only a temporary synthetic workspace."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile

root = Path(sys.argv[1]).resolve()
common_path = root / "tools/common.py"
spec = importlib.util.spec_from_file_location("g06_common", common_path)
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
parent_name = "experiments/quantum/Cargo.toml"
core_name = "experiments/quantum/qsim-core/Cargo.toml"
parent_source = (root / parent_name).read_text()
core_source = (root / core_name).read_text()
if 'edition = "2021"' not in parent_source or "edition.workspace = true" not in core_source:
    raise ValueError("This negative control expects the reviewed workspace inheritance")
with tempfile.TemporaryDirectory(prefix="nieuw-os-g06-") as temporary:
    fixture = Path(temporary)
    (fixture / core_name).parent.mkdir(parents=True)
    (fixture / core_name).write_text(core_source)
    parent = fixture / parent_name
    parent.write_text(parent_source)
    common.ROOT = fixture
    before = common.source_hash()
    before_parent = hashlib.sha256(parent.read_bytes()).hexdigest()
    parent.write_text(parent_source.replace('edition = "2021"', 'edition = "2024"'))
    after = common.source_hash()
    after_parent = hashlib.sha256(parent.read_bytes()).hexdigest()
report = {
    "task": "G06", "root_source_modified": False,
    "common_source_sha256": hashlib.sha256(common_path.read_bytes()).hexdigest(),
    "actual_inherited_input": parent_name, "inheriting_package_manifest": core_name,
    "temporary_before_parent_sha256": before_parent, "temporary_after_parent_sha256": after_parent,
    "temporary_before_fingerprint": before, "temporary_after_fingerprint": after,
    "omission_reproduced": before == after and before_parent != after_parent,
    "current_qsim_gate_compensation": "run_qsim.validate_build compares artifact source hashes with native builder.sources before and after execution, including the parent manifest.",
}
print(json.dumps(report, indent=2) + "\n", end="")
