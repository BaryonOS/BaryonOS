use nieuw_os_usb_core::hid_control::Request;

#[test]
fn boot_protocol_addresses_every_interface_without_a_data_stage() {
    for interface in 0..=u8::MAX {
        assert_eq!(
            Request::set_boot_protocol(interface).setup(),
            [0x21, 0x0b, 0, 0, interface, 0, 0, 0]
        );
    }
}

#[test]
fn every_idle_duration_preserves_interface_and_uses_high_value_byte() {
    for interface in 0..=u8::MAX {
        for duration in 0..=u8::MAX {
            let setup = Request::set_idle(interface, duration).setup();
            assert_eq!(setup[0], 0x21);
            assert_eq!(setup[1], 0x0a);
            assert_eq!(
                u16::from_le_bytes([setup[2], setup[3]]),
                u16::from(duration) << 8
            );
            assert_eq!(
                u16::from_le_bytes([setup[4], setup[5]]),
                u16::from(interface)
            );
            assert_eq!(u16::from_le_bytes([setup[6], setup[7]]), 0);
        }
    }
}
