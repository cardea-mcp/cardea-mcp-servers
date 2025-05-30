use crate::CONNECTION_CONFIG;
use endpoints::rag::keyword_search::{IndexRequest, IndexResponse, QueryRequest, QueryResponse};
use gaia_kwsearch_mcp_common::{
    CreateIndexRequest, CreateIndexResponse, SearchDocumentsRequest, SearchDocumentsResponse,
};
use rmcp::{
    Error as McpError, ServerHandler,
    model::{CallToolResult, Content, ErrorCode, ServerCapabilities, ServerInfo},
    tool,
};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct KeywordSearchServer;
#[tool(tool_box)]
impl ServerHandler for KeywordSearchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A MCP server that can access the KeywordSearch database".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
#[tool(tool_box)]
impl KeywordSearchServer {
    #[tool(description = "Create an index in the KeywordSearch database")]
    async fn create_index(
        &self,
        #[tool(aggr)] CreateIndexRequest { name, documents }: CreateIndexRequest,
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
        let url = format!("{}/v1/index/create", base_url);

        let index_request = IndexRequest {
            index: name,
            documents: documents.into_iter().map(|d| d.into()).collect::<Vec<_>>(),
        };

        let response = reqwest::Client::new()
            .post(&url)
            .json(&index_request)
            .send()
            .await
            .map_err(|e| {
                let error_message = format!("Failed to create index: {}", e);

                error!("{}", error_message);

                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?;

        let index_response = response.json::<IndexResponse>().await.map_err(|e| {
            let error_message = format!("Failed to parse index response: {}", e);

            error!("{}", error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        let content = Content::json(CreateIndexResponse::from(index_response))?;

        info!("Index created in KeywordSearch database");

        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "Search for documents in the KeywordSearch database")]
    async fn search_documents(
        &self,
        #[tool(aggr)] SearchDocumentsRequest {
            index_name,
            query,
            limit,
        }: SearchDocumentsRequest,
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
        let url = format!("{}/v1/search", base_url);

        let query_request = QueryRequest {
            query,
            top_k: limit,
            index: index_name,
        };

        let response = reqwest::Client::new()
            .post(&url)
            .json(&query_request)
            .send()
            .await
            .map_err(|e| {
                let error_message = format!("Failed to search documents: {}", e);

                error!("{}", error_message);

                McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
            })?;

        let query_response = response.json::<QueryResponse>().await.map_err(|e| {
            let error_message = format!("Failed to parse query response: {}", e);

            error!("{}", error_message);

            McpError::new(ErrorCode::INTERNAL_ERROR, error_message, None)
        })?;

        let content = Content::json(SearchDocumentsResponse::from(query_response))?;

        info!("Documents searched in KeywordSearch database");

        Ok(CallToolResult::success(vec![content]))
    }
}
