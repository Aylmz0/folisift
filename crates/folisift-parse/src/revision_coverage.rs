//! Source-scope coverage for revision sheets and knowledge organisers.
//!
//! These genres are selective by design, so they do not use the A–G heading
//! gate. They declare a page scope in frontmatter and must cite every page in
//! that scope at least once, using `[PD2, s. 26–29]` style references.

use serde::Deserialize;

use crate::inventory::Inventory;

static PAGE_REF_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b([A-Z]{2}\d*)\s*,\s*s\.\s*(\d+)(?:\s*[–-]\s*(\d+))?")
        .expect("valid page reference regex")
});

#[derive(Debug, Deserialize)]
struct ScopeFrontmatter {
    #[serde(default)]
    source_scope: Option<String>,
}

#[derive(Debug, Clone)]
struct Scope {
    source: String,
    start: usize,
    end: usize,
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
        ));
    }
    for scope in &scopes {
        let Some((_, pages)) = known.iter().find(|(id, _)| id == &scope.source) else {
            issues.push(format!(
                "source_scope references unknown source {:?}",
                scope.source
            ));
            continue;
        };
        if scope.start == 0 || scope.end < scope.start || scope.end > *pages {
            issues.push(format!(
                "source_scope {}:{}-{} is outside the source page range 1-{pages}",
                scope.source, scope.start, scope.end
            ));
            continue;
        }
        let mut cited = vec![false; scope.end - scope.start + 1];
        let mut saw_reference = false;
        for capture in PAGE_REF_RE.captures_iter(markdown) {
            let source = capture[1].to_string();
            if source != scope.source {
                continue;
            }
            saw_reference = true;
            let start = capture[2].parse::<usize>().unwrap_or(0);
            let end = capture
                .get(3)
                .and_then(|value| value.as_str().parse::<usize>().ok())
                .unwrap_or(start);
            if start < scope.start || end > scope.end || end < start {
                issues.push(format!(
                    "reference {}:{}-{} falls outside source_scope {}:{}-{}",
                    source, start, end, scope.source, scope.start, scope.end
                ));
                continue;
            }
            for page in start..=end {
                cited[page - scope.start] = true;
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
            .filter_map(|(index, seen)| (!seen).then_some(scope.start + index))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            issues.push(format!(
                "source_scope {}:{}-{} has uncited pages: {missing:?}",
                scope.source, scope.start, scope.end
            ));
        }
    }
    issues
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
        let Some((source, range)) = part.rsplit_once(':') else {
            return Err(format!(
                "invalid source_scope entry {part:?}; use SOURCE:START-END"
            ));
        };
        let Some((start, end)) = range.split_once('-') else {
            return Err(format!(
                "invalid source_scope range {range:?}; use START-END"
            ));
        };
        let start = start
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("invalid source_scope start in {part:?}"))?;
        let end = end
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("invalid source_scope end in {part:?}"))?;
        scopes.push(Scope {
            source: source.trim().to_string(),
            start,
            end,
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
    use crate::inventory::{SourceInventory, Totals};

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

    #[test]
    fn accepts_a_fully_cited_scope() {
        let markdown = guide("PD2:26-29", "A [PD2, s. 26–29] and B [PD2, s. 27].");
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
