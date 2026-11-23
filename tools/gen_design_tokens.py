#!/usr/bin/env python3
"""Generate ui/design/system/tokens.css from ui/design/system/tokens.source.json.

The ds-1.1 accessibility axis blocks are produced from structured data: each rule names
a theme scope, an axis and a value, and the generator builds the selector, the device
preference mirror (``@media`` with ``:not([data-axis])``) and the reduced-motion blocks.
The ds-1.0 base section (header, @font-face, :root, themes, neutral surface and density)
is carried verbatim from the source and validated: every theme defines the same token
set and every axis override names an existing token.

Usage: ``python3 -B tools/gen_design_tokens.py --check`` (default) or ``--write``.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "ui/design/system/tokens.source.json"
OUTPUT = ROOT / "ui/design/system/tokens.css"
FORMAT = "baryon-design-tokens-source/1"

SCOPES = {
    "dark": ':is(:root:not([data-theme]), [data-theme="dark"])',
    "light": '[data-theme="light"]',
    "contrast": '[data-theme="contrast"]',
    "any": ":is(:root, [data-theme])",
    "all": "",
}
THEME_SELECTORS = {
    "dark": ':root, [data-theme="dark"]',
    "light": '[data-theme="light"]',
    "contrast": '[data-theme="contrast"]',
}
NAME_RE = re.compile(r"--[a-z0-9]+(?:-[a-z0-9]+)*\Z")
IDENT_RE = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*\Z")
VALUE_RE = re.compile(r"[^;{}\n]+\Z")
BLOCK_RE = re.compile(r"^([^\s/@}][^{\n]*?)\s*\{(.*?)^\}", re.S | re.M)
DECL_RE = re.compile(r"(--[a-z0-9-]+)\s*:\s*([^;]+);")
MOTION_PREFIX = "--motion-duration-"


class SourceError(ValueError):
    pass


def base_blocks(base: str) -> dict[str, dict[str, str]]:
    blocks: dict[str, dict[str, str]] = {}
    for selector, body in BLOCK_RE.findall(base):
        body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
        blocks[selector.strip()] = dict(DECL_RE.findall(body))
    return blocks


def _check_decl(name: str, value: str, where: str) -> None:
    if not NAME_RE.match(name):
        raise SourceError(f"{where}: invalid token name {name!r}")
    if not VALUE_RE.match(value) or value != value.strip():
        raise SourceError(f"{where}: invalid value {value!r} for {name}")


def validate(source: dict) -> dict[str, dict[str, str]]:
    if source.get("format") != FORMAT:
        raise SourceError(f"unsupported source format {source.get('format')!r}")
    blocks = base_blocks("\n".join(source["base_css"]))
    themes = {}
    for theme, selector in THEME_SELECTORS.items():
        if selector not in blocks:
            raise SourceError(f"base: missing theme block {selector!r}")
        themes[theme] = blocks[selector]
    reference = set(themes["dark"])
    for theme, decls in themes.items():
        for name, value in decls.items():
            _check_decl(name, value.strip(), f"base {theme}")
        if set(decls) != reference:
            missing = sorted(reference - set(decls))
            extra = sorted(set(decls) - reference)
            raise SourceError(f"base {theme}: token set differs (missing {missing}, extra {extra})")
    if ":root" not in blocks:
        raise SourceError("base: missing :root block")
    known = set(reference) | set(blocks[":root"])
    for section in source["sections"]:
        for group in section["groups"]:
            values = set()
            for item in group:
                if "device" in item:
                    dev = item["device"]
                    if (dev["axis"], dev["value"]) not in values:
                        raise SourceError(f"device mirror {dev} has no preceding rules")
                    continue
                if item["scope"] not in SCOPES:
                    raise SourceError(f"unknown scope {item['scope']!r}")
                for key in ("axis", "value"):
                    if not IDENT_RE.match(item[key]):
                        raise SourceError(f"invalid {key} {item[key]!r}")
                values.add((item["axis"], item["value"]))
                where = f"axis {item['axis']}={item['value']} ({item['scope']})"
                if not item["decls"] or not all(item["decls"]):
                    raise SourceError(f"{where}: empty declaration line")
                for line in item["decls"]:
                    for name, value in line:
                        _check_decl(name, value, where)
                        if name not in known:
                            raise SourceError(f"{where}: unknown token {name}")
    if not any(n.startswith(MOTION_PREFIX) for n in blocks[":root"]):
        raise SourceError("base: :root defines no motion durations")
    return blocks


def _decl_lines(lines: list) -> list[str]:
    return ["  " + " ".join(f"{name}: {value};" for name, value in line) for line in lines]


def _rule(selector: str, lines: list) -> list[str]:
    return [selector + " {", *_decl_lines(lines), "}"]


def _condition(axis: str, value: str | None) -> str:
    if value is None:
        return f":not([data-{axis}], [data-{axis}] *)"
    attr = f'[data-{axis}="{value}"]'
    return f":is({attr}, {attr} *)"


def render(source: dict) -> str:
    blocks = validate(source)
    out = list(source["base_css"])
    out += ["", ""] + source["axis_header"]
    for section in source["sections"]:
        out.append("")
        out += section["comment"]
        for index, group in enumerate(section["groups"]):
            if index:
                out.append("")
            for item in group:
                if "device" in item:
                    dev = item["device"]
                    out.append(f"@media {dev['media']} {{")
                    for rule in group:
                        if "device" not in rule and (rule["axis"], rule["value"]) == (dev["axis"], dev["value"]):
                            out += _rule(SCOPES[rule["scope"]] + _condition(rule["axis"], None), rule["decls"])
                    out.append("}")
                else:
                    out += _rule(SCOPES[item["scope"]] + _condition(item["axis"], item["value"]), item["decls"])
    motion = source["motion"]
    durations = [(n, v.strip()) for n, v in blocks[":root"].items() if n.startswith(MOTION_PREFIX)]
    zero = " ".join(f"{n}: 0ms;" for n, _ in durations)
    full = " ".join(f"{n}: {v};" for n, v in durations)
    out += ["", motion["reduce_comment"], "@media (prefers-reduced-motion: reduce) {",
            f'  :root:not([data-motion="full"]) {{ {zero} }}', "}",
            f'[data-motion="reduce"] {{ {zero} }}', motion["full_comment"],
            f'[data-motion="full"] {{ {full} }}']
    return "\n".join(out) + "\n"


def load(path: Path = SOURCE) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="fail when tokens.css differs (default)")
    mode.add_argument("--write", action="store_true", help="rewrite tokens.css from the source")
    args = parser.parse_args(argv)
    try:
        text = render(load())
    except (SourceError, KeyError, TypeError, ValueError) as exc:
        print(f"gen_design_tokens: source invalid: {exc}", file=sys.stderr)
        return 2
    if args.write:
        OUTPUT.write_text(text, encoding="utf-8", newline="\n")
        print(f"gen_design_tokens: wrote {OUTPUT.relative_to(ROOT)}")
        return 0
    if OUTPUT.read_text(encoding="utf-8") != text:
        print("gen_design_tokens: tokens.css differs from tokens.source.json; run --write", file=sys.stderr)
        return 1
    print("gen_design_tokens: tokens.css matches tokens.source.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
