//! D2 diagram contract, source generation, manifest and SVG post-processing.
//!
//! The renderer never runs D2. The CLI owns the external render step; the
//! build path only reads validated manifests and SVG assets.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::assets::sanitize_svg;
use crate::error::{Error, Result};

pub const D2_VERSION: &str = "v0.9.0";
pub const SPEC_SCHEMA_VERSION: u32 = 2;
pub const POSTPROCESS_VERSION: u32 = 1;
pub const MAX_NODES: usize = 12;
pub const MAX_EDGES: usize = 20;
pub const MAX_DEPTH: usize = 4;
pub const DEFAULT_LAYOUT: &str = "tala";

const DEFAULT_DIRECTION: &str = "right";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagramSpec {
    #[serde(default = "default_schema")]
    pub schema_version: u32,
    pub id: String,
    pub kind: String,
    pub title: String,
    pub caption: String,
    pub alt: String,
    pub source: String,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Group {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiagramTheme {
    pub ink: String,
    pub muted: String,
    pub accent: String,
    pub paper: String,
    pub rule: String,
    pub font: String,
}

impl Default for DiagramTheme {
    fn default() -> Self {
        Self {
            ink: "#211E19".to_string(),
            muted: "#5E574B".to_string(),
            accent: "#8A3E2F".to_string(),
            paper: "#FAF6EC".to_string(),
            rule: "#D8CEBD".to_string(),
            font: "Inter".to_string(),
        }
    }
}

impl DiagramTheme {
    pub fn from_tokens(tokens: &serde_json::Value, font: &str) -> Self {
        let palette = tokens.get("palette");
        let get = |key: &str, fallback: &str| {
            palette
                .and_then(|value| value.get(key))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(fallback)
                .to_string()
        };
        Self {
            ink: get("ink", "#211E19"),
            muted: get("muted", "#5E574B"),
            accent: get("accent", "#8A3E2F"),
            paper: get("paper", "#FAF6EC"),
            rule: get("rule", "#D8CEBD"),
            font: font.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagramManifest {
    pub schema_version: u32,
    pub id: String,
    pub kind: String,
    pub title: String,
    pub caption: String,
    pub alt: String,
    pub source: String,
    pub spec_sha256: String,
    pub svg_sha256: String,
    pub svg: String,
    pub d2_version: String,
    pub layout: String,
    pub salt: String,
    pub fonts: Vec<String>,
    pub postprocess_version: u32,
}

#[derive(Debug, Clone)]
pub struct DiagramAsset {
    pub id: String,
    pub path: String,
    pub caption: String,
    pub alt: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramState {
    Ok,
    Missing,
    Stale,
    Invalid,
    EngineMismatch,
    MissingSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagramStatus {
    pub id: String,
    pub state: DiagramState,
    pub spec: String,
    pub manifest: Option<String>,
    pub svg: Option<String>,
    pub command: String,
    pub message: Option<String>,
}

pub fn default_schema() -> u32 {
    SPEC_SCHEMA_VERSION
}

pub fn default_layout() -> String {
    DEFAULT_LAYOUT.to_string()
}

pub fn spec_path(guide_root: &Path, id: &str) -> PathBuf {
    guide_root.join("diagrams").join(format!("{id}.json"))
}

pub fn d2_path(guide_root: &Path, id: &str) -> PathBuf {
    guide_root.join("data/d2").join(format!("{id}.d2"))
}

pub fn manifest_path(guide_root: &Path, id: &str) -> PathBuf {
    guide_root.join("data/diagrams").join(format!("{id}.json"))
}

pub fn spec_from_json(text: &str) -> Result<DiagramSpec> {
    let spec: DiagramSpec = serde_json::from_str(text)
        .map_err(|error| Error::Diagram(format!("diagram spec JSON: {error}")))?;
    validate_spec(&spec)?;
    Ok(spec)
}

pub fn spec_to_json(spec: &DiagramSpec) -> Result<String> {
    serde_json::to_string_pretty(spec)
        .map(|text| format!("{text}\n"))
        .map_err(|error| Error::Diagram(format!("diagram spec serialise: {error}")))
}

pub fn spec_hash(spec: &DiagramSpec) -> Result<String> {
    let text = serde_json::to_vec(spec)
        .map_err(|error| Error::Diagram(format!("diagram spec hash: {error}")))?;
    Ok(blake3::hash(&text).to_hex().to_string())
}

pub fn salt(spec_hash: &str) -> String {
    format!("fsl-{}", &spec_hash[..12.min(spec_hash.len())])
}

pub fn validate_spec(spec: &DiagramSpec) -> Result<()> {
    if spec.schema_version != SPEC_SCHEMA_VERSION {
        return Err(Error::Diagram(format!(
            "diagram schema_version is {}, expected {SPEC_SCHEMA_VERSION}",
            spec.schema_version
        )));
    }
    validate_id(&spec.id)?;
    if !matches!(spec.kind.as_str(), "flow" | "concept" | "tree" | "sequence") {
        return Err(Error::Diagram(format!(
            "unsupported diagram kind {:?}; use flow, concept, tree or sequence",
            spec.kind
        )));
    }
    for (name, value) in [
        ("title", &spec.title),
        ("caption", &spec.caption),
        ("alt", &spec.alt),
        ("source", &spec.source),
    ] {
        if value.trim().is_empty() {
            return Err(Error::Diagram(format!("{name} must not be empty")));
        }
    }
    if spec.alt.trim().to_lowercase().starts_with("image of")
        || spec.alt.trim().to_lowercase().starts_with("resim")
    {
        return Err(Error::Diagram(
            "alt must describe the takeaway, not say “image of”".to_string(),
        ));
    }
    if spec.nodes.is_empty() || spec.nodes.len() > MAX_NODES {
        return Err(Error::Diagram(format!(
            "diagram must have 1-{MAX_NODES} nodes (found {})",
            spec.nodes.len()
        )));
    }
    if spec.edges.len() > MAX_EDGES {
        return Err(Error::Diagram(format!(
            "diagram has too many edges ({} > {MAX_EDGES}); split it",
            spec.edges.len()
        )));
    }
    if !matches!(spec.layout.as_str(), "tala" | "elk" | "dagre") {
        return Err(Error::Diagram(format!(
            "unsupported D2 layout {:?}; use tala, elk or dagre",
            spec.layout
        )));
    }
    if let Some(direction) = &spec.direction
        && !matches!(direction.as_str(), "right" | "down" | "left" | "up")
    {
        return Err(Error::Diagram(format!(
            "unsupported direction {direction:?}; use right, down, left or up"
        )));
    }
    let ids = spec
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    if ids.len() != spec.nodes.len() {
        return Err(Error::Diagram("duplicate node id".to_string()));
    }
    for node in &spec.nodes {
        if node.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "node {:?} has an empty label",
                node.id
            )));
        }
    }
    for edge in &spec.edges {
        if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) {
            return Err(Error::Diagram(format!(
                "edge {:?} -> {:?} references an unknown node",
                edge.from, edge.to
            )));
        }
    }
    if spec.kind == "tree" {
        validate_tree(spec)?;
    }
    let group_ids = spec
        .groups
        .iter()
        .map(|group| group.id.as_str())
        .collect::<HashSet<_>>();
    if group_ids.len() != spec.groups.len() {
        return Err(Error::Diagram("duplicate group id".to_string()));
    }
    for group in &spec.groups {
        validate_id(&group.id)?;
        if group.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "group {:?} has an empty label",
                group.id
            )));
        }
        for id in &group.nodes {
            if !ids.contains(id.as_str()) {
                return Err(Error::Diagram(format!(
                    "group {:?} references unknown node {id:?}",
                    group.id
                )));
            }
        }
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<()> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && id.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '_'
                || character == '-'
        });
    if valid {
        Ok(())
    } else {
        Err(Error::Diagram(format!(
            "diagram id must match ^[a-z0-9][a-z0-9_-]{{0,63}}$: {id:?}"
        )))
    }
}

