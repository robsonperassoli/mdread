// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use mdread_lib::DecorationsMode;
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
}

fn main() {
    let args = Args::parse();
    let file = match args.file {
        Some(path) => path,
        None if cfg!(debug_assertions) => default_sample(),
        None => {
            eprintln!("mdread: missing markdown file\nTry: mdread README.md");
            std::process::exit(2);
        }
    };

    mdread_lib::run(file, args.decorations);
}

fn default_sample() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../examples/sample.md")
}
