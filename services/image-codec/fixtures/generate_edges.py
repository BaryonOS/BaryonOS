#!/usr/bin/env python3
"""Host-only small/odd JPEG geometry fixtures; no production decode path."""
import hashlib
import json
from PIL import Image, __version__
from generate import HERE, pattern

rows = []
for w, h in [(1, 1), (1, 17), (17, 1), (3, 5), (5, 3), (17, 19)]:
    for progressive in (False, True):
        name = f"edge-{w}x{h}-{'progressive' if progressive else 'baseline'}.jpg"
        path = HERE / name
        pattern(w, h).save(path, quality=95, subsampling=2, progressive=progressive)
        with Image.open(path) as image:
            reference = image.convert("RGBA").tobytes()
        (HERE / (name + ".rgba")).write_bytes(reference)
        rows.append({"name": name, "width": w, "height": h,
                     "input_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                     "reference_sha256": hashlib.sha256(reference).hexdigest(),
                     "reference_tolerance": 3})
(HERE / "edges.json").write_text(json.dumps({"scope": "Host fixtures/oracles only",
    "pillow_version": __version__, "images": rows}, indent=2) + "\n")
