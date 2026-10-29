//! Deterministic, engine-owned diagram compiler.
//!
//! Guides provide a small JSON spec; this module turns it into accessible SVG.
//! There is no browser, network or arbitrary code path.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const GAP_X: f32 = 46.0;
const GAP_Y: f32 = 82.0;
const NODE_H: f32 = 38.0;
const MARGIN: f32 = 40.0;
const FONT_SIZE: f32 = 12.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagramSpec {
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
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

pub fn render_svg(spec: &DiagramSpec, theme: &DiagramTheme) -> Result<String> {
    validate(spec)?;
    let layout = match spec.kind.as_str() {
        "flow" | "concept" => layered_layout(spec, false)?,
        "tree" => tree_layout(spec)?,
        "sequence" => sequence_layout(spec)?,
        other => {
            return Err(Error::Diagram(format!(
                "unsupported diagram kind {other:?}; use flow, concept, tree or sequence"
            )));
        }
    };
    if layout.width > 900.0 {
        return Err(Error::Diagram(format!(
            "diagram is too wide ({:.0} units); split it or use fewer nodes per level",
            layout.width
        )));
    }
    if layout.height > 1_200.0 {
        return Err(Error::Diagram(format!(
            "diagram is too tall ({:.0} units); split it into overview and detail",
            layout.height
        )));
    }
    let title = xml_escape(&spec.title);
    let desc = xml_escape(spec.alt.as_deref().unwrap_or("A labeled study diagram."));
    let id = stable_id(spec);
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"title-{id} desc-{id}\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">\n",
        layout.width, layout.height, layout.width, layout.height
    ));
    svg.push_str(&format!(
        "<title id=\"title-{id}\">{title}</title>\n<desc id=\"desc-{id}\">{desc}</desc>\n"
    ));
    svg.push_str(&format!(
        "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n<defs><marker id=\"arrow-{id}\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"{}\"/></marker></defs>\n",
        theme.paper, theme.accent
    ));
    // Group boxes first.
    for group in &spec.groups {
        let members = group
            .nodes
            .iter()
            .filter_map(|id| layout.positions.get(id))
            .collect::<Vec<_>>();
        if members.is_empty() {
            continue;
        }
        let x = members.iter().map(|(x, _)| *x).fold(f32::MAX, f32::min) - 18.0;
        let y = members.iter().map(|(_, y)| *y).fold(f32::MAX, f32::min) - 28.0;
        let right = members.iter().map(|(x, _)| *x).fold(f32::MIN, f32::max) + 18.0;
        let bottom = members.iter().map(|(_, y)| *y).fold(f32::MIN, f32::max) + 28.0;
        let width = (right - x).max(60.0);
        let height = (bottom - y).max(50.0);
        svg.push_str(&format!(
            "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"3\" fill=\"none\" stroke=\"{}\" stroke-dasharray=\"4 3\"/>\n<text x=\"{:.1}\" y=\"{:.1}\" font-family=\"{}\" font-size=\"10\" fill=\"{}\">{}</text>\n",
            theme.rule,
            x + 6.0,
            y + 14.0,
            theme.font,
            theme.muted,
            xml_escape(&group.label)
        ));
    }
    // Edges.
    for edge in &spec.edges {
        let Some((x1, y1)) = layout.positions.get(&edge.from) else {
            continue;
        };
        let Some((x2, y2)) = layout.positions.get(&edge.to) else {
            continue;
        };
        let (sx, sy) = edge_start(*x1, *y1, *x2, *y2);
        let (ex, ey) = edge_end(*x1, *y1, *x2, *y2);
        svg.push_str(&format!(
            "<line x1=\"{sx:.1}\" y1=\"{sy:.1}\" x2=\"{ex:.1}\" y2=\"{ey:.1}\" stroke=\"{}\" stroke-width=\"1.2\" marker-end=\"url(#arrow-{id})\"/>\n",
            theme.accent
        ));
        if let Some(label) = &edge.label {
            let mx = (sx + ex) / 2.0;
            let my = (sy + ey) / 2.0;
            svg.push_str(&format!(
                "<text x=\"{mx:.1}\" y=\"{my:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"10\" fill=\"{}\" paint-order=\"stroke\" stroke=\"{}\" stroke-width=\"4\">{}</text>\n",
                theme.font,
                theme.muted,
                theme.paper,
                xml_escape(label)
            ));
        }
    }
    // Nodes.
    for node in &spec.nodes {
        let Some((x, y)) = layout.positions.get(&node.id) else {
            continue;
        };
        let (width, lines, height) = node_box(&node.label);
        let left = x - width / 2.0;
        let top = y - height / 2.0;
        let fill = theme.paper.as_str();
        svg.push_str(&format!(
            "<rect x=\"{left:.1}\" y=\"{top:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"3\" fill=\"{fill}\" stroke=\"{}\" stroke-width=\"1.2\"/>\n",
            theme.accent
        ));
        let mut ty = y - (lines.len() as f32 - 1.0) * 6.0 + 4.0;
        for line in lines {
            svg.push_str(&format!(
                "<text x=\"{x:.1}\" y=\"{ty:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{FONT_SIZE}\" fill=\"{}\">{}</text>\n",
                theme.font,
                theme.ink,
                xml_escape(&line)
            ));
            ty += 15.0;
        }
    }
    svg.push_str("</svg>\n");
    Ok(svg)
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

