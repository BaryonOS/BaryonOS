use nieuw_os_graphics::Rect;
use nieuw_os_input_core::{wire, CancelReason, Event};
use nieuw_os_sdk_runtime::{Error, Handle, ReceiveEndpoint};
use nieuw_os_surface_protocol::{
    self as sur, Action, CommitReady, Geometry, Header, Operation, Rectangle,
};
use nieuw_os_window_session::{
    idle::{CompositorState, Idle, IdleIo, InitState},
    input::Queue,
    recovery::{Group, Recovery},
    scene::Damage,
    surface::TrackedReceiver,
};

struct Io {
    now: Result<u64, Error>,
    mask: Result<u16, Error>,
    yielded: Result<(), Error>,
    ticks: usize,
    waits: Vec<(ReceiveEndpoint, u64)>,
    yields: usize,
}
impl Default for Io {
    fn default() -> Self {
        Self {
            now: Ok(40),
            mask: Ok(0),
            yielded: Ok(()),
            ticks: 0,
            waits: Vec::new(),
            yields: 0,
        }
    }
}
impl IdleIo for Io {
    fn ticks(&mut self) -> Result<u64, Error> {
        self.ticks += 1;
        self.now
    }
    fn wait(&mut self, endpoint: ReceiveEndpoint, deadline: u64) -> Result<u16, Error> {
        self.waits.push((endpoint, deadline));
        self.mask
    }
    fn yield_now(&mut self) -> Result<(), Error> {
        self.yields += 1;
        self.yielded
    }
}
fn endpoint() -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(Handle::from_raw(0x7_0000_0012).unwrap())
}

#[test]
fn only_reviewed_abis_wait_and_each_known_legacy_abi_keeps_yield() {
    for minor in 0..=11 {
        let mut idle = Idle::new(0x1_0000_0000 | minor).unwrap();
        let mut io = Io::default();
        idle.pause(true, endpoint(), &mut io).unwrap();
        if minor >= 10 {
            assert_eq!(io.waits, [(endpoint(), 41)]);
            assert_eq!(io.ticks, 1);
            assert_eq!(io.yields, 0);
        } else {
            assert!(io.waits.is_empty());
            assert_eq!(io.ticks, 0);
            assert_eq!(io.yields, 1);
        }
    }
    for abi in [0, 11, 0x1_0000_000c, 0x1_ffff_ffff, 0x2_0000_0000, u64::MAX] {
        assert!(matches!(Idle::new(abi), Err(Error::UnsupportedAbi)));
    }
}

#[test]
fn readiness_is_only_a_hint_and_every_wait_gets_a_fresh_one_tick_bound() {
    let mut idle = Idle::new(0x1_0000_000b).unwrap();
    let mut io = Io::default();
    idle.pause(true, endpoint(), &mut io).unwrap();
    io.now = Ok(45);
    io.mask = Ok(1);
    idle.pause(true, endpoint(), &mut io).unwrap();
    assert_eq!(io.waits, [(endpoint(), 41), (endpoint(), 46)]);
    // The caller's actual receive may still be EAGAIN: no synthetic work or ACK.
    io.mask = Ok(0);
    idle.pause(true, endpoint(), &mut io).unwrap();
    assert_eq!(io.waits[2], (endpoint(), 46));
    assert_eq!(io.yields, 0);
}

#[test]
fn bad_masks_and_failed_authority_checks_fail_closed_without_yield_fallback() {
    for result in [
        Ok(2),
        Ok(u16::MAX),
        Err(Error::InvalidHandle),
        Err(Error::InvalidRights),
        Err(Error::Kernel(-8)),
        Err(Error::Kernel(-1)),
    ] {
        let mut idle = Idle::new(0x1_0000_000b).unwrap();
        let mut io = Io {
            mask: result,
            ..Io::default()
        };
        assert_eq!(
            idle.pause(true, endpoint(), &mut io),
            Err(result.err().unwrap_or(Error::InvalidReturn))
        );
        assert_eq!(io.waits, [(endpoint(), 41)]);
        assert_eq!(io.yields, 0);
    }
}

