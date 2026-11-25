"""Unit tests for the CI impact selector; temporary git repositories, no network."""

import contextlib
import copy
import functools
import importlib.util
import io
import json
import os
import re
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

[policy.budgets]
pull_request_wait_minutes = 0
merge_group_wait_minutes = 45

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
                 # A `mod` file beside an external target is compiled with it.
                 "shared/sibling.rs": {"leaves#quality/solo"},
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
        # #494 C1: the blocked row names the registered input its filter hides.
        self.assertEqual(rows["storage#s"]["filter_hidden_inputs"], ["tools/native/run.py"])
        self.assertEqual(rows["net#native"]["filter_hidden_inputs"], [])
        plan = self.plan(self.branch({"kernel/main.rs": "// k\n"}, message="kernel"))
        self.assertFalse({r["id"]: r for r in plan["selected"]}["storage#s"]["blocked_by_filter"])

    def test_filter_hiding_only_metadata_lists_no_hidden_input(self):
        plan = self.plan(self.branch({"tools/native/NOTES.md": "notes\n"}, message="notes"))
        rows = {row["id"]: row for row in plan["selected"]}
        self.assertTrue(rows["storage#s"]["blocked_by_filter"])
        self.assertEqual(rows["storage#s"]["filter_hidden_inputs"], [])

    def test_blocked_by_filter_is_unknown_beyond_the_filter_limit(self):
        # GitHub does not document filter results past 300 files, so no definite answer
        many = {f"tools/native/f{i}.py": "# x\n" for i in range(ci_impact.FILTER_FILE_LIMIT + 1)}
        plan = self.plan(self.branch(many, message="many"))
        self.assertTrue(plan["filter_eval_uncertain"])
        rows = {row["id"]: row for row in plan["selected"]}
        self.assertIsNone(rows["storage#s"]["blocked_by_filter"])
        summary = ci_impact.summary_markdown(plan)
        self.assertIn("| storage#s | rwa | yes | unknown |", summary)

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
        # The selector's own fetch reads global config; pin the file transport in the clone.
        run("-C", str(clone), "config", "protocol.file.allow", "always")
        self.assertEqual(run("-C", str(clone), "rev-list", "--parents", "-n", "1", tested).stdout.split()[1:], [])
        plan = ci_impact.make_plan(str(clone), "pull_request", self.base, head, tested,
                                   selector_bytes=SELECTOR.read_bytes(), allow_fetch=True)
        self.assertSelect(plan, {"leaves#quality/solo"})

    def test_shallow_clone_accepts_a_merge_base_only_with_full_history(self):
        # A visible single merge base behind a shallow boundary may not be the true one.
        self.repo.commit("c1", {"docs/guide.md": "guide 1\n"})
        tip = self.repo.commit("c2", {"docs/guide.md": "guide 2\n"})
        self.repo.git("checkout", "-q", "-B", "topic", tip)
        head = self.repo.commit("head", {"apps/solo/src/lib.rs": "// solo 2\n"})
        clone = Path(self._tmp.name) / "shallow"
        run = lambda *args: subprocess.run(["git", *args], check=True, env=GIT_ENV, capture_output=True)  # noqa: E731
        run("clone", "-q", "--no-local", "--depth=2", "--branch", "topic", f"file://{self.repo.root}", str(clone))
        run("-C", str(clone), "config", "protocol.file.allow", "always")
        shallow = lambda: run("-C", str(clone), "rev-parse", "--is-shallow-repository").stdout.strip()  # noqa: E731
        self.assertEqual(shallow(), b"true")
        make = functools.partial(ci_impact.make_plan, str(clone), "pull_request", tip, head,
                                 selector_bytes=SELECTOR.read_bytes())
        self.assertFull(make(allow_fetch=False), "shallow-history")
        self.assertSelect(make(allow_fetch=True), {"leaves#quality/solo"})  # unshallowed, then recomputed
        self.assertEqual(shallow(), b"false")

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

    def test_fallback_before_classification_hides_every_changed_input(self):
        # #544 review: without hits a blocked row must not read as "hides only metadata".
        plan = self.plan(self.branch({"tools/native/run.py": "# changed\n",
                                      "apps/solo/Cargo.toml": manifest(
            "solo", extra='\n[patch.crates-io]\nx = { path = "../../crates/codec" }\n')}))
        self.assertFull(plan, "cargo-patch-unresolved")
        rows = {row["id"]: row for row in plan["selected"]}
        self.assertTrue(rows["storage#s"]["blocked_by_filter"])
        self.assertEqual(rows["storage#s"]["filter_hidden_inputs"],
                         ["apps/solo/Cargo.toml", "tools/native/run.py"])

    def test_explicit_package_workspace_is_not_guessed(self):
        # The workspace root lies outside the ancestor walk; a change to it alone would be missed.
        solo = manifest("solo").replace('version = "0.1.0"', 'version = "0.1.0"\nworkspace = "../../ws"')
        plan = self.plan(self.branch({"apps/solo/Cargo.toml": solo, "ws/Cargo.toml": "[workspace]\n"}))
        self.assertFull(plan, "cargo-explicit-workspace")
        self.assertEqual(plan["fallback"]["detail"], ["apps/solo/Cargo.toml"])

    def test_legacy_project_table_is_not_guessed(self):
        # Cargo still reads [project] as [package], including its workspace key.
        solo = manifest("solo").replace("[package]", "[project]")
        plan = self.plan(self.branch({"apps/solo/Cargo.toml": solo}))
        self.assertFull(plan, "cargo-legacy-project-table")
        self.assertEqual(plan["fallback"]["detail"], ["apps/solo/Cargo.toml"])

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

    def test_failed_publication_removes_a_stale_plan(self):
        head = self.branch({"apps/solo/src/lib.rs": "// solo 2\n"})
        out = Path(self._tmp.name) / "out" / "plan.json"
        argv = ["plan", "--event", "pull_request", "--base-sha", self.base, "--head-sha", head,
                "--repo", self.repo.root, "--no-fetch", "--out", str(out)]
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(ci_impact.main(argv), 0)
        self.assertEqual(json.loads(out.read_text())["mode"], "select")  # an earlier run's plan
        with mock.patch.object(ci_impact.os, "replace", side_effect=OSError("blocked")):
            with contextlib.redirect_stdout(io.StringIO()) as stdout, contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(ci_impact.main(argv), 0)
        self.assertFull(json.loads(stdout.getvalue()), "output-error:OSError")
        self.assertEqual(list(out.parent.iterdir()), [])  # neither the stale plan nor a temporary file

    def test_stale_plan_is_overwritten_when_it_cannot_be_removed(self):
        out = Path(self._tmp.name) / "stale" / "plan.json"
        out.parent.mkdir()
        out.write_text('{"mode": "select"}\n')
        plan = ci_impact.minimal_full_plan("pull_request", "output-error:OSError", None, None, None)
        with mock.patch.object(ci_impact.os, "unlink", side_effect=PermissionError("read-only dir")):
            ci_impact.discard_stale_plan(str(out), plan)
        self.assertFull(json.loads(out.read_text()), "output-error:OSError")
        real_open = os.open
        with mock.patch.object(ci_impact.os, "unlink", side_effect=PermissionError("read-only dir")), \
                mock.patch.object(ci_impact.os, "open", side_effect=lambda *a, **k: (_ for _ in ()).throw(
                    PermissionError("read-only file")) if a and str(a[0]) == str(out) else real_open(*a, **k)), \
                contextlib.redirect_stderr(io.StringIO()) as stderr:
            ci_impact.discard_stale_plan(str(out), plan)
        self.assertIn("ci-impact: stale-plan-not-removed:", stderr.getvalue())

    def test_stale_plan_overwrite_never_follows_a_symlink(self):
        victim = Path(self._tmp.name) / "victim.txt"
        victim.write_text("keep me\n")
        out = Path(self._tmp.name) / "plan.json"
        out.symlink_to(victim)
        plan = ci_impact.minimal_full_plan("pull_request", "output-error:OSError", None, None, None)
        with mock.patch.object(ci_impact.os, "unlink", side_effect=PermissionError("read-only dir")), \
                contextlib.redirect_stderr(io.StringIO()) as stderr:
            ci_impact.discard_stale_plan(str(out), plan)
        self.assertEqual(victim.read_text(), "keep me\n")
        self.assertTrue(out.is_symlink())
        self.assertIn("ci-impact: stale-plan-not-removed:", stderr.getvalue())

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


