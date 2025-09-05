use crate::CONNECTION_CONFIG;
use cardea_kwsearch_mcp_common::{
    CreateIndexRequest, CreateIndexResponse, SearchDocumentsRequest, SearchDocumentsResponse,
};
use endpoints::keyword_search::{IndexRequest, IndexResponse, QueryRequest, QueryResponse};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use tracing::{error, info};

const PROMPT_SEARCH_TOOL: &str = r#"
You are a multilingual AI assistant. Your task is to (1) extract the most relevant and concise keywords or key phrases from the given user query, and (2) return a tool call that invokes the `search` tool with the extracted keywords.

### Requirements for keyword extraction
Follow these requirements strictly:
- Detect the language of the query automatically.
- Return 3 to 7 keywords or keyphrases that best represent the query's core intent.
- Keep the extracted keywords in the **original language** (do not translate).
- Include **multi-word expressions** if they convey meaningful concepts.
- **Avoid all types of stop words, question words, filler words, or overly generic terms**, such as:
  - English: what, how, why, is, the, of, and, etc.
  - Chinese: 什么、怎么、如何、是、的、了、吗、啊 等。
- Do **not** include punctuation or meaningless words.
- Only return the final keywords, separated by a **single space**.

Examples:
- Input: "What is the impact of artificial intelligence on education?"
  Output: artificial intelligence education impact
- Input: "什么是人工智能对教育的影响？"
  Output: 人工智能 教育 影响

### Requirements for tool call
- Return a tool call that invokes the `search` tool with the extracted keywords.
- For each function call, return a json object with function name and arguments within <tool_call></tool_call> XML tags:
  <tool_call>
  {"name": <function-name>, "arguments": <args-json-object>}
  </tool_call>

Examples:
- Input: "What is the impact of artificial intelligence on education?"
  Output:
    <tool_call>
    {"name": "search", "arguments": {"query": "artificial intelligence education impact"}}
    </tool_call>
"#;

#[derive(Debug, Clone)]
pub struct KeywordSearchServer {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl KeywordSearchServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create an index in the KeywordSearch database")]
    async fn create_index(
        &self,
        Parameters(CreateIndexRequest { index, documents }): Parameters<CreateIndexRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("Creating index in KeywordSearch database");

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
        let url = format!("{base_url}/v1/index/create");

        let index_request = IndexRequest {
            index: Some(index),
            documents: documents.into_iter().map(|d| d.into()).collect::<Vec<_>>(),
        };

        let response = reqwest::Client::new()
            .post(&url)
            .json(&index_request)
            .send()
            .await
            .map_err(|e| {
                let error_message = format!("Failed to create index: {e}");

                error!("{}", error_message);

                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?;

        let index_response = response.json::<IndexResponse>().await.map_err(|e| {
            let error_message = format!("Failed to parse index response: {e}");

            error!("{}", error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        let content = Content::json(CreateIndexResponse::from(index_response))?;

        info!("Index created in KeywordSearch database");

        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "Perform a keyword search")]
    async fn search(
        &self,
        Parameters(SearchDocumentsRequest { query }): Parameters<SearchDocumentsRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("Searching for documents in KeywordSearch database");

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
        let url = format!("{base_url}/v1/search");

        let query_request = QueryRequest {
            query,
            top_k: conn_config.limit,
            index: conn_config.index,
        };

        let response = reqwest::Client::new()
            .post(&url)
            .json(&query_request)
            .send()
            .await
            .map_err(|e| {
                let error_message = format!("Failed to search documents: {e}");

                error!("{}", error_message);

                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?;

        let query_response = response.json::<QueryResponse>().await.map_err(|e| {
            let error_message = format!("Failed to parse query response: {e}");

            error!("{}", error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        let content = Content::json(SearchDocumentsResponse::from(query_response))?;

        info!("Documents searched in KeywordSearch database");

        Ok(CallToolResult::success(vec![content]))
    }
}
#[tool_handler]
impl ServerHandler for KeywordSearchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            instructions: Some("A MCP server that can access the KeywordSearch database".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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
                    "This prompt is for the `search` tool, which takes a query and returns a list of hits",
                ),
                Some(vec![PromptArgument {
                    name: "query".to_string(),
                    description: Some("A user query to search for".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "search" => {
                let query = arguments
                    .and_then(|json| json.get("query")?.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| {
                        McpError::invalid_params("No query provided to `search` tool", None)
                    })?;

                let prompt = format!("{PROMPT_SEARCH_TOOL}\n\n### Input Query\n{query:#?}");

                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => {
                let error_message = format!("prompt not found: {name}");
                error!("{error_message}");
                Err(McpError::invalid_params(error_message, None))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub base_url: String,
    #[allow(dead_code)]
    pub api_key: Option<String>,
    pub index: String,
    pub limit: usize,
}
