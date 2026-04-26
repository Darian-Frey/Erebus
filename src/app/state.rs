// Top-level mutable application state — the single source of truth for
// preset parameters, viewport size, dirty flags, and selected output mode.

use crate::export::ExportRequest;
use crate::preset::PresetAction;
use crate::render::{
    gradient, GradientStop, LightingUniforms, NebulaUniforms, PostUniforms, StarfieldUniforms,
};

#[derive(Debug, Clone)]
pub struct State {
    pub time: f32,
    pub frame_index: u32,
    pub fps_ema: f32,
    pub frame_ms_ema: f32,
    pub seed: u32,
    /// Sub-1.0 scales the offscreen HDR target down; the composite pass
    /// upscales linearly for free. 0.5 → 4× cheaper raymarch.
    pub preview_scale: f32,
    pub nebula: NebulaUniforms,
    pub lighting: LightingUniforms,
    pub starfield: StarfieldUniforms,
    pub post: PostUniforms,
    pub gradient: Vec<GradientStop>,
    /// Set by anything that mutates `gradient`; cleared once the renderer
    /// re-uploads the LUT.
    pub gradient_dirty: bool,
    pub last_shader_error: Option<String>,

    /// Set by the Export button; consumed by the app update loop next frame.
    pub pending_export: Option<ExportRequest>,
    /// Last export result message (success path or error string).
    pub last_export_status: Option<String>,
    /// UI memory: last-chosen export width.
    pub export_width: u32,

    pub pending_preset: Option<PresetAction>,
    pub last_preset_status: Option<String>,
    /// Saved name to round-trip through `Preset::name`.
    pub preset_name: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0.0,
            frame_index: 0,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
            seed: 0xCAFEBABE,
            preview_scale: 0.5,
            nebula: NebulaUniforms::default(),
            lighting: LightingUniforms::default(),
            starfield: StarfieldUniforms::default(),
            post: PostUniforms::default(),
            gradient: gradient::synthwave_default(),
            gradient_dirty: true,
            last_shader_error: None,
            pending_export: None,
            last_export_status: None,
            export_width: 4096,
            pending_preset: None,
            last_preset_status: None,
            preset_name: "Untitled".to_string(),
        }
    }
}
