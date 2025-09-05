use crate::CONNECTION_CONFIG;
use cardea_elastic_mcp_common::*;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use tracing::error;

#[derive(Debug, Clone)]
pub struct ElasticSearchServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ElasticSearchServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List all available Elasticsearch indices")]
    async fn list_indices(&self) -> Result<CallToolResult, McpError> {
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
        let url = format!("{base_url}/_cat/indices?v=true&s=index&format=json");

        // get api key
        let api_key = conn_config.api_key;

        let client = reqwest::Client::new();
        let result = match api_key {
            Some(api_key) => {
                client
                    .get(&url)
                    .header("Authorization", api_key)
                    .send()
                    .await
            }
            None => client.get(&url).send().await,
        };

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                let error_message = format!("Failed to list indices: {e}");

                error!(error_message);

                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        match response.status().is_success() {
            true => match response.json::<Value>().await {
                Ok(json) => {
                    let indices: Vec<IndexInfo> = serde_json::from_value(json).map_err(|e| {
                        let error_message = format!("Failed to parse indices: {e}");

                        error!(error_message);

                        McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
                    })?;

                    let content = Content::json(ListIndicesResponse { indices })?;

                    Ok(CallToolResult::success(vec![content]))
                }
                Err(e) => {
                    let error_message = format!("Failed to parse indices: {e}");

                    error!(error_message);

                    Err(McpError::new(
                        ErrorCode::INTERNAL_ERROR,
                        error_message,
                        None,
                    ))
                }
            },
            false => {
                let error_message = format!("Failed to list indices: {}", response.status());

                error!(error_message);

                Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ))
            }
        }
    }

    #[tool(
        description = "Get the cluster's index aliases, including filter and routing information. Note that this tool does not return data stream aliases."
    )]
    async fn list_aliases(&self) -> Result<CallToolResult, McpError> {
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
        let url = format!("{base_url}/_cat/aliases?format=json&v=true");

        // get api key
        let api_key = conn_config.api_key;

        let client = reqwest::Client::new();
        let result = match api_key {
            Some(api_key) => {
                client
                    .get(&url)
                    .header("Authorization", api_key)
                    .send()
                    .await
            }
            None => client.get(&url).send().await,
        };

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                let error_message = format!("Failed to list aliases: {e}");

                error!(error_message);

                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        match response.status().is_success() {
            true => match response.json::<Value>().await {
                Ok(json) => {
                    let aliases: Vec<AliasInfo> = serde_json::from_value(json).map_err(|e| {
                        let error_message = format!("Failed to parse aliases: {e}");

                        error!(error_message);

                        McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
                    })?;

                    let content = Content::json(ListAliasesResponse { aliases })?;

                    Ok(CallToolResult::success(vec![content]))
                }
                Err(e) => {
                    let error_message = format!("Failed to parse aliases: {e}");

                    error!(error_message);

                    Err(McpError::new(
                        ErrorCode::INTERNAL_ERROR,
                        error_message,
                        None,
                    ))
                }
            },
            false => {
                let error_message = format!("Failed to list aliases: {}", response.status());

                error!(error_message);

                Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ))
            }
        }
    }

    #[tool(description = "Perform a keyword search")]
    async fn search(
        &self,
        Parameters(SearchRequest { query }): Parameters<SearchRequest>,
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

        let index = conn_config.index;
        let fields = conn_config.fields;
        let size = conn_config.size;

        tracing::info!("index: {}", index);
        tracing::info!("fields: {:?}", fields);
        tracing::info!("size: {}", size);

        // build url
        let base_url = conn_config.base_url.trim_end_matches('/');
        let url = format!("{base_url}/{index}/_search");

        // get api key
        let api_key = conn_config.api_key;

        let body = json!({
            "query": {
                "multi_match": {
                    "query": query,
                    "fields": fields
                },
            },
            "size": size
        });

        let client = reqwest::Client::new();
        let result = match api_key {
            Some(api_key) => {
                client
                    .post(&url)
                    .header("Authorization", api_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
            }
            None => {
                client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
            }
        };

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                let error_message = format!("Failed to search: {e}");

                error!(error_message);

                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        match response.status().is_success() {
            true => match response.json::<Value>().await {
                Ok(json) => {
                    tracing::debug!(
                        "search result json:\n{}",
                        serde_json::to_string_pretty(&json).unwrap()
                    );

                    let search_response: SearchResponse =
                        serde_json::from_value(json).map_err(|e| {
                            let error_message = format!("Failed to parse search result: {e}");

                            error!(error_message);

                            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
                        })?;

                    let content = Content::json(search_response)?;

                    Ok(CallToolResult::success(vec![content]))
                }
                Err(e) => {
                    let error_message = format!("Failed to parse search result: {e}");

                    error!(error_message);

                    Err(McpError::new(
                        ErrorCode::INTERNAL_ERROR,
                        error_message,
                        None,
                    ))
                }
            },
            false => {
                let error_message = format!("Failed to search: {}", response.status());

                error!(error_message);

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
impl ServerHandler for ElasticSearchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            instructions: Some("A ElasticSearch MCP server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: std::env!("CARGO_PKG_NAME").to_string(),
                version: std::env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}
