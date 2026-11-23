"""HOST_ONLY H03-T3a oracle counterexamples; fixtures are not native evidence."""
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_virtio_snd as gate

FEATURES = "SND:FEATURES:0000010179000000"
FEATURES_AP = "SND:FEATURES:0000010379000000"


def wav(payload: bytes, *, fmt=(1, 2, 48_000, 192_000, 4, 16), finalised=True, extra=b"") -> bytes:
    body = b"WAVE" + b"fmt " + struct.pack("<IHHIIHH", 16, *fmt) + extra
    body += b"data" + struct.pack("<I", len(payload) if finalised else 0) + payload
    return b"RIFF" + struct.pack("<I", len(body) if finalised else 0) + body


def silence(frames: int) -> bytes:
    return b"\0" * (4 * frames)


def snd_lines(case: str, submitted: int = 6240) -> list:
    fills = {"tsc_per_ms": "SND:TSC_PER_MS:1000018", "bar_base": "SND:BAR_BASE:000000c000004000",
             "bar_size": "SND:BAR_SIZE:0000000000004000", "mmio_pages": "SND:MMIO_PAGES:3",
             "features": FEATURES_AP if case == "access-platform" else FEATURES,
             "features_epoch_1": FEATURES,
             "pcm_info_used_len_0": "SND:PCM_INFO:USED_LEN=4:RECORDS_LEN=36",
             "pcm_info_used_len_1": "SND:PCM_INFO:USED_LEN=36:RECORDS_LEN=36",
             "submitted_frames": f"SND:SUBMITTED_FRAMES:{submitted}"}
    return [line if isinstance(line, str) else fills[line[1]] for line in gate.expected_lines(case)]


def transcript(case: str, lines=None, boots: int = 1) -> str:
    lines = snd_lines(case) if lines is None else lines
    boot = list(gate.BOOT) * boots
    return "\x1b[2JBdsDxe: loading Boot0001\n" + "\n".join(boot + lines + list(gate.END)) + "\n"


