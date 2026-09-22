use std::path::PathBuf;

mod app;
mod appearance;
mod color;
mod config;
mod fonts;
mod markdown;
mod watch;
mod wm;

pub use config::DecorationsMode;

pub fn run(file: PathBuf, decorations: Option<DecorationsMode>) {
    let file = file.canonicalize().unwrap_or(file);
    let mut settings = config::load();
    if let Some(mode) = decorations {
        settings.decorations = mode;
    }

    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    app.run(move |cx| {
        gpui_kit::init(cx);
        app::init(cx);
        app::open(file, settings, cx);
    });
}