fn validate_tree(spec: &DiagramSpec) -> Result<()> {
    let mut incoming = HashSet::new();
    for edge in &spec.edges {
        incoming.insert(edge.to.as_str());
    }
    let roots = spec
        .nodes
        .iter()
        .filter(|node| !incoming.contains(node.id.as_str()))
        .count();
    if roots != 1 {
        return Err(Error::Diagram(
            "tree needs exactly one root (no incoming edges)".to_string(),
        ));
    }
    let depth = tree_depth(spec);
    if depth > MAX_DEPTH {
        return Err(Error::Diagram(format!(
            "tree depth {depth} exceeds {MAX_DEPTH}; split it"
        )));
    }
    Ok(())
}

fn tree_depth(spec: &DiagramSpec) -> usize {
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut incoming = HashSet::new();
    for edge in &spec.edges {
        children
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        incoming.insert(edge.to.as_str());
    }
    fn walk(id: &str, children: &HashMap<&str, Vec<&str>>, seen: &mut HashSet<String>) -> usize {
        if !seen.insert(id.to_string()) {
            return MAX_DEPTH + 1;
        }
        let depth = children
            .get(id)
            .into_iter()
            .flatten()
            .map(|child| walk(child, children, seen))
            .max()
            .unwrap_or(0)
            + 1;
        seen.remove(id);
        depth
    }
    spec.nodes
        .iter()
        .filter(|node| !incoming.contains(node.id.as_str()))
        .map(|node| walk(&node.id, &children, &mut HashSet::new()))
        .max()
        .unwrap_or(0)
}

