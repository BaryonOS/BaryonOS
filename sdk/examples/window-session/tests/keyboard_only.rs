//! U04-K1: keyboard-only reachability and activation of the trusted shell
//! controls (host contracts over the real `Shell`/`Scene`; no VM is run).
//!
//! Scope: every control `Shell::controls()` exposes on every shell page, at
//! 1280x720 scale 1 and 1920x1080 scale 2. Each page is reached by keys from
//! `Shell::new`, except `Page::Notifications` (no key route; see the ignored
//! defect test) and `Page::Capability` (a preview page with no route at all),
//! which are entered with `Shell::open`.
//!
//! Order rule: Tab/Shift+Tab follow reading order
//! (docs/implementation/QUIET-SESSION-SPEC.md, "Accessibility and visual
//! acceptance"); DESKTOP-SHELL.md defines no separate rule. Reading order is
//! taken as the control bounds sorted by (y, x).
//!
//! Observable effect: page, emitted commands, search query, hidden windows,
//! reduced motion and a hash of the full rendered shell frame. The frame
//! carries the private focus ring, caret and notice line, so notice-only
//! controls are not compared vacuously.
//!
//! Deviation by design: the search field (Search control 0) is a text field.
//! A click focuses it; Enter launches the first result; Space types. Its
//! contract is asserted separately instead of pointer parity.
//!
//! Scene-owned controls: the title-bar minimise and window-menu buttons are
//! public (`Scene::minimize_button`, `Scene::window_menu_button`) and are
//! compared with Alt+F9 and Alt+Space here. Scene has no maximise or close
//! button. The window-menu rows (Minimaliseren, Ordenen, Focus) have private
//! bounds, so their pointer parity is out of scope; their keyboard routes and
//! repeat ownership are covered in window_states.rs and workspace_v2.rs, and
//! the Veld notification panel in notifications.rs.
use nieuw_os_graphics::{Canvas, Rect};
use nieuw_os_window_session::{
    scene::{Damage, Scene, WindowState},
    shell::{Command, Page, Shell},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

type Display = (u32, u32, u32);
const DISPLAYS: [Display; 2] = [(1280, 720, 1), (1920, 1080, 2)];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Start {
    Lock,
    Users,
    AuthGuest,
    AuthLocal,
    Search,
    Settings,
    Session,
    Notifications,
    Capability,
}
const STARTS: [Start; 9] = [
    Start::Lock,
    Start::Users,
    Start::AuthGuest,
    Start::AuthLocal,
    Start::Search,
    Start::Settings,
    Start::Session,
    Start::Notifications,
    Start::Capability,
];

fn event(key: PhysicalKey, action: KeyAction, shift: bool) -> KeyboardEvent {
    KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers {
            left_shift: shift,
            ..Default::default()
        },
    }
}

fn send(s: &mut Shell, (w, h, sc): Display, key: PhysicalKey, action: KeyAction) -> Command {
    s.key(event(key, action, false), w, h, sc).command
}

/// One physical press: Press, `repeats` auto-repeat events, Release.
fn press(s: &mut Shell, d: Display, key: PhysicalKey, repeats: usize) -> Vec<Command> {
    let mut out = vec![send(s, d, key, KeyAction::Press)];
    for _ in 0..repeats {
        out.push(send(s, d, key, KeyAction::Repeat));
    }
    out.push(send(s, d, key, KeyAction::Release));
    out
}

fn tabs(s: &mut Shell, (w, h, sc): Display, count: usize, shift: bool) {
    for _ in 0..count {
        for action in [KeyAction::Press, KeyAction::Release] {
            s.key(event(PhysicalKey::Tab, action, shift), w, h, sc);
        }
    }
}

fn keys(s: &mut Shell, d: Display, sequence: &[PhysicalKey]) {
    for &k in sequence {
        press(s, d, k, 0);
    }
}

