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


class FixtureTest(unittest.TestCase):
    """H06-T5 terminal fixtures: markers in order, protocol failure, no exit port."""
    PANIC = "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM2:PANIC_LINE:0000000310\nM1:FAIL:KERNEL_PANIC\n"
    M2 = "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM2:FAULT:KERNEL_UD:PASS\nM2:FAIL:KERNEL_EXCEPTION\n"

    def failed(self, text, fixture, alive=True):
        return {n for n, ok in run_hw_inventory.evaluate_fixture(text, fixture, alive).items() if not ok}

    def test_expected_terminal_logs_pass(self):
        self.assertEqual(self.failed(self.PANIC, "panic"), set())
        self.assertEqual(self.failed(self.M2, "m2-fault"), set())
        self.assertEqual(self.failed("M2:PASS\nM1:PASS\nHWINV:BEGIN\nHWINV:FAIL:FIXTURE\n", "hwinv-fail"), set())
        self.assertEqual(self.failed("BdsDxe: loading\nM1:FAIL:KERNEL_OPEN\n", "missing-kernel"), set())

    def test_each_check_detects_its_own_deviation(self):
        self.assertEqual(self.failed(self.PANIC, "panic", alive=False), {"halted_without_exit_port"})
        self.assertEqual(self.failed(self.PANIC.replace("M2:PANIC_LINE:0000000310\n", ""), "panic"),
                         {"fixture_markers_in_order"})
        self.assertEqual(self.failed(self.PANIC + "HWINV:PASS\n", "panic"), {"no_hwinv_pass"})
        # M2:FAIL is a protocol failure marker since HW-INV v3.
        self.assertNotIn("protocol_failure_recognised", self.failed(self.M2, "m2-fault"))
        self.assertIn("M2:FAIL", hw_inventory.FAILURES)
        self.assertEqual(self.failed("M2:FAULT:KERNEL_UD:PASS\nM2:FAIL_X\n".replace("M2:FAIL_X", "OTHER"), "m2-fault"),
                         {"fixture_markers_in_order", "protocol_failure_recognised"})

    def test_full_line_markers_need_an_exact_line(self):
        # A longer line that merely starts with a full-line marker must not satisfy it.
        for good, bad in (("M2:PASS\n", "M2:PASSED\n"), ("HWINV:BEGIN\n", "HWINV:BEGIN_X\n"),
                          ("HWINV:FAIL:FIXTURE\n", "HWINV:FAIL:FIXTURE2\n")):
            log = "M2:PASS\nM1:PASS\nHWINV:BEGIN\nHWINV:FAIL:FIXTURE\n".replace(good, bad, 1)
            self.assertIn("fixture_markers_in_order", self.failed(log, "hwinv-fail"), bad)
        self.assertIn("fixture_markers_in_order",
                      self.failed(self.PANIC.replace("BOOTINFO:OK\n", "BOOTINFO:OK_X\n"), "panic"))
        self.assertIn("fixture_markers_in_order",
                      self.failed("BdsDxe: loading\nM1:FAIL:KERNEL_OPEN_X\n", "missing-kernel"))

    def test_value_markers_need_a_value(self):
        self.assertIn("fixture_markers_in_order",
                      self.failed(self.PANIC.replace("M2:PANIC_LINE:0000000310\n", "M2:PANIC_LINE:\n"), "panic"))
        self.assertTrue(run_hw_inventory.marker_matches("M2:PANIC_LINE:0000000310", "M2:PANIC_LINE:"))
        self.assertFalse(run_hw_inventory.marker_matches(" M2:PASS", "M2:PASS"))


class SerialTimeoutFixtureTest(unittest.TestCase):
    """H06-T7: the timed-out write leaves a truncated block that the protocol rejects."""
    LOG = "M2:PASS\nM1:PASS\nHWINV:BEGIN\nHWINV:FIXTURE:SERIAL_TIMEOUT_ARMED\n"

    def failed(self, text, alive=True):
        return {n for n, ok in run_hw_inventory.evaluate_fixture(text, "serial-timeout", alive).items() if not ok}

    def test_truncated_log_passes(self):
        self.assertEqual(self.failed(self.LOG), set())

    def test_each_check_detects_its_own_deviation(self):
        self.assertEqual(self.failed(self.LOG, alive=False), {"halted_without_exit_port"})
        # The write after the injection reached the UART: the timeout did not fire.
        self.assertEqual(self.failed(self.LOG + "HWINV:CPUID:VENDOR=AuthenticAMD:HYPERVISOR=0\n"),
                         {"silent_after_injection"})
        # A failure marker is not the expected rejection reason for a silent timeout.
        self.assertEqual(self.failed(self.LOG + "HWINV:FAIL:X\n"),
                         {"silent_after_injection", "protocol_failure_recognised"})
        # A repeated armed marker must not hide output that followed the first one.
        self.assertEqual(self.failed(self.LOG + "HWINV:CPUID:X\nHWINV:FIXTURE:SERIAL_TIMEOUT_ARMED\n"),
                         {"silent_after_injection"})
        self.assertEqual(self.failed(self.LOG.replace("HWINV:FIXTURE:SERIAL_TIMEOUT_ARMED\n", "")),
                         {"fixture_markers_in_order", "silent_after_injection"})

    def test_cpu_halted_reads_hmp_info_registers(self):
        class Control:
            def __init__(self, reply):
                self.reply = reply

            def command(self, name, arguments=None):
                assert (name, arguments) == ("human-monitor-command", {"command-line": "info registers"})
                return self.reply

        self.assertTrue(run_hw_inventory.cpu_halted(Control("EFL=00000006 [-----P-] CPL=0 II=0 A20=1 SMM=0 HLT=1\r\n")))
        self.assertFalse(run_hw_inventory.cpu_halted(Control("EFL=00000006 [-----P-] CPL=0 II=0 A20=1 SMM=0 HLT=0\r\n")))


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