NET_HEADER = """path = ".github/workflows/net.yml"
owner = "ci-owner"
granularity = "job"
events = ["pull_request", "push"]
obligation = [{ cite = "README.md:1", anchor = "fixture readme" }]
"""


class NonnativeFilterTests(FixtureTestCase):
    """#494 phase 5 C2: an @nonnative filter hides core_only paths, never a registered input."""

    catalogue = CATALOGUE.replace(NET_HEADER, NET_HEADER + """\
trigger.pull_request = { paths_ignore = "@nonnative" }
trigger.push = { branches = ["main"], paths_ignore = "@metadata" }
""").replace('needs = ["broker"]\ninput_sets = ["native"]\n', 'needs = ["broker"]\ninput_sets = ["native"]\ninputs = ["LICENSE"]\n')

    def test_core_only_input_hidden_by_the_macro_is_a_hidden_input(self):
        self.assertIn('paths_ignore = "@nonnative"', self.catalogue)
        self.assertIn('inputs = ["LICENSE"]', self.catalogue)
        plan = self.plan(self.branch({"LICENSE": "changed\n", "docs/guide.md": "changed\n"}))
        self.assertEqual(plan["mode"], "select", plan["fallback"])
        rows = {row["id"]: row for row in plan["selected"]}
        # merge-hold fails this row (C1): the filter would hide a registered input.
        self.assertIs(rows["net#native"]["blocked_by_filter"], True)
        self.assertEqual(rows["net#native"]["filter_hidden_inputs"], ["LICENSE"])

    def test_code_change_is_not_hidden(self):
        plan = self.plan(self.branch({"LICENSE": "changed\n", "kernel/main.rs": "// k\n"}))
        rows = {row["id"]: row for row in plan["selected"]}
        self.assertIs(rows["net#native"]["blocked_by_filter"], False)
        self.assertEqual(rows["net#native"]["filter_hidden_inputs"], [])


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

    def test_real_catalogue_registers_the_token_guard_script_for_its_native_users(self):
        # #494 C1 (D-2): tools/tokenguard.py runs this core_only script in both workflows.
        with REAL_CATALOGUE.open("rb") as handle:
            catalogue = tomllib.load(handle)
        execs = ci_impact.executions(catalogue)
        for execution in execs:  # static inputs suffice: the script is no Cargo source
            execution["globs"] = sorted(ci_impact.execution_static_globs(execution, catalogue))
        _, hits, unknown = ci_impact.classify(
            ["plugins/token-guard/skills/token-guard/scripts/tokenguard.py"], catalogue["policy"], execs)
        self.assertEqual(unknown, [])
        workflows = {x["workflow"]["path"] for x in execs if x["id"] in hits}
        self.assertEqual(workflows, {".github/workflows/native-hid-input.yml",
                                     ".github/workflows/native-xhci.yml"})
        users = {path.name for path in (TOOLS.parent / ".github" / "workflows").glob("*.yml")
                 if "tools/tokenguard.py" in path.read_text(encoding="utf-8")} - {"ci.yml"}
        self.assertEqual(users, {"native-hid-input.yml", "native-xhci.yml"})
        # #494 phase 5 C2 (N7): both start on it (@metadata); the 13 @nonnative workflows skip it.
        changed = ["plugins/token-guard/skills/token-guard/scripts/tokenguard.py"]
        planner = ci_impact.Planner(str(TOOLS.parent), "pull_request", "", "", None, ci_impact.CATALOGUE_PATH,
                                    None, False)
        self.assertEqual({x["workflow"]["path"]: planner.blocked(x, changed, catalogue["policy"])
                          for x in execs if x["id"] in hits},
                         {".github/workflows/native-hid-input.yml": False, ".github/workflows/native-xhci.yml": False})
        nonnative = [x for x in execs if x["workflow"].get("trigger", {}).get("pull_request", {})
                     .get("paths_ignore") == "@nonnative"]
        self.assertTrue(nonnative)
        self.assertTrue(all(planner.blocked(x, changed, catalogue["policy"]) for x in nonnative))

    def test_nonnative_needs_a_pull_request_trigger_with_a_main_metadata_backstop(self):
        # #494 phase 5 C2 (N6).
        backstop = {"branches": ["main"], "paths_ignore": "@metadata"}

        def leaves(**trigger):
            catalogue = valid_catalogue()
            catalogue["workflow"][1]["trigger"] = trigger
            return catalogue

        self.assertEqual(ci_impact.validate(leaves(pull_request={"paths_ignore": "@nonnative"}, push=backstop)), [])
        rejected = "@nonnative only on pull_request, with a push trigger on main filtered by @metadata"
        for trigger in ({"pull_request": {"paths_ignore": "@nonnative"}},
                        {"pull_request": {"paths_ignore": "@metadata"},
                         "push": {"branches": ["main"], "paths_ignore": "@nonnative"}},
                        {"pull_request": {"paths_ignore": "@nonnative"}, "push": {"branches": ["main"]}},
                        {"pull_request": {"paths_ignore": "@nonnative"},
                         "push": {"branches": ["main"], "paths_ignore": ["docs/**"]}},
                        {"pull_request": {"paths_ignore": "@nonnative"},
                         "push": {"branches": ["release"], "paths_ignore": "@metadata"}}):
            with self.subTest(trigger=trigger):
                self.assertProblem(leaves(**trigger), rejected)
        catalogue = leaves(pull_request={"paths_ignore": "@nonnative"}, push=backstop)
        catalogue["workflow"][1]["events"] = ["pull_request", "workflow_dispatch"]
        self.assertProblem(catalogue, rejected)
        catalogue = leaves(merge_group={"paths_ignore": "@nonnative"}, push=backstop)
        catalogue["workflow"][1]["events"].append("merge_group")
        self.assertEqual([problem for problem in ci_impact.validate(catalogue) if "@nonnative" in problem],
                         ["workflow '.github/workflows/leaves.yml': trigger.merge_group: " + rejected])
        # The macro must not hide a central path: core_only may not name or contain one.
        for pattern in ("tools/build.py", ".github/workflows/**", "**.toml", "tools/**", "Cargo.lock"):
            catalogue = leaves(pull_request={"paths_ignore": "@nonnative"}, push=backstop)
            catalogue["policy"]["core_only"].append(pattern)
            with self.subTest(core_only=pattern):
                self.assertProblem(catalogue, "policy.core_only overlaps always_full, which @nonnative would hide")

    def test_real_central_changes_start_every_native_workflow_with_a_paths_ignore_filter(self):
        # #494 phase 5 C2 (N3): catalogue, selector, merge-hold, candidate and workflow changes
        # give a full plan that no @metadata or @nonnative filter hides.
        with REAL_CATALOGUE.open("rb") as handle:
            catalogue = tomllib.load(handle)
        policy = catalogue["policy"]
        planner = ci_impact.Planner(str(TOOLS.parent), "pull_request", "", "", None, ci_impact.CATALOGUE_PATH,
                                    None, False)
        native = [x for x in ci_impact.executions(catalogue) if "native" in x["job"].get("input_sets", [])]
        allowlists = {x["workflow"]["path"] for x in native
                      if "paths" in x["workflow"].get("trigger", {}).get("pull_request", {})}
        # The documented C1 exception: allowlists without a push backstop (docs/CI-CONTRACT.md).
        self.assertEqual(allowlists, {".github/workflows/storage-resilience.yml",
                                      ".github/workflows/volume-acceptance.yml"})
        self.assertEqual(len({x["workflow"]["path"] for x in native
                              if x["workflow"]["trigger"]["pull_request"].get("paths_ignore") == "@nonnative"}), 13)
        for path in (".github/ci-impact.toml", "tools/ci_impact.py", "tools/merge_hold.py",
                     ".github/ci-impact.candidate.toml", ".github/workflows/native-rtc.yml"):
            with self.subTest(path=path):
                self.assertTrue(ci_impact.first_match(policy["always_full"], path))
                self.assertIsNone(ci_impact.first_match(policy["core_only"], path))
                blocked = {x["workflow"]["path"] for x in native if planner.blocked(x, [path], policy)}
                self.assertLessEqual(blocked, allowlists)
        # validate's literal check misses a partial overlap such as plugins/**/Cargo.lock, so no
        # tracked path may be both central and core_only: @nonnative then never hides one.
        tracked = subprocess.run(["git", "-C", str(TOOLS.parent), "ls-files", "-z"], capture_output=True,
                                 env=GIT_ENV, check=True).stdout.decode("utf-8").split("\0")
        self.assertGreater(len(tracked), 1000)
        self.assertEqual([path for path in tracked if path and ci_impact.first_match(policy["always_full"], path)
                          and ci_impact.first_match(policy["core_only"], path)], [])

    def test_real_allowlist_filters_cover_their_native_inputs(self):
        # #494 C1 (D-1): an allowlist may not hide a registered input of its own gates.
        with REAL_CATALOGUE.open("rb") as handle:
            catalogue = tomllib.load(handle)
        native = catalogue["input_sets"]["native"]
        checked = 0
        for workflow in catalogue["workflow"]:
            paths = workflow.get("trigger", {}).get("pull_request", {}).get("paths")
            if paths is None or not any("native" in job.get("input_sets", []) for job in workflow["job"]):
                continue
            checked += 1
            with self.subTest(workflow=workflow["path"]):
                self.assertEqual([glob for glob in native if glob not in paths], [])
        self.assertEqual(checked, 2)

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

