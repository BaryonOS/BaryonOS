"""HOST_ONLY H03-T3a oracle counterexamples; fixtures are not native evidence."""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_virtio_snd as gate

REPOSITORY = Path(__file__).resolve().parents[2]

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


# Independent literal serial fixtures: the acceptance test does not derive its
# input from `gate.expected_lines`, so a wrong template cannot accept itself.
_PROBE = ["SND:BEGIN", "SND:TSC_PER_MS:1000018"]
_DEVICE = _PROBE + ["SND:DEVICE:1af4:1059:00:08.0", "SND:BAR_BASE:000000c000004000",
                    "SND:BAR_SIZE:0000000000004000", "SND:MMIO_PAGES:3"]
_PARAMS = "SND:PARAMS:BUFFER=7680:PERIOD=1920:FEATURES=0:CHANNELS=2:FORMAT=S16:RATE=48000"
_EPOCH_0 = [FEATURES, "SND:DRIVER_FEATURES:0000000100000000", "SND:CONFIG:JACKS=0:STREAMS=1:CHMAPS=0",
            "SND:RXQ:DISABLED", "SND:DRIVER_OK:EPOCH=0", "SND:PCM_INFO:USED_LEN=4:RECORDS_LEN=36",
            "SND:STREAM:0:OUTPUT", _PARAMS, "SND:PREPARE:OK", "SND:START:OK"]
_EPOCH_1 = [FEATURES, "SND:DRIVER_FEATURES:0000000100000000", "SND:CONFIG:JACKS=0:STREAMS=1:CHMAPS=0",
            "SND:RXQ:DISABLED", "SND:DRIVER_OK:EPOCH=1", "SND:PCM_INFO:USED_LEN=36:RECORDS_LEN=36",
            "SND:STREAM:0:OUTPUT", _PARAMS, "SND:PREPARE:OK", "SND:START:OK"]
_FULL = ["SND:SUBMITTED_FRAMES:48000", "SND:FRAMES:48000", "SND:TX_COMPLETIONS:100"]
_FINISH = ["SND:DRAIN:200MS:RUNNING", "SND:STOP:OK", "SND:IDLE:200MS:STOPPED", "SND:RELEASE:OK",
           "SND:RELEASE_TWICE:REFUSED:NOT_SENT", "SND:DEVICE_STATUS:0x0f", "SND:EVENTS:INVALID=0",
           "SND:QUIESCE:ACK", "SND:USED:STABLE", "SND:PAGES:RETAINED", "SND:PASS"]
