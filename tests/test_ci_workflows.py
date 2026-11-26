"""CI-HOSTED: guard public-runner isolation and mandatory workflow coverage.

These checks inspect the checked-in configuration. The supply-chain fixture
executes one workflow shell fragment against isolated local fakes; it does not
execute GitHub's expression engine, download from the network, or provide
native QEMU acceptance.
"""

import ast
import fnmatch
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shlex
import signal
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unicodedata
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_ACTION = ".github/actions/preserve-evidence/action.yml"
CACHE_ACTION_VERSION = "v6.1.0"
CACHE_ACTION_SHA = "55cc8345863c7cc4c66a329aec7e433d2d1c52a9"
ACTIONLINT_VERSION = "1.7.12"
ACTIONLINT_SHA256 = "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8"
ACTIONLINT_TAG_COMMIT = "914e7df21a07ef503a81201c76d2b11c789d3fca"
CARGO_LLVM_COV_VERSION = "0.9.1"
CARGO_LLVM_COV_SHA256 = "b3f68e625481fed9b16444174f3fa5ebcdbde4a1878803a35eabe2dcefcdc41a"
CARGO_LLVM_COV_TAG_COMMIT = "0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7"
HOST_PYTHON_STEP = "Python syntax and DocumentSession host tests"
# #494: the step's serial commands before it ran lanes, byte-identical; lanes may only regroup them.
HOST_PYTHON_COMMANDS = (
    "mkdir -p build/coverage-raw/python",
    "python3 -m compileall -q tools tests userspace/tests experiments/quantum/native "
    "experiments/quantum/native/job .agents/skills/os-dev",
    "python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'",
    "coverage run --parallel-mode -m unittest discover -s tests -p 'test_ci_workflows.py'",
    "python3 -B -m unittest discover -s tests -p 'test_security_alerts_summary.py'",
    "python3 -B -m unittest discover -s tests -p 'test_tokenguard*.py'",
    "python3 -B tools/package_token_guard.py",
    "coverage run --parallel-mode -m unittest discover -s tests -p 'test_document_session_*.py'",
    "coverage run --parallel-mode -m unittest discover -s tests -p 'test_volume_*.py'",
    "coverage run --parallel-mode -m unittest discover -s tests -p 'test_system_*.py'",
    "coverage run --parallel-mode -m unittest discover -s tools/tests -p 'test_*.py'",
    "python3 .agents/skills/os-dev/scripts/validate_skill.py",
    "coverage run --parallel-mode -m unittest discover -s .agents/skills/os-dev/tests -p 'test_*.py'",
)
# test_roadmap_runner needs default SIGINT/SIGQUIT, which a `&` job of a non-interactive shell loses.
HOST_PYTHON_FOREGROUND = "coverage run --parallel-mode -m unittest discover -s tools/tests -p 'test_*.py'"
# Stands in for coverage and python3: records argv and ignored signals, fails on FAKE_FAIL.
HOST_PYTHON_FAKE = f"""#!{sys.executable} -I
import json, os, shlex, signal, sys
argv = [os.path.basename(sys.argv[0])] + sys.argv[1:]
ignored = [s.name for s in (signal.SIGINT, signal.SIGQUIT) if signal.getsignal(s) == signal.SIG_IGN]
record = os.open(os.environ["FAKE_RECORD"], os.O_WRONLY | os.O_APPEND | os.O_CREAT)
os.write(record, (json.dumps({{"argv": argv, "ignored": ignored}}) + "\\n").encode())
print("ran:", shlex.join(argv))
sys.exit(1 if shlex.join(argv) == os.environ["FAKE_FAIL"] else 0)
"""


def default_entry_signals():
    """Enter the step shell as a runner does, with SIGINT and SIGQUIT at their defaults."""
    for number in (signal.SIGINT, signal.SIGQUIT):
        signal.signal(number, signal.SIG_DFL)


# Non-required workflows skip only documentation and project metadata. The
# required ci.yml gates stay unfiltered (see test_required_gates_...).
METADATA_PATHS_IGNORE = [
    "docs/**",
    "**.md",
    ".github/ISSUE_TEMPLATE/**",
    ".github/DISCUSSION_TEMPLATE/**",
    ".github/workflows/project-classification.yml",
    "tools/project_classification.py",
    "tests/test_project_classification.py",
]
# "@nonnative" (#494 phase 5 C2): the pull_request trigger of these native workflows also
# skips the catalogue's core_only tooling metadata; their @metadata push run on main is the
# backstop. native-hid-input and native-xhci run plugins/** and stay on METADATA.
NONNATIVE_PATHS_IGNORE = METADATA_PATHS_IGNORE + [
    ".agents/**",
    ".claude/**",
    ".claude-plugin/**",
    ".codex/**",
    "plugins/**",
    ".c8rc.json",
    ".coveragerc",
    ".coderabbit.yaml",
    "LICENSE",
    "MPL-2.0-FILES.txt",
]
NONNATIVE_WORKFLOWS = {
    "entropy-keys", "nas-core", "native-display-input", "native-foundation",
    "native-hw-inventory", "native-power", "native-rtc", "native-smp", "native-system",
    "native-window-session", "net-broker-quality", "process-management", "signed-updates",
}


def github_path_filter_matches(pattern, path):
    """Match GitHub path-filter globs: '*' stays within a segment, '**' spans."""
    regex, i = "", 0
    while i < len(pattern):
        if pattern.startswith("**", i):
            regex += ".*"
            i += 2
        elif pattern[i] == "*":
            regex += "[^/]*"
            i += 1
        else:
            regex += re.escape(pattern[i])
            i += 1
    return re.fullmatch(regex, path) is not None


def uses_reference_is_pinned(reference):
    """Accept immutable external references and checkout-contained local actions."""
    if not isinstance(reference, str) or not reference:
        return False
    if reference.startswith("./"):
        relative = reference[2:]
        if not relative or "\\" in relative or "@" in relative:
            return False
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            return False
        resolved = (ROOT / path).resolve()
        return resolved.is_relative_to(ROOT.resolve()) and resolved.exists()
    if reference.startswith("docker://"):
        return re.fullmatch(
            r"docker://[^\s@]+@sha256:[0-9a-f]{64}", reference
        ) is not None
    source, separator, revision = reference.rpartition("@")
    if not separator or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        return False
    parts = source.split("/")
    return len(parts) >= 2 and all(
        part not in {"", ".", ".."}
        and re.fullmatch(r"[A-Za-z0-9_.-]+", part) is not None
        for part in parts
    )


MILESTONE_DOCUMENT = ROOT / "docs/GITHUB-MILESTONES.md"
ROADMAP_DOCUMENT = ROOT / "docs/ROADMAP.md"
CANONICAL_MILESTONE_TITLES = (
    "M6 — Storage and Recovery",
    "M7 — Native Product Integration",
    "M8 — Networking and Security Hardening",
    "M9 — First Reference PC",
    "M10 — Second Architecture",
    "M11 — Experimental Community Release",
)
MILESTONE_FIELDS = (
    "Required outcome",
    "Acceptance evidence",
    "Decision gate",
    "Canonical sources",
)
CANONICAL_MILESTONE_LINKS = (
    "https://github.com/simonvanieperen/BaryonOS/blob/main/"
    "docs/ROADMAP.md#technical-milestones",
    "https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md",
)
LEGACY_MILESTONE_DESTINATIONS = {
    "K06 — Multicore and process isolation": "M7",
    "B02 / S09 — Encrypted storage and backups": "M6",
    "U02–U06 — A native desktop for everyday use": "M7",
    "N02–N04 — Internet, TLS and network permissions": "M8",
    "A02–A04 — Installing apps and using them daily": "M7",
    "P01–P04 — NAS, synchronisation and multiple devices": "M8",
    "L01 — Local system AI": "M7",
    "H02–H06 — Hardware, drivers and power": "M9",
    "B04–B06 — Chain of trust and security review": "M8",
    "R01–R03 — Full test release and maintenance": "M11",
}


class GitHubMilestoneDocumentContractTests(unittest.TestCase):
    """Validate checked-in governance, not the live GitHub metadata state."""

    @classmethod
    def setUpClass(cls):
        """Decode the actual tracked bytes and load the canonical roadmap."""
        cls.raw = MILESTONE_DOCUMENT.read_bytes()
        cls.text = cls.raw.decode("utf-8", errors="strict")
        cls.roadmap = ROADMAP_DOCUMENT.read_text(encoding="utf-8")

    def milestone_section(self, title):
        """Return one exact milestone body without including later sections."""
        heading = f"## {title}\n"
        self.assertEqual(self.text.count(heading), 1, title)
        body = self.text.split(heading, 1)[1]
        return body.split("\n## ", 1)[0]

    def test_document_is_normalized_utf8_without_mojibake(self):
        """Copy-ready titles must retain their intended Unicode punctuation."""
        self.assertFalse(self.raw.startswith(b"\xef\xbb\xbf"), "unexpected UTF-8 BOM")
        self.assertEqual(self.text, unicodedata.normalize("NFC", self.text))
        for marker in ("\ufffd", "Ã", "Â", "â€"):
            with self.subTest(marker=marker):
                self.assertNotIn(marker, self.text)

    def test_exact_titles_match_the_canonical_roadmap(self):
        """Keep one ordered M6–M11 title family in both active documents."""
        titles = re.findall(r"^## (M(?:6|7|8|9|10|11)\b[^\n]*)$", self.text, re.MULTILINE)
        self.assertEqual(titles, list(CANONICAL_MILESTONE_TITLES))
        for title in CANONICAL_MILESTONE_TITLES:
            with self.subTest(title=title):
                self.assertEqual(self.roadmap.count(f"**{title}**"), 1)

    def test_each_description_has_four_fields_and_both_sources(self):
        """Every copyable description keeps the complete governance contract."""
        for title in CANONICAL_MILESTONE_TITLES:
            section = self.milestone_section(title)
            fields = re.findall(r"^\*\*([^*:\n]+):\*\*", section, re.MULTILINE)
            with self.subTest(title=title):
                self.assertEqual(fields, list(MILESTONE_FIELDS))
                for link in CANONICAL_MILESTONE_LINKS:
                    self.assertEqual(section.count(link), 1, link)

                roadmap_row = re.search(
                    rf"^\| \*\*{re.escape(title)}\*\* \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$",
                    self.roadmap,
                    re.MULTILINE,
                )
                self.assertIsNotNone(roadmap_row)
                for field, expected in zip(MILESTONE_FIELDS[:3], roadmap_row.groups()):
                    value = re.search(
                        rf"^\*\*{re.escape(field)}:\*\*\s*(.*?)(?=^\*\*|\Z)",
                        section,
                        re.MULTILINE | re.DOTALL,
                    )
                    self.assertIsNotNone(value)
                    self.assertEqual(
                        " ".join(value.group(1).split()),
                        " ".join(expected.split()),
                        field,
                    )

    def test_legacy_objects_have_one_closing_migration_route(self):
        """Ten rubric objects map once, close after migration, and retain history."""
        rows = re.findall(
            r"^\| `([^`]+)` \| (M(?:6|7|8|9|10|11)) \|",
            self.text,
            re.MULTILINE,
        )
        self.assertEqual(len(rows), len(LEGACY_MILESTONE_DESTINATIONS))
        self.assertEqual(dict(rows), LEGACY_MILESTONE_DESTINATIONS)

        compact = " ".join(self.text.split())
        for invariant in (
            "Re-query every issue assigned to a legacy object",
            "at most one primary GitHub Milestone",
            "close rather than delete all ten legacy objects",
            "descriptions and closed history remain available",
            "no parallel active taxonomy remains",
        ):
            with self.subTest(invariant=invariant):
                self.assertIn(invariant, compact)

    def test_accepted_criteria_are_not_presented_as_unfinished(self):
        """Assignments #39 and #41 cover only their current residual scope."""
        rows = {
            number: (destination.strip(), scope.strip())
            for number, destination, scope in re.findall(
                r"^\| \[#(39|41)\]\([^)]*\)[^|]*\| ([^|]+) \| ([^|]+) \|$",
                self.text,
                re.MULTILINE,
            )
        }
        self.assertEqual(set(rows), {"39", "41"})
        self.assertTrue(rows["39"][0].startswith("M8"))
        self.assertIn("N01 is accepted", rows["39"][1])
        self.assertTrue(rows["41"][0].startswith("M8"))
        self.assertIn("B03 is accepted", rows["41"][1])


class UniqueKeyLoader(yaml.BaseLoader):
    """Keep `on` and boolean scalars as text and reject overwritten YAML keys."""

    def construct_mapping(self, node, deep=False):
        """Fail instead of silently accepting a duplicate workflow setting."""
        result = {}
        for key_node, value_node in node.value:
            key = self.construct_object(key_node, deep=deep)
            if key in result:
                raise yaml.constructor.ConstructorError(
                    None, None, f"duplicate YAML key: {key}", key_node.start_mark,
                )
            result[key] = self.construct_object(value_node, deep=deep)
        return result


class DependabotContractTests(unittest.TestCase):
    """Cover independent Cargo workspaces without multiplying update PRs."""

    def setUp(self):
        self.config = yaml.load(
            (ROOT / ".github/dependabot.yml").read_text(encoding="utf-8"),
            Loader=UniqueKeyLoader,
        )

    def test_each_owned_cargo_workspace_is_covered_once(self):
        """Find workspace boundaries, excluding members and vendored packages."""
        manifests = subprocess.check_output(
            ["git", "ls-files", "-z", "--", "Cargo.toml", "**/Cargo.toml"],
            cwd=ROOT,
        ).decode().split("\0")
        expected = set()
        for manifest in filter(None, manifests):
            data = tomllib.loads((ROOT / manifest).read_text(encoding="utf-8"))
            if "workspace" in data:
                directory = Path(manifest).parent.as_posix()
                expected.add("/" if directory == "." else "/" + directory)
        self.assertTrue(expected)
        configured = []
        for update in self.config["updates"]:
            if update["package-ecosystem"] == "cargo":
                self.assertNotEqual("directory" in update, "directories" in update)
                configured.extend(
                    update["directories"] if "directories" in update
                    else [update["directory"]]
                )
        self.assertEqual(len(configured), len(set(configured)), "duplicate workspace")
        self.assertEqual(set(configured), expected)

    def test_multi_directory_updates_are_monthly_and_grouped(self):
        """Keep same-dependency updates together across independent roots."""
        for update in self.config["updates"]:
            if update["package-ecosystem"] != "cargo" or "directories" not in update:
                continue
            with self.subTest(directories=update["directories"]):
                self.assertEqual(update["schedule"]["interval"], "monthly")
                self.assertEqual(update["open-pull-requests-limit"], "2")
                self.assertEqual(update["groups"], {
                    "cargo-independent": {
                        "patterns": ["*"], "group-by": "dependency-name",
                    },
                })


