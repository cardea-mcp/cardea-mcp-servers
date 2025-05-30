use clap::{Parser, ValueEnum};
use gaia_kwsearch_common::{CreateIndexResponse, KwDocumentInput, SearchDocumentsResponse};
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    service::ServiceExt,
    transport::{SseClientTransport, StreamableHttpClientTransport, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SOCKET_ADDR: &str = "127.0.0.1:8005";
const KWSEARCH_INDEX_NAME: &str = "mcp-test";

#[derive(Debug, Clone, ValueEnum)]
enum TransportType {
    Stdio,
    Sse,
    StreamHttp,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Gaia Keyword Search MCP client")]
struct Args {
    /// Transport type to use (tcp or stdio)
    #[arg(short, long, value_enum, default_value = "tcp")]
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

    match cli.transport {
        TransportType::Sse => {
            let url = format!("http://{SOCKET_ADDR}/sse");
            tracing::info!(
                "Connecting to Gaia KeywordSearch MCP server via sse: {}",
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

            // * create index
            let documents = vec![
                KwDocumentInput {
                    content: String::from(
                        "Gaianet is revolutionizing the AI landscape with a distributed AI infrastructure that seeks to decentralize the dominance of major players such as OpenAI, Google, and Anthropic. By leveraging a network of edge-computing nodes owned by individuals around the world, Gaianet enables hosting of both open-source and finely-tuned models. This infrastructure is designed to cater to diverse AI demands, offering a scalable alternative to traditional centralized servers.",
                    ),
                    title: Some("section 1".to_string()),
                },
                KwDocumentInput {
                    content: String::from(
                        "The inception of Gaianet is driven by the necessity to address key issues in the current AI industry: censorship and bias in AI outputs, lack of privacy for user data, and the high costs associated with accessing and developing on centralized AI models. These challenges have restricted the dissemination of unbiased information, compromised data security, and erected barriers to innovation and broader application of AI technologies.",
                    ),
                    title: Some("section 2".to_string()),
                },
            ];
            let request_param = CallToolRequestParam {
                name: "create_index".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    (
                        "documents".to_string(),
                        serde_json::Value::Array(
                            documents
                                .into_iter()
                                .map(|d| serde_json::to_value(d).unwrap())
                                .collect(),
                        ),
                    ),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "create index response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );

            let index_response = CreateIndexResponse::from(tool_result);
            tracing::info!("create index response:\n{:?}", &index_response);

            // * search documents
            let request_param = CallToolRequestParam {
                name: "search_documents".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "index_name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    ("query".to_string(), serde_json::Value::from("Gaianet")),
                    ("limit".to_string(), serde_json::Value::from(2)),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "search documents response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );
            let search_response = SearchDocumentsResponse::from(tool_result);
            tracing::info!("search documents response:\n{:?}", &search_response);

            mcp_client.cancel().await?;
        }
        TransportType::Stdio => {
            tracing::info!("Connecting to MCP server via stdio");

            let mut cmd = Command::new("./target/release/gaia-kwsearch-mcp-server-stdio");
            cmd.arg("--base-url").arg("http://127.0.0.1:12306");

            let transport = TokioChildProcess::new(cmd)?;

            let mcp_client = ().serve(transport).await?;
            tracing::info!("Connected to server");

            // Initialize
            let server_info = mcp_client.peer_info();
            tracing::info!("Connected to server: {server_info:#?}");

            // list tools
            let tools = mcp_client.peer().list_tools(Default::default()).await?;
            tracing::info!("{}", serde_json::to_string_pretty(&tools)?);

            // * create index

            let documents = vec![
                KwDocumentInput {
                    content: String::from(
                        "Gaianet is revolutionizing the AI landscape with a distributed AI infrastructure that seeks to decentralize the dominance of major players such as OpenAI, Google, and Anthropic. By leveraging a network of edge-computing nodes owned by individuals around the world, Gaianet enables hosting of both open-source and finely-tuned models. This infrastructure is designed to cater to diverse AI demands, offering a scalable alternative to traditional centralized servers.",
                    ),
                    title: Some("section 1".to_string()),
                },
                KwDocumentInput {
                    content: String::from(
                        "The inception of Gaianet is driven by the necessity to address key issues in the current AI industry: censorship and bias in AI outputs, lack of privacy for user data, and the high costs associated with accessing and developing on centralized AI models. These challenges have restricted the dissemination of unbiased information, compromised data security, and erected barriers to innovation and broader application of AI technologies.",
                    ),
                    title: Some("section 2".to_string()),
                },
            ];
            let request_param = CallToolRequestParam {
                name: "create_index".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    (
                        "documents".to_string(),
                        serde_json::Value::Array(
                            documents
                                .into_iter()
                                .map(|d| serde_json::to_value(d).unwrap())
                                .collect(),
                        ),
                    ),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "create index response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );

            let index_response = CreateIndexResponse::from(tool_result);
            tracing::info!("create index response:\n{:?}", &index_response);

            // * search documents
            let request_param = CallToolRequestParam {
                name: "search_documents".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "index_name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    ("query".to_string(), serde_json::Value::from("Gaianet")),
                    ("limit".to_string(), serde_json::Value::from(2)),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "search documents response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );
            let search_response = SearchDocumentsResponse::from(tool_result);
            tracing::info!("search documents response:\n{:?}", &search_response);

            mcp_client.cancel().await?;
        }
        TransportType::StreamHttp => {
            let url = format!("http://{SOCKET_ADDR}/mcp");
            tracing::info!(
                "Connecting to Gaia KeywordSearch MCP server via stream-http: {}",
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

            // list tools
            let tools = mcp_client.peer().list_tools(Default::default()).await?;
            tracing::info!("{}", serde_json::to_string_pretty(&tools)?);

            // * create index

            let documents = vec![
                KwDocumentInput {
                    content: String::from(
                        "Gaianet is revolutionizing the AI landscape with a distributed AI infrastructure that seeks to decentralize the dominance of major players such as OpenAI, Google, and Anthropic. By leveraging a network of edge-computing nodes owned by individuals around the world, Gaianet enables hosting of both open-source and finely-tuned models. This infrastructure is designed to cater to diverse AI demands, offering a scalable alternative to traditional centralized servers.",
                    ),
                    title: Some("section 1".to_string()),
                },
                KwDocumentInput {
                    content: String::from(
                        "The inception of Gaianet is driven by the necessity to address key issues in the current AI industry: censorship and bias in AI outputs, lack of privacy for user data, and the high costs associated with accessing and developing on centralized AI models. These challenges have restricted the dissemination of unbiased information, compromised data security, and erected barriers to innovation and broader application of AI technologies.",
                    ),
                    title: Some("section 2".to_string()),
                },
            ];
            let request_param = CallToolRequestParam {
                name: "create_index".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    (
                        "documents".to_string(),
                        serde_json::Value::Array(
                            documents
                                .into_iter()
                                .map(|d| serde_json::to_value(d).unwrap())
                                .collect(),
                        ),
                    ),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "create index response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );

            let index_response = CreateIndexResponse::from(tool_result);
            tracing::info!("create index response:\n{:?}", &index_response);

            // * search documents
            let request_param = CallToolRequestParam {
                name: "search_documents".into(),
                arguments: Some(serde_json::Map::from_iter([
                    (
                        "index_name".to_string(),
                        serde_json::Value::from(KWSEARCH_INDEX_NAME),
                    ),
                    ("query".to_string(), serde_json::Value::from("Gaianet")),
                    ("limit".to_string(), serde_json::Value::from(2)),
                ])),
            };

            let tool_result = mcp_client.peer().call_tool(request_param).await?;
            tracing::info!(
                "search documents response:\n{}",
                serde_json::to_string_pretty(&tool_result)?
            );
            let search_response = SearchDocumentsResponse::from(tool_result);
            tracing::info!("search documents response:\n{:?}", &search_response);

            mcp_client.cancel().await?;
        }
    }

    Ok(())
}
