//! CLI authoring commands for the D2-only diagram pipeline.
//!
//! This module is the only place that executes D2. The MCP server and the
//! guide build never call into it.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use folisift_core::{Guide, Workspace};
use folisift_render::diagram::{
    self, D2_VERSION, DiagramManifest, DiagramSpec, DiagramState, DiagramTheme,
    POSTPROCESS_VERSION, SPEC_SCHEMA_VERSION,
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
            json!({
                "id": spec.id,
                "ok": true,
                "kind": spec.kind,
                "nodes": spec.nodes.len(),
                "edges": spec.edges.len(),
                "spec": diagram::spec_path(&guide.dir, &spec.id).display().to_string(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "guide": guide.slug,
        "ok": true,
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
        "items": statuses,
    }))
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
    let loaded = styles::load(workspace.root(), &style.aesthetic)?;
    Ok(DiagramTheme::from_tokens(&loaded.tokens, "Inter"))
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
    let spec_sha = diagram::spec_hash(spec)?;
    let manifest_path = diagram::manifest_path(&guide.dir, &spec.id);
    if !force && let Some(manifest) = diagram::read_manifest(&guide.dir, &spec.id)? {
        let svg_path = diagram::safe_asset_path(&guide.dir, &manifest.svg)?;
        let d2_source_exists = diagram::d2_path(&guide.dir, &spec.id).is_file();
        let valid = d2_source_exists
            && manifest.spec_sha256 == spec_sha
            && manifest.d2_version == D2_VERSION
            && manifest.font_digest == font_digest
            && manifest.postprocess_version == POSTPROCESS_VERSION
            && std::fs::read(&svg_path)
                .map(|bytes| blake3::hash(&bytes).to_hex().to_string() == manifest.svg_sha256)
                .unwrap_or(false);
        if valid {
            return Ok(json!({
                "id": spec.id,
                "cache_hit": true,
                "svg": manifest.svg,
                "manifest": manifest_path.display().to_string(),
            }));
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
    let output = Command::new(d2)
        .arg(format!("--layout={}", spec.layout))
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
        .stderr(Stdio::piped())
        .output()?;
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
        salt: diagram::salt(&spec_sha),
        fonts: vec![
            "Inter-Regular.ttf".to_string(),
            "Inter-Bold.ttf".to_string(),
            "Inter-Italic.ttf".to_string(),
            "Inter-SemiBold.ttf".to_string(),
            "NotoSansMono-Regular.ttf".to_string(),
        ],
        font_digest: font_digest.clone(),
        postprocess_version: POSTPROCESS_VERSION,
    };
    let manifest_path = diagram::write_manifest(&guide.dir, &manifest)?;
    Ok(json!({
        "id": spec.id,
        "cache_hit": false,
        "svg": relative,
        "svg_sha256": svg_hash,
        "manifest": manifest_path.display().to_string(),
    }))
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
    let output = Command::new(d2)
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
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(format!("d2{}", std::env::consts::EXE_SUFFIX)))
            .find(|candidate| candidate.is_file())
    })
}
