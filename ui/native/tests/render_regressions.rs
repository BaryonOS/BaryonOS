use nieuw_os_document_session::volume::ListEntry;
use nieuw_os_materiaal::{
    controller::{Focus, Screen, Status, View},
    render::{Frame, Role, SemanticId, SemanticNode},
    Scale, Viewport,
};

fn view<'a>(viewport: Viewport, objects: &'a [ListEntry]) -> View<'a> {
    View {
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
