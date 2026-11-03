use nieuw_os_image_codec::{decode, pixel_hash, Cancellation, Error, Format, Limits, Pixels};

fn limits() -> Limits {
    Limits {
        input_bytes: 1024 * 1024,
        width: 4096,
        height: 4096,
        pixels: 1_000_000,
        work_bytes: 128 * 1024 * 1024,
        metadata_bytes: 65536,
        png_chunks: 1024,
        jpeg_scans: 100,
    }
}

struct Fixture {
    name: &'static str,
    input: &'static [u8],
    reference: &'static [u8],
    width: usize,
    height: usize,
}
macro_rules! fixture {
    ($name:literal, $w:literal, $h:literal) => {
        Fixture {
            name: $name,
            input: include_bytes!(concat!("../fixtures/", $name)),
            reference: include_bytes!(concat!("../fixtures/", $name, ".rgba")),
            width: $w,
            height: $h,
        }
    };
}

fn pngs() -> [Fixture; 7] {
    [
        fixture!("rgb.png", 231, 149),
        fixture!("alpha.png", 128, 96),
        fixture!("palette4.png", 73, 47),
        fixture!("mono1.png", 65, 33),
        fixture!("gray-alpha.png", 97, 65),
        fixture!("adam7.png", 129, 77),
        fixture!("rgba16.png", 97, 65),
    ]
}
fn jpegs() -> [Fixture; 3] {
    [
        fixture!("baseline.jpg", 257, 129),
        fixture!("gray.jpg", 113, 67),
        fixture!("progressive.jpg", 193, 113),
    ]
}

fn verify(f: &Fixture, tolerance: u8, format: Format) {
    let image = decode(f.input, limits(), &Cancellation::default()).expect(f.name);
    assert_eq!(
        (image.width, image.height, image.format),
        (f.width, f.height, format),
        "{}",
        f.name
    );
    assert_eq!(image.source.as_ptr(), f.input.as_ptr());
    assert_eq!(image.source, f.input);
    let mut rgba = vec![0; f.width * f.height * 4];
    image.rgba8_into(&mut rgba).unwrap();
    let max_difference = rgba
        .iter()
        .zip(f.reference)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap();
    let total_difference: usize = rgba
        .iter()
        .zip(f.reference)
        .map(|(x, y)| usize::from(x.abs_diff(*y)))
        .sum();
    println!(
        "FIXTURE {} {}x{} pixel_hash={:016x} max_difference={} total_difference={}",
        f.name,
        f.width,
        f.height,
        pixel_hash(&image),
        max_difference,
        total_difference
    );
    if max_difference > tolerance {
        let beyond = rgba
            .iter()
            .zip(f.reference)
            .filter(|(x, y)| x.abs_diff(**y) > tolerance)
            .count();
        let squared: usize = rgba
            .iter()
            .zip(f.reference)
            .map(|(x, y)| usize::from(x.abs_diff(*y)).pow(2))
            .sum();
        println!(
            "QUALITY {} beyond={} squared_difference={} channels={}",
            f.name,
            beyond,
            squared,
            rgba.len()
        );
        for (i, (x, y)) in rgba
            .iter()
            .zip(f.reference)
            .enumerate()
            .filter(|(_, (x, y))| x.abs_diff(**y) == max_difference)
            .take(8)
        {
            println!(
                "WORST {} x={} y={} channel={} actual={} reference={}",
                f.name,
                (i / 4) % f.width,
                i / (4 * f.width),
                i % 4,
                x,
                y
            );
        }
    }
    assert!(
        max_difference <= tolerance,
        "{}: max difference {max_difference}, total {total_difference}",
        f.name
    );
}

fn fuzz_limits() -> Limits {
    Limits {
        input_bytes: 64 * 1024,
        width: 193,
        height: 128,
        pixels: 193 * 113,
        work_bytes: 2 * 1024 * 1024,
        metadata_bytes: 8192,
        png_chunks: 128,
        jpeg_scans: 16,
    }
}

struct DeterministicRng(u64);

impl DeterministicRng {
    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn fill(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            *byte = self.next() as u8;
        }
    }
}

fn exercise_bounded_input(input: &[u8], limits: Limits) {
    assert!(input.len() <= limits.input_bytes);
    let result = decode(input, limits, &Cancellation::default());
    if !input.starts_with(b"\x89PNG\r\n\x1a\n") && !input.starts_with(b"\xff\xd8") {
        assert!(matches!(result, Err(Error::UnknownFormat)));
        return;
    }

    if let Ok(image) = result {
        assert_eq!(image.source, input);
        assert!(image.width > 0 && image.width <= limits.width);
        assert!(image.height > 0 && image.height <= limits.height);
        assert_eq!(
            image.width * image.height,
            match &image.pixels {
                Pixels::Rgba8(samples) => {
                    assert_eq!(samples.len(), image.width * image.height * 4);
                    samples.len() / 4
                }
                Pixels::Rgba16(samples) => {
                    assert_eq!(samples.len(), image.width * image.height * 4);
                    samples.len() / 4
                }
            }
        );
    }
}

