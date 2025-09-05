use cardea_github_mcp_common::{GetStarCountRequest, GetStarCountResponse};
use octocrab::OctocrabBuilder;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use tracing::error;

#[derive(Debug, Clone)]
pub struct GithubServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GithubServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get the star count of a Github repository")]
    async fn get_star_count(
        &self,
        Parameters(GetStarCountRequest { owner, repo }): Parameters<GetStarCountRequest>,
    ) -> Result<CallToolResult, McpError> {
        let octocrab = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            OctocrabBuilder::new()
                .personal_token(token)
                .build()
                .map_err(|e| {
                    let error_message = format!("Failed to build Octocrab: {e}");
                    error!("{}", error_message);
                    McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
                })?
        } else {
            OctocrabBuilder::new().build().map_err(|e| {
                let error_message = format!("Failed to build Octocrab: {e}");
                error!("{}", error_message);
                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?
        };

        let repo = octocrab.repos(owner, repo).get().await.map_err(|e| {
            let error_message = format!("Failed to get repository: {e}");
            error!("{}", error_message);
            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        let response = GetStarCountResponse {
            count: repo.stargazers_count.unwrap_or(0),
        };

        let content = Content::json(response)?;

        Ok(CallToolResult::success(vec![content]))
    }
}

#[tool_handler]
impl ServerHandler for GithubServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            instructions: Some("Gaia Github MCP server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: std::env!("CARGO_PKG_NAME").to_string(),
                version: std::env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}
