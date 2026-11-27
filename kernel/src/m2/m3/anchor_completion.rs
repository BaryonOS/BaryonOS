// SPDX-License-Identifier: MPL-2.0

//! Private ticket lifetime for the S7 TEST profile. The caller supplies
//! authority resolved by the capability engine. No user pointer reaches here.
use capability_core::{Error, MAX_ANCHOR_DOMAINS};

pub(super) const LIFETIME_TICKS: u64 = 1_000;
type Digest = [u8; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Authority {
    pub pid: u64,
    pub handle: u64,
    pub domain: u8,
    pub object_key: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Read,
    Extend,
    Instance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Request {
    Read,
    Extend { expected: Digest, store: Digest },
    Instance,
}

impl Request {
    const fn operation(self) -> Operation {
        match self {
            Self::Read => Operation::Read,
            Self::Extend { .. } => Operation::Extend,
            Self::Instance => Operation::Instance,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Failure {
    Unavailable = 1,
    Corrupt = 2,
    Conflict = 3,
    RateLimited = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Completion {
    Value(Digest),
    Extended,
    Failed(Failure),
}

impl Completion {
    const fn status(self) -> u64 {
        match self {
            Self::Value(_) | Self::Extended => 0,
            Self::Failed(error) => error as u64,
        }
    }

    fn checked_for(self, operation: Operation) -> Self {
        match (operation, self) {
            (_, Self::Failed(_))
            | (Operation::Extend, Self::Extended)
            | (Operation::Read | Operation::Instance, Self::Value(_)) => self,
            _ => Self::Failed(Failure::Corrupt),
        }
    }
}

#[derive(Clone, Copy)]
struct Delivery {
    authority: Authority,
    ticket: u64,
    operation: Operation,
    expires_at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Work {
    pub authority: Authority,
    pub ticket: u64,
    pub request: Request,
}

#[derive(Clone, Copy)]
enum Entry {
    Empty,
    Pending(Delivery, Request),
    Running(Delivery),
    Draining(u64),
    Ready(Delivery, Completion),
}

pub(super) struct Mailboxes {
    entries: [Entry; MAX_ANCHOR_DOMAINS],
    last_ticket: u64,
}

impl Mailboxes {
    pub(super) const fn new() -> Self {
        Self {
            entries: [Entry::Empty; MAX_ANCHOR_DOMAINS],
            last_ticket: 0,
        }
    }

    pub(super) fn submit(
        &mut self,
        authority: Authority,
        request: Request,
        now: u64,
    ) -> Result<u64, Error> {
        let entry = self
            .entries
            .get_mut(usize::from(authority.domain))
            .ok_or(Error::Invalid)?;
        if !matches!(entry, Entry::Empty) {
            return Err(Error::Again);
        }
        let expires_at = now.checked_add(LIFETIME_TICKS).ok_or(Error::Quota)?;
        let ticket = self.last_ticket.checked_add(1).ok_or(Error::Quota)?;
        *entry = Entry::Pending(
            Delivery {
                authority,
                ticket,
                operation: request.operation(),
                expires_at,
            },
            request,
        );
        self.last_ticket = ticket;
        Ok(ticket)
    }

    /// Run under the kernel lock before starting queued work or delivering
    /// results. Dropping a delivery must never drop an in-flight CRB command.
    pub(super) fn reap(&mut self, now: u64, mut is_live: impl FnMut(Authority) -> bool) {
        for entry in &mut self.entries {
            let (delivery, expired) = match *entry {
                Entry::Pending(delivery, _) | Entry::Ready(delivery, _) => (delivery, Entry::Empty),
                Entry::Running(delivery) => (delivery, Entry::Draining(delivery.ticket)),
                Entry::Empty | Entry::Draining(_) => continue,
            };
            if now >= delivery.expires_at || !is_live(delivery.authority) {
                *entry = expired;
            }
        }
    }

    pub(super) fn begin(
        &mut self,
        domain: u8,
        now: u64,
        is_live: impl FnOnce(Authority) -> bool,
    ) -> Option<Work> {
        let entry = self.entries.get_mut(usize::from(domain))?;
        let Entry::Pending(delivery, request) = *entry else {
            return None;
        };
        if now >= delivery.expires_at || !is_live(delivery.authority) {
            *entry = Entry::Empty;
            return None;
        }
        *entry = Entry::Running(delivery);
        Some(Work {
            authority: delivery.authority,
            ticket: delivery.ticket,
            request,
        })
    }

    /// Called once the CRB owner has settled the operation, including a
    /// failed or cancelled operation. A draining operation has no recipient.
    pub(super) fn complete(
        &mut self,
        domain: u8,
        ticket: u64,
        now: u64,
        completion: Completion,
    ) -> Result<(), Error> {
        let entry = self
            .entries
            .get_mut(usize::from(domain))
            .ok_or(Error::Invalid)?;
        *entry = match *entry {
            Entry::Running(delivery) if delivery.ticket == ticket => {
                if now < delivery.expires_at {
                    Entry::Ready(delivery, completion.checked_for(delivery.operation))
                } else {
                    Entry::Empty
                }
            }
            Entry::Draining(retired) if retired == ticket => Entry::Empty,
            _ => return Err(Error::BadHandle),
        };
        Ok(())
    }

    /// The copy validates the entire destination before writing any bytes.
    /// Commit the consumption only after that copy succeeds.
    pub(super) fn collect(
        &mut self,
        authority: Authority,
        operation: Operation,
        ticket: u64,
        now: u64,
        copy: impl FnOnce(&Digest) -> Result<(), Error>,
    ) -> Result<u64, Error> {
        let entry = self
            .entries
            .get_mut(usize::from(authority.domain))
            .ok_or(Error::Invalid)?;
        let (delivery, completion) = match *entry {
            Entry::Pending(delivery, _) | Entry::Running(delivery) => (delivery, None),
            Entry::Ready(delivery, completion) => (delivery, Some(completion)),
            Entry::Draining(_) | Entry::Empty => return Err(Error::BadHandle),
        };
        if delivery.authority != authority
            || delivery.ticket != ticket
            || delivery.operation != operation
            || now >= delivery.expires_at
        {
            return Err(Error::BadHandle);
        }
        let completion = completion.ok_or(Error::Again)?;
        if let Completion::Value(digest) = completion {
            copy(&digest)?;
        }
        *entry = Entry::Empty;
        Ok(completion.status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NETWORK: Authority = Authority {
        pid: 1,
        handle: 0x1_0000_0001,
        domain: 0,
        object_key: 0x1_0000_00ff,
    };
    const STORAGE: Authority = Authority {
        pid: 2,
        domain: 1,
        object_key: 0x1_0000_0100,
        ..NETWORK
    };

    #[test]
    fn collection_checks_every_binding_and_commits_the_copy_once() {
        let mut mailboxes = Mailboxes::new();
        let ticket = mailboxes.submit(NETWORK, Request::Read, 10).unwrap();
        let original = mailboxes.begin(0, 0, |_| true).unwrap();
        assert_eq!(original.authority, NETWORK);
        assert_eq!(original.ticket, ticket);
        assert_eq!(original.request, Request::Read);
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Read, ticket, 11, |_| panic!()),
            Err(Error::Again)
        );
        mailboxes
            .complete(0, ticket, 12, Completion::Value([0x32; 32]))
            .unwrap();
        for invalid in [
            STORAGE,
            Authority { pid: 3, ..NETWORK },
            Authority {
                handle: 0x2_0000_0001,
                ..NETWORK
            },
            Authority {
                object_key: 0x2_0000_00ff,
                ..NETWORK
            },
        ] {
            assert_eq!(
                mailboxes.collect(invalid, Operation::Read, ticket, 12, |_| panic!()),
                Err(Error::BadHandle)
            );
        }
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Instance, ticket, 12, |_| panic!()),
            Err(Error::BadHandle)
        );
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Read, ticket + 1, 12, |_| panic!()),
            Err(Error::BadHandle)
        );
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Read, ticket, 12, |_| Err(Error::Fault)),
            Err(Error::Fault)
        );
        let mut output = [0; 32];
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Read, ticket, 13, |value| {
                output.copy_from_slice(value);
                Ok(())
            }),
            Ok(0)
        );
        assert_eq!(output, [0x32; 32]);
        assert_eq!(
            mailboxes.collect(NETWORK, Operation::Read, ticket, 13, |_| panic!()),
            Err(Error::BadHandle)
        );
    }

    #[test]
    fn expired_running_work_blocks_its_domain_until_transport_settles() {
        let mut mailboxes = Mailboxes::new();
        let request = Request::Extend {
            expected: [1; 32],
            store: [2; 32],
        };
        let ticket = mailboxes.submit(NETWORK, request, 0).unwrap();
        assert_eq!(mailboxes.begin(0, 0, |_| true).unwrap().request, request);
        assert_eq!(
            mailboxes.collect(
                NETWORK,
                Operation::Extend,
                ticket,
                LIFETIME_TICKS - 1,
                |_| panic!()
            ),
            Err(Error::Again)
        );
        mailboxes.reap(LIFETIME_TICKS, |_| true);
        assert_eq!(
            mailboxes.collect(
                NETWORK,
                Operation::Extend,
                ticket,
                LIFETIME_TICKS,
                |_| panic!()
            ),
            Err(Error::BadHandle)
        );
        assert_eq!(
            mailboxes.submit(NETWORK, Request::Read, LIFETIME_TICKS),
            Err(Error::Again)
        );
        let storage = mailboxes
            .submit(STORAGE, Request::Instance, LIFETIME_TICKS)
            .unwrap();
        assert_ne!(storage, ticket);
        assert_eq!(
            mailboxes.begin(1, LIFETIME_TICKS, |_| true).unwrap().ticket,
            storage
        );
        assert_eq!(
            mailboxes.complete(0, storage, LIFETIME_TICKS, Completion::Extended),
            Err(Error::BadHandle)
        );
        mailboxes
            .complete(0, ticket, LIFETIME_TICKS, Completion::Extended)
            .unwrap();
        assert_ne!(
            mailboxes
                .submit(NETWORK, Request::Read, LIFETIME_TICKS)
                .unwrap(),
            ticket
        );
    }

    #[test]
    fn authority_loss_discards_pending_and_ready_work_and_drains_running_work() {
        let mut mailboxes = Mailboxes::new();
        for stage in 0..3 {
            let ticket = mailboxes.submit(NETWORK, Request::Read, 0).unwrap();
            if stage != 0 {
                mailboxes.begin(0, 0, |_| true).unwrap();
            }
            if stage == 2 {
                mailboxes
                    .complete(0, ticket, 0, Completion::Value([1; 32]))
                    .unwrap();
            }
            mailboxes.reap(1, |_| false);
            assert_eq!(
                mailboxes.collect(NETWORK, Operation::Read, ticket, 1, |_| panic!()),
                Err(Error::BadHandle)
            );
            assert_eq!(mailboxes.begin(0, 0, |_| true), None);
            if stage == 1 {
                assert_eq!(
                    mailboxes.submit(NETWORK, Request::Read, 1),
                    Err(Error::Again)
                );
                mailboxes
                    .complete(0, ticket, 1, Completion::Value([1; 32]))
                    .unwrap();
            }
        }
        mailboxes.submit(NETWORK, Request::Read, 1).unwrap();
        assert_eq!(mailboxes.begin(0, 1, |_| false), None);
        mailboxes.submit(NETWORK, Request::Read, 1).unwrap();
        assert_eq!(mailboxes.begin(0, 1 + LIFETIME_TICKS, |_| true), None);
        assert!(mailboxes
            .submit(NETWORK, Request::Read, 1 + LIFETIME_TICKS)
            .is_ok());
    }

    #[test]
    fn completion_status_is_canonical_and_never_copies_on_error() {
        let mut mailboxes = Mailboxes::new();
        for (completion, status) in [
            (Completion::Failed(Failure::Unavailable), 1),
            (Completion::Extended, 2),
            (Completion::Failed(Failure::Conflict), 3),
            (Completion::Failed(Failure::RateLimited), 4),
        ] {
            let ticket = mailboxes.submit(NETWORK, Request::Instance, 0).unwrap();
            mailboxes.begin(0, 0, |_| true).unwrap();
            mailboxes.complete(0, ticket, 0, completion).unwrap();
            assert_eq!(
                mailboxes.collect(NETWORK, Operation::Instance, ticket, 0, |_| panic!()),
                Ok(status)
            );
        }
    }

    #[test]
    fn tick_and_ticket_exhaustion_never_wrap() {
        let mut mailboxes = Mailboxes::new();
        assert_eq!(
            mailboxes.submit(NETWORK, Request::Read, u64::MAX),
            Err(Error::Quota)
        );
        mailboxes.last_ticket = u64::MAX - 1;
        assert_eq!(mailboxes.submit(NETWORK, Request::Read, 0), Ok(u64::MAX));
        mailboxes.reap(LIFETIME_TICKS, |_| true);
        assert_eq!(mailboxes.begin(0, 0, |_| true), None);
        assert_eq!(
            mailboxes.submit(NETWORK, Request::Read, LIFETIME_TICKS),
            Err(Error::Quota)
        );
    }
}
