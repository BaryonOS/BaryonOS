//! Private-RAM host evidence only; no settings router or native acceptance.
#[allow(dead_code)]
mod common;

use common::{controller, Host, Port, INPUT_EPOCH};
use nieuw_os_document_session::volume::ListEntry;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{
        Controller, DocumentMenu, DocumentTool, Error, Focus, InputEnvelope, Screen, Status, View,
    },
    desktop::{Desktop, Target},
    render::{Frame, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Rect, Scale, Viewport, TILE_PIXELS,
};
use std::{cell::RefCell, rc::Rc};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xffffff;
const MUTED: u32 = 0xb8b8b8;
const FOCUS: u32 = 0xffff00;
const SELECTION: u32 = 0x00ffff;
const DANGER: u32 = 0xffb4b4;

fn view<'a>(viewport: Viewport, objects: &'a [ListEntry]) -> View<'a> {
    View {
        draw_pointer: true,
        high_contrast: false,
        desktop: None,
        viewport,
        generation: 1,
        dirty: true,
        screen: Screen::Material,
        tool: DocumentTool::Write,
        focus: Focus::New,
        status: Status::Ready,
        objects,
        list_loaded: true,
        list_scroll: 0,
        contents: None,
        title: b"Mijn document",
        caret: 0,
        anchor: 0,
        scroll: 0,
        editable: true,
        can_save: false,
        saving: false,
        document_dirty: false,
        cursor: None,
        document_menu: None,
    }
}

fn pixels(state: View<'_>) -> Vec<u32> {
    let frame = Frame::new(state);
    let mut image = vec![0; (state.viewport.width() * state.viewport.height()) as usize];
    let mut tile = [0; TILE_PIXELS];
    for tile_rect in frame.tiles() {
        let r = frame
            .render_tile(tile_rect.x, tile_rect.y, &mut tile)
            .unwrap();
        for row in 0..r.height {
            let source = (row * r.width) as usize;
            let destination = ((r.y + row) * state.viewport.width() + r.x) as usize;
            image[destination..destination + r.width as usize]
                .copy_from_slice(&tile[source..source + r.width as usize]);
        }
    }
    image
}

