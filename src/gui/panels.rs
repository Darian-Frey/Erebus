// Phase 2 panel surface: Frame + Nebula sliders. More groups land in
// Phase 3 (Lighting), Phase 4 (Starfield), Phase 5 (PostFX).

use crate::app::state::{OrbitCamera, QualityTier, State, ViewMode};
use crate::export::{ExportFormat, ExportKind, ExportRequest};
use crate::preset::{PresetAction, ShippedPreset};

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
        preset_group(ui, state);
        ui.separator();
        frame_group(ui, state);
        ui.separator();
        post_group(ui, state);
        ui.separator();
        nebula_group(ui, state);
        ui.separator();
        lighting_group(ui, state);
        ui.separator();
        starfield_group(ui, state);
        ui.separator();
        // Performance is native-only — the bench would freeze the tab for
        // ~10 s and isn't useful in-browser. Export ships on both targets,
        // but the wasm version is restricted to equirect PNG and triggers a
        // browser download instead of using a native file dialog.
        #[cfg(not(target_arch = "wasm32"))]
        {
            performance_group(ui, state);
            ui.separator();
        }
        export_group(ui, state);
        ui.separator();
        ui.label(
            egui::RichText::new("Edit shaders/ on disk — pipelines hot-reload.")
                .italics()
                .weak(),
        );
    });
}

fn preset_group(ui: &mut egui::Ui, state: &mut State) {
    egui::CollapsingHeader::new("Preset")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("name");
                ui.add(egui::TextEdit::singleline(&mut state.preset_name).desired_width(180.0));
            });

            ui.horizontal(|ui| {
                if ui.button("Save…").clicked() {
                    state.pending_preset = Some(PresetAction::SaveToFile);
                }
                if ui.button("Load…").clicked() {
                    state.pending_preset = Some(PresetAction::LoadFromFile);
                }
            });

            ui.separator();
            ui.label(egui::RichText::new("Shipped").strong());
            ui.horizontal_wrapped(|ui| {
                for shipped in ShippedPreset::ALL {
                    if ui.button(shipped.label()).clicked() {
                        state.pending_preset = Some(PresetAction::LoadShipped(*shipped));
                    }
                }
            });

            if let Some(msg) = &state.last_preset_status {
                ui.label(egui::RichText::new(msg).weak());
            }
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn performance_group(ui: &mut egui::Ui, state: &mut State) {
    egui::CollapsingHeader::new("Performance")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "Runs ~5 timed configs at varying resolution × march steps. \
                     Uses your current lighting / starfield / scattering settings. \
                     UI freezes for the duration.",
                )
                .italics()
                .weak(),
            );

            ui.horizontal(|ui| {
                let busy = state.bench_running || state.pending_bench;
                if ui
                    .add_enabled(!busy, egui::Button::new("Run benchmark"))
                    .clicked()
                {
                    state.pending_bench = true;
                }
                if busy {
                    ui.spinner();
                }
            });

            if !state.bench_results.is_empty() {
                egui::Grid::new("bench_results_grid")
                    .num_columns(4)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("config").strong());
                        ui.label(egui::RichText::new("size").strong());
                        ui.label(egui::RichText::new("ms").strong());
                        ui.label(egui::RichText::new("fps").strong());
                        ui.end_row();

                        for r in &state.bench_results {
                            ui.label(&r.label);
                            ui.label(format!("{}×{}", r.width, r.height));
                            ui.label(format!("{:.2}", r.ms_median));
                            let fps = r.fps();
                            let color = if fps >= 60.0 {
                                egui::Color32::from_rgb(0x80, 0xff, 0x80)
                            } else if fps >= 30.0 {
                                egui::Color32::from_rgb(0xff, 0xc0, 0x60)
                            } else {
                                egui::Color32::from_rgb(0xff, 0x70, 0x70)
                            };
                            ui.colored_label(color, format!("{fps:.1}"));
                            ui.end_row();
                        }
                    });
            }
        });
}

