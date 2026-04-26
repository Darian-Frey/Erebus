// egui-driven control surface. The central panel hosts the live preview via
// a wgpu paint callback; a side panel hosts parameter controls.

pub mod panels;
pub mod widgets;
pub mod theme;

use crate::app::state::State;
use crate::render::{ErebusRenderer, FrameUniforms, LightingUniforms, NebulaUniforms};

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
        .frame(egui::Frame::none().fill(egui::Color32::BLACK))
        .show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let ppp = ui.ctx().pixels_per_point();
            let scale = state.preview_scale.clamp(0.1, 1.0);
            let target_size = (
                (rect.width() * ppp * scale).round().max(1.0) as u32,
                (rect.height() * ppp * scale).round().max(1.0) as u32,
            );

            let frame = FrameUniforms {
                resolution: [target_size.0 as f32, target_size.1 as f32],
                time: state.time,
                exposure: state.exposure,
                seed: state.seed,
                frame_index: state.frame_index,
                ..Default::default()
            };

            let cb = egui_wgpu::Callback::new_paint_callback(
                rect,
                NebulaCallback {
                    frame,
                    nebula: state.nebula,
                    lighting: state.lighting,
                    target_size,
                },
            );
            ui.painter().add(cb);
        });
}

struct NebulaCallback {
    frame: FrameUniforms,
    nebula: NebulaUniforms,
    lighting: LightingUniforms,
    target_size: (u32, u32),
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
        if let Some(r) = resources.get_mut::<ErebusRenderer>() {
            r.prepare(
                queue,
                encoder,
                self.frame,
                self.nebula,
                self.lighting,
                self.target_size,
            );
        }
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        if let Some(r) = resources.get::<ErebusRenderer>() {
            r.composite(render_pass);
        }
    }
}
