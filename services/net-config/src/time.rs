use nieuw_os_net_stack::{MAX_MONOTONIC_TICKS, MONOTONIC_HZ};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ClockWentBackwards,
    ClockOverflow,
}

/// DHCP seconds are floored: quantization must never extend lease expiry.
pub fn seconds(ticks: u64) -> Result<u64, Error> {
    if ticks > MAX_MONOTONIC_TICKS {
        return Err(Error::ClockOverflow);
    }
    Ok(ticks / MONOTONIC_HZ)
}

pub fn ticks(seconds: u64) -> Result<u64, Error> {
    seconds
        .checked_mul(MONOTONIC_HZ)
        .filter(|value| *value <= MAX_MONOTONIC_TICKS)
        .ok_or(Error::ClockOverflow)
}

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    last_ticks: u64,
}

impl Clock {
    pub fn new(ticks: u64) -> Result<Self, Error> {
        seconds(ticks)?;
        Ok(Self { last_ticks: ticks })
    }

    pub fn observe(&mut self, ticks: u64) -> Result<u64, Error> {
        if ticks < self.last_ticks {
            return Err(Error::ClockWentBackwards);
        }
        let seconds = seconds(ticks)?;
        self.last_ticks = ticks;
        Ok(seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_never_rounds_expiry_up_or_wraps() {
        assert_eq!(seconds(199), Ok(1));
        let expires_at = ticks(seconds(199).unwrap() + 80).unwrap();
        assert_eq!(expires_at, 8100);
        assert!(expires_at < 199 + 80 * MONOTONIC_HZ);
        assert_eq!(seconds(MAX_MONOTONIC_TICKS), Ok(MAX_MONOTONIC_TICKS / 100));
        assert_eq!(seconds(MAX_MONOTONIC_TICKS + 1), Err(Error::ClockOverflow));
        assert_eq!(ticks(u64::MAX), Err(Error::ClockOverflow));
        let limit = MAX_MONOTONIC_TICKS / MONOTONIC_HZ;
        assert!(ticks(limit).is_ok());
        assert_eq!(ticks(limit + 1), Err(Error::ClockOverflow));
    }

    #[test]
    fn rollback_within_the_same_dhcp_second_is_rejected() {
        let mut clock = Clock::new(199).unwrap();
        assert_eq!(clock.observe(199), Ok(1));
        assert_eq!(clock.observe(198), Err(Error::ClockWentBackwards));
        assert_eq!(clock.observe(u64::MAX), Err(Error::ClockOverflow));
        assert_eq!(clock.observe(200), Ok(2));
    }
}
