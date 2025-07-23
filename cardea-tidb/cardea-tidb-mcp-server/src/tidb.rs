use crate::TIDB_ACCESS_CONFIG;
use cardea_tidb_mcp_common::*;
use mysql::prelude::*;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::*},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use std::result::Result;
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
pub struct TidbServer {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl TidbServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Perform keyword search in TiDB")]
    async fn search(
        &self,
        Parameters(TidbSearchRequest { query }): Parameters<TidbSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let config = match TIDB_ACCESS_CONFIG.get() {
            Some(config) => config.read().await,
            None => {
                let error_message = "TIDB_ACCESS_CONFIG is not set";
                error!(error_message);
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };

        // get connection
        info!("Getting connection...");
        let mut conn = config.pool.get_conn().map_err(|e| {
            let error_message = format!("Failed to get connection: {e}");

            error!(error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        // test connection
        info!("Testing connection...");
        let version: String = match conn.query_first("SELECT VERSION()").map_err(|e| {
            let error_message = format!("Failed to query version: {e}");

            error!(error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })? {
            Some(version) => version,
            None => {
                let error_message = "Failed to query version";

                error!(error_message);

                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    error_message,
                    None,
                ));
            }
        };
        info!("Connected to TiDB Cloud! Version: {}", version);

        // check if table exists
        info!("Checking if table exists...");
        let check_table_sql = format!(
            "SELECT COUNT(*) FROM information_schema.tables
                WHERE table_schema = '{}' AND table_name = '{}'",
            config.database, config.table_name
        );
        info!("Executing check table SQL: {}", check_table_sql);
        let table_exists: i32 = conn
            .query_first(&check_table_sql)
            .map_err(|e| {
                let error_message = format!("Failed to check table: {e}");

                error!(error_message);

                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?
            .unwrap_or(0);

        if table_exists == 0 {
            let error_message = format!(
                "Not found table `{}` in database `{}`",
                config.table_name, config.database
            );

            error!(error_message);

            return Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                error_message,
                None,
            ));
        }

        // execute full-text search
        info!("\nExecuting full-text search for '{}'...", query);

        // ! Do not remove this block
        {
            // let search_sql = format!(
            //     r"SELECT * FROM {}
            //         WHERE fts_match_word(?, content)
            //         ORDER BY fts_match_word(?, content)
            //         DESC LIMIT ?",
            //     config.table_name
            // );

            // let hits: Vec<TidbSearchHit> = conn
            //     .exec(&search_sql, (query.clone(), query, config.limit))
            //     .map_err(|e| {
            //         let error_message = format!("Failed to execute search: {e}");

            //         error!(error_message);

            //         McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            //     })?;
        }

        let safe_query = query.replace("'", "''"); // ! This is a workaround for the issue that the query contains single quotes.

        let search_sql = format!(
            r"SELECT * FROM {}
                    WHERE fts_match_word('{}', content)
                    ORDER BY fts_match_word('{}', content)
                    DESC LIMIT {}",
            config.table_name, safe_query, safe_query, config.limit
        );

        let hits: Vec<TidbSearchHit> = conn.query(&search_sql).map_err(|e| {
            let error_message = format!("Failed to execute search: {e}");

            error!(error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        info!("\nSearch results:");
        info!("Found {} matching records:", hits.len());
        for hit in hits.iter() {
            info!("\nID: {}", &hit.id);
            info!("Title: {}", &hit.title);
            info!("Content: {}", &hit.content);
        }

        let content = Content::json(TidbSearchResponse { hits })?;

        info!("Search results fetched from TiDB");

        Ok(CallToolResult::success(vec![content]))
    }
}

#[tool_handler]
impl ServerHandler for TidbServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            instructions: Some("Cardea TiDB MCP server".into()),
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
                    description: Some("A user query to search for in TiDB".to_string()),
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
                error!("{}", error_message);
                Err(McpError::invalid_params(error_message, None))
            }
        }
    }
}
