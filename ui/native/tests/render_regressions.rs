use nieuw_os_document_session::volume::ListEntry;
use nieuw_os_materiaal::{
    controller::{Focus, Screen, Status, View},
    design,
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
        tool: Default::default(),
        focus: Focus::New,
        status: Status::Ready,
        objects,
        list_loaded: true,
        list_scroll: 0,
        contents: None,
        title: b"Materiaal",
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

    let mut nodes = [SemanticNode::EMPTY; 15];
    let count = frame.semantics(&mut nodes).unwrap();
    assert_eq!(count, 15);
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

    let mut short = [SemanticNode::EMPTY; 14];
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