INDEPENDENT_SERIAL = {
    "no-device": _PROBE + ["SND:ABSENT"],
    "access-platform": _DEVICE + [FEATURES_AP, "SND:OFFLINE:FEATURES",
                                  "SND:OFFLINE:BUS_MASTER=0:DRIVER_OK=0"],
    "normal": _DEVICE + _EPOCH_0 + _FULL + _FINISH,
    "fence-reinit": _DEVICE + _EPOCH_0
    + ["SND:SUBMITTED_FRAMES:4800", "SND:FRAMES:4800", "SND:TX_COMPLETIONS:10",
       "SND:FENCE:ACK", "SND:USED:STABLE", "SND:EPOCH:1"] + _EPOCH_1 + _FULL + _FINISH,
    "quarantine": _DEVICE + _EPOCH_0
    + ["SND:SUBMITTED_FRAMES:6240", "SND:FRAMES:4800", "SND:TX_COMPLETIONS:10",
       "SND:QUARANTINED", "SND:PUBLISH:REFUSED", "SND:PAGES:RETAINED", "SND:PASS"],
}


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
        self.assertEqual(set(INDEPENDENT_SERIAL), set(gate.FEATURE))
        for case in gate.FEATURE:
            with self.subTest(case):
                literal = transcript(case, INDEPENDENT_SERIAL[case])
                values = gate.evaluate_serial(case, literal, 33, False)
                self.assertEqual(values["tsc_per_ms"], "1000018")
                # The template-filled helper used by the mutation tests matches.
                self.assertEqual(snd_lines(case), INDEPENDENT_SERIAL[case])
        literal = transcript("quarantine", INDEPENDENT_SERIAL["quarantine"])
        values = gate.evaluate_serial("quarantine", literal, 33, False)
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
        # Submitted minus the exact 4800 completed frames is what was still
        # driver-outstanding at the fence: 0 (4800) and 4 periods (6720) fail.
        for submitted in (4320, 4800, 6000, 6720):
            with self.subTest(submitted), self.assertRaises(ValueError):
                gate.evaluate_serial("quarantine", transcript("quarantine", snd_lines("quarantine", submitted)),
                                     33, False)
        for submitted in (5280, 5760, 6240):
            with self.subTest(submitted):
                values = gate.evaluate_serial("quarantine", transcript("quarantine", snd_lines("quarantine", submitted)),
                                              33, False)
                self.assertEqual(gate.submitted_a("quarantine", values), submitted)

    def test_output_beyond_the_bound_fails_the_run(self):
        bound = gate.MAX_OUTPUT
        gate.check_output_bound(b"x" * bound, b"y" * bound)
        for stdout, stderr in ((b"x" * (bound + 1), b""), (b"", b"y" * (bound + 1))):
            with self.assertRaises(ValueError):
                gate.check_output_bound(stdout, stderr)
        good = transcript("normal").encode()
        for tail in (b"PANIC\nBOOT:EBS_OK\nM1:FAIL:X\n", b"SND:PASS\n"):
            with self.subTest(tail):
                # Harmless pre-handoff output sized so the bound ends right after M1:PASS.
                padded = b"#" * (bound - len(good) - 1) + b"\n" + good
                self.assertEqual(len(padded), bound)
                out = padded + tail
                # The bounded prefix alone hides the tail: why the bound check exists.
                gate.evaluate_serial("normal", gate.bounded(out), 33, False)
                with self.assertRaises(ValueError):
                    gate.check_output_bound(out, b"")
                    gate.evaluate_serial("normal", gate.bounded(out), 33, False)

    def test_running_process_is_killed_at_the_output_bound(self):
        """A flooding guest is stopped while it runs; no more than the bound plus one byte is retained."""
        bound = gate.MAX_OUTPUT
        flood = "import os\nwhile True:\n    os.write({fd}, b'x' * 65536)\n"
        for fd in (1, 2):
            with self.subTest(stream=fd):
                start = time.monotonic()
                rc, out, err, timed_out, stats = gate.run_bounded([sys.executable, "-c", flood.format(fd=fd)],
                                                                  dict(os.environ), timeout=60)
                self.assertLess(time.monotonic() - start, 30)
                self.assertFalse(timed_out)
                self.assertNotEqual(rc, 0)
                self.assertEqual(len(out if fd == 1 else err), bound + 1)
                # Every byte read is counted, including those discarded past the bound.
                self.assertGreaterEqual(stats["stdout_drained_bytes" if fd == 1 else "stderr_drained_bytes"],
                                        bound + 1)
                self.assertTrue(stats["output_overflow_killed"])
                with self.assertRaises(ValueError):
                    gate.check_output_bound(out, err)
        rc, out, err, timed_out, stats = gate.run_bounded(
            [sys.executable, "-c", "import sys; sys.stdout.write('A\\n'); sys.stderr.write('B'); sys.exit(33)"],
            dict(os.environ), timeout=60)
        self.assertEqual((rc, out, err, timed_out), (33, b"A\n", b"B", False))
        self.assertEqual(stats, dict(stdout_drained_bytes=2, stderr_drained_bytes=1, output_overflow_killed=False))
        rc, out, err, timed_out, stats = gate.run_bounded(
            [sys.executable, "-c", "import sys, time; print('early', flush=True); time.sleep(60)"],
            dict(os.environ), timeout=5)
        self.assertEqual((rc, out, timed_out, stats["output_overflow_killed"]), (None, b"early\n", True, False))

    def test_deadline_and_overflow_together_report_both(self):
        """The overflow is seen only once the deadline has passed: timed_out stays true beside the kill."""
        fake = FakeProcess(stdout=[b"x" * (gate.MAX_OUTPUT + 10)])
        with mock.patch.object(gate.subprocess, "Popen", return_value=fake):
            rc, out, err, timed_out, stats = gate.run_bounded(["qemu"], {}, timeout=0)
        self.assertEqual((rc, len(out), err, timed_out, fake.killed), (None, gate.MAX_OUTPUT + 1, b"", True, True))
        self.assertEqual(stats, dict(stdout_drained_bytes=gate.MAX_OUTPUT + 10, stderr_drained_bytes=0,
                                     output_overflow_killed=True))

    def test_a_failed_drain_raises_instead_of_returning_partial_output(self):
        fake = FakeProcess(stdout=[b"partial", OSError("pipe read failed")])
        start = time.monotonic()
        with mock.patch.object(gate.subprocess, "Popen", return_value=fake):
            with self.assertRaisesRegex(OSError, "pipe read failed"):
                gate.run_bounded(["qemu"], {}, timeout=30)
        # The drain error itself stops the wait; the deadline must not be what ends it.
        self.assertLess(time.monotonic() - start, 2)
        self.assertTrue(fake.killed)

    def test_overflow_of_a_process_that_exits_by_itself_is_counted_but_not_a_kill(self):
        """Several chunks cross the bound, then QEMU exits before the harness can kill it."""
        fake = SelfExitingProcess(stdout=[b"x" * gate.MAX_OUTPUT, b"y" * 10, b"z" * 10, b"w" * 10])
        with mock.patch.object(gate.subprocess, "Popen", return_value=fake):
            rc, out, err, timed_out, stats = gate.run_bounded(["qemu"], {}, timeout=30)
        self.assertEqual((rc, len(out), err, timed_out, fake.killed), (0, gate.MAX_OUTPUT + 1, b"", False, False))
        self.assertEqual(stats, dict(stdout_drained_bytes=gate.MAX_OUTPUT + 30, stderr_drained_bytes=0,
                                     output_overflow_killed=False))

    def test_qemu_is_killed_when_a_reader_thread_cannot_start(self):
        started, popen = [], subprocess.Popen

        def spawn(*args, **kwargs):
            started.append(popen(*args, **kwargs))
            return started[-1]
        with mock.patch.object(gate.subprocess, "Popen", side_effect=spawn), \
                mock.patch.object(gate.threading.Thread, "start", side_effect=RuntimeError("can't start new thread")):
            with self.assertRaisesRegex(RuntimeError, "can't start new thread"):
                gate.run_bounded([sys.executable, "-c", "import time; time.sleep(60)"], dict(os.environ), timeout=30)
        process = started[0]
        self.addCleanup(lambda: (process.kill(), process.wait()))
        self.assertIsNotNone(process.poll(), "QEMU stand-in left running after Thread.start failed")
        self.assertTrue(process.stdout.closed and process.stderr.closed, "pipes of unstarted readers left open")


