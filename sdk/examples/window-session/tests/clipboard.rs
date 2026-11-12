//! Real Queue offer/ACK contracts, not fabricated clipboard action grants.
//! These host tests do not replace native input, focus or crash acceptance.
use nieuw_os_input_client::clipboard::{
    Header, Operation, Reply, Request, Status, MAX_PAYLOAD, MAX_TEXT,
};
use nieuw_os_input_core::{wire, CancelReason, Event};
use nieuw_os_window_session::{
    clipboard::{clear_gesture, copy_gesture, Broker, ClearAction, ClearChord},
    input::Queue,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

const PIDS: [u64; 2] = [42, 43];
const EPOCH: u64 = 17;
fn queues() -> [Queue; 2] {
    core::array::from_fn(|i| Queue::new(PIDS[i], i as u64 + 1, EPOCH).unwrap())
}
fn key(physical: PhysicalKey) -> KeyboardEvent {
    KeyboardEvent {
        key: physical,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_control: true,
            ..KeyboardModifiers::default()
        },
    }
}
fn input(
    queue: &mut Queue,
    index: usize,
    operation: wire::Operation,
    id: u64,
    arg0: u64,
) -> [u8; wire::SIZE] {
    queue
        .handle(
            PIDS[index],
            &wire::Request {
                operation,
                id,
                epoch: EPOCH,
                arg0,
                arg1: 0,
            }
            .encode(),
        )
        .unwrap()
}
fn offer(
    queue: &mut Queue,
    index: usize,
    key: KeyboardEvent,
    operation: Operation,
    id: u64,
) -> Header {
    queue.push(Event::Key(key)).unwrap();
    let bytes = input(queue, index, wire::Operation::Peek, id, 0);
    Header {
        operation,
        request: id + 1,
        epoch: EPOCH,
        sequence: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
        generation: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
    }
}
fn ack(queue: &mut Queue, index: usize, header: Header) {
    input(
        queue,
        index,
        wire::Operation::Ack,
        header.request + 1,
        header.sequence,
    );
}
fn copy(broker: &mut Broker, queues: &mut [Queue; 2], text: &str) {
    broker.clear(queues).unwrap();
    let header = offer(&mut queues[0], 0, key(PhysicalKey::C), Operation::Copy, 1);
    let pending = broker.accept(0, 0, PIDS[0], &queues[0], Request { header, text });
    assert_eq!(broker.reply(&pending, 0, &queues[0]).status, Status::Ok);
    ack(&mut queues[0], 0, header);
}
fn status(
    broker: &mut Broker,
    queue: &Queue,
    index: usize,
    focused: usize,
    sender: u64,
    header: Header,
) -> Status {
    let request = Request {
        header,
        text: if header.operation == Operation::Copy {
            "wrong"
        } else {
            ""
        },
    };
    let pending = broker.accept(index, focused, sender, queue, request);
    broker.reply(&pending, focused, queue).status
}

#[test]
fn cross_app_copy_paste_requires_two_real_gestures_and_one_use_per_offer() {
    let mut queues = queues();
    let mut broker = Broker::new();
    copy(&mut broker, &mut queues, "2+3");
    queues[0].cancel(CancelReason::FocusLost).unwrap();
    broker.revoke(&queues).unwrap();
    let header = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
    let pending = broker.accept(1, 1, PIDS[1], &queues[1], Request { header, text: "" });
    let response = broker.reply(&pending, 1, &queues[1]);
    let mut bytes = [0; MAX_PAYLOAD];
    let length = response.encode(&mut bytes).unwrap();
    let decoded = Reply::decode(&bytes[..length]).unwrap();
    assert_eq!(decoded.header, header);
    assert_eq!(decoded.status, Status::Ok);
    assert_eq!(decoded.text, "2+3");
    assert_eq!(
        status(
            &mut broker,
            &queues[1],
            1,
            1,
            PIDS[1],
            Header {
                request: header.request + 1,
                ..header
            }
        ),
        Status::Used
    );
    ack(&mut queues[1], 1, header);
    assert_eq!(broker.reply(&pending, 1, &queues[1]).status, Status::Denied);
    assert!(broker.reply(&pending, 1, &queues[1]).text.is_empty());
}

#[test]
fn pointer_leave_never_authorizes_clipboard_or_changes_focus_generation() {
    let mut queues = queues();
    let generation = queues[0].generation();
    queues[0].push(Event::PointerLeave).unwrap();
    let bytes = input(&mut queues[0], 0, wire::Operation::Peek, 1, 0);
    assert_eq!(bytes[40], 4);
    assert!(bytes[41..48].iter().chain(&bytes[64..]).all(|b| *b == 0));
    for operation in [Operation::Copy, Operation::Paste] {
        let header = Header {
            operation,
            request: 2,
            epoch: EPOCH,
            sequence: 1,
            generation,
        };
        assert!(!queues[0].authorizes_clipboard(PIDS[0], header));
        assert!(!queues[0].authorizes_clipboard(PIDS[1], header));
    }
    assert_eq!(queues[0].generation(), generation);
    input(&mut queues[0], 0, wire::Operation::Ack, 2, 1);
    assert!(!queues[0].has_pending());
}

