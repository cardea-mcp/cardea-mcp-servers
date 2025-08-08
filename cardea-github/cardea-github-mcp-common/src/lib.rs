use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetStarCountRequest {
    #[schemars(description = "The owner of the Github repository")]
    pub owner: String,
    #[schemars(description = "The name of the Github repository")]
    pub repo: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetStarCountResponse {
    #[schemars(description = "The star count of the Github repository")]
    pub count: u32,
}
impl From<CallToolResult> for GetStarCountResponse {
    fn from(result: CallToolResult) -> Self {
        let content = match result.content {
            Some(contents) if !contents.is_empty() => {
                contents[0].as_text().unwrap().text.to_string()
            }
            _ => String::new(),
        };

        serde_json::from_str::<GetStarCountResponse>(&content).unwrap()
    }
}
