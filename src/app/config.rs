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

// Live-preview offscreen HDR is rendered at a fixed 2:1 aspect (the natural
// equirect ratio), independent of canvas size. The skybox composite samples
// this target via reconstructed equirect UV; canvas-derived sizing causes
// pole compression to show through as visible pixel "clusters" when the
// orbit camera looks at high pitch. Fixed 2:1 → uniform solid-angle coverage.
//
// Slider `preview_scale` multiplies the long axis. `interacting` halves it
// further during drags. Hero shot uses the *_HERO* axis.
#[cfg(target_arch = "wasm32")]
pub const HDR_BASE_LONG_AXIS: f32 = 1024.0;
#[cfg(not(target_arch = "wasm32"))]
pub const HDR_BASE_LONG_AXIS: f32 = 2048.0;

#[cfg(target_arch = "wasm32")]
pub const HDR_HERO_LONG_AXIS: f32 = 2048.0;
#[cfg(not(target_arch = "wasm32"))]
pub const HDR_HERO_LONG_AXIS: f32 = 4096.0;
