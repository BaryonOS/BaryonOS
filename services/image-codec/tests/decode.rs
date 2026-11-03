use nieuw_os_image_codec::{
    decode, detect_format, pixel_hash, Cancellation, Error, Format, Limits, Orientation, Pixels,
};

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
    source_width: usize,
    source_height: usize,
    display_width: usize,
    display_height: usize,
    orientation: Option<Orientation>,
}
macro_rules! fixture {
    ($name:literal, $w:literal, $h:literal) => {
        fixture!($name, $w, $h, $w, $h, None)
    };
    ($name:literal, $source_w:literal, $source_h:literal, $display_w:literal, $display_h:literal) => {
        fixture!($name, $source_w, $source_h, $display_w, $display_h, None)
    };
    ($name:literal, $source_w:literal, $source_h:literal, $display_w:literal, $display_h:literal, $orientation:expr) => {
        Fixture {
            name: $name,
            input: include_bytes!(concat!("../fixtures/", $name)),
            reference: include_bytes!(concat!("../fixtures/", $name, ".rgba")),
            source_width: $source_w,
            source_height: $source_h,
            display_width: $display_w,
            display_height: $display_h,
            orientation: $orientation,
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
        fixture!(
            "baseline.jpg",
            257,
            129,
            129,
            257,
            Some(Orientation::Rotate90)
        ),
        fixture!("gray.jpg", 113, 67),
        fixture!("progressive.jpg", 193, 113),
    ]
}

#[test]
fn format_routing_uses_supported_content_signatures() {
    assert_eq!(
        detect_format(include_bytes!("../fixtures/rgb.png")),
        Some(Format::Png)
    );
    assert_eq!(
        detect_format(include_bytes!("../fixtures/baseline.jpg")),
        Some(Format::Jpeg)
    );
    assert_eq!(Format::Png.media_type(), "image/png");
    assert_eq!(Format::Jpeg.media_type(), "image/jpeg");

    assert_eq!(detect_format(b""), None);
    assert_eq!(detect_format(b"\x89PNG\r\n\x1a"), None);
    assert_eq!(detect_format(b"\xff\xd8"), Some(Format::Jpeg));
    assert_eq!(detect_format(b"not an image"), None);
}

fn verify(f: &Fixture, tolerance: u8, format: Format) {
    let image = decode(f.input, limits(), &Cancellation::default()).expect(f.name);
    assert_eq!(
        (
            image.width,
            image.height,
            image.display_width,
            image.display_height,
            image.format
        ),
        (
            f.source_width,
            f.source_height,
            f.display_width,
            f.display_height,
            format
        ),
        "{}",
        f.name
    );
    assert_eq!(image.source.as_ptr(), f.input.as_ptr());
    assert_eq!(image.source, f.input);
    let (reference, reference_width, reference_height) = orient_reference(
        f.reference,
        f.source_width,
        f.source_height,
        f.orientation.unwrap_or(Orientation::Normal),
    );
    assert_eq!(
        (reference_width, reference_height),
        (f.display_width, f.display_height)
    );
    let mut rgba = vec![0; f.display_width * f.display_height * 4];
    image.rgba8_into(&mut rgba).unwrap();
    let max_difference = rgba
        .iter()
        .zip(&reference)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap();
    let total_difference: usize = rgba
        .iter()
        .zip(&reference)
        .map(|(x, y)| usize::from(x.abs_diff(*y)))
        .sum();
    println!(
        "FIXTURE {} {}x{} pixel_hash={:016x} max_difference={} total_difference={}",
        f.name,
        f.display_width,
        f.display_height,
        pixel_hash(&image),
        max_difference,
        total_difference
    );
    if max_difference > tolerance {
        let beyond = rgba
            .iter()
            .zip(&reference)
            .filter(|(x, y)| x.abs_diff(**y) > tolerance)
            .count();
        let squared: usize = rgba
            .iter()
            .zip(&reference)
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
            .zip(&reference)
            .enumerate()
            .filter(|(_, (x, y))| x.abs_diff(**y) == max_difference)
            .take(8)
        {
            println!(
                "WORST {} x={} y={} channel={} actual={} reference={}",
                f.name,
                (i / 4) % f.display_width,
                i / (4 * f.display_width),
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
fn jpeg_exif_orientation_and_source_metadata_are_preserved() {
    let image = decode(
        include_bytes!("../fixtures/baseline.jpg"),
        limits(),
        &Cancellation::default(),
    )
    .unwrap();
    assert!(image.has_exif);
    assert_eq!(image.orientation, Some(Orientation::Rotate90));
    assert_eq!((image.width, image.height), (257, 129));
    assert_eq!((image.display_width, image.display_height), (129, 257));
    assert!(!image.metadata_transforms_applied);
    assert!(!image.icc_color_transform_applied);
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

fn jpeg_with_orientation(
    input: &[u8],
    orientation: u16,
    little_endian: bool,
    padding: usize,
) -> Vec<u8> {
    let mut tiff = Vec::new();
    if little_endian {
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&1u16.to_le_bytes());
        tiff.extend_from_slice(&0x0112u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&orientation.to_le_bytes());
    } else {
        tiff.extend_from_slice(b"MM");
        tiff.extend_from_slice(&42u16.to_be_bytes());
        tiff.extend_from_slice(&8u32.to_be_bytes());
        tiff.extend_from_slice(&1u16.to_be_bytes());
        tiff.extend_from_slice(&0x0112u16.to_be_bytes());
        tiff.extend_from_slice(&3u16.to_be_bytes());
        tiff.extend_from_slice(&1u32.to_be_bytes());
        tiff.extend_from_slice(&orientation.to_be_bytes());
    }
    tiff.extend_from_slice(&[0, 0]);
    tiff.extend_from_slice(&[0; 4]);
    tiff.resize(tiff.len() + padding, 0);

    let mut payload = b"Exif\0\0".to_vec();
    payload.extend_from_slice(&tiff);
    let segment_length = u16::try_from(payload.len() + 2).unwrap();
    let mut jpeg = Vec::with_capacity(input.len() + payload.len() + 4);
    jpeg.extend_from_slice(&input[..2]);
    jpeg.extend_from_slice(&[0xff, 0xe1]);
    jpeg.extend_from_slice(&segment_length.to_be_bytes());
    jpeg.extend_from_slice(&payload);
    jpeg.extend_from_slice(&input[2..]);
    jpeg
}

fn orient_reference(
    raw: &[u8],
    width: usize,
    height: usize,
    orientation: Orientation,
) -> (Vec<u8>, usize, usize) {
    let (display_width, display_height) = match orientation {
        Orientation::Transpose
        | Orientation::Rotate90
        | Orientation::Transverse
        | Orientation::Rotate270 => (height, width),
        _ => (width, height),
    };
    let mut display = vec![0; raw.len()];
    for display_y in 0..display_height {
        for display_x in 0..display_width {
            let (source_x, source_y) = match orientation {
                Orientation::Normal => (display_x, display_y),
                Orientation::MirrorHorizontal => (width - 1 - display_x, display_y),
                Orientation::Rotate180 => (width - 1 - display_x, height - 1 - display_y),
                Orientation::MirrorVertical => (display_x, height - 1 - display_y),
                Orientation::Transpose => (display_y, display_x),
                Orientation::Rotate90 => (display_y, height - 1 - display_x),
                Orientation::Transverse => (width - 1 - display_y, height - 1 - display_x),
                Orientation::Rotate270 => (width - 1 - display_y, display_x),
            };
            let source = (source_y * width + source_x) * 4;
            let destination = (display_y * display_width + display_x) * 4;
            display[destination..destination + 4].copy_from_slice(&raw[source..source + 4]);
        }
    }
    (display, display_width, display_height)
}

#[test]
fn exif_orientation_reference_matches_asymmetric_pixel_matrix() {
    let raw = (1..=6)
        .flat_map(|value| [value, 0, 0, 255])
        .collect::<Vec<_>>();
    for (orientation, expected_width, expected_height, expected) in [
        (Orientation::Normal, 3, 2, [1, 2, 3, 4, 5, 6]),
        (Orientation::MirrorHorizontal, 3, 2, [3, 2, 1, 6, 5, 4]),
        (Orientation::Rotate180, 3, 2, [6, 5, 4, 3, 2, 1]),
        (Orientation::MirrorVertical, 3, 2, [4, 5, 6, 1, 2, 3]),
        (Orientation::Transpose, 2, 3, [1, 4, 2, 5, 3, 6]),
        (Orientation::Rotate90, 2, 3, [4, 1, 5, 2, 6, 3]),
        (Orientation::Transverse, 2, 3, [6, 3, 5, 2, 4, 1]),
        (Orientation::Rotate270, 2, 3, [3, 6, 2, 5, 1, 4]),
    ] {
        let (display, width, height) = orient_reference(&raw, 3, 2, orientation);
        assert_eq!((width, height), (expected_width, expected_height));
        assert_eq!(width * height, expected.len());
        assert_eq!(
            display
                .chunks_exact(4)
                .map(|pixel| pixel[0])
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn all_jpeg_exif_orientations_apply_for_both_tiff_byte_orders() {
    let source = include_bytes!("../fixtures/progressive.jpg");
    let raw = decode(source, limits(), &Cancellation::default()).unwrap();
    let mut raw_rgba = vec![0; raw.display_width * raw.display_height * 4];
    raw.rgba8_into(&mut raw_rgba).unwrap();

    for (value, orientation) in [
        (1, Orientation::Normal),
        (2, Orientation::MirrorHorizontal),
        (3, Orientation::Rotate180),
        (4, Orientation::MirrorVertical),
        (5, Orientation::Transpose),
        (6, Orientation::Rotate90),
        (7, Orientation::Transverse),
        (8, Orientation::Rotate270),
    ] {
        for little_endian in [true, false] {
            let input = jpeg_with_orientation(source, value, little_endian, 0);
            let image = decode(&input, limits(), &Cancellation::default()).unwrap();
            let (expected, width, height) =
                orient_reference(&raw_rgba, raw.width, raw.height, orientation);
            assert_eq!(image.orientation, Some(orientation));
            assert_eq!(
                (
                    image.width,
                    image.height,
                    image.display_width,
                    image.display_height
                ),
                (raw.width, raw.height, width, height)
            );
            let mut display = vec![0; width * height * 4];
            image.rgba8_into(&mut display).unwrap();
            assert_eq!(
                display, expected,
                "orientation {value}, little={little_endian}"
            );
            assert_eq!(pixel_hash(&image), pixel_hash(&raw));
            assert!(!image.metadata_transforms_applied);
            assert_eq!(image.source, input);
        }
    }
}

#[test]
fn malformed_exif_is_ignored_and_oversized_exif_is_rejected() {
    let source = include_bytes!("../fixtures/progressive.jpg");
    let raw = decode(source, limits(), &Cancellation::default()).unwrap();
    let mut raw_rgba = vec![0; raw.display_width * raw.display_height * 4];
    raw.rgba8_into(&mut raw_rgba).unwrap();

    let invalid_value = jpeg_with_orientation(source, 9, true, 0);
    let image = decode(&invalid_value, limits(), &Cancellation::default()).unwrap();
    assert_eq!(image.orientation, None);
    let mut display = vec![0; image.display_width * image.display_height * 4];
    image.rgba8_into(&mut display).unwrap();
    assert_eq!(display, raw_rgba);

    for offset in [4, u32::MAX] {
        let mut invalid_offset = jpeg_with_orientation(source, 6, true, 0);
        let tiff = invalid_offset
            .windows(6)
            .position(|window| window == b"Exif\0\0")
            .unwrap()
            + 6;
        invalid_offset[tiff + 4..tiff + 8].copy_from_slice(&offset.to_le_bytes());
        let image = decode(&invalid_offset, limits(), &Cancellation::default()).unwrap();
        assert_eq!(image.orientation, None);
        let mut display = vec![0; image.display_width * image.display_height * 4];
        image.rgba8_into(&mut display).unwrap();
        assert_eq!(display, raw_rgba);
    }

    let oversized = jpeg_with_orientation(source, 6, true, 64);
    assert!(matches!(
        decode(
            &oversized,
            Limits {
                metadata_bytes: 32,
                ..limits()
            },
            &Cancellation::default()
        ),
        Err(Error::MetadataLimit)
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
