//! Host contract tests. The callback is a recorder/fault injector, not display
//! hardware. Native PID/capability and actual blit evidence remain separate.
use nieuw_os_surface_protocol::*;

fn geometry() -> Geometry {
    Geometry {
        width: 800,
        height: 600,
        assigned: Rectangle {
            x: 8,
            y: 24,
            width: 784,
            height: 568,
        },
        epoch: 11,
    }
}
fn h(operation: Operation, request: u64, frame: u64, tile: u64) -> Header {
    Header {
        operation,
        request,
        epoch: 11,
        frame,
        tile,
        arg0: 0,
        arg1: 0,
    }
}
fn begin(g: Geometry, request: u64, frame: u64, tile: u64) -> Vec<u8> {
    let r = g.rectangle(tile).unwrap();
    let header = Header {
        arg0: r.x,
        arg1: r.y,
        ..h(Operation::Begin, request, frame, tile)
    };
    let mut out = header.encode().to_vec();
    out.extend_from_slice(&r.width.to_le_bytes());
    out.extend_from_slice(&r.height.to_le_bytes());
    out.extend_from_slice(&(r.bytes() as u32).to_le_bytes());
    out
}
fn chunk(request: u64, frame: u64, tile: u64, offset: usize, bytes: &[u8]) -> Vec<u8> {
    let header = Header {
        arg0: offset as u32,
        arg1: bytes.len() as u32,
        ..h(Operation::Chunk, request, frame, tile)
    };
    let mut out = header.encode().to_vec();
    out.extend_from_slice(bytes);
    out
}
fn ready(
    receiver: &mut Receiver,
    g: Geometry,
    request: u64,
    frame: u64,
    tile: u64,
    pixels: &[u8],
) -> CommitReady {
    assert!(matches!(
        receiver.accept(7, 1, &begin(g, request, frame, tile)),
        Ok(Action::Staged)
    ));
    for (offset, data) in pixels.chunks(CHUNK).enumerate() {
        assert!(matches!(
            receiver.accept(7, 1, &chunk(request, frame, tile, offset * CHUNK, data)),
            Ok(Action::Staged)
        ));
    }
    match receiver
        .accept(7, 1, &h(Operation::Commit, request, frame, tile).encode())
        .unwrap()
    {
        Action::Commit(token) => token,
        _ => panic!("no complete token"),
    }
}

#[test]
fn info_and_complete_tile_bind_assigned_bounds_and_only_callback_can_ack() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    let info = h(Operation::Info, 1, 0, 0);
    let reply = match receiver.accept(7, 0, &info.encode()).unwrap() {
        Action::Info(reply) => reply,
        _ => panic!(),
    };
    assert_eq!(
        Reply::decode(reply.bytes(), info).unwrap().geometry(),
        Ok(g)
    );
    let pixels = [0x20, 0x40, 0x80, 0].repeat(256);
    let token = ready(&mut receiver, g, 2, 1, 0, &pixels);
    let mut calls = 0;
    let reply = receiver
        .commit_with(token, 1, |r, bytes| {
            calls += 1;
            assert_eq!(
                r,
                Rectangle {
                    x: 8,
                    y: 24,
                    width: 32,
                    height: 8
                }
            );
            assert_eq!(bytes, pixels);
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(calls, 1);
    let reply = Reply::decode(reply.bytes(), h(Operation::Commit, 2, 1, 0)).unwrap();
    assert_eq!(reply.status(), Ok(()));
    assert_eq!(reply.pixels(), 256);
}

#[test]
fn failed_blit_cannot_create_positive_ack() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    let token = ready(&mut receiver, g, 1, 1, 0, &[0; 1024]);
    let reply = receiver
        .commit_with(token, 1, |_, _| Err::<(), ()>(()))
        .unwrap();
    assert_eq!(reply.status(), Err(Error::Blit));
    assert_eq!(reply.pixels(), 0);
}

