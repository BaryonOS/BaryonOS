"""Host-only bridge: wrap a Rust SimDisk result in the existing synthetic image."""
from pathlib import Path
import json
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import disk_oracle

raw_path = Path(sys.argv[1])
raw = raw_path.read_bytes()
disk = disk_oracle.volume_disk
if len(raw) != disk.VOLUME_SECTORS * disk.SECTOR:
    raise ValueError("wrong host SimDisk extent")
path = raw_path.with_name("synthetic.img")
disk.create_image(path)
image = bytearray(path.read_bytes())
at = disk.FIRST * disk.SECTOR
image[at:at + len(raw)] = raw
path.write_bytes(image)
result = disk_oracle.check_image(bytes(image), 11, 12)
print(json.dumps(result))
