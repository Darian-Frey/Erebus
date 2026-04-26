// 6-face cubemap export: writes six per-face PNGs alongside the user-
// chosen base path. Naming convention is the OpenGL / DirectX standard
// (+X, -X, +Y, -Y, +Z, -Z) so the files drop straight into Unity / Unreal
// / Bevy / Godot cubemap importers without renaming.

use std::path::Path;

use crate::export::png;

pub const FACE_SUFFIXES: [&str; 6] = ["px", "nx", "py", "ny", "pz", "nz"];

/// Write the six face buffers to `<base>_<suffix>.png` next to `base_path`.
/// `base_path` may end in `.png` (the extension is stripped before suffixing).
pub fn write_six(
    base_path: &Path,
    face_size: u32,
    faces: &[Vec<u8>; 6],
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let stem = base_path.with_extension("");
    let parent = stem
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let base_name = stem
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "erebus_cubemap".to_string());

    let mut written = Vec::with_capacity(6);
    for (i, suffix) in FACE_SUFFIXES.iter().enumerate() {
        let path = parent.join(format!("{base_name}_{suffix}.png"));
        png::write_rgba8(&path, face_size, face_size, &faces[i])?;
        written.push(path);
    }
    Ok(written)
}
