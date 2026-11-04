//! CLI authoring commands for the D2-only diagram pipeline.
//!
//! This module is the only place that executes D2. The MCP server and the
//! guide build never call into it.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use folisift_core::{Guide, Workspace};
use folisift_render::diagram::{
    self, D2_VERSION, DiagramManifest, DiagramSpec, DiagramState, DiagramTheme,
    POSTPROCESS_VERSION, SPEC_SCHEMA_VERSION, TALA_SEEDS,
};
use folisift_render::styles;
use serde_json::{Value, json};

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn validate(
    workspace: &Workspace,
    guide_slug: &str,
    id: Option<&str>,
    all: bool,
) -> CliResult<Value> {
    let guide = Guide::load(workspace, guide_slug)?;
    let specs = selected_specs(&guide, id, all)?;
    let items = specs
        .iter()
        .map(|spec| {
            diagram::result_item(
                &spec.id,
                DiagramState::Ok,
                false,
                None,
                Vec::new(),
                Vec::new(),
                json!({
                    "kind": spec.kind,
                    "nodes": spec.nodes.len(),
                    "edges": spec.edges.len(),
                    "spec": diagram::spec_path(&guide.dir, &spec.id).display().to_string(),
                    "spec_sha256": diagram::spec_hash(spec).ok(),
                    "layout": spec.layout,
                }),
            )
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "guide": guide.slug,
        "ok": true,
        "schema_version": diagram::DIAGRAM_RESULT_SCHEMA_VERSION,
        "items": items,
    }))
}

pub fn build(
    workspace: &Workspace,
    guide_slug: &str,
    id: Option<&str>,
    all: bool,
    force: bool,
) -> CliResult<Value> {
    let guide = Guide::load(workspace, guide_slug)?;
    let theme = theme(workspace, &guide)?;
    let specs = selected_specs(&guide, id, all)?;
    if specs.is_empty() {
        return Ok(json!({
            "guide": guide.slug,
            "d2_version": D2_VERSION,
            "items": [],
        }));
    }
    let d2 = find_d2().ok_or_else(|| {
        format!("D2 {D2_VERSION} is required for `diagram build`; set FOLISIFT_D2 or install it")
    })?;
    check_d2_version(&d2)?;
    let mut items = Vec::new();
    for spec in specs {
        items.push(build_one(workspace, &guide, &theme, &d2, &spec, force)?);
    }
    Ok(json!({
        "guide": guide.slug,
        "d2": d2.display().to_string(),
        "d2_version": D2_VERSION,
        "schema_version": diagram::DIAGRAM_RESULT_SCHEMA_VERSION,
        "items": items,
    }))
}

pub fn status(workspace: &Workspace, guide_slug: &str) -> CliResult<Value> {
    let guide = Guide::load(workspace, guide_slug)?;
    let statuses = diagram::statuses(&guide.slug, &guide.dir)?;
    let ok = statuses.iter().all(|item| item.state == DiagramState::Ok);
    Ok(json!({
        "guide": guide.slug,
        "ok": ok,
        "d2_version": D2_VERSION,
        "schema_version": diagram::DIAGRAM_RESULT_SCHEMA_VERSION,
        "items": statuses.iter().map(diagram::status_value).collect::<Vec<_>>(),
    }))
}

pub fn fit(
    workspace: &Workspace,
    guide_slug: &str,
    id: &str,
    width_percent: f64,
) -> CliResult<Value> {
    let guide = Guide::load(workspace, guide_slug)?;
    let spec = diagram::read_spec(&guide.dir, id)?;
    let manifest = diagram::read_manifest(&guide.dir, id)?
        .ok_or_else(|| format!("diagram {id:?} has no manifest; run `folisift diagram build`"))?;
    let svg_path = guide.dir.join(&manifest.svg);
    let svg = std::fs::read_to_string(&svg_path)?;
    let (content_width_mm, content_height_mm) = content_box(workspace, &guide)?;
    diagram::fit_report(
        id,
        &spec,
        &svg,
        content_width_mm,
        content_height_mm,
        width_percent,
    )
    .map_err(Into::into)
}