#[test]
fn wrong_actor_epoch_and_replayed_request_do_not_grant_a_surface() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    let bytes = begin(g, 1, 1, 0);
    assert!(matches!(receiver.accept(8, 0, &bytes), Err(Error::Access)));
    assert!(receiver.reject(8, &bytes, Error::Access).is_none());
    let mut wrong = bytes.clone();
    wrong[16..24].copy_from_slice(&12u64.to_le_bytes());
    assert!(matches!(receiver.accept(7, 0, &wrong), Err(Error::Epoch)));
    let token = ready(&mut receiver, g, 1, 1, 0, &[0; 1024]);
    receiver
        .commit_with(token, 1, |_, _| Ok::<(), ()>(()))
        .unwrap();
    assert!(matches!(receiver.accept(7, 4, &bytes), Err(Error::Replay)));
}

#[test]
fn fragment_offset_length_duplicates_and_incomplete_commit_are_not_success() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    receiver.accept(7, 0, &begin(g, 1, 1, 0)).unwrap();
    assert!(matches!(
        receiver.accept(7, 1, &chunk(1, 1, 0, 0, &[0; 207])),
        Err(Error::Length)
    ));
    let bytes = chunk(1, 1, 0, 0, &[0; 208]);
    receiver.accept(7, 1, &bytes).unwrap();
    assert!(matches!(receiver.accept(7, 1, &bytes), Err(Error::Length)));
    assert!(matches!(
        receiver.accept(7, 1, &h(Operation::Commit, 1, 1, 0).encode()),
        Err(Error::Length)
    ));
    let reply = receiver.reject(7, &bytes, Error::Length).unwrap();
    assert_eq!(reply.status(), Err(Error::Length));
    assert!(receiver.reject(7, &bytes, Error::Length).is_none());
    assert!(matches!(
        receiver.accept(7, 2, &begin(g, 1, 1, 0)),
        Err(Error::Replay)
    ));
}

#[test]
fn noncanonical_pixels_and_geometry_cannot_reach_blit_callback() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    let mut malformed = begin(g, 1, 1, 0);
    malformed[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        receiver.accept(7, 0, &malformed),
        Err(Error::Grid)
    ));
    receiver.accept(7, 0, &begin(g, 1, 1, 0)).unwrap();
    let mut pixels = [0; 1024];
    pixels[1023] = 1;
    for (i, data) in pixels.chunks(CHUNK).enumerate() {
        receiver
            .accept(7, 0, &chunk(1, 1, 0, i * CHUNK, data))
            .unwrap();
    }
    assert!(matches!(
        receiver.accept(7, 0, &h(Operation::Commit, 1, 1, 0).encode()),
        Err(Error::Pixels)
    ));
    let outside = Geometry {
        assigned: Rectangle {
            x: 799,
            y: 0,
            width: 2,
            height: 1,
        },
        ..g
    };
    assert!(Receiver::new(7, outside, 100).is_err());
}

#[test]
fn grid_has_exact_short_edges_no_overlap_and_new_frame_starts_at_zero() {
    let g = Geometry {
        assigned: Rectangle {
            x: 8,
            y: 24,
            width: 35,
            height: 9,
        },
        ..geometry()
    };
    assert_eq!(g.tiles(), 4);
    let mut receiver = Receiver::new(7, g, 100).unwrap();
    let mut rectangles = Vec::new();
    for tile in 0..4 {
        let r = g.rectangle(tile).unwrap();
        let token = ready(&mut receiver, g, tile + 1, 1, tile, &vec![0; r.bytes()]);
        receiver
            .commit_with(token, 1, |r, _| {
                rectangles.push(r);
                Ok::<(), ()>(())
            })
            .unwrap();
    }
    assert_eq!(
        rectangles,
        vec![
            Rectangle {
                x: 8,
                y: 24,
                width: 32,
                height: 8
            },
            Rectangle {
                x: 40,
                y: 24,
                width: 3,
                height: 8
            },
            Rectangle {
                x: 8,
                y: 32,
                width: 32,
                height: 1
            },
            Rectangle {
                x: 40,
                y: 32,
                width: 3,
                height: 1
            }
        ]
    );
    assert_eq!(receiver.completed_frame(), Some(1));
    assert!(matches!(
        receiver.accept(7, 1, &begin(g, 5, 1, 0)),
        Err(Error::Grid)
    ));
    assert!(matches!(
        receiver.accept(7, 1, &begin(g, 5, 2, 1)),
        Err(Error::Grid)
    ));
    let token = ready(&mut receiver, g, 5, 2, 0, &[0; 1024]);
    receiver
        .commit_with(token, 1, |_, _| Ok::<(), ()>(()))
        .unwrap();
    assert!(matches!(
        receiver.accept(7, 4, &begin(g, 6, 1, 1)),
        Err(Error::Grid)
    ));
}

