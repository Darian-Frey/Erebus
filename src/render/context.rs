// Resolves to the path of a WGSL source file at build time. Hot-reload reads
// the same file from disk, but at runtime we still ship a known-good baseline
// compiled into the binary so a fresh checkout works without the source tree.

use std::path::PathBuf;

pub fn shader_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders")
}
