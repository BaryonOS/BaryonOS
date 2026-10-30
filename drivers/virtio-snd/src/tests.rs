use super::*;

fn features() -> Negotiated {
    negotiate_features(VERSION_1).unwrap()
}

fn config(streams: u32) -> [u8; 12] {
    let mut bytes = [0; 12];
    bytes[4..8].copy_from_slice(&streams.to_le_bytes());
    bytes
}

fn parsed(streams: u32) -> Config {
    Config::parse(features(), &config(streams)).unwrap()
}

fn query() -> Query {
    parsed(2).query_stream(1).unwrap()
}

fn record(direction: u8) -> [u8; 32] {
    let mut item = [0; 32];
    item[8..16].copy_from_slice(&FMT_S16.to_le_bytes());
    item[16..24].copy_from_slice(&RATE_48000.to_le_bytes());
    item[24] = direction;
    item[25] = 1;
    item[26] = 2;
    item
}

fn response() -> [u8; 36] {
    let mut bytes = [0; 36];
    bytes[..4].copy_from_slice(&S_OK.to_le_bytes());
    bytes[4..].copy_from_slice(&record(D_OUTPUT));
    bytes
}

fn parse_one(bytes: &[u8]) -> Result<StreamInfo, Error> {
    query()
        .parse_response(bytes)
        .map(|infos| infos.as_slice()[0])
}

// Prototype test 1, now bound to negotiated features.
#[test]
fn config_bounds_and_absent_streams() {
    for length in 0..12 {
        assert_eq!(
            Config::parse(features(), &config(2)[..length]),
            Err(Error::Length)
        );
    }
    assert_eq!(Config::parse(features(), &[0; 16]), Err(Error::Length));
    for offset in [0, 4, 8] {
        let mut bad = config(2);
        bad[offset..offset + 4].copy_from_slice(&(MAX_ITEMS + 1).to_le_bytes());
        assert_eq!(Config::parse(features(), &bad), Err(Error::Limit));
        bad[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Config::parse(features(), &bad), Err(Error::Limit));
    }
    let absent = parsed(0);
    assert_eq!(absent.query_stream(0), Err(Error::Range));
    let max = parsed(MAX_ITEMS);
    assert!(max.query_stream(MAX_ITEMS - 1).is_ok());
    assert_eq!(max.query_stream(MAX_ITEMS), Err(Error::Range));
    assert_eq!(max.query_stream(u32::MAX), Err(Error::Overflow));
}

// Prototype test 2.
#[test]
fn request_and_output_selection_are_exact() {
    let bytes = query().encode();
    assert_eq!(u32_at(&bytes, 0), Ok(R_PCM_INFO));
    assert_eq!(u32_at(&bytes, 4), Ok(1));
    assert_eq!(u32_at(&bytes, 8), Ok(1));
    assert_eq!(u32_at(&bytes, 12), Ok(32));
    assert_eq!(query().response_len(), Ok(36));
    let info = parse_one(&response()).unwrap();
    assert_eq!(info.id(), 1);
    assert_eq!(info.stereo_s16_48000().unwrap().stream_id(), 1);
}

// Prototype test 3.
#[test]
fn short_and_oversized_success_never_parse() {
    let bytes = response();
    for length in 0..bytes.len() {
        assert_eq!(query().parse_response(&bytes[..length]), Err(Error::Length));
    }
    let mut oversized = [0; 37];
    oversized[..36].copy_from_slice(&bytes);
    assert_eq!(query().parse_response(&oversized), Err(Error::Length));
}

// Prototype test 4.
#[test]
fn all_failed_and_unknown_statuses_are_errors() {
    for status in [0, 0x8001, 0x8002, 0x8003, 0x8004, u32::MAX] {
        assert_eq!(
            query().parse_response(&status.to_le_bytes()),
            Err(Error::Status(status))
        );
        let mut bad = response();
        bad[..4].copy_from_slice(&status.to_le_bytes());
        assert_eq!(query().parse_response(&bad), Err(Error::Status(status)));
    }
}

// Prototype test 5, extended to every reserved bit and padding byte.
#[test]
fn malformed_fields_are_rejected() {
    for (offset, value, expected) in [
        (28, 2, Error::InvalidDirection),
        (28, 0xff, Error::InvalidDirection),
        (29, 0, Error::InvalidChannels),
        (29, 3, Error::InvalidChannels),
        (8, 0x20, Error::Reserved),
        (11, 0x80, Error::Reserved),
        (15, 2, Error::Reserved),
        (19, 0x80, Error::Reserved),
        (21, 0x40, Error::Reserved),
        (27, 0x80, Error::Reserved),
    ] {
        let mut bad = response();
        bad[offset] = value;
        assert_eq!(parse_one(&bad), Err(expected), "offset {offset}");
    }
    for offset in 31..36 {
        let mut bad = response();
        bad[offset] = 1;
        assert_eq!(parse_one(&bad), Err(Error::Reserved));
    }
    let mut all_known = response();
    all_known[8..12].copy_from_slice(&PCM_FEATURES_MASK.to_le_bytes());
    all_known[12..20].copy_from_slice(&FORMATS_MASK.to_le_bytes());
    all_known[20..28].copy_from_slice(&RATES_MASK.to_le_bytes());
    assert!(parse_one(&all_known).is_ok());
}

