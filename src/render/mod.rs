// Render graph orchestration. Owns the wgpu Device/Queue, the HDR RGBA16F
// targets, the bind groups, and the ordered pass list.

pub mod context;
pub mod graph;
pub mod uniforms;
pub mod hot_reload;
pub mod passes;
pub mod resources;

pub use graph::ErebusRenderer;
pub use uniforms::FrameUniforms;