pub fn d2_source(spec: &DiagramSpec, theme: &DiagramTheme) -> Result<String> {
    validate_spec(spec)?;
    let hash = spec_hash(spec)?;
    let node_keys = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), format!("n{index}")))
        .collect::<HashMap<_, _>>();
    let group_keys = spec
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| (group.id.as_str(), format!("g{index}")))
        .collect::<HashMap<_, _>>();
    let mut out = String::new();
    out.push_str(&format!("# spec_sha256: {hash}\n"));
    out.push_str(&format!("# engine: d2 {D2_VERSION}\n"));
    if spec.kind == "sequence" {
        out.push_str("shape: sequence_diagram\n");
        for node in &spec.nodes {
            out.push_str(&format!(
                "{}: {}\n",
                node_keys[node.id.as_str()],
                d2_quote(&node.label)
            ));
        }
        for edge in &spec.edges {
            out.push_str(&format!(
                "{} -> {}{}\n",
                node_keys[edge.from.as_str()],
                node_keys[edge.to.as_str()],
                edge.label
                    .as_deref()
                    .map(|label| format!(": {}", d2_quote(label)))
                    .unwrap_or_default()
            ));
        }
        return Ok(out);
    }

    let direction = spec.direction.as_deref().unwrap_or(if spec.kind == "tree" {
        "down"
    } else {
        DEFAULT_DIRECTION
    });
    out.push_str(&format!("direction: {direction}\n"));
    out.push_str(&format!("style: {{ fill: {} }}\n", d2_quote(&theme.paper)));
    out.push_str("*.style: {\n");
    out.push_str(&format!("  fill: {}\n", d2_quote(&theme.paper)));
    out.push_str(&format!("  stroke: {}\n", d2_quote(&theme.accent)));
    out.push_str(&format!("  font-color: {}\n", d2_quote(&theme.ink)));
    out.push_str("  font-size: 28\n");
    out.push_str("  border-radius: 3\n");
    out.push_str("}\n");
    out.push_str("(* -> *)[*].style: {\n");
    out.push_str(&format!("  stroke: {}\n", d2_quote(&theme.accent)));
    out.push_str(&format!("  font-color: {}\n", d2_quote(&theme.muted)));
    out.push_str("  font-size: 22\n");
    out.push_str("}\n");

    let mut grouped: HashMap<&str, &str> = HashMap::new();
    for group in &spec.groups {
        for node in &group.nodes {
            grouped.insert(node.as_str(), group.id.as_str());
        }
        out.push_str(&format!(
            "{}: {} {{\n",
            group_keys[group.id.as_str()],
            d2_quote(&group.label)
        ));
        for node in &group.nodes {
            let Some(node_spec) = spec.nodes.iter().find(|candidate| &candidate.id == node) else {
                continue;
            };
            out.push_str(&format!(
                "  {}: {}\n",
                node_keys[node_spec.id.as_str()],
                d2_quote(&node_spec.label)
            ));
        }
        out.push_str("}\n");
    }
    for node in &spec.nodes {
        if grouped.contains_key(node.id.as_str()) {
            continue;
        }
        out.push_str(&format!(
            "{}: {}\n",
            node_keys[node.id.as_str()],
            d2_quote(&node.label)
        ));
    }
    for edge in &spec.edges {
        out.push_str(&format!(
            "{} -> {}{}\n",
            d2_reference(edge.from.as_str(), &node_keys, &group_keys, &grouped),
            d2_reference(edge.to.as_str(), &node_keys, &group_keys, &grouped),
            edge.label
                .as_deref()
                .map(|label| format!(": {}", d2_quote(label)))
                .unwrap_or_default()
        ));
    }
    Ok(out)
}