/// Reach `start` with keys only, except the two pages without a key route.
fn reach(start: Start, d: Display) -> Shell {
    use PhysicalKey::*;
    let (mut s, sequence): (_, &[PhysicalKey]) = match start {
        Start::Lock => (Shell::new(true), &[]),
        Start::Users => (Shell::new(true), &[Enter]),
        Start::AuthGuest => (Shell::new(true), &[Enter, Enter]),
        Start::AuthLocal => (Shell::new(true), &[Enter, Tab, Enter]),
        Start::Search => (Shell::new(false), &[F3]),
        Start::Settings => (Shell::new(false), &[F3, I, N, S, Enter]),
        Start::Session => (Shell::new(false), &[Menu]),
        Start::Notifications | Start::Capability => (Shell::new(false), &[]),
    };
    keys(&mut s, d, sequence);
    match start {
        Start::Notifications => s.open(Page::Notifications),
        Start::Capability => s.open(Page::Capability),
        _ => {}
    }
    let expected = match start {
        Start::Lock => Page::Lock,
        Start::Users => Page::Users,
        Start::AuthGuest | Start::AuthLocal => Page::Auth,
        Start::Search => Page::Search,
        Start::Settings => Page::Settings,
        Start::Session => Page::Session,
        Start::Notifications => Page::Notifications,
        Start::Capability => Page::Capability,
    };
    assert_eq!(s.page(), expected, "{start:?} route at {d:?}");
    s
}

/// Control bounds in reading order, and a check that `controls()` already
/// lists them in that order (Tab steps through `controls()` order).
fn controls(s: &Shell, (w, h, sc): Display) -> Vec<Rect> {
    let bounds: Vec<Rect> = s
        .controls(w, h, sc)
        .into_iter()
        .flatten()
        .map(|c| c.bounds)
        .collect();
    let mut reading = bounds.clone();
    reading.sort_by_key(|r| (r.y, r.x));
    assert_eq!(bounds, reading, "controls() not in reading order");
    assert!(!bounds.is_empty());
    bounds
}

fn click(s: &mut Shell, (w, h, sc): Display, r: Rect) -> Vec<Command> {
    let p = (
        (r.x + r.width as i32 / 2) as u32,
        (r.y + r.height as i32 / 2) as u32,
    );
    vec![
        s.mouse(p, true, true, w, h, sc).command,
        s.mouse(p, false, false, w, h, sc).command,
    ]
}

#[derive(Debug, PartialEq)]
struct Obs {
    page: Page,
    commands: Vec<Command>,
    query: String,
    hidden: [bool; 2],
    reduced_motion: bool,
    frame: u64,
}

fn frame(s: &Shell, (w, h, sc): Display) -> u64 {
    let mut pixels = vec![0u32; (w * h) as usize];
    let Ok(mut canvas) = Canvas::new(Rect::new(0, 0, w, h), &mut pixels) else {
        panic!("canvas");
    };
    s.render(&mut canvas, w, h, sc, 0);
    // FNV-1a keeps failure output small; equality of frames is the contract.
    pixels.iter().fold(0xcbf2_9ce4_8422_2325, |hash, p| {
        (hash ^ u64::from(*p)).wrapping_mul(0x100_0000_01b3)
    })
}

fn observe(s: &Shell, d: Display, commands: Vec<Command>) -> Obs {
    Obs {
        page: s.page(),
        commands: commands
            .into_iter()
            .filter(|c| *c != Command::None)
            .collect(),
        query: s.query().into(),
        hidden: s.hidden,
        reduced_motion: s.reduced_motion,
        frame: frame(s, d),
    }
}

/// Tab `k` times from the page's initial focus, then press `key`.
fn by_key(start: Start, d: Display, k: usize, key: PhysicalKey, repeats: usize) -> Obs {
    let mut s = reach(start, d);
    tabs(&mut s, d, k, false);
    let commands = press(&mut s, d, key, repeats);
    observe(&s, d, commands)
}

