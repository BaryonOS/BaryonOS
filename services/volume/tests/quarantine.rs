//! The kernel's post-wake lane quarantine (-24, H05-T5c) reaches clients as
//! the distinct, terminal `Quarantined`, not as Io or Protocol, on every
//! certain-abort path. A path whose outcome is unknowable stays `Unknown`.
mod common;

use common::{fill, SimDisk};
use volume_core::{
    client::{Client, Transport},
    service::Service,
    wire, Disk, Status, SECTOR,
};

const EPOCH: u64 = 7;

/// The kernel broker after quarantine: every block request from the n-th
/// mutating one (or from the start) is refused with -24.
struct Lane {
    disk: SimDisk,
    mutating: usize,
    from: Option<usize>,
    reads_refused: bool,
    refused: usize,
}

impl Lane {
    fn refuse(&mut self, mutating: bool) -> Result<(), Status> {
        let index = self.mutating;
        if mutating {
            self.mutating += 1;
        }
        let refused = if mutating {
            self.from.is_some_and(|from| index >= from)
        } else {
            self.reads_refused
        };
        if refused {
            self.refused += 1;
            return Err(Status::from_code(-24));
        }
        Ok(())
    }
    fn quarantine(&mut self) {
        self.from = Some(0);
        self.reads_refused = true;
    }
}

impl Disk for Lane {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        self.refuse(false)?;
        self.disk.read(sector, buffer)
    }
    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        self.refuse(true)?;
        self.disk.write(sector, buffer)
    }
    fn flush(&mut self) -> Result<(), Status> {
        self.refuse(true)?;
        self.disk.flush()
    }
}

struct Local<'a> {
    service: &'a mut Service,
    lane: &'a mut Lane,
}

impl Transport for Local<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        Ok(self.service.handle(0, request, self.lane))
    }
}

fn lane() -> Lane {
    Lane {
        disk: SimDisk::new(),
        mutating: 0,
        from: None,
        reads_refused: false,
        refused: 0,
    }
}

#[test]
fn status_code_is_distinct() {
    assert_eq!(Status::from_code(-24), Status::Quarantined);
    assert_eq!(Status::Quarantined as i64, -24);
    assert_eq!(Status::from_code(-13), Status::Timeout);
    assert_eq!(Status::from_code(-14), Status::NoDevice);
    // Neighbouring unassigned codes stay Protocol (fail closed).
    for code in [-22, -23, -25, -26] {
        assert_eq!(Status::from_code(code), Status::Protocol, "{code}");
    }
}

#[test]
fn recovery_on_a_quarantined_lane_fails_quarantined() {
    let mut lane = lane();
    lane.quarantine();
    assert_eq!(
        Service::recover(EPOCH, true, &mut lane).err(),
        Some(Status::Quarantined)
    );
    assert_eq!(lane.disk.ops, 0, "no device access reached the disk");
}

#[test]
fn certain_aborts_report_quarantined_not_io() {
    let mut lane = lane();
    let mut service = Service::recover(EPOCH, false, &mut lane).unwrap();
    let length = 3 * SECTOR + 17;
    {
        let mut client = Client::new(
            Local {
                service: &mut service,
                lane: &mut lane,
            },
            EPOCH,
            1,
        );
        client.write(0, 0, b"a.txt", length, fill(0, 1)).unwrap();
    }
    // A fresh instance has no read cache, so READ reaches the device.
    let mut service = Service::recover(EPOCH, false, &mut lane).unwrap();
    lane.quarantine();
    let durable = lane.disk.durable.clone();
    let mut client = Client::new(
        Local {
            service: &mut service,
            lane: &mut lane,
        },
        EPOCH,
        1,
    );
    // READ: the data path refused by the kernel.
    assert_eq!(
        client.read(0, 1, length, |_, _| {}).err(),
        Some(Status::Quarantined)
    );
    // APPEND of full sectors, then the staged write and flush of COMMIT.
    assert_eq!(
        client.write(1, 0, b"b.txt", 2 * SECTOR, fill(1, 1)).err(),
        Some(Status::Quarantined)
    );
    assert_eq!(
        client.write(2, 0, b"c.txt", 9, fill(2, 1)).err(),
        Some(Status::Quarantined)
    );
    // Terminal for the request, but no table write: STAT still answers from
    // the recovered table and nothing durable changed.
    assert_eq!(client.stat(0).unwrap().revision, 1);
    assert!(!service.uncertain());
    // One READ, one APPEND sector write, one COMMIT partial-sector write.
    assert_eq!(lane.refused, 3);
    assert_eq!(lane.disk.durable, durable);
}

#[test]
fn an_unknowable_table_write_stays_unknown() {
    let mut lane = lane();
    let mut service = Service::recover(EPOCH, false, &mut lane).unwrap();
    let mut write = |lane: &mut Lane, object| {
        Client::new(
            Local {
                service: &mut service,
                lane,
            },
            EPOCH,
            object + 1,
        )
        .write(object, 0, b"a.txt", 9, fill(object, 1))
    };
    // The first upload also writes the format marker.
    write(&mut lane, 0).unwrap();
    // A 9-byte upload: COMMIT writes one partial sector and flushes (both
    // certain-abort), then the table copy and its flush (outcome unknowable).
    lane.from = Some(lane.mutating + 2);
    assert_eq!(write(&mut lane, 1).err(), Some(Status::Unknown));
    assert_eq!(lane.refused, 1);
    assert!(service.uncertain());
}
