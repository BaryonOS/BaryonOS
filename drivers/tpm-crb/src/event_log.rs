//! Bounded parser for the TCG2 crypto-agile firmware event log
//! (`EFI_TCG2_EVENT_LOG_FORMAT_TCG_2`, TCG PC Client Platform Firmware Profile):
//! a SHA-1-format `TCG_PCR_EVENT` carrying `Spec ID Event03`, then packed
//! little-endian `TCG_PCR_EVENT2` entries. The caller passes firmware memory as
//! slices; nothing is hashed here, digests are returned for the caller to check.

/// `EV_NO_ACTION`: the type of the `Spec ID Event03` header event.
pub const EV_NO_ACTION: u32 = 0x03;
const SPEC_ID: &[u8; 16] = b"Spec ID Event03\0";
/// Bank algorithms one header may list (SHA-1, SHA-2, SM3 and SHA-3 banks).
pub const MAX_ALGORITHMS: usize = 8;
/// Largest digest a header may declare (SHA-512).
pub const MAX_DIGEST: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A field runs past its slice: an entry crosses `EventLogLastEntry`, or a
    /// length is larger than the bytes that follow it.
    Truncated = 1,
    /// The first event is not a well-formed `Spec ID Event03` header.
    Header = 2,
    /// An entry names an algorithm the header does not list, or too many.
    Algorithm = 3,
    /// An entry before the last one names the expected PCR.
    Earlier = 4,
    /// The last entry is not exactly the expected event.
    Event = 5,
}

/// The header's algorithm table, after a whole prefix walk succeeded.
#[derive(Clone, Copy, Debug)]
pub struct Log {
    count: usize,
    algorithms: [(u16, u16); MAX_ALGORITHMS],
}

/// The digests of one entry, in log order.
#[derive(Clone, Copy, Debug)]
pub struct Digests<'a> {
    count: usize,
    items: [(u16, &'a [u8]); MAX_ALGORITHMS],
}

impl<'a> Digests<'a> {
    pub fn iter(&self) -> impl Iterator<Item = (u16, &'a [u8])> + '_ {
        self.items[..self.count].iter().copied()
    }

    pub fn get(&self, algorithm: u16) -> Option<&'a [u8]> {
        self.iter()
            .find(|(id, _)| *id == algorithm)
            .map(|(_, digest)| digest)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.at.checked_add(len).ok_or(Error::Truncated)?;
        let bytes = self.bytes.get(self.at..end).ok_or(Error::Truncated)?;
        self.at = end;
        Ok(bytes)
    }

    fn u16(&mut self) -> Result<u16, Error> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn len(&mut self) -> Result<usize, Error> {
        usize::try_from(self.u32()?).map_err(|_| Error::Truncated)
    }
}

impl Log {
    /// Walks `before` = [`EventLogLocation`, `EventLogLastEntry`): the header,
    /// then whole entries that end exactly at the slice end, none for `pcr`.
    pub fn parse(before: &[u8], pcr: u32) -> Result<Self, Error> {
        let mut reader = Reader {
            bytes: before,
            at: 0,
        };
        let log = Self::header(&mut reader)?;
        while reader.at < before.len() {
            if log.entry(&mut reader)?.0 == pcr {
                return Err(Error::Earlier);
            }
        }
        Ok(log)
    }

    fn header(reader: &mut Reader) -> Result<Self, Error> {
        // TCG_PCR_EVENT: PCRIndex 0, EV_NO_ACTION and an all-zero SHA-1 digest.
        if reader.u32()? != 0
            || reader.u32()? != EV_NO_ACTION
            || reader.take(20)?.iter().any(|byte| *byte != 0)
        {
            return Err(Error::Header);
        }
        let size = reader.len()?;
        let mut spec = Reader {
            bytes: reader.take(size)?,
            at: 0,
        };
        if spec.take(16)? != SPEC_ID {
            return Err(Error::Header);
        }
        // platformClass, specVersionMinor, specVersionMajor, specErrata, uintnSize.
        spec.take(8)?;
        let count = spec.len()?;
        if !(1..=MAX_ALGORITHMS).contains(&count) {
            return Err(Error::Header);
        }
        let mut algorithms = [(0u16, 0u16); MAX_ALGORITHMS];
        for at in 0..count {
            let (id, size) = (spec.u16()?, spec.u16()?);
            if size == 0
                || usize::from(size) > MAX_DIGEST
                || algorithms[..at].iter().any(|(seen, _)| *seen == id)
            {
                return Err(Error::Header);
            }
            algorithms[at] = (id, size);
        }
        let vendor = spec.take(1)?[0];
        spec.take(usize::from(vendor))?;
        if spec.at != size {
            return Err(Error::Header);
        }
        Ok(Self { count, algorithms })
    }

    fn size(&self, algorithm: u16) -> Option<usize> {
        self.algorithms[..self.count]
            .iter()
            .find(|(id, _)| *id == algorithm)
            .map(|(_, size)| usize::from(*size))
    }

    /// One `TCG_PCR_EVENT2`: PCR, event type, digests and event data.
    #[allow(clippy::type_complexity)]
    fn entry<'a>(
        &self,
        reader: &mut Reader<'a>,
    ) -> Result<(u32, u32, Digests<'a>, &'a [u8]), Error> {
        let (pcr, kind) = (reader.u32()?, reader.u32()?);
        let count = reader.len()?;
        if count > MAX_ALGORITHMS {
            return Err(Error::Algorithm);
        }
        let mut digests = Digests {
            count,
            items: [(0, &[][..]); MAX_ALGORITHMS],
        };
        for item in &mut digests.items[..count] {
            let algorithm = reader.u16()?;
            let size = self.size(algorithm).ok_or(Error::Algorithm)?;
            *item = (algorithm, reader.take(size)?);
        }
        let size = reader.len()?;
        Ok((pcr, kind, digests, reader.take(size)?))
    }

    /// Exact length of an entry with one digest per header algorithm and
    /// `data` bytes of event data: the bytes to read at `EventLogLastEntry`.
    pub fn entry_len(&self, data: usize) -> Option<usize> {
        let digests: usize = self.algorithms[..self.count]
            .iter()
            .map(|(_, size)| 2 + usize::from(*size))
            .sum();
        // PCRIndex, EventType, digest count and EventSize: four u32 fields.
        digests.checked_add(16)?.checked_add(data)
    }

    /// `last` must be exactly one entry for `pcr` of type `kind` with event
    /// data `data` and one digest per header algorithm, in header order.
    pub fn last_event<'a>(
        &self,
        last: &'a [u8],
        pcr: u32,
        kind: u32,
        data: &[u8],
    ) -> Result<Digests<'a>, Error> {
        let mut reader = Reader { bytes: last, at: 0 };
        let (found, found_kind, digests, event) = self.entry(&mut reader)?;
        let banks = digests.count == self.count
            && digests
                .iter()
                .zip(&self.algorithms[..self.count])
                .all(|((id, _), (header, _))| id == *header);
        if found != pcr || found_kind != kind || !banks || event != data || reader.at != last.len()
        {
            return Err(Error::Event);
        }
        Ok(digests)
    }
}
