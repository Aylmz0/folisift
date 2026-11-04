//! Deterministic document graph built from stored layout blocks.
//!
//! The graph is a review artifact, not a coverage gate. Nodes carry source and
//! page provenance; edges are structural (`contains`, `next_page`, `cites`).
//! Semantic similarity lives in `folisift-links` and is written separately.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, io};
use crate::layout::{LayoutBlockKind, Rect, SourceLayout};

pub const GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentGraph {
    pub schema_version: u32,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_kind: Option<LayoutBlockKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub confidence: String,
}

impl DocumentGraph {
    pub fn build(layouts: &[SourceLayout]) -> Self {
        let mut builder = Builder::default();
        for layout in layouts {
            let source_id = layout.source.clone();
            builder.node(GraphNode {
                id: source_id.clone(),
                kind: "source".to_string(),
                source: source_id.clone(),
                page: None,
                label: Some(layout.source.clone()),
                block_kind: None,
                bbox: None,
                row: None,
                column: None,
            });
            let mut previous_page: Option<String> = None;
            let mut current_heading: Option<String> = None;
            for page in &layout.pages {
                let page_id = format!("{}:p{}", source_id, page.page);
                builder.node(GraphNode {
                    id: page_id.clone(),
                    kind: "page".to_string(),
                    source: source_id.clone(),
                    page: Some(page.page),
                    label: page.page_label.clone(),
                    block_kind: None,
                    bbox: page.content_bounds,
                    row: None,
                    column: None,
                });
                builder.edge(&source_id, &page_id, "contains");
                if let Some(previous) = previous_page.replace(page_id.clone()) {
                    builder.edge(&previous, &page_id, "next_page");
                }
                for block in &page.blocks {
                    let kind = if block.kind == LayoutBlockKind::Heading {
                        "heading"
                    } else {
                        "block"
                    };
                    builder.node(GraphNode {
                        id: block.id.clone(),
                        kind: kind.to_string(),
                        source: source_id.clone(),
                        page: Some(page.page),
                        label: block_label(block),
                        block_kind: Some(block.kind),
                        bbox: block.bbox,
                        row: None,
                        column: None,
                    });
                    builder.edge(&page_id, &block.id, "contains");
                    if let Some(heading) = &current_heading {
                        builder.edge(heading, &block.id, "contains");
                    }
                    if block.kind == LayoutBlockKind::Heading {
                        current_heading = Some(block.id.clone());
                    }
                    if block.kind.is_table() {
                        builder.table_cells(block, &source_id, page.page);
                    }
                    if block.kind == LayoutBlockKind::Figure {
                        let figure_id = format!("{}:figure", block.id);
                        builder.node(GraphNode {
                            id: figure_id.clone(),
                            kind: "figure".to_string(),
                            source: source_id.clone(),
                            page: Some(page.page),
                            label: block.figure_id.clone().or_else(|| block_label(block)),
                            block_kind: Some(block.kind),
                            bbox: block.bbox,
                            row: None,
                            column: None,
                        });
                        builder.edge(&block.id, &figure_id, "contains");
                    }
                }
            }
        }
        builder.finish()
    }

    /// Add `cites` edges from page references in guide Markdown. Unknown pages
    /// are ignored because the graph must not invent provenance.
    pub fn add_citations(&mut self, markdown: &str) {
        let mut seen = BTreeSet::new();
        for capture in PAGE_REF_RE.captures_iter(markdown) {
            let source = capture[1].trim();
            let Ok(page) = capture[2].parse::<usize>() else {
                continue;
            };
            let page_id = format!("{source}:p{page}");
            if !self.nodes.iter().any(|node| node.id == page_id) {
                continue;
            }
            let citation_id = format!("citation:{source}:p{page}");
            if seen.insert(citation_id.clone()) {
                self.nodes.push(GraphNode {
                    id: citation_id.clone(),
                    kind: "citation".to_string(),
                    source: source.to_string(),
                    page: Some(page),
                    label: Some(format!("[{source}, s. {page}]")),
                    block_kind: None,
                    bbox: None,
                    row: None,
                    column: None,
                });
            }
            if !self.edges.iter().any(|edge| {
                edge.from == citation_id && edge.to == page_id && edge.relation == "cites"
            }) {
                self.edges.push(GraphEdge {
                    from: citation_id,
                    to: page_id,
                    relation: "cites".to_string(),
                    confidence: "EXTRACTED".to_string(),
                });
            }
        }
        self.sort();
    }

    pub fn counts(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for node in &self.nodes {
            *counts.entry(node.kind.as_str()).or_insert(0) += 1;
        }
        counts
    }

    pub fn write_json(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        std::fs::write(path, format!("{json}\n")).map_err(|error| io(path, error))
    }

    pub fn read_json(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|error| io(path, error))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    }

    pub fn sort(&mut self) {
        self.nodes.sort_by(|left, right| left.id.cmp(&right.id));
        self.edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation).cmp(&(&right.from, &right.to, &right.relation))
        });
    }
}

