use nieuw_os_image_codec::{decode, Cancellation, Limits};
use nieuw_os_image_crop::{crop, Rect};
use std::io::{self, Read, Write};

const INPUT_LIMIT: usize = 8 * 1024 * 1024;
const OUTPUT_LIMIT: usize = 8 * 1024 * 1024;
const PIXELS: usize = 1024 * 1024;

fn main() {
    if let Err(message) = run() {
        eprintln!("image-crop: {message}");
        std::process::exit(1);
    }
}

fn parse_number(value: &str) -> Result<usize, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("coordinaten en afmetingen moeten decimale gehele getallen zijn".into());
    }
    value.parse().map_err(|_| "getal buiten bereik".into())
}

fn arguments() -> Result<Rect, String> {
    let mut args = std::env::args().skip(1);
    let (mut x, mut y, mut width, mut height) = (None, None, None, None);
    let (mut drop_metadata, mut unmanaged) = (false, false);
    while let Some(arg) = args.next() {
        let target = match arg.as_str() {
            "--x" => &mut x,
            "--y" => &mut y,
            "--width" => &mut width,
            "--height" => &mut height,
            "--drop-metadata" if !drop_metadata => {
                drop_metadata = true;
                continue;
            }
            "--unmanaged-colour" if !unmanaged => {
                unmanaged = true;
                continue;
            }
            _ => return Err(format!("onbekend of dubbel argument: {arg}")),
        };
        if target.is_some() {
            return Err(format!("dubbel argument: {arg}"));
        }
        *target = Some(parse_number(
            &args.next().ok_or("ontbrekende getalwaarde")?,
        )?);
    }
    if !drop_metadata || !unmanaged {
        return Err("vereist --drop-metadata --unmanaged-colour: uitvoer bewaart samples/alpha, niet metadata of ICC/gamma-kleurweergave".into());
    }
    Ok(Rect {
        x: x.ok_or("ontbreekt --x")?,
        y: y.ok_or("ontbreekt --y")?,
        width: width.ok_or("ontbreekt --width")?,
        height: height.ok_or("ontbreekt --height")?,
    })
}

struct BoundedOutput(Vec<u8>);

impl Write for BoundedOutput {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if data.len() > OUTPUT_LIMIT - self.0.len() {
            return Err(io::Error::other("PNG-uitvoer overschrijdt 8 MiB"));
        }
        self.0
            .try_reserve(data.len())
            .map_err(|_| io::Error::other("geen uitvoergeheugen"))?;
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run() -> Result<(), String> {
    let rect = arguments()?;
    let mut input = Vec::new();
    input
        .try_reserve_exact(INPUT_LIMIT + 1)
        .map_err(|_| "geen invoergeheugen")?;
    io::stdin()
        .lock()
        .take((INPUT_LIMIT + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|e| format!("lezen: {e}"))?;
    let limits = Limits {
        input_bytes: INPUT_LIMIT,
        width: 4096,
        height: 4096,
        pixels: PIXELS,
        work_bytes: 128 * 1024 * 1024,
        metadata_bytes: 256 * 1024,
        png_chunks: 1024,
        jpeg_scans: 128,
    };
    let image = decode(&input, limits, &Cancellation::default())
        .map_err(|e| format!("decoder weigert invoer: {e:?}"))?;
    let clipped = crop(&image, rect, PIXELS * 8).map_err(|e| format!("crop geweigerd: {e:?}"))?;
    let mut output = BoundedOutput(Vec::new());
    let mut encoder = png::Encoder::new(&mut output, clipped.width as u32, clipped.height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| format!("PNG-header: {e}"))?;
    writer
        .write_image_data(&clipped.rgba)
        .map_err(|e| format!("PNG-pixels: {e}"))?;
    writer.finish().map_err(|e| format!("PNG-afronden: {e}"))?;
    io::stdout()
        .lock()
        .write_all(&output.0)
        .map_err(|e| format!("schrijven: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_encoder_sink_accepts_exact_limit_and_rejects_one_more_byte() {
        let mut output = BoundedOutput(Vec::new());
        output.write_all(&vec![0x71; OUTPUT_LIMIT]).unwrap();
        assert!(output.write_all(&[0x72]).is_err());
        assert_eq!(output.0.len(), OUTPUT_LIMIT);
        assert!(output.0.iter().all(|byte| *byte == 0x71));
    }
}
