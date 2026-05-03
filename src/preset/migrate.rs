// Format-version migrations. Each `migrate_v<N>_to_v<N+1>` runs in sequence
// when a loaded preset declares an older `format_version`. Phase 7 ships
// at format_version 1; this file is a stub until v2 introduces a breaking
// change.

use crate::preset::schema::{Preset, CURRENT_VERSION};

pub fn migrate(mut preset: Preset) -> Preset {
    while preset.format_version < CURRENT_VERSION {
        match preset.format_version {
            // v1 → v2 (Phase 10.5 R2): NebulaUniforms.sigma_e changed shape
            // from f32 to [f32; 3]. The custom deserializer on the field
            // handles either form, so by the time we get here the value is
            // already a vec3 — no struct mutation needed. Several other
            // fields gained serde defaults (warp_kind, phase_kind, etc.)
            // and load fine without action. Just stamp the new version.
            1 => {
                preset.format_version = 2;
            }
            other => {
                log::warn!(
                    "preset is at unknown format_version {other}; loading as-is",
                );
                preset.format_version = CURRENT_VERSION;
            }
        }
    }
    preset
}
