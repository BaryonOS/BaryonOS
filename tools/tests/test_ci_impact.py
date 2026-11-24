"""Unit tests for the CI impact selector; temporary git repositories, no network."""

import contextlib
import copy
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from unittest import TestCase, main, mock, skipUnless

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import ci_impact  # noqa: E402

SELECTOR = TOOLS / "ci_impact.py"
REAL_CATALOGUE = TOOLS.parent / ".github" / "ci-impact.toml"
HAVE_YAML = importlib.util.find_spec("yaml") is not None

CATALOGUE = """\
schema_version = 1
revision = 1

[policy]
mode = "shadow"
always_full = [".github/**", "tools/ci_impact.py", "Cargo.lock", "**/Cargo.lock",
               "rust-toolchain.toml", "tools/build.py",
               "rustfmt.toml", "**/rustfmt.toml", ".rustfmt.toml", "**/.rustfmt.toml",
               "clippy.toml", "**/clippy.toml", ".clippy.toml", "**/.clippy.toml",
               "rust-toolchain*", "**/rust-toolchain*", ".cargo/**", "**/.cargo/**"]
inert = ["docs/**", "**.md"]
core_only = ["LICENSE"]

[policy.events]
pull_request = "select"
merge_group = "full"
push = "full"
schedule = "full"
workflow_dispatch = "manual"

[input_sets]
native = ["kernel/**", "tools/native/**"]

[[crate_escape]]
crate = "apps/calc"
when = "always"
inputs = ["ui/design.rs", "ui/tokens.css"]

[[crate_escape]]
crate = "svc/stack"
when = "root"
inputs = ["probe/contract.rs"]

[[workflow]]
path = ".github/workflows/ci.yml"
owner = "ci-owner"
granularity = "job"
events = ["pull_request", "merge_group", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]

[[workflow.job]]
id = "host-quality"
class = "core"
activation = "automatic"

[[workflow.job]]
id = "upload"
class = "publication"
activation = "automatic"
needs = ["host-quality"]

[[workflow]]
path = ".github/workflows/leaves.yml"
owner = "ci-owner"
granularity = "variant"
events = ["pull_request", "push", "workflow_dispatch"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]
trigger.pull_request = { paths_ignore = "@metadata" }

[[workflow.job]]
id = "quality"
name = "quality (${{ matrix.component }})"
class = "rwa"
activation = "automatic"
variant = [
  { id = "viewer", check_name = "quality (viewer)", cargo = ["apps/viewer/Cargo.toml"] },
  { id = "crop", check_name = "quality (crop)", cargo = ["apps/crop/Cargo.toml"] },
  { id = "sheet", check_name = "quality (sheet)", cargo = ["apps/sheet/Cargo.toml"] },
  { id = "deck", check_name = "quality (deck)", cargo = ["apps/deck/Cargo.toml"] },
  { id = "solo", check_name = "quality (solo)", cargo = ["apps/solo/Cargo.toml"] },
  { id = "config", check_name = "quality (config)", cargo = ["svc/config/Cargo.toml"] },
]

[[workflow]]
path = ".github/workflows/net.yml"
owner = "ci-owner"
granularity = "job"
events = ["pull_request", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]

[[workflow.job]]
id = "broker"
class = "rwa"
activation = "automatic"
cargo = ["svc/stack/Cargo.toml"]

[[workflow.job]]
id = "native"
class = "rwa"
activation = "automatic"
needs = ["broker"]
input_sets = ["native"]

[[workflow]]
path = ".github/workflows/window.yml"
owner = "ci-owner"
granularity = "workflow"
events = ["pull_request", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]

[[workflow.job]]
id = "a"
class = "rwa"
activation = "automatic"
inputs = ["ui/**"]

[[workflow.job]]
id = "b"
class = "rwa"
activation = "automatic"
input_sets = ["native"]

[[workflow.job]]
id = "c"
class = "rwa"
activation = "automatic"
needs = ["a", "b"]
input_sets = ["native"]

[[workflow]]
path = ".github/workflows/storage.yml"
owner = "ci-owner"
granularity = "workflow"
events = ["pull_request"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]
trigger.pull_request = { paths = ["kernel/**"] }

[[workflow.job]]
id = "s"
class = "rwa"
activation = "automatic"
input_sets = ["native"]

[[workflow]]
path = ".github/workflows/misc.yml"
owner = "ci-owner"
granularity = "job"
events = ["pull_request", "push", "merge_group", "schedule", "workflow_dispatch"]

[[workflow.job]]
id = "manual"
class = "manual"
activation = "dispatch"
inputs = ["kernel/**"]

[[workflow.job]]
id = "ops"
class = "operational"
activation = "schedule"

[[workflow.job]]
id = "advice"
class = "advisory"
activation = "automatic"
inputs = ["kernel/**"]

[[workflow.job]]
id = "publish"
class = "publication"
activation = "push-branch:release"
"""

CORE = {"ci#host-quality"}
ALL_RWA = {"leaves#quality/" + v for v in ("viewer", "crop", "sheet", "deck", "solo", "config")} | {
    "net#broker", "net#native", "window#a", "window#b", "window#c", "storage#s"}
NEVER_FULL = {"ci#upload", "misc#manual", "misc#ops", "misc#advice", "misc#publish"}


def manifest(name, deps=(), extra=""):
    lines = ["[package]", f'name = "{name}"', 'version = "0.1.0"', "", "[dependencies]"]
    lines += [f'{dep.rsplit("/", 1)[-1]} = {{ path = "{dep}" }}' for dep in deps]
    return "\n".join(lines) + "\n" + extra


BASE_FILES = {
    "README.md": "fixture readme\n",
    "LICENSE": "fixture licence\n",
    "docs/guide.md": "guide\n",
    "Cargo.lock": "# lock\n",
    "rust-toolchain.toml": "[toolchain]\nchannel = \"1.0.0\"\n",
    "tools/build.py": "# build\n",
    "tools/native/run.py": "# native harness\n",
    "kernel/main.rs": "fn main() {}\n",
    ".github/workflows/ci.yml": "name: ci\n",
    ".github/actions/setup-python/action.yml": "name: setup\n",
    "crates/codec/Cargo.toml": manifest("codec"),
    "crates/codec/src/lib.rs": "// codec\n",
    "crates/gfx/Cargo.toml": manifest("gfx"),
    "crates/gfx/src/lib.rs": "// gfx\n",
    "crates/usb/Cargo.toml": manifest("usb"),
    "crates/usb/src/lib.rs": "// usb\n",
    "crates/input/Cargo.toml": manifest("input", ["../usb"]),
    "crates/input/src/lib.rs": "// input\n",
    "crates/legacy/Cargo.toml": manifest("legacy"),
    "crates/legacy/src/lib.rs": "// legacy\n",
    "apps/viewer/Cargo.toml": manifest("viewer", ["../../crates/codec", "../../crates/input"]),
    "apps/viewer/src/lib.rs": "// viewer\n",
    "apps/crop/Cargo.toml": manifest("crop", ["../../crates/codec"]),
    "apps/crop/src/lib.rs": "// crop\n",
    "apps/calc/Cargo.toml": manifest("calc", ["../../crates/gfx", "../../crates/input"]),
    "apps/calc/src/render.rs": '#[path = "../../../ui/design.rs"]\npub mod design;\n',
    "apps/sheet/Cargo.toml": manifest("sheet", ["../calc"]),
    "apps/sheet/src/lib.rs": "// sheet\n",
    "apps/deck/Cargo.toml": manifest("deck", ["../../crates/gfx", "../../crates/legacy"]),
    "apps/deck/src/lib.rs": "// deck\n",
    "apps/deck/src/extra.rs": "// extra\n",
    "apps/solo/Cargo.toml": manifest("solo", extra='\n[[bin]]\nname = "solo"\npath = "../../shared/main.rs"\n'),
    "apps/solo/Cargo.lock": "# leaf lock\n",
    "apps/solo/src/lib.rs": "// solo\n",
    "apps/solo/src/a.rs": "// moved later\n",
    "shared/main.rs": "fn main() {}\n",
    "svc/stack/Cargo.toml": manifest("stack"),
    "svc/stack/src/lib.rs": "// stack\n",
    "svc/config/Cargo.toml": manifest("config", ["../stack"]),
    "svc/config/src/lib.rs": "// config\n",
    "ui/design.rs": 'pub const T: &str = include_str!("tokens.css");\n',
    "ui/tokens.css": ":root {}\n",
    "probe/contract.rs": "// contract\n",
}

