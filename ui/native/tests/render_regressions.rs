use nieuw_os_document_session::volume::{Error as StorageError, ListEntry};
use nieuw_os_graphics::{Font, Size, Weight};
use nieuw_os_materiaal::{
    controller::{Error as UiError, Focus, Screen, Status, View},
    design,
    desktop::{Desktop, Page, Target},
    render::{status_label, Frame, Layout, Role, SemanticId, SemanticNode},
    Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH,
};

fn view<'a>(viewport: Viewport, objects: &'a [ListEntry]) -> View<'a> {
    static WORKSPACE: nieuw_os_materiaal::workspace::Workspace =
        nieuw_os_materiaal::workspace::Workspace::new();
    View {
        high_contrast: false,
        draw_pointer: true,
        desktop: None,
        viewport,
        generation: 1,
        dirty: true,
        screen: Screen::Material,
        tool: Default::default(),
        focus: Focus::New,
        status: Status::Ready,
        objects,
        workspace: &WORKSPACE,
        list_loaded: true,
        list_scroll: 0,
        contents: None,
        title: b"Materiaal",
        revision: 0,
        caret: 0,
        caret_trailing: false,
        anchor: 0,
        scroll: 0,
        editable: true,
        can_save: false,
        can_inspect: false,
        saving: false,
        document_dirty: false,
        cursor: None,
        document_menu: None,
    }
}

#[test]
fn semantics_budget_includes_paging_and_preserves_short_output() {
    let mut objects = [ListEntry::EMPTY; 8];
    for (index, object) in objects.iter_mut().enumerate() {
        object.object = index as u64;
        object.name[0] = b'A' + index as u8;
        object.name_length = 1;
    }
    let viewport = Viewport::new(800, 720, Scale::Percent100, 1).unwrap();
    let frame = Frame::new(view(viewport, &objects));

    let mut nodes = [SemanticNode::EMPTY; 16];
    let count = frame.semantics(&mut nodes).unwrap();
    assert_eq!(count, 16);
    assert_eq!(
        nodes[..count]
            .iter()
            .filter(|node| node.role == Role::ListItem)
            .count(),
        6
    );
    assert!(nodes[..count]
        .iter()
        .any(|node| node.id == SemanticId::PreviousPage));
    assert!(nodes[..count]
        .iter()
        .any(|node| node.id == SemanticId::NextPage));
    assert_eq!(nodes[count - 1].id, SemanticId::Status);

    let mut short = [SemanticNode::EMPTY; 15];
    assert!(frame.semantics(&mut short).is_err());
    assert!(short.iter().all(|node| node.id == SemanticId::Empty));
}

#[test]
fn window_semantic_bounds_match_physical_viewport_at_100_and_200_percent() {
    for (scale, width, height) in [(Scale::Percent100, 800, 600), (Scale::Percent200, 800, 600)] {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let frame = Frame::new(view(viewport, &[]));
        let mut nodes = [SemanticNode::EMPTY; 15];
        let count = frame.semantics(&mut nodes).unwrap();
        let window = nodes[..count]
            .iter()
            .find(|node| node.id == SemanticId::Window)
            .unwrap();
        assert_eq!(window.bounds.x, 0);
        assert_eq!(window.bounds.y, 0);
        assert_eq!(window.bounds.width, width);
        assert_eq!(window.bounds.height, height);
    }
}

fn pixel(frame: &Frame<'_>, x: u32, y: u32) -> u32 {
    let mut tile = [0; TILE_PIXELS];
    let rect = frame
        .render_tile(
            x / TILE_WIDTH * TILE_WIDTH,
            y / TILE_HEIGHT * TILE_HEIGHT,
            &mut tile,
        )
        .unwrap();
    tile[((y - rect.y) * rect.width + x - rect.x) as usize]
}

fn image(frame: &Frame<'_>, width: u32, height: u32) -> Vec<u32> {
    let mut pixels = vec![0; (width * height) as usize];
    let mut tile = [0; TILE_PIXELS];
    for rect in frame.tiles() {
        let rect = frame.render_tile(rect.x, rect.y, &mut tile).unwrap();
        for row in 0..rect.height {
            let source = (row * rect.width) as usize;
            let destination = (rect.y + row) as usize * width as usize + rect.x as usize;
            pixels[destination..destination + rect.width as usize]
                .copy_from_slice(&tile[source..source + rect.width as usize]);
        }
    }
    pixels
}

