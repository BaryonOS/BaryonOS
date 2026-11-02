//! Idle-only scheduling: no transaction/retry deadline is created or changed.
use nieuw_os_sdk_runtime::{Error, ReceiveEndpoint, WaitCondition};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Legacy,
    Wait,
}
impl Mode {
    /// Only the reviewed display/input ABI profiles are compatible with this app.
    pub const fn from_abi(version: u64) -> Option<Self> {
        match version {
            0x1_0000_0008 | 0x1_0000_0009 => Some(Self::Legacy),
            0x1_0000_000a | 0x1_0000_000b => Some(Self::Wait),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Poll {
    conditions: [WaitCondition; 1],
    deadline: u64,
}
impl Poll {
    /// PEEK is a pull protocol. A new input event need not enqueue a reply, so
    /// even an empty input observation may sleep only until the next PIT tick.
    /// The stable control grant survives presentation retirement/rebinding.
    pub fn new(now: u64, control: ReceiveEndpoint) -> Option<Self> {
        Some(Self {
            conditions: [WaitCondition::receive(control)],
            deadline: now.checked_add(1)?,
        })
    }
    pub const fn deadline(&self) -> u64 {
        self.deadline
    }
    /// A wake is advisory: the caller still validates/receives normally.
    /// A clock, rights or mask failure is never a fallback to polling.
    pub fn park(
        &self,
        wait: impl FnOnce(&[WaitCondition], u64) -> Result<u16, Error>,
    ) -> Result<(), Error> {
        match wait(&self.conditions, self.deadline)? {
            0 | 1 => Ok(()),
            _ => Err(Error::InvalidReturn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::Handle;

    fn control() -> ReceiveEndpoint {
        ReceiveEndpoint::from_handle(Handle::from_raw(0x1_0000_0001).unwrap())
    }
    #[test]
    fn exact_supported_profiles_only() {
        for version in [0x1_0000_0008, 0x1_0000_0009] {
            assert_eq!(Mode::from_abi(version), Some(Mode::Legacy));
        }
        for version in [0x1_0000_000a, 0x1_0000_000b] {
            assert_eq!(Mode::from_abi(version), Some(Mode::Wait));
        }
        for version in [0, 0x1_0000_0007, 0x1_0000_000c, 0x2_0000_000a, u64::MAX] {
            assert_eq!(Mode::from_abi(version), None);
        }
    }
    #[test]
    fn next_tick_preserves_control_condition_and_rejects_overflow() {
        let plan = Poll::new(80, control()).unwrap();
        assert_eq!(plan.deadline(), 81);
        for mask in [0, 1] {
            assert_eq!(
                plan.park(|conditions, deadline| {
                    assert_eq!(conditions, &[WaitCondition::receive(control())]);
                    assert_eq!(deadline, 81);
                    // An arrival after Empty may report immediate control
                    // readiness; this plan neither consumes it nor moves time.
                    Ok(mask)
                }),
                Ok(())
            );
        }
        assert_eq!(Poll::new(u64::MAX, control()), None);
        assert_eq!(
            Poll::new(u64::MAX - 1, control()).unwrap().deadline(),
            u64::MAX
        );
    }
    #[test]
    fn wait_failure_and_unexpected_mask_never_become_success() {
        let plan = Poll::new(80, control()).unwrap();
        for error in [Error::Kernel(-4), Error::Kernel(-9), Error::InvalidReturn] {
            assert_eq!(plan.park(|_, _| Err(error)), Err(error));
        }
        assert_eq!(plan.park(|_, _| Ok(2)), Err(Error::InvalidReturn));
        assert_eq!(plan.deadline(), 81);
    }
}