#[test]
fn png_real_variants_match_independent_pixels_exactly() {
    for f in pngs() {
        verify(&f, 0, Format::Png);
    }
}

#[test]
fn jpeg_baseline_progressive_and_grayscale_match_libjpeg_oracle() {
    for f in jpegs() {
        verify(&f, 3, Format::Jpeg);
    }
}

#[test]
fn png16_preserves_every_sample_and_display_conversion_does_not_modify_it() {
    let input = include_bytes!("../fixtures/rgba16.png");
    let expected = include_bytes!("../fixtures/rgba16.png.rgba16be");
    let image = decode(input, limits(), &Cancellation::default()).unwrap();
    let Pixels::Rgba16(samples) = &image.pixels else {
        panic!("full precision missing")
    };
    let original_hash = pixel_hash(&image);
    assert_eq!(samples.len(), 97 * 65 * 4);
    assert!(samples
        .iter()
        .zip(expected.chunks_exact(2))
        .all(|(sample, pair)| *sample == u16::from_be_bytes([pair[0], pair[1]])));
    assert!(samples.iter().any(|x| *x > 255 && x % 257 != 0));
    let mut preview = vec![0; samples.len()];
    image.rgba8_into(&mut preview).unwrap();
    assert_eq!(preview, include_bytes!("../fixtures/rgba16.png.rgba"));
    assert_eq!(pixel_hash(&image), original_hash);
}

#[test]
fn bounds_and_cancel_are_explicit_without_publishing_partial_pixels() {
    let input = include_bytes!("../fixtures/alpha.png");
    for (bounded, error) in [
        (
            Limits {
                input_bytes: 8,
                ..limits()
            },
            Error::InputLimit,
        ),
        (
            Limits {
                width: 127,
                ..limits()
            },
            Error::Dimensions,
        ),
        (
            Limits {
                pixels: 128 * 96 - 1,
                ..limits()
            },
            Error::Dimensions,
        ),
        (
            Limits {
                work_bytes: 1000,
                ..limits()
            },
            Error::WorkLimit,
        ),
        (
            Limits {
                width: 0,
                ..limits()
            },
            Error::InvalidLimits,
        ),
        (
            Limits {
                metadata_bytes: usize::MAX,
                ..limits()
            },
            Error::WorkLimit,
        ),
    ] {
        assert!(matches!(decode(input, bounded, &Cancellation::default()), Err(e) if e == error));
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        decode(input, limits(), &cancel),
        Err(Error::Cancelled)
    ));
}

#[test]
fn compressed_metadata_bomb_is_rejected_before_upstream_default_inflater() {
    assert!(matches!(
        decode(
            include_bytes!("../fixtures/icc-bomb.png"),
            limits(),
            &Cancellation::default()
        ),
        Err(Error::MetadataLimit)
    ));
    let normal = decode(
        include_bytes!("../fixtures/rgb.png"),
        limits(),
        &Cancellation::default(),
    )
    .unwrap();
    assert!(normal.has_icc);
}

#[test]
fn preserved_metadata_is_not_falsely_claimed_as_oriented_or_color_managed() {
    let image = decode(
        include_bytes!("../fixtures/baseline.jpg"),
        limits(),
        &Cancellation::default(),
    )
    .unwrap();
    assert!(image.has_exif);
    assert!(!image.metadata_transforms_applied);
    assert_eq!(image.source, include_bytes!("../fixtures/baseline.jpg"));
    assert!(matches!(
        decode(
            include_bytes!("../fixtures/cmyk.jpg"),
            limits(),
            &Cancellation::default()
        ),
        Err(Error::UnsupportedColor)
    ));
}

#[test]
fn truncated_corrupt_and_unknown_inputs_do_not_poison_the_next_decode() {
    for f in [
        fixture!("alpha.png", 128, 96),
        fixture!("progressive.jpg", 193, 113),
    ] {
        for end in (0..f.input.len()).step_by(31) {
            assert!(
                decode(&f.input[..end], limits(), &Cancellation::default()).is_err(),
                "{} prefix {end}",
                f.name
            );
        }
        let mut corrupt = f.input.to_vec();
        if f.name.ends_with("png") {
            *corrupt.last_mut().unwrap() ^= 1;
        } else {
            corrupt[0] = 0;
        }
        assert!(decode(&corrupt, limits(), &Cancellation::default()).is_err());
        verify(
            &f,
            if f.name.ends_with("png") { 0 } else { 3 },
            if f.name.ends_with("png") {
                Format::Png
            } else {
                Format::Jpeg
            },
        );
    }
    assert!(matches!(
        decode(b"not an image", limits(), &Cancellation::default()),
        Err(Error::UnknownFormat)
    ));
}

