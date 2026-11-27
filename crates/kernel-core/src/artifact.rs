//! L01 sealed-artifact state machine (ADR-L01-ARTIFACT-MAP-20261127; design
//! `docs/implementation/LOCAL-AI-L01-MODEL-MAPPING.md` section 4).
//!
//! `Empty → Filling → Sealed → Verified → Released`, with the failure path
//! `Sealed → Rejected → Released`. Nothing links before `Verified`: the ring-3
//! verifier reads the sealed bytes through a bounded kernel copy-out and never
//! maps the artifact. Rights checks (`ARTIFACT_MAP`, `ARTIFACT_VERIFY`) stay in
//! `capability-core`; this type holds only state, binding and counters and
//! never touches memory.
use crate::{Error, Frames, ARTIFACT_QUOTA, PAGE};

/// Upper bound on the bytes one `artifact_read` call may copy (contract §3:
/// no unbounded work in a syscall).
pub const READ_CHUNK: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Empty,
    Filling,
    Sealed,
    Verified,
    Rejected,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactError {
    /// The request is illegal in the current state.
    State,
    /// The caller's binding (identity, generation, length) is not this artifact.
    Stale,
    /// The verifier has not read every byte of this generation yet.
    Incomplete,
    /// The reported digest differs; the artifact is now `Rejected`.
    Mismatch,
    /// Links remain, or the link counter would overflow.
    Busy,
    /// The caller's copy into user memory failed; no progress was recorded.
    Fault,
    Frames(Error),
}

/// What a verdict, a read or a link is bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub id: u16,
    pub generation: u32,
    pub length: u64,
}

/// Deliberately neither `Clone` nor `Copy`: a duplicate would carry its own
/// link counter and could be released while the original still has links.
#[derive(Debug, PartialEq, Eq)]
pub struct Artifact {
    binding: Binding,
    /// Pinned public reference digest from the manifest descriptor.
    reference: [u8; 32],
    state: State,
    cursor: u64,
    links: u16,
}

impl Artifact {
    /// A new, empty artifact. The length must be nonzero and fit [`ARTIFACT_QUOTA`].
    pub fn new(binding: Binding, reference: [u8; 32]) -> Result<Self, ArtifactError> {
        if binding.id == 0
            || binding.length == 0
            || binding.length.div_ceil(PAGE) > ARTIFACT_QUOTA as u64
        {
            return Err(ArtifactError::Frames(Error::Range));
        }
        Ok(Self {
            binding,
            reference,
            state: State::Empty,
            cursor: 0,
            links: 0,
        })
    }
    pub const fn binding(&self) -> Binding {
        self.binding
    }
    pub const fn state(&self) -> State {
        self.state
    }
    pub const fn links(&self) -> u16 {
        self.links
    }
    pub const fn cursor(&self) -> u64 {
        self.cursor
    }
    /// Frames the artifact occupies; bytes after `length` in the last page are zero.
    pub const fn pages(&self) -> usize {
        self.binding.length.div_ceil(PAGE) as usize
    }

    fn expect(&self, binding: Binding, state: State) -> Result<(), ArtifactError> {
        // The binding is checked first, so a stale caller learns nothing about state.
        if binding != self.binding {
            return Err(ArtifactError::Stale);
        }
        if self.state != state {
            return Err(ArtifactError::State);
        }
        Ok(())
    }

    /// `Empty → Filling`, after the kernel reserved [`Artifact::pages`] frames.
    pub fn begin_fill(&mut self) -> Result<(), ArtifactError> {
        if self.state != State::Empty {
            return Err(ArtifactError::State);
        }
        self.state = State::Filling;
        Ok(())
    }

    /// `Filling → Sealed`: every frame of this artifact loses its writable
    /// alias for good. Refused, with nothing changed, while a writable alias
    /// exists or when the frame count is not exactly [`Artifact::pages`].
    pub fn seal<const N: usize>(&mut self, frames: &mut Frames<N>) -> Result<(), ArtifactError> {
        if self.state != State::Filling {
            return Err(ArtifactError::State);
        }
        frames
            .seal_artifact(self.binding.id, self.binding.generation, self.pages())
            .map_err(ArtifactError::Frames)?;
        self.state = State::Sealed;
        Ok(())
    }

