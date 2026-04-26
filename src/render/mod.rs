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
    RENDER_MODE_CUBEMAP,
};
#[cfg(target_arch = "wasm32")]
pub use graph::PendingExport;