fn content_box(workspace: &Workspace, guide: &Guide) -> CliResult<(f64, f64)> {
    let style = guide.style();
    let loaded = styles::load_with(
        workspace.root(),
        &style.aesthetic,
        Some(style.palette.as_str()),
        Some(style.fonts.as_str()),
    )?;
    let layout = loaded.tokens.get("layout");
    let get = |key: &str, default: f64| {
        layout
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(default)
    };
    let page_width = get("page_width_mm", 210.0);
    let page_height = get("page_height_mm", 297.0);
    let side = get("margin_side_mm", 16.2);
    let top = get("margin_top_mm", 18.3);
    let bottom = get("margin_bottom_mm", 15.9);
    Ok((page_width - 2.0 * side, page_height - top - bottom))
}

pub fn clean(workspace: &Workspace, guide_slug: &str) -> CliResult<Value> {
    let guide = Guide::load(workspace, guide_slug)?;
    let specs = diagram::read_specs(&guide.dir)?;
    let referenced = specs
        .iter()
        .map(|spec| spec.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let mut removed = Vec::new();
    let d2_dir = guide.dir.join("data/d2");
    if d2_dir.is_dir() {
        for entry in std::fs::read_dir(&d2_dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if path.extension().and_then(|value| value.to_str()) == Some("d2")
                && !referenced.contains(id)
            {
                std::fs::remove_file(&path)?;
                removed.push(path.display().to_string());
            }
        }
    }
    let generated = guide.dir.join("assets/generated");
    if generated.is_dir() {
        let manifests = specs
            .iter()
            .filter_map(|spec| diagram::read_manifest(&guide.dir, &spec.id).ok().flatten())
            .filter_map(|manifest| manifest.svg.rsplit('/').next().map(ToString::to_string))
            .collect::<std::collections::HashSet<_>>();
        for entry in std::fs::read_dir(&generated)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name.ends_with(".svg") && !manifests.contains(name) {
                std::fs::remove_file(&path)?;
                removed.push(path.display().to_string());
            }
        }
    }
    Ok(json!({
        "guide": guide.slug,
        "removed": removed,
    }))
}

fn selected_specs(guide: &Guide, id: Option<&str>, all: bool) -> CliResult<Vec<DiagramSpec>> {
    match id {
        Some(id) => Ok(vec![diagram::read_spec(&guide.dir, id)?]),
        None if all => Ok(diagram::read_specs(&guide.dir)?),
        None => Err("pass --id <diagram> or --all".into()),
    }
}

fn theme(workspace: &Workspace, guide: &Guide) -> CliResult<DiagramTheme> {
    let style = guide.style();
    let loaded = styles::load_with(
        workspace.root(),
        &style.aesthetic,
        Some(style.palette.as_str()),
        Some(style.fonts.as_str()),
    )?;
    Ok(DiagramTheme::from_tokens(
        &loaded.tokens,
        &loaded.fonts.heading,
    ))
}

