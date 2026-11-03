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
pub const SPEC_SCHEMA_VERSION: u32 = 3;
pub const SUPPORTED_SPEC_SCHEMA_VERSIONS: &[u32] = &[2, 3];
pub const POSTPROCESS_VERSION: u32 = 1;
pub const TALA_SEEDS: &str = "1,2,3";
pub const MAX_NODES: usize = 24;
pub const MAX_EDGES: usize = 40;
pub const MAX_DEPTH: usize = 8;
pub const MAX_SEQUENCE_ACTORS: usize = 6;
pub const MAX_SEQUENCE_MESSAGES: usize = 10;
pub const MAX_CLASSES: usize = 6;
pub const MAX_CLASS_MEMBERS: usize = 12;
pub const MAX_TABLES: usize = 6;
pub const MAX_COLUMNS: usize = 12;
pub const DEFAULT_LAYOUT: &str = "tala";

const DEFAULT_DIRECTION: &str = "right";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default)]
    pub actors: Vec<Actor>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub notes: Vec<SequenceNote>,
    #[serde(default)]
    pub classes: Vec<ClassSpec>,
    #[serde(default)]
    pub tables: Vec<TableSpec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub shape: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub arrowhead: Option<String>,
    #[serde(default)]
    pub line: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceNote {
    pub actor: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSpec {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TableSpec {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagramTheme {
    pub ink: String,
    pub muted: String,
    pub accent: String,
    pub paper: String,
    pub rule: String,
    pub font: String,
    pub label_size: f64,
    pub edge_size: f64,
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
            label_size: 28.0,
            edge_size: 22.0,
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
        let size = |key: &str, fallback: f64| {
            tokens
                .get("diagram")
                .and_then(|value| value.get(key))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(fallback)
                .clamp(14.0, 48.0)
        };
        Self {
            ink: get("ink", "#211E19"),
            muted: get("muted", "#5E574B"),
            accent: get("accent", "#8A3E2F"),
            paper: get("paper", "#FAF6EC"),
            rule: get("rule", "#D8CEBD"),
            font: font.to_string(),
            label_size: size("label_size", 28.0),
            edge_size: size("edge_size", 22.0),
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
    #[serde(default)]
    pub tala_seeds: String,
    pub salt: String,
    pub fonts: Vec<String>,
    #[serde(default)]
    pub font_digest: String,
    #[serde(default)]
    pub theme_digest: String,
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

pub fn theme_digest(theme: &DiagramTheme) -> Result<String> {
    let bytes = serde_json::to_vec(theme)
        .map_err(|error| Error::Diagram(format!("diagram theme serialise: {error}")))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
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
    // Hash a canonical value so v2 specs without the new v3 fields keep their
    // old manifest hash. Empty new arrays/null option fields are omitted.
    let mut value = serde_json::to_value(spec)
        .map_err(|error| Error::Diagram(format!("diagram spec hash: {error}")))?;
    if let Some(object) = value.as_object_mut() {
        for key in ["actors", "messages", "notes", "classes", "tables"] {
            if object
                .get(key)
                .and_then(serde_json::Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                object.remove(key);
            }
        }
        if let Some(nodes) = object
            .get_mut("nodes")
            .and_then(serde_json::Value::as_array_mut)
        {
            for node in nodes {
                if let Some(node) = node.as_object_mut()
                    && node.get("shape").is_some_and(serde_json::Value::is_null)
                {
                    node.remove("shape");
                }
            }
        }
        if let Some(edges) = object
            .get_mut("edges")
            .and_then(serde_json::Value::as_array_mut)
        {
            for edge in edges {
                if let Some(edge) = edge.as_object_mut() {
                    for key in ["arrowhead", "line"] {
                        if edge.get(key).is_some_and(serde_json::Value::is_null) {
                            edge.remove(key);
                        }
                    }
                }
            }
        }
    }
    let text = serde_json::to_vec(&value)
        .map_err(|error| Error::Diagram(format!("diagram spec hash: {error}")))?;
    Ok(blake3::hash(&text).to_hex().to_string())
}

pub fn salt(spec_hash: &str) -> String {
    format!("fsl-{}", &spec_hash[..12.min(spec_hash.len())])
}

pub fn validate_spec(spec: &DiagramSpec) -> Result<()> {
    if !SUPPORTED_SPEC_SCHEMA_VERSIONS.contains(&spec.schema_version) {
        return Err(Error::Diagram(format!(
            "diagram schema_version is {}, expected one of {SUPPORTED_SPEC_SCHEMA_VERSIONS:?}",
            spec.schema_version
        )));
    }
    validate_id(&spec.id)?;
    if !matches!(
        spec.kind.as_str(),
        "flow" | "concept" | "tree" | "decision" | "sequence" | "class" | "erd"
    ) {
        return Err(Error::Diagram(format!(
            "unsupported diagram kind {:?}; use flow, concept, tree, decision, sequence, class or erd",
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
    if spec.layout != DEFAULT_LAYOUT {
        return Err(Error::Diagram(format!(
            "unsupported D2 layout {:?}; use {DEFAULT_LAYOUT:?}",
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
    match spec.kind.as_str() {
        "flow" | "concept" | "tree" | "decision" => validate_graph(spec),
        "sequence" if !spec.actors.is_empty() || !spec.messages.is_empty() => {
            validate_sequence(spec)
        }
        "sequence" => validate_graph(spec),
        "class" => validate_class(spec),
        "erd" => validate_erd(spec),
        _ => unreachable!("kind checked above"),
    }
}

fn validate_graph(spec: &DiagramSpec) -> Result<()> {
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
        if let Some(shape) = &node.shape
            && !matches!(
                shape.as_str(),
                "rectangle" | "document" | "diamond" | "cylinder" | "cloud" | "person"
            )
        {
            return Err(Error::Diagram(format!(
                "unsupported node shape {shape:?}; use rectangle, document, diamond, cylinder, cloud or person"
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
        validate_edge_style(edge)?;
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
    let mut grouped_nodes = HashSet::new();
    for group in &spec.groups {
        validate_id(&group.id)?;
        if group.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "group {:?} has an empty label",
                group.id
            )));
        }
        let mut local_nodes = HashSet::new();
        for id in &group.nodes {
            if !ids.contains(id.as_str()) {
                return Err(Error::Diagram(format!(
                    "group {:?} references unknown node {id:?}",
                    group.id
                )));
            }
            if !local_nodes.insert(id.as_str()) || !grouped_nodes.insert(id.as_str()) {
                return Err(Error::Diagram(format!(
                    "node {id:?} belongs to more than one group"
                )));
            }
        }
    }
    Ok(())
}

fn validate_sequence(spec: &DiagramSpec) -> Result<()> {
    if !spec.nodes.is_empty() || !spec.edges.is_empty() || !spec.groups.is_empty() {
        return Err(Error::Diagram(
            "sequence diagrams use actors/messages, not nodes/edges/groups".to_string(),
        ));
    }
    if spec.actors.len() < 2 || spec.actors.len() > MAX_SEQUENCE_ACTORS {
        return Err(Error::Diagram(format!(
            "sequence needs 2-{MAX_SEQUENCE_ACTORS} actors (found {})",
            spec.actors.len()
        )));
    }
    if spec.messages.is_empty() || spec.messages.len() > MAX_SEQUENCE_MESSAGES {
        return Err(Error::Diagram(format!(
            "sequence needs 1-{MAX_SEQUENCE_MESSAGES} messages (found {})",
            spec.messages.len()
        )));
    }
    let ids = spec
        .actors
        .iter()
        .map(|actor| actor.id.as_str())
        .collect::<HashSet<_>>();
    if ids.len() != spec.actors.len() {
        return Err(Error::Diagram("duplicate actor id".to_string()));
    }
    for actor in &spec.actors {
        validate_id(&actor.id)?;
        if actor.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "actor {:?} has an empty label",
                actor.id
            )));
        }
    }
    for message in &spec.messages {
        if !ids.contains(message.from.as_str()) || !ids.contains(message.to.as_str()) {
            return Err(Error::Diagram(format!(
                "message {:?} -> {:?} references an unknown actor",
                message.from, message.to
            )));
        }
        if message.label.trim().is_empty() {
            return Err(Error::Diagram(
                "message label must not be empty".to_string(),
            ));
        }
    }
    for note in &spec.notes {
        if !ids.contains(note.actor.as_str()) {
            return Err(Error::Diagram(format!(
                "note references unknown actor {:?}",
                note.actor
            )));
        }
        if note.label.trim().is_empty() {
            return Err(Error::Diagram("note label must not be empty".to_string()));
        }
    }
    Ok(())
}

fn validate_class(spec: &DiagramSpec) -> Result<()> {
    if !spec.nodes.is_empty() || !spec.actors.is_empty() || !spec.messages.is_empty() {
        return Err(Error::Diagram(
            "class diagrams use classes/edges, not nodes/actors/messages".to_string(),
        ));
    }
    if spec.classes.is_empty() || spec.classes.len() > MAX_CLASSES {
        return Err(Error::Diagram(format!(
            "class diagram needs 1-{MAX_CLASSES} classes (found {})",
            spec.classes.len()
        )));
    }
    let ids = spec
        .classes
        .iter()
        .map(|class| class.id.as_str())
        .collect::<HashSet<_>>();
    if ids.len() != spec.classes.len() {
        return Err(Error::Diagram("duplicate class id".to_string()));
    }
    for class in &spec.classes {
        validate_id(&class.id)?;
        if class.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "class {:?} has an empty label",
                class.id
            )));
        }
        if class.fields.len() + class.methods.len() > MAX_CLASS_MEMBERS {
            return Err(Error::Diagram(format!(
                "class {:?} has too many members (max {MAX_CLASS_MEMBERS})",
                class.id
            )));
        }
    }
    for edge in &spec.edges {
        if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) {
            return Err(Error::Diagram(format!(
                "class edge {:?} -> {:?} references an unknown class",
                edge.from, edge.to
            )));
        }
        validate_edge_style(edge)?;
    }
    Ok(())
}

fn validate_erd(spec: &DiagramSpec) -> Result<()> {
    if !spec.nodes.is_empty() || !spec.actors.is_empty() || !spec.messages.is_empty() {
        return Err(Error::Diagram(
            "ERD diagrams use tables/edges, not nodes/actors/messages".to_string(),
        ));
    }
    if spec.tables.is_empty() || spec.tables.len() > MAX_TABLES {
        return Err(Error::Diagram(format!(
            "ERD needs 1-{MAX_TABLES} tables (found {})",
            spec.tables.len()
        )));
    }
    let ids = spec
        .tables
        .iter()
        .map(|table| table.id.as_str())
        .collect::<HashSet<_>>();
    if ids.len() != spec.tables.len() {
        return Err(Error::Diagram("duplicate table id".to_string()));
    }
    for table in &spec.tables {
        validate_id(&table.id)?;
        if table.label.trim().is_empty() {
            return Err(Error::Diagram(format!(
                "table {:?} has an empty label",
                table.id
            )));
        }
        if table.columns.is_empty() || table.columns.len() > MAX_COLUMNS {
            return Err(Error::Diagram(format!(
                "table {:?} needs 1-{MAX_COLUMNS} columns",
                table.id
            )));
        }
        for column in &table.columns {
            if column.name.trim().is_empty() || column.type_name.trim().is_empty() {
                return Err(Error::Diagram(format!(
                    "table {:?} has a column with an empty name/type",
                    table.id
                )));
            }
            if let Some(constraint) = &column.constraint
                && !matches!(constraint.as_str(), "PK" | "FK" | "UNQ")
            {
                return Err(Error::Diagram(format!(
                    "unsupported column constraint {constraint:?}; use PK, FK or UNQ"
                )));
            }
        }
    }
    for edge in &spec.edges {
        validate_table_reference(&edge.from, spec)?;
        validate_table_reference(&edge.to, spec)?;
        validate_edge_style(edge)?;
    }
    Ok(())
}

fn validate_table_reference(reference: &str, spec: &DiagramSpec) -> Result<()> {
    let Some((table, column)) = reference.split_once('.') else {
        return Err(Error::Diagram(format!(
            "ERD reference {reference:?} must be table.column"
        )));
    };
    let Some(table) = spec.tables.iter().find(|candidate| candidate.id == table) else {
        return Err(Error::Diagram(format!(
            "ERD reference {reference:?} uses an unknown table"
        )));
    };
    if !table
        .columns
        .iter()
        .any(|candidate| candidate.name == column)
    {
        return Err(Error::Diagram(format!(
            "ERD reference {reference:?} uses an unknown column"
        )));
    }
    Ok(())
}

fn validate_edge_style(edge: &Edge) -> Result<()> {
    if let Some(arrowhead) = &edge.arrowhead
        && !matches!(
            arrowhead.as_str(),
            "triangle" | "none" | "cf-one" | "cf-many" | "cf-one-required" | "cf-many-required"
        )
    {
        return Err(Error::Diagram(format!(
            "unsupported arrowhead {arrowhead:?}"
        )));
    }
    if let Some(line) = &edge.line
        && !matches!(line.as_str(), "solid" | "dashed")
    {
        return Err(Error::Diagram(format!("unsupported line style {line:?}")));
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
    let mut parent_count = HashMap::<&str, usize>::new();
    for edge in &spec.edges {
        *parent_count.entry(edge.to.as_str()).or_default() += 1;
    }
    if parent_count.values().any(|count| *count > 1) {
        return Err(Error::Diagram(
            "tree node has multiple parents; use concept/flow or split it".to_string(),
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
    let mut out = String::new();
    out.push_str(&format!("# spec_sha256: {hash}\n"));
    out.push_str(&format!("# engine: d2 {D2_VERSION}\n"));
    match spec.kind.as_str() {
        "sequence" if !spec.actors.is_empty() || !spec.messages.is_empty() => {
            sequence_source(spec, &mut out)?;
        }
        "class" => class_source(spec, theme, &mut out)?,
        "erd" => erd_source(spec, theme, &mut out)?,
        _ => graph_source(spec, theme, &mut out)?,
    }
    ensure_source_allowlist(&out)?;
    Ok(out)
}

fn ensure_source_allowlist(source: &str) -> Result<()> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        for forbidden in [
            "import ",
            "icon:",
            "link:",
            "tooltip:",
            "vars:",
            "layers:",
            "scenarios:",
            "steps:",
            "animate",
            "near:",
            "far:",
        ] {
            if trimmed.starts_with(forbidden) {
                return Err(Error::Diagram(format!(
                    "generated D2 source contains forbidden directive {forbidden:?}"
                )));
            }
        }
    }
    Ok(())
}

fn sequence_source(spec: &DiagramSpec, out: &mut String) -> Result<()> {
    out.push_str("shape: sequence_diagram\n");
    let keys = spec
        .actors
        .iter()
        .enumerate()
        .map(|(index, actor)| (actor.id.as_str(), format!("a{index}")))
        .collect::<HashMap<_, _>>();
    for actor in &spec.actors {
        out.push_str(&format!(
            "{}: {}\n",
            keys[actor.id.as_str()],
            d2_quote(&actor.label)
        ));
    }
    for message in &spec.messages {
        out.push_str(&format!(
            "{} -> {}: {}\n",
            keys[message.from.as_str()],
            keys[message.to.as_str()],
            d2_quote(&message.label)
        ));
    }
    for note in &spec.notes {
        out.push_str(&format!(
            "{}: {{ note: {} }}\n",
            keys[note.actor.as_str()],
            d2_quote(&note.label)
        ));
    }
    Ok(())
}

fn class_source(spec: &DiagramSpec, theme: &DiagramTheme, out: &mut String) -> Result<()> {
    let direction = spec.direction.as_deref().unwrap_or(DEFAULT_DIRECTION);
    out.push_str(&format!("direction: {direction}\n"));
    let keys = spec
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.id.as_str(), format!("c{index}")))
        .collect::<HashMap<_, _>>();
    for class in &spec.classes {
        out.push_str(&format!(
            "{}: {} {{\n  shape: class\n",
            keys[class.id.as_str()],
            d2_quote(&class.label)
        ));
        for field in &class.fields {
            let (key, value) = split_member(field);
            out.push_str(&format!("  {}: {}\n", d2_quote(key), d2_quote(value)));
        }
        for method in &class.methods {
            let (key, value) = split_member(method);
            out.push_str(&format!("  {}: {}\n", d2_quote(key), d2_quote(value)));
        }
        out.push_str("}\n");
    }
    for edge in &spec.edges {
        write_edge(
            out,
            &keys[edge.from.as_str()],
            &keys[edge.to.as_str()],
            edge,
        );
    }
    apply_graph_style(spec, theme, out);
    Ok(())
}

fn erd_source(spec: &DiagramSpec, theme: &DiagramTheme, out: &mut String) -> Result<()> {
    let direction = spec.direction.as_deref().unwrap_or(DEFAULT_DIRECTION);
    out.push_str(&format!("direction: {direction}\n"));
    let keys = spec
        .tables
        .iter()
        .enumerate()
        .map(|(index, table)| (table.id.as_str(), format!("t{index}")))
        .collect::<HashMap<_, _>>();
    for table in &spec.tables {
        out.push_str(&format!(
            "{}: {} {{\n  shape: sql_table\n",
            keys[table.id.as_str()],
            d2_quote(&table.label)
        ));
        for column in &table.columns {
            let constraint = column
                .constraint
                .as_deref()
                .map(|value| match value {
                    "PK" => "primary_key",
                    "FK" => "foreign_key",
                    "UNQ" => "unique",
                    other => other,
                })
                .map(|value| format!(" {{constraint: {value}}}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "  {}: {}{}\n",
                d2_quote(&column.name),
                d2_quote(&column.type_name),
                constraint
            ));
        }
        out.push_str("}\n");
    }
    for edge in &spec.edges {
        let from = table_reference(&edge.from, &keys);
        let to = table_reference(&edge.to, &keys);
        write_edge(out, &from, &to, edge);
    }
    apply_graph_style(spec, theme, out);
    Ok(())
}

fn graph_source(spec: &DiagramSpec, theme: &DiagramTheme, out: &mut String) -> Result<()> {
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
    if spec.kind == "sequence" {
        out.push_str("shape: sequence_diagram\n");
    } else {
        let direction = spec.direction.as_deref().unwrap_or(if spec.kind == "tree" {
            "down"
        } else {
            DEFAULT_DIRECTION
        });
        out.push_str(&format!("direction: {direction}\n"));
    }
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
            write_node(out, "  ", &node_keys[node_spec.id.as_str()], node_spec);
        }
        out.push_str("}\n");
    }
    for node in &spec.nodes {
        if grouped.contains_key(node.id.as_str()) {
            continue;
        }
        write_node(out, "", &node_keys[node.id.as_str()], node);
    }
    for edge in &spec.edges {
        write_edge(
            out,
            &d2_reference(edge.from.as_str(), &node_keys, &group_keys, &grouped),
            &d2_reference(edge.to.as_str(), &node_keys, &group_keys, &grouped),
            edge,
        );
    }
    apply_graph_style(spec, theme, out);
    Ok(())
}

fn write_node(out: &mut String, indent: &str, key: &str, node: &Node) {
    if let Some(shape) = &node.shape {
        out.push_str(&format!(
            "{indent}{key}: {} {{\n{indent}  shape: {shape}\n{indent}}}\n",
            d2_quote(&node.label)
        ));
    } else {
        out.push_str(&format!("{indent}{key}: {}\n", d2_quote(&node.label)));
    }
}

fn write_edge(out: &mut String, from: &str, to: &str, edge: &Edge) {
    let connection = if edge.arrowhead.as_deref() == Some("none") {
        format!("{from} -- {to}")
    } else {
        format!("{from} -> {to}")
    };
    let label = edge
        .label
        .as_deref()
        .map(|label| format!(": {}", d2_quote(label)))
        .unwrap_or_default();
    let mut style = String::new();
    if let Some(arrowhead) = &edge.arrowhead
        && arrowhead != "none"
        && arrowhead != "triangle"
    {
        style.push_str(&format!("  target-arrowhead.shape: {arrowhead}\n"));
    }
    if edge.line.as_deref() == Some("dashed") {
        style.push_str("  style.stroke-dash: 4\n");
    }
    if style.is_empty() {
        out.push_str(&format!("{connection}{label}\n"));
    } else {
        out.push_str(&format!("{connection}{label} {{\n{style}}}\n"));
    }
}

fn apply_graph_style(spec: &DiagramSpec, theme: &DiagramTheme, out: &mut String) {
    if spec.kind == "sequence" {
        return;
    }
    out.push_str(&format!("style: {{ fill: {} }}\n", d2_quote(&theme.paper)));
    out.push_str("*.style: {\n");
    out.push_str(&format!("  fill: {}\n", d2_quote(&theme.paper)));
    out.push_str(&format!("  stroke: {}\n", d2_quote(&theme.accent)));
    out.push_str(&format!("  font-color: {}\n", d2_quote(&theme.ink)));
    out.push_str(&format!("  font-size: {:.0}\n", theme.label_size));
    out.push_str("  border-radius: 3\n");
    out.push_str("}\n");
    out.push_str("(* -> *)[*].style: {\n");
    out.push_str(&format!("  stroke: {}\n", d2_quote(&theme.accent)));
    out.push_str(&format!("  font-color: {}\n", d2_quote(&theme.muted)));
    out.push_str(&format!("  font-size: {:.0}\n", theme.edge_size));
    out.push_str("}\n");
}

fn split_member(member: &str) -> (&str, &str) {
    member
        .split_once(':')
        .map(|(key, value)| (key.trim(), value.trim()))
        .unwrap_or((member.trim(), "void"))
}

fn table_reference(reference: &str, keys: &HashMap<&str, String>) -> String {
    let Some((table, column)) = reference.split_once('.') else {
        return reference.to_string();
    };
    let key = keys
        .get(table)
        .cloned()
        .unwrap_or_else(|| table.to_string());
    format!("{key}.{column}")
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
    load_assets_filtered(guide_root, None, None)
}

/// Load only diagrams referenced by a Markdown body. Drafts with a spec but no
/// manifest therefore do not block an unrelated build.
pub fn load_assets_for(guide_root: &Path, markdown: &str) -> Result<HashMap<String, DiagramAsset>> {
    load_assets_with_theme(guide_root, markdown, None)
}

/// Load referenced diagrams and reject a manifest built with another theme.
pub fn load_assets_with_theme(
    guide_root: &Path,
    markdown: &str,
    theme_digest: Option<&str>,
) -> Result<HashMap<String, DiagramAsset>> {
    let ids = referenced_ids(markdown);
    load_assets_filtered(guide_root, Some(&ids), theme_digest)
}

pub fn referenced_ids(markdown: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(":::") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix("diagram") else {
            continue;
        };
        if !rest.starts_with(char::is_whitespace) {
            continue;
        }
        for token in rest.split_whitespace() {
            if let Some(value) = token.strip_prefix("id=") {
                let value = value.trim_matches(['"', '\'']);
                if !value.is_empty() {
                    ids.insert(value.to_string());
                }
            }
        }
    }
    ids
}

fn load_assets_filtered(
    guide_root: &Path,
    wanted: Option<&HashSet<String>>,
    expected_theme: Option<&str>,
) -> Result<HashMap<String, DiagramAsset>> {
    let slug = guide_slug(guide_root);
    let mut assets = HashMap::new();
    for spec in read_specs(guide_root)? {
        if wanted.is_some_and(|ids| !ids.contains(&spec.id)) {
            continue;
        }
        let manifest = read_manifest(guide_root, &spec.id)?.ok_or_else(|| {
            Error::Diagram(format!(
                "diagram {:?} has no manifest; run `folisift diagram build --guide {slug} --id {}`",
                spec.id, spec.id
            ))
        })?;
        let expected = spec_hash(&spec)?;
        if manifest.spec_sha256 != expected {
            return Err(Error::Diagram(format!(
                "diagram {:?} is stale; run `folisift diagram build --guide {slug} --id {}`",
                spec.id, spec.id
            )));
        }
        if manifest.d2_version != D2_VERSION
            || manifest.postprocess_version != POSTPROCESS_VERSION
            || manifest.tala_seeds != TALA_SEEDS
            || manifest.theme_digest.is_empty()
        {
            return Err(Error::Diagram(format!(
                "diagram {:?} was built with a different engine/post-processor; rebuild it with `folisift diagram build --guide {slug} --id {}`",
                spec.id, spec.id
            )));
        }
        if expected_theme.is_some_and(|digest| manifest.theme_digest != digest) {
            return Err(Error::Diagram(format!(
                "diagram {:?} was built with a different palette/font theme; rebuild it with `folisift diagram build --guide {slug} --id {}`",
                spec.id, spec.id
            )));
        }
        let svg_path = safe_asset_path(guide_root, &manifest.svg)?;
        let svg = std::fs::read(&svg_path).map_err(|error| {
            Error::Diagram(format!(
                "diagram {:?} SVG is missing ({}); run `folisift diagram build --guide {slug} --id {}`",
                spec.id,
                error,
                spec.id
            ))
        })?;
        let actual = blake3::hash(&svg).to_hex().to_string();
        if actual != manifest.svg_sha256 {
            return Err(Error::Diagram(format!(
                "diagram {:?} SVG hash mismatch; run `folisift diagram build --guide {slug} --id {}`",
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
    let mut known = HashSet::new();
    let directory = guide_root.join("diagrams");
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
            known.insert(id.to_string());
            match read_spec(guide_root, id) {
                Ok(spec) => statuses.push(status_for_spec(guide_slug, guide_root, &spec)?),
                Err(error) => statuses.push(DiagramStatus {
                    id: id.to_string(),
                    state: DiagramState::Invalid,
                    spec: path.display().to_string(),
                    manifest: None,
                    svg: None,
                    command: format!("folisift diagram status --guide {guide_slug}"),
                    message: Some(error.to_string()),
                }),
            }
        }
    }
    let manifests = guide_root.join("data/diagrams");
    if manifests.is_dir() {
        for entry in std::fs::read_dir(&manifests).map_err(|error| Error::Io {
            path: manifests.clone(),
            source: error,
        })? {
            let entry = entry.map_err(|error| Error::Io {
                path: manifests.clone(),
                source: error,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if !known.contains(id) {
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
    }
    statuses.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(statuses)
}

fn guide_slug(guide_root: &Path) -> String {
    guide_root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("<guide>")
        .to_string()
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
        || !d2_path(guide_root, &spec.id).is_file()
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
    if manifest.d2_version != D2_VERSION
        || manifest.postprocess_version != POSTPROCESS_VERSION
        || manifest.tala_seeds != TALA_SEEDS
        || manifest.theme_digest.is_empty()
    {
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
                    shape: None,
                },
                Node {
                    id: "lex".to_string(),
                    label: "Sözcüksel analiz".to_string(),
                    group: None,
                    shape: None,
                },
                Node {
                    id: "code".to_string(),
                    label: "Kod üretimi".to_string(),
                    group: None,
                    shape: None,
                },
            ],
            edges: vec![
                Edge {
                    from: "src".to_string(),
                    to: "lex".to_string(),
                    label: Some("karakterler".to_string()),
                    arrowhead: None,
                    line: None,
                },
                Edge {
                    from: "lex".to_string(),
                    to: "code".to_string(),
                    label: Some("tokenlar".to_string()),
                    arrowhead: None,
                    line: None,
                },
            ],
            groups: Vec::new(),
            actors: Vec::new(),
            messages: Vec::new(),
            notes: Vec::new(),
            classes: Vec::new(),
            tables: Vec::new(),
        }
    }

    #[test]
    fn referenced_diagrams_ignore_unrelated_drafts() {
        let markdown =
            "::: diagram id=\"pipeline\" caption=\"x\"\n:::\n\n::: note\nid=other\n:::\n";
        let ids = referenced_ids(markdown);
        assert!(ids.contains("pipeline"));
        assert!(!ids.contains("other"));
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
    fn accepts_twenty_four_nodes_and_rejects_twenty_five() {
        let mut spec = spec();
        spec.nodes = (0..24)
            .map(|index| Node {
                id: format!("n{index}"),
                label: format!("Node {index}"),
                group: None,
                shape: None,
            })
            .collect();
        spec.edges.clear();
        assert!(validate_spec(&spec).is_ok());
        spec.nodes.push(Node {
            id: "extra".to_string(),
            label: "Extra".to_string(),
            group: None,
            shape: None,
        });
        assert!(validate_spec(&spec).is_err());
    }

    #[test]
    fn rejects_too_many_nodes_and_bad_ids() {
        let mut spec = spec();
        spec.nodes = (0..25)
            .map(|index| Node {
                id: format!("n{index}"),
                label: format!("Node {index}"),
                group: None,
                shape: None,
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
            arrowhead: None,
            line: None,
        }];
        let source = d2_source(&spec, &DiagramTheme::default()).expect("sequence");

        assert!(source.contains("shape: sequence_diagram"));
        assert!(source.contains(" -> "));
    }

    #[test]
    fn sequence_v3_uses_actors_messages_and_notes() {
        let mut spec = spec();
        spec.kind = "sequence".to_string();
        spec.nodes.clear();
        spec.edges.clear();
        spec.actors = vec![
            Actor {
                id: "client".to_string(),
                label: "Client".to_string(),
            },
            Actor {
                id: "server".to_string(),
                label: "Server".to_string(),
            },
        ];
        spec.messages = vec![Message {
            from: "client".to_string(),
            to: "server".to_string(),
            label: "SYN".to_string(),
        }];
        spec.notes = vec![SequenceNote {
            actor: "client".to_string(),
            label: "seq = x".to_string(),
        }];
        let source = d2_source(&spec, &DiagramTheme::default()).expect("sequence");

        assert!(source.contains("shape: sequence_diagram"));
        assert!(source.contains("a0: \"Client\""));
        assert!(source.contains("a0 -> a1: \"SYN\""));
        assert!(source.contains("note: \"seq = x\""));
    }

    #[test]
    fn class_v3_uses_class_shape_and_fields() {
        let mut spec = spec();
        spec.kind = "class".to_string();
        spec.nodes.clear();
        spec.edges.clear();
        spec.classes = vec![
            ClassSpec {
                id: "expr".to_string(),
                label: "Expr".to_string(),
                fields: vec!["span: Span".to_string()],
                methods: vec!["eval(): Value".to_string()],
            },
            ClassSpec {
                id: "literal".to_string(),
                label: "Literal".to_string(),
                fields: vec!["value: Value".to_string()],
                methods: Vec::new(),
            },
        ];
        spec.edges = vec![Edge {
            from: "literal".to_string(),
            to: "expr".to_string(),
            label: Some("inherits".to_string()),
            arrowhead: Some("triangle".to_string()),
            line: None,
        }];
        let source = d2_source(&spec, &DiagramTheme::default()).expect("class");

        assert!(source.contains("shape: class"));
        assert!(source.contains("\"span\": \"Span\""));
        assert!(source.contains("\"eval()\": \"Value\""));
        assert!(source.contains("c1 -> c0"));
    }

    #[test]
    fn erd_v3_uses_sql_table_and_constraints() {
        let mut spec = spec();
        spec.kind = "erd".to_string();
        spec.nodes.clear();
        spec.edges.clear();
        spec.tables = vec![
            TableSpec {
                id: "customers".to_string(),
                label: "customers".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    type_name: "int".to_string(),
                    constraint: Some("PK".to_string()),
                }],
            },
            TableSpec {
                id: "orders".to_string(),
                label: "orders".to_string(),
                columns: vec![Column {
                    name: "customer_id".to_string(),
                    type_name: "int".to_string(),
                    constraint: Some("FK".to_string()),
                }],
            },
        ];
        spec.edges = vec![Edge {
            from: "orders.customer_id".to_string(),
            to: "customers.id".to_string(),
            label: Some("many-to-one".to_string()),
            arrowhead: Some("cf-many".to_string()),
            line: None,
        }];
        let source = d2_source(&spec, &DiagramTheme::default()).expect("erd");

        assert!(source.contains("shape: sql_table"));
        assert!(source.contains("constraint: primary_key"));
        assert!(source.contains("t1.customer_id -> t0.id"));
        assert!(source.contains("target-arrowhead.shape: cf-many"));
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
    fn rejects_unknown_spec_fields_and_non_tala_layout() {
        let mut value = serde_json::to_value(spec()).expect("value");
        value["engine"] = serde_json::json!("graphviz");
        assert!(spec_from_json(&value.to_string()).is_err());
        let mut value = serde_json::to_value(spec()).expect("value");
        value["layout"] = serde_json::json!("dagre");
        assert!(spec_from_json(&value.to_string()).is_err());
    }

    #[test]
    fn tree_rejects_multiple_parents() {
        let mut spec = spec();
        spec.kind = "tree".to_string();
        spec.edges.push(Edge {
            from: "src".to_string(),
            to: "code".to_string(),
            label: None,
            arrowhead: None,
            line: None,
        });
        let error = validate_spec(&spec).expect_err("multi-parent tree");
        assert!(error.to_string().contains("multiple parents"), "{error}");
    }

    #[test]
    fn inline_diagram_blocks_are_rejected() {
        let error = ensure_no_inline_diagrams("```diagram\n{}\n```\n").expect_err("reject");
        assert!(error.to_string().contains("inline diagram"));
    }
}
