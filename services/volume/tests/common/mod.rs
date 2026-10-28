#![allow(dead_code)]
//! Host-only simulated block device with volatile write cache, flush barrier,
//! fault injection and crash images. It models the guarantees the service
//! relies on: a completed write is visible to later reads, but only flushed
//! writes are durable; an unflushed suffix may be lost, kept partially, or
//! leave one torn sector.
use volume_core::{
    client::Transport, service::Service, table, wire, Disk, Status, SECTOR, VOLUME_SECTORS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Read,
    Write,
    Flush,
}

#[derive(Clone)]
pub struct SimDisk {
    pub durable: Vec<[u8; SECTOR]>,
    pub pending: Vec<(u64, [u8; SECTOR])>,
    pub ops: usize,
    pub mutating_ops: usize,
    /// Fail the n-th mutating operation (write or flush) and every later op.
    pub crash_at: Option<usize>,
    pub crashed: bool,
    /// Fail exactly the n-th mutating operation, then keep working.
    pub fail_once_at: Option<usize>,
    pub fail_reads_from: Option<u64>,
    pub log: Vec<(Op, u64)>,
}

impl SimDisk {
    pub fn new() -> Self {
        Self::from_image(vec![[0; SECTOR]; VOLUME_SECTORS as usize])
    }
    pub fn from_image(durable: Vec<[u8; SECTOR]>) -> Self {
        assert_eq!(durable.len(), VOLUME_SECTORS as usize);
        Self {
            durable,
            pending: Vec::new(),
            ops: 0,
            mutating_ops: 0,
            crash_at: None,
            crashed: false,
            fail_once_at: None,
            fail_reads_from: None,
            log: Vec::new(),
        }
    }
    fn current(&self, sector: u64) -> [u8; SECTOR] {
        self.pending
            .iter()
            .rev()
            .find(|(s, _)| *s == sector)
            .map(|(_, d)| *d)
            .unwrap_or(self.durable[sector as usize])
    }
    fn mutating(&mut self) -> Result<(), Status> {
        if self.crashed {
            return Err(Status::Io);
        }
        let index = self.mutating_ops;
        self.mutating_ops += 1;
        if self.crash_at == Some(index) {
            self.crashed = true;
            return Err(Status::Io);
        }
        if self.fail_once_at == Some(index) {
            return Err(Status::Io);
        }
        Ok(())
    }
    /// Durable image after a crash, keeping `keep` of the unflushed writes in
    /// submission order and optionally tearing the next one after `tear` bytes.
    pub fn crash_image(&self, keep: usize, tear: Option<usize>) -> Vec<[u8; SECTOR]> {
        let mut image = self.durable.clone();
        for (sector, data) in self.pending.iter().take(keep) {
            image[*sector as usize] = *data;
        }
        if let (Some(bytes), Some((sector, data))) = (tear, self.pending.get(keep)) {
            image[*sector as usize][..bytes].copy_from_slice(&data[..bytes]);
        }
        image
    }
    /// Durable image when the device persisted an arbitrary, possibly
    /// reordered subset of the unflushed writes.
    pub fn crash_subset(&self, keep: impl Fn(usize) -> bool) -> Vec<[u8; SECTOR]> {
        let mut image = self.durable.clone();
        for (index, (sector, data)) in self.pending.iter().enumerate() {
            if keep(index) {
                image[*sector as usize] = *data;
            }
        }
        image
    }
    /// The durable image if the machine stopped right now (flushed + all
    /// completed writes; QEMU keeps completed writes in this harness).
    pub fn settled(&self) -> Vec<[u8; SECTOR]> {
        self.crash_image(self.pending.len(), None)
    }
}

impl Disk for SimDisk {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        assert!(sector < VOLUME_SECTORS, "read outside scope");
        if self.crashed {
            return Err(Status::Io);
        }
        if self.fail_reads_from.is_some_and(|from| sector >= from) {
            return Err(Status::Io);
        }
        self.ops += 1;
        self.log.push((Op::Read, sector));
        *buffer = self.current(sector);
        Ok(())
    }
    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        assert!(sector < VOLUME_SECTORS, "write outside scope");
        self.mutating()?;
        self.ops += 1;
        self.log.push((Op::Write, sector));
        self.pending.push((sector, *buffer));
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        self.mutating()?;
        self.ops += 1;
        self.log.push((Op::Flush, 0));
        for (sector, data) in self.pending.drain(..) {
            self.durable[sector as usize] = data;
        }
        Ok(())
    }
}

pub struct Local<'a> {
    pub service: &'a mut Service,
    pub disk: &'a mut SimDisk,
    pub domain: usize,
}

impl Transport for Local<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        Ok(self.service.handle(self.domain, request, self.disk))
    }
}

pub fn pattern(object: u64, revision: u64, offset: usize) -> u8 {
    (object as usize * 31 + revision as usize * 7 + offset * 13 + offset / 251) as u8
}

pub fn fill(object: u64, revision: u64) -> impl FnMut(usize, &mut [u8]) {
    move |offset, buffer| {
        for (index, byte) in buffer.iter_mut().enumerate() {
            *byte = pattern(object, revision, offset + index);
        }
    }
}

pub fn recover(image: Vec<[u8; SECTOR]>) -> Result<table::Table, Status> {
    let mut disk = SimDisk::from_image(image);
    table::recover(&mut disk)
}
