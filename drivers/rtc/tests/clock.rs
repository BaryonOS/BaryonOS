//! Device-state fixtures only; no host wall clock or hardware I/O is used.
use nieuw_os_rtc::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IoError {
    Offline,
    Read,
    Select,
    Settle,
    Restore,
}
struct Ports {
    bytes: [u8; 128],
    selected: u8,
    selections: Vec<u8>,
    reads: usize,
    pauses: usize,
    overrides: BTreeMap<u8, VecDeque<u8>>,
    fail_read: bool,
    fail_select: bool,
    fail_settle: bool,
    fail_restore: bool,
    absent: bool,
}
impl Ports {
    fn bcd() -> Self {
        let mut bytes = [0; 128];
        for (index, value) in [
            (0, 0x59),
            (2, 0x58),
            (4, 0x23),
            (7, 0x30),
            (8, 0x10),
            (9, 0x26),
            (10, 0x26),
            (11, 2),
            (13, 0x80),
            (0x32, 0x20),
        ] {
            bytes[index] = value;
        }
        Self {
            bytes,
            selected: 15,
            selections: Vec::new(),
            reads: 0,
            pauses: 0,
            overrides: BTreeMap::new(),
            fail_read: false,
            fail_select: false,
            fail_settle: false,
            fail_restore: false,
            absent: false,
        }
    }
    fn script(&mut self, register: u8, values: &[u8]) {
        self.overrides
            .insert(register, values.iter().copied().collect());
    }
}
impl CmosIo for Ports {
    type Error = IoError;
    fn select(&mut self, selection: IndexSelection) -> Result<(), IoError> {
        self.selections.push(selection.encoded());
        self.selected = selection.register();
        if selection.register() == 15 && self.fail_restore {
            return Err(IoError::Restore);
        }
        if core::mem::take(&mut self.fail_select) {
            return Err(IoError::Select);
        }
        Ok(())
    }
    fn settle(&mut self) -> Result<(), IoError> {
        self.pauses += 1;
        if core::mem::take(&mut self.fail_settle) {
            return Err(IoError::Settle);
        }
        Ok(())
    }
    fn read_data(&mut self) -> Result<u8, IoError> {
        self.reads += 1;
        if self.absent {
            return Err(IoError::Offline);
        }
        if core::mem::take(&mut self.fail_read) {
            return Err(IoError::Read);
        }
        Ok(self
            .overrides
            .get_mut(&self.selected)
            .and_then(|q| q.pop_front())
            .unwrap_or(self.bytes[self.selected as usize]))
    }
}
fn config() -> Configuration {
    Configuration {
        century: CenturyRegister::new(0x32),
        basis: TimeBasis::ConfiguredUtc,
        register_reads: 512,
        attempts: 8,
        minimum: None,
    }
}
fn rtc(ports: Ports, nmi: NmiState) -> Rtc<Ports> {
    Rtc::new(ports, IndexSelection::new(15, nmi).unwrap(), config()).unwrap()
}
fn expected() -> DateUtc {
    DateUtc {
        year: 2026,
        month: 10,
        day: 30,
        hour: 23,
        minute: 58,
        second: 59,
    }
}
fn clock_fault(error: ReadError<IoError>, fault: Fault) {
    assert_eq!(
        error,
        ReadError {
            failure: Failure::Clock(fault),
            restore_error: None
        }
    );
}

#[test]
fn stable_bcd_clock_is_utc_but_never_authenticated_and_nmi_is_preserved() {
    for nmi in [NmiState::Enabled, NmiState::Masked] {
        let mut clock = rtc(Ports::bcd(), nmi);
        let reading = clock.read_utc().unwrap();
        assert_eq!(reading.date, expected());
        assert_eq!(reading.utc, expected().unix().unwrap());
        assert_eq!(reading.source, Source::CmosRtc);
        assert_eq!(reading.quality, Quality::UnauthenticatedRtc);
        assert_eq!(reading.rollback, RollbackCheck::Unchecked);
        let io = clock.into_io();
        assert_eq!(io.reads, 26);
        assert_eq!(io.selections.len(), 27);
        assert_eq!(
            *io.selections.last().unwrap(),
            if nmi == NmiState::Enabled { 15 } else { 143 }
        );
        for selection in io.selections {
            assert_eq!(
                selection & 128,
                if nmi == NmiState::Enabled { 0 } else { 128 }
            );
            assert_ne!(selection & 127, 12);
        }
    }
}

