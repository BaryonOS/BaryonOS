//! Allocation-free DNS cache. No transport, authentication or destination rights.

use crate::dns::{self, Name, QueryType, Question, Resolution};

pub const CACHE_CAPACITY: usize = 16;
/// Local negative-cache limit in seconds, below the positive-cache cap.
pub const MAX_NEGATIVE_TTL: u32 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Dns(dns::Error),
    NxDomain,
    ServFail,
}

impl From<dns::Error> for Error {
    fn from(error: dns::Error) -> Self {
        match error {
            dns::Error::ResponseCode(2) => Self::ServFail,
            other => Self::Dns(other),
        }
    }
}

#[derive(Clone, Debug)]
struct Entry {
    name: Name,
    kind: QueryType,
    expires: u64,
    result: Result<Resolution, Error>,
}

/// Caller supplies trusted, nondecreasing ticks and the current configuration
/// epoch on every operation. Replies must first be bound to the pending query's
/// transport endpoints/deadline by the caller; DNS identity is not authentication.
pub struct Resolver {
    entries: [Option<Entry>; CACHE_CAPACITY],
    epoch: u64,
    ticks_per_second: u64,
    next_victim: usize,
}

impl Resolver {
    pub fn new(epoch: u64, ticks_per_second: u64) -> Result<Self, Error> {
        if ticks_per_second == 0 {
            return Err(dns::Error::InvalidLifetime.into());
        }
        Ok(Self {
            entries: [const { None }; CACHE_CAPACITY],
            epoch,
            ticks_per_second,
            next_victim: 0,
        })
    }

    fn configure(&mut self, epoch: u64) {
        if self.epoch != epoch {
            self.entries.fill(None);
            self.next_victim = 0;
            self.epoch = epoch;
        }
    }

    /// Misses and exact expiry return None. Positive TTLs reflect remaining
    /// whole seconds, never the original lifetime or a renewed cache deadline.
    pub fn lookup(
        &mut self,
        name: Name,
        kind: QueryType,
        epoch: u64,
        now: u64,
    ) -> Option<Result<Resolution, Error>> {
        self.configure(epoch);
        for slot in &mut self.entries {
            if slot.as_ref().is_some_and(|entry| now >= entry.expires) {
                *slot = None;
            }
            if let Some(entry) = slot {
                if entry.name == name && entry.kind == kind {
                    return Some(entry.result.clone().map(|mut result| {
                        result.cache_ttl = ((entry.expires - now) / self.ticks_per_second) as u32;
                        result
                    }));
                }
            }
        }
        None
    }

    /// Validate the complete supported reply and checked deadline before cache
    /// mutation. NXDOMAIN is an explicit error, optionally cached using its SOA.
    /// Other errors leave existing entries intact within the same epoch.
    pub fn accept(
        &mut self,
        message: &[u8],
        question: Question,
        epoch: u64,
        now: u64,
    ) -> Result<Resolution, Error> {
        self.configure(epoch);
        let (response, rcode) = dns::parse_resolver_response(message, question)?;
        let (result, ttl) = if rcode == 3 {
            let ttl = response.negative_cache_ttl()?.min(MAX_NEGATIVE_TTL);
            (Err(Error::NxDomain), ttl)
        } else {
            let resolution = response.resolve()?;
            let ttl = resolution.cache_ttl;
            (Ok(resolution), ttl)
        };
        let duration = u64::from(ttl)
            .checked_mul(self.ticks_per_second)
            .ok_or(dns::Error::ClockOverflow)?;
        let expires = now.checked_add(duration).ok_or(dns::Error::ClockOverflow)?;
        let existing = self.entries.iter().position(|slot| {
            slot.as_ref()
                .is_some_and(|entry| entry.name == question.name && entry.kind == question.kind)
        });
        if ttl == 0 {
            if let Some(index) = existing {
                self.entries[index] = None;
            }
            return result;
        }
        let index = existing
            .or_else(|| {
                self.entries
                    .iter()
                    .position(|slot| slot.as_ref().is_none_or(|entry| now >= entry.expires))
            })
            .unwrap_or(self.next_victim);
        self.entries[index] = Some(Entry {
            name: question.name,
            kind: question.kind,
            expires,
            result: result.clone(),
        });
        if existing.is_none() {
            self.next_victim = (index + 1) % CACHE_CAPACITY;
        }
        result
    }
}
