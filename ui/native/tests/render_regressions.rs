use nieuw_os_document_session::volume::{Error as StorageError, ListEntry};
use nieuw_os_materiaal::{
    controller::{Error as UiError, Focus, Screen, Status, View},
    design,
    desktop::{Desktop, Page, Target},
    render::{Frame, Role, SemanticId, SemanticNode},
    Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH,
};

fn view<'a>(viewport: Viewport, objects: &'a [ListEntry]) -> View<'a> {
    View {
        draw_pointer: true,
        desktop: None,
        viewport,
        generation: 1,
        dirty: true,
        screen: Screen::Material,
        focus: Focus::New,
        status: Status::Ready,
        objects,
        list_loaded: true,
        list_scroll: 0,
        contents: None,
        title: b"Materiaal",
        query: "",
        field_caret: 0,
        field_anchor: 0,
        caret: 0,
        anchor: 0,
        scroll: 0,
        editable: true,
        can_save: false,
        saving: false,
        document_dirty: false,
        cursor: None,
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
        7
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
        assert_eq!(
            start.width,
            if viewport.logical_width() >= 464 {
                88
            } else {
                44
            }
        );
        if viewport.logical_width() < 640 {
            assert_eq!(card.height, 96, "narrow home uses the compact card");
            assert_eq!(card.y, 64, "narrow home leaves room for navigation");
        } else if logical_height >= 420 {
            assert_eq!(card.height, 200, "roomy home keeps its full card");
        }
        let mut home_view = view(viewport, &[]);
        home_view.desktop = Some(&desktop);
        assert_eq!(
            pixel(
                &Frame::new(home_view),
                (card.x + card.width / 2) * scale.factor(),
                (card.y + 8) * scale.factor()
            ),
            design::SURFACE,
            "home paint follows the same compact layout as hit testing"
        );
        assert_eq!(start_label, "Start");
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
