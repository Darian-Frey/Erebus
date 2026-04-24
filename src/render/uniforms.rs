// Packed POD uniform structs (#[repr(C)] + bytemuck) shared with WGSL.
// Field order and padding here MUST match the matching WGSL `struct` exactly.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FrameUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub exposure: f32,
    pub seed: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

impl Default for FrameUniforms {
    fn default() -> Self {
        Self {
            resolution: [1.0, 1.0],
            time: 0.0,
            exposure: 0.0,
            seed: 0,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }
    }
}