#[test]
fn compact_home_status_text_stays_inside_card() {
    let mut desktop = Desktop::new();
    desktop.activate(Target::Guest);
    desktop.activate(Target::Submit);
    for (width, height, scale) in [(320, 240, Scale::Percent100), (640, 480, Scale::Percent200)] {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let card = desktop
            .controls(viewport)
            .into_iter()
            .flatten()
            .find(|(target, _, _)| *target == Target::Material)
            .unwrap()
            .1;
        let mut baseline_view = view(viewport, &[]);
        baseline_view.desktop = Some(&desktop);
        baseline_view.list_loaded = false;
        let baseline = image(&Frame::new(baseline_view), width, height);
        let factor = scale.factor();
        for status in [
            Status::Ready,
            Status::Failed(UiError::Storage(StorageError::Busy)),
            Status::Failed(UiError::NoFreeObject),
            Status::Failed(UiError::InvalidInput),
        ] {
            let mut state_view = baseline_view;
            state_view.list_loaded = true;
            state_view.status = status;
            let rendered = image(&Frame::new(state_view), width, height);
            assert!(rendered != baseline, "{status:?} must differ from unloaded");
            for (at, (before, after)) in baseline.iter().zip(&rendered).enumerate() {
                if before != after {
                    let x = at as u32 % width;
                    let y = at as u32 / width;
                    assert!(
                        x >= card.x * factor
                            && x < (card.x + card.width) * factor
                            && y >= card.y * factor
                            && y < (card.y + card.height) * factor,
                        "{status:?} changed a pixel outside the card at {x},{y}"
                    );
                }
            }
        }
    }
}

#[test]
fn compact_home_distinguishes_storage_from_other_failures() {
    let mut desktop = Desktop::new();
    desktop.activate(Target::Guest);
    desktop.activate(Target::Submit);
    let viewport = Viewport::new(320, 240, Scale::Percent100, 1).unwrap();
    let mut home_view = view(viewport, &[]);
    home_view.desktop = Some(&desktop);
    home_view.status = Status::Failed(UiError::Storage(StorageError::Busy));
    let storage = image(&Frame::new(home_view), 320, 240);
    home_view.status = Status::Failed(UiError::NoFreeObject);
    let no_space = image(&Frame::new(home_view), 320, 240);
    assert!(
        storage != no_space,
        "non-storage failure must not report a storage fault"
    );
}

