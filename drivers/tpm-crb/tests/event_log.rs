//! REL-05: host tests for the TCG2 crypto-agile event log parser. Synthetic
//! logs follow the PC Client PFP layout; native firmware logs are a separate
//! run (`tools/run_tpm.py`).
use nieuw_os_tpm_crb::event_log::{Error, Log, EV_NO_ACTION, MAX_ALGORITHMS};

#[allow(dead_code)]
#[path = "../../../crates/update-core/src/trial_tpm.rs"]
mod trial_tpm;

const EV_IPL: u32 = 0x0d;
const EV_SEPARATOR: u32 = 0x04;
const BANKS: [(u16, u16); 4] = [(0x0004, 20), (0x000b, 32), (0x000c, 48), (0x000d, 64)];

fn header(algorithms: &[(u16, u16)]) -> Vec<u8> {
    let mut spec = b"Spec ID Event03\0".to_vec();
    spec.extend([0, 0, 0, 0, 0, 2, 0, 2]);
    spec.extend((algorithms.len() as u32).to_le_bytes());
    for (id, size) in algorithms {
        spec.extend(id.to_le_bytes());
        spec.extend(size.to_le_bytes());
    }
    spec.push(0);
    let mut out = [0u32.to_le_bytes(), EV_NO_ACTION.to_le_bytes()].concat();
    out.extend([0; 20]);
    out.extend((spec.len() as u32).to_le_bytes());
    out.extend(spec);
    out
}

/// One `TCG_PCR_EVENT2`; digest bytes are the algorithm's low byte.
fn entry(pcr: u32, kind: u32, algorithms: &[(u16, u16)], data: &[u8]) -> Vec<u8> {
    let mut out = [pcr.to_le_bytes(), kind.to_le_bytes()].concat();
    out.extend((algorithms.len() as u32).to_le_bytes());
    for (id, size) in algorithms {
        out.extend(id.to_le_bytes());
        out.extend(vec![*id as u8; usize::from(*size)]);
    }
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    out
}

fn closed() -> Vec<u8> {
    entry(13, EV_IPL, &BANKS, trial_tpm::EVENT_CLOSED)
}

/// Header plus two ordinary firmware entries before the loader's own one.
fn before() -> Vec<u8> {
    let mut log = header(&BANKS);
    log.extend(entry(0, EV_SEPARATOR, &BANKS, &[0; 4]));
    log.extend(entry(7, EV_NO_ACTION, &BANKS[..1], b"vendor"));
    log
}

fn last(log: &Log, entry: &[u8]) -> Result<Vec<(u16, Vec<u8>)>, Error> {
    let digests = log.last_event(entry, 13, EV_IPL, trial_tpm::EVENT_CLOSED)?;
    Ok(digests
        .iter()
        .map(|(id, digest)| (id, digest.to_vec()))
        .collect())
}

#[test]
fn the_last_entry_is_the_only_pcr13_event_with_every_bank() {
    for prefix in [before(), header(&BANKS)] {
        let log = Log::parse(&prefix, 13).unwrap();
        let entry = closed();
        assert_eq!(
            log.entry_len(trial_tpm::EVENT_CLOSED.len()),
            Some(entry.len())
        );
        let digests = last(&log, &entry).unwrap();
        let expected: Vec<_> = BANKS
            .iter()
            .map(|(id, size)| (*id, vec![*id as u8; usize::from(*size)]))
            .collect();
        assert_eq!(digests, expected);
    }
    let log = Log::parse(&before(), 13).unwrap();
    let entry = closed();
    let digests = log
        .last_event(&entry, 13, EV_IPL, trial_tpm::EVENT_CLOSED)
        .unwrap();
    assert_eq!(digests.get(0x000b), Some(&[0x0b; 32][..]));
    assert_eq!(digests.get(0x0012), None);
}

