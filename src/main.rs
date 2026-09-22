#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use mdread::DecorationsMode;
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mdread",
    version,
    about = "Open a markdown file in a clean reading window"
)]
struct Args {
    /// Markdown file to read. Reloads automatically when the file changes.
    file: Option<PathBuf>,

    /// Title bar: auto hides on tiling WMs, always/never override detection.
    #[arg(long, value_parser = clap::value_parser!(DecorationsMode))]
    decorations: Option<DecorationsMode>,

    /// Keep the process attached to the terminal (release builds detach by default).
    #[arg(long)]
    foreground: bool,
}

fn main() {
    let args = Args::parse();
    let file = match resolve_file(args.file) {
        Some(path) => path,
        None => std::process::exit(0),
    };

    if should_detach(args.foreground) {
        detach();
    }

    mdread::run(file, args.decorations);
}

fn resolve_file(file: Option<PathBuf>) -> Option<PathBuf> {
    let file = file.filter(|path| !path.as_os_str().is_empty());
    let path = match file {
        Some(path) => path,
        #[cfg(debug_assertions)]
        None => default_sample(),
        #[cfg(not(debug_assertions))]
        None => pick_markdown()?,
    };
    Some(path.canonicalize().unwrap_or(path))
}

#[cfg_attr(debug_assertions, allow(dead_code))]
fn pick_markdown() -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    dialog = dialog
        .add_filter("Markdown", &["md", "markdown", "mdown"])
        .set_title("Open markdown file");
    if let Some(home) = dirs::home_dir() {
        dialog = dialog.set_directory(home);
    }
    dialog.pick_file()
}

fn should_detach(foreground_flag: bool) -> bool {
    if foreground_flag || cfg!(debug_assertions) {
        return false;
    }
    std::io::stdin().is_terminal()
}

fn detach() {
    #[cfg(unix)]
    {
        // SAFETY: called once, before the GPU window is initialized.
        let rc = unsafe { libc::daemon(1, 0) };
        if rc != 0 {
            eprintln!(
                "mdread: could not detach: {}",
                std::io::Error::last_os_error()
            );
            std::process::exit(1);
        }
    }
}

#[cfg(debug_assertions)]
fn default_sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/sample.md")
}