use std::collections::BTreeMap;

static PAGE_REF_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)(?:\[)?\b([\p{L}\p{N}_.-]+)\s*,\s*(?:s|p)\.\s*(\d+)(?:\s*[–-]\s*(\d+))?(?:\])?",
    )
    .expect("valid page reference regex")
});

impl DocumentGraph {
    /// Case-insensitive label query. Heading nodes rank before blocks.
    pub fn query_matches(&self, term: &str) -> Vec<&GraphNode> {
        let needle = term.to_lowercase();
        let mut hits = self
            .nodes
            .iter()
            .filter(|node| {
                node.label
                    .as_deref()
                    .is_some_and(|label| label.to_lowercase().contains(&needle))
            })
            .collect::<Vec<_>>();
        hits.sort_by_key(|node| {
            (
                if node.kind == "heading" { 0 } else { 1 },
                node.page.unwrap_or(usize::MAX),
                node.id.clone(),
            )
        });
        hits
    }

    pub fn query(&self, term: &str, limit: usize) -> Vec<&GraphNode> {
        let mut hits = self.query_matches(term);
        hits.truncate(limit.clamp(1, 100));
        hits
    }

    /// Breadth-first shortest path between two node ids or labels.
    pub fn path(&self, from: &str, to: &str, max_depth: usize) -> Vec<String> {
        let Some(start) = self.resolve_node(from) else {
            return Vec::new();
        };
        let Some(goal) = self.resolve_node(to) else {
            return Vec::new();
        };
        let max_depth = max_depth.clamp(1, 12);
        let mut queue =
            std::collections::VecDeque::from([(start.id.clone(), vec![start.id.clone()])]);
        let mut seen = std::collections::BTreeSet::new();
        while let Some((current, path)) = queue.pop_front() {
            if current == goal.id {
                return path;
            }
            if path.len() > max_depth || !seen.insert(current.clone()) {
                continue;
            }
            for edge in self.edges.iter().filter(|edge| edge.from == current) {
                let mut next = path.clone();
                next.push(edge.to.clone());
                queue.push_back((edge.to.clone(), next));
            }
            for edge in self.edges.iter().filter(|edge| edge.to == current) {
                let mut next = path.clone();
                next.push(edge.from.clone());
                queue.push_back((edge.from.clone(), next));
            }
        }
        Vec::new()
    }

    /// Explain one node and its local edges. Returns bounded JSON-ready data.
    pub fn explain(&self, id: &str, limit: usize) -> Option<serde_json::Value> {
        let node = self.resolve_node(id)?;
        let limit = limit.clamp(1, 100);
        let outgoing = self
            .edges
            .iter()
            .filter(|edge| edge.from == node.id)
            .take(limit)
            .map(|edge| {
                serde_json::json!({
                    "to": edge.to,
                    "relation": edge.relation,
                    "confidence": edge.confidence,
                    "label": self.nodes.iter().find(|candidate| candidate.id == edge.to).and_then(|candidate| candidate.label.clone()),
                })
            })
            .collect::<Vec<_>>();
        let incoming = self
            .edges
            .iter()
            .filter(|edge| edge.to == node.id)
            .take(limit)
            .map(|edge| {
                serde_json::json!({
                    "from": edge.from,
                    "relation": edge.relation,
                    "confidence": edge.confidence,
                    "label": self.nodes.iter().find(|candidate| candidate.id == edge.from).and_then(|candidate| candidate.label.clone()),
                })
            })
            .collect::<Vec<_>>();
        Some(serde_json::json!({
            "node": node,
            "outgoing": outgoing,
            "incoming": incoming,
        }))
    }

    fn resolve_node(&self, query: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == query).or_else(|| {
            let needle = query.to_lowercase();
            self.nodes.iter().find(|node| {
                node.label
                    .as_deref()
                    .is_some_and(|label| label.to_lowercase().contains(&needle))
            })
        })
    }
}

#[derive(Default)]
struct Builder {
    graph: DocumentGraph,
    nodes: BTreeSet<String>,
    edges: BTreeSet<(String, String, String)>,
}

impl Builder {
    fn node(&mut self, node: GraphNode) {
        if self.nodes.insert(node.id.clone()) {
            self.graph.nodes.push(node);
        }
    }

    fn edge(&mut self, from: &str, to: &str, relation: &str) {
        let key = (from.to_string(), to.to_string(), relation.to_string());
        if self.edges.insert(key) {
            self.graph.edges.push(GraphEdge {
                from: from.to_string(),
                to: to.to_string(),
                relation: relation.to_string(),
                confidence: "EXTRACTED".to_string(),
            });
        }
    }

