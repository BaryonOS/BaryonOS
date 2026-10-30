#![no_std]
#![forbid(unsafe_code)]
//! Read-only MC146818-compatible CMOS RTC policy for a platform-owned port pair.
//! This does not authenticate time, own hardware grants, or provide Sys3 time.

const SECONDS: u8 = 0;
const MINUTES: u8 = 2;
const HOURS: u8 = 4;
const DAY: u8 = 7;
const MONTH: u8 = 8;
const YEAR: u8 = 9;
const A: u8 = 10;
const B: u8 = 11;
const D: u8 = 13;
pub const MAX_REGISTER_READS: u32 = 65_536;
pub const MAX_ATTEMPTS: u8 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiState {
    Enabled,
    Masked,
}

/// Authoritative platform shadow of port 0x70, not a read from that port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexSelection {
    register: u8,
    nmi: NmiState,
}
impl IndexSelection {
    pub const fn new(register: u8, nmi: NmiState) -> Option<Self> {
        if register < 128 {
            Some(Self { register, nmi })
        } else {
            None
        }
    }
    pub const fn register(self) -> u8 {
        self.register
    }
    pub const fn encoded(self) -> u8 {
        self.register
            | match self.nmi {
                NmiState::Enabled => 0,
                NmiState::Masked => 128,
            }
    }
}