#[test]
fn failed_overflowing_and_rewound_clocks_never_wait() {
    for now in [Err(Error::Kernel(-1)), Ok(u64::MAX)] {
        let mut idle = Idle::new(0x1_0000_000a).unwrap();
        let mut io = Io {
            now,
            ..Io::default()
        };
        assert_eq!(
            idle.pause(true, endpoint(), &mut io),
            Err(now.err().unwrap_or(Error::InvalidReturn))
        );
        assert!(io.waits.is_empty());
        assert_eq!(io.yields, 0);
    }
    let mut idle = Idle::new(0x1_0000_000b).unwrap();
    let mut io = Io::default();
    idle.pause(true, endpoint(), &mut io).unwrap();
    io.now = Ok(39);
    assert_eq!(
        idle.pause(true, endpoint(), &mut io),
        Err(Error::InvalidReturn)
    );
    assert_eq!(io.waits.len(), 1);
    assert_eq!(io.yields, 0);
}

#[test]
fn busy_path_does_not_touch_clock_or_wait_and_yield_errors_are_preserved() {
    let mut idle = Idle::new(0x1_0000_000b).unwrap();
    let mut io = Io {
        now: Err(Error::Kernel(-1)),
        yielded: Err(Error::Kernel(-2)),
        ..Io::default()
    };
    assert_eq!(
        idle.pause(false, endpoint(), &mut io),
        Err(Error::Kernel(-2))
    );
    assert_eq!(io.ticks, 0);
    assert!(io.waits.is_empty());
    assert_eq!(io.yields, 1);
}

#[test]
fn every_compositor_backlog_and_unproven_empty_pass_prevents_parking() {
    let quiet = CompositorState {
        ingress_empty: true,
        ..CompositorState::default()
    };
    assert!(quiet.quiescent());
    let blocked = [
        CompositorState {
            ingress_empty: false,
            ..quiet
        },
        CompositorState {
            received: true,
            ..quiet
        },
        CompositorState {
            reply_pending: true,
            ..quiet
        },
        CompositorState {
            input_pending: true,
            ..quiet
        },
        CompositorState {
            input_backlog: true,
            ..quiet
        },
        CompositorState {
            surface_pending: true,
            ..quiet
        },
        CompositorState {
            damage_pending: true,
            ..quiet
        },
    ];
    let mut idle = Idle::new(0x1_0000_000b).unwrap();
    let mut io = Io::default();
    for state in blocked {
        assert!(!state.quiescent());
        idle.pause(state.quiescent(), endpoint(), &mut io).unwrap();
    }
    assert!(io.waits.is_empty());
    assert_eq!(io.yields, blocked.len());
}

#[test]
fn boot_missing_ready_recovery_stop_and_control_messages_prevent_init_parking() {
    let quiet = InitState {
        registry_empty: true,
        running: true,
        ready_mask: 31,
        ..InitState::default()
    };
    assert!(quiet.quiescent());
    for state in [
        InitState {
            registry_empty: false,
            ..quiet
        },
        InitState {
            received: true,
            ..quiet
        },
        InitState {
            running: false,
            ..quiet
        },
        InitState {
            recovering: true,
            ..quiet
        },
        InitState {
            recovery_pending: true,
            ..quiet
        },
        InitState {
            stopping: true,
            ..quiet
        },
    ] {
        assert!(!state.quiescent());
    }
    for mask in 0..=u8::MAX {
        assert_eq!(
            InitState {
                ready_mask: mask,
                ..quiet
            }
            .quiescent(),
            mask == 31
        );
    }
}

