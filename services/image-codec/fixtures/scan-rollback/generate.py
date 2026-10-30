#!/usr/bin/env python3
"""Synthetic JPEG test data writer, not a decoder/parser or OS implementation."""
from pathlib import Path
import hashlib
import io
import json
import struct
import warnings
from PIL import Image, features

out = Path(__file__).resolve().parent
image = bytearray()
layout = []

def marker(code, name, body=None):
    start = len(image)
    image.extend(bytes((255, code)))
    if body is not None:
        image.extend(struct.pack('>H', len(body) + 2))
        image.extend(body)
    layout.append(dict(name=name, offset=start, end=len(image), hex=image[start:].hex()))

def entropy(value, name):
    start = len(image)
    image.append(value)
    layout.append(dict(name=name, offset=start, end=len(image), hex=f'{value:02x}'))

def dqt(value):
    return bytes((0,)) + bytes((value,)) * 64

def dc_zero():
    # DC table 0: one length-1 code, symbol category 0 => code 0.
    return bytes((0, 1)) + bytes(15) + bytes((0,))

def dc_changed():
    # DC table 0: category 1 has code 0, category 0 has code 10.
    return bytes((0, 1, 1)) + bytes(14) + bytes((1, 0))

def ac_eob():
    # AC table 0: one length-1 code, EOB symbol 0 => code 0.
    return bytes((16, 1)) + bytes(15) + bytes((0,))

def sos(component):
    return bytes((1, component, 0, 0, 63, 0))

def adobe(transform):
    return b'Adobe' + struct.pack('>HHH', 100, 0, 0) + bytes((transform,))

marker(0xd8, 'SOI')
marker(0xdb, 'DQT q0=1', dqt(1))
marker(0xc0, 'SOF0 8x8 3 components 1x1 sampling',
       bytes((8, 0, 8, 0, 8, 3, 1, 17, 0, 2, 17, 0, 3, 17, 0)))
marker(0xc4, 'DHT DC0=0/AC0=EOB', dc_zero() + ac_eob())
marker(0xda, 'SOS component 1', sos(1))
entropy(0x3f, 'scan1 DC0=0 AC-EOB=0 padding=111111')
marker(0xdb, 'between scans 1-2: DQT q0=2', dqt(2))
marker(0xc4, 'between scans 1-2: DHT changed DC0', dc_changed())
marker(0xdd, 'between scans 1-2: DRI=1', struct.pack('>H', 1))
marker(0xfe, 'between scans 1-2: COM', b'synthetic snapshot rollback')
marker(0xe0, 'between scans 1-2: APP0 AVI1', b'AVI1\0')
marker(0xee, 'between scans 1-2: APP14 Adobe transform=1', adobe(1))
marker(0xda, 'SOS component 2', sos(2))
entropy(0x9f, 'scan2 DC0=10 AC-EOB=0 padding=11111')
marker(0xda, 'SOS component 3', sos(3))
entropy(0x9f, 'scan3 DC0=10 AC-EOB=0 padding=11111')
marker(0xd9, 'EOI')
path = out / 'baseline-three-sos-table-changes.jpg'
path.write_bytes(image)
with warnings.catch_warnings(record=True) as caught:
    warnings.simplefilter('always')
    with Image.open(io.BytesIO(image)) as reference:
        reference.load()
        rgb = reference.convert('RGB')
        pixels = rgb.tobytes()
        result = dict(format=reference.format, source_mode=reference.mode,
                      size=list(reference.size), rgb_extrema=rgb.getextrema(),
                      distinct_rgb_pixels=sorted(set(zip(pixels[0::3], pixels[1::3], pixels[2::3]))),
                      rgb_sha256=hashlib.sha256(pixels).hexdigest(),
                      expected_all_channels_128=(pixels == bytes((128,)) * (8 * 8 * 3)))
        assert result['expected_all_channels_128'], result
oracle = dict(synthetic_test_data=True, jpeg_path=path.name,
              jpeg_bytes=len(image), jpeg_sha256=hashlib.sha256(image).hexdigest(),
              pillow_version=Image.__version__, libjpeg_compat_version=features.version('jpg'),
              libjpeg_turbo=features.check_feature('libjpeg_turbo'),
              libjpeg_turbo_version=features.version_feature('libjpeg_turbo'),
              reference=result, warnings=[str(x.message) for x in caught], markers=layout)
(out / 'oracle.json').write_text(json.dumps(oracle, indent=2) + '\n')
(out / 'fixture.hex').write_text(image.hex() + '\n')
print(json.dumps({k: oracle[k] for k in ('jpeg_path', 'jpeg_bytes', 'jpeg_sha256',
                    'pillow_version', 'libjpeg_compat_version', 'libjpeg_turbo_version',
                    'reference', 'warnings')}, indent=2))