GIT_ENV = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
GIT_ENV.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT="0")


class Repo:
    def __init__(self, root):
        self.root = str(root)
        self.git("init", "-q", "-b", "main")

    def git(self, *args):
        result = subprocess.run(
            ["git", "-c", "user.name=fixture", "-c", "user.email=fixture@example.invalid",
             "-c", "commit.gpgsign=false", "-C", self.root, *args],
            capture_output=True, env=GIT_ENV, check=True)
        return result.stdout.decode().strip()

    def write(self, path, content):
        target = Path(self.root, path)
        target.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            target.write_bytes(content)
        else:
            target.write_text(content, encoding="utf-8")

    def commit(self, message="change", files=None, remove=()):
        for path, content in (files or {}).items():
            self.write(path, content)
        for path in remove:
            self.git("rm", "-q", path)
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")


class FixtureTestCase(TestCase):
    catalogue = CATALOGUE

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.repo = Repo(self._tmp.name)
        files = dict(BASE_FILES)
        files[".github/ci-impact.toml"] = self.catalogue
        files["tools/ci_impact.py"] = SELECTOR.read_bytes()
        self.base = self.repo.commit("base", files)

    def branch(self, files=None, remove=(), message="head"):
        self.repo.git("checkout", "-q", "-B", "topic", self.base)
        return self.repo.commit(message, files, remove)

    def plan(self, head, base=None, event="pull_request", selector=None, **kwargs):
        return ci_impact.make_plan(self.repo.root, event, self.base if base is None else base, head,
                                   selector_bytes=selector or SELECTOR.read_bytes(),
                                   allow_fetch=False, **kwargs)

    def selected(self, plan):
        return {row["id"] for row in plan["selected"]}

    def assertSelect(self, plan, expected):
        self.assertEqual(plan["mode"], "select", plan.get("fallback"))
        self.assertIsNone(plan["fallback"])
        self.assertEqual(self.selected(plan), CORE | set(expected))

    def assertFull(self, plan, code):
        self.assertEqual(plan["mode"], "full")
        self.assertEqual(plan["fallback"]["code"], code, plan["fallback"])


