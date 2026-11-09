//! Conservative, bounded damage journal in physical surface-local pixels.
//! No hash or app-authored rectangle grants authority. Transport still checks
//! actor, geometry, ordering, backing preservation and completion separately.

const CAPACITY: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Bounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
impl Bounds {
    const EMPTY: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };
    fn end(self) -> Option<(u32, u32)> {
        Some((
            self.x.checked_add(self.width)?,
            self.y.checked_add(self.height)?,
        ))
    }
    fn joins(self, other: Self) -> bool {
        let (ax, ay) = self.end().expect("validated bounds");
        let (bx, by) = other.end().expect("validated bounds");
        self.x <= bx && other.x <= ax && self.y <= by && other.y <= ay
    }
    fn union(self, other: Self) -> Self {
        let (ax, ay) = self.end().expect("validated bounds");
        let (bx, by) = other.end().expect("validated bounds");
        let (x, y) = (self.x.min(other.x), self.y.min(other.y));
        Self {
            x,
            y,
            width: ax.max(bx) - x,
            height: ay.max(by) - y,
        }
    }
}

/// Retain this journal across superseded partial paints. Only a confirmed
/// complete current-generation presentation may replace it with EMPTY.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Damage {
    full: bool,
    length: usize,
    regions: [Bounds; CAPACITY],
}
impl Default for Damage {
    fn default() -> Self {
        Self::EMPTY
    }
}
impl Damage {
    pub const EMPTY: Self = Self {
        full: false,
        length: 0,
        regions: [Bounds::EMPTY; CAPACITY],
    };
    pub const FULL: Self = Self {
        full: true,
        length: 0,
        regions: [Bounds::EMPTY; CAPACITY],
    };
    pub const fn is_empty(self) -> bool {
        !self.full && self.length == 0
    }
    pub const fn is_full(self) -> bool {
        self.full
    }
    pub fn include(&mut self, other: Self) {
        if other.full {
            *self = Self::FULL;
            return;
        }
        for region in &other.regions[..other.length] {
            self.include_rect(region.x, region.y, region.width, region.height);
        }
    }
    pub fn include_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        if self.full || width == 0 || height == 0 {
            return;
        }
        let mut region = Bounds {
            x,
            y,
            width,
            height,
        };
        if region.end().is_none() {
            *self = Self::FULL;
            return;
        }
        let mut index = 0;
        while index < self.length {
            if self.regions[index].joins(region) {
                region = region.union(self.regions[index]);
                self.length -= 1;
                self.regions[index] = self.regions[self.length];
                self.regions[self.length] = Bounds::EMPTY;
                index = 0;
            } else {
                index += 1;
            }
        }
        if self.length == CAPACITY {
            *self = Self::FULL;
            return;
        }
        self.regions[self.length] = region;
        self.length += 1;
    }
    pub fn intersects(self, x: u32, y: u32, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        let Some((right, bottom)) = (Bounds {
            x,
            y,
            width,
            height,
        })
        .end() else {
            return true;
        };
        self.full
            || self.regions[..self.length].iter().any(|region| {
                let (rx, ry) = region.end().expect("validated bounds");
                x < rx && region.x < right && y < ry && region.y < bottom
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distant_text_and_status_do_not_dirty_the_gap() {
        let mut damage = Damage::EMPTY;
        damage.include_rect(20, 100, 400, 24);
        damage.include_rect(20, 450, 400, 32);
        assert!(damage.intersects(32, 104, 32, 8));
        assert!(damage.intersects(32, 464, 32, 8));
        assert!(!damage.intersects(32, 256, 32, 8));
        assert!(!damage.intersects(420, 100, 32, 8));
    }
    #[test]
    fn journal_keeps_old_partial_work_until_explicit_presentation_reset() {
        let mut journal = Damage::EMPTY;
        journal.include_rect(0, 0, 32, 8);
        let old = journal;
        journal.include_rect(160, 80, 32, 8);
        journal.include(old);
        assert!(journal.intersects(0, 0, 32, 8));
        assert!(journal.intersects(160, 80, 32, 8));
        journal = Damage::EMPTY;
        assert!(!journal.intersects(0, 0, 32, 8));
    }
    #[test]
    fn coalescing_preserves_every_included_pixel_and_allows_half_open_edges() {
        let mut damage = Damage::EMPTY;
        for x in [0, 16, 8] {
            damage.include_rect(x, 0, 8, 8);
        }
        for x in 0..24 {
            assert!(damage.intersects(x, 0, 1, 1));
        }
        assert!(!damage.intersects(24, 0, 1, 1));
        assert!(!damage.intersects(0, 8, 1, 1));
    }
    #[test]
    fn overflow_or_excess_disjoint_regions_conservatively_requires_full_paint() {
        let mut damage = Damage::EMPTY;
        damage.include_rect(u32::MAX, 0, 1, 1);
        assert!(damage.is_full());
        let mut damage = Damage::EMPTY;
        for x in [0, 64, 128, 192, 256] {
            damage.include_rect(x, 0, 8, 8);
        }
        assert!(damage.is_full());
        assert!(Damage::EMPTY.intersects(u32::MAX, 0, 1, 1));
        assert!(!Damage::FULL.intersects(0, 0, 0, 8));
    }
}
