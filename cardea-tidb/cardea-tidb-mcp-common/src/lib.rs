use endpoints::rag::keyword_search::SearchHit;
use mysql_common::prelude::FromRow;
use rmcp::{model::CallToolResult, schemars};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TidbSearchRequest {
    #[schemars(description = "The query to search for")]
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TidbSearchResponse {
    #[schemars(description = "The hits of the tidb server")]
    pub hits: Vec<TidbSearchHit>,
}
impl From<CallToolResult> for TidbSearchResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content {
            Some(contents) if !contents.is_empty() => {
                contents[0].as_text().unwrap().text.to_string()
            }
            _ => String::new(),
        };

        serde_json::from_str::<TidbSearchResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, FromRow)]
pub struct TidbSearchHit {
    #[schemars(description = "The id of the tidb server")]
    pub id: i32,
    #[schemars(description = "The title of the tidb server")]
    pub title: String,
    #[schemars(description = "The content of the tidb server")]
    pub content: String,
}
impl From<TidbSearchHit> for SearchHit {
    fn from(value: TidbSearchHit) -> Self {
        SearchHit {
            title: value.title,
            content: value.content,
            score: 0.0,
        }
    }
}
