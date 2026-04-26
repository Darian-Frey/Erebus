// Native binary entry. Defers to the library so the WASM build (which has
// no `main`) and the desktop build share the same shell.

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    erebus::app::run_native()
}
