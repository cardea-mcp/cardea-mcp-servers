use crate::CONNECTION_CONFIG;
use cardea_qdrant_mcp_common::*;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use tracing::error;

#[derive(Debug, Clone)]
pub struct QdrantServer {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl QdrantServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Perform vector search in the Qdrant database")]
    async fn search(
        &self,
        Parameters(SearchPointsRequest { vector }): Parameters<SearchPointsRequest>,
    ) -> Result<CallToolResult, McpError> {
        // get connection config
        let conn_config = match CONNECTION_CONFIG.get() {
            Some(connection_config) => {
                let conn_config = connection_config.read().await;
                conn_config.clone()
            }
            None => {
                let error_message = "Connection config not found";
                error!("{}", error_message);
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        // build url
        let base_url = conn_config.base_url.trim_end_matches('/');
        let url = format!(
            "{}/collections/{}/points/search",
            base_url, conn_config.collection
        );

        // get api key
        let api_key = conn_config.api_key;

        // build params
        let params = json!({
            "vector": vector,
            "limit": conn_config.limit,
            "with_payload": true,
            "with_vector": true,
            "score_threshold": conn_config.score_threshold,
        });

        let client = reqwest::Client::new();
        let result = match api_key {
            Some(api_key) => {
                client
                    .post(&url)
                    .header("api-key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&params)
                    .send()
                    .await
            }
            None => {
                client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&params)
                    .send()
                    .await
            }
        };

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                let error_message = format!("Failed to search points: {e}");

                error!("{}", error_message);

                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        match response.status().is_success() {
            true => match response.json::<Value>().await {
                Ok(json) => match json.get("result") {
                    Some(result) => {
                        let scored_points = result
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| ScoredPoint {
                                score: v.get("score").unwrap().as_f64().unwrap(),
                                payload: v
                                    .get("payload")
                                    .unwrap()
                                    .as_object()
                                    .unwrap()
                                    .to_owned()
                                    .into_iter()
                                    .map(|(k, v)| (k.to_string(), v.clone()))
                                    .collect(),
                                vector: v
                                    .get("vector")
                                    .unwrap()
                                    .as_array()
                                    .unwrap()
                                    .to_owned()
                                    .iter()
                                    .map(|v| v.as_f64().unwrap())
                                    .collect::<Vec<f64>>(),
                            })
                            .collect();

                        let content = Content::json(SearchPointsResponse {
                            result: scored_points,
                            time: json.get("time").unwrap().as_f64().unwrap(),
                        })?;

                        Ok(CallToolResult::success(vec![content]))
                    }
                    None => {
                        let error_message =
                            "Failed to search points. The given key 'result' does not exist.";

                        error!("{}", error_message);

                        Err(McpError::new(
                            ErrorCode::INTERNAL_ERROR,
                            error_message,
                            None,
                        ))
                    }
                },
                Err(e) => {
                    let error_message = format!("Failed to search points: {e}");

                    error!("{}", error_message);

                    Err(McpError::new(
                        ErrorCode::INTERNAL_ERROR,
                        error_message,
                        None,
                    ))
                }
            },
            false => {
                let error_message = format!("Failed to search points: {}", response.status());

                error!("{}", error_message);

                Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ))
            }
        }
    }
}
#[tool_handler]
impl ServerHandler for QdrantServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            instructions: Some(
                "A MCP server that performs vector search in the Qdrant database".into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: std::env!("CARGO_PKG_NAME").to_string(),
                version: std::env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "search",
                Some(
                    "This prompt is for the `search` tool, which takes a vector and returns a list of points",
                ),
                Some(vec![PromptArgument {
                    name: "vector".to_string(),
                    description: Some("A vector to search for in the Qdrant database".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, .. }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "search" => {
                let prompt = "Perform vector search with the input vector. Return a tool call that invokes the vector search tool.\n\nThe input vector is: [0.0,0.0,0.0,0.0]";

                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt.to_string()),
                    }],
                })
            }
            _ => {
                let error_message = format!("prompt not found: {name}");
                error!("{}", error_message);
                Err(McpError::invalid_params(error_message, None))
            }
        }
    }
}