// Prototype test 6.
#[test]
fn capture_and_unadvertised_profiles_are_not_playback() {
    let mut capture = response();
    capture[28] = D_INPUT;
    let info = parse_one(&capture).unwrap();
    assert!(!info.is_output());
    assert_eq!(info.stereo_s16_48000(), Err(Error::Unsupported));
    for offset in [12, 20] {
        let mut other = response();
        other[offset..offset + 8].copy_from_slice(&1u64.to_le_bytes());
        assert_eq!(
            parse_one(&other).unwrap().stereo_s16_48000(),
            Err(Error::Unsupported)
        );
        other[offset..offset + 8].fill(0);
        assert_eq!(parse_one(&other), Err(Error::Unsupported));
    }
    for (min, max) in [(1, 1), (3, 8)] {
        let mut channels = response();
        channels[29] = min;
        channels[30] = max;
        assert_eq!(
            parse_one(&channels).unwrap().stereo_s16_48000(),
            Err(Error::Unsupported)
        );
    }
}

#[test]
fn features_require_modern_and_never_ctls_or_platform_claim() {
    assert_eq!(negotiate_features(VERSION_1).unwrap().bits(), VERSION_1);
    assert_eq!(
        negotiate_features(VERSION_1 | F_CTLS | u32::MAX as u64)
            .unwrap()
            .bits(),
        VERSION_1
    );
    for offered in [
        0,
        F_CTLS,
        u32::MAX as u64,
        VERSION_1 | ACCESS_PLATFORM,
        u64::MAX,
    ] {
        assert_eq!(negotiate_features(offered), Err(Error::Features));
    }
    assert_eq!(
        Config::parse(Negotiated(VERSION_1 | F_CTLS), &config(2)),
        Err(Error::Features)
    );
}

#[test]
fn ranges_are_bounded_and_overflow_checked() {
    let config = parsed(4);
    assert_eq!(config.query_streams(0, 0), Err(Error::Range));
    assert_eq!(config.query_streams(0, 5), Err(Error::Range));
    assert_eq!(config.query_streams(3, 2), Err(Error::Range));
    assert_eq!(config.query_streams(4, 1), Err(Error::Range));
    assert_eq!(config.query_streams(u32::MAX, 1), Err(Error::Overflow));
    assert_eq!(config.query_streams(1, MAX_ITEMS + 1), Err(Error::Range));
    assert_eq!(config.query_streams(0, u32::MAX), Err(Error::Range));
    let full = parsed(MAX_ITEMS).query_streams(0, MAX_ITEMS).unwrap();
    assert_eq!(full.response_len(), Ok(MAX_RESPONSE_BYTES));
}

#[test]
fn multi_record_response_assigns_ids_and_fails_on_any_bad_record() {
    let query = parsed(4).query_streams(1, 3).unwrap();
    assert_eq!(u32_at(&query.encode(), 4), Ok(1));
    assert_eq!(u32_at(&query.encode(), 8), Ok(3));
    let mut bytes = [0u8; 4 + 3 * 32];
    bytes[..4].copy_from_slice(&S_OK.to_le_bytes());
    bytes[4..36].copy_from_slice(&record(D_INPUT));
    bytes[36..68].copy_from_slice(&record(D_OUTPUT));
    bytes[68..].copy_from_slice(&record(D_OUTPUT));
    let infos = query.parse_response(&bytes).unwrap();
    let ids: [u32; 3] = core::array::from_fn(|i| infos.as_slice()[i].id());
    assert_eq!(ids, [1, 2, 3]);
    assert_eq!(infos.first_stereo_s16_48000().unwrap().stream_id(), 2);

    // Count mismatch: one record short or one extra is never accepted.
    assert_eq!(query.parse_response(&bytes[..68]), Err(Error::Length));
    let mut extra = [0u8; 4 + 4 * 32];
    extra[..bytes.len()].copy_from_slice(&bytes);
    assert_eq!(query.parse_response(&extra), Err(Error::Length));

    for broken in 0..3 {
        let mut bad = bytes;
        bad[4 + broken * 32 + 31] = 1;
        assert_eq!(query.parse_response(&bad), Err(Error::Reserved));
    }

    let mut capture_only = bytes;
    capture_only[36 + 24] = D_INPUT;
    capture_only[68 + 24] = D_INPUT;
    assert_eq!(
        query
            .parse_response(&capture_only)
            .unwrap()
            .first_stereo_s16_48000(),
        Err(Error::Unsupported)
    );
}

#[test]
fn readers_never_panic_on_bad_offsets() {
    assert_eq!(u32_at(&[0; 3], 0), Err(Error::Length));
    assert_eq!(u64_at(&[0; 8], 1), Err(Error::Length));
    assert_eq!(u32_at(&[0; 4], usize::MAX), Err(Error::Overflow));
    assert_eq!(u64_at(&[0; 8], usize::MAX - 3), Err(Error::Overflow));
}

#[test]
fn qemu_identity_constants_match_observed_device() {
    assert_eq!(VIRTIO_SND_PCI_DEVICE_ID, 0x1059);
}
