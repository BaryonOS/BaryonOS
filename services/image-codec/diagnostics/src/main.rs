//! HOST ONLY diagnostic: inspect the maintained decoder before/after its
//! YCbCr-to-RGBA conversion. Never linked into the OS probe or decoder service.
use std::{env, fs};
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_jpeg::JpegDecoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 4, "input output ycbcr|rgba|planes");
    let color = match args[3].as_str() {
        "ycbcr" => ColorSpace::YCbCr,
        "rgba" => ColorSpace::RGBA,
        "planes" => ColorSpace::YCbCr,
        _ => panic!("unsupported diagnostic colorspace"),
    };
    let input = fs::read(&args[1]).unwrap();
    let options = DecoderOptions::new_safe()
        .set_strict_mode(true)
        .set_use_unsafe(false)
        .set_max_width(4096)
        .set_max_height(4096)
        .jpeg_set_max_scans(100)
        .jpeg_set_out_colorspace(color);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(&input), options);
    decoder.decode_headers().unwrap();
    let info = decoder.info().unwrap();
    if args[3] == "planes" {
        let mut raw = decoder.raw_output();
        let layout = raw.layout().unwrap();
        let count = raw.num_components().unwrap();
        let mut buffers: Vec<Vec<u8>> = (0..count).map(|n| vec![0; layout[n].byte_size]).collect();
        let mut planes: Vec<&mut [u8]> = buffers.iter_mut().map(Vec::as_mut_slice).collect();
        raw.decode_into_planes(&mut planes).unwrap();
        for (n, bytes) in buffers.iter().enumerate() {
            let p = &layout[n];
            fs::write(format!("{}.{}", args[2], n), bytes).unwrap();
            println!(
                "HOST_PLANE index={} width={} height={} stride={} bytes={}",
                n, p.width, p.height, p.stride, p.byte_size
            );
        }
        return;
    }
    let mut output = vec![0; decoder.output_buffer_size().unwrap()];
    decoder.decode_into(&mut output).unwrap();
    fs::write(&args[2], &output).unwrap();
    println!(
        "HOST_ONLY width={} height={} colorspace={:?} bytes={}",
        info.width,
        info.height,
        color,
        output.len()
    );
}
