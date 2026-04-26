// 8-bit sRGB PNG writer. The pixel buffer is exactly `width * height * 4`
// bytes of tightly packed RGBA in sRGB encoding (which is what the export
// tonemap pass already writes when the target format is `Rgba8UnormSrgb`).

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use image::{ImageBuffer, Rgba};

fn build_buffer(
    width: u32,
    height: u32,
    pixels: &[u8],
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let expected = (width as usize) * (height as usize) * 4;
    anyhow::ensure!(
        pixels.len() == expected,
        "pixel buffer size mismatch: got {}, expected {}",
        pixels.len(),
        expected,
    );
    ImageBuffer::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| anyhow::anyhow!("ImageBuffer::from_raw rejected the buffer"))
}

/// Encode the given RGBA8 pixel buffer to a PNG byte stream in memory. Used
/// by the wasm download path (no filesystem) and by `write_rgba8` (native).
pub fn encode_rgba8(width: u32, height: u32, pixels: &[u8]) -> anyhow::Result<Vec<u8>> {
    let img = build_buffer(width, height, pixels)?;
    let mut out = Vec::with_capacity(pixels.len() / 4); // PNG ≈ 25-50 % of raw
    let mut cursor = std::io::Cursor::new(&mut out);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_rgba8(path: &Path, width: u32, height: u32, pixels: &[u8]) -> anyhow::Result<()> {
    let img = build_buffer(width, height, pixels)?;
    img.save(path)?;
    Ok(())
}
