//! Physical-frame pressure tiers and a one-shot reclaim opportunity.
use crate::{Error, FrameAvailability};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Normal,
    Elevated,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thresholds {
    pub elevated_at_or_below: usize,
    pub critical_at_or_below: usize,
}

impl Thresholds {
    /// Derive watermarks from admitted physical memory, not the metadata-array
    /// size. The critical reserve is 1/32 and elevated begins at 1/8.
    pub const fn for_managed_frames(managed: usize) -> Self {
        let elevated_at_or_below = managed / 8 + if managed % 8 == 0 { 0 } else { 1 };
        let critical_at_or_below = managed / 32 + if managed % 32 == 0 { 0 } else { 1 };
        Self {
            elevated_at_or_below,
            critical_at_or_below,
        }
    }

    pub const fn classify(self, free: usize) -> Level {
        if free <= self.critical_at_or_below {
            Level::Critical
        } else if free <= self.elevated_at_or_below {
            Level::Elevated
        } else {
            Level::Normal
        }
    }
}

pub fn classify(availability: FrameAvailability) -> Result<Level, Error> {
    if availability.free > availability.managed {
        return Err(Error::Range);
    }
    Ok(Thresholds::for_managed_frames(availability.managed).classify(availability.free))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opportunity {
    NotCritical,
    Granted,
    AlreadyGranted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReclaimBudget {
    granted: bool,
}

impl ReclaimBudget {
    pub const fn new() -> Self {
        Self { granted: false }
    }

    /// Grant one bounded attempt only at Critical pressure. This does not pick
    /// a victim or reclaim memory; the authorized consumer performs that work.
    pub fn offer(&mut self, level: Level) -> Opportunity {
        if level != Level::Critical {
            return Opportunity::NotCritical;
        }
        if self.granted {
            return Opportunity::AlreadyGranted;
        }
        self.granted = true;
        Opportunity::Granted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_watermarks_scale_with_managed_frames_and_include_boundaries() {
        let thresholds = Thresholds::for_managed_frames(1024);
        assert_eq!(thresholds.elevated_at_or_below, 128);
        assert_eq!(thresholds.critical_at_or_below, 32);
        assert_eq!(thresholds.classify(129), Level::Normal);
        assert_eq!(thresholds.classify(128), Level::Elevated);
        assert_eq!(thresholds.classify(33), Level::Elevated);
        assert_eq!(thresholds.classify(32), Level::Critical);
        assert_eq!(thresholds.classify(0), Level::Critical);
    }

    #[test]
    fn invalid_free_frame_count_is_rejected() {
        assert_eq!(
            classify(FrameAvailability {
                managed: 4,
                free: 5
            }),
            Err(Error::Range)
        );
    }

    #[test]
    fn reclaim_budget_grants_exactly_one_critical_opportunity() {
        let mut budget = ReclaimBudget::new();
        assert_eq!(budget.offer(Level::Normal), Opportunity::NotCritical);
        assert_eq!(budget.offer(Level::Elevated), Opportunity::NotCritical);
        assert_eq!(budget.offer(Level::Critical), Opportunity::Granted);
        assert_eq!(budget.offer(Level::Critical), Opportunity::AlreadyGranted);
        assert_eq!(budget.offer(Level::Normal), Opportunity::NotCritical);
        assert_eq!(budget.offer(Level::Critical), Opportunity::AlreadyGranted);
    }
}
