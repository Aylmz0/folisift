use serde::Serialize;

/// Semantic version of the binary, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build number embedded at compile time, usually `git rev-list --count HEAD`.
pub const BUILD: &str = env!("FOLISIFT_BUILD");

/// Version of the MCP tool contract. Bumped independently of the binary version.
pub const MCP_TOOL_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub version: &'static str,
    pub build: &'static str,
    pub mcp_tool_schema_version: u32,
}

pub const fn info() -> VersionInfo {
    VersionInfo {
        version: VERSION,
        build: BUILD,
        mcp_tool_schema_version: MCP_TOOL_SCHEMA_VERSION,
    }
}
