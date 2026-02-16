use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::*,
    service::RequestContext,
    tool_handler, tool_router,
    transport::stdio,
};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct SpecreMcpServer {
    tool_router: ToolRouter<SpecreMcpServer>,
}

#[tool_router]
impl SpecreMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
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
            server_info: Implementation::from_build_env(),
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
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { meta: _, uri }: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::resource_not_found(
            "resource_not_found",
            Some(serde_json::json!({ "uri": uri })),
        ))
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

    let service = SpecreMcpServer::new()
        .serve(stdio())
        .await
        .map_err(|e| format!("MCP server error: {e}"))?;

    service
        .waiting()
        .await
        .map_err(|e| format!("MCP server error: {e}"))?;

    Ok(())
}