class SelectionTests(FixtureTestCase):
    def test_leaf_change_selects_only_that_variant_and_core(self):
        plan = self.plan(self.branch({"apps/solo/src/lib.rs": "// solo 2\n"}))
        self.assertSelect(plan, {"leaves#quality/solo"})
        self.assertEqual(plan["selected"][1]["reasons"], [["apps/solo/**", "apps/solo/src/lib.rs"]])
        self.assertTrue(plan["selector"]["trusted"])

    def test_shared_dependency_selects_every_dependent(self):
        plan = self.plan(self.branch({"crates/codec/src/lib.rs": "// codec 2\n"}))
        self.assertSelect(plan, {"leaves#quality/viewer", "leaves#quality/crop"})

    def test_transitive_dependency_through_calc(self):
        plan = self.plan(self.branch({"crates/gfx/src/lib.rs": "// gfx 2\n"}))
        self.assertSelect(plan, {"leaves#quality/deck", "leaves#quality/sheet"})

    def test_usb_reaches_input_dependents(self):
        plan = self.plan(self.branch({"crates/usb/src/lib.rs": "// usb 2\n"}))
        self.assertSelect(plan, {"leaves#quality/viewer", "leaves#quality/sheet"})

    def test_central_inputs_force_full(self):
        for path in ("Cargo.lock", "apps/solo/Cargo.lock", "rust-toolchain.toml", "tools/build.py",
                     ".github/actions/setup-python/action.yml", ".github/workflows/ci.yml"):
            with self.subTest(path=path):
                plan = self.plan(self.branch({path: "changed\n"}))
                self.assertFull(plan, "always-full")
                self.assertLessEqual(ALL_RWA | CORE, self.selected(plan))

    def test_unknown_path_forces_full_with_reason(self):
        plan = self.plan(self.branch({"stray/file.txt": "?\n"}))
        self.assertFull(plan, "unknown-path")
        self.assertIn("unknown-path:stray/file.txt", plan["fallback"]["reasons"])

    def test_inert_and_core_only_paths_select_only_core(self):
        plan = self.plan(self.branch({"docs/guide.md": "new\n", "LICENSE": "new\n", "README.md": "x\n"}))
        self.assertSelect(plan, set())

    def test_multi_commit_diff_is_cumulative_from_merge_base(self):
        self.branch({"apps/solo/src/lib.rs": "// tmp\n"})
        self.repo.commit("revert", {"apps/solo/src/lib.rs": BASE_FILES["apps/solo/src/lib.rs"]})
        head = self.repo.commit("crop", {"apps/crop/src/lib.rs": "// crop 2\n"})
        self.repo.git("checkout", "-q", "main")
        base_tip = self.repo.commit("main moves", {"apps/viewer/src/lib.rs": "// viewer on main\n"})
        plan = self.plan(head, base=base_tip)
        self.assertSelect(plan, {"leaves#quality/crop"})
        self.assertEqual(plan["shas"]["merge_base"], self.base)

    def test_rename_selects_old_and_new_owners(self):
        self.repo.git("checkout", "-q", "-B", "topic", self.base)
        self.repo.git("mv", "apps/solo/src/a.rs", "apps/crop/src/a.rs")
        head = self.repo.commit("rename")
        plan = self.plan(head)
        self.assertSelect(plan, {"leaves#quality/solo", "leaves#quality/crop"})
        self.assertEqual(plan["changes"]["paths"], ["apps/crop/src/a.rs", "apps/solo/src/a.rs"])

    def test_delete_selects_owner_of_old_path(self):
        plan = self.plan(self.branch(remove=["apps/deck/src/extra.rs"]))
        self.assertSelect(plan, {"leaves#quality/deck"})

    def test_ancestor_manifest_selects_the_roots_below_it(self):
        plan = self.plan(self.branch({"apps/Cargo.toml": "[workspace]\n"}))
        self.assertSelect(plan, {"leaves#quality/" + v for v in ("viewer", "crop", "sheet", "deck", "solo")})

    def test_ancestor_manifest_is_not_narrowed_to_another_input(self):
        base = self.repo.commit("native apps", {".github/ci-impact.toml": CATALOGUE.replace(
            'native = ["kernel/**", "tools/native/**"]', 'native = ["kernel/**", "tools/native/**", "apps/**"]')})
        self.repo.git("checkout", "-q", "-b", "topic", base)
        head = self.repo.commit("workspace", {"apps/Cargo.toml": "[workspace]\n"})
        plan = self.plan(head, base=base)
        self.assertEqual(plan["mode"], "select", plan["fallback"])
        apps = {"leaves#quality/" + v for v in ("viewer", "crop", "sheet", "deck", "solo")}
        self.assertLessEqual(apps, self.selected(plan))

    def test_dependency_ancestor_manifest_selects_its_dependents(self):
        # workspace.package / workspace.lints inheritance in a dependency reads this manifest
        base = self.repo.commit("native crates", {".github/ci-impact.toml": CATALOGUE.replace(
            'native = ["kernel/**", "tools/native/**"]', 'native = ["kernel/**", "tools/native/**", "crates/**"]')})
        self.repo.git("checkout", "-q", "-b", "topic", base)
        head = self.repo.commit("workspace", {"crates/Cargo.toml": "[workspace.package]\nedition = \"2021\"\n"})
        plan = self.plan(head, base=base)
        self.assertEqual(plan["mode"], "select", plan["fallback"])
        self.assertLessEqual({"leaves#quality/viewer", "leaves#quality/crop"}, self.selected(plan))

    def test_nested_tool_configuration_forces_full(self):
        for path in ("kernel/rustfmt.toml", "kernel/.rustfmt.toml", "kernel/clippy.toml",
                     "kernel/.clippy.toml", "kernel/rust-toolchain", "kernel/.cargo/config.toml"):
            with self.subTest(path=path):
                self.assertFull(self.plan(self.branch({path: "# x\n"}, message=path)), "always-full")

    def test_extra_target_sources_select_their_consumers(self):
        cases = {"ui/tokens.css": {"leaves#quality/sheet", "window#a", "window#b", "window#c"},
                 "shared/main.rs": {"leaves#quality/solo"},
                 "probe/contract.rs": {"net#broker"}}
        for path, expected in cases.items():
            with self.subTest(path=path):
                self.assertSelect(self.plan(self.branch({path: "changed\n"})), expected)

    def test_new_dependency_in_head_is_followed(self):
        head = self.branch({"apps/solo/Cargo.toml": manifest("solo", ["../../crates/legacy"]),
                            "crates/legacy/src/lib.rs": "// legacy 2\n"})
        self.assertSelect(self.plan(head), {"leaves#quality/solo", "leaves#quality/deck"})

    def test_dependency_removed_in_head_still_follows_base_graph(self):
        head = self.branch({"apps/deck/Cargo.toml": manifest("deck", ["../../crates/gfx"]),
                            "crates/legacy/src/lib.rs": "// legacy 2\n"})
        self.assertSelect(self.plan(head), {"leaves#quality/deck"})

    def test_needs_closure_selects_upstream_job(self):
        plan = self.plan(self.branch({"tools/native/run.py": "# changed\n"}))
        self.assertSelect(plan, {"net#native", "net#broker", "window#a", "window#b", "window#c", "storage#s"})
        self.assertNotIn("misc#advice", self.selected(plan))

    def test_workflow_granularity_selects_all_window_jobs(self):
        plan = self.plan(self.branch({"ui/other.rs": "// ui\n"}))
        self.assertSelect(plan, {"window#a", "window#b", "window#c"})

    def test_blocked_by_filter_is_reported(self):
        plan = self.plan(self.branch({"tools/native/run.py": "# changed\n"}))
        rows = {row["id"]: row for row in plan["selected"]}
        self.assertTrue(rows["storage#s"]["blocked_by_filter"])
        self.assertFalse(rows["net#native"]["blocked_by_filter"])
        plan = self.plan(self.branch({"kernel/main.rs": "// k\n"}, message="kernel"))
        self.assertFalse({r["id"]: r for r in plan["selected"]}["storage#s"]["blocked_by_filter"])

    def test_merge_ref_must_join_base_and_head(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        self.repo.git("checkout", "-q", "--detach", self.base)
        self.repo.git("-c", "user.name=f", "-c", "user.email=f@e", "merge", "-q", "--no-ff",
                      "-m", "merge", head)
        merge = self.repo.git("rev-parse", "HEAD")
        self.assertSelect(self.plan(head, tested=merge), {"leaves#quality/solo"})
        self.assertFull(self.plan(head, tested=self.base), "merge-ref-mismatch")


class FallbackTests(FixtureTestCase):
    def test_missing_history_falls_back_to_full(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        clone = Path(self._tmp.name) / "shallow"
        subprocess.run(["git", "clone", "-q", "--no-local", "--depth=1", "--branch", "topic",
                        f"file://{self.repo.root}", str(clone)], check=True, env=GIT_ENV,
                       capture_output=True)
        subprocess.run(["git", "-C", str(clone), "fetch", "-q", "--depth=1", "origin", "main"],
                       check=True, env=GIT_ENV, capture_output=True)
        subprocess.run(["git", "-C", str(clone), "remote", "remove", "origin"], check=True,
                       env=GIT_ENV, capture_output=True)
        main_tip = self.repo.git("rev-parse", "main")
        plan = ci_impact.make_plan(str(clone), "pull_request", main_tip, head,
                                   selector_bytes=SELECTOR.read_bytes(), allow_fetch=True)
        self.assertFull(plan, "missing-history")

    def test_criss_cross_history_has_no_single_merge_base(self):
        git = self.repo.git
        x1 = self.branch({"apps/solo/src/lib.rs": "// x\n"}, message="x1")
        git("checkout", "-q", "-B", "other", self.base)
        y1 = self.repo.commit("y1", {"apps/crop/src/lib.rs": "// y\n"})
        git("checkout", "-q", "--detach", x1)
        git("merge", "-q", "--no-edit", y1)
        x2 = git("rev-parse", "HEAD")
        git("checkout", "-q", "--detach", y1)
        git("merge", "-q", "--no-edit", x1)
        y2 = self.repo.commit("y3", {"apps/deck/src/lib.rs": "// y3\n"})
        self.assertEqual(len(git("merge-base", "--all", x2, y2).split()), 2)
        self.assertFull(self.plan(y2, base=x2), "ambiguous-merge-base")

    def test_shallow_merge_ref_keeps_its_recorded_parents(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        self.repo.git("checkout", "-q", "-B", "merge", self.base)
        self.repo.git("merge", "-q", "--no-ff", "--no-edit", head)
        tested = self.repo.git("rev-parse", "HEAD")
        clone = Path(self._tmp.name) / "shallow"
        run = lambda *args: subprocess.run(["git", *args], check=True, env=GIT_ENV, capture_output=True)  # noqa: E731
        run("clone", "-q", "--no-local", "--depth=1", "--branch", "merge", f"file://{self.repo.root}", str(clone))
        run("-C", str(clone), "fetch", "-q", "--depth=1", "origin", "main", "topic")
        self.assertEqual(run("-C", str(clone), "rev-list", "--parents", "-n", "1", tested).stdout.split()[1:], [])
        plan = ci_impact.make_plan(str(clone), "pull_request", self.base, head, tested,
                                   selector_bytes=SELECTOR.read_bytes(), allow_fetch=True)
        self.assertSelect(plan, {"leaves#quality/solo"})

    def test_fetch_failure_falls_back_to_full(self):
        plan = self.plan("e" * 40)
        self.assertFull(plan, "fetch-failed")

    def test_fork_pull_request_uses_base_catalogue_and_selector(self):
        head_catalogue = CATALOGUE.replace(
            '  { id = "solo", check_name = "quality (solo)", cargo = ["apps/solo/Cargo.toml"] },\n', "")
        head_selector = SELECTOR.read_bytes() + b"\n# head edit\n"
        head = self.branch({".github/ci-impact.toml": head_catalogue, "tools/ci_impact.py": head_selector,
                            "apps/solo/src/lib.rs": "// solo 2\n"})
        plan = self.plan(head, selector=SELECTOR.read_bytes())
        self.assertTrue(plan["selector"]["trusted"])
        self.assertEqual(plan["catalogue"]["source_sha"], self.base)
        self.assertEqual(plan["catalogue"]["blob"], ci_impact.blob_id(CATALOGUE.encode()))
        self.assertIn("leaves#quality/solo", self.selected(plan))
        self.assertFull(plan, "always-full")
        self.assertFalse(self.plan(head, selector=head_selector)["selector"]["trusted"])

    def test_trusted_base_copy_runs_isolated(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        trusted = Path(self._tmp.name) / "trusted_ci_impact.py"
        trusted.write_bytes(subprocess.run(["git", "-C", self.repo.root, "show", f"{self.base}:tools/ci_impact.py"],
                                           capture_output=True, check=True, env=GIT_ENV).stdout)
        result = subprocess.run([sys.executable, "-B", "-I", str(trusted), "plan", "--event", "pull_request",
                                 "--base-sha", self.base, "--head-sha", head, "--repo", self.repo.root,
                                 "--no-fetch"], capture_output=True, env=GIT_ENV, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["mode"], "select")
        self.assertTrue(plan["selector"]["trusted"])

    def test_merge_group_is_full_on_its_queue_commit(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        plan = self.plan(head, event="merge_group")
        self.assertFull(plan, "merge-group")
        self.assertEqual(plan["changes"]["count"], 1)
        self.assertEqual(self.selected(plan), CORE)  # only ci has merge_group among core/rwa

    def test_forced_full_events_ignore_a_select_or_manual_policy(self):
        for event, code in (("merge_group", "merge-group"), ("push", "push-main")):
            for policy in ("select", "manual"):
                with self.subTest(event=event, policy=policy):
                    self.repo.git("checkout", "-q", "--detach", self.base)
                    base = self.repo.commit(policy, {".github/ci-impact.toml": CATALOGUE.replace(
                        f'{event} = "full"', f'{event} = "{policy}"')})
                    head = self.repo.commit("leaf", {"apps/solo/src/lib.rs": "// solo 2\n"})
                    with mock.patch.object(ci_impact, "validate", return_value=[]):
                        self.assertFull(self.plan(head, base=base, event=event), code)

    def test_policy_changes_force_full(self):
        for path in (".github/ci-impact.toml", "tools/ci_impact.py", ".github/workflows/net.yml"):
            with self.subTest(path=path):
                content = Path(self.repo.root, path).read_bytes() + b"\n# x\n" if Path(self.repo.root, path).exists() \
                    else b"name: net\n"
                self.assertFull(self.plan(self.branch({path: content})), "always-full")

    def test_rollback_mode_full(self):
        base = self.repo.commit("rollback", {".github/ci-impact.toml": CATALOGUE.replace(
            'mode = "shadow"', 'mode = "full"')})
        self.repo.git("checkout", "-q", "-b", "topic", base)
        head = self.repo.commit("leaf", {"apps/solo/src/lib.rs": "// solo 2\n"})
        self.assertFull(self.plan(head, base=base), "policy-rollback")

    def test_invalid_trusted_catalogue_is_full(self):
        base = self.repo.commit("bad", {".github/ci-impact.toml": CATALOGUE.replace('class = "core"', 'class = "x"')})
        self.repo.git("checkout", "-q", "-b", "topic", base)
        head = self.repo.commit("leaf", {"apps/solo/src/lib.rs": "// solo 2\n"})
        self.assertFull(self.plan(head, base=base), "invalid-trusted-catalogue")

    def test_full_never_selects_manual_operational_advisory_or_publication(self):
        head = self.branch({"kernel/main.rs": "// k\n"})
        for event in ("merge_group", "push"):
            with self.subTest(event=event):
                plan = self.plan(head, event=event)
                self.assertEqual(plan["mode"], "full")
                self.assertFalse(NEVER_FULL & self.selected(plan))
                reasons = {row["id"]: row["reason"] for row in plan["excluded"]}
                self.assertEqual(reasons["misc#manual"], "manual")
                self.assertEqual(reasons["misc#ops"], "operational")
                self.assertEqual(reasons["misc#advice"], "advisory")
                self.assertEqual(reasons["misc#publish"], "publication")
        plan = self.plan(self.branch({"stray.txt": "x\n"}, message="stray"))
        self.assertEqual(self.selected(plan), CORE | ALL_RWA)

    def test_push_to_main_keeps_full_integration_coverage(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        plan = self.plan(head, event="push")
        self.assertFull(plan, "push-main")
        self.assertEqual(self.selected(plan), CORE | (ALL_RWA - {"storage#s"}))
        self.assertEqual({r["id"]: r["reason"] for r in plan["excluded"]}["storage#s"], "event-not-applicable")

    def test_push_with_unknown_before_is_full(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        for before in ("", ci_impact.ZERO_SHA):
            with self.subTest(before=before):
                self.assertFull(self.plan(head, base=before, event="push"), "push-unknown-before")
        self.repo.git("checkout", "-q", "main")
        unrelated = self.repo.commit("main only", {"apps/crop/src/lib.rs": "// main\n"})
        self.assertFull(self.plan(head, base=unrelated, event="push"), "push-unknown-before")

    def test_workflow_dispatch_is_manual_and_selects_nothing(self):
        plan = self.plan(self.branch({"kernel/main.rs": "// k\n"}), event="workflow_dispatch")
        self.assertEqual(plan["mode"], "manual")
        self.assertEqual(plan["selected"], [])
        self.assertIsNone(plan["fallback"])

    def test_events_without_a_base_are_full_and_untrusted(self):
        for event in ("workflow_dispatch", "schedule"):
            with self.subTest(event=event):
                plan = self.plan(self.base, base="", event=event)
                self.assertFull(plan, "no-trusted-base")
                self.assertFalse(plan["selector"]["trusted"])
                self.assertEqual(plan["catalogue"]["source_sha"], self.base)  # listing only

    def test_cargo_patch_tables_are_not_guessed(self):
        plan = self.plan(self.branch({"apps/solo/Cargo.toml": manifest(
            "solo", extra='\n[patch.crates-io]\nx = { path = "../../crates/codec" }\n')}))
        self.assertFull(plan, "cargo-patch-unresolved")

    def test_missing_cargo_root_is_full_not_narrowed(self):
        base = self.repo.commit("typo", {".github/ci-impact.toml": CATALOGUE.replace(
            'cargo = ["svc/stack/Cargo.toml"]', 'cargo = ["svc/stak/Cargo.toml"]')})
        self.repo.git("checkout", "-q", "-b", "topic", base)
        head = self.repo.commit("stack", {"svc/stack/src/lib.rs": "// stack 2\n"})
        plan = self.plan(head, base=base)
        self.assertFull(plan, "cargo-manifest-missing")
        self.assertEqual(plan["fallback"]["detail"], ["svc/stak/Cargo.toml"])
        self.assertIn("net#broker", self.selected(plan))

    def test_git_read_errors_are_not_absence(self):
        self.assertIsNone(ci_impact.git_blob(self.repo.root, self.base, "no/such/Cargo.toml"))
        with self.assertRaises(ci_impact.Full) as raised:
            ci_impact.git_blob(self.repo.root, "e" * 40, "README.md")
        self.assertEqual(raised.exception.code, "git-read-failed")
        real = ci_impact.git

        def failing(repo, *args, **kwargs):
            if args[0] in ("cat-file", "ls-tree") and args[-1].endswith("Cargo.toml"):
                return 124, b""
            return real(repo, *args, **kwargs)

        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        with mock.patch.object(ci_impact, "git", side_effect=failing):
            self.assertFull(self.plan(head), "git-read-failed")

    def test_ancestor_patch_tables_are_not_guessed(self):
        for table in ("[patch.crates-io]\nx = { path = \"../crates/codec\" }\n", "[replace]\n"):
            with self.subTest(table=table):
                plan = self.plan(self.branch({"apps/Cargo.toml": "[workspace]\n" + table}, message=table))
                self.assertFull(plan, "cargo-patch-unresolved")
                self.assertEqual(plan["fallback"]["detail"], ["apps/Cargo.toml"])

    def test_bootstrap_without_base_catalogue_or_selector_is_full(self):
        for removed in (".github/ci-impact.toml", "tools/ci_impact.py"):
            with self.subTest(removed=removed):
                self.repo.git("checkout", "-q", "--detach", self.base)
                base = self.repo.commit("no " + removed, remove=[removed])
                self.repo.git("checkout", "-q", "--detach", base)
                restored = CATALOGUE if removed.endswith(".toml") else SELECTOR.read_bytes()
                head = self.repo.commit("introduce", {removed: restored})
                plan = self.plan(head, base=base)
                self.assertFull(plan, "bootstrap-no-base-catalogue")
                self.assertLessEqual(ALL_RWA, self.selected(plan))  # listed from the head catalogue

    def test_bootstrap_cli_exits_zero(self):
        self.repo.git("checkout", "-q", "--detach", self.base)
        base = self.repo.commit("no catalogue", remove=[".github/ci-impact.toml"])
        head = self.repo.commit("add catalogue", {".github/ci-impact.toml": CATALOGUE})
        out = Path(self._tmp.name) / "out" / "plan.json"
        summary = Path(self._tmp.name) / "summary.md"
        result = subprocess.run([sys.executable, "-B", str(SELECTOR), "plan", "--event", "pull_request",
                                 "--base-sha", base, "--head-sha", head, "--repo", self.repo.root, "--no-fetch",
                                 "--out", str(out), "--summary", str(summary)], capture_output=True, env=GIT_ENV)
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = json.loads(out.read_text())
        self.assertFull(plan, "bootstrap-no-base-catalogue")
        self.assertIn("bootstrap-no-base-catalogue", summary.read_text())


    def test_hostile_head_catalogue_ids_never_reach_the_summary(self):
        hostile = 'x|<img src=x onerror=alert(1)>[a](https://example.invalid)'
        self.repo.git("checkout", "-q", "--detach", self.base)
        base = self.repo.commit("no catalogue", remove=[".github/ci-impact.toml"])
        head = self.repo.commit("hostile", {".github/ci-impact.toml": CATALOGUE.replace(
            '"broker"', f"'{hostile}'")})
        summary = ci_impact.summary_markdown(self.plan(head, base=base))
        self.assertNotIn("<img", summary)
        self.assertNotIn("](https", summary)

    def test_summary_cells_are_escaped(self):
        plan = {"mode": "select", "plan_id": "sha256:0", "fallback": None, "excluded": [],
                "selected": [{"id": "a|b<c>[d](e)`f`&g\nh", "class": "rwa", "blocked_by_filter": False}]}
        row = ci_impact.summary_markdown(plan).splitlines()[-1]
        self.assertEqual(row, "| a&#124;b&lt;c&gt;&#91;d&#93;(e)&#96;f&#96;&amp;g h | rwa | yes | False |")


class DeterminismAndRobustnessTests(FixtureTestCase):
    def test_plan_is_byte_identical_and_id_tracks_inputs(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        first, second = self.plan(head), self.plan(head)
        self.assertEqual(ci_impact.canonical(first), ci_impact.canonical(second))
        other = self.repo.commit("more", {"apps/crop/src/lib.rs": "// crop 2\n"})
        self.assertNotEqual(first["plan_id"], self.plan(other)["plan_id"])
        self.assertNotEqual(first["plan_id"], self.plan(head, event="merge_group")["plan_id"])
        self.assertNotEqual(first["plan_id"], self.plan(head, selector=b"other")["plan_id"])

    def test_control_character_and_non_utf8_paths(self):
        weird = "apps/solo/src/we\nird\x01\t.rs"
        head = self.branch({weird: "// weird\n", os.fsdecode(b"apps/crop/src/\xff.rs"): "// bytes\n"})
        plan = self.plan(head)
        self.assertSelect(plan, {"leaves#quality/solo", "leaves#quality/crop"})
        text = json.dumps(plan, ensure_ascii=True)
        self.assertNotIn("\n", ci_impact.canonical(plan))
        self.assertIn(weird, json.loads(text)["changes"]["paths"])
        stray = self.plan(self.branch({"stray\n\x1b[31m": "x\n"}, message="stray"))
        self.assertFull(stray, "unknown-path")
        self.assertNotIn("\x1b", ci_impact.canonical(stray))

    def test_unknown_diff_status_falls_back_to_full(self):
        with mock.patch.object(ci_impact, "git", return_value=(0, b"X\0path\0")):
            with self.assertRaises(ci_impact.Full) as raised:
                ci_impact.changed_paths(".", "a" * 40, "b" * 40)
        self.assertEqual(raised.exception.code, "unknown-diff-status")
        with mock.patch.object(ci_impact, "changed_paths", side_effect=ci_impact.Full("unknown-diff-status")):
            self.assertFull(self.plan(self.branch({"apps/solo/src/lib.rs": "// 2\n"})), "unknown-diff-status")

    def test_unreadable_manifest_falls_back_to_full(self):
        plan = self.plan(self.branch({"apps/solo/Cargo.toml": "[package\n"}))
        self.assertFull(plan, "cargo-unreadable")
        self.assertEqual(plan["fallback"]["detail"], ["apps/solo/Cargo.toml"])

    def test_path_dependency_escaping_the_repository_is_full(self):
        plan = self.plan(self.branch({"apps/solo/Cargo.toml": manifest("solo", ["../../../outside"])}))
        self.assertFull(plan, "cargo-path-escapes-repo")

    def test_selector_error_yields_full_with_exit_0(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        with mock.patch.object(ci_impact, "changed_paths", side_effect=RuntimeError("boom")):
            plan = self.plan(head)
        self.assertFull(plan, "selector-error:RuntimeError")
        not_a_repo = Path(self._tmp.name) / "empty"
        not_a_repo.mkdir()
        result = subprocess.run([sys.executable, "-B", str(SELECTOR), "plan", "--event", "pull_request",
                                 "--base-sha", "a" * 40, "--head-sha", "b" * 40, "--repo", str(not_a_repo),
                                 "--no-fetch"], capture_output=True, env=GIT_ENV)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(json.loads(result.stdout)["mode"], "full")
        with mock.patch.object(ci_impact, "make_plan", side_effect=MemoryError):
            with contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()):
                code = ci_impact.main(["plan", "--event", "push", "--head-sha", "b" * 40])
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out.getvalue())["fallback"]["code"], "selector-error:MemoryError")

    def test_emergency_plan_has_every_v1_field(self):
        normal = self.plan(self.branch({"apps/solo/src/lib.rs": "// solo 2\n"}))
        with mock.patch.object(ci_impact, "make_plan", side_effect=MemoryError):
            with contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()):
                ci_impact.main(["plan", "--event", "push", "--base-sha", "a" * 40, "--head-sha", "b" * 40])
        emergency = json.loads(out.getvalue())
        self.assertEqual(sorted(emergency), sorted(normal))
        for key in ("catalogue", "selector", "shas", "changes"):
            self.assertEqual(sorted(emergency[key]), sorted(normal[key]), key)
        self.assertIs(emergency["selector"]["trusted"], False)
        self.assertEqual(emergency["shas"]["head"], "b" * 40)

    def test_output_error_never_leaves_a_select_plan(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        out = Path(self._tmp.name) / "out" / "plan.json"
        blocked = Path(self._tmp.name) / "summary-is-a-directory"
        blocked.mkdir()
        argv = ["plan", "--event", "pull_request", "--base-sha", self.base, "--head-sha", head,
                "--repo", self.repo.root, "--no-fetch", "--summary", str(blocked)]
        for target in (str(out), "-"):
            with self.subTest(out=target):
                with contextlib.redirect_stdout(io.StringIO()) as stdout, contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(ci_impact.main(argv + ["--out", target]), 0)
                plan = json.loads(out.read_text() if target != "-" else stdout.getvalue())
                self.assertFull(plan, "output-error:IsADirectoryError")
                if target != "-":
                    self.assertEqual(stdout.getvalue(), "")
                    self.assertEqual([p.name for p in out.parent.iterdir()], ["plan.json"])  # no temp file left

    def test_invalid_sha_and_unknown_event_are_full(self):
        self.assertFull(self.plan("not-a-sha"), "invalid-sha")
        self.assertFull(self.plan(self.base, event="release"), "unknown-event")

    def test_usage_errors_exit_2(self):
        for argv in ([], ["plan", "--event", "push"], ["bogus"]):
            with self.subTest(argv=argv), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as raised:
                    ci_impact.main(argv)
                self.assertEqual(raised.exception.code, 2)


def valid_catalogue():
    return tomllib.loads(CATALOGUE)


class GlobTests(TestCase):
    def test_double_star_slash_matches_zero_or_more_directories(self):
        match = ci_impact.github_path_filter_matches
        for pattern, path in (("**/x.toml", "x.toml"), ("**/x.toml", "a/b/x.toml"), ("a/**/b.rs", "a/b.rs"),
                              ("a/**/b.rs", "a/c/d/b.rs"), ("a/**", "a/b/c"), ("**", "a/b"),
                              ("**.md", "docs/a.md")):
            with self.subTest(pattern=pattern, path=path):
                self.assertTrue(match(pattern, path))
        for pattern, path in (("**/x.toml", "ax.toml"), ("a/**/b.rs", "ab.rs"), ("a/*.rs", "a/b/c.rs"),
                              ("*.md", "docs/a.md")):
            with self.subTest(pattern=pattern, path=path):
                self.assertFalse(match(pattern, path))


class CatalogueValidationTests(TestCase):
    def assertProblem(self, catalogue, fragment):
        problems = ci_impact.validate(catalogue)
        self.assertTrue(any(fragment in p for p in problems), problems)

    def test_fixture_and_real_catalogue_validate(self):
        self.assertEqual(ci_impact.validate(valid_catalogue()), [])
        with REAL_CATALOGUE.open("rb") as handle:
            self.assertEqual(ci_impact.validate(tomllib.load(handle)), [])

    def test_cli_validate_exit_codes(self):
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(ci_impact.main(["validate", str(REAL_CATALOGUE)]), 0)
            with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as handle:
                handle.write(CATALOGUE.replace("schema_version = 1", "schema_version = 2"))
            self.addCleanup(os.unlink, handle.name)
            self.assertEqual(ci_impact.main(["validate", handle.name]), 1)

    def test_rejections(self):
        def mutate(change):
            catalogue = valid_catalogue()
            change(catalogue)
            return catalogue

        wf = lambda c, i: c["workflow"][i]  # noqa: E731
        cases = [
            (lambda c: c.update(schema_version=2), "schema_version"),
            (lambda c: c.update(extra=1), "unknown top-level key"),
            (lambda c: c["policy"]["events"].pop("merge_group"), "policy.events.merge_group"),
            (lambda c: c["policy"].update(mode="loose"), "policy.mode"),
            (lambda c: c["policy"]["events"].update(merge_group="select"), "merge_group must be 'full'"),
            (lambda c: c["policy"]["events"].update(push="manual"), "push must be 'full'"),
            (lambda c: c["policy"]["events"].update(schedule="select"), "schedule must be 'full'"),
            (lambda c: c["policy"]["events"].update(workflow_dispatch="select"),
             "workflow_dispatch must be 'full' or 'manual'"),
            (lambda c: c["policy"]["always_full"].append("../x"), "'..' segment"),
            (lambda c: c["policy"]["inert"].append("/abs"), "leading '/'"),
            (lambda c: c["policy"]["core_only"].append("!neg"), "leading '/', '!'"),
            (lambda c: c["input_sets"]["native"].append("kernel/a?.rs"), "unsupported glob syntax"),
            (lambda c: c["input_sets"]["native"].append("kernel/[ab].rs"), "unsupported glob syntax"),
            (lambda c: c["input_sets"]["native"].append("kernel/x].rs"), "unsupported glob syntax"),
            (lambda c: c["input_sets"]["native"].append("kernel/a+.rs"), "unsupported glob syntax"),
            (lambda c: c["policy"]["always_full"].remove("**/clippy.toml"), "always_full must include"),
            (lambda c: wf(c, 0)["job"][0].update(**{"class": "x"}), "class must be one of"),
            (lambda c: wf(c, 0)["job"][0].update(condition="${{ x }}"), "core job must not have a condition"),
            (lambda c: wf(c, 0)["job"][0].update(activation="dispatch"), "core jobs must be automatic"),
            (lambda c: wf(c, 2)["job"][0].pop("cargo"), "rwa requires inputs"),
            (lambda c: wf(c, 2).update(obligation=[]), "rwa requires a workflow obligation"),
            (lambda c: wf(c, 2).update(obligation=[{"cite": "no-line", "anchor": "a"}]), "file:line"),
            (lambda c: wf(c, 2).update(obligation=["README.md:1"]), "file:line"),
            (lambda c: wf(c, 2).update(obligation=[{"cite": "README.md:1"}]), "anchor"),
            (lambda c: wf(c, 2).update(obligation=[{"cite": "README.md:1", "anchor": " "}]), "anchor"),
            (lambda c: wf(c, 2)["job"][1].update(needs=["ghost"]), "needs unknown job ghost"),
            (lambda c: wf(c, 2)["job"][1].update(needs=["merge-hold"]), "no job may need merge-hold"),
            (lambda c: wf(c, 3)["job"][0].update(needs=["c"]), "needs cycle"),
            (lambda c: wf(c, 1)["job"][0]["variant"][1].update(check_name="quality (viewer)"), "check name"),
            (lambda c: wf(c, 1)["job"][0]["variant"][1].update(id="viewer"), "duplicate variant ids"),
            (lambda c: wf(c, 1)["job"][0]["variant"][1].update(dynamic=True), "dynamic = true"),
            (lambda c: wf(c, 1)["job"][0]["variant"][1].update(cargo=["apps/*/Cargo.toml"]), "cargo root"),
            (lambda c: wf(c, 2)["job"][1].update(input_sets=["ghost"]), "unknown input set"),
            (lambda c: wf(c, 5)["job"][0].update(activation="automatic"), "manual job cannot be automatic"),
            (lambda c: wf(c, 4)["trigger"]["pull_request"].update(paths_ignore=["x"]), "both paths"),
            (lambda c: c["workflow"].append(copy.deepcopy(wf(c, 0))), "registered twice"),
            (lambda c: c["crate_escape"][0].update(when="sometimes"), "when must be"),
            (lambda c: wf(c, 0)["job"][0].update(surprise=True), "unknown key: surprise"),
            (lambda c: wf(c, 0)["job"][0].update(id="a b"), "invalid job id"),
            (lambda c: wf(c, 0)["job"][0].update(id="-x"), "invalid job id"),
            (lambda c: wf(c, 1)["job"][0]["variant"][0].update(id="x|y"), "invalid variant id"),
            (lambda c: wf(c, 0)["job"][1].update(id=["upload"]), "invalid job id"),
            (lambda c: wf(c, 5)["job"].append({"id": "gate", "class": "rwa", "activation": "automatic",
                                               "inputs": ["kernel/**"], "needs": ["advice"]}),
             "misc#gate: needs advice, which is never selected (advisory)"),
            (lambda c: wf(c, 5)["job"].extend([
                {"id": "gate", "class": "rwa", "activation": "automatic", "inputs": ["kernel/**"], "needs": ["mid"]},
                {"id": "mid", "class": "rwa", "activation": "automatic", "inputs": ["kernel/**"],
                 "needs": ["publish"]}]),
             "misc#gate: needs publish, which is never selected (publication)"),
            (lambda c: wf(c, 1)["job"][0]["variant"][0].update(id=["viewer"]), "invalid variant id"),
            (lambda c: wf(c, 1)["job"][0]["variant"][0].update(id={"a": 1}), "invalid variant id"),
            (lambda c: wf(c, 0).update(path=".github/workflows/a|b.yml"), "path must be"),
            (lambda c: wf(c, 0).update(path=".github/workflows/sub/a.yml"), "path must be"),
        ]
        for change, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assertProblem(mutate(change), fragment)


COVERAGE_CATALOGUE = """\
schema_version = 1
revision = 1

[policy]
mode = "shadow"
always_full = [".github/**", "rustfmt.toml", "**/rustfmt.toml", ".rustfmt.toml", "**/.rustfmt.toml",
               "clippy.toml", "**/clippy.toml", ".clippy.toml", "**/.clippy.toml",
               "rust-toolchain*", "**/rust-toolchain*", ".cargo/**", "**/.cargo/**"]
inert = ["docs/**", "**.md"]
core_only = []

[policy.events]
pull_request = "select"
merge_group = "full"
push = "full"
schedule = "full"
workflow_dispatch = "manual"

[[workflow]]
path = ".github/workflows/ci.yml"
owner = "ci-owner"
granularity = "job"
events = ["pull_request", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]
trigger.pull_request = { types = ["opened"] }
trigger.push = { branches = ["main"], paths_ignore = "@metadata" }

[[workflow.job]]
id = "host"
name = "host"
class = "core"
activation = "automatic"
timeout_minutes = 30
evidence = ["host-evidence"]

[[workflow.job]]
id = "leaf"
name = "leaf (${{ matrix.component }})"
class = "rwa"
activation = "automatic"
needs = ["host"]
variant = [
  { id = "a", check_name = "leaf (a)", matrix = { component = "a" }, inputs = ["src/a/**"] },
  { id = "b", check_name = "leaf (b)", matrix = { component = "b" }, inputs = ["src/b/**"] },
]

[[workflow.job]]
id = "pair"
class = "rwa"
activation = "automatic"
inputs = ["src/**"]
variant = [
  { id = "1", check_name = "pair (1)", matrix = { scale = 1 } },
  { id = "2", check_name = "pair (2)", matrix = { scale = 2 } },
]
"""

COVERAGE_WORKFLOWS = {
    ".github/workflows/ci.yml": {
        "on": {"pull_request": {"types": ["opened"]},
               "push": {"branches": ["main"], "paths-ignore": ["docs/**", "**.md"]}},
        "jobs": {
            "host": {"name": "host", "timeout-minutes": "30",
                     "steps": [{"uses": "./.github/actions/preserve-evidence", "with": {"name": "host-evidence"}}]},
            "leaf": {"name": "leaf (${{ matrix.component }})", "needs": "host",
                     "strategy": {"matrix": {"include": [{"component": "a"}, {"component": "b"}]}}},
            "pair": {"strategy": {"matrix": {"scale": ["1", "2"]}}},
        },
    },
}

COVERAGE_YAML = """\
on:
  pull_request:
    types: [opened]
  push:
    branches: [main]
    paths-ignore: ['docs/**', '**.md']
jobs:
  host:
    name: host
    timeout-minutes: 30
    steps:
      - uses: ./.github/actions/preserve-evidence
        with:
          name: host-evidence
  leaf:
    name: leaf (${{ matrix.component }})
    needs: host
    strategy:
      matrix:
        include:
          - component: a
          - component: b
  pair:
    strategy:
      matrix:
        scale: [1, 2]
"""


class CoverageTests(TestCase):
    def test_clean_fixture_has_no_problems(self):
        catalogue = tomllib.loads(COVERAGE_CATALOGUE)
        self.assertEqual(ci_impact.validate(catalogue), [])
        self.assertEqual(ci_impact.workflow_problems(COVERAGE_WORKFLOWS, catalogue), [])

    def test_unregistered_or_removed_workflow_job_or_variant_is_reported(self):
        catalogue = tomllib.loads(COVERAGE_CATALOGUE)
        workflows = copy.deepcopy(COVERAGE_WORKFLOWS)
        ci = workflows[".github/workflows/ci.yml"]
        ci["jobs"]["leaf"]["strategy"]["matrix"]["include"].append({"component": "c"})
        del ci["jobs"]["pair"]
        ci["jobs"]["host"]["timeout-minutes"] = "45"
        ci["on"]["merge_group"] = {}
        workflows[".github/workflows/new.yml"] = {"on": {"pull_request": {}}, "jobs": {"x": {}}}
        problems = ci_impact.workflow_problems(workflows, catalogue)
        for fragment in ("variant-unregistered: .github/workflows/ci.yml#leaf: leaf (c)",
                         "job-removed: .github/workflows/ci.yml#pair",
                         "workflow-unregistered: .github/workflows/new.yml",
                         "field-mismatch: .github/workflows/ci.yml#host: timeout_minutes",
                         "events-mismatch: .github/workflows/ci.yml"):
            with self.subTest(fragment=fragment):
                self.assertTrue(any(p.startswith(fragment) for p in problems), problems)
        workflows = copy.deepcopy(COVERAGE_WORKFLOWS)
        workflows[".github/workflows/ci.yml"]["jobs"]["extra"] = {}
        workflows[".github/workflows/ci.yml"]["jobs"]["host"]["steps"][0]["with"]["name"] = "renamed"
        workflows[".github/workflows/ci.yml"]["on"]["push"]["paths-ignore"] = ["docs/**"]
        problems = ci_impact.workflow_problems(workflows, catalogue)
        for fragment in ("job-unregistered: .github/workflows/ci.yml#extra", "evidence-mismatch",
                         "trigger-mismatch: .github/workflows/ci.yml: push.paths_ignore"):
            with self.subTest(fragment=fragment):
                self.assertTrue(any(p.startswith(fragment) for p in problems), problems)
        self.assertIn("workflow-removed: .github/workflows/ci.yml", ci_impact.workflow_problems({}, catalogue))

    def test_dynamic_matrix_evidence_templates_are_compared(self):
        workflows = copy.deepcopy(COVERAGE_WORKFLOWS)
        workflows[".github/workflows/ci.yml"]["jobs"]["pair"] = {
            "strategy": {"matrix": {"scale": "${{ fromJSON(inputs.scales) }}"}},
            "steps": [{"uses": "./.github/actions/preserve-evidence", "with": {"name": "pair-${{ matrix.scale }}"}}]}
        old = """variant = [
  { id = "1", check_name = "pair (1)", matrix = { scale = 1 } },
  { id = "2", check_name = "pair (2)", matrix = { scale = 2 } },
]"""
        for evidence, expected in (("pair-${{ matrix.scale }}", []), ("pair-renamed", ["evidence-mismatch"])):
            with self.subTest(evidence=evidence):
                catalogue = tomllib.loads(COVERAGE_CATALOGUE.replace(old, (
                    'variant = [{ id = "*", dynamic = true, check_name = "pair (${{ matrix.scale }})", '
                    'matrix = { scale = "${{ fromJSON(inputs.scales) }}" }, evidence = ["%s"] }]' % evidence)))
                self.assertEqual(ci_impact.validate(catalogue), [])
                problems = ci_impact.workflow_problems(workflows, catalogue)
                self.assertEqual([p.split(":")[0] for p in problems], expected, problems)

    def test_activation_must_match_the_workflow_events(self):
        problem = ci_impact.activation_problem
        on = ["workflow_dispatch", "schedule", "pull_request"]
        self.assertIsNotNone(problem("schedule", on, "github.event_name != 'pull_request' || x"))
        self.assertIsNone(problem("schedule", on, "github.event_name == 'schedule'"))
        self.assertIsNone(problem("automatic", on, None))
        self.assertIsNone(problem("schedule", ["schedule"], None))
        self.assertIsNotNone(problem("automatic", ["schedule", "workflow_dispatch"], None))
        self.assertIsNotNone(problem("dispatch", ["pull_request"], None))
        self.assertIsNotNone(problem("label:x", ["push"], None))
        self.assertIsNotNone(problem("push-branch:release", ["pull_request"], None))
        catalogue = tomllib.loads(COVERAGE_CATALOGUE.replace(
            'id = "host"\nname = "host"\nclass = "core"\nactivation = "automatic"',
            'id = "host"\nname = "host"\nclass = "operational"\nactivation = "schedule"'))
        problems = ci_impact.workflow_problems(COVERAGE_WORKFLOWS, catalogue)
        self.assertTrue(any(p.startswith("activation-mismatch: .github/workflows/ci.yml#host") for p in problems),
                        problems)

    @skipUnless(HAVE_YAML, "PyYAML is not importable")
    def test_coverage_report_on_a_fixture_repository(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Repo(tmp)
            repo.commit("base", {".github/ci-impact.toml": COVERAGE_CATALOGUE,
                                 ".github/workflows/ci.yml": COVERAGE_YAML, "README.md": "fixture readme\n",
                                 "src/a/lib.rs": "// a\n"})
            report = ci_impact.coverage(tmp, ".github/ci-impact.toml")
            self.assertEqual(report["status"], "clean", report)
            repo.commit("drift", {"stray.txt": "x\n", ".github/workflows/ci.yml": COVERAGE_YAML.replace(
                "          - component: b\n", "          - component: b\n          - component: c\n")})
            report = ci_impact.coverage(tmp, ".github/ci-impact.toml")
            self.assertEqual(report["status"], "drift")
            self.assertEqual(report["unclassified"], ["stray.txt"])
            self.assertIn("variant-unregistered: .github/workflows/ci.yml#leaf: leaf (c)", report["problems"])
            with contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(ci_impact.main(["coverage", "--repo", tmp]), 0)
            self.assertEqual(json.loads(out.getvalue())["status"], "drift")

    def test_citation_anchors_are_checked(self):
        lines = ["one", "two", "three", "the fixture rule", "five", "six", "seven", "eight", "nine"]
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "R.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
            check = lambda cite, anchor: ci_impact.citation_problem(tmp, {"cite": cite, "anchor": anchor})  # noqa: E731
            self.assertIsNone(check("R.md:4", "fixture rule"))
            self.assertIsNone(check("R.md:2-5", "fixture rule"))
            self.assertEqual(check("R.md:8", "fixture rule"), "citation-moved: R.md:8 -> R.md:4")
            self.assertEqual(check("R.md:1", "absent"), "citation-anchor-missing: R.md:1")
            self.assertEqual(check("R.md:99", "absent"), "citation-anchor-missing: R.md:99")
            self.assertEqual(check("gone.md:1", "x"), "citation-missing: gone.md:1")

    @skipUnless(HAVE_YAML, "PyYAML is not importable")
    def test_tracked_symlinks_in_inputs_are_reported(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Repo(tmp)
            Path(tmp, "src/a").mkdir(parents=True)
            Path(tmp, "docs").mkdir()
            os.symlink("../../README.md", Path(tmp, "src/a/link.rs"))
            os.symlink("../README.md", Path(tmp, "docs/link.md"))
            repo.commit("base", {".github/ci-impact.toml": COVERAGE_CATALOGUE,
                                 ".github/workflows/ci.yml": COVERAGE_YAML, "README.md": "fixture readme\n"})
            problems = ci_impact.coverage(tmp, ".github/ci-impact.toml")["problems"]
        self.assertIn("symlink-input: src/a/link.rs", problems)
        self.assertFalse(any("docs/link.md" in p for p in problems), problems)

    def test_coverage_without_yaml_is_unavailable(self):
        with mock.patch.dict(sys.modules, {"yaml": None}):
            report = ci_impact.coverage(".", ".github/ci-impact.toml")
        self.assertEqual(report["status"], "coverage-unavailable")

    def test_escape_scanner_reports_unregistered_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "app/src").mkdir(parents=True)
            Path(tmp, "app/src/lib.rs").write_text(
                '#[path = "../../shared/x.rs"]\nmod x;\nconst A: &[u8] = include_bytes!(\n  "../../data/a.bin"\n);\n'
                'const B: &[u8] = include_bytes!(concat!("../../fix/", "b"));\nconst C: &str = include_str!("own.txt");\n')
            Path(tmp, "shared").mkdir()
            Path(tmp, "shared/x.rs").write_text('const D: &str = include_str!("../tokens.css");\n')
            found = {(crate, target) for crate, target, _ in ci_impact.scan_escapes(tmp, {"app"})}
        self.assertEqual(found, {("app", "shared/x.rs"), ("app", "data/a.bin"), ("app", "fix/**"),
                                 ("app", "tokens.css")})


if __name__ == "__main__":
    main()
