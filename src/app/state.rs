// Top-level mutable application state — the single source of truth for
// preset parameters, viewport size, dirty flags, and selected output mode.

use web_time::Instant;

use crate::export::{ExportFormat, ExportKind, ExportRequest};
use crate::preset::PresetAction;
use crate::render::bench::BenchResult;
use crate::render::{
    gradient, GradientStop, LightingUniforms, NebulaUniforms, PostUniforms, StarfieldUniforms,
};

/// Live-preview projection mode. The export pipeline always writes equirect
/// (or cubemap as configured) — this toggle only changes how the on-screen
/// canvas presents the cached HDR target.
///
/// `Flat` displays the equirect 2:1 unwrap directly: export-faithful but with
/// heavy pole distortion when shown on a non-2:1 canvas.
/// `Skybox` re-samples the equirect through a virtual perspective camera
/// (mouse-drag yaw/pitch + scroll FOV). The HDR target is unchanged — only
/// the composite pass branches — so orbiting through the cached scene is
/// effectively free (no nebula re-march).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Flat,
    Skybox,
}

impl ViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            ViewMode::Flat => "Flat (equirect)",
            ViewMode::Skybox => "Skybox",
        }
    }
}

/// Free-look orbit camera state used when `ViewMode::Skybox` is active.
/// Yaw and pitch are stored in radians; FOV in degrees so the slider/scroll
/// inputs map naturally. Pitch is clamped to ±89° to avoid the singularity
/// at the poles. Not serialised into preset RON — purely viewer state.
#[derive(Debug, Clone, Copy)]
pub struct OrbitCamera {
    pub yaw_rad: f32,
    pub pitch_rad: f32,
    pub fov_y_deg: f32,
    /// Angular velocity (radians per second) carried over from the last
    /// drag frame. Decays exponentially while not being driven, giving the
    /// orbit a momentum feel after release.
    pub yaw_rate: f32,
    pub pitch_rate: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw_rad: 0.0,
            pitch_rad: 0.0,
            fov_y_deg: 70.0,
            yaw_rate: 0.0,
            pitch_rate: 0.0,
        }
    }
}

/// Quality preset that snaps the render parameters to a known-good tier.
/// Applied via the Frame panel buttons; users can still tweak individual
/// sliders afterward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier {
    Draft,
    Preview,
    Quality,
    Export,
}

impl QualityTier {
    pub fn label(&self) -> &'static str {
        match self {
            QualityTier::Draft => "Draft",
            QualityTier::Preview => "Preview",
            QualityTier::Quality => "Quality",
            QualityTier::Export => "Export",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            QualityTier::Draft => "Half-res preview, 64 march steps, 4 shadow steps. Fast slider iteration on integrated GPUs.",
            QualityTier::Preview => "Full-res preview, 96 march steps, 4 shadow steps. The default.",
            QualityTier::Quality => "Full-res, 128 march steps, 6 shadow steps. Hero-shot quality at interactive rates on a discrete GPU.",
            QualityTier::Export => "256 march steps, 8 shadow steps. Used for offline export; not playable in real time.",
        }
    }
}

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
    /// UI memory: last-chosen export width (or per-face size for cubemap).
    pub export_width: u32,
    pub export_kind: ExportKind,
    pub export_format: ExportFormat,

    pub pending_preset: Option<PresetAction>,
    pub last_preset_status: Option<String>,
    /// Saved name to round-trip through `Preset::name`.
    pub preset_name: String,

    // ---- Phase 8: performance ------------------------------------------
    /// Hash of the visual parameters last frame; used to detect interaction
    /// for the adaptive preview.
    pub last_param_hash: u64,
    /// When the user last touched a slider. Drives the adaptive preview
    /// auto-downscale.
    pub last_interaction_at: Instant,
    /// Derived each frame: true while the user is actively dragging a
    /// slider. Read by the GUI to scale the offscreen target down.
    pub interacting: bool,
    pub pending_bench: bool,
    /// True while the bench is mid-run; used to disable the button and show
    /// a spinner.
    pub bench_running: bool,
    pub bench_results: Vec<BenchResult>,

    /// Live-preview projection. Flat (default) shows the equirect 2:1 unwrap;
    /// Skybox re-samples the same HDR target through a perspective camera so
    /// the user can drag-to-look around. Does not affect export.
    pub view_mode: ViewMode,
    /// Orbit camera state used when `view_mode == Skybox`. Driven by mouse
    /// drag (yaw/pitch) and scroll (fov). Lives outside the offscreen-render
    /// hash so dragging is free — only the composite pass re-runs.
    pub orbit_camera: OrbitCamera,

    /// Browser-only "render hero shot" mode. While true, the GUI overrides
    /// the per-frame uniforms with Quality-tier values and lifts the wasm
    /// render-target cap, so the canvas shows one full-quality render of the
    /// user's current composition. Auto-clears the moment the user moves a
    /// slider (any param-hash change).
    pub hero_shot: bool,
    /// In-flight wasm export. Holds the readback buffer and the map-async rx.
    /// Set when the user clicks `Export PNG…`; cleared once the GPU readback
    /// completes and the PNG download has been triggered.
    #[cfg(target_arch = "wasm32")]
    pub pending_export_job: Option<crate::render::PendingExport>,
    /// Hash of the (uniforms, target_size, view_mode) tuple last rendered into
    /// the cached HDR + bloom textures. The shader is purely a function of
    /// these inputs, so frames where the hash is unchanged can skip every
    /// offscreen pass and just re-composite — turns the idle preview from a
    /// sustained ~8 fps raymarch into a ~60 fps blit. Cleared whenever the
    /// HDR target is resized or any parameter changes.
    pub last_rendered_hash: Option<u64>,
}