[policy.budgets]
pull_request_wait_minutes = 0
merge_group_wait_minutes = 45

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

    def test_nonnative_filter_is_compared_in_expansion_order(self):
        catalogue = tomllib.loads(COVERAGE_CATALOGUE.replace("core_only = []", 'core_only = ["LICENSE"]').replace(
            'trigger.pull_request = { types = ["opened"] }',
            'trigger.pull_request = { types = ["opened"], paths_ignore = "@nonnative" }'))
        self.assertEqual(ci_impact.validate(catalogue), [])
        for ignore, expected in ((["docs/**", "**.md", "LICENSE"], []),
                                 (["docs/**", "**.md"], ["trigger-mismatch"]),
                                 (["LICENSE", "docs/**", "**.md"], ["trigger-mismatch"])):
            workflows = copy.deepcopy(COVERAGE_WORKFLOWS)
            workflows[".github/workflows/ci.yml"]["on"]["pull_request"]["paths-ignore"] = ignore
            with self.subTest(ignore=ignore):
                problems = ci_impact.workflow_problems(workflows, catalogue)
                self.assertEqual([p.split(":")[0] for p in problems], expected, problems)

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

    def test_whole_matrix_expression_only_for_a_dynamic_variant(self):
        expression = "${{ fromJSON(needs.prepare.outputs.matrix) }}"
        old = """variant = [
  { id = "1", check_name = "pair (1)", matrix = { scale = 1 } },
  { id = "2", check_name = "pair (2)", matrix = { scale = 2 } },
]"""
        catalogue = tomllib.loads(COVERAGE_CATALOGUE.replace(old, (
            'variant = [{ id = "*", dynamic = true, check_name = "pair (*)", matrix = "%s" }]' % expression)))
        self.assertEqual(ci_impact.validate(catalogue), [])
        workflows = copy.deepcopy(COVERAGE_WORKFLOWS)
        workflows[".github/workflows/ci.yml"]["jobs"]["pair"] = {"strategy": {"matrix": expression}}
        self.assertEqual(ci_impact.workflow_problems(workflows, catalogue), [])
        workflows[".github/workflows/ci.yml"]["jobs"]["pair"]["strategy"]["matrix"] = "${{ fromJSON(inputs.x) }}"
        self.assertEqual(ci_impact.workflow_problems(workflows, catalogue),
                         ["matrix-mismatch: .github/workflows/ci.yml#pair/*"])
        static = tomllib.loads(COVERAGE_CATALOGUE.replace("matrix = { scale = 1 }", 'matrix = "%s"' % expression))
        self.assertIn("ci#pair/1: matrix must be a table (an expression string only when dynamic = true)",
                      ci_impact.validate(static))

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

    @skipUnless(HAVE_YAML, "PyYAML is not importable")
    def test_duplicate_workflow_keys_are_drift(self):
        # The last value matches the catalogue, so a last-wins parse would report clean.
        duplicated = COVERAGE_YAML.replace("    timeout-minutes: 30\n",
                                           "    timeout-minutes: 99\n    timeout-minutes: 30\n")
        with tempfile.TemporaryDirectory() as tmp:
            Repo(tmp).commit("base", {".github/ci-impact.toml": COVERAGE_CATALOGUE,
                                      ".github/workflows/ci.yml": duplicated, "README.md": "fixture readme\n",
                                      "src/a/lib.rs": "// a\n"})
            report = ci_impact.coverage(tmp, ".github/ci-impact.toml")
        self.assertEqual(report["status"], "drift", report)
        self.assertEqual(report["problems"], [
            "workflow-duplicate-key: .github/workflows/ci.yml: duplicate key 'timeout-minutes' at line 11"])

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
            tracked = ["app/src/lib.rs", "shared/x.rs"]
            found = {(crate, target) for crate, target, *_ in ci_impact.scan_escapes(tmp, {"app"}, tracked)}
        self.assertEqual(found, {("app", "shared/x.rs"), ("app", "data/a.bin"), ("app", "fix/**"),
                                 ("app", "tokens.css")})

    @skipUnless(HAVE_YAML, "PyYAML is not importable")
    def test_escape_scanner_ignores_untracked_generated_sources(self):
        catalogue = COVERAGE_CATALOGUE.replace('inputs = ["src/a/**"] }',
                                               'inputs = ["src/a/**"], cargo = ["src/a/Cargo.toml"] }')
        escape = "escape-unregistered: src/a -> outside.rs (src/a/target/gen.rs:1)"
        with tempfile.TemporaryDirectory() as tmp:
            repo = Repo(tmp)
            repo.commit("base", {".github/ci-impact.toml": catalogue, ".github/workflows/ci.yml": COVERAGE_YAML,
                                 "README.md": "fixture readme\n", "src/a/Cargo.toml": '[package]\nname = "a"\n',
                                 "src/a/lib.rs": "// a\n"})
            repo.write("src/a/target/gen.rs", '#[path = "../../../outside.rs"]\nmod x;\n')  # untracked build output
            report = ci_impact.coverage(tmp, ".github/ci-impact.toml")
            self.assertEqual(report["problems"], [], report)
            repo.git("add", "-f", "src/a/target/gen.rs")
            repo.git("commit", "-q", "-m", "track")
            self.assertIn(escape, ci_impact.coverage(tmp, ".github/ci-impact.toml")["problems"])


