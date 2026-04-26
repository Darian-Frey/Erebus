// Versioned preset definition. Saved to disk as RON; loaded back through
// `preset::io`. Bump `format_version` whenever a field is added/removed in
// a breaking way and add a migrate step in `preset::migrate`.

use serde::{Deserialize, Serialize};

use crate::render::{
    GradientStop, LightingUniforms, NebulaUniforms, PostUniforms, StarfieldUniforms,
};

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub format_version: u32,
    pub name: String,
    pub seed: u32,
    pub nebula: NebulaUniforms,
    pub lighting: LightingUniforms,
    pub starfield: StarfieldUniforms,
    pub post: PostUniforms,
    pub gradient: Vec<GradientStop>,
}

impl Preset {
    /// Construct a preset capturing the current default uniforms — used as a
    /// starting point when the user hasn't loaded anything yet. Will be
    /// referenced by the gradient editor in Phase 8.
    #[allow(dead_code)]
    pub fn current(name: impl Into<String>, seed: u32) -> Self {
        Self {
            format_version: CURRENT_VERSION,
            name: name.into(),
            seed,
            nebula: NebulaUniforms::default(),
            lighting: LightingUniforms::default(),
            starfield: StarfieldUniforms::default(),
            post: PostUniforms::default(),
            gradient: crate::render::gradient::synthwave_default(),
        }
    }
}