fn build_one(
    workspace: &Workspace,
    guide: &Guide,
    theme: &DiagramTheme,
    d2: &Path,
    spec: &DiagramSpec,
    force: bool,
) -> CliResult<Value> {
    let fonts = font_dir(workspace)?;
    let font_digest = font_digest(&fonts)?;
    let theme_digest = diagram::theme_digest(theme)?;
    let spec_sha = diagram::spec_hash(spec)?;
    let manifest_path = diagram::manifest_path(&guide.dir, &spec.id);
    if !force && let Some(manifest) = diagram::read_manifest(&guide.dir, &spec.id)? {
        let svg_path = diagram::safe_asset_path(&guide.dir, &manifest.svg)?;
        let d2_source_exists = diagram::d2_path(&guide.dir, &spec.id).is_file();
        let valid = d2_source_exists
            && manifest.spec_sha256 == spec_sha
            && manifest.d2_version == D2_VERSION
            && manifest.tala_seeds == TALA_SEEDS
            && manifest.theme_digest == theme_digest
            && manifest.font_digest == font_digest
            && manifest.postprocess_version == POSTPROCESS_VERSION
            && std::fs::read(&svg_path)
                .map(|bytes| blake3::hash(&bytes).to_hex().to_string() == manifest.svg_sha256)
                .unwrap_or(false);
        if valid {
            return Ok(diagram::result_item(
                &spec.id,
                DiagramState::Ok,
                true,
                Some(manifest.svg.clone()),
                Vec::new(),
                Vec::new(),
                json!({
                    "manifest": manifest_path.display().to_string(),
                    "svg_sha256": manifest.svg_sha256,
                    "layout": manifest.layout,
                    "direction": manifest.direction,
                    "aspect_ratio": manifest.aspect_ratio,
                    "allow_page_span": manifest.allow_page_span,
                }),
            ));
        }
    }

    let d2_path = diagram::write_d2_source(&guide.dir, spec, theme)?;
    let temp_svg = guide
        .dir
        .join("data/d2")
        .join(format!(".{}.svg.tmp", spec.id));
    if let Some(parent) = temp_svg.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let d2_home = guide.dir.join("data/d2-home");
    std::fs::create_dir_all(&d2_home)?;
    let mut command = Command::new(d2);
    command
        .env_clear()
        .env("PATH", "")
        .env("HOME", &d2_home)
        .env("TMPDIR", &d2_home)
        .arg(format!("--layout={}", spec.layout))
        .arg(format!("--tala-seeds={TALA_SEEDS}"))
        .arg(format!("--salt={}", diagram::salt(&spec_sha)))
        .arg("--pad=20")
        .arg("--theme=0")
        .arg("--timeout=120")
        .arg("--no-xml-tag")
        .arg("--omit-version")
        .arg(format!(
            "--font-regular={}",
            fonts.join("Inter-Regular.ttf").display()
        ))
        .arg(format!(
            "--font-bold={}",
            fonts.join("Inter-Bold.ttf").display()
        ))
        .arg(format!(
            "--font-italic={}",
            fonts.join("Inter-Italic.ttf").display()
        ))
        .arg(format!(
            "--font-semibold={}",
            fonts.join("Inter-SemiBold.ttf").display()
        ))
        .arg(format!(
            "--font-mono={}",
            fonts.join("NotoSansMono-Regular.ttf").display()
        ))
        .arg(&d2_path)
        .arg(&temp_svg)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if spec.sketch {
        command.arg("--sketch");
    }
    let output = command.output()?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&temp_svg);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "d2 failed for {:?} (exit {}): {}{}",
            spec.id,
            output.status,
            stderr.trim(),
            if stdout.trim().is_empty() { "" } else { " | " }
        )
        .into());
    }
    let raw = std::fs::read_to_string(&temp_svg)?;
    let _ = std::fs::remove_file(&temp_svg);
    let key = &spec_sha[..12.min(spec_sha.len())];
    let svg = diagram::postprocess_svg(&raw, &spec.title, &spec.alt, &theme.paper, key)?;
    let (content_width, content_height) = content_box(workspace, guide)?;
    let requested_width = spec
        .width
        .as_deref()
        .and_then(|value| value.strip_suffix('%'))
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(90.0);
    let max_width = spec
        .max_width
        .as_deref()
        .and_then(|value| value.strip_suffix('%'))
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(100.0);
    let width_percent = requested_width.min(max_width);
    let fit = diagram::fit_report(
        &spec.id,
        spec,
        &svg,
        content_width,
        content_height,
        width_percent,
    )?;
    if fit["ok"].as_bool() != Some(true) {
        return Err(format!(
            "diagram {:?} does not fit one page (estimated share {:.2}); set allow_page_span=true or reduce width: {}",
            spec.id,
            fit["page_share"].as_f64().unwrap_or(0.0),
            fit["warnings"]
                .as_array()
                .map(|warnings| warnings
                    .iter()
                    .filter_map(|warning| warning.as_str())
                    .collect::<Vec<_>>()
                    .join("; "))
                .unwrap_or_default()
        )
        .into());
    }
    let svg_hash = blake3::hash(svg.as_bytes()).to_hex().to_string();
    let asset = diagram::write_asset(&guide.dir, &svg_hash, &svg)?;
    let relative = asset
        .strip_prefix(&guide.dir)
        .unwrap_or(&asset)
        .display()
        .to_string()
        .replace('\\', "/");
    let manifest = DiagramManifest {
        schema_version: SPEC_SCHEMA_VERSION,
        id: spec.id.clone(),
        kind: spec.kind.clone(),
        title: spec.title.clone(),
        caption: spec.caption.clone(),
        alt: spec.alt.clone(),
        source: spec.source.clone(),
        spec_sha256: spec_sha.clone(),
        svg_sha256: svg_hash.clone(),
        svg: relative.clone(),
        d2_version: D2_VERSION.to_string(),
        layout: spec.layout.clone(),
        direction: spec.direction.clone(),
        aspect_ratio: spec.aspect_ratio,
        allow_page_span: spec.allow_page_span,
        sketch: spec.sketch,
        tala_seeds: TALA_SEEDS.to_string(),
        salt: diagram::salt(&spec_sha),
        fonts: vec![
            "Inter-Regular.ttf".to_string(),
            "Inter-Bold.ttf".to_string(),
            "Inter-Italic.ttf".to_string(),
            "Inter-SemiBold.ttf".to_string(),
            "NotoSansMono-Regular.ttf".to_string(),
        ],
        font_digest: font_digest.clone(),
        theme_digest,
        postprocess_version: POSTPROCESS_VERSION,
    };
    let manifest_path = diagram::write_manifest(&guide.dir, &manifest)?;
    Ok(diagram::result_item(
        &spec.id,
        DiagramState::Ok,
        false,
        Some(relative.clone()),
        Vec::new(),
        Vec::new(),
        json!({
            "svg_sha256": svg_hash,
            "manifest": manifest_path.display().to_string(),
            "layout": manifest.layout,
            "direction": manifest.direction,
            "aspect_ratio": manifest.aspect_ratio,
            "allow_page_span": manifest.allow_page_span,
        }),
    ))
}

