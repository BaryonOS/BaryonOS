use super::{Error, Phase, ProbeTiming, Service};
use crate::time;

/// Native integration supplies an already-authorized entropy session. There
/// is deliberately no built-in RNG, clock seed, constant or network fallback.
pub trait RandomSource {
    type Error;
    fn random_u32(&mut self) -> Result<u32, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriveError<E> {
    Service(Error),
    Random(E),
    RandomRejected,
}

impl<E> From<Error> for DriveError<E> {
    fn from(error: Error) -> Self {
        Self::Service(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveOutcome {
    LinkDown,
    WaitingUntil(u64),
    Backpressure,
    Published(u64),
    Advanced(Phase),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Schedule {
    phase: Phase,
    /// XID whose retry deadline `due`/`backoff` describe. A new transaction
    /// (fresh XID from start or begin_renewal) never inherits the previous
    /// transaction's deadline, even when it lands in the same phase.
    transaction: u32,
    due: u64,
    backoff: u64,
    starting: bool,
}

fn uniform<R: RandomSource>(random: &mut R, count: u32) -> Result<u32, DriveError<R::Error>> {
    let threshold = count.wrapping_neg() % count;
    // Bounded rejection prevents a broken source from blocking the service
    // loop. A rejected sample never becomes a success-shaped default.
    for _ in 0..8 {
        let value = random.random_u32().map_err(DriveError::Random)?;
        if value >= threshold {
            return Ok(value % count);
        }
    }
    Err(DriveError::RandomRejected)
}

fn fresh_xid<R: RandomSource>(random: &mut R, previous: u32) -> Result<u32, DriveError<R::Error>> {
    for _ in 0..8 {
        let xid = random.random_u32().map_err(DriveError::Random)?;
        if xid != previous {
            return Ok(xid);
        }
    }
    Err(DriveError::RandomRejected)
}

impl Service<'_> {
    /// Actual link-up/resume notification from the trusted owner. This is not
    /// an authority grant or device-reset proof; retained TX still blocks it.
    pub fn automatic_link_up(&mut self, ticks: u64) -> Result<(), Error> {
        self.link_down(ticks)?;
        self.automatic = Some(Schedule {
            phase: Phase::Idle,
            transaction: self.transaction_id,
            due: ticks,
            backoff: 400,
            starting: false,
        });
        Ok(())
    }

    /// One bounded service-loop step. Call receive_frame before this step for
    /// ingress observed at this timestamp; real completions remain separate.
    /// The low-level start/send APIs must not be interleaved by another owner.
    pub fn drive<R: RandomSource>(
        &mut self,
        ticks: u64,
        random: &mut R,
    ) -> Result<DriveOutcome, DriveError<R::Error>> {
        self.now(ticks)?;
        let Some(mut schedule) = self.automatic else {
            return Ok(DriveOutcome::LinkDown);
        };
        let mut phase = self.phase(ticks)?;
        if matches!(
            phase,
            Phase::Idle | Phase::Expired | Phase::TimedOut | Phase::Invalidated
        ) {
            // Finish pending DECLINE before starting a new transaction.
            if self.declining.is_some() {
                self.poll(ticks)?;
                return Ok(DriveOutcome::Backpressure);
            }
            if ticks < self.retry_after {
                return Ok(DriveOutcome::WaitingUntil(self.retry_after));
            }
            if !self.stack.can_queue_control_frame() {
                return Ok(DriveOutcome::Backpressure);
            }
            if !schedule.starting || schedule.phase != phase {
                let delay = 100 + u64::from(uniform(random, 901)?);
                schedule.due = self.deadline(ticks, delay)?;
                schedule.phase = phase;
                schedule.starting = true;
                self.automatic = Some(schedule);
            }
            if ticks < schedule.due {
                return Ok(DriveOutcome::WaitingUntil(schedule.due));
            }
            let xid = fresh_xid(random, self.transaction_id)?;
            let timing = ProbeTiming::new(
                u64::from(uniform(random, 101)?),
                100 + u64::from(uniform(random, 101)?),
                100 + u64::from(uniform(random, 101)?),
            )?;
            self.start(xid, ticks, 120)?;
            self.timing = timing;
            phase = Phase::Selecting;
            schedule.starting = false;
        } else if phase == Phase::RenewalDue {
            if !self.stack.can_queue_control_frame() {
                return Ok(DriveOutcome::Backpressure);
            }
            let xid = fresh_xid(random, self.transaction_id)?;
            self.begin_renewal(xid, ticks)?;
            phase = self.phase(ticks)?;
        }
        if matches!(
            phase,
            Phase::Selecting | Phase::Requesting | Phase::Renewing | Phase::Rebinding
        ) {
            if schedule.phase != phase || schedule.transaction != self.transaction_id {
                schedule.phase = phase;
                schedule.transaction = self.transaction_id;
                schedule.due = ticks;
                schedule.backoff = 400;
            }
            self.automatic = Some(schedule);
            if ticks < schedule.due {
                self.poll(ticks)?;
                return Ok(DriveOutcome::WaitingUntil(schedule.due));
            }
            if !self.stack.can_queue_control_frame() {
                return Ok(DriveOutcome::Backpressure);
            }
            let delay = if matches!(phase, Phase::Renewing | Phase::Rebinding) {
                let seconds = self.now(ticks)?;
                let lease = self
                    .client
                    .lease(seconds)
                    .map_err(Error::from)?
                    .ok_or(Error::from(nieuw_os_net_stack::Error::InvalidState))?;
                let boundary = time::ticks(if phase == Phase::Renewing {
                    lease.rebind_at
                } else {
                    lease.expires_at
                })
                .map_err(|clock| Error::ClockFault {
                    clock,
                    withdrawal: None,
                })?;
                let remaining = boundary
                    .checked_sub(ticks)
                    .ok_or(Error::from(nieuw_os_net_stack::Error::InvalidState))?;
                let half = (remaining / 2).max(6000);
                (half + u64::from(uniform(random, 101)?)).min(remaining)
            } else {
                let low = schedule.backoff - 100;
                let high = (schedule.backoff + 100).min(6400);
                low + u64::from(uniform(random, (high - low + 1) as u32)?)
            };
            let next = self.deadline(ticks, delay)?;
            let id = self.send_dhcp(ticks)?;
            schedule.due = next;
            schedule.backoff = (schedule.backoff * 2).min(6400);
            self.automatic = Some(schedule);
            return Ok(DriveOutcome::Published(id));
        }
        schedule.phase = phase;
        schedule.starting = false;
        self.automatic = Some(schedule);
        self.poll(ticks)?;
        Ok(DriveOutcome::Advanced(self.phase(ticks)?))
    }
}
