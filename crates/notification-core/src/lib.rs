#![no_std]

use core::num::{NonZeroU32, NonZeroU64};

const PRODUCER_SLOTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotificationId(NonZeroU64);

impl NotificationId {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProducerIdentity(NonZeroU64);

impl ProducerIdentity {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageReference(NonZeroU32);

impl MessageReference {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionReference(NonZeroU32);

impl ActionReference {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Sensitive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Notification {
    pub id: NotificationId,
    pub producer: ProducerIdentity,
    pub priority: Priority,
    pub visibility: Visibility,
    pub expires_at: u64,
    pub message: MessageReference,
    pub action: Option<ActionReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotificationView {
    pub id: NotificationId,
    pub producer: ProducerIdentity,
    pub priority: Priority,
    pub message: MessageReference,
    pub action: Option<ActionReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidCapacity,
    InvalidRateLimit,
    TimeRegressed,
    AlreadyExpired,
    DuplicateId,
    Full,
    ProducerLimit,
    ProducerExited,
    UnknownProducer,
    UnknownNotification,
    RateLimited,
}

#[derive(Clone, Copy)]
struct ProducerState {
    identity: Option<ProducerIdentity>,
    window_start: u64,
    admitted: u16,
    exited: bool,
}

impl ProducerState {
    const EMPTY: Self = Self {
        identity: None,
        window_start: 0,
        admitted: 0,
        exited: false,
    };
}

pub struct NotificationCore<const CAPACITY: usize> {
    entries: [Option<Notification>; CAPACITY],
    producers: [ProducerState; PRODUCER_SLOTS],
    window_ticks: u64,
    max_posts_per_window: u16,
    last_ticks: Option<u64>,
}

impl<const CAPACITY: usize> NotificationCore<CAPACITY> {
    pub fn new(window_ticks: u64, max_posts_per_window: u16) -> Result<Self, Error> {
        if CAPACITY == 0 {
            return Err(Error::InvalidCapacity);
        }
        if window_ticks == 0 || max_posts_per_window == 0 {
            return Err(Error::InvalidRateLimit);
        }
        Ok(Self {
            entries: [None; CAPACITY],
            producers: [ProducerState::EMPTY; PRODUCER_SLOTS],
            window_ticks,
            max_posts_per_window,
            last_ticks: None,
        })
    }

    pub fn post(&mut self, notification: Notification, now: u64) -> Result<(), Error> {
        self.check_time(now)?;
        self.expire_entries(now);
        if notification.expires_at <= now {
            return Err(Error::AlreadyExpired);
        }
        if self
            .entries
            .iter()
            .flatten()
            .any(|entry| entry.id == notification.id)
        {
            return Err(Error::DuplicateId);
        }
        let slot = self
            .entries
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let producer_slot = self.producer_slot(notification.producer, now)?;
        let producer = &mut self.producers[producer_slot];
        if producer.exited {
            return Err(Error::ProducerExited);
        }
        if now.saturating_sub(producer.window_start) >= self.window_ticks {
            producer.window_start = now;
            producer.admitted = 0;
        }
        if producer.admitted >= self.max_posts_per_window {
            return Err(Error::RateLimited);
        }
        producer.admitted += 1;
        self.entries[slot] = Some(notification);
        Ok(())
    }

    pub fn expire(&mut self, now: u64) -> Result<usize, Error> {
        self.check_time(now)?;
        Ok(self.expire_entries(now))
    }

    pub fn dismiss(&mut self, id: NotificationId) -> Result<(), Error> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.id == id))
            .ok_or(Error::UnknownNotification)?;
        self.entries[index] = None;
        Ok(())
    }

