// Render graph orchestration. Owns the wgpu Device/Queue, the HDR RGBA16F
// targets, the bind groups, and the ordered pass list.

pub mod bench;
pub mod context;
pub mod gradient;
pub mod graph;
pub mod uniforms;
pub mod hot_reload;
pub mod passes;
pub mod resources;

pub use gradient::GradientStop;
pub use graph::ErebusRenderer;
pub use uniforms::{
    FrameUniforms, LightingUniforms, NebulaUniforms, PostUniforms, StarfieldUniforms,
    DENSITY_LEGACY, DENSITY_MULTICHANNEL, PALETTE_HOO, PALETTE_NATURAL, PHASE_CS, PHASE_HG,
    REDDENING_GRAY, REDDENING_ISM, RENDER_MODE_CUBEMAP, SIGMA_LAW_CUSTOM, SIGMA_LAW_GRAY,
    SIGMA_LAW_ISM, WARP_CURL, WARP_FBM,
};
#[cfg(target_arch = "wasm32")]
pub use graph::PendingExport;