#[test]
fn desktop_home_card_follows_the_real_library_state_and_stays_above_the_bar() {
    let mut desktop = Desktop::new();
    desktop.activate(Target::Guest);
    desktop.activate(Target::Submit);
    assert_eq!(desktop.page(), Page::Home);

    for (width, height, scale) in [
        (768, 536, Scale::Percent100),
        (1280, 720, Scale::Percent100),
        (800, 600, Scale::Percent200),
        (320, 240, Scale::Percent100),
        (320, 536, Scale::Percent100),
        (400, 600, Scale::Percent100),
        (639, 536, Scale::Percent100),
        (640, 536, Scale::Percent100),
    ] {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let logical_height = viewport.logical_height();
        let controls = desktop.controls(viewport);
        let card = controls
            .iter()
            .flatten()
            .find(|(target, _, _)| *target == Target::Material)
            .unwrap()
            .1;
        let start = controls
            .iter()
            .flatten()
            .find(|(target, _, _)| *target == Target::Start)
            .unwrap()
            .1;
        let start_label = controls
            .iter()
            .flatten()
            .find(|(target, _, _)| *target == Target::Start)
            .unwrap()
            .2;
        assert!(card.x + card.width <= viewport.logical_width());
        assert!(card.y + card.height <= logical_height - 68);
        assert_eq!(start.width, 44, "the overview mark is a round 44 px target");
        if viewport.logical_width() < 640 {
            assert_eq!(card.height, 96, "narrow home uses the compact card");
            assert_eq!(card.y, 64, "narrow home leaves room for navigation");
        } else if logical_height >= 420 {
            assert_eq!(card.height, 200, "roomy home keeps its full card");
        }
        let mut home_view = view(viewport, &[]);
        home_view.desktop = Some(&desktop);
        let frame = Frame::new(home_view);
        let inside = pixel(
            &frame,
            (card.x + card.width / 2) * scale.factor(),
            (card.y + 8) * scale.factor(),
        );
        let above = pixel(
            &frame,
            (card.x + card.width / 2) * scale.factor(),
            (card.y - 8) * scale.factor(),
        );
        // The card is dark glass over the wallpaper: dark where hit testing says.
        assert!(
            [16, 8, 0]
                .iter()
                .all(|shift| (inside >> shift) & 255 <= 0x60)
                && inside != above,
            "home paint follows the same compact layout as hit testing"
        );
        assert_eq!(start_label, "Zenit");
    }

    let viewport = Viewport::new(768, 536, Scale::Percent100, 1).unwrap();
    let mut empty_view = view(viewport, &[]);
    empty_view.desktop = Some(&desktop);
    let empty = image(&Frame::new(empty_view), viewport.width(), viewport.height());

    let mut unloaded_view = view(viewport, &[]);
    unloaded_view.desktop = Some(&desktop);
    unloaded_view.list_loaded = false;
    let unloaded = image(
        &Frame::new(unloaded_view),
        viewport.width(),
        viewport.height(),
    );

    let mut document = ListEntry::EMPTY;
    document.object = 1;
    document.name[..8].copy_from_slice(b"Tuinplan");
    document.name_length = 8;
    let documents = [document];
    let mut populated_view = view(viewport, &documents);
    populated_view.desktop = Some(&desktop);
    let populated = image(
        &Frame::new(populated_view),
        viewport.width(),
        viewport.height(),
    );
    let card = desktop
        .controls(viewport)
        .into_iter()
        .flatten()
        .find(|(target, _, _)| *target == Target::Material)
        .unwrap()
        .1;
    let content_top = card.y + 140;
    let content_left = card.x + 24;
    let content_right = card.x + card.width - 24;
    assert!(
        (content_top..content_top + 20).any(|y| {
            (content_left..content_right).any(|x| {
                let at = (y * viewport.width() + x) as usize;
                empty[at] != unloaded[at]
            })
        }),
        "the home surface must distinguish an unloaded library from an empty one"
    );
    assert!(
        (content_top..content_top + 20).any(|y| {
            (content_left..content_right).any(|x| {
                let at = (y * viewport.width() + x) as usize;
                empty[at] != populated[at]
            })
        }),
        "the home surface must reflect whether the real library has documents"
    );
}

#[test]
fn primary_buttons_preserve_accent_and_focus_when_hovered_at_both_scales() {
    for scale in [Scale::Percent100, Scale::Percent200] {
        let viewport = Viewport::new(800, 600, scale, 1).unwrap();
        let factor = scale.factor();
        for (screen, focus) in [
            (Screen::Material, Focus::New),
            (Screen::Editor, Focus::Save),
            (Screen::ConfirmClose, Focus::SaveAndBack),
        ] {
            let mut state = view(viewport, &[]);
            state.screen = screen;
            state.focus = focus;
            state.contents = Some("draft");
            state.can_save = true;
            let idle = Frame::new(state);
            let layout = idle.layout();
            let button = match screen {
                Screen::Material => layout.new,
                Screen::Editor => layout.save,
                Screen::ConfirmClose => layout.modal_save,
                _ => unreachable!("this test enumerates only primary screens"),
            };
            state.cursor = Some(((button.x + 8) * factor, (button.y + 8) * factor));
            let hovered = Frame::new(state);
            let x = (button.x + button.width / 2) * factor;
            let fill_y = (button.y + button.height - 8) * factor;
            assert_eq!(pixel(&idle, x, fill_y), design::ACCENT);
            assert_eq!(
                pixel(&hovered, x, fill_y),
                design::ACCENT,
                "{screen:?} loses primary fill at {scale:?}"
            );
            assert_eq!(pixel(&idle, x, button.y * factor), design::ACCENT);
            assert_eq!(pixel(&hovered, x, button.y * factor), design::TEXT);
            let focus_y = (button.y - design::FOCUS_OFFSET) * factor;
            assert_eq!(pixel(&idle, x, focus_y), design::ACCENT);
            assert_eq!(pixel(&hovered, x, focus_y), design::ACCENT);

            let label_y = (button.y + 14) * factor;
            let mut foreground_pixels = 0;
            for y in label_y..label_y + 16 * factor {
                for x in (button.x + 24) * factor..(button.x + button.width - 24) * factor {
                    let original = pixel(&idle, x, y);
                    if original != design::ACCENT {
                        assert_eq!(pixel(&hovered, x, y), original);
                        foreground_pixels += 1;
                    }
                }
            }
            assert!(
                foreground_pixels > 0,
                "primary label must actually be rendered: {screen:?} at {scale:?}"
            );
        }
    }
}

