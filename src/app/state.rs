// Top-level mutable application state — the single source of truth for
// preset parameters, viewport size, dirty flags, and selected output mode.

#[derive(Debug, Clone)]
pub struct State {
    pub time: f32,
    pub exposure: f32,
    pub seed: u32,
    pub last_shader_error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0.0,
            exposure: 0.0,
            seed: 0xCAFEBABE,
            last_shader_error: None,
        }
    }
}
