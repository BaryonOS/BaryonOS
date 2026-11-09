//! A01 native acceptance for an ordinary SDK-built app and a separate userspace service.
//! The kernel grants only endpoint rights and verifies lifecycle/sandbox evidence.
use super::*;
use capability_core::{RECV, SEND};

const PROCESS_COUNT: usize = 2;
const TIMEOUT_TICKS: u64 = 1200;

#[derive(Clone, Copy)]
struct Evidence {
    app_exited: bool,
    app_code: u64,
    docs: u64,
    bytes: u64,
    app_counter: u64,
    service_counter: u64,
}
static mut EVIDENCE: Evidence = Evidence {
    app_exited: false,
    app_code: 0,
    docs: 0,
    bytes: 0,
    app_counter: 0,
    service_counter: 0,
};

fn evidence() -> &'static mut Evidence {
    // SAFETY: One CPU, IF=0; serialized kernel service-stack access.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"SDKAPP:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn put(state: &State, slot: usize, offset: u64, value: u64) {
    write_word(state.processes[slot].physical(DATA + offset), value);
}
fn word(state: &State, slot: usize, offset: u64) -> u64 {
    read_word(state.processes[slot].physical(DATA + offset))
}
fn releasable_frames(state: &State) -> usize {
    let mut total = 0usize;
    for process in &state.processes[..state.amount] {
        if process.status == Status::Dead {
            continue;
        }
        let Some(paging) = process.paging.as_ref() else {
            fail(b"MISSING_ADDRESS_SPACE");
        };
        total = total
            .checked_add(process.count)
            .and_then(|value| value.checked_add(process.kernel_frames.len()))
            .and_then(|value| value.checked_add(paging.owned_table_count()))
            .unwrap_or_else(|| fail(b"FRAME_COUNT_OVERFLOW"));
    }
    total
}

pub fn manifest(state: &mut State) {
    if state.amount != PROCESS_COUNT || state.fault || state.exited || state.unexpected {
        fail(b"MANIFEST_STATE");
    }
    let service = state.processes[0].pid;
    let app = state.processes[1].pid;
    let engine = m4::engine();
    engine
        .register(service)
        .unwrap_or_else(|_| fail(b"REGISTER_SERVICE"));
    engine
        .register(app)
        .unwrap_or_else(|_| fail(b"REGISTER_APP"));

    let request_root = engine
        .create_endpoint(service)
        .unwrap_or_else(|_| fail(b"REQUEST_ENDPOINT"));
    let request_recv = engine
        .derive(service, request_root, RECV)
        .unwrap_or_else(|_| fail(b"REQUEST_RECV"));
    let request_send = if cfg!(feature = "test-sdk-app-denied") {
        0
    } else {
        engine
            .bootstrap_delegate(service, request_root, app, SEND)
            .unwrap_or_else(|_| fail(b"REQUEST_SEND"))
    };
    engine
        .close(service, request_root)
        .unwrap_or_else(|_| fail(b"REQUEST_ROOT_CLOSE"));

    let reply_root = engine
        .create_endpoint(app)
        .unwrap_or_else(|_| fail(b"REPLY_ENDPOINT"));
    let reply_send = engine
        .bootstrap_delegate(app, reply_root, service, SEND)
        .unwrap_or_else(|_| fail(b"REPLY_SEND"));
    // Receiving is restricted to the endpoint owner at the delegation boundary.
    // Checking service-side RECV on the handle alone would only test ownership.
    if engine.bootstrap_delegate(app, reply_root, service, RECV)
        != Err(capability_core::Error::Access)
    {
        fail(b"REPLY_RECV_TRANSFER");
    }
    let reply_recv = if cfg!(feature = "test-sdk-app-denied") {
        0
    } else {
        engine
            .derive(app, reply_root, RECV)
            .unwrap_or_else(|_| fail(b"REPLY_RECV"))
    };
    engine
        .close(app, reply_root)
        .unwrap_or_else(|_| fail(b"REPLY_ROOT_CLOSE"));

    for slot in 0..2 {
        put(state, slot, 32, 1);
        put(state, slot, 40, 0);
        put(state, slot, 240, 0);
        put(state, slot, 248, 0);
    }
    put(state, 0, 0, 0);
    put(state, 0, 56, request_recv);
    put(state, 0, 64, reply_send);
    put(state, 0, 176, app);
    put(state, 1, 0, 1);
    put(state, 1, 56, reply_recv);
    put(state, 1, 64, request_send);
    put(state, 1, 176, service);

    if !cfg!(feature = "test-sdk-app-denied") {
        if engine.check_endpoint(app, request_send, SEND).is_err()
            || engine.check_endpoint(app, reply_recv, RECV).is_err()
            || engine.check_endpoint(service, request_recv, RECV).is_err()
            || engine.check_endpoint(service, reply_send, SEND).is_err()
        {
            fail(b"RIGHTS_MISSING");
        }
        if engine.check_endpoint(app, request_send, RECV).is_ok()
            || engine.check_endpoint(app, reply_recv, SEND).is_ok()
            || engine.check_endpoint(service, request_recv, SEND).is_ok()
            || engine
                .check_block(app, request_send, capability_core::BLOCK_READ)
                .is_ok()
            || engine
                .check_block(app, reply_recv, capability_core::BLOCK_READ)
                .is_ok()
        {
            fail(b"RIGHTS_WIDENED");
        }
        if engine.stats().handles != 4 {
            fail(b"HANDLE_COUNT");
        }
    } else if request_send != 0 || reply_recv != 0 || engine.stats().handles != 2 {
        // Only the service owns its receive/send handles; the app owns none.
        fail(b"DENIED_HANDLE_COUNT");
    }
    if !engine.check_invariants() || engine.stats().queued_messages != 0 {
        fail(b"MANIFEST_ENGINE");
    }
    state.free_baseline = state
        .free_count()
        .checked_add(releasable_frames(state))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    *evidence() = Evidence {
        app_exited: false,
        app_code: 0,
        docs: 0,
        bytes: 0,
        app_counter: 0,
        service_counter: 0,
    };
    serial::write(b"SDKAPP:MANIFEST:PASS\n");
}

