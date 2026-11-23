"""Design-token generator regressions (O-P1): tokens.css is reproducible from tracked source."""
import copy
import hashlib
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import gen_design_tokens as gen

PINNED_SHA256 = "c4513523d43435c64691749d7d8b9771c3657fa8db9cdcb01835aa08fe96ee39"


class DesignTokenGenerator(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = gen.load()

    def first_rule(self, source, axis):
        for section in source["sections"]:
            for group in section["groups"]:
                for item in group:
                    if item.get("axis") == axis:
                        return item
        raise AssertionError(axis)

    def assert_rejected(self, source, fragment):
        with self.assertRaises(gen.SourceError) as caught:
            gen.render(source)
        self.assertIn(fragment, str(caught.exception))

    def test_generated_output_is_byte_identical_to_pinned_tokens_css(self):
        text = gen.render(self.source)
        self.assertEqual(text, gen.OUTPUT.read_text(encoding="utf-8"))
        self.assertEqual(hashlib.sha256(text.encode("utf-8")).hexdigest(), PINNED_SHA256)
        self.assertEqual(gen.main(["--check"]), 0)

    def test_device_mirror_and_reduced_motion_are_derived(self):
        source = copy.deepcopy(self.source)
        rule = self.first_rule(source, "contrast")
        rule["decls"][0][0][1] = "#FEFEFE"
        text = gen.render(source)
        self.assertEqual(text.count("--color-text: #FEFEFE;"), 2)
        root = source["base_css"]
        index = next(i for i, line in enumerate(root) if "--motion-duration-fast:" in line)
        root[index] = root[index].replace("140ms", "150ms")
        self.assertIn('[data-motion="full"] { --motion-duration-fast: 150ms;', gen.render(source))

    def test_unknown_token_in_axis_is_rejected(self):
        source = copy.deepcopy(self.source)
        self.first_rule(source, "stimulus")["decls"][0].append(["--color-invented", "#000"])
        self.assert_rejected(source, "unknown token --color-invented")

    def test_invalid_token_name_and_value_are_rejected(self):
        source = copy.deepcopy(self.source)
        self.first_rule(source, "targets")["decls"][0][0][0] = "--Space_Target"
        self.assert_rejected(source, "invalid token name")
        source = copy.deepcopy(self.source)
        self.first_rule(source, "targets")["decls"][0][0][1] = "1px} :root{--x:0"
        self.assert_rejected(source, "invalid value")

    def test_theme_token_sets_must_match(self):
        source = copy.deepcopy(self.source)
        index = next(i for i, line in enumerate(source["base_css"]) if line.startswith('[data-theme="light"] {'))
        while "--color-text:" not in source["base_css"][index]:
            index += 1
        del source["base_css"][index]
        self.assert_rejected(source, "base light: token set differs")

    def test_unknown_scope_and_orphan_device_mirror_are_rejected(self):
        source = copy.deepcopy(self.source)
        self.first_rule(source, "transparency")["scope"] = "sepia"
        self.assert_rejected(source, "unknown scope")
        source = copy.deepcopy(self.source)
        source["sections"][0]["groups"][0].insert(0, {"device": {"media": "(x)", "axis": "contrast", "value": "increased"}})
        self.assert_rejected(source, "no preceding rules")


if __name__ == "__main__":
    unittest.main()
