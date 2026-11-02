//! Source-scope coverage for revision sheets and knowledge organisers.
//!
//! These genres are selective by design, so they do not use the A–G heading
//! gate. They declare a page scope in frontmatter and must cite every page in
//! that scope at least once, using `[PD2, s. 26–29]` style references.

use serde::Deserialize;

use crate::inventory::Inventory;

static PAGE_REF_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)(?:\[)?\b([\p{L}\p{N}_.-]+)\s*,\s*(?:s|p)\.\s*(\d+)(?:\s*[–-]\s*(\d+))?(?:\])?",
    )
    .expect("valid page reference regex")
});

#[derive(Debug, Deserialize)]
struct ScopeFrontmatter {
    #[serde(default)]
    source_scope: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RevisionCoverageReport {
    pub scope: Option<String>,
    pub issues: Vec<String>,
    pub passed: bool,
}

impl RevisionCoverageReport {
    pub fn write_markdown(&self, path: &std::path::Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| crate::error::io(parent, error))?;
        }
        let mut out = String::from("# Revision source coverage\n\n");
        if let Some(scope) = &self.scope {
            out.push_str(&format!("Scope: `{scope}`\n\n"));
        }
        out.push_str(&format!(
            "Result: **{}**\n\n",
            if self.passed { "PASS" } else { "FAIL" }
        ));
        if self.issues.is_empty() {
            out.push_str("Every page in the declared scope is cited.\n");
        } else {
            for issue in &self.issues {
                out.push_str(&format!("- {issue}\n"));
            }
        }
        std::fs::write(path, out).map_err(|error| crate::error::io(path, error))
    }
}

#[derive(Debug, Clone)]
enum ScopeSelector {
    Pages { start: usize, end: usize },
    Section(String),
}

#[derive(Debug, Clone)]
struct Scope {
    source: String,
    selector: ScopeSelector,
}

pub fn revision_coverage_report(inventory: &Inventory, markdown: &str) -> RevisionCoverageReport {
    let scope = frontmatter(markdown).and_then(|value| value.source_scope);
    let issues = revision_coverage_issues(inventory, markdown);
    RevisionCoverageReport {
        scope,
        passed: issues.is_empty(),
        issues,
    }
}

pub fn revision_coverage_issues(inventory: &Inventory, markdown: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let frontmatter = frontmatter(markdown);
    let Some(scope_text) = frontmatter.and_then(|value| value.source_scope) else {
        issues.push(
            "revision/organiser frontmatter must declare source_scope, e.g. \"PD2:26-29\""
                .to_string(),
        );
        return issues;
    };
    let scopes = match parse_scopes(&scope_text) {
        Ok(scopes) => scopes,
        Err(error) => {
            issues.push(error);
            return issues;
        }
    };
    let mut known = Vec::new();
    for source in &inventory.sources {
        known.push((
            source.filename.trim_end_matches(".pdf").to_string(),
            source.pages,
            &source.headings,
        ));
    }
    for scope in &scopes {
        let Some((_, pages, headings)) = known.iter().find(|(id, _, _)| id == &scope.source) else {
            issues.push(format!(
                "source_scope references unknown source {:?}",
                scope.source
            ));
            continue;
        };
        let (start, end) = match &scope.selector {
            ScopeSelector::Pages { start, end } => (*start, *end),
            ScopeSelector::Section(section) => match resolve_section(*pages, headings, section) {
                Some(range) => range,
                None => {
                    issues.push(format!(
                        "source_scope section {section:?} was not found in source {:?}",
                        scope.source
                    ));
                    continue;
                }
            },
        };
        if start == 0 || end < start || end > *pages {
            issues.push(format!(
                "source_scope {}:{}-{} is outside the source page range 1-{pages}",
                scope.source, start, end
            ));
            continue;
        }
        let mut cited = vec![false; end - start + 1];
        let mut saw_reference = false;
        for capture in PAGE_REF_RE.captures_iter(markdown) {
            let source = capture[1].trim().to_string();
            if source != scope.source {
                continue;
            }
            saw_reference = true;
            let reference_start = capture[2].parse::<usize>().unwrap_or(0);
            let reference_end = capture
                .get(3)
                .and_then(|value| value.as_str().parse::<usize>().ok())
                .unwrap_or(reference_start);
            if reference_start < start || reference_end > end || reference_end < reference_start {
                issues.push(format!(
                    "reference {}:{}-{} falls outside source_scope {}:{}-{}",
                    source, reference_start, reference_end, scope.source, start, end
                ));
                continue;
            }
            for page in reference_start..=reference_end {
                cited[page - start] = true;
            }
        }
        if !saw_reference {
            issues.push(format!(
                "no page references for scoped source {}",
                scope.source
            ));
            continue;
        }
        let missing = cited
            .iter()
            .enumerate()
            .filter_map(|(index, seen)| (!seen).then_some(start + index))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            issues.push(format!(
                "source_scope {}:{}-{} has uncited pages: {missing:?}",
                scope.source, start, end
            ));
        }
    }
    issues
}