fn export_group(ui: &mut egui::Ui, state: &mut State) {
    egui::CollapsingHeader::new("Export")
        .default_open(false)
        .show(ui, |ui| {
            // Wasm restriction: equirect PNG only. Cubemap (6 downloads) and
            // EXR (~10× file size) are deferred to a later phase; pin the
            // state here so the user can't pick an invalid combination.
            #[cfg(target_arch = "wasm32")]
            {
                state.export_kind = ExportKind::Equirect;
                state.export_format = ExportFormat::Png;
                ui.label(
                    egui::RichText::new(
                        "Equirect PNG → browser download. Cubemap and EXR are desktop-only.",
                    )
                    .weak(),
                );
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.horizontal(|ui| {
                    ui.label("kind");
                    egui::ComboBox::from_id_salt("export_kind")
                        .selected_text(state.export_kind.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut state.export_kind,
                                ExportKind::Equirect,
                                ExportKind::Equirect.label(),
                            );
                            ui.selectable_value(
                                &mut state.export_kind,
                                ExportKind::Cubemap,
                                ExportKind::Cubemap.label(),
                            );
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("format");
                    egui::ComboBox::from_id_salt("export_format")
                        .selected_text(state.export_format.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut state.export_format,
                                ExportFormat::Png,
                                ExportFormat::Png.label(),
                            );
                            // EXR currently only meaningful for equirect — cubemap
                            // EXR is doable but defers to Phase 7.5; gate the
                            // option to avoid silently producing invalid output.
                            ui.add_enabled_ui(state.export_kind == ExportKind::Equirect, |ui| {
                                ui.selectable_value(
                                    &mut state.export_format,
                                    ExportFormat::Exr,
                                    ExportFormat::Exr.label(),
                                );
                            });
                        });
                });
                // If the user switched to cubemap with EXR previously selected,
                // bounce them back to PNG.
                if state.export_kind == ExportKind::Cubemap
                    && state.export_format == ExportFormat::Exr
                {
                    state.export_format = ExportFormat::Png;
                }
            }

            ui.horizontal(|ui| {
                // Wasm caps at 1K. 2K at Quality-tier (128 march × 6 shadow
                // × 3 star layers + bloom) still triggers Chrome's GPU
                // watchdog (TDR) on integrated GPUs, which crashes the
                // device and blanks the tab. 1K renders cleanly in a few
                // seconds and the user can upscale offline if they need
                // more pixels. Higher resolutions ship in the desktop build.
                #[cfg(target_arch = "wasm32")]
                let labels: &[(u32, &str)] = &[(1024, "1K")];
                #[cfg(not(target_arch = "wasm32"))]
                let labels: &[(u32, &str)] = match state.export_kind {
                    ExportKind::Equirect => &[(1024, "1K"), (2048, "2K"), (4096, "4K"), (8192, "8K")],
                    // Cubemap face sizes — each face is square; 4K-per-face
                    // = 24K total cubemap which is more than most pipelines
                    // need.
                    ExportKind::Cubemap => &[(512, "512"), (1024, "1K"), (2048, "2K"), (4096, "4K")],
                };
                #[cfg(target_arch = "wasm32")]
                if state.export_width != 1024 {
                    state.export_width = 1024;
                }
                ui.label(match state.export_kind {
                    ExportKind::Equirect => "width",
                    ExportKind::Cubemap => "face size",
                });
                let current = labels
                    .iter()
                    .find(|(w, _)| *w == state.export_width)
                    .map(|(_, l)| *l)
                    .unwrap_or("custom");
                egui::ComboBox::from_id_salt("export_width")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (w, l) in labels {
                            ui.selectable_value(&mut state.export_width, *w, *l);
                        }
                    });
                let dims = match state.export_kind {
                    ExportKind::Equirect => format!("→ {}×{}", state.export_width, state.export_width / 2),
                    ExportKind::Cubemap => {
                        format!("→ 6 × {0}×{0}", state.export_width)
                    }
                };
                ui.label(dims);
            });

            ui.horizontal(|ui| {
                #[cfg(target_arch = "wasm32")]
                let busy = state.pending_export.is_some()
                    || state.pending_export_job.is_some();
                #[cfg(not(target_arch = "wasm32"))]
                let busy = state.pending_export.is_some();
                let label = match (state.export_kind, state.export_format) {
                    (ExportKind::Equirect, ExportFormat::Png) => "Export PNG…",
                    (ExportKind::Equirect, ExportFormat::Exr) => "Export EXR…",
                    (ExportKind::Cubemap, _) => "Export cube faces…",
                };
                if ui.add_enabled(!busy, egui::Button::new(label)).clicked() {
                    state.pending_export = Some(ExportRequest {
                        format: state.export_format,
                        kind: state.export_kind,
                        width: state.export_width,
                        path: None,
                    });
                    state.last_export_status = Some("rendering…".to_string());
                }
                if busy {
                    ui.spinner();
                }
            });

            if let Some(msg) = &state.last_export_status {
                ui.label(egui::RichText::new(msg).weak());
            }
        });
}

