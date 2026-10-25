use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use folisift_core::{Guide, Workspace, list_guides, version};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "folisift",
    version = version::VERSION,
    about = "Deterministic study-guide pipeline"
)]
struct Cli {
    /// Workspace root. Defaults to the current directory.
    #[arg(long, global = true, env = "FOLISIFT_ROOT", default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show workspace root, version and guide count.
    WorkspaceInfo,
    /// List guides and whether each config is valid.
    ListGuides,
    /// Show a validated guide config and resolved paths.
    GuideStatus { slug: String },
}

#[derive(Debug, Serialize)]
struct GuideStatus {
    slug: String,
    title: String,
    schema_version: u32,
    language: String,
    theme: String,
    content: String,
    output: String,
    sources: Vec<String>,
    inventory: Option<String>,
    coverage: Option<String>,
    target_pages: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let workspace = Workspace::open(&cli.root)?;

    match cli.command {
        Command::WorkspaceInfo => {
            let guides = list_guides(&workspace)?;
            let value = json!({
                "root": workspace.root().display().to_string(),
                "version": version::VERSION,
                "build": version::BUILD,
                "mcp_tool_schema_version": version::MCP_TOOL_SCHEMA_VERSION,
                "guides": guides.len(),
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Command::ListGuides => {
            let guides = list_guides(&workspace)?;
            println!("{}", serde_json::to_string_pretty(&guides)?);
        }
        Command::GuideStatus { slug } => {
            let guide = Guide::load(&workspace, &slug)?;
            let status = GuideStatus {
                slug: guide.slug,
                title: guide.config.title,
                schema_version: guide.config.schema_version,
                language: guide.config.language,
                theme: guide.config.theme,
                content: guide.content.display().to_string(),
                output: guide.output.display().to_string(),
                sources: guide
                    .sources
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
                inventory: guide.inventory.map(|path| path.display().to_string()),
                coverage: guide.coverage.map(|path| path.display().to_string()),
                target_pages: guide.config.target_pages,
            };
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
    }
    Ok(())
}
