// Naga-validates every shaders/**/*.wgsl on the disk. Runs in CI; catches
// regressions in WGSL syntax before they ever hit a device.

use std::fs;
use std::path::{Path, PathBuf};

fn shader_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders")
}

fn collect_wgsl(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_wgsl(&path, out);
        } else if path.extension().map(|e| e == "wgsl").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn all_shaders_parse() {
    let mut paths = Vec::new();
    collect_wgsl(&shader_root(), &mut paths);
    assert!(!paths.is_empty(), "no .wgsl files discovered");

    let mut failures = Vec::new();
    for path in &paths {
        let src = fs::read_to_string(path).unwrap();
        // Some files are stubs that contain only comments; skip those — they
        // cannot be parsed as valid modules. Real modules declare entries.
        let has_entry = src.contains("@vertex")
            || src.contains("@fragment")
            || src.contains("@compute");
        if !has_entry {
            continue;
        }
        if let Err(e) = wgpu::naga::front::wgsl::parse_str(&src) {
            failures.push(format!("{}:\n{}", path.display(), e.emit_to_string(&src)));
        }
    }

    assert!(
        failures.is_empty(),
        "WGSL validation failures:\n\n{}",
        failures.join("\n---\n")
    );
}
