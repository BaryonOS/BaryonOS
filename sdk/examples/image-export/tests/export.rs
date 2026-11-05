use nieuw_os_image_codec::{Cancellation, Format, Image, Orientation, Pixels};
use nieuw_os_image_export::{convert, decode_limits, export, Error};

const PNG: &[u8] = include_bytes!("../../../../services/image-codec/fixtures/alpha.png");
const JPEG: &[u8] = include_bytes!("../../../../services/image-codec/fixtures/baseline.jpg");

fn image(width: usize, height: usize, pixels: Pixels) -> Image<'static> {
    Image {
        source: b"original",
        format: Format::Png,
        width,
        height,
        display_width: width,
        display_height: height,
        orientation: None,
        pixels,
        has_exif: false,
        has_icc: false,
        metadata_transforms_applied: false,
        icc_color_transform_applied: false,
    }
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn read_bmp(bytes: &[u8]) -> (usize, usize, Vec<u32>) {
    assert_eq!(&bytes[..2], b"BM");
    assert_eq!(u32_at(bytes, 2) as usize, bytes.len());
    assert_eq!(&bytes[6..10], &[0; 4]);
    assert_eq!(u32_at(bytes, 10), 54);
    assert_eq!(u32_at(bytes, 14), 40);
    assert_eq!(&bytes[26..30], &[1, 0, 24, 0]);
    assert_eq!(u32_at(bytes, 30), 0);
    assert_eq!(&bytes[38..54], &[0; 16]);
    let (width, height) = (u32_at(bytes, 18) as usize, u32_at(bytes, 22) as usize);
    let stride = (width * 24).div_ceil(32) * 4;
    assert_eq!(u32_at(bytes, 34) as usize, height * stride);
    assert_eq!(bytes.len(), 54 + height * stride);
    let mut rgb = Vec::new();
    for row in bytes[54..].chunks_exact(stride).rev() {
        for pixel in row[..width * 3].chunks_exact(3) {
            rgb.push(
                (u32::from(pixel[2]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[0]),
            );
        }
        assert!(row[width * 3..].iter().all(|byte| *byte == 0));
    }
    (width, height, rgb)
}

#[test]
fn bottom_up_bgr_header_all_padding_lengths_and_alpha_match_independent_values() {
    for width in 1..=4 {
        let pixels = [255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 1, 2, 3, 255];
        let raw = pixels[..width * 4]
            .iter()
            .copied()
            .chain(pixels[..width * 4].iter().copied())
            .collect();
        let source = image(width, 2, Pixels::Rgba8(raw));
        let output = export(&source, 0x0000ff, 1024).unwrap();
        let (w, h, actual) = read_bmp(&output);
        assert_eq!((w, h), (width, 2));
        let expected = [0xff0000, 0x00807f, 0x0000ff, 0x010203];
        assert_eq!(actual, [&expected[..width], &expected[..width]].concat());
        assert_eq!(source.source, b"original");
    }
}

#[test]
fn rgba16_rounding_and_orientation_are_baked_without_mutating_source() {
    let mut source = image(
        3,
        2,
        Pixels::Rgba16(
            (1u16..=6)
                .flat_map(|red| [red * 257, 0, 0, 65535])
                .collect(),
        ),
    );
    source.orientation = Some(Orientation::Rotate90);
    source.display_width = 2;
    source.display_height = 3;
    let before = match &source.pixels {
        Pixels::Rgba16(pixels) => pixels.clone(),
        _ => unreachable!(),
    };
    let output = export(&source, 0, 1024).unwrap();
    let (width, height, rgb) = read_bmp(&output);
    assert_eq!((width, height), (2, 3));
    assert_eq!(rgb, [4, 1, 5, 2, 6, 3].map(|red| red << 16));
    assert_eq!(source.source, b"original");
    let Pixels::Rgba16(after) = &source.pixels else {
        panic!("changed raw pixel depth")
    };
    assert_eq!(after, &before);
}

#[test]
fn all_eight_orientations_match_asymmetric_independent_display_matrices() {
    let cases = [
        (Orientation::Normal, 3, 2, [1, 2, 3, 4, 5, 6]),
        (Orientation::MirrorHorizontal, 3, 2, [3, 2, 1, 6, 5, 4]),
        (Orientation::Rotate180, 3, 2, [6, 5, 4, 3, 2, 1]),
        (Orientation::MirrorVertical, 3, 2, [4, 5, 6, 1, 2, 3]),
        (Orientation::Transpose, 2, 3, [1, 4, 2, 5, 3, 6]),
        (Orientation::Rotate90, 2, 3, [4, 1, 5, 2, 6, 3]),
        (Orientation::Transverse, 2, 3, [6, 3, 5, 2, 4, 1]),
        (Orientation::Rotate270, 2, 3, [3, 6, 2, 5, 1, 4]),
    ];
    for (orientation, width, height, expected) in cases {
        let mut source = image(
            3,
            2,
            Pixels::Rgba8((1..=6).flat_map(|red| [red, 0, 0, 255]).collect()),
        );
        source.orientation = Some(orientation);
        source.display_width = width;
        source.display_height = height;
        let output = export(&source, 0, 1024).unwrap();
        let (w, h, rgb) = read_bmp(&output);
        assert_eq!((w, h), (width, height));
        assert_eq!(rgb, expected.map(|red| red << 16), "{orientation:?}");
    }
}

#[test]
fn oversize_encoded_input_is_rejected_by_existing_codec_input_gate() {
    let input = vec![0; nieuw_os_image_export::MAX_INPUT_BYTES + 1];
    assert_eq!(
        convert(&input, 0, usize::MAX),
        Err(Error::Decode(nieuw_os_image_codec::Error::InputLimit))
    );
}

#[test]
fn actual_png_and_jpeg_export_every_pixel_matches_existing_codec_and_explicit_background() {
    for input in [PNG, JPEG] {
        let decoded =
            nieuw_os_image_codec::decode(input, decode_limits(), &Cancellation::default()).unwrap();
        let mut rgba = vec![0; decoded.display_width * decoded.display_height * 4];
        decoded.rgba8_into(&mut rgba).unwrap();
        let output = convert(input, 0x123456, 1024 * 1024).unwrap();
        let (width, height, pixels) = read_bmp(&output);
        assert_eq!(
            (width, height),
            (decoded.display_width, decoded.display_height)
        );
        for (pixel, expected) in pixels.iter().zip(rgba.chunks_exact(4)) {
            let a = u32::from(expected[3]);
            let channel = |value: u8, bg: u32| (u32::from(value) * a + bg * (255 - a) + 127) / 255;
            assert_eq!(
                *pixel,
                (channel(expected[0], 0x12) << 16)
                    | (channel(expected[1], 0x34) << 8)
                    | channel(expected[2], 0x56)
            );
        }
        assert_eq!(decoded.source, input);
    }
}

#[test]
fn invalid_geometry_samples_background_and_exact_output_limit_fail_explicitly() {
    let source = image(1, 1, Pixels::Rgba8(vec![1, 2, 3, 255]));
    assert_eq!(export(&source, 0, 57), Err(Error::OutputLimit));
    assert_eq!(export(&source, 0, 58).unwrap().len(), 58);
    assert_eq!(
        export(&source, 0x1000000, 58),
        Err(Error::InvalidBackground)
    );
    assert_eq!(
        convert(b"invalid", u32::MAX, 58),
        Err(Error::InvalidBackground)
    );
    assert!(matches!(
        convert(b"invalid", 0, 1024),
        Err(Error::Decode(_))
    ));
    for (width, height) in [(0, 1), (1, 0), (usize::MAX, 1), (16384, 16384)] {
        assert_eq!(
            export(&image(width, height, Pixels::Rgba8(vec![])), 0, usize::MAX),
            Err(Error::Dimensions)
        );
    }
    for pixels in [Pixels::Rgba8(vec![1, 2, 3]), Pixels::Rgba16(vec![1; 5])] {
        assert_eq!(
            export(&image(1, 1, pixels), 0, 1024),
            Err(Error::Decode(nieuw_os_image_codec::Error::OutputSize))
        );
    }
    let mut source = image(3, 2, Pixels::Rgba8(vec![0; 24]));
    source.orientation = Some(Orientation::Rotate90);
    assert_eq!(
        export(&source, 0, 1024),
        Err(Error::Decode(nieuw_os_image_codec::Error::OutputSize))
    );
}
