use nieuw_os_image_codec::{Cancellation, Format, Image, Limits, Pixels};
use nieuw_os_image_viewer::{Command, Error, Failure, Selection, State, Viewer, Zoom};
use nieuw_os_input_core::{CancelReason, Event};
use nieuw_os_surface_protocol::{Geometry, Rectangle};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

const PNG: &[u8] = include_bytes!("../../../../services/image-codec/fixtures/alpha.png");
const JPEG: &[u8] = include_bytes!("../../../../services/image-codec/fixtures/progressive.jpg");

fn geometry(width: u16, height: u16) -> Geometry {
    Geometry {
        width: 1920,
        height: 1080,
        epoch: 7,
        assigned: Rectangle {
            x: 100,
            y: 80,
            width,
            height,
        },
    }
}
fn selection() -> Selection {
    Selection {
        object: 0,
        revision: 3,
        byte_budget: 1024 * 1024,
        pixel_budget: 1_000_000,
    }
}
fn limits() -> Limits {
    Limits {
        input_bytes: 1024 * 1024,
        width: 4096,
        height: 4096,
        pixels: 1_000_000,
        work_bytes: 4 * 1024 * 1024,
        metadata_bytes: 65536,
        png_chunks: 1024,
        jpeg_scans: 100,
    }
}
fn viewer() -> Viewer<'static> {
    Viewer::new(geometry(128, 96), 1).unwrap()
}
fn loaded() -> Viewer<'static> {
    let mut viewer = viewer();
    let load = viewer.begin(selection()).unwrap();
    viewer
        .decode(load, selection(), PNG, limits(), &Cancellation::default())
        .unwrap();
    viewer
}
fn image(width: usize, height: usize, pixels: Pixels) -> Image<'static> {
    Image {
        source: b"original",
        format: Format::Png,
        width,
        height,
        pixels,
        has_exif: true,
        has_icc: true,
        metadata_transforms_applied: false,
    }
}
fn mouse(x: u32, y: u32, down: bool, changed: bool) -> Event {
    Event::Mouse {
        input: MouseEvent {
            dx: 0,
            dy: 0,
            buttons: MouseButtons::checked(u8::from(down)).unwrap(),
            changed: MouseButtons::checked(u8::from(changed)).unwrap(),
            x_overflow: false,
            y_overflow: false,
        },
        x,
        y,
    }
}
fn key(key: PhysicalKey, action: KeyAction) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers::default(),
    })
}

#[test]
fn actual_png_and_jpeg_decode_keep_original_samples_and_metadata() {
    for (bytes, format, dimensions) in [
        (PNG, Format::Png, (128, 96)),
        (JPEG, Format::Jpeg, (193, 113)),
    ] {
        let mut v = viewer();
        let ticket = v.begin(selection()).unwrap();
        assert_eq!(v.state(), State::Loading);
        v.decode(
            ticket,
            selection(),
            bytes,
            limits(),
            &Cancellation::default(),
        )
        .unwrap();
        let decoded = v.image().unwrap();
        assert_eq!(decoded.format, format);
        assert_eq!(decoded.source.as_ptr(), bytes.as_ptr());
        assert_eq!((decoded.width, decoded.height), dimensions);
        assert_eq!(v.semantics().image_size, Some(dimensions));
        assert!(!v.semantics().metadata_transforms_applied);
        assert!(!v.semantics().presented);
    }
}

