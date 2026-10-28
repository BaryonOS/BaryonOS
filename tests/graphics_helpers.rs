//! Host-only wrapper around the unchanged kernel graphics source.
//!
//! The three pure libraries are real project crates. Paging and serial entry
//! points panic instead of pretending to map hardware or produce a guest log.
//! Port I/O, MMIO writes and bootstrap entry points must never run on the host.
//! This wrapper proves helper logic only, not native mappings or device access.
#![deny(unsafe_op_in_unsafe_fn)]

#[allow(dead_code)]
mod serial {
    pub fn write(_: &[u8]) {
        panic!("host helper test unexpectedly attempted kernel serial output");
    }
    pub fn fail(message: &[u8]) -> ! {
        panic!("kernel graphics rejected input: {:?}", message);
    }
    pub fn exit(_: u32) -> ! {
        panic!("host helper test unexpectedly attempted a guest exit");
    }
}

#[allow(dead_code)]
mod m2 {
    pub const FRAME_COUNT: usize = 65_536;
    pub mod paging {
        pub struct Paging;
        impl Paging {
            pub fn map_framebuffer(
                &mut self,
                _: &mut kernel_core::Frames<{ super::FRAME_COUNT }>,
                _: &boot_protocol::graphics::FramebufferInfo,
                _: &boot_protocol::MemoryMap<'_>,
            ) -> Result<(), kernel_core::Error> {
                panic!("host helper test must not claim a hardware mapping");
            }
        }
    }
}

#[allow(dead_code)]
#[path = "../kernel/src/graphics.rs"]
mod graphics;

#[test]
fn decoder_reset_clears_both_modifier_sides_caps_and_partial_prefixes() {
    use ui_core::{KeyCode, Keyboard, Modifiers};

    // G0 treats controller parity/timeout and keyboard 0xff as a stream error
    // and calls this real decoder's reset. The included kernel tests verify
    // error-to-sentinel conversion; this test verifies the reset consequence.
    for prefix in [&[][..], &[0xe0][..], &[0xe1][..], &[0xe1, 0x1d, 0x45][..]] {
        let mut keyboard = Keyboard::new();
        for byte in [0x2a, 0x36, 0x1d, 0xe0, 0x1d, 0x38, 0xe0, 0x38, 0x3a] {
            keyboard.feed(byte);
        }
        for byte in prefix {
            keyboard.feed(*byte);
        }
        keyboard.reset();
        let next = keyboard
            .feed(0x1e)
            .expect("reset must clear partial prefixes");
        assert_eq!(next.code, KeyCode::Character(b'a'));
        assert!(next.pressed);
        assert_eq!(next.text, Some(b'a'));
        assert_eq!(next.modifiers, Modifiers::default());
        // Reset also releases caps_down: a subsequent real make may toggle it.
        keyboard.feed(0x3a);
        assert_eq!(keyboard.feed(0x1e).unwrap().text, Some(b'A'));
    }
}