#[test]
fn timeout_retirement_and_clock_rollback_invalidate_pending_completion() {
    let g = geometry();
    let mut receiver = Receiver::new(7, g, 10).unwrap();
    let token = ready(&mut receiver, g, 1, 1, 0, &[0; 1024]);
    let expired = receiver.expire(11).unwrap().unwrap();
    assert_eq!(expired.status(), Err(Error::Timeout));
    assert!(matches!(
        receiver.commit_with(token, 11, |_, _| Ok::<(), ()>(())),
        Err(Error::Token)
    ));
    assert!(receiver.expire(12).unwrap().is_none());
    assert!(matches!(
        receiver.accept(7, 0, &begin(g, 2, 2, 0)),
        Err(Error::Clock)
    ));
    assert!(matches!(
        receiver.accept(7, 13, &begin(g, 2, 2, 0)),
        Err(Error::Closed)
    ));
}

#[test]
fn hostile_header_and_reply_bindings_are_rejected() {
    let request = h(Operation::Info, 1, 0, 0);
    let good = request.encode();
    for at in [0, 4, 6, 8, 16, 24, 32, 40, 44] {
        let mut bytes = good;
        bytes[at] ^= 0x80;
        assert!(Header::decode(&bytes).is_err() || at == 8 || at == 16);
    }
    for length in 0..HEADER {
        assert!(Header::decode(&good[..length]).is_err());
    }
    let mut receiver = Receiver::new(7, geometry(), 100).unwrap();
    let reply = match receiver.accept(7, 0, &good).unwrap() {
        Action::Info(reply) => reply,
        _ => panic!(),
    };
    for at in [0, 4, 6, 8, 16, 24, 32, 40] {
        let mut bytes = reply.bytes().to_vec();
        bytes[at] ^= 0x80;
        assert!(Reply::decode(&bytes, request).is_err());
    }
    let mut bytes = reply.bytes().to_vec();
    bytes[64..68].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Reply::decode(&bytes, request).unwrap().geometry().is_err());
}

#[test]
fn known_expiry_after_malformed_input_never_completes_a_ready_tile() {
    let g = Geometry {
        assigned: Rectangle {
            x: 8,
            y: 24,
            width: 32,
            height: 8,
        },
        ..geometry()
    };
    let mut receiver = Receiver::new(7, g, 10).unwrap();
    let token = ready(&mut receiver, g, 1, 1, 0, &[0; 1024]);
    assert!(receiver.accept(7, 12, b"malformed").is_err());
    let mut calls = 0;
    let completion = receiver.commit_with(token, 12, |_, _| {
        calls += 1;
        Ok::<(), ()>(())
    });
    assert!(match &completion {
        Err(_) => true,
        Ok(reply) => reply.status().is_err(),
    });
    assert_eq!(calls, 0);
    assert_eq!(receiver.completed_frame(), None);
}

