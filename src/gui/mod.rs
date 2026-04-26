// egui-driven control surface. The central panel hosts the live preview via
// a wgpu paint callback; a side panel hosts parameter controls.

pub mod panels;
pub mod widgets;
pub mod theme;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::app::state::State;
use crate::render::{
    ErebusRenderer, FrameUniforms, LightingUniforms, NebulaUniforms, PostUniforms,
    StarfieldUniforms,
};

pub fn render(ctx: &egui::Context, state: &mut State) {
    egui::SidePanel::left("controls")
        .resizable(true)
        .default_width(320.0)
        .show(ctx, |ui| panels::controls(ui, state));

    if let Some(err) = state.last_shader_error.clone() {
        egui::TopBottomPanel::bottom("shader_error")
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(egui::Color32::from_rgb(0xff, 0x70, 0x70), "shader error:");
                ui.code(err);
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let ppp = ctx.pixels_per_point();

            // Hero-shot mode: full preview scale, full march/shadow steps,
            // bloom on, all star layers, no wasm pixel cap. The user pays the
            // multi-second render cost knowingly.
            let hero = state.hero_shot;
            let chosen = if hero {
                1.0
            } else {
                state.preview_scale.clamp(0.1, 1.0)
            };
            let scale = if state.interacting && !hero {
                (chosen * 0.5).max(0.15)
            } else {
                chosen
            };
            #[allow(unused_mut)]
            let mut tw = (rect.width() * ppp * scale).round().max(1.0);
            #[allow(unused_mut)]
            let mut th = (rect.height() * ppp * scale).round().max(1.0);

            // wasm hard cap: per-pass overhead in browser WebGPU dominates
            // the raymarch cost on integrated GPUs, so a HiDPI canvas drives
            // a ~1 s/frame render even at scale 0.35. Bound the long axis at
            // 384 px (preserve aspect) so cost is deterministic. Skipped in
            // hero-shot mode. Native is unbounded.
            #[cfg(target_arch = "wasm32")]
            if !hero {
                let max_axis: f32 = 384.0;
                let m = tw.max(th);
                if m > max_axis {
                    let k = max_axis / m;
                    tw = (tw * k).round().max(1.0);
                    th = (th * k).round().max(1.0);
                }
            }

            let target_size = (tw as u32, th as u32);

            let mut nebula = state.nebula;
            let mut lighting = state.lighting;
            let mut starfield = state.starfield;
            let mut post = state.post;
            if hero {
                nebula.steps = nebula.steps.max(128);
                lighting.shadow_steps = lighting.shadow_steps.max(6);
                starfield.layers = starfield.layers.max(3);
                if post.bloom_intensity < 0.05 {
                    post.bloom_intensity = 0.6;
                }
            }

            let (mode, cube_face) = state.view_mode.frame_uniforms();

            // Hash everything that affects the offscreen render. The shader
            // is a pure function of these inputs, so any frame whose hash
            // matches the last successfully-rendered hash can skip the
            // entire raymarch + bloom chain and just re-composite the cached
            // HDR + bloom textures. Idle preview cost: 1 swapchain pass.
            let mut hasher = DefaultHasher::new();
            bytemuck::bytes_of(&nebula).hash(&mut hasher);
            bytemuck::bytes_of(&lighting).hash(&mut hasher);
            bytemuck::bytes_of(&starfield).hash(&mut hasher);
            bytemuck::bytes_of(&post).hash(&mut hasher);
            state.seed.hash(&mut hasher);
            target_size.0.hash(&mut hasher);
            target_size.1.hash(&mut hasher);
            mode.hash(&mut hasher);
            cube_face.hash(&mut hasher);
            let render_hash = hasher.finish();

            let skip_prepare = state.last_rendered_hash == Some(render_hash);

            let frame = FrameUniforms {
                resolution: [target_size.0 as f32, target_size.1 as f32],
                time: state.time,
                exposure: post.exposure,
                seed: state.seed,
                frame_index: state.frame_index,
                mode,
                cube_face,
                ..Default::default()
            };

            let cb = egui_wgpu::Callback::new_paint_callback(
                rect,
                NebulaCallback {
                    frame,
                    nebula,
                    lighting,
                    starfield,
                    post,
                    target_size,
                    skip_prepare,
                },
            );
            ui.painter().add(cb);

            if !skip_prepare {
                state.last_rendered_hash = Some(render_hash);
            }
        });
}

struct NebulaCallback {
    frame: FrameUniforms,
    nebula: NebulaUniforms,
    lighting: LightingUniforms,
    starfield: StarfieldUniforms,
    post: PostUniforms,
    target_size: (u32, u32),
    /// Skip every offscreen pass — leaves the cached HDR + bloom textures
    /// untouched so paint() composites the previously-rendered hero frame.
    skip_prepare: bool,
}

impl egui_wgpu::CallbackTrait for NebulaCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if self.skip_prepare {
            return Vec::new();
        }
        if let Some(r) = resources.get_mut::<ErebusRenderer>() {
            r.prepare(
                queue,
                encoder,
                self.frame,
                self.nebula,
                self.lighting,
                self.starfield,
                self.post,
                self.target_size,
            );
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(r) = resources.get::<ErebusRenderer>() {
            r.composite(render_pass);
        }
    }
}