fn post_group(ui: &mut egui::Ui, state: &mut State) {
    let p = &mut state.post;
    egui::CollapsingHeader::new("PostFX")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Tonemap").strong());
            ui.horizontal(|ui| {
                ui.label("curve");
                let labels = ["AgX", "ACES Fitted", "Reinhard"];
                let mut mode = p.tonemap_mode as usize;
                egui::ComboBox::from_id_salt("tonemap_mode")
                    .selected_text(labels.get(mode).copied().unwrap_or("?"))
                    .show_ui(ui, |ui| {
                        for (i, l) in labels.iter().enumerate() {
                            ui.selectable_value(&mut mode, i, *l);
                        }
                    });
                p.tonemap_mode = mode as u32;
            });
            slider_tip(
                ui, "exposure (stops)", &mut p.exposure, -4.0..=4.0,
                "EV stops applied right before tonemap. Each ±1 doubles / halves the linear scene radiance.",
            );

            ui.separator();
            ui.label(egui::RichText::new("Bloom").strong());
            slider(ui, "intensity", &mut p.bloom_intensity, 0.0..=2.0);
            slider(ui, "threshold", &mut p.bloom_threshold, 0.0..=4.0);
            slider(ui, "radius", &mut p.bloom_radius, 0.5..=3.0);

            ui.separator();
            ui.label(egui::RichText::new("Grade").strong());
            slider(ui, "saturation", &mut p.grade_saturation, 0.0..=2.0);
            slider(ui, "contrast", &mut p.grade_contrast, 0.5..=1.5);

            ui.separator();
            slider(ui, "deband amount", &mut p.deband_amount, 0.0..=2.0);
        });
}

