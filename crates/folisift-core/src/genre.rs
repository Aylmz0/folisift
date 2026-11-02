//! Deterministic genre grammar checks that do not need an LLM.
//!
//! These checks encode the document rules that make a revision sheet different
//! from a long-form reader. They are deliberately conservative: they report a
//! small set of high-signal violations instead of trying to judge prose.

/// Check a revision-sheet Markdown body against the revision grammar.
///
/// The rules are house standards derived from the evidence note, not universal
/// laws. Errors are actionable and mention the first offending line.
pub fn revision_issues(markdown: &str) -> Vec<String> {
    let body = strip_frontmatter(markdown);
    let mut issues = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut paragraph_start = 0usize;
    let mut fenced = false;
    let mut table_rows = 0usize;
    let mut table_columns = 0usize;
    let mut table_start = 0usize;

    let flush_paragraph = |paragraph: &mut Vec<String>, start: usize, issues: &mut Vec<String>| {
        if paragraph.is_empty() {
            return;
        }
        let words = paragraph
            .iter()
            .map(|line| line.split_whitespace().count())
            .sum::<usize>();
        if words > 60 {
            let preview = paragraph.join(" ");
            let preview = preview.chars().take(48).collect::<String>();
            issues.push(format!(
                "paragraph at line {start} has {words} words (max 60): {preview}…"
            ));
        }
        paragraph.clear();
    };

    let flush_table = |rows: usize, columns: usize, start: usize, issues: &mut Vec<String>| {
        if columns > 5 {
            issues.push(format!(
                "table at line {start} has {columns} columns (max 5)"
            ));
        }
        let data_rows = rows.saturating_sub(2);
        if data_rows > 8 {
            issues.push(format!(
                "table at line {start} has {data_rows} data rows (max 8)"
            ));
        }
    };

    for (index, raw) in body.lines().enumerate() {
        let line = raw.trim();
        let line_number = index + 1;
        if line.starts_with("```") || line.starts_with("~~~") {
            fenced = !fenced;
            flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
            if table_rows > 0 {
                flush_table(table_rows, table_columns, table_start, &mut issues);
                table_rows = 0;
                table_columns = 0;
            }
            continue;
        }
        if fenced {
            continue;
        }
        if line.is_empty() {
            flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
            if table_rows > 0 {
                flush_table(table_rows, table_columns, table_start, &mut issues);
                table_rows = 0;
                table_columns = 0;
            }
            continue;
        }
        if line.starts_with('#')
            || line.starts_with(":::")
            || line.starts_with("<!--")
            || line.starts_with("![")
            || line.starts_with('>')
            || line.starts_with('|')
        {
            flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
            if line.starts_with('|') {
                if table_rows == 0 {
                    table_start = line_number;
                }
                table_rows += 1;
                table_columns = table_columns.max(line.trim_matches('|').split('|').count());
            } else if table_rows > 0 {
                flush_table(table_rows, table_columns, table_start, &mut issues);
                table_rows = 0;
                table_columns = 0;
            }
            continue;
        }
        if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
            let words = item.split_whitespace().count();
            if words > 25 {
                issues.push(format!(
                    "bullet at line {line_number} has {words} words (max 25)"
                ));
            }
            continue;
        }
        if line
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
            && (line.contains(". ") || line.contains(") "))
        {
            flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
            continue;
        }
        if paragraph.is_empty() {
            paragraph_start = line_number;
        }
        paragraph.push(line.to_string());
    }
    flush_paragraph(&mut paragraph, paragraph_start, &mut issues);
    if table_rows > 0 {
        flush_table(table_rows, table_columns, table_start, &mut issues);
    }

    for (needle, label) in [
        ("::: must-know", "must-know card block"),
        ("::: diagram", "concept map or diagram"),
        ("::: recall", "recall prompt"),
        ("::: answer", "answer key"),
        ("::: synthesis", "one-page synthesis"),
    ] {
        if !body.contains(needle) {
            issues.push(format!("revision sheet is missing a {label} ({needle})"));
        }
    }

    let recalls = body.matches("::: recall").count();
    let answers = body.matches("::: answer").count();
    if answers < recalls {
        issues.push(format!(
            "revision sheet has {recalls} recall prompts but only {answers} answer blocks"
        ));
    }

    issues
}

fn strip_frontmatter(markdown: &str) -> &str {
    let markdown = markdown.strip_prefix("---\n").unwrap_or(markdown);
    if let Some((_, body)) = markdown.split_once("\n---\n") {
        body
    } else {
        markdown
    }
}

/// Guided notes: cues, a summary and no pre-filled answer key.
pub fn notes_issues(markdown: &str) -> Vec<String> {
    let body = strip_frontmatter(markdown);
    let mut issues = Vec::new();
    if !body.contains("::: cue") {
        issues.push("guided notes need at least one cue block (::: cue)".to_string());
    }
    if !body.contains("::: summary") {
        issues.push("guided notes need a summary block (::: summary)".to_string());
    }
    if body.contains("::: answer-key") {
        issues.push("guided notes must not contain a pre-filled answer key".to_string());
    }
    for (index, line) in body.lines().enumerate() {
        if line.trim().starts_with("- ") && line.split_whitespace().count() > 22 {
            issues.push(format!(
                "guided-note bullet at line {} is longer than 22 words",
                index + 1
            ));
        }
    }
    issues
}

