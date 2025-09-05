use clap::{Parser, ValueEnum};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::{SseClientTransport, StreamableHttpClientTransport},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SOCKET_ADDR: &str = "127.0.0.1:8001";

#[derive(Debug, Clone, ValueEnum)]
enum TransportType {
    Sse,
    StreamHttp,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Cardea Calculator MCP client")]
struct Args {
    /// Transport type to use
    #[arg(short, long, value_enum, default_value = "stream-http")]
    transport: TransportType,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();

    let cli = Args::parse();

    // create a mcp client
    match cli.transport {
        TransportType::Sse => {
            let url = format!("http://{SOCKET_ADDR}/sse");
            tracing::info!(
                "Connecting to Cardea Calculator MCP server via sse: {}",
                url
            );

            let transport = SseClientTransport::start(url).await?;
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "test sse client".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            let mcp_client = client_info.serve(transport).await.inspect_err(|e| {
                tracing::error!("client error: {:?}", e);
            })?;

            // Initialize
            let server_info = mcp_client.peer_info();
            tracing::info!("Connected to server: {server_info:#?}");

            // List available tools
            let tools = mcp_client.peer().list_tools(Default::default()).await?;
            tracing::info!("Available tools: {}", serde_json::to_string_pretty(&tools)?);

            let request_param = CallToolRequestParam {
                name: "sum".into(),
                arguments: Some(serde_json::Map::from_iter([
                    ("a".to_string(), serde_json::Value::Number(1.into())),
                    ("b".to_string(), serde_json::Value::Number(2.into())),
                ])),
            };

            // Call the sum tool
            let sum_result = mcp_client.peer().call_tool(request_param).await?;

            tracing::info!("Sum result: {}", serde_json::to_string_pretty(&sum_result)?);
        }
        TransportType::StreamHttp => {
            let url = format!("http://{SOCKET_ADDR}/mcp");
            tracing::info!(
                "Connecting to Cardea Calculator MCP server via stream-http: {}",
                url
            );

            let transport = StreamableHttpClientTransport::from_uri(url);
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "test stream-http client".to_string(),
                    version: "0.0.1".to_string(),
                },
            };
            let mcp_client = client_info.serve(transport).await.inspect_err(|e| {
                tracing::error!("client error: {:?}", e);
            })?;

            // Initialize
            let server_info = mcp_client.peer_info();
            tracing::info!("Connected to server: {server_info:#?}");

            // List tools
            let tools = mcp_client.list_tools(Default::default()).await?;
            tracing::info!("Available tools: {tools:#?}");

            let request_param = CallToolRequestParam {
                name: "sum".into(),
                arguments: Some(serde_json::Map::from_iter([
                    ("a".to_string(), serde_json::Value::Number(1.into())),
                    ("b".to_string(), serde_json::Value::Number(2.into())),
                ])),
            };

            // Call the sum tool
            let sum_result = mcp_client.peer().call_tool(request_param).await?;

            tracing::info!("Sum result: {}", serde_json::to_string_pretty(&sum_result)?);

            mcp_client.cancel().await?;
        }
    };

    Ok(())
}