    /// Bounded sequential copy-out for the verifier (`artifact_read`). `copy`
    /// receives `(offset, len)` with `len <= min(max, READ_CHUNK)` and must copy
    /// those sealed bytes into the caller's private buffer; the cursor advances
    /// only when it succeeds. Returns the bytes copied, 0 at the end.
    pub fn read<E>(
        &mut self,
        binding: Binding,
        max: u64,
        copy: impl FnOnce(u64, u64) -> Result<(), E>,
    ) -> Result<u64, ArtifactError> {
        self.expect(binding, State::Sealed)?;
        let len = (self.binding.length - self.cursor).min(max).min(READ_CHUNK);
        if len == 0 {
            return Ok(0);
        }
        copy(self.cursor, len).map_err(|_| ArtifactError::Fault)?;
        self.cursor += len;
        Ok(len)
    }

    /// The verifier's verdict (`artifact_verify`): `Sealed → Verified` only for
    /// this exact binding, after a complete read, and on an exact digest match;
    /// any mismatch fails closed to `Rejected`. The reference digest is public,
    /// so a plain comparison leaks nothing.
    ///
    /// The digest is trusted only because the caller holds the exclusive,
    /// manifest-only `ARTIFACT_VERIFY` right: the verifier is part of the
    /// trusted base for integrity (ADR Q1). The cursor never rewinds; a verifier
    /// that dies mid-read means release and refill.
    pub fn verify(&mut self, binding: Binding, digest: &[u8; 32]) -> Result<(), ArtifactError> {
        self.expect(binding, State::Sealed)?;
        if self.cursor != self.binding.length {
            return Err(ArtifactError::Incomplete);
        }
        if *digest != self.reference {
            self.state = State::Rejected;
            return Err(ArtifactError::Mismatch);
        }
        self.state = State::Verified;
        Ok(())
    }

    /// Record one `ARTIFACT_MAP` link; only a `Verified` artifact links.
    /// Returns the exact length the kernel reports to the caller.
    pub fn link(&mut self, binding: Binding) -> Result<u64, ArtifactError> {
        self.expect(binding, State::Verified)?;
        self.links = self.links.checked_add(1).ok_or(ArtifactError::Busy)?;
        Ok(self.binding.length)
    }

    pub fn unlink(&mut self, binding: Binding) -> Result<(), ArtifactError> {
        self.expect(binding, State::Verified)?;
        self.links = self.links.checked_sub(1).ok_or(ArtifactError::State)?;
        Ok(())
    }

