// App shell: window, eframe lifecycle, top-level state owner.

pub mod state;
pub mod config;

pub fn run() -> anyhow::Result<()> {
    // TODO: wire eframe NativeOptions with wgpu backend, RGBA16F surface, and launch ErebusApp.
    Ok(())
}