#[test]
fn every_recovery_phase_blocks_idle_until_both_recovery_states_are_cleared() {
    let old = Group {
        generation: 1,
        surface_epoch: 101,
        input_epoch: 201,
    };
    let fresh = Group {
        generation: 2,
        surface_epoch: 102,
        input_epoch: 202,
    };
    // Even a complete old readiness mask must not make recovery quiescent.
    let state = |recovering, recovery: Option<Recovery>| InitState {
        registry_empty: true,
        running: true,
        ready_mask: 31,
        recovering,
        recovery_pending: recovery.is_some(),
        ..InitState::default()
    };
    let mut recovery = Recovery::begin(old).unwrap();
    let mut phases = Vec::new();
    assert!(recovery.awaiting_pause());
    phases.push(recovery);
    recovery.installed(fresh, 3).unwrap();
    assert!(recovery.awaiting_ready());
    phases.push(recovery);
    recovery.rebind_sent(fresh, 31, 0).unwrap();
    assert!(recovery.awaiting_rebind_ack());
    assert!(!recovery.complete(1));
    phases.push(recovery);
    assert!(recovery.complete(3));
    // Observing both ACKs alone does not clear the retained phase.
    phases.push(recovery);
    let mut idle = Idle::new(0x1_0000_000b).unwrap();
    let mut io = Io::default();
    for phase in phases {
        for recovering in [true, false] {
            let quiet = state(recovering, Some(phase)).quiescent();
            assert!(!quiet);
            idle.pause(quiet, endpoint(), &mut io).unwrap();
        }
    }
    assert!(!state(true, None).quiescent());
    assert!(io.waits.is_empty());
    assert_eq!(io.yields, 8);
    assert!(state(false, None).quiescent());
    idle.pause(state(false, None).quiescent(), endpoint(), &mut io)
        .unwrap();
    assert_eq!(io.waits, [(endpoint(), 41)]);
}

#[test]
fn damage_query_cannot_consume_budgeted_tiles_or_lose_redirty() {
    let mut damage = Damage::new(1280, 16).unwrap();
    assert!(!damage.has_pending());
    damage.mark(Rect::new(0, 0, 1280, 16));
    for _ in 0..32 {
        assert!(damage.has_pending());
        assert!(damage.has_pending());
        assert!(damage.next_tile().is_some());
    }
    assert!(damage.has_pending());
    let mut remaining = 0;
    while damage.next_tile().is_some() {
        remaining += 1;
    }
    assert_eq!(remaining, 48);
    assert!(!damage.has_pending());
    damage.mark(Rect::new(0, 0, 1, 1));
    assert!(damage.has_pending());
    let tile = damage.next_tile().unwrap();
    assert_eq!((tile.x, tile.y, tile.width, tile.height), (0, 0, 32, 8));
    assert!(!damage.has_pending());
}

fn input_request(operation: wire::Operation, id: u64, arg0: u64) -> [u8; wire::SIZE] {
    wire::Request {
        operation,
        id,
        epoch: 7,
        arg0,
        arg1: 0,
    }
    .encode()
}
fn sequence(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes[48..56].try_into().unwrap())
}

#[test]
fn queued_and_offered_input_remains_pending_until_actual_matching_ack() {
    let mut queue = Queue::new(42, 1, 7).unwrap();
    assert!(!queue.has_pending());
    queue.push(Event::Cancel(CancelReason::Resync)).unwrap();
    assert!(queue.has_pending());
    let old = queue
        .handle(42, &input_request(wire::Operation::Peek, 1, 0))
        .unwrap();
    assert!(queue.has_pending());
    queue.cancel(CancelReason::FocusLost).unwrap();
    queue
        .handle(42, &input_request(wire::Operation::Ack, 2, sequence(&old)))
        .unwrap();
    assert!(queue.has_pending());
    let fresh = queue
        .handle(42, &input_request(wire::Operation::Peek, 3, 0))
        .unwrap();
    queue
        .handle(
            42,
            &input_request(wire::Operation::Ack, 4, sequence(&fresh)),
        )
        .unwrap();
    assert!(!queue.has_pending());
    queue.push(Event::Cancel(CancelReason::Resync)).unwrap();
    queue.retire();
    assert!(!queue.has_pending());
}