class Wav(unittest.TestCase):
    def test_exact_pattern_is_accepted_with_bounded_silence(self):
        self.assertEqual(gate.parse_wav(wav(gate.PATTERN_A)), gate.PATTERN_A)
        result = gate.check_audio("normal", silence(4800) + gate.PATTERN_A + silence(4800))
        self.assertEqual((result["leading_silent_frames"], result["trailing_silent_frames"]), (4800, 4800))
        self.assertEqual(gate.check_audio("normal", gate.PATTERN_A)["pattern_a_frames"], 48_000)

    def test_frozen_pattern_values(self):
        self.assertEqual(gate.frame_a(0), bytes([0, 0, 0, 0x80]))
        self.assertEqual(gate.frame_a(0x1234), bytes([0x34, 0x12, 0x34, 0x92]))
        self.assertEqual(gate.frame_b(0), bytes([0xFF, 0xFF, 0xFF, 0xDF]))
        self.assertNotIn(gate.SILENT, {gate.frame_a(n) for n in range(48_000)})
        self.assertFalse({gate.frame_a(n) for n in range(48_000)} & {gate.frame_b(n) for n in range(48_000)})

    def test_container_mutations_fail_closed(self):
        payload = gate.PATTERN_A
        for name, data in {
            "wrong channels": wav(payload, fmt=(1, 1, 48_000, 96_000, 2, 16)),
            "wrong rate": wav(payload, fmt=(1, 2, 44_100, 176_400, 4, 16)),
            "float format": wav(payload, fmt=(3, 2, 48_000, 192_000, 4, 16)),
            "8 bit": wav(payload, fmt=(1, 2, 48_000, 96_000, 2, 8)),
            "unfinalised header": wav(payload, finalised=False),
            "truncated": wav(payload)[:-2],
            "extra chunk": wav(payload, extra=b"LIST" + struct.pack("<I", 4) + b"INFO"),
            "not riff": b"RIFX" + wav(payload)[4:],
        }.items():
            with self.subTest(name), self.assertRaises(ValueError):
                gate.check_audio("normal", gate.parse_wav(data))

    def test_payload_mutations_are_rejected(self):
        a = gate.PATTERN_A
        period = 480 * 4
        flipped = bytearray(a)
        flipped[12_345] ^= 1
        for name, payload in {
            "gap inside A": a[:24_000 * 4] + silence(1) + a[24_000 * 4:],
            "repeated period": a[:10 * period] + a[9 * period:],
            "missing frame": a[:4 * 100] + a[4 * 101:],
            "missing last period": a[:-period],
            "flipped byte": bytes(flipped),
            "leading silence beyond 4800": silence(4801) + a,
            "trailing silence beyond 4800": a + silence(4801),
            "pattern B instead": gate.PATTERN_B,
            "trailing garbage": a + b"\x01\x00\x00\x00",
            "empty": b"",
        }.items():
            with self.subTest(name), self.assertRaises(ValueError):
                gate.check_audio("normal", payload)

    def test_fence_reinit_requires_a_prefix_then_all_of_b(self):
        a, b = gate.PATTERN_A, gate.PATTERN_B
        ok = gate.check_audio("fence-reinit", a[:4800 * 4] + b, 4800)
        self.assertEqual((ok["pattern_a_frames"], ok["gap_silent_frames"], ok["pattern_b_frames"]), (4800, 0, 48_000))
        self.assertEqual(gate.check_audio("fence-reinit", silence(10) + a[:100] + silence(4800) + b, 4800)
                         ["pattern_a_frames"], 25)
        self.assertEqual(gate.check_audio("fence-reinit", b, 4800)["pattern_a_frames"], 0)
        for name, payload in {
            "A longer than submitted": a[:4801 * 4] + b,
            "A after B": a[:4800 * 4] + b + a[4800 * 4:4801 * 4],
            "B incomplete": a[:4800 * 4] + b[:-4],
            "gap beyond 4800": a[:4800 * 4] + silence(4801) + b,
            "A not a prefix": a[4:4800 * 4] + b,
            "B repeated period": a[:4800 * 4] + b[:1920] + b,
        }.items():
            with self.subTest(name), self.assertRaises(ValueError):
                gate.check_audio("fence-reinit", payload, 4800)

    def test_quarantine_allows_only_an_a_prefix(self):
        a = gate.PATTERN_A
        self.assertEqual(gate.check_audio("quarantine", a[:6240 * 4], 6240)["pattern_a_frames"], 6240)
        self.assertEqual(gate.check_audio("quarantine", b"", 6240)["frames"], 0)
        for payload in (a[:6241 * 4], a[:4800 * 4] + gate.PATTERN_B[:4], a[:4800 * 4] + silence(1) + a[:4]):
            with self.assertRaises(ValueError):
                gate.check_audio("quarantine", payload, 6240)

    def test_offline_cases_tolerate_only_absent_or_empty_wav(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "out.wav"
            self.assertEqual(gate.check_no_audio(path)["wav"], "absent")
            path.write_bytes(wav(b""))
            self.assertEqual(gate.check_no_audio(path)["wav"], "empty")
            path.write_bytes(wav(gate.PATTERN_A[:4]))
            with self.assertRaises(ValueError):
                gate.check_no_audio(path)
            path.write_bytes(wav(b"", finalised=False) + b"\0\0\0\0")
            with self.assertRaises(ValueError):
                gate.check_no_audio(path)
        with self.assertRaises(ValueError):
            gate.check_audio("no-device", b"")


class Serial(unittest.TestCase):
    def test_every_case_accepts_its_exact_sequence(self):
        for case in gate.FEATURE:
            with self.subTest(case):
                values = gate.evaluate_serial(case, transcript(case), 33, False)
                self.assertEqual(values["tsc_per_ms"], "1000018")
        values = gate.evaluate_serial("quarantine", transcript("quarantine"), 33, False)
        self.assertEqual(gate.submitted_a("quarantine", values), 6240)
        self.assertEqual(gate.submitted_a("fence-reinit", {}), 4800)

    def test_exit_boot_and_failure_counterexamples(self):
        good = transcript("normal")
        for name, (stdout, rc, timed_out) in {
            "exit 35": (good, 35, False),
            "timeout": (good, None, True),
            "two boots": (transcript("normal", boots=2), 33, False),
            "fail marker": (good.replace("SND:PASS", "SND:PASS\nM1:FAIL:SND"), 33, False),
            "panic": (good + "PANIC\n", 33, False),
            "snd before boot": ("SND:BEGIN\n" + good, 33, False),
            "missing M2": (good.replace("M2:PASS\n", ""), 33, False),
        }.items():
            with self.subTest(name), self.assertRaises(ValueError):
                gate.evaluate_serial("normal", stdout, rc, timed_out)

    def test_sequence_mutations_are_rejected(self):
        lines = snd_lines("normal")
        for name, mutated in {
            "release twice sent": [line for line in lines if not line.startswith("SND:RELEASE_TWICE")],
            "needs reset status": [line.replace("0x0f", "0x4f") for line in lines],
            "short playback": [line.replace("SND:FRAMES:48000", "SND:FRAMES:47520") for line in lines],
            "reordered": lines[:3] + [lines[4], lines[3]] + lines[5:],
            "duplicated": lines + ["SND:PASS"],
            "access platform offered": [line.replace(FEATURES, FEATURES_AP) for line in lines],
            "unknown used length": [line.replace("USED_LEN=4", "USED_LEN=8") for line in lines],
            "invalid events": [line.replace("INVALID=0", "INVALID=1") for line in lines],
        }.items():
            with self.subTest(name), self.assertRaises(ValueError):
                gate.evaluate_serial("normal", transcript("normal", mutated), 33, False)
        with self.assertRaises(ValueError):
            gate.evaluate_serial("access-platform", transcript("access-platform", snd_lines("normal")), 33, False)
        with self.assertRaises(ValueError):
            gate.evaluate_serial("quarantine", transcript("quarantine", snd_lines("quarantine", 4320)), 33, False)
        with self.assertRaises(ValueError):
            gate.evaluate_serial("quarantine", transcript("quarantine", snd_lines("quarantine", 6000)), 33, False)

    def test_qemu_command_per_case(self):
        rel = "build/run/x"
        base = gate.qemu_arguments("no-device", "qemu", rel)
        self.assertIn("-icount", base)
        self.assertEqual(base[base.index("-icount") + 1], "shift=0,sleep=off")
        self.assertIn(f"wav,id=snd0,path={rel}/out.wav,out.frequency=48000,out.channels=2,out.format=s16", base)
        self.assertFalse(any("virtio-sound-pci" in item for item in base))
        normal = gate.qemu_arguments("normal", "qemu", rel)
        self.assertEqual(normal[-2:], ["-device", "virtio-sound-pci,addr=08.0,audiodev=snd0,streams=1"])
        self.assertTrue(gate.qemu_arguments("access-platform", "qemu", rel)[-1].endswith(",iommu_platform=on"))
        for case in ("fence-reinit", "quarantine"):
            self.assertEqual(gate.qemu_arguments(case, "qemu", rel)[-2:], normal[-2:])
        with self.assertRaises(ValueError):
            gate.qemu_arguments("unknown", "qemu", rel)

    def test_machine_is_pinned_to_the_fixture(self):
        unset = {k: v for k, v in os.environ.items() if k != "NOS_QEMU_MACHINE"}
        with mock.patch.dict(os.environ, unset, clear=True):
            self.assertEqual(gate.machine(), "pc-q35-8.2")
            command = gate.qemu_arguments("normal", "qemu", "build/run/x")
            self.assertEqual(command[command.index("-machine") + 1], "pc-q35-8.2")
        for other in ("pc-q35-10.0", "pc-q35-7.2", ""):
            with mock.patch.dict(os.environ, {"NOS_QEMU_MACHINE": other}):
                with self.assertRaises(ValueError):
                    gate.machine()
                with self.assertRaises(ValueError):
                    gate.qemu_arguments("normal", "qemu", "build/run/x")


if __name__ == "__main__":
    unittest.main()