#[test]
fn sender_focus_role_epoch_sequence_generation_and_offered_state_are_authority() {
    for wrong in 0..10 {
        let mut queues = queues();
        let mut broker = Broker::new();
        copy(&mut broker, &mut queues, "private-selection");
        let mut header = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
        let mut sender = PIDS[1];
        let mut focused = 1;
        let mut index = 1;
        match wrong {
            0 => sender = PIDS[0],
            1 => focused = 0,
            2 => index = 2,
            3 => header.epoch += 1,
            4 => header.sequence += 1,
            5 => header.generation += 1,
            6 => header.operation = Operation::Copy,
            7 => ack(&mut queues[1], 1, header),
            8 => queues[1].retire(),
            9 => header.request = 0,
            _ => unreachable!(),
        }
        assert_eq!(
            status(&mut broker, &queues[1], index, focused, sender, header),
            Status::Denied
        );
    }
    let mut queues = queues();
    let mut broker = Broker::new();
    queues[0].push(Event::Key(key(PhysicalKey::V))).unwrap();
    let header = Header {
        operation: Operation::Paste,
        request: 1,
        epoch: EPOCH,
        sequence: 1,
        generation: 1,
    };
    assert_eq!(
        status(&mut broker, &queues[0], 0, 0, PIDS[0], header),
        Status::Denied
    );
}

#[test]
fn noncanonical_paste_body_cannot_release_data_or_reuse_the_claimed_gesture() {
    let mut queues = queues();
    let mut broker = Broker::new();
    copy(&mut broker, &mut queues, "private-selection");
    let header = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
    let pending = broker.accept(
        1,
        1,
        PIDS[1],
        &queues[1],
        Request {
            header,
            text: "untrusted",
        },
    );
    let reply = broker.reply(&pending, 1, &queues[1]);
    assert_eq!(reply.status, Status::InvalidText);
    assert!(reply.text.is_empty());
    assert_eq!(
        status(&mut broker, &queues[1], 1, 1, PIDS[1], header),
        Status::Used
    );
}

#[test]
fn action_labels_repeat_release_shift_alt_and_super_cannot_create_a_clipboard_grant() {
    for variant in 0..7 {
        let mut queues = queues();
        let mut broker = Broker::new();
        let mut event = key(PhysicalKey::V);
        match variant {
            0 => event.action = KeyAction::Repeat,
            1 => event.action = KeyAction::Release,
            2 => event.modifiers.left_control = false,
            3 => event.modifiers.left_shift = true,
            4 => event.modifiers.left_alt = true,
            5 => event.modifiers.left_super = true,
            6 => event.key = PhysicalKey::C,
            _ => unreachable!(),
        }
        let header = offer(&mut queues[0], 0, event, Operation::Paste, 1);
        assert_eq!(
            status(&mut broker, &queues[0], 0, 0, PIDS[0], header),
            Status::Denied
        );
    }
    assert!(copy_gesture(key(PhysicalKey::C)));
    assert!(!copy_gesture(key(PhysicalKey::V)));
    let mut clear = key(PhysicalKey::Delete);
    assert!(!clear_gesture(clear));
    clear.modifiers.left_shift = true;
    assert!(clear_gesture(clear));
    let mut chord = ClearChord::default();
    assert_eq!(chord.key(clear), ClearAction::Clear);
    clear.action = KeyAction::Repeat;
    assert!(!clear_gesture(clear));
    assert_eq!(chord.key(clear), ClearAction::Consumed);
    clear.action = KeyAction::Release;
    assert!(!clear_gesture(clear));
    assert_eq!(chord.key(clear), ClearAction::Consumed);
}

#[test]
fn consumed_clear_delete_never_turns_into_app_delete_after_modifiers_are_released() {
    let mut chord = ClearChord::default();
    let mut delete = key(PhysicalKey::Delete);
    delete.modifiers.left_shift = true;
    assert_eq!(chord.key(delete), ClearAction::Clear);
    for modifier in [PhysicalKey::LeftControl, PhysicalKey::LeftShift] {
        let event = KeyboardEvent {
            key: modifier,
            action: KeyAction::Release,
            modifiers: KeyboardModifiers::default(),
        };
        assert_eq!(chord.key(event), ClearAction::Pass);
    }
    delete.modifiers = KeyboardModifiers::default();
    delete.action = KeyAction::Repeat;
    assert_eq!(chord.key(delete), ClearAction::Consumed);
    // Focus changes and source cancellation leave the chord instance in place.
    assert_eq!(chord.key(delete), ClearAction::Consumed);
    delete.action = KeyAction::Release;
    assert_eq!(chord.key(delete), ClearAction::Consumed);
    delete.action = KeyAction::Press;
    assert_eq!(chord.key(delete), ClearAction::Pass);
}

