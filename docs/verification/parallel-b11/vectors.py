"""Independent synthetic fixed-vector derivation; no sockets or native run."""
import json
from pathlib import Path
import struct


def checksum(data):
    padded = data + (b"\0" if len(data) % 2 else b"")
    # Host reference: sum all big-endian words first, then fold the carries.
    value = sum(int.from_bytes(padded[i:i+2], "big") for i in range(0, len(padded), 2))
    while value >> 16:
        value = (value & 0xffff) + (value >> 16)
    return (~value) & 0xffff


assert checksum(bytes.fromhex("0001f203f4f5f6f7")) == 0x220d  # RFC 1071 §3
addresses = bytes([192, 0, 2, 1, 198, 51, 100, 2])
ethernet = bytes.fromhex("0200000000020200000000010800")


def packet(payload, absent=False):
    udp_length = 8 + len(payload)
    udp = struct.pack("!HHHH", 12345, 54321, udp_length, 0) + payload
    pseudo = addresses + bytes([0, 17]) + struct.pack("!H", udp_length)
    computed = checksum(pseudo + udp)
    transmitted = 0 if absent else computed or 0xffff
    udp = udp[:6] + struct.pack("!H", transmitted) + udp[8:]
    ip = struct.pack("!BBHHHBBH", 0x45, 0, 20 + udp_length, 0x1234, 0x4000, 64, 17, 0) + addresses
    ip_checksum = checksum(ip)
    ip = ip[:10] + struct.pack("!H", ip_checksum) + ip[12:]
    assert checksum(ip) == 0
    assert absent or checksum(pseudo + udp) == 0
    return dict(hex=(ethernet + ip + udp).hex(), payload_hex=payload.hex(),
                ipv4_checksum=ip_checksum, udp_checksum=transmitted, computed_udp_checksum=computed)


zero_payload = packet(b"\0\0")["computed_udp_checksum"].to_bytes(2, "big")
vectors = {
    "odd": packet(b"nieuwOS"),
    "even": packet(b"01234567"),
    "empty": packet(b""),
    "absent": packet(b"nieuwOS", absent=True),
    "zero_as_ffff": packet(zero_payload),
}
assert vectors["zero_as_ffff"]["computed_udp_checksum"] == 0
out = Path(__file__).with_name("vectors.json")
out.write_text(json.dumps(dict(scope=__doc__, vectors=vectors), indent=2) + "\n")
for name, value in vectors.items():
    print(name, value["hex"], "IP", hex(value["ipv4_checksum"]), "UDP", hex(value["udp_checksum"]))
