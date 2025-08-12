use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::*},
    model::*,
    schemars, tool, tool_handler, tool_router,
};

#[derive(Debug, Clone)]
pub struct WeatherServer {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl WeatherServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get the weather for a given city")]
    async fn get_current_weather(
        &self,
        Parameters(GetWeatherRequest { location, unit }): Parameters<GetWeatherRequest>,
    ) -> Result<CallToolResult, McpError> {
        let api_key = match std::env::var("OPENWEATHERMAP_API_KEY") {
            Ok(api_key) => api_key,
            Err(_) => {
                let err_message = "No API key provided. Please set the `OPENWEATHERMAP_API_KEY` environment variable.";
                tracing::error!("{}", err_message);
                return Err(McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    err_message.to_string(),
                    None,
                ));
            }
        };

        let unit = unit.unwrap_or(TemperatureUnit::Celsius);
        let openweathermap_unit = unit.to_openweathermap_unit();

        // * get geographic coordinates of the city
        tracing::info!("getting geocode for {}", location);

        let geocode_url = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={location}&appid={api_key}&limit=1&units={openweathermap_unit}"
        );

        // send the request to get the geocode
        let response = reqwest::get(geocode_url).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to get geocode: {e}"),
                None,
            )
        })?;

        let geocode_data = response.json::<serde_json::Value>().await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to parse geocode response: {e}"),
                None,
            )
        })?;

        let lat = geocode_data[0]["lat"].as_f64().unwrap();
        let lon = geocode_data[0]["lon"].as_f64().unwrap();

        // * get weather data
        tracing::info!("getting weather for {} at {} {}", location, lat, lon);
        let weather_url = format!(
            "http://api.openweathermap.org/data/2.5/weather?lat={lat}&lon={lon}&appid={api_key}&units={openweathermap_unit}"
        );

        // send the request to get the weather
        let response = reqwest::get(weather_url).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to get weather: {e}"),
                None,
            )
        })?;

        let weather_data = response.json::<serde_json::Value>().await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to parse weather response: {e}"),
                None,
            )
        })?;

        let temperature = weather_data["main"]["temp"].as_f64().unwrap();
        tracing::info!("temperature: {}", temperature);
        let description = weather_data["weather"][0]["description"].as_str().unwrap();
        tracing::info!("description: {}", description);

        let content = Content::json(GetWeatherResponse { temperature, unit })?;

        let res = CallToolResult::success(vec![content]);

        Ok(res)
    }
}

#[tool_handler]
impl ServerHandler for WeatherServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            instructions: Some("A MCP server that can get the weather for a given city".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: std::env!("CARGO_PKG_NAME").to_string(),
                version: std::env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetWeatherRequest {
    #[schemars(
        description = "The city to get the weather for, e.g., 'Beijing', 'New York', 'Tokyo'"
    )]
    pub location: String,
    #[schemars(description = "The unit to use for the temperature, e.g., 'celsius', 'fahrenheit'")]
    pub unit: Option<TemperatureUnit>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Clone)]
pub enum TemperatureUnit {
    #[serde(rename = "celsius")]
    Celsius,
    #[serde(rename = "fahrenheit")]
    Fahrenheit,
}
impl TemperatureUnit {
    pub fn to_openweathermap_unit(&self) -> String {
        match self {
            TemperatureUnit::Celsius => "metric".to_string(),
            TemperatureUnit::Fahrenheit => "imperial".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetWeatherResponse {
    #[schemars(description = "The temperature in celsius or fahrenheit")]
    pub temperature: f64,
    #[schemars(description = "The unit of the temperature")]
    pub unit: TemperatureUnit,
    // #[schemars(description = "the description of the weather")]
    // pub description: String,
}