fn geometry() -> Geometry {
    Geometry {
        width: 64,
        height: 8,
        epoch: 11,
        assigned: Rectangle {
            x: 0,
            y: 0,
            width: 64,
            height: 8,
        },
    }
}
fn header(operation: Operation, request: u64, tile: u64) -> Header {
    Header {
        operation,
        request,
        epoch: 11,
        frame: 1,
        tile,
        arg0: 0,
        arg1: 0,
    }
}
fn begin(request: u64, tile: u64) -> Vec<u8> {
    begin_frame(1, request, tile)
}
fn begin_frame(frame: u64, request: u64, tile: u64) -> Vec<u8> {
    let rectangle = geometry().rectangle(tile).unwrap();
    let mut bytes = Header {
        frame,
        arg0: rectangle.x,
        arg1: rectangle.y,
        ..header(Operation::Begin, request, tile)
    }
    .encode()
    .to_vec();
    bytes.extend_from_slice(&rectangle.width.to_le_bytes());
    bytes.extend_from_slice(&rectangle.height.to_le_bytes());
    bytes.extend_from_slice(&(rectangle.bytes() as u32).to_le_bytes());
    bytes
}
fn staged_tile(
    receiver: &mut TrackedReceiver,
    request: u64,
    tile: u64,
    start: u64,
    now: u64,
) -> CommitReady {
    staged_frame_tile(receiver, 1, request, tile, start, now)
}
fn staged_frame_tile(
    receiver: &mut TrackedReceiver,
    frame: u64,
    request: u64,
    tile: u64,
    start: u64,
    now: u64,
) -> CommitReady {
    assert!(matches!(
        receiver.accept(7, start, &begin_frame(frame, request, tile)),
        Ok(Action::Staged)
    ));
    assert!(receiver.has_pending());
    for offset in (0..sur::TILE_BYTES).step_by(sur::CHUNK) {
        let count = (sur::TILE_BYTES - offset).min(sur::CHUNK);
        let mut chunk = Header {
            frame,
            arg0: offset as u32,
            arg1: count as u32,
            ..header(Operation::Chunk, request, tile)
        }
        .encode()
        .to_vec();
        chunk.resize(sur::HEADER + count, 0);
        assert!(matches!(
            receiver.accept(7, now, &chunk),
            Ok(Action::Staged)
        ));
        assert!(receiver.has_pending());
    }
    match receiver
        .accept(
            7,
            now,
            &Header {
                frame,
                ..header(Operation::Commit, request, tile)
            }
            .encode(),
        )
        .unwrap()
    {
        Action::Commit(token) => token,
        _ => panic!("complete tile did not produce real commit token"),
    }
}