class EnforcementPrerequisiteTests(FixtureTestCase):
    """Phase 3: limits K4-K7 of docs/CI-CONTRACT.md, closed before merge-hold enforces a plan."""

    def rebase_catalogue(self, text):
        self.repo.git("checkout", "-q", "main")
        self.base = self.repo.commit("catalogue", {".github/ci-impact.toml": text})

    def test_condition_subset_is_evaluated_and_everything_else_is_undecidable(self):
        smp = ("${{ !inputs.clock_diagnostic && !inputs.margin_diagnostic && "
               "(github.event_name != 'workflow_dispatch' || !inputs.full_matrix) }}")
        select = "${{ needs.select.outputs.run-quality == 'true' }}"
        cases = [
            ("", "pull_request", True, True),
            (None, "push", True, True),
            ("${{ github.event_name != 'workflow_dispatch' }}", "pull_request", True, True),
            ("${{ github.event_name != 'workflow_dispatch' }}", "workflow_dispatch", True, False),
            ("github.event_name == 'PUSH'", "push", True, True),
            ("github.event_name == 'push'", "merge_group", True, False),
            (smp, "pull_request", True, True),
            (smp, "merge_group", True, True),
            (smp, "workflow_dispatch", True, None),
            (select, "pull_request", True, True),
            (select, "pull_request", False, False),
            (select, "pull_request", None, None),
            ("${{ always() }}", "pull_request", True, None),
            ("!cancelled()", "push", True, None),
            ("contains(github.event.pull_request.labels.*.name, 'x')", "pull_request", True, None),
            ("github.event.pull_request.head.repo.full_name == github.repository", "pull_request", True, None),
            ("inputs.mode == 'x'", "pull_request", True, None),
            ("(github.event_name == 'push'", "push", True, None),
            ("github.event_name == 'push' ${{ x }}", "push", True, None),
            ("github.event_name ==", "push", True, None),
        ]
        for condition, event, selection, expected in cases:
            with self.subTest(condition=condition, event=event, selection=selection):
                self.assertIs(ci_impact.condition_value(condition, event, selection), expected)

    def test_condition_false_jobs_and_their_dependents_are_excluded(self):
        self.rebase_catalogue(CATALOGUE.replace(
            'id = "a"\nclass = "rwa"', 'id = "a"\ncondition = "${{ github.event_name == \'push\' }}"\nclass = "rwa"'))
        head = self.branch({"stray.txt": "x\n"})
        plan = self.plan(head)
        self.assertFull(plan, "unknown-path")
        reasons = {row["id"]: row["reason"] for row in plan["excluded"]}
        self.assertEqual(reasons["window#a"], "condition-false")
        self.assertEqual(reasons["window#c"], "condition-false")  # needs a, which GitHub skips
        self.assertIn("window#b", self.selected(plan))
        push = self.plan(head, event="push")
        self.assertLessEqual({"window#a", "window#c"}, self.selected(push))
        # In select mode a job whose condition is false is not selected either.
        select = self.plan(self.branch({"ui/x.css": "x\n"}))
        self.assertEqual(select["mode"], "select")
        self.assertNotIn("window#a", self.selected(select))

    def test_undecidable_conditions_of_selected_jobs_are_rejected(self):
        catalogue = valid_catalogue()
        catalogue["workflow"][3]["job"][0]["condition"] = "${{ always() }}"
        problems = ci_impact.validate(catalogue)
        self.assertTrue(any("window#a: condition-undecidable on pull_request" in p for p in problems), problems)
        # A dependent inherits an undecidable prerequisite.
        self.assertTrue(any("window#c: condition-undecidable" in p for p in problems), problems)
        catalogue = valid_catalogue()
        catalogue["workflow"][5]["job"][2]["condition"] = "${{ always() }}"  # advisory: never selected
        self.assertEqual(ci_impact.validate(catalogue), [])
        # Review S1: a status function runs the job after a skipped prerequisite, so a false
        # prerequisite must not turn it into condition-false; it stays undecidable.
        catalogue = valid_catalogue()
        catalogue["workflow"][2]["events"] = ["pull_request"]
        catalogue["workflow"][2]["job"][0]["condition"] = "${{ github.event_name == 'push' }}"
        catalogue["workflow"][2]["job"][1]["condition"] = "${{ always() }}"
        problems = ci_impact.validate(catalogue)
        self.assertTrue(any("net#native: condition-undecidable on pull_request" in p for p in problems), problems)
        jobs = {job["id"]: job for job in catalogue["workflow"][2]["job"]}
        self.assertIsNone(ci_impact.job_runs(jobs, "native", "pull_request"))

    def test_dynamic_gate_check_names_need_literal_text(self):
        """Review N2: a bare template would match every job of the run in merge-hold."""
        catalogue = valid_catalogue()
        catalogue["workflow"][1]["job"][0]["variant"] = [
            {"id": "*", "dynamic": True, "check_name": "${{ matrix.component }}", "matrix": "${{ fromJSON(x) }}",
             "cargo": ["apps/viewer/Cargo.toml"]}]
        self.assertTrue(any("needs literal text" in p for p in ci_impact.validate(catalogue)))
        catalogue["workflow"][1]["job"][0]["variant"][0]["check_name"] = "quality (${{ matrix.component }})"
        self.assertEqual(ci_impact.validate(catalogue), [])

    def test_branch_filters_follow_the_target_branch(self):
        self.rebase_catalogue(CATALOGUE.replace(
            'path = ".github/workflows/net.yml"\nowner = "ci-owner"\ngranularity = "job"\n'
            'events = ["pull_request", "push"]\n',
            'path = ".github/workflows/net.yml"\nowner = "ci-owner"\ngranularity = "job"\n'
            'events = ["pull_request", "push"]\ntrigger.pull_request = { branches = ["main", "release/**"] }\n'
            'trigger.push = { branches = ["main"] }\n'))
        head = self.branch({"stray.txt": "x\n"})
        plans = {ref: self.plan(head, base_ref=ref)
                 for ref in ("", "main", "refs/heads/main", "release/1.0", "feature/x")}
        for ref in ("", "main", "refs/heads/main", "release/1.0"):
            with self.subTest(ref=ref):
                self.assertLessEqual({"net#broker", "net#native"}, self.selected(plans[ref]))
        reasons = {row["id"]: row["reason"] for row in plans["feature/x"]["excluded"]}
        self.assertEqual(reasons["net#broker"], "event-not-applicable")
        self.assertEqual(plans["feature/x"]["ref"], "feature/x")
        self.assertNotEqual(plans["main"]["plan_id"], plans["feature/x"]["plan_id"])
        push = self.plan(head, event="push", base=self.base, base_ref="topic")
        self.assertNotIn("net#broker", self.selected(push))
        self.assertIn("net#broker", self.selected(self.plan(head, event="push", base=self.base)))

    def test_base_ref_reaches_the_plan_through_the_cli(self):
        head = self.branch({"stray.txt": "x\n"})
        out = Path(self._tmp.name, "plan.json")
        with contextlib.redirect_stderr(io.StringIO()):
            code = ci_impact.main(["plan", "--event=pull_request", f"--base-sha={self.base}", f"--head-sha={head}",
                                   f"--repo={self.repo.root}", "--base-ref=main", "--no-fetch", f"--out={out}"])
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out.read_text())["ref"], "main")

    def test_branch_patterns_that_are_not_modelled_are_rejected(self):
        for pattern in ("!main", "rel?ase", "[ab]", ""):
            catalogue = valid_catalogue()
            catalogue["workflow"][2]["trigger"] = {"pull_request": {"branches": [pattern]}}
            with self.subTest(pattern=pattern):
                self.assertTrue(any("unsupported branch pattern" in p for p in ci_impact.validate(catalogue)))

    def test_budgets_are_required_and_bounded(self):
        for change, fragment in (
            (lambda b: b.clear(), "policy.budgets.pull_request_wait_minutes"),
            (lambda b: b.update(merge_group_wait_minutes=46), "merge_group_wait_minutes must be"),
            (lambda b: b.update(pull_request_wait_minutes=True), "pull_request_wait_minutes must be"),
            (lambda b: b.update(pull_request_wait_minutes=-1), "pull_request_wait_minutes must be"),
            (lambda b: b.update(extra=1), "policy.budgets: unknown key: extra"),
        ):
            catalogue = valid_catalogue()
            change(catalogue["policy"]["budgets"])
            with self.subTest(fragment=fragment):
                self.assertTrue(any(fragment in p for p in ci_impact.validate(catalogue)))
        catalogue = valid_catalogue()
        del catalogue["policy"]["budgets"]
        self.assertIn("policy.budgets table missing", ci_impact.validate(catalogue))

    def test_trusted_plans_carry_budgets_and_excluded_check_names(self):
        plan = self.plan(self.branch({"apps/solo/src/lib.rs": "// solo 2\n"}))
        self.assertEqual(plan["budgets"], {"pull_request_wait_minutes": 0, "merge_group_wait_minutes": 45})
        excluded = {row["id"]: row for row in plan["excluded"]}
        self.assertEqual(excluded["leaves#quality/viewer"]["check_name"], "quality (viewer)")
        self.assertEqual(excluded["leaves#quality/viewer"]["workflow"], ".github/workflows/leaves.yml")
        self.assertEqual(excluded["leaves#quality/viewer"]["job"], "quality")
        untrusted = self.plan(self.base, base="", event="workflow_dispatch")
        self.assertEqual(untrusted["budgets"], {})