#[test]
fn binary_and_bcd_12h_and_24h_have_exact_midnight_and_noon() {
    for binary in [false, true] {
        for (input, pm, expected_hour) in [
            (12, false, 0),
            (12, true, 12),
            (1, false, 1),
            (1, true, 13),
            (11, true, 23),
        ] {
            let mut io = Ports::bcd();
            if binary {
                for (i, v) in [(0, 59), (2, 58), (7, 30), (8, 10), (9, 26), (0x32, 20)] {
                    io.bytes[i] = v;
                }
            }
            io.bytes[11] = if binary { 4 } else { 0 };
            io.bytes[4] = (if binary {
                input
            } else {
                (input / 10) * 16 + input % 10
            }) | if pm { 128 } else { 0 };
            assert_eq!(
                rtc(io, NmiState::Enabled).read_utc().unwrap().date,
                DateUtc {
                    hour: expected_hour,
                    ..expected()
                }
            );
        }
        for hour in [0, 12, 23] {
            let mut io = Ports::bcd();
            io.bytes[11] = if binary { 6 } else { 2 };
            if binary {
                for (i, v) in [(0, 59), (2, 58), (7, 30), (8, 10), (9, 26), (0x32, 20)] {
                    io.bytes[i] = v;
                }
            }
            io.bytes[4] = if binary {
                hour
            } else {
                (hour / 10) * 16 + hour % 10
            };
            assert_eq!(
                rtc(io, NmiState::Enabled).read_utc().unwrap().date.hour,
                hour
            );
        }
    }
}

#[test]
fn invalid_bcd_ranges_calendar_and_leap_seconds_are_rejected() {
    for (register, value, fault) in [
        (0, 0x1a, Fault::InvalidBcd),
        (2, 0x6a, Fault::InvalidBcd),
        (0x32, 0xa0, Fault::InvalidBcd),
        (0, 0x60, Fault::InvalidDate),
        (2, 0x60, Fault::InvalidDate),
        (4, 0x24, Fault::InvalidDate),
        (4, 0x92, Fault::InvalidDate),
        (7, 0, Fault::InvalidDate),
        (7, 0x32, Fault::InvalidDate),
        (8, 0, Fault::InvalidDate),
        (8, 0x13, Fault::InvalidDate),
    ] {
        let mut io = Ports::bcd();
        io.bytes[register] = value;
        clock_fault(rtc(io, NmiState::Enabled).read_utc().unwrap_err(), fault);
    }
    for hour in [0, 0x13, 0x80] {
        let mut io = Ports::bcd();
        io.bytes[11] = 0;
        io.bytes[4] = hour;
        clock_fault(
            rtc(io, NmiState::Enabled).read_utc().unwrap_err(),
            Fault::InvalidDate,
        );
    }
    for (register, value) in [
        (0, 60),
        (2, 60),
        (4, 24),
        (7, 32),
        (8, 13),
        (9, 100),
        (0x32, 100),
    ] {
        let mut io = Ports::bcd();
        io.bytes[11] = 6;
        for (i, v) in [
            (0, 59),
            (2, 58),
            (4, 23),
            (7, 30),
            (8, 10),
            (9, 26),
            (0x32, 20),
        ] {
            io.bytes[i] = v;
        }
        io.bytes[register] = value;
        clock_fault(
            rtc(io, NmiState::Enabled).read_utc().unwrap_err(),
            Fault::InvalidDate,
        );
    }
}

#[test]
fn published_calendar_epoch_vectors_and_full_gregorian_cycle_agree() {
    for (year, month, day, hour, minute, second, unix) in [
        (1, 1, 1, 0, 0, 0, -62135596800),
        (1900, 1, 1, 0, 0, 0, -2208988800),
        (1969, 12, 31, 23, 59, 59, -1),
        (1970, 1, 1, 0, 0, 0, 0),
        (2000, 1, 1, 0, 0, 0, 946684800),
        (2000, 2, 29, 0, 0, 0, 951782400),
        (2038, 1, 19, 3, 14, 8, 2147483648),
        (2100, 3, 1, 0, 0, 0, 4107542400),
        (9999, 12, 31, 23, 59, 59, 253402300799),
    ] {
        assert_eq!(
            DateUtc {
                year,
                month,
                day,
                hour,
                minute,
                second
            }
            .unix()
            .unwrap()
            .seconds(),
            unix
        );
    }
    let mut unix = 946684800;
    for year in 2000..2400 {
        for month in 1..=12 {
            let days = match month {
                4 | 6 | 9 | 11 => 30,
                2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
                2 => 28,
                _ => 31,
            };
            for day in 1..=days {
                assert_eq!(
                    DateUtc {
                        year,
                        month,
                        day,
                        hour: 0,
                        minute: 0,
                        second: 0
                    }
                    .unix()
                    .unwrap()
                    .seconds(),
                    unix
                );
                unix += 86_400;
            }
        }
    }
    assert_eq!(unix, 13569465600); // 2400-01-01 UTC.
    for year in [1900, 2100, 2200, 2300] {
        assert_eq!(
            DateUtc {
                year,
                month: 2,
                day: 29,
                ..expected()
            }
            .unix(),
            Err(Fault::InvalidDate)
        );
    }
    for year in [0, 10000, u16::MAX] {
        assert_eq!(
            DateUtc { year, ..expected() }.unix(),
            Err(Fault::InvalidDate)
        );
    }
    assert_eq!(UnixUtc::from_seconds(i64::MIN), None);
    assert_eq!(UnixUtc::from_seconds(i64::MAX), None);
}