#[test]
fn partial_tile_unblitted_token_and_partial_frame_all_prevent_parking() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    assert!(!receiver.has_pending());
    let token = staged_tile(&mut receiver, 1, 0, 10, 11);
    assert!(receiver.has_pending());
    let reply = receiver
        .commit_with(token, 12, |rect, bytes| {
            assert_eq!(rect, geometry().rectangle(0).unwrap());
            assert_eq!(bytes.len(), 1024);
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(reply.status(), Ok(()));
    assert_eq!(reply.pixels(), 256);
    assert_eq!(receiver.completed_frame(), None);
    assert!(receiver.has_pending());
    let token = staged_tile(&mut receiver, 2, 1, 13, 14);
    assert!(receiver.has_pending());
    assert_eq!(
        receiver
            .commit_with(token, 15, |_, _| Ok::<(), ()>(()))
            .unwrap()
            .status(),
        Ok(())
    );
    assert_eq!(receiver.completed_frame(), Some(1));
    assert!(!receiver.has_pending());
}

#[test]
fn retained_coverage_stays_pending_until_the_whole_new_frame_is_accounted_for() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    for tile in 0..2 {
        let token = staged_tile(&mut receiver, tile + 1, tile, 10 + tile, 10 + tile);
        let reply = receiver
            .commit_with(token, 10 + tile, |_, _| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(reply.status(), Ok(()));
    }
    assert_eq!(receiver.retained_base(), Some(1));
    assert!(!receiver.has_pending());
    for tile in 0..2 {
        let mut request = Header {
            operation: Operation::Retain,
            request: tile + 3,
            frame: 2,
            arg0: 1,
            ..header(Operation::Retain, tile + 3, tile)
        }
        .encode()
        .to_vec();
        request.extend_from_slice(&1u64.to_le_bytes());
        let reply = match receiver.accept(7, 20 + tile, &request).unwrap() {
            Action::Retain(reply) => reply,
            _ => panic!("retained coverage must have its own reply, not a blit token"),
        };
        assert_eq!(reply.pixels(), 0);
        assert_eq!(reply.retained_tiles(), 1);
        assert_eq!(receiver.has_pending(), tile == 0);
        assert_eq!(
            receiver.completed_frame(),
            if tile == 0 { None } else { Some(2) }
        );
    }
}

#[test]
fn completed_old_frame_cannot_make_a_partial_new_frame_idle() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    let may_idle = |surface_pending| {
        CompositorState {
            ingress_empty: true,
            surface_pending,
            ..CompositorState::default()
        }
        .quiescent()
    };
    let mut blits = 0;
    for tile in 0..2 {
        let start = 10 + tile * 3;
        let token = staged_frame_tile(&mut receiver, 1, tile + 1, tile, start, start + 1);
        let reply = receiver
            .commit_with(token, start + 1, |_, _| {
                blits += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(reply.status(), Ok(()));
    }
    assert_eq!(blits, 2);
    assert_eq!(receiver.completed_frame(), Some(1));
    assert!(may_idle(receiver.has_pending()));

    for tile in 0..2 {
        let start = 20 + tile * 3;
        let token = staged_frame_tile(&mut receiver, 2, tile + 3, tile, start, start + 1);
        // BEGIN/chunks/COMMIT token cannot reuse the previous frame's success.
        assert_eq!(receiver.completed_frame(), None);
        assert!(!may_idle(receiver.has_pending()));
        let before = blits;
        let reply = receiver
            .commit_with(token, start + 1, |_, _| {
                blits += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(blits, before + 1);
        assert_eq!(reply.status(), Ok(()));
        assert_eq!(reply.pixels(), 256);
        if tile == 0 {
            assert_eq!(receiver.completed_frame(), None);
            assert!(!may_idle(receiver.has_pending()));
        }
    }
    assert_eq!(blits, 4);
    assert_eq!(receiver.completed_frame(), Some(2));
    assert!(may_idle(receiver.has_pending()));
}

#[test]
fn malformed_wrong_sender_and_ignored_reject_cannot_clear_partial_surface() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    assert!(matches!(
        receiver.accept(7, 10, &begin(1, 0)),
        Ok(Action::Staged)
    ));
    assert!(matches!(
        receiver.accept(8, 11, &[]),
        Err(sur::Error::Access)
    ));
    assert!(matches!(
        receiver.accept(7, 11, &[]),
        Err(sur::Error::Malformed)
    ));
    assert!(receiver.reject(7, &[], sur::Error::Malformed).is_none());
    assert!(receiver
        .reject(8, &begin(1, 0), sur::Error::Access)
        .is_none());
    assert!(receiver.has_pending());
    let reply = receiver
        .reject(7, &begin(1, 0), sur::Error::Malformed)
        .unwrap();
    assert_eq!(reply.status(), Err(sur::Error::Malformed));
    assert_eq!(receiver.completed_frame(), None);
    assert!(!receiver.has_pending());
    receiver.retire();
    assert!(!receiver.has_pending());
    assert!(matches!(
        receiver.accept(7, 12, &begin(2, 0)),
        Err(sur::Error::Closed)
    ));
}

#[test]
fn idle_queries_and_fragments_never_extend_the_original_two_hundred_tick_deadline() {
    for commit_at in [209, 210] {
        let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
        let token = staged_tile(&mut receiver, 1, 0, 10, 209);
        for _ in 0..100 {
            assert!(receiver.has_pending());
        }
        let mut blits = 0;
        let reply = receiver
            .commit_with(token, commit_at, |_, _| {
                blits += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(blits, usize::from(commit_at == 209));
        assert_eq!(
            reply.status(),
            if commit_at == 209 {
                Ok(())
            } else {
                Err(sur::Error::Timeout)
            }
        );
        assert_eq!(receiver.has_pending(), commit_at == 209);
    }
}

#[test]
fn abandoned_stages_expire_without_new_messages_or_a_completed_frame() {
    for completed_base in [false, true] {
        for fragment in 0..3 {
            let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
            if completed_base {
                for tile in 0..geometry().tiles() {
                    let start = 1 + 2 * tile;
                    let token =
                        staged_frame_tile(&mut receiver, 1, tile + 1, tile, start, start + 1);
                    receiver
                        .commit_with(token, start + 1, |_, _| Ok::<(), ()>(()))
                        .unwrap();
                }
                assert_eq!(receiver.completed_frame(), Some(1));
            }
            let frame = if completed_base { 2 } else { 1 };
            let request = geometry().tiles() + 1;
            let token = if fragment == 2 {
                Some(staged_frame_tile(&mut receiver, frame, request, 0, 10, 11))
            } else {
                assert!(matches!(
                    receiver.accept(7, 10, &begin_frame(frame, request, 0)),
                    Ok(Action::Staged)
                ));
                if fragment == 1 {
                    let mut chunk = Header {
                        frame,
                        arg1: 1,
                        ..header(Operation::Chunk, request, 0)
                    }
                    .encode()
                    .to_vec();
                    chunk.push(0);
                    assert!(matches!(receiver.accept(7, 11, &chunk), Ok(Action::Staged)));
                }
                None
            };
            assert!(receiver.expire(209).unwrap().is_none());
            assert!(receiver.has_pending());
            let reply = receiver.expire(210).unwrap().unwrap();
            let expected = Header {
                frame,
                ..header(Operation::Commit, request, 0)
            };
            let reply = sur::Reply::decode(reply.bytes(), expected).unwrap();
            assert_eq!(reply.status(), Err(sur::Error::Timeout));
            assert_eq!(reply.pixels(), 0);
            assert_eq!(receiver.completed_frame(), None);
            assert_eq!(receiver.retained_base(), None);
            assert!(!receiver.has_pending());
            let mut state = CompositorState {
                ingress_empty: true,
                surface_pending: receiver.has_pending(),
                reply_pending: true,
                ..CompositorState::default()
            };
            assert!(!state.quiescent());
            state.reply_pending = false;
            assert!(state.quiescent());
            assert!(receiver.expire(211).unwrap().is_none());
            if let Some(token) = token {
                assert_eq!(
                    receiver
                        .commit_with(token, 211, |_, _| -> Result<(), ()> {
                            panic!("expired token must never blit")
                        })
                        .err(),
                    Some(sur::Error::Token)
                );
            }
            for tile in 0..geometry().tiles() {
                let start = 212 + 2 * tile;
                let token = staged_frame_tile(
                    &mut receiver,
                    frame + 1,
                    request + tile + 1,
                    tile,
                    start,
                    start + 1,
                );
                let reply = receiver
                    .commit_with(token, start + 1, |_, _| Ok::<(), ()>(()))
                    .unwrap();
                assert_eq!(reply.status(), Ok(()));
            }
            assert_eq!(receiver.completed_frame(), Some(frame + 1));
            assert!(!receiver.has_pending());
        }
    }
}

#[test]
fn surface_expiry_propagates_clock_rollback_without_success() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    assert!(matches!(
        receiver.accept(7, 10, &begin(1, 0)),
        Ok(Action::Staged)
    ));
    assert_eq!(receiver.expire(9).err(), Some(sur::Error::Clock));
    assert_eq!(receiver.completed_frame(), None);
    assert!(matches!(
        receiver.accept(7, 11, &begin(2, 0)),
        Err(sur::Error::Closed)
    ));
}

#[test]
fn failed_real_blit_and_info_response_do_not_claim_a_completed_frame() {
    let mut receiver = TrackedReceiver::new(7, geometry(), 200).unwrap();
    let token = staged_tile(&mut receiver, 1, 0, 10, 11);
    assert_eq!(
        receiver
            .commit_with(token, 12, |_, _| Err(()))
            .unwrap()
            .status(),
        Err(sur::Error::Blit)
    );
    let info = Header {
        frame: 0,
        ..header(Operation::Info, 2, 0)
    }
    .encode();
    assert!(matches!(receiver.accept(7, 13, &info), Ok(Action::Info(_))));
    assert_eq!(receiver.completed_frame(), None);
    assert!(!receiver.has_pending());
}
