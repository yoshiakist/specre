use crate::commands::index::{collect_md_files, parse_frontmatter};
use crate::config;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::*,
    service::RequestContext,
    tool_handler, tool_router,
    transport::stdio,
};
use std::fs;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

const URI_PREFIX: &str = "specre:///";

#[derive(Clone)]
pub struct SpecreMcpServer {
    specre_dir: PathBuf,
    tool_router: ToolRouter<SpecreMcpServer>,
}

#[tool_router]
impl SpecreMcpServer {
    pub fn new(specre_dir: PathBuf) -> Self {
        Self {
            specre_dir,
            tool_router: Self::tool_router(),
        }
    }
}

/// Scan specre_dir and return (ULID, name, status, file_path) for each card.
fn scan_cards(specre_dir: &Path) -> Vec<(String, String, String, PathBuf)> {
    let mut cards = Vec::new();
    collect_md_files(specre_dir, &mut |path| {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Some(fm) = parse_frontmatter(&content) {
            cards.push((fm.id, fm.name, fm.status, path.to_path_buf()));
        }
    });
    cards.sort_by(|a, b| a.0.cmp(&b.0));
    cards
}

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
        let cards = scan_cards(&self.specre_dir);
        let resources = cards
            .into_iter()
            .map(|(id, name, status, _path)| {
                RawResource {
                    uri: format!("{URI_PREFIX}{id}"),
                    name: name.clone(),
                    title: None,
                    description: Some(format!("[{status}] {name}")),
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
            .ok_or_else(|| {
                McpError::invalid_params("URI must start with specre:///", None)
            })?;

        // Find the card whose frontmatter id matches the requested ULID
        let cards = scan_cards(&self.specre_dir);
        let (_id, _name, _status, path) = cards
            .into_iter()
            .find(|(id, _, _, _)| id == ulid)
            .ok_or_else(|| {
                McpError::resource_not_found(
                    "specre card not found",
                    Some(serde_json::json!({ "ulid": ulid })),
                )
            })?;

        let content = fs::read_to_string(&path).map_err(|e| {
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

pub fn execute() -> Result<(), String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(run_server())
}

async fn run_server() -> Result<(), String> {
    // stdout is reserved for JSON-RPC; all logs go to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting specre MCP server v{}", env!("CARGO_PKG_VERSION"));

    let config = config::load().map_err(|e| format!("Config error: {e}"))?;
    let specre_dir = PathBuf::from(&config.specre_dir);

    let service = SpecreMcpServer::new(specre_dir)
        .serve(stdio())
        .await
        .map_err(|e| format!("MCP server error: {e}"))?;

    service
        .waiting()
        .await
        .map_err(|e| format!("MCP server error: {e}"))?;

    Ok(())
}