#[test]
fn disabled_primary_and_secondary_hover_keep_their_distinct_styles() {
    for scale in [Scale::Percent100, Scale::Percent200] {
        let viewport = Viewport::new(800, 600, scale, 1).unwrap();
        let factor = scale.factor();
        let mut state = view(viewport, &[]);
        state.editable = false;
        let layout = Frame::new(state).layout();
        for (button, expected) in [
            (layout.new, design::LINE),
            (layout.refresh, design::ACCENT_SOFT),
        ] {
            state.cursor = Some(((button.x + 8) * factor, (button.y + 8) * factor));
            let frame = Frame::new(state);
            assert_eq!(
                pixel(
                    &frame,
                    (button.x + button.width / 2) * factor,
                    (button.y + button.height - 8) * factor
                ),
                expected
            );
        }
    }
}

#[test]
fn taskbar_pinned_app_opens_materiaal_even_after_a_search_miss() {
    use nieuw_os_materiaal::desktop::Action;
    for (width, height, scale) in [
        (320, 240, Scale::Percent100),
        (640, 536, Scale::Percent100),
        (768, 536, Scale::Percent100),
        (1280, 720, Scale::Percent100),
        (800, 600, Scale::Percent200),
    ] {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let mut desktop = Desktop::new();
        desktop.activate(Target::Guest);
        desktop.activate(Target::Submit);
        assert_eq!(desktop.page(), Page::Home);
        let bar: Vec<_> = desktop
            .controls(viewport)
            .into_iter()
            .flatten()
            .filter(|(t, _, _)| {
                matches!(
                    t,
                    Target::Start | Target::Search | Target::Pinned | Target::SignOut
                )
            })
            .map(|(_, r, _)| r)
            .collect();
        assert_eq!(bar.len(), 4);
        for pair in bar.windows(2) {
            assert!(pair[0].x + pair[0].width <= pair[1].x, "{width}x{height}");
        }
        desktop.activate(Target::Search);
        assert!(desktop.set_query("zzz"));
        assert!(!desktop.matches_material());
        assert_eq!(desktop.activate(Target::Pinned), Action::OpenMaterial);
        assert_eq!(desktop.page(), Page::Material);
    }
}

#[test]
fn orbit_dock_is_bounded_on_tiny_viewports() {
    for width in [0_u32, 1, 44, 100, 227, 228, 419, 420, 768] {
        let bar = nieuw_os_materiaal::desktop::dock(width, 240);
        assert!(bar.width <= width, "{width}");
        assert!(bar.x <= width, "{width}");
        assert!(bar.x + bar.width <= width, "{width}");
    }
}