class FakePipe:
    """Yields its scripted chunks (raising exceptions), then blocks until the process is killed, then EOF."""

    def __init__(self, chunks, process, done):
        self.chunks, self.process, self.done = list(chunks), process, done

    def read1(self, _size):
        if self.chunks:
            chunk = self.chunks.pop(0)
            if isinstance(chunk, BaseException):
                self.done.set()
                raise chunk
            return chunk
        self.done.set()
        self.process.dead.wait(10)
        return b""

    def close(self):
        pass


class FakeProcess:
    """A QEMU stand-in that runs until killed; poll() first waits until stdout delivered its chunks."""

    def __init__(self, stdout):
        self.dead, self.drained, self.killed, self.returncode = threading.Event(), threading.Event(), False, None
        self.stdout, self.stderr = FakePipe(stdout, self, self.drained), FakePipe([], self, threading.Event())

    def poll(self):
        if not self.killed:
            self.drained.wait(10)
        return self.returncode

    def wait(self, timeout=None):
        if not self.dead.wait(timeout):
            raise subprocess.TimeoutExpired("qemu", timeout)
        return self.returncode

    def kill(self):
        self.killed, self.returncode = True, -9
        self.dead.set()


class SelfExitingProcess(FakeProcess):
    """Running when first polled (after stdout delivered its chunks); exits with 0 by itself at the next poll."""

    def __init__(self, stdout):
        super().__init__(stdout)
        self.polls = 0

    def poll(self):
        self.polls += 1
        if self.polls == 1:
            return super().poll()
        if self.returncode is None:
            self.returncode = 0
            self.dead.set()
        return self.returncode


