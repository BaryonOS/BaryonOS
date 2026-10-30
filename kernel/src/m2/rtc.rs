//! Exclusive read-only CMOS acceptance in the single-CPU IF0 bootstrap.
#[cfg(feature = "native-rtc")]
use crate::serial;
use core::{arch::asm, convert::Infallible};
use nieuw_os_rtc::{
    CenturyRegister, CmosIo, Configuration, IndexSelection, NmiState, Rtc, TimeBasis,
};
#[cfg(feature = "native-rtc")]
use nieuw_os_rtc::{Failure, Fault, FloorSource, Quality, RollbackCheck, Source, UnixUtc};

struct Ports {
    selected: IndexSelection,
    reads: u32,
    selections: u32,
}

impl CmosIo for Ports {
    type Error = Infallible;

    fn select(&mut self, selection: IndexSelection) -> Result<(), Infallible> {
        // SAFETY: Own IDT is installed; one CPU, IF=0, no firmware/runtime or
        // NMI handler accesses CMOS. Only the index port is writable. The
        // explicit platform NMI policy is Enabled, never inferred from 0x70.
        unsafe {
            asm!("out dx, al", in("dx") 0x70u16, in("al") selection.encoded(),
                 options(nomem, nostack, preserves_flags));
        }
        self.selected = selection;
        self.selections += 1;
        Ok(())
    }

    fn settle(&mut self) -> Result<(), Infallible> {
        // QEMU's index write is synchronous; a bounded scalar delay does not
        // touch port 0x80, acknowledge register C, or write any CMOS data.
        for _ in 0..16 {
            core::hint::spin_loop();
        }
        Ok(())
    }

    fn read_data(&mut self) -> Result<u8, Infallible> {
        let value: u8;
        // SAFETY: Same exclusive CPL0 lifetime as select. Reads only the CMOS
        // data port. No memory, DMA, user pointer or other I/O is accessed.
        unsafe {
            asm!("in al, dx", in("dx") 0x71u16, out("al") value,
                 options(nomem, nostack, preserves_flags));
        }
        self.reads += 1;
        Ok(value)
    }
}

fn configuration() -> Configuration {
    Configuration {
        century: CenturyRegister::new(0x32),
        basis: TimeBasis::ConfiguredUtc,
        register_reads: 512,
        attempts: 8,
        minimum: None,
    }
}

fn restored(mut ports: Ports, initial: IndexSelection) -> Result<Ports, ()> {
    if ports.selected != initial || ports.read_data() != Ok(0x80) {
        return Err(());
    }
    Ok(ports)
}

fn interrupts_disabled() -> bool {
    let flags: u64;
    // SAFETY: Reads RFLAGS on the current mapped kernel stack; no state changes.
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(preserves_flags));
    }
    flags & (1 << 9) == 0
}

#[cfg(feature = "native-rtc")]
fn number(value: i64) {
    let mut digits = [0u8; 20];
    let mut n = value.unsigned_abs();
    let mut at = digits.len();
    loop {
        at -= 1;
        digits[at] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if value < 0 {
        serial::write(b"-");
    }
    serial::write(&digits[at..]);
}

#[cfg(feature = "native-system-clock")]
pub(super) fn read_current_utc() -> Result<i64, ()> {
    if !interrupts_disabled() {
        return Err(());
    }
    let initial = IndexSelection::new(13, NmiState::Enabled).ok_or(())?;
    let mut ports = Ports {
        selected: initial,
        reads: 0,
        selections: 0,
    };
    ports.select(initial).map_err(|_| ())?;
    ports.settle().map_err(|_| ())?;
    let mut clock = Rtc::new(ports, initial, configuration()).map_err(|_| ())?;
    let reading = clock.read_utc();
    let restored = restored(clock.into_io(), initial);
    match (reading, restored) {
        (Ok(reading), Ok(_)) => Ok(reading.utc.seconds()),
        _ => Err(()),
    }
}

#[cfg(feature = "native-rtc")]
pub fn run() {
    assert!(interrupts_disabled());
    let initial = IndexSelection::new(13, NmiState::Enabled).unwrap();
    let mut ports = Ports {
        selected: initial,
        reads: 0,
        selections: 0,
    };
    // Take platform ownership explicitly rather than guessing firmware's index
    // or NMI shadow. No CMOS content or RTC mode is changed.
    ports.select(initial).unwrap();
    ports.settle().unwrap();
    let mut clock = Rtc::new(ports, initial, configuration()).unwrap();
    let first = clock.read_utc().expect("native RTC");
    assert_eq!(first.source, Source::CmosRtc);
    assert_eq!(first.quality, Quality::UnauthenticatedRtc);
    assert_eq!(first.rollback, RollbackCheck::Unchecked);
    assert_eq!(first.date.unix(), Ok(first.utc));
    let second = clock.read_utc().expect("second native RTC");
    assert!(second.utc >= first.utc);
    assert_eq!(
        second.rollback,
        RollbackCheck::NotBelow {
            floor: first.utc,
            source: FloorSource::EarlierRtcReading,
        }
    );
    serial::write(b"RTC:UTC:");
    number(first.utc.seconds());
    serial::write(b"\nRTC:DATE:");
    for (i, value) in [
        i64::from(first.date.year),
        i64::from(first.date.month),
        i64::from(first.date.day),
        i64::from(first.date.hour),
        i64::from(first.date.minute),
        i64::from(first.date.second),
    ]
    .into_iter()
    .enumerate()
    {
        if i != 0 {
            serial::write(b",");
        }
        number(value);
    }
    serial::write(b"\nRTC:QUALITY:UNAUTHENTICATED\n");
    let mut ports = restored(clock.into_io(), initial).expect("RTC index restored");

    for (config, fault) in [
        (
            Configuration {
                register_reads: 1,
                ..configuration()
            },
            Fault::ReadBudgetExhausted,
        ),
        (
            Configuration {
                minimum: UnixUtc::from_seconds(UnixUtc::MAX),
                ..configuration()
            },
            Fault::Rollback,
        ),
        (
            Configuration {
                basis: TimeBasis::Unknown,
                ..configuration()
            },
            Fault::TimeBasisUnknown,
        ),
        (
            Configuration {
                century: None,
                ..configuration()
            },
            Fault::CenturyUnavailable,
        ),
    ] {
        let before = (ports.reads, ports.selections);
        let mut clock = Rtc::new(ports, initial, config).unwrap();
        let error = clock.read_utc().unwrap_err();
        assert_eq!(error.failure, Failure::Clock(fault));
        assert_eq!(error.restore_error, None);
        ports = clock.into_io();
        if matches!(fault, Fault::TimeBasisUnknown | Fault::CenturyUnavailable) {
            assert_eq!((ports.reads, ports.selections), before);
        } else {
            if fault == Fault::ReadBudgetExhausted {
                assert_eq!(ports.reads - before.0, 1);
            }
            ports = restored(ports, initial).expect("RTC index restored after error");
        }
    }
    serial::write(b"RTC:NEGATIVE:PASS\n");
    let mut clock = Rtc::new(ports, initial, configuration()).unwrap();
    assert!(clock.read_utc().expect("read after errors").utc >= second.utc);
    let _ = restored(clock.into_io(), initial).expect("RTC index restored after read");
    serial::write(b"RTC:RESTORE:PASS\nRTC:PASS\n");
}
