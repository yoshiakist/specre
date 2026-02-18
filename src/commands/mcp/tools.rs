// @specre 01KHK7MFZJZ12XFPQE4RHCBHQN
// @specre 01KHQJG96BS5STGSENPNDHEH1H
// @specre 01KHQKZ5M6N304YYJNW8VDKT4W
// @specre 01KHQKZ5VKTHSD483ZWK0RYPR9
// @specre 01KHQKZ633JHVDK0WADPPVP3CM
// @specre 01KHQKZ6AAMY6Y6AQB3VDVSF6Z
// @specre 01KHQKZ6H8FB46ESFXB03N85AN
// @specre 01KHQKZ6RE7Z3WEDZ54ZKHM6BM
// @specre 01KHQKZ6ZHSZX3GR2D7DS23XTE

use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use std::path::PathBuf;

use super::helpers;
use super::search;

// ---------------------------------------------------------------------------
// Server struct
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SpecreMcpServer {
    pub(crate) specre_dir: PathBuf,
    pub(crate) tool_router: ToolRouter<Self>,
}

// ---------------------------------------------------------------------------
// Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NewToolRequest {
    /// Directory where the specre file will be created (e.g., "docs/specres/auth")
    pub target_dir: String,
    /// Specre name describing the behavior (default: "untitled")
    pub name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TagToolRequest {
    /// ULID to insert as a @specre marker (26 uppercase alphanumeric characters)
    pub ulid: String,
    /// Path to the source file where the marker will be inserted
    pub file: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatusToolRequest {
    /// Number of days after which a stable specre's `last_verified` is stale (default: 30)
    pub threshold: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TraceToolRequest {
    /// ULID (26 uppercase alphanumeric characters) or file path to look up
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchToolRequest {
    /// Free-text substring to match against card content (case-insensitive, multi-keyword AND by default)
    pub query: Option<String>,
    /// Filter by status: draft, in-development, stable, deprecated
    pub status: Option<String>,
    /// Filter by domain (top-level directory under `specre_dir`)
    pub domain: Option<String>,
    /// Include only specres verified before this date (YYYY-MM-DD)
    pub verified_before: Option<String>,
    /// Include only specres verified on or after this date (YYYY-MM-DD)
    pub verified_after: Option<String>,
    /// Use OR logic for multi-keyword queries (default: false)
    pub or: Option<bool>,
    /// Return at most N results
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Tool router — thin wrappers delegating to helpers / search modules
// ---------------------------------------------------------------------------

#[tool_router]
impl SpecreMcpServer {
    pub fn new(specre_dir: PathBuf) -> Self {
        Self {
            specre_dir,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "new", description = "Create a new specre card from a template, auto-generating a ULID")]
    #[allow(clippy::unused_self)]
    fn new_card(
        &self,
        Parameters(req): Parameters<NewToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        helpers::execute_new(&req)
    }

    #[tool(name = "tag", description = "Insert a @specre marker into a source file at line 1")]
    #[allow(clippy::unused_self)]
    fn tag_file(
        &self,
        Parameters(req): Parameters<TagToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        helpers::execute_tag(&req)
    }

    #[tool(
        name = "index",
        description = "Regenerate index.json and per-domain _INDEX.md files"
    )]
    #[allow(clippy::unused_self)]
    fn run_index(&self) -> Result<CallToolResult, McpError> {
        helpers::execute_index()
    }

    #[tool(
        name = "status",
        description = "Report specre counts by status and flag stale last_verified dates"
    )]
    #[allow(clippy::unused_self)]
    fn run_status(
        &self,
        Parameters(req): Parameters<StatusToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        helpers::execute_status(&req)
    }

    #[tool(
        name = "trace",
        description = "Bidirectional traceability lookup by ULID or file path"
    )]
    #[allow(clippy::unused_self)]
    fn run_trace(
        &self,
        Parameters(req): Parameters<TraceToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        helpers::execute_trace(&req)
    }

    #[tool(
        name = "orphans",
        description = "Detect unlinked specre cards and dangling @specre markers"
    )]
    #[allow(clippy::unused_self)]
    fn run_orphans(&self) -> Result<CallToolResult, McpError> {
        helpers::execute_orphans()
    }

    #[tool(
        name = "search",
        description = "Search specre cards by text query and structured filters"
    )]
    #[allow(clippy::unused_self)]
    fn run_search(
        &self,
        Parameters(req): Parameters<SearchToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        search::execute_search(&req)
    }

    #[tool(
        name = "coverage",
        description = "Report the percentage of source files covered by @specre tags"
    )]
    #[allow(clippy::unused_self)]
    fn run_coverage(&self) -> Result<CallToolResult, McpError> {
        helpers::execute_coverage()
    }

    #[tool(
        name = "health-check",
        description = "Comprehensive health check for AI agent preflight"
    )]
    #[allow(clippy::unused_self)]
    fn run_health_check(&self) -> Result<CallToolResult, McpError> {
        helpers::execute_health_check()
    }
}
