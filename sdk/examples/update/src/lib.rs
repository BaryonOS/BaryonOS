#![no_std]
#[cfg(test)]
mod install;
#[cfg(test)]
mod tests {
    extern crate std;
    use super::install::{install, Storage};
    use std::{collections::BTreeMap, vec, vec::Vec};
    use update_core::{
        image_sector, manifest_for, manifest_sector, Header, Reject, SigningKeys, INBOX_IMAGE,
        MANIFEST_LEN, SECTOR,
    };

    struct FakeDisk {
        sectors: BTreeMap<u32, [u8; SECTOR]>,
        events: Vec<(char, u32)>,
        corrupt_readback: bool,
        flushed: bool,
    }
    impl Storage for FakeDisk {
        fn read(&mut self, sector: u32, buffer: &mut [u8; SECTOR]) -> Result<(), i64> {
            *buffer = *self.sectors.get(&sector).unwrap_or(&[0; SECTOR]);
            if self.corrupt_readback && self.flushed && sector == image_sector(1) {
                buffer[0] ^= 1;
            }
            self.events.push(('r', sector));
            Ok(())
        }
        fn write(&mut self, sector: u32, buffer: &[u8; SECTOR]) -> Result<(), i64> {
            self.sectors.insert(sector, *buffer);
            self.events.push(('w', sector));
            Ok(())
        }
        fn flush(&mut self) -> Result<(), i64> {
            self.flushed = true;
            self.events.push(('f', 0));
            Ok(())
        }
        fn write_manifest(&mut self, sector: u32, _: &[u8; MANIFEST_LEN]) -> Result<(), i64> {
            self.events.push(('m', sector));
            Ok(())
        }
        fn mark(&mut self, _: u64, _: u64) {}
    }
    fn run(corrupt: bool) -> (Result<Result<(), Reject>, i64>, FakeDisk) {
        let image = vec![42u8; SECTOR + 3];
        let keys = SigningKeys::from_seeds(&[7; 32], &[9; 32]);
        let manifest = manifest_for(&image, 2, 0, &keys.anchors()).unwrap();
        let mut disk = FakeDisk {
            sectors: BTreeMap::new(),
            events: Vec::new(),
            corrupt_readback: corrupt,
            flushed: false,
        };
        for (i, chunk) in image.chunks(SECTOR).enumerate() {
            let mut data = [0; SECTOR];
            data[..chunk.len()].copy_from_slice(chunk);
            disk.sectors.insert(INBOX_IMAGE + i as u32, data);
        }
        let result = install(
            &mut disk,
            &Header {
                active: 0,
                generation: 7,
            },
            &[0; MANIFEST_LEN],
            &manifest,
            1,
        );
        (result, disk)
    }
    #[test]
    fn readback_mismatch_does_not_publish() {
        let (result, disk) = run(true);
        assert_eq!(result, Ok(Err(Reject::Digest)));
        assert!(disk.events.contains(&('w', image_sector(1))));
        assert!(!disk
            .events
            .iter()
            .any(|event| *event == ('m', manifest_sector(1)) || *event == ('w', 0)));
        assert_eq!(disk.events.iter().filter(|event| event.0 == 'f').count(), 1);
    }
    #[test]
    fn successful_readback_publishes_manifest_then_header() {
        let (result, disk) = run(false);
        assert_eq!(result, Ok(Ok(())));
        let manifest = disk
            .events
            .iter()
            .position(|event| *event == ('m', manifest_sector(1)))
            .unwrap();
        let header = disk
            .events
            .iter()
            .position(|event| *event == ('w', 0))
            .unwrap();
        assert!(manifest < header);
        assert!(disk.events[..manifest].contains(&('r', image_sector(1))));
        assert_eq!(disk.events.iter().filter(|event| event.0 == 'f').count(), 3);
    }
}
