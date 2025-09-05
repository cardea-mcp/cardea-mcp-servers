use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListIndicesResponse {
    pub indices: Vec<IndexInfo>,
}
impl From<CallToolResult> for ListIndicesResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content.is_empty() {
            false => result.content[0].as_text().unwrap().text.to_string(),
            true => String::new(),
        };

        serde_json::from_str::<ListIndicesResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IndexInfo {
    /// current health status
    pub health: String,
    /// open/close status
    pub status: String,
    /// index name
    pub index: String,
    /// uuid
    pub uuid: String,
    /// number of primary shards
    pub pri: String,
    /// number of replica shards
    pub rep: String,
    /// available docs
    #[serde(rename = "docs.count")]
    pub docs_count: String,
    /// deleted docs
    #[serde(rename = "docs.deleted")]
    pub docs_deleted: String,
    /// store size
    #[serde(rename = "store.size")]
    pub store_size: String,
    /// primary shard size
    #[serde(rename = "pri.store.size")]
    pub pri_store_size: String,
    /// dataset size
    #[serde(rename = "dataset.size")]
    pub dataset_size: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListAliasesResponse {
    pub aliases: Vec<AliasInfo>,
}
impl From<CallToolResult> for ListAliasesResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content.is_empty() {
            false => result.content[0].as_text().unwrap().text.to_string(),
            true => String::new(),
        };

        serde_json::from_str::<ListAliasesResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AliasInfo {
    /// alias name
    pub alias: String,
    /// index name
    pub index: String,
    /// filter
    pub filter: String,
    /// index routing
    #[serde(rename = "routing.index")]
    pub routing_index: String,
    /// search routing
    #[serde(rename = "routing.search")]
    pub routing_search: String,
    /// write index
    #[serde(rename = "is_write_index")]
    pub is_write_index: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    /// user query
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchResponse {
    /// The number of milliseconds it took Elasticsearch to run the request.
    pub took: u64,
    /// If `true`, the request timed out before completion; returned results may be partial or empty.
    pub timed_out: bool,
    #[serde(rename = "_shards")]
    pub shards: Shards,
    pub hits: Hits,
}
impl From<CallToolResult> for SearchResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content.is_empty() {
            false => result.content[0].as_text().unwrap().text.to_string(),
            true => String::new(),
        };

        serde_json::from_str::<SearchResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Hits {
    pub hits: Vec<Hit>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Hit {
    #[serde(rename = "_index")]
    pub index: String,
    #[serde(rename = "_score")]
    pub score: f64,
    #[serde(rename = "_source")]
    pub source: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Shards {
    pub total: u64,
    pub successful: u64,
    pub skipped: u64,
    pub failed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failures: Option<Vec<Failure>>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Failure {
    pub index: String,
    pub node: String,
    pub shard: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Reason {
    /// The type of error
    #[serde(rename = "type")]
    pub ty: String,
    /// A human-readable explanation of the error, in English.
    pub reason: String,
}