    /// Any state with zero links `→ Released`. The kernel then frees every
    /// frame through [`Frames::free`], which zeroes it before reuse; an id may
    /// be reused for a new artifact only after all its frames are free (S1b).
    pub fn release(&mut self) -> Result<(), ArtifactError> {
        if self.state == State::Released {
            return Err(ArtifactError::State);
        }
        if self.links != 0 {
            return Err(ArtifactError::Busy);
        }
        self.state = State::Released;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::{Alias, Kind, Owner, Range, USER_QUOTA};
    use std::{boxed::Box, vec, vec::Vec};

    // Compile-time: the lifecycle value has one owner. This line is ambiguous,
    // so a build error, as soon as `Artifact` is `Clone` (and so also `Copy`).
    trait NotClone<A> {
        fn ok() {}
    }
    impl<T> NotClone<()> for T {}
    impl<T: Clone> NotClone<u8> for T {}
    const _: fn() = || <Artifact as NotClone<_>>::ok();

    const N: usize = 8192;
    const ID: u16 = 7;
    const REF: [u8; 32] = [0x5a; 32];
    const BIND: Binding = Binding {
        id: ID,
        generation: 3,
        length: 5 * PAGE + 123,
    };

    fn pool() -> Box<Frames<N>> {
        let mut f = Box::new(Frames::<N>::new());
        f.admit(Range::pages(PAGE, N as u64 - 1).unwrap(), 7, 0x0f, &[]);
        f
    }
    fn owner(sealed: bool) -> Owner {
        Owner::Artifact {
            id: ID,
            generation: BIND.generation,
            sealed,
        }
    }
    /// A reserved artifact in `Filling`, with no fill alias recorded.
    fn filled(f: &mut Frames<N>) -> (Artifact, Vec<u64>) {
        let mut a = Artifact::new(BIND, REF).unwrap();
        let mut pages = vec![0; a.pages()];
        f.allocate_many(owner(false), &mut pages, |_| {}).unwrap();
        a.begin_fill().unwrap();
        (a, pages)
    }
    fn read_all(a: &mut Artifact) -> u64 {
        let mut total = 0;
        loop {
            let n = a.read(BIND, u64::MAX, |_, _| Ok::<(), ()>(())).unwrap();
            if n == 0 {
                return total;
            }
            total += n;
        }
    }

    // Design test 1: a user alias of an artifact frame is RO/NX only, and
    // only once sealed; no alias of an artifact frame is ever executable.
    #[test]
    fn user_alias_is_read_only_no_execute_and_only_after_seal() {
        let mut f = pool();
        let (mut a, pages) = filled(&mut f);
        let pa = pages[0];
        assert_eq!(f.map(pa, false, false, true), Err(Error::Permission));
        assert_eq!(f.map(pa, false, true, false), Err(Error::Permission));
        a.seal(&mut f).unwrap();
        assert_eq!(f.map(pa, true, false, true), Err(Error::Permission));
        assert_eq!(f.map(pa, false, true, true), Err(Error::Permission));
        assert_eq!(f.map(pa, false, true, false), Err(Error::Permission));
        f.map(pa, false, false, true).unwrap();
        assert_eq!(
            f.alias(pa).unwrap(),
            Alias {
                writable: 0,
                executable: 0,
                total: 1
            }
        );
        // A private user frame is still mapped as before.
        let user = f.allocate(Owner::User(1, Kind::Data), |_| {}).unwrap();
        f.map(user, true, false, true).unwrap();
    }

    // Design test 2: seal is refused while a writable alias exists, with no
    // owner changed; after sealing no writable alias can be recorded.
    #[test]
    fn seal_requires_no_writable_alias_and_blocks_later_writes() {
        let mut f = pool();
        let (mut a, pages) = filled(&mut f);
        f.map(pages[2], true, false, false).unwrap();
        assert_eq!(a.seal(&mut f), Err(ArtifactError::Frames(Error::Mapped)));
        assert_eq!(a.state(), State::Filling);
        assert!(pages.iter().all(|&pa| f.owner(pa) == Ok(owner(false))));
        f.unmap(pages[2], true, false).unwrap();
        a.seal(&mut f).unwrap();
        assert!(pages.iter().all(|&pa| f.owner(pa) == Ok(owner(true))));
        for &pa in &pages {
            assert_eq!(f.map(pa, true, false, false), Err(Error::Permission));
        }
        // A sealed artifact never gains a fresh, writable-aliasable frame.
        let free = f.availability();
        assert_eq!(f.allocate(owner(false), |_| {}), Err(Error::Owner));
        let mut more = [0u64; 2];
        assert_eq!(
            f.allocate_many(owner(false), &mut more, |_| {}),
            Err(Error::Owner)
        );
        assert_eq!(f.availability(), free);
        // Read-only kernel scratch reads stay possible.
        f.map(pages[0], false, false, false).unwrap();
        assert_eq!(a.seal(&mut f), Err(ArtifactError::State));
    }

    #[test]
    fn seal_requires_exactly_the_reserved_page_count() {
        let mut f = pool();
        let mut a = Artifact::new(BIND, REF).unwrap();
        let mut short = vec![0; a.pages() - 1];
        f.allocate_many(owner(false), &mut short, |_| {}).unwrap();
        a.begin_fill().unwrap();
        assert_eq!(a.seal(&mut f), Err(ArtifactError::Frames(Error::Range)));
        assert!(short.iter().all(|&pa| f.owner(pa) == Ok(owner(false))));
    }

    #[test]
    fn stale_generation_cannot_seal_reused_frames() {
        let mut f = pool();
        let old_binding = Binding {
            generation: BIND.generation - 1,
            ..BIND
        };
        let mut stale = Artifact::new(old_binding, REF).unwrap();
        let old_owner = Owner::Artifact {
            id: ID,
            generation: old_binding.generation,
            sealed: false,
        };
        let mut old_pages = vec![0; stale.pages()];
        f.allocate_many(old_owner, &mut old_pages, |_| {}).unwrap();
        stale.begin_fill().unwrap();
        let mut premature = [0; 1];
        assert_eq!(
            f.allocate_many(owner(false), &mut premature, |_| {}),
            Err(Error::Owner)
        );
        for pa in old_pages {
            f.free(pa, old_owner, |_| {}).unwrap();
        }

        let (current, current_pages) = filled(&mut f);
        assert_eq!(current.state(), State::Filling);
        assert_eq!(stale.seal(&mut f), Err(ArtifactError::Frames(Error::Range)));
        assert_eq!(stale.state(), State::Filling);
        assert_eq!(current.state(), State::Filling);
        assert!(current_pages
            .iter()
            .all(|&pa| f.owner(pa) == Ok(owner(false))));
    }

    // Design test 3: nothing links before Verified; Verified needs a complete
    // read and an exact digest; a one-bit difference fails closed to Rejected.
    #[test]
    fn links_only_from_verified_and_mismatch_rejects() {
        let mut f = pool();
        let mut empty = Artifact::new(BIND, REF).unwrap();
        assert_eq!(empty.link(BIND), Err(ArtifactError::State));
        let (mut a, _) = filled(&mut f);
        assert_eq!(a.link(BIND), Err(ArtifactError::State));
        assert_eq!(a.verify(BIND, &REF), Err(ArtifactError::State));
        a.seal(&mut f).unwrap();
        assert_eq!(a.link(BIND), Err(ArtifactError::State));
        assert_eq!(a.verify(BIND, &REF), Err(ArtifactError::Incomplete));
        assert_eq!(read_all(&mut a), BIND.length);
        let mut wrong = REF;
        wrong[31] ^= 1;
        // The failure path runs on its own, identically prepared artifact.
        let mut g = pool();
        let (mut rejected, _) = filled(&mut g);
        rejected.seal(&mut g).unwrap();
        assert_eq!(rejected.link(BIND), Err(ArtifactError::State));
        assert_eq!(read_all(&mut rejected), BIND.length);
        assert_eq!(rejected.verify(BIND, &wrong), Err(ArtifactError::Mismatch));
        assert_eq!(rejected.state(), State::Rejected);
        assert_eq!(rejected.link(BIND), Err(ArtifactError::State));
        assert_eq!(rejected.verify(BIND, &REF), Err(ArtifactError::State));
        // A verdict for another identity, generation or length changes nothing.
        for stale in [
            Binding { id: ID + 1, ..BIND },
            Binding {
                generation: BIND.generation + 1,
                ..BIND
            },
            Binding {
                length: BIND.length - 1,
                ..BIND
            },
        ] {
            assert_eq!(a.verify(stale, &REF), Err(ArtifactError::Stale));
            assert_eq!(a.link(stale), Err(ArtifactError::Stale));
            assert_eq!(a.state(), State::Sealed);
        }
        a.verify(BIND, &REF).unwrap();
        assert_eq!(a.link(BIND), Ok(BIND.length));
        empty.release().unwrap();
        assert_eq!(empty.link(BIND), Err(ArtifactError::State));
    }

    #[test]
    fn read_is_bounded_sequential_and_advances_only_on_success() {
        let big = Binding {
            length: READ_CHUNK * 2 + 9,
            ..BIND
        };
        let mut f = pool();
        let mut a = Artifact::new(big, REF).unwrap();
        let mut pages = vec![0; a.pages()];
        f.allocate_many(owner(false), &mut pages, |_| {}).unwrap();
        let ok = |_: u64, _: u64| Ok::<(), ()>(());
        assert_eq!(a.read(big, 1, ok), Err(ArtifactError::State));
        a.begin_fill().unwrap();
        a.seal(&mut f).unwrap();
        assert_eq!(a.read(BIND, 1, ok), Err(ArtifactError::Stale));
        assert_eq!(
            a.read(big, u64::MAX, |_, _| Err(())),
            Err(ArtifactError::Fault)
        );
        assert_eq!(a.cursor(), 0);
        let mut seen = Vec::new();
        for _ in 0..3 {
            a.read(big, u64::MAX, |o, l| {
                seen.push((o, l));
                Ok::<(), ()>(())
            })
            .unwrap();
        }
        assert_eq!(
            seen,
            [
                (0, READ_CHUNK),
                (READ_CHUNK, READ_CHUNK),
                (2 * READ_CHUNK, 9)
            ]
        );
        assert_eq!(a.read(big, 100, ok), Ok(0));
        assert_eq!(a.verify(big, &REF), Ok(()));
    }

    #[test]
    fn read_honours_a_smaller_caller_buffer() {
        let mut f = pool();
        let (mut a, _) = filled(&mut f);
        a.seal(&mut f).unwrap();
        assert_eq!(a.read(BIND, 10, |_, _| Ok::<(), ()>(())), Ok(10));
        assert_eq!(a.cursor(), 10);
        assert_eq!(a.verify(BIND, &REF), Err(ArtifactError::Incomplete));
    }

    // Design test 4: reservation beyond the budget fails with no owner change;
    // artifact frames never count toward USER_QUOTA.
    #[test]
    fn artifact_budget_is_separate_from_user_quota() {
        let mut f = pool();
        let before = f.availability();
        let mut over = vec![0; ARTIFACT_QUOTA + 1];
        assert_eq!(
            f.allocate_many(owner(false), &mut over, |_| panic!("zeroed on failure")),
            Err(Error::Quota)
        );
        assert_eq!(f.availability(), before);
        let mut full = vec![0; ARTIFACT_QUOTA];
        f.allocate_many(owner(false), &mut full, |_| {}).unwrap();
        assert_eq!(
            f.allocate(owner(false), |_| panic!("zeroed on failure")),
            Err(Error::Quota)
        );
        // The budget is global: another artifact id gets no second budget.
        assert_eq!(
            f.allocate(
                Owner::Artifact {
                    id: ID + 1,
                    generation: BIND.generation,
                    sealed: false
                },
                |_| {}
            ),
            Err(Error::Quota)
        );
        // Id 0 and pre-sealed owners are refused.
        assert_eq!(
            f.allocate(
                Owner::Artifact {
                    id: 0,
                    generation: BIND.generation,
                    sealed: false
                },
                |_| {}
            ),
            Err(Error::Owner)
        );
        assert_eq!(f.allocate(owner(true), |_| {}), Err(Error::Owner));
        // The process quota is untouched by the artifact frames.
        let mut private = vec![0; USER_QUOTA];
        f.allocate_many(Owner::User(1, Kind::Data), &mut private, |_| {})
            .unwrap();
        assert_eq!(
            f.allocate(Owner::User(1, Kind::Data), |_| {}),
            Err(Error::Quota)
        );
        assert!(Artifact::new(
            Binding {
                length: ARTIFACT_QUOTA as u64 * PAGE + 1,
                ..BIND
            },
            REF
        )
        .is_err());
        assert!(Artifact::new(Binding { length: 0, ..BIND }, REF).is_err());
        assert!(Artifact::new(Binding { id: 0, ..BIND }, REF).is_err());
    }

    // Design test 5 (host part): release needs zero links, and every frame is
    // handed to the zeroing callback before it becomes free. Real zeroing of
    // physical memory is native evidence (design test 13), not claimed here.
    #[test]
    fn release_needs_zero_links_and_frees_through_zeroing() {
        let mut f = pool();
        let before = f.availability();
        let (mut a, pages) = filled(&mut f);
        a.seal(&mut f).unwrap();
        read_all(&mut a);
        a.verify(BIND, &REF).unwrap();
        a.link(BIND).unwrap();
        a.link(BIND).unwrap();
        assert_eq!(a.release(), Err(ArtifactError::Busy));
        a.unlink(BIND).unwrap();
        assert_eq!(a.release(), Err(ArtifactError::Busy));
        a.unlink(BIND).unwrap();
        assert_eq!(a.unlink(BIND), Err(ArtifactError::State));
        // A frame with a live alias cannot be freed.
        f.map(pages[0], false, false, true).unwrap();
        assert_eq!(f.free(pages[0], owner(true), |_| {}), Err(Error::Mapped));
        f.unmap(pages[0], false, false).unwrap();
        a.release().unwrap();
        assert_eq!(a.release(), Err(ArtifactError::State));
        let mut zeroed = Vec::new();
        for &pa in &pages {
            f.free(pa, owner(true), |z| zeroed.push(z)).unwrap();
        }
        assert_eq!(zeroed, pages);
        assert_eq!(f.availability(), before);
    }
}
