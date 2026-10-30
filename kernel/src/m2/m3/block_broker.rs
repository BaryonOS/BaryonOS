//! Common mappings for the scoped block brokers.
use super::{paging, user, Owner, State, PAGE};
use capability_core::{BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE};
use virtio_core::Operation;

pub(super) fn err(e: virtio_core::Error) -> i64 {
    use virtio_core::Error::*;
    match e {
        Busy | Pending => -8,
        Range | Invalid | WrongTicket | StaleEpoch => -1,
        Protocol => -15,
        Io | Unsupported => -12,
        Timeout => -13,
        Offline | Exhausted => -14,
    }
}

pub(super) fn operation_index(op: Operation) -> usize {
    match op {
        Operation::Read { .. } => 0,
        Operation::Write { .. } => 1,
        Operation::Flush => 2,
    }
}

pub(super) fn required(op: Operation) -> u64 {
    match op {
        Operation::Read { .. } => BLOCK_READ,
        Operation::Write { .. } => BLOCK_WRITE,
        Operation::Flush => BLOCK_FLUSH,
    }
}

pub(super) fn input_plan(s: &State, slot: usize, pointer: u64) -> Result<user::CopyPlan, i64> {
    let p = &s.processes[slot];
    user::sector_input_plan(pointer, p.pid, |va| {
        let (physical, writable, _, user) = p.paging.as_ref()?.permissions(va)?;
        let page = physical & !(PAGE - 1);
        let Owner::User(owner, _) = s.frames.owner(page).ok()? else {
            return None;
        };
        Some(user::PageAccess {
            physical: page,
            owner,
            user,
            writable,
        })
    })
    .map_err(|_| -3)
}

pub(super) fn input_bytes(plan: &user::CopyPlan) -> [u8; 512] {
    let mut bytes = [0; 512];
    let mut at = 0;
    for c in &plan.chunks[..plan.count] {
        paging::physical_read(c.physical, |source| {
            for i in 0..c.length {
                // SAFETY: Whole plan was validated caller-owned/effectively USER;
                // RO scratch reads exact bytes, without a writable text alias.
                bytes[at + i] = unsafe { source.add(c.offset + i).read_volatile() };
            }
        });
        at += c.length;
    }
    assert_eq!(at, 512);
    bytes
}