fn exercise_states(viewport: Viewport, mut visit: impl FnMut(&str, View<'_>)) {
    let mut object = ListEntry::EMPTY;
    object.object = 17;
    object.name[..8].copy_from_slice(b"Dagboek.");
    object.name_length = 8;
    let objects = [object];
    let base = view(viewport, &[]);
    visit("empty", base);
    let mut list = view(viewport, &objects);
    list.focus = Focus::Object(0);
    visit("list", list);
    let mut editor = base;
    editor.screen = Screen::Editor;
    editor.contents = Some("WWW Mijn eigen tekst\nTweede regel");
    editor.focus = Focus::Document;
    editor.status = Status::Dirty;
    editor.caret = 3;
    editor.document_dirty = true;
    editor.can_save = true;
    visit("write", editor);
    editor.caret = 0;
    editor.anchor = 3;
    visit("selection", editor);
    editor.tool = DocumentTool::Read;
    editor.editable = false;
    visit("read", editor);
    let mut dialog = editor;
    dialog.screen = Screen::ConfirmClose;
    dialog.focus = Focus::SaveAndBack;
    visit("dialog", dialog);
    let mut menu = editor;
    menu.document_menu = Some(DocumentMenu {
        bounds: nieuw_os_materiaal::Rect {
            x: 24,
            y: 24,
            width: viewport.logical_width() - 48,
            height: 192,
        },
        selected: 0,
    });
    menu.focus = Focus::ContextMenu;
    visit("menu", menu);
    let mut desktop = Desktop::new();
    let mut overlay = base;
    overlay.desktop = Some(&desktop);
    visit("accounts", overlay);
    desktop.activate(Target::Personal);
    let mut overlay = base;
    overlay.desktop = Some(&desktop);
    visit("login", overlay);
    desktop.home();
    let mut overlay = base;
    overlay.desktop = Some(&desktop);
    visit("home", overlay);
    desktop.activate(Target::Search);
    let mut overlay = base;
    overlay.desktop = Some(&desktop);
    visit("search", overlay);
    assert!(desktop.set_query("xyz"));
    let mut overlay = base;
    overlay.desktop = Some(&desktop);
    visit("no-results", overlay);
}

fn viewports() -> impl Iterator<Item = Viewport> {
    [
        (320, 240, Scale::Percent100),
        (640, 480, Scale::Percent200),
        (880, 510, Scale::Percent100),
        (1120, 660, Scale::Percent200),
        (960, 540, Scale::Percent100),
        (1920, 1080, Scale::Percent200),
    ]
    .map(|(w, h, s)| Viewport::new(w, h, s, 1).unwrap())
    .into_iter()
}

#[test]
fn default_source_fingerprints() {
    // Measured on 3799ec63 before changing controller/render, not regenerated
    // from the implementation under test. Includes all 12 states per viewport.
    let expected = [
        0xf6c36261c348347d,
        0x53b2bec58077a9a5,
        0x01449deded10e3f4,
        0xdc1cd3e95d118912,
        0x0c5d4444ae5177a5,
        0xc4aa18c9d64dd471,
    ];
    for (viewport, expected) in viewports().zip(expected) {
        let mut hash = 0xcbf29ce484222325u64;
        exercise_states(viewport, |_, state| {
            for pixel in pixels(state) {
                for byte in pixel.to_le_bytes() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
            }
        });
        assert_eq!(hash, expected, "{viewport:?}");
    }
}

fn luminance(color: u32) -> f64 {
    let channel = |shift| {
        let c = f64::from((color >> shift) & 255u32) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}

fn contrast(a: u32, b: u32) -> f64 {
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn at(image: &[u32], viewport: Viewport, x: u32, y: u32) -> u32 {
    let f = viewport.scale().factor();
    image[(y * f * viewport.width() + x * f) as usize]
}

fn has(image: &[u32], viewport: Viewport, bounds: Rect, color: u32) -> bool {
    let r = bounds.physical(viewport.scale());
    (r.y..(r.y + r.height).min(viewport.height())).any(|y| {
        (r.x..(r.x + r.width).min(viewport.width()))
            .any(|x| image[(y * viewport.width() + x) as usize] == color)
    })
}

fn pair(image: &[u32], viewport: Viewport, bounds: Rect, fg: u32, bg: u32, minimum: f64) {
    // Measure a real glyph-core pixel, not only the requested paint color.
    // Antialiased edges blend into their background and are not text roles.
    let r = bounds.physical(viewport.scale());
    let distance = |color: u32| -> u32 {
        [0, 8, 16]
            .into_iter()
            .map(|shift| {
                ((color >> shift) & 255)
                    .abs_diff((fg >> shift) & 255)
                    .pow(2)
            })
            .sum()
    };
    let actual = (r.y..(r.y + r.height).min(viewport.height()))
        .flat_map(|y| {
            (r.x..(r.x + r.width).min(viewport.width()))
                .map(move |x| image[(y * viewport.width() + x) as usize])
        })
        .min_by_key(|color| distance(*color))
        .unwrap();
    assert!(
        distance(actual) <= 3 * 32 * 32,
        "missing glyph core near {fg:#08x} in {bounds:?} at {viewport:?}: {actual:#08x}"
    );
    assert!(
        has(image, viewport, bounds, bg),
        "missing {bg:#08x} in {bounds:?} at {viewport:?}"
    );
    assert!(contrast(fg, bg) >= minimum, "{fg:#08x}/{bg:#08x}");
    assert!(
        contrast(actual, bg) >= minimum,
        "actual glyph core {actual:#08x}/{bg:#08x}"
    );
    println!(
        "role {fg:#08x}/{bg:#08x} {:.6}:1; actual core {actual:#08x}/{bg:#08x} {:.6}:1",
        contrast(fg, bg),
        contrast(actual, bg)
    );
}

fn label_bounds(r: Rect) -> Rect {
    Rect {
        x: r.x + 16,
        y: r.y + 8,
        width: r.width - 32,
        height: 28,
    }
}

#[test]
fn actual_frame_role_pairs_focus_selection_menu_and_dialog_at_100_and_200() {
    for viewport in viewports() {
        exercise_states(viewport, |name, mut state| {
            state.high_contrast = true;
            let image = pixels(state);
            let layout = Frame::new(state).layout();
            assert_eq!(at(&image, viewport, 0, 0), BLACK, "{name}");
            match name {
                "empty" => {
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: layout.content_left(),
                            y: layout.title_y,
                            width: 240,
                            height: 32,
                        },
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    pair(
                        &image,
                        viewport,
                        label_bounds(layout.new),
                        BLACK,
                        FOCUS,
                        7.0,
                    );
                    assert_eq!(
                        at(
                            &image,
                            viewport,
                            layout.new.x + layout.new.width / 2,
                            layout.new.y - 3
                        ),
                        FOCUS
                    );
                }
                "list" => {
                    let r = layout.object(0);
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 56,
                            y: r.y + 6,
                            width: r.width - 96,
                            height: 24,
                        },
                        BLACK,
                        SELECTION,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                    assert!(contrast(FOCUS, BLACK) >= 3.0);
                    assert!(contrast(SELECTION, BLACK) >= 3.0);
                }
                "write" | "selection" | "read" => {
                    let r = layout.editor;
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 8,
                            y: r.y + 8,
                            width: 3 * layout.character_width() as u32,
                            height: 16,
                        },
                        BLACK,
                        SELECTION,
                        7.0,
                    );
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 8 + 4 * layout.character_width() as u32,
                            y: r.y + 8,
                            width: r.width - 64,
                            height: 16,
                        },
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x - 8, r.y + 8), FOCUS);
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y), WHITE);
                }
                "dialog" => {
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: layout.modal.x + 16,
                            y: layout.modal.y + 16,
                            width: layout.modal.width - 32,
                            height: 24,
                        },
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    pair(
                        &image,
                        viewport,
                        label_bounds(layout.modal_save),
                        BLACK,
                        FOCUS,
                        7.0,
                    );
                    pair(
                        &image,
                        viewport,
                        label_bounds(layout.modal_discard),
                        DANGER,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(
                        at(
                            &image,
                            viewport,
                            layout.modal.x + layout.modal.width / 2,
                            layout.modal.y
                        ),
                        WHITE
                    );
                    assert_eq!(
                        at(
                            &image,
                            viewport,
                            layout.modal_save.x + layout.modal_save.width / 2,
                            layout.modal_save.y - 3
                        ),
                        FOCUS
                    );
                }
                "menu" => {
                    let menu = state.document_menu.unwrap();
                    pair(
                        &image,
                        viewport,
                        label_bounds(menu.item(0)),
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(
                        at(
                            &image,
                            viewport,
                            menu.bounds.x + menu.bounds.width / 2,
                            menu.bounds.y
                        ),
                        WHITE
                    );
                    let r = menu.item(0);
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                }
                "accounts" => {
                    let desktop = state.desktop.unwrap();
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::Guest)
                        .unwrap();
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 72,
                            y: r.y + 7,
                            width: r.width - 100,
                            height: 60,
                        },
                        BLACK,
                        SELECTION,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                }
                "search" => {
                    let desktop = state.desktop.unwrap();
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::Material)
                        .unwrap();
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 60,
                            y: r.y + 8,
                            width: r.width - 80,
                            height: 24,
                        },
                        BLACK,
                        SELECTION,
                        7.0,
                    );
                }
                "login" => {
                    let desktop = state.desktop.unwrap();
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::Password)
                        .unwrap();
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 44,
                            y: r.y + 10,
                            width: r.width - 60,
                            height: 24,
                        },
                        MUTED,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::Submit)
                        .unwrap();
                    pair(&image, viewport, label_bounds(r), BLACK, FOCUS, 7.0);
                }
                "home" => {
                    let desktop = state.desktop.unwrap();
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::Material)
                        .unwrap();
                    let compact = viewport.logical_height() < 420;
                    let x = if compact { 80 } else { 18 };
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + x,
                            y: r.y + if compact { 17 } else { 96 },
                            width: r.width - x - 4,
                            height: 24,
                        },
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y), WHITE);
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                }
                "no-results" => {
                    let desktop = state.desktop.unwrap();
                    let (_, r, _) = desktop
                        .controls(viewport)
                        .into_iter()
                        .flatten()
                        .find(|(t, _, _)| *t == Target::SearchField)
                        .unwrap();
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: r.x + 44,
                            y: r.y + 10,
                            width: r.width - 60,
                            height: 24,
                        },
                        WHITE,
                        BLACK,
                        7.0,
                    );
                    assert_eq!(at(&image, viewport, r.x + r.width / 2, r.y - 3), FOCUS);
                    pair(
                        &image,
                        viewport,
                        Rect {
                            x: (viewport.logical_width()
                                - viewport.logical_width().saturating_sub(48).min(620))
                                / 2
                                + 24,
                            y: if viewport.logical_height() < 420 {
                                120
                            } else {
                                228
                            },
                            width: 240,
                            height: 24,
                        },
                        MUTED,
                        BLACK,
                        7.0,
                    );
                }
                _ => panic!("missing pixel assertions for {name}"),
            }
        });
    }
}

