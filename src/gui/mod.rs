// egui-driven control surface. The central panel hosts the live preview via
// a wgpu paint callback; a side panel hosts parameter controls.

pub mod panels;
pub mod widgets;
pub mod theme;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::app::config::{HDR_BASE_LONG_AXIS, HDR_HERO_LONG_AXIS};
use crate::app::state::{OrbitCamera, State, ViewMode};
use crate::render::{
    ErebusRenderer, FrameUniforms, LightingUniforms, NebulaUniforms, PostUniforms,
    StarfieldUniforms,
};

pub fn render(ctx: &egui::Context, state: &mut State) {
    // Keyboard shortcuts apply globally (panel or canvas focused doesn't
    // matter). Run before panel layout so the panel's RichText readout
    // reflects this frame's state.
    handle_keyboard_shortcuts(ctx, state);

    // Drift the orbit camera by its current angular velocity. Runs every
    // frame regardless of view mode so a quick toggle Flat → Skybox doesn't
    // surprise the user with stale momentum, and so inertia decays even if
    // the user has the panel focused.
    let dt = (state.frame_ms_ema * 0.001).clamp(0.001, 0.1);
    apply_orbit_inertia(&mut state.orbit_camera, dt);

    // Web preview banner. Sets expectations for visitors and routes serious
    // users to the desktop binary. Native users don't need this — they're
    // already on the full product.
    #[cfg(target_arch = "wasm32")]
    egui::TopBottomPanel::top("web_preview_banner")
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.add(egui::Label::new(
                    egui::RichText::new("Erebus Web Preview")
                        .strong()
                        .color(egui::Color32::from_rgb(0xff, 0xa0, 0xe0)),
                ));
                ui.label(
                    egui::RichText::new(
                        "— low-fidelity in-browser demo. The desktop app is faster, \
                         exports up to 8K equirect / 4K cubemap / EXR.",
                    )
                    .weak(),
                );
                ui.hyperlink_to(
                    egui::RichText::new("Download for desktop")
                        .strong()
                        .color(egui::Color32::from_rgb(0xc0, 0xa0, 0xff)),
                    "https://github.com/Darian-Frey/Erebus/releases",
                );
            });
        });

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

            // Mouse drag + scroll wheel drive the orbit camera in skybox mode.
            // The response is registered before we build the uniforms so the
            // updated yaw/pitch/fov land in this frame's composite pass —
            // dragging feels instant.
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            if state.view_mode == ViewMode::Skybox {
                handle_orbit_input(ui, &response, &mut state.orbit_camera, dt);
            }

            // Offscreen HDR is always rendered at a fixed 2:1 aspect (the
            // natural equirect ratio), independent of canvas size. Skybox
            // composite samples it via reconstructed equirect UV — canvas-
            // derived sizing causes pole-compression artefacts at high pitch.
            //
            // Hero shot bumps to a higher base for clean output.
            // `preview_scale` multiplies the long axis; `interacting` halves
            // it during drags so slider scrubbing stays responsive even at
            // a high base.
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
            let base_axis = if hero { HDR_HERO_LONG_AXIS } else { HDR_BASE_LONG_AXIS };
            let tw = (base_axis * scale).round().max(2.0);
            let th = (tw * 0.5).round().max(2.0);
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

            // Skybox-camera params are written every frame regardless of
            // view_mode (cheap), but they're injected AFTER the offscreen
            // hash so dragging the camera doesn't bust the freeze — the
            // composite pass is the only thing that re-reads them, and it
            // runs every frame anyway.
            let aspect = (rect.width() / rect.height().max(1.0)).max(0.01);
            post.view_mode = match state.view_mode {
                ViewMode::Flat => 0,
                ViewMode::Skybox => 1,
            };
            post.yaw = state.orbit_camera.yaw_rad;
            post.pitch = state.orbit_camera.pitch_rad;
            post.fov_y = state.orbit_camera.fov_y_deg.to_radians();
            post.aspect = aspect;

            // Hash everything that affects the *offscreen* render. The
            // skybox camera fields are deliberately excluded — they only
            // affect composite, which always re-runs.
            let mut hasher = DefaultHasher::new();
            bytemuck::bytes_of(&nebula).hash(&mut hasher);
            bytemuck::bytes_of(&lighting).hash(&mut hasher);
            bytemuck::bytes_of(&starfield).hash(&mut hasher);
            // Hash only the offscreen-relevant prefix of post — exposure,
            // tonemap, bloom, grade, dither. The view_mode/yaw/pitch/fov/
            // aspect tail is composite-only.
            let post_offscreen_bytes =
                &bytemuck::bytes_of(&post)[..(8 * 4 + 2 * 4 + 2 * 4)];
            post_offscreen_bytes.hash(&mut hasher);
            state.seed.hash(&mut hasher);
            target_size.0.hash(&mut hasher);
            target_size.1.hash(&mut hasher);
            let render_hash = hasher.finish();

            let skip_prepare = state.last_rendered_hash == Some(render_hash);

            // Live preview always renders the full equirect into HDR; the
            // composite pass handles flat-vs-skybox display. Cube-face mode
            // was removed when skybox subsumed it — export still uses
            // MODE_CUBEMAP via its own FrameUniforms construction.
            let frame = FrameUniforms {
                resolution: [target_size.0 as f32, target_size.1 as f32],
                time: state.time,
                exposure: post.exposure,
                seed: state.seed,
                frame_index: state.frame_index,
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

/// Mouse-drag yaw/pitch + scroll-wheel FOV. Pulled out as a free function
/// so the central-panel closure stays readable. `dt` is the previous frame's
/// duration in seconds — used to convert drag-pixels to angular velocity for
/// the inertia hand-off when the user releases.
fn handle_orbit_input(
    ui: &egui::Ui,
    response: &egui::Response,
    cam: &mut OrbitCamera,
    dt: f32,
) {
    if response.dragged() {
        let delta = response.drag_delta();
        // 0.25°/pixel — feels right for a 70° default FOV.
        let sens = 0.25_f32.to_radians();
        let dyaw = -delta.x * sens;
        let dpitch = -delta.y * sens;
        cam.yaw_rad += dyaw;
        cam.pitch_rad += dpitch;

        // Cache the per-frame motion as an angular velocity so a fast flick
        // releases into momentum. Divide by dt to recover rad/s. Skip when
        // dt is too small to avoid spikes.
        if dt > 1e-4 {
            cam.yaw_rate = dyaw / dt;
            cam.pitch_rate = dpitch / dt;
        }
    } else if response.drag_stopped() {
        // Release frame: keep the velocity we cached on the last drag tick
        // (already set above) so apply_orbit_inertia carries it forward.
    }

    // Clamp pitch + wrap yaw every frame in case the deltas above pushed us
    // out of range. Done unconditionally so inertia-driven motion is also
    // bounded.
    let max_pitch = 89.0_f32.to_radians();
    cam.pitch_rad = cam.pitch_rad.clamp(-max_pitch, max_pitch);
    cam.yaw_rad = cam.yaw_rad.rem_euclid(std::f32::consts::TAU);

    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            // Negative scroll (wheel down) zooms out; positive zooms in.
            cam.fov_y_deg = (cam.fov_y_deg - scroll * 0.05).clamp(30.0, 110.0);
        }
    }

    // While actively dragging, kill any leftover inertia from a previous
    // release so the user has full control. Also kills it if the cursor is
    // hovering but not dragging (i.e. they're aiming a new flick).
    if response.dragged() {
        // Velocity is already overwritten this frame; nothing to do.
    } else if response.clicked() {
        cam.yaw_rate = 0.0;
        cam.pitch_rate = 0.0;
    }
}