class Command(unittest.TestCase):
    def test_qemu_command_per_case(self):
        rel = "build/run/x"
        base = gate.qemu_arguments("no-device", "qemu", rel, gate.MACHINE)
        self.assertEqual(base[base.index("-machine") + 1], "pc-q35-8.2")
        self.assertIn("-icount", base)
        self.assertEqual(base[base.index("-icount") + 1], "shift=0,sleep=off")
        self.assertIn(f"wav,id=snd0,path={rel}/out.wav,out.frequency=48000,out.channels=2,out.format=s16", base)
        self.assertFalse(any("virtio-sound-pci" in item for item in base))
        normal = gate.qemu_arguments("normal", "qemu", rel, gate.MACHINE)
        self.assertEqual(normal[-2:], ["-device", "virtio-sound-pci,addr=08.0,audiodev=snd0,streams=1"])
        self.assertTrue(gate.qemu_arguments("access-platform", "qemu", rel, gate.MACHINE)[-1]
                        .endswith(",iommu_platform=on"))
        for case in ("fence-reinit", "quarantine"):
            self.assertEqual(gate.qemu_arguments(case, "qemu", rel, gate.MACHINE)[-2:], normal[-2:])
        with self.assertRaises(ValueError):
            gate.qemu_arguments("unknown", "qemu", rel, gate.MACHINE)

    def test_machine_is_pinned_to_the_fixture_profile(self):
        with mock.patch.dict(os.environ):
            os.environ.pop("NOS_QEMU_MACHINE", None)
            self.assertEqual(gate.fixture_machine(), "pc-q35-8.2")
            args = gate.qemu_arguments("normal", "qemu", "build/run/x", gate.fixture_machine())
            self.assertEqual(args[args.index("-machine") + 1], "pc-q35-8.2")
            os.environ["NOS_QEMU_MACHINE"] = "pc-q35-8.2"
            self.assertEqual(gate.fixture_machine(), "pc-q35-8.2")
            for value in ("pc-q35-10.0", "pc-q35-7.2", "", "pc-q35-9.0", "q35"):
                os.environ["NOS_QEMU_MACHINE"] = value
                with self.subTest(value), self.assertRaises(ValueError):
                    gate.fixture_machine()
        for machine in ("pc-q35-10.0", "pc-q35-7.2", "", "q35"):
            with self.subTest(machine), self.assertRaises(ValueError):
                gate.qemu_arguments("normal", "qemu", "build/run/x", machine)

    def test_qemu_option_paths_are_refused_unless_literal(self):
        self.assertEqual(gate.qemu_path("build/xhci-native/snd/run/snd-normal-0123456789"),
                         "build/xhci-native/snd/run/snd-normal-0123456789")
        for rel in ("build,x/run/x", "build,,x/run/x", "c:d/run/x", "build x/run/x", "/abs/run/x",
                    "build=x/run/x", "../build/run/x", "build/\nrun", ""):
            for case in gate.FEATURE:
                with self.subTest(rel=rel, case=case), self.assertRaises(ValueError):
                    gate.qemu_arguments(case, "qemu", rel, gate.MACHINE)

    def test_refused_inputs_create_nothing_and_launch_nothing(self):
        refuse = mock.Mock(side_effect=AssertionError("must not be reached"))
        with tempfile.TemporaryDirectory() as directory, mock.patch.dict(os.environ), \
                mock.patch.object(gate, "source_hash", refuse), \
                mock.patch.object(gate.subprocess, "run", refuse), mock.patch.object(gate.subprocess, "Popen", refuse), \
                mock.patch.object(gate.shutil, "which", refuse):
            root = Path(directory).resolve()
            os.environ["NOS_QEMU_MACHINE"] = "pc-q35-10.0"
            with mock.patch.object(gate, "ROOT", root), mock.patch.object(gate, "BUILD", root / "build"):
                with self.assertRaises(ValueError):
                    gate.run_case("normal")
            os.environ.pop("NOS_QEMU_MACHINE")
            with mock.patch.object(gate, "ROOT", root), mock.patch.object(gate, "BUILD", root / "build,x"):
                with self.assertRaises(ValueError):
                    gate.run_case("normal")
            self.assertEqual(list(root.iterdir()), [])
        refuse.assert_not_called()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class FakeRun:
    """A checkout with a matching build and a scripted QEMU; no guest runs."""

    def __init__(self, root: Path, *, tamper=None, stdout: bytes | None = None):
        self.root, self.build, self.tamper = root, root / "build", tamper
        self.stdout = transcript("normal").encode() if stdout is None else stdout
        (self.build / "esp/EFI/BOOT").mkdir(parents=True)
        (self.build / "esp/kernel.elf").write_bytes(b"kernel")
        (self.build / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"loader")
        manifest = {"kernel_feature": "native-virtio-snd-probe", "loader_feature": None,
                    "source_fingerprint_version": 3, "source_sha256": "f" * 64,
                    "kernel": {"sha256": digest(b"kernel")}, "loader": {"sha256": digest(b"loader")}}
        (self.build / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        self.code, self.variables = root / "fw/CODE.fd", root / "fw/VARS.fd"
        self.code.parent.mkdir()
        self.code.write_bytes(b"code" * 64)
        self.variables.write_bytes(b"vars" * 64)

    def qemu(self, args, **kwargs):
        if args[1:] == ["--version"]:
            return subprocess.CompletedProcess(args, 0, stdout="QEMU emulator version 8.2.2 (fake)\n")
        run = self.root / next(item for item in args if item.startswith("if=pflash,unit=1")) \
            .split("file=")[1].rsplit("/", 1)[0]
        (run / "OVMF_VARS.fd").write_bytes(b"nvram written by the firmware")
        (run / "out.wav").write_bytes(wav(gate.PATTERN_A))
        if self.tamper:
            (run / self.tamper).write_bytes(b"changed")
        return subprocess.CompletedProcess(args, 33, stdout=self.stdout, stderr=b"")

    def bounded(self, args, env, *, timeout):
        """The scripted QEMU behind run_bounded: it keeps at most MAX_OUTPUT + 1 bytes per stream."""
        result = self.qemu(args)
        stats = dict(stdout_drained_bytes=len(result.stdout), stderr_drained_bytes=len(result.stderr),
                     output_overflow_killed=len(result.stdout) > gate.MAX_OUTPUT)
        return (result.returncode, result.stdout[:gate.MAX_OUTPUT + 1], result.stderr[:gate.MAX_OUTPUT + 1], False,
                stats)

    def run(self) -> dict:
        with mock.patch.dict(os.environ), mock.patch.object(gate, "ROOT", self.root), \
                mock.patch.object(gate, "BUILD", self.build), \
                mock.patch.object(gate, "source_hash", return_value="f" * 64), \
                mock.patch.object(gate, "environment", return_value={"PATH": "/fake"}), \
                mock.patch.object(gate, "firmware", return_value=(self.code, self.variables)), \
                mock.patch.object(gate.shutil, "which", return_value="/fake/qemu-system-x86_64"), \
                mock.patch.object(gate.subprocess, "run", side_effect=self.qemu), \
                mock.patch.object(gate, "run_bounded", side_effect=self.bounded), \
                mock.patch.object(gate.subprocess, "Popen", side_effect=AssertionError("unscripted QEMU launch")), \
                mock.patch("builtins.print"):
            os.environ.pop("NOS_QEMU_MACHINE", None)
            return gate.run_case("normal")


class Report(unittest.TestCase):
    def test_firmware_inputs_are_hashed_before_launch_and_final_nvram_separately(self):
        with tempfile.TemporaryDirectory() as directory:
            fake = FakeRun(Path(directory).resolve())
            report = fake.run()
            self.assertTrue(report["passed"], report.get("error"))
            self.assertEqual(report["schema_version"], 2)
            self.assertEqual(report["qemu_machine"], "pc-q35-8.2")
            self.assertEqual(report["command"][report["command"].index("-machine") + 1], "pc-q35-8.2")
            self.assertEqual(report["ovmf_code_sha256"], digest(b"code" * 64))
            self.assertEqual(report["ovmf_vars_template_sha256"], digest(b"vars" * 64))
            self.assertEqual(report["ovmf_vars_final_sha256"], digest(b"nvram written by the firmware"))
            self.assertNotIn("ovmf_vars_sha256", report)
            self.assertEqual((report["stdout_bytes"], report["stdout_drained_bytes"], report["output_truncated"],
                              report["output_overflow_killed"]), (len(fake.stdout), len(fake.stdout), False, False))

    def test_changed_firmware_inputs_or_oversized_output_fail_the_run(self):
        for tamper in ("OVMF_CODE.fd", "OVMF_VARS.template.fd"):
            with self.subTest(tamper), tempfile.TemporaryDirectory() as directory:
                report = FakeRun(Path(directory).resolve(), tamper=tamper).run()
                self.assertFalse(report["passed"])
                self.assertIn("OVMF", report["error"])
        with tempfile.TemporaryDirectory() as directory:
            good = transcript("normal").encode()
            report = FakeRun(Path(directory).resolve(),
                             stdout=b"#" * (gate.MAX_OUTPUT - len(good)) + good + b"SND:PASS\n").run()
            self.assertFalse(report["passed"])
            self.assertTrue(report["output_truncated"])
            self.assertEqual((report["stdout_bytes"], report["stdout_drained_bytes"], report["output_overflow_killed"]),
                             (gate.MAX_OUTPUT + 1, gate.MAX_OUTPUT + len(b"SND:PASS\n"), True))
            self.assertIn("bound", report["error"])

    def test_a_staged_copy_that_differs_from_its_source_is_refused_before_launch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            code, variables = root / "CODE.fd", root / "VARS.fd"
            code.write_bytes(b"code")
            variables.write_bytes(b"vars")
            original = gate.shutil.copy2

            def corrupting(source, target):
                original(source, target)
                if Path(target).name == "OVMF_VARS.fd":
                    Path(target).write_bytes(b"corrupt")

            run = root / "run"
            run.mkdir()
            with mock.patch.object(gate.shutil, "copy2", side_effect=corrupting):
                with self.assertRaises(ValueError):
                    gate.stage_firmware(run, code, variables)
            hashes = gate.stage_firmware(run, code, variables)
            self.assertEqual(hashes, {"ovmf_code_sha256": digest(b"code"),
                                      "ovmf_vars_template_sha256": digest(b"vars")})


class HistoricalChecksums(unittest.TestCase):
    """drivers/virtio-snd/verification/SHA256SUMS is historical evidence of an
    earlier basis; its documented drift must match the tree exactly."""

    RLIB = "target/x86_64-unknown-none/release/libnieuw_os_virtio_snd.rlib"
    CHANGED = {"README.md", "TRANSPORT.md", "src/lib.rs", "src/pcm.rs", "src/pcm_tests.rs",
               "verification/README.md"}

    def test_documented_drift_is_exactly_the_observed_drift(self):
        crate = REPOSITORY / "drivers/virtio-snd"
        entries = [line.split(maxsplit=1) for line in
                   (crate / "verification/SHA256SUMS").read_text(encoding="utf-8").splitlines()]
        self.assertEqual(len(entries), 31)
        missing, changed = set(), set()
        for expected, name in entries:
            path = crate / name
            if not path.is_file():
                missing.add(name)
            elif hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                changed.add(name)
        # The ignored release rlib is the only entry that may be absent or differ.
        self.assertLessEqual(missing, {self.RLIB})
        self.assertEqual(changed - {self.RLIB}, self.CHANGED)
        readme = (crate / "verification/README.md").read_text(encoding="utf-8")
        self.assertIn("sha256sum -c verification/SHA256SUMS", readme)
        for name in sorted(self.CHANGED) + [self.RLIB]:
            self.assertIn(f"`{name}`", readme)


if __name__ == "__main__":
    unittest.main()