#[test]
fn disabled_hover_status_and_tile_bounds_are_role_based() {
    for viewport in viewports() {
        let mut state = view(viewport, &[]);
        state.high_contrast = true;
        state.editable = false;
        state.focus = Focus::Refresh;
        let layout = Frame::new(state).layout();
        let f = viewport.scale().factor();
        state.cursor = Some(((layout.refresh.x + 8) * f, (layout.refresh.y + 8) * f));
        let image = pixels(state);
        pair(
            &image,
            viewport,
            label_bounds(layout.new),
            MUTED,
            BLACK,
            7.0,
        );
        let dash_x = layout.new.x + nieuw_os_materiaal::design::CONTROL_RADIUS;
        assert_eq!(at(&image, viewport, dash_x, layout.new.y), BLACK);
        assert_eq!(at(&image, viewport, dash_x + 4, layout.new.y), MUTED);
        pair(
            &image,
            viewport,
            label_bounds(layout.refresh),
            BLACK,
            SELECTION,
            7.0,
        );
        for status in [
            Status::Failed(Error::InvalidInput),
            Status::Saved { revision: 3 },
            Status::Saving {
                completed: 1,
                total: 3,
            },
        ] {
            state.status = status;
            let image = pixels(state);
            pair(
                &image,
                viewport,
                Rect {
                    x: layout.status.x,
                    y: layout.status.y,
                    width: layout.status.width,
                    height: 24,
                },
                if matches!(status, Status::Failed(_)) {
                    DANGER
                } else {
                    MUTED
                },
                BLACK,
                7.0,
            );
            let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
            let frame = Frame::new(state);
            let count = frame.semantics(&mut nodes).unwrap();
            let label = nodes[..count]
                .iter()
                .find(|n| n.id == SemanticId::Status)
                .unwrap()
                .label;
            assert!(!label.is_empty(), "status also needs a non-color label");
            let mut guarded = [0xdead_beef; TILE_PIXELS + 3];
            let r = frame
                .render_tile(viewport.width() - 1, viewport.height() - 1, &mut guarded)
                .unwrap();
            assert_eq!(r.width * r.height, 1);
            assert!(guarded[1..].iter().all(|p| *p == 0xdead_beef));
            let before = guarded;
            assert!(frame
                .render_tile(viewport.width(), 0, &mut guarded)
                .is_err());
            assert_eq!(guarded, before);
            assert!(frame.render_tile(0, 0, &mut []).is_err());
        }
    }
}

