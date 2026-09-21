use base64::Engine;
use comrak::options::Plugins;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, Options};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

const EMPTY: &str = "<p>This file could not be read.</p>";

pub fn render_file(path: &Path, theme: &str) -> Result<String, String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(render_markdown(&source, path, theme))
}

pub fn missing_html() -> String {
    EMPTY.to_string()
}

pub fn render_markdown(source: &str, path: &Path, theme: &str) -> String {
    let body = strip_frontmatter(source);
    let html = markdown_to_html(body, theme);
    let html = sanitize(&html);
    inline_local_images(&html, path.parent().unwrap_or_else(|| Path::new(".")))
}

fn strip_frontmatter(source: &str) -> &str {
    let trimmed = source.trim_start_matches('\u{feff}');
    let rest = if let Some(rest) = trimmed.strip_prefix("---\n") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("---\r\n") {
        rest
    } else {
        return trimmed;
    };

    for needle in ["\n---\n", "\n---\r\n", "\r\n---\r\n", "\r\n---\n"] {
        if let Some(idx) = rest.find(needle) {
            return &rest[idx + needle.len()..];
        }
    }
    trimmed
}

fn markdown_to_html(source: &str, theme: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.header_id_prefix = Some(String::new());
    options.render.github_pre_lang = true;
    options.render.r#unsafe = false;

    let syntect_theme = if theme == "light" || theme == "sepia" {
        "InspiredGitHub"
    } else {
        "base16-ocean.dark"
    };
    let adapter = SyntectAdapter::new(Some(syntect_theme));
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
    markdown_to_html_with_plugins(source, &options, &plugins)
}

fn sanitize(html: &str) -> String {
    let mut builder = ammonia::Builder::default();
    builder
        .add_tags(["input", "section", "figure", "figcaption", "picture"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_tag_attributes("code", ["class"])
        .add_tag_attributes("pre", ["class", "lang"])
        .add_tag_attributes("span", ["class", "style"])
        .add_tag_attributes("div", ["class"])
        .add_tag_attributes("h1", ["id"])
        .add_tag_attributes("h2", ["id"])
        .add_tag_attributes("h3", ["id"])
        .add_tag_attributes("h4", ["id"])
        .add_tag_attributes("h5", ["id"])
        .add_tag_attributes("h6", ["id"])
        .add_tag_attributes("th", ["align"])
        .add_tag_attributes("td", ["align"])
        .add_tag_attributes("img", ["src", "alt", "title"])
        .url_schemes(["http", "https", "mailto", "data"].into_iter().collect());
    builder.clean(html).to_string()
}

fn inline_local_images(html: &str, base: &Path) -> String {
    static SRC: OnceLock<Regex> = OnceLock::new();
    let src =
        SRC.get_or_init(|| Regex::new(r#"(<img\b[^>]*?\bsrc=")([^"]+)(")"#).expect("img regex"));
    src.replace_all(html, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let url = &caps[2];
        let suffix = &caps[3];
        if looks_remote(url) {
            return format!("{prefix}{url}{suffix}");
        }
        let path = base.join(url);
        match encode_image(&path) {
            Some(data) => format!("{prefix}{data}{suffix}"),
            None => format!("{prefix}{url}{suffix}"),
        }
    })
    .into_owned()
}

fn looks_remote(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("mailto:")
}

fn encode_image(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{b64}"))
}
