// Top-level mutable application state — the single source of truth for
// preset parameters, viewport size, dirty flags, and selected output mode.

use crate::render::{LightingUniforms, NebulaUniforms};

#[derive(Debug, Clone)]
pub struct State {
    pub time: f32,
    pub frame_index: u32,
    pub fps_ema: f32,
    pub frame_ms_ema: f32,
    pub exposure: f32,
    pub seed: u32,
    /// Sub-1.0 scales the offscreen HDR target down; the composite pass
    /// upscales linearly for free. 0.5 → 4× cheaper raymarch.
    pub preview_scale: f32,
    pub nebula: NebulaUniforms,
    pub lighting: LightingUniforms,
    pub last_shader_error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0.0,
            frame_index: 0,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
            exposure: 0.0,
            seed: 0xCAFEBABE,
            preview_scale: 0.5,
            nebula: NebulaUniforms::default(),
            lighting: LightingUniforms::default(),
            last_shader_error: None,
        }
    }
}