fn scenario(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}

fn key(ui: &mut Controller<'_, Port>, sequence: &mut u64, physical: PhysicalKey, shift: bool) {
    *sequence += 1;
    ui.deliver(InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence: *sequence,
        generation: 1,
        event: Event::Key(KeyboardEvent {
            key: physical,
            action: KeyAction::Press,
            modifiers: KeyboardModifiers {
                left_shift: shift,
                ..Default::default()
            },
        }),
    })
    .unwrap();
}

fn preserved(ui: &Controller<'_, Port>) -> String {
    let v = ui.view();
    let objects: Vec<_> = v
        .objects
        .iter()
        .map(|o| (o.object, o.name, o.name_length, o.revision, o.length))
        .collect();
    let desktop = v.desktop.map(|d| {
        (
            d.page(),
            d.focus(),
            d.is_guest(),
            d.password_len(),
            d.unavailable(),
            d.query(),
        )
    });
    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = Frame::new(v).semantics(&mut nodes).unwrap();
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        (
            v.viewport,
            v.draw_pointer,
            v.screen,
            v.tool,
            v.focus,
            v.status
        ),
        (
            v.contents,
            v.title,
            v.caret,
            v.anchor,
            v.scroll,
            v.cursor,
            v.document_menu
        ),
        (
            v.editable,
            v.can_save,
            v.saving,
            v.document_dirty,
            v.list_loaded,
            v.list_scroll
        ),
        objects,
        desktop,
        (
            ui.document().draft(),
            ui.document().is_closed(),
            ui.document().is_uncertain(),
            ui.document().is_conflicted()
        ),
        &nodes[..count]
    )
}

