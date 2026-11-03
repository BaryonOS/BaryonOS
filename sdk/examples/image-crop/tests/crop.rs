use nieuw_os_image_codec::{Format, Image, Orientation, Pixels};
use nieuw_os_image_crop::{crop, Error, Rect};

fn image() -> Image<'static> {
    Image {
        source: b"synthetic-unit-fixture",
        format: Format::Png,
        width: 3,
        height: 2,
        display_width: 3,
        display_height: 2,
        orientation: None,
        pixels: Pixels::Rgba8((1..=6).flat_map(|i| [i, i + 10, i + 20, i * 31]).collect()),
        has_exif: false,
        has_icc: false,
        metadata_transforms_applied: false,
        icc_color_transform_applied: false,
    }
}

#[test]
fn selected_rectangle_preserves_alpha_exactly_and_keeps_source_unchanged() {
    let source = image();
    let rect = Rect {
        x: 1,
        y: 0,
        width: 2,
        height: 2,
    };
    let result = crop(&source, rect, 40).unwrap();
    assert_eq!(
        result.rgba,
        [2, 12, 22, 62, 3, 13, 23, 93, 5, 15, 25, 155, 6, 16, 26, 186]
    );
    assert_eq!((result.width, result.height), (2, 2));
    assert_eq!(source.source, b"synthetic-unit-fixture");
    let Pixels::Rgba8(bytes) = &source.pixels else {
        panic!("fixture");
    };
    assert_eq!(bytes.len(), 24);
    assert_eq!(&bytes[..4], &[1, 11, 21, 31]);
    assert_eq!(crop(&source, rect, 39), Err(Error::WorkLimit));
}

#[test]
fn crop_uses_display_coordinates_after_existing_exif_orientation() {
    let mut source = image();
    source.orientation = Some(Orientation::Rotate90);
    source.display_width = 2;
    source.display_height = 3;
    let result = crop(
        &source,
        Rect {
            x: 0,
            y: 1,
            width: 1,
            height: 2,
        },
        32,
    )
    .unwrap();
    assert_eq!(result.rgba, [5, 15, 25, 155, 6, 16, 26, 186]);
    assert_eq!((source.width, source.height), (3, 2));
}

#[test]
fn no_clamp_overflow_or_silent_precision_reduction() {
    let mut source = image();
    for rect in [
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 1,
        },
        Rect {
            x: 3,
            y: 0,
            width: 1,
            height: 1,
        },
        Rect {
            x: 0,
            y: 2,
            width: 1,
            height: 1,
        },
        Rect {
            x: usize::MAX,
            y: 0,
            width: 1,
            height: 1,
        },
        Rect {
            x: 1,
            y: 0,
            width: usize::MAX,
            height: 1,
        },
    ] {
        assert_eq!(
            crop(&source, rect, usize::MAX),
            Err(Error::InvalidRectangle)
        );
    }

    source.pixels = Pixels::Rgba16(vec![65535; 24]);
    assert_eq!(
        crop(
            &source,
            Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            },
            28
        ),
        Err(Error::UnsupportedPrecision)
    );
}

#[test]
fn malformed_caller_image_returns_codec_error_instead_of_indexing_bad_pixels() {
    let mut source = image();
    source.pixels = Pixels::Rgba8(vec![0; 23]);
    let rect = Rect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
    };
    assert_eq!(
        crop(&source, rect, 28),
        Err(Error::Codec(nieuw_os_image_codec::Error::OutputSize))
    );
    source.pixels = Pixels::Rgba8(vec![0; 24]);
    source.display_width = 2;
    assert_eq!(
        crop(&source, rect, 28),
        Err(Error::Codec(nieuw_os_image_codec::Error::OutputSize))
    );
}
