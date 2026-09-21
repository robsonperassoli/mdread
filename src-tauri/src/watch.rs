use crate::config::Settings;
use crate::fonts;
use crate::markdown;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub file: PathBuf,
    pub settings: Mutex<Settings>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub html: String,
    pub title: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootPayload {
    pub document: Document,
    pub settings: Settings,
    pub fonts: Vec<String>,
    pub tiling_wm: bool,
}

#[tauri::command]
pub fn get_boot(state: State<AppState>) -> BootPayload {
    let settings = state.settings.lock().expect("settings").clone();
    let light = settings.light_syntax();
    BootPayload {
        document: load_document(&state.file, light),
        settings,
        fonts: fonts::installed(),
        tiling_wm: crate::wm::is_tiling_wm(),
    }
}

#[tauri::command]
pub fn save_settings(
    settings: Settings,
    state: State<AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let (syntax_changed, decorations_changed) = state
        .settings
        .lock()
        .map(|current| {
            (
                current.light_syntax() != settings.light_syntax(),
                current.decorations != settings.decorations,
            )
        })
        .unwrap_or((true, true));
    crate::config::save(&settings)?;
    *state.settings.lock().expect("settings") = settings.clone();
    if decorations_changed {
        if let Some(window) = app.get_webview_window("main") {
            let hide = settings.decorations.hide_title_bar();
            let _ = window.set_decorations(!hide);
        }
    }
    if syntax_changed {
        let _ = app.emit(
            "document-updated",
            load_document(&state.file, settings.light_syntax()),
        );
    }
    Ok(())
}

pub fn load_document(path: &Path, light_syntax: bool) -> Document {
    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mdread")
        .to_string();
    let html =
        markdown::render_file(path, light_syntax).unwrap_or_else(|_| markdown::missing_html());
    Document {
        html,
        title,
        path: path.display().to_string(),
    }
}

pub fn start_watcher(app: AppHandle) {
    let file = app.state::<AppState>().file.clone();
    let config_app = app.clone();
    std::thread::spawn(move || watch_loop(app, file));
    std::thread::spawn(move || watch_config(config_app));
}

fn watch_loop(app: AppHandle, file: PathBuf) {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("mdread: file watcher failed: {err}");
            return;
        }
    };

    let watch_dir = file.parent().unwrap_or_else(|| Path::new("."));
    if let Err(err) = watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
        eprintln!("mdread: could not watch {}: {err}", watch_dir.display());
        return;
    }

    // Keep the watcher alive for the thread lifetime.
    let _watcher = watcher;
    // Opening the file for the first render can emit access events; ignore the burst.
    std::thread::sleep(Duration::from_millis(250));
    while rx.try_recv().is_ok() {}

    let mut last_source = std::fs::read(&file).unwrap_or_default();

    while let Ok(event) = rx.recv() {
        let Ok(event) = event else { continue };
        if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
            continue;
        }
        if !targets_file(&event, &file) {
            continue;
        }

        // Coalesce bursts from atomic saves (write temp, rename).
        while rx.recv_timeout(Duration::from_millis(120)).is_ok() {}

        let Ok(source) = std::fs::read(&file) else {
            let light = syntax_is_light(&app);
            let _ = app.emit("document-updated", load_document(&file, light));
            continue;
        };
        if source == last_source {
            continue;
        }
        last_source = source;

        let light = syntax_is_light(&app);
        let document = load_document(&file, light);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title(&document.title);
        }
        let _ = app.emit("document-updated", document);
    }
}

fn syntax_is_light(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.light_syntax())
        .unwrap_or(false)
}

fn watch_config(app: AppHandle) {
    let path = crate::config::config_path();
    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!("mdread: could not create {}: {err}", parent.display());
            return;
        }
    }
    let Some(parent) = path.parent().map(Path::to_path_buf) else {
        return;
    };

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = match notify::recommended_watcher(tx) {
        Ok(watcher) => watcher,
        Err(err) => {
            eprintln!("mdread: config watcher failed: {err}");
            return;
        }
    };
    if let Err(err) = watcher.watch(&parent, RecursiveMode::NonRecursive) {
        eprintln!("mdread: could not watch {}: {err}", parent.display());
        return;
    }
    let _watcher = watcher;
    std::thread::sleep(Duration::from_millis(250));
    while rx.try_recv().is_ok() {}

    while let Ok(event) = rx.recv() {
        let Ok(event) = event else { continue };
        if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
            continue;
        }
        if !targets_file(&event, &path) {
            continue;
        }
        while rx.recv_timeout(Duration::from_millis(120)).is_ok() {}
        apply_external_config(&app);
    }
}

fn apply_external_config(app: &AppHandle) {
    let loaded = crate::config::load();
    let state = app.state::<AppState>();
    let (syntax_changed, decorations_changed) = {
        let mut current = state.settings.lock().expect("settings");
        if *current == loaded {
            return;
        }
        let syntax_changed = current.light_syntax() != loaded.light_syntax();
        let decorations_changed = current.decorations != loaded.decorations;
        *current = loaded.clone();
        (syntax_changed, decorations_changed)
    };
    if decorations_changed {
        if let Some(window) = app.get_webview_window("main") {
            let hide = loaded.decorations.hide_title_bar();
            let _ = window.set_decorations(!hide);
        }
    }
    let _ = app.emit("settings-updated", &loaded);
    if syntax_changed {
        let file = state.file.clone();
        let _ = app.emit(
            "document-updated",
            load_document(&file, loaded.light_syntax()),
        );
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