/// Decay yaw/pitch velocity exponentially and integrate into yaw/pitch. A
/// 5 Hz half-life feels lively without being unmanageable.
fn apply_orbit_inertia(cam: &mut OrbitCamera, dt: f32) {
    // Snap to zero when below an imperceptible threshold so we don't spend
    // forever decaying a dead motion.
    let dead = 0.01_f32; // rad/s
    if cam.yaw_rate.abs() < dead && cam.pitch_rate.abs() < dead {
        cam.yaw_rate = 0.0;
        cam.pitch_rate = 0.0;
        return;
    }

    cam.yaw_rad += cam.yaw_rate * dt;
    cam.pitch_rad += cam.pitch_rate * dt;
    let max_pitch = 89.0_f32.to_radians();
    cam.pitch_rad = cam.pitch_rad.clamp(-max_pitch, max_pitch);
    cam.yaw_rad = cam.yaw_rad.rem_euclid(std::f32::consts::TAU);

    // 5 Hz exponential decay → 50 % loss every 200 ms.
    let half_life_s = 0.2_f32;
    let decay = 0.5_f32.powf(dt / half_life_s);
    cam.yaw_rate *= decay;
    cam.pitch_rate *= decay;
}

/// Global keyboard shortcuts for the skybox preview. Active regardless of
/// view mode for `Space` (toggle); the others only do anything in skybox.
fn handle_keyboard_shortcuts(ctx: &egui::Context, state: &mut State) {
    use egui::Key;

    ctx.input(|i| {
        // Space toggles Flat ↔ Skybox.
        if i.key_pressed(Key::Space) {
            state.view_mode = match state.view_mode {
                ViewMode::Flat => ViewMode::Skybox,
                ViewMode::Skybox => ViewMode::Flat,
            };
        }

        if state.view_mode != ViewMode::Skybox {
            return;
        }

        // R resets the camera.
        if i.key_pressed(Key::R) {
            state.orbit_camera = OrbitCamera::default();
        }

        // Arrow keys nudge yaw/pitch by 5°.
        let nudge = 5.0_f32.to_radians();
        if i.key_pressed(Key::ArrowLeft)  { state.orbit_camera.yaw_rad -= nudge; }
        if i.key_pressed(Key::ArrowRight) { state.orbit_camera.yaw_rad += nudge; }
        if i.key_pressed(Key::ArrowUp)    { state.orbit_camera.pitch_rad += nudge; }
        if i.key_pressed(Key::ArrowDown)  { state.orbit_camera.pitch_rad -= nudge; }

        // [ / ] adjust FOV by 5°.
        if i.key_pressed(Key::OpenBracket) {
            state.orbit_camera.fov_y_deg =
                (state.orbit_camera.fov_y_deg - 5.0).clamp(30.0, 110.0);
        }
        if i.key_pressed(Key::CloseBracket) {
            state.orbit_camera.fov_y_deg =
                (state.orbit_camera.fov_y_deg + 5.0).clamp(30.0, 110.0);
        }

        // Keyboard nudges should also kill inertia so the new direction
        // takes immediately.
        if i.key_pressed(Key::ArrowLeft)
            || i.key_pressed(Key::ArrowRight)
            || i.key_pressed(Key::ArrowUp)
            || i.key_pressed(Key::ArrowDown)
            || i.key_pressed(Key::R)
        {
            state.orbit_camera.yaw_rate = 0.0;
            state.orbit_camera.pitch_rate = 0.0;
        }
    });

    let max_pitch = 89.0_f32.to_radians();
    state.orbit_camera.pitch_rad =
        state.orbit_camera.pitch_rad.clamp(-max_pitch, max_pitch);
    state.orbit_camera.yaw_rad = state.orbit_camera.yaw_rad.rem_euclid(std::f32::consts::TAU);
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
            // Still re-upload the post UBO so skybox camera changes
            // (yaw/pitch/fov) reach the composite pass even though the
            // offscreen render is frozen. Without this, drag/scroll updates
            // the panel readout but the canvas reads stale uniforms.
            if let Some(r) = resources.get::<ErebusRenderer>() {
                r.upload_post(queue, self.post);
            }
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