fn d2_reference(
    id: &str,
    node_keys: &HashMap<&str, String>,
    group_keys: &HashMap<&str, String>,
    grouped: &HashMap<&str, &str>,
) -> String {
    let key = node_keys.get(id).cloned().unwrap_or_else(|| d2_key(id));
    match grouped.get(id) {
        Some(group) => {
            let group = group_keys
                .get(group)
                .cloned()
                .unwrap_or_else(|| d2_key(group));
            format!("{group}.{key}")
        }
        None => key,
    }
}

fn d2_key(id: &str) -> String {
    let mut key = String::from("n");
    for byte in id.bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(byte as char);
        } else {
            key.push('_');
        }
    }
    key
}

fn d2_quote(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' => out.push(' '),
            _ => out.push(character),
        }
    }
    out.push('"');
    out
}

pub fn postprocess_svg(
    svg: &str,
    title: &str,
    alt: &str,
    paper: &str,
    key: &str,
) -> Result<String> {
    let root_start = svg
        .find("<svg")
        .ok_or_else(|| Error::Diagram("D2 output has no <svg> root".to_string()))?;
    let root_end = svg[root_start..]
        .find('>')
        .map(|offset| root_start + offset)
        .ok_or_else(|| Error::Diagram("D2 SVG root is not closed".to_string()))?;
    let root = &svg[root_start..root_end];
    let title_id = format!("diagram-{key}-title");
    let desc_id = format!("diagram-{key}-desc");
    let mut root = root.to_string();
    if !root.contains("role=") {
        root = root.replacen("<svg", "<svg role=\"img\"", 1);
    }
    if !root.contains("aria-labelledby=") {
        root = root.replacen(
            "<svg",
            &format!("<svg aria-labelledby=\"{title_id} {desc_id}\""),
            1,
        );
    }
    let mut out = String::new();
    out.push_str(&svg[..root_start]);
    out.push_str(&root);
    out.push('>');
    out.push_str(&format!(
        "<title id=\"{title_id}\">{}</title><desc id=\"{desc_id}\">{}</desc>",
        xml_escape(title),
        xml_escape(alt)
    ));
    out.push_str(&svg[root_end + 1..]);
    let attrs = format!(
        "paint-order=\"stroke\" stroke=\"{}\" stroke-width=\"4\" stroke-linejoin=\"round\"",
        xml_escape(paper)
    );
    for class in ["class=\"text-italic fill-N2\"", "class=\"text-italic\""] {
        out = out.replace(class, &format!("{class} {attrs}"));
    }
    sanitize_svg(&out, Path::new("<d2-generated>"))?;
    Ok(out)
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn write_spec(guide_root: &Path, spec: &DiagramSpec) -> Result<PathBuf> {
    validate_spec(spec)?;
    let path = spec_path(guide_root, &spec.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            path: parent.to_path_buf(),
            source: error,
        })?;
    }
    std::fs::write(&path, spec_to_json(spec)?).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    Ok(path)
}

pub fn write_d2_source(
    guide_root: &Path,
    spec: &DiagramSpec,
    theme: &DiagramTheme,
) -> Result<PathBuf> {
    let path = d2_path(guide_root, &spec.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            path: parent.to_path_buf(),
            source: error,
        })?;
    }
    std::fs::write(&path, d2_source(spec, theme)?).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    Ok(path)
}

pub fn read_spec(guide_root: &Path, id: &str) -> Result<DiagramSpec> {
    let path = spec_path(guide_root, id);
    let text = std::fs::read_to_string(&path).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    let spec = spec_from_json(&text)?;
    if spec.id != id {
        return Err(Error::Diagram(format!(
            "spec id {:?} does not match filename {:?}",
            spec.id, id
        )));
    }
    Ok(spec)
}

