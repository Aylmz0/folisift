//! Core domain model, configuration validation and workspace path safety.
//!
//! This crate deliberately has no dependency on the CLI, MCP, PDF parser or
//! renderer. Those layers call into these types.

pub mod cards;
pub mod claims;
pub mod config;
pub mod diagnostic;
pub mod error;
pub mod gates;
pub mod genre;
pub mod intent;
pub mod paths;
pub mod scaffold;
pub mod structures;
pub mod version;

pub use config::{
    Guide, GuideConfig, GuideSummary, SUPPORTED_SCHEMA_VERSIONS, StyleProfile, list_guides,
};
pub use diagnostic::{Diagnostic, ErrorCode};
pub use error::{Error, Result};
pub use paths::{Workspace, validate_slug};
pub use scaffold::{ScaffoldOptions, scaffold};
pub use version::{BUILD, MCP_TOOL_SCHEMA_VERSION, VERSION, VersionInfo};