fn font_digest(fonts: &Path) -> CliResult<String> {
    let mut bytes = Vec::new();
    for name in [
        "Inter-Regular.ttf",
        "Inter-Bold.ttf",
        "Inter-Italic.ttf",
        "Inter-SemiBold.ttf",
        "NotoSansMono-Regular.ttf",
    ] {
        let data = std::fs::read(fonts.join(name))?;
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&data);
    }
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn font_dir(workspace: &Workspace) -> CliResult<PathBuf> {
    let candidates = [
        std::env::var_os("FOLISIFT_FONTS").map(PathBuf::from),
        Some(workspace.root().join("templates/fonts")),
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|parent| parent.join("templates/fonts"))),
        Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/fonts")),
    ];
    for candidate in candidates.into_iter().flatten() {
        if candidate.join("Inter-Regular.ttf").is_file() {
            return Ok(candidate);
        }
    }
    Err("diagram fonts not found; set FOLISIFT_FONTS to the templates/fonts directory".into())
}

fn check_d2_version(d2: &Path) -> CliResult<()> {
    let home = std::env::temp_dir().join("folisift-d2-version");
    std::fs::create_dir_all(&home)?;
    let output = Command::new(d2)
        .env_clear()
        .env("PATH", "")
        .env("HOME", &home)
        .env("TMPDIR", &home)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let version = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .trim()
    .to_string();
    if !output.status.success() || version != D2_VERSION {
        return Err(format!(
            "D2 version mismatch: found {version:?}, expected {D2_VERSION}; set FOLISIFT_D2 to the pinned binary"
        )
        .into());
    }
    Ok(())
}

pub fn engine_info() -> Value {
    let Some(path) = find_d2() else {
        return json!({
            "ok": true,
            "required": false,
            "path": null,
            "version": null,
            "detail": "not installed; only `diagram build` needs D2",
        });
    };
    let output = Command::new(&path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let version = output
        .ok()
        .map(|output| {
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
            .trim()
            .to_string()
        })
        .unwrap_or_default();
    json!({
        "ok": version == D2_VERSION,
        "required": false,
        "path": path.display().to_string(),
        "version": version,
        "detail": format!("pinned {D2_VERSION}; only `diagram build` invokes it"),
    })
}

fn find_d2() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("FOLISIFT_D2") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    let exe = std::env::current_exe().ok();
    if let Some(exe) = exe
        && let Some(parent) = exe.parent()
    {
        let candidate = parent.join(format!("d2{}", std::env::consts::EXE_SUFFIX));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
