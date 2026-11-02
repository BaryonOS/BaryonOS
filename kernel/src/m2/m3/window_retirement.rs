//! Bounded, host-testable authority and retirement mechanism for the fixed
//! unreleased window-group syscall. This knows no renderer, app state or IPC
//! protocol. The adapter supplies kernel-owned manifest identities only.
use capability_core::{Engine, Error, OBSERVE, PROCESS_RUNNING};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Restart,
    Retire,
}
#[derive(Clone, Copy)]
pub struct Authority {
    pub init: u64,
    pub launch: u64,
    pub image: u32,
    pub process: u64,
    pub compositor: u64,
    pub apps: [(u64, [u64; 4]); 2],
}

pub fn authorize(
    engine: &Engine,
    caller: u64,
    authority: &Authority,
    args: [u64; 6],
) -> Result<Phase, i64> {
    if caller != authority.init {
        return Err(-5);
    }
    let phase = match args[2] {
        0 => Phase::Restart,
        1 => Phase::Retire,
        _ => return Err(-1),
    };
    if args[3..].iter().any(|value| *value != 0) {
        return Err(-1);
    }
    if args[0] != authority.launch
        || engine.check_launch(caller, args[0]).map_err(|e| e.code())? != authority.image
    {
        return Err(-5);
    }
    let process = engine
        .check_process(caller, args[1], OBSERVE)
        .map_err(|e| e.code())?;
    if args[1] != authority.process || process.child != authority.compositor {
        return Err(-5);
    }
    if process.state == PROCESS_RUNNING {
        return Err(-9);
    }
    Ok(phase)
}

/// Call only after authorize and adapter checks of live app roles and a dead
/// current compositor slot. Handles come from kernel bookkeeping, never the
/// app's mutable startup page. No allocation or delegation occurs here.
pub fn retire(engine: &mut Engine, authority: &Authority) -> Result<(), i64> {
    for (pid, handles) in authority.apps {
        for handle in handles {
            match engine.close(pid, handle) {
                Ok(()) | Err(Error::BadHandle) => {}
                Err(error) => return Err(error.code()),
            }
        }
    }
    Ok(())
}