#[test]
fn uip_and_midnight_changes_require_two_whole_stable_snapshots() {
    let mut io = Ports::bcd();
    io.script(10, &[0xa6, 0xa6, 0x26]);
    io.script(0, &[0x59, 0, 0, 0]);
    io.bytes[0] = 0;
    io.script(2, &[0x59, 0, 0, 0]);
    io.bytes[2] = 0;
    io.script(4, &[0x23, 0, 0, 0]);
    io.bytes[4] = 0;
    io.script(7, &[0x31, 1, 1, 1]);
    io.bytes[7] = 1;
    io.script(8, &[0x12, 1, 1, 1]);
    io.bytes[8] = 1;
    io.script(9, &[0x26, 0x27, 0x27, 0x27]);
    io.bytes[9] = 0x27;
    let mut clock = rtc(io, NmiState::Enabled);
    let r = clock.read_utc().unwrap();
    assert_eq!(
        r.date,
        DateUtc {
            year: 2027,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0
        }
    );
    assert!(clock.into_io().reads > 26);
    let mut io = Ports::bcd();
    io.script(10, &[0x26, 0xa6]);
    assert_eq!(
        rtc(io, NmiState::Enabled).read_utc().unwrap().date,
        expected()
    );
    let mut io = Ports::bcd();
    io.script(11, &[2, 6]);
    assert_eq!(
        rtc(io, NmiState::Enabled).read_utc().unwrap().date,
        expected()
    );
}

#[test]
fn stuck_uip_unstable_snapshots_and_tiny_budgets_terminate() {
    let mut io = Ports::bcd();
    io.bytes[10] = 0xa6;
    let mut clock = rtc(io, NmiState::Enabled);
    clock_fault(clock.read_utc().unwrap_err(), Fault::ReadBudgetExhausted);
    let io = clock.into_io();
    assert_eq!(io.reads, 512);
    assert_eq!(*io.selections.last().unwrap(), 15);
    let mut io = Ports::bcd();
    io.script(0, &(0..32).collect::<Vec<u8>>());
    let mut clock = rtc(io, NmiState::Enabled);
    clock_fault(clock.read_utc().unwrap_err(), Fault::Unstable);
    assert!(clock.into_io().reads <= 512);
    let mut configuration = config();
    configuration.register_reads = 1;
    let mut clock = Rtc::new(
        Ports::bcd(),
        IndexSelection::new(15, NmiState::Enabled).unwrap(),
        configuration,
    )
    .unwrap();
    clock_fault(clock.read_utc().unwrap_err(), Fault::ReadBudgetExhausted);
    assert_eq!(clock.into_io().reads, 1);
}

#[test]
fn missing_battery_stopped_divider_set_and_dst_never_make_a_timestamp() {
    for (register, value, fault) in [
        (13, 0xff, Fault::MissingRtc),
        (13, 0, Fault::BatteryInvalid),
        (13, 0x81, Fault::InvalidControl),
        (10, 0xff, Fault::MissingRtc),
        (10, 0x66, Fault::ClockStopped),
        (11, 0x82, Fault::ClockStopped),
        (11, 3, Fault::DaylightSavingEnabled),
    ] {
        let mut io = Ports::bcd();
        io.bytes[register] = value;
        let mut clock = rtc(io, NmiState::Masked);
        clock_fault(clock.read_utc().unwrap_err(), fault);
        assert_eq!(*clock.into_io().selections.last().unwrap(), 143);
    }
    let mut io = Ports::bcd();
    io.absent = true;
    assert_eq!(
        rtc(io, NmiState::Enabled).read_utc().unwrap_err(),
        ReadError {
            failure: Failure::Io(IoError::Offline),
            restore_error: None
        }
    );
}