fn by_pointer(start: Start, d: Display, k: usize) -> Obs {
    let mut s = reach(start, d);
    let bounds = controls(&s, d)[k];
    let commands = click(&mut s, d, bounds);
    observe(&s, d, commands)
}

fn is_search_field(start: Start, k: usize) -> bool {
    start == Start::Search && k == 0
}

#[test]
fn tab_reaches_every_control_in_reading_order_and_shift_tab_reverses_without_trap() {
    for d in DISPLAYS {
        for start in STARTS {
            let mut s = reach(start, d);
            let n = controls(&s, d).len();
            let initial = frame(&s, d);
            let mut seen = vec![initial];
            for k in 1..=n {
                tabs(&mut s, d, 1, false);
                let now = frame(&s, d);
                if k < n {
                    // Focus is visible and moves to a new control each step.
                    assert!(!seen.contains(&now), "{start:?} tab {k} at {d:?}");
                    seen.push(now);
                } else {
                    // The cycle wraps to the first control: no focus trap.
                    assert_eq!(now, initial, "{start:?} wrap at {d:?}");
                }
            }
            assert_eq!(seen.len(), n);
            for (k, expected) in seen.iter().enumerate() {
                let mut forward = reach(start, d);
                tabs(&mut forward, d, k, false);
                let mut backward = reach(start, d);
                tabs(&mut backward, d, (n - k) % n, true);
                assert_eq!(frame(&backward, d), *expected, "{start:?} shift-tab {k}");
                assert_eq!(frame(&forward, d), *expected, "{start:?} tab {k}");
            }
            // Escape leaves every temporary page; Lock is the root cover.
            let mut s = reach(start, d);
            press(&mut s, d, PhysicalKey::Escape, 0);
            let expected = match start {
                Start::Lock | Start::Users | Start::AuthGuest | Start::AuthLocal => Page::Lock,
                _ => Page::Desktop,
            };
            assert_eq!(s.page(), expected, "{start:?} escape at {d:?}");
        }
    }
}

#[test]
fn tab_click_ordering_matches_pointer_focus_on_each_control() {
    // A pointer press focuses the control under it without activating; the
    // same frame must result from Tab×k, proving Tab order equals k.
    for d in DISPLAYS {
        for start in STARTS {
            let n = controls(&reach(start, d), d).len();
            for k in 0..n {
                let mut pointer = reach(start, d);
                let r = controls(&pointer, d)[k];
                let (w, h, sc) = d;
                let p = ((r.x + 4) as u32, (r.y + 4) as u32);
                pointer.mouse(p, true, true, w, h, sc);
                pointer.cancel_pointer();
                let mut keyboard = reach(start, d);
                tabs(&mut keyboard, d, k, false);
                assert_eq!(
                    frame(&keyboard, d),
                    frame(&pointer, d),
                    "{start:?} control {k} at {d:?}"
                );
            }
        }
    }
}

#[test]
fn enter_and_space_activate_each_control_like_a_pointer_click() {
    for d in DISPLAYS {
        for start in STARTS {
            let n = controls(&reach(start, d), d).len();
            for k in 0..n {
                let click = by_pointer(start, d, k);
                if !is_search_field(start, k) {
                    let enter = by_key(start, d, k, PhysicalKey::Enter, 0);
                    assert_eq!(enter, click, "Enter {start:?} control {k} at {d:?}");
                }
                // The search field types Space; its contract is tested below.
                if !is_search_field(start, k) {
                    let space = by_key(start, d, k, PhysicalKey::Space, 0);
                    assert_eq!(space, click, "Space {start:?} control {k} at {d:?}");
                }
            }
        }
    }
}

#[test]
fn search_field_is_a_text_field_whose_enter_launches_the_first_result() {
    for d in DISPLAYS {
        let click = by_pointer(Start::Search, d, 0);
        assert_eq!(click.page, Page::Search);
        assert!(click.commands.is_empty());
        let enter = by_key(Start::Search, d, 0, PhysicalKey::Enter, 0);
        assert_eq!(enter.commands, [Command::Activate(0)]);
        assert_eq!(enter.page, Page::Desktop);
        let space = by_key(Start::Search, d, 0, PhysicalKey::Space, 0);
        assert_eq!((space.page, space.query.as_str()), (Page::Search, " "));
    }
}

