// Versioned preset (de)serialization. RON is the on-disk format; embedded
// shipped presets live under `assets/presets/`. Migrations run on load so
// older preset files keep working as the schema evolves.

pub mod schema;
pub mod io;
pub mod migrate;

pub use schema::Preset;

/// Identifies a shipped preset that lives inside the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippedPreset {
    Synthwave,
    Cyberpunk,
    RetroScifi,
}

impl ShippedPreset {
    pub const ALL: &'static [ShippedPreset] = &[
        ShippedPreset::Synthwave,
        ShippedPreset::Cyberpunk,
        ShippedPreset::RetroScifi,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ShippedPreset::Synthwave => "Synthwave",
            ShippedPreset::Cyberpunk => "Cyberpunk",
            ShippedPreset::RetroScifi => "Retro Sci-Fi",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            ShippedPreset::Synthwave => "synthwave",
            ShippedPreset::Cyberpunk => "cyberpunk",
            ShippedPreset::RetroScifi => "retro_scifi",
        }
    }

    pub fn load(&self) -> anyhow::Result<Preset> {
        io::load_embedded(self.slug())
    }
}

/// Action requested by the GUI; consumed by the app update loop next frame.
#[derive(Debug, Clone)]
pub enum PresetAction {
    SaveToFile,
    LoadFromFile,
    LoadShipped(ShippedPreset),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_presets_load() {
        for shipped in ShippedPreset::ALL {
            let p = shipped
                .load()
                .unwrap_or_else(|e| panic!("{}: {e}", shipped.label()));
            assert_eq!(p.format_version, schema::CURRENT_VERSION);
            assert!(!p.name.is_empty(), "{}: empty name", shipped.label());
            assert!(
                !p.gradient.is_empty(),
                "{}: empty gradient",
                shipped.label()
            );
        }
    }

    #[test]
    fn shipped_presets_round_trip() {
        let pretty = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .indentor("  ".to_string());
        for shipped in ShippedPreset::ALL {
            let original = shipped.load().expect("shipped preset loads");
            let serialised = ron::ser::to_string_pretty(&original, pretty.clone()).unwrap();
            let raw: ron::Value = ron::from_str(&serialised).unwrap();
            let again: Preset = raw.into_rust().unwrap();
            assert_eq!(again.name, original.name);
            assert_eq!(again.seed, original.seed);
            assert_eq!(again.gradient.len(), original.gradient.len());
            assert!((again.nebula.density_scale - original.nebula.density_scale).abs() < 1e-5);
        }
    }
}
