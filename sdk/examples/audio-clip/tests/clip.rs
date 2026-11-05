use nieuw_os_audio_clip::{Error, Wave, MAX_CHUNKS, MAX_INPUT_BYTES};

fn chunk(kind: &[u8; 4], bytes: &[u8]) -> Vec<u8> {
    let mut result = kind.to_vec();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    if bytes.len() & 1 != 0 {
        result.push(0);
    }
    result
}
fn riff(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut output = b"RIFF".to_vec();
    let size = 4 + chunks.iter().map(Vec::len).sum::<usize>();
    output.extend_from_slice(&(size as u32).to_le_bytes());
    output.extend_from_slice(b"WAVE");
    for chunk in chunks {
        output.extend_from_slice(chunk);
    }
    output
}
fn format(channels: u16) -> Vec<u8> {
    let mut fmt = Vec::new();
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&channels.to_le_bytes());
    fmt.extend_from_slice(&48000u32.to_le_bytes());
    fmt.extend_from_slice(&(48000 * u32::from(channels) * 2).to_le_bytes());
    fmt.extend_from_slice(&(channels * 2).to_le_bytes());
    fmt.extend_from_slice(&16u16.to_le_bytes());
    fmt
}
fn samples() -> Vec<u8> {
    [-32768i16, 32767, -1, 1, 0, 0, 1000, -2000]
        .into_iter()
        .flat_map(i16::to_le_bytes)
        .collect()
}
fn valid(channels: u16) -> Vec<u8> {
    riff(&[
        chunk(b"fmt ", &format(channels)),
        chunk(b"data", &samples()),
    ])
}

#[test]
fn clipping_keeps_all_channel_samples_rate_and_original_unchanged() {
    for channels in [1, 2] {
        let input = valid(channels);
        let before = input.clone();
        let wave = Wave::parse(&input).unwrap();
        let block = usize::from(channels) * 2;
        assert_eq!(wave.frames(), samples().len() / block);
        assert_eq!(wave.channels(), channels);
        assert_eq!(wave.sample_rate(), 48000);
        let clipped = wave.clip(1..3, false, usize::MAX).unwrap();
        let parsed = Wave::parse(&clipped).unwrap();
        assert_eq!(parsed.frames(), 2);
        assert_eq!(parsed.channels(), channels);
        assert_eq!(parsed.sample_rate(), 48000);
        assert_eq!(&clipped[44..], &samples()[block..3 * block]);
        assert_eq!(input, before);
    }
}

#[test]
fn metadata_loss_requires_choice_and_odd_chunks_and_data_first_are_validated() {
    let mut fmt = format(2);
    fmt.extend_from_slice(&0u16.to_le_bytes());
    let input = riff(&[
        chunk(b"JUNK", &[7]),
        chunk(b"data", &samples()),
        chunk(b"fmt ", &fmt),
        chunk(b"LIST", b"info"),
    ]);
    let wave = Wave::parse(&input).unwrap();
    assert_eq!(wave.metadata_chunks(), 2);
    assert_eq!(
        wave.clip(0..1, false, 1024),
        Err(Error::MetadataChoiceRequired)
    );
    let output = wave.clip(0..1, true, 1024).unwrap();
    assert_eq!(Wave::parse(&output).unwrap().metadata_chunks(), 0);
    assert_eq!(&output[44..], &samples()[..4]);
}

#[test]
fn zero_empty_overflow_and_exact_output_ranges_are_not_silently_clamped() {
    let input = valid(2);
    let wave = Wave::parse(&input).unwrap();
    for (start, end) in [(0, 0), (3, 2), (0, 5), (usize::MAX, usize::MAX)] {
        let range = start..end;
        assert_eq!(wave.clip(range, false, usize::MAX), Err(Error::Range));
    }
    assert_eq!(wave.clip(0..1, false, 47), Err(Error::OutputLimit));
    assert_eq!(wave.clip(0..1, false, 48).unwrap().len(), 48);
    let input = riff(&[chunk(b"fmt ", &format(1)), chunk(b"data", &[])]);
    assert_eq!(Wave::parse(&input).unwrap().frames(), 0);
}

