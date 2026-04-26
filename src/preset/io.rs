// RON read/write for the preset schema. Migration runs automatically on
// load so older format versions still produce a current-shape `Preset`.

use std::path::Path;

use crate::preset::migrate;
use crate::preset::schema::Preset;

pub fn save_to_file(path: &Path, preset: &Preset) -> anyhow::Result<()> {
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(8)
        .indentor("  ".to_string())
        .struct_names(true);
    let s = ron::ser::to_string_pretty(preset, pretty)?;
    std::fs::write(path, s)?;
    Ok(())
}

pub fn load_from_file(path: &Path) -> anyhow::Result<Preset> {
    let s = std::fs::read_to_string(path)?;
    let raw: ron::Value = ron::from_str(&s)?;
    let preset: Preset = raw.into_rust()?;
    Ok(migrate::migrate(preset))
}

/// Embedded shipped preset, loaded from `assets/presets/<name>.ron` at
/// compile time so they ship inside the binary without a separate asset
/// install step.
pub fn load_embedded(name: &str) -> anyhow::Result<Preset> {
    let raw = match name {
        "synthwave" => include_str!("../../assets/presets/synthwave.ron"),
        "cyberpunk" => include_str!("../../assets/presets/cyberpunk.ron"),
        "retro_scifi" => include_str!("../../assets/presets/retro_scifi.ron"),
        other => anyhow::bail!("unknown shipped preset: {other}"),
    };
    let preset: Preset = ron::from_str(raw)?;
    Ok(migrate::migrate(preset))
}
