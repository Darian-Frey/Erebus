// egui visuals: dark, slightly desaturated, faint synthwave tint on accents.

pub fn install(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = 6.0.into();
    visuals.widgets.noninteractive.rounding = 4.0.into();
    visuals.widgets.inactive.rounding = 4.0.into();
    visuals.widgets.hovered.rounding = 4.0.into();
    visuals.widgets.active.rounding = 4.0.into();
    visuals.selection.bg_fill = egui::Color32::from_rgb(0xc0, 0x40, 0xa0);
    ctx.set_visuals(visuals);
}
