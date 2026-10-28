#!/usr/bin/env python3
"""Execute real G0 helper tests and QMP boundary checks on the development host.

No VM is started and no hardware operation is mocked as a successful mapping.
Each invocation owns a new output directory, exact source hashes and raw logs.
The Rust wrapper includes kernel/src/graphics.rs unchanged; m2 and serial stubs
panic if entered. This is host evidence only, never guest/MMIO/PTE evidence.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import unittest
from unittest.mock import Mock, patch
import uuid

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_HELPERS = {
    'graphics::tests::span_checks_last_pixel_stride_and_exact_extent',
    'graphics::tests::span_rejects_oversize_empty_high_bits_and_arithmetic_edges',
    'graphics::tests::span_revalidates_metadata_before_any_hardware_access',
    'graphics::tests::canonical_colors_encode_both_supported_device_formats',
    'graphics::tests::input_errors_reset_the_stream_and_aux_is_discarded',
    'graphics::tests::helpers_reject_use_before_mapping_or_input_setup',
}


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load_qmp(path):
    # Loading the selected real runner supplies its QMP implementation, not a
    # rewritten copy. Only transport endpoints/replies in the tests are doubles.
    sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location('graphics_qmp_under_test', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def qmp_suite(path):
    module = load_qmp(path)

    def stream(*replies):
        control = module.Qmp.__new__(module.Qmp)
        control.socket = Mock()
        control.reader = io.BytesIO(b''.join(json.dumps(r).encode() + b'\n' for r in replies))
        control.pending = bytearray()
        control.sequence, control.trace = 0, []
        control.deadline = time.monotonic() + 5
        return control

    class QmpBoundaries(unittest.TestCase):
        def test_exact_reply_after_event_is_accepted(self):
            control = stream({'event': 'STOP', 'timestamp': {'seconds': 0, 'microseconds': 0}},
                             {'id': 1, 'return': {'running': False}})
            self.assertEqual(control.command('query-status'), {'running': False})
            self.assertEqual(control.trace[-1]['request']['id'], 1)
            self.assertEqual(control.trace[-1]['response']['id'], 1)

        def test_success_requires_return_and_events_are_not_responses(self):
            for reply in ({'id': 1}, {'id': 1, 'event': 'STOP'},
                          {'id': 1, 'return': {}, 'event': 'STOP'}):
                with self.subTest(reply=reply), self.assertRaises((RuntimeError, ValueError)):
                    stream(reply).command('screendump')

        def test_response_id_preserves_the_integer_request_type(self):
            for value in (True, 1.0, '1', None, 2):
                with self.subTest(value=value), self.assertRaises((RuntimeError, ValueError)):
                    stream({'id': value, 'return': {}}).command('query-status')

        def test_error_and_conflicting_success_fields_are_rejected(self):
            for reply in ({'id': 1, 'error': {'class': 'GenericError', 'desc': 'test'}},
                          {'id': 1, 'return': {}, 'error': {'class': 'GenericError', 'desc': 'test'}}):
                with self.subTest(reply=reply), self.assertRaises((RuntimeError, ValueError)):
                    stream(reply).command('screendump')

        def test_expired_command_is_never_sent(self):
            control = stream({'id': 1, 'return': {}})
            control.deadline = time.monotonic() - 1
            with self.assertRaises(TimeoutError):
                control.command('send-key')
            control.socket.sendall.assert_not_called()

        def test_socket_send_timeout_is_bounded_by_remaining_run_time(self):
            control = stream()
            control.deadline = time.monotonic() + 0.1
            control.send(b'{}\n')
            control.socket.settimeout.assert_called()
            timeout = control.socket.settimeout.call_args[0][0]
            self.assertGreater(timeout, 0)
            self.assertLessEqual(timeout, 0.1)

        def test_slow_socket_read_is_not_accepted_after_shared_deadline(self):
            control = stream()
            clock = Mock(return_value=10.0)
            control.deadline = 11.0

            def late_line(_limit):
                clock.return_value = 12.0
                return b'{"id":1,"return":{}}\n'

            control.reader = Mock()
            control.reader.readline.side_effect = late_line
            control.reader.read1.side_effect = late_line
            with patch.object(module.time, 'monotonic', clock), self.assertRaises(TimeoutError):
                control.read()

        def test_expired_pipe_send_does_not_touch_the_writer(self):
            control = module.QmpPipe.__new__(module.QmpPipe)
            read_fd, write_fd = os.pipe()
            control.writer = os.fdopen(write_fd, 'wb', buffering=0)
            try:
                control.deadline = time.monotonic() - 1
                with self.assertRaises(TimeoutError):
                    control.send(b'{}\n')
                control.writer.close()
                self.assertEqual(os.read(read_fd, 1), b'')
            finally:
                control.writer.close()
                os.close(read_fd)

        @unittest.skipIf(os.name == 'nt', 'G0 pipe runner currently has a Unix-only contract')
        def test_full_pipe_send_has_a_deadline_instead_of_blocking(self):
            # Isolate the potentially blocking old implementation in a child.
            # A full, unread pipe must cause a deadline error within 50 ms.
            code = '''import importlib.util, os, sys, time
from pathlib import Path
path = Path(sys.argv[1])
sys.path.insert(0, str(path.parent))
spec = importlib.util.spec_from_file_location("g0_pipe_probe", path)
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
r, w = os.pipe()
try:
    os.set_blocking(w, False)
    try:
        while True: os.write(w, b"x" * 4096)
    except BlockingIOError: pass
    os.set_blocking(w, True)
    control = module.QmpPipe.__new__(module.QmpPipe)
    control.writer = os.fdopen(w, "wb", buffering=0)
    control.deadline = time.monotonic() + 0.05
    try: control.send(b"{}\\n")
    except TimeoutError: pass
    else: raise AssertionError("full pipe write did not fail by deadline")
finally:
    os.close(r)
'''
            try:
                result = subprocess.run([sys.executable, '-B', '-c', code, str(path)],
                                        capture_output=True, timeout=2, check=False)
            except subprocess.TimeoutExpired:
                self.fail('full QMP pipe blocked beyond the command deadline')
            self.assertEqual(result.returncode, 0, result.stderr.decode('utf-8', 'replace'))

    return unittest.defaultTestLoader.loadTestsFromTestCase(QmpBoundaries)


def run_helpers(output_root, qmp_source, rustc):
    directory = output_root.resolve() / ('graphics-helpers-' + uuid.uuid4().hex)
    directory.mkdir(parents=True, exist_ok=False)
    binary = directory / ('graphics_helpers.exe' if os.name == 'nt' else 'graphics_helpers')
    files = {ROOT / 'kernel/src/graphics.rs', ROOT / 'tests/graphics_helpers.rs',
             Path(__file__).resolve(), qmp_source, qmp_source.parent / 'common.py',
             ROOT / 'rust-toolchain.toml'}
    for crate in ('boot-protocol', 'ui-core', 'kernel-core'):
        files.update((ROOT / 'crates' / crate / 'src').rglob('*.rs'))
    before = {str(path): sha256(path) for path in sorted(files)}
    report = dict(scope='host helper logic and QMP protocol tests; no native mapping, port I/O or VM proof',
                  status='running', passed=False, started_utc=datetime.now(timezone.utc).isoformat(),
                  base_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  qmp_source=str(qmp_source), source_sha256=before, commands=[], checks={})

    def save():
        (directory / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')

    def command(name, argv, timeout=30):
        started = datetime.now(timezone.utc).isoformat()
        try:
            result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=timeout, check=False)
            stdout, stderr, code = result.stdout, result.stderr, result.returncode
            timed_out = False
        except subprocess.TimeoutExpired as error:
            stdout, stderr, code = error.stdout or b'', error.stderr or b'', None
            timed_out = True
        paths = {}
        for stream, raw in (('stdout', stdout), ('stderr', stderr)):
            path = directory / (name + '.' + stream + '.txt')
            path.write_bytes(raw)
            paths[stream] = path.name
            paths[stream + '_sha256'] = sha256(path)
        record = dict(name=name, command=argv, cwd=str(ROOT), started_utc=started,
                      completed_utc=datetime.now(timezone.utc).isoformat(),
                      exit_code=code, timed_out=timed_out, timeout_seconds=timeout, **paths)
        report['commands'].append(record)
        report['checks'][name] = code == 0 and not timed_out
        save()
        print(f'{name}: exit={code}; timeout={timed_out}')
        return report['checks'][name], stdout

    save()
    try:
        ok, version = command('rustc-version', [rustc, '-vV'])
        report['checks']['rust-1.90.0'] = ok and version.startswith(b'rustc 1.90.0 ')
        if not report['checks']['rust-1.90.0']:
            raise RuntimeError('G0 helper gate requires the pinned Rust 1.90.0 compiler')
        libraries = []
        for name in ('boot-protocol', 'ui-core', 'kernel-core'):
            crate = name.replace('-', '_')
            artifact = directory / ('lib' + crate + '.rlib')
            ok, _ = command('build-' + name, [rustc, '--edition=2021', '--crate-name', crate,
                              '--crate-type=rlib', '-D', 'warnings',
                              str(ROOT / 'crates' / name / 'src/lib.rs'), '-o', str(artifact)])
            if not ok:
                raise RuntimeError('Real project library failed to compile: ' + name)
            libraries.extend(['--extern', crate + '=' + str(artifact)])
        ok, _ = command('build-graphics-helpers', [rustc, '--edition=2021', '--test',
                          '-D', 'warnings', str(ROOT / 'tests/graphics_helpers.rs'),
                          *libraries, '-o', str(binary)])
        if not ok:
            raise RuntimeError('Unmodified kernel helper test module failed to compile')
        ok, listing = command('list-graphics-helpers', [str(binary), '--list'])
        names = {line[:-6] for line in listing.decode().splitlines() if line.endswith(': test')}
        report['checks']['six-real-kernel-tests-present'] = ok and EXPECTED_HELPERS <= names
        report['kernel_helper_tests'] = sorted(EXPECTED_HELPERS & names)
        report['listed_tests'] = sorted(names)
        ok, executed = command('run-graphics-helpers', [str(binary), '--test-threads=1'])
        executed_lines = set(executed.decode().splitlines())
        report['checks']['six-real-kernel-tests-executed'] = ok and all(
            'test ' + name + ' ... ok' in executed_lines for name in EXPECTED_HELPERS)
        report['checks']['decoder-reset-test-executed'] = (
            'test decoder_reset_clears_both_modifier_sides_caps_and_partial_prefixes ... ok'
            in executed_lines)
        command('qmp-boundaries', [sys.executable, '-B', str(Path(__file__).resolve()),
                                  '--qmp-tests', '--qmp-source', str(qmp_source)], timeout=20)
    except Exception as error:
        report['error'] = f'{type(error).__name__}: {error}'
    finally:
        after = {str(path): sha256(path) for path in sorted(files)}
        report['checks']['source-unchanged'] = before == after
        report['artifact_sha256'] = {path.name: sha256(path) for path in directory.iterdir()
                                     if path.is_file() and path.name != 'result.json'}
        report['status'] = 'completed'
        report['passed'] = 'error' not in report and all(report['checks'].values())
        report['finished_utc'] = datetime.now(timezone.utc).isoformat()
        save()
    print(f'Host graphics helpers passed={report["passed"]}; {directory}')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-root', type=Path, default=ROOT / 'build/graphics-helpers')
    parser.add_argument('--qmp-source', type=Path, default=ROOT / 'tools/run_graphics.py',
                        help='Exact real runner to inspect; its path and bytes are bound in the report')
    parser.add_argument('--rustc', default=shutil.which('rustc') or 'rustc')
    parser.add_argument('--qmp-tests', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    source = args.qmp_source.resolve(strict=True)
    if args.qmp_tests:
        result = unittest.TextTestRunner(verbosity=2).run(qmp_suite(source))
        return 0 if result.wasSuccessful() else 1
    return 0 if run_helpers(args.output_root, source, args.rustc)['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