pub fn read_specs(guide_root: &Path) -> Result<Vec<DiagramSpec>> {
    let directory = guide_root.join("diagrams");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut specs = Vec::new();
    for entry in std::fs::read_dir(&directory).map_err(|error| Error::Io {
        path: directory.clone(),
        source: error,
    })? {
        let entry = entry.map_err(|error| Error::Io {
            path: directory.clone(),
            source: error,
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::Diagram(format!("invalid spec filename: {}", path.display())))?;
        specs.push(read_spec(guide_root, id)?);
    }
    specs.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(specs)
}

pub fn load_assets(guide_root: &Path) -> Result<HashMap<String, DiagramAsset>> {
    let mut assets = HashMap::new();
    for spec in read_specs(guide_root)? {
        let manifest = read_manifest(guide_root, &spec.id)?.ok_or_else(|| {
            Error::Diagram(format!(
                "diagram {:?} has no manifest; run `folisift diagram build --guide <slug> --id {}`",
                spec.id, spec.id
            ))
        })?;
        let expected = spec_hash(&spec)?;
        if manifest.spec_sha256 != expected {
            return Err(Error::Diagram(format!(
                "diagram {:?} is stale; run `folisift diagram build --guide <slug> --id {}`",
                spec.id, spec.id
            )));
        }
        if manifest.d2_version != D2_VERSION || manifest.postprocess_version != POSTPROCESS_VERSION
        {
            return Err(Error::Diagram(format!(
                "diagram {:?} was built with a different engine/post-processor; rebuild it",
                spec.id
            )));
        }
        let svg_path = safe_asset_path(guide_root, &manifest.svg)?;
        let svg = std::fs::read(&svg_path).map_err(|error| {
            Error::Diagram(format!(
                "diagram {:?} SVG is missing ({}); run `folisift diagram build --guide <slug> --id {}`",
                spec.id,
                error,
                spec.id
            ))
        })?;
        let actual = blake3::hash(&svg).to_hex().to_string();
        if actual != manifest.svg_sha256 {
            return Err(Error::Diagram(format!(
                "diagram {:?} SVG hash mismatch; run `folisift diagram build --guide <slug> --id {}`",
                spec.id, spec.id
            )));
        }
        assets.insert(
            spec.id.clone(),
            DiagramAsset {
                id: spec.id.clone(),
                path: manifest.svg.clone(),
                caption: compose_caption(&spec),
                alt: spec.alt.clone(),
                source: spec.source.clone(),
            },
        );
    }
    Ok(assets)
}

pub fn compose_caption(spec: &DiagramSpec) -> String {
    format!("{} [{}]", spec.caption, spec.source)
}

pub fn read_manifest(guide_root: &Path, id: &str) -> Result<Option<DiagramManifest>> {
    let path = manifest_path(guide_root, id);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    let manifest = serde_json::from_str(&text)
        .map_err(|error| Error::Diagram(format!("diagram manifest {id:?}: {error}")))?;
    Ok(Some(manifest))
}

pub fn statuses(guide_slug: &str, guide_root: &Path) -> Result<Vec<DiagramStatus>> {
    let mut statuses = Vec::new();
    for spec in read_specs(guide_root)? {
        statuses.push(status_for_spec(guide_slug, guide_root, &spec)?);
    }
    let directory = guide_root.join("data/diagrams");
    if directory.is_dir() {
        for entry in std::fs::read_dir(&directory).map_err(|error| Error::Io {
            path: directory.clone(),
            source: error,
        })? {
            let entry = entry.map_err(|error| Error::Io {
                path: directory.clone(),
                source: error,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if read_specs(guide_root)?.iter().any(|spec| spec.id == id) {
                continue;
            }
            statuses.push(DiagramStatus {
                id: id.to_string(),
                state: DiagramState::MissingSpec,
                spec: spec_path(guide_root, id).display().to_string(),
                manifest: Some(path.display().to_string()),
                svg: None,
                command: format!("folisift diagram status --guide {guide_slug}"),
                message: Some("manifest has no matching spec".to_string()),
            });
        }
    }
    statuses.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(statuses)
}

fn status_for_spec(
    guide_slug: &str,
    guide_root: &Path,
    spec: &DiagramSpec,
) -> Result<DiagramStatus> {
    let command = format!(
        "folisift diagram build --guide {guide_slug} --id {}",
        spec.id
    );
    let manifest = match read_manifest(guide_root, &spec.id) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => {
            return Ok(DiagramStatus {
                id: spec.id.clone(),
                state: DiagramState::Missing,
                spec: spec_path(guide_root, &spec.id).display().to_string(),
                manifest: None,
                svg: None,
                command,
                message: Some("no manifest".to_string()),
            });
        }
        Err(error) => {
            return Ok(DiagramStatus {
                id: spec.id.clone(),
                state: DiagramState::Invalid,
                spec: spec_path(guide_root, &spec.id).display().to_string(),
                manifest: Some(manifest_path(guide_root, &spec.id).display().to_string()),
                svg: None,
                command,
                message: Some(error.to_string()),
            });
        }
    };
    if manifest.spec_sha256 != spec_hash(spec)?
        || manifest.svg_sha256.is_empty()
        || manifest.svg.is_empty()
    {
        return Ok(DiagramStatus {
            id: spec.id.clone(),
            state: DiagramState::Stale,
            spec: spec_path(guide_root, &spec.id).display().to_string(),
            manifest: Some(manifest_path(guide_root, &spec.id).display().to_string()),
            svg: Some(manifest.svg),
            command,
            message: Some("spec or hash is stale".to_string()),
        });
    }
    if manifest.d2_version != D2_VERSION || manifest.postprocess_version != POSTPROCESS_VERSION {
        return Ok(DiagramStatus {
            id: spec.id.clone(),
            state: DiagramState::EngineMismatch,
            spec: spec_path(guide_root, &spec.id).display().to_string(),
            manifest: Some(manifest_path(guide_root, &spec.id).display().to_string()),
            svg: Some(manifest.svg),
            command,
            message: Some(format!(
                "engine {} postprocess {}",
                manifest.d2_version, manifest.postprocess_version
            )),
        });
    }
    let svg_path = safe_asset_path(guide_root, &manifest.svg)?;
    let valid_svg = std::fs::read(&svg_path)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string() == manifest.svg_sha256)
        .unwrap_or(false);
    if !valid_svg {
        return Ok(DiagramStatus {
            id: spec.id.clone(),
            state: DiagramState::Stale,
            spec: spec_path(guide_root, &spec.id).display().to_string(),
            manifest: Some(manifest_path(guide_root, &spec.id).display().to_string()),
            svg: Some(manifest.svg),
            command,
            message: Some("SVG missing or hash mismatch".to_string()),
        });
    }
    Ok(DiagramStatus {
        id: spec.id.clone(),
        state: DiagramState::Ok,
        spec: spec_path(guide_root, &spec.id).display().to_string(),
        manifest: Some(manifest_path(guide_root, &spec.id).display().to_string()),
        svg: Some(manifest.svg),
        command,
        message: None,
    })
}

pub fn safe_asset_path(guide_root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(Error::Diagram(format!(
            "diagram asset path must stay inside the guide: {relative:?}"
        )));
    }
    Ok(guide_root.join(path))
}

