//! Idle accounting for SUR1 with the window compositor's private backing.
use nieuw_os_surface_protocol::{Action, CommitReady, Error, Geometry, Receiver, Rectangle, Reply};

pub struct TrackedReceiver {
    inner: Receiver,
    staged: bool,
    frame_started: bool,
}
impl TrackedReceiver {
    pub fn new(peer: u64, geometry: Geometry, timeout: u64) -> Result<Self, Error> {
        Ok(Self {
            inner: Receiver::new_retained(peer, geometry, timeout)?,
            staged: false,
            frame_started: false,
        })
    }
    pub fn has_pending(&self) -> bool {
        self.staged || (self.frame_started && self.inner.completed_frame().is_none())
    }
    pub fn completed_frame(&self) -> Option<u64> {
        self.inner.completed_frame()
    }
    pub fn retained_base(&self) -> Option<u64> {
        self.inner.retained_base()
    }
    pub fn accept(&mut self, sender: u64, now: u64, bytes: &[u8]) -> Result<Action, Error> {
        let result = self.inner.accept(sender, now, bytes);
        if matches!(result, Ok(Action::Staged)) {
            self.staged = true;
            self.frame_started = true;
        }
        if matches!(result, Ok(Action::Retain(_))) {
            // Retain starts/advances a frame without creating pixel staging.
            // Partial coverage must still keep the compositor runnable.
            self.frame_started = true;
        }
        // Neither a malformed fragment nor an uncommitted token is evidence
        // that staging has been cleared. Conservatively keep the old yield.
        result
    }
    pub fn commit_with<E>(
        &mut self,
        token: CommitReady,
        now: u64,
        blit: impl FnOnce(Rectangle, &[u8]) -> Result<(), E>,
    ) -> Result<Reply, Error> {
        let result = self.inner.commit_with(token, now, blit);
        if result.is_ok() {
            // Both positive and negative replies are produced after SUR1's
            // own clear. A failed/partial frame still prevents idle above.
            self.staged = false;
        }
        result
    }
    pub fn reject(&mut self, sender: u64, bytes: &[u8], error: Error) -> Option<Reply> {
        let result = self.inner.reject(sender, bytes, error);
        if result.is_some() {
            self.staged = false;
        }
        result
    }
    pub fn retire(&mut self) {
        self.inner.retire();
        self.staged = false;
        self.frame_started = false;
    }
}
