// Phase 2 panel surface: Frame + Nebula sliders. More groups land in
// Phase 3 (Lighting), Phase 4 (Starfield), Phase 5 (PostFX).

use crate::app::state::State;

pub fn controls(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("Erebus");
    ui.label(format!("t = {:.2}s   frame {}", state.time, state.frame_index));
    let fps_color = if state.fps_ema >= 50.0 {
        egui::Color32::from_rgb(0x80, 0xff, 0x80)
    } else if state.fps_ema >= 24.0 {
        egui::Color32::from_rgb(0xff, 0xc0, 0x60)
    } else {
        egui::Color32::from_rgb(0xff, 0x70, 0x70)
    };
    ui.colored_label(
        fps_color,
        format!(
            "{:.1} fps  ({:.1} ms/frame)",
            state.fps_ema, state.frame_ms_ema
        ),
    );
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        frame_group(ui, state);
        ui.separator();
        nebula_group(ui, state);
        ui.separator();
        lighting_group(ui, state);
        ui.separator();
        ui.label(
            egui::RichText::new("Edit shaders/ on disk — pipelines hot-reload.")
                .italics()
                .weak(),
        );
    });
}

fn lighting_group(ui: &mut egui::Ui, state: &mut State) {
    let l = &mut state.lighting;
    egui::CollapsingHeader::new("Lighting")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("active lights");
                ui.add(egui::Slider::new(&mut l.count, 0..=4));
            });
            slider_u32(ui, "shadow steps", &mut l.shadow_steps, 1..=12);
            slider(
                ui,
                "ambient emission",
                &mut l.ambient_emission,
                0.0..=1.5,
            );

            ui.separator();
            for (i, light) in l.lights.iter_mut().enumerate() {
                let active = (i as u32) < l.count;
                ui.add_enabled_ui(active, |ui| {
                    egui::CollapsingHeader::new(format!("Light {}", i + 1))
                        .default_open(active)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("position");
                                ui.add(egui::DragValue::new(&mut light.position[0]).speed(0.01).prefix("x "));
                                ui.add(egui::DragValue::new(&mut light.position[1]).speed(0.01).prefix("y "));
                                ui.add(egui::DragValue::new(&mut light.position[2]).speed(0.01).prefix("z "));
                            });
                            ui.horizontal(|ui| {
                                ui.label("colour");
                                ui.color_edit_button_rgb(&mut light.color);
                            });
                            ui.horizontal(|ui| {
                                ui.label("intensity");
                                ui.add(egui::Slider::new(&mut light.intensity, 0.0..=10.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("falloff");
                                ui.add(egui::Slider::new(&mut light.falloff, 0.0..=4.0));
                            });
                        });
                });
            }
        });
}

fn frame_group(ui: &mut egui::Ui, state: &mut State) {
    egui::CollapsingHeader::new("Frame")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("exposure (stops)");
                ui.add(egui::Slider::new(&mut state.exposure, -4.0..=4.0));
            });
            ui.horizontal(|ui| {
                ui.label("preview scale");
                ui.add(egui::Slider::new(&mut state.preview_scale, 0.25..=1.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("seed");
                ui.add(egui::DragValue::new(&mut state.seed).speed(1.0));
                if ui.button("shuffle").clicked() {
                    state.seed = state.seed.wrapping_mul(0x9E37_79B1).wrapping_add(1);
                }
            });
        });
}

fn nebula_group(ui: &mut egui::Ui, state: &mut State) {
    let n = &mut state.nebula;
    egui::CollapsingHeader::new("Nebula")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Shape").strong());
            slider(ui, "density scale", &mut n.density_scale, 0.1..=8.0);
            slider_u32(ui, "octaves (density)", &mut n.octaves_density, 1..=8);
            slider(ui, "lacunarity", &mut n.lacunarity, 1.5..=2.5);
            slider(ui, "gain", &mut n.gain, 0.2..=0.7);
            slider(ui, "ridged blend", &mut n.ridged_blend, 0.0..=1.0);

            ui.separator();
            ui.label(egui::RichText::new("Domain warp").strong());
            slider(ui, "warp strength", &mut n.warp_strength, 0.0..=4.0);
            slider_u32(ui, "octaves (warp)", &mut n.octaves_warp, 0..=6);

            ui.separator();
            ui.label(egui::RichText::new("March").strong());
            slider_u32(ui, "steps", &mut n.steps, 16..=256);
            slider(ui, "march length", &mut n.march_length, 0.25..=4.0);
            slider(
                ui,
                "transmittance cutoff",
                &mut n.transmittance_cutoff,
                0.0..=0.1,
            );
            slider(
                ui,
                "step density bias",
                &mut n.step_density_bias,
                0.5..=3.0,
            );

            ui.separator();
            ui.label(egui::RichText::new("Scattering").strong());
            slider(ui, "extinction (σₑ)", &mut n.sigma_e, 0.1..=8.0);
            slider(ui, "albedo", &mut n.albedo, 0.0..=1.0);
            slider(ui, "HG anisotropy (g)", &mut n.hg_g, -0.9..=0.9);
            slider(ui, "density curve (γ)", &mut n.density_curve, 0.25..=2.0);
        });
}

fn slider(ui: &mut egui::Ui, label: &str, v: &mut f32, range: std::ops::RangeInclusive<f32>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Slider::new(v, range));
    });
}

fn slider_u32(
    ui: &mut egui::Ui,
    label: &str,
    v: &mut u32,
    range: std::ops::RangeInclusive<u32>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Slider::new(v, range));
    });
}