fn resolve_section(
    pages: usize,
    headings: &[crate::inventory::Heading],
    section: &str,
) -> Option<(usize, usize)> {
    let section = section.trim().trim_end_matches('.');
    if section.is_empty() {
        return None;
    }
    let mut ordered = headings
        .iter()
        .filter_map(|heading| {
            let number = heading_number(&heading.heading)?;
            Some((heading.page, number))
        })
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(page, _)| *page);
    let start = ordered
        .iter()
        .find(|(_, number)| *number == section)
        .map(|(page, _)| *page)?;
    let end = ordered
        .iter()
        .filter(|(page, number)| {
            *page > start && *number != section && !number.starts_with(&format!("{section}."))
        })
        .map(|(page, _)| page.saturating_sub(1))
        .min()
        .unwrap_or(pages);
    Some((start, end))
}

fn heading_number(heading: &str) -> Option<&str> {
    let first = heading.split_whitespace().next()?.trim_end_matches('.');
    (!first.is_empty()
        && first
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.'))
    .then_some(first)
}

fn frontmatter(markdown: &str) -> Option<ScopeFrontmatter> {
    let rest = markdown.strip_prefix("---\n")?;
    let (yaml, _) = rest.split_once("\n---\n")?;
    serde_yaml::from_str(yaml).ok()
}

fn parse_scopes(value: &str) -> Result<Vec<Scope>, String> {
    let mut scopes = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((source, range)) = part.split_once(':') else {
            return Err(format!(
                "invalid source_scope entry {part:?}; use SOURCE:START-END or SOURCE:section:NUMBER"
            ));
        };
        let selector = if let Some(section) = range.trim().strip_prefix("section:") {
            let section = section.trim();
            if section.is_empty() {
                return Err(format!("empty source_scope section in {part:?}"));
            }
            ScopeSelector::Section(section.to_string())
        } else {
            let (start, end) = match range.split_once('-') {
                Some((start, end)) => (start, end),
                None => (range, range),
            };
            let start = start
                .trim()
                .parse::<usize>()
                .map_err(|_| format!("invalid source_scope start in {part:?}"))?;
            let end = end
                .trim()
                .parse::<usize>()
                .map_err(|_| format!("invalid source_scope end in {part:?}"))?;
            ScopeSelector::Pages { start, end }
        };
        scopes.push(Scope {
            source: source.trim().to_string(),
            selector,
        });
    }
    if scopes.is_empty() {
        Err("source_scope is empty".to_string())
    } else {
        Ok(scopes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Heading, SourceInventory, Totals};

    fn inventory() -> Inventory {
        Inventory {
            generated_at: "now".to_string(),
            low_text_threshold: 120,
            sources: vec![SourceInventory {
                path: "PD2.pdf".to_string(),
                filename: "PD2.pdf".to_string(),
                title: "PD2".to_string(),
                pages: 30,
                chars: 0,
                words: 0,
                low_text_threshold: 120,
                source_digest: None,
                low_text_pages: Vec::new(),
                headings: Vec::new(),
                layout: None,
                sections: Vec::new(),
                page_map: Vec::new(),
            }],
            totals: Totals {
                pages: 30,
                words: 0,
                low_text_pages: 0,
            },
        }
    }

    fn guide(scope: &str, refs: &str) -> String {
        format!("---\ntitle: Test\nsource_scope: \"{scope}\"\n---\n\n{refs}\n")
    }

    fn section_inventory() -> Inventory {
        let mut inventory = inventory();
        inventory.sources[0].pages = 40;
        inventory.sources[0].headings = vec![
            Heading {
                heading: "2.17 Önceki konu".to_string(),
                page: 20,
                level: Some(1),
                block_id: None,
            },
            Heading {
                heading: "2.18 Kapsam konusu".to_string(),
                page: 26,
                level: Some(1),
                block_id: None,
            },
            Heading {
                heading: "2.18.1 Alt konu".to_string(),
                page: 27,
                level: Some(2),
                block_id: None,
            },
            Heading {
                heading: "2.19 Sonraki konu".to_string(),
                page: 30,
                level: Some(1),
                block_id: None,
            },
            Heading {
                heading: "2.20 Son konu".to_string(),
                page: 35,
                level: Some(1),
                block_id: None,
            },
        ];
        inventory
    }

    #[test]
    fn accepts_a_section_scope() {
        let markdown = guide("PD2:section:2.18", "A [PD2, s. 26–29].");
        assert!(
            revision_coverage_issues(&section_inventory(), &markdown).is_empty(),
            "{:?}",
            revision_coverage_issues(&section_inventory(), &markdown)
        );
    }

    #[test]
    fn rejects_uncited_pages_in_a_section_scope() {
        let markdown = guide("PD2:section:2.18", "Only [PD2, s. 26].");
        let issues = revision_coverage_issues(&section_inventory(), &markdown);
        assert!(
            issues.iter().any(|issue| issue.contains("uncited pages")),
            "{issues:?}"
        );
    }

    #[test]
    fn rejects_an_unknown_section_scope() {
        let markdown = guide("PD2:section:9.99", "[PD2, s. 26].");
        let issues = revision_coverage_issues(&section_inventory(), &markdown);
        assert!(
            issues.iter().any(|issue| issue.contains("was not found")),
            "{issues:?}"
        );
    }

    #[test]
    fn section_scope_extends_to_the_source_end() {
        let markdown = guide("PD2:section:2.20", "[PD2, s. 35–40].");
        assert!(
            revision_coverage_issues(&section_inventory(), &markdown).is_empty(),
            "{:?}",
            revision_coverage_issues(&section_inventory(), &markdown)
        );
    }

    #[test]
    fn accepts_a_fully_cited_scope() {
        let markdown = guide("PD2:26-29", "A [PD2, s. 26–29] and B [PD2, s. 27].");
        assert!(revision_coverage_issues(&inventory(), &markdown).is_empty());
    }

    #[test]
    fn accepts_plain_scope_references() {
        let markdown = guide("PD2:26-29", "Aralık: PD2, s. 26–29.");
        assert!(
            revision_coverage_issues(&inventory(), &markdown).is_empty(),
            "{:?}",
            revision_coverage_issues(&inventory(), &markdown)
        );
    }

    #[test]
    fn accepts_a_single_page_scope() {
        let markdown = guide("PD2:26", "Only [PD2, s. 26].");
        assert!(revision_coverage_issues(&inventory(), &markdown).is_empty());
    }

    #[test]
    fn rejects_a_missing_scope_and_uncited_pages() {
        let markdown = "---\ntitle: Test\n---\n\nA [PD2, s. 26].\n";
        assert!(!revision_coverage_issues(&inventory(), markdown).is_empty());
        let markdown = guide("PD2:26-29", "Only [PD2, s. 26].");
        let issues = revision_coverage_issues(&inventory(), &markdown);
        assert!(
            issues.iter().any(|issue| issue.contains("uncited pages")),
            "{issues:?}"
        );
    }

    #[test]
    fn accepts_unicode_source_ids() {
        let mut inventory = inventory();
        inventory.sources[0].filename = "Ders_Notu_1_VİZE.pdf".to_string();
        let markdown = guide("Ders_Notu_1_VİZE:1-2", "A [Ders_Notu_1_VİZE, s. 1-2].");
        assert!(
            revision_coverage_issues(&inventory, &markdown).is_empty(),
            "{:?}",
            revision_coverage_issues(&inventory, &markdown)
        );
    }

    #[test]
    fn rejects_out_of_scope_and_unknown_sources() {
        let markdown = guide("PD2:26-29", "[PD2, s. 31] [PD0, s. 1]");
        let issues = revision_coverage_issues(&inventory(), &markdown);
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("outside source_scope"))
        );
        let markdown = guide("PD0:1-2", "[PD0, s. 1–2]");
        let issues = revision_coverage_issues(&inventory(), &markdown);
        assert!(issues.iter().any(|issue| issue.contains("unknown source")));
    }
}
