//! Guide asset policy: no remote images, no decorative stock, no active SVG.
//!
//! The renderer never fetches a URL. An image that is not a local file would
//! silently disappear in Typst, so the engine rejects it instead.

use std::path::Path;

use crate::error::{Error, Result};

const STOCK_HOSTS: &[&str] = &[
    "unsplash.com",
    "pexels.com",
    "pixabay.com",
    "shutterstock.com",
    "istockphoto.com",
    "gettyimages.com",
    "freepik.com",
    "adobe stock",
];

pub fn lint_assets(markdown: &str, guide_root: &Path) -> Result<()> {
    for (index, line) in markdown.lines().enumerate() {
        for (alt, dest) in images(line) {
            let line_number = index + 1;
            if alt.trim().is_empty() {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "image has no caption/alt text; describe the takeaway".to_string(),
                });
            }
            let lower = alt.to_lowercase();
            if lower.starts_with("image of") || lower.starts_with("resim") {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "alt text must describe the takeaway, not say “image of”".to_string(),
                });
            }
            if alt.trim().chars().count() < 20 {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "figure caption is too short; state the takeaway".to_string(),
                });
            }
            if !has_provenance(&alt) {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "figure caption needs provenance such as [PD0, s. 35] or [original]"
                        .to_string(),
                });
            }
            let dest_lower = dest.to_lowercase();
            if dest_lower.starts_with("http://") || dest_lower.starts_with("https://") {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "remote images are not allowed; download the asset into assets/"
                        .to_string(),
                });
            }
            if STOCK_HOSTS
                .iter()
                .any(|host| dest_lower.contains(host) || lower.contains(host))
            {
                return Err(Error::Markdown {
                    line: line_number,
                    message: "decorative stock images are not allowed in a study guide".to_string(),
                });
            }
            let dest_path = Path::new(&dest);
            if dest_path.is_absolute()
                || dest_path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                return Err(Error::Markdown {
                    line: line_number,
                    message: format!("asset path must stay inside the guide directory: {dest:?}"),
                });
            }
            let asset = guide_root.join(dest_path);
            if !asset.is_file() {
                return Err(Error::Markdown {
                    line: line_number,
                    message: format!("asset not found: {}", asset.display()),
                });
            }
            if dest_lower.ends_with(".svg") {
                let svg = std::fs::read_to_string(&asset).map_err(|error| Error::Io {
                    path: asset.clone(),
                    source: error,
                })?;
                sanitize_svg(&svg, &asset)?;
            }
        }
    }
    Ok(())
}

/// Reject active content and external references in an SVG asset.
pub fn sanitize_svg(svg: &str, path: &Path) -> Result<()> {
    let lower = svg.to_lowercase();
    if !lower.contains("<svg") {
        return Err(asset_error(path, "not an SVG document"));
    }
    if !lower.contains("viewbox=") {
        return Err(asset_error(
            path,
            "SVG needs a viewBox for deterministic scaling",
        ));
    }
    for forbidden in [
        "<script",
        "<foreignobject",
        "<iframe",
        "<object",
        "<embed",
        "javascript:",
        "data:text/html",
        "@import",
        "url(http",
        "url(https",
        "url(//",
    ] {
        if lower.contains(forbidden) {
            return Err(asset_error(
                path,
                &format!("SVG contains forbidden content {forbidden:?}"),
            ));
        }
    }
    if has_event_handler(&lower) {
        return Err(asset_error(path, "SVG contains an event handler"));
    }
    if has_external_reference(&lower) {
        return Err(asset_error(
            path,
            "SVG references an external resource; inline or embed it locally",
        ));
    }
    Ok(())
}

fn has_provenance(caption: &str) -> bool {
    let lower = caption.to_lowercase();
    if !lower.contains('[') || !lower.contains(']') {
        return false;
    }
    lower.contains("pd")
        || lower.contains("original")
        || lower.contains("özgün")
        || lower.contains(" s. ")
        || lower.contains(" p. ")
}

fn asset_error(path: &Path, message: &str) -> Error {
    Error::Markdown {
        line: 0,
        message: format!("{}: {message}", path.display()),
    }
}

fn has_event_handler(svg: &str) -> bool {
    for (index, _) in svg.match_indices(" on") {
        let rest = &svg[index + 3..];
        let name = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .collect::<String>();
        if rest[name.len()..].trim_start().starts_with('=')
            && matches!(
                name.as_str(),
                "load" | "click" | "error" | "mouseover" | "focus" | "begin"
            )
        {
            return true;
        }
    }
    false
}

fn has_external_reference(svg: &str) -> bool {
    for attribute in ["href=", "src=", "xlink:href="] {
        let mut rest = svg;
        while let Some(index) = rest.find(attribute) {
            let value = rest[index + attribute.len()..].trim_start();
            let value = value.trim_start_matches(['"', '\'']);
            if value.starts_with("http://")
                || value.starts_with("https://")
                || value.starts_with("//")
            {
                return true;
            }
            rest = &rest[index + attribute.len()..];
        }
    }
    false
}

fn images(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("![") {
        let after = &rest[start + 2..];
        let Some(close) = after.find("](") else {
            break;
        };
        let alt = &after[..close];
        let after_dest = &after[close + 2..];
        let Some(end) = after_dest.find(')') else {
            break;
        };
        out.push((alt.to_string(), after_dest[..end].to_string()));
        rest = &after_dest[end + 1..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_script_and_external_svg() {
        let path = Path::new("asset.svg");
        let script = r#"<svg viewBox="0 0 10 10"><script>alert(1)</script></svg>"#;
        assert!(sanitize_svg(script, path).is_err());
        let external =
            r#"<svg viewBox="0 0 10 10"><image href="https://example.com/a.png"/></svg>"#;
        assert!(sanitize_svg(external, path).is_err());
        let handler = r#"<svg viewBox="0 0 10 10" onload="x()"></svg>"#;
        assert!(sanitize_svg(handler, path).is_err());
    }

    #[test]
    fn accepts_an_inline_svg() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><title>X</title><rect width="10" height="10" fill="#fff"/></svg>"##;
        assert!(sanitize_svg(svg, Path::new("asset.svg")).is_ok());
    }

    #[test]
    fn rejects_remote_and_stock_images() {
        let root = Path::new(".");
        assert!(lint_assets("![A conceptual diagram](https://example.com/a.svg)\n", root).is_err());
        assert!(
            lint_assets(
                "![Usable comparison](https://images.unsplash.com/a.jpg)\n",
                root
            )
            .is_err()
        );
        assert!(lint_assets("![](assets/a.svg)\n", root).is_err());
        assert!(lint_assets("![A diagram without provenance](assets/a.svg)\n", root).is_err());
    }
}