fn toggle_roundtrip(ui: &mut Controller<'_, Port>, host: &Rc<RefCell<Host>>) {
    assert!(!ui.view().high_contrast);
    let before = preserved(ui);
    let requests = host.borrow().requests.len();
    let closed = host.borrow().closed.clone();
    let original = pixels(ui.view());
    let generation = ui.view().generation;
    let viewport = ui.view().viewport;
    assert!(ui.presented(generation, viewport.epoch()));
    ui.set_high_contrast(false).unwrap();
    assert_eq!(ui.view().generation, generation);
    assert!(!ui.view().dirty);
    assert!(!ui
        .paint_damage()
        .intersects(0, 0, viewport.width(), viewport.height()));
    ui.set_high_contrast(true).unwrap();
    assert_eq!(ui.view().generation, generation + 1);
    assert!(ui.view().high_contrast && ui.view().dirty);
    assert_eq!(preserved(ui), before);
    assert!(ui.paint_damage().intersects(0, 0, 1, 1));
    assert!(ui
        .paint_damage()
        .intersects(viewport.width() - 1, viewport.height() - 1, 1, 1));
    assert_eq!(
        ui.activate_semantic(SemanticId::NewDocument, generation),
        Err(Error::StaleInput)
    );
    assert!(ui.presented(generation + 1, viewport.epoch()));
    ui.set_high_contrast(true).unwrap();
    assert_eq!(ui.view().generation, generation + 1);
    assert!(!ui.view().dirty);
    ui.set_high_contrast(false).unwrap();
    assert_eq!(ui.view().generation, generation + 2);
    assert_eq!(preserved(ui), before);
    assert_eq!(
        pixels(ui.view()),
        original,
        "default roundtrip must be byte-identical"
    );
    assert_eq!(
        host.borrow().requests.len(),
        requests,
        "presentation must not issue storage RPCs"
    );
    assert_eq!(
        host.borrow().closed,
        closed,
        "presentation must not close capabilities"
    );
}

#[test]
fn toggling_preserves_document_identity_selection_scroll_tool_menu_dialog_and_save() {
    scenario(|| {
        for mode in [EditMode::ReadWrite, EditMode::ReadOnly] {
            let host = Host::new();
            let original = "Een lange echte regel.\n".repeat(45);
            host.borrow_mut().seed(0, original.as_bytes());
            let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                mode,
                Buffers {
                    draft: &mut d,
                    original: &mut o,
                    staging: &mut s,
                },
            );
            toggle_roundtrip(&mut ui, &host);
            ui.refresh().unwrap();
            toggle_roundtrip(&mut ui, &host);
            ui.open(0).unwrap();
            if mode == EditMode::ReadWrite {
                ui.insert("Mijn draft. ").unwrap();
            }
            let mut sequence = 0;
            for _ in 0..40 {
                key(&mut ui, &mut sequence, PhysicalKey::Down, true);
            }
            assert!(ui.view().scroll > 0);
            assert_ne!(ui.view().caret, ui.view().anchor);
            toggle_roundtrip(&mut ui, &host);
            // Same admitted input generation continues after presentation changes.
            key(&mut ui, &mut sequence, PhysicalKey::Left, true);
            ui.switch_tool().unwrap();
            toggle_roundtrip(&mut ui, &host);
            ui.set_high_contrast(true).unwrap();
            let text = ui.view().contents.unwrap().to_owned();
            let requests = host.borrow().requests.len();
            assert_eq!(ui.insert("mag niet"), Err(Error::InvalidInput));
            ui.paste_clipboard("mag niet").unwrap();
            assert_eq!(
                ui.set_semantic_value(SemanticId::Document, ui.view().generation, "mag niet"),
                Err(Error::InvalidInput)
            );
            assert_eq!(ui.view().contents, Some(text.as_str()));
            assert_eq!(host.borrow().requests.len(), requests);
            ui.set_high_contrast(false).unwrap();
            ui.activate_semantic(SemanticId::DocumentMenu, ui.view().generation)
                .unwrap();
            assert!(ui.view().document_menu.is_some());
            key(&mut ui, &mut sequence, PhysicalKey::Down, false);
            assert_eq!(ui.view().document_menu.unwrap().selected, 1);
            toggle_roundtrip(&mut ui, &host);
            ui.activate_semantic(SemanticId::CancelDocumentMenu, ui.view().generation)
                .unwrap();
            if mode == EditMode::ReadWrite {
                ui.request_back().unwrap();
                assert_eq!(ui.view().screen, Screen::ConfirmClose);
                toggle_roundtrip(&mut ui, &host);
                ui.activate_semantic(SemanticId::SaveAndBack, ui.view().generation)
                    .unwrap();
                assert!(ui.view().saving);
                toggle_roundtrip(&mut ui, &host);
                while !ui.advance_save().unwrap() {}
                assert_eq!(
                    ui.view().screen,
                    Screen::Material,
                    "pending save-and-back must survive"
                );
                assert_eq!(host.borrow_mut().bytes(0), text.as_bytes());
            } else {
                ui.switch_tool().unwrap();
                ui.set_high_contrast(true).unwrap();
                assert!(!ui.view().editable);
                assert_eq!(
                    ui.insert("ook niet"),
                    Err(Error::Storage(
                        nieuw_os_document_session::volume::Error::ReadOnly
                    ))
                );
                assert_eq!(host.borrow_mut().bytes(0), original.as_bytes());
            }
        }
    });
}

