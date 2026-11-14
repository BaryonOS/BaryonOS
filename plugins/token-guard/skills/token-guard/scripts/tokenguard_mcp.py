# SPDX-License-Identifier: MPL-2.0
"""Bounded stdio MCP adapter; command execution is authorized by the MCP host."""
from __future__ import annotations

import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time


VERSION = "1.2.1"
MODERN = "2026-07-28"
LEGACY = ("2025-11-25", "2025-06-18")
SUPPORTED = (MODERN, *LEGACY)
META_VERSION = "io.modelcontextprotocol/protocolVersion"
META_CAPABILITIES = "io.modelcontextprotocol/clientCapabilities"
MAX_FRAME = 256 * 1024
MAX_TIMEOUT = 3600


def tools_list() -> list[dict]:
    readonly = {"readOnlyHint": True, "destructiveHint": False,
                "idempotentHint": True, "openWorldHint": False}
    return [
        {"name": "inspect_file", "description": "Bounded source outline, line ranges and SHA-256 in the configured project root. Python AST; other languages heuristic. Use hosted Graphify for dependencies.",
         "inputSchema": {"type": "object", "properties": {"filepath": {"type": "string"}},
                         "required": ["filepath"], "additionalProperties": False},
         "annotations": readonly},
        {"name": "read_slice", "description": "Exact source range, at most 200 lines / 12000 bytes. Optional expected_sha rejects changed source.",
         "inputSchema": {"type": "object", "properties": {"filepath": {"type": "string"},
                         "start": {"type": "integer", "minimum": 1}, "end": {"type": "integer", "minimum": 1},
                         "expected_sha": {"type": "string"}}, "required": ["filepath", "start", "end"],
                         "additionalProperties": False}, "annotations": readonly},
        {"name": "run_compact", "description": "Execute authorized foreground argv in the configured project. No implicit shell; stdin closed; default 300-second deadline unless configured. Preserves exit code and full local log. Can change files or access network; host MCP approval applies, shell permission rules do not transfer.",
         "inputSchema": {"type": "object", "properties": {
                         "argv": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 256},
                         "timeout_seconds": {"type": "number", "exclusiveMinimum": 0, "maximum": MAX_TIMEOUT},
                         "output": {"type": "string", "enum": ["summary", "bounded"]}},
                         "required": ["argv"], "additionalProperties": False},
         "annotations": {"readOnlyHint": False, "destructiveHint": True,
                         "idempotentHint": False, "openWorldHint": True}},
    ]


def valid_id(value) -> bool:
    return type(value) in (str, int)


def deadline(value) -> float:
    if type(value) not in (int, float) or not math.isfinite(value) or not 0 < value <= MAX_TIMEOUT:
        raise ValueError(f"timeout_seconds must be finite, positive and at most {MAX_TIMEOUT}")
    return float(value)


def stop_worker(process: subprocess.Popen) -> None:
    """Ask the helper to clean up its command before escalating termination."""
    if process.poll() is not None:
        return
    if os.name == "nt":
        taskkill = Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32" / "taskkill.exe"
        try:
            subprocess.run([str(taskkill), "/PID", str(process.pid), "/T", "/F"],
                           stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL, timeout=5, check=False)
        except (OSError, subprocess.TimeoutExpired):
            # Native Windows descendant cleanup cannot be guaranteed without taskkill.
            process.terminate()
    else:
        process.terminate()
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=2)


