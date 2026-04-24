// Build-time and runtime config. Numeric constants live here so panel code
// doesn't ad-hoc invent magic numbers.

pub const WINDOW_TITLE: &str = "Erebus";
pub const INITIAL_WIDTH: f32 = 1280.0;
pub const INITIAL_HEIGHT: f32 = 800.0;
pub const MIN_WIDTH: f32 = 800.0;
pub const MIN_HEIGHT: f32 = 600.0;

// HDR working-space format. RGBA16F is filterable on every wgpu backend
// we target and gives the dynamic range emissive cores and bright stars need.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