pub fn write_manifest(guide_root: &Path, manifest: &DiagramManifest) -> Result<PathBuf> {
    let path = manifest_path(guide_root, &manifest.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            path: parent.to_path_buf(),
            source: error,
        })?;
    }
    let text = serde_json::to_string_pretty(manifest)
        .map_err(|error| Error::Diagram(format!("manifest serialise: {error}")))?;
    std::fs::write(&path, format!("{text}\n")).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    Ok(path)
}

pub fn write_asset(guide_root: &Path, hash: &str, svg: &str) -> Result<PathBuf> {
    let path = guide_root
        .join("assets/generated")
        .join(format!("{hash}.svg"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            path: parent.to_path_buf(),
            source: error,
        })?;
    }
    let tmp = path.with_extension("svg.tmp");
    std::fs::write(&tmp, svg).map_err(|error| Error::Io {
        path: tmp.clone(),
        source: error,
    })?;
    std::fs::rename(&tmp, &path).map_err(|error| Error::Io {
        path: path.clone(),
        source: error,
    })?;
    Ok(path)
}

pub fn ensure_no_inline_diagrams(markdown: &str) -> Result<()> {
    let mut fenced = false;
    for (index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            if trimmed.trim_start_matches(['`', '~']).trim() == "diagram" {
                return Err(Error::Markdown {
                    line: index + 1,
                    message: "inline diagram blocks are retired; move the JSON to diagrams/<id>.json and use ::: diagram id=\"<id>\"".to_string(),
                });
            }
            continue;
        }
        if fenced {
            continue;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> DiagramSpec {
        DiagramSpec {
            schema_version: SPEC_SCHEMA_VERSION,
            id: "pipeline".to_string(),
            kind: "flow".to_string(),
            title: "Derleme hattı".to_string(),
            caption: "Kaynak programdan amaç koda dönüşüm.".to_string(),
            alt: "Kaynak program analizden geçerek amaç koda dönüşür.".to_string(),
            source: "PD0, s. 35".to_string(),
            direction: Some("right".to_string()),
            layout: DEFAULT_LAYOUT.to_string(),
            nodes: vec![
                Node {
                    id: "src".to_string(),
                    label: "Kaynak program".to_string(),
                    group: None,
                },
                Node {
                    id: "lex".to_string(),
                    label: "Sözcüksel analiz".to_string(),
                    group: None,
                },
                Node {
                    id: "code".to_string(),
                    label: "Kod üretimi".to_string(),
                    group: None,
                },
            ],
            edges: vec![
                Edge {
                    from: "src".to_string(),
                    to: "lex".to_string(),
                    label: Some("karakterler".to_string()),
                },
                Edge {
                    from: "lex".to_string(),
                    to: "code".to_string(),
                    label: Some("tokenlar".to_string()),
                },
            ],
            groups: Vec::new(),
        }
    }

    #[test]
    fn validates_and_generates_deterministic_d2() {
        let spec = spec();
        assert!(validate_spec(&spec).is_ok());
        let theme = DiagramTheme::default();
        let first = d2_source(&spec, &theme).expect("d2");
        let second = d2_source(&spec, &theme).expect("d2");
        assert_eq!(first, second);
        assert!(first.contains("direction: right"));
        assert!(first.contains("n1:"));
        assert!(first.contains("karakterler"));
        assert!(first.contains("spec_sha256"));
    }

    #[test]
    fn accepts_twelve_nodes_and_rejects_thirteen() {
        let mut spec = spec();
        spec.nodes = (0..12)
            .map(|index| Node {
                id: format!("n{index}"),
                label: format!("Node {index}"),
                group: None,
            })
            .collect();
        spec.edges.clear();
        assert!(validate_spec(&spec).is_ok());
        spec.nodes.push(Node {
            id: "extra".to_string(),
            label: "Extra".to_string(),
            group: None,
        });
        assert!(validate_spec(&spec).is_err());
    }

    #[test]
    fn rejects_too_many_nodes_and_bad_ids() {
        let mut spec = spec();
        spec.nodes = (0..13)
            .map(|index| Node {
                id: format!("n{index}"),
                label: format!("Node {index}"),
                group: None,
            })
            .collect();
        spec.edges.clear();
        assert!(validate_spec(&spec).is_err());
        spec.nodes.truncate(1);
        spec.id = "Bad ID".to_string();
        assert!(validate_spec(&spec).is_err());
    }

    #[test]
    fn sequence_uses_sequence_shape() {
        let mut spec = spec();
        spec.kind = "sequence".to_string();
        spec.edges = vec![Edge {
            from: "src".to_string(),
            to: "lex".to_string(),
            label: Some("istek".to_string()),
        }];
        let source = d2_source(&spec, &DiagramTheme::default()).expect("sequence");
        assert!(source.contains("shape: sequence_diagram"));
        assert!(source.contains(" -> "));
    }

    #[test]
    fn postprocess_adds_accessibility_and_label_background() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><text class="text-italic fill-N2" x="1" y="1">etiket</text></svg>"#;
        let out =
            postprocess_svg(svg, "Başlık", "Açıklama", "#FAF6EC", "abc").expect("postprocess");
        assert!(out.contains("role=\"img\""));
        assert!(out.contains("aria-labelledby"));
        assert!(out.contains("<title"));
        assert!(out.contains("<desc"));
        assert!(out.contains("paint-order=\"stroke\""));
    }

    #[test]
    fn inline_diagram_blocks_are_rejected() {
        let error = ensure_no_inline_diagrams("```diagram\n{}\n```\n").expect_err("reject");
        assert!(error.to_string().contains("inline diagram"));
    }
}