/// Replace ```diagram fences with generated, hashed SVG assets.
pub fn compile_body(
    body: &str,
    guide_root: &std::path::Path,
    theme: &DiagramTheme,
) -> Result<String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.starts_with("```diagram") {
            let start = index;
            let mut end = index + 1;
            while end < lines.len() && lines[end].trim() != "```" {
                end += 1;
            }
            if end >= lines.len() {
                return Err(Error::Diagram(format!(
                    "unclosed diagram block at line {}",
                    start + 1
                )));
            }
            let json = lines[start + 1..end].join("\n");
            let spec = spec_from_json(&json)?;
            let svg = render_svg(&spec, theme)?;
            let hash = blake3::hash(svg.as_bytes()).to_hex().to_string();
            let dir = guide_root.join("assets/generated");
            std::fs::create_dir_all(&dir).map_err(|error| Error::Io {
                path: dir.clone(),
                source: error,
            })?;
            let path = dir.join(format!("{hash}.svg"));
            if !path.is_file() {
                let tmp = path.with_extension("svg.tmp");
                std::fs::write(&tmp, &svg).map_err(|error| Error::Io {
                    path: tmp.clone(),
                    source: error,
                })?;
                std::fs::rename(&tmp, &path).map_err(|error| Error::Io {
                    path: path.clone(),
                    source: error,
                })?;
            }
            let caption = caption(&spec);
            let source = spec
                .source
                .as_deref()
                .map(|source| format!(" [{source}]"))
                .unwrap_or_default();
            out.push_str(&format!(
                "![{}{source}](assets/generated/{hash}.svg)\n",
                caption.replace(']', "\\]")
            ));
            index = end + 1;
        } else {
            out.push_str(lines[index]);
            out.push('\n');
            index += 1;
        }
    }
    Ok(out)
}

pub fn spec_from_json(text: &str) -> Result<DiagramSpec> {
    serde_json::from_str(text).map_err(|error| Error::Diagram(format!("diagram JSON: {error}")))
}

pub fn caption(spec: &DiagramSpec) -> String {
    spec.caption.clone().unwrap_or_else(|| spec.title.clone())
}

pub fn stable_id(spec: &DiagramSpec) -> String {
    let json = serde_json::to_string(spec).unwrap_or_default();
    blake3::hash(json.as_bytes()).to_hex().to_string()[..12].to_string()
}

fn validate(spec: &DiagramSpec) -> Result<()> {
    if spec.title.trim().is_empty() {
        return Err(Error::Diagram("title is required".to_string()));
    }
    if spec.nodes.is_empty() {
        return Err(Error::Diagram("at least one node is required".to_string()));
    }
    if spec.nodes.len() > 12 {
        return Err(Error::Diagram(
            "too many nodes (max 12); split the diagram".to_string(),
        ));
    }
    if spec.edges.len() > 20 {
        return Err(Error::Diagram(
            "too many edges (max 20); split the diagram".to_string(),
        ));
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
        let mut incoming = HashSet::new();
        for edge in &spec.edges {
            incoming.insert(edge.to.as_str());
        }
        if spec
            .nodes
            .iter()
            .filter(|node| !incoming.contains(node.id.as_str()))
            .count()
            != 1
        {
            return Err(Error::Diagram(
                "tree needs exactly one root (no incoming edges)".to_string(),
            ));
        }
    }
    Ok(())
}

struct Layout {
    positions: HashMap<String, (f32, f32)>,
    width: f32,
    height: f32,
}