class EscapeScopeTests(TestCase):
    """K5 (runtime escapes) and K6 (root-scoped library escapes)."""

    def scan(self, files, crates=("app",), manifests=None):
        with tempfile.TemporaryDirectory() as tmp:
            for path, text in files.items():
                Path(tmp, path).parent.mkdir(parents=True, exist_ok=True)
                Path(tmp, path).write_text(text)

            def read(path):
                full = Path(tmp, path)
                return full.read_bytes() if full.exists() else None

            roots = ci_impact.root_target_paths(read, set(crates))
            external = ci_impact.external_root_targets(read, set(crates))
            return ci_impact.scan_escapes(tmp, set(crates), sorted(files), roots, external)

    def test_declared_targets_outside_the_crate_are_scanned_for_it(self):
        """CodeRabbit on #504: a bin outside its crate, and the `mod` files beside it, are scanned."""
        found = self.scan({
            "app/Cargo.toml": '[package]\nname = "app"\n[[bin]]\nname = "x"\npath = "../ext/src/main.rs"\n',
            "app/src/lib.rs": "// lib\n",
            "ext/src/main.rs": "mod m;\n",
            "ext/src/m.rs": 'const A: &str = include_str!("../../data/a.txt");\n'
                            'const B: &str = include_str!("near.txt");\n',
            "other/src/lib.rs": 'const C: &str = include_str!("../../data/c.txt");\n'})
        self.assertEqual(found, [("app", "data/a.txt", "ext/src/m.rs:1", True)])
        # Library code that reaches into the external directory stays reported, at library scope:
        # only root gates take that directory as an input (CodeRabbit on #510).
        found = self.scan({
            "app/Cargo.toml": '[package]\nname = "app"\n[[bin]]\nname = "x"\npath = "../ext/src/main.rs"\n',
            "app/src/lib.rs": '#[path = "../../ext/src/helper.rs"]\nmod helper;\n',
            "ext/src/main.rs": "fn main() {}\n",
            "ext/src/helper.rs": "// helper\n"})
        self.assertEqual(found, [("app", "ext/src/helper.rs", "app/src/lib.rs:1", False)])
        # A target at the repository root: its sibling root files are scanned, nothing deeper.
        found = self.scan({
            "app/Cargo.toml": '[package]\nname = "app"\n[[bin]]\nname = "x"\npath = "../main.rs"\n',
            "app/src/lib.rs": "// lib\n",
            "main.rs": 'mod helper;\nconst S: &str = include_str!("settings.txt");\n',
            "helper.rs": 'const A: &str = include_str!("data/a.txt");\n',
            "settings.txt": "x\n",
            "deep/other.rs": 'const B: &str = include_str!("../data/b.txt");\n'})
        # Only root-level `*.rs` is an input of such a target: other root files stay reported.
        self.assertEqual(found, [("app", "data/a.txt", "helper.rs:1", True),
                                 ("app", "settings.txt", "main.rs:2", True)])

    def test_external_target_keeps_the_declaring_crate_for_escapes(self):
        """CodeRabbit on #510: an external target must not rebind the crate directory."""
        files = {"app/Cargo.toml": b'[package]\nname = "app"\n[[bin]]\nname = "x"\npath = "../ext/main.rs"\n'}
        escapes = [{"crate": "app", "when": "always", "inputs": ["data/**"], "evidence": []}]
        globs, dirs = ci_impact.cargo_closure(["app/Cargo.toml"], files.get, escapes)
        self.assertIn("data/**", globs)
        self.assertIn("ext/**", globs)
        self.assertIn("ext/main.rs", globs)
        self.assertIn("app", dirs)
        self.assertEqual(ci_impact.external_root_targets(
            lambda path: b'[[bin]]\nname = "x"\npath = "../ext/src/main.rs"\n' if path == "app/Cargo.toml" else None,
            {"app"}), {"app": ["ext/src"]})

    def test_runtime_manifest_dir_joins_are_found(self):
        found = self.scan({
            "app/Cargo.toml": '[package]\nname = "app"\n\n[[test]]\nname = "c"\npath = "checks/c.rs"\n',
            "app/src/lib.rs": 'fn a() -> String {\n    format!("{}/../data/x.bin", env!("CARGO_MANIFEST_DIR"))\n}\n'
                              'const B: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fix/y"));\n'
                              'include!(concat!(env!("OUT_DIR"), "/gen.rs"));\n'
                              'const C: &str = include_str!(env!("SOME_PATH"));\n'
                              'fn d() { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/own"); }\n',
            "app/tests/t.rs": 'fn k() {\n    let keys = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../keys");\n}\n',
            "app/checks/c.rs": 'fn k() { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../probe/p.py"); }\n',
        })
        rows = {(target, origin, root_only) for _crate, target, origin, root_only in found}
        self.assertEqual(rows, {
            ("data/x.bin", "app/src/lib.rs:2", False),
            ("fix/y", "app/src/lib.rs:4", False),
            (None, "app/src/lib.rs:6", False),
            ("keys", "app/tests/t.rs:2", True),
            ("probe/p.py", "app/checks/c.rs:1", True),
        })

    def test_library_reachable_helpers_and_runtime_included_sources_are_followed(self):
        """CodeRabbit on #504: scope follows intra-crate links; runtime-included .rs is scanned."""
        found = self.scan({
            "app/src/lib.rs": '#[path = "../tests/helper.rs"]\nmod helper;\n'
                              'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../shared/x.rs"));\n',
            "app/tests/helper.rs": 'const T: &str = include_str!("../../data/t.txt");\n',
            "shared/x.rs": 'const U: &str = include_str!("tokens.css");\n',
        })
        rows = {(target, origin, root_only) for _crate, target, origin, root_only in found}
        self.assertIn(("data/t.txt", "app/tests/helper.rs:1", False), rows)  # library scope wins
        self.assertIn(("data/t.txt", "app/tests/helper.rs:1", True), rows)
        self.assertIn(("shared/x.rs", "app/src/lib.rs:3", False), rows)
        self.assertIn(("shared/tokens.css", "shared/x.rs:1", False), rows)
        root = [{"crate": "app", "when": "root", "inputs": ["data/**"], "evidence": []}]
        self.assertTrue(ci_impact.escape_problem(root, "app", "data/t.txt", "app/tests/helper.rs:1", False)
                        .startswith("escape-scope-mismatch"))

    def test_evidence_registers_only_an_unresolvable_target(self):
        cited = [{"crate": "app", "when": "always", "inputs": ["ui/old.css"], "evidence": ["app/src/lib.rs:1"]}]
        self.assertTrue(ci_impact.escape_problem(cited, "app", "ui/new.css", "app/src/lib.rs:1", False)
                        .startswith("escape-unregistered"))
        self.assertIsNone(ci_impact.escape_problem(cited, "app", None, "app/src/lib.rs:1", False))

    def test_build_output_is_not_an_input(self):
        found = self.scan({"a/b/tests/t.rs": 'fn o() { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../build/x"); }\n'},
                          crates=("a/b",))
        self.assertEqual(found, [])

    def test_root_scope_is_rejected_for_library_source(self):
        root = [{"crate": "app", "when": "root", "inputs": ["data/**"], "evidence": []}]
        always = [{"crate": "app", "when": "always", "inputs": ["data/**"], "evidence": []}]
        problem = ci_impact.escape_problem
        self.assertTrue(problem(root, "app", "data/x.bin", "app/src/lib.rs:2", False)
                        .startswith("escape-scope-mismatch: app -> data/x.bin"))
        self.assertIsNone(problem(root, "app", "data/x.bin", "app/tests/t.rs:2", True))
        self.assertIsNone(problem(always, "app", "data/x.bin", "app/src/lib.rs:2", False))
        self.assertTrue(problem([], "app", "data/x.bin", "app/src/lib.rs:2", False)
                        .startswith("escape-unregistered"))
        directory = [{"crate": "app", "when": "always", "inputs": ["keys/**"], "evidence": []}]
        self.assertIsNone(problem(directory, "app", "keys", "app/tests/t.rs:2", True, descendants=["keys/a", "keys/b"]))
        # Every tracked file below a directory target must be covered (CodeRabbit on #504).
        partial = [{"crate": "app", "when": "always", "inputs": ["keys/a"], "evidence": []}]
        self.assertTrue(problem(partial, "app", "keys", "app/tests/t.rs:2", True, descendants=["keys/a", "keys/b"])
                        .startswith("escape-unregistered"))
        # The directory's own name covers none of its files (CodeRabbit on #510).
        bare = [{"crate": "app", "when": "always", "inputs": ["keys"], "evidence": []}]
        self.assertTrue(problem(bare, "app", "keys", "app/tests/t.rs:2", True, descendants=["keys/a"])
                        .startswith("escape-unregistered"))
        self.assertIsNone(problem(bare, "app", "keys", "app/tests/t.rs:2", True))  # a file target
        by_type = [{"crate": "app", "when": "always", "inputs": ["keys/*.bin"], "evidence": []}]
        self.assertIsNone(problem(by_type, "app", "keys", "app/tests/t.rs:2", True, descendants=["keys/a.bin"]))
        # A one-level glob leaves a deeper tracked file uncovered.
        shallow = [{"crate": "app", "when": "always", "inputs": ["keys/*"], "evidence": []}]
        self.assertTrue(problem(shallow, "app", "keys", "app/tests/t.rs:2", True, descendants=["keys/a", "keys/s/b"])
                        .startswith("escape-unregistered"))
        # Without tracked descendants the target is a file: a glob over its would-be
        # contents never covers the file itself.
        self.assertTrue(problem(directory, "app", "keys", "app/tests/t.rs:2", True)
                        .startswith("escape-unregistered"))
        child_only = [{"crate": "app", "when": "always", "inputs": ["data/a.bin/**"], "evidence": []}]
        self.assertTrue(problem(child_only, "app", "data/a.bin", "app/src/lib.rs:2", False)
                        .startswith("escape-unregistered"))
        cited = [{"crate": "app", "when": "always", "inputs": ["x/**"], "evidence": ["app/src/lib.rs:6"]}]
        self.assertIsNone(problem(cited, "app", None, "app/src/lib.rs:6", False))
        self.assertTrue(problem(cited, "app", None, "app/src/lib.rs:7", False).startswith("escape-unregistered"))

    @skipUnless(HAVE_YAML, "PyYAML is not importable")
    def test_strict_coverage_exit_status(self):
        catalogue = COVERAGE_CATALOGUE.replace('inputs = ["src/a/**"] }',
                                               'inputs = ["src/a/**"], cargo = ["src/a/Cargo.toml"] }')
        with tempfile.TemporaryDirectory() as tmp:
            repo = Repo(tmp)
            repo.commit("base", {".github/ci-impact.toml": catalogue, ".github/workflows/ci.yml": COVERAGE_YAML,
                                 "README.md": "fixture readme\n", "src/a/Cargo.toml": '[package]\nname = "a"\n',
                                 "src/a/lib.rs": "// a\n"})
            args = ["coverage", "--repo", tmp, "--strict"]
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(ci_impact.main(args), 0)
                # Citation drift is reported but never fails --strict: citations never change a plan.
                repo.commit("shift", {"README.md": "\n" * 20 + "fixture readme\n"})
                self.assertEqual(ci_impact.main(args), 0)
                repo.commit("escape", {"src/a/lib.rs": 'const X: &str = include_str!("../../outside.txt");\n'})
                self.assertEqual(ci_impact.main(args), 1)
                self.assertEqual(ci_impact.main(args[:-1]), 0)  # without --strict it stays a report