impl Default for State {
    fn default() -> Self {
        // Browser default is below the Draft tier: even at 64 steps the live
        // preview ran at ~0.9 fps in Chrome on integrated GPUs because each
        // render pass carries ~order-of-magnitude per-pass overhead vs native.
        // Halving render-target dimensions (×0.25 pixels) and dropping march
        // steps to 48 keeps the preview interactive; users can still bump to
        // Preview/Quality once they're happy with composition.
        #[allow(unused_mut)]
        let mut nebula = NebulaUniforms::default();
        #[allow(unused_mut)]
        let mut lighting = LightingUniforms::default();
        #[allow(unused_mut)]
        let mut starfield = StarfieldUniforms::default();
        #[allow(unused_mut)]
        let mut post = PostUniforms::default();
        #[cfg(target_arch = "wasm32")]
        {
            // Aggressive defaults for browser WebGPU on integrated GPUs.
            // Empirically the nebula raymarch costs ~440 ms/frame at 48 steps
            // / 4 shadow steps / 0.35 scale on this baseline; cutting march
            // steps and shadow steps roughly halves that. Users can scroll
            // up to Preview/Quality once they're on a discrete GPU.
            nebula.steps = 32;
            lighting.shadow_steps = 2;
            starfield.layers = 1;
            // Bloom off: 9-pass pyramid is wasted budget when the user is
            // still composing.
            post.bloom_intensity = 0.0;
        }
        let preview_scale: f32 = if cfg!(target_arch = "wasm32") { 0.25 } else { 0.5 };

        Self {
            time: 0.0,
            frame_index: 0,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
            seed: 0xCAFEBABE,
            preview_scale,
            nebula,
            lighting,
            starfield,
            post,
            gradient: gradient::synthwave_default(),
            gradient_dirty: true,
            last_shader_error: None,
            pending_export: None,
            last_export_status: None,
            export_width: 4096,
            export_kind: ExportKind::Equirect,
            export_format: ExportFormat::Png,
            pending_preset: None,
            last_preset_status: None,
            preset_name: "Untitled".to_string(),
            last_param_hash: 0,
            last_interaction_at: Instant::now(),
            interacting: false,
            pending_bench: false,
            bench_running: false,
            bench_results: Vec::new(),
            view_mode: ViewMode::Flat,
            orbit_camera: OrbitCamera::default(),
            hero_shot: false,
            #[cfg(target_arch = "wasm32")]
            pending_export_job: None,
            last_rendered_hash: None,
        }
    }
}
