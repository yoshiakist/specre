// @specre 01KHK7MFZJZ12XFPQE4RHCBHQN
// @specre 01KHQJG96BS5STGSENPNDHEH1H

use crate::card::to_forward_slash;
use crate::commands::tag::comment_syntax;
use crate::{config, template, ulid};
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SpecreMcpServer {
    pub(crate) specre_dir: PathBuf,
    pub(crate) tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct NewToolRequest {
    /// Directory where the specre file will be created (e.g., "docs/specres/auth")
    target_dir: String,
    /// Specre name describing the behavior (default: "untitled")
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TagToolRequest {
    /// ULID to insert as a @specre marker (26 uppercase alphanumeric characters)
    ulid: String,
    /// Path to the source file where the marker will be inserted
    file: String,
}

#[tool_router]
impl SpecreMcpServer {
    pub fn new(specre_dir: PathBuf) -> Self {
        Self {
            specre_dir,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "new", description = "Create a new specre card from a template, auto-generating a ULID")]
    #[allow(clippy::unused_self)] // &self required by rmcp #[tool] macro
    fn new_card(
        &self,
        Parameters(req): Parameters<NewToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        let name = req.name.unwrap_or_else(|| "untitled".to_string());
        let target = Path::new(&req.target_dir);

        if target.is_file() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("'{}' is a file, not a directory", target.display()),
            )]));
        }

        let file_name = format!("{name}.md");
        let file_path = target.join(&file_name);

        if file_path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("'{}' already exists", file_path.display()),
            )]));
        }

        if !target.exists() {
            fs::create_dir_all(target).map_err(|e| {
                McpError::internal_error(
                    "failed to create directory",
                    Some(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
        }

        let language = config::load_language();
        let id = ulid::generate();
        let content = template::render(&id, &name, &language);

        fs::write(&file_path, &content).map_err(|e| {
            McpError::internal_error(
                "failed to write specre card",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        let result = serde_json::json!({
            "id": id,
            "path": to_forward_slash(&file_path).as_ref(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(name = "tag", description = "Insert a @specre marker into a source file at line 1")]
    #[allow(clippy::unused_self)] // &self required by rmcp #[tool] macro
    fn tag_file(
        &self,
        Parameters(req): Parameters<TagToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        if !ulid::is_valid(&req.ulid) {
            return Ok(CallToolResult::error(vec![Content::text(
                "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
            )]));
        }

        let file_path = Path::new(&req.file);

        if !file_path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("file not found: {}", to_forward_slash(file_path)),
            )]));
        }

        if file_path.is_dir() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!(
                    "'{}' is a directory, not a file",
                    to_forward_slash(file_path)
                ),
            )]));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            McpError::internal_error(
                "failed to read file",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        // Check if marker already exists
        let marker_pattern = format!("@specre {}", req.ulid);
        if content.contains(&marker_pattern) {
            let line = content
                .lines()
                .position(|l| l.contains(&marker_pattern))
                .map_or(1, |n| n + 1);

            let result = serde_json::json!({
                "id": req.ulid,
                "file": to_forward_slash(file_path).as_ref(),
                "line": line,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }

        // Determine comment syntax from file extension
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let Some((prefix, suffix)) = comment_syntax(ext) else {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("unsupported file extension '.{ext}' — comment syntax is unknown"),
            )]));
        };

        let marker_line = format!("{prefix}@specre {}{suffix}\n", req.ulid);
        let new_content = format!("{marker_line}{content}");

        fs::write(file_path, &new_content).map_err(|e| {
            McpError::internal_error(
                "failed to write file",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        let result = serde_json::json!({
            "id": req.ulid,
            "file": to_forward_slash(file_path).as_ref(),
            "line": 1,
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }
}