pub fn observe_exit(state: &mut State, slot: usize, counter: u64) {
    if slot != 1 || evidence().app_exited {
        fail(b"UNEXPECTED_EXIT");
    }
    let expected = if cfg!(feature = "test-sdk-app-denied") {
        1
    } else {
        0
    };
    if state.last_event.code != expected {
        fail(b"EXIT_CODE");
    }
    let e = evidence();
    e.app_exited = true;
    e.app_code = state.last_event.code;
    e.docs = word(state, slot, 240);
    e.bytes = word(state, slot, 248);
    e.app_counter = counter;
    e.service_counter = word(state, 0, 8);
}

pub fn observe_fault(_state: &mut State, _slot: usize, _context: &[u64; 22], _counter: u64) {
    fail(b"UNEXPECTED_FAULT");
}

pub fn poll(state: &mut State) -> Option<*const u64> {
    if state.ticks >= TIMEOUT_TICKS || state.fault || state.unexpected {
        fail(b"TIMEOUT_OR_FAULT");
    }
    let e = *evidence();
    if !e.app_exited {
        // The m3 event/select loop alone schedules; idle observers must not recurse.
        return None;
    }
    if state.processes[1].status != Status::Dead || state.processes[0].status == Status::Dead {
        fail(b"LIFECYCLE");
    }
    if word(state, 0, 8) <= e.service_counter + 4 {
        // The m3 event/select loop alone schedules; idle observers must not recurse.
        return None;
    }

    if cfg!(feature = "test-sdk-app-denied") {
        if e.app_code != 1
            || e.docs != 0
            || e.bytes != 0
            || word(state, 0, 40) != 0
            || word(state, 0, 240) != 0
            || word(state, 0, 248) != 0
        {
            fail(b"DENIED_RESULT");
        }
        serial::write(b"SDKAPP:DENIED:PASS\nSDKAPP:ERROR_EXIT:PASS\n");
    } else {
        if e.app_code != 0
            || e.docs != 1
            || e.bytes != 12
            || word(state, 0, 40) != 1
            || word(state, 0, 240) != 2
            || word(state, 0, 248) != 2
        {
            fail(b"NORMAL_RESULT");
        }
        serial::write(b"SDKAPP:READ:PASS\nSDKAPP:SANDBOX:PASS\nSDKAPP:EXIT:PASS\n");
    }

    // The app has already completed its exit cleanup. Keep the established
    // address-space/W^X audit for the still-live service before releasing it.
    state.audit_process(0);
    let released = releasable_frames(state);
    if state.free_count().checked_add(released) != Some(state.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    state.processes[0].status = Status::Dying;
    state.cleanup(0);
    let stats = m4::engine().stats();
    if state.free_count() != state.free_baseline
        || stats.processes != 0
        || stats.handles != 0
        || stats.nodes != 0
        || stats.endpoints != 0
        || stats.queued_messages != 0
    {
        fail(b"CLEANUP");
    }
    print_number(b"SDKAPP:APP_COUNTER:", e.app_counter);
    print_number(b"SDKAPP:SERVICE_COUNTER:", e.service_counter);
    serial::write(b"SDKAPP:CLEANUP:PASS\nSDKAPP:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
