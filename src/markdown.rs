use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

const EMPTY: &str = "This file could not be read.";

pub fn load_source(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(source) => prepare(&source, path),
        Err(_) => EMPTY.to_string(),
    }
}

pub fn prepare(source: &str, path: &Path) -> String {
    let body = strip_frontmatter(source);
    rewrite_local_images(body, path.parent().unwrap_or_else(|| Path::new(".")))
}

pub fn strip_frontmatter(source: &str) -> &str {
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

fn rewrite_local_images(source: &str, base: &Path) -> String {
    static IMAGE: OnceLock<Regex> = OnceLock::new();
    let image = IMAGE.get_or_init(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").expect("image regex"));
    image
        .replace_all(source, |caps: &regex::Captures| {
            let alt = &caps[1];
            let url = caps[2].trim();
            let (href, title) = split_href_title(url);
            if looks_remote(href) {
                return caps[0].to_string();
            }
            let path = base.join(href);
            let abs = path.canonicalize().unwrap_or(path);
            let file_url = format!("file://{}", abs.display());
            match title {
                Some(title) => format!("![{alt}]({file_url} {title})"),
                None => format!("![{alt}]({file_url})"),
            }
        })
        .into_owned()
}

fn split_href_title(url: &str) -> (&str, Option<&str>) {
    let url = url.trim();
    if let Some(idx) = url.find(|c: char| c.is_whitespace()) {
        (url[..idx].trim(), Some(url[idx..].trim()))
    } else {
        (url, None)
    }
}

fn looks_remote(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("mailto:")
        || lower.starts_with("file:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn strips_yaml_frontmatter() {
        let source = "---\ntitle: Plan\n---\n\n# Hello\n";
        assert_eq!(strip_frontmatter(source).trim(), "# Hello");
    }

    #[test]
    fn leaves_plain_markdown() {
        assert_eq!(strip_frontmatter("# Hello\n"), "# Hello\n");
    }

    #[test]
    fn rewrites_relative_images() {
        let prepared = prepare("![alt](./pic.png)", Path::new("/tmp/doc.md"));
        assert!(prepared.contains("file://"), "{prepared}");
        assert!(prepared.contains("pic.png"), "{prepared}");
    }

    #[test]
    fn keeps_remote_images() {
        let prepared = prepare(
            "![alt](https://example.com/a.png)",
            Path::new("/tmp/doc.md"),
        );
        assert_eq!(prepared, "![alt](https://example.com/a.png)");
    }
}
