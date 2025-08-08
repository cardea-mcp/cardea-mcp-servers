use rmcp::{model::CallToolResult, schemars};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateCollectionRequest {
    #[schemars(description = "The name of the collection to create")]
    pub name: String,
    #[schemars(description = "The size of the vectors in the collection")]
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateCollectionResponse {
    #[schemars(description = "Whether the operation made changes")]
    pub result: bool,
    #[schemars(description = "The time it took to create the collection")]
    pub time: f64,
}
impl From<CallToolResult> for CreateCollectionResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content {
            Some(contents) if !contents.is_empty() => {
                contents[0].as_text().unwrap().text.to_string()
            }
            _ => String::new(),
        };

        serde_json::from_str::<CreateCollectionResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCollectionsResponse {
    #[schemars(description = "The list of collection names")]
    pub collections: Vec<String>,
    #[schemars(description = "The time it took to list the collections")]
    pub time: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CollectionExistsRequest {
    #[schemars(description = "The name of the collection to check")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CollectionExistsResponse {
    #[schemars(description = "Whether the collection exists")]
    pub result: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteCollectionRequest {
    #[schemars(description = "The name of the collection to delete")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteCollectionResponse {
    #[schemars(description = "Whether the collection was deleted")]
    pub result: bool,
    #[schemars(description = "The time it took to delete the collection")]
    pub time: f64,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Point {
    #[schemars(description = "The id of the point")]
    pub id: u64,
    #[schemars(description = "The payload of the point")]
    pub payload: Map<String, Value>,
    #[schemars(description = "The vector of the point")]
    pub vector: Vec<f32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpsertPointsRequest {
    #[schemars(description = "The name of the collection to upsert points into")]
    pub name: String,
    #[schemars(description = "The points to upsert")]
    pub points: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpsertPointsResponse {
    #[schemars(
        description = "The status of the upsert operation. Allowed values: 'acknowledged', 'completed'"
    )]
    pub status: String,
    #[schemars(description = "The time it took to upsert the points")]
    pub time: f64,
}
impl From<CallToolResult> for UpsertPointsResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content {
            Some(contents) if !contents.is_empty() => {
                contents[0].as_text().unwrap().text.to_string()
            }
            _ => String::new(),
        };

        serde_json::from_str::<UpsertPointsResponse>(&content).unwrap()
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchPointsRequest {
    #[schemars(description = "The vector to search for")]
    pub vector: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchPointsResponse {
    #[schemars(description = "The results of the search")]
    pub result: Vec<ScoredPoint>,
    #[schemars(description = "The time it took to search the points")]
    pub time: f64,
}
impl From<CallToolResult> for SearchPointsResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content {
            Some(contents) if !contents.is_empty() => {
                contents[0].as_text().unwrap().text.to_string()
            }
            _ => String::new(),
        };

        serde_json::from_str::<SearchPointsResponse>(&content).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScoredPoint {
    #[schemars(description = "The score of the point")]
    pub score: f64,
    #[schemars(description = "The payload of the point")]
    pub payload: HashMap<String, Value>,
    #[schemars(description = "The vector of the point")]
    pub vector: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub base_url: String,
    pub api_key: Option<String>,
}