fn layered_layout(spec: &DiagramSpec, _undirected: bool) -> Result<Layout> {
    let mut indegree = spec
        .nodes
        .iter()
        .map(|node| (node.id.clone(), 0usize))
        .collect::<HashMap<_, _>>();
    let mut outgoing = HashMap::<String, Vec<String>>::new();
    for edge in &spec.edges {
        *indegree.entry(edge.to.clone()).or_default() += 1;
        outgoing
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }
    let mut levels = HashMap::<String, usize>::new();
    let mut queue = VecDeque::new();
    for node in &spec.nodes {
        if indegree.get(&node.id).copied().unwrap_or(0) == 0 {
            levels.insert(node.id.clone(), 0);
            queue.push_back(node.id.clone());
        }
    }
    if queue.is_empty() {
        for node in &spec.nodes {
            levels.insert(node.id.clone(), 0);
            queue.push_back(node.id.clone());
        }
    }
    while let Some(id) = queue.pop_front() {
        let level = levels.get(&id).copied().unwrap_or(0);
        for target in outgoing.get(&id).into_iter().flatten() {
            let entry = levels.entry(target.clone()).or_insert(level + 1);
            if *entry <= level {
                *entry = level + 1;
            }
            if let Some(value) = indegree.get_mut(target) {
                *value = value.saturating_sub(1);
                if *value == 0 {
                    queue.push_back(target.clone());
                }
            }
        }
    }
    for node in &spec.nodes {
        levels.entry(node.id.clone()).or_insert(0);
    }
    let mut by_level = HashMap::<usize, Vec<String>>::new();
    for node in &spec.nodes {
        by_level
            .entry(levels.get(&node.id).copied().unwrap_or(0))
            .or_default()
            .push(node.id.clone());
    }
    let mut positions = HashMap::new();
    let max_level = by_level.keys().copied().max().unwrap_or(0);
    let mut max_width = 0.0f32;
    for level in 0..=max_level {
        let nodes = by_level.get(&level).cloned().unwrap_or_default();
        let widths = nodes
            .iter()
            .map(|id| {
                spec.nodes
                    .iter()
                    .find(|node| &node.id == id)
                    .map(|node| node_box(&node.label).0)
                    .unwrap_or(100.0)
            })
            .collect::<Vec<_>>();
        let total = widths.iter().sum::<f32>() + GAP_X * (nodes.len().saturating_sub(1)) as f32;
        max_width = max_width.max(total);
        let mut x = -total / 2.0;
        for (id, width) in nodes.iter().zip(widths.iter()) {
            positions.insert(id.clone(), (x + width / 2.0, level as f32 * GAP_Y));
            x += width + GAP_X;
        }
    }
    let width = max_width + MARGIN * 2.0;
    let height = max_level as f32 * GAP_Y + NODE_H + MARGIN * 2.0;
    for (_, y) in positions.values_mut() {
        *y += MARGIN + NODE_H / 2.0;
    }
    for (x, _) in positions.values_mut() {
        *x += width / 2.0;
    }
    Ok(Layout {
        positions,
        width,
        height,
    })
}

fn tree_layout(spec: &DiagramSpec) -> Result<Layout> {
    let mut children = HashMap::<String, Vec<String>>::new();
    let mut incoming = HashSet::new();
    for edge in &spec.edges {
        children
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        incoming.insert(edge.to.as_str());
    }
    let root = spec
        .nodes
        .iter()
        .find(|node| !incoming.contains(node.id.as_str()))
        .map(|node| node.id.clone())
        .ok_or_else(|| Error::Diagram("tree root not found".to_string()))?;
    let mut positions = HashMap::new();
    let mut next_x = 0.0;
    fn place(
        id: &str,
        depth: usize,
        children: &HashMap<String, Vec<String>>,
        positions: &mut HashMap<String, (f32, f32)>,
        next_x: &mut f32,
    ) -> f32 {
        let kids = children.get(id).cloned().unwrap_or_default();
        let x = if kids.is_empty() {
            let x = *next_x;
            *next_x += 130.0;
            x
        } else {
            let xs = kids
                .iter()
                .map(|kid| place(kid, depth + 1, children, positions, next_x))
                .collect::<Vec<_>>();
            (xs.first().copied().unwrap_or(0.0) + xs.last().copied().unwrap_or(0.0)) / 2.0
        };
        positions.insert(id.to_string(), (x, depth as f32 * GAP_Y));
        x
    }
    place(&root, 0, &children, &mut positions, &mut next_x);
    let width = (next_x + 60.0).max(260.0);
    let max_depth = positions.values().map(|(_, y)| *y).fold(0.0f32, f32::max);
    for (x, y) in positions.values_mut() {
        *x += 60.0;
        *y += MARGIN + NODE_H / 2.0;
    }
    Ok(Layout {
        positions,
        width,
        height: max_depth + NODE_H + MARGIN * 2.0,
    })
}

fn sequence_layout(spec: &DiagramSpec) -> Result<Layout> {
    if spec.edges.len() > 12 {
        return Err(Error::Diagram(
            "sequence diagrams support at most 12 messages".to_string(),
        ));
    }
    let mut positions = HashMap::new();
    for (index, node) in spec.nodes.iter().enumerate() {
        positions.insert(node.id.clone(), (80.0 + index as f32 * 170.0, 40.0));
    }
    Ok(Layout {
        positions,
        width: 160.0 + spec.nodes.len() as f32 * 170.0,
        height: 120.0 + spec.edges.len() as f32 * 55.0,
    })
}