#[test]
fn held_enter_or_space_activates_exactly_once() {
    let mut failures = Vec::new();
    for d in DISPLAYS {
        for start in STARTS {
            let n = controls(&reach(start, d), d).len();
            for k in 0..n {
                for key in [PhysicalKey::Enter, PhysicalKey::Space] {
                    if start == Start::Search && key == PhysicalKey::Space {
                        continue; // Space is text input on Search (see above).
                    }
                    let once = by_key(start, d, k, key, 0);
                    let held = by_key(start, d, k, key, 3);
                    if once != held {
                        failures.push(format!(
                            "{key:?} {start:?} control {k} at {d:?}: {:?} -> {:?}",
                            once.page, held.page
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
#[ignore = "U04 defect: FR-U05 keyboard-only — Page::Notifications opens only by header click (Shell::mouse, shell.rs:528-537 at a91d5ade); the Desktop key branch (shell.rs:435) and search results offer no route (Alt+N opens the separate Scene panel)"]
fn notifications_page_is_reachable_by_keyboard_from_the_desktop() {
    use PhysicalKey::*;
    for d in DISPLAYS {
        let mut s = Shell::new(false);
        keys(&mut s, d, &[F3, M, E, L, D, I, N, G, E, N, Enter]);
        assert_eq!(s.page(), Page::Notifications, "at {d:?}");
    }
}

fn alt(scene: &mut Scene, damage: &mut Damage, key: PhysicalKey) {
    for action in [KeyAction::Press, KeyAction::Release] {
        let event = KeyboardEvent {
            key,
            action,
            modifiers: KeyboardModifiers {
                left_alt: true,
                ..Default::default()
            },
        };
        assert!(scene.palette_key(event, damage).unwrap());
    }
}

fn scene_click(scene: &mut Scene, damage: &mut Damage, r: Rect) {
    let (x, y) = (
        (r.x + r.width as i32 / 2) as u32,
        (r.y + r.height as i32 / 2) as u32,
    );
    for (down, held) in [(true, true), (false, false)] {
        scene.mouse_with_damage(x, y, down, held, damage).unwrap();
    }
}

#[test]
fn scene_minimise_and_window_menu_buttons_match_their_shortcuts() {
    for (w, h, sc) in DISPLAYS {
        let state = |scene: &Scene| {
            (
                scene.window_state(0),
                scene.window_state(1),
                scene.focused,
                scene.window_menu_is_open(),
            )
        };
        let mut damage = Damage::new(w, h).unwrap();
        let mut pointer = Scene::new(w, h, sc, [1, 2]).unwrap();
        let mut keyboard = Scene::new(w, h, sc, [1, 2]).unwrap();
        pointer.select(0).unwrap();
        keyboard.select(0).unwrap();
        let button = pointer.minimize_button(0);
        scene_click(&mut pointer, &mut damage, button);
        alt(&mut keyboard, &mut damage, PhysicalKey::F9);
        assert_eq!(pointer.window_state(0), Some(WindowState::Minimized));
        assert_eq!(state(&keyboard), state(&pointer), "minimise at {w}x{h}");

        let mut pointer = Scene::new(w, h, sc, [1, 2]).unwrap();
        let mut keyboard = Scene::new(w, h, sc, [1, 2]).unwrap();
        pointer.select(0).unwrap();
        keyboard.select(0).unwrap();
        let button = pointer.window_menu_button(0);
        scene_click(&mut pointer, &mut damage, button);
        alt(&mut keyboard, &mut damage, PhysicalKey::Space);
        assert!(pointer.window_menu_is_open());
        assert_eq!(state(&keyboard), state(&pointer), "menu at {w}x{h}");
    }
}