    pub fn producer_exit(&mut self, identity: ProducerIdentity) -> Result<usize, Error> {
        let producer = self
            .producers
            .iter_mut()
            .find(|producer| producer.identity == Some(identity))
            .ok_or(Error::UnknownProducer)?;
        producer.exited = true;
        let mut removed = 0;
        for entry in &mut self.entries {
            if entry.is_some_and(|entry| entry.producer == identity) {
                *entry = None;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn display(&self, id: NotificationId, session_locked: bool) -> Option<NotificationView> {
        let notification = self.entries.iter().flatten().find(|entry| entry.id == id)?;
        if session_locked && notification.visibility != Visibility::Public {
            return None;
        }
        Some(NotificationView {
            id: notification.id,
            producer: notification.producer,
            priority: notification.priority,
            message: notification.message,
            action: notification.action,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn check_time(&mut self, now: u64) -> Result<(), Error> {
        if self.last_ticks.is_some_and(|last| now < last) {
            return Err(Error::TimeRegressed);
        }
        self.last_ticks = Some(now);
        Ok(())
    }

    fn expire_entries(&mut self, now: u64) -> usize {
        let mut removed = 0;
        for entry in &mut self.entries {
            if entry.is_some_and(|notification| notification.expires_at <= now) {
                *entry = None;
                removed += 1;
            }
        }
        removed
    }

    fn producer_slot(&mut self, identity: ProducerIdentity, now: u64) -> Result<usize, Error> {
        if let Some(index) = self
            .producers
            .iter()
            .position(|producer| producer.identity == Some(identity))
        {
            return Ok(index);
        }
        let index = self
            .producers
            .iter()
            .position(|producer| producer.identity.is_none())
            .ok_or(Error::ProducerLimit)?;
        self.producers[index] = ProducerState {
            identity: Some(identity),
            window_start: now,
            admitted: 0,
            exited: false,
        };
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(
        id: u64,
        producer: u64,
        visibility: Visibility,
        expires_at: u64,
    ) -> Notification {
        Notification {
            id: NotificationId::new(id).unwrap(),
            producer: ProducerIdentity::new(producer).unwrap(),
            priority: Priority::Normal,
            visibility,
            expires_at,
            message: MessageReference::new(1).unwrap(),
            action: Some(ActionReference::new(2).unwrap()),
        }
    }

    #[test]
    fn construction_rejects_zero_bounds_and_ids() {
        assert_eq!(
            NotificationCore::<0>::new(1, 1).err(),
            Some(Error::InvalidCapacity)
        );
        assert_eq!(
            NotificationCore::<1>::new(0, 1).err(),
            Some(Error::InvalidRateLimit)
        );
        assert_eq!(
            NotificationCore::<1>::new(1, 0).err(),
            Some(Error::InvalidRateLimit)
        );
        assert!(NotificationId::new(0).is_none());
        assert!(ProducerIdentity::new(0).is_none());
        assert!(MessageReference::new(0).is_none());
        assert!(ActionReference::new(0).is_none());
    }

    #[test]
    fn capacity_duplicate_expiry_and_dismissal_are_bounded() {
        let mut core = NotificationCore::<2>::new(10, 4).unwrap();
        let first = notification(1, 10, Visibility::Public, 5);
        let second = notification(2, 10, Visibility::Private, 20);
        core.post(first, 1).unwrap();
        assert_eq!(core.post(first, 2), Err(Error::DuplicateId));
        core.post(second, 2).unwrap();
        assert_eq!(
            core.post(notification(3, 10, Visibility::Public, 20), 3),
            Err(Error::Full)
        );
        assert_eq!(core.expire(5), Ok(1));
        core.post(notification(3, 10, Visibility::Public, 20), 5)
            .unwrap();
        assert_eq!(core.dismiss(NotificationId::new(2).unwrap()), Ok(()));
        assert_eq!(
            core.dismiss(NotificationId::new(2).unwrap()),
            Err(Error::UnknownNotification)
        );
        assert_eq!(core.len(), 1);
    }

    #[test]
    fn producer_exit_removes_notifications_and_prevents_reuse() {
        let mut core = NotificationCore::<4>::new(10, 4).unwrap();
        core.post(notification(1, 10, Visibility::Public, 20), 1)
            .unwrap();
        core.post(notification(2, 10, Visibility::Private, 20), 2)
            .unwrap();
        assert_eq!(
            core.producer_exit(ProducerIdentity::new(10).unwrap()),
            Ok(2)
        );
        assert!(core.is_empty());
        assert_eq!(
            core.post(notification(3, 10, Visibility::Public, 20), 3),
            Err(Error::ProducerExited)
        );
        assert_eq!(
            core.producer_exit(ProducerIdentity::new(11).unwrap()),
            Err(Error::UnknownProducer)
        );
    }

    #[test]
    fn locked_display_hides_private_and_sensitive_references() {
        let mut core = NotificationCore::<3>::new(10, 3).unwrap();
        for (id, visibility) in [
            (1, Visibility::Public),
            (2, Visibility::Private),
            (3, Visibility::Sensitive),
        ] {
            core.post(notification(id, 10, visibility, 20), id).unwrap();
        }
        assert!(core
            .display(NotificationId::new(1).unwrap(), true)
            .is_some());
        assert!(core
            .display(NotificationId::new(2).unwrap(), true)
            .is_none());
        assert!(core
            .display(NotificationId::new(3).unwrap(), true)
            .is_none());
        assert_eq!(
            core.display(NotificationId::new(3).unwrap(), false)
                .unwrap()
                .action,
            Some(ActionReference::new(2).unwrap())
        );
    }

    #[test]
    fn rate_limit_is_per_producer_and_time_cannot_regress() {
        let mut core = NotificationCore::<4>::new(10, 1).unwrap();
        core.post(notification(1, 10, Visibility::Public, 20), 1)
            .unwrap();
        assert_eq!(
            core.post(notification(2, 10, Visibility::Public, 20), 2),
            Err(Error::RateLimited)
        );
        core.post(notification(3, 11, Visibility::Public, 20), 2)
            .unwrap();
        core.post(notification(4, 10, Visibility::Public, 20), 11)
            .unwrap();
        assert_eq!(core.expire(10), Err(Error::TimeRegressed));
    }

    #[test]
    fn expired_input_fails_without_adding_a_notification() {
        let mut core = NotificationCore::<2>::new(10, 2).unwrap();
        assert_eq!(
            core.post(notification(1, 10, Visibility::Public, 5), 5),
            Err(Error::AlreadyExpired)
        );
        assert!(core.is_empty());
    }
}
