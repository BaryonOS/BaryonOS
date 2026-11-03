#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Active,
    Idle,
    Suspending,
    Suspended,
    Resuming,
    Resetting,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transition {
    EnterIdle,
    Wake,
    BeginSuspend,
    SuspendComplete,
    CancelSuspend,
    SuspendFailed,
    BeginResume,
    ResumeComplete,
    ResumeFailed,
    BeginReset,
    ResetComplete,
    ResetFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidTransition,
    StaleGeneration,
    GenerationExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransitionRequest {
    generation: u64,
    transition: Transition,
}

pub struct Lifecycle {
    state: State,
    generation: u64,
    suspend_origin: Option<State>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    pub const fn new() -> Self {
        Self {
            state: State::Active,
            generation: 1,
            suspend_origin: None,
        }
    }

    pub const fn state(&self) -> State {
        self.state
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn request(&self, transition: Transition) -> TransitionRequest {
        TransitionRequest {
            generation: self.generation,
            transition,
        }
    }

    pub fn apply(&mut self, request: TransitionRequest) -> Result<State, Error> {
        if request.generation != self.generation {
            return Err(Error::StaleGeneration);
        }
        let next = match (self.state, request.transition) {
            (State::Active | State::Idle, Transition::BeginSuspend) => {
                self.suspend_origin = Some(self.state);
                State::Suspending
            }
            (State::Suspending, Transition::SuspendComplete) => {
                self.suspend_origin = None;
                State::Suspended
            }
            (State::Suspending, Transition::CancelSuspend) => {
                let origin = self.suspend_origin.ok_or(Error::InvalidTransition)?;
                self.suspend_origin = None;
                origin
            }
            (State::Suspending, Transition::SuspendFailed) => {
                self.suspend_origin = None;
                State::Failed
            }
            (State::Suspended, Transition::BeginResume) => State::Resuming,
            (State::Resuming, Transition::ResumeComplete) => State::Active,
            (State::Resuming, Transition::ResumeFailed) => State::Failed,
            (State::Active, Transition::EnterIdle) => State::Idle,
            (State::Idle, Transition::Wake) => State::Active,
            (
                State::Active
                | State::Idle
                | State::Suspending
                | State::Suspended
                | State::Resuming
                | State::Failed,
                Transition::BeginReset,
            ) => {
                self.suspend_origin = None;
                State::Resetting
            }
            (State::Resetting, Transition::ResetComplete) => State::Active,
            (State::Resetting, Transition::ResetFailed) => State::Failed,
            _ => return Err(Error::InvalidTransition),
        };
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::GenerationExhausted)?;
        self.state = next;
        self.generation = generation;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(lifecycle: &mut Lifecycle, transition: Transition) -> Result<State, Error> {
        lifecycle.apply(lifecycle.request(transition))
    }

    #[test]
    fn active_idle_suspend_and_resume_follow_valid_order() {
        let mut lifecycle = Lifecycle::new();
        assert_eq!(lifecycle.state(), State::Active);
        assert_eq!(
            apply(&mut lifecycle, Transition::EnterIdle),
            Ok(State::Idle)
        );
        assert_eq!(apply(&mut lifecycle, Transition::Wake), Ok(State::Active));
        assert_eq!(
            apply(&mut lifecycle, Transition::BeginSuspend),
            Ok(State::Suspending)
        );
        assert_eq!(
            apply(&mut lifecycle, Transition::SuspendComplete),
            Ok(State::Suspended)
        );
        assert_eq!(
            apply(&mut lifecycle, Transition::BeginResume),
            Ok(State::Resuming)
        );
        assert_eq!(
            apply(&mut lifecycle, Transition::ResumeComplete),
            Ok(State::Active)
        );
    }

    #[test]
    fn invalid_transition_does_not_change_state_or_generation() {
        let mut lifecycle = Lifecycle::new();
        let generation = lifecycle.generation();
        assert_eq!(
            apply(&mut lifecycle, Transition::SuspendComplete),
            Err(Error::InvalidTransition)
        );
        assert_eq!(lifecycle.state(), State::Active);
        assert_eq!(lifecycle.generation(), generation);
    }

    #[test]
    fn cancellation_returns_to_the_suspend_origin() {
        let mut lifecycle = Lifecycle::new();
        apply(&mut lifecycle, Transition::EnterIdle).unwrap();
        apply(&mut lifecycle, Transition::BeginSuspend).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::CancelSuspend),
            Ok(State::Idle)
        );

        apply(&mut lifecycle, Transition::BeginSuspend).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::CancelSuspend),
            Ok(State::Idle)
        );
    }

    #[test]
    fn suspend_resume_and_reset_failures_are_explicit() {
        let mut lifecycle = Lifecycle::new();
        apply(&mut lifecycle, Transition::BeginSuspend).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::SuspendFailed),
            Ok(State::Failed)
        );
        apply(&mut lifecycle, Transition::BeginReset).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::ResetComplete),
            Ok(State::Active)
        );
        apply(&mut lifecycle, Transition::BeginSuspend).unwrap();
        apply(&mut lifecycle, Transition::SuspendComplete).unwrap();
        apply(&mut lifecycle, Transition::BeginResume).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::ResumeFailed),
            Ok(State::Failed)
        );
        apply(&mut lifecycle, Transition::BeginReset).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::ResetFailed),
            Ok(State::Failed)
        );
    }

    #[test]
    fn reset_can_recover_failed_state_and_invalid_reset_completion_is_rejected() {
        let mut lifecycle = Lifecycle::new();
        assert_eq!(
            apply(&mut lifecycle, Transition::ResetComplete),
            Err(Error::InvalidTransition)
        );
        apply(&mut lifecycle, Transition::BeginSuspend).unwrap();
        apply(&mut lifecycle, Transition::SuspendFailed).unwrap();
        apply(&mut lifecycle, Transition::BeginReset).unwrap();
        assert_eq!(
            apply(&mut lifecycle, Transition::ResetComplete),
            Ok(State::Active)
        );
    }

    #[test]
    fn earlier_transition_request_is_stale_after_state_changes() {
        let mut lifecycle = Lifecycle::new();
        let old = lifecycle.request(Transition::EnterIdle);
        lifecycle
            .apply(lifecycle.request(Transition::BeginSuspend))
            .unwrap();
        assert_eq!(lifecycle.apply(old), Err(Error::StaleGeneration));
        assert_eq!(lifecycle.state(), State::Suspending);
    }
}