class WorkflowContractTests(unittest.TestCase):
    """Detect runner, credential, gate and deduplication regressions."""

    @classmethod
    def setUpClass(cls):
        """Read the actual workflows and local composite actions once per run."""
        cls.workflows = {}
        cls.actions = {}
        for directory, pattern, destination in [
            (ROOT / ".github/workflows", "*.y*ml", cls.workflows),
            (ROOT / ".github/actions", "**/action.y*ml", cls.actions),
        ]:
            for path in sorted(directory.glob(pattern)):
                destination[path.relative_to(ROOT).as_posix()] = yaml.load(
                    path.read_text(encoding="utf-8"), Loader=UniqueKeyLoader,
                )

    def jobs(self):
        """Yield every job, including manual and private-only maintenance jobs."""
        for path, workflow in self.workflows.items():
            for job_id, job in workflow["jobs"].items():
                yield path, job_id, job

    def test_rust_coverage_report_survives_the_actual_collection_step(self):
        self.assert_coverage_report_retained("Rust", 2349735, 4 * 1024 * 1024)

    def test_python_coverage_report_survives_the_actual_collection_step(self):
        self.assert_coverage_report_retained("Python", 1057659, 2 * 1024 * 1024)

    def assert_coverage_report_retained(self, language, observed_size, file_limit):
        """Archive an observed failing XML size through the actual caller and shell step."""
        steps = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]["steps"]
        caller = next(step for step in steps
                      if step.get("name") == f"Preserve {language} coverage artifact")
        action = self.actions[EVIDENCE_ACTION]
        inputs = {name: spec.get("default", "") for name, spec in action["inputs"].items()}
        inputs.update(caller["with"])
        inputs["job-status"] = "success"
        collect = next(step for step in action["runs"]["steps"] if step.get("id") == "collect")
        header, footer = b'<coverage><packages/>', b'</coverage>\n'
        original = header + b" " * (observed_size - len(header) - len(footer)) + footer
        with tempfile.TemporaryDirectory(prefix="coverage-evidence-") as directory:
            root = Path(directory)
            workspace, runner = root / "workspace", root / "runner"
            (workspace / "tools").mkdir(parents=True)
            runner.mkdir()
            (workspace / "tools/ci_evidence.py").write_bytes((ROOT / "tools/ci_evidence.py").read_bytes())
            report = workspace / inputs["path"]
            report.parent.mkdir(parents=True)
            report.write_bytes(original)
            env = dict(os.environ, GITHUB_WORKSPACE=str(workspace), RUNNER_TEMP=str(runner),
                       GITHUB_OUTPUT=str(root / "outputs"), GITHUB_STEP_SUMMARY=str(root / "summary"))
            for name, expression in collect["env"].items():
                match = re.fullmatch(r"\$\{\{ inputs\.([\w-]+) \}\}", expression)
                self.assertIsNotNone(match)
                env[name] = str(inputs[match.group(1)])
            result = subprocess.run(["bash", "-e", "-o", "pipefail", "-c", collect["run"]],
                                    cwd=workspace, env=env, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)
            outputs = dict(line.split("=", 1) for line in (root / "outputs").read_text().splitlines())
            retained = Path(outputs["path"]) / inputs["path"]
            self.assertTrue(retained.is_file(), f"The {language} XML was omitted from compact evidence")
            self.assertEqual(retained.read_bytes(), original)
            receipt = json.loads((Path(outputs["path"]) / "_meta/receipt.json").read_text())
            self.assertEqual(receipt["retained"][0]["sha256"], hashlib.sha256(original).hexdigest())
            self.assertEqual(receipt["limits"]["file_bytes"], file_limit)
            self.assertEqual(receipt["limits"]["total_bytes_including_receipt"], 8 * 1024 * 1024)
            self.assertEqual(receipt["required_paths"], [inputs["path"]])
            self.assertEqual(outputs["required-files"], "true")

    def window_job(self):
        """Return the five-leg native window matrix job."""
        return self.workflows[WINDOW_WORKFLOW]["jobs"]["native-window-session"]

    def window_evidence_fixture(self, leg, *, serial_size=4 * 1024 * 1024):
        """Make synthetic archive inputs for one leg, never a native execution or acceptance report."""
        temporary = tempfile.TemporaryDirectory(prefix="window-evidence-")
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        workspace, runner = root / "workspace", root / "runner"
        (workspace / "tools").mkdir(parents=True)
        runner.mkdir()
        (workspace / "tools/ci_evidence.py").write_bytes((ROOT / "tools/ci_evidence.py").read_bytes())
        source = "a" * 40
        combination, = [entry for entry in matrix_combinations(self.window_job()) if entry["leg"] == leg]
        scale = int(combination["scale"])
        env = dict(os.environ, GITHUB_WORKSPACE=str(workspace), RUNNER_TEMP=str(runner),
                   GITHUB_SHA=source, NOS_WINDOW_LEG=leg, NOS_WINDOW_SCALE=combination["scale"],
                   CI_JOB_STATUS="success", GITHUB_OUTPUT=str(root / "index-output"))
        roles = {"native": ["native"], "library": ["library-write", "library-cold"],
                 "switcher": ["switcher"]}[combination["mode"]]
        expected = {}
        reports = {}
        serial_line = b"Synthetic archive-size fixture; not QEMU evidence.\n"
        serial = (serial_line * (serial_size // len(serial_line) + 1))[:serial_size]
        self.assertEqual(len(serial), serial_size)
        for number, role in enumerate(roles, 1):
            run = workspace / f"build/window-session-ci/{leg}/run/window-{number:032x}"
            run.mkdir(parents=True)
            report = dict(source_commit=source, run_directory=str(run), scale=scale,
                          workspace_requested=role == "native", clipboard_requested=role == "native",
                          library_requested=role.startswith("library-"), library_cold=role == "library-cold",
                          switcher_requested=role == "switcher", passed=True,
                          serial_sha256=hashlib.sha256(serial).hexdigest(),
                          qmp_trace=[{"synthetic_archive_fixture": role}])
            data = json.dumps(report).encode()
            # Exact failed scale-2 report size from hosted artifact 14884099633.
            data += b" " * (1553981 - len(data))
            for name, content in (("result.json", data), ("serial.txt", serial)):
                path = run / name
                path.write_bytes(content)
                expected[path.relative_to(workspace).as_posix()] = content
            reports[role] = run / "result.json"
            # Existing summary/copy paths deliberately duplicate complete reports.
            (workspace / f"build/window-session-ci/{leg}/window-session-report.json").write_bytes(data)
            if role != "native":
                name = f"switcher-native-scale{scale}.json" if role == "switcher" else role + "-report.json"
                copy = workspace / "build/window-session-ci/evidence" / name
                copy.parent.mkdir(parents=True, exist_ok=True)
                copy.write_bytes(data)
        for relative in ("evidence/source.json", leg + "/manifest.json"):
            path = workspace / "build/window-session-ci" / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            data = b'{"synthetic_archive_metadata": true}\n'
            path.write_bytes(data)
            expected[path.relative_to(workspace).as_posix()] = data
        return workspace, env, reports, expected, combination

    @staticmethod
    def action_outputs(path):
        """Read the runner's single-line and delimited output-file formats."""
        result = {}
        lines = iter(path.read_text().splitlines())
        for line in lines:
            if "<<" in line:
                key, delimiter = line.split("<<", 1)
                value = []
                for item in lines:
                    if item == delimiter:
                        break
                    value.append(item)
                result[key] = "\n".join(value)
            else:
                key, value = line.split("=", 1)
                result[key] = value
        return result

    def window_index(self, workspace, env):
        """Run the real proof-locator step of the native window matrix."""
        index, = [step for step in self.window_job()["steps"] if step.get("id") == "window-proof-paths"]
        return subprocess.run(["bash", "-e", "-o", "pipefail", "-c", index["run"]],
                              cwd=workspace, env=env, capture_output=True, text=True, check=False)

    def preserve_inputs(self, step, combination, env, outputs=None):
        """Render one preserve-evidence call for a matrix combination and step outputs."""
        action = self.actions[EVIDENCE_ACTION]
        inputs = {name: spec.get("default", "") for name, spec in action["inputs"].items()}
        inputs.update(step["with"])
        inputs["job-status"] = env["CI_JOB_STATUS"]
        inputs["full-evidence"] = "false"  # Ordinary PR fixture; no uploads are executed.
        for key in ("path", "required-paths", "name", "package-path"):
            value = render_matrix(inputs[key], combination)
            value = re.sub(r"\$\{\{ matrix\.scale == 1 && '([^']+)' \|\| '' \}\}",
                           lambda match: match.group(1) if combination["scale"] == "1" else "", value)
            value = re.sub(r"\$\{\{ steps\.window-proof-paths\.outputs\.([\w-]+) \}\}",
                           lambda match: outputs[match.group(1)], value)
            self.assertNotIn("${{", value)
            inputs[key] = value
        return inputs

    def window_evidence_callers(self, workspace, env, combination):
        """Run the real path-index step and render this leg's preservation inputs."""
        result = self.window_index(workspace, env)
        self.assertEqual(result.returncode, 0, result.stderr)
        outputs = self.action_outputs(Path(env["GITHUB_OUTPUT"]))
        callers = []
        for step in self.window_job()["steps"]:
            if step.get("uses") != "./.github/actions/preserve-evidence":
                continue
            if step.get("if") == "${{ always() && matrix.mode == 'library' }}" and combination["mode"] != "library":
                continue
            callers.append((step.get("id", "final"), self.preserve_inputs(step, combination, env, outputs)))
        return callers

    def collect_window_fixture(self, workspace, base_env, inputs, name):
        """Execute the actual composite collector shell, with its unchanged bounds."""
        collect = next(step for step in self.actions[EVIDENCE_ACTION]["runs"]["steps"]
                       if step.get("id") == "collect")
        output = workspace.parent / (name + ".outputs")
        env = dict(base_env, GITHUB_OUTPUT=str(output),
                   GITHUB_STEP_SUMMARY=str(workspace.parent / (name + ".summary")))
        for key, expression in collect["env"].items():
            match = re.fullmatch(r"\$\{\{ inputs\.([\w-]+) \}\}", expression)
            self.assertIsNotNone(match)
            env[key] = str(inputs[match.group(1)])
        result = subprocess.run(["bash", "-e", "-o", "pipefail", "-c", collect["run"]],
                                cwd=workspace, env=env, capture_output=True, text=True, check=False)
        outputs = self.action_outputs(output)
        directory = Path(outputs["path"])
        receipt = json.loads((directory / "_meta/receipt.json").read_text())
        return result, directory, receipt, outputs

    def test_window_every_leg_retains_its_original_proof_pairs(self):
        for leg in WINDOW_LEGS:
            with self.subTest(leg=leg):
                self.assert_window_evidence_plan(leg)

    def test_window_observed_scale2_report_and_serial_are_not_omitted(self):
        self.assert_window_evidence_plan("native-scale2", serial_size=1556536)

    def assert_window_evidence_plan(self, leg, **options):
        """Large independent runs must not compete with duplicates in one 7 MiB payload."""
        workspace, env, reports, expected, combination = self.window_evidence_fixture(leg, **options)
        retained = {}
        proof_count = 0
        for name, inputs in self.window_evidence_callers(workspace, env, combination):
            result, directory, receipt, outputs = self.collect_window_fixture(workspace, env, inputs, name)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["limits"]["total_bytes_including_receipt"], 8 * 1024 * 1024)
            self.assertLessEqual(sum(item["size_bytes"] for item in receipt["retained"]), 7 * 1024 * 1024)
            self.assertEqual(receipt["unretained_required_paths"], [])
            for item in receipt["retained"]:
                data = (directory / item["path"]).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(), item["sha256"])
                retained[item["path"]] = data
            if name != "final":
                proof_count += 1
                self.assertEqual(len(receipt["required_paths"]), 2)
                self.assertEqual(set(receipt["required_paths"]),
                                 {item["path"] for item in receipt["retained"]})
                self.assertEqual(receipt["limits"]["file_bytes"], 4 * 1024 * 1024)
                self.assertEqual(outputs["required-files"], "true")
                self.assertEqual(inputs["full-evidence"], "false")
                self.assertEqual(inputs["package-path"], "")
        for path, content in expected.items():
            self.assertIn(path, retained, "Required native proof was omitted: " + path)
            self.assertEqual(retained[path], content)
        self.assertEqual(proof_count, len(reports))

    def test_window_required_proof_omissions_fail_the_actual_collector(self):
        """Missing, oversized and aggregate-excess evidence must fail a successful job."""
        for cause in ("missing", "per-file", "aggregate"):
            with self.subTest(cause=cause):
                workspace, env, reports, _, combination = self.window_evidence_fixture("native-scale2")
                callers = dict(self.window_evidence_callers(workspace, env, combination))
                inputs = callers["window-native-proof"]
                serial = reports["native"].with_name("serial.txt")
                if cause == "missing":
                    serial.unlink()
                elif cause == "per-file":
                    serial.write_bytes(b"x" * (4 * 1024 * 1024 + 1))
                else:
                    report = reports["native"]
                    report.write_bytes(report.read_bytes().ljust(4 * 1024 * 1024, b" "))
                result, _, receipt, outputs = self.collect_window_fixture(workspace, env, inputs, cause)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertEqual(outputs["required-files"], "false")
                self.assertIn(serial.relative_to(workspace).as_posix(), receipt["unretained_required_paths"])

    def test_window_failed_run_and_first_library_run_remain_diagnostic_evidence(self):
        """A later failed case does not discard earlier proof or invent a job pass."""
        workspace, env, reports, _, combination = self.window_evidence_fixture("library")
        env["CI_JOB_STATUS"] = "failure"
        raw = json.loads(reports["library-cold"].read_text())
        raw["passed"] = False
        reports["library-cold"].write_text(json.dumps(raw))
        callers = dict(self.window_evidence_callers(workspace, env, combination))
        for identifier, role in (("window-native-proof", "library-write"),
                                 ("window-library-cold-proof", "library-cold")):
            result, _, receipt, _ = self.collect_window_fixture(workspace, env, callers[identifier], role)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["job_status_before_archive"], "failure")
            self.assertEqual(receipt["unretained_required_paths"], [])
            self.assertIn(reports[role].relative_to(workspace).as_posix(),
                          {item["path"] for item in receipt["retained"]})
        # A failed leg without any run keeps its index (and failure) instead of crashing.
        workspace, env, reports, _, combination = self.window_evidence_fixture("switcher-scale2")
        env["CI_JOB_STATUS"] = "failure"
        reports["switcher"].unlink()
        self.window_evidence_callers(workspace, env, combination)

    def test_window_missing_normal_manifest_fails_despite_other_retained_proof(self):
        """The packaging job's collection binds the normal build it packaged."""
        workspace, env, _, _, _ = self.window_evidence_fixture("native-scale2")
        job = self.workflows[WINDOW_WORKFLOW]["jobs"]["normal-and-package"]
        preserve, = [step for step in job["steps"] if step.get("uses") == "./.github/actions/preserve-evidence"]
        prefix = "build/window-session-ci/"
        self.assertEqual(set(self.preserve_inputs(preserve, {"scale": "1"}, env)["required-paths"].split()), {
            prefix + "evidence/source.json", prefix + "normal/manifest.json",
            prefix + "evidence/package.json", prefix + "evidence/package.sha256"})
        inputs = self.preserve_inputs(preserve, {"scale": "2"}, env)
        missing = prefix + "normal/manifest.json"
        result, _, receipt, _ = self.collect_window_fixture(workspace, env, inputs, "missing-normal")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["unretained_required_paths"], [missing])
        (workspace / missing).parent.mkdir(parents=True)
        (workspace / missing).write_bytes(b'{"synthetic_archive_metadata": true}\n')
        result, _, receipt, _ = self.collect_window_fixture(workspace, env, inputs, "present-normal")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_window_proof_index_rejects_stale_or_ambiguous_source(self):
        """A previous head or two matching run directories cannot supply current proof."""
        for cause in ("stale", "ambiguous", "missing"):
            with self.subTest(cause=cause):
                workspace, env, reports, _, _ = self.window_evidence_fixture("native-scale2")
                self.assertEqual(self.window_index(workspace, env).returncode, 0)
                report = reports["native"]
                if cause == "stale":
                    data = json.loads(report.read_text())
                    data["source_commit"] = "b" * 40
                    report.write_text(json.dumps(data))
                elif cause == "ambiguous":
                    other = report.parent.with_name("window-" + "f" * 32)
                    other.mkdir()
                    data = json.loads(report.read_text())
                    data["run_directory"] = str(other)
                    (other / "result.json").write_text(json.dumps(data))
                else:
                    report.unlink()
                result = self.window_index(workspace, env)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                expected = {"stale": "source or run directory differs", "ambiguous": "Ambiguous native proof",
                            "missing": "Missing native proof"}[cause]
                self.assertIn(expected, result.stderr)

    def steps(self):
        """Include nested local-action steps in supply-chain and error checks."""
        for path, job_id, job in self.jobs():
            for step in job.get("steps", []):
                yield path, job_id, step
        for path, action in self.actions.items():
            for step in action.get("runs", {}).get("steps", []):
                yield path, "composite", step

    def uses_references(self):
        """Yield job, workflow-step and composite-action executable references."""
        for path, job_id, job in self.jobs():
            self.assertIsInstance(
                job, dict, f"{path} job {job_id} must be a mapping"
            )
            if "uses" in job:
                yield path, f"job:{job_id}", job["uses"]
            steps = job.get("steps", [])
            self.assertIsInstance(
                steps, list, f"{path} job {job_id} steps must be a list"
            )
            for index, step in enumerate(steps):
                self.assertIsInstance(
                    step, dict, f"{path} job {job_id} step {index} must be a mapping"
                )
                if "uses" in step:
                    yield path, f"job:{job_id}:step:{index}", step["uses"]
        for path, action in self.actions.items():
            runs = action.get("runs", {})
            self.assertIsInstance(runs, dict, f"{path} runs must be a mapping")
            steps = runs.get("steps", [])
            self.assertIsInstance(steps, list, f"{path} composite steps must be a list")
            for index, step in enumerate(steps):
                self.assertIsInstance(
                    step, dict, f"{path} composite step {index} must be a mapping"
                )
                if "uses" in step:
                    yield path, f"composite:step:{index}", step["uses"]

    def test_all_jobs_use_standard_hosted_ubuntu(self):
        """No custom label, dynamic selector or persistent runner can return."""
        # ubuntu-26.04 pins QEMU 10.2.1 for K06; ubuntu-slim (1 vCPU, hard 15-minute cap) only for short jobs.
        self.assertTrue(self.workflows)
        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                self.assertTrue(job.get("runs-on") in ("ubuntu-24.04", "ubuntu-26.04") or (job.get("runs-on") == "ubuntu-slim" and int(job.get("timeout-minutes", 360)) <= 15), job.get("runs-on"))

    def test_volume_gate_stage_counts_match_plan(self):
        """Pin the volume report count and step label to the real gate PLAN."""
        # Parse rather than import: importing pulls in the build and QEMU runner chain.
        source = (ROOT / "tools/test_volume.py").read_text(encoding="utf-8")
        plan = next(node.value for node in ast.parse(source).body
                    if isinstance(node, ast.Assign)
                    and [getattr(target, "id", None) for target in node.targets] == ["PLAN"])
        stages = len(ast.literal_eval(plan))
        pilot = "\n".join(step.get("run", "") for step in
                          self.workflows[".github/workflows/kvm-pilot.yml"]["jobs"]["kvm-pilot"]["steps"])
        self.assertEqual(re.findall(r'"volume": \("run/volume-\*/result\.json", (\d+),', pilot),
                         [str(stages)])
        names = [step.get("name", "") for step in
                 self.workflows[".github/workflows/volume-acceptance.yml"]["jobs"]["m6e"]["steps"]
                 if step.get("run") == "python3 -B -u tools/test_volume.py"]
        self.assertEqual(names, [f"M6e/S06 native volume gate ({stages} QEMU runs)"])

    def test_kvm_pilot_is_manual_hosted_and_cannot_replace_tcg_acceptance(self):
        """Keep KVM opt-in, fail-closed, narrow, and outside required checks."""
        path = ".github/workflows/kvm-pilot.yml"
        workflow = self.workflows[path]
        self.assertEqual(set(workflow["on"]), {"workflow_dispatch"})
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        self.assertEqual(set(workflow["jobs"]), {"kvm-pilot"})
        job = workflow["jobs"]["kvm-pilot"]
        self.assertEqual(job["runs-on"], "ubuntu-24.04")
        self.assertEqual(job["env"]["NOS_QEMU_ACCEL"], "kvm")
        self.assertEqual(job["strategy"]["matrix"]["gate"], ["system", "volume", "window"])

        report_steps = [
            step for step in job["steps"]
            if step.get("name") == "Require passing reports bound to KVM"
        ]
        self.assertEqual(len(report_steps), 1)
        self.assertEqual(report_steps[0].get("if"), "${{ always() }}")

        commands = "\n".join(step.get("run", "") for step in job["steps"])
        for required in (
            "test -c /dev/kvm", "test -r /dev/kvm", "test -w /dev/kvm",
            "qemu-system-x86_64 -accel help", "tools/test_system.py",
            "tools/test_volume.py", "native-window-session-probe",
            "tools/run_window_session.py", 'acceleration != "kvm"',
        ):
            self.assertIn(required, commands)
        for excluded in (
            "tools/test_net_system.py", "tools/test_tcp_system.py",
            "tools/run_smp.py", "tools/test_memory_system.py",
        ):
            self.assertNotIn(excluded, commands)

        for other_path in self.workflows:
            if other_path == path:
                continue
            with self.subTest(path=other_path):
                self.assertNotIn(
                    "NOS_QEMU_ACCEL",
                    (ROOT / other_path).read_text(encoding="utf-8"),
                )

    def test_native_network_fault_scenarios_preserve_separate_required_reports(self):
        """Merged revocation and backpressure probes must run without overwriting evidence."""
        service = self.workflows[".github/workflows/net-broker-quality.yml"]["jobs"]["native-network-service"]
        packet_step, = [step for step in service["steps"]
                       if "tools/run_network_service.py" in step.get("run", "")]
        self.assertNotIn("if", packet_step)
        commands = packet_step["run"].replace("\\\n", " ").splitlines()
        observed = {}
        for command in commands:
            tokens = shlex.split(command)
            if tokens:
                self.assertEqual(tokens[1:-1], ["python3", "-B", "-u", "tools/run_network_service.py", "--case"])
                self.assertNotIn(tokens[-1], observed)
                observed[tokens[-1]] = tokens[0]
        self.assertEqual(observed, {
            case: f"NOS_BUILD_DIR=$GITHUB_WORKSPACE/build/n01-ci-{case}"
            for case in ("normal", "client-exit", "revoke", "helper", "device-reset", "exhaustion")
        })
        preserve, = [step for step in service["steps"]
                     if step.get("uses") == "./.github/actions/preserve-evidence"]
        self.assertEqual(set(preserve["with"]["required-paths"].splitlines()), {
            f"build/n01-ci-{case}/network-service-{case}-report.json" for case in observed
        })
        self.assertIn("build/n01-ci-*/run/network-service-*/network-discard.bin",
                      preserve["with"]["path"].splitlines())

        system = self.workflows[".github/workflows/native-system.yml"]["jobs"]["native-system"]
        tcp_steps = [step for step in system["steps"]
                     if "tools/test_tcp_system.py" in step.get("run", "")]
        self.assertEqual([step["run"] for step in tcp_steps], [
            "python3 -B -u tools/test_tcp_system.py",
            "python3 -B -u tools/test_tcp_system.py --scenario loss-backpressure",
        ])
        for step in tcp_steps:
            self.assertNotIn("if", step)
        for key in ("NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR"):
            self.assertEqual(len({step["env"][key] for step in tcp_steps}), 2)
        preserve, = [step for step in system["steps"]
                     if step.get("uses") == "./.github/actions/preserve-evidence"]
        required = {"build/tcp-ci/tcp-system-gate.json", "build/tcp-loss-ci/tcp-system-gate.json"}
        self.assertLessEqual(required, set(preserve["with"]["required-paths"].splitlines()))
        self.assertLessEqual(required, set(preserve["with"]["path"].splitlines()))

    def test_public_prs_keep_minimal_token_permissions(self):
        """Allow narrowly scoped writes only for CodeQL, orphan-run cleanup, coverage uploads
        and the merge-hold re-check."""
        for path, workflow in self.workflows.items():
            with self.subTest(path=path):
                self.assertNotIn("pull_request_target", workflow["on"])
                permissions = workflow.get("permissions")
                self.assertIsInstance(permissions, dict)
                self.assertEqual(permissions.get("contents"), "read")
                self.assertLessEqual(set(permissions.values()), {"read", "none"})

        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                permissions = job.get("permissions", {})
                self.assertIsInstance(permissions, dict)

                if (path, job_id) == (".github/workflows/codeql.yml", "analyze"):
                    self.assertEqual(
                        permissions,
                        {
                            "actions": "read",
                            "contents": "read",
                            "security-events": "write",
                        },
                    )
                elif (path, job_id) == (
                    ".github/workflows/clear-stale-actions.yml", "cancel-orphans"
                ):
                    self.assertEqual(permissions, {"actions": "write"})
                elif (path, job_id) == (".github/workflows/merge-hold-recheck.yml", "recheck"):
                    # #494 B9: actions write only to re-run merge-hold.
                    self.assertEqual(
                        permissions,
                        {"actions": "write", "contents": "read", "pull-requests": "read"},
                    )
                elif (path, job_id) in (
                    (".github/workflows/ci.yml", "upload-coverage-rust"),
                    (".github/workflows/ci.yml", "upload-coverage-python"),
                    (".github/workflows/ci.yml", "upload-coverage-javascript"),
                ):
                    self.assertEqual(
                        permissions, {"contents": "read", "code-quality": "write"}
                    )
                else:
                    self.assertLessEqual(
                        set(permissions.values()), {"read", "none"}
                    )

    def test_codeql_uses_fast_pre_merge_and_extended_main_analysis(self):
        """Keep all languages pre-merge while deferring only extra lower-precision queries."""
        job = self.workflows[".github/workflows/codeql.yml"]["jobs"]["analyze"]
        self.assertEqual(job["strategy"]["matrix"]["language"], [
            "actions", "javascript-typescript", "python", "rust",
        ])
        init = [step for step in job["steps"]
                if step.get("uses", "").startswith("github/codeql-action/init@")]
        self.assertEqual(len(init), 2)
        fast, extended = init
        self.assertEqual(
            fast["if"],
            "github.event_name == 'pull_request' || github.event_name == 'merge_group'",
        )
        self.assertNotIn("queries", fast["with"])
        self.assertEqual(
            extended["if"],
            "github.event_name != 'pull_request' && github.event_name != 'merge_group'",
        )
        self.assertEqual(extended["with"]["queries"], "security-extended")
        for step in init:
            self.assertEqual(step["with"]["build-mode"], "none")
            self.assertEqual(step["with"]["languages"], "${{ matrix.language }}")

    def test_external_actions_are_pinned_to_full_commits(self):
        """All executable references are immutable or safe checkout-local paths."""
        references = list(self.uses_references())
        self.assertTrue(references, "no executable Actions references found")
        for path, scope, reference in references:
            with self.subTest(path=path, scope=scope, uses=reference):
                self.assertTrue(
                    uses_reference_is_pinned(reference),
                    f"mutable, invalid or checkout-escaping uses reference: {reference!r}",
                )

        fixtures = {
            "actions/checkout@" + "a" * 40: True,
            "owner/repository/.github/workflows/reuse.yml@" + "b" * 40: True,
            "./.github/actions/setup-python": True,
            "docker://ghcr.io/example/tool@sha256:" + "c" * 64: True,
            "actions/checkout@v7": False,
            "https://github.com/actions/checkout@" + "d" * 40: False,
            "owner/repository/../reuse.yml@" + "e" * 40: False,
            "./../outside": False,
            "../outside": False,
            "docker://ghcr.io/example/tool:latest": False,
            "docker://ghcr.io/example/tool@sha256:short": False,
        }
        for reference, expected in fixtures.items():
            with self.subTest(fixture=reference):
                self.assertEqual(uses_reference_is_pinned(reference), expected)

    def test_cargo_llvm_cov_is_a_checksum_pinned_release_binary(self):
        """#494 B3: the required host gate verifies the upstream archive before extracting it."""
        steps = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]["steps"]
        step, = [step for step in steps if step.get("name") == "Coverage tooling (cargo-llvm-cov, coverage.py, c8)"]
        self.assertNotIn("if", step)
        self.assertEqual(step.get("continue-on-error", "false"), "false")
        self.assertEqual(step["env"], {"CARGO_LLVM_COV_VERSION": CARGO_LLVM_COV_VERSION,
                                       "CARGO_LLVM_COV_SHA256": CARGO_LLVM_COV_SHA256})
        command = step["run"]
        self.assertTrue(command.startswith("set -euo pipefail\n"))
        self.assertNotIn("install cargo-llvm-cov", command)
        self.assertNotIn("|| true", command)
        self.assertIn("https://github.com/taiki-e/cargo-llvm-cov/releases/download/"
                      "v${CARGO_LLVM_COV_VERSION}/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz", command)
        checksum = command.index("sha256sum --check --strict")
        extraction = command.index("tar --extract")
        self.assertLess(checksum, extraction)
        self.assertIn("${CARGO_LLVM_COV_SHA256}", command[:checksum])
        self.assertLess(extraction, command.index('llvm-cov --version | grep -F "${CARGO_LLVM_COV_VERSION}"'))
        self.assertLess(steps.index(step), min(index for index, candidate in enumerate(steps)
                                               if "llvm-cov --locked" in candidate.get("run", "")))
        evidence = (ROOT / "docs/CI-EVIDENCE.md").read_text(encoding="utf-8")
        for fact in (CARGO_LLVM_COV_SHA256, CARGO_LLVM_COV_TAG_COMMIT, f"v{CARGO_LLVM_COV_VERSION}"):
            self.assertIn(fact, evidence)
        restore, = [step for step in steps if step.get("id") == "host-build-cache"]
        self.assertIn("target/llvm-cov-target", restore["with"]["path"].split())
        self.assertTrue(restore["with"]["key"].startswith("host-rust-v4-"))

    def test_cargo_llvm_cov_step_fails_closed(self):
        """#494 B3: a bad checksum installs nothing and a wrong version fails the step."""
        steps = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]["steps"]
        step, = [step for step in steps if step.get("name") == "Coverage tooling (cargo-llvm-cov, coverage.py, c8)"]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive_path = root / "cargo-llvm-cov-fixture.tar.gz"
            binary = b'#!/usr/bin/env bash\necho "cargo-llvm-cov ${FAKE_LLVM_COV_VERSION}"\n'
            with tarfile.open(archive_path, "w:gz") as archive:
                entry = tarfile.TarInfo("cargo-llvm-cov")
                entry.mode = 0o755
                entry.size = len(binary)
                archive.addfile(entry, io.BytesIO(binary))
            fake_bin = root / "bin"
            fake_bin.mkdir()
            fakes = {
                "curl": 'while (( $# )); do [[ "$1" == --output ]] && output="$2"; shift; done\n'
                        'cp -- "$LLVM_COV_FIXTURE" "$output"\n',
                # cargo +toolchain llvm-cov --version runs the extracted subcommand binary.
                "cargo": 'exec "$HOME/.cargo/bin/cargo-$2" "${@:3}"\n',
                "python3": "exit 0\n",
                "npm": "exit 0\n",
            }
            for name, body in fakes.items():
                (fake_bin / name).write_text("#!/usr/bin/env bash\nset -euo pipefail\n" + body, encoding="utf-8")
                (fake_bin / name).chmod(0o755)
            good_hash = hashlib.sha256(archive_path.read_bytes()).hexdigest()

            def run(label, checksum, version):
                home = root / label
                (home / "temp").mkdir(parents=True)
                environment = dict(os.environ, PATH=f"{fake_bin}{os.pathsep}{os.environ['PATH']}", HOME=str(home),
                                   RUNNER_TEMP=str(home / "temp"), LLVM_COV_FIXTURE=str(archive_path),
                                   CARGO_LLVM_COV_VERSION=CARGO_LLVM_COV_VERSION, CARGO_LLVM_COV_SHA256=checksum,
                                   FAKE_LLVM_COV_VERSION=version)
                result = subprocess.run(["bash", "-c", step["run"]], cwd=ROOT, env=environment,
                                        capture_output=True, text=True, check=False)
                return result.returncode, (home / ".cargo/bin/cargo-llvm-cov").exists()

            self.assertEqual(run("success", good_hash, CARGO_LLVM_COV_VERSION), (0, True))
            code, installed = run("bad-checksum", "0" * 64, CARGO_LLVM_COV_VERSION)
            self.assertNotEqual(code, 0)
            self.assertFalse(installed)
            self.assertNotEqual(run("wrong-version", good_hash, "0.0.1")[0], 0)

    def host_python_step(self):
        steps = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]["steps"]
        step, = [step for step in steps if step.get("name") == HOST_PYTHON_STEP]
        return step

    def run_with_host_python_fakes(self, script, work, shell=("bash", "-e"), failing=""):
        """Run step text in work, with coverage and python3 replaced by HOST_PYTHON_FAKE."""
        fake_bin = work.parent / "bin"
        if not fake_bin.is_dir():
            fake_bin.mkdir()
            for name in ("coverage", "python3"):
                (fake_bin / name).write_text(HOST_PYTHON_FAKE, encoding="utf-8")
                (fake_bin / name).chmod(0o755)
        (work / "temp").mkdir(parents=True, exist_ok=True)
        record = work / "record.jsonl"
        environment = dict(os.environ, PATH=f"{fake_bin}{os.pathsep}{os.environ['PATH']}",
                           RUNNER_TEMP=str(work / "temp"), FAKE_RECORD=str(record),
                           FAKE_FAIL=shlex.join(shlex.split(failing)))
        result = subprocess.run([*shell, "-c", script], cwd=work, env=environment,
                                preexec_fn=default_entry_signals, capture_output=True,
                                text=True, timeout=60, check=False)
        records = [json.loads(line) for line in record.read_text().splitlines()] if record.exists() else []
        return result, records

    def assert_host_python_lanes(self, run):
        """Run the step against fakes: every command once, tools/tests in front, any failing lane fails."""
        lines = [line.strip() for line in run.splitlines()]
        commands = [line.removesuffix(" || rc=1") for line in lines
                    if line.startswith(("mkdir ", "python3 ", "coverage "))]
        self.assertEqual(sorted(commands), sorted(HOST_PYTHON_COMMANDS))
        self.assertEqual(lines[:2], list(HOST_PYTHON_COMMANDS[:2]))
        self.assertIn(HOST_PYTHON_FOREGROUND + " || rc=1", lines)
        # Only a printed log is removed: a failed print keeps it for the leftover-log step.
        self.assertIn('cat "$RUNNER_TEMP/python-lane-$lane.log" && rm -f "$RUNNER_TEMP/python-lane-$lane.log" || rc=1',
                      lines)
        launches =[line for line in lines if re.search(r"(^|[^&])&$", line)]
        pids = [match.group(1) for line in lines if (match := re.fullmatch(r"(\w+)=\$!", line))]
        waits = [line for line in lines if re.match(r"wait\b", line)]
        self.assertEqual(len(pids), len(launches))
        self.assertEqual(waits, [f'wait "${pid}" || rc=1' for pid in pids])
        self.assertEqual(lines[-1], 'exit "$rc"')
        with tempfile.TemporaryDirectory(prefix="host-python-lanes-") as directory:
            root = Path(directory)

            def run_step(label, failing=""):
                work = root / label
                result, records = self.run_with_host_python_fakes(run, work, failing=failing)
                self.assertEqual(result.stdout.count("::group::"), len(launches), result.stdout)
                for entry in records:  # also the output of the background lanes
                    self.assertIn("ran: " + shlex.join(entry["argv"]), result.stdout)
                # A printed lane log is removed, so the leftover-log step shows only unprinted ones.
                self.assertEqual(list((work / "temp").iterdir()), [], "printed lane logs stay behind")
                return work, result, [entry["argv"] for entry in records], records

            work, result, ran, records = run_step("success")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue((work / "build/coverage-raw/python").is_dir())
            self.assertEqual(sorted(ran), sorted(shlex.split(command) for command in HOST_PYTHON_COMMANDS[1:]))
            self.assertEqual(ran[0], shlex.split(HOST_PYTHON_COMMANDS[1]))
            serial = (shlex.split(HOST_PYTHON_COMMANDS[1]), shlex.split(HOST_PYTHON_FOREGROUND))
            self.assertEqual(sorted(entry["argv"] for entry in records if entry["argv"] in serial), sorted(serial))
            for entry in records:
                if entry["argv"] in serial:
                    self.assertEqual(entry["ignored"], [], entry["argv"])
            # Lane commands are not pinned: inside a `( ... ) &` subshell, bash 5.2 (ubuntu-24.04) runs
            # them with SIGINT/SIGQUIT restored and bash 5.3 keeps them ignored. The probe itself is
            # shown to see ignored signals, so the serial [] above is not vacuous.
            _, probe = self.run_with_host_python_fakes("trap '' INT QUIT\npython3 probe\n", root / "probe")
            self.assertEqual(probe, [{"argv": ["python3", "probe"], "ignored": ["SIGINT", "SIGQUIT"]}])
            for index, command in enumerate(HOST_PYTHON_COMMANDS[2:]):  # no subTest: callers expect a raise
                _, result, ran, _ = run_step(f"fail-{index}", command)
                self.assertNotEqual(result.returncode, 0, command)
                self.assertIn(shlex.split(command), ran)

    def test_host_python_suites_run_in_waited_lanes_with_tools_tests_in_front(self):
        """#494: concurrent lanes keep every command; the step fails when any lane fails."""
        step = self.host_python_step()
        for key in ("if", "continue-on-error", "shell"):
            self.assertNotIn(key, step)
        self.assertEqual(step["env"], {"PYTHONDONTWRITEBYTECODE": "1"})
        self.assert_host_python_lanes(step["run"])

    def test_host_python_lane_contract_rejects_weakened_steps(self):
        """The lane pin fails for a dropped command, a backgrounded tools/tests run, a bare wait, a
        tools/tests failure that skips the lane waits and logs, a kept printed log or a lane that continues."""
        run = self.host_python_step()["run"]
        foreground = HOST_PYTHON_FOREGROUND + " || rc=1\n"
        mutations = {
            "dropped command": run.replace("  python3 .agents/skills/os-dev/scripts/validate_skill.py\n", "", 1),
            "background tools/tests": run.replace(foreground, "", 1).replace(
                "  set -e\n", f"  set -e\n  {foreground}", 1),
            "bare wait": run.replace('wait "$lane_b" || rc=1\nwait "$lane_c" || rc=1\n', "wait\n", 1),
            "tools/tests failure skips lane waits and logs": run.replace(
                foreground, HOST_PYTHON_FOREGROUND + "\n", 1),
            "printed lane log kept": run.replace(' && rm -f "$RUNNER_TEMP/python-lane-$lane.log"', "", 1),
            "lane continues after a failure": run.replace("  set -e\n", "  set +e\n", 1),
        }
        for label, mutated in mutations.items():
            with self.subTest(mutation=label):
                self.assertNotEqual(mutated, run)
                with self.assertRaises(AssertionError):
                    self.assert_host_python_lanes(mutated)

    def test_host_python_lane_logs_left_by_an_interrupted_run_are_printed(self):
        """#494: after a step timeout or cancellation an always() step prints only the unprinted lane logs."""
        job = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]
        step = self.host_python_step()
        leftover = job["steps"][job["steps"].index(step) + 1]
        self.assertEqual(leftover["name"], "Python lane logs left by an interrupted run")
        self.assertEqual(leftover["if"], "${{ always() }}")  # a step condition; the job has none
        self.assertNotIn("if", job)
        # A hung lane hits the step timeout first, so the job still reaches the leftover step;
        # 15 is over twice the slowest measured serial step (363 s).
        self.assertEqual(step["timeout-minutes"], "15")
        self.assertLess(int(step["timeout-minutes"]), int(job["timeout-minutes"]))
        shell = ("bash", "--noprofile", "--norc", "-eo", "pipefail")  # `shell: bash` on a runner
        self.assertEqual(leftover["shell"], "bash")
        interrupted = step["run"][:step["run"].index("for lane in b c")]  # stopped before printing
        with tempfile.TemporaryDirectory(prefix="host-python-leftover-") as directory:
            root = Path(directory)
            for label, script in (("interrupted", interrupted), ("complete", step["run"])):
                with self.subTest(run=label):
                    work = root / label
                    result, records = self.run_with_host_python_fakes(script, work)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    (work / "temp/unrelated.log").write_text("not a lane log\n", encoding="utf-8")
                    printed, _ = self.run_with_host_python_fakes(leftover["run"], work, shell=shell)
                    self.assertEqual(printed.returncode, 0, printed.stderr)
                    self.assertNotIn("not a lane log", printed.stdout)
                    if label == "complete":
                        self.assertEqual(printed.stdout, "")
                        continue
                    self.assertEqual(printed.stdout.count("::group::"), 2, printed.stdout)
                    serial = (shlex.split(HOST_PYTHON_COMMANDS[1]), shlex.split(HOST_PYTHON_FOREGROUND))
                    lanes = [entry["argv"] for entry in records if entry["argv"] not in serial]
                    self.assertEqual(len(lanes), len(HOST_PYTHON_COMMANDS) - 3)
                    for argv in lanes:
                        self.assertIn("ran: " + shlex.join(argv), printed.stdout)
                    self.assertNotIn("ran: " + shlex.join(serial[1]), printed.stdout)

    def test_actionlint_is_pinned_in_required_host_gate(self):
        """Workflow validation is required, checksum-pinned and narrowly scoped."""
        workflow = self.workflows[".github/workflows/ci.yml"]
        steps = workflow["jobs"]["host-quality"]["steps"]
        matches = [
            (index, step)
            for index, step in enumerate(steps)
            if step.get("name") == f"Validate workflows with actionlint {ACTIONLINT_VERSION}"
        ]
        self.assertEqual(len(matches), 1)
        index, step = matches[0]
        checkout = next(
            position
            for position, candidate in enumerate(steps)
            if candidate.get("name") == "Checkout"
        )
        self.assertEqual(index, checkout + 1)
        self.assertNotIn("if", step)
        self.assertEqual(step.get("continue-on-error", "false"), "false")
        self.assertEqual(step.get("shell"), "bash")
        self.assertEqual(
            step.get("env"),
            {
                "ACTIONLINT_VERSION": ACTIONLINT_VERSION,
                "ACTIONLINT_SHA256": ACTIONLINT_SHA256,
            },
        )
        command = step["run"]
        self.assertIn(
            "https://github.com/rhysd/actionlint/releases/download/"
            "v${ACTIONLINT_VERSION}/actionlint_${ACTIONLINT_VERSION}_linux_amd64.tar.gz",
            command,
        )
        self.assertNotIn("|| true", command)
        checksum = command.index("sha256sum --check --strict")
        extraction = command.index("tar --extract")
        self.assertLess(checksum, extraction)
        self.assertIn('${ACTIONLINT_SHA256}', command[:checksum])
        self.assertIn('${actionlint_archive}', command[:checksum])
        self.assertIn('${actionlint_archive}', command[extraction:])
        self.assertRegex(
            command,
            r'tar[^\n]*--extract[^\n]*\\\n[^\n]*--directory "\$\{actionlint_dir\}" actionlint',
        )
        self.assertIn(
            '"${actionlint_dir}/actionlint" -shellcheck= -pyflakes=',
            [line.strip() for line in command.splitlines()],
        )

        policy = (ROOT / "docs/CI-EVIDENCE.md").read_text(encoding="utf-8")
        section_match = re.search(
            r"^## Workflow syntax validation\n(?P<section>.*?)(?=^## )",
            policy,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(section_match)
        section = section_match.group("section")
        for fact in (
            f"actionlint v{ACTIONLINT_VERSION}",
            ACTIONLINT_SHA256,
            ACTIONLINT_TAG_COMMIT,
            "unsigned",
        ):
            self.assertIn(fact, section)

    def test_actionlint_step_fails_closed(self):
        """The verified archive and linter result are both mandatory (CI-HOSTED)."""
        workflow = self.workflows[".github/workflows/ci.yml"]
        expected_name = f"Validate workflows with actionlint {ACTIONLINT_VERSION}"
        step = next(
            candidate
            for candidate in workflow["jobs"]["host-quality"]["steps"]
            if candidate.get("name") == expected_name
        )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive_path = root / "actionlint-fixture.tar.gz"
            actionlint = b'#!/usr/bin/env bash\nexit "${FAKE_ACTIONLINT_EXIT:-0}"\n'
            with tarfile.open(archive_path, "w:gz") as archive:
                entry = tarfile.TarInfo("actionlint")
                entry.mode = 0o755
                entry.size = len(actionlint)
                archive.addfile(entry, io.BytesIO(actionlint))

            fake_bin = root / "bin"
            fake_bin.mkdir()
            fake_curl = fake_bin / "curl"
            fake_curl.write_text(
                """#!/usr/bin/env bash
set -euo pipefail
output=''
while (( $# )); do
  case "$1" in
    --output) output="$2"; shift 2 ;;
    *) shift ;;
  esac
done
test -n "$output"
cp -- "$ACTIONLINT_FIXTURE" "$output"
""",
                encoding="utf-8",
            )
            fake_curl.chmod(0o755)
            expected_hash = hashlib.sha256(archive_path.read_bytes()).hexdigest()

            def run(label, checksum, actionlint_exit):
                runner_temp = root / label
                runner_temp.mkdir()
                environment = os.environ.copy()
                environment.update(
                    {
                        "PATH": f"{fake_bin}{os.pathsep}{environment['PATH']}",
                        "RUNNER_TEMP": str(runner_temp),
                        "ACTIONLINT_VERSION": ACTIONLINT_VERSION,
                        "ACTIONLINT_SHA256": checksum,
                        "ACTIONLINT_FIXTURE": str(archive_path),
                        "FAKE_ACTIONLINT_EXIT": str(actionlint_exit),
                    }
                )
                return subprocess.run(
                    ["bash", "-c", step["run"]],
                    cwd=ROOT,
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                )

            self.assertEqual(run("success", expected_hash, 0).returncode, 0)

            checksum_failure = run("bad-checksum", "0" * 64, 0)
            self.assertNotEqual(checksum_failure.returncode, 0)
            self.assertFalse(
                (root / "bad-checksum" / f"actionlint-{ACTIONLINT_VERSION}").exists()
            )

            actionlint_failure = run("bad-actionlint", expected_hash, 23)
            self.assertEqual(actionlint_failure.returncode, 23)

    def test_host_build_cache_is_saved_only_by_a_successful_push_to_main(self):
        """#494 B2: PR and merge-queue saves are unreadable by other PRs and only evict main's entries."""
        steps = self.workflows[".github/workflows/ci.yml"]["jobs"]["host-quality"]["steps"]
        self.assertFalse([step for step in steps if step.get("uses", "").startswith("actions/cache@")
                          and "host-rust" in json.dumps(step)])
        restore, = [step for step in steps if step.get("id") == "host-build-cache"]
        self.assertTrue(restore["uses"].startswith("actions/cache/restore@"))
        self.assertNotIn("if", restore)
        save, = [step for step in steps if step.get("uses", "").startswith("actions/cache/save@")]
        self.assertEqual(save["if"], "${{ github.event_name == 'push' && github.ref == 'refs/heads/main' "
                                     "&& steps.host-build-cache.outputs.cache-hit != 'true' }}")
        self.assertNotIn("always()", save["if"])
        self.assertEqual(save["with"]["path"], restore["with"]["path"])
        self.assertEqual(save["with"]["key"], "${{ steps.host-build-cache.outputs.cache-primary-key }}")
        self.assertIn("${{ github.sha }}", restore["with"]["key"])
        self.assertLess(max(index for index, step in enumerate(steps) if "run" in step
                            and "cargo" in step["run"]), steps.index(save))
        diagnostics, = [step for step in steps if step.get("name") == "Cache diagnostics (not test evidence)"]
        self.assertEqual(diagnostics["if"], "${{ always() }}")
        self.assertEqual(diagnostics["env"]["HOST_BUILD_HIT"], "${{ steps.host-build-cache.outputs.cache-hit }}")
        # #494 B1: a later merge must not cancel main's running push run, or the save never happens.
        self.assertEqual(self.workflows[".github/workflows/ci.yml"]["concurrency"]["cancel-in-progress"],
                         "${{ github.event_name == 'pull_request' }}")

    def test_native_jobs_leave_shared_format_and_clippy_to_host_quality(self):
        """#494 B5: host-quality runs on every pull request, merge group and main push with the same
        toolchain, so native jobs do not repeat its root format check or clippy of these packages."""
        ci = self.workflows[".github/workflows/ci.yml"]
        for event in ("pull_request", "merge_group"):
            self.assertFalse({"paths", "paths-ignore", "branches", "branches-ignore"} & set(ci["on"][event]), event)
        self.assertFalse({"paths", "paths-ignore"} & set(ci["on"]["push"]))
        self.assertNotIn("if", ci["jobs"]["host-quality"])
        covering = [step for step in ci["jobs"]["host-quality"]["steps"] if step.get("name") in
                    ("Root Rust host tests", "SDK tests and lint", "User allocator and RTC core")]
        self.assertEqual(len(covering), 3)
        for step in covering:
            self.assertFalse({"if", "continue-on-error"} & set(step), step["name"])
        host = "\n".join(step["run"] for step in covering)
        clippy, = [line for line in host.splitlines() if "cargo +1.100.0 clippy --locked -p boot-protocol " in line]
        self.assertTrue(clippy.rstrip().endswith(" --all-targets -- -D warnings"))
        for package in ("boot-protocol", "kernel-core", "pci-core", "keyring-core"):
            self.assertIn(f" -p {package} ", clippy)
        for covered in ("cargo +1.100.0 fmt --all -- --check",
                        "cargo +1.100.0 fmt --manifest-path sdk/Cargo.toml --all -- --check",
                        "cargo +1.100.0 clippy --locked --manifest-path drivers/rtc/Cargo.toml "
                        "--all-targets -- -D warnings"):
            self.assertIn(covered, host)
        for name in ("native-foundation", "native-hw-inventory", "native-power", "native-rtc",
                     "native-smp", "native-xhci"):
            text = (ROOT / ".github/workflows" / f"{name}.yml").read_text(encoding="utf-8")
            self.assertIsNone(re.search(r"fmt --all -- --check|fmt --manifest-path sdk/Cargo\.toml|clippy [^\n]*"
                                        r"(-p (boot-protocol|kernel-core|pci-core|keyring-core)\b|drivers/rtc/Cargo\.toml)",
                                        text), name)

    def test_cache_action_matches_documented_provenance(self):
        """Every cache use must match its reviewed pin, version and policy source."""
        references = [
            (path, job_id, step["uses"])
            for path, job_id, step in self.steps()
            if re.fullmatch(
                r"actions/cache(?:/(?:restore|save))?@[0-9a-f]{40}",
                step.get("uses", ""),
            )
        ]
        self.assertTrue(references, "no actions/cache use found")
        for path, job_id, reference in references:
            with self.subTest(path=path, job=job_id):
                self.assertEqual(reference.rsplit("@", 1)[1], CACHE_ACTION_SHA)

        labelled_reference = re.compile(
            rf"^\s*uses:\s*actions/cache(?:/(?:restore|save))?@"
            rf"{re.escape(CACHE_ACTION_SHA)}\s+"
            rf"#\s*{re.escape(CACHE_ACTION_VERSION)}\s*$",
            re.MULTILINE,
        )
        labelled_uses = sum(
            len(labelled_reference.findall((ROOT / path).read_text(encoding="utf-8")))
            for path in self.workflows
        )
        self.assertEqual(labelled_uses, len(references))

        policy = (ROOT / "CODEX-WORKFLOW.md").read_text(encoding="utf-8")
        self.assertIn(f"reviewed {CACHE_ACTION_VERSION} commit", policy)
        self.assertIn(
            f"*actions/cache*, version {CACHE_ACTION_VERSION.removeprefix('v')}.",
            policy,
        )
        self.assertIn(f"`{CACHE_ACTION_SHA}`", policy)
        self.assertIn(
            f"https://github.com/actions/cache/tree/{CACHE_ACTION_SHA}",
            policy,
        )

    def test_checkout_does_not_persist_credentials(self):
        """Builds and public PR scripts do not inherit checkout credentials."""
        for path, job_id, step in self.steps():
            if step.get("uses", "").startswith("actions/checkout@"):
                with self.subTest(path=path, job=job_id):
                    self.assertEqual(step.get("with", {}).get("persist-credentials"), "false")

    def test_required_gates_are_unfiltered_and_independent(self):
        """Required check names always exist and run without a serial dependency."""
        workflow = self.workflows[".github/workflows/ci.yml"]
        for event in ("pull_request", "merge_group", "push"):
            trigger = workflow["on"][event]
            with self.subTest(event=event):
                self.assertNotIn("paths", trigger)
                self.assertNotIn("paths-ignore", trigger)
        self.assertEqual(workflow["on"]["merge_group"], {"types": ["checks_requested"]})
        self.assertIn("main", workflow["on"]["push"]["branches"])
        # Merge groups have unique refs, so concurrent groups never cancel each other.
        self.assertIn("${{ github.ref }}", workflow["concurrency"]["group"])
        for job_id in ("upload-coverage-rust", "upload-coverage-python",
                       "upload-coverage-javascript"):
            with self.subTest(job=job_id):
                self.assertIn("github.event_name != 'merge_group'", workflow["jobs"][job_id]["if"])
                self.assertEqual(workflow["jobs"][job_id]["timeout-minutes"], "10")
        for job_id in ("host-quality", "qemu-smoke"):
            job = workflow["jobs"][job_id]
            with self.subTest(job=job_id):
                self.assertEqual(job.get("name", job_id), job_id)
                self.assertNotIn("if", job)
                self.assertFalse(job.get("needs"))
                self.assertEqual(job.get("continue-on-error", "false"), "false")
                self.assertTrue(any("run" in step for step in job["steps"]))

    def test_every_automatic_gate_workflow_reports_on_the_queue_commit(self):
        """A merge-queue commit starts every automatic gate unfiltered (#494 D1 step 1)."""
        catalogue = tomllib.loads((ROOT / ".github/ci-impact.toml").read_text(encoding="utf-8"))
        gated = [workflow["path"] for workflow in catalogue["workflow"]
                 if any(job["class"] in ("core", "rwa") and job["activation"] == "automatic"
                        for job in workflow.get("job", []))]
        # #494: exact, so adding or removing a gated workflow is a deliberate edit; no other
        # workflow runs on the queue commit.
        self.assertEqual(len(gated), 25)
        self.assertEqual({path for path, workflow in self.workflows.items()
                          if isinstance(workflow.get("on"), dict) and "merge_group" in workflow["on"]},
                         set(gated))
        for path in gated:
            with self.subTest(path=path):
                # Exactly this mapping: no paths, paths-ignore or branches filter.
                self.assertEqual(self.workflows[path]["on"]["merge_group"],
                                 {"types": ["checks_requested"]})
                # A queue ref is refs/heads/gh-readonly-queue/..., so its group never cancels a
                # pull_request or main run. The fallback form resolves to github.ref on a queue commit.
                group = self.workflows[path]["concurrency"]["group"]
                self.assertTrue("${{ github.ref }}" in group or (
                    "${{ github.event_name }}" in group
                    and "${{ github.event.pull_request.number || github.ref }}" in group), group)

    def test_select_steps_plan_a_queue_commit_from_its_own_head_and_base(self):
        """#494: every select-ci step and BASE_REF use ci.yml's exact pull_request, merge_group and push values."""
        head = "${{ github.event.pull_request.head.sha || github.event.merge_group.head_sha || github.sha }}"
        base = "${{ github.event.merge_group.base_sha || (github.event_name == 'push' && github.event.before) || '' }}"
        ref = "${{ github.event.pull_request.base.ref || github.event.merge_group.base_ref || github.ref_name }}"
        steps = [(path, job_id, step) for path, job_id, step in self.steps() if path in self.workflows]
        selects = [(path, job_id, step) for path, job_id, step in steps
                   if step.get("uses") == "./.github/actions/select-ci"]
        base_refs = [(path, job_id, step["env"]["BASE_REF"]) for path, job_id, step in steps
                     if "BASE_REF" in step.get("env", {})]
        # ci.yml has two select steps (enforced and candidate); the three select jobs one each.
        self.assertEqual((len(selects), len(base_refs)), (5, 3))
        for path, job_id, step in selects:
            with self.subTest(path=path, job=job_id, step=step.get("name")):
                self.assertEqual((step["with"]["head-sha"], step["with"]["base-sha"]), (head, base))
                # ci.yml passes no base-ref; where one is passed it is exactly this.
                self.assertEqual(step["with"].get("base-ref", ref), ref)
        for path, job_id, value in base_refs:
            with self.subTest(path=path, job=job_id):
                self.assertEqual(value, ref)

    def test_waited_queue_check_names_are_unique_on_the_queue_commit(self):
        """merge-hold keys queue-commit check runs by name, so each waited name has one producer."""
        names = [job.get("name", job_id) for workflow in self.workflows.values()
                 if isinstance(workflow.get("on"), dict) and "merge_group" in workflow["on"]
                 for job_id, job in workflow["jobs"].items()]
        for name in ("host-quality", "qemu-smoke", "merge-hold"):
            with self.subTest(name=name):
                self.assertEqual(names.count(name), 1)

    def test_paths_ignore_skips_only_metadata_or_tooling_metadata_with_a_main_backstop(self):
        """Metadata-only changes must not start native matrices; code still must.

        #494 phase 5 C2 (N12): a pull_request filter is METADATA or, for exactly the
        backstopped native workflows, NONNATIVE; a push filter is always METADATA."""
        filtered, nonnative = 0, set()
        for path, workflow in self.workflows.items():
            for event in ("pull_request", "push"):
                trigger = workflow["on"].get(event)
                if isinstance(trigger, dict) and "paths-ignore" in trigger:
                    filtered += 1
                    with self.subTest(path=path, event=event):
                        if event == "pull_request" and trigger["paths-ignore"] == NONNATIVE_PATHS_IGNORE:
                            nonnative.add(Path(path).stem)
                            push = workflow["on"]["push"]
                            self.assertEqual(push["paths-ignore"], METADATA_PATHS_IGNORE)
                            self.assertIn("main", push["branches"])
                        else:
                            self.assertEqual(trigger["paths-ignore"], METADATA_PATHS_IGNORE)
        self.assertGreater(filtered, 0)
        self.assertEqual(nonnative, NONNATIVE_WORKFLOWS)

        # Every native workflow keeps a metadata-only filter, except the two documented
        # allowlists without a push backstop (#494 C1; their coverage of the native input
        # set is pinned in tools/tests/test_ci_impact.py).
        catalogue = tomllib.loads((ROOT / ".github/ci-impact.toml").read_text(encoding="utf-8"))
        allowlists = set()
        for entry in catalogue["workflow"]:
            if not any("native" in job.get("input_sets", []) for job in entry["job"]):
                continue
            trigger = self.workflows[entry["path"]]["on"]["pull_request"]
            with self.subTest(native=entry["path"]):
                if "paths" in trigger:
                    allowlists.add(Path(entry["path"]).stem)
                    self.assertNotIn("push", self.workflows[entry["path"]]["on"])
                else:
                    self.assertIn(trigger["paths-ignore"], (METADATA_PATHS_IGNORE, NONNATIVE_PATHS_IGNORE))
        self.assertEqual(allowlists, {"storage-resilience", "volume-acceptance"})

        def ignored(changed, patterns):
            return any(github_path_filter_matches(p, changed) for p in patterns)

        for changed in ("README.md", "docs/PROJECT-STATE.md", "sdk/README.md",
                        ".github/pull_request_template.md",
                        ".github/ISSUE_TEMPLATE/bug.yml",
                        "tools/project_classification.py"):
            with self.subTest(ignored=changed):
                self.assertTrue(ignored(changed, METADATA_PATHS_IGNORE))
                self.assertTrue(ignored(changed, NONNATIVE_PATHS_IGNORE))
        for changed in (".agents/skills/os-dev/SKILL.json", "plugins/token-guard/x.py", "LICENSE",
                        ".coderabbit.yaml", ".codex/config.toml"):
            with self.subTest(tooling=changed):
                self.assertFalse(ignored(changed, METADATA_PATHS_IGNORE))
                self.assertTrue(ignored(changed, NONNATIVE_PATHS_IGNORE))
        for changed in ("kernel/src/main.rs", "Cargo.lock", "tools/build.py",
                        "tools/ci_evidence.py", "tests/test_ci_workflows.py",
                        "tools/ci_impact.py", "tools/merge_hold.py", "tools/tokenguard.py",
                        ".github/ci-impact.toml", ".github/ci-impact.candidate.toml",
                        ".github/workflows/ci.yml",
                        ".github/workflows/native-smp.yml",
                        ".github/actions/setup-qemu/action.yml",
                        "sdk/examples/window-session/src/main.rs",
                        "docs.rs", "tools/project_classification.py.orig",
                        "plugins.rs", "kernel/plugins/mod.rs", "LICENSE.rs"):
            with self.subTest(built=changed):
                self.assertFalse(ignored(changed, METADATA_PATHS_IGNORE))
                self.assertFalse(ignored(changed, NONNATIVE_PATHS_IGNORE))

    def test_merge_hold_is_an_unfiltered_live_read_required_gate(self):
        """The hold must always report, read live state and never be skipped."""
        path = ".github/workflows/merge-hold.yml"
        workflow = self.workflows[path]
        trigger = workflow["on"]["pull_request"]
        self.assertEqual(set(workflow["on"]), {"pull_request", "merge_group"})
        self.assertEqual(workflow["on"]["merge_group"], {"types": ["checks_requested"]})
        self.assertEqual(
            workflow["concurrency"]["group"],
            "merge-hold-${{ github.event_name }}-"
            "${{ github.event.pull_request.number || github.ref }}",
        )
        self.assertEqual(
            set(trigger["types"]),
            {"opened", "synchronize", "reopened", "labeled", "unlabeled"},
        )
        for key in ("paths", "paths-ignore", "branches", "branches-ignore"):
            self.assertNotIn(key, trigger)
        self.assertNotIn("defaults", workflow)
        self.assertEqual(list(workflow["jobs"]), ["merge-hold"])
        job = workflow["jobs"]["merge-hold"]
        self.assertEqual(job["name"], "merge-hold")
        self.assertEqual(
            job["permissions"],
            {"actions": "read", "checks": "read", "contents": "read", "issues": "read",
             "pull-requests": "read"},
        )
        for key in ("if", "needs", "continue-on-error", "strategy", "defaults",
                    "container", "services"):
            self.assertNotIn(key, job)
        # On merge_group the hold waits for these ci.yml checks, then decides.
        # A rename would make every queue group time out, so pin the names.
        source = (ROOT / "tools/merge_hold.py").read_text(encoding="utf-8")
        waited = ast.literal_eval(re.search(r"^WAIT_CHECKS = (.+)$", source, re.M).group(1))
        minutes = re.search(r"^WAIT_SECONDS = ([0-9]+) \* 60$", source, re.M)
        wait_seconds = int(minutes.group(1)) * 60
        # Phase 3: catalogue budgets are capped by MAX_WAIT_SECONDS, which equals the fixed wait.
        cap = re.search(r"^MAX_WAIT_SECONDS = ([0-9]+) \* 60$", source, re.M)
        self.assertEqual(int(cap.group(1)) * 60, wait_seconds)
        ci = self.workflows[".github/workflows/ci.yml"]
        self.assertEqual(set(waited), {"host-quality", "qemu-smoke"})
        for name in waited:
            with self.subTest(waited=name):
                self.assertEqual(ci["jobs"][name].get("name", name), name)
                self.assertNotIn("if", ci["jobs"][name])
                # The wait budget outlasts each waited job and stays below our own timeout.
                self.assertGreater(wait_seconds, int(ci["jobs"][name]["timeout-minutes"]) * 60)
        self.assertLess(wait_seconds, int(job["timeout-minutes"]) * 60)
        # Entries ahead count as finished once this job's own name completed on them;
        # the walk ahead stops only at one that succeeded (tools/tests/test_merge_hold.py).
        self.assertRegex(source, r'(?m)^AHEAD_CHECKS = WAIT_CHECKS \+ \("merge-hold",\)$')
        self.assertLess(int(job["timeout-minutes"]), 100)  # the planned queue check timeout
        checkout, load, decide = job["steps"]
        self.assertTrue(checkout["uses"].startswith("actions/checkout@"))
        self.assertEqual(checkout["with"], {"persist-credentials": "false", "fetch-depth": "0"})
        for step in (checkout, load, decide):
            for key in ("if", "continue-on-error", "working-directory"):
                self.assertNotIn(key, step)
        # Phase 3 (#494): the decision runs the base commit's copies, never the PR's.
        self.assertNotIn("uses", load)
        self.assertEqual(load["id"], "base")
        self.assertNotIn("${{", load["run"])
        for needle in ('git rev-list --parents -n 1 HEAD', '"$second" != "$HEAD_SHA"',
                       'git cat-file blob "$base:tools/merge_hold.py"',
                       'git cat-file blob "$base:tools/ci_impact.py"', 'set -euo pipefail'):
            self.assertIn(needle.replace('"$second"', '"${second:-}"'), load["run"])
        self.assertEqual(load["env"], {
            "EVENT_NAME": "${{ github.event_name }}",
            "HEAD_SHA": "${{ github.event.pull_request.head.sha }}",
            "MERGE_GROUP_BASE_SHA": "${{ github.event.merge_group.base_sha }}",
        })
        self.assertNotIn("uses", decide)
        self.assertEqual(decide["shell"], "bash")
        self.assertEqual(decide["run"].strip(), 'python3 -B -I "$RUNNER_TEMP/merge-hold-base/merge_hold.py"')
        # The workflow may add variables but must keep every one an older base copy reads.
        self.assertEqual(decide["env"], {
            "GH_TOKEN": "${{ github.token }}",
            "REPO": "${{ github.repository }}",
            "EVENT_NAME": "${{ github.event_name }}",
            "PR_NUMBER": "${{ github.event.pull_request.number }}",
            "HEAD_SHA": "${{ github.event.pull_request.head.sha }}",
            "BASE_SHA": "${{ steps.base.outputs.base-sha }}",
            "TESTED_SHA": "${{ steps.base.outputs.tested-sha }}",
            "MERGE_GROUP_HEAD_SHA": "${{ github.event.merge_group.head_sha }}",
            "MERGE_GROUP_HEAD_REF": "${{ github.event.merge_group.head_ref }}",
            "MERGE_GROUP_BASE_REF": "${{ github.event.merge_group.base_ref }}",
            "MERGE_GROUP_BASE_SHA": "${{ github.event.merge_group.base_sha }}",
        })
        read = set(re.findall(r'os\.environ(?:\.get)?[\[(]"([A-Z_]+)"', source))
        self.assertLessEqual(read - {"GITHUB_RUN_ID", "GITHUB_STEP_SUMMARY"}, set(decide["env"]))

    def test_merge_hold_recheck_reruns_from_the_default_branch_after_every_gate_workflow(self):
        """#494 B9: merge-hold decides at once on pull_request; an event-driven re-check re-runs it."""
        with open(ROOT / ".github/ci-impact.toml", "rb") as handle:
            catalogue = tomllib.load(handle)
        self.assertEqual(catalogue["policy"]["budgets"],
                         {"pull_request_wait_minutes": 0, "merge_group_wait_minutes": 80})
        spec = importlib.util.spec_from_file_location("merge_hold_recheck", ROOT / "tools/merge_hold_recheck.py")
        recheck = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(recheck)
        gates = recheck.gate_workflows(catalogue)
        self.assertIn(".github/workflows/ci.yml", gates)
        self.assertNotIn(".github/workflows/merge-hold.yml", gates)
        path = ".github/workflows/merge-hold-recheck.yml"
        workflow = self.workflows[path]
        # The trigger list is exactly the name of every gate workflow, so each gate run that
        # the script waits for starts another re-check when it completes, plus every other
        # catalogue workflow with a pull_request trigger: merge-hold counts their runs too, so
        # one that ends after the last gate can leave it Pending (#494 B10).
        others = {row["path"] for row in catalogue["workflow"] if "pull_request" in row["events"]}
        triggers = (gates | others) - {".github/workflows/merge-hold.yml"}
        self.assertIn(".github/workflows/windows-qemu-test-version.yml", triggers)
        self.assertNotIn(path, triggers)
        self.assertEqual(list(workflow["on"]), ["workflow_run"])
        trigger = workflow["on"]["workflow_run"]
        self.assertEqual(set(trigger), {"workflows", "types", "branches-ignore"})
        self.assertEqual(trigger["types"], ["in_progress", "completed"])
        self.assertEqual(trigger["branches-ignore"], ["main"])
        self.assertEqual(sorted(trigger["workflows"]), sorted({self.workflows[item]["name"] for item in triggers}))
        self.assertEqual(len(trigger["workflows"]), len(triggers))
        self.assertNotIn(workflow["name"], trigger["workflows"])
        self.assertNotIn(self.workflows[".github/workflows/merge-hold.yml"]["name"], trigger["workflows"])
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        self.assertEqual(workflow["concurrency"], {
            "group": "merge-hold-recheck-${{ github.event.workflow_run.event }}-"
                     "${{ github.event.workflow_run.head_repository.full_name }}-"
                     "${{ github.event.workflow_run.head_sha }}-"
                     "${{ github.event.workflow_run.run_attempt > 1 && "
                     "format('{0}-{1}-{2}', github.event.workflow_run.id, "
                     "github.event.workflow_run.run_attempt, github.event.action) || "
                     "github.event.action }}",
            "cancel-in-progress": "false"})
        self.assertEqual(list(workflow["jobs"]), ["recheck"])
        job = workflow["jobs"]["recheck"]
        self.assertEqual(job["if"], "github.event.workflow_run.event == 'pull_request' && "
                                    "github.event.workflow_run.head_repository.full_name == github.repository && "
                                    "(github.event.action == 'completed' || "
                                    "github.event.workflow_run.run_attempt > 1)")
        self.assertEqual(job["timeout-minutes"], "5")
        # Either lifecycle path, including conflict confirmation, stays below the timeout.
        rerun_budget = (recheck.RERUN_POST_LIMIT * (recheck.RERUN_CONFIRM_POLLS - 1)
                        * recheck.RERUN_CONFIRM_SECONDS)
        completed_budget = recheck.WAIT_POLLS * recheck.POLL_SECONDS + rerun_budget
        cancel_budget = ((recheck.CANCEL_CONFIRM_POLLS
                          + recheck.FORCE_CANCEL_CONFIRM_POLLS - 2)
                         * recheck.RERUN_CONFIRM_SECONDS + rerun_budget)
        wait_budget = max(completed_budget, cancel_budget)
        self.assertLessEqual(wait_budget, 5 * 60)
        self.assertEqual(job["permissions"],
                         {"actions": "write", "contents": "read", "pull-requests": "read"})
        for key in ("needs", "continue-on-error", "strategy", "defaults", "container", "services", "env"):
            self.assertNotIn(key, job)
        checkout, run = job["steps"]
        self.assertTrue(checkout["uses"].startswith("actions/checkout@"))
        # No ref: workflow_run checks out the default branch, never pull request code.
        self.assertEqual(checkout["with"], {
            "persist-credentials": "false",
            "sparse-checkout": "tools/merge_hold_recheck.py\n.github/ci-impact.toml\n",
            "sparse-checkout-cone-mode": "false"})
        self.assertNotIn("uses", run)
        self.assertEqual(run["run"], "python3 -B tools/merge_hold_recheck.py")
        self.assertEqual(run["env"], {"GH_TOKEN": "${{ github.token }}", "REPO": "${{ github.repository }}"})
        for step in (checkout, run):
            for key in ("if", "continue-on-error", "working-directory"):
                self.assertNotIn(key, step)

    def test_only_artifact_uploads_can_tolerate_errors(self):
        """Storage failure may be optional; failed builds, tests and collectors are not."""
        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                self.assertEqual(job.get("continue-on-error", "false"), "false")
        for path, job_id, step in self.steps():
            if step.get("continue-on-error", "false") != "false":
                with self.subTest(path=path, job=job_id, step=step.get("name")):
                    self.assertEqual(step["continue-on-error"], "true")
                    self.assertEqual(path, EVIDENCE_ACTION)
                    self.assertTrue(step.get("uses", "").startswith("actions/upload-artifact@"))

    def test_window_matrix_requires_package_only_for_scale_one(self):
        """Scale two must retain evidence without requiring or colliding on a scale-one bundle."""
        workflow = self.workflows[".github/workflows/native-window-session.yml"]
        job = workflow["jobs"]["normal-and-package"]
        preserve, = (
            step for step in job["steps"]
            if step.get("uses") == "./.github/actions/preserve-evidence"
            and step.get("with", {}).get("package-path")
        )
        inputs = preserve["with"]
        self.assertEqual(
            inputs["package-path"],
            "${{ matrix.scale == 1 && "
            "'build/window-session-ci/evidence/NieuwOS-Venstersessie-Windows-QEMU.zip' || '' }}",
        )
        self.assertIn("${{ matrix.scale }}", inputs["name"])

        package = next(
            step for step in job["steps"]
            if step.get("name") == "Package the reviewable Windows QEMU bundle"
        )
        self.assertIn("matrix.scale == 1", package["if"])

    def test_materiaal_measurement_is_opt_in_without_weaker_native_bounds(self):
        """A selected PR head can run the real gate without enabling it globally."""
        workflow = self.workflows[".github/workflows/materiaal-probe.yml"]
        self.assertEqual(set(workflow["on"]), {"workflow_dispatch", "pull_request"})
        self.assertEqual(workflow["on"]["pull_request"], {
            "branches": ["main"],
            "types": ["opened", "reopened", "synchronize", "labeled", "ready_for_review"],
        })
        self.assertEqual(workflow["on"]["workflow_dispatch"]["inputs"]["full_evidence"]["default"],
                         "false")
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        self.assertEqual(set(workflow["jobs"]), {"materiaal-probe"})
        job = workflow["jobs"]["materiaal-probe"]
        # Pin the guard as configuration; actionlint checks GitHub expression syntax.
        # This test does not emulate GitHub's event delivery or expression engine.
        self.assertEqual(" ".join(job["if"].split()),
                         "${{ github.event_name == 'workflow_dispatch' || "
                         "contains(github.event.pull_request.labels.*.name, 'acceptance-materiaal-probe') }}")
        self.assertEqual(int(job["timeout-minutes"]), 40)
        self.assertNotIn("continue-on-error", job)
        for step in job["steps"]:
            self.assertNotIn("continue-on-error", step)
        native, = [step for step in job["steps"]
                   if step.get("name") == "Actual QMP keys, mouse and exact pixels for the Materiaal chain"]
        self.assertNotIn("if", native)
        self.assertEqual(int(native["timeout-minutes"]), 10)
        self.assertIn("--case normal", native["run"])
        self.assertIn("--timeout 240", native["run"])
        self.assertNotIn("||", native["run"])
        preserve, = [step for step in job["steps"]
                     if step.get("uses") == "./.github/actions/preserve-evidence"]
        self.assertEqual(preserve["with"]["full-evidence"],
                         "${{ github.event_name == 'workflow_dispatch' && inputs.full_evidence }}")

    def test_pr_lifecycle_runs_once_without_skipping_drafts(self):
        """Automatic gates test drafts; explicit measurement has its own lifecycle."""
        for path, workflow in self.workflows.items():
            if "pull_request" not in workflow["on"]:
                continue
            trigger = workflow["on"]["pull_request"] or {}
            types = trigger.get("types", ["opened", "synchronize", "reopened"])
            with self.subTest(path=path):
                self.assertLessEqual({"opened", "synchronize", "reopened"}, set(types))
                if path == ".github/workflows/materiaal-probe.yml":
                    # The opt-in measurement is separately guarded above; ready
                    # is an explicit opportunity to measure a selected final head.
                    self.assertIn("ready_for_review", types)
                else:
                    self.assertNotIn("ready_for_review", types)
                for job in workflow["jobs"].values():
                    self.assertNotIn("draft", job.get("if", ""))

    def test_deduplication_keeps_host_and_scalar_component_coverage(self):
        """Dedicated gates retain existing and new standalone component coverage."""
        # Phase 4 (#494): the leaves matrix comes from the catalogue through the select job.
        catalogue = tomllib.loads((ROOT / ".github/ci-impact.toml").read_text(encoding="utf-8"))
        leaves = next(wf for wf in catalogue["workflow"] if wf["path"] == ".github/workflows/component-leaves.yml")
        matrix = [variant["matrix"] for job in leaves["job"] if job["id"] == "component-quality"
                  for variant in job["variant"]]
        self.assertEqual(len(matrix), 16)
        self.assertTrue({"artifact-store", "materiaal-ui"}.isdisjoint(
            entry["component"] for entry in matrix
        ))
        for filename, job_id, component in [
            ("artifact-store-quality.yml", "artifact-store-core", "artifact-store-core"),
            ("native-display-input.yml", "native-display-input", "ui/native/Cargo.toml"),
            ("native-display-input.yml", "native-display-input", "sdk/examples/settings-volume/Cargo.toml"),
            ("native-display-input.yml", "native-display-input", "sdk/examples/search-volume/Cargo.toml"),
        ]:
            job = self.workflows[f".github/workflows/{filename}"]["jobs"][job_id]
            relevant = [step for step in job["steps"] if component in step.get("run", "")]
            commands = "\n".join(step["run"] for step in relevant)
            with self.subTest(component=component):
                for operation in ("test", "clippy", "fmt"):
                    self.assertRegex(commands, rf"\bcargo\b[^\n]*\b{operation}\b")
                scalar = [step for step in relevant if re.search(
                    r"\bcargo\b[^\n]*\bbuild\b", step["run"],
                ) and "x86_64-unknown-none" in step["run"]]
                self.assertTrue(scalar, "missing freestanding component build")
                for step in scalar:
                    flags = {**job.get("env", {}), **step.get("env", {})}.get("RUSTFLAGS", "")
                    if filename == "artifact-store-quality.yml":
                        args = shlex.split(step["run"])
                        extra = tomllib.loads(args[args.index("--config") + 1])
                        target = "x86_64-unknown-none"
                        base = tomllib.loads((ROOT / ".cargo/config.toml").read_text())
                        flags = " ".join(base["target"][target]["rustflags"] +
                                         extra["target"][target]["rustflags"])
                    self.assertIn("no-redzone=yes", flags)
                    self.assertIn("-sse2", flags)
                    self.assertIn("-avx2", flags)

    def test_settings_and_search_oracles_run_in_display_gate(self):
        """Standalone adapters need independent oracle checks beside Rust coverage."""
        job = self.workflows[".github/workflows/native-display-input.yml"]["jobs"][
            "native-display-input"
        ]
        commands = "\n".join(step.get("run", "") for step in job["steps"])
        for module in ("tests.test_settings_oracle", "tests.test_search_oracle",
                       "tools.tests.test_run_search"):
            with self.subTest(module=module):
                self.assertRegex(commands, rf"python3 -B -m unittest[^\n]*\b{re.escape(module)}\b")

    def test_settings_native_gate_and_search_compile_keep_separate_evidence(self):
        """Settings has a six-boot gate; search is still an S2 target binary."""
        job = self.workflows[".github/workflows/native-display-input.yml"]["jobs"][
            "native-display-input"
        ]
        commands = "\n".join(step.get("run", "") for step in job["steps"])
        self.assertIn("for case in settings settings-recover", commands)
        self.assertIn('tools/build.py --kernel-feature "test-volume-$case"', commands)
        self.assertIn('tools/run_settings.py', commands)
        self.assertIn('--settings-build "$NOS_BUILD_DIR/settings"', commands)
        self.assertIn('--recover-build "$NOS_BUILD_DIR/settings-recover"', commands)
        self.assertIn('"--bin", "search-app"', commands)
        self.assertIn('"--target", "x86_64-unknown-none"', commands)
        self.assertIn('sdk/examples/volume-docs/linker.ld', commands)
        self.assertNotIn('tools/run_search.py', commands)

    def test_readonly_window_profile_has_a_separate_compile_only_step(self):
        """The new cfg path compiles without claiming a service-refusal run."""
        job = self.workflows[".github/workflows/native-window-session.yml"]["jobs"][
            "readonly-compile"
        ]
        step = next(step for step in job["steps"]
                    if "--kernel-feature test-window-session-readonly" in step.get("run", ""))
        self.assertNotIn("if", step)
        self.assertIn("readonly-compile", job["env"]["NOS_BUILD_DIR"])
        self.assertIn("readonly-compile", job["env"]["NOS_TARGET_DIR"])
        self.assertNotIn("run_window_session.py", step["run"])

    def test_artifact_build_preserves_target_rustflags(self):
        """Scalar CI additions must merge with the mandatory target contract."""
        workflow = self.workflows[".github/workflows/artifact-store-quality.yml"]
        job = workflow["jobs"]["artifact-store-core"]
        step = next(s for s in job["steps"] if s.get("name") == "Freestanding library build")
        for scope in (workflow, job, step):
            env = scope.get("env", {})
            for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                        "CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS"):
                self.assertNotIn(key, env)
        args = shlex.split(step["run"])
        self.assertEqual(args[:3], ["cargo", "+1.100.0", "build"])
        self.assertEqual(args.count("--config"), 1)
        additions = tomllib.loads(args[args.index("--config") + 1])
        self.assertEqual(additions, {"target": {"x86_64-unknown-none": {
            "rustflags": ["-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"],
        }}})

    def test_artifact_component_filter_includes_shared_build_inputs(self):
        """The dedicated gate replaces its broad matrix entry for shared changes too."""
        workflow = self.workflows[".github/workflows/artifact-store-quality.yml"]
        for event in ("pull_request", "push"):
            with self.subTest(event=event):
                self.assertLessEqual({
                    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/**",
                    ".github/actions/**", "crates/artifact-store-core/**",
                }, set(workflow["on"][event]["paths"]))


WINDOW_WORKFLOW = ".github/workflows/native-window-session.yml"
# Every report assertion of the native window gates (CI-EVIDENCE, U02-U06),
# keyed by the run content that selects its own step. Parallel legs may move
# steps between jobs but must not drop or exchange an assertion.
WINDOW_STEP_ASSERTS = {
    "--clipboard": {
        'assert report["passed"] is True and report["native_gate"] is True',
        'assert report["clipboard_requested"] is True and report["clipboard_gate"] is True',
        'assert report["workspace_requested"] is True and report["workspace_gate"] is True',
        'assert report["document_tools_gate"] is True',
        'assert report["document_menu_gate"] is True',
        'assert report["pointer_leave_gate"] is True',
        'assert report["app_high_contrast_gate"] is True',
        'assert report["window_states_gate"] is True',
        'assert report["keyboard_window_move_gate"] is True',
        'assert report["notification_panel_gate"] is True',
        'assert report["trusted_lock_gate"] is True',
        'assert report["locked_f7_gate"] is True',
        'assert report["smoke_only"] is False and report["raw_qemu_exit"] == 33',
        'assert report["kernel_feature"] == os.environ["NOS_WINDOW_PROBE_FEATURE"]',
        'assert report["scale"] == int(os.environ["NOS_WINDOW_SCALE"])',
        'assert report["native_scale200_gate"] is (report["scale"] == 2)',
    },
    "library-write.txt": {
        'assert report["passed"] is True and report["library_gate"] is True',
        'assert report["library_cold"] is False and report["native_gate"] is False',
        'assert report["raw_qemu_exit"] == 33 and len(report["phases"]) == 10',
    },
    "--library-cold-parent": {
        'assert report["passed"] is True and report["library_gate"] is True',
        'assert report["library_cold"] is True and report["native_gate"] is False',
        'assert report["raw_qemu_exit"] == 33 and len(report["phases"]) == 4',
        'assert report["volume_sha256"] == report["library_parent"]["volume_sha256"]',
    },
    "--switcher": {
        'assert report["passed"] is True and report["switcher_gate"] is True',
        'assert report["native_gate"] is False and report["scale"] == scale',
        'assert report["raw_qemu_exit"] == 33 and len(report["phases"]) == 18',
        'assert report["source_commit"] == os.environ["GITHUB_SHA"]',
    },
}
# Fail-closed rules, keyed like WINDOW_STEP_ASSERTS: the immutable proof
# locator and the full gate's volume digest record.
WINDOW_STEP_RULES = {
    "Unsafe or oversized native proof path": (
        "path.is_symlink() or path.parent.is_symlink()",
        'not re.fullmatch(r"window-[0-9a-f]{32}", path.parent.name)',
        "not path.resolve().is_relative_to(root)",
        "path.stat().st_size > 4 * 1024 * 1024",
        'report.get("source_commit") != os.environ["GITHUB_SHA"]',
        'Path(report["run_directory"]).resolve() != path.parent.resolve()',
        "if len(matches) > 1:",
        'if missing and os.environ["CI_JOB_STATUS"] == "success":',
    ),
    "--clipboard": ('volume.relative_to(Path.cwd()).as_posix()',),
}


def window_step_problems(jobs):
    """List assertions or rules missing from the single step that owns them."""
    problems = []
    for needle, required in list(WINDOW_STEP_ASSERTS.items()) + list(WINDOW_STEP_RULES.items()):
        found = [step for job in jobs.values() for step in job["steps"] if needle in step.get("run", "")]
        if len(found) != 1:
            problems.append(f"{needle}: {len(found)} owning steps")
            continue
        run = found[0]["run"]
        lines = {line.strip() for line in run.splitlines()}
        for item in required:
            if (item not in lines) if item.startswith("assert ") else (item not in run):
                problems.append(f"{needle}: {item}")
    return problems


WINDOW_PROOF_NAMES = {
    "native-window-scale-1-proof", "native-window-scale-2-proof",
    "native-window-library-write-proof", "native-window-library-cold-proof",
    "native-window-switcher-scale1-proof", "native-window-switcher-scale2-proof",
}
ROMS = {"--bios-rom", "--vga-rom", "--timeout"}
# Harness flag sets per mode (U02-U06 window gates).
WINDOW_INVOCATIONS = {
    frozenset({"--clipboard", "--workspace", "--scale"} | ROMS),
    frozenset({"--library"} | ROMS),
    frozenset({"--library", "--library-cold-parent"} | ROMS),
    frozenset({"--switcher", "--scale"} | ROMS),
}
# leg: (mode, scale, probe feature, primary proof artifact, harness seconds).
WINDOW_LEGS = {
    "native-scale1": ("native", "1", "native-window-session-probe", "native-window-scale-1-proof", 1800),
    "native-scale2": ("native", "2", "native-window-session-hidpi-probe", "native-window-scale-2-proof", 2700),
    "library": ("library", "1", "native-window-session-probe", "native-window-library-write-proof", 2 * 1800),
    "switcher-scale1": ("switcher", "1", "native-window-session-probe", "native-window-switcher-scale1-proof", 1800),
    "switcher-scale2": ("switcher", "2", "native-window-session-hidpi-probe", "native-window-switcher-scale2-proof", 1800),
}
# Setup, locked fetch, probe build and evidence took about two minutes on run
# 45424451804, and fixture generation runs cargo tests (timeout 180 s each) before
# the harness deadline starts; 15 minutes is the smallest accepted allowance.
WINDOW_SETUP_MINUTES = 15


def matrix_combinations(job):
    """Expand a job's static matrix (include-only or list product) into combinations."""
    matrix = job.get("strategy", {}).get("matrix")
    if not matrix:
        return [{}]
    if set(matrix) == {"include"}:
        return [dict(entry) for entry in matrix["include"]]
    combinations = [{}]
    for key, values in matrix.items():
        combinations = [dict(combo, **{key: value}) for combo in combinations for value in values]
    return combinations


def render_matrix(text, combination):
    """Replace plain `${{ matrix.key }}` references with one combination's values."""
    return re.sub(r"\$\{\{ matrix\.(\w+) \}\}", lambda match: combination[match.group(1)], text)


def packaging_gate_problems(jobs):
    """Packaging may start only after every gate passed: full needs, no job-level override."""
    job = jobs["normal-and-package"]
    problems = sorted({"host-checks", "native-window-session", "readonly-compile"} - set(job.get("needs", [])))
    if "if" in job:
        problems.append("job-level if: " + job["if"])
    return problems


FETCH_ACTION = ".github/actions/fetch-locked-deps/action.yml"
FETCH_MANIFESTS = {
    "sdk/Cargo.toml", "sdk/examples/materiaal/Cargo.toml", "sdk/examples/input-service/Cargo.toml",
    "sdk/examples/window-session/Cargo.toml", "sdk/examples/calculator/Cargo.toml",
    "ui/graphics/Cargo.toml", "ui/native/Cargo.toml", "ui/surface-protocol/Cargo.toml",
}


class NativeWindowSessionLayoutTests(unittest.TestCase):
    """CI-EVIDENCE: parallel window legs keep every gate, proof and isolation rule."""

    @classmethod
    def setUpClass(cls):
        """Load the actual workflow once."""
        cls.text = (ROOT / WINDOW_WORKFLOW).read_text(encoding="utf-8")
        cls.workflow = yaml.load(cls.text, Loader=UniqueKeyLoader)
        cls.jobs = cls.workflow["jobs"]

    def runs(self, job=None):
        """Join the run scripts of one job, or of the whole workflow."""
        jobs = [self.jobs[job]] if job else self.jobs.values()
        return "\n".join(step.get("run", "") for item in jobs for step in item["steps"])

    def step(self, job, needle):
        """Return the single step whose run script contains the needle."""
        found = [step for step in self.jobs[job]["steps"] if needle in step.get("run", "")]
        self.assertEqual(len(found), 1, needle)
        return found[0]

    def test_every_report_assertion_and_locator_rule_stays_in_its_step(self):
        self.assertEqual(window_step_problems(self.jobs), [])

    def test_assertion_moved_between_steps_is_rejected(self):
        """Swapping the library write and cold phase counts must fail."""
        ten = 'len(report["phases"]) == 10'
        four = 'len(report["phases"]) == 4'
        self.assertEqual((self.text.count(ten), self.text.count(four)), (1, 1))
        swapped = self.text.replace(ten, "@@TEN@@").replace(four, ten).replace("@@TEN@@", four)
        jobs = yaml.load(swapped, Loader=UniqueKeyLoader)["jobs"]
        problems = window_step_problems(jobs)
        self.assertIn("library-write.txt: assert report[\"raw_qemu_exit\"] == 33 and " + ten, problems)
        self.assertIn("--library-cold-parent: assert report[\"raw_qemu_exit\"] == 33 and " + four, problems)

    def test_harness_flag_sets_are_unchanged(self):
        invocations = set()
        for segment in self.runs().replace("\\\n", " ").split("tools/run_window_session.py")[1:]:
            command = segment.split("2>&1", 1)[0]
            invocations.add(frozenset(token for token in shlex.split(command) if token.startswith("--")))
        self.assertEqual(invocations, WINDOW_INVOCATIONS)

    def test_six_proof_artifacts_keep_unique_names(self):
        names = []
        for job in self.jobs.values():
            for combination in matrix_combinations(job):
                for step in job["steps"]:
                    guard = re.search(r"matrix\.(\w+) == '?([\w-]+)'?", step.get("if", ""))
                    if guard and combination[guard.group(1)] != guard.group(2):
                        continue
                    if step.get("uses") == "./.github/actions/preserve-evidence":
                        names.append(render_matrix(step["with"]["name"], combination))
        self.assertLessEqual(WINDOW_PROOF_NAMES, set(names))
        self.assertEqual(len(names), len(set(names)), "artifact names must be unique per run")

    def test_every_job_checks_tracked_source_integrity(self):
        for job_id, job in self.jobs.items():
            with self.subTest(job=job_id):
                commands = "\n".join(step.get("run", "") for step in job["steps"])
                self.assertIn("git diff --exit-code", commands)
                self.assertIn('test -z "$(git status --porcelain)"', commands)

    def test_layout_is_host_native_readonly_and_packaging(self):
        self.assertEqual(set(self.jobs), {"host-checks", "native-window-session",
                                          "readonly-compile", "normal-and-package"})
        self.assertNotIn("actions/download-artifact", self.text)
        self.assertNotIn("actions/upload-artifact", self.text)

    def test_host_checks_run_once_without_scale_variants(self):
        job = self.jobs["host-checks"]
        self.assertNotIn("strategy", job)
        self.assertNotIn("needs", job)
        self.assertNotIn("NOS_WINDOW", json.dumps(job))
        commands = self.runs("host-checks")
        for required in ("tools/test_window_session.py", "tools.tests.test_package_windows_qemu",
                         "tools/preflight.py --strict", "sdk/examples/window-session/Cargo.toml",
                         "ui/surface-protocol/Cargo.toml", "ui/graphics/Cargo.toml",
                         "for crate in materiaal calculator", "dpkg-query -W qemu-system-x86 ovmf seabios"):
            with self.subTest(required=required):
                self.assertIn(required, commands)
        for operation in ("test", "clippy", "fmt"):
            self.assertRegex(commands, rf"cargo \+1\.100\.0 {operation}\b")
        self.assertNotIn("run_window_session.py", commands)
        self.assertNotIn("tools/build.py", commands)

    def test_native_legs_are_independent_isolated_and_bounded(self):
        job = self.jobs["native-window-session"]
        self.assertNotIn("needs", job)
        self.assertEqual(job["strategy"]["max-parallel"], "5")
        self.assertEqual(job["strategy"]["fail-fast"], "false")
        self.assertEqual(job["timeout-minutes"], "${{ matrix.timeout }}")
        legs = {entry["leg"]: entry for entry in matrix_combinations(job)}
        self.assertEqual(set(legs), set(WINDOW_LEGS))
        directories = {key: set() for key in ("NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR")}
        for leg, (mode, scale, feature, proof, harness) in WINDOW_LEGS.items():
            entry = legs[leg]
            with self.subTest(leg=leg):
                self.assertEqual((entry["mode"], entry["scale"], entry["probe_feature"], entry["proof"]),
                                 (mode, scale, feature, proof))
                self.assertGreaterEqual(int(entry["timeout"]), harness // 60 + WINDOW_SETUP_MINUTES)
                for key, seen in directories.items():
                    seen.add(render_matrix(job["env"][key], entry))
        for key, seen in directories.items():
            self.assertEqual(len(seen), len(WINDOW_LEGS), key)
        self.assertEqual(len(set().union(*directories.values())), 3 * len(WINDOW_LEGS))
        self.assertEqual(job["env"]["NOS_WINDOW_SCALE"], "${{ matrix.scale }}")
        self.assertEqual(job["env"]["NOS_WINDOW_PROBE_FEATURE"], "${{ matrix.probe_feature }}")
        build = self.step("native-window-session", '--kernel-feature "$NOS_WINDOW_PROBE_FEATURE"')
        self.assertNotIn("if", build)
        self.assertIn("actions/cache@", json.dumps(job["steps"]))
        native = self.step("native-window-session", "--clipboard")
        self.assertEqual(native["if"], "${{ matrix.mode == 'native' }}")
        self.assertIn("gate_timeout=1800", native["run"])
        self.assertIn('if [ "$NOS_WINDOW_SCALE" = 2 ]; then gate_timeout=2700; fi', native["run"])
        switcher = self.step("native-window-session", "--switcher")
        self.assertEqual(switcher["if"], "${{ matrix.mode == 'switcher' }}")
        self.assertIn("--timeout 1800", switcher["run"])

    def test_library_cold_reopens_the_parent_written_in_the_same_leg(self):
        steps = self.jobs["native-window-session"]["steps"]
        write = self.step("native-window-session", "library-write.txt")
        cold = self.step("native-window-session", "--library-cold-parent")
        self.assertLess(steps.index(write), steps.index(cold))
        for step in (write, cold):
            self.assertEqual(step["if"], "${{ matrix.mode == 'library' }}")
            self.assertIn("--timeout 1800", step["run"])
            self.assertNotIn("env", step)
        self.assertIn('(output / "library-parent.txt").write_text(report["run_directory"] + "\\n")', write["run"])
        self.assertIn('read -r library_parent < "$CI_EVIDENCE_DIR/library-parent.txt"', cold["run"])
        between = steps[steps.index(write) + 1:steps.index(cold)]
        self.assertEqual(between, [])

    def test_packaging_waits_for_every_gate_and_uses_only_the_normal_build(self):
        job = self.jobs["normal-and-package"]
        self.assertEqual(packaging_gate_problems(self.jobs), [])
        self.assertEqual(job["strategy"]["matrix"]["scale"], ["1", "2"])
        self.assertEqual(job["env"]["NOS_WINDOW_NORMAL_FEATURE"],
                         "${{ matrix.scale == 2 && 'native-window-session-hidpi' || 'native-window-session' }}")
        self.assertTrue(job["env"]["NOS_BUILD_DIR"].endswith("/build/window-session-ci/normal"))
        commands = self.runs("normal-and-package")
        self.assertNotIn("probe", commands)
        self.assertNotIn("run_window_session.py", commands)
        build = self.step("normal-and-package", '--kernel-feature "$NOS_WINDOW_NORMAL_FEATURE"')
        package = self.step("normal-and-package", "tools/package_windows_qemu.py")
        self.assertIn("matrix.scale == 1", package["if"])
        self.assertNotIn("--build-dir", package["run"])
        self.assertLess(job["steps"].index(build), job["steps"].index(package))
        self.assertGreaterEqual(int(job["timeout-minutes"]), WINDOW_SETUP_MINUTES)

    def test_packaging_override_after_failed_gate_is_rejected(self):
        """An `always()` job condition would package after a failed native leg."""
        header = "  normal-and-package:\n"
        self.assertEqual(self.text.count(header), 1)
        mutated = self.text.replace(header, header + "    if: ${{ always() }}\n")
        jobs = yaml.load(mutated, Loader=UniqueKeyLoader)["jobs"]
        self.assertEqual(packaging_gate_problems(jobs), ["job-level if: ${{ always() }}"])

    def test_every_offline_job_fetches_through_the_one_shared_action(self):
        """The locked manifest list exists once; each job fetches before offline work."""
        action = yaml.load((ROOT / FETCH_ACTION).read_text(encoding="utf-8"), Loader=UniqueKeyLoader)
        step, = action["runs"]["steps"]
        self.assertEqual(step["shell"], "bash")
        lines = [line.strip().rstrip("\\").strip() for line in step["run"].splitlines()]
        self.assertEqual(lines[0], "cargo +1.100.0 fetch --locked")
        manifests = lines[lines.index("for manifest in") + 1:lines.index("do")]
        self.assertEqual(len(manifests), len(set(manifests)))
        self.assertEqual(set(manifests), FETCH_MANIFESTS)
        self.assertIn('cargo +1.100.0 fetch --locked --manifest-path "$manifest"', lines)
        self.assertNotIn("fetch --locked", self.text)
        for job_id, job in self.jobs.items():
            with self.subTest(job=job_id):
                steps = job["steps"]
                uses = [item.get("uses") for item in steps]
                self.assertEqual(uses.count("./.github/actions/fetch-locked-deps"), 1)
                fetch = uses.index("./.github/actions/fetch-locked-deps")
                rust = next(index for index, item in enumerate(steps)
                            if "rustup toolchain install 1.100.0" in item.get("run", ""))
                offline = [index for index, item in enumerate(steps)
                           if "--offline" in item.get("run", "")
                           or item.get("env", {}).get("CARGO_NET_OFFLINE") == "true"]
                self.assertTrue(offline)
                self.assertLess(rust, fetch)
                self.assertLess(fetch, min(offline))

    def test_readonly_compile_is_standalone(self):
        job = self.jobs["readonly-compile"]
        self.assertNotIn("needs", job)
        self.assertIn("readonly-compile", job["env"]["NOS_BUILD_DIR"])
        self.assertNotIn("run_window_session.py", self.runs("readonly-compile"))
        self.step("readonly-compile", "--kernel-feature test-window-session-readonly")
        self.assertGreaterEqual(int(job["timeout-minutes"]), WINDOW_SETUP_MINUTES)



class NativeCoreOnlyClosureTests(unittest.TestCase):
    """#494 phase 5 C2 (N13): an @nonnative filter may hide only what native gates never run.

    From every pull-request job of a native workflow, follow an over-approximate execution
    closure over the tracked Python and shell files: imports (a package's imported names
    too), `-m` and `import_module` names, every `<name>.py`/`<name>.sh` token (subprocess,
    runpy, path joins) and named discovery globs, each matched by file name in any
    directory. No reached file may be a core_only path, or name a core_only top-level
    component as a path component, unless the catalogue registers it as an input of that
    job. A tool that reads a configuration file by convention (`git status` reads
    `.gitignore`, `coverage` `.coveragerc`, `c8` `.c8rc.json`) counts as naming that file.
    Limits: a bare `*`/`**` walk is a data read, not an execution edge; a `*.ext` glob
    is resolved below its caller's directory; Rust compile inputs are covered by the
    `include_*!`/`path` scan, not by the closure; other implicit readers are not modelled."""

    WORD = r"(?<![\w.-])({})(?![\w-])"
    IMPORT = re.compile(r"^\s*(?:from\s+([\w.]+)\s+import\s+([\w, ()]+)|import\s+([\w., ]+))", re.M)
    MODULE = re.compile(r"(?:-m\s+|import_module\(\s*[\'\"]|spec_from_file_location\(\s*[\'\"])([\w.]+)")
    FILE = re.compile(r"[\w.-]+\.(?:py|sh)\b")
    GLOB = re.compile(r"(?<![^\s\'\"=])([\w./-]*\*[\w.*/-]*)(?![^\s\'\"])")
    LOCAL_ACTION = re.compile(r"\./(\.github/actions/[\w-]+)")
    IMPLICIT = {".gitignore": re.compile(r"\bgit\W+(?:status|clean|ls-files|check-ignore|add)\b"),
                ".coveragerc": re.compile(r"\bcoverage\W+(?:run|combine|report|xml|html|json)\b"),
                ".c8rc.json": re.compile(r"\bc8\b")}

    @classmethod
    def setUpClass(cls):
        cls.catalogue = tomllib.loads((ROOT / ".github/ci-impact.toml").read_text(encoding="utf-8"))
        cls.core_only = cls.catalogue["policy"]["core_only"]
        components = sorted({pattern.split("/")[0] for pattern in cls.core_only})
        cls.mention = re.compile(cls.WORD.format("|".join(map(re.escape, components))))
        tracked = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-z"], capture_output=True,
                                 check=True).stdout.decode("utf-8").split("\0")
        cls.tracked = [path for path in tracked if path]
        cls.pool = [path for path in cls.tracked if path.endswith((".py", ".sh"))]
        cls.by_name, cls.by_stem = {}, {}
        for path in cls.pool:
            cls.by_name.setdefault(Path(path).name, []).append(path)
            if path.endswith(".py"):
                cls.by_stem.setdefault(Path(path).stem, []).append(path)
        cls.edge_cache = {}

    def modules(self, name):
        return {path for part in name.split(".") for path in self.by_stem.get(part, [])}

    def edges(self, text, here):
        found = set()
        for source, names, plain in self.IMPORT.findall(text):
            wanted = [source] if source else [name.split(" as ")[0].strip() for name in plain.split(",")]
            if source and not self.by_stem.get(source.rsplit(".", 1)[-1]):
                # `from package import module`: the names may be modules.
                wanted += [name.split(" as ")[0].strip()
                           for name in names.replace("(", "").replace(")", "").split(",")]
            for name in wanted:
                found |= self.modules(name)
        for name in self.MODULE.findall(text):
            found |= self.modules(name)
        for name in set(self.FILE.findall(text)):
            found.update(self.by_name.get(name, []))
        for pattern in set(self.GLOB.findall(text)):
            pattern = pattern.rsplit("/", 1)[-1]
            if not re.search(r"\w", pattern):
                continue  # a bare directory walk executes nothing by name
            scope = here if pattern.startswith("*") else ""
            found.update(path for path in self.pool if path.startswith(scope) and (
                fnmatch.fnmatchcase(Path(path).name, pattern) or fnmatch.fnmatchcase(Path(path).stem, pattern)))
        return found

    def mentions(self, text):
        return set(self.mention.findall(text)) | {path for path, reader in self.IMPLICIT.items()
                                                  if reader.search(text)}

    def file_edges(self, path):
        if path not in self.edge_cache:
            text = (ROOT / path).read_text(encoding="utf-8", errors="replace")
            self.edge_cache[path] = (self.edges(text, Path(path).parent.as_posix() + "/"),
                                     self.mentions(text))
        return self.edge_cache[path]

    @staticmethod
    def strings(node):
        if isinstance(node, dict):
            return [text for key, value in node.items() for text in [str(key), *NativeCoreOnlyClosureTests.strings(value)]]
        if isinstance(node, list):
            return [text for value in node for text in NativeCoreOnlyClosureTests.strings(value)]
        return [str(node)]

    def closure(self, workflow, job_id):
        """Return {reached path: core_only components it names} for one job."""
        doc = yaml.load((ROOT / workflow).read_text(encoding="utf-8"), Loader=UniqueKeyLoader)
        text = "\n".join(self.strings(doc["jobs"][job_id]))
        actions, pending = set(), self.LOCAL_ACTION.findall(text)
        while pending:  # composite actions may use further local actions
            action = pending.pop()
            if action not in actions:
                actions.add(action)
                body = (ROOT / action / "action.yml").read_text(encoding="utf-8")
                text += "\n" + body
                pending += self.LOCAL_ACTION.findall(body)
        seeds = self.edges(text, ".github/") | {"tools/build.py"}
        reached = {workflow: self.mentions(text)}
        todo = list(seeds)
        while todo:
            path = todo.pop()
            if path in reached:
                continue
            successors, mentioned = self.file_edges(path)
            reached[path] = mentioned
            todo += successors - set(reached)
        return reached

    def native_jobs(self):
        for entry in self.catalogue["workflow"]:
            nonnative = entry.get("trigger", {}).get("pull_request", {}).get("paths_ignore") == "@nonnative"
            for job in entry["job"]:
                # @nonnative skips the whole workflow, so its host jobs count as well.
                if job["activation"] == "automatic" and ("native" in job.get("input_sets", []) or nonnative):
                    yield entry, job

    def violations(self, reached, inputs):
        found = []
        for path, components in sorted(reached.items()):
            if any(github_path_filter_matches(pattern, path) for pattern in self.core_only) and \
                    not any(github_path_filter_matches(pattern, path) for pattern in inputs):
                found.append((path, "core_only file"))
            for component in sorted(components):
                if component in self.IMPLICIT:  # a convention read matters only for a core_only file
                    hidden = any(github_path_filter_matches(pattern, component) for pattern in self.core_only)
                    covered = any(github_path_filter_matches(pattern, component) for pattern in inputs)
                else:
                    hidden, covered = True, any(pattern.startswith(component + "/") for pattern in inputs)
                if hidden and not covered:
                    found.append((path, component))
        return found

    def test_native_pull_request_jobs_reach_core_only_paths_only_through_registered_inputs(self):
        users, checked = set(), set()
        for entry, job in self.native_jobs():
            reached = self.closure(entry["path"], job["id"])
            stem = Path(entry["path"]).stem
            checked.add(f"{stem}#{job['id']}")
            with self.subTest(job=f"{stem}#{job['id']}"):
                self.assertEqual(self.violations(reached, job.get("inputs", [])), [])
                if self.violations(reached, []):
                    users.add(stem)
                    self.assertNotIn(stem, NONNATIVE_WORKFLOWS)
                # Control: import edges work, so a broken pattern cannot pass vacuously.
                self.assertIn("tools/common.py", reached)
        # Control: the closure finds tools/tokenguard.py -> plugins/** (#494 C1, D-2), and
        # with the registered inputs removed those two jobs would fail.
        self.assertEqual(users, {"native-hid-input", "native-xhci"})
        for stem in users:
            reached = self.closure(f".github/workflows/{stem}.yml", stem)
            self.assertIn("plugins", reached["tools/tokenguard.py"])
        # Control: the host jobs of an @nonnative workflow are covered too.
        self.assertIn("net-broker-quality#net-broker-quality", checked)
        # Control: the convention scan sees the tree-cleanliness check, so .gitignore must stay
        # central; were it core_only, @nonnative would hide an input of these jobs.
        workflow = ".github/workflows/native-foundation.yml"
        self.assertIn(".gitignore", self.closure(workflow, "native-foundation")[workflow])
        self.assertIn(".gitignore", self.catalogue["policy"]["always_full"])
        self.assertNotIn(".gitignore", self.core_only)

    def test_closure_follows_harness_discovery_and_imports(self):
        """The design's under-approximation counterexample (CI-P5-A section 6.3) is reached."""
        reached = self.closure(".github/workflows/signed-updates.yml", "b03-native-update")
        for path in ("tools/test_update.py", "tests/test_update_floor_characterization.py",
                     "tools/characterize_update_floor.py"):
            self.assertIn(path, reached)
        reached = self.closure(".github/workflows/entropy-keys.yml", "b01-native-supervisor")
        self.assertTrue(any(fnmatch.fnmatchcase(path, "tests/test_keyring_*.py") for path in reached))

    def test_no_compiled_source_includes_a_core_only_path(self):
        include = re.compile(r"(?:include_(?:str|bytes)!\s*\(\s*|#\[path\s*=\s*|\bpath\s*=\s*)\"([^\"]*)\"")
        checked = 0
        for path in self.tracked:
            if path.endswith((".rs", "Cargo.toml")):
                checked += 1
                text = (ROOT / path).read_text(encoding="utf-8", errors="replace")
                with self.subTest(path=path):
                    self.assertEqual([value for value in include.findall(text) if self.mention.search(value)], [])
        self.assertGreater(checked, 100)


SHADOW_SELECTOR = ROOT / "tools/ci_impact.py"
SELECT_ACTION = ".github/actions/select-ci/action.yml"
CANDIDATE_INCREMENT = "# Candidate increment:"


def candidate_drift(candidate, catalogue):
    """Why the candidate breaks its declared increment state (#494 C3), or None.

    Its leading comment block declares `# Candidate increment: <none or a name>` once.
    With none, the candidate is that comment header plus a verbatim copy of the catalogue.
    """
    leading = []
    for line in candidate.splitlines():
        if not line.startswith("#"):
            break
        leading.append(line)
    declared = [line[len(CANDIDATE_INCREMENT):].strip() for line in leading
                if line.startswith(CANDIDATE_INCREMENT)]
    if len(declared) != 1 or not declared[0]:
        return f"the header must declare '{CANDIDATE_INCREMENT} <none or a name>' exactly once"
    if declared[0] != "none":
        # A declared increment names its issue, so a look-alike such as "None" cannot switch the check off.
        return None if re.search(r"#[0-9]+", declared[0]) else "a declared increment names its issue (#<number>)"
    header = candidate[:len(candidate) - len(catalogue)]
    if not candidate.endswith(catalogue) or not header.endswith("\n") \
            or any(not line.startswith("#") for line in header.splitlines()):
        return "with no declared increment the candidate must be its header plus a verbatim catalogue copy"
    return None


SHADOW_CATALOGUE = """\
schema_version = 1
revision = 1

[policy]
mode = "shadow"
always_full = [".github/**", "tools/ci_impact.py",
               "rustfmt.toml", "**/rustfmt.toml", ".rustfmt.toml", "**/.rustfmt.toml",
               "clippy.toml", "**/clippy.toml", ".clippy.toml", "**/.clippy.toml",
               "rust-toolchain*", "**/rust-toolchain*", ".cargo/**", "**/.cargo/**"]
inert = ["docs/**", "README.md"]
core_only = []

[policy.events]
pull_request = "select"
merge_group = "full"
push = "full"
schedule = "full"
workflow_dispatch = "manual"

[policy.budgets]
pull_request_wait_minutes = 0
merge_group_wait_minutes = 45

[[workflow]]
path = ".github/workflows/ci.yml"
owner = "ci-owner"
granularity = "job"
events = ["workflow_dispatch", "pull_request", "merge_group", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]

[[workflow.job]]
id = "host-quality"
class = "core"
activation = "automatic"
"""


class CiImpactShadowTests(unittest.TestCase):
    """CI-IMPACT phase 2b: the shadow plan is independent, read-only and base-trusted."""

    @classmethod
    def setUpClass(cls):
        load = lambda path: yaml.load((ROOT / path).read_text(encoding="utf-8"),
                                      Loader=UniqueKeyLoader)
        cls.load = staticmethod(load)
        cls.ci = load(".github/workflows/ci.yml")
        cls.action = load(SELECT_ACTION)

    def test_shadow_job_is_independent_read_only_and_preserves_its_plan(self):
        jobs = self.ci["jobs"]
        job = jobs["ci-impact-shadow"]
        self.assertEqual(job["name"], "ci-impact-shadow")
        self.assertEqual(job["runs-on"], "ubuntu-24.04")
        self.assertEqual(job["timeout-minutes"], "10")
        self.assertEqual(job["permissions"], {"contents": "read"})
        for key in ("needs", "if", "continue-on-error", "strategy"):
            self.assertNotIn(key, job)
        for job_id, other in jobs.items():
            with self.subTest(consumer=job_id):
                needs = other.get("needs", [])
                self.assertNotIn("ci-impact-shadow", [needs] if isinstance(needs, str) else needs)
        steps = job["steps"]
        checkout = steps[0]
        host_checkout = next(step for step in jobs["host-quality"]["steps"]
                             if step.get("uses", "").startswith("actions/checkout@"))
        self.assertEqual(checkout["uses"], host_checkout["uses"])
        self.assertEqual(checkout["with"], {"persist-credentials": "false", "fetch-depth": "0"})
        select, candidate = [step for step in steps if step.get("uses") == "./.github/actions/select-ci"]
        self.assertEqual(set(select["with"]), {"event-name", "tested-sha", "head-sha", "base-sha"})
        self.assertNotIn("pull_request.base.sha", select["with"]["base-sha"])
        # #494 phase 5 C3: the same base selector plans the base copy of the candidate catalogue.
        self.assertEqual(candidate["with"], {**select["with"],
                                             "catalogue": ".github/ci-impact.candidate.toml",
                                             "out": "build/ci-impact/plan-candidate.json"})
        self.assertNotIn("id", candidate)
        self.assertEqual(select["id"], "select")
        for key in ("if", "continue-on-error"):  # a failed candidate step stays a failed job
            self.assertNotIn(key, candidate)
            self.assertNotIn(key, select)
        preserve, = [step for step in steps if step.get("uses") == "./.github/actions/preserve-evidence"]
        self.assertEqual(preserve["if"], "${{ always() }}")
        self.assertEqual(preserve["with"], {
            "name": "ci-impact-plan", "job-status": "${{ job.status }}",
            "if-no-files-found": "error",
            "path": "build/ci-impact/plan.json\nbuild/ci-impact/plan-candidate.json\n"
                    "build/ci-impact/coverage.json\n",
            "required-paths": "build/ci-impact/plan.json"})
        # The shadow job keeps a report-only coverage step; host-quality asserts it (phase 3).
        coverage, = [step for step in steps if "ci_impact.py coverage" in step.get("run", "")]
        self.assertNotIn("${{", coverage["run"])
        self.assertNotIn("--strict", coverage["run"])
        self.assertNotIn("continue-on-error", coverage)
        # It depends on the enforced plan only: a failed advisory candidate step does not skip it.
        self.assertEqual(coverage["if"], "${{ !cancelled() && steps.select.outcome == 'success' }}")
        self.assertLess(steps.index(candidate), steps.index(coverage))
        self.assertEqual(steps[1]["with"], {"host-dependencies": "true"})

    def test_only_the_shadow_job_reads_the_candidate_catalogue(self):
        """#494 phase 5 C3: no selection or enforcement step consumes the candidate; host-quality only validates it."""
        users = {}
        for path in [*(ROOT / ".github").rglob("*.y*ml"), *(ROOT / "tools").glob("*.py")]:
            text = path.read_text(encoding="utf-8")
            for marker in ("ci-impact.candidate.toml", "plan-candidate.json"):
                if marker in text:
                    users.setdefault(marker, set()).add(path.relative_to(ROOT).as_posix())
        self.assertEqual(users, {"ci-impact.candidate.toml": {".github/workflows/ci.yml"},
                                 "plan-candidate.json": {".github/workflows/ci.yml"}})
        for job_id, job in self.ci["jobs"].items():
            if job_id != "ci-impact-shadow":
                with self.subTest(job=job_id):
                    self.assertNotIn("candidate", json.dumps(job))
        with (ROOT / ".github/ci-impact.candidate.toml").open("rb") as handle:
            candidate = tomllib.load(handle)
        self.assertEqual(subprocess.run(
            ["python3", "-B", "-I", str(SHADOW_SELECTOR), "validate", str(ROOT / ".github/ci-impact.candidate.toml")],
            capture_output=True, text=True, check=False).returncode, 0)
        self.assertEqual(candidate["schema_version"], 1)

    def test_the_candidate_copies_the_catalogue_until_an_increment_is_declared(self):
        """#494 C3: the candidate never drifts silently; an intentional increment is declared in its header."""
        catalogue = (ROOT / ".github/ci-impact.toml").read_text(encoding="utf-8")
        candidate = (ROOT / ".github/ci-impact.candidate.toml").read_text(encoding="utf-8")
        self.assertIsNone(candidate_drift(candidate, catalogue))
        # The rule itself, on a synthetic candidate around the real catalogue.
        marker = CANDIDATE_INCREMENT + " none\n"
        copy = "# CI impact CANDIDATE catalogue (fixture)\n" + marker + "#\n" + catalogue
        self.assertIsNone(candidate_drift(copy, catalogue))
        drifted = copy.replace("\nrevision = ", "\nrevision = 1", 1)
        self.assertNotEqual(tomllib.loads(drifted), tomllib.loads(catalogue))
        cases = {"candidate data drift": (drifted, catalogue),
                 "catalogue change not copied": (copy, catalogue.replace("\nrevision = ", "\nrevision = 1", 1)),
                 "data in the header": (copy.replace(marker, marker + "revision = 1\n", 1), catalogue),
                 "comment drift in the copy": (copy + "# extra\n", catalogue),
                 "no declaration": (copy.replace(marker, "", 1), catalogue),
                 "empty declaration": (copy.replace(marker, CANDIDATE_INCREMENT + "\n", 1), catalogue),
                 "two declarations": (copy.replace(marker, marker + marker, 1), catalogue),
                 "look-alike of none": (copy.replace(marker, CANDIDATE_INCREMENT + " None\n", 1), catalogue),
                 "increment without an issue": (drifted.replace(marker, CANDIDATE_INCREMENT + " C4\n", 1),
                                                catalogue)}
        for label, (text, base) in cases.items():
            with self.subTest(case=label):
                self.assertIsNotNone(candidate_drift(text, base))
        # A declared increment may differ from the catalogue (validate still checks its data).
        increment = drifted.replace(marker, CANDIDATE_INCREMENT + " C4 example (#494)\n", 1)
        self.assertIsNone(candidate_drift(increment, catalogue))

    def test_host_quality_fails_on_an_invalid_or_drifted_catalogue(self):
        """Phase 3 (#494): merge-hold enforces plans, so catalogue drift must fail a required gate."""
        steps = self.ci["jobs"]["host-quality"]["steps"]
        names = [step.get("name", step.get("uses", "")) for step in steps]
        gate = steps[names.index("CI impact catalogue is valid and matches the workflows")]
        self.assertLess(names.index("Pinned Python"), names.index(gate["name"]))
        self.assertEqual(steps[names.index("Pinned Python")]["with"], {"host-dependencies": "true"})
        lines = [line.strip() for line in gate["run"].strip().splitlines()]
        self.assertEqual(lines, ["python3 -B tools/ci_impact.py validate .github/ci-impact.toml",
                                 "python3 -B tools/ci_impact.py coverage --strict"])
        for key in ("if", "continue-on-error"):
            self.assertNotIn(key, gate)

    def test_phase4_select_jobs_render_the_base_plan_and_gate_their_workflow(self):
        """Phase 4 (#494): a select job per selected workflow; no gate needs merge-hold."""
        cases = {"component-leaves": ["component-quality"], "app-management-quality": ["app-package-quality"],
                 "net-tls-quality": ["net-tls-quality"]}
        host_checkout = next(step for step in self.ci["jobs"]["host-quality"]["steps"]
                             if step.get("uses", "").startswith("actions/checkout@"))
        for stem, gated in cases.items():
            workflow = self.load(f".github/workflows/{stem}.yml")
            jobs = workflow["jobs"]
            select = jobs["select"]
            with self.subTest(workflow=stem):
                self.assertEqual(select["name"], f"{stem} select")
                self.assertEqual(select["permissions"], {"contents": "read"})
                for key in ("needs", "if", "continue-on-error", "strategy"):
                    self.assertNotIn(key, select)
                checkout, python, plan, render = select["steps"]
                self.assertEqual(checkout["uses"], host_checkout["uses"])
                self.assertEqual(checkout["with"], {"persist-credentials": "false", "fetch-depth": "0"})
                self.assertEqual(plan["uses"], "./.github/actions/select-ci")
                self.assertEqual(set(plan["with"]), {"event-name", "tested-sha", "head-sha", "base-sha", "base-ref"})
                # Only a select-mode plan from select-ci narrows.
                self.assertEqual(render["env"]["PLAN_MODE"], "${{ steps.plan.outputs.mode }}")
                self.assertEqual(render["env"]["BASE_REF"], plan["with"]["base-ref"])
                self.assertIn('--mode="$PLAN_MODE" --base-ref="$BASE_REF"', render["run"])
                self.assertNotIn("${{", render["run"])
                self.assertIn(f"--workflow=.github/workflows/{stem}.yml", render["run"])
                self.assertIn('| tee -a "$GITHUB_OUTPUT"', render["run"])
                for job_id in gated:
                    job = jobs[job_id]
                    self.assertEqual(job["needs"], "select")
                    self.assertEqual(job["if"], f"${{{{ needs.select.outputs.run-{job_id} == 'true' }}}}")
                    self.assertEqual(select["outputs"][f"run-{job_id}"],
                                     f"${{{{ steps.jobs.outputs.run-{job_id} }}}}")
                for job_id, job in jobs.items():
                    needs = job.get("needs", [])
                    self.assertNotIn("merge-hold", [needs] if isinstance(needs, str) else needs)
        leaves = self.load(".github/workflows/component-leaves.yml")["jobs"]["component-quality"]
        self.assertEqual(leaves["strategy"]["matrix"],
                         "${{ fromJSON(needs.select.outputs.matrix-component-quality) }}")
        self.assertEqual(leaves["strategy"]["max-parallel"], "4")

    def test_select_action_passes_expressions_only_through_env(self):
        steps = self.action["runs"]["steps"]
        self.assertEqual(self.action["runs"]["using"], "composite")
        self.assertEqual(set(self.action["inputs"]),
                         {"event-name", "tested-sha", "head-sha", "base-sha", "base-ref", "catalogue", "out"})
        # Defaults keep the enforced catalogue and plan path (#494 phase 5 C3).
        self.assertEqual((self.action["inputs"]["catalogue"]["default"], self.action["inputs"]["out"]["default"]),
                         ("", ""))
        self.assertEqual(set(self.action["outputs"]), {"plan-id", "mode", "plan-path"})
        for step in steps:
            with self.subTest(step=step.get("name")):
                self.assertNotIn("${{", step.get("run", ""))
                self.assertNotIn("continue-on-error", step)
                for value in step.get("env", {}).values():
                    self.assertRegex(value, r"^\$\{\{ inputs\.[a-z-]+ \}\}$")
        run = steps[0]["run"]
        self.assertIn('git cat-file blob "${BASE_SHA}:tools/ci_impact.py"', run)
        self.assertIn('python3 -B -I "$selector" plan', run)
        self.assertIn('--summary "$GITHUB_STEP_SUMMARY"', run)
        self.assertIn("set -euo pipefail", run)

    def run_action(self, repo, env, label):
        """Execute the action's script as a composite step would, against `repo`."""
        temp = Path(repo).parent / label
        temp.mkdir()
        files = {name: temp / name for name in ("summary.md", "output.txt")}
        for path in files.values():
            path.write_text("", encoding="utf-8")
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                           GITHUB_WORKSPACE=str(repo), RUNNER_TEMP=str(temp),
                           GITHUB_STEP_SUMMARY=str(files["summary.md"]),
                           GITHUB_OUTPUT=str(files["output.txt"]), **env)
        result = subprocess.run(["bash", "-c", self.action["runs"]["steps"][0]["run"]], cwd=repo,
                                env=environment, capture_output=True, text=True, check=False)
        outputs = dict(line.split("=", 1) for line in
                       files["output.txt"].read_text(encoding="utf-8").splitlines())
        return result, outputs

    def test_select_action_runs_the_base_selector_and_never_narrows_without_one(self):
        def git(*args):
            return subprocess.run(
                ["git", "-c", "user.name=fixture", "-c", "user.email=fixture@example.invalid",
                 "-c", "commit.gpgsign=false", *args], cwd=repo, check=True, capture_output=True,
                text=True, env={**{k: v for k, v in os.environ.items() if not k.startswith("GIT_")},
                                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull},
            ).stdout.strip()

        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            (repo / "tools").mkdir(parents=True)
            (repo / ".github/workflows").mkdir(parents=True)
            (repo / "docs").mkdir()
            (repo / "README.md").write_text("fixture readme\n", encoding="utf-8")
            (repo / ".github/workflows/ci.yml").write_text("name: ci\n", encoding="utf-8")
            (repo / ".github/ci-impact.toml").write_text(SHADOW_CATALOGUE, encoding="utf-8")
            (repo / "tools/ci_impact.py").write_bytes(SHADOW_SELECTOR.read_bytes())
            git("init", "-q", "-b", "main")
            git("add", "-A")
            git("commit", "-q", "-m", "base")
            base = git("rev-parse", "HEAD")
            git("checkout", "-q", "-b", "topic")
            (repo / "docs/note.md").write_text("note\n", encoding="utf-8")
            git("add", "-A")
            git("commit", "-q", "-m", "head")
            head = git("rev-parse", "HEAD")
            git("checkout", "-q", "main")
            git("merge", "-q", "--no-ff", "-m", "merge", "topic")
            tested = git("rev-parse", "HEAD")
            # A hostile checkout copy must not run when the base has a selector.
            (repo / "tools/ci_impact.py").write_text("raise SystemExit(9)\n", encoding="utf-8")

            result, outputs = self.run_action(repo, {
                "EVENT_NAME": "pull_request", "TESTED_SHA": tested, "HEAD_SHA": head,
                "BASE_SHA": ""}, "pull-request")
            self.assertEqual(result.returncode, 0, result.stderr)
            plan = json.loads((repo / outputs["plan-path"]).read_text(encoding="ascii"))
            self.assertEqual((outputs["mode"], plan["mode"], plan["fallback"]), ("select", "select", None))
            self.assertTrue(plan["selector"]["trusted"])
            self.assertEqual(plan["shas"], {"base": base, "merge_base": base, "head": head,
                                            "tested": tested})
            self.assertEqual(outputs["plan-id"], plan["plan_id"])
            self.assertEqual(plan["ref"], "")  # no base-ref: the phase-2 command line
            # The summary names the catalogue, so the enforced plan is never mistaken for the candidate.
            summary = (repo.parent / "pull-request" / "summary.md").read_text(encoding="utf-8")
            self.assertTrue(summary.startswith("\n# Plan from the enforced catalogue `.github/ci-impact.toml`\n"),
                            summary)
            self.assertIn("\nSelector source for the enforced catalogue `.github/ci-impact.toml`: `base`. "
                          "merge-hold recomputes and enforces its own plan.\n", summary)
            self.assertNotIn("candidate", summary)

            # Phase 4: the target branch reaches the base selector as --base-ref.
            result, outputs = self.run_action(repo, {
                "EVENT_NAME": "pull_request", "TESTED_SHA": tested, "HEAD_SHA": head,
                "BASE_SHA": "", "BASE_REF": "main"}, "pull-request-ref")
            self.assertEqual(result.returncode, 0, result.stderr)
            plan = json.loads((repo / outputs["plan-path"]).read_text(encoding="ascii"))
            self.assertEqual((outputs["mode"], plan["ref"]), ("select", "main"))

            # Without a base, the checkout copy runs and the output is forced to full.
            (repo / "tools/ci_impact.py").write_bytes(SHADOW_SELECTOR.read_bytes())
            result, outputs = self.run_action(repo, {
                "EVENT_NAME": "workflow_dispatch", "TESTED_SHA": tested, "HEAD_SHA": tested,
                "BASE_SHA": ""}, "dispatch")
            self.assertEqual(result.returncode, 0, result.stderr)
            plan = json.loads((repo / outputs["plan-path"]).read_text(encoding="ascii"))
            self.assertEqual((outputs["mode"], plan["fallback"]["code"]), ("full", "no-trusted-base"))
            self.assertFalse(plan["selector"]["trusted"])

            # A hostile checkout copy that claims select is overridden when no base selector ran.
            (repo / "tools/ci_impact.py").write_text(
                "import json, os, sys\n"
                "out = sys.argv[sys.argv.index('--out') + 1]\n"
                "os.makedirs(os.path.dirname(out), exist_ok=True)\n"
                "json.dump({'plan_id': 'sha256:' + '0' * 64, 'mode': 'select'}, open(out, 'w'))\n",
                encoding="utf-8")
            result, outputs = self.run_action(repo, {
                "EVENT_NAME": "workflow_dispatch", "TESTED_SHA": tested, "HEAD_SHA": tested,
                "BASE_SHA": ""}, "hostile-checkout")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(outputs["mode"], "full")
            (repo / "tools/ci_impact.py").write_bytes(SHADOW_SELECTOR.read_bytes())

            # C3: the candidate catalogue is planned from the base commit by the same base selector.
            # Without a base copy the plan widens instead of failing the step.
            candidate_env = {"EVENT_NAME": "pull_request", "TESTED_SHA": tested, "HEAD_SHA": head,
                             "BASE_SHA": "", "CATALOGUE": ".github/ci-impact.candidate.toml",
                             "OUT": "build/ci-impact/plan-candidate.json"}
            result, outputs = self.run_action(repo, candidate_env, "candidate-bootstrap")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(outputs["plan-path"], "build/ci-impact/plan-candidate.json")
            plan = json.loads((repo / outputs["plan-path"]).read_text(encoding="ascii"))
            self.assertEqual((outputs["mode"], plan["mode"], plan["fallback"]["code"],
                              plan["catalogue"]["path"]),
                             ("full", "full", "bootstrap-no-base-catalogue", ".github/ci-impact.candidate.toml"))
            git("checkout", "-q", "main")
            (repo / ".github/ci-impact.candidate.toml").write_text(SHADOW_CATALOGUE, encoding="utf-8")
            git("add", "-A")
            git("commit", "-q", "-m", "candidate")
            git("checkout", "-q", "topic")
            git("merge", "-q", "--no-ff", "-m", "catch up", "main")
            git("checkout", "-q", "main")
            git("merge", "-q", "--no-ff", "-m", "merge", "topic")
            with_candidate = git("rev-parse", "HEAD")
            result, outputs = self.run_action(repo, {**candidate_env, "TESTED_SHA": with_candidate,
                                                     "HEAD_SHA": git("rev-parse", "topic")}, "candidate")
            self.assertEqual(result.returncode, 0, result.stderr)
            plan = json.loads((repo / outputs["plan-path"]).read_text(encoding="ascii"))
            self.assertEqual((outputs["mode"], plan["fallback"]), ("select", None))
            self.assertEqual(plan["catalogue"]["path"], ".github/ci-impact.candidate.toml")
            summary = (repo.parent / "candidate" / "summary.md").read_text(encoding="utf-8")
            self.assertTrue(summary.startswith("\n# Plan from the advisory candidate catalogue "
                                               "`.github/ci-impact.candidate.toml`\n"), summary)
            self.assertIn("\nSelector source for the advisory candidate catalogue "
                          "`.github/ci-impact.candidate.toml`: `base`. Advisory: nothing consumes this plan.\n",
                          summary)
            self.assertNotIn("enforce", summary)
            for index, (key, value) in enumerate((("OUT", "../plan.json"), ("OUT", "build/ci-impact/sub/plan.json"),
                                                  ("CATALOGUE", "--catalogue=x.toml"),
                                                  ("CATALOGUE", "tools/x.toml"))):
                result, _ = self.run_action(repo, {**candidate_env, key: value}, f"reject-{index}")
                self.assertEqual(result.returncode, 2, (key, value))
                self.assertEqual((repo.parent / f"reject-{index}" / "summary.md").read_text(encoding="utf-8"), "")

            # A malformed base is rejected before it reaches git; the plan widens.
            result, outputs = self.run_action(repo, {
                "EVENT_NAME": "push", "TESTED_SHA": tested, "HEAD_SHA": tested,
                "BASE_SHA": "--output=/dev/null"}, "push-bad-base")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(outputs["mode"], "full")


if __name__ == "__main__":
    unittest.main()
