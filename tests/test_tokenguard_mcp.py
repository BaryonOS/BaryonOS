# SPDX-License-Identifier: MPL-2.0
"""Exercise the actual stdio transport, not a mocked method dispatcher."""
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time
import unittest


CLI = Path(__file__).resolve().parents[1] / "tools" / "tokenguard.py"
MODERN_META = {"io.modelcontextprotocol/protocolVersion": "2026-07-28",
               "io.modelcontextprotocol/clientCapabilities": {}}


class MCPTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
        self.env.pop("TOKENGUARD_TIMEOUT", None)
        self.processes = []
        self.addCleanup(self.stop_all)

    def start(self, *, modern=False, env=None):
        proc = subprocess.Popen([sys.executable, "-B", str(CLI), "--root", str(self.root), "mcp"],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                env=self.env if env is None else env)
        messages = queue.Queue()

        def reader():
            for line in proc.stdout:
                try:
                    messages.put(json.loads(line))
                except ValueError:
                    messages.put({"invalid_stdout": line.decode("utf-8", errors="replace")})

        thread = threading.Thread(target=reader, daemon=True)
        thread.start()
        self.processes.append((proc, messages, thread))
        self.proc, self.messages = proc, messages
        if not modern:
            self.send({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "tokenguard-tests", "version": "1"}}})
            self.assertEqual(self.receive(0)["result"]["protocolVersion"], "2025-11-25")
            self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        return proc

    def stop_all(self):
        for proc, _, thread in self.processes:
            if proc.stdin and not proc.stdin.closed:
                proc.stdin.close()
            try:
                proc.wait(timeout=12)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=3)
            thread.join(timeout=2)
            proc.stdout.close()
            proc.stderr.close()

    def send(self, message):
        self.proc.stdin.write((json.dumps(message, ensure_ascii=False) + "\n").encode("utf-8"))
        self.proc.stdin.flush()

    def request(self, mid, method, params=None, *, modern=False):
        params = {} if params is None else params
        if modern:
            params = {**params, "_meta": MODERN_META}
        self.send({"jsonrpc": "2.0", "id": mid, "method": method, "params": params})

    def receive(self, mid, timeout=10):
        result = self.messages.get(timeout=timeout)
        self.assertEqual(result.get("id"), mid, result)
        self.assertEqual(result.get("jsonrpc"), "2.0", result)
        self.assertNotIn("invalid_stdout", result)
        return result

    def call(self, mid, name, args, *, modern=False):
        self.request(mid, "tools/call", {"name": name, "arguments": args}, modern=modern)

    def text(self, message):
        return message["result"]["content"][0]["text"]

    def wait_file(self, path):
        end = time.monotonic() + 5
        while time.monotonic() < end and not path.exists():
            time.sleep(0.01)
        self.assertTrue(path.exists(), f"worker did not create {path}")

    def test_lifecycle_notifications_ping_and_three_deterministic_tools(self):
        self.start()
        self.send({"jsonrpc": "2.0", "method": "notifications/cancelled", "params": {"requestId": 999}})
        self.request(1, "ping")
        self.assertEqual(self.receive(1)["result"], {})
        self.request(2, "tools/list")
        listed = self.receive(2)["result"]["tools"]
        self.assertEqual([x["name"] for x in listed], ["inspect_file", "read_slice", "run_compact"])
        self.assertFalse(listed[-1]["annotations"]["readOnlyHint"])
        self.assertTrue(listed[-1]["annotations"]["destructiveHint"])
        self.assertTrue(self.messages.empty())

    def test_modern_discovery_list_and_read_without_handshake(self):
        self.start(modern=True)
        self.request(1, "server/discover", modern=True)
        discovered = self.receive(1)["result"]
        self.assertIn("2026-07-28", discovered["supportedVersions"])
        self.assertNotIn("2025-03-26", discovered["supportedVersions"])
        self.assertEqual((discovered["resultType"], discovered["ttlMs"], discovered["cacheScope"]), ("complete", 0, "private"))
        self.request(2, "tools/list", modern=True)
        self.assertEqual(self.receive(2)["result"]["ttlMs"], 0)
        (self.root / "m.py").write_text("def f(x: int) -> str:\n    return str(x)\n", encoding="utf-8")
        self.call(3, "inspect_file", {"filepath": "m.py"}, modern=True)
        result = self.receive(3)
        self.assertEqual(result["result"]["resultType"], "complete")
        self.assertIn("def f(x: int) -> str", self.text(result))

    def test_modern_metadata_errors_and_legacy_version_negotiation(self):
        self.start(modern=True)
        self.request(1, "server/discover")
        self.assertEqual(self.receive(1)["error"]["code"], -32602)
        self.request(2, "tools/list", {"_meta": {**MODERN_META, "io.modelcontextprotocol/protocolVersion": "2999-01-01"}})
        self.assertEqual(self.receive(2)["error"]["code"], -32022)
        self.request(3, "initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "old", "version": "1"}})
        self.assertEqual(self.receive(3)["result"]["protocolVersion"], "2025-11-25")

    def test_child_cannot_steal_protocol_stdin_and_unicode_survives_cp1252(self):
        self.start(env={**self.env, "PYTHONIOENCODING": "cp1252"})
        self.call(1, "run_compact", {"argv": [sys.executable, "-c", "import sys; sys.stdout.buffer.write((repr(sys.stdin.read()) + '\\n──> §9 … π\\n').encode('utf-8'))"]})
        result = self.receive(1)
        self.assertFalse(result["result"]["isError"], result)
        self.assertIn("''", self.text(result))
        self.assertIn("──> §9 … π", self.text(result))
        self.request(2, "ping")
        self.assertEqual(self.receive(2)["result"], {})
        receipt = json.loads(next((self.root / "build/tokenguard").glob("*.json")).read_text())
        self.assertEqual(receipt["stdin"], "DEVNULL")
        self.assertEqual(receipt["timeout_seconds"], 300)

    def test_literal_argv_failure_exit_and_source_context(self):
        self.start()
        (self.root / "x.rs").write_text("\n".join(f"row_{n}" for n in range(1, 12)), encoding="utf-8")
        self.call(1, "run_compact", {"argv": [sys.executable, "-c", "import sys; print(sys.argv[1]); print('error: mismatch\\n --> x.rs:6:1'); sys.exit(7)", "$(touch unwanted)"]})
        result = self.receive(1)
        self.assertTrue(result["result"]["isError"])
        self.assertEqual(json.loads(self.text(result).splitlines()[0])["exit_code"], 7)
        self.assertIn("row_6", self.text(result))
        self.assertIn("$(touch unwanted)", self.text(result))
        self.assertFalse((self.root / "unwanted").exists())

    def test_timeout_is_failure_and_ping_during_command_remains_responsive(self):
        self.start()
        self.call(1, "run_compact", {"argv": [sys.executable, "-c", "import time; time.sleep(10)"], "timeout_seconds": 0.3})
        self.request(2, "ping")
        self.assertEqual(self.receive(2)["result"], {})
        result = self.receive(1)
        self.assertTrue(result["result"]["isError"])
        self.assertEqual(json.loads(self.text(result).splitlines()[0])["exit_code"], 124)

    def test_tool_errors_and_path_escape_do_not_crash_transport(self):
        self.start()
        cases = [("read_slice", {"filepath": "missing.py", "start": True, "end": 2}),
                 ("inspect_file", {"filepath": str(CLI)}),
                 ("run_compact", {"argv": "echo wrong"}),
                 ("run_compact", {"argv": [sys.executable], "timeout_seconds": 0})]
        for mid, (name, args) in enumerate(cases, 1):
            self.call(mid, name, args)
            self.assertTrue(self.receive(mid)["result"]["isError"])
        self.call(8, "not_a_tool", {})
        self.assertEqual(self.receive(8)["error"]["code"], -32602)
        self.request(9, "ping")
        self.assertEqual(self.receive(9)["result"], {})

    def test_malformed_frames_null_ids_and_notifications_never_execute(self):
        self.start()
        self.proc.stdin.write(b"{bad json}\n")
        self.proc.stdin.flush()
        self.assertEqual(self.receive(None)["error"]["code"], -32700)
        message = {"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "run_compact", "arguments": {"argv": [sys.executable, "-c", "open('unexpected','w').close()"]}}}
        self.send(message)
        self.send({**message, "id": None})
        self.assertEqual(self.receive(None)["error"]["code"], -32600)
        self.request(9, "ping")
        self.assertEqual(self.receive(9)["result"], {})
        self.assertFalse((self.root / "unexpected").exists())

    @unittest.skipUnless(os.name == "posix", "POSIX descendant cleanup evidence")
    def test_cancel_and_disconnect_stop_commands_and_suppress_cancelled_response(self):
        self.start()
        code = "import pathlib,time; pathlib.Path('started').write_text('yes'); time.sleep(1); pathlib.Path('escaped').write_text('bad')"
        self.call(1, "run_compact", {"argv": [sys.executable, "-c", code]})
        self.wait_file(self.root / "started")
        self.send({"jsonrpc": "2.0", "method": "notifications/cancelled", "params": {"requestId": 1}})
        self.request(2, "ping")
        self.assertEqual(self.receive(2)["result"], {})
        time.sleep(1.2)
        self.assertFalse((self.root / "escaped").exists())
        self.assertTrue(self.messages.empty())
        (self.root / "started").unlink()
        self.call(3, "run_compact", {"argv": [sys.executable, "-c", code]})
        self.wait_file(self.root / "started")
        self.proc.stdin.close()
        self.proc.wait(timeout=10)
        time.sleep(1.1)
        self.assertFalse((self.root / "escaped").exists())


if __name__ == "__main__":
    unittest.main()
