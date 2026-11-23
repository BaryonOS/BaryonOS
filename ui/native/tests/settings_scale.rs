//! U04 host slice: persisted settings mapped to native presentation, and the
//! 200 % rendering reached through that mapping. Private-RAM host evidence
//! only; no native QEMU run, human test or U04 acceptance.
#[allow(dead_code)]
mod common;
use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_graphics::{Font, Size, Weight};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    a11y::{
        presentation, scale_for_text_percent, Presentation, MAX_TEXT_SCALE_PERCENT,
        MIN_TEXT_SCALE_PERCENT, PERCENT200_THRESHOLD,
    },
    controller::{Controller, InputEnvelope, Screen},
    render::{Frame, Role, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport, TILE_PIXELS,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

#[test]
fn text_scale_boundaries_fail_closed_to_rendered_scales() {
    // Same range as settings-core MIN/MAX_TEXT_SCALE_PERCENT.
    assert_eq!(MIN_TEXT_SCALE_PERCENT, 75);
    assert_eq!(MAX_TEXT_SCALE_PERCENT, 300);
    assert_eq!(PERCENT200_THRESHOLD, 150);
    for (percent, scale) in [
        // Out of range: fail closed to the 100 % default.
        (0, Scale::Percent100),
        (74, Scale::Percent100),
        (301, Scale::Percent100),
        (u16::MAX, Scale::Percent100),
        // In range: nearer rendered step.
        (75, Scale::Percent100),
        (100, Scale::Percent100),
        (125, Scale::Percent100),
        (149, Scale::Percent100),
        (150, Scale::Percent200),
        (200, Scale::Percent200),
        (300, Scale::Percent200),
    ] {
        assert_eq!(scale_for_text_percent(percent), scale, "{percent}");
        assert_eq!(presentation(percent, false).scale, scale, "{percent}");
    }
}

#[test]
fn reduced_motion_passes_through_unchanged() {
    for reduced_motion in [false, true] {
        for percent in [0, 74, 75, 149, 150, 300, 301, u16::MAX] {
            assert_eq!(
                presentation(percent, reduced_motion),
                Presentation {
                    scale: scale_for_text_percent(percent),
                    reduced_motion,
                }
            );
        }
    }
}

fn run(f: fn()) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

/// Text render.rs draws on a button where it differs from the semantic label
/// (render.rs:496-532, 679-765). `None` for icon-only buttons, whose label is
/// an accessible name only. Mirrors render.rs; update both together.
fn drawn_label<'a>(
    node: &SemanticNode<'a>,
    frame: &Frame<'_>,
    logical_width: u32,
) -> Option<&'a str> {
    let layout = frame.layout();
    let wide = logical_width >= 560;
    match node.id {
        SemanticId::Back => None,
        SemanticId::Refresh if wide => None,
        SemanticId::NewDocument => Some(if wide { "Nieuw document" } else { "Nieuw" }),
        SemanticId::CollectionFilter => Some("Collecties"),
        SemanticId::DocumentMenu => Some("···"),
        SemanticId::Rename => Some("Hernoemen"),
        SemanticId::Organize => Some("Ordenen"),
        SemanticId::History => Some("Geschiedenis"),
        // Narrow tool button draws "Lezen" or the longer "Schrijven".
        SemanticId::DocumentTool if layout.tool.width < 156 => Some("Schrijven"),
        _ => Some(node.label),
    }
}

/// Render every tile and check that each workspace button label and the
/// status line fit their drawn bounds, so nothing is elided at 200 %.
fn check_no_clipped_labels(ui: &Controller<'_, Port>, name: &str) {
    let view = ui.view();
    let viewport = view.viewport;
    assert_eq!(viewport.scale(), Scale::Percent200);
    let frame = Frame::new(view);
    let mut tile = [0; TILE_PIXELS];
    for rect in frame.tiles() {
        frame.render_tile(rect.x, rect.y, &mut tile).unwrap();
    }
    let factor = viewport.scale().factor();
    let button = Font::new(Size::Body, Weight::Semibold, true);
    let small = Font::new(Size::Small, Weight::Regular, true);
    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = frame.semantics(&mut nodes).unwrap();
    let mut checked = 0;
    for node in &nodes[..count] {
        let b = node.bounds;
        assert!(
            b.x + b.width <= viewport.width() && b.y + b.height <= viewport.height(),
            "{name}: {:?} outside the viewport",
            node.id
        );
        match node.role {
            // render.rs button(): drawn labels elide beyond width - 16
            // logical px. Measure the text render.rs actually draws.
            Role::Button => {
                let Some(drawn) = drawn_label(node, &frame, viewport.logical_width()) else {
                    continue;
                };
                assert!(
                    button.measure(drawn) <= b.width.saturating_sub(16 * factor),
                    "{name}: {:?} label {drawn:?} clipped in {} px",
                    node.id,
                    b.width
                );
                checked += 1;
            }
            // render.rs status line: elided beyond logical width - 48.
            Role::Status => {
                assert!(
                    small.measure(node.label) <= (viewport.logical_width() - 48) * factor,
                    "{name}: status {:?} clipped",
                    node.label
                );
                checked += 1;
            }
            _ => {}
        }
    }
    assert!(checked > 1, "{name}: nothing checked");
}

#[test]
fn persisted_200_percent_renders_workspace_screens_without_clipped_labels() {
    run(|| {
        for percent in [PERCENT200_THRESHOLD, 200, MAX_TEXT_SCALE_PERCENT] {
            let p = presentation(percent, true);
            let host = Host::new();
            host.borrow_mut().seed_named(0, b"Reis", b"Earlier");
            host.borrow_mut().seed_named(0, b"Reis", b"Current");
            let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                EditMode::ReadWrite,
                Buffers {
                    draft: &mut d,
                    original: &mut o,
                    staging: &mut s,
                },
            );
            ui.resize(Viewport::new(1920, 1080, p.scale, 2).unwrap())
                .unwrap();
            ui.refresh().unwrap();
            check_no_clipped_labels(&ui, &format!("Material at {percent} %"));
            ui.open(0).unwrap();
            check_no_clipped_labels(&ui, &format!("Editor at {percent} %"));
            for (seq, screen) in (1..).zip([Screen::Rename, Screen::Organize, Screen::History]) {
                match screen {
                    Screen::Rename => ui.begin_rename().unwrap(),
                    Screen::Organize => ui.begin_organize().unwrap(),
                    _ => ui.begin_history().unwrap(),
                }
                assert_eq!(ui.view().screen, screen);
                check_no_clipped_labels(&ui, &format!("{screen:?} at {percent} %"));
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence: seq,
                    generation: 1,
                    event: Event::Key(KeyboardEvent {
                        key: PhysicalKey::Escape,
                        action: KeyAction::Press,
                        modifiers: KeyboardModifiers::default(),
                    }),
                })
                .unwrap();
                assert_eq!(ui.view().screen, Screen::Editor);
            }
        }
    });
}