#[test]
fn the_prefix_must_end_on_an_entry_boundary_without_pcr13() {
    let mut earlier = before();
    earlier.extend(entry(13, EV_NO_ACTION, &BANKS, b"x"));
    let mut crossing = before();
    crossing.extend(&closed()[..10]);
    let mut overflow = before();
    overflow.extend(entry(1, EV_SEPARATOR, &BANKS, &[]));
    let at = overflow.len() - 4;
    overflow[at..].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut unknown = before();
    unknown.extend(entry(1, EV_SEPARATOR, &[(0x0012, 32)], &[]));
    let mut many = before();
    many.extend(entry(1, EV_SEPARATOR, &BANKS, &[]));
    let at = many.len()
        - 4
        - BANKS
            .iter()
            .map(|(_, s)| 2 + usize::from(*s))
            .sum::<usize>()
        - 4;
    many[at..at + 4].copy_from_slice(&(MAX_ALGORITHMS as u32 + 1).to_le_bytes());
    let short = before()[..before().len() - 1].to_vec();
    for (prefix, error) in [
        (earlier, Error::Earlier),
        (crossing, Error::Truncated),
        (overflow, Error::Truncated),
        (unknown, Error::Algorithm),
        (many, Error::Algorithm),
        (short, Error::Truncated),
        (Vec::new(), Error::Truncated),
    ] {
        assert_eq!(Log::parse(&prefix, 13).unwrap_err(), error);
    }
}

#[test]
fn only_a_spec_id_event03_header_is_accepted() {
    let good = header(&BANKS);
    let mut cases = Vec::new();
    for (at, value) in [(0, 1), (4, 1), (8, 1), (32, b'2')] {
        let mut bad = good.clone();
        bad[at] = value;
        cases.push(bad);
    }
    let algorithms = 32 + 16 + 8;
    let mut zero = good.clone();
    zero[algorithms..algorithms + 4].copy_from_slice(&0u32.to_le_bytes());
    cases.push(zero);
    cases.push(header(&[(0x0004, 0)]));
    cases.push(header(&[(0x0004, 65)]));
    cases.push(header(&[(0x000b, 32), (0x000b, 32)]));
    cases.push(header(&[(0x0004, 20); MAX_ALGORITHMS + 1]));
    // vendorInfoSize larger than the remaining spec event, and a trailing byte.
    let mut vendor = good.clone();
    let last = vendor.len() - 1;
    vendor[last] = 1;
    let mut trailing = good.clone();
    trailing.push(0);
    let at = 28;
    let size = u32::from_le_bytes(trailing[at..at + 4].try_into().unwrap()) + 1;
    trailing[at..at + 4].copy_from_slice(&size.to_le_bytes());
    for (index, bad) in cases.iter().enumerate() {
        assert_eq!(
            Log::parse(bad, 13).unwrap_err(),
            Error::Header,
            "case {index}"
        );
    }
    assert_eq!(Log::parse(&vendor, 13).unwrap_err(), Error::Truncated);
    assert_eq!(Log::parse(&trailing, 13).unwrap_err(), Error::Header);
}

#[test]
fn the_last_entry_must_be_exactly_the_closed_event() {
    let log = Log::parse(&before(), 13).unwrap();
    let mut reordered = BANKS;
    reordered.swap(0, 1);
    let mut trailing = closed();
    trailing.push(0);
    let mut wrong = trial_tpm::EVENT_CLOSED.to_vec();
    wrong[0] ^= 1;
    for bad in [
        entry(12, EV_IPL, &BANKS, trial_tpm::EVENT_CLOSED),
        entry(13, EV_SEPARATOR, &BANKS, trial_tpm::EVENT_CLOSED),
        entry(13, EV_IPL, &BANKS, trial_tpm::EVENT_OPEN),
        entry(13, EV_IPL, &BANKS, &wrong),
        entry(13, EV_IPL, &BANKS[1..], trial_tpm::EVENT_CLOSED),
        entry(13, EV_IPL, &reordered, trial_tpm::EVENT_CLOSED),
        trailing,
    ] {
        assert_eq!(last(&log, &bad).unwrap_err(), Error::Event);
    }
    let mut unknown = closed();
    unknown[12..14].copy_from_slice(&0x0012u16.to_le_bytes());
    assert_eq!(last(&log, &unknown).unwrap_err(), Error::Algorithm);
    assert_eq!(
        last(&log, &closed()[..closed().len() - 1]).unwrap_err(),
        Error::Truncated
    );
}