fn node_box(label: &str) -> (f32, Vec<String>, f32) {
    let words = label.split_whitespace().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in words {
        if !current.is_empty() && current.chars().count() + word.chars().count() + 1 > 26 {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    let max_chars = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(1);
    let width = (max_chars as f32 * 7.0 + 24.0).clamp(96.0, 210.0);
    let height = NODE_H + (lines.len().saturating_sub(1) as f32 * 15.0);
    (width, lines, height)
}

fn edge_start(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
    if (y2 - y1).abs() > 20.0 {
        (x1, y1 + NODE_H / 2.0)
    } else if x2 > x1 {
        (x1 + 50.0, y1)
    } else {
        (x1 - 50.0, y1)
    }
}

fn edge_end(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
    if (y2 - y1).abs() > 20.0 {
        (x2, y2 - NODE_H / 2.0)
    } else if x2 > x1 {
        (x2 - 50.0, y2)
    } else {
        (x2 + 50.0, y2)
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow(node_count: usize) -> DiagramSpec {
        DiagramSpec {
            kind: "flow".to_string(),
            title: "Pipeline".to_string(),
            alt: Some("A pipeline from source to output.".to_string()),
            caption: None,
            source: Some("PD0, s. 4".to_string()),
            direction: None,
            groups: Vec::new(),
            nodes: (0..node_count)
                .map(|index| Node {
                    id: format!("n{index}"),
                    label: format!("Stage {index}"),
                    group: None,
                })
                .collect(),
            edges: (1..node_count)
                .map(|index| Edge {
                    from: format!("n{}", index - 1),
                    to: format!("n{index}"),
                    label: None,
                })
                .collect(),
        }
    }

    #[test]
    fn output_is_deterministic_and_accessible() {
        let spec = flow(4);
        let first = render_svg(&spec, &DiagramTheme::default()).expect("svg");
        let second = render_svg(&spec, &DiagramTheme::default()).expect("svg");
        assert_eq!(first, second);
        assert_eq!(stable_id(&spec), stable_id(&spec));
        assert!(first.contains("role=\"img\""));
        assert!(first.contains("<title"));
        assert!(first.contains("<desc"));
        assert!(first.contains("viewBox"));
        assert!(!first.contains("<script"));
    }

    #[test]
    fn checked_in_fixtures_render_to_accessible_svg() {
        for name in ["concept", "flow", "tree", "sequence"] {
            let json = std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("../../tests/fixtures/diagrams/{name}.json")),
            )
            .unwrap_or_else(|error| panic!("{name}: {error}"));
            let spec = spec_from_json(&json).unwrap_or_else(|error| panic!("{name}: {error}"));
            let svg = render_svg(&spec, &DiagramTheme::default())
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            assert!(svg.contains("role=\"img\""), "{name}");
            assert!(svg.contains("<title"), "{name}");
            assert!(svg.contains("<desc"), "{name}");
            assert!(!svg.contains("<script"), "{name}");
        }
    }

    #[test]
    fn all_four_kinds_render() {
        for kind in ["concept", "flow", "tree", "sequence"] {
            let mut spec = flow(4);
            spec.kind = kind.to_string();
            let svg = render_svg(&spec, &DiagramTheme::default())
                .unwrap_or_else(|error| panic!("{kind}: {error}"));
            assert!(svg.starts_with("<svg"), "{kind}");
        }
    }

    #[test]
    fn rejects_a_diagram_over_the_node_limit() {
        let error = render_svg(&flow(13), &DiagramTheme::default()).expect_err("too large");
        assert!(error.to_string().contains("max 12"));
    }

    #[test]
    fn rejects_unknown_edge_targets() {
        let mut spec = flow(2);
        spec.edges.push(Edge {
            from: "n0".to_string(),
            to: "missing".to_string(),
            label: None,
        });
        let error = render_svg(&spec, &DiagramTheme::default()).expect_err("bad edge");
        assert!(error.to_string().contains("unknown node"));
    }

    #[test]
    fn rejects_a_tree_with_two_roots() {
        let spec = DiagramSpec {
            kind: "tree".to_string(),
            title: "Two roots".to_string(),
            alt: None,
            caption: None,
            source: None,
            direction: None,
            groups: Vec::new(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    group: None,
                },
            ],
            edges: Vec::new(),
        };
        let error = render_svg(&spec, &DiagramTheme::default()).expect_err("two roots");
        assert!(error.to_string().contains("exactly one root"));
    }
}