#[test]
fn unknown_utc_basis_or_century_never_reads_or_guesses() {
    for configuration in [
        Configuration {
            basis: TimeBasis::Unknown,
            ..config()
        },
        Configuration {
            century: None,
            ..config()
        },
    ] {
        let mut clock = Rtc::new(
            Ports::bcd(),
            IndexSelection::new(15, NmiState::Enabled).unwrap(),
            configuration,
        )
        .unwrap();
        let error = if configuration.century.is_none() {
            Fault::CenturyUnavailable
        } else {
            Fault::TimeBasisUnknown
        };
        clock_fault(clock.read_utc().unwrap_err(), error);
        let io = clock.into_io();
        assert_eq!(io.reads, 0);
        assert!(io.selections.is_empty());
    }
    assert!(CenturyRegister::new(12).is_none());
    assert!(CenturyRegister::new(128).is_none());
    assert!(IndexSelection::new(128, NmiState::Enabled).is_none());
    for configuration in [
        Configuration {
            register_reads: 0,
            ..config()
        },
        Configuration {
            register_reads: MAX_REGISTER_READS + 1,
            ..config()
        },
        Configuration {
            attempts: 0,
            ..config()
        },
        Configuration {
            attempts: MAX_ATTEMPTS + 1,
            ..config()
        },
    ] {
        assert!(matches!(
            Rtc::new(
                Ports::bcd(),
                IndexSelection::new(15, NmiState::Enabled).unwrap(),
                configuration
            ),
            Err(Fault::Configuration)
        ));
    }
}

#[test]
fn rollback_and_caller_floor_are_visible_and_never_silently_reset() {
    let mut io = Ports::bcd();
    io.script(9, &[0x26, 0x26, 0x25, 0x25, 0x26, 0x26]);
    let mut clock = rtc(io, NmiState::Enabled);
    let first = clock.read_utc().unwrap();
    clock_fault(clock.read_utc().unwrap_err(), Fault::Rollback);
    assert_eq!(
        clock.read_utc().unwrap().rollback,
        RollbackCheck::NotBelow {
            floor: first.utc,
            source: FloorSource::EarlierRtcReading
        }
    );
    let minimum = DateUtc {
        year: 2030,
        ..expected()
    }
    .unix()
    .unwrap();
    let mut io = Ports::bcd();
    io.script(9, &[0x26, 0x26, 0x31, 0x31]);
    let mut clock = Rtc::new(
        io,
        IndexSelection::new(15, NmiState::Enabled).unwrap(),
        Configuration {
            minimum: Some(minimum),
            ..config()
        },
    )
    .unwrap();
    clock_fault(clock.read_utc().unwrap_err(), Fault::Rollback);
    let r = clock.read_utc().unwrap();
    assert_eq!(
        r.rollback,
        RollbackCheck::NotBelow {
            floor: minimum,
            source: FloorSource::CallerSupplied
        }
    );
    assert_eq!(r.quality, Quality::UnauthenticatedRtc);
}

#[test]
fn port_errors_restore_index_and_restore_failure_poison_prevents_more_io() {
    for kind in [IoError::Read, IoError::Select, IoError::Settle] {
        let mut io = Ports::bcd();
        match kind {
            IoError::Read => io.fail_read = true,
            IoError::Select => io.fail_select = true,
            _ => io.fail_settle = true,
        }
        let mut clock = rtc(io, NmiState::Masked);
        assert_eq!(
            clock.read_utc().unwrap_err(),
            ReadError {
                failure: Failure::Io(kind),
                restore_error: None
            }
        );
        assert!(!clock.is_poisoned());
        assert_eq!(*clock.into_io().selections.last().unwrap(), 143);
    }
    for read_fails in [false, true] {
        let mut io = Ports::bcd();
        io.fail_restore = true;
        io.fail_read = read_fails;
        let mut clock = rtc(io, NmiState::Enabled);
        assert_eq!(
            clock.read_utc().unwrap_err(),
            ReadError {
                failure: if read_fails {
                    Failure::Io(IoError::Read)
                } else {
                    Failure::Clock(Fault::RestoreFailed)
                },
                restore_error: Some(IoError::Restore)
            }
        );
        assert!(clock.is_poisoned());
        clock_fault(clock.read_utc().unwrap_err(), Fault::Poisoned);
        let io = clock.into_io();
        assert_eq!(io.selections.len(), if read_fails { 2 } else { 27 });
    }
}