#[test]
fn stale_object_revision_budget_or_late_reply_never_decode_or_replace_current_image() {
    for observed in [
        Selection {
            object: 1,
            ..selection()
        },
        Selection {
            revision: 4,
            ..selection()
        },
        Selection {
            byte_budget: 1,
            ..selection()
        },
        Selection {
            pixel_budget: 1,
            ..selection()
        },
    ] {
        let mut v = viewer();
        let ticket = v.begin(selection()).unwrap();
        assert_eq!(
            v.decode(ticket, observed, PNG, limits(), &Cancellation::default()),
            Err(Error::Load(Failure::Stale))
        );
        assert!(v.image().is_none());
    }
    let mut v = viewer();
    let old = v.begin(selection()).unwrap();
    let current = v
        .begin(Selection {
            revision: 4,
            ..selection()
        })
        .unwrap();
    assert_eq!(
        v.decode(old, selection(), PNG, limits(), &Cancellation::default()),
        Err(Error::ObsoleteLoad)
    );
    v.decode(
        current,
        Selection {
            revision: 4,
            ..selection()
        },
        PNG,
        limits(),
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(v.fail(old, Failure::Io), Err(Error::ObsoleteLoad));
    assert_eq!(v.state(), State::Ready);
}

#[test]
fn real_decode_failures_are_explicit_and_never_publish_pixels() {
    for (bytes, bounded, expected) in [
        (b"not an image".as_slice(), limits(), Failure::Unsupported),
        (&PNG[..PNG.len() / 2], limits(), Failure::Corrupt),
        (
            PNG,
            Limits {
                pixels: 1,
                ..limits()
            },
            Failure::TooLarge,
        ),
        (
            PNG,
            Limits {
                work_bytes: 1,
                ..limits()
            },
            Failure::TooLarge,
        ),
    ] {
        let mut v = loaded();
        let ticket = v.begin(selection()).unwrap();
        assert!(v.image().is_none());
        assert_eq!(
            v.decode(
                ticket,
                selection(),
                bytes,
                bounded,
                &Cancellation::default()
            ),
            Err(Error::Load(expected))
        );
        assert_eq!(v.state(), State::Failed(expected));
        let mut out = [0xdeadbeef; 256];
        assert_eq!(v.render_tile(0, 0, &mut out), Err(Error::NoImage));
        assert_eq!(out, [0xdeadbeef; 256]);
        assert!(!v.semantics().status.is_empty());
        let retry = v.begin(selection()).unwrap();
        v.decode(retry, selection(), PNG, limits(), &Cancellation::default())
            .unwrap();
        assert_eq!(v.state(), State::Ready);
    }
}

#[test]
fn selected_input_and_decoded_pixel_budgets_are_enforced() {
    for selected in [
        Selection {
            byte_budget: 8,
            ..selection()
        },
        Selection {
            pixel_budget: 4,
            ..selection()
        },
    ] {
        let mut v = viewer();
        let ticket = v.begin(selected).unwrap();
        assert_eq!(
            v.decode(ticket, selected, PNG, limits(), &Cancellation::default()),
            Err(Error::Load(Failure::TooLarge))
        );
    }
    let mut v = viewer();
    let selected = Selection {
        pixel_budget: 1,
        ..selection()
    };
    let ticket = v.begin(selected).unwrap();
    assert_eq!(
        v.decoded(ticket, selected, image(2, 1, Pixels::Rgba8(vec![0; 8]))),
        Err(Error::Load(Failure::TooLarge))
    );
}

#[test]
fn malformed_decoded_result_rejected_before_render() {
    for (w, h, pixels) in [
        (0, 1, Pixels::Rgba8(vec![])),
        (1, 1, Pixels::Rgba8(vec![0; 3])),
        (usize::MAX, 2, Pixels::Rgba16(vec![])),
        (1, 1, Pixels::Rgba16(vec![0; 5])),
    ] {
        let mut v = viewer();
        let ticket = v.begin(selection()).unwrap();
        assert_eq!(
            v.decoded(ticket, selection(), image(w, h, pixels)),
            Err(Error::Load(Failure::Corrupt))
        );
        assert!(v.image().is_none());
    }
}

#[test]
fn failure_cancel_revoke_close_clear_pixels_and_invalidate_old_work() {
    for failure in [Failure::Denied, Failure::Io, Failure::OutOfMemory] {
        let mut v = viewer();
        let ticket = v.begin(selection()).unwrap();
        v.fail(ticket, failure).unwrap();
        assert_eq!(v.state(), State::Failed(failure));
        assert!(v.image().is_none());
        assert_eq!(v.fail(ticket, failure), Err(Error::ObsoleteLoad));
    }
    let mut v = viewer();
    let ticket = v.begin(selection()).unwrap();
    v.command(Command::Cancel).unwrap();
    assert_eq!(v.state(), State::Failed(Failure::Cancelled));
    assert_eq!(
        v.decode(ticket, selection(), PNG, limits(), &Cancellation::default()),
        Err(Error::ObsoleteLoad)
    );
    let ticket = v.begin(selection()).unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        v.decode(ticket, selection(), PNG, limits(), &cancelled),
        Err(Error::Load(Failure::Cancelled))
    );
    let mut v = loaded();
    v.revoke().unwrap();
    assert_eq!(v.state(), State::Failed(Failure::Revoked));
    assert!(v.image().is_none());
    assert!(v.selection().is_none());
    v.close().unwrap();
    v.close().unwrap();
    assert_eq!(v.begin(selection()), Err(Error::Closed));
    assert_eq!(v.command(Command::ZoomIn), Err(Error::Closed));
}

