"""Targeted G12 negative controls on copied real G11 compiler records.

No implementation source, original artifact, VM or guest log is modified.
Usage: python3 -B compiler_binding_probe.py IMMUTABLE_GATE_SNAPSHOT G11_ROOT
"""
import json
from pathlib import Path
import shutil
import sys
import tempfile

sys.path.insert(0, str(Path(sys.argv[1]) / "tools"))
import test_document_session as gate

root = Path(sys.argv[2]).resolve(strict=True)
build = root / "build/parallel-g11-document"
results = {}
with tempfile.TemporaryDirectory(prefix="nieuw-os-g12-binding-") as temporary:
    snapshot = Path(temporary)
    (snapshot / "logs").mkdir()
    names = [row[1] for row in gate.LIBRARIES.values()] + ["build-users-m6c_probe"]
    for name in names:
        for suffix in (".json", ".stdout.txt", ".stderr.txt"):
            shutil.copyfile(build / "logs" / (name + suffix), snapshot / "logs" / (name + suffix))
    gate.compiler_binding(snapshot, build, root)
    results["original_actual_compiler_records_accepted"] = True
    path = snapshot / "logs/build-document-library-nieuw_os_catalog_client.json"
    original = path.read_bytes()
    for mutation in ("foreign_rlib_dependency", "later_conflicting_abi_flag"):
        metadata = json.loads(original)
        if mutation == "foreign_rlib_dependency":
            command = metadata["command"]
            expected = "catalog_core=" + str(build / "users/libcatalog_core.rlib")
            command[command.index(expected)] = "catalog_core=/foreign/unbound-catalog-core.rlib"
        else:
            metadata["command"] += ["-C", "no-redzone=no"]
        path.write_text(json.dumps(metadata) + "\n")
        try:
            gate.compiler_binding(snapshot, build, root)
        except ValueError as error:
            results[mutation] = {"rejected": True, "reason": str(error)}
        else:
            results[mutation] = {"rejected": False}
        path.write_bytes(original)
print(json.dumps(results, indent=2))