// Generic block broker shared by the volume and update profiles. The hooks
// retain each profile's accounting, crash fixture and poll/reset policy.
macro_rules! scoped_block_broker {
    ($poll_before_wait:expr, $write_enabled:expr, $write_hook:ident) => {
        fn device_call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
            if !matches!(n, 0x40..=0x43) {
                return Err(-2);
            }
            if a[4] != 0 || a[5] != 0 || n == 0x43 && a[1..4].iter().any(|v| *v != 0) {
                return Err(-1);
            }
            if n == 0x41 && a[3] != 320 || n == 0x42 && a[3] != 512 {
                return Err(-11);
            }
            let pid = s.processes[slot].pid;
            let rights = match n {
                0x40 => BLOCK_READ,
                0x42 => BLOCK_WRITE,
                0x43 => BLOCK_FLUSH,
                _ => 0,
            };
            let auth = m4::engine()
                .check_block(pid, a[0], rights)
                .map_err(|e| e.code())?;
            if auth.scope().device != 0 {
                return Err(-5);
            }
            let binding = Binding {
                pid,
                object_key: auth.object_key(),
            };
            if n == 0x41 {
                // One non-blocking device poll before answering: a completed request
                // is reported now instead of after the next timer tick. The kernel
                // still never waits in a syscall; an unfinished request is EAGAIN.
                if $poll_before_wait {
                    advance(s);
                }
                let request = runtime()
                    .error
                    .map(|(q, _)| q)
                    .or_else(|| runtime().policy.request())
                    .ok_or(-1)?;
                m4::engine()
                    .check_block(pid, a[0], required(request.operation))
                    .map_err(|e| e.code())?;
                if request.binding != binding || request.ticket != a[1] {
                    return Err(-1);
                }
                let output = m4::plan(s, slot, a[2], 320, true).map_err(|e| e.code())?;
                let r = runtime();
                if let Some((_, code)) = r.error.take() {
                    if r.policy.peek(binding, a[1]).is_ok() {
                        let _ = r.policy.consume(binding, a[1]);
                    }
                    return Err(code);
                }
                let request = r.policy.peek(binding, a[1]).map_err(err)?;
                if request.completion == Some(Completion::Unknown) {
                    r.policy.consume(binding, a[1]).map_err(err)?;
                    return Err(-16);
                }
                if request.completion != Some(Completion::Success) {
                    return Err(-8);
                }
                let mut message = Message::new();
                message.tag = a[1];
                if matches!(request.operation, Operation::Read { .. }) {
                    message.len = u32::from(request.length);
                    let start = usize::from(request.offset);
                    let end = start + usize::from(request.length);
                    message.data[..usize::from(request.length)]
                        .copy_from_slice(&r.data[start..end]);
                }
                let bytes = message.encode();
                m4::write_plan(&output, &bytes);
                assert_eq!(m4::read_plan(&output), bytes);
                r.policy.consume(binding, a[1]).map_err(err)?;
                return Ok(u64::from(message.len));
            }
            let op = if n == 0x43 {
                Operation::Flush
            } else {
                if n == 0x40 && (a[3] == 0 || a[3] > 128) {
                    return Err(-11);
                }
                if n == 0x40 && a[2].checked_add(a[3]).filter(|v| *v <= 512).is_none() {
                    return Err(-1);
                }
                if a[1] >= auth.scope().sector_count {
                    return Err(-5);
                }
                let sector = auth.sector(a[1]).map_err(|e| e.code())?;
                if n == 0x40 {
                    Operation::Read {
                        sector,
                        offset: a[2],
                        length: a[3],
                    }
                } else {
                    Operation::Write { sector }
                }
            };
            let input = if n == 0x42 {
                Some(input_bytes(&input_plan(s, slot, a[2])?))
            } else {
                None
            };
            let r = runtime();
            if r.offline != 0 {
                return Err(r.offline);
            }
            if r.resetting || r.error.is_some() || r.policy.phase() != Phase::Ready {
                return Err(-8);
            }
            let capacity = r.device.as_ref().ok_or(-14)?.capacity;
            let request = r
                .policy
                .prepare(binding, op, capacity, s.ticks, TIMEOUT)
                .map_err(err)?;
            if let Err(error) =
                r.device
                    .as_mut()
                    .unwrap()
                    .prepare_operation(op, input.as_ref(), false)
            {
                r.policy.reject_prepared(binding, request.ticket).unwrap();
                return Err(err(error));
            }
            r.policy.publish(request.epoch, request.ticket).unwrap();
            r.device.as_mut().unwrap().publish_prepared(false).unwrap();
            r.publications[operation_index(op)] += 1;
            Ok(request.ticket)
        }

        fn reset(r: &mut Runtime, now: u64) {
            if r.quarantined {
                return;
            }
            r.policy.start_reset().unwrap();
            r.resetting = true;
            r.reset_deadline = now + RESET_TIMEOUT;
            if let Some(device) = r.device.as_mut() {
                device.begin_reset();
            }
        }

        pub fn detach(pid: u64) {
            let r = runtime();
            if r.error.is_some_and(|(q, _)| q.binding.pid == pid) {
                r.error = None;
            }
            if r.policy.exit(pid).is_some() && !r.quarantined {
                let now = r.now;
                reset(r, now);
            }
        }

        fn advance(s: &State) {
            let r = runtime();
            let now = s.ticks;
            r.now = now;
            if r.resetting {
                if r.device.as_ref().is_some_and(Device::reset_acknowledged) {
                    // SAFETY: Real reset acknowledgement permits reuse of the same
                    // pinned owned RAM pages and permanent validated supervisor MMIO.
                    match unsafe {
                        Device::initialize_profile(
                            r.info.unwrap(),
                            r.mapped,
                            r.pool,
                            $write_enabled,
                            false,
                        )
                    } {
                        Ok(device) => {
                            r.device = Some(device);
                            r.policy.reset_completed(true).unwrap();
                            r.resetting = false;
                        }
                        Err(_) => {
                            let _ = r.policy.reset_completed(false);
                            r.offline = -14;
                            r.resetting = false;
                            r.quarantined = true;
                        }
                    }
                } else if now >= r.reset_deadline {
                    let _ = r.policy.reset_completed(false);
                    r.resetting = false;
                    r.quarantined = true;
                    r.offline = -13;
                }
                return;
            }
            if r.offline != 0 || r.policy.phase() != Phase::Published {
                return;
            }
            let request = r.policy.request().unwrap();
            if r.policy.timeout(now) {
                let code = if request.operation.is_mutating() {
                    -16
                } else {
                    -13
                };
                r.error = Some((request, code));
                reset(r, now);
                return;
            }
            match r.device.as_mut().unwrap().poll(0) {
                Ok(Some((data, Completion::Success))) => {
                    r.data = data;
                    r.completions[operation_index(request.operation)] += 1;
                    r.policy
                        .complete(request.epoch, request.ticket, Completion::Success)
                        .unwrap();
                    if let Operation::Write { sector } = request.operation {
                        $write_hook(s, r, sector);
                    }
                }
                Ok(Some((_, _))) => {
                    r.policy
                        .complete(request.epoch, request.ticket, Completion::Io)
                        .unwrap();
                    let code = if request.operation.is_mutating() {
                        -16
                    } else {
                        -12
                    };
                    r.error = Some((request, code));
                    reset(r, now);
                }
                Ok(None) => {}
                Err(error) => {
                    let code = if request.operation.is_mutating() {
                        -16
                    } else {
                        err(error)
                    };
                    r.error = Some((request, code));
                    reset(r, now);
                }
            }
        }
    };
}
