#!/usr/bin/env python3
"""Convert a HW-INV serial log into the schema-2 inventory JSON (protocol step S6).

The log must hold the complete, ordered HWINV block of one native-hw-inventory
boot: BEGIN, CPUID, SECUREBOOT, ACPI:DMAR, the PCI lines, COUNT and PASS, with
no HWINV:FAIL line. The converter only restates those lines; it adds no fact.
`tools/hw_evidence.py --check` remains the acceptance check of a signed record,
and rejects a virtual (QEMU) capture by its hypervisor bit or PCI vendors.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from hw_evidence import (HWINV_CPUID, HWINV_DMAR, HWINV_PCI_COUNT, HWINV_PCI_LINE, HWINV_SECUREBOOT,
                         PCI_FIELDS, SECUREBOOT_STATES)


class InventoryLogError(ValueError):
    """The serial log does not hold one complete, passing HWINV block."""


def hwinv_lines(text: str) -> list[str]:
    return [line for line in text.replace("\r", "").splitlines() if line.startswith("HWINV:")]


def inventory_from_log(text: str) -> dict:
    lines = hwinv_lines(text)
    if "HWINV:FAIL" in text:  # anywhere: firmware output can share the marker's line
        raise InventoryLogError("log reports HWINV:FAIL")
    if lines.count("HWINV:BEGIN") != 1 or lines.count("HWINV:PASS") != 1:
        raise InventoryLogError("log needs exactly one HWINV:BEGIN and one HWINV:PASS")
    if len(lines) < 7 or lines[0] != "HWINV:BEGIN" or lines[-1] != "HWINV:PASS":
        raise InventoryLogError("HWINV block is not BEGIN ... PASS")
    cpuid = HWINV_CPUID.fullmatch(lines[1])
    secure = HWINV_SECUREBOOT.fullmatch(lines[2])
    dmar = HWINV_DMAR.fullmatch(lines[3])
    count = HWINV_PCI_COUNT.fullmatch(lines[-2])
    if cpuid is None or secure is None or dmar is None or count is None:
        raise InventoryLogError("CPUID, SECUREBOOT, ACPI:DMAR or PCI:COUNT line missing or out of order")
    if dmar[1] == "ABSENT" and dmar[2] != "0":
        raise InventoryLogError("an absent DMAR table cannot have DRHD units")
    pci = []
    for line in lines[4:-2]:
        match = HWINV_PCI_LINE.fullmatch(line)
        if match is None:
            raise InventoryLogError(f"malformed or unexpected line in the PCI block: {line!r}")
        pci.append(dict(zip(PCI_FIELDS, match.groups())))
    if int(count[1]) != len(pci) or not pci:
        raise InventoryLogError(f"HWINV:PCI:COUNT={count[1]} but {len(pci)} PCI lines")
    if len({function["address"] for function in pci}) != len(pci):
        raise InventoryLogError("a PCI address is listed twice")
    return {
        "schema": 2,
        "kind": "baryonos-hw-inventory",
        "cpuid": {"vendor": cpuid[1], "hypervisor_bit": cpuid[2] == "1"},
        "secure_boot": SECUREBOOT_STATES[secure[1]],
        "acpi": {"dmar": dmar[1].lower(), "drhd_units": int(dmar[2])},
        "pci": pci,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log", type=Path, required=True, help="captured serial log")
    parser.add_argument("--out", type=Path, required=True, help="inventory JSON to write")
    args = parser.parse_args()
    inventory = inventory_from_log(args.log.read_bytes().decode("utf-8", "replace"))
    args.out.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