fn starfield_group(ui: &mut egui::Ui, state: &mut State) {
    let s = &mut state.starfield;
    egui::CollapsingHeader::new("Starfield")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Distribution").strong());
            slider_tip(
                ui, "density (grid scale)", &mut s.density, 20.0..=200.0,
                "Grid scale of layer 0. Doubles each parallax layer. Higher = more, smaller stars.",
            );
            slider(ui, "brightness", &mut s.brightness, 0.0..=4.0);
            slider_u32(ui, "parallax layers", &mut s.layers, 1..=3);
            slider_tip(
                ui, "IMF exponent", &mut s.imf_exponent, 1.0..=8.0,
                "mag = pow(rand, exp). 5 → ~95 % dim stars (realistic); 1 → uniform brightness across the field.",
            );

            ui.separator();
            ui.label(egui::RichText::new("Galactic plane").strong());
            slider(ui, "strength", &mut s.galactic_strength, 0.0..=4.0);
            slider(ui, "width", &mut s.galactic_width, 0.05..=1.0);
            ui.horizontal(|ui| {
                ui.label("plane normal");
                ui.add(egui::DragValue::new(&mut s.galactic_dir[0]).speed(0.01).prefix("x "));
                ui.add(egui::DragValue::new(&mut s.galactic_dir[1]).speed(0.01).prefix("y "));
                ui.add(egui::DragValue::new(&mut s.galactic_dir[2]).speed(0.01).prefix("z "));
            });

            ui.separator();
            ui.label(egui::RichText::new("Colour (Kelvin)").strong());
            slider_tip(
                ui, "T min", &mut s.temperature_min, 1500.0..=6000.0,
                "Coolest stellar temperature in the distribution. 2700 K ≈ M-class red dwarfs; 3500 K ≈ orange dwarfs.",
            );
            slider_tip(
                ui, "T max", &mut s.temperature_max, 8000.0..=40000.0,
                "Hottest stellar temperature. 10 000 K ≈ A-class white; 30 000 K ≈ O/B-class blue giants.",
            );

            ui.separator();
            ui.label(egui::RichText::new("PSF / diffraction").strong());
            slider(ui, "PSF threshold", &mut s.psf_threshold, 0.0..=1.0);
            slider(ui, "PSF intensity", &mut s.psf_intensity, 0.0..=2.0);
            slider(ui, "spike length", &mut s.spike_length, 0.001..=0.05);
            slider_u32(ui, "spike count", &mut s.spike_count, 4..=8);
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
            slider_u32_tip(
                ui, "shadow steps", &mut l.shadow_steps, 1..=12,
                "Per-light shadow march steps; 4 = Heckel's lower bound, 6+ for export quality. Cost is N × shadow_steps per main step.",
            );
            slider_tip(
                ui, "ambient emission", &mut l.ambient_emission, 0.0..=1.5,
                "Isotropic self-glow floor. 0 = pure lit-only (Horsehead silhouette); 1+ = Phase-2 self-glow look. Re-uses the gradient LUT colour.",
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
                ui.label("quality").on_hover_text(
                    "Snap to a known-good performance/quality tier. \
                     Individual sliders can still be tweaked afterward.",
                );
                for tier in [
                    QualityTier::Draft,
                    QualityTier::Preview,
                    QualityTier::Quality,
                    QualityTier::Export,
                ] {
                    if ui
                        .button(tier.label())
                        .on_hover_text(tier.tooltip())
                        .clicked()
                    {
                        apply_quality(state, tier);
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("view").on_hover_text(
                    "Live-preview projection. Flat shows the equirect 2:1 \
                     unwrap directly (with pole stretching). Skybox lets you \
                     drag in the canvas to look around the cached scene — \
                     dragging is free, no nebula re-march. Scroll over the \
                     canvas to adjust FOV. Export is unaffected.",
                );
                egui::ComboBox::from_id_salt("view_mode")
                    .selected_text(state.view_mode.label())
                    .show_ui(ui, |ui| {
                        for m in [ViewMode::Flat, ViewMode::Skybox] {
                            ui.selectable_value(&mut state.view_mode, m, m.label());
                        }
                    });
                if state.view_mode == ViewMode::Skybox
                    && ui.button("Reset view").clicked()
                {
                    state.orbit_camera = OrbitCamera::default();
                }
            });
            if state.view_mode == ViewMode::Skybox {
                ui.horizontal(|ui| {
                    ui.label("fov").on_hover_text(
                        "Vertical field of view, 30°–110°. Scroll inside the canvas, \
                         use [ / ], or drag this slider.",
                    );
                    ui.add(
                        egui::Slider::new(&mut state.orbit_camera.fov_y_deg, 30.0..=110.0)
                            .suffix("°")
                            .step_by(1.0),
                    );
                });
                ui.label(
                    egui::RichText::new(format!(
                        "yaw {:>5.0}°  pitch {:>4.0}°",
                        state.orbit_camera.yaw_rad.to_degrees(),
                        state.orbit_camera.pitch_rad.to_degrees(),
                    ))
                    .weak()
                    .monospace(),
                );
                ui.label(
                    egui::RichText::new(
                        "drag canvas to look · scroll to zoom · Space toggles · R resets · \
                         arrows nudge · [ ] FOV",
                    )
                    .weak()
                    .small(),
                );
            }
            ui.horizontal(|ui| {
                ui.label("preview scale");
                ui.add(egui::Slider::new(&mut state.preview_scale, 0.25..=1.0).step_by(0.05));
                if state.interacting {
                    ui.label(egui::RichText::new("(auto ½)").weak());
                }
            });
            ui.horizontal(|ui| {
                ui.label("seed");
                ui.add(egui::DragValue::new(&mut state.seed).speed(1.0));
                if ui.button("shuffle").clicked() {
                    state.seed = state.seed.wrapping_mul(0x9E37_79B1).wrapping_add(1);
                }
            });
            #[cfg(target_arch = "wasm32")]
            {
                let label = if state.hero_shot {
                    "Stop hero shot"
                } else {
                    "Render hero shot"
                };
                if ui
                    .button(label)
                    .on_hover_text(
                        "Render the current composition once at Quality settings \
                         (full resolution, 128 march steps, full bloom). The hero \
                         frame is then frozen on the canvas so subsequent frames \
                         cost almost nothing. Stays on until you move a slider.",
                    )
                    .clicked()
                {
                    state.hero_shot = !state.hero_shot;
                    // Force a re-render at the new (or restored) settings.
                    state.last_rendered_hash = None;
                }
            }
        });
}

fn apply_quality(state: &mut State, tier: QualityTier) {
    match tier {
        QualityTier::Draft => {
            state.preview_scale = 0.5;
            state.nebula.steps = 64;
            state.lighting.shadow_steps = 4;
            state.starfield.layers = 1;
        }
        QualityTier::Preview => {
            state.preview_scale = 1.0;
            state.nebula.steps = 96;
            state.lighting.shadow_steps = 4;
            state.starfield.layers = 3;
        }
        QualityTier::Quality => {
            state.preview_scale = 1.0;
            state.nebula.steps = 128;
            state.lighting.shadow_steps = 6;
            state.starfield.layers = 3;
        }
        QualityTier::Export => {
            state.preview_scale = 1.0;
            state.nebula.steps = 256;
            state.lighting.shadow_steps = 8;
            state.starfield.layers = 3;
        }
    }
}