#[test]
fn fresh_commit_admission_clock_rejects_expired_token_without_other_messages() {
    let g = Geometry {
        assigned: Rectangle {
            x: 8,
            y: 24,
            width: 32,
            height: 8,
        },
        ..geometry()
    };
    let mut receiver = Receiver::new(7, g, 10).unwrap();
    let token = ready(&mut receiver, g, 1, 1, 0, &[0; 1024]);
    let mut calls = 0;
    let response = receiver
        .commit_with(token, 11, |_, _| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(response.status(), Err(Error::Timeout));
    assert_eq!(response.pixels(), 0);
    assert_eq!(calls, 0);
    assert_eq!(receiver.completed_frame(), None);
}

fn small_geometry() -> Geometry {
    Geometry {
        assigned: Rectangle {
            width: 64,
            height: 8,
            ..geometry().assigned
        },
        ..geometry()
    }
}
fn retain(request: u64, frame: u64, tile: u64, count: u32, base: u64) -> Vec<u8> {
    let mut bytes = Header {
        arg0: count,
        ..h(Operation::Retain, request, frame, tile)
    }
    .encode()
    .to_vec();
    bytes.extend_from_slice(&base.to_le_bytes());
    bytes
}
fn write_tile(receiver: &mut Receiver, g: Geometry, request: u64, frame: u64, tile: u64) {
    let token = ready(
        receiver,
        g,
        request,
        frame,
        tile,
        &vec![0; g.rectangle(tile).unwrap().bytes()],
    );
    assert_eq!(
        receiver
            .commit_with(token, 1, |_, _| Ok::<(), ()>(()))
            .unwrap()
            .status(),
        Ok(())
    );
}
fn initial_frame(receiver: &mut Receiver, g: Geometry) {
    for tile in 0..g.tiles() {
        write_tile(receiver, g, tile + 1, 1, tile);
    }
    assert_eq!(receiver.retained_base(), Some(1));
}

#[test]
fn retained_mode_is_explicit_and_requires_an_initial_complete_frame() {
    let g = small_geometry();
    for enabled in [false, true] {
        let mut receiver = if enabled {
            Receiver::new_retained(7, g, 10).unwrap()
        } else {
            Receiver::new(7, g, 10).unwrap()
        };
        let bytes = retain(1, 1, 0, 2, 1);
        assert!(matches!(receiver.accept(7, 1, &bytes), Err(Error::Access)));
        assert_eq!(receiver.retained_base(), None);
        assert_eq!(receiver.completed_frame(), None);
    }
}

#[test]
fn retained_ranges_have_exact_coverage_and_separate_ack_counts() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    let bytes = retain(3, 2, 0, 1, 1);
    let expected = Header::decode(&bytes).unwrap();
    let reply = match receiver.accept(7, 1, &bytes).unwrap() {
        Action::Retain(reply) => reply,
        _ => panic!("retain is not a new commit"),
    };
    let reply = Reply::decode(reply.bytes(), expected).unwrap();
    assert_eq!(reply.status(), Ok(()));
    assert_eq!(reply.retained_tiles(), 1);
    assert_eq!(reply.pixels(), 0);
    assert_eq!(receiver.completed_frame(), None);
    assert_eq!(receiver.retained_base(), Some(1));
    let mut wrong_count = reply.bytes().to_vec();
    wrong_count[44..48].copy_from_slice(&2u32.to_le_bytes());
    assert!(Reply::decode(&wrong_count, expected).is_err());
    assert!(matches!(
        receiver.accept(7, 1, &retain(4, 2, 1, 1, 1)),
        Ok(Action::Retain(_))
    ));
    assert_eq!(receiver.completed_frame(), Some(2));
    assert_eq!(receiver.retained_base(), Some(2));
}

#[test]
fn changed_tiles_survive_partial_supersedes_until_a_complete_new_baseline() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    write_tile(&mut receiver, g, 3, 2, 0);
    assert_eq!(receiver.retained_base(), Some(1));
    // Superseding D must rewrite C's modified tile even if D's model happens
    // to want the original B pixels. The untouched other tile may be retained.
    write_tile(&mut receiver, g, 4, 3, 0);
    assert!(matches!(
        receiver.accept(7, 1, &retain(5, 3, 1, 1, 1)),
        Ok(Action::Retain(_))
    ));
    assert_eq!(receiver.retained_base(), Some(3));
    assert!(matches!(
        receiver.accept(7, 1, &retain(6, 4, 0, 2, 3)),
        Ok(Action::Retain(_))
    ));
    assert_eq!(receiver.completed_frame(), Some(4));
}

