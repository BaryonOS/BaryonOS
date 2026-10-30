//! Compositor-owned focus bookkeeping, not an authority or a kernel boundary.
use crate::{
    keyboard::KEY_COUNT, KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent,
    PhysicalKey,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusOwner(u64);
impl FocusOwner {
    /// The trusted caller resolves this identity to a live, authorized surface.
    pub const fn new(identity: u64) -> Option<Self> {
        if identity == 0 {
            None
        } else {
            Some(Self(identity))
        }
    }
    pub const fn identity(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusToken {
    owner: FocusOwner,
    generation: u64,
}
impl FocusToken {
    pub const fn owner(self) -> FocusOwner {
        self.owner
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusError {
    GenerationExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Routed<T> {
    token: FocusToken,
    event: T,
}
impl<T> Routed<T> {
    pub const fn token(&self) -> FocusToken {
        self.token
    }
    pub const fn event(&self) -> &T {
        &self.event
    }
    pub fn into_event(self) -> T {
        self.event
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputRouter {
    focus: Option<FocusOwner>,
    generation: u64,
    exhausted: bool,
    /// Physical down state is independent of delivery ownership: keys pressed
    /// with no focus must also remain suppressed across a decoder reset.
    physical_down: [bool; KEY_COUNT],
    keys: [Option<FocusToken>; KEY_COUNT],
    buttons: [Option<FocusToken>; 3],
    device_buttons: u8,
}
impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}
impl InputRouter {
    pub const fn new() -> Self {
        Self {
            focus: None,
            generation: 0,
            exhausted: false,
            physical_down: [false; KEY_COUNT],
            keys: [None; KEY_COUNT],
            buttons: [None; 3],
            device_buttons: 0,
        }
    }
    pub fn current(&self) -> Option<FocusToken> {
        if self.exhausted {
            return None;
        }
        self.focus.map(|owner| FocusToken {
            owner,
            generation: self.generation,
        })
    }
    /// Only the compositor calls this after resolving surface/lifecycle rights.
    /// Notify the old owner to cancel held input, and recheck tokens on dequeue.
    /// Held hardware keys/buttons retain their old token until physical release.
    pub fn set_focus(
        &mut self,
        owner: Option<FocusOwner>,
    ) -> Result<Option<FocusToken>, FocusError> {
        let Some(generation) = self.generation.checked_add(1).filter(|_| !self.exhausted) else {
            self.exhausted = true;
            self.focus = None;
            return Err(FocusError::GenerationExhausted);
        };
        self.generation = generation;
        self.focus = owner;
        Ok(self.current())
    }
    pub fn is_current(&self, token: FocusToken) -> bool {
        self.current() == Some(token)
    }
    /// Reset decoder and notify client together with this call after loss.
    /// Invalidates queued events even if focus keeps the same surface identity.
    /// Retains old ownership as suppression markers: a known held key/button
    /// must not become a new press merely because the decoder was reset.
    pub fn reset_input(&mut self) -> Result<Option<FocusToken>, FocusError> {
        self.set_focus(self.focus)
    }
    pub fn route_key(&mut self, mut event: KeyboardEvent) -> Option<Routed<KeyboardEvent>> {
        let slot = event.key as usize;
        let token = match event.action {
            KeyAction::Press => {
                // A second Press cannot transfer a still-held physical key.
                if self.physical_down[slot] {
                    return None;
                }
                self.physical_down[slot] = true;
                self.keys[slot] = self.current();
                self.keys[slot]
            }
            KeyAction::Repeat => {
                self.physical_down[slot] = true;
                self.keys[slot]
            }
            KeyAction::Release => {
                self.physical_down[slot] = false;
                self.keys[slot].take()
            }
            KeyAction::Pulse => self.current(),
        }?;
        if !self.is_current(token) {
            return None;
        }
        event.modifiers = self.owned_modifiers(token, event.modifiers);
        Some(Routed { token, event })
    }
    fn owned_modifiers(&self, token: FocusToken, locks: KeyboardModifiers) -> KeyboardModifiers {
        use PhysicalKey::*;
        let owned = |key: PhysicalKey| self.keys[key as usize] == Some(token);
        KeyboardModifiers {
            left_shift: owned(LeftShift),
            right_shift: owned(RightShift),
            left_control: owned(LeftControl),
            right_control: owned(RightControl),
            left_alt: owned(LeftAlt),
            right_alt: owned(RightAlt),
            left_super: owned(LeftSuper),
            right_super: owned(RightSuper),
            caps_lock: locks.caps_lock,
            num_lock: locks.num_lock,
            scroll_lock: locks.scroll_lock,
        }
    }
    fn owned_buttons(&self, token: FocusToken) -> u8 {
        let mut bits = 0;
        for (index, owner) in self.buttons.iter().enumerate() {
            if *owner == Some(token) {
                bits |= 1 << index;
            }
        }
        bits
    }
    /// This router has one compositor-selected target. Hit testing and pointer
    /// capture belong to the compositor, which establishes the target first.
    pub fn route_mouse(&mut self, mut event: MouseEvent) -> Option<Routed<MouseEvent>> {
        let current = self.current();
        let before = current.map_or(0, |token| self.owned_buttons(token));
        let bits = event.buttons.bits();
        let changed = bits ^ self.device_buttons;
        self.device_buttons = bits;
        for index in 0..3 {
            if changed & (1 << index) != 0 {
                self.buttons[index] = if bits & (1 << index) != 0 {
                    current
                } else {
                    None
                };
            }
        }
        let token = current?;
        let after = self.owned_buttons(token);
        event.buttons = MouseButtons::from_bits(after);
        event.changed = MouseButtons::from_bits(before ^ after);
        if event.dx == 0
            && event.dy == 0
            && event.changed.bits() == 0
            && !event.x_overflow
            && !event.y_overflow
        {
            return None;
        }
        Some(Routed { token, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ps2Mouse, Set1Keyboard};
    fn owner(n: u64) -> FocusOwner {
        FocusOwner::new(n).unwrap()
    }
    fn key(
        router: &mut InputRouter,
        keyboard: &mut Set1Keyboard,
        byte: u8,
    ) -> Option<Routed<KeyboardEvent>> {
        keyboard
            .feed(byte)
            .unwrap()
            .and_then(|e| router.route_key(e))
    }
    fn mouse(
        router: &mut InputRouter,
        decoder: &mut Ps2Mouse,
        flags: u8,
        dx: u8,
    ) -> Option<Routed<MouseEvent>> {
        decoder.feed(flags).unwrap();
        decoder.feed(dx).unwrap();
        router.route_mouse(decoder.feed(0).unwrap().unwrap())
    }
    #[test]
    fn queued_events_and_held_keys_do_not_cross_focus_even_to_same_owner() {
        assert_eq!(FocusOwner::new(0), None);
        let mut r = InputRouter::new();
        let mut k = Set1Keyboard::new();
        assert!(key(&mut r, &mut k, 0x1e).is_none()); // pressed with no target
        let first = r.set_focus(Some(owner(1))).unwrap().unwrap();
        assert!(key(&mut r, &mut k, 0x1e).is_none()); // repeat cannot acquire target
        assert!(key(&mut r, &mut k, 0x9e).is_none());
        let queued = key(&mut r, &mut k, 0x1e).unwrap();
        assert_eq!(queued.token(), first);
        assert!(r.is_current(queued.token()));
        let next = r.set_focus(Some(owner(1))).unwrap().unwrap();
        assert_ne!(first, next);
        assert!(!r.is_current(queued.token()));
        assert!(key(&mut r, &mut k, 0x1e).is_none());
        assert!(key(&mut r, &mut k, 0x9e).is_none());
        assert_eq!(key(&mut r, &mut k, 0x1e).unwrap().token(), next);
    }
    #[test]
    fn previous_focus_modifiers_are_removed_from_new_focus_chords() {
        let mut r = InputRouter::new();
        let mut k = Set1Keyboard::new();
        r.set_focus(Some(owner(1))).unwrap();
        assert!(
            key(&mut r, &mut k, 0x2a)
                .unwrap()
                .event()
                .modifiers
                .left_shift
        );
        r.set_focus(Some(owner(2))).unwrap();
        let e = key(&mut r, &mut k, 0x1e).unwrap();
        assert!(k.modifiers().shift());
        assert!(!e.event().modifiers.shift());
        assert!(key(&mut r, &mut k, 0xaa).is_none());
        assert!(
            key(&mut r, &mut k, 0x36)
                .unwrap()
                .event()
                .modifiers
                .right_shift
        );
        assert!(!key(&mut r, &mut k, 0xb6).unwrap().event().modifiers.shift());
    }
    #[test]
    fn button_drag_release_cannot_be_inherited_by_another_focus() {
        let mut r = InputRouter::new();
        let mut m = Ps2Mouse::new();
        r.set_focus(Some(owner(1))).unwrap();
        let press = mouse(&mut r, &mut m, 0x09, 0).unwrap();
        assert!(press.event().buttons.left());
        r.set_focus(Some(owner(2))).unwrap();
        assert!(!r.is_current(press.token()));
        let movement = mouse(&mut r, &mut m, 0x09, 4).unwrap();
        assert_eq!(
            (
                movement.event().dx,
                movement.event().buttons.bits(),
                movement.event().changed.bits()
            ),
            (4, 0, 0)
        );
        assert!(mouse(&mut r, &mut m, 0x08, 0).is_none());
        let press = mouse(&mut r, &mut m, 0x09, 0).unwrap();
        assert!(press.event().buttons.left() && press.event().changed.left());
        let release = mouse(&mut r, &mut m, 0x08, 0).unwrap();
        assert!(!release.event().buttons.left() && release.event().changed.left());
    }
    #[test]
    fn device_reset_invalidates_queue_and_clears_held_input() {
        let mut r = InputRouter::new();
        let mut k = Set1Keyboard::new();
        r.set_focus(Some(owner(1))).unwrap();
        let queued = key(&mut r, &mut k, 0x1d).unwrap();
        r.reset_input().unwrap();
        k.reset();
        assert!(!r.is_current(queued.token()));
        let e = key(&mut r, &mut k, 0x1e).unwrap();
        assert!(!e.event().modifiers.control());
    }
    #[test]
    fn reset_does_not_turn_known_held_input_into_new_presses() {
        let mut r = InputRouter::new();
        let mut k = Set1Keyboard::new();
        let mut m = Ps2Mouse::new();
        r.set_focus(Some(owner(1))).unwrap();
        assert!(key(&mut r, &mut k, 0x1e).is_some());
        assert!(mouse(&mut r, &mut m, 0x09, 0).is_some());
        r.reset_input().unwrap();
        k.reset();
        m.reset();
        // The physical repeat looks like a Press to the reset decoder, but
        // its known old owner still prevents delivery to the new generation.
        assert!(key(&mut r, &mut k, 0x1e).is_none());
        assert!(mouse(&mut r, &mut m, 0x09, 0).is_none());
        assert!(key(&mut r, &mut k, 0x9e).is_none());
        assert!(mouse(&mut r, &mut m, 0x08, 0).is_none());
        assert!(key(&mut r, &mut k, 0x1e).is_some());
        assert!(mouse(&mut r, &mut m, 0x09, 0).is_some());
    }
    #[test]
    fn generation_exhaustion_fails_closed_without_token_reuse() {
        let mut r = InputRouter::new();
        r.generation = u64::MAX - 1;
        let final_token = r.set_focus(Some(owner(1))).unwrap().unwrap();
        assert_eq!(final_token.generation(), u64::MAX);
        assert_eq!(
            r.set_focus(Some(owner(2))),
            Err(FocusError::GenerationExhausted)
        );
        assert_eq!(r.current(), None);
        assert!(!r.is_current(final_token));
        assert_eq!(r.reset_input(), Err(FocusError::GenerationExhausted));
        assert_eq!(
            r.set_focus(Some(owner(1))),
            Err(FocusError::GenerationExhausted)
        );
        let mut k = Set1Keyboard::new();
        assert!(key(&mut r, &mut k, 0x1e).is_none());
    }
}