#[test]
fn all_truncations_wrong_lengths_duplicates_and_partial_frames_are_rejected() {
    let input = valid(2);
    for end in 0..input.len() {
        assert!(Wave::parse(&input[..end]).is_err());
    }
    for at in [0, 4, 8, 16, 40] {
        let mut broken = input.clone();
        broken[at] ^= 0xff;
        assert!(Wave::parse(&broken).is_err(), "offset {at}");
    }
    assert!(matches!(
        Wave::parse(&riff(&[chunk(b"fmt ", &format(2))])),
        Err(Error::MissingAudioChunk)
    ));
    assert!(matches!(
        Wave::parse(&riff(&[
            chunk(b"fmt ", &format(2)),
            chunk(b"fmt ", &format(2)),
            chunk(b"data", &samples())
        ])),
        Err(Error::DuplicateAudioChunk)
    ));
    assert!(matches!(
        Wave::parse(&riff(&[
            chunk(b"fmt ", &format(2)),
            chunk(b"data", &samples()),
            chunk(b"data", &[])
        ])),
        Err(Error::DuplicateAudioChunk)
    ));
    assert!(matches!(
        Wave::parse(&riff(&[
            chunk(b"fmt ", &format(2)),
            chunk(b"data", &[0; 3])
        ])),
        Err(Error::IncompleteFrame)
    ));
}

#[test]
fn compressed_float_multichannel_or_inconsistent_format_is_not_claimed_as_pcm16() {
    for (offset, value, expected) in [
        (0, 3, Error::UnsupportedFormat),
        (2, 3, Error::UnsupportedFormat),
        (4, 0, Error::UnsupportedFormat),
        (8, 1, Error::InconsistentFormat),
        (12, 1, Error::InconsistentFormat),
        (14, 8, Error::UnsupportedFormat),
    ] {
        let mut fmt = format(2);
        fmt[offset..offset + if matches!(offset, 4 | 8) { 4 } else { 2 }].fill(0);
        fmt[offset] = value;
        assert_eq!(
            Wave::parse(&riff(&[chunk(b"fmt ", &fmt), chunk(b"data", &samples())])).unwrap_err(),
            expected
        );
    }
    assert!(matches!(
        Wave::parse(&vec![0; MAX_INPUT_BYTES + 1]),
        Err(Error::InputLimit)
    ));
    let mut chunks = vec![chunk(b"JUNK", &[]); MAX_CHUNKS - 2];
    chunks.extend([chunk(b"fmt ", &format(2)), chunk(b"data", &samples())]);
    assert!(Wave::parse(&riff(&chunks)).is_ok());
    chunks.push(chunk(b"JUNK", &[]));
    assert!(matches!(
        Wave::parse(&riff(&chunks)),
        Err(Error::ChunkLimit)
    ));
}

#[test]
fn malformed_tail_odd_padding_and_forged_lengths_cannot_hide_after_valid_audio() {
    let input = valid(2);
    for tail in [vec![0], vec![0; 7], b"JUNK\xff\xff\xff\xff".to_vec()] {
        let mut malformed = input.clone();
        malformed.extend_from_slice(&tail);
        let length = (malformed.len() - 8) as u32;
        malformed[4..8].copy_from_slice(&length.to_le_bytes());
        assert!(matches!(Wave::parse(&malformed), Err(Error::Truncated)));
    }
    let mut missing_pad = riff(&[
        chunk(b"fmt ", &format(2)),
        chunk(b"data", &samples()),
        chunk(b"JUNK", &[7]),
    ]);
    missing_pad.pop();
    let length = (missing_pad.len() - 8) as u32;
    missing_pad[4..8].copy_from_slice(&length.to_le_bytes());
    assert!(matches!(Wave::parse(&missing_pad), Err(Error::Truncated)));
}