#[test]
fn retaining_a_tainted_tile_is_negative_and_requires_new_full_resynchronization() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    write_tile(&mut receiver, g, 3, 2, 0);
    let bytes = retain(4, 3, 0, 2, 1);
    assert!(matches!(receiver.accept(7, 1, &bytes), Err(Error::Grid)));
    let reply = receiver.reject(7, &bytes, Error::Grid).unwrap();
    let reply = Reply::decode(reply.bytes(), Header::decode(&bytes).unwrap()).unwrap();
    assert_eq!(reply.status(), Err(Error::Grid));
    assert_eq!(reply.retained_tiles(), 0);
    assert_eq!(receiver.retained_base(), None);
    assert_eq!(receiver.completed_frame(), None);
    // Finishing the abandoned frame does not reestablish a baseline.
    assert!(matches!(
        receiver.accept(7, 1, &begin(g, 5, 2, 1)),
        Err(Error::Grid)
    ));
    write_tile(&mut receiver, g, 5, 3, 0);
    write_tile(&mut receiver, g, 6, 3, 1);
    assert_eq!(receiver.retained_base(), Some(3));
}

#[test]
fn failed_callback_after_retained_coverage_cannot_complete_or_restore_the_base() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    receiver.accept(7, 1, &retain(3, 2, 0, 1, 1)).unwrap();
    let token = ready(&mut receiver, g, 4, 2, 1, &[0; 1024]);
    let mut changed_backing = false;
    let reply = receiver
        .commit_with(token, 1, |_, _| {
            changed_backing = true;
            Err::<(), ()>(())
        })
        .unwrap();
    assert!(changed_backing);
    assert_eq!(reply.status(), Err(Error::Blit));
    assert_eq!(receiver.retained_base(), None);
    assert_eq!(receiver.completed_frame(), None);
    assert!(receiver.accept(7, 1, &begin(g, 5, 2, 1)).is_err());
    write_tile(&mut receiver, g, 5, 3, 0);
    write_tile(&mut receiver, g, 6, 3, 1);
    assert_eq!(receiver.retained_base(), Some(3));
}

#[test]
fn retain_canonical_ranges_identity_and_base_are_enforced() {
    let g = small_geometry();
    for (count, base, tile, frame, error) in [
        (0, 1, 0, 2, Error::Malformed),
        (257, 1, 0, 2, Error::Malformed),
        (1, 0, 0, 2, Error::Malformed),
        (3, 1, 0, 2, Error::Grid),
        (1, 1, u64::MAX, 2, Error::Grid),
        (1, 1, 1, 2, Error::Grid),
        (1, 2, 0, 2, Error::Replay),
        (1, 1, 0, 1, Error::Grid),
    ] {
        let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
        initial_frame(&mut receiver, g);
        assert!(
            matches!(receiver.accept(7, 1, &retain(3, frame, tile, count, base)), Err(e) if e == error)
        );
    }
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    let good = retain(3, 2, 0, 2, 1);
    assert!(matches!(receiver.accept(8, 1, &good), Err(Error::Access)));
    let mut wrong_epoch = good.clone();
    wrong_epoch[16..24].copy_from_slice(&12u64.to_le_bytes());
    assert!(matches!(
        receiver.accept(7, 1, &wrong_epoch),
        Err(Error::Epoch)
    ));
    assert_eq!(receiver.retained_base(), Some(1));
    let mut bad = good.clone();
    bad[44] = 0xff;
    assert!(Header::decode(&bad).is_err());
    assert!(Header::decode(&good[..55]).is_err());
    assert!(matches!(
        receiver.accept(7, 1, &good),
        Ok(Action::Retain(_))
    ));
    assert!(matches!(receiver.accept(7, 1, &good), Err(Error::Replay)));
}

