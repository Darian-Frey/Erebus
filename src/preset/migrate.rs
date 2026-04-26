// Format-version migrations. Each `migrate_v<N>_to_v<N+1>` runs in sequence
// when a loaded preset declares an older `format_version`. Phase 7 ships
// at format_version 1; this file is a stub until v2 introduces a breaking
// change.

use crate::preset::schema::{Preset, CURRENT_VERSION};

pub fn migrate(mut preset: Preset) -> Preset {
    while preset.format_version < CURRENT_VERSION {
        match preset.format_version {
            // Add migration arms here as the schema evolves:
            //   0 => preset = migrate_v0_to_v1(preset),
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
