//! Shared build-gate policy for CLI and MCP.
//!
//! Keeping this in `folisift-core` prevents the two host interfaces from
//! drifting: the same genre, page-count and top-level pass rules apply.

use crate::genre;

/// A–G coverage is a course-guide gate, not a revision/organiser gate.
pub fn uses_course_coverage(structure: &str) -> bool {
    matches!(
        structure,
        "a-g" | "imrad" | "irac" | "soap" | "literature-matrix" | "executive-brief"
    )
}

/// Revision/organiser coverage is a declared source page scope, not A–G.
pub fn uses_revision_coverage(structure: &str) -> bool {
    matches!(
        structure,
        "revision-sheet"
            | "knowledge-organiser"
            | "lecture-notes"
            | "workbook"
            | "formula-sheet"
            | "concept-summary"
    )
}

/// Genre-specific Markdown checks that do not depend on PDF inspection.
pub fn genre_issues(structure: &str, markdown: &str) -> Vec<String> {
    match structure {
        "revision-sheet" => genre::revision_issues(markdown),
        "lecture-notes" => genre::notes_issues(markdown),
        "workbook" => genre::workbook_issues(markdown),
        "formula-sheet" => genre::reference_issues(markdown),
        "concept-summary" => genre::concept_summary_issues(markdown),
        _ => Vec::new(),
    }
}

/// Validate the rendered page count against the guide's declared budget.
pub fn page_range_issues(target_pages: Option<&str>, pages: usize) -> Vec<String> {
    let Some(target) = target_pages
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Vec::new();
    };
    let (min, max) = if let Some((start, end)) = target.split_once('-') {
        let (Ok(start), Ok(end)) = (start.trim().parse::<usize>(), end.trim().parse::<usize>())
        else {
            return vec![format!("invalid page target {target:?}")];
        };
        (start, end)
    } else {
        let Ok(page) = target.parse::<usize>() else {
            return vec![format!("invalid page target {target:?}")];
        };
        (page, page)
    };
    if pages < min || pages > max {
        vec![format!(
            "page count is {pages}, expected {min}-{max} (target {target})"
        )]
    } else {
        Vec::new()
    }
}

/// Page-count rules that only apply after rendering.
pub fn page_count_issues(structure: &str, pages: usize) -> Vec<String> {
    if structure == "knowledge-organiser" && pages != 1 {
        vec![format!(
            "knowledge organiser must be exactly 1 page (got {pages}); split the unit or cut content"
        )]
    } else {
        Vec::new()
    }
}

/// One pass/fail rule for every host and every genre.
pub fn top_level_passed(
    layout_passed: bool,
    genre_ok: bool,
    coverage_passed: Option<bool>,
    visual_passed: Option<bool>,
) -> bool {
    layout_passed && genre_ok && coverage_passed.unwrap_or(true) && visual_passed.unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_and_organiser_skip_course_coverage() {
        assert!(!uses_course_coverage("revision-sheet"));
        assert!(!uses_course_coverage("knowledge-organiser"));
        assert!(uses_course_coverage("a-g"));
        assert!(uses_course_coverage("imrad"));
        assert!(!uses_course_coverage("lecture-notes"));
        assert!(!uses_course_coverage("workbook"));
        assert!(!uses_course_coverage("formula-sheet"));
    }

    #[test]
    fn revision_and_organiser_use_source_scope_coverage() {
        assert!(uses_revision_coverage("revision-sheet"));
        assert!(uses_revision_coverage("knowledge-organiser"));
        assert!(!uses_revision_coverage("a-g"));
    }

    #[test]
    fn page_range_matches_exact_and_ranges() {
        assert!(page_range_issues(Some("5"), 5).is_empty());
        assert!(page_range_issues(Some("4-6"), 4).is_empty());
        assert!(page_range_issues(Some("4-6"), 6).is_empty());
        assert!(!page_range_issues(Some("4-6"), 8).is_empty());
        assert!(page_range_issues(None, 8).is_empty());
    }

    #[test]
    fn organiser_requires_one_page() {
        assert!(
            page_count_issues("knowledge-organiser", 2)
                .iter()
                .any(|issue| issue.contains("exactly 1 page"))
        );
        assert!(page_count_issues("knowledge-organiser", 1).is_empty());
        assert!(page_count_issues("revision-sheet", 4).is_empty());
    }

    #[test]
    fn top_level_requires_every_gate() {
        assert!(top_level_passed(true, true, Some(true), Some(true)));
        assert!(!top_level_passed(true, true, Some(false), Some(true)));
        assert!(!top_level_passed(true, true, Some(true), Some(false)));
        assert!(!top_level_passed(false, true, Some(true), Some(true)));
        assert!(!top_level_passed(true, false, Some(true), Some(true)));
    }
}