#[test]
fn toggling_does_not_cancel_or_synthesize_an_admitted_pointer_gesture() {
    scenario(|| {
        let host = Host::new();
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
        let r = Frame::new(ui.view()).layout().new;
        let event = |buttons| Event::Mouse {
            x: r.x + r.width / 2,
            y: r.y + r.height / 2,
            input: MouseEvent {
                dx: 0,
                dy: 0,
                buttons: MouseButtons::checked(buttons).unwrap(),
                changed: MouseButtons::checked(1).unwrap(),
                x_overflow: false,
                y_overflow: false,
            },
        };
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence: 1,
            generation: 1,
            event: event(1),
        })
        .unwrap();
        assert_eq!(ui.view().screen, Screen::Material);
        toggle_roundtrip(&mut ui, &host);
        assert_eq!(ui.view().screen, Screen::Material);
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence: 2,
            generation: 1,
            event: event(0),
        })
        .unwrap();
        assert_eq!(
            ui.view().screen,
            Screen::Editor,
            "only the genuine release activates New"
        );
    });
}

#[test]
fn toggling_preserves_desktop_credential_entry_and_search_without_new_authority() {
    scenario(|| {
        let host = Host::new();
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
        ui.enable_vm_desktop().unwrap();
        toggle_roundtrip(&mut ui, &host);
        ui.activate_semantic(SemanticId::Desktop(Target::Personal), ui.view().generation)
            .unwrap();
        let mut sequence = 0;
        key(&mut ui, &mut sequence, PhysicalKey::Tab, false);
        ui.set_semantic_value(
            SemanticId::Desktop(Target::Password),
            ui.view().generation,
            "test",
        )
        .unwrap();
        assert_eq!(ui.view().desktop.unwrap().password_len(), 4);
        toggle_roundtrip(&mut ui, &host);
        key(&mut ui, &mut sequence, PhysicalKey::A, false);
        assert_eq!(ui.view().desktop.unwrap().password_len(), 5);
        key(&mut ui, &mut sequence, PhysicalKey::Escape, false);
        ui.activate_semantic(SemanticId::Desktop(Target::Guest), ui.view().generation)
            .unwrap();
        toggle_roundtrip(&mut ui, &host);
        ui.activate_semantic(SemanticId::Desktop(Target::Submit), ui.view().generation)
            .unwrap();
        toggle_roundtrip(&mut ui, &host);
        ui.activate_semantic(SemanticId::Desktop(Target::Search), ui.view().generation)
            .unwrap();
        ui.set_semantic_value(
            SemanticId::Desktop(Target::SearchField),
            ui.view().generation,
            "mat",
        )
        .unwrap();
        toggle_roundtrip(&mut ui, &host);
        assert_eq!(ui.view().desktop.unwrap().query(), "mat");
    });
}
