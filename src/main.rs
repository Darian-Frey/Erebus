// Erebus — procedural nebula & starfield generator
// Entry point. Initializes logging and hands off to the app shell.

mod app;
mod gui;
mod render;
mod export;
mod preset;
mod noise;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    app::run()
}
