use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum WatchEvent {
    File,
    Config,
}

pub fn spawn(file: PathBuf, tx: smol::channel::Sender<WatchEvent>) {
    let config = crate::config::config_path();
    let file_tx = tx.clone();
    let config_tx = tx;
    std::thread::spawn(move || watch_path(file, WatchEvent::File, file_tx));
    std::thread::spawn(move || watch_path(config, WatchEvent::Config, config_tx));
}

fn watch_path(path: PathBuf, kind: WatchEvent, tx: smol::channel::Sender<WatchEvent>) {
    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!("mdread: could not create {}: {err}", parent.display());
            return;
        }
    }
    let Some(parent) = path.parent().map(Path::to_path_buf) else {
        return;
    };

    let (notify_tx, notify_rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = match notify::recommended_watcher(notify_tx) {
        Ok(watcher) => watcher,
        Err(err) => {
            eprintln!("mdread: watcher failed: {err}");
            return;
        }
    };
    if let Err(err) = watcher.watch(&parent, RecursiveMode::NonRecursive) {
        eprintln!("mdread: could not watch {}: {err}", parent.display());
        return;
    }
    let _watcher = watcher;
    std::thread::sleep(Duration::from_millis(250));
    while notify_rx.try_recv().is_ok() {}

    let mut last_bytes = std::fs::read(&path).unwrap_or_default();

    while let Ok(event) = notify_rx.recv() {
        let Ok(event) = event else { continue };
        if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
            continue;
        }
        if !targets_file(&event, &path) {
            continue;
        }
        while notify_rx.recv_timeout(Duration::from_millis(120)).is_ok() {}

        if matches!(kind, WatchEvent::File) {
            let Ok(bytes) = std::fs::read(&path) else {
                let _ = tx.send_blocking(kind.clone());
                continue;
            };
            if bytes == last_bytes {
                continue;
            }
            last_bytes = bytes;
        }

        if tx.send_blocking(kind.clone()).is_err() {
            break;
        }
    }
}

fn targets_file(event: &Event, file: &Path) -> bool {
    let name = file.file_name();
    event.paths.iter().any(|path| {
        path == file
            || (name.is_some() && path.file_name() == name)
            || path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                n.starts_with('.') && n.contains(name.and_then(|s| s.to_str()).unwrap_or("\0"))
            })
    })
}