fn nebula_group(ui: &mut egui::Ui, state: &mut State) {
    let n = &mut state.nebula;
    egui::CollapsingHeader::new("Nebula")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Shape").strong());
            slider_tip(
                ui, "density scale", &mut n.density_scale, 0.1..=8.0,
                "Frequency of the noise field. Higher = finer detail, smaller cells. Re-bakes the noise volume.",
            );
            slider_u32_tip(
                ui, "octaves (density)", &mut n.octaves_density, 1..=8,
                "Number of FBM octaves baked into the noise volume. Past 8 the gain-0.5 contribution is < 0.2 %. Re-bake on change.",
            );
            slider_tip(
                ui, "lacunarity", &mut n.lacunarity, 1.5..=2.5,
                "Frequency multiplier between octaves. 2.02 (default) breaks axis-aligned beating that pure 2.0 produces. Re-bake on change.",
            );
            slider_tip(
                ui, "gain", &mut n.gain, 0.2..=0.7,
                "Amplitude multiplier between octaves. 0.5 is the universal default across cloud/nebula references. Re-bake on change.",
            );
            slider_tip(
                ui, "ridged blend", &mut n.ridged_blend, 0.0..=1.0,
                "0 = wispy gas (smooth FBM); 1 = filaments / lightning (ridged FBM). 0.5 = trifid-style mix.",
            );

            ui.separator();
            ui.label(egui::RichText::new("Domain warp").strong());
            slider_tip(
                ui, "warp strength", &mut n.warp_strength, 0.0..=4.0,
                "Magnitude of the FBM displacement applied to the sample position. 0 = flat clouds, 1.5 = trifid tendrils, 4+ = chaos.",
            );
            slider_u32(ui, "octaves (warp)", &mut n.octaves_warp, 0..=6);

            ui.separator();
            ui.label(egui::RichText::new("March").strong());
            slider_u32_tip(
                ui, "steps", &mut n.steps, 16..=256,
                "Raymarch sample count per pixel. 64 preview, 128 quality, 256 export. Linear cost.",
            );
            slider(ui, "march length", &mut n.march_length, 0.25..=4.0);
            slider_tip(
                ui, "transmittance cutoff", &mut n.transmittance_cutoff, 0.0..=0.1,
                "Early-out threshold; stops marching once the residual transmittance falls below this. 0.01 saves 30–50 % in dense regions.",
            );
            slider_tip(
                ui, "step density bias", &mut n.step_density_bias, 0.5..=3.0,
                "dt = base * max(0.25, bias - density). Higher bias = denser regions take smaller steps; halves visible banding for free.",
            );

            ui.separator();
            ui.label(egui::RichText::new("Scattering").strong());
            slider_tip(
                ui, "extinction (σₑ)", &mut n.sigma_e, 0.1..=8.0,
                "Beer–Lambert extinction per unit density per scene unit. ~0.3 = wispy haze, 1.5 = default, ~6 = bright Trifid-style core.",
            );
            slider_tip(
                ui, "albedo", &mut n.albedo, 0.0..=1.0,
                "σs / σe — fraction of extinguished light that re-scatters. 0.6 is Space Engine's default; lower for darker dust lanes.",
            );
            slider_tip(
                ui, "HG anisotropy (g)", &mut n.hg_g, -0.9..=0.9,
                "Henyey–Greenstein phase function. 0 = isotropic clouds, 0.6 = forward-scatter dust (default), -0.3 = back-scatter rim.",
            );
            slider_tip(
                ui, "density curve (γ)", &mut n.density_curve, 0.25..=2.0,
                "pow(d, γ) before the gradient LUT lookup. 0.5 (sqrt) lifts wispy tails; 1.0 = linear; 2.0 hides the tails entirely.",
            );
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

fn slider_tip(
    ui: &mut egui::Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    tip: &str,
) {
    ui.horizontal(|ui| {
        ui.label(label).on_hover_text(tip);
        ui.add(egui::Slider::new(v, range)).on_hover_text(tip);
    });
}

fn slider_u32_tip(
    ui: &mut egui::Ui,
    label: &str,
    v: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    tip: &str,
) {
    ui.horizontal(|ui| {
        ui.label(label).on_hover_text(tip);
        ui.add(egui::Slider::new(v, range)).on_hover_text(tip);
    });
}