#[test]
fn every_delayed_paste_retry_revalidates_without_retaining_plaintext() {
    for cancellation in 0..5 {
        let mut queues = queues();
        let mut broker = Broker::new();
        copy(&mut broker, &mut queues, "private-selection");
        let header = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
        let pending = broker.accept(1, 1, PIDS[1], &queues[1], Request { header, text: "" });
        let before = broker.reply(&pending, 1, &queues[1]).outgoing().unwrap();
        assert_eq!(&before.as_bytes()[128..145], b"private-selection");
        assert!(!format!("{pending:?}").contains("private-selection"));
        drop(before); // an EAGAIN attempt retains only `pending`, never this body.
        let mut focused = 1;
        match cancellation {
            0 => focused = 0,
            1 => queues[1].cancel(CancelReason::FocusLost).unwrap(),
            2 => queues[1].cancel(CancelReason::SourceLost).unwrap(),
            3 => {
                for _ in 0..17 {
                    queues[1]
                        .push(Event::Key(key(PhysicalKey::Digit2)))
                        .unwrap();
                }
            }
            4 => broker.clear(&queues).unwrap(),
            _ => unreachable!(),
        }
        let reply = broker.reply(&pending, focused, &queues[1]);
        assert_eq!(reply.status, Status::Denied);
        assert!(reply.text.is_empty());
        let retry = reply.outgoing().unwrap();
        assert!(retry.as_bytes()[128..].iter().all(|b| *b == 0));
    }
}

#[test]
fn explicit_clear_revokes_already_queued_gestures_but_new_paste_observes_empty() {
    let mut queues = queues();
    let mut broker = Broker::new();
    copy(&mut broker, &mut queues, "5");
    let old = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
    broker.clear(&queues).unwrap();
    assert_eq!(
        status(&mut broker, &queues[1], 1, 1, PIDS[1], old),
        Status::Used
    );
    ack(&mut queues[1], 1, old);
    let new = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 4);
    assert_eq!(
        status(&mut broker, &queues[1], 1, 1, PIDS[1], new),
        Status::Empty
    );
}

#[test]
fn oversized_or_malformed_new_copy_never_silently_pastes_the_previous_item() {
    for malformed in [false, true] {
        let mut queues = queues();
        let mut broker = Broker::new();
        copy(&mut broker, &mut queues, "old private item");
        broker.clear(&queues).unwrap(); // actual next Ctrl+C, before app inspects bytes.
        let header = offer(&mut queues[0], 0, key(PhysicalKey::C), Operation::Copy, 4);
        if malformed {
            let request = Request {
                header,
                text: "new",
            };
            let mut bytes = [0; MAX_PAYLOAD];
            let length = request.encode(&mut bytes).unwrap();
            bytes[64] = 0xff;
            assert!(Request::decode(&bytes[..length]).is_err());
        } else {
            let oversized = "é".repeat(MAX_TEXT / 2 + 1);
            let pending = broker.accept(
                0,
                0,
                PIDS[0],
                &queues[0],
                Request {
                    header,
                    text: &oversized,
                },
            );
            assert_eq!(
                broker.reply(&pending, 0, &queues[0]).status,
                Status::TooLong
            );
        }
        ack(&mut queues[0], 0, header);
        let paste = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
        assert_eq!(
            status(&mut broker, &queues[1], 1, 1, PIDS[1], paste),
            Status::Empty
        );
    }
}

#[test]
fn exact_utf8_boundary_is_preserved_and_a_fresh_compositor_has_no_history() {
    let text = "é".repeat(MAX_TEXT / 2);
    let mut queues = queues();
    let mut broker = Broker::new();
    copy(&mut broker, &mut queues, &text);
    let header = offer(&mut queues[1], 1, key(PhysicalKey::V), Operation::Paste, 1);
    let pending = broker.accept(1, 1, PIDS[1], &queues[1], Request { header, text: "" });
    assert_eq!(broker.reply(&pending, 1, &queues[1]).text, text);
    let mut fresh = Broker::new();
    let pending = fresh.accept(1, 1, PIDS[1], &queues[1], Request { header, text: "" });
    assert_eq!(fresh.reply(&pending, 1, &queues[1]).status, Status::Empty);
}