    fn table_cells(&mut self, block: &crate::layout::LayoutBlock, source: &str, page: usize) {
        for (column, cell) in block.header.iter().enumerate() {
            self.cell(block, source, page, 0, column, cell, "th");
        }
        for (row, cells) in block.rows.iter().enumerate() {
            for (column, cell) in cells.iter().enumerate() {
                self.cell(block, source, page, row + 1, column, cell, "td");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn cell(
        &mut self,
        block: &crate::layout::LayoutBlock,
        source: &str,
        page: usize,
        row: usize,
        column: usize,
        cell: &crate::layout::LayoutCell,
        role: &str,
    ) {
        let id = format!("{}:c{row}:{column}", block.id);
        self.node(GraphNode {
            id: id.clone(),
            kind: "table_cell".to_string(),
            source: source.to_string(),
            page: Some(page),
            label: Some(cell.text.clone()),
            block_kind: Some(LayoutBlockKind::Table),
            bbox: cell.bbox,
            row: Some(row),
            column: Some(column),
        });
        self.edge(&block.id, &id, "contains");
        let _ = role;
    }

    fn finish(mut self) -> DocumentGraph {
        self.graph.schema_version = GRAPH_SCHEMA_VERSION;
        self.graph.sort();
        self.graph
    }
}

fn block_label(block: &crate::layout::LayoutBlock) -> Option<String> {
    let text = block.text_or_empty().trim();
    if !text.is_empty() {
        return Some(text.chars().take(160).collect());
    }
    if !block.header.is_empty() {
        return Some(
            block
                .header
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
                .chars()
                .take(160)
                .collect(),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutBlock, LayoutCell, PageLayout};

    fn layout() -> SourceLayout {
        SourceLayout::new(
            "PD0",
            2,
            vec![
                PageLayout {
                    page: 1,
                    page_label: None,
                    width: 595.0,
                    height: 842.0,
                    content_bounds: None,
                    blocks: vec![
                        LayoutBlock {
                            id: "PD0:p1:b0".to_string(),
                            kind: LayoutBlockKind::Heading,
                            bbox: None,
                            text: Some("1. Giriş".to_string()),
                            level: Some(1),
                            bold: false,
                            italic: false,
                            ordered: None,
                            marker: None,
                            lines: vec![],
                            lang: None,
                            header: vec![],
                            rows: vec![],
                            header_rows: None,
                            figure_id: None,
                            figure_format: None,
                        },
                        LayoutBlock {
                            id: "PD0:p1:b1".to_string(),
                            kind: LayoutBlockKind::Table,
                            bbox: None,
                            text: None,
                            level: None,
                            bold: false,
                            italic: false,
                            ordered: None,
                            marker: None,
                            lines: vec![],
                            lang: None,
                            header: vec![LayoutCell {
                                text: "Ad".to_string(),
                                bbox: None,
                                colspan: None,
                                rowspan: None,
                            }],
                            rows: vec![vec![LayoutCell {
                                text: "Değer".to_string(),
                                bbox: None,
                                colspan: None,
                                rowspan: None,
                            }]],
                            header_rows: None,
                            figure_id: None,
                            figure_format: None,
                        },
                    ],
                },
                PageLayout {
                    page: 2,
                    page_label: None,
                    width: 595.0,
                    height: 842.0,
                    content_bounds: None,
                    blocks: vec![],
                },
            ],
        )
    }

    #[test]
    fn graph_has_source_page_block_and_cell_edges() {
        let graph = DocumentGraph::build(&[layout()]);
        assert!(graph.nodes.iter().any(|node| node.id == "PD0"));
        assert!(graph.nodes.iter().any(|node| node.id == "PD0:p1"));
        assert!(graph.nodes.iter().any(|node| node.id == "PD0:p1:b1:c0:0"));
        assert!(graph.edges.iter().any(|edge| {
            edge.from == "PD0:p1" && edge.to == "PD0:p2" && edge.relation == "next_page"
        }));
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.from == "PD0:p1:b0" && edge.to == "PD0:p1:b1")
        );
    }

    #[test]
    fn heading_nodes_are_queryable_and_explainable() {
        let graph = DocumentGraph::build(&[layout()]);
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.kind == "heading" && node.id == "PD0:p1:b0")
        );
        let hits = graph.query("giriş", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "heading");
        let path = graph.path("PD0:p2", "PD0:p1:b0", 8);
        assert_eq!(path.first().map(String::as_str), Some("PD0:p2"));
        assert_eq!(path.last().map(String::as_str), Some("PD0:p1:b0"));
        let explained = graph.explain("PD0:p1:b0", 10).expect("explain");
        assert_eq!(explained["node"]["kind"], "heading");
        assert!(
            !explained["incoming"]
                .as_array()
                .expect("incoming")
                .is_empty()
        );
    }

    #[test]
    fn citations_only_link_known_pages() {
        let mut graph = DocumentGraph::build(&[layout()]);
        graph.add_citations("[PD0, s. 1] [PD0, s. 99]");
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.relation == "cites" && edge.to == "PD0:p1")
        );
        assert!(
            !graph
                .edges
                .iter()
                .any(|edge| edge.relation == "cites" && edge.to == "PD0:p99")
        );
    }
}