/// Exclusively owned, serialized index/data port pair (PC: 0x70/0x71).
/// Every operation must return in bounded time. `select` writes only the index
/// port; it must never write CMOS data. The adapter handles required I/O delay
/// and excludes every other CPU/IRQ/NMI user of the pair for the whole read.
pub trait CmosIo {
    type Error;
    fn select(&mut self, selection: IndexSelection) -> Result<(), Self::Error>;
    fn settle(&mut self) -> Result<(), Self::Error>;
    fn read_data(&mut self) -> Result<u8, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CenturyRegister(u8);
impl CenturyRegister {
    /// Must come from verified platform configuration/FADT, not a year guess.
    /// Excludes time/control registers, including interrupt-acknowledging C.
    pub const fn new(index: u8) -> Option<Self> {
        if index >= 14 && index < 128 {
            Some(Self(index))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeBasis {
    /// Platform contract asserts RTC contains UTC (e.g. explicit QEMU -rtc).
    ConfiguredUtc,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub century: Option<CenturyRegister>,
    pub basis: TimeBasis,
    pub register_reads: u32,
    pub attempts: u8,
    /// Optional external floor; this crate cannot authenticate its provenance.
    pub minimum: Option<UnixUtc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Configuration,
    CenturyUnavailable,
    TimeBasisUnknown,
    MissingRtc,
    BatteryInvalid,
    InvalidControl,
    ClockStopped,
    DaylightSavingEnabled,
    ReadBudgetExhausted,
    Unstable,
    InvalidBcd,
    InvalidDate,
    Overflow,
    Rollback,
    RestoreFailed,
    Poisoned,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Failure<E> {
    Clock(Fault),
    Io(E),
}
impl<E> From<Fault> for Failure<E> {
    fn from(fault: Fault) -> Self {
        Self::Clock(fault)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ReadError<E> {
    pub failure: Failure<E>,
    /// A failed restoration poisons the reader; no later I/O is attempted.
    pub restore_error: Option<E>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixUtc(i64);
impl UnixUtc {
    pub const MIN: i64 = -62_135_596_800;
    pub const MAX: i64 = 253_402_300_799;
    pub const fn from_seconds(seconds: i64) -> Option<Self> {
        if seconds >= Self::MIN && seconds <= Self::MAX {
            Some(Self(seconds))
        } else {
            None
        }
    }
    pub const fn seconds(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateUtc {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
impl DateUtc {
    pub fn unix(self) -> Result<UnixUtc, Fault> {
        let y = i64::from(self.year);
        let m = i64::from(self.month);
        let leap = self.year.is_multiple_of(4)
            && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400));
        let max_day = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return Err(Fault::InvalidDate),
        };
        if !(1..=9999).contains(&self.year)
            || self.day == 0
            || self.day > max_day
            || self.hour > 23
            || self.minute > 59
            || self.second > 59
        {
            return Err(Fault::InvalidDate);
        }
        // Howard Hinnant, days_from_civil (2021-09-01, public domain).
        // March-based 400-year eras; input validation keeps adjusted y >= 0.
        let adjusted = y
            .checked_sub(i64::from(self.month <= 2))
            .ok_or(Fault::Overflow)?;
        let era = adjusted / 400;
        let year_of_era = adjusted
            .checked_sub(era.checked_mul(400).ok_or(Fault::Overflow)?)
            .ok_or(Fault::Overflow)?;
        let march_month = m
            .checked_add(if self.month > 2 { -3 } else { 9 })
            .ok_or(Fault::Overflow)?;
        let day_of_year = march_month
            .checked_mul(153)
            .and_then(|n| n.checked_add(2))
            .map(|n| n / 5)
            .and_then(|n| n.checked_add(i64::from(self.day)))
            .and_then(|n| n.checked_sub(1))
            .ok_or(Fault::Overflow)?;
        let day_of_era = year_of_era
            .checked_mul(365)
            .and_then(|n| n.checked_add(year_of_era / 4))
            .and_then(|n| n.checked_sub(year_of_era / 100))
            .and_then(|n| n.checked_add(day_of_year))
            .ok_or(Fault::Overflow)?;
        let days = era
            .checked_mul(146_097)
            .and_then(|n| n.checked_add(day_of_era))
            .and_then(|n| n.checked_sub(719_468))
            .ok_or(Fault::Overflow)?;
        let second_of_day = i64::from(self.hour)
            .checked_mul(3600)
            .and_then(|n| n.checked_add(i64::from(self.minute) * 60))
            .and_then(|n| n.checked_add(i64::from(self.second)))
            .ok_or(Fault::Overflow)?;
        let seconds = seconds_from_days(days, second_of_day)?;
        UnixUtc::from_seconds(seconds).ok_or(Fault::Overflow)
    }
}
fn seconds_from_days(days: i64, seconds: i64) -> Result<i64, Fault> {
    days.checked_mul(86_400)
        .and_then(|n| n.checked_add(seconds))
        .ok_or(Fault::Overflow)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    CmosRtc,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    /// Stable/valid representation, not authenticated or secure UTC.
    UnauthenticatedRtc,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorSource {
    CallerSupplied,
    EarlierRtcReading,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackCheck {
    Unchecked,
    NotBelow { floor: UnixUtc, source: FloorSource },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    pub utc: UnixUtc,
    pub date: DateUtc,
    pub source: Source,
    pub quality: Quality,
    pub rollback: RollbackCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Raw {
    a: u8,
    b: u8,
    fields: [u8; 7],
}
fn number(value: u8, binary: bool) -> Result<u8, Fault> {
    if binary {
        Ok(value)
    } else if value & 15 <= 9 && value >> 4 <= 9 {
        Ok((value >> 4) * 10 + (value & 15))
    } else {
        Err(Fault::InvalidBcd)
    }
}
fn decode(raw: Raw) -> Result<DateUtc, Fault> {
    let binary = raw.b & 4 != 0;
    let mut hour = number(raw.fields[2] & 0x7f, binary)?;
    if raw.b & 2 != 0 {
        if raw.fields[2] & 0x80 != 0 {
            return Err(Fault::InvalidDate);
        }
    } else {
        if !(1..=12).contains(&hour) {
            return Err(Fault::InvalidDate);
        }
        hour = hour % 12 + if raw.fields[2] & 0x80 != 0 { 12 } else { 0 };
    }
    let year = number(raw.fields[5], binary)?;
    let century = number(raw.fields[6], binary)?;
    if year > 99 || century > 99 {
        return Err(Fault::InvalidDate);
    }
    let date = DateUtc {
        year: u16::from(century)
            .checked_mul(100)
            .and_then(|n| n.checked_add(u16::from(year)))
            .ok_or(Fault::Overflow)?,
        month: number(raw.fields[4], binary)?,
        day: number(raw.fields[3], binary)?,
        hour,
        minute: number(raw.fields[1], binary)?,
        second: number(raw.fields[0], binary)?,
    };
    date.unix()?;
    Ok(date)
}

pub struct Rtc<I> {
    io: I,
    initial: IndexSelection,
    configuration: Configuration,
    floor: Option<(UnixUtc, FloorSource)>,
    poisoned: bool,
}
impl<I: CmosIo> Rtc<I> {
    /// Transfers exclusive access to the pair, including the authoritative NMI
    /// policy/index shadow. Construction does not access any hardware.
    pub fn new(
        io: I,
        initial: IndexSelection,
        configuration: Configuration,
    ) -> Result<Self, Fault> {
        if configuration.register_reads == 0
            || configuration.register_reads > MAX_REGISTER_READS
            || configuration.attempts == 0
            || configuration.attempts > MAX_ATTEMPTS
        {
            return Err(Fault::Configuration);
        }
        Ok(Self {
            io,
            initial,
            configuration,
            floor: configuration
                .minimum
                .map(|utc| (utc, FloorSource::CallerSupplied)),
            poisoned: false,
        })
    }
    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }
    pub fn into_io(self) -> I {
        self.io
    }

    fn register(&mut self, index: u8, remaining: &mut u32) -> Result<u8, Failure<I::Error>> {
        *remaining = remaining.checked_sub(1).ok_or(Fault::ReadBudgetExhausted)?;
        self.io
            .select(IndexSelection {
                register: index,
                nmi: self.initial.nmi,
            })
            .map_err(Failure::Io)?;
        self.io.settle().map_err(Failure::Io)?;
        self.io.read_data().map_err(Failure::Io)
    }
    fn battery(value: u8) -> Result<(), Fault> {
        match value {
            0xff => Err(Fault::MissingRtc),
            0x80 => Ok(()),
            v if v & 0x80 == 0 => Err(Fault::BatteryInvalid),
            _ => Err(Fault::InvalidControl),
        }
    }
    fn sample(
        &mut self,
        century: CenturyRegister,
        remaining: &mut u32,
    ) -> Result<Option<Raw>, Failure<I::Error>> {
        Self::battery(self.register(D, remaining)?)?;
        let a = loop {
            let a = self.register(A, remaining)?;
            if a == 0xff {
                return Err(Fault::MissingRtc.into());
            }
            // Exact PC/QEMU profile: running 32.768-kHz divider. Do not
            // silently interpret a reset/stopped or unreviewed divider mode.
            if a & 0x70 != 0x20 {
                return Err(Fault::ClockStopped.into());
            }
            if a & 0x80 == 0 {
                break a;
            }
        };
        let b = self.register(B, remaining)?;
        if b & 0x80 != 0 {
            return Err(Fault::ClockStopped.into());
        }
        if b & 1 != 0 {
            return Err(Fault::DaylightSavingEnabled.into());
        }
        let mut fields = [0; 7];
        for (field, index) in fields
            .iter_mut()
            .zip([SECONDS, MINUTES, HOURS, DAY, MONTH, YEAR, century.0])
        {
            *field = self.register(index, remaining)?;
        }
        let after_a = self.register(A, remaining)?;
        let after_b = self.register(B, remaining)?;
        let after_d = self.register(D, remaining)?;
        Self::battery(after_d)?;
        if a != after_a || b != after_b {
            return Ok(None);
        }
        Ok(Some(Raw { a, b, fields }))
    }
    fn read_inner(&mut self, century: CenturyRegister) -> Result<Reading, Failure<I::Error>> {
        let mut remaining = self.configuration.register_reads;
        for _ in 0..self.configuration.attempts {
            let Some(first) = self.sample(century, &mut remaining)? else {
                continue;
            };
            let Some(second) = self.sample(century, &mut remaining)? else {
                continue;
            };
            if first != second {
                continue;
            }
            let date = decode(second)?;
            let utc = date.unix()?;
            let rollback = match self.floor {
                Some((floor, _)) if utc < floor => return Err(Fault::Rollback.into()),
                Some((floor, source)) => RollbackCheck::NotBelow { floor, source },
                None => RollbackCheck::Unchecked,
            };
            return Ok(Reading {
                utc,
                date,
                source: Source::CmosRtc,
                quality: Quality::UnauthenticatedRtc,
                rollback,
            });
        }
        Err(Fault::Unstable.into())
    }
    pub fn read_utc(&mut self) -> Result<Reading, ReadError<I::Error>> {
        let early = if self.poisoned {
            Some(Fault::Poisoned)
        } else if self.configuration.century.is_none() {
            Some(Fault::CenturyUnavailable)
        } else if self.configuration.basis != TimeBasis::ConfiguredUtc {
            Some(Fault::TimeBasisUnknown)
        } else {
            None
        };
        if let Some(fault) = early {
            return Err(ReadError {
                failure: Failure::Clock(fault),
                restore_error: None,
            });
        }
        let result = self.read_inner(self.configuration.century.unwrap());
        // Restore even after a failed select/read. Never attempt CMOS-data
        // writes or derive the NMI mask from reading port 0x70.
        let restoration = self.io.select(self.initial).and_then(|_| self.io.settle());
        match restoration {
            Err(error) => {
                self.poisoned = true;
                Err(ReadError {
                    failure: result.err().unwrap_or(Failure::Clock(Fault::RestoreFailed)),
                    restore_error: Some(error),
                })
            }
            Ok(()) => match result {
                Ok(reading) => {
                    self.floor = Some((reading.utc, FloorSource::EarlierRtcReading));
                    Ok(reading)
                }
                Err(failure) => Err(ReadError {
                    failure,
                    restore_error: None,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_overflow_never_wraps() {
        assert_eq!(seconds_from_days(i64::MAX, 0), Err(Fault::Overflow));
        assert_eq!(seconds_from_days(i64::MIN, 0), Err(Fault::Overflow));
        assert_eq!(seconds_from_days(1, i64::MAX), Err(Fault::Overflow));
    }
}
