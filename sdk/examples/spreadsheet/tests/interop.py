#!/usr/bin/env python3
"""Independent host consumer oracle; stdlib Decimal, not calculator code."""
from decimal import Decimal, ROUND_HALF_EVEN, localcontext
from pathlib import Path
import subprocess
import sys


def main():
    binary = Path(sys.argv[1]).resolve()
    commands = []
    expected = []
    for index in range(32):
        a = Decimal(index - 16) / 100
        b = Decimal(index + 1) / 1000
        commands += [
            "--set", "A1", str(a), "--set", "A2", str(b),
            "--set", "B1", "=(A1+A2)*3", "--set", "C1", "=B1/8", "--get", "C1",
        ]
        expected.append((a + b) * 3 / 8)
    result = subprocess.run([str(binary), *commands], capture_output=True, check=True, text=True)
    assert not result.stderr
    actual = [Decimal(line.replace(",", ".")) for line in result.stdout.splitlines()]
    assert actual == expected
    print("32 edit/formula/dependency/recalc CLI results match independent Decimal exactly")
    with localcontext() as context:
        context.prec = 40
        context.rounding = ROUND_HALF_EVEN
        for numerator in [1, 2, 4, -1, -2]:
            result = subprocess.run(
                [str(binary), "--set", "A1", f"={numerator}/3",
                 "--set", "B1", "=A1*3", "--get", "B1"],
                capture_output=True, check=True, text=True,
            )
            rounded = (Decimal(numerator) / 3).quantize(Decimal("0.000000001"))
            number, marker = result.stdout.strip().split(" ")
            assert marker == "[afgerond]" and not result.stderr
            assert Decimal(number.replace(",", ".")) == rounded * 3
    print("5 rounded dependency results independently checked, rounding remains explicit")


if __name__ == "__main__":
    main()
