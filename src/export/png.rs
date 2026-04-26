// 8-bit sRGB PNG writer. The pixel buffer is exactly `width * height * 4`
// bytes of tightly packed RGBA in sRGB encoding (which is what the export
// tonemap pass already writes when the target format is `Rgba8UnormSrgb`).

use std::path::Path;

use image::{ImageBuffer, Rgba};

pub fn write_rgba8(path: &Path, width: u32, height: u32, pixels: &[u8]) -> anyhow::Result<()> {
    let expected = (width as usize) * (height as usize) * 4;
    anyhow::ensure!(
        pixels.len() == expected,
        "pixel buffer size mismatch: got {}, expected {}",
        pixels.len(),
        expected,
    );
    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| anyhow::anyhow!("ImageBuffer::from_raw rejected the buffer"))?;
    img.save(path)?;
    Ok(())
}