#[test]
fn scalar_rgba8_and_rgba16_pixels_alpha_and_letterbox_match_independent_values() {
    for pixels in [
        Pixels::Rgba8(vec![255, 0, 0, 255, 0, 255, 0, 128]),
        Pixels::Rgba16(vec![65535, 0, 0, 65535, 0, 65535, 0, 32896]),
    ] {
        let mut v = Viewer::new(geometry(4, 4), 1).unwrap();
        let ticket = v.begin(selection()).unwrap();
        v.decoded(ticket, selection(), image(2, 1, pixels)).unwrap();
        let mut out = [0xdeadbeef; 17];
        let tile = v.render_tile(0, 0x0000ff, &mut out).unwrap();
        assert_eq!(tile.rectangle().bytes(), 64);
        assert_eq!(&out[..4], &[0x0000ff; 4]);
        assert_eq!(&out[4..8], &[0xff0000, 0xff0000, 0x00807f, 0x00807f]);
        assert_eq!(&out[8..12], &out[4..8]);
        assert_eq!(&out[12..16], &[0x0000ff; 4]);
        assert_eq!(out[16], 0xdeadbeef);
        assert_eq!(v.image().unwrap().source, b"original");
        assert!(!v.semantics().metadata_transforms_applied);
    }
}

#[test]
fn invalid_tile_capacity_and_background_do_not_touch_output() {
    let v = loaded();
    let mut out = [0xdeadbeef; 256];
    assert_eq!(v.render_tile(u64::MAX, 0, &mut out), Err(Error::Tile));
    assert_eq!(
        v.render_tile(0, 0xff000000, &mut out),
        Err(Error::Configuration)
    );
    assert_eq!(v.render_tile(0, 0, &mut out[..2]), Err(Error::Capacity));
    assert_eq!(out, [0xdeadbeef; 256]);
}

#[test]
fn full_frame_requires_ordered_real_ack_hook_and_current_generation() {
    let mut v = loaded();
    let mut out = [0; 256];
    let first = v.render_tile(0, 0, &mut out).unwrap();
    let second = v.render_tile(1, 0, &mut out).unwrap();
    assert_eq!(v.acknowledge(second), Err(Error::ObsoleteFrame));
    v.acknowledge(first).unwrap();
    assert_eq!(v.acknowledge(first), Err(Error::ObsoleteFrame));
    assert!(!v.semantics().presented);
    v.command(Command::ZoomIn).unwrap();
    assert_eq!(v.acknowledge(second), Err(Error::ObsoleteFrame));
    for index in 0..geometry(128, 96).tiles() {
        let tile = v.render_tile(index, 0, &mut out).unwrap();
        v.acknowledge(tile).unwrap();
    }
    assert!(v.semantics().presented);
    v.command(Command::Pan { x: 1, y: 0 }).unwrap();
    assert!(!v.semantics().presented);
}

#[test]
fn zoom_pan_extremes_remain_bounded_and_scale_changes_actual_size() {
    for scale in [1, 2] {
        let mut v = Viewer::new(geometry(32, 24), scale).unwrap();
        let ticket = v.begin(selection()).unwrap();
        v.decoded(
            ticket,
            selection(),
            image(16, 16, Pixels::Rgba8(vec![255; 1024])),
        )
        .unwrap();
        v.command(Command::Actual).unwrap();
        assert_eq!(v.semantics().zoom, Zoom::Percent(100));
        for _ in 0..20 {
            v.command(Command::ZoomIn).unwrap();
        }
        assert_eq!(v.semantics().zoom, Zoom::Percent(800));
        v.command(Command::Pan {
            x: i32::MAX,
            y: i32::MAX,
        })
        .unwrap();
        assert_eq!(v.pan(), (16 * 8 * scale - 32, 16 * 8 * scale - 24));
        v.command(Command::Pan {
            x: i32::MIN,
            y: i32::MIN,
        })
        .unwrap();
        assert_eq!(v.pan(), (0, 0));
        for _ in 0..20 {
            v.command(Command::ZoomOut).unwrap();
        }
        assert_eq!(v.semantics().zoom, Zoom::Percent(25));
        v.command(Command::Fit).unwrap();
        assert_eq!(v.pan(), (0, 0));
        let mut out = [0; 256];
        v.render_tile(0, 0, &mut out).unwrap();
    }
}

