#!/usr/bin/env python3
"""Independent Python wave writer/reader around actual PCM clipping processes."""
from io import BytesIO
from pathlib import Path
import struct
import subprocess
import sys
import wave


def main():
    binary = Path(sys.argv[1]).resolve()
    cases = 0
    for channels in [1, 2]:
        for rate in [8000, 44100, 48000, 192000]:
            pcm = b"".join(struct.pack("<h", (index * 7919) % 65536 - 32768)
                           for index in range(32 * channels))
            source = BytesIO()
            with wave.open(source, "wb") as writer:
                writer.setparams((channels, 2, rate, 0, "NONE", "not compressed"))
                writer.writeframes(pcm)
            for start, count in [(0, 1), (0, 32), (1, 3), (15, 17), (31, 1)]:
                result = subprocess.run(
                    [str(binary), "--start-frame", str(start), "--frames", str(count)],
                    input=source.getvalue(), capture_output=True, check=True,
                )
                assert not result.stderr
                with wave.open(BytesIO(result.stdout), "rb") as reader:
                    assert (reader.getnchannels(), reader.getsampwidth(), reader.getframerate(),
                            reader.getnframes(), reader.getcomptype()) == (channels, 2, rate, count, "NONE")
                    assert reader.readframes(count + 1) == pcm[start * channels * 2:(start + count) * channels * 2]
                cases += 1
            metadata = b"JUNK\x01\x00\x00\x00X\0"
            decorated = source.getvalue()[:12] + metadata + source.getvalue()[12:]
            decorated = decorated[:4] + struct.pack("<I", len(decorated) - 8) + decorated[8:]
            args = [str(binary), "--start-frame", "1", "--frames", "2"]
            denied = subprocess.run(args, input=decorated, capture_output=True)
            assert denied.returncode != 0 and not denied.stdout and denied.stderr
            allowed = subprocess.run(args + ["--drop-metadata"], input=decorated, capture_output=True, check=True)
            with wave.open(BytesIO(allowed.stdout), "rb") as reader:
                assert reader.readframes(2) == pcm[channels * 2:channels * 6]
            cases += 1
            if len(sys.argv) > 2:
                Path(sys.argv[2]).write_bytes(allowed.stdout)
    for args, content in [([], b""), (["--start-frame", "-1", "--frames", "2"], b""),
                          (["--start-frame", "0", "--frames", "0"], b""),
                          (["--start-frame", "18446744073709551615", "--frames", "2"], b""),
                          (["--start-frame", "0", "--frames", "2"], b"invalid")]:
        result = subprocess.run([str(binary), *args], input=content, capture_output=True)
        assert result.returncode != 0 and not result.stdout and result.stderr
    print(f"{cases} actual clip outputs independently checked with Python wave; metadata choice and invalid CLI cases checked")


if __name__ == "__main__":
    main()