#[test]
fn retained_timeout_and_retirement_remove_the_baseline() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    let token = ready(&mut receiver, g, 3, 2, 0, &[0; 1024]);
    assert_eq!(
        receiver.expire(11).unwrap().unwrap().status(),
        Err(Error::Timeout)
    );
    assert_eq!(receiver.retained_base(), None);
    assert_eq!(receiver.completed_frame(), None);
    assert!(receiver
        .commit_with(token, 11, |_, _| Ok::<(), ()>(()))
        .is_err());
    receiver.retire();
    assert_eq!(receiver.retained_base(), None);
    assert!(matches!(
        receiver.accept(7, 12, &retain(4, 3, 0, 2, 1)),
        Err(Error::Closed)
    ));
    assert_eq!(
        Receiver::new_retained(7, g, 10).unwrap().retained_base(),
        None
    );
}

fn solid(g: Geometry, request: u64, frame: u64, tile: u64, color: u32) -> Vec<u8> {
    let mut bytes = begin(g, request, frame, tile);
    bytes[6..8].copy_from_slice(&(Operation::SolidBegin as u16).to_le_bytes());
    bytes.extend_from_slice(&color.to_le_bytes());
    bytes
}

#[test]
fn solid_stages_exact_short_edge_pixels_and_still_requires_commit_callback() {
    let g = Geometry {
        assigned: Rectangle {
            width: 3,
            height: 1,
            ..geometry().assigned
        },
        ..geometry()
    };
    let bytes = solid(g, 1, 1, 0, 0x0055_aa33);
    let mut denied = Receiver::new(7, g, 10).unwrap();
    assert!(matches!(denied.accept(7, 1, &bytes), Err(Error::Access)));
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    assert!(matches!(receiver.accept(7, 1, &bytes), Ok(Action::Staged)));
    assert_eq!(receiver.completed_frame(), None);
    let token = match receiver
        .accept(7, 1, &h(Operation::Commit, 1, 1, 0).encode())
        .unwrap()
    {
        Action::Commit(token) => token,
        _ => panic!(),
    };
    let reply = receiver
        .commit_with(token, 1, |rect, pixels| {
            assert_eq!(rect.width, 3);
            assert_eq!(pixels, [0x33, 0xaa, 0x55, 0].repeat(3));
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(reply.status(), Ok(()));
    assert_eq!(reply.pixels(), 3);
    assert_eq!(receiver.completed_frame(), Some(1));
    assert_eq!(receiver.retained_base(), Some(1));

    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    assert!(matches!(
        receiver.accept(7, 1, &solid(g, 1, 1, 0, 0xff55_aa33)),
        Err(Error::Pixels)
    ));
    assert!(Header::decode(&bytes[..59]).is_err());
}

#[test]
fn solid_commit_expiry_cannot_invoke_callback_or_establish_a_baseline() {
    let g = small_geometry();
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    receiver
        .accept(7, 1, &solid(g, 1, 1, 0, 0x0012_3456))
        .unwrap();
    let token = match receiver
        .accept(7, 1, &h(Operation::Commit, 1, 1, 0).encode())
        .unwrap()
    {
        Action::Commit(token) => token,
        _ => panic!(),
    };
    let mut calls = 0;
    let reply = receiver
        .commit_with(token, 11, |_, _| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(reply.status(), Err(Error::Timeout));
    assert_eq!(calls, 0);
    assert_eq!(receiver.retained_base(), None);
    assert_eq!(receiver.completed_frame(), None);
}

#[test]
fn maximum_geometry_and_run_cover_last_bitmap_word_without_unbounded_ranges() {
    let g = Geometry {
        width: 1920,
        height: 1080,
        assigned: Rectangle {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        epoch: 11,
    };
    assert_eq!(g.tiles() as usize, MAX_TILES);
    let mut receiver = Receiver::new_retained(7, g, 10).unwrap();
    initial_frame(&mut receiver, g);
    let mut tile = 0;
    let mut request = g.tiles() + 1;
    while tile < g.tiles() {
        let count = (g.tiles() - tile).min(u64::from(MAX_RETAIN_TILES)) as u32;
        assert!(matches!(
            receiver.accept(7, 1, &retain(request, 2, tile, count, 1)),
            Ok(Action::Retain(_))
        ));
        tile += u64::from(count);
        request += 1;
    }
    assert_eq!(receiver.completed_frame(), Some(2));
    assert_eq!(receiver.retained_base(), Some(2));
}
