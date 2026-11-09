//! Pure acceptance observer, not a GUI service or product policy. Only the
//! optional native probe uses its one-bit stop barrier after real user input.
#[derive(Clone, Copy, Debug, Default)]
pub struct Frame {
    pub generation: u64,
    pub screen: u64,
    pub dirty: u64,
    pub save_acks: u64,
    pub revision: u64,
    pub saved_bytes: u64,
    pub saved_sum: u64,
    pub draft_bytes: u64,
    pub draft_sum: u64,
    pub object: u64,
    pub f12: u64,
}
#[derive(Default)]
pub struct Gate {
    phase: u8,
    generation: u64,
    object: u64,
    revision: u64,
    bytes: u64,
    sum: u64,
    f12: u64,
}
impl Gate {
    pub const fn new() -> Self {
        Self {
            phase: 0,
            generation: 0,
            object: 0,
            revision: 0,
            bytes: 0,
            sum: 0,
            f12: 0,
        }
    }
    /// Caller already established coherent Ready=1 after native tile ACKs.
    /// Normal chain requires Saved->later list->later reopened same document.
    /// Cold recovery has no save ACK and requires list->reopened expected data.
    pub fn observe(
        &mut self,
        f: Frame,
        recovery: bool,
        expected_bytes: u64,
        expected_sum: u64,
    ) -> bool {
        if self.phase == 3 {
            return f.generation >= self.generation
                && f.screen == 2
                && f.dirty == 0
                && f.object == self.object
                && f.draft_bytes == self.bytes
                && f.draft_sum == self.sum
                && f.f12 > self.f12;
        }
        if f.generation <= self.generation || f.dirty != 0 {
            return false;
        }
        match self.phase {
            0 if recovery && f.screen == 1 && f.save_acks == 0 => {
                self.phase = 2;
                self.generation = f.generation;
                self.bytes = expected_bytes;
                self.sum = expected_sum;
            }
            0 if !recovery
                && f.screen == 2
                && f.save_acks > 0
                && f.revision > 0
                && f.saved_bytes == expected_bytes
                && f.saved_sum == expected_sum
                && f.draft_bytes == expected_bytes
                && f.draft_sum == expected_sum =>
            {
                self.phase = 1;
                self.generation = f.generation;
                self.object = f.object;
                self.revision = f.revision;
                self.bytes = f.saved_bytes;
                self.sum = f.saved_sum;
            }
            1 if f.screen == 1
                && f.object == self.object
                && f.revision == self.revision
                && f.saved_bytes == self.bytes
                && f.saved_sum == self.sum =>
            {
                self.phase = 2;
                self.generation = f.generation;
            }
            2 if f.screen == 2
                && (recovery || f.object == self.object)
                && f.draft_bytes == self.bytes
                && f.draft_sum == self.sum =>
            {
                self.phase = 3;
                self.generation = f.generation;
                self.object = f.object;
                self.f12 = f.f12;
            }
            _ => {}
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn saved() -> Frame {
        Frame {
            generation: 5,
            screen: 2,
            save_acks: 1,
            revision: 1,
            saved_bytes: 14,
            saved_sum: 42,
            draft_bytes: 14,
            draft_sum: 42,
            object: 8,
            ..Frame::default()
        }
    }
    #[test]
    fn save_alone_and_more_keys_cannot_stop() {
        let mut g = Gate::new();
        let mut f = saved();
        assert!(!g.observe(f, false, 14, 42));
        for n in 6..20 {
            f.generation = n;
            f.f12 = n;
            assert!(!g.observe(f, false, 14, 42));
        }
    }
    #[test]
    fn complete_chain_waits_for_later_press_not_release() {
        let mut g = Gate::new();
        let mut f = saved();
        assert!(!g.observe(f, false, 14, 42));
        f.generation = 6;
        f.screen = 1;
        assert!(!g.observe(f, false, 14, 42));
        f.generation = 7;
        f.screen = 2;
        f.f12 = 1;
        assert!(!g.observe(f, false, 14, 42));
        assert!(!g.observe(f, false, 14, 42));
        f.f12 = 2;
        assert!(g.observe(f, false, 14, 42));
    }
    #[test]
    fn changed_object_or_data_cannot_certify_reopen() {
        for which in 0..3 {
            let mut g = Gate::new();
            let mut f = saved();
            g.observe(f, false, 14, 42);
            f.generation = 6;
            f.screen = 1;
            g.observe(f, false, 14, 42);
            f.generation = 7;
            f.screen = 2;
            match which {
                0 => f.object += 1,
                1 => f.draft_bytes += 1,
                _ => f.draft_sum += 1,
            }
            assert!(!g.observe(f, false, 14, 42));
            f.f12 = 10;
            assert!(!g.observe(f, false, 14, 42));
        }
    }
    #[test]
    fn old_list_generation_cannot_advance() {
        let mut g = Gate::new();
        let mut f = saved();
        g.observe(f, false, 14, 42);
        f.screen = 1;
        g.observe(f, false, 14, 42);
        f.screen = 2;
        f.generation = 6;
        g.observe(f, false, 14, 42);
        f.f12 = 1;
        assert!(!g.observe(f, false, 14, 42));
    }
    #[test]
    fn dirty_never_opens_stop_barrier() {
        let mut g = Gate::new();
        let mut f = saved();
        f.dirty = 1;
        assert!(!g.observe(f, false, 14, 42));
        f.generation = 6;
        f.screen = 1;
        g.observe(f, false, 14, 42);
        f.generation = 7;
        f.screen = 2;
        f.f12 = 9;
        assert!(!g.observe(f, false, 14, 42));
    }
    #[test]
    fn recovery_has_no_synthetic_save() {
        let mut g = Gate::new();
        let mut f = Frame {
            generation: 1,
            screen: 1,
            ..Frame::default()
        };
        assert!(!g.observe(f, true, 14, 42));
        f.generation = 2;
        f.screen = 2;
        f.object = 8;
        f.draft_bytes = 14;
        f.draft_sum = 42;
        assert!(!g.observe(f, true, 14, 42));
        f.f12 = 1;
        assert!(g.observe(f, true, 14, 42));
        assert_eq!(f.save_acks, 0);
    }
}
