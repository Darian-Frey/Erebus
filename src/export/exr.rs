// 16-bit linear OpenEXR writer. Pixel buffer is tightly packed RGBA32F in
// scene-referred linear space — exactly what the renderer's
// `render_equirect_rgba32f` returns when the export-linear pipeline is
// driven with `tonemap_mode = 3` (passthrough).
//
// EXR is the right format for HDR users who want to grade / re-tonemap in
// their own pipeline (Photoshop, Affinity, Resolve, Blender comp, Nuke):
// the on-disk values are physical radiance, not display-encoded sRGB.

use std::path::Path;

pub fn write_rgba32f(
    path: &Path,
    width: u32,
    height: u32,
    pixels: &[f32],
) -> anyhow::Result<()> {
    let expected = (width as usize) * (height as usize) * 4;
    anyhow::ensure!(
        pixels.len() == expected,
        "pixel buffer size mismatch: got {}, expected {}",
        pixels.len(),
        expected,
    );

    use exr::prelude::*;

    let w = width as usize;
    let h = height as usize;
    write_rgba_file(path, w, h, |x, y| {
        let i = (y * w + x) * 4;
        (pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3])
    })?;
    Ok(())
}
