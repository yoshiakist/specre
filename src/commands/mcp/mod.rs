// @specre 01KHJ98T83DPJGMEFH9HAXXAZ1
// @specre 01KHJ98TFCDTCARMMX1GC5ZHXE

mod helpers;
mod search;
pub mod tools;

use crate::card::{self, to_forward_slash};
use crate::config;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    model::{
        AnnotateAble, Implementation, ListResourcesResult, PaginatedRequestParams, ProtocolVersion,
        RawResource, ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler,
    transport::stdio,
};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

pub use tools::SpecreMcpServer;

const URI_PREFIX: &str = "specre:///";

#[tool_handler]
impl ServerHandler for SpecreMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "specre".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "specre MCP server — provides specre cards as resources and specre CLI operations as tools."
                    .to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let specre_dir_str = to_forward_slash(&self.specre_dir);
        let cards = card::scan_specre_cards(&self.specre_dir, &specre_dir_str);
        let resources = cards
            .into_iter()
            .map(|c| {
                let description = Some(format!("[{}] {}", c.status, c.name));
                RawResource {
                    uri: format!("{URI_PREFIX}{}", c.id),
                    name: c.name,
                    title: None,
                    description,
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                }
                .no_annotation()
            })
            .collect();

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { meta: _, uri }: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let ulid = uri
            .strip_prefix(URI_PREFIX)
            .ok_or_else(|| McpError::invalid_params("URI must start with specre:///", None))?;

        // Find the card whose frontmatter id matches the requested ULID
        let specre_dir_str = to_forward_slash(&self.specre_dir);
        let cards = card::scan_specre_cards(&self.specre_dir, &specre_dir_str);
        let found = cards.into_iter().find(|c| c.id == ulid).ok_or_else(|| {
            McpError::resource_not_found(
                "specre card not found",
                Some(serde_json::json!({ "ulid": ulid })),
            )
        })?;

        let content = fs::read_to_string(&found.path).map_err(|e| {
            McpError::internal_error(
                "failed to read specre card",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(content, uri)],
        })
    }
}

/// # Errors
///
/// Returns [`SpecreError`] on runtime creation, MCP init, or server failure.
pub fn execute() -> Result<(), crate::error::SpecreError> {
    let rt = tokio::runtime::Runtime::new().map_err(crate::error::SpecreError::TokioRuntime)?;
    rt.block_on(run_server())
}

async fn run_server() -> Result<(), crate::error::SpecreError> {
    // stdout is reserved for JSON-RPC; all logs go to stderr
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting specre MCP server v{}", env!("CARGO_PKG_VERSION"));

    let config = config::load()?;
    let specre_dir = PathBuf::from(&config.specre_dir);

    let service = SpecreMcpServer::new(specre_dir)
        .serve(stdio())
        .await
        .map_err(|e| crate::error::SpecreError::McpInit(e.to_string()))?;

    service
        .waiting()
        .await
        .map_err(|e| crate::error::SpecreError::McpTask(e.to_string()))?;

    Ok(())
}