#[test]
fn keyboard_only_navigation_and_focus_cancel_never_close_or_invent_drag() {
    let mut v = loaded();
    v.input(key(PhysicalKey::Equal, KeyAction::Press)).unwrap();
    assert_eq!(v.semantics().zoom, Zoom::Percent(200));
    v.input(key(PhysicalKey::Right, KeyAction::Repeat)).unwrap();
    assert!(v.pan().0 > 64);
    v.input(key(PhysicalKey::Home, KeyAction::Release)).unwrap();
    assert_eq!(v.semantics().zoom, Zoom::Percent(200));
    v.input(Event::Cancel(CancelReason::FocusLost)).unwrap();
    assert!(!v.semantics().focused);
    assert_eq!(v.state(), State::Ready);
    v.input(key(PhysicalKey::Home, KeyAction::Press)).unwrap();
    assert_eq!(v.semantics().zoom, Zoom::Fit);
    v.input(key(PhysicalKey::Escape, KeyAction::Press)).unwrap();
    assert_eq!(v.state(), State::Closed);
}

#[test]
fn global_mouse_coordinates_drag_offsets_and_outside_reentry_are_not_clamped_clicks() {
    let mut v = loaded();
    v.command(Command::ZoomIn).unwrap();
    let start = v.pan();
    v.input(mouse(150, 100, true, true)).unwrap();
    v.input(mouse(140, 90, true, false)).unwrap();
    assert_eq!(v.pan(), (start.0 + 10, start.1 + 10));
    v.input(mouse(99, 90, true, false)).unwrap();
    v.input(mouse(180, 90, true, false)).unwrap();
    assert_eq!(v.pan(), (start.0 + 10, start.1 + 10));
    v.input(Event::Cancel(CancelReason::QueueFull)).unwrap();
    v.input(mouse(170, 90, true, false)).unwrap();
    assert_eq!(v.pan(), (start.0 + 10, start.1 + 10));
}

#[test]
fn geometry_selection_and_scale_are_checked() {
    assert!(matches!(
        Viewer::new(geometry(0, 1), 1),
        Err(Error::Configuration)
    ));
    assert!(matches!(
        Viewer::new(geometry(32, 24), 3),
        Err(Error::Configuration)
    ));
    let mut v = viewer();
    for selected in [
        Selection {
            revision: 0,
            ..selection()
        },
        Selection {
            byte_budget: 0,
            ..selection()
        },
        Selection {
            pixel_budget: 0,
            ..selection()
        },
    ] {
        assert_eq!(v.begin(selected), Err(Error::Configuration));
    }
    assert_eq!(v.command(Command::ZoomIn), Err(Error::NoImage));
}

#[test]
fn zoom_direction_from_fit_uses_actual_ratio_including_limits_and_ui_scale() {
    for (width, height, scale, command, expected) in [
        (32, 24, 1, Command::ZoomIn, Zoom::Percent(800)),
        (1024, 768, 1, Command::ZoomOut, Zoom::Fit),
        (32, 24, 2, Command::ZoomIn, Zoom::Percent(400)),
        (32, 24, 2, Command::ZoomOut, Zoom::Percent(100)),
        (1, 1, 1, Command::ZoomIn, Zoom::Fit),
        (1, 1, 1, Command::ZoomOut, Zoom::Percent(800)),
    ] {
        let mut v = Viewer::new(geometry(128, 96), scale).unwrap();
        let ticket = v.begin(selection()).unwrap();
        v.decoded(
            ticket,
            selection(),
            image(width, height, Pixels::Rgba8(vec![255; width * height * 4])),
        )
        .unwrap();
        assert_eq!(v.semantics().zoom, Zoom::Fit);
        v.command(command).unwrap();
        assert_eq!(
            v.semantics().zoom,
            expected,
            "{width}x{height}, {scale}, {command:?}"
        );
    }
}
