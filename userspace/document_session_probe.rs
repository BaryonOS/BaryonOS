#![forbid(unsafe_code)]
//! Opt-in headless native document acceptance, using the actual SDK transport.
//! The existing M6c kernel observer still checks every request, ACK, WRITE and
//! FLUSH. This module emits no success markers and creates no capabilities.

use crate::rt;
use nieuw_os_catalog_client::{NativeTransport, Session, Transport};
use nieuw_os_document_session::{DocumentSession, EditMode, Error, SaveOutcome};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};

/// Only acceptance instrumentation: each underlying real native call increments
/// the existing private word-8 counter. Results and actual envelopes are passed
/// through unchanged. Service-side fault gates remain in sdk_probe_runtime.
struct CountedNativeTransport(NativeTransport);

fn count_call() {
    rt::put(8, rt::word(8).wrapping_add(1));
}

impl Transport for CountedNativeTransport {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        count_call();
        self.0.send(endpoint, message)
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        count_call();
        self.0.receive(endpoint)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        count_call();
        self.0.yield_now()
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        count_call();
        self.0.close(handle)
    }
}

type Editor = DocumentSession<CountedNativeTransport>;

fn take_endpoints(domain: u64, mode: EditMode) -> Editor {
    // The preceding raw request has fully completed. From this point until
    // close, only this Session owns and uses these already granted endpoints.
    // Constructing typed handles grants no authority; kernel/service checks
    // remain identical to the earlier raw acceptance steps.
    if domain >= 2 || rt::word(0) != domain + 1 {
        rt::fail(120);
    }
    let send = Handle::from_raw(rt::word(64)).unwrap_or_else(|_| rt::fail(121));
    let receive = Handle::from_raw(rt::word(56)).unwrap_or_else(|_| rt::fail(122));
    let catalog = Session::new(
        CountedNativeTransport(NativeTransport),
        rt::word(32),
        rt::word(176),
        domain as usize,
        SendEndpoint::from_handle(send),
        ReceiveEndpoint::from_handle(receive),
        rt::LIMIT,
    )
    .unwrap_or_else(|_| rt::fail(123));
    DocumentSession::new(catalog, mode)
}

fn check_document(editor: &Editor, object: u64, revision: u64, data_revision: u64) {
    let draft = editor.draft().unwrap_or_else(|| rt::fail(124));
    if draft.object() != object
        || draft.loaded_revision() != revision
        || draft.contents() != crate::value(object, data_revision)
        || draft.name() != crate::name(object, revision)
        || editor.is_dirty()
        || editor.is_conflicted()
        || editor.is_uncertain()
        || editor.is_closed()
    {
        rt::fail(125);
    }
}

fn check_catalog(editor: &mut Editor, object: u64) {
    let catalog = editor.list().unwrap_or_else(|_| rt::fail(126));
    let entry = catalog.entries[0];
    if catalog.count != 1
        || catalog.generation != 5
        || entry.object != object
        || entry.revision != 3
        || entry.length != 128
        || &entry.name[..entry.name_length] != crate::name(object, 3)
    {
        rt::fail(127);
    }
}

fn finish(mut editor: Editor, object: u64) -> ! {
    editor.close().unwrap_or_else(|_| rt::fail(128));
    if !editor.is_closed() {
        rt::fail(129);
    }
    // Session has closed send and receive once. Keep the original real stale-
    // send denial probe, but do not call the old finish's second send-close.
    let message = crate::request(999, object, 0, crate::wire::GET, None, None);
    rt::expect(rt::send_once(rt::word(64), &message), -4);
    rt::put(40, 1);
    rt::worker()
}

pub fn complete_normal(domain: u64, object: u64) -> ! {
    let mut editor = take_endpoints(domain, EditMode::ReadWrite);
    // This GET replaces the earlier raw GET9; the intervening raw operations
    // affected the other object only. Its payload is revision 1 after RENAME2.
    editor.open(object).unwrap_or_else(|_| rt::fail(130));
    check_document(&editor, object, 2, 1);
    editor
        .replace_contents(&crate::value(object, 3))
        .unwrap_or_else(|_| rt::fail(131));
    if !editor.is_dirty() || editor.draft().unwrap().loaded_revision() != 2 {
        rt::fail(132);
    }
    if editor.save() != Ok(SaveOutcome::Saved { revision: 3 }) {
        rt::fail(133);
    }
    check_document(&editor, object, 3, 3);
    editor.reopen().unwrap_or_else(|_| rt::fail(134));
    check_document(&editor, object, 3, 3);
    check_catalog(&mut editor, object);
    check_document(&editor, object, 3, 3);
    finish(editor, object)
}

pub fn recover(domain: u64, object: u64) -> ! {
    // ReadOnly is local app policy, not a replacement for endpoint authority.
    let mut editor = take_endpoints(domain, EditMode::ReadOnly);
    editor.open(object).unwrap_or_else(|_| rt::fail(135));
    check_document(&editor, object, 3, 3);
    if editor.open(object + 1)
        != Err(Error::NotFound {
            object: object + 1,
            revision: 2,
        })
    {
        rt::fail(136);
    }
    check_document(&editor, object, 3, 3);
    check_catalog(&mut editor, object);
    check_document(&editor, object, 3, 3);
    finish(editor, object)
}
