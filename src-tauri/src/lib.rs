use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

mod config;
mod fonts;
mod markdown;
mod watch;
mod wm;

pub use config::DecorationsMode;

use watch::{start_watcher, AppState};

pub fn run(file: PathBuf, decorations: Option<DecorationsMode>) {
    let file = file.canonicalize().unwrap_or(file);
    let mut settings = config::load();
    if let Some(mode) = decorations {
        settings.decorations = mode;
    }
    let hide_chrome = settings.decorations.hide_title_bar();

    let mut ctx = tauri::generate_context!();
    if let Some(window) = ctx.config_mut().app.windows.first_mut() {
        window.decorations = !hide_chrome;
    }

    tauri::Builder::default()
        .manage(AppState {
            file,
            settings: Mutex::new(settings),
        })
        .invoke_handler(tauri::generate_handler![
            watch::get_boot,
            watch::save_settings
        ])
        .setup(|app| {
            start_watcher(app.handle().clone());
            if let Some(window) = app.get_webview_window("main") {
                let state = app.state::<AppState>();
                let title = state
                    .file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("mdread");
                let _ = window.set_title(title);
            }
            Ok(())
        })
        .run(ctx)
        .expect("error while running mdread");
}
