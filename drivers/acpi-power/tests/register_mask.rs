//! Firmware-byte fixtures only; never reads or writes a hardware port.
use nieuw_os_acpi_power::{parse_s5, AcpiTableSnapshot, S5Plan, TableSnapshot};

fn checksum(table: &mut [u8]) {
    let length = table.len() as u32;
    table[4..8].copy_from_slice(&length.to_le_bytes());
    table[9] = 0;
    table[9] = 0u8.wrapping_sub(table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
}

fn plan(first: u8, second: u8, secondary: bool) -> S5Plan {
    let mut fadt = [0; 116];
    fadt[..4].copy_from_slice(b"FACP");
    fadt[40..44].copy_from_slice(&0x2000u32.to_le_bytes());
    fadt[64..68].copy_from_slice(&0x604u32.to_le_bytes());
    if secondary {
        fadt[68..72].copy_from_slice(&0x608u32.to_le_bytes());
    }
    fadt[89] = 2;
    checksum(&mut fadt);
    let mut dsdt = [0; 48];
    dsdt[..4].copy_from_slice(b"DSDT");
    dsdt[36..].copy_from_slice(&[
        0x08, b'_', b'S', b'5', b'_', 0x12, 0x06, 0x02, 0x0a, first, 0x0a, second,
    ]);
    checksum(&mut dsdt);
    parse_s5(AcpiTableSnapshot::new(
        TableSnapshot::new(&fadt, 0x1000),
        TableSnapshot::new(&dsdt, 0x2000),
    ))
    .unwrap()
}

#[test]
fn all_register_values_preserve_non_sleep_bits_and_use_each_banks_own_type() {
    const TYPE: u16 = 0x1c00;
    const ENABLE: u16 = 0x2000;
    const PRESERVED: u16 = 0xc3ff;
    for first in 0..8 {
        for second in 0..8 {
            let plan = plan(first, second, true);
            assert_eq!(plan.pm1a().address(), 0x604);
            assert_eq!(plan.pm1b().unwrap().address(), 0x608);
            for (setting, expected_type) in [(plan.pm1a(), first), (plan.pm1b().unwrap(), second)] {
                assert_eq!(setting.sleep_type(), expected_type);
                for current in 0..=u16::MAX {
                    let prepared = setting.apply_to(current);
                    assert_eq!(prepared & TYPE, u16::from(expected_type) * 1024);
                    assert_eq!(prepared & ENABLE, ENABLE);
                    assert_eq!(prepared & PRESERVED, current & PRESERVED);
                    assert_eq!(setting.apply_to(prepared), prepared);
                }
            }
        }
    }
}

#[test]
fn absent_secondary_bank_does_not_change_primary_preparation() {
    for first in 0..8 {
        for second in 0..8 {
            let primary_only = plan(first, second, false);
            let both = plan(first, second, true);
            assert_eq!(primary_only.pm1b(), None);
            assert_eq!(primary_only.pm1a(), both.pm1a());
            for current in [0, 1, 0x03ff, 0x1c00, 0x2000, 0xc000, u16::MAX] {
                assert_eq!(
                    primary_only.pm1a().apply_to(current),
                    both.pm1a().apply_to(current)
                );
            }
        }
    }
}
