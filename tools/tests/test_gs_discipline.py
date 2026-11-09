# SPDX-License-Identifier: MPL-2.0
"""Audit F8 / ADR-076: static GS discipline of the IDT stubs (spec §5).

STATIC check only: QEMU-TCG does not model speculation, so this is the sole
evidence for GS-5 (`lfence` on the join). It reads the `global_asm!` blocks in
kernel/src/m2/ and fails when a stub, `iretq`, `swapgs`, `gs:` operand, GS
selector load or GS MSR write appears outside the reviewed pattern. A new stub
or exit therefore fails here until it is added deliberately.
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

M2 = Path(__file__).resolve().parents[2] / "kernel" / "src" / "m2"
MACRO = re.compile(r'(?:arch::)?gs_(entry|exit)!\(\s*"([^"]*)"\s*\)')
LABEL = re.compile(r"^\s*([A-Za-z_.$][\w.$]*):")
# IDT entry points that use GS (prologue first) and the one that does not.
GS_STUBS = {"nos_smp_ipi_entry", "nos_smp_shootdown_entry", "nos_smp_timer_entry",
            "nos_smp_resched_entry"}
NO_GS_STUBS = {"nos_smp_spurious_entry"}
# Step 5c-2: the PIT IRQ0 clock stub (BSP, CPL0 only) touches no GS and returns
# through the spurious stub's reviewed `iretq`.
JOIN_STUBS = {"nos_smp_clock_entry"}
# Stubs that only normalize the frame and jump to the shared prologue.
# Step 5d-2: `m3_ap_timer` (AP M3 tick) joins the prologue from CPL3; its CPL0
# path only EOIs and returns through the spurious stub's reviewed `iretq`.
COMMON_STUBS = {"m3_timer", "m3_syscall", "m3_ap_timer"}
EXITS = {"nos_iret_user_ipi", "nos_iret_user_shootdown", "nos_iret_user_timer",
         "nos_iret_user_m3"}
PARANOID = {2, 8, 18}
# LD-1 (audit F3): the per-CPU IDT nesting counter, `CpuLocal::irq_depth`.
DEPTH_INC = "inc dword ptr gs:[{depth}]"
DEPTH_DEC = "dec dword ptr gs:[{depth}]"


def string_end(text: str, start: int) -> tuple[str, int]:
    """Decode the Rust string literal at `start`; return (value, index after it)."""
    raw = re.match(r'r(#*)"', text[start:])
    if raw:
        close = '"' + raw[1]
        end = text.index(close, start + len(raw[0]))
        return text[start + len(raw[0]):end], end + len(close)
    value, index = [], start + 1
    while text[index] != '"':
        if text[index] == "\\":
            value.append({"n": "\n", '"': '"', "\\": "\\"}.get(text[index + 1], text[index + 1]))
            index += 2
        else:
            value.append(text[index])
            index += 1
    return "".join(value), index + 1


def asm_blocks(path: Path) -> list[list[str]]:
    """Every global_asm! block as lines; gs_entry!/gs_exit! become `@ENTRY`/`@EXIT`."""
    text, blocks = path.read_text(encoding="utf-8"), []
    for found in re.finditer(r"global_asm!\(", text):
        index, depth, lines = found.end(), 1, []
        while depth:
            macro = MACRO.match(text, index)
            if macro:
                lines.append(f"@{macro[1].upper()} {macro[2]}")
                index = macro.end()
            elif text.startswith(('"', 'r"', 'r#"'), index):
                value, index = string_end(text, index)
                lines.extend(value.split("\n"))
            else:
                depth += {"(": 1, ")": -1}.get(text[index], 0)
                index += 1
        blocks.append([line.split("//")[0].strip() for line in lines])
    return blocks


def all_lines() -> list[str]:
    return [line for path in sorted(M2.glob("*.rs")) for block in asm_blocks(path)
            for line in block if line]


def label_of(line: str) -> str | None:
    match = LABEL.match(line)
    return match[1] if match else None


def region(lines: list[str], start: str, stop) -> list[str]:
    """Lines after label `start` up to (excluding) the first line matching `stop`."""
    first = lines.index(f"{start}:") + 1
    end = next(i for i in range(first, len(lines)) if stop(lines[i]))
    return lines[first:end]


def ends_stub(line: str) -> bool:
    label = label_of(line)
    return line.startswith(("@EXIT", ".popsection", "iretq")) or bool(
        label and label.endswith("_entry"))


class GsDiscipline(unittest.TestCase):
    lines = all_lines()

    def test_macros_swap_only_on_cpl3_and_fence_the_join(self):
        source = (M2 / "arch.rs").read_text(encoding="utf-8")
        body = lambda name: re.sub(r"\s+", " ", re.search(
            rf"macro_rules! {name} \{{(.*?)\n\}}", source, re.S)[1])
        entry, exit_ = body("gs_entry"), body("gs_exit")

        def in_order(body, *steps):
            positions = [body.index(step) for step in steps]
            self.assertEqual(positions, sorted(positions))

        in_order(entry, "test byte ptr [rsp+", "jz 7f", "swapgs", "7:", "lfence")
        in_order(exit_, "test byte ptr [rsp+8], 3", "jz 7f", "swapgs", "7:", '$label, ":', "iretq")
        self.assertEqual(entry.count("swapgs"), 1)
        self.assertEqual(exit_.count("swapgs"), 1)

    def test_every_iretq_and_stub_is_reviewed(self):
        # Rule 1: raw `iretq` only right after the spurious label; every IDT
        # entry label is allowlisted.
        raw = [i for i, line in enumerate(self.lines) if line == "iretq"]
        self.assertEqual([label_of(self.lines[i - 1]) for i in raw], sorted(NO_GS_STUBS))
        entries = {label for line in self.lines if (label := label_of(line))
                   and re.fullmatch(r"nos_smp_\w+_entry|m3_timer|m3_syscall|m3_ap_timer", label)}
        self.assertEqual(entries, GS_STUBS | NO_GS_STUBS | JOIN_STUBS | COMMON_STUBS)
        for stub in sorted(JOIN_STUBS):
            with self.subTest(stub=stub):
                body = region(self.lines, stub, ends_stub)
                self.assertFalse(any(re.search(r"gs:|swapgs|@ENTRY|@EXIT", line) for line in body))
                self.assertIn("jmp nos_smp_spurious_entry", body)

    def test_gs_stubs_start_with_exactly_one_prologue(self):
        # Rules 2/3 (GS-3, GS-4): the prologue is the first instruction and the
        # only one; 0xe8 joins the timer body after the timer prologue.
        for stub in sorted(GS_STUBS):
            with self.subTest(stub=stub):
                body = region(self.lines, stub, ends_stub)
                self.assertEqual(body[0], "@ENTRY 8")
                self.assertEqual(sum(line.startswith("@ENTRY") for line in body), 1)
                self.assertFalse(any("swapgs" in line for line in body))
        resched = region(self.lines, "nos_smp_resched_entry", ends_stub)
        self.assertEqual(resched[-1], "jmp nos_smp_timer_body")
        timer = region(self.lines, "nos_smp_timer_entry", lambda line: line.startswith("push"))
        self.assertEqual(timer, ["@ENTRY 8", ".globl nos_smp_timer_body", "nos_smp_timer_body:",
                                 DEPTH_INC])

    def test_gs_stubs_count_their_nesting_once(self):
        # Audit F3 (LD-1): the first instruction after the prologue counts the
        # entry and the last before each exit uncounts it; 0xe8 is counted
        # once, in the timer body it joins.
        for stub in ("nos_smp_ipi_entry", "nos_smp_shootdown_entry"):
            with self.subTest(stub=stub):
                body = region(self.lines, stub, ends_stub)
                self.assertEqual(body[1], DEPTH_INC)
                self.assertEqual(sum("{depth}" in line for line in body), 2)
        resched = region(self.lines, "nos_smp_resched_entry", ends_stub)
        self.assertFalse(any("{depth}" in line for line in resched))
        exits = [i for i, line in enumerate(self.lines)
                 if line.startswith("@EXIT nos_iret_user_") and line != "@EXIT nos_iret_user_m3"]
        self.assertEqual(len(exits), 3)
        self.assertTrue(all(self.lines[i - 1] == DEPTH_DEC for i in exits))

    def test_exception_prologue_and_fixup_list(self):
        # Rules 2/4 (GS-4, GS-7): checked prologue before any push; every
        # labelled CPL3 exit is in the fixup list and nothing else is.
        common = region(self.lines, "m2_exception_common",
                        lambda line: line == "m2_exception_paranoid:")
        self.assertEqual(common[0], "test byte ptr [rsp+24],3")
        self.assertEqual(common[-4:], ["7:", "swapgs", "8:", "lfence"])
        self.assertEqual(sum(line == "swapgs" for line in common), 1)
        self.assertFalse(any(re.search(r"gs:|\bcall\b|push", line) for line in common))
        fixups = {m[1] for line in common
                  if (m := re.fullmatch(r"cmp qword ptr \[rsp\+16\],offset (\w+)", line))}
        exits = {line.split()[1] for line in self.lines if line.startswith("@EXIT")}
        self.assertEqual(fixups, EXITS)
        self.assertEqual(exits, EXITS)
        for stub in sorted(COMMON_STUBS):
            body = region(self.lines, stub, lambda line: line.startswith("jmp"))
            self.assertFalse(any(re.search(r"gs:|swapgs|call", line) for line in body))
            self.assertEqual(self.lines[self.lines.index(f"{stub}:") + len(body) + 1],
                             "jmp m2_exception_common")

    def test_paranoid_vectors_never_touch_gs(self):
        # Rule 5 (GS-6): NMI, #DF and #MC bypass the prologue and stay terminal.
        vectors = {}
        for line in self.lines:
            if m := re.fullmatch(r"EX (\d+)(?:,(\d))?(?:,(\w+))?", line):
                vectors[int(m[1])] = m[3] or "m2_exception_common"
        self.assertEqual(set(vectors), set(range(32)))
        for vector, path in vectors.items():
            expected = "m2_exception_paranoid" if vector in PARANOID else "m2_exception_common"
            self.assertEqual(path, expected, vector)
        paranoid = region(self.lines, "m2_exception_paranoid",
                          lambda line: line == "jmp m2_dispatch")
        self.assertFalse(any(re.search(r"gs:|swapgs|call", line) for line in paranoid))
        dispatch = " ".join(self.lines)
        for vector in PARANOID:
            self.assertIn(f"cmp qword ptr [rsp+120],{vector} je m3_terminal", dispatch)

    def test_gs_selector_and_msr_writes_stay_in_init(self):
        # Rule 6 (GS-9): selector loads only before the anchor is written;
        # GS_BASE/KERNEL_GS_BASE writes only in install/install_ap (zeroing
        # loops, both MSRs each) and start/AP main (anchor); reads in
        # this_cpu/any_cpu.
        sources = {path.name: re.sub(r"//[^\n]*", "", path.read_text(encoding="utf-8"))
                   for path in M2.glob("*.rs")}
        selector = re.compile(r"mov\w?\s+(?:gs,|%ax,\s*%gs)")
        self.assertEqual({name: len(selector.findall(text)) for name, text in sources.items()
                          if selector.search(text)}, {"arch.rs": 2, "smp.rs": 1})
        msr = re.compile(r"0xc000_?010[12]")
        self.assertEqual({name: len(msr.findall(text)) for name, text in sources.items()
                          if msr.search(text)}, {"arch.rs": 4, "smp.rs": 4})
        smp = sources["smp.rs"]
        self.assertEqual(len(re.findall(r"wrmsr\(0xc000_0101,", smp)), 2)
        self.assertEqual(len(re.findall(r"rdmsr\(0xc000_0101\)", smp)), 2)
        self.assertNotIn("0xc000_0102", smp)

    def test_gs_operands_only_in_native_smp_code(self):
        # Rule 7: `gs:` only in the native-smp module.
        for path in M2.glob("*.rs"):
            uses = [line for block in asm_blocks(path) for line in block if "gs:" in line]
            self.assertEqual(bool(uses), path.name == "smp.rs", path.name)
        self.assertRegex((M2.parent / "m2.rs").read_text(encoding="utf-8"),
                         r'#\[cfg\(feature = "native-smp"\)\]\n(pub\(crate\) )?mod smp;')


if __name__ == "__main__":
    unittest.main()
