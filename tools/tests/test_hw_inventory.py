"""HW-INV log conversion and QEMU-oracle evaluation (H06-T3); host-only, no hardware execution."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import hw_inventory
import run_hw_inventory

# Marker lines of the native `iommu` case captured on 2026-11-24 (QEMU q35, TCG).
LOG = """\
BOOT:EBS_OK
KERNEL:ENTRY
BOOTINFO:OK
KS-01:PASS
KS-02:PASS
ACPI:DISCOVERY:MCFG_DMAR
DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT
KS-03:PASS
M2:PASS
M1:PASS
HWINV:BEGIN
HWINV:CPUID:VENDOR=GenuineIntel:HYPERVISOR=1
HWINV:SECUREBOOT=ABSENT
HWINV:ACPI:DMAR=PRESENT:DRHD=1
HWINV:PCI:0000:00:00.0:8086:29c0:06:00:00:00
HWINV:PCI:0000:00:01.0:1af4:1001:01:00:00:00
HWINV:PCI:0000:00:06.0:1b36:000c:06:04:00:00
HWINV:PCI:0000:00:1f.0:8086:2918:06:01:00:02
HWINV:PCI:0000:00:1f.2:8086:2922:01:06:01:02
HWINV:PCI:0000:00:1f.3:8086:2930:0c:05:00:02
HWINV:PCI:0000:01:00.0:1b36:000d:0c:03:30:01
HWINV:PCI:COUNT=7
HWINV:PASS
"""
QMP = {
    ("0000:00:00.0", "8086", "29c0", "06", "00"), ("0000:00:01.0", "1af4", "1001", "01", "00"),
    ("0000:00:06.0", "1b36", "000c", "06", "04"), ("0000:00:1f.0", "8086", "2918", "06", "01"),
    ("0000:00:1f.2", "8086", "2922", "01", "06"), ("0000:00:1f.3", "8086", "2930", "0c", "05"),
    ("0000:01:00.0", "1b36", "000d", "0c", "03"),
}


def edit(old: str, new: str) -> str:
    assert old in LOG, old
    return LOG.replace(old, new, 1)


class ConvertTest(unittest.TestCase):
    def test_complete_log_converts_to_schema_2(self):
        inventory = hw_inventory.inventory_from_log(LOG.replace("\n", "\r\n"))
        self.assertEqual(inventory["schema"], 2)
        self.assertEqual(inventory["kind"], "baryonos-hw-inventory")
        self.assertEqual(inventory["cpuid"], {"vendor": "GenuineIntel", "hypervisor_bit": True})
        self.assertEqual(inventory["secure_boot"], "unknown")
        self.assertEqual(inventory["acpi"], {"dmar": "present", "drhd_units": 1})
        self.assertEqual(len(inventory["pci"]), 7)
        self.assertEqual(inventory["pci"][-1], {"address": "0000:01:00.0", "vendor": "1b36", "device": "000d",
                                                "class": "0c", "subclass": "03", "prog_if": "30",
                                                "revision": "01"})

    def test_incomplete_or_inconsistent_logs_are_rejected(self):
        cases = {
            "failure marker": LOG + "HWINV:FAIL:PCI_LIMIT\n",
            "prefixed failure marker": LOG + "fw: HWINV:FAIL:PCI_LIMIT\n",
            "kernel panic": LOG + "firmware: KERNEL_PANIC\n",
            "m1 failure": LOG + "M1:FAIL:X\n",
            "m2 panic line": LOG + "M2:PANIC_LINE:42\n",
            "embedded PCI marker": LOG + "fw: HWINV:PCI:0000:02:00.0:8086:1234:02:00:00:00\n",
            "count mismatch": edit("COUNT=7", "COUNT=6"),
            "duplicate address": edit("HWINV:PCI:0000:00:1f.3:8086:2930", "HWINV:PCI:0000:00:1f.2:8086:2930"),
            "order": edit("HWINV:CPUID:VENDOR=GenuineIntel:HYPERVISOR=1\nHWINV:SECUREBOOT=ABSENT",
                          "HWINV:SECUREBOOT=ABSENT\nHWINV:CPUID:VENDOR=GenuineIntel:HYPERVISOR=1"),
            "absent with units": edit("DMAR=PRESENT:DRHD=1", "DMAR=ABSENT:DRHD=1"),
            "uppercase hex": edit("8086:29c0", "8086:29C0"),
            "missing pass": edit("HWINV:PASS\n", ""),
            "second begin": LOG + "HWINV:BEGIN\n",
            "foreign line": edit("HWINV:PCI:COUNT=7", "HWINV:EXTRA\nHWINV:PCI:COUNT=7"),
        }
        for name, text in cases.items():
            with self.subTest(name), self.assertRaises(hw_inventory.InventoryLogError):
                hw_inventory.inventory_from_log(text)


class EvaluateTest(unittest.TestCase):
    def test_captured_iommu_log_passes_every_oracle(self):
        checks, inventory, error = run_hw_inventory.evaluate(LOG, "iommu", QMP, True)
        self.assertIsNone(error)
        self.assertIsNotNone(inventory)
        self.assertEqual({name for name, ok in checks.items() if not ok}, set())

    def test_each_oracle_detects_its_own_deviation(self):
        def failed(text=LOG, case="iommu", qmp=QMP, alive=True):
            checks, _, _ = run_hw_inventory.evaluate(text, case, qmp, alive)
            return {name for name, ok in checks.items() if not ok}

        self.assertEqual(failed(qmp=QMP - {("0000:01:00.0", "1b36", "000d", "0c", "03")}),
                         {"pci_equals_qmp_query_pci"})
        self.assertEqual(failed(qmp=None), {"pci_equals_qmp_query_pci"})
        self.assertEqual(failed(alive=False), {"halted_without_exit_port"})
        self.assertEqual(failed(case="plain"),
                         {"cpuid_matches_configuration", "dmar_matches_configuration", "kernel_acpi_marker"})
        self.assertEqual(failed(text=edit("SECUREBOOT=ABSENT", "SECUREBOOT=1")), {"secure_boot_not_claimed"})
        self.assertIn("no_failure_marker", failed(text=LOG.replace("M1:PASS", "M1:FAIL:X")))
        self.assertIn("no_failure_marker", failed(text=LOG + "fw KERNEL_PANIC\n"))
        self.assertIn("single_ebs", failed(text="BOOT:EBS_OK\n" + LOG))

    def test_a_physical_looking_inventory_would_not_be_flagged_virtual(self):
        physical = (LOG.replace("HYPERVISOR=1", "HYPERVISOR=0")
                    .replace(":1af4:", ":10ec:").replace(":1b36:", ":8086:"))
        qmp = {(a, {"1af4": "10ec", "1b36": "8086"}.get(v, v), d, c, s) for a, v, d, c, s in QMP}
        checks, _, _ = run_hw_inventory.evaluate(physical, "iommu", qmp, True)
        self.assertFalse(checks["rejected_as_virtual_capture"])

    def test_query_pci_walk_includes_functions_behind_bridges(self):
        reply = [{"bus": 0, "devices": [
            {"bus": 0, "slot": 6, "function": 0, "class_info": {"class": 0x0604},
             "id": {"vendor": 0x1b36, "device": 0x000c},
             "pci_bridge": {"bus": {"number": 1}, "devices": [
                 {"bus": 1, "slot": 0, "function": 0, "class_info": {"class": 0x0c03},
                  "id": {"vendor": 0x1b36, "device": 0x000d}}]}}]}]
        self.assertEqual(run_hw_inventory.qmp_functions(reply), {
            ("0000:00:06.0", "1b36", "000c", "06", "04"), ("0000:01:00.0", "1b36", "000d", "0c", "03")})


if __name__ == "__main__":
    unittest.main()