class Server:
    def __init__(self, root: Path, helper: Path, inspect_file, read_slice):
        self.root = root.resolve(strict=True)
        self.helper = helper.resolve(strict=True)
        self.inspect_file, self.read_slice = inspect_file, read_slice
        self.default_timeout = deadline(float(os.environ.get("TOKENGUARD_TIMEOUT", "300")))
        self.version = None
        self.initialized = False
        self.lock = threading.RLock()
        self.closed = False
        self.active: dict = {}

    def emit(self, message: dict) -> None:
        with self.lock:
            if not self.closed:
                try:
                    sys.stdout.write(json.dumps(message, ensure_ascii=True, allow_nan=False) + "\n")
                    sys.stdout.flush()
                except (BrokenPipeError, OSError):
                    self.closed = True
                    for entry in self.active.values():
                        entry["cancel"].set()

    def error(self, mid, code: int, message: str, data=None) -> None:
        response = {"jsonrpc": "2.0", "error": {"code": code, "message": message}}
        if valid_id(mid):
            response["id"] = mid
        if data is not None:
            response["error"]["data"] = data
        self.emit(response)

    def result(self, mid, body: dict, modern: bool = False, cacheable: bool = False) -> None:
        if modern:
            body = {**body, "resultType": "complete"}
            if cacheable:
                body.update(ttlMs=0, cacheScope="private")
        self.emit({"jsonrpc": "2.0", "id": mid, "result": body})

    def tool_result(self, mid, text: str, modern: bool, failed: bool = False) -> None:
        self.result(mid, {"content": [{"type": "text", "text": text}], "isError": failed}, modern)

    def worker_result(self, mid, entry: dict, text: str, modern: bool, failed: bool) -> None:
        with self.lock:
            if not self.closed and not entry["cancel"].is_set():
                self.tool_result(mid, text, modern, failed)

    def validate_arguments(self, name: str, args: dict) -> None:
        definitions = {t["name"]: t["inputSchema"] for t in tools_list()}
        if name not in definitions:
            raise ValueError("unknown tool")
        schema = definitions[name]
        if not isinstance(args, dict) or set(args) - set(schema["properties"]) or set(schema["required"]) - set(args):
            raise ValueError("missing or unsupported tool arguments")
        if name in ("inspect_file", "read_slice"):
            if not isinstance(args["filepath"], str) or not args["filepath"] or len(args["filepath"]) > 4096:
                raise ValueError("filepath must be a nonempty string of at most 4096 characters")
        if name == "read_slice":
            if type(args["start"]) is not int or type(args["end"]) is not int:
                raise ValueError("start and end must be integers")
            if "expected_sha" in args and not isinstance(args["expected_sha"], str):
                raise ValueError("expected_sha must be a string")
        if name == "run_compact":
            argv = args["argv"]
            if (not isinstance(argv, list) or not 1 <= len(argv) <= 256
                    or any(not isinstance(x, str) or "\x00" in x for x in argv)
                    or not argv[0] or sum(len(x.encode("utf-8")) for x in argv) > 64_000):
                raise ValueError("argv must be 1..256 literal strings without NUL, at most 64000 UTF-8 bytes")
            deadline(args.get("timeout_seconds", self.default_timeout))
            if args.get("output", "summary") not in ("summary", "bounded"):
                raise ValueError("output must be summary or bounded")

    def run(self, mid, args: dict, modern: bool, entry: dict) -> None:
        process = None
        try:
            timeout = deadline(args.get("timeout_seconds", self.default_timeout))
            argv = [sys.executable, "-B", str(self.helper), "--root", str(self.root),
                    "run", "--stdin-null", "--timeout", str(timeout),
                    "--output", args.get("output", "summary"), "--", *args["argv"]]
            if entry["cancel"].is_set():
                return
            process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, start_new_session=os.name == "posix")
            expires = time.monotonic() + timeout + 5
            while True:
                if entry["cancel"].is_set() or self.closed:
                    stop_worker(process)
                    return
                try:
                    stdout, stderr = process.communicate(timeout=0.1)
                    break
                except subprocess.TimeoutExpired:
                    if time.monotonic() > expires:
                        stop_worker(process)
                        self.worker_result(mid, entry, "ERROR: helper deadline exceeded; inspect the local run log", modern, True)
                        return
            text = stdout.decode("utf-8", errors="replace")
            if stderr:
                text += "\n" + stderr[:4000].decode("utf-8", errors="replace")
            self.worker_result(mid, entry, text, modern, process.returncode != 0)
        except (OSError, ValueError) as exc:
            self.worker_result(mid, entry, f"ERROR: {type(exc).__name__}: {str(exc)[:1000]}", modern, True)
        finally:
            if process is not None and process.poll() is None:
                stop_worker(process)
            with self.lock:
                self.active.pop(mid, None)

    def receive(self, request) -> None:
        if not isinstance(request, dict):
            self.error(None, -32600, "Invalid Request: expected one JSON object")
            return
        mid, method = request.get("id"), request.get("method")
        if request.get("jsonrpc") != "2.0" or not isinstance(method, str):
            self.error(mid, -32600, "Invalid Request")
            return
        if "id" not in request:
            # Notifications never execute tools and never produce a response.
            if method == "notifications/initialized" and self.version in LEGACY:
                self.initialized = True
            elif method == "notifications/cancelled":
                params = request.get("params", {})
                if isinstance(params, dict) and valid_id(params.get("requestId")):
                    with self.lock:
                        entry = self.active.get(params["requestId"])
                        if entry:
                            entry["cancel"].set()
            return
        if not valid_id(mid):
            self.error(None, -32600, "Invalid Request: id must be a string or integer, never null")
            return
        params = request.get("params", {})
        if not isinstance(params, dict):
            self.error(mid, -32602, "params must be an object")
            return
        with self.lock:
            if mid in self.active:
                self.error(mid, -32600, "Request id is already active")
                return
        meta = params.get("_meta", {})
        modern = method == "server/discover" or (isinstance(meta, dict) and META_VERSION in meta)
        if modern:
            if not isinstance(meta, dict) or not isinstance(meta.get(META_VERSION), str) or not isinstance(meta.get(META_CAPABILITIES), dict):
                self.error(mid, -32602, "Modern requests require protocolVersion and clientCapabilities in params._meta")
                return
            if meta[META_VERSION] != MODERN:
                self.error(mid, -32022, "Unsupported protocol version", {"supported": list(SUPPORTED), "requested": meta[META_VERSION]})
                return
        if method == "initialize" and not modern:
            if self.version is not None:
                self.error(mid, -32600, "Already initialized")
                return
            requested = params.get("protocolVersion")
            if not isinstance(requested, str) or not isinstance(params.get("capabilities"), dict) or not isinstance(params.get("clientInfo"), dict):
                self.error(mid, -32602, "initialize requires protocolVersion, capabilities and clientInfo")
                return
            self.version = requested if requested in LEGACY else LEGACY[0]
            self.result(mid, {"protocolVersion": self.version, "capabilities": {"tools": {}},
                             "serverInfo": {"name": "token-guard", "version": VERSION},
                             "instructions": f"Project root: {self.root}. Read tools are bounded. run_compact is not a sandbox; apply the host's MCP permissions."})
            return
        if method == "server/discover":
            self.result(mid, {"supportedVersions": list(SUPPORTED), "capabilities": {"tools": {}},
                             "_meta": {"io.modelcontextprotocol/serverInfo": {"name": "token-guard", "version": VERSION}},
                             "instructions": f"Project root: {self.root}. Command execution uses host MCP approval, not shell permission rules."}, True, True)
            return
        if method == "ping" and not modern:
            self.result(mid, {})
            return
        if not modern and not self.initialized:
            self.error(mid, -32002, "Send initialize and notifications/initialized first")
            return
        if method == "tools/list":
            definitions = tools_list()
            self.result(mid, {"tools": definitions}, modern, True)
        elif method == "tools/call":
            name, args = params.get("name"), params.get("arguments", {})
            if name not in ("inspect_file", "read_slice", "run_compact") or not isinstance(args, dict):
                self.error(mid, -32602, "Unknown tool or malformed tools/call arguments")
                return
            try:
                self.validate_arguments(name, args)
                if name == "inspect_file":
                    self.tool_result(mid, self.inspect_file(self.root, args["filepath"]), modern)
                elif name == "read_slice":
                    self.tool_result(mid, self.read_slice(self.root, args["filepath"], args["start"], args["end"], args.get("expected_sha")), modern)
                else:
                    with self.lock:
                        if self.active:
                            self.tool_result(mid, "ERROR: one command is already running; wait or cancel it", modern, True)
                            return
                        entry = {"cancel": threading.Event()}
                        thread = threading.Thread(target=self.run, args=(mid, args, modern, entry), daemon=True)
                        entry["thread"] = thread
                        self.active[mid] = entry
                        thread.start()
            except Exception as exc:
                # Tool failures are protocol results, not crashes or successful checks.
                self.tool_result(mid, f"ERROR: {type(exc).__name__}: {str(exc)[:2000]}", modern, True)
        else:
            self.error(mid, -32601, "Method not found")

    def close(self) -> None:
        with self.lock:
            self.closed = True
            entries = list(self.active.values())
            for entry in entries:
                entry["cancel"].set()
        for entry in entries:
            entry["thread"].join(timeout=9)


def serve(root: Path, helper: Path, inspect_file, read_slice) -> int:
    server = Server(root, helper, inspect_file, read_slice)

    def shutdown(_signal, _frame):
        raise KeyboardInterrupt

    previous = {s: signal.signal(s, shutdown) for s in (signal.SIGINT, signal.SIGTERM)}
    try:
        # Read bytes so a local cp1252 default cannot alter JSON-RPC framing.
        stream = sys.stdin.buffer
        while not server.closed:
            line = stream.readline(MAX_FRAME + 1)
            if not line:
                break
            if len(line) > MAX_FRAME:
                server.error(None, -32600, "JSON-RPC frame exceeds 256 KiB; connection closed")
                break
            if not line.strip():
                continue
            try:
                request = json.loads(line.decode("utf-8"), parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))
            except (UnicodeError, ValueError, RecursionError):
                server.error(None, -32700, "Parse error")
                continue
            server.receive(request)
    except KeyboardInterrupt:
        return 130
    finally:
        server.close()
        for signum, handler in previous.items():
            signal.signal(signum, handler)
    return 0
