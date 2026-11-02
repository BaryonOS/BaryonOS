//! One bounded, volatile text clipboard inside the existing compositor.
//! No app can read it without its own currently offered real paste gesture.
use nieuw_os_input_client::clipboard::{Header, Operation, Reply, Request, Status, Text};
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};

use crate::input::Queue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    None,
    Copied,
    Offered,
    Cleared,
    Empty,
    Denied,
}
impl Notice {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Copied => "Tekst gekopieerd",
            Self::Offered => "Plakken aangeboden",
            Self::Cleared => "Klembord leeg",
            Self::Empty => "Geen gekopieerde tekst",
            Self::Denied => "Klembordactie vervallen",
        }
    }
}

pub fn copy_gesture(key: KeyboardEvent) -> bool {
    key.key == PhysicalKey::C
        && key.action == KeyAction::Press
        && key.modifiers.control()
        && !key.modifiers.alt()
        && !key.modifiers.super_key()
        && !key.modifiers.shift()
}
pub fn clear_gesture(key: KeyboardEvent) -> bool {
    clear_shortcut_key(key) && key.action == KeyAction::Press
}
fn clear_shortcut_key(key: KeyboardEvent) -> bool {
    key.key == PhysicalKey::Delete
        && key.modifiers.control()
        && key.modifiers.shift()
        && !key.modifiers.alt()
        && !key.modifiers.super_key()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearAction {
    Pass,
    Clear,
    Consumed,
}
#[derive(Default)]
pub struct ClearChord {
    delete_held: bool,
}
impl ClearChord {
    /// Once a trusted clear Press was consumed, consume its whole physical
    /// Delete gesture even if modifiers/focus change before Repeat/Release.
    /// A source cancellation does not authorize an unmatched repeat either.
    pub fn key(&mut self, key: KeyboardEvent) -> ClearAction {
        if key.key == PhysicalKey::Delete && self.delete_held {
            if key.action == KeyAction::Release {
                self.delete_held = false;
            }
            return ClearAction::Consumed;
        }
        if clear_gesture(key) {
            self.delete_held = true;
            ClearAction::Clear
        } else if clear_shortcut_key(key) {
            ClearAction::Consumed
        } else {
            ClearAction::Pass
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Grant {
    index: usize,
    sender: u64,
    revision: u64,
}
/// Retained backpressure state contains correlation and authority only. In
/// particular, it never retains a prepared plaintext paste reply.
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub header: Header,
    status: Status,
    grant: Option<Grant>,
}

pub struct Broker {
    text: Text,
    revision: u64,
    used_through: [u64; 2],
    notice: Notice,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exhausted;
impl Default for Broker {
    fn default() -> Self {
        Self::new()
    }
}
impl Broker {
    pub const fn new() -> Self {
        Self {
            text: Text::empty(),
            revision: 1,
            used_through: [0; 2],
            notice: Notice::None,
        }
    }
    pub const fn notice(&self) -> Notice {
        self.notice
    }
    /// Focus loss, source cancellation, queue overflow and explicit clear
    /// revoke every earlier gesture. Text survives focus switching so copying
    /// in one app and pasting in the other remains possible.
    pub fn revoke(&mut self, queues: &[Queue; 2]) -> Result<(), Exhausted> {
        self.revision = self.revision.checked_add(1).ok_or(Exhausted)?;
        for (used, queue) in self.used_through.iter_mut().zip(queues) {
            *used = (*used).max(queue.sequence_watermark());
        }
        Ok(())
    }
    /// The trusted copy key clears the previous item before the application
    /// inspects selection/size. Failed or absent COPY never falls back to it.
    pub fn clear(&mut self, queues: &[Queue; 2]) -> Result<(), Exhausted> {
        self.text = Text::empty();
        self.notice = Notice::Cleared;
        self.revoke(queues)
    }
    pub fn accept(
        &mut self,
        index: usize,
        focused: usize,
        sender: u64,
        queue: &Queue,
        request: Request<'_>,
    ) -> Pending {
        let header = request.header;
        let denied = Pending {
            header,
            status: Status::Denied,
            grant: None,
        };
        if index >= 2 || focused != index || !queue.authorizes_clipboard(sender, header) {
            return denied;
        }
        if header.sequence <= self.used_through[index] {
            return Pending {
                status: Status::Used,
                ..denied
            };
        }
        // One use is claimed by the broker before any data is accepted or
        // reply is attempted. EAGAIN cannot turn an old offer into a new read.
        self.used_through[index] = header.sequence;
        let status = match header.operation {
            Operation::Copy => {
                self.text = Text::empty();
                match Text::try_from(request.text) {
                    Ok(text) if !text.is_empty() => {
                        self.text = text;
                        self.notice = Notice::Copied;
                        Status::Ok
                    }
                    Ok(_) => {
                        self.notice = Notice::Empty;
                        Status::Empty
                    }
                    Err(_) => {
                        self.notice = Notice::Empty;
                        Status::TooLong
                    }
                }
            }
            Operation::Paste => {
                if !request.text.is_empty() {
                    Status::InvalidText
                } else if self.text.is_empty() {
                    self.notice = Notice::Empty;
                    Status::Empty
                } else {
                    Status::Ok
                }
            }
        };
        Pending {
            header,
            status,
            grant: Some(Grant {
                index,
                sender,
                revision: self.revision,
            }),
        }
    }
    /// Called immediately before EACH send attempt, including after EAGAIN.
    /// Rebuild from current private bytes only while the exact gesture remains
    /// offered, focused and unrevoked. Revocation produces a bodyless denial.
    pub fn reply<'a>(&'a self, pending: &Pending, focused: usize, queue: &Queue) -> Reply<'a> {
        let status = match pending.grant {
            Some(grant)
                if grant.index == focused
                    && grant.revision == self.revision
                    && queue.authorizes_clipboard(grant.sender, pending.header) =>
            {
                pending.status
            }
            Some(_) => Status::Denied,
            None => pending.status,
        };
        Reply {
            header: pending.header,
            status,
            text: if status == Status::Ok && pending.header.operation == Operation::Paste {
                self.text.as_str()
            } else {
                ""
            },
        }
    }
    pub fn delivered(&mut self, operation: Operation, status: Status) {
        if status == Status::Denied {
            self.notice = Notice::Denied;
        } else if operation == Operation::Paste && status == Status::Ok {
            self.notice = Notice::Offered;
        }
    }
}
