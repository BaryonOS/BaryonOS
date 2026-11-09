//! Versioned, fixed-capacity AP handoff data and ACPI MADT decoding.

use crate::{Error, PAGE_SIZE};

pub const MAX_CPUS: usize = 4;
pub const INFO_MAGIC: [u8; 8] = *b"NOSSMP01";
pub const INFO_VERSION: u16 = 1;
pub const MAX_MADT_LEN: usize = 1024 * 1024;
pub const SIPI_LIMIT: u64 = 1 << 20;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub magic: [u8; 8],
    pub version: u16,
    pub size: u16,
    pub cpu_count: u16,
    pub reserved: u16,
    pub bootstrap_apic_id: u32,
    pub local_apic_base: u64,
    pub trampoline_page: u64,
    pub apic_ids: [u32; MAX_CPUS],
}

const _: () = {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<Info>() == 56);
    assert!(align_of::<Info>() == 8);
    assert!(offset_of!(Info, bootstrap_apic_id) == 16);
    assert!(offset_of!(Info, local_apic_base) == 24);
    assert!(offset_of!(Info, trampoline_page) == 32);
    assert!(offset_of!(Info, apic_ids) == 40);
};

impl Info {
    pub fn validate(&self) -> Result<(), Error> {
        if self.magic != INFO_MAGIC
            || self.version != INFO_VERSION
            || self.size as usize != size_of::<Self>()
            || !(1..=MAX_CPUS as u16).contains(&self.cpu_count)
            || self.reserved != 0
            || self.local_apic_base == 0
            || !self.local_apic_base.is_multiple_of(PAGE_SIZE)
            || self.local_apic_base >= 1u64 << 52
            || !self.trampoline_page.is_multiple_of(PAGE_SIZE)
            || self.trampoline_page == 0
            || self.trampoline_page >= SIPI_LIMIT
        {
            return Err(Error::Header);
        }
        let ids = &self.apic_ids[..self.cpu_count as usize];
        if !ids.contains(&self.bootstrap_apic_id) {
            return Err(Error::SmpTopology);
        }
        for (index, id) in ids.iter().enumerate() {
            if ids[..index].contains(id) {
                return Err(Error::SmpTopology);
            }
        }
        if self.apic_ids[self.cpu_count as usize..]
            .iter()
            .any(|id| *id != 0)
        {
            return Err(Error::SmpTopology);
        }
        Ok(())
    }

    pub fn bootstrap_index(&self) -> Result<usize, Error> {
        self.validate()?;
        self.apic_ids[..self.cpu_count as usize]
            .iter()
            .position(|id| *id == self.bootstrap_apic_id)
            .ok_or(Error::SmpTopology)
    }
}

pub fn parse_madt(
    bytes: &[u8],
    bootstrap_apic_id: u32,
    trampoline_page: u64,
) -> Result<Info, Error> {
    if bytes.len() < 44 || bytes.len() > MAX_MADT_LEN || &bytes[..4] != b"APIC" {
        return Err(Error::SmpTopology);
    }
    let length = read_u32(bytes, 4)? as usize;
    if length != bytes.len() || checksum(bytes) != 0 {
        return Err(Error::SmpTopology);
    }
    let mut local_apic_base = u64::from(read_u32(bytes, 36)?);
    let mut apic_ids = [0u32; MAX_CPUS];
    let mut cpu_count = 0usize;
    let mut offset = 44usize;
    while offset < length {
        let record_type = *bytes.get(offset).ok_or(Error::SmpTopology)?;
        let record_len = usize::from(*bytes.get(offset + 1).ok_or(Error::SmpTopology)?);
        let end = offset.checked_add(record_len).ok_or(Error::SmpTopology)?;
        if record_len < 2 || end > length {
            return Err(Error::SmpTopology);
        }
        match record_type {
            0 => {
                if record_len < 8 {
                    return Err(Error::SmpTopology);
                }
                let flags = read_u32(bytes, offset + 4)?;
                if flags & 3 != 0 {
                    add_cpu(&mut apic_ids, &mut cpu_count, u32::from(bytes[offset + 3]))?;
                }
            }
            5 => {
                if record_len < 12 {
                    return Err(Error::SmpTopology);
                }
                local_apic_base = read_u64(bytes, offset + 4)?;
            }
            9 => {
                if record_len < 16 {
                    return Err(Error::SmpTopology);
                }
                let flags = read_u32(bytes, offset + 8)?;
                if flags & 3 != 0 {
                    add_cpu(&mut apic_ids, &mut cpu_count, read_u32(bytes, offset + 4)?)?;
                }
            }
            _ => {}
        }
        offset = end;
    }
    let info = Info {
        magic: INFO_MAGIC,
        version: INFO_VERSION,
        size: size_of::<Info>() as u16,
        cpu_count: u16::try_from(cpu_count).map_err(|_| Error::SmpTopology)?,
        reserved: 0,
        bootstrap_apic_id,
        local_apic_base,
        trampoline_page,
        apic_ids,
    };
    info.validate()?;
    Ok(info)
}