/// Wide Editor (784 logical px and up), compact heights included: the box
/// holds one full text line, keeps the non-compact gap above the status
/// row, and the title clears both the Rename row and the editor.
#[test]
fn wide_editor_keeps_one_line_inside_the_box_at_every_height() {
    let overlaps = |a: nieuw_os_materiaal::Rect, b: nieuw_os_materiaal::Rect| {
        a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
    };
    // Gap the regular (non-compact) layout keeps between editor and status.
    let reference = Layout::for_screen(
        Viewport::new(960, 540, Scale::Percent100, 1).unwrap(),
        Screen::Editor,
    );
    let gap = reference.status.y - (reference.editor.y + reference.editor.height);
    assert_eq!(gap, 16);
    let mut checked = 0;
    for w in [784, 800, 880, 960, 1024, 1280] {
        for h in [240, 250, 260, 287, 288, 300, 359, 360, 510, 540, 720] {
            for scale in [Scale::Percent100, Scale::Percent200] {
                let k = scale.factor();
                let Ok(viewport) = Viewport::new(w * k, h * k, scale, 1) else {
                    continue;
                };
                let l = Layout::for_screen(viewport, Screen::Editor);
                let name = format!("{w}x{h} {scale:?}");
                let (editor, bottom) = (l.editor, l.editor.y + l.editor.height);
                assert!(
                    editor.height >= 32 + design::LINE_HEIGHT,
                    "{name}: {editor:?}"
                );
                assert!(l.rows() >= 1, "{name}");
                assert!(l.text_top() + design::LINE_HEIGHT <= bottom, "{name}");
                assert!(
                    bottom + gap <= l.status.y,
                    "{name}: {editor:?} {:?}",
                    l.status
                );
                let title = l.editor_title();
                assert!(title.width > 0 && title.height > 0, "{name}: {title:?}");
                for button in [l.rename, l.organize, l.history] {
                    // The Rename/Organize/History row, at full width.
                    let row = nieuw_os_materiaal::Rect {
                        x: 0,
                        width: w,
                        ..button
                    };
                    assert!(
                        !overlaps(title, row),
                        "{name}: title {title:?} on row {row:?}"
                    );
                    assert!(editor.y >= button.y + button.height, "{name}");
                }
                assert!(
                    !overlaps(title, editor),
                    "{name}: title {title:?} in {editor:?}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 66, "only {checked} viewports were valid");
}

/// Independent of the renderer's own counter: the break rules of
/// `Paint::wrapped` for body text.
fn wrapped_lines(label: &str, width: u32, scale: Scale) -> u32 {
    let font = Font::new(Size::Body, Weight::Regular, scale == Scale::Percent200);
    let measure = |s: &str| font.measure(s).div_ceil(scale.factor());
    let (mut rest, mut lines) = (label, 0);
    while !rest.is_empty() {
        let mut take = rest.len();
        let mut space = None;
        for (index, ch) in rest.char_indices() {
            if ch.is_whitespace() {
                space = Some(index);
            }
            if measure(&rest[..index + ch.len_utf8()]) > width {
                take = space.filter(|i| *i > 0).unwrap_or(index.max(ch.len_utf8()));
                break;
            }
        }
        rest = rest[take..].trim_start_matches(char::is_whitespace);
        lines += 1;
    }
    lines
}

fn failures() -> impl Iterator<Item = Status> {
    use volume_core::Status as V;
    [
        StorageError::Uncertain,
        StorageError::Conflict,
        StorageError::Closed,
        StorageError::Busy,
        StorageError::ReadOnly,
        StorageError::ContentTooLarge,
        StorageError::Service(V::Full),
        StorageError::Service(V::Quota),
        StorageError::Service(V::Io),
        StorageError::Service(V::Access),
    ]
    .map(|error| Status::Failed(UiError::Storage(error)))
    .into_iter()
    .chain(
        [
            UiError::NoFreeObject,
            UiError::InvalidUtf8,
            UiError::InvalidInput,
        ]
        .map(Status::Failed),
    )
}

/// Compact (< 360 logical px high) viewports, where `Layout::status` holds one line.
const COMPACT: [(u32, u32, Scale); 4] = [
    (320, 240, Scale::Percent100),
    (640, 480, Scale::Percent200),
    (800, 600, Scale::Percent200),
    (1120, 660, Scale::Percent200),
];

/// UX-SR11: the failure status fits, stays on screen and changes no pixel
/// outside its divider and text band compared with a one-line failure.
fn assert_status_fits(state: View<'_>, x: u32, floor: u32) {
    let viewport = state.viewport;
    let factor = viewport.scale().factor();
    let (width, height) = (viewport.width(), viewport.height());
    let base = Layout::for_screen(viewport, state.screen).status;
    let mut one_line = state;
    one_line.status = Status::Failed(UiError::Storage(StorageError::Busy));
    let baseline = image(&Frame::new(one_line), width, height);
    for status in failures() {
        let mut view = state;
        view.status = status;
        let frame = Frame::new(view);
        let area = frame.status_area();
        let label = status_label(status);
        let lines = wrapped_lines(
            label,
            viewport.logical_width() - x - design::PAD,
            viewport.scale(),
        );
        let at = format!("{width}x{height} {:?} {status:?}", state.screen);
        assert!(
            lines * design::LINE_HEIGHT <= area.height,
            "{at}: {lines} lines clipped"
        );
        assert_eq!(
            area.y + area.height,
            base.y + base.height,
            "{at}: not bottom-anchored"
        );
        assert!(area.y + area.height <= viewport.logical_height(), "{at}");
        assert!(
            area.y >= floor + 16,
            "{at}: status meets the content above it"
        );
        let mut nodes = [SemanticNode::EMPTY; 64];
        let count = frame.semantics(&mut nodes).unwrap();
        let node = nodes[..count]
            .iter()
            .find(|n| n.id == SemanticId::Status)
            .unwrap();
        assert_eq!(node.bounds, area.physical(viewport.scale()), "{at}");
        let rendered = image(&frame, width, height);
        let top = area.y.min(base.y).saturating_sub(8) * factor;
        for (index, (before, after)) in baseline.iter().zip(&rendered).enumerate() {
            if before != after {
                let y = index as u32 / width;
                assert!(
                    y >= top,
                    "{at}: pixel changed above the status band at y {y}"
                );
            }
        }
    }
}

#[test]
fn sr11_editor_failure_status_fits_in_compact_layouts() {
    for (width, height, scale) in COMPACT {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let mut state = view(viewport, &[]);
        state.screen = Screen::Editor;
        state.focus = Focus::Document;
        state.contents = Some("Tekst");
        let editor = Layout::for_screen(viewport, Screen::Editor).editor;
        assert_status_fits(state, design::PAD, editor.y + editor.height);
    }
}

#[test]
fn sr11_library_failure_status_fits_below_the_rows() {
    let mut objects = [ListEntry::EMPTY; 8];
    for (index, object) in objects.iter_mut().enumerate() {
        object.object = index as u64;
        object.name[0] = b'A' + index as u8;
        object.name_length = 1;
    }
    let mut long = objects;
    for object in &mut long {
        object.name = [b'W'; 32];
        object.name_length = 32;
    }
    // Logical 240 high (320x240, 640x480 @ 200 %) is excluded: its single list
    // row leaves no room for a second status line (ignored SR11 test).
    let viewports = COMPACT[2..]
        .iter()
        .chain(&[(320, 300, Scale::Percent100), (320, 350, Scale::Percent100)]);
    for (width, height, scale) in viewports {
        let viewport = Viewport::new(*width, *height, *scale, 1).unwrap();
        let layout = Layout::for_screen(viewport, Screen::Material);
        let rows = layout.list_rows();
        let last = layout.object(rows - 1);
        let row_floor = last.y + last.height + 1;
        assert_status_fits(view(viewport, &objects), layout.status.x, row_floor);
        // An unbreakable name wraps past its row: the status grows only below
        // it, else keeps its layout rect (no free space, as at height 240).
        let name = core::str::from_utf8(&long[0].name).unwrap();
        let names = (0..rows)
            .map(|row| {
                let rect = layout.object(row);
                rect.y + 6 + wrapped_lines(name, rect.width - 96, *scale) * design::LINE_HEIGHT
            })
            .max()
            .unwrap();
        for status in failures() {
            let mut state = view(viewport, &long);
            state.status = status;
            let area = Frame::new(state).status_area();
            assert!(
                area == layout.status || area.y >= names.max(row_floor) + 16,
                "{width}x{height} {status:?}: status meets a wrapped name"
            );
        }
    }
}

#[test]
fn sr11_status_keeps_its_rect_where_no_space_is_free() {
    let objects = [ListEntry::EMPTY; 1];
    for (width, height, scale) in &COMPACT[..2] {
        let viewport = Viewport::new(*width, *height, *scale, 1).unwrap();
        let mut state = view(viewport, &objects);
        state.status = Status::Failed(UiError::Storage(StorageError::Conflict));
        assert_eq!(
            Frame::new(state).status_area(),
            Layout::for_screen(viewport, Screen::Material).status
        );
    }
}
