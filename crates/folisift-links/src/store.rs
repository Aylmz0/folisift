use std::path::Path;

use crate::link::{Error, LinkReport, Result};

pub const LINKS_SCHEMA_VERSION: u32 = 1;

pub fn write_report(path: &Path, report: &LinkReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, format!("{json}\n"))?;
    Ok(())
}

pub fn read_report(path: &Path) -> Result<LinkReport> {
    let bytes = std::fs::read(path)?;
    let report: LinkReport = serde_json::from_slice(&bytes)?;
    if report.schema_version != LINKS_SCHEMA_VERSION {
        return Err(Error::Invalid(format!(
            "unsupported links schema {}",
            report.schema_version
        )));
    }
    Ok(report)
}

/// Write a compact human-review table beside the JSON report.
pub fn write_markdown_report(path: &Path, report: &LinkReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let sections = report
        .sections
        .iter()
        .map(|section| (section.id.as_str(), section))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut out = String::from("# Related sections\n\n");
    if let Some(model) = &report.model {
        out.push_str(&format!("Model: `{model}`\n\n"));
    }
    out.push_str("| Section | Related section | Score | Pages |\n|---|---|---:|---|\n");
    for link in &report.links {
        let from = sections
            .get(link.from.as_str())
            .and_then(|section| section.heading.as_deref())
            .unwrap_or(&link.from);
        let to = sections
            .get(link.to.as_str())
            .and_then(|section| section.heading.as_deref())
            .unwrap_or(&link.to);
        out.push_str(&format!(
            "| {} | {} | {:.3} | {} s.{} → {} s.{} |\n",
            from.replace('|', "\\|"),
            to.replace('|', "\\|"),
            link.score,
            link.from_source,
            link.from_page,
            link.to_source,
            link.to_page
        ));
    }
    out.push_str("\nSimilarity is a review proposal, not a verified relation.\n");
    std::fs::write(path, out)?;
    Ok(())
}
