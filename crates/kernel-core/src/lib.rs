#![no_std]
#![forbid(unsafe_code)]
//! Fixed-capacity ownership and physical-alias policy. No physical dereferences.
pub const PAGE: u64 = 4096;
pub const USER_QUOTA: usize = 2048;
pub mod user;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Reserved,
    Free,
    Kernel,
    PageTable(u16),
    User(u64, Kind),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Data,
    Stack,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Range,
    Overlap,
    Owner,
    Exhausted,
    Quota,
    Mapped,
    Permission,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}
impl Range {
    pub fn pages(start: u64, pages: u64) -> Result<Self, Error> {
        let end = start
            .checked_add(pages.checked_mul(PAGE).ok_or(Error::Range)?)
            .ok_or(Error::Range)?;
        if pages == 0 || start % PAGE != 0 {
            return Err(Error::Range);
        }
        Ok(Self { start, end })
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

pub struct Frames<const N: usize> {
    owners: [Owner; N],
    aliases: [Alias; N],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alias {
    pub writable: u16,
    pub executable: u16,
    pub total: u16,
}
const EMPTY: Alias = Alias {
    writable: 0,
    executable: 0,
    total: 0,
};
impl<const N: usize> Default for Frames<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> Frames<N> {
    pub const fn new() -> Self {
        Self {
            owners: [Owner::Reserved; N],
            aliases: [EMPTY; N],
        }
    }
    /// Call only after validating the complete firmware map for overlaps/overflow.
    /// Only normal WB-capable ConventionalMemory is reusable. UEFI attributes
    /// describe capabilities, so UC/WC/WT/WB may coexist (OVMF reports 0x0f).
    /// All non-cache capability flags and unknown attributes fail closed.
    pub fn admit(&mut self, range: Range, memory_type: u32, attributes: u64, retained: &[Range]) {
        if memory_type != 7 || attributes & !0x0f != 0 || attributes & 8 == 0 {
            return;
        }
        let end = core::cmp::min(range.end / PAGE, N as u64) as usize;
        for i in core::cmp::min(range.start / PAGE, N as u64) as usize..end {
            let page = Range {
                start: i as u64 * PAGE,
                end: (i as u64 + 1) * PAGE,
            };
            if i != 0 && !retained.iter().any(|r| r.overlaps(page)) {
                self.owners[i] = Owner::Free;
            }
        }
    }
    pub fn owner(&self, address: u64) -> Result<Owner, Error> {
        Ok(self.owners[self.index(address)?])
    }
    pub fn retain_kernel(&mut self, range: Range) -> Result<(), Error> {
        for address in (range.start..range.end).step_by(PAGE as usize) {
            let i = self.index(address)?;
            if self.owners[i] != Owner::Reserved {
                return Err(Error::Owner);
            }
        }
        for address in (range.start..range.end).step_by(PAGE as usize) {
            self.owners[(address / PAGE) as usize] = Owner::Kernel;
        }
        Ok(())
    }
    fn index(&self, address: u64) -> Result<usize, Error> {
        if address % PAGE != 0 || address / PAGE >= N as u64 {
            return Err(Error::Range);
        }
        Ok((address / PAGE) as usize)
    }
    pub fn allocate(&mut self, owner: Owner, mut zero: impl FnMut(u64)) -> Result<u64, Error> {
        if matches!(owner, Owner::Free | Owner::Reserved) {
            return Err(Error::Owner);
        }
        if let Owner::User(pid, _) = owner {
            if pid == 0 {
                return Err(Error::Owner);
            }
            if self
                .owners
                .iter()
                .filter(|o| matches!(o, Owner::User(p, _) if *p == pid))
                .count()
                >= USER_QUOTA
            {
                return Err(Error::Quota);
            }
        }
        let i = self
            .owners
            .iter()
            .position(|o| *o == Owner::Free)
            .ok_or(Error::Exhausted)?;
        let address = i as u64 * PAGE;
        zero(address);
        self.owners[i] = owner;
        Ok(address)
    }
    pub fn free(
        &mut self,
        address: u64,
        owner: Owner,
        mut zero: impl FnMut(u64),
    ) -> Result<(), Error> {
        let i = self.index(address)?;
        if matches!(owner, Owner::Reserved | Owner::Free) || self.owners[i] != owner {
            return Err(Error::Owner);
        }
        if self.aliases[i] != EMPTY {
            return Err(Error::Mapped);
        }
        zero(address);
        self.owners[i] = Owner::Free;
        Ok(())
    }
    pub fn map(
        &mut self,
        address: u64,
        writable: bool,
        executable: bool,
        user: bool,
    ) -> Result<(), Error> {
        let i = self.index(address)?;
        if matches!(self.owners[i], Owner::Free)
            || (user && !matches!(self.owners[i], Owner::User(_, _)))
        {
            return Err(Error::Owner);
        }
        let a = self.aliases[i];
        if (writable && (executable || a.executable != 0)) || (executable && a.writable != 0) {
            return Err(Error::Permission);
        }
        let total = a.total.checked_add(1).ok_or(Error::Quota)?;
        self.aliases[i] = Alias {
            writable: a.writable + u16::from(writable),
            executable: a.executable + u16::from(executable),
            total,
        };
        Ok(())
    }
    /// Caller must clear the PTE and invalidate its TLB entry BEFORE releasing metadata.
    pub fn unmap(&mut self, address: u64, writable: bool, executable: bool) -> Result<(), Error> {
        let i = self.index(address)?;
        let a = self.aliases[i];
        // W^X makes writable, executable and RO/NX aliases disjoint classes.
        // A total count alone cannot authorize removing a different alias class.
        if (writable && executable)
            || a.total == 0
            || (writable && a.writable == 0)
            || (executable && a.executable == 0)
            || (!writable
                && !executable
                && u32::from(a.total) <= u32::from(a.writable) + u32::from(a.executable))
        {
            return Err(Error::Mapped);
        }
        let candidate = Alias {
            total: a.total - 1,
            writable: a.writable - u16::from(writable),
            executable: a.executable - u16::from(executable),
        };
        if u32::from(candidate.writable) + u32::from(candidate.executable)
            > u32::from(candidate.total)
            || (candidate.writable != 0 && candidate.executable != 0)
        {
            return Err(Error::Mapped);
        }
        self.aliases[i] = candidate;
        Ok(())
    }
    pub fn alias(&self, address: u64) -> Result<Alias, Error> {
        Ok(self.aliases[self.index(address)?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pool<const N: usize>() -> Frames<N> {
        let mut p = Frames::new();
        p.admit(Range::pages(0, N as u64).unwrap(), 7, 8, &[]);
        p
    }
    #[test]
    fn checked_ranges() {
        assert_eq!(Range::pages(0, u64::MAX), Err(Error::Range));
        assert_eq!(Range::pages(u64::MAX - 4095, 2), Err(Error::Range));
        assert!(Range::pages(4096, 2)
            .unwrap()
            .overlaps(Range::pages(8192, 1).unwrap()));
    }
    #[test]
    fn reserved_and_special_memory() {
        let mut p = Frames::<16>::new();
        p.admit(
            Range::pages(0, 16).unwrap(),
            7,
            8,
            &[Range::pages(8192, 2).unwrap()],
        );
        assert_eq!(p.owner(8192), Ok(Owner::Reserved));
        assert_eq!(p.owner(0), Ok(Owner::Reserved));
        let mut p = Frames::<16>::new();
        for (t, a) in [(1, 8), (2, 8), (4, 8), (7, 0x8000000000000008), (7, 0x8008)] {
            p.admit(Range::pages(4096, 1).unwrap(), t, a, &[]);
            assert_eq!(p.owner(4096), Ok(Owner::Reserved));
        }
    }
    #[test]
    fn ovmf_cache_capabilities_are_not_special_memory() {
        let mut p = Frames::<4>::new();
        p.admit(Range::pages(PAGE, 1).unwrap(), 7, 0x0f, &[]);
        assert_eq!(p.owner(PAGE), Ok(Owner::Free));
        for attributes in [0u64, 1, 0x1f, 0x100f, 0x800f, 0x4000f, 0x800000000000000f] {
            let mut p = Frames::<4>::new();
            p.admit(Range::pages(PAGE, 1).unwrap(), 7, attributes, &[]);
            assert_eq!(p.owner(PAGE), Ok(Owner::Reserved));
        }
    }
    #[test]
    fn unique_exhaustion_ownership_and_zero_order() {
        let mut p = pool::<4>();
        let mut zeros = 0;
        let a = p
            .allocate(Owner::User(1, Kind::Data), |_| zeros += 1)
            .unwrap();
        assert_eq!(zeros, 1);
        assert_eq!(
            p.free(a, Owner::User(2, Kind::Data), |_| panic!()),
            Err(Error::Owner)
        );
        let b = p.allocate(Owner::Kernel, |_| {}).unwrap();
        let c = p.allocate(Owner::PageTable(1), |_| {}).unwrap();
        assert!(a != b && a != c && b != c);
        assert_eq!(
            p.allocate(Owner::Kernel, |_| panic!()),
            Err(Error::Exhausted)
        );
        p.free(a, Owner::User(1, Kind::Data), |_| zeros += 1)
            .unwrap();
        assert_eq!(zeros, 2);
        assert_eq!(
            p.allocate(Owner::User(2, Kind::Data), |_| zeros += 1),
            Ok(a)
        );
        assert_eq!(zeros, 3);
        assert_eq!(p.free(b, Owner::Kernel, |_| {}), Ok(()));
        assert_eq!(p.free(b, Owner::Kernel, |_| panic!()), Err(Error::Owner));
    }
    #[test]
    fn physical_alias_wx_and_mapped_free() {
        let mut p = pool::<3>();
        let a = p.allocate(Owner::User(1, Kind::Text), |_| {}).unwrap();
        p.map(a, false, true, true).unwrap();
        assert_eq!(p.map(a, true, false, false), Err(Error::Permission));
        assert_eq!(
            p.free(a, Owner::User(1, Kind::Text), |_| panic!()),
            Err(Error::Mapped)
        );
        p.unmap(a, false, true).unwrap();
        p.map(a, true, false, false).unwrap();
        assert_eq!(p.map(a, false, true, true), Err(Error::Permission));
        assert_eq!(p.map(8192, true, false, true), Err(Error::Owner));
    }
    #[test]
    fn unmap_mismatched_alias_is_atomic_and_cannot_free_executable() {
        for (writable, executable) in [(false, true), (true, false), (false, false)] {
            let mut p = pool::<3>();
            let owner = Owner::User(1, Kind::Text);
            let a = p.allocate(owner, |_| {}).unwrap();
            p.map(a, writable, executable, true).unwrap();
            let before = p.alias(a).unwrap();
            for (wrong_w, wrong_x) in [(false, false), (false, true), (true, false), (true, true)] {
                if (wrong_w, wrong_x) == (writable, executable) {
                    continue;
                }
                assert_eq!(p.unmap(a, wrong_w, wrong_x), Err(Error::Mapped));
                assert_eq!(p.alias(a), Ok(before));
                assert_eq!(p.owner(a), Ok(owner));
                assert_eq!(
                    p.free(a, owner, |_| panic!("mapped frame must not be zeroed")),
                    Err(Error::Mapped)
                );
            }
            p.unmap(a, writable, executable).unwrap();
            assert_eq!(p.alias(a), Ok(EMPTY));
            assert_eq!(p.unmap(a, writable, executable), Err(Error::Mapped));
            p.free(a, owner, |_| {}).unwrap();
        }
    }
    #[test]
    fn quota_no_partial_allocation() {
        // Heap allocation avoids the Windows test runner's small thread stack.
        extern crate std;
        let mut p = std::boxed::Box::new(pool::<2051>());
        for _ in 0..2048 {
            p.allocate(Owner::User(1, Kind::Data), |_| {}).unwrap();
        }
        assert_eq!(
            p.allocate(Owner::User(1, Kind::Stack), |_| panic!()),
            Err(Error::Quota)
        );
        assert!(p.allocate(Owner::User(2, Kind::Data), |_| {}).is_ok());
    }
}
