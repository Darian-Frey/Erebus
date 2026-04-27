//! Erebus — procedural nebula and starfield generator.
//!
//! Library crate shared by the native binary (`src/main.rs`) and the
//! WebAssembly target. The WASM entry point [`start`] is exposed via
//! `wasm_bindgen` and called from `assets/web/index.html`.

pub mod app;
pub mod gui;
pub mod render;
pub mod export;
pub mod preset;
pub mod noise;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WebAssembly entry point. Resolves the canvas element by id, hands off to
/// `eframe::WebRunner` with the same `ErebusApp` the native binary uses.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: String) -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;

    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    let canvas = web_sys::window()
        .ok_or_else(|| JsValue::from_str("no window"))?
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?
        .get_element_by_id(&canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas element not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("element is not a HtmlCanvasElement"))?;

    // Force HighPerformance adapter selection. Browsers default to LowPower
    // (integrated) which on a laptop with a dGPU costs 5–10× the framerate
    // for free. Set on both targets — there's no reason for native to
    // prefer integrated either.
    let mut web_options = eframe::WebOptions::default();
    use egui_wgpu::WgpuSetup;
    if let WgpuSetup::CreateNew { power_preference, .. } =
        &mut web_options.wgpu_options.wgpu_setup
    {
        *power_preference = wgpu::PowerPreference::HighPerformance;
    }

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::ErebusApp::new(cc)?))),
            )
            .await
        {
            log::error!("erebus: WebRunner::start failed: {e:?}");
        }
    });

    Ok(())
}