class SelectOutputTests(TestCase):
    """Phase 4: the select job narrows only for a trusted, select-mode pull_request plan."""

    LEAVES = ".github/workflows/leaves.yml"

    def catalogue(self):
        catalogue = valid_catalogue()
        for variant in catalogue["workflow"][1]["job"][0]["variant"]:
            variant["matrix"] = {"component": variant["id"]}
        return catalogue

    def plan(self, selected=("leaves#quality/viewer",), **over):
        ids = [x["id"] for x in ci_impact.executions(self.catalogue())]
        plan = {"event": "pull_request", "ref": "main", "mode": "select", "policy_mode": "select",
                "selector": {"trusted": True}, "shas": {"tested": "a" * 40, "base": "c" * 40},
                "catalogue": {"source_sha": "c" * 40}, "plan_id": "sha256:x",
                "selected": [{"id": xid} for xid in selected],
                "excluded": [{"id": xid} for xid in ids if xid not in selected]}
        plan.update(over)
        return plan

    def outputs(self, plan, catalogue=None, workflow=None, mode="select", base_ref="main"):
        return ci_impact.select_outputs(plan, catalogue or self.catalogue(), workflow or self.LEAVES, "a" * 40,
                                        mode, base_ref)

    def test_trusted_select_plan_narrows_to_the_selected_variants(self):
        out = self.outputs(self.plan())
        self.assertEqual(out["narrowed"], "true")
        self.assertEqual(out["run-quality"], "true")
        self.assertEqual(json.loads(out["matrix-quality"]), {"include": [{"component": "viewer"}]})

    def test_nothing_selected_skips_the_job_with_a_non_empty_matrix(self):
        out = self.outputs(self.plan(selected=()))
        self.assertEqual(out["run-quality"], "false")
        self.assertEqual(len(json.loads(out["matrix-quality"])["include"]), 6)

    def test_every_other_plan_runs_everything(self):
        for over in ({"policy_mode": "enforce"}, {"policy_mode": "full"}, {"mode": "full"}, {"event": "push"},
                     {"selector": {"trusted": False}}, {"shas": {"tested": "b" * 40}}, {"event": "merge_group"}):
            with self.subTest(over=over):
                out = self.outputs(self.plan(selected=(), **over))
                self.assertEqual(out["narrowed"], "false")
                self.assertEqual(out["run-quality"], "true")
                self.assertEqual(len(json.loads(out["matrix-quality"])["include"]), 6)
        # The plan must also agree with select-ci's mode output, the target branch and its base catalogue.
        cases = [({}, {"mode": "full"}), ({}, {"mode": ""}), ({}, {"base_ref": "release"}), ({}, {"base_ref": ""}),
                 ({"ref": "release"}, {}), ({"catalogue": {"source_sha": "d" * 40}}, {}),
                 ({"catalogue": None}, {}), ({"shas": {"tested": "a" * 40, "base": None}}, {})]
        for over, call in cases:
            with self.subTest(over=over, call=call):
                out = self.outputs(self.plan(selected=(), **over), **call)
                self.assertEqual((out["narrowed"], out["run-quality"]), ("false", "true"))
        # Malformed nested data from any selector never narrows and never fails the job (CodeRabbit on #510).
        for over in ({"selector": ["invalid"]}, {"selected": [["x"]]}, {"selected": "x"},
                     {"excluded": [{"id": 3}]}, {"excluded": None}):
            with self.subTest(over=over):
                plan = self.plan(selected=())
                plan.update(over)
                out = self.outputs(plan)
                self.assertEqual((out["narrowed"], out["run-quality"]), ("false", "true"))
        for garbage in ([], "plan", None, {}):
            with self.subTest(plan=garbage):
                out = self.outputs(garbage)
                self.assertEqual((out["narrowed"], out["run-quality"]), ("false", "true"))

    def test_variants_and_jobs_unknown_to_the_base_plan_always_run(self):
        catalogue = self.catalogue()
        catalogue["workflow"][1]["job"][0]["variant"].append(
            {"id": "fresh", "check_name": "quality (fresh)", "matrix": {"component": "fresh"},
             "cargo": ["apps/fresh/Cargo.toml"]})
        plan = self.plan()
        out = self.outputs(plan, catalogue)
        self.assertEqual(json.loads(out["matrix-quality"]),
                         {"include": [{"component": "viewer"}, {"component": "fresh"}]})
        net = self.outputs(self.plan(selected=()), workflow=".github/workflows/net.yml")
        self.assertEqual((net["run-broker"], net["run-native"]), ("false", "false"))
        unknown = self.plan(selected=())
        unknown["excluded"] = [row for row in unknown["excluded"] if row["id"] != "net#native"]
        net = self.outputs(unknown, workflow=".github/workflows/net.yml")
        self.assertEqual((net["run-broker"], net["run-native"]), ("false", "true"))

    def test_invalid_catalogue_or_unknown_workflow_fails_the_select_job(self):
        catalogue = self.catalogue()
        catalogue["schema_version"] = 2
        with self.assertRaises(ValueError):
            self.outputs(self.plan(), catalogue)
        with self.assertRaises(ValueError):
            self.outputs(self.plan(), workflow=".github/workflows/ghost.yml")

    def test_cli_prints_outputs_and_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, ".github").mkdir()
            Path(tmp, ".github/ci-impact.toml").write_text(re.sub(
                r'\{ id = "(\w+)", check_name = "quality \(\1\)", ',
                r'{ id = "\1", check_name = "quality (\1)", matrix = { component = "\1" }, ', CATALOGUE))
            Path(tmp, "plan.json").write_text(json.dumps(self.plan()))
            out, err = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                code = ci_impact.main(["jobs", f"--plan={tmp}/plan.json", f"--workflow={self.LEAVES}",
                                       f"--tested-sha={'a' * 40}", f"--repo={tmp}"])
            self.assertEqual(code, 0, err.getvalue())
            self.assertIn("run-quality=true", out.getvalue().splitlines())
            # Without --mode=select nothing narrows.
            self.assertIn("narrowed=false", out.getvalue().splitlines())
            out = io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
                code = ci_impact.main(["jobs", f"--plan={tmp}/plan.json", f"--workflow={self.LEAVES}",
                                       f"--tested-sha={'a' * 40}", f"--repo={tmp}", "--mode=select",
                                       "--base-ref=main"])
            self.assertEqual(code, 0)
            self.assertIn("narrowed=true", out.getvalue().splitlines())
            # A missing or malformed plan (a failed planner step) runs everything.
            Path(tmp, "list.json").write_text("[]")
            for bad in ("missing.json", "list.json"):
                out = io.StringIO()
                with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
                    code = ci_impact.main(["jobs", f"--plan={tmp}/{bad}", f"--workflow={self.LEAVES}",
                                           f"--tested-sha={'a' * 40}", f"--repo={tmp}", "--mode=select",
                                           "--base-ref=main"])
                self.assertEqual(code, 0, bad)
                self.assertIn("narrowed=false", out.getvalue().splitlines())
                self.assertIn("run-quality=true", out.getvalue().splitlines())
            # A broken catalogue fails the select job; its dependents are skipped and merge-hold blocks.
            Path(tmp, ".github/ci-impact.toml").write_text("schema_version = [\n")
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(ci_impact.main(["jobs", f"--plan={tmp}/plan.json", f"--workflow={self.LEAVES}",
                                                 f"--tested-sha={'a' * 40}", f"--repo={tmp}"]), 1)

    def test_selection_condition_is_decidable_and_true_when_planned(self):
        catalogue = valid_catalogue()
        catalogue["workflow"][1]["job"].insert(0, {"id": "select", "class": "core", "activation": "automatic"})
        quality = catalogue["workflow"][1]["job"][1]
        quality["needs"] = ["select"]
        quality["condition"] = "${{ needs.select.outputs.run-quality == 'true' }}"
        self.assertEqual(ci_impact.validate(catalogue), [])
        jobs = {job["id"]: job for job in catalogue["workflow"][1]["job"]}
        self.assertIs(ci_impact.job_runs(jobs, "quality", "pull_request"), True)
        # Any other use of a select output is rejected: the evaluator answers every key alike.
        for condition in ("${{ needs.select.outputs.run-quality != 'true' }}",
                          "${{ needs.select.outputs.run-quality == 'false' }}",
                          "${{ needs.select.outputs.run-other == 'true' }}",
                          "${{ needs.select.outputs.run-quality == 'true' && github.event_name == 'push' }}"):
            with self.subTest(condition=condition):
                quality["condition"] = condition
                self.assertTrue(any("select-condition" in p for p in ci_impact.validate(catalogue)))
        quality["condition"] = "${{ needs.select.outputs.run-quality == 'true' }}"
        quality["needs"] = []
        self.assertTrue(any("select-condition needs the select job" in p for p in ci_impact.validate(catalogue)))


    def test_coverage_accepts_only_a_correctly_wired_catalogue_matrix(self):
        catalogue = self.catalogue()
        leaves = catalogue["workflow"][1]
        leaves["job"].insert(0, {"id": "select", "name": "leaves select", "class": "core", "activation": "automatic"})
        quality = leaves["job"][1]
        quality["needs"] = ["select"]
        quality["condition"] = "${{ needs.select.outputs.run-quality == 'true' }}"
        catalogue["workflow"] = [leaves]
        doc = {"on": {"pull_request": {"paths-ignore": catalogue["policy"]["inert"]}, "push": {},
                      "workflow_dispatch": {}},
               "jobs": {"select": {"name": "leaves select"},
                        "quality": {"name": "quality (${{ matrix.component }})", "needs": "select",
                                    "if": "${{ needs.select.outputs.run-quality == 'true' }}",
                                    "strategy": {"matrix": "${{ fromJSON(needs.select.outputs.matrix-quality) }}"}}}}
        problems = ci_impact.workflow_problems({self.LEAVES: doc}, catalogue)
        self.assertEqual(problems, [])
        wrong = copy.deepcopy(doc)
        wrong["jobs"]["quality"]["strategy"]["matrix"] = "${{ fromJSON(needs.select.outputs.matrix-other) }}"
        self.assertTrue(any("selects the matrix of other" in p
                            for p in ci_impact.workflow_problems({self.LEAVES: wrong}, catalogue)))
        unwired = copy.deepcopy(catalogue)
        unwired["workflow"][0]["job"][1]["condition"] = "${{ github.event_name != 'push' }}"
        doc_unwired = copy.deepcopy(doc)
        doc_unwired["jobs"]["quality"]["if"] = "${{ github.event_name != 'push' }}"
        self.assertTrue(any("condition must be the run-quality output" in p
                            for p in ci_impact.workflow_problems({self.LEAVES: doc_unwired}, unwired)))
        renamed = copy.deepcopy(catalogue)
        renamed["workflow"][0]["job"][1]["variant"][0]["check_name"] = "quality (renamed)"
        self.assertTrue(any("rendered name differs" in p
                            for p in ci_impact.workflow_problems({self.LEAVES: doc}, renamed)))


if __name__ == "__main__":
    main()