/// Workbook: problems, workspace, a worked example and an answer key.
pub fn workbook_issues(markdown: &str) -> Vec<String> {
    let body = strip_frontmatter(markdown);
    let mut issues = Vec::new();
    for (needle, label, minimum) in [
        ("::: problem", "problem block", 2usize),
        ("::: workspace", "workspace block", 1usize),
        ("::: example", "worked/faded example", 1usize),
        ("::: answer-key", "answer key", 1usize),
    ] {
        if body.matches(needle).count() < minimum {
            issues.push(format!(
                "workbook needs at least {minimum} {label} ({needle})"
            ));
        }
    }
    let answer = body.find("::: answer-key");
    let practice = body
        .find("## Practice")
        .or_else(|| body.find("## Alıştırma"));
    if let (Some(answer), Some(practice)) = (answer, practice)
        && answer < practice
    {
        issues.push("workbook answer key must come after the practice section".to_string());
    }
    issues
}

/// Reference sheet: formula cards, at least one table and a pitfall block.
pub fn reference_issues(markdown: &str) -> Vec<String> {
    let body = strip_frontmatter(markdown);
    let mut issues = Vec::new();
    if !body.contains("::: formula") {
        issues.push("reference sheet needs at least one formula card (::: formula)".to_string());
    }
    if !body.contains("| ") {
        issues.push("reference sheet needs at least one symbol/unit table".to_string());
    }
    if !body.contains("::: critical") {
        issues.push("reference sheet needs a pitfalls block (::: critical)".to_string());
    }
    for (index, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with(":::")
            || trimmed.starts_with('|')
            || trimmed.starts_with('-')
            || trimmed.starts_with("```")
            || trimmed.starts_with("~~~")
        {
            continue;
        }
        if trimmed.split_whitespace().count() > 40 {
            issues.push(format!(
                "reference prose at line {} is longer than 40 words",
                index + 1
            ));
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"## Scope and map

A short paragraph with a source reference. [PD0, p. 1]

::: must-know
- **Term:** A short definition. [PD0, p. 1]
:::

::: diagram id="map"
:::

## Recall

::: recall
What is X?
:::

::: answer
X is Y. [PD0, p. 1]
:::

## Synthesis

::: synthesis
- One
- Two
:::
"#;

    #[test]
    fn accepts_a_well_formed_revision_sheet() {
        assert!(
            revision_issues(GOOD).is_empty(),
            "{:?}",
            revision_issues(GOOD)
        );
    }

    #[test]
    fn rejects_long_paragraphs_and_bullets() {
        let long = format!(
            "## Scope\n\n{}\n\n::: must-know\n- **Term:** {}\n:::\n\n::: diagram id=\"x\"\n:::\n\n::: recall\nQ\n:::\n\n::: answer\nA\n:::\n\n::: synthesis\n- One\n:::\n",
            "word ".repeat(61),
            "word ".repeat(26)
        );
        let issues = revision_issues(&long);
        assert!(
            issues.iter().any(|issue| issue.contains("max 60")),
            "{issues:?}"
        );
        assert!(
            issues.iter().any(|issue| issue.contains("max 25")),
            "{issues:?}"
        );
    }

    #[test]
    fn demo_revision_sheet_passes_the_grammar() {
        let markdown = include_str!("../../../guides/pd2-rev/guide.md");
        let issues = revision_issues(markdown);
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn accepts_eight_data_rows() {
        let mut table = String::from("| A | B |\n|---|---|\n");
        for index in 0..8 {
            table.push_str(&format!("| {index} | value |\n"));
        }
        let markdown = format!(
            "## Scope\n\n::: must-know\n- **Term:** x\n:::\n\n::: diagram id=\"x\"\n:::\n\n::: recall\nQ\n:::\n\n::: answer\nA\n:::\n\n::: synthesis\n- One\n:::\n\n{table}"
        );
        let issues = revision_issues(&markdown);
        assert!(
            !issues.iter().any(|issue| issue.contains("data rows")),
            "{issues:?}"
        );
    }

    #[test]
    fn rejects_incomplete_genre_fixtures() {
        assert!(
            notes_issues("## Goals\nText.")
                .iter()
                .any(|i| i.contains("cue"))
        );
        assert!(
            notes_issues("::: cue\nQ\n:::\n\n::: answer-key\nA\n:::")
                .iter()
                .any(|i| i.contains("answer key"))
        );
        assert!(
            workbook_issues("## Problems\nText.")
                .iter()
                .any(|i| i.contains("problem block"))
        );
        assert!(
            workbook_issues("::: answer-key\nA\n:::\n\n## Practice\nText.")
                .iter()
                .any(|i| i.contains("after the practice"))
        );
        assert!(
            reference_issues("## Formulas\nText.")
                .iter()
                .any(|i| i.contains("formula"))
        );
    }

    #[test]
    fn previews_pass_their_genre_grammar() {
        let notes = include_str!("../../../styles/notes/preview.en.md");
        assert!(notes_issues(notes).is_empty(), "{:?}", notes_issues(notes));
        let workbook = include_str!("../../../styles/workbook/preview.en.md");
        assert!(
            workbook_issues(workbook).is_empty(),
            "{:?}",
            workbook_issues(workbook)
        );
        let reference = include_str!("../../../styles/reference/preview.en.md");
        assert!(
            reference_issues(reference).is_empty(),
            "{:?}",
            reference_issues(reference)
        );
    }

    #[test]
    fn rejects_missing_answer_and_components() {
        let issues = revision_issues("## Scope\n\nText.\n\n::: recall\nQ\n:::\n");
        assert!(
            issues.iter().any(|issue| issue.contains("must-know")),
            "{issues:?}"
        );
        assert!(
            issues.iter().any(|issue| issue.contains("answer")),
            "{issues:?}"
        );
    }
}
