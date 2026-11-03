//! Canonical INP1 application replies. No unsafe enum casts or hardware decode.
use super::{Binding, Offer, ProtocolError, Reply, ServiceError};
use nieuw_os_input_core::{
    wire::{Operation, REPLY, SIZE, TAG, VERSION},
    CancelReason, Event,
};
use nieuw_os_sdk_runtime::Received;
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

fn u64at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn u32at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn i16at(bytes: &[u8], at: usize) -> i16 {
    i16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn zeros(bytes: &[u8]) -> bool {
    bytes.iter().all(|value| *value == 0)
}

// Explicit supported physical-key variants. Matching their public repr values
// is safe even if declaration order changes; unknown values never transmute.
const KEYS: &[PhysicalKey] = &[
    PhysicalKey::Escape,
    PhysicalKey::Digit1,
    PhysicalKey::Digit2,
    PhysicalKey::Digit3,
    PhysicalKey::Digit4,
    PhysicalKey::Digit5,
    PhysicalKey::Digit6,
    PhysicalKey::Digit7,
    PhysicalKey::Digit8,
    PhysicalKey::Digit9,
    PhysicalKey::Digit0,
    PhysicalKey::Minus,
    PhysicalKey::Equal,
    PhysicalKey::Backspace,
    PhysicalKey::Tab,
    PhysicalKey::Q,
    PhysicalKey::W,
    PhysicalKey::E,
    PhysicalKey::R,
    PhysicalKey::T,
    PhysicalKey::Y,
    PhysicalKey::U,
    PhysicalKey::I,
    PhysicalKey::O,
    PhysicalKey::P,
    PhysicalKey::LeftBracket,
    PhysicalKey::RightBracket,
    PhysicalKey::Enter,
    PhysicalKey::LeftControl,
    PhysicalKey::A,
    PhysicalKey::S,
    PhysicalKey::D,
    PhysicalKey::F,
    PhysicalKey::G,
    PhysicalKey::H,
    PhysicalKey::J,
    PhysicalKey::K,
    PhysicalKey::L,
    PhysicalKey::Semicolon,
    PhysicalKey::Quote,
    PhysicalKey::Backquote,
    PhysicalKey::LeftShift,
    PhysicalKey::Backslash,
    PhysicalKey::Z,
    PhysicalKey::X,
    PhysicalKey::C,
    PhysicalKey::V,
    PhysicalKey::B,
    PhysicalKey::N,
    PhysicalKey::M,
    PhysicalKey::Comma,
    PhysicalKey::Period,
    PhysicalKey::Slash,
    PhysicalKey::RightShift,
    PhysicalKey::KeypadMultiply,
    PhysicalKey::LeftAlt,
    PhysicalKey::Space,
    PhysicalKey::CapsLock,
    PhysicalKey::F1,
    PhysicalKey::F2,
    PhysicalKey::F3,
    PhysicalKey::F4,
    PhysicalKey::F5,
    PhysicalKey::F6,
    PhysicalKey::F7,
    PhysicalKey::F8,
    PhysicalKey::F9,
    PhysicalKey::F10,
    PhysicalKey::NumLock,
    PhysicalKey::ScrollLock,
    PhysicalKey::Keypad7,
    PhysicalKey::Keypad8,
    PhysicalKey::Keypad9,
    PhysicalKey::KeypadSubtract,
    PhysicalKey::Keypad4,
    PhysicalKey::Keypad5,
    PhysicalKey::Keypad6,
    PhysicalKey::KeypadAdd,
    PhysicalKey::Keypad1,
    PhysicalKey::Keypad2,
    PhysicalKey::Keypad3,
    PhysicalKey::Keypad0,
    PhysicalKey::KeypadDecimal,
    PhysicalKey::IsoBackslash,
    PhysicalKey::F11,
    PhysicalKey::F12,
    PhysicalKey::KeypadEnter,
    PhysicalKey::RightControl,
    PhysicalKey::KeypadDivide,
    PhysicalKey::PrintScreen,
    PhysicalKey::RightAlt,
    PhysicalKey::Home,
    PhysicalKey::Up,
    PhysicalKey::PageUp,
    PhysicalKey::Left,
    PhysicalKey::Right,
    PhysicalKey::End,
    PhysicalKey::Down,
    PhysicalKey::PageDown,
    PhysicalKey::Insert,
    PhysicalKey::Delete,
    PhysicalKey::LeftSuper,
    PhysicalKey::RightSuper,
    PhysicalKey::Pause,
    PhysicalKey::Break,
    PhysicalKey::SysRq,
    PhysicalKey::Menu,
];
fn key(value: u8) -> Result<PhysicalKey, ProtocolError> {
    KEYS.iter()
        .copied()
        .find(|key| *key as u8 == value)
        .ok_or(ProtocolError::Event)
}
fn status(value: u64) -> Option<ServiceError> {
    Some(match value {
        2 => ServiceError::Configuration,
        3 => ServiceError::Access,
        4 => ServiceError::UnexpectedCaps,
        5 => ServiceError::Malformed,
        6 => ServiceError::WrongEpoch,
        7 => ServiceError::WrongRole,
        8 => ServiceError::StaleRequest,
        9 => ServiceError::UnknownClient,
        10 => ServiceError::ActorDead,
        11 => ServiceError::NotSynchronized,
        12 => ServiceError::StreamLost,
        13 => ServiceError::ControllerFault,
        14 => ServiceError::DecodeFault,
        15 => ServiceError::QueueFull,
        16 => ServiceError::StaleAck,
        17 => ServiceError::CounterExhausted,
        18 => ServiceError::Halted,
        _ => return None,
    })
}
fn modifiers(bits: u16) -> KeyboardModifiers {
    KeyboardModifiers {
        left_shift: bits & 1 != 0,
        right_shift: bits & 2 != 0,
        left_control: bits & 4 != 0,
        right_control: bits & 8 != 0,
        left_alt: bits & 16 != 0,
        right_alt: bits & 32 != 0,
        left_super: bits & 64 != 0,
        right_super: bits & 128 != 0,
        caps_lock: bits & 256 != 0,
        num_lock: bits & 512 != 0,
        scroll_lock: bits & 1024 != 0,
    }
}

pub(super) fn decode(
    message: &Received,
    binding: Binding,
    operation: Operation,
    id: u64,
) -> Result<Reply, ProtocolError> {
    if message.sender() != binding.service_pid
        || message.tag() != TAG
        || message.cap_count() != 0
        || message.payload().len() != SIZE
    {
        return Err(ProtocolError::Envelope);
    }
    let b = message.payload();
    if &b[..4] != b"INP1"
        || u16::from_le_bytes([b[4], b[5]]) != VERSION
        || u16::from_le_bytes([b[6], b[7]]) != operation as u16 | REPLY
        || u64at(b, 8) != id
        || u64at(b, 16) != binding.epoch
        || u64at(b, 32) != binding.client_id
    {
        return Err(ProtocolError::Header);
    }
    let result = u64at(b, 24);
    if result != 0 {
        if !zeros(&b[40..]) {
            return Err(ProtocolError::Body);
        }
        return if result == 1 && operation == Operation::Peek {
            Ok(Reply::Empty)
        } else {
            status(result)
                .map(Reply::Failure)
                .ok_or(ProtocolError::Body)
        };
    }
    if operation == Operation::Ack {
        return if zeros(&b[40..]) {
            Ok(Reply::Acknowledged)
        } else {
            Err(ProtocolError::Body)
        };
    }
    let sequence = u64at(b, 48);
    let generation = u64at(b, 56);
    if sequence == 0 || generation == 0 || !zeros(&b[72..]) {
        return Err(ProtocolError::Body);
    }
    let event = match b[40] {
        1 => {
            if b[43] != 0 || !zeros(&b[46..48]) || !zeros(&b[64..]) {
                return Err(ProtocolError::Body);
            }
            let action = match b[41] {
                1 => KeyAction::Press,
                2 => KeyAction::Repeat,
                3 => KeyAction::Release,
                4 => KeyAction::Pulse,
                _ => return Err(ProtocolError::Event),
            };
            let key = key(b[42])?;
            if (action == KeyAction::Pulse) != (key == PhysicalKey::Pause) {
                return Err(ProtocolError::Event);
            }
            let bits = u16::from_le_bytes([b[44], b[45]]);
            if bits & !0x07ff != 0 {
                return Err(ProtocolError::Event);
            }
            Event::Key(KeyboardEvent {
                key,
                action,
                modifiers: modifiers(bits),
            })
        }
        2 => {
            if b[41] & !3 != 0 {
                return Err(ProtocolError::Event);
            }
            let buttons = MouseButtons::checked(b[42]).ok_or(ProtocolError::Event)?;
            let changed = MouseButtons::checked(b[43]).ok_or(ProtocolError::Event)?;
            let dx = i16at(b, 44);
            let dy = i16at(b, 46);
            let x_overflow = b[41] & 1 != 0;
            let y_overflow = b[41] & 2 != 0;
            let x = u32at(b, 64);
            let y = u32at(b, 68);
            if !(-256..=255).contains(&dx)
                || !(-256..=255).contains(&dy)
                || (x_overflow && dx != 0)
                || (y_overflow && dy != 0)
                || x >= binding.width
                || y >= binding.height
            {
                return Err(ProtocolError::Event);
            }
            Event::Mouse {
                input: MouseEvent {
                    dx,
                    dy,
                    buttons,
                    changed,
                    x_overflow,
                    y_overflow,
                },
                x,
                y,
            }
        }
        3 => {
            if !zeros(&b[42..48]) || !zeros(&b[64..]) {
                return Err(ProtocolError::Body);
            }
            Event::Cancel(match b[41] {
                1 => CancelReason::FocusLost,
                2 => CancelReason::SourceLost,
                3 => CancelReason::QueueFull,
                4 => CancelReason::Resync,
                5 => CancelReason::SourceDead,
                6 => CancelReason::CompositorDead,
                _ => return Err(ProtocolError::Event),
            })
        }
        4 => {
            if !zeros(&b[41..48]) || !zeros(&b[64..]) {
                return Err(ProtocolError::Body);
            }
            Event::PointerLeave
        }
        _ => return Err(ProtocolError::Event),
    };
    Ok(Reply::Event(Offer {
        epoch: binding.epoch,
        sequence,
        generation,
        event,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use capability_core::Message;
    use nieuw_os_input_core::wire::{self, Request};
    use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, SendEndpoint};

    fn binding() -> Binding {
        Binding {
            epoch: 7,
            service_pid: 10,
            client_id: 4,
            width: 1280,
            height: 720,
            send: SendEndpoint::from_handle(Handle::from_raw((1 << 32) | 1).unwrap()),
            receive: ReceiveEndpoint::from_handle(Handle::from_raw((1 << 32) | 2).unwrap()),
        }
    }

    fn response() -> Message {
        let mut message = Message::new();
        message.sender_pid = 10;
        message.tag = TAG;
        message.len = SIZE as u32;
        message.data[..SIZE].copy_from_slice(&wire::response(
            Request {
                operation: Operation::Peek,
                id: 1,
                epoch: 7,
                arg0: 0,
                arg1: 0,
            },
            4,
            Some((2, 3, Event::PointerLeave)),
        ));
        message
    }

    fn parse(message: &Message) -> Result<Reply, ProtocolError> {
        decode(
            &Received::from_bytes(message.encode(), u64::from(message.len)).unwrap(),
            binding(),
            Operation::Peek,
            1,
        )
    }

    #[test]
    fn pointer_leave_has_no_coordinates_buttons_or_authority_payload() {
        let message = response();
        let Reply::Event(offer) = parse(&message).unwrap() else {
            panic!("Expected pointer-leave offer");
        };
        assert_eq!(offer.event(), Event::PointerLeave);
        assert_eq!((offer.sequence(), offer.generation()), (2, 3));
        for offset in (41..48).chain(64..SIZE) {
            let mut malformed = message;
            malformed.data[offset] = 1;
            assert!(parse(&malformed).is_err(), "Accepted payload byte {offset}");
        }
    }

    #[test]
    fn pointer_leave_keeps_existing_sender_correlation_and_type_checks() {
        let mut message = response();
        message.sender_pid = 11;
        assert!(matches!(parse(&message), Err(ProtocolError::Envelope)));
        message = response();
        message.data[8] = 2;
        assert!(parse(&message).is_err());
        message = response();
        message.data[40] = 5;
        assert!(matches!(parse(&message), Err(ProtocolError::Event)));
    }
}
