//! Real IPC sequencing. No service state is inferred from another process's RAM.
use crate::{
    boot::Bootstrap,
    wire::{self, StopRequest, StopStatus},
};
use nieuw_os_native_input_service::{discard_caps, Ipc};
use nieuw_os_sdk_runtime::{Outgoing, Received, SendEndpoint};
pub const BOOT_TICKS: u64 = 5_000;
pub const CMP_TICKS: u64 = 500;
pub const STOP_TICKS: u64 = 1_000;
pub const REPLY_TICKS: u64 = 200;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Phase {
    Boot = 0,
    Focus = 1,
    Go = 2,
    Running = 3,
    Stop = 4,
    Retire = 5,
    Reply = 6,
    Stopped = 7,
    Failed = 8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Fatal {
    Clock = 1,
    Deadline = 2,
    Send = 3,
    Receive = 4,
    Capabilities = 5,
    Protocol = 6,
    Exhausted = 7,
    Reported = 8,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub source_ready: u64,
    pub policy_ready: u64,
    pub compositor_ready: u64,
    pub app_startup: u64,
    pub focus_ack: u64,
    pub go_sent: u64,
    pub stop_sent: u64,
    pub refused: u64,
    pub retire_ack: u64,
    pub controller_replies: u64,
    pub rejected: u64,
    pub closed_caps: u64,
    pub app_done: u64,
    pub registry_receives: u64,
    pub ready_tick: u64,
}
struct Pending {
    message: Outgoing,
    target: SendEndpoint,
    sent: bool,
}
pub struct Supervisor {
    config: Bootstrap,
    phase: Phase,
    started: u64,
    last_tick: u64,
    deadline: u64,
    roles: u8,
    app_ready: bool,
    pending: Option<Pending>,
    active: Option<StopRequest>,
    last_control: u64,
    reply_status: StopStatus,
    fatal: Option<Fatal>,
    counts: Counts,
}
impl Supervisor {
    pub fn new(config: Bootstrap, now: u64) -> Result<Self, Fatal> {
        Ok(Self {
            config,
            phase: Phase::Boot,
            started: now,
            last_tick: now,
            deadline: now.checked_add(BOOT_TICKS).ok_or(Fatal::Clock)?,
            roles: 0,
            app_ready: false,
            pending: None,
            active: None,
            last_control: 0,
            reply_status: StopStatus::Failed,
            fatal: None,
            counts: Counts::default(),
        })
    }
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    pub const fn counts(&self) -> Counts {
        self.counts
    }
    pub const fn fatal(&self) -> Option<Fatal> {
        self.fatal
    }
    pub const fn start_tick(&self) -> u64 {
        self.started
    }
    fn transition(
        &mut self,
        phase: Phase,
        now: u64,
        ticks: u64,
        target: SendEndpoint,
        message: Outgoing,
    ) -> Result<(), Fatal> {
        self.deadline = now.checked_add(ticks).ok_or(Fatal::Clock)?;
        self.phase = phase;
        self.pending = Some(Pending {
            message,
            target,
            sent: false,
        });
        Ok(())
    }
    fn focus(&mut self, now: u64, retire: bool) -> Result<(), Fatal> {
        let message = wire::compositor(&self.config, if retire { 2 } else { 1 }, retire)
            .map_err(|_| Fatal::Protocol)?;
        self.transition(
            if retire { Phase::Retire } else { Phase::Focus },
            now,
            CMP_TICKS,
            self.config.compositor_control,
            message,
        )
    }
    fn app(&mut self, now: u64, stop: bool) -> Result<(), Fatal> {
        let message =
            wire::app(if stop { wire::STOP } else { wire::GO }).map_err(|_| Fatal::Protocol)?;
        self.transition(
            if stop { Phase::Stop } else { Phase::Go },
            now,
            if stop { STOP_TICKS } else { REPLY_TICKS },
            self.config.app_control,
            message,
        )
    }
    fn reply(&mut self, now: u64, status: StopStatus) -> Result<(), Fatal> {
        let request = self.active.ok_or(Fatal::Protocol)?;
        let controller = self.config.controller.ok_or(Fatal::Protocol)?;
        self.reply_status = status;
        self.transition(
            Phase::Reply,
            now,
            REPLY_TICKS,
            controller.reply,
            request.reply(status).map_err(|_| Fatal::Protocol)?,
        )
    }
    fn reject(&mut self) {
        self.counts.rejected = self.counts.rejected.saturating_add(1);
    }
    fn close_caps(&mut self, io: &mut impl Ipc, m: &Received) -> Result<bool, Fatal> {
        if m.cap_count() == 0 {
            return Ok(false);
        }
        let count = m.cap_count() as u64;
        discard_caps(io, m).map_err(|_| Fatal::Capabilities)?;
        self.counts.closed_caps = self.counts.closed_caps.saturating_add(count);
        self.reject();
        Ok(true)
    }
    fn controller(&mut self, now: u64, io: &mut impl Ipc) -> Result<(), Fatal> {
        if self.phase != Phase::Running {
            return Ok(());
        }
        let Some(controller) = self.config.controller else {
            return Ok(());
        };
        let message = match io.receive(controller.receive) {
            Ok(m) => m,
            Err(e) if e.is_again() => return Ok(()),
            Err(_) => return Err(Fatal::Receive),
        };
        if self.close_caps(io, &message)? {
            return Ok(());
        }
        let request = match StopRequest::decode(&self.config, &message) {
            Ok(r) => r,
            Err(_) => {
                self.reject();
                return Ok(());
            }
        };
        if request.id <= self.last_control {
            self.reject();
            return Ok(());
        }
        if request.id == u64::MAX {
            return Err(Fatal::Exhausted);
        }
        self.last_control = request.id;
        self.active = Some(request);
        self.app(now, true)
    }
    fn registry(&mut self, now: u64, io: &mut impl Ipc) -> Result<(), Fatal> {
        for _ in 0..4 {
            let message = match io.receive(self.config.registry) {
                Ok(m) => m,
                Err(e) if e.is_again() => break,
                Err(_) => return Err(Fatal::Receive),
            };
            self.counts.registry_receives = self.counts.registry_receives.saturating_add(1);
            if self.close_caps(io, &message)? {
                continue;
            }
            if self.phase == Phase::Boot {
                if let Ok(role) = wire::ready(&self.config, &message) {
                    if self.roles & role != 0 {
                        self.reject();
                        continue;
                    }
                    self.roles |= role;
                    match role {
                        1 => self.counts.source_ready = 1,
                        2 => self.counts.policy_ready = 1,
                        4 => self.counts.compositor_ready = 1,
                        _ => return Err(Fatal::Protocol),
                    };
                    continue;
                }
                if wire::system(&message, self.config.app_pid) == Ok(wire::READY) && !self.app_ready
                {
                    self.app_ready = true;
                    self.counts.app_startup = 1;
                    continue;
                }
                self.reject();
                continue;
            }
            let sent = self.pending.as_ref().is_some_and(|p| p.sent);
            match self.phase {
                Phase::Focus | Phase::Retire
                    if sent
                        && message.sender() == self.config.compositor_pid
                        && message.tag() == wire::CMP =>
                {
                    let retire = self.phase == Phase::Retire;
                    wire::compositor_reply(
                        &self.config,
                        &message,
                        if retire { 2 } else { 1 },
                        retire,
                    )
                    .map_err(|_| Fatal::Protocol)?;
                    self.pending = None;
                    if retire {
                        self.counts.retire_ack = self.counts.retire_ack.saturating_add(1);
                        self.reply(now, StopStatus::Stopped)?;
                    } else {
                        self.counts.focus_ack = 1;
                        self.app(now, false)?;
                    }
                }
                Phase::Stop
                    if sent
                        && message.sender() == self.config.app_pid
                        && message.tag() == wire::SYSTEM =>
                {
                    match wire::system(&message, self.config.app_pid)
                        .map_err(|_| Fatal::Protocol)?
                    {
                        wire::DONE => {
                            self.counts.app_done = self.counts.app_done.saturating_add(1);
                            self.pending = None;
                            self.focus(now, true)?;
                        }
                        wire::STOP_REFUSED => {
                            self.counts.refused = self.counts.refused.saturating_add(1);
                            self.pending = None;
                            self.reply(now, StopStatus::Refused)?;
                        }
                        _ => return Err(Fatal::Protocol),
                    }
                }
                _ => self.reject(),
            }
        }
        if self.phase == Phase::Boot && self.roles == 7 && self.app_ready {
            self.focus(now, false)?;
        }
        Ok(())
    }
    fn publish(&mut self, now: u64, io: &mut impl Ipc) -> Result<(), Fatal> {
        let Some(pending) = &mut self.pending else {
            return Ok(());
        };
        if pending.sent {
            return Ok(());
        }
        match io.send(pending.target, &pending.message) {
            Ok(()) => pending.sent = true,
            Err(e) if e.is_again() => return Ok(()),
            Err(_) => return Err(Fatal::Send),
        }
        match self.phase {
            Phase::Go => {
                self.counts.go_sent = 1;
                self.counts.ready_tick = now;
                self.pending = None;
                self.phase = Phase::Running;
            }
            Phase::Stop => self.counts.stop_sent = self.counts.stop_sent.saturating_add(1),
            Phase::Reply => {
                self.counts.controller_replies = self.counts.controller_replies.saturating_add(1);
                self.pending = None;
                self.active = None;
                self.phase = match self.reply_status {
                    StopStatus::Stopped => Phase::Stopped,
                    StopStatus::Refused => Phase::Running,
                    StopStatus::Failed => Phase::Failed,
                };
            }
            _ => {}
        }
        Ok(())
    }
    fn advance(&mut self, now: u64, io: &mut impl Ipc) -> Result<(), Fatal> {
        if now < self.last_tick {
            return Err(Fatal::Clock);
        }
        self.last_tick = now;
        if matches!(self.phase, Phase::Failed | Phase::Stopped) {
            return Ok(());
        }
        if self.phase != Phase::Running && now >= self.deadline {
            return Err(Fatal::Deadline);
        }
        self.controller(now, io)?;
        self.registry(now, io)?;
        self.publish(now, io)
    }
    pub fn step(&mut self, now: u64, io: &mut impl Ipc) -> Result<Phase, Fatal> {
        if let Err(error) = self.advance(now, io) {
            self.fatal = Some(error);
            if error != Fatal::Clock && self.active.is_some() && self.phase != Phase::Reply {
                if let Err(reply_error) = self.reply(now, StopStatus::Failed) {
                    self.phase = Phase::Failed;
                    return Err(reply_error);
                }
            } else {
                self.pending = None;
                self.phase = Phase::Failed;
                return Err(error);
            }
        }
        Ok(self.phase)
    }
}
