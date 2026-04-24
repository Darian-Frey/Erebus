// Dev-only WGSL file watcher. Posts a single "shaders dirty" signal to the
// renderer; coalesces multiple events between frames so a multi-file save
// doesn't trigger N rebuilds.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

// Receiver is Send but !Sync; CallbackResources demands Sync. Mutex grants it.
pub struct ShaderWatcher {
    _watcher: RecommendedWatcher,
    rx: Mutex<Receiver<Event>>,
    last_event: Option<Instant>,
    debounce: Duration,
}

impl ShaderWatcher {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res {
                let _ = tx.send(ev);
            }
        })?;
        watcher.watch(&root, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher: watcher,
            rx: Mutex::new(rx),
            last_event: None,
            debounce: Duration::from_millis(150),
        })
    }

    /// Returns true once per debounced batch of shader file changes.
    pub fn poll(&mut self) -> bool {
        let mut got_change = false;
        let rx = self.rx.get_mut().expect("watcher mutex poisoned");
        while let Ok(ev) = rx.try_recv() {
            if matches!(
                ev.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                got_change = true;
            }
        }
        if got_change {
            self.last_event = Some(Instant::now());
            return false;
        }
        if let Some(t) = self.last_event {
            if t.elapsed() >= self.debounce {
                self.last_event = None;
                return true;
            }
        }
        false
    }
}
