// UI panels: Nebula, Starfield, Lighting, PostFX, Export, Presets.
// Phase 1 ships only a debug panel — the rest are populated as their
// underlying passes come online.

use crate::app::state::State;

pub fn controls(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("Erebus");
    ui.label(format!("t = {:.2}s", state.time));
    ui.separator();

    ui.collapsing("Frame", |ui| {
        ui.horizontal(|ui| {
            ui.label("exposure (stops)");
            ui.add(egui::Slider::new(&mut state.exposure, -4.0..=4.0));
        });
        ui.horizontal(|ui| {
            ui.label("seed");
            ui.add(egui::DragValue::new(&mut state.seed).speed(1.0));
            if ui.button("shuffle").clicked() {
                state.seed = state.seed.wrapping_mul(0x9E37_79B1).wrapping_add(1);
            }
        });
    });

    ui.separator();
    ui.label(egui::RichText::new("Phase 1 placeholder.").italics().weak());
    ui.label("Edit shaders/ on disk — pipelines hot-reload.");
}