#[test]
fn oversized_header_and_apng_remain_explicit_open_profiles() {
    let mut data = include_bytes!("../fixtures/alpha.png").to_vec();
    data[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(
        decode(&data, limits(), &Cancellation::default()),
        Err(Error::Dimensions)
    ));
    let mut apng = include_bytes!("../fixtures/alpha.png").to_vec();
    // Rename the first IDAT chunk: resource preflight refuses animation before
    // CRC/pixel work. This is a negative frame, no animation-support evidence.
    let offset = apng.windows(4).position(|x| x == b"IDAT").unwrap();
    apng[offset..offset + 4].copy_from_slice(b"fdAT");
    assert!(matches!(
        decode(&apng, limits(), &Cancellation::default()),
        Err(Error::UnsupportedAnimation)
    ));
}

#[test]
fn output_extent_checks_preserve_destination_on_mismatch() {
    let mut image = decode(
        include_bytes!("../fixtures/rgba16.png"),
        limits(),
        &Cancellation::default(),
    )
    .unwrap();
    let mut out = vec![0xa5; image.width * image.height * 4 - 1];
    assert_eq!(image.rgba8_into(&mut out), Err(Error::OutputSize));
    assert!(out.iter().all(|x| *x == 0xa5));
    image.width = 1;
    let mut out = vec![0xa5; image.height * 4];
    assert_eq!(image.rgba8_into(&mut out), Err(Error::OutputSize));
    assert!(out.iter().all(|x| *x == 0xa5));
}

#[test]
fn deterministic_malformed_mutations_return_or_fail_without_panicking() {
    let bounded = Limits {
        width: 1024,
        height: 1024,
        pixels: 262144,
        work_bytes: 32 * 1024 * 1024,
        ..limits()
    };
    for original in [
        include_bytes!("../fixtures/alpha.png").as_slice(),
        include_bytes!("../fixtures/progressive.jpg").as_slice(),
    ] {
        for n in 0..256usize {
            let mut data = original.to_vec();
            let index = (n * 1013 + 17) % data.len();
            data[index] ^= 1 << (n % 8);
            if n % 5 == 0 {
                data.truncate(index);
            }
            let _ = decode(&data, bounded, &Cancellation::default());
        }
    }
}

#[test]
fn seeded_bounded_parser_sweep_fails_closed_without_panicking() {
    const MAX_CASES: usize = 2048;
    let limits = fuzz_limits();
    let mut rng = DeterministicRng(0x6e65_7577_6f73_0001);

    let progressive = include_bytes!("../fixtures/progressive.jpg");
    assert!(decode(progressive, limits, &Cancellation::default()).is_ok());
    exercise_bounded_input(progressive, limits);

    for _ in 0..MAX_CASES {
        let length = (rng.next() as usize) % 8193;
        let mut input = vec![0; length];
        rng.fill(&mut input);
        exercise_bounded_input(&input, limits);

        for signature in [b"\x89PNG\r\n\x1a\n".as_slice(), b"\xff\xd8".as_slice()] {
            let mut parser_input = input.clone();
            if parser_input.len() < signature.len() {
                parser_input.resize(signature.len(), 0);
            }
            parser_input[..signature.len()].copy_from_slice(signature);
            exercise_bounded_input(&parser_input, limits);
        }
    }

    for original in [
        include_bytes!("../fixtures/alpha.png").as_slice(),
        include_bytes!("../fixtures/progressive.jpg").as_slice(),
    ] {
        for _ in 0..MAX_CASES / 4 {
            let mut input = original.to_vec();
            let index = (rng.next() as usize) % input.len();
            input[index] ^= 1 << (rng.next() % 8);
            if rng.next() & 3 == 0 {
                input.truncate((rng.next() as usize) % input.len());
            }
            exercise_bounded_input(&input, limits);
        }
    }
}

#[test]
fn baseline_and_progressive_tiny_odd_and_single_axis_geometries_match_oracle() {
    for f in [
        fixture!("edge-1x1-baseline.jpg", 1, 1),
        fixture!("edge-1x1-progressive.jpg", 1, 1),
        fixture!("edge-1x17-baseline.jpg", 1, 17),
        fixture!("edge-1x17-progressive.jpg", 1, 17),
        fixture!("edge-17x1-baseline.jpg", 17, 1),
        fixture!("edge-17x1-progressive.jpg", 17, 1),
        fixture!("edge-3x5-baseline.jpg", 3, 5),
        fixture!("edge-3x5-progressive.jpg", 3, 5),
        fixture!("edge-5x3-baseline.jpg", 5, 3),
        fixture!("edge-5x3-progressive.jpg", 5, 3),
        fixture!("edge-17x19-baseline.jpg", 17, 19),
        fixture!("edge-17x19-progressive.jpg", 17, 19),
    ] {
        verify(&f, 3, Format::Jpeg);
    }
}