fn add_cpu(apic_ids: &mut [u32; MAX_CPUS], count: &mut usize, apic_id: u32) -> Result<(), Error> {
    if apic_ids[..*count].contains(&apic_id) || *count == MAX_CPUS {
        return Err(Error::SmpTopology);
    }
    apic_ids[*count] = apic_id;
    *count += 1;
    Ok(())
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let field: [u8; 4] = bytes
        .get(offset..offset.checked_add(4).ok_or(Error::SmpTopology)?)
        .ok_or(Error::SmpTopology)?
        .try_into()
        .map_err(|_| Error::SmpTopology)?;
    Ok(u32::from_le_bytes(field))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, Error> {
    let field: [u8; 8] = bytes
        .get(offset..offset.checked_add(8).ok_or(Error::SmpTopology)?)
        .ok_or(Error::SmpTopology)?
        .try_into()
        .map_err(|_| Error::SmpTopology)?;
    Ok(u64::from_le_bytes(field))
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::{vec, vec::Vec};

    fn madt(records: &[&[u8]], local_apic_base: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 44];
        bytes[..4].copy_from_slice(b"APIC");
        bytes[36..40].copy_from_slice(&local_apic_base.to_le_bytes());
        for record in records {
            bytes.extend_from_slice(record);
        }
        let length = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&length.to_le_bytes());
        let sum = bytes.iter().fold(0u8, |acc, byte| acc.wrapping_add(*byte));
        bytes[9] = 0u8.wrapping_sub(sum);
        bytes
    }

    fn local_apic(id: u8, flags: u32) -> [u8; 8] {
        let mut record = [0u8; 8];
        record[0] = 0;
        record[1] = 8;
        record[3] = id;
        record[4..8].copy_from_slice(&flags.to_le_bytes());
        record
    }

    fn x2apic(id: u32, flags: u32) -> [u8; 16] {
        let mut record = [0u8; 16];
        record[0] = 9;
        record[1] = 16;
        record[4..8].copy_from_slice(&id.to_le_bytes());
        record[8..12].copy_from_slice(&flags.to_le_bytes());
        record
    }

    #[test]
    fn madt_accepts_enabled_and_online_capable_processors() {
        let bsp = local_apic(0, 1);
        let disabled = local_apic(1, 0);
        let online_capable = x2apic(7, 2);
        let offline = x2apic(8, 0);
        let bytes = madt(&[&bsp, &disabled, &online_capable, &offline], 0xfee0_0000);
        let info = parse_madt(&bytes, 0, 0x8000).unwrap();
        assert_eq!(info.cpu_count, 2);
        assert_eq!(info.apic_ids[..2], [0, 7]);
        assert_eq!(info.bootstrap_index(), Ok(0));
    }

    #[test]
    fn madt_represents_a_valid_single_cpu_system() {
        let bytes = madt(&[&local_apic(0, 1)], 0xfee0_0000);
        let info = parse_madt(&bytes, 0, 0x8000).unwrap();
        assert_eq!(info.cpu_count, 1);
        assert_eq!(info.bootstrap_index(), Ok(0));
    }

    #[test]
    fn madt_accepts_address_override_and_sparse_x2apic_ids() {
        let mut override_record = [0u8; 12];
        override_record[0] = 5;
        override_record[1] = 12;
        override_record[4..12].copy_from_slice(&0xfee1_0000u64.to_le_bytes());
        let first = x2apic(2, 1);
        let bootstrap = x2apic(300, 1);
        let bytes = madt(&[&override_record, &first, &bootstrap], 0xfee0_0000);
        let info = parse_madt(&bytes, 300, 0x9000).unwrap();
        assert_eq!(info.local_apic_base, 0xfee1_0000);
        assert_eq!(info.apic_ids[..2], [2, 300]);
        assert_eq!(info.bootstrap_index(), Ok(1));
    }

    #[test]
    fn madt_rejects_bad_checksum_lengths_duplicates_missing_bsp_and_capacity_overflow() {
        let valid = madt(&[&local_apic(0, 1), &local_apic(1, 1)], 0xfee0_0000);
        let mut bad_checksum = valid.clone();
        bad_checksum[8] ^= 1;
        assert_eq!(
            parse_madt(&bad_checksum, 0, 0x8000),
            Err(Error::SmpTopology)
        );
        let mut truncated = valid.clone();
        truncated[45] = 0;
        assert_eq!(parse_madt(&truncated, 0, 0x8000), Err(Error::SmpTopology));
        let duplicate = madt(&[&local_apic(0, 1), &local_apic(0, 1)], 0xfee0_0000);
        assert_eq!(parse_madt(&duplicate, 0, 0x8000), Err(Error::SmpTopology));
        assert_eq!(parse_madt(&valid, 2, 0x8000), Err(Error::SmpTopology));
        let too_many = madt(
            &[
                &local_apic(0, 1),
                &local_apic(1, 1),
                &local_apic(2, 1),
                &local_apic(3, 1),
                &local_apic(4, 1),
            ],
            0xfee0_0000,
        );
        assert_eq!(parse_madt(&too_many, 0, 0x8000), Err(Error::SmpTopology));
    }

    #[test]
    fn info_rejects_invalid_sipi_pages_and_duplicate_ids() {
        let info = parse_madt(
            &madt(&[&local_apic(0, 1), &local_apic(1, 1)], 0xfee0_0000),
            0,
            0x8000,
        )
        .unwrap();
        let mut invalid = info;
        invalid.trampoline_page = SIPI_LIMIT;
        assert_eq!(invalid.validate(), Err(Error::Header));
        let mut duplicate = info;
        duplicate.apic_ids[1] = 0;
        assert_eq!(duplicate.validate(), Err(Error::SmpTopology));
    }
}
